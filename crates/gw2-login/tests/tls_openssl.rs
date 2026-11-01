//! Interop check for `gw2_login::tls` against an independent TLS 1.2 client,
//! `openssl s_client`, restricted to `AES128-SHA` (TLS_RSA_WITH_AES_128_CBC_SHA).
//!
//! It covers RSA premaster decrypt, master secret, key block, both Finished messages and
//! application records in both directions. A throwaway 1024-bit key and self-signed
//! certificate are generated in a temp dir at test time (1024 bits matches the size of
//! the client's pinned cligate keys). Nothing is written to the repo. Skipped when
//! `openssl` isn't on PATH.

use std::io::Write;
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use gw2_login::tls::{ServerConfig, accept};
use rsa::RsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;

fn have_openssl() -> bool {
    Command::new("openssl").arg("version").output().is_ok_and(|o| o.status.success())
}

#[test]
fn openssl_s_client_completes_handshake_and_exchanges_data() {
    if !have_openssl() {
        eprintln!("skipped: openssl not found");
        return;
    }
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tls-interop-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (key_pem, cert_der) = (dir.join("key.pem"), dir.join("cert.der"));
    let st = Command::new("openssl")
        .args(["req", "-x509", "-newkey", "rsa:1024", "-nodes", "-days", "1"])
        .args(["-subj", "/CN=cligate.101.ncplatform.net", "-outform", "DER"])
        .arg("-keyout")
        .arg(&key_pem)
        .arg("-out")
        .arg(&cert_der)
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(st.success(), "openssl req failed");
    let cfg = ServerConfig {
        key: RsaPrivateKey::from_pkcs8_pem(&std::fs::read_to_string(&key_pem).unwrap()).unwrap(),
        cert_chain: vec![std::fs::read(&cert_der).unwrap()],
    };

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (sock, _) = listener.accept().unwrap();
        sock.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
        sock.set_write_timeout(Some(Duration::from_secs(20))).unwrap();
        let mut tls = accept(sock, &cfg, Vec::new()).expect("handshake");
        let mut got = Vec::new();
        while !got.ends_with(b"\n") {
            let chunk = tls.read_app().expect("read_app");
            assert!(!chunk.is_empty(), "client closed early");
            got.extend(chunk);
        }
        tls.write_app(b"pong from gw2-login\n").expect("write_app");
        tls.close_notify().expect("close_notify");
        (got, tls.offered_suites.clone())
    });

    let mut child = Command::new("openssl")
        .args(["s_client", "-connect", &format!("127.0.0.1:{port}"), "-tls1_2"])
        .args(["-cipher", "AES128-SHA:@SECLEVEL=0", "-legacy_server_connect", "-ign_eof"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"ping from s_client\n").unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("s_client timed out");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let (got, suites) = server.join().expect("server thread");
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(got, b"ping from s_client\n");
    assert!(suites.contains(&0x002f));
    assert!(stdout.contains("pong from gw2-login"), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(stdout.contains("Protocol: TLSv1.2"), "stdout:\n{stdout}");
    assert!(stdout.contains("Cipher: AES128-SHA") || stdout.contains("Cipher is AES128-SHA"), "stdout:\n{stdout}");
    for line in stdout.lines().filter(|l| l.contains("Protocol:") || l.contains("Cipher")) {
        eprintln!("s_client: {}", line.trim());
    }
}
