use super::*;
use gw2_net::ClientCodec;
use gw2_proto::Encode;
use std::net::Shutdown;

#[test]
fn auth_and_mission_loopback_capture_plaintext_with_exact_limit() {
    let mut record = vec![0; 16]; // Synthetic character GUID.
    record.push(0); // Opaque Dword.
    record.extend_from_slice(&[0; 16]); // Opaque Qwords.
    record.extend_from_slice(&[b'Z', 0, 0, 0]);
    record.extend_from_slice(&159u16.to_le_bytes());
    let mut summary = [0; 159];
    summary[0] = 11;
    summary[4] = 1;
    summary[5] = 1;
    record.extend_from_slice(&summary);
    let fixture = auth::ReplyFixture::new(
        Bytes::from_static(&[0x44; 2]),
        Some(Bytes::from(record.clone())),
    )
    .unwrap();
    let mut request = BytesMut::new();
    10u16.encode(&mut request);
    300u32.encode(&mut request);
    request.extend_from_slice(&[0; 36]); // Two GUIDs and two empty strings.
    let mut expected = BytesMut::new();
    8u16.encode(&mut expected);
    300u32.encode(&mut expected);
    expected.extend_from_slice(&[0x44; 2]);
    auth::tests::world_list(&mut expected);
    10u16.encode(&mut expected);
    300u32.encode(&mut expected);
    expected.extend_from_slice(&[0; 2]);
    17u16.encode(&mut expected);
    300u32.encode(&mut expected);
    expected.extend_from_slice(&record);
    7u16.encode(&mut expected);
    300u32.encode(&mut expected);
    expected.extend_from_slice(&[0; 6]);
    for (mode, payload, auth_fixture, expected_auth) in [
        (Preamble::Auth, vec![0x9a, 1], None, None),
        (Preamble::Mission, vec![0x9a, 1], None, None),
        (
            Preamble::Mission,
            vec![0xa5; CAPTURE_LIMIT + 23],
            None,
            None,
        ),
        (
            Preamble::Auth,
            request.to_vec(),
            Some(fixture),
            Some(expected.freeze()),
        ),
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
                Fixtures {
                    reply: Some(&Bytes::from_static(&[0x19, 4])),
                    auth_body: auth_fixture,
                    ..Fixtures::default()
                },
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
        if let Some(expected) = expected_auth {
            let received = loop {
                if let Some(frame) = codec.decode(&mut pending).unwrap() {
                    break frame;
                }
                let mut chunk = [0u8; 32];
                let n = client.read(&mut chunk).unwrap();
                assert_ne!(n, 0);
                pending.extend_from_slice(&chunk[..n]);
            };
            assert_eq!(received, expected);
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

#[test]
fn only_credential_safe_service_mode_binds_beyond_loopback() {
    let tailnet: SocketAddr = "100.74.44.12:6112".parse().unwrap();
    assert!(bind_allowed(&"127.1.0.3:6112".parse().unwrap(), false));
    assert!(!bind_allowed(&tailnet, false));
    assert!(bind_allowed(&tailnet, true));
    assert!(!bind_allowed(&"0.0.0.0:6112".parse().unwrap(), true));
    assert!(!bind_allowed(&"[::]:6112".parse().unwrap(), true));
}

#[test]
fn play_phase_extends_but_bounds_the_read_timeout() {
    assert_eq!(read_timeout(false), Duration::from_secs(30));
    assert_eq!(read_timeout(true), Duration::from_secs(600));
}

#[test]
fn mission_phase_loopback_sends_map_data_then_load_map_after_key_reply() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gw2-observe-mission-{}-{epoch}.bin",
        std::process::id()
    ));
    // Public toy group for this local test; no stored identities are loaded.
    let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
    let shared = key.public().modpow(&BigUint::from(5u32), &key.p);
    let server_path = path.clone();
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let fixtures = Fixtures {
            mission_map_key: Some(15),
            ..Fixtures::default()
        };
        observe(stream, &key, &server_path, fixtures, Preamble::Mission)
    });
    let mut client = TcpStream::connect(address).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut input = preamble::fixture(Preamble::Mission);
    input.extend_from_slice(&[0, 6, 0, 4, 0, 0]);
    client.write_all(&input).unwrap();
    let mut reply = [0u8; 22];
    client.read_exact(&mut reply).unwrap();
    assert_eq!(&reply[..2], &[1, 22]);
    let mut codec = ClientCodec::new(&session_secret(&shared, &reply[2..].try_into().unwrap()));
    let mut pending = BytesMut::new();
    let received = loop {
        if let Some(frame) = codec.decode(&mut pending).unwrap() {
            break frame;
        }
        let mut chunk = [0u8; 32];
        let n = client.read(&mut chunk).unwrap();
        assert_ne!(n, 0);
        pending.extend_from_slice(&chunk[..n]);
    };
    assert_eq!(received, mission::MissionPhase::new(15).unwrap().opening());
    let mut answer = BytesMut::new();
    423u16.encode(&mut answer);
    answer.extend_from_slice(&[0, 0]);
    420u16.encode(&mut answer);
    let mut wire = BytesMut::new();
    codec.encode(Bytes::copy_from_slice(&answer), &mut wire).unwrap();
    client.write_all(&wire).unwrap();
    client.shutdown(Shutdown::Both).unwrap();
    server.join().unwrap().unwrap();
    assert_eq!(fs::read(&path).unwrap(), answer);
    fs::remove_file(&path).unwrap();
    fs::remove_file(path.with_extension("preamble.bin")).unwrap();
}

