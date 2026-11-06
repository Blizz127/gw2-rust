use super::*;
use gw2_net::ClientCodec;
use gw2_proto::Encode;
use std::net::Shutdown;

fn shared_listener_peer(address: SocketAddr, mut prefix: Vec<u8>, cut: usize) -> (TcpStream, ClientCodec, BytesMut) {
    let mut client = TcpStream::connect(address).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    prefix.extend_from_slice(&[0, 6, 0, 4, 0, 0]);
    client.write_all(&prefix[..cut]).unwrap();
    if cut != 0 {
        std::thread::sleep(Duration::from_millis(10));
    }
    client.write_all(&prefix[cut..]).unwrap();
    let mut reply = [0; 22];
    client.read_exact(&mut reply).unwrap();
    assert_eq!(&reply[..2], &[1, 22]);
    let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
    let shared = key.public().modpow(&BigUint::from(5u32), &key.p);
    let codec = ClientCodec::new(&session_secret(&shared, &reply[2..].try_into().unwrap()));
    (client, codec, BytesMut::new())
}

fn next_shared_frame(client: &mut TcpStream, codec: &mut ClientCodec, pending: &mut BytesMut) -> Bytes {
    loop {
        if let Some(frame) = codec.decode(pending).unwrap() { return frame; }
        let mut chunk = [0; 8192];
        let count = client.read(&mut chunk).unwrap();
        assert_ne!(count, 0, "shared listener closed before its response");
        pending.extend_from_slice(&chunk[..count]);
    }
}

fn send_shared_frame(client: &mut TcpStream, codec: &mut ClientCodec, plain: Bytes) {
    let mut wire = BytesMut::new();
    codec.encode(plain, &mut wire).unwrap();
    client.write_all(&wire).unwrap();
}

fn frozen_auth_server() -> (Vec<gw2_proto::schema::Message>, gw2_proto::dynamic::Rules) {
    use gw2_proto::{dynamic::{CStringEncoding, CountPrefix, Rules, StringEncoding}, schema};
    let server = schema::parse(include_str!("../../../../../data/protocol/209026/server-tables.xml"))
        .unwrap().protocols.into_iter().flat_map(|p| p.groups)
        .find(|g| g.name == "table-0x141923fe0").unwrap().server;
    let rules = Rules {
        string: Some(StringEncoding::Utf16Terminated),
        cstring: Some(CStringEncoding::Terminated),
        array_var_small: Some(CountPrefix::U8),
        array_var_large: Some(CountPrefix::U16),
        buffer_var_small: Some(CountPrefix::U8),
        buffer_var_large: Some(CountPrefix::U16),
        buffer_fixed_is_size_bytes: true,
    };
    (server, rules)
}

#[test]
fn auth_transport_groups_complete_messages_with_compression_headroom() {
    use gw2_proto::dynamic;
    let (schema, rules) = frozen_auth_server();
    let mut expected = BytesMut::new();
    let mut state = 0x12345678u32;
    for request in 0..36u32 {
        // The frozen auth server's largest buffer is message10's 4096 bytes.
        // Vary it across messages to exercise expansion, rather than relying
        // on highly compressible character fixtures to fit a transport block.
        10u16.encode(&mut expected);
        request.encode(&mut expected);
        expected.extend_from_slice(&4096u16.to_le_bytes());
        for _ in 0..4096 {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            expected.extend_from_slice(&[state as u8]);
        }
    }
    let expected = expected.freeze();
    assert!(expected.len() > 2 * u16::MAX as usize);
    let key = b"synthetic-auth-transport";
    let mut codec = ServerCodec::new(key);
    let mut wire = BytesMut::new();
    encode_auth_reply(&mut codec, expected.clone(), &mut wire).unwrap();
    assert!(wire.len() > expected.len(), "fixture must exercise compression expansion");
    let mut peer = ClientCodec::new(key);
    let mut restored = BytesMut::new();
    let mut pending = BytesMut::new();
    let (mut frames, mut messages) = (0, 0);
    for part in wire.chunks(113) {
        pending.extend_from_slice(part);
        while let Some(frame) = peer.decode(&mut pending).unwrap() {
            assert!(frame.len() <= AUTH_TRANSPORT_CHUNK);
            let mut cursor = &frame[..];
            while !cursor.is_empty() {
                assert_eq!(dynamic::decode_frame(&schema, &rules, &mut cursor).unwrap().0.id, 10);
                messages += 1;
            }
            restored.extend_from_slice(&frame);
            frames += 1;
        }
    }
    assert_eq!(frames, 3);
    assert_eq!(messages, 36);
    assert!(pending.is_empty());
    assert_eq!(&restored[..], &expected[..]);

    // Existing small responses retain exactly the same transport bytes,
    // including the same RC4 continuation after the preceding blocks.
    let small = Bytes::from_static(&[4, 0, 1, 0, 0, 0, 0]);
    let mut reference = ServerCodec::new(key);
    let mut expected_wire = BytesMut::new();
    reference.encode(small.clone(), &mut expected_wire).unwrap();
    let mut fresh = ServerCodec::new(key);
    let mut actual_wire = BytesMut::new();
    encode_auth_reply(&mut fresh, small.clone(), &mut actual_wire).unwrap();
    assert_eq!(actual_wire, expected_wire);
    let mut following = BytesMut::new();
    encode_auth_reply(&mut codec, small.clone(), &mut following).unwrap();
    assert_eq!(peer.decode(&mut following).unwrap().unwrap(), small);
    assert!(following.is_empty());
}

