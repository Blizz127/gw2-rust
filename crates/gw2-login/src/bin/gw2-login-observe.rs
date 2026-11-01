//! `gw2-login-observe [--listen ADDR]... [--log DIR] [--greet-hex HEX] [--answer-starttls BODY] [--tls-key PEM --tls-cert DER]`
//!
//! Capture listener for stage 3. It accepts TCP connections and, by default,
//! **never sends a byte**. `--greet-hex` is an opt-in experiment: it sends those
//! bytes right after accept, to test whether the peer expects the server to
//! speak first. It logs what arrives:
//! - STS/1.0 messages, decoded (start line, headers, body)
//! - everything after the first non-STS byte (e.g. TLS-style records after
//!   `/Auth/StartTls`, or a binary game handshake) as a raw hex dump
//!
//! With an explicit TLS identity, StartTls defaults to `<Reply/>`, completes TLS,
//! and captures decrypted STS messages to private files. Use synthetic accounts only.
//! `--answer-gethostname HOST` additionally answers GetHostname with that hostname
//! and a Local marker, allowing an offline login experiment to advance.
//! `--reply-xml /PATH=FILE` adds an explicit synthetic XML reply fixture inside TLS.
//! These fixtures are protocol experiments, not account authentication.
//!
//! Defaults: listens on 127.0.0.1:6112 and 127.0.0.1:6600 (portal/game ports in the
//! public notes, unverified). Logs to `./capture/`. Use it inside the
//! `--network none` sandbox. For build 209026, bind 127.1.0.1 and use the sandbox
//! DNS sink; its resolver cannot use an answer in 127.0.0.0/24.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use gw2_login::{sts, tls};

static CONN_ID: AtomicU64 = AtomicU64::new(1);

fn main() {
    let mut listen: Vec<SocketAddr> = Vec::new();
    let mut log_dir = PathBuf::from("capture");
    let mut greet: Vec<u8> = Vec::new();
    let mut starttls: Option<String> = None;
    let mut key = None;
    let mut cert = None;
    let mut replies: Vec<(String, Vec<u8>)> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match (a.as_str(), args.next()) {
            ("--listen", Some(v)) => match v.parse() {
                Ok(addr) => listen.push(addr),
                Err(e) => die(&format!("bad --listen {v}: {e}")),
            },
            ("--log", Some(v)) => log_dir = PathBuf::from(v),
            ("--greet-hex", Some(v)) => {
                greet = (0..v.len())
                    .step_by(2)
                    .map(|i| v.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok()))
                    .collect::<Option<Vec<u8>>>()
                    .unwrap_or_else(|| die("bad --greet-hex"))
            }
            // Opt-in: answer `/Auth/StartTls` with `STS/1.0 200 OK`, `s:<n>R` and this body
            // (`-` = empty). Without a TLS identity, capture the following handshake raw.
            ("--answer-starttls", Some(v)) => starttls = Some(if v == "-" { String::new() } else { v }),
            ("--tls-key", Some(v)) => key = Some(PathBuf::from(v)),
            ("--tls-cert", Some(v)) => cert = Some(PathBuf::from(v)),
            ("--answer-gethostname", Some(v)) => {
                if v.is_empty() || v.len() > 253 || !v.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-') {
                    die("--answer-gethostname requires an ASCII DNS hostname");
                }
                // Build 209026's XML accessor rejects bodies without a final LF.
                replies.push(("/Auth/GetHostname".into(), format!("<Reply><Hostname>{v}</Hostname><Local/></Reply>\n").into_bytes()));
            }
            ("--reply-xml", Some(v)) => {
                let (path, file) = v.split_once('=').unwrap_or_else(|| die("--reply-xml requires /PATH=FILE"));
                if !path.starts_with('/') || !path.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'/' || c == b'_') {
                    die("--reply-xml requires an ASCII STS request path");
                }
                let input = std::fs::File::open(file).unwrap_or_else(|e| die(&format!("open reply fixture: {e}")));
                let mut body = Vec::new();
                input.take((sts::MAX_BODY + 1) as u64).read_to_end(&mut body).unwrap_or_else(|e| die(&format!("read reply fixture: {e}")));
                if body.len() > sts::MAX_BODY || !body.ends_with(b"\n") || std::str::from_utf8(&body).is_err() {
                    die("XML reply fixture must be UTF-8, at most MAX_BODY, and end with LF");
                }
                replies.push((path.into(), body));
            }
            _ => die("usage: gw2-login-observe [--listen ADDR]... [--log DIR] [--greet-hex HEX] [--answer-starttls BODY] [--tls-key PEM --tls-cert DER] [--answer-gethostname HOST] [--reply-xml /PATH=FILE]..."),
        }
    }
    let tls_config = match (key, cert) {
        (None, None) => None,
        (Some(k), Some(c)) => {
            starttls.get_or_insert_with(|| "<Reply/>".into());
            Some(Arc::new(tls::ServerConfig::load(&k, &c).unwrap_or_else(|e| die(&e.to_string()))))
        }
        _ => die("--tls-key and --tls-cert must be supplied together"),
    };
    if !replies.is_empty() && tls_config.is_none() {
        die("TLS reply fixtures require --tls-key and --tls-cert");
    }
    for (i, (path, _)) in replies.iter().enumerate() {
        if replies[..i].iter().any(|(other, _)| other == path) {
            die(&format!("duplicate reply fixture for {path}"));
        }
    }
    let replies = Arc::new(replies);
    if listen.is_empty() {
        listen = vec!["127.0.0.1:6112".parse().unwrap(), "127.0.0.1:6600".parse().unwrap()];
    }
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        die(&format!("cannot create {}: {e}", log_dir.display()));
    }

    let mut handles = Vec::new();
    for addr in listen {
        if !addr.ip().is_loopback() {
            eprintln!("warning: {addr} is not loopback; make sure this runs in the offline sandbox");
        }
        let listener = TcpListener::bind(addr).unwrap_or_else(|e| die(&format!("bind {addr}: {e}")));
        let addr = listener.local_addr().unwrap_or(addr);
        println!("observing {addr} -> {}", log_dir.display());
        let dir = log_dir.clone();
        let greet = greet.clone();
        let starttls = starttls.clone();
        let tls_config = tls_config.clone();
        let replies = replies.clone();
        handles.push(std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(s) => {
                        let dir = dir.clone();
                        let greet = greet.clone();
                        let starttls = starttls.clone();
                        let tls_config = tls_config.clone();
                        let replies = replies.clone();
                        std::thread::spawn(move || observe(s, addr, dir, greet, starttls, tls_config, replies));
                    }
                    Err(e) => eprintln!("{addr}: accept failed: {e}"),
                }
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
}

fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(2);
}

fn unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis())
}

fn observe(mut stream: TcpStream, local: SocketAddr, dir: PathBuf, greet: Vec<u8>, starttls: Option<String>, tls_config: Option<Arc<tls::ServerConfig>>, replies: Arc<Vec<(String, Vec<u8>)>>) {
    let id = CONN_ID.fetch_add(1, Ordering::Relaxed);
    let peer = stream.peer_addr().map_or("?".into(), |p| p.to_string());
    let path = dir.join(format!("{}-conn{id}-port{}.log", unix_ms(), local.port()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let Ok(mut log) = options.open(&path) else {
        eprintln!("cannot create {}", path.display());
        return;
    };
    println!("conn {id}: {peer} -> {local}, logging to {}", path.display());
    let _ = writeln!(log, "# conn {id} peer={peer} local={local} start_unix_ms={}", unix_ms());

    let start = Instant::now();
    if !greet.is_empty() {
        let r = stream.write_all(&greet);
        let _ = writeln!(log, "## sent greeting {} bytes: {:02x?} -> {r:?}", greet.len(), greet);
    }
    let mut buf = Vec::new();
    let mut raw_mode = false;
    let mut total = 0usize;
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let n = match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                let _ = writeln!(log, "# read error: {e}");
                break;
            }
        };
        let t = start.elapsed().as_millis();
        if raw_mode {
            let _ = writeln!(log, "## +{t}ms raw {n} bytes @{total}");
            let _ = log.write_all(hexdump(&chunk[..n]).as_bytes());
            total += n;
            continue;
        }
        buf.extend_from_slice(&chunk[..n]);
        loop {
            match sts::decode(&buf) {
                Ok(Some((msg, used))) => {
                    let _ = writeln!(log, "## +{t}ms STS {used} bytes @{total}");
                    let _ = log.write_all(&buf[..used - msg.body.len()]);
                    let _ = log.write_all(&msg.body);
                    let _ = writeln!(log);
                    buf.drain(..used);
                    total += used;
                    if let (Some(body), Some("/Auth/StartTls")) = (&starttls, msg.path()) {
                        let reply = sts::Message::reply(200, "OK", msg.seq().unwrap_or(0), body.as_bytes()).encode();
                        let r = stream.write_all(&reply);
                        let _ = writeln!(log, "## +{t}ms sent StartTls reply ({} bytes) -> {r:?}", reply.len());
                        let _ = log.write_all(&reply);
                        if r.is_err() { return; }
                        if let Some(cfg) = &tls_config {
                            observe_tls(stream, cfg, buf, &mut log, start, &replies);
                            println!("conn {id}: TLS capture closed");
                            return;
                        }
                        let _ = writeln!(log, "\n## switching to raw after StartTls");
                        let _ = log.write_all(hexdump(&buf).as_bytes());
                        total += buf.len();
                        buf.clear();
                        raw_mode = true;
                        break;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    let _ = writeln!(log, "## +{t}ms switching to raw: {e}");
                    let _ = log.write_all(hexdump(&buf).as_bytes());
                    total += buf.len();
                    buf.clear();
                    raw_mode = true;
                    break;
                }
            }
        }
    }
    if !buf.is_empty() {
        let _ = writeln!(log, "## trailing {} undecoded bytes", buf.len());
        let _ = log.write_all(hexdump(&buf).as_bytes());
    }
    let _ = writeln!(log, "# closed after {}ms, {total} bytes logged", start.elapsed().as_millis());
    println!("conn {id}: closed");
}