#[test]
fn mission_map_key_is_bounded() {
    assert!(mission::MissionPhase::new(0x3fff_ffff).is_ok());
    assert!(mission::MissionPhase::new(0x4000_0000).is_err());
}

#[test]
fn mission_population_loopback_follows_client_420_and_412() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gw2-observe-population-{}-{epoch}.bin",
        std::process::id()
    ));
    let record = mission::tests::guardian_record(15, 1);
    let fixture = mission::PopulationFixture::new(&record, [0.0; 3]).unwrap();
    let expected_opening = mission::MissionPhase::new(15)
        .unwrap()
        .with_population(fixture.clone())
        .unwrap()
        .opening();
    // Public toy group for this local test; no stored identities are loaded.
    let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
    let shared = key.public().modpow(&BigUint::from(5u32), &key.p);
    let server_path = path.clone();
    let server = std::thread::spawn(move || {
        let fixtures = Fixtures {
            mission_map_key: Some(15),
            population: Some(fixture),
            ..Fixtures::default()
        };
        observe(stream_of(listener), &key, &server_path, fixtures, Preamble::Mission)
    });
    let mut client = TcpStream::connect(address).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut input = preamble::fixture(Preamble::Mission);
    input.extend_from_slice(&[0, 6, 0, 4, 0, 0]);
    client.write_all(&input).unwrap();
    let mut reply = [0u8; 22];
    client.read_exact(&mut reply).unwrap();
    let mut codec = ClientCodec::new(&session_secret(&shared, &reply[2..].try_into().unwrap()));
    let mut pending = BytesMut::new();
    let mut next = |client: &mut TcpStream, codec: &mut ClientCodec| loop {
        if let Some(frame) = codec.decode(&mut pending).unwrap() {
            break frame;
        }
        let mut chunk = [0u8; 256];
        let n = client.read(&mut chunk).unwrap();
        assert_ne!(n, 0);
        pending.extend_from_slice(&chunk[..n]);
    };
    assert_eq!(next(&mut client, &mut codec), expected_opening);
    let send = |client: &mut TcpStream, codec: &mut ClientCodec, id: u16| {
        let mut wire = BytesMut::new();
        codec
            .encode(Bytes::copy_from_slice(&id.to_le_bytes()), &mut wire)
            .unwrap();
        client.write_all(&wire).unwrap();
    };
    send(&mut client, &mut codec, 420);
    let population = next(&mut client, &mut codec);
    assert_eq!(&population[..2], &428u16.to_le_bytes());
    send(&mut client, &mut codec, 412);
    assert_eq!(&next(&mut client, &mut codec)[..], &[0x0c, 0x04, 0xaf, 0x01]);
    client.shutdown(Shutdown::Both).unwrap();
    server.join().unwrap().unwrap();
    fs::remove_file(&path).unwrap();
    fs::remove_file(path.with_extension("preamble.bin")).unwrap();
}

fn stream_of(listener: TcpListener) -> TcpStream {
    listener.accept().unwrap().0
}

#[test]
fn no_capture_mode_serves_without_writing_files() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "gw2-observe-nocapture-{}-{epoch}.bin",
        std::process::id()
    ));
    // Public toy group for this local test; no stored identities are loaded.
    let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
    let shared = key.public().modpow(&BigUint::from(5u32), &key.p);
    let server_path = path.clone();
    let server = std::thread::spawn(move || {
        let fixtures = Fixtures {
            mission_map_key: Some(15),
            no_capture: true,
            ..Fixtures::default()
        };
        observe(stream_of(listener), &key, &server_path, fixtures, Preamble::Mission)
    });
    let mut client = TcpStream::connect(address).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut input = preamble::fixture(Preamble::Mission);
    input.extend_from_slice(&[0, 6, 0, 4, 0, 0]);
    client.write_all(&input).unwrap();
    let mut reply = [0u8; 22];
    client.read_exact(&mut reply).unwrap();
    let mut codec = ClientCodec::new(&session_secret(&shared, &reply[2..].try_into().unwrap()));
    let mut pending = BytesMut::new();
    let opening = loop {
        if let Some(frame) = codec.decode(&mut pending).unwrap() {
            break frame;
        }
        let mut chunk = [0u8; 256];
        let n = client.read(&mut chunk).unwrap();
        assert_ne!(n, 0);
        pending.extend_from_slice(&chunk[..n]);
    };
    assert_eq!(opening, mission::MissionPhase::new(15).unwrap().opening());
    let mut wire = BytesMut::new();
    codec
        .encode(Bytes::copy_from_slice(&420u16.to_le_bytes()), &mut wire)
        .unwrap();
    client.write_all(&wire).unwrap();
    client.shutdown(Shutdown::Both).unwrap();
    server.join().unwrap().unwrap();
    assert!(!path.exists(), "no-capture mode created a capture file");
    assert!(!path.with_extension("preamble.bin").exists());
}