#[test]
fn auth_transport_full_saved_roster_and_coalesced_refreshes_over_tcp() {
    use gw2_proto::{Guid, dynamic::{self, Value}};
    let scenario = saved_population::tests::SavedScenario::new();
    let store = scenario.profile.store.clone();
    for slot in 2..character_store::MAX_CHARACTERS {
        store.create(&character_store::test_request(&format!("Roster Slot {slot:02}"))).unwrap();
    }
    let records = store.list().unwrap();
    assert_eq!(records.len(), 64);
    let (schema, rules) = frozen_auth_server();
    let server8 = schema.iter().find(|m| m.id == 8).unwrap();
    let values = server8.fields.iter().map(|field| {
        let value = match field.kind.as_str() {
            "Dword" => Value::U32(0),
            "Guid" => Value::Guid(Guid([0; 16])),
            "Byte" => Value::U8(0),
            "BufferFixed" => Value::Bytes(vec![0; field.size.unwrap() as usize]),
            "ArrayVarSmall" => Value::Array(vec![]),
            _ => panic!("unexpected server8 field"),
        };
        (field.name.clone(), value)
    }).collect();
    let mut body = BytesMut::new();
    dynamic::encode_message(server8, &rules, &Value::Struct(values), &mut body).unwrap();
    let fixture = auth::ReplyFixture::new(body.freeze().slice(1..), None).unwrap()
        .with_character_store(store).unwrap();
    let route = auth::PlayRoute::new("127.1.0.4:6112".parse().unwrap(), Guid([0; 16])).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let path = scenario.capture_path("full-roster-refresh");
    let server_path = path.clone();
    let worker = std::thread::spawn(move || {
        let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
        observe(stream_of(listener), &key, &server_path, Fixtures {
            auth_body: Some(fixture), play: Some(route), no_capture: true,
            ..Fixtures::default()
        }, Preamble::Auth)
    });
    let (mut client, mut codec, mut pending) =
        shared_listener_peer(address, preamble::fixture(Preamble::Auth), 0);
    let mut login = BytesMut::new();
    10u16.encode(&mut login);
    123456u32.encode(&mut login);
    Guid([0x11; 16]).encode(&mut login);
    Guid([0x33; 16]).encode(&mut login);
    login.extend_from_slice(&[0; 4]);
    send_shared_frame(&mut client, &mut codec, login.freeze());
    let login_reply = next_shared_frame(&mut client, &mut codec, &mut pending);
    let check_list = |cursor: &mut &[u8], request: u32, completion: u16| {
        for record in &records {
            let (message, value) = dynamic::decode_frame(&schema, &rules, cursor).unwrap();
            assert_eq!(message.id, 17);
            let Value::Struct(fields) = value else { panic!() };
            assert_eq!(fields[0].1, Value::U32(request));
            assert_eq!(fields[1].1, Value::Guid(record.guid()));
            assert_eq!(fields[5].1, Value::Text(record.name().to_owned()));
            assert_eq!(fields[6].1, Value::Bytes(record.summary().to_vec()));
        }
        let (message, value) = dynamic::decode_frame(&schema, &rules, cursor).unwrap();
        assert_eq!(message.id, completion);
        let Value::Struct(fields) = value else { panic!() };
        assert_eq!(fields[0].1, Value::U32(request));
        assert!(fields[1..].iter().all(|(_, value)| *value == Value::U32(0)));
    };
    let mut cursor = &login_reply[..];
    for id in [8, 23, 25, 24, 10] {
        assert_eq!(dynamic::decode_frame(&schema, &rules, &mut cursor).unwrap().0.id, id);
    }
    check_list(&mut cursor, 123456, 7);
    assert!(cursor.is_empty());

    // One client write owes more than 65535 plaintext bytes. Varied request
    // widths also ensure that framing cannot assume a fixed record stride.
    for requests in [&[0, 127, 128, 16383, 16384, 123456, u32::MAX, 1][..], &[42][..]] {
        let mut input = BytesMut::new();
        let mut expected = BytesMut::new();
        for &request in requests {
            25u16.encode(&mut input);
            request.encode(&mut input);
            for record in &records {
                17u16.encode(&mut expected);
                request.encode(&mut expected);
                expected.extend_from_slice(&record.auth_body());
            }
            4u16.encode(&mut expected);
            for value in [request, 0, 0, 0, 0] { value.encode(&mut expected); }
        }
        if requests.len() > 1 { assert!(expected.len() > u16::MAX as usize); }
        send_shared_frame(&mut client, &mut codec, input.freeze());
        let mut restored = BytesMut::new();
        while restored.len() < expected.len() {
            let frame = next_shared_frame(&mut client, &mut codec, &mut pending);
            assert!(frame.len() <= AUTH_TRANSPORT_CHUNK);
            // Each compressed block is independently decodable under the
            // frozen schema; no inner-message carry buffer is necessary.
            let mut cursor = &frame[..];
            while !cursor.is_empty() {
                dynamic::decode_frame(&schema, &rules, &mut cursor).unwrap();
            }
            restored.extend_from_slice(&frame);
        }
        assert_eq!(restored, expected);
        let mut cursor = &restored[..];
        for &request in requests { check_list(&mut cursor, request, 4); }
        assert!(cursor.is_empty());
    }
    client.shutdown(Shutdown::Both).unwrap();
    worker.join().unwrap().unwrap();
    assert!(!path.exists());
    assert!(!path.with_extension("preamble.bin").exists());
}

