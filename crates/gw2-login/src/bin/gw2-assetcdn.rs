//! `gw2-assetcdn [--listen ADDR]... [--latest "B E ES M MS"] [--files DIR] [--log FILE]`
//!
//! Offline asset-CDN stand-in for the sandbox (see `gw2_login::assetcdn`). It answers
//! `/latest64/<program>` and `/latest/<program>` with the configured five numbers, and
//! serves `/program/<program>/<x>/<y>/<fileId>` from `DIR/<fileId>` (raw bytes; the
//! `/compressed` variant gets 404 so the client falls back to raw), and answers every
//! other path with 404. Every request line is logged so the next requests
//! the client makes (manifests, files) show up.
//!
//! Default listen address: 127.1.0.1:80. Use 127.1.0.x, never 127.0.0.x
//! (docs/re/209026-connect-path.md).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use gw2_login::assetcdn::{Latest, Route, decode_request, response, route};

type Log = Arc<Mutex<Box<dyn Write + Send>>>;

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)
}

fn usage() -> ! {
    eprintln!("usage: gw2-assetcdn [--listen ADDR]... [--latest \"B E ES M MS\"] [--log FILE]");
    std::process::exit(2)
}

fn main() {
    let mut listen: Vec<SocketAddr> = Vec::new();
    let mut latest = Latest { build: 209026, ..Latest::default() };
    let mut log_path = None;
    let mut files: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let v = args.next().unwrap_or_else(|| usage());
        match a.as_str() {
            "--listen" => listen.push(v.parse().unwrap_or_else(|_| usage())),
            // `_` or `,` may stand in for spaces (the sandbox SERVER string splits on spaces).
            "--latest" => latest = Latest::parse(&v.replace(['_', ','], " ")).unwrap_or_else(|| usage()),
            "--log" => log_path = Some(v),
            "--files" => files = Some(v),
            _ => usage(),
        }
    }
    if listen.is_empty() {
        listen.push("127.1.0.1:80".parse().unwrap());
    }
    let log: Log = Arc::new(Mutex::new(match log_path {
        Some(p) => Box::new(
            std::fs::OpenOptions::new().create(true).append(true).open(&p).unwrap_or_else(|e| {
                eprintln!("open {p}: {e}");
                std::process::exit(1)
            }),
        ),
        None => Box::new(std::io::stdout()),
    }));
    let mut handles = Vec::new();
    for addr in listen {
        let l = TcpListener::bind(addr).unwrap_or_else(|e| {
            eprintln!("bind {addr}: {e}");
            std::process::exit(1)
        });
        println!("asset cdn on {addr}, latest = {}", latest.body().trim_end());
        let log = log.clone();
        let files = files.clone();
        handles.push(std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                let log = log.clone();
                let files = files.clone();
                std::thread::spawn(move || serve(s, latest, files.as_deref(), &log));
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
}

fn serve(mut s: TcpStream, latest: Latest, files: Option<&str>, log: &Log) {
    let peer = s.peer_addr().map(|a| a.to_string()).unwrap_or_default();
    let line = |msg: String| {
        if let Ok(mut w) = log.lock() {
            let _ = writeln!(w, "{}\t{peer}\t{msg}", now_ms());
            let _ = w.flush();
        }
    };
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match decode_request(&buf) {
            Err(e) => {
                line(format!("ERROR {e}"));
                return;
            }
            Ok(Some((req, used))) => {
                buf.drain(..used);
                let reply = match route(&req) {
                    Route::Latest { program, bits64 } => {
                        line(format!("{} {} -> latest(program {program}, 64bit {bits64}) {}", req.method, req.path, latest.body().trim_end()));
                        response(200, "OK", "text/plain", latest.body().as_bytes())
                    }
                    Route::File { file_id, compressed: false } if files.is_some() => {
                        let p = format!("{}/{file_id}", files.unwrap_or_default());
                        match std::fs::read(&p) {
                            Ok(b) => {
                                line(format!("{} {} -> file {file_id} ({} bytes)", req.method, req.path, b.len()));
                                response(200, "OK", "application/octet-stream", &b)
                            }
                            Err(_) => {
                                line(format!("{} {} -> 404 (no {p})", req.method, req.path));
                                response(404, "Not Found", "text/plain", b"")
                            }
                        }
                    }
                    Route::File { .. } | Route::Other => {
                        line(format!("{} {} -> 404", req.method, req.path));
                        response(404, "Not Found", "text/plain", b"")
                    }
                };
                if s.write_all(&reply).is_err() {
                    return;
                }
                continue;
            }
            Ok(None) => {}
        }
        match s.read(&mut chunk) {
            Ok(0) | Err(_) => {
                line("closed".into());
                return;
            }
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    }
}
