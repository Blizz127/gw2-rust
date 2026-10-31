//! `gw2-login-observe [--listen ADDR]... [--log DIR] [--greet-hex HEX]`
//!
//! Capture listener for stage 3. It accepts TCP connections and, by default,
//! **never sends a byte**. `--greet-hex` is an opt-in experiment: it sends those
//! bytes right after accept, to test whether the peer expects the server to
//! speak first. It logs what arrives:
//! - STS/1.0 messages, decoded (start line, headers, body)
//! - everything after the first non-STS byte (e.g. TLS-style records after
//!   `/Auth/StartTls`, or a binary game handshake) as a raw hex dump
//!
//! Defaults: listens on 127.0.0.1:6112 and 127.0.0.1:6600 (portal/game ports in the
//! public notes, unverified). Logs to `./capture/`. Use it inside the
//! `--network none` sandbox with the client pointed at 127.0.0.1.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use gw2_login::sts;

static CONN_ID: AtomicU64 = AtomicU64::new(1);

fn main() {
    let mut listen: Vec<SocketAddr> = Vec::new();
    let mut log_dir = PathBuf::from("capture");
    let mut greet: Vec<u8> = Vec::new();
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
            _ => die("usage: gw2-login-observe [--listen ADDR]... [--log DIR] [--greet-hex HEX]"),
        }
    }
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
        println!("observing {addr} -> {}", log_dir.display());
        let dir = log_dir.clone();
        let greet = greet.clone();
        handles.push(std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(s) => {
                        let dir = dir.clone();
                        let greet = greet.clone();
                        std::thread::spawn(move || observe(s, addr, dir, greet));
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

fn observe(mut stream: TcpStream, local: SocketAddr, dir: PathBuf, greet: Vec<u8>) {
    let id = CONN_ID.fetch_add(1, Ordering::Relaxed);
    let peer = stream.peer_addr().map_or("?".into(), |p| p.to_string());
    let path = dir.join(format!("{}-conn{id}-port{}.log", unix_ms(), local.port()));
    let Ok(mut log) = std::fs::File::create(&path) else {
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