#[test]
fn shared_creation_world_listener_uses_map_and_preserves_each_saved_identity() {
    use gw2_proto::character_creation_result::CreationResponseFrame209026;
    use saved_population::tests::{SavedScenario, frames};
    let scenario = SavedScenario::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let paths: Vec<_> = (0..4).map(|i| scenario.capture_path(&format!("shared-{i}"))).collect();
    let server_paths = paths.clone();
    let profile = scenario.profile.clone();
    let server = std::thread::spawn(move || {
        let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
        for path in server_paths {
            observe(listener.accept().unwrap().0, &key, &path, Fixtures {
                mission_map_key: Some(379),
                creation: Some((profile.store.clone(), address)),
                saved_population: Some(profile.clone()),
                no_capture: true,
                ..Fixtures::default()
            }, Preamble::Mission).unwrap();
        }
    });

    // Same socket: creation, selected character0, creation with a retained
    // nonzero GUID, then selected character1. Offset16 must not select a phase.
    for (index, (map, selected, scalar, cut)) in [
        (0u32, 0, 0u32, 0), (379, 0, 1, 26),
        (0, 1, 9, 55), (379, 1, 9, 84),
    ].into_iter().enumerate() {
        let mut prefix = scenario.preamble(selected);
        prefix[16..20].copy_from_slice(&scalar.to_le_bytes());
        prefix[24..28].copy_from_slice(&map.to_le_bytes());
        if index == 0 { prefix[48..64].fill(0); }
        let (mut client, mut codec, mut pending) = shared_listener_peer(address, prefix, cut);
        let opening = next_shared_frame(&mut client, &mut codec, &mut pending);
        if map == 0 {
            assert_eq!(opening.as_ref(), &1049u16.to_le_bytes());
            let request = character_store::test_request(&format!("Shared creator{index}"));
            let mut plain = BytesMut::new();
            410u16.encode(&mut plain);
            411u16.encode(&mut plain);
            plain.extend_from_slice(&request);
            430u16.encode(&mut plain);
            send_shared_frame(&mut client, &mut codec, plain.freeze());
            let response = next_shared_frame(&mut client, &mut codec, &mut pending);
            let mut cursor = response.as_ref();
            assert!(matches!(CreationResponseFrame209026::decode_frame(&mut cursor).unwrap(),
                CreationResponseFrame209026::Startup1050));
            let CreationResponseFrame209026::Result1048(result) =
                CreationResponseFrame209026::decode_frame(&mut cursor).unwrap()
            else { panic!("creator did not send1048"); };
            let saved = scenario.profile.store.find_guid(result.character_guid).unwrap().unwrap();
            assert_eq!(saved.request(), request);
            let mut redirect = BytesMut::new();
            1065u16.encode(&mut redirect);
            address.encode(&mut redirect);
            0u32.encode(&mut redirect);
            0u32.encode(&mut redirect); // reserved scalar, not auth game type9
            379u16.encode(&mut redirect);
            0u32.encode(&mut redirect);
            0u32.encode(&mut redirect);
            assert_eq!(cursor, redirect.as_ref(), "redirect returns to this same listener");
        } else {
            let expected = frames(scenario.expected(selected));
            assert_eq!(opening, expected[0], "world opening, without creator1049");
            let mut plain = BytesMut::new();
            420u16.encode(&mut plain);
            412u16.encode(&mut plain);
            send_shared_frame(&mut client, &mut codec, plain.freeze());
            assert_eq!(next_shared_frame(&mut client, &mut codec, &mut pending), expected[1]);
            assert_eq!(next_shared_frame(&mut client, &mut codec, &mut pending), expected[2]);
        }
        assert!(pending.is_empty(), "another phase produced an extra frame");
        client.shutdown(Shutdown::Both).unwrap();
    }
    server.join().unwrap();
    assert_eq!(scenario.profile.store.list().unwrap().len(), 4);
    for path in paths {
        assert!(!path.exists());
        assert!(!path.with_extension("preamble.bin").exists());
    }
}

