use super::*;
use gw2_net::ClientCodec;
use std::net::Shutdown;

#[test]
fn auth_and_mission_loopback_capture_plaintext_with_exact_limit() {
    for (mode, payload) in [
        (Preamble::Auth, vec![0x9a, 1]),
        (Preamble::Mission, vec![0x9a, 1]),
        (Preamble::Mission, vec![0xa5; CAPTURE_LIMIT + 23]),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gw2-observe-test-{}-{epoch}.bin",
            std::process::id()
        ));
        // Public toy group for this local test; no stored identities are loaded.
        let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
        let client_exponent = BigUint::from(5u32);
        let shared = key.public().modpow(&client_exponent, &key.p);
        let server_path = path.clone();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            observe(
                stream,
                &key,
                &server_path,
                Some(&Bytes::from_static(&[0x19, 4])),
                None,
                mode,
            )
        });
        let mut client = TcpStream::connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        client
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut input = preamble::fixture(mode);
        // g^5 = 1024, four-byte little-endian public value.
        input.extend_from_slice(&[0, 6, 0, 4, 0, 0]);
        // Split within the opaque prefix, across the auth/mission boundary.
        for part in input.chunks(7) {
            client.write_all(part).unwrap();
        }
        let mut reply = [0u8; 22];
        client.read_exact(&mut reply).unwrap();
        assert_eq!(&reply[..2], &[1, 22]);
        let block = reply[2..].try_into().unwrap();
        let mut codec = ClientCodec::new(&session_secret(&shared, &block));
        let mut pending = BytesMut::new();
        let received = loop {
            let mut chunk = [0u8; 32];
            let n = client.read(&mut chunk).unwrap();
            assert_ne!(n, 0);
            pending.extend_from_slice(&chunk[..n]);
            if let Some(frame) = codec.decode(&mut pending).unwrap() {
                break frame;
            }
        };
        assert_eq!(&received[..], &[0x19, 4]);
        let mut wire = BytesMut::new();
        codec
            .encode(Bytes::copy_from_slice(&payload), &mut wire)
            .unwrap();
        if payload.len() > CAPTURE_LIMIT {
            // Force a non-aligned first read, so a full final chunk would
            // exceed the cap even though the cap is divisible by 8192.
            client.write_all(&wire[..1]).unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while fs::metadata(&path).unwrap().len() == 0 {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(1));
            }
            client.write_all(&wire[1..]).unwrap();
        } else {
            client.write_all(&wire).unwrap();
        }
        client.shutdown(Shutdown::Both).unwrap();
        server.join().unwrap().unwrap();
        assert_eq!(
            fs::read(&path).unwrap(),
            payload[..payload.len().min(CAPTURE_LIMIT)]
        );
        let prefix_path = path.with_extension("preamble.bin");
        assert_eq!(fs::read(&prefix_path).unwrap(), preamble::fixture(mode));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert_eq!(
                fs::metadata(&prefix_path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        // Reopening either path must fail rather than replacing evidence.
        assert_eq!(
            private_file(&path).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(
            private_file(&prefix_path).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        fs::remove_file(path).unwrap();
        fs::remove_file(prefix_path).unwrap();
    }
}