fn observe_tls(stream: TcpStream, cfg: &tls::ServerConfig, pre: Vec<u8>, log: &mut std::fs::File, start: Instant, replies: &[(String, Vec<u8>)]) {
    // Preserve bytes read beyond the plaintext STS message (possibly a ClientHello).
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
    let mut secure = match tls::accept(stream, cfg, pre) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(log, "\n# TLS handshake failed: {e}");
            return;
        }
    };
    let _ = writeln!(log, "\n# TLS established: TLS1.2 AES128-SHA");
    let mut pending = Vec::new();
    let mut raw = false;
    loop {
        let bytes = match secure.read_app() {
            Ok(b) if b.is_empty() => break,
            Ok(b) => b,
            Err(e) => {
                let _ = writeln!(log, "# TLS read ended: {e}");
                break;
            }
        };
        if raw {
            let _ = writeln!(log, "## decrypted raw {} bytes\n{}", bytes.len(), hexdump(&bytes));
            continue;
        }
        pending.extend(bytes);
        loop {
            match sts::decode(&pending) {
                Ok(Some((msg, used))) => {
                    let _ = writeln!(log, "## +{}ms decrypted STS {used} bytes", start.elapsed().as_millis());
                    let _ = log.write_all(&pending[..used]);
                    let _ = writeln!(log);
                    println!("TLS STS: {} ({} body bytes)", msg.path().unwrap_or("response"), msg.body.len());
                    pending.drain(..used);
                    let fixture = replies.iter().find(|(path, _)| Some(path.as_str()) == msg.path());
                    if let (Some((path, body)), Some(seq)) = (fixture, msg.seq()) {
                        let reply = sts::Message::reply(200, "OK", seq, body.clone()).encode();
                        if let Err(e) = secure.write_app(&reply) {
                            let _ = writeln!(log, "# TLS {path} reply failed: {e}");
                            return;
                        }
                        let _ = writeln!(log, "## sent TLS {path} reply ({} bytes)", reply.len());
                        let _ = log.write_all(&reply);
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    let _ = writeln!(log, "## decrypted non-STS {} bytes\n{}", pending.len(), hexdump(&pending));
                    pending.clear();
                    raw = true;
                    break;
                }
            }
        }
    }
    if !pending.is_empty() {
        let _ = writeln!(log, "## trailing decrypted {} bytes\n{}", pending.len(), hexdump(&pending));
    }
    let _ = secure.close_notify();
    let _ = writeln!(log, "# TLS capture closed after {}ms", start.elapsed().as_millis());
}

fn hexdump(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (i, row) in bytes.chunks(16).enumerate() {
        out.push_str(&format!("{:08x}  ", i * 16));
        for j in 0..16 {
            match row.get(j) {
                Some(b) => out.push_str(&format!("{b:02x} ")),
                None => out.push_str("   "),
            }
        }
        out.push(' ');
        out.extend(row.iter().map(|&b| if (0x20..0x7f).contains(&b) { b as char } else { '.' }));
        out.push('\n');
    }
    out
}