#[test]
fn check_config_validates_saved_profile_and_identity_without_binding() {
    let scenario = saved_population::tests::SavedScenario::new();
    let params = scenario.capture_path("check-params");
    let private = scenario.capture_path("check-private");
    let equipment = scenario.capture_path("check-equipment");
    let store = scenario.capture_path("check-store");
    let log = scenario.capture_path("check-no-log");
    // Public synthetic identity for configuration checks; no client is run.
    let modulus = (BigUint::from(1u32) << 511usize) + BigUint::from(12345u32);
    let key = ServerKey::from_private(modulus, 4u32.into(), 7u32.into());
    fs::write(&params, key.param_blob(1).to_bytes(ByteOrder::Little).unwrap()).unwrap();
    let mut exponent = [0u8; 64];
    exponent[0] = 7;
    fs::write(&private, exponent).unwrap();
    let base = scenario.profile.equipment.as_ref().unwrap();
    fs::write(&equipment, base).unwrap();
    let occupied = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = occupied.local_addr().unwrap().to_string();
    let args: Vec<String> = [
        "--check-config", "--no-capture", "--listen", &address,
        "--params", params.to_str().unwrap(), "--private", private.to_str().unwrap(),
        "--log", log.to_str().unwrap(), "--mission-preamble", "--character-store", store.to_str().unwrap(),
        "--creation-scene", "127.1.0.4:6112", "--mission-map-key", "379",
        "--mission-spawn", "0,0,0", "--mission-mesmer-values", "3,0",
        "--mission-reward-tracks", "1,2,3,4", "--mission-equipment", equipment.to_str().unwrap(),
        "--mission-heart", "56,5",
    ].into_iter().map(str::to_owned).collect();
    // This would fail with AddrInUse if the check attempted to bind.
    run(args.clone().into_iter()).unwrap();
    for duration in ["1", "65535"] {
        let mut echo = args.clone();
        echo.extend(["--mission-skill-order-echo".into(), duration.into()]);
        run(echo.into_iter()).unwrap();
    }
    for value in ["0", "37", "65535", "default-curve"] {
        let mut attributes = args.clone();
        attributes.extend(["--mission-base-attributes".into(), value.into()]);
        run(attributes.into_iter()).unwrap();
    }
    assert!(character_store::CharacterStore::open(store.clone()).unwrap().list().unwrap().is_empty());
    assert!(!log.exists());
    let mut zero_target = args.clone();
    let heart_value = zero_target.iter().position(|arg| arg == "--mission-heart").unwrap() + 1;
    zero_target[heart_value] = "56,0".into();
    assert_eq!(run(zero_target.into_iter()).unwrap_err().to_string(), "heart target must be nonzero");
    assert!(character_store::CharacterStore::open(store.clone()).unwrap().list().unwrap().is_empty());
    assert!(!log.exists());
    let no_population = ["--check-config", "--no-capture", "--mission-heart", "56,5"];
    assert_eq!(run(no_population.into_iter().map(str::to_owned)).unwrap_err().to_string(),
        "--mission-heart requires a population");
    let with_mask = equipment::EquipmentFixture::parse(base, 1).unwrap().with_mesmer_biography(&[187]).unwrap();
    fs::write(&equipment, with_mask.bytes()).unwrap();
    assert!(run(args.clone().into_iter()).unwrap_err().to_string().contains("slot5"));
    fs::write(&equipment, base).unwrap();
    exponent[0] = 8;
    fs::write(&private, exponent).unwrap();
    assert!(run(args.clone().into_iter()).unwrap_err().to_string().contains("identity mismatch"));
    exponent[0] = 7;
    fs::write(&private, exponent).unwrap();
    fs::write(store.join("unrecognized"), b"preserve me").unwrap();
    assert!(run(args.into_iter()).is_err());
    assert_eq!(fs::read(store.join("unrecognized")).unwrap(), b"preserve me");
    assert!(!log.exists());
}

#[test]
fn base_attributes_config_requires_explicit_profile_and_population() {
    let parse = |tail: &[&str]| {
        run(["--check-config", "--no-capture"].into_iter().chain(tail.iter().copied())
            .map(str::to_owned)).unwrap_err().to_string()
    };
    assert_eq!(parse(&["--mission-base-attributes"]), "missing initial base attributes");
    for value in ["65536", "-1", "1.5", "NaN", "default", "--mission-preamble"] {
        assert_eq!(parse(&["--mission-base-attributes", value]), "base attributes must be a u16 or default-curve");
    }
    assert_eq!(parse(&["--mission-base-attributes", "0", "--mission-base-attributes", "37"]),
        "duplicate --mission-base-attributes");
    assert_eq!(parse(&["--mission-base-attributes", "37"]),
        "--mission-base-attributes requires a population");
    assert_eq!(parse(&["--mission-base-attributes", "default-curve"]),
        "--mission-base-attributes requires a population");
}

#[test]
fn skill_order_echo_config_requires_an_explicit_nonzero_u16_duration() {
    let parse = |tail: &[&str]| {
        run(["--check-config", "--no-capture"].into_iter().chain(tail.iter().copied())
            .map(str::to_owned)).unwrap_err()
    };
    assert_eq!(parse(&["--mission-skill-order-echo"]).to_string(),
        "missing research skill order duration in ms");
    for duration in ["0", "65536", "-1", "1.5", "--mission-preamble"] {
        assert_eq!(parse(&["--mission-skill-order-echo", duration]).to_string(),
            "research skill order duration must be 1..=65535 ms");
    }
    assert_eq!(parse(&["--mission-skill-order-echo", "1", "--mission-skill-order-echo", "2"]).to_string(),
        "duplicate --mission-skill-order-echo");
    assert_eq!(parse(&["--mission-skill-order-echo", "1"]).to_string(),
        "--mission-skill-order-echo requires a population");
}