/// The deployed game-server configuration (Banshee: --no-capture
/// --mission-preamble --mission-map-key 15 --mission-population <record>
/// --mission-spawn 0,0,7.4) end to end over TCP: key exchange, then 1063/1060,
/// the population sequence after client 420 MapLoaded, then 1036 + 431 after
/// client 412 AgentsLoaded. 420 and 412 are sent as one plaintext, with the
/// encrypted bytes split across two writes at every offset; no file is written.
#[test]
fn deployed_mission_population_sequence_over_tcp_for_every_split() {
    let record = mission::tests::guardian_record(15, 1);
    let spawn = [0.0, 0.0, 7.4];
    let fixture = mission::PopulationFixture::new(&record, spawn).unwrap();
    let mut expect = mission::MissionPhase::new(15).unwrap().with_population(fixture.clone()).unwrap();
    let opening = expect.opening();
    let population = expect
        .respond(&mission::Seen::Map(gw2_proto::map_load::ClientMapEvent::MapLoaded))
        .unwrap()
        .unwrap();
    let begin = expect
        .respond(&mission::Seen::Map(gw2_proto::map_load::ClientMapEvent::AgentsLoaded))
        .unwrap()
        .unwrap();
    let mut plain = BytesMut::new();
    gw2_proto::map_load::encode_empty(gw2_proto::map_load::MAP_LOADED_420, &mut plain);
    gw2_proto::map_load::encode_empty(gw2_proto::map_load::AGENTS_LOADED_412, &mut plain);
    for cut in 0..=plain.len() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let path = std::env::temp_dir().join(format!("gw2-observe-deployed-{}-{cut}.bin", std::process::id()));
        // Public toy group for this local test; no stored identities are loaded.
        let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
        let shared = key.public().modpow(&BigUint::from(5u32), &key.p);
        let (server_path, server_fixture) = (path.clone(), fixture.clone());
        let server = std::thread::spawn(move || {
            let fixtures = Fixtures {
                mission_map_key: Some(15),
                population: Some(server_fixture),
                no_capture: true,
                ..Fixtures::default()
            };
            observe(stream_of(listener), &key, &server_path, fixtures, Preamble::Mission)
        });
        let mut client = TcpStream::connect(address).unwrap();
        client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut input = preamble::fixture(Preamble::Mission);
        input.extend_from_slice(&[0, 6, 0, 4, 0, 0]);
        client.write_all(&input).unwrap();
        let mut reply = [0u8; 22];
        client.read_exact(&mut reply).unwrap();
        let mut codec = ClientCodec::new(&session_secret(&shared, &reply[2..].try_into().unwrap()));
        let mut pending = BytesMut::new();
        let mut frames = Vec::new();
        let mut wire = BytesMut::new();
        let mut sent = false;
        while frames.len() < 3 {
            if let Some(frame) = codec.decode(&mut pending).unwrap() {
                frames.push(frame);
                if frames.len() == 1 && !sent {
                    codec.encode(Bytes::copy_from_slice(&plain), &mut wire).unwrap();
                    client.write_all(&wire[..cut.min(wire.len())]).unwrap();
                    client.flush().unwrap();
                    std::thread::sleep(Duration::from_millis(20));
                    client.write_all(&wire[cut.min(wire.len())..]).unwrap();
                    sent = true;
                }
                continue;
            }
            let mut chunk = [0u8; 4096];
            let n = client.read(&mut chunk).unwrap();
            assert_ne!(n, 0, "cut {cut}: server closed early after {} frames", frames.len());
            pending.extend_from_slice(&chunk[..n]);
        }
        assert_eq!(frames[0], opening, "cut {cut}: 1063 then 1060");
        assert_eq!(frames[1], population, "cut {cut}: population after 420");
        assert_eq!(frames[2], begin, "cut {cut}: 1036 + 431 after 412");
        client.shutdown(Shutdown::Both).unwrap();
        server.join().unwrap().unwrap();
        assert!(!path.exists(), "no-capture mode created a capture file");
        assert!(!path.with_extension("preamble.bin").exists());
    }
}

