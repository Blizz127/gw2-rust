//! Offline build-209026 game-handshake observer. See docs/re/209026-game-handshake.md.
//! Uses an explicitly supplied local DH identity; captures decrypted bytes privately.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bytes::{Bytes, BytesMut};
use gw2_net::{
    ServerCodec,
    dh::{ByteOrder, ParamBlob, ServerKey, session_secret},
};
use num_bigint::BigUint;
use tokio_util::codec::{Decoder, Encoder};

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn private_file(path: &Path) -> io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    opts.open(path)
}

fn read_exact_file(path: &Path, size: usize) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)?
        .take((size + 1) as u64)
        .read_to_end(&mut data)?;
    if data.len() != size {
        return Err(invalid("wrong identity file length"));
    }
    Ok(data)
}

fn observe(
    mut stream: TcpStream,
    key: &ServerKey,
    log: &Path,
    reply: Option<&Bytes>,
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    // Run 47 has a 16-byte preamble, with build at +4. Other fields remain opaque.
    let mut preamble = [0u8; 16];
    stream.read_exact(&mut preamble)?;
    if u32::from_le_bytes(preamble[4..8].try_into().unwrap()) != 209026 {
        return Err(invalid("expected frozen build 209026 preamble"));
    }
    let mut header = [0u8; 2];
    stream.read_exact(&mut header)?;
    let len = usize::from(header[1]);
    if header[0] != 0 || !(6..=66).contains(&len) || (len - 2) % 4 != 0 {
        return Err(invalid("invalid client key record"));
    }
    let mut public = vec![0u8; len - 2];
    stream.read_exact(&mut public)?;
    let shared = key
        .shared(&BigUint::from_bytes_le(&public))
        .ok_or_else(|| invalid("degenerate client public value"))?;
    let mut block = [0u8; 20];
    getrandom::fill(&mut block).map_err(io::Error::other)?;
    let mut codec = ServerCodec::new(&session_secret(&shared, &block));
    let mut capture = private_file(log)?;
    stream.write_all(&[1, 22])?;
    stream.write_all(&block)?;
    eprintln!("Game key reply sent; client public length {}", public.len());
    if let Some(reply) = reply {
        let mut wire = BytesMut::new();
        codec.encode(reply.clone(), &mut wire)?;
        stream.write_all(&wire)?;
        eprintln!("Explicit game reply sent: {} plaintext bytes", reply.len());
    }
    let mut chunk = [0u8; 8192];
    let mut total = 0;
    while total < 1024 * 1024 {
        let n = match stream.read(&mut chunk) {
            Ok(n) => n,
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(e) => return Err(e),
        };
        if n == 0 {
            break;
        }
        let mut pending = BytesMut::from(&chunk[..n]);
        if let Some(decoded) = codec.decode(&mut pending)? {
            capture.write_all(&decoded)?;
            capture.flush()?;
            total += decoded.len();
            eprintln!("Decrypted game bytes: {} (total {total})", decoded.len());
        }
    }
    eprintln!("Game capture complete: {total} bytes");
    Ok(())
}

fn main() -> io::Result<()> {
    let mut listen: SocketAddr = "127.1.0.2:6112".parse().unwrap();
    let (mut params, mut private, mut logs) = (None, None, None);
    let mut reply = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--listen" => {
                listen = args
                    .next()
                    .ok_or_else(|| invalid("missing listen address"))?
                    .parse()
                    .map_err(|_| invalid("invalid listen address"))?
            }
            "--params" => params = args.next().map(PathBuf::from),
            "--private" => private = args.next().map(PathBuf::from),
            "--log" => logs = args.next().map(PathBuf::from),
            "--reply-frame" => {
                let path = args.next().ok_or_else(|| invalid("missing reply file"))?;
                let mut bytes = Vec::new();
                File::open(path)?.take(65536).read_to_end(&mut bytes)?;
                if bytes.is_empty() || bytes.len() > 65535 {
                    return Err(invalid("reply must contain 1..65535 plaintext bytes"));
                }
                reply = Some(Bytes::from(bytes));
            }
            _ => {
                return Err(invalid(
                    "usage: gw2-game-observe --params BLOB136 --private EXPONENT64 --log DIR [--listen LOOPBACK:PORT] [--reply-frame FILE]",
                ));
            }
        }
    }
    if !listen.ip().is_loopback() {
        return Err(invalid("offline observer requires loopback"));
    }
    let params = read_exact_file(&params.ok_or_else(|| invalid("missing --params"))?, 136)?;
    let params = ParamBlob::parse(&params, ByteOrder::Little)
        .ok_or_else(|| invalid("invalid parameters"))?;
    let private = read_exact_file(&private.ok_or_else(|| invalid("missing --private"))?, 64)?;
    let exponent = BigUint::from_bytes_le(&private);
    let one = BigUint::from(1u32);
    if params.version != 1
        || params.p.bits() != 512
        || params.g != BigUint::from(4u32)
        || (&params.p & &one) != one
        || exponent <= one
        || exponent >= &params.p - &one
    {
        return Err(invalid("invalid local DH identity"));
    }
    let key = ServerKey::from_private(params.p, params.g, exponent);
    if key.public() != params.server_public {
        return Err(invalid("public/private identity mismatch"));
    }
    let key = Arc::new(key);
    let logs = logs.ok_or_else(|| invalid("missing --log"))?;
    fs::create_dir_all(&logs)?;
    let listener = TcpListener::bind(listen)?;
    eprintln!("Game observer listening on {}", listener.local_addr()?);
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    for (index, stream) in listener.incoming().enumerate() {
        let stream = stream?;
        let key = Arc::clone(&key);
        let reply = reply.clone();
        let log = logs.join(format!("{epoch}-{index}.decrypted.bin"));
        std::thread::spawn(move || {
            if let Err(e) = observe(stream, &key, &log, reply.as_ref()) {
                eprintln!("Game observation ended: {e}");
            }
        });
    }
    Ok(())
}