#[test]
fn shared_creation_combat_options_apply_only_to_the_saved_world_join() {
    use gw2_proto::{agent_behavior::AgentBehaviorFrame209026, map_load::ClientMapEvent, renown::RenownFrame209026};
    let scenario = saved_population::tests::SavedScenario::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let paths = [scenario.capture_path("combat-creator"), scenario.capture_path("combat-world")];
    let server_paths = paths.clone();
    let profile = scenario.profile.clone();
    // Synthetic combat inputs exercise routing, not a retail encounter.
    let talk = mission::TalkNpc::new(100, [1.0, 2.0, 3.0], "Test NPC".into()).unwrap();
    let server_talk = talk.clone();
    let server = std::thread::spawn(move || {
        let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
        for path in server_paths {
            observe(listener.accept().unwrap().0, &key, &path, Fixtures {
                mission_map_key: Some(379),
                creation: Some((profile.store.clone(), address)),
                saved_population: Some(profile.clone()),
                talk_npc: Some(server_talk.clone()),
                hostile_npcs: vec![(101, [4.0, 5.0, 6.0])],
                skill_order_echo: NonZeroU16::new(4321), npc_cast: Some(10176), damage_on_skill: Some(200.0),
                xp_per_kill: Some(50), starter_bag: Some(959),
                kill_loot: true, npc_attack: Some((10.0, 777.0)), sourced_damage: Some(1),
                heart: Some((56, 5)),
                no_capture: true,
                ..Fixtures::default()
            }, Preamble::Mission).unwrap();
        }
    });
    let mut prefix = scenario.preamble(0);
    prefix[24..28].fill(0);
    let (mut client, mut codec, mut pending) = shared_listener_peer(address, prefix, 26);
    assert_eq!(next_shared_frame(&mut client, &mut codec, &mut pending).as_ref(),
        &1049u16.to_le_bytes());
    let mut ready = BytesMut::new();
    410u16.encode(&mut ready);
    411u16.encode(&mut ready);
    send_shared_frame(&mut client, &mut codec, ready.freeze());
    assert_eq!(next_shared_frame(&mut client, &mut codec, &mut pending).as_ref(),
        &1050u16.to_le_bytes());
    assert!(pending.is_empty());
    client.shutdown(Shutdown::Both).unwrap();

    // A subsequent world join on that same listener keeps every combat option.
    // Compare to a standalone world phase with the same explicit population.
    let mut expected = mission::MissionPhase::new(379).unwrap()
        .with_population(scenario.expected(0)).unwrap()
        .with_talk_npc(talk).with_hostile_npc(101, [4.0, 5.0, 6.0]).unwrap()
        .with_skill_order_echo(NonZeroU16::new(4321).unwrap()).with_npc_cast(10176).with_damage_on_skill(200.0)
        .with_xp_per_kill(50).with_starter_bag(959)
        .with_kill_loot().with_npc_attack(10.0, 777.0).with_sourced_damage(1)
        .with_heart(56, 5).unwrap();
    let mut heart_opening = BytesMut::new();
    for frame in [
        RenownFrame209026::AddRegion { player: 1, region: 1 },
        RenownFrame209026::CreateCounter { objective: 0x38, target: 5 },
        RenownFrame209026::AddSubRegion { player: 1, region: 1, objective: 0x38, heart: 56 },
    ] {
        frame.encode_frame(&mut heart_opening).unwrap();
    }
    // 655 (enter) follows only when the spawn lies inside heart 56's client
    // polygon; the region, counter and sub-region always open.
    let mut prefix = scenario.preamble(0);
    prefix[24..28].copy_from_slice(&379u32.to_le_bytes());
    let (mut client, mut codec, mut pending) = shared_listener_peer(address, prefix, 55);
    assert_eq!(next_shared_frame(&mut client, &mut codec, &mut pending), expected.opening());
    let mut loaded = BytesMut::new();
    420u16.encode(&mut loaded);
    412u16.encode(&mut loaded);
    send_shared_frame(&mut client, &mut codec, loaded.freeze());
    for event in [ClientMapEvent::MapLoaded, ClientMapEvent::AgentsLoaded] {
        let reply = expected.respond(&mission::Seen::Map(event)).unwrap().unwrap();
        let received = next_shared_frame(&mut client, &mut codec, &mut pending);
        assert_eq!(received, reply);
        if event == ClientMapEvent::AgentsLoaded {
            // With a talk NPC the batch also ends with 384 (heart marker on
            // that NPC) after the four opening frames, so check containment.
            assert!(received.windows(heart_opening.len()).any(|w| w == &heart_opening[..]),
                "saved world must show heart56 with target5");
        }
    }
    let mut skill = BytesMut::new();
    AgentBehaviorFrame209026 {
        agent_index: 1, remote_tick: 77, command_flags: 3, behavior_value: 0,
        // Attack10170 exercises damage; healing10176 is explicitly excluded.
        behavior_data: vec![0x23, 3, 0, 0, 0, 0xba, 0x27, 0, 0, 1],
    }.encode_client_frame(&mut skill).unwrap();
    let seen = expected.push(&skill);
    assert!(matches!(seen.as_slice(), [mission::Seen::Skill(_)]));
    let reply = expected.respond(&seen[0]).unwrap().unwrap();
    send_shared_frame(&mut client, &mut codec, skill.freeze());
    let received = next_shared_frame(&mut client, &mut codec, &mut pending);
    assert_eq!(received, reply);
    let mut first_kill = BytesMut::new();
    RenownFrame209026::SetCounter { objective: 0x38, value: 1 }.encode_frame(&mut first_kill).unwrap();
    assert!(received.windows(first_kill.len()).any(|bytes| bytes == first_kill.as_ref()),
        "combat kill on the saved world must advance its heart");
    assert!(pending.is_empty());
    client.shutdown(Shutdown::Both).unwrap();
    server.join().unwrap();
    for path in paths {
        assert!(!path.exists());
        assert!(!path.with_extension("preamble.bin").exists());
    }
}

