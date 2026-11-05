//! Opt-in creation scene for the explicit local account's saved characters.
//! Source: 209026-character-creation.md and 209026-creation-mission-binding.md.

use bytes::{Bytes, BytesMut};
use gw2_proto::{
    Encode, Guid,
    character_creation_result::{CreationResponseFrame209026, CreationResult209026, RESULT_CONSUMER_BLOB_LIMIT209026},
    dynamic,
    schema::Message,
};
use std::{io, net::SocketAddr};
use super::{character_store::CharacterStore, mission};

pub struct CreationScene {
    store: CharacterStore,
    world_address: SocketAddr,
    client: Vec<Message>,
    pending: BytesMut,
    started: bool,
    ready: bool,
    created_request: Option<Bytes>,
    redirected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use gw2_proto::{
        character_creation::CreationRequest209026,
        dynamic::Value,
        schema,
    };
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "gw2-creation-scene-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(&path).unwrap();
            Self(path)
        }

        fn store(&self, case: usize) -> CharacterStore {
            CharacterStore::open(self.0.join(format!("case-{case}"))).unwrap()
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn world() -> SocketAddr {
        "127.1.0.4:6112".parse().unwrap()
    }

    fn empty(id: u16) -> [u8; 2] {
        id.to_le_bytes()
    }

    // Decode replies independently through the frozen client's extracted table.
    fn server_messages() -> Vec<Message> {
        const TABLES: &str =
            include_str!("../../../../../data/protocol/209026/server-tables.xml");
        schema::parse(TABLES)
            .unwrap()
            .protocols
            .into_iter()
            .find(|protocol| protocol.name == "0")
            .unwrap()
            .groups
            .into_iter()
            .find(|group| group.name == "table-0x1422ddbf0")
            .unwrap()
            .server
    }

    #[test]
    fn opening_and_fragmented_startup_emit_each_expected_response_once() {
        let temp = Temp::new();
        let store = temp.store(0);
        let mut scene = CreationScene::new(store.clone(), world()).unwrap();
        assert_eq!(&scene.opening()[..], &[0x19, 0x04]); // 1049
        let server = server_messages();
        let opening = scene.opening();
        let mut cursor = &opening[..];
        let (message, _) =
            dynamic::decode_frame(&server, &mission::rules(), &mut cursor).unwrap();
        assert_eq!(message.id, 1049);
        assert!(cursor.is_empty());

        let start = empty(410);
        assert!(scene.push(&start[..1]).unwrap().is_none());
        let response = scene.push(&start[1..]).unwrap().unwrap();
        assert_eq!(&response[..], &[0x1a, 0x04]); // 1050
        let mut cursor = &response[..];
        let (message, _) =
            dynamic::decode_frame(&server, &mission::rules(), &mut cursor).unwrap();
        assert_eq!(message.id, 1050);
        assert!(cursor.is_empty());
        assert!(scene.push(&empty(410)).unwrap().is_none());

        let ready = empty(411);
        assert!(scene.push(&ready[..1]).unwrap().is_none());
        assert!(scene.push(&ready[1..]).unwrap().is_none());
        assert!(scene.push(&[empty(410), empty(411)].concat()).unwrap().is_none());
        assert!(store.list().unwrap().is_empty());
        assert!(CreationScene::new(store, "192.0.2.1:6112".parse().unwrap()).is_err());
    }

    #[test]
    fn readiness_and_saved_character_are_required_before_creation_or_transfer() {
        let temp = Temp::new();
        let request = crate::character_store::test_request("Early Mesmer");
        let cases = [
            empty(411).to_vec(),
            request.clone(),
            [empty(410).as_slice(), request.as_slice()].concat(),
            empty(430).to_vec(),
            [empty(410), empty(430)].concat(),
            [empty(410), empty(411), empty(430)].concat(),
        ];
        for (case, input) in cases.into_iter().enumerate() {
            let store = temp.store(case);
            let mut scene = CreationScene::new(store.clone(), world()).unwrap();
            let error = scene.push(&input).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData, "case {case}");
            assert!(store.list().unwrap().is_empty(), "case {case} persisted a character");
        }
    }

    #[test]
    fn fragmented_and_coalesced_creation_persists_then_redirects_with_exact_fields() {
        let temp = Temp::new();
        let server = server_messages();
        // Accepted wire/scalar fixture, not claimed to be a retail cosmetic or
        // biography-valid capture. It includes opaque bytes and multibyte colors.
        let name = "Saved Ω🦀";
        let request = crate::character_store::test_request(name);
        let parsed = CreationRequest209026::parse(&request).unwrap();
        assert!(parsed.passes_context_scalar_checks());
        assert_eq!((parsed.appearance()[27], parsed.profession()), (3, 7));
        let input = [
            empty(410).as_slice(),
            empty(411).as_slice(),
            request.as_slice(),
            empty(430).as_slice(),
        ].concat();

        for cut in 0..=input.len() {
            let store = temp.store(cut);
            let mut scene = CreationScene::new(store.clone(), world()).unwrap();
            let mut responses = BytesMut::new();
            for (index, part) in [&input[..cut], &input[cut..]].into_iter().enumerate() {
                if let Some(response) = scene.push(part).unwrap() {
                    responses.extend_from_slice(&response);
                }
                if index == 0 && cut < 4 + request.len() {
                    assert!(store.list().unwrap().is_empty(), "partial 415 persisted at split {cut}");
                }
            }
            let mut cursor = &responses[..];
            assert!(matches!(
                CreationResponseFrame209026::decode_frame(&mut cursor).unwrap(),
                CreationResponseFrame209026::Startup1050
            ));
            let CreationResponseFrame209026::Result1048(result) =
                CreationResponseFrame209026::decode_frame(&mut cursor).unwrap()
            else {
                panic!("creation did not return 1048 at split {cut}");
            };
            assert_eq!(result.name_units, name.encode_utf16().collect::<Vec<_>>());
            assert_ne!(result.character_guid, Guid([0; 16]));
            assert_eq!(result.raw_guid, Guid([0; 16]));
            assert_eq!(result.raw_word, 0);
            assert_eq!(result.raw_qwords, [0; 2]);
            assert_eq!(result.blob.len(), 159);
            assert_eq!(result.blob[0], 11);

            // A fresh store handle must recover the exact request and identity.
            let reopened = temp.store(cut);
            let saved = reopened.find_guid(result.character_guid).unwrap().unwrap();
            assert_eq!(saved.name(), name);
            assert_eq!(saved.request(), request);
            assert_eq!(saved.summary().as_slice(), result.blob);
            assert_eq!(reopened.list().unwrap().len(), 1);

            let (message, value) =
                dynamic::decode_frame(&server, &mission::rules(), &mut cursor).unwrap();
            assert_eq!(message.id, 1065);
            let Value::Struct(fields) = value else { panic!() };
            let values: Vec<_> = fields.into_iter().map(|(_, value)| value).collect();
            assert_eq!(values, [
                Value::Address(world()),
                Value::U32(0),
                Value::U32(9),
                Value::U16(379),
                Value::U32(0),
                Value::U32(0),
            ]);
            assert!(cursor.is_empty(), "unexpected trailing response at split {cut}");

            // All three responses must also fit the independent extracted table.
            let mut cursor = &responses[..];
            for id in [1050, 1048, 1065] {
                let (message, _) =
                    dynamic::decode_frame(&server, &mission::rules(), &mut cursor).unwrap();
                assert_eq!(message.id, id);
            }
            assert!(cursor.is_empty());

            let retry = [request.as_slice(), empty(430).as_slice()].concat();
            assert!(scene.push(&retry).unwrap().is_none());
            assert_eq!(reopened.list().unwrap().len(), 1);
            assert_eq!(reopened.find_name(name).unwrap().unwrap().guid(), saved.guid());
        }
    }

    #[test]
    fn saved_name_conflict_returns_error_ui_and_corrected_name_can_create() {
        let temp = Temp::new();
        let store = temp.store(0);
        let existing_request = crate::character_store::test_request("Existing Mesmer");
        let existing = store.create(&existing_request).unwrap();
        // A different spelling with the same local name key is a real store
        // conflict, rather than the store's exact-request idempotent retry.
        let conflict = crate::character_store::test_request("EXISTING MESMER");
        let corrected = crate::character_store::test_request("New Mesmer");
        let mut scene = CreationScene::new(store.clone(), world()).unwrap();
        scene.push(&[empty(410), empty(411)].concat()).unwrap().unwrap();

        let response = scene.push(&conflict).unwrap().unwrap();
        let mut cursor = &response[..];
        assert!(matches!(
            CreationResponseFrame209026::decode_frame(&mut cursor).unwrap(),
            CreationResponseFrame209026::ErrorUi1051 { raw_dwords: [0x19, 0, 0, 0] }
        ));
        assert!(cursor.is_empty());
        let server = server_messages();
        let mut cursor = &response[..];
        let (message, value) =
            dynamic::decode_frame(&server, &mission::rules(), &mut cursor).unwrap();
        assert_eq!(message.id, 1051);
        let Value::Struct(fields) = value else { panic!() };
        assert_eq!(
            fields.into_iter().map(|(_, value)| value).collect::<Vec<_>>(),
            [Value::U32(0x19), Value::U32(0), Value::U32(0), Value::U32(0)],
        );
        assert!(cursor.is_empty());
        assert_eq!(store.list().unwrap().len(), 1);
        assert_eq!(store.find_guid(existing.guid()).unwrap().unwrap().request(), existing_request);

        // The error UI permits a renamed 415 without restarting the scene.
        let response = scene.push(&corrected).unwrap().unwrap();
        let mut cursor = &response[..];
        let CreationResponseFrame209026::Result1048(result) =
            CreationResponseFrame209026::decode_frame(&mut cursor).unwrap()
        else {
            panic!("corrected name did not succeed");
        };
        assert!(cursor.is_empty());
        assert_eq!(result.name_units, "New Mesmer".encode_utf16().collect::<Vec<_>>());
        assert_ne!(result.character_guid, existing.guid());
        let reopened = temp.store(0);
        assert_eq!(reopened.list().unwrap().len(), 2);
        assert_eq!(reopened.find_guid(result.character_guid).unwrap().unwrap().request(), corrected);
        assert_eq!(reopened.find_guid(existing.guid()).unwrap().unwrap().request(), existing_request);
    }

    #[test]
    fn repeated_creation_is_idempotent_and_different_request_is_rejected() {
        let temp = Temp::new();
        let store = temp.store(0);
        let mut scene = CreationScene::new(store.clone(), world()).unwrap();
        let request = crate::character_store::test_request("One Mesmer");
        let input = [
            empty(410).as_slice(), empty(411).as_slice(),
            request.as_slice(), request.as_slice(),
        ].concat();
        let reply = scene.push(&input).unwrap().unwrap();
        let mut cursor = &reply[..];
        CreationResponseFrame209026::decode_frame(&mut cursor).unwrap();
        assert!(matches!(
            CreationResponseFrame209026::decode_frame(&mut cursor).unwrap(),
            CreationResponseFrame209026::Result1048(_)
        ));
        assert!(cursor.is_empty(), "identical 415 must not emit a second result");
        let saved = store.list().unwrap();
        assert_eq!(saved.len(), 1);
        assert!(scene.push(&request).unwrap().is_none());

        let different = crate::character_store::test_request("Other Mesmer");
        let error = scene.push(&different).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(store.list().unwrap().len(), 1);
        assert!(store.find_name("Other Mesmer").unwrap().is_none());
        assert_eq!(store.find_name("One Mesmer").unwrap().unwrap().guid(), saved[0].guid());

        let redirect = scene.push(&[empty(430), empty(430)].concat()).unwrap().unwrap();
        let server = server_messages();
        let mut cursor = &redirect[..];
        let (message, _) =
            dynamic::decode_frame(&server, &mission::rules(), &mut cursor).unwrap();
        assert_eq!(message.id, 1065);
        assert!(cursor.is_empty(), "repeated 430 must not emit another redirect");
        assert!(scene.push(&empty(430)).unwrap().is_none());
    }
}


fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

impl CreationScene {
    pub fn new(store: CharacterStore, world_address: SocketAddr) -> io::Result<Self> {
        if !world_address.ip().is_loopback() {
            return Err(invalid("creation scene destination must be loopback"));
        }
        Ok(Self {
            store, world_address, client: mission::client_messages()?,
            pending: BytesMut::new(), started: false, ready: false,
            created_request: None, redirected: false,
        })
    }

    pub fn opening(&self) -> Bytes {
        let mut out = BytesMut::new();
        CreationResponseFrame209026::Startup1049.encode_frame(&mut out)
            .expect("empty creation readiness frame");
        out.freeze()
    }

    pub fn push(&mut self, bytes: &[u8]) -> io::Result<Option<Bytes>> {
        if self.pending.len() + bytes.len() > 65536 {
            return Err(invalid("creation scene input exceeds bound"));
        }
        self.pending.extend_from_slice(bytes);
        let mut output = BytesMut::new();
        while !self.pending.is_empty() {
            let mut cursor = &self.pending[..];
            let id = match dynamic::decode_frame(&self.client, &mission::rules(), &mut cursor) {
                Ok((message, _)) => message.id,
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(error) => return Err(error),
            };
            let used = self.pending.len() - cursor.len();
            let frame = self.pending.split_to(used).freeze();
            eprintln!("Creation scene: client message {id} ({used} bytes)");
            match id {
                410 if !self.started => {
                    self.started = true;
                    CreationResponseFrame209026::Startup1050.encode_frame(&mut output)?;
                }
                411 => {
                    if !self.started { return Err(invalid("creation readiness precedes startup")); }
                    self.ready = true;
                }
                415 => {
                    if !self.ready { return Err(invalid("creation request precedes scene readiness")); }
                    if let Some(previous) = &self.created_request {
                        if *previous != frame { return Err(invalid("creation scene already created a different character")); }
                        continue;
                    }
                    let saved = match self.store.create(&frame) {
                        Ok(saved) => saved,
                        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                            // The frozen creation formatter maps 0x19 to a name
                            // already used by this account. Leave the scene ready
                            // for a corrected request; no character was created.
                            CreationResponseFrame209026::ErrorUi1051 {
                                raw_dwords: [0x19, 0, 0, 0],
                            }.encode_frame(&mut output)?;
                            eprintln!("Creation scene: saved-name conflict returned error 1051");
                            continue;
                        }
                        Err(error) => return Err(error),
                    };
                    if saved.summary().len() > RESULT_CONSUMER_BLOB_LIMIT209026 {
                        return Err(invalid("saved summary exceeds creation-result consumer bound"));
                    }
                    CreationResponseFrame209026::Result1048(CreationResult209026 {
                        character_guid: saved.guid(),
                        // These unused/unresolved result fields are explicit
                        // local-profile zeros, not inferred retail values.
                        raw_guid: Guid([0; 16]),
                        name_units: saved.name().encode_utf16().collect(),
                        raw_word: 0, raw_qwords: [0; 2],
                        blob: saved.summary().to_vec(),
                    }).encode_frame(&mut output)?;
                    self.created_request = Some(frame);
                    eprintln!("Creation scene: saved character and sent result 1048");
                }
                430 if !self.redirected => {
                    if self.created_request.is_none() {
                        return Err(invalid("creation transfer requested before a saved character"));
                    }
                    // Source-supported local continuation, pending playback:
                    // 1065 carries destination, unused Dword, join scalar,
                    // map Word, join scalar and unused Dword. Norn379/type9
                    // are sourced; remaining join values preserve research0.
                    1065u16.encode(&mut output);
                    self.world_address.encode(&mut output);
                    0u32.encode(&mut output);
                    9u32.encode(&mut output);
                    379u16.encode(&mut output);
                    0u32.encode(&mut output);
                    0u32.encode(&mut output);
                    self.redirected = true;
                    eprintln!("Creation scene: redirecting saved character to Norn tutorial379");
                }
                _ => {}
            }
        }
        Ok((!output.is_empty()).then(|| output.freeze()))
    }
}
