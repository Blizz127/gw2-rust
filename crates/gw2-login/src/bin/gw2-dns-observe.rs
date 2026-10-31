//! `gw2-dns-observe [--listen 127.0.0.1:53] [--answer 127.0.0.1] [--log FILE]`
//!
//! Offline-sandbox DNS responder. It logs `unix_ms<TAB>qtype<TAB>name` for every
//! query and answers A queries with `--answer`. Run it ONLY inside the
//! `--network none` container: it is a sinkhole, not a resolver.

use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

use gw2_login::dns::{build_response, parse_query};

fn main() {
    let mut listen: SocketAddr = "127.0.0.1:53".parse().unwrap();
    let mut answer = Ipv4Addr::LOCALHOST;
    let mut log_path = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let v = args.next();
        match (a.as_str(), v) {
            ("--listen", Some(v)) => listen = v.parse().expect("bad --listen"),
            ("--answer", Some(v)) => answer = v.parse().expect("bad --answer"),
            ("--log", Some(v)) => log_path = Some(v),
            _ => {
                eprintln!("usage: gw2-dns-observe [--listen ADDR] [--answer IPV4] [--log FILE]");
                std::process::exit(2);
            }
        }
    }
    let sock = UdpSocket::bind(listen).unwrap_or_else(|e| {
        eprintln!("bind {listen}: {e}");
        std::process::exit(1)
    });
    let mut log: Box<dyn Write> = match log_path {
        Some(p) => Box::new(std::fs::OpenOptions::new().create(true).append(true).open(p).expect("open log")),
        None => Box::new(std::io::stdout()),
    };
    println!("dns sinkhole on {listen}, answering {answer}");
    let mut buf = [0u8; 1500];
    loop {
        let Ok((n, peer)) = sock.recv_from(&mut buf) else { continue };
        let pkt = &buf[..n];
        let Some(q) = parse_query(pkt) else { continue };
        let ms = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
        let _ = writeln!(log, "{ms}\t{}\t{}", q.qtype, q.name);
        let _ = log.flush();
        let _ = sock.send_to(&build_response(pkt, &q, answer), peer);
    }
}