#[test]
fn shared_creation_combat_configuration_is_validated_before_binding() {
    let scenario = saved_population::tests::SavedScenario::new();
    let store = scenario.capture_path("combat-cli-store");
    let args: Vec<String> = [
        "--no-capture", "--mission-preamble", "--character-store", store.to_str().unwrap(),
        "--creation-scene", "127.1.0.4:6112", "--mission-map-key", "379",
        "--mission-spawn", "0,0,0", "--mission-mesmer-values", "3,0",
        "--mission-reward-tracks", "1,2,3,4", "--mission-health", "777",
        "--mission-hostile-npc", "101,4,5,6", "--mission-damage-on-skill", "10",
        "--mission-npc-attack", "5", "--mission-starter-bag", "959", "--mission-kill-loot",
    ].into_iter().map(str::to_owned).collect();
    assert_eq!(run(args.clone().into_iter()).unwrap_err().to_string(), "missing --params");
    let npc = args.iter().position(|s| s == "--mission-hostile-npc").unwrap();
    for value in ["0,4,5,6", "268435456,4,5,6", "101,NaN,5,6", "101,4,inf,6"] {
        let mut bad = args.clone();
        bad[npc + 1] = value.into();
        assert_ne!(run(bad.into_iter()).unwrap_err().to_string(), "missing --params", "{value}");
    }
    for option in ["--mission-hostile-npc", "--mission-damage-on-skill", "--mission-health", "--mission-starter-bag"] {
        let mut bad = args.clone();
        let at = bad.iter().position(|s| s == option).unwrap();
        bad.drain(at..at + 2);
        assert_ne!(run(bad.into_iter()).unwrap_err().to_string(), "missing --params", "{option}");
    }
    let mut bad = args;
    let at = bad.iter().position(|s| s == "--mission-starter-bag").unwrap();
    bad[at + 1] = "123".into();
    assert!(run(bad.into_iter()).unwrap_err().to_string().contains("bag creation message id"));
}

#[test]
fn shared_creation_world_listener_rejects_bad_world_selection_and_other_maps_before_key_reply() {
    use saved_population::tests::SavedScenario;
    let scenario = SavedScenario::new();
    let mut world = scenario.preamble(0);
    world[24..28].copy_from_slice(&379u32.to_le_bytes());
    let mut cases = Vec::new();
    for guid in [[0; 16], [0xfe; 16]] {
        let mut prefix = world.clone();
        prefix[48..64].copy_from_slice(&guid);
        cases.push((prefix, io::ErrorKind::NotFound));
    }
    for map in [1u32, 15, 28, 138, u32::MAX] {
        let mut prefix = world.clone();
        prefix[24..28].copy_from_slice(&map.to_le_bytes());
        cases.push((prefix, io::ErrorKind::InvalidData));
    }
    let mut wrong_header = world[..16].to_vec();
    wrong_header[12] ^= 1;
    cases.push((wrong_header, io::ErrorKind::InvalidData));
    cases.push((world[..63].to_vec(), io::ErrorKind::UnexpectedEof));
    for (index, (prefix, expected)) in cases.into_iter().enumerate() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let path = scenario.capture_path(&format!("shared-rejected-{index}"));
        let server_path = path.clone();
        let profile = scenario.profile.clone();
        let server = std::thread::spawn(move || {
            let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
            observe(stream_of(listener), &key, &server_path, Fixtures {
                mission_map_key: Some(379),
                creation: Some((profile.store.clone(), address)),
                saved_population: Some(profile),
                no_capture: true,
                ..Fixtures::default()
            }, Preamble::Mission)
        });
        let mut client = TcpStream::connect(address).unwrap();
        client.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        client.write_all(&prefix).unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).unwrap();
        assert!(response.is_empty(), "rejected join received a key reply or creator opening");
        assert_eq!(server.join().unwrap().unwrap_err().kind(), expected);
        assert!(!path.exists());
        assert!(!path.with_extension("preamble.bin").exists());
    }
    assert_eq!(scenario.profile.store.list().unwrap().len(), 2);
}

#[test]
fn shared_creation_cli_requires_the_complete_saved_world_profile() {
    let scenario = saved_population::tests::SavedScenario::new();
    let store = scenario.capture_path("cli-store").to_string_lossy().into_owned();
    let args: Vec<String> = [
        "--no-capture", "--mission-preamble", "--character-store", &store,
        "--creation-scene", "127.1.0.4:6112", "--mission-map-key", "379",
        "--mission-spawn", "0,0,0", "--mission-mesmer-values", "3,0",
        "--mission-reward-tracks", "1,2,3,4",
    ].into_iter().map(str::to_owned).collect();
    // All role/profile checks pass, then the omitted test identity stops the
    // command before binding. No actual private identity is opened.
    assert_eq!(run(args.clone().into_iter()).unwrap_err().to_string(), "missing --params");
    let mut standalone = args[..6].to_vec();
    assert_eq!(run(standalone.clone().into_iter()).unwrap_err().to_string(), "missing --params");
    standalone.remove(0);
    assert!(run(standalone.into_iter()).unwrap_err().to_string().contains("--no-capture"));
    for (option, width) in [("--mission-preamble", 1), ("--character-store", 2),
        ("--mission-spawn", 2), ("--mission-mesmer-values", 2)]
    {
        let mut bad = args.clone();
        let at = bad.iter().position(|value| value == option).unwrap();
        bad.drain(at..at + width);
        assert_ne!(run(bad.into_iter()).unwrap_err().to_string(), "missing --params", "{option}");
    }
    let mut wrong_map = args;
    let at = wrong_map.iter().position(|value| value == "379").unwrap();
    wrong_map[at] = "15".into();
    assert!(run(wrong_map.into_iter()).unwrap_err().to_string().contains("Norn map379"));
}

