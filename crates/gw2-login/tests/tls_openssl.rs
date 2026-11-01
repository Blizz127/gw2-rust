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
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use gw2_login::tls::{ServerConfig, accept};

fn have_openssl() -> bool {
    Command::new("openssl").arg("version").output().is_ok_and(|o| o.status.success())
}

fn gen_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/gen-test-cert.sh")
}

/// Runs `tools/gen-test-cert.sh` into `dir` and loads the result.
fn gen_and_load(dir: &Path) -> ServerConfig {
    let _ = std::fs::remove_dir_all(dir);
    let out = Command::new("bash").arg(gen_script()).arg(dir).output().unwrap();
    assert!(out.status.success(), "gen-test-cert.sh: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("CN=cligate.101.ncplatform.net,L=Bellevue,ST=Washington,C=US"), "{stdout}");
    ServerConfig::load(&dir.join("cligate-101.key.pem"), &dir.join("cligate-101.cert.der")).expect("load")
}

#[test]
fn gen_test_cert_output_loads_and_bad_shapes_are_rejected() {
    if !have_openssl() {
        eprintln!("skipped: openssl not found");
        return;
    }
    let base = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tls-gen-{}", std::process::id()));
    let (a, b) = (base.join("a"), base.join("b"));
    let cfg = gen_and_load(&a);
    assert_eq!(cfg.modulus().len(), 128);
    assert!(cfg.modulus()[0] & 0x80 != 0);
    // The script refuses to overwrite an existing key without --force.
    let again = Command::new("bash").arg(gen_script()).arg(&a).output().unwrap();
    assert!(!again.status.success());
    // A certificate for a different key is rejected.
    gen_and_load(&b);
    let mixed = ServerConfig::load(&a.join("cligate-101.key.pem"), &b.join("cligate-101.cert.der"));
    assert!(mixed.is_err());
    // A 2048-bit key doesn't fit the pinned 128-byte slot.
    let big = base.join("big.key.pem");
    let st = Command::new("openssl")
        .args(["genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:2048", "-out"])
        .arg(&big)
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(st.success());
    let err = ServerConfig::load(&big, &a.join("cligate-101.cert.der")).err().expect("2048-bit key rejected");
    assert!(err.to_string().contains("RSA-1024"), "{err}");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn openssl_s_client_completes_handshake_and_exchanges_data() {
    if !have_openssl() {
        eprintln!("skipped: openssl not found");
        return;
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("tls-interop-{}", std::process::id()));
    let cfg = gen_and_load(&dir);
    assert_eq!(cfg.modulus().len(), 128);

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