/// The client rejects a key reply split across reads (owner's device, error 58:11:5:535:
/// 2 B arrived, the remaining 20 B one Nagle round trip later). The whole key reply and the
/// first frame must come from one write, so the client's first read holds all of them.
#[test]
fn key_reply_and_first_frame_arrive_in_one_read() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let path = std::env::temp_dir().join(format!("gw2-observe-oneread-{}.bin", std::process::id()));
    // Public toy group for this local test; no stored identities are loaded.
    let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
    let server_path = path.clone();
    let server = std::thread::spawn(move || {
        let fixtures = Fixtures { mission_map_key: Some(15), no_capture: true, ..Fixtures::default() };
        observe(stream_of(listener), &key, &server_path, fixtures, Preamble::Mission)
    });
    let mut client = TcpStream::connect(address).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut input = preamble::fixture(Preamble::Mission);
    input.extend_from_slice(&[0, 6, 0, 4, 0, 0]);
    client.write_all(&input).unwrap();
    let mut buffer = [0u8; 4096];
    let n = client.read(&mut buffer).unwrap();
    assert_eq!(&buffer[..2], &[1, 22]);
    assert!(n > 22 + 4, "first read held {n} bytes: key reply and opening frame must be one write");
    client.shutdown(Shutdown::Both).unwrap();
    server.join().unwrap().unwrap();
}

/// Two client events in one read (420 MapLoaded + 412 AgentsLoaded) are answered by ONE
/// write: the client's next single read holds both the population and 1036 + 431.
#[test]
fn replies_owed_for_one_read_arrive_in_one_read() {
    let record = mission::tests::guardian_record(15, 1);
    let fixture = mission::PopulationFixture::new(&record, [0.0, 0.0, 7.4]).unwrap();
    let mut expect = mission::MissionPhase::new(15).unwrap().with_population(fixture.clone()).unwrap();
    let population = expect.respond(&mission::Seen::Map(gw2_proto::map_load::ClientMapEvent::MapLoaded)).unwrap().unwrap();
    let begin = expect.respond(&mission::Seen::Map(gw2_proto::map_load::ClientMapEvent::AgentsLoaded)).unwrap().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let path = std::env::temp_dir().join(format!("gw2-observe-coalesce-{}.bin", std::process::id()));
    // Public toy group for this local test; no stored identities are loaded.
    let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
    let shared = key.public().modpow(&BigUint::from(5u32), &key.p);
    let (server_path, server_fixture) = (path.clone(), fixture);
    let server = std::thread::spawn(move || {
        let fixtures = Fixtures { mission_map_key: Some(15), population: Some(server_fixture), no_capture: true, ..Fixtures::default() };
        observe(stream_of(listener), &key, &server_path, fixtures, Preamble::Mission)
    });
    let mut client = TcpStream::connect(address).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut input = preamble::fixture(Preamble::Mission);
    input.extend_from_slice(&[0, 6, 0, 4, 0, 0]);
    client.write_all(&input).unwrap();
    let mut reply = [0u8; 22];
    client.read_exact(&mut reply).unwrap();
    let mut codec = ClientCodec::new(&session_secret(&shared, &reply[2..].try_into().unwrap()));
    // Drain the opening frame.
    let mut pending = BytesMut::new();
    while codec.decode(&mut pending).unwrap().is_none() {
        let mut chunk = [0u8; 4096];
        let n = client.read(&mut chunk).unwrap();
        pending.extend_from_slice(&chunk[..n]);
    }
    assert!(pending.is_empty(), "opening and nothing else so far");
    let mut plain = BytesMut::new();
    gw2_proto::map_load::encode_empty(gw2_proto::map_load::MAP_LOADED_420, &mut plain);
    gw2_proto::map_load::encode_empty(gw2_proto::map_load::AGENTS_LOADED_412, &mut plain);
    let mut wire = BytesMut::new();
    codec.encode(Bytes::copy_from_slice(&plain), &mut wire).unwrap();
    client.write_all(&wire).unwrap();
    // ONE read must carry both replies.
    let mut chunk = [0u8; 8192];
    let n = client.read(&mut chunk).unwrap();
    let mut once = BytesMut::from(&chunk[..n]);
    assert_eq!(codec.decode(&mut once).unwrap().as_deref(), Some(&population[..]), "population in the first read");
    assert_eq!(codec.decode(&mut once).unwrap().as_deref(), Some(&begin[..]), "1036 + 431 in the same read");
    client.shutdown(Shutdown::Both).unwrap();
    server.join().unwrap().unwrap();
}