#[test]
fn shared_creation_change_preserves_explicit_standalone_creator() {
    let scenario = saved_population::tests::SavedScenario::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let path = scenario.capture_path("standalone-creator");
    let server_path = path.clone();
    let store = scenario.profile.store.clone();
    let server = std::thread::spawn(move || {
        let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
        observe(stream_of(listener), &key, &server_path, Fixtures {
            creation: Some((store, address)),
            no_capture: true,
            ..Fixtures::default()
        }, Preamble::Mission)
    });
    // Without the explicit combined option, the preamble's formerly opaque
    // map value does not override the configured standalone creation role.
    let mut prefix = scenario.preamble(0);
    prefix[24..28].copy_from_slice(&379u32.to_le_bytes());
    let (mut client, mut codec, mut pending) = shared_listener_peer(address, prefix, 0);
    assert_eq!(next_shared_frame(&mut client, &mut codec, &mut pending).as_ref(),
        &1049u16.to_le_bytes());
    client.shutdown(Shutdown::Both).unwrap();
    server.join().unwrap().unwrap();
    assert!(!path.exists());
    assert!(!path.with_extension("preamble.bin").exists());
}

#[test]
fn saved_population_over_tcp_selects_reopened_character_at_exact_preamble_offset() {
    use saved_population::tests::{SavedScenario, frames};
    let scenario = SavedScenario::new();
    for selected in 0..2 {
        let expected = frames(scenario.expected(selected));
        // Fragment immediately before, inside and after the selected GUID.
        for cut in [48, 55, 64] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let path = scenario.capture_path(&format!("selected-{selected}-cut-{cut}"));
            let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
            let shared = key.public().modpow(&BigUint::from(5u32), &key.p);
            let server_path = path.clone();
            let profile = scenario.profile.clone();
            // A fixed population must not override the selected saved record.
            let other_character = scenario.expected(1 - selected);
            let server = std::thread::spawn(move || {
                observe(
                    stream_of(listener),
                    &key,
                    &server_path,
                    Fixtures {
                        mission_map_key: Some(379),
                        population: Some(other_character),
                        saved_population: Some(profile),
                        no_capture: true,
                        ..Fixtures::default()
                    },
                    Preamble::Mission,
                )
            });
            let mut client = TcpStream::connect(address).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut input = scenario.preamble(selected);
            input.extend_from_slice(&[0, 6, 0, 4, 0, 0]);
            client.write_all(&input[..cut]).unwrap();
            client.flush().unwrap();
            std::thread::sleep(Duration::from_millis(10));
            client.write_all(&input[cut..]).unwrap();
            let mut reply = [0; 22];
            client.read_exact(&mut reply).unwrap();
            assert_eq!(&reply[..2], &[1, 22]);
            let mut codec =
                ClientCodec::new(&session_secret(&shared, &reply[2..].try_into().unwrap()));
            let mut pending = BytesMut::new();
            let mut actual = Vec::new();
            while actual.len() < expected.len() {
                if let Some(frame) = codec.decode(&mut pending).unwrap() {
                    actual.push(frame);
                    if actual.len() == 1 {
                        let mut plain = BytesMut::new();
                        gw2_proto::map_load::encode_empty(
                            gw2_proto::map_load::MAP_LOADED_420,
                            &mut plain,
                        );
                        gw2_proto::map_load::encode_empty(
                            gw2_proto::map_load::AGENTS_LOADED_412,
                            &mut plain,
                        );
                        let mut wire = BytesMut::new();
                        codec.encode(plain.freeze(), &mut wire).unwrap();
                        client.write_all(&wire).unwrap();
                    }
                    continue;
                }
                let mut chunk = [0; 4096];
                let count = client.read(&mut chunk).unwrap();
                assert_ne!(count, 0, "selected {selected}, cut {cut}: premature close");
                pending.extend_from_slice(&chunk[..count]);
            }
            assert_eq!(
                actual.as_slice(),
                &expected,
                "selected {selected}, cut {cut}"
            );
            client.shutdown(Shutdown::Both).unwrap();
            server.join().unwrap().unwrap();
            assert!(!path.exists());
            assert!(!path.with_extension("preamble.bin").exists());
        }
    }
}

#[test]
fn saved_population_over_tcp_rejects_unknown_or_unvalidated_selection_before_key_reply() {
    use saved_population::tests::SavedScenario;
    let scenario = SavedScenario::new();
    let mut unknown = scenario.preamble(0);
    // Neighbor 32..48 still contains a real saved GUID, and a fixed fallback
    // is supplied below. Neither may rescue an unknown selected GUID.
    unknown[48..64].fill(0);
    let mut wrong_build = unknown[..16].to_vec();
    wrong_build[4] ^= 1;
    let mut wrong_length = unknown[..16].to_vec();
    wrong_length[12] ^= 1;
    for (label, input, expected_kind) in [
        ("unknown", unknown, io::ErrorKind::NotFound),
        ("wrong-build", wrong_build, io::ErrorKind::InvalidData),
        ("wrong-length", wrong_length, io::ErrorKind::InvalidData),
        (
            "incomplete",
            scenario.preamble(0)[..63].to_vec(),
            io::ErrorKind::UnexpectedEof,
        ),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let path = scenario.capture_path(label);
        let server_path = path.clone();
        let profile = scenario.profile.clone();
        let fallback = scenario.expected(1);
        let server = std::thread::spawn(move || {
            let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
            observe(
                stream_of(listener),
                &key,
                &server_path,
                Fixtures {
                    mission_map_key: Some(379),
                    population: Some(fallback),
                    saved_population: Some(profile),
                    no_capture: true,
                    ..Fixtures::default()
                },
                Preamble::Mission,
            )
        });
        let mut client = TcpStream::connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        client.write_all(&input).unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        let mut reply = Vec::new();
        client.read_to_end(&mut reply).unwrap();
        assert!(
            reply.is_empty(),
            "{label}: sent data before validating selection"
        );
        assert_eq!(
            server.join().unwrap().unwrap_err().kind(),
            expected_kind,
            "{label}"
        );
        assert!(!path.exists());
        assert!(!path.with_extension("preamble.bin").exists());
    }
}

#[test]
fn explicit_profession_fixture_csv_checks_width_and_range() {
    assert_eq!(parse_csv::<u8, 2>("3,0", "bad").unwrap(), [3, 0]);
    assert_eq!(parse_csv::<u32, 4>("1,128,16384,4294967295", "bad").unwrap(), [1, 128, 16384, u32::MAX]);
    for input in ["", "3", "3,0,1", "256,0", "-1,0", "3,", "3,0x0"] {
        assert!(parse_csv::<u8, 2>(input, "bad").is_err(), "{input}");
    }
    assert!(parse_csv::<u32, 4>("1,2,3,4294967296", "bad").is_err());
}

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
    assert_eq!(&next(&mut client, &mut codec)[..], &[0x0c, 0x04, 0xaf, 0x01, 0x53, 0x00, 0x01, 0x00, 0x01], "1036, 431, then 83 client control for agent 1");
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
    mission_population_over_tcp_for_every_split(15, fixture);
}

#[test]
fn norn_mesmer_tutorial_population_over_tcp_for_every_split() {
    let mut record = mission::tests::guardian_record(379, 7);
    let summary_start = record.len() - 159;
    record[summary_start + 0x21] = 3; // Norn, preserved in coreStats appearance.
    let fixture = mission::PopulationFixture::with_profession(
        &record, [0.0; 3],
        gw2_proto::population::PopulationProfession209026::Mesmer { values: [3, 0] },
        [1, 2, 3, 4], // Deliberately synthetic, not selected retail tracks.
    ).unwrap();
    mission_population_over_tcp_for_every_split(379, fixture);
}

fn mission_population_over_tcp_for_every_split(map_key: u32, fixture: mission::PopulationFixture) {
    let mut expect = mission::MissionPhase::new(map_key).unwrap().with_population(fixture.clone()).unwrap();
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
        let path = std::env::temp_dir().join(format!("gw2-observe-deployed-{}-{map_key}-{cut}.bin", std::process::id()));
        // Public toy group for this local test; no stored identities are loaded.
        let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
        let shared = key.public().modpow(&BigUint::from(5u32), &key.p);
        let (server_path, server_fixture) = (path.clone(), fixture.clone());
        let server = std::thread::spawn(move || {
            let fixtures = Fixtures {
                mission_map_key: Some(map_key),
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

/// --mission-population-delay-ms over TCP: nothing follows 420 until the hold has passed,
/// with no client traffic meanwhile; then the population, and 1036 + 431 after 412.
#[test]
fn held_population_goes_out_after_the_delay_over_tcp() {
    let record = mission::tests::guardian_record(15, 1);
    let fixture = mission::PopulationFixture::new(&record, [0.0, 0.0, 7.4]).unwrap();
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
    let delay = Duration::from_millis(400);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let path = std::env::temp_dir().join(format!("gw2-observe-held-{}.bin", std::process::id()));
    // Public toy group for this local test; no stored identities are loaded.
    let key = ServerKey::from_private(7919u32.into(), 4u32.into(), 7u32.into());
    let shared = key.public().modpow(&BigUint::from(5u32), &key.p);
    let server_path = path.clone();
    let server = std::thread::spawn(move || {
        let fixtures = Fixtures {
            mission_map_key: Some(15),
            population: Some(fixture),
            population_delay: delay,
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
    let mut next = |client: &mut TcpStream, codec: &mut ClientCodec| loop {
        if let Some(frame) = codec.decode(&mut pending).unwrap() {
            return frame;
        }
        let mut chunk = [0u8; 4096];
        let n = client.read(&mut chunk).unwrap();
        assert_ne!(n, 0, "server closed early");
        pending.extend_from_slice(&chunk[..n]);
    };
    assert_eq!(next(&mut client, &mut codec), opening);
    let send = |client: &mut TcpStream, codec: &mut ClientCodec, id| {
        let mut plain = BytesMut::new();
        gw2_proto::map_load::encode_empty(id, &mut plain);
        let mut wire = BytesMut::new();
        codec.encode(plain.freeze(), &mut wire).unwrap();
        client.write_all(&wire).unwrap();
    };
    let loaded_at = std::time::Instant::now();
    send(&mut client, &mut codec, gw2_proto::map_load::MAP_LOADED_420);
    assert_eq!(next(&mut client, &mut codec), population, "population after the hold");
    let waited = loaded_at.elapsed();
    assert!(waited >= delay, "population came {waited:?} after 420, before the {delay:?} hold");
    assert!(waited < delay + Duration::from_secs(2), "population came late: {waited:?}");
    send(&mut client, &mut codec, gw2_proto::map_load::AGENTS_LOADED_412);
    assert_eq!(next(&mut client, &mut codec), begin, "1036 + 431 after 412");
    client.shutdown(Shutdown::Both).unwrap();
    server.join().unwrap().unwrap();
    assert!(!path.exists(), "no-capture mode created a capture file");
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
