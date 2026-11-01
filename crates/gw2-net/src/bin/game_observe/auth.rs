//! Explicit synthetic authentication probe, not an account service.
//! Message layouts: docs/re/209026-auth-messages.md and
//! docs/re/209026-character-protocol.md.
use bytes::{Bytes, BytesMut};
use gw2_proto::{
    Encode, Guid,
    character_summary::SummaryV11,
    dynamic::{self, CStringEncoding, CountPrefix, Rules, StringEncoding, Value},
    play_route::{NameSelectComplete3, RouteRequest29, RouteResult20},
    schema::{self, Field, Message},
};
use std::io::{self, ErrorKind};
use std::net::SocketAddr;

pub const CHARACTER_BODY_LIMIT: usize = 238;

/// Explicit local fixtures. No Debug implementation: bodies can contain names.
#[derive(Clone)]
pub struct ReplyFixture {
    body: Bytes,
    character: Option<Bytes>,
}

impl ReplyFixture {
    pub fn new(body: Bytes, character: Option<Bytes>) -> io::Result<Self> {
        if body.is_empty() || body.len() > 65000 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "auth fixture body exceeds bound",
            ));
        }
        if let Some(record) = &character {
            validate_character(record)?;
        }
        Ok(Self { body, character })
    }
}

/// Opt-in second phase for the existing-character Play path
/// (docs/re/209026-play-route.md). Synthetic values only: `address` is a
/// loopback mission endpoint and `character` the value returned by server 8's
/// name-select branch. Result codes are zero; their meaning beyond "success"
/// and server 20's three values and scalar are unresolved.
#[derive(Clone, Copy)]
pub struct PlayRoute {
    address: SocketAddr,
    character: Guid,
}

impl PlayRoute {
    pub fn new(address: SocketAddr, character: Guid) -> io::Result<Self> {
        if !address.ip().is_loopback() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "play-route address must be loopback",
            ));
        }
        Ok(Self { address, character })
    }
}

const AUTH_SCHEMA: &str = include_str!("../../../../../data/protocol/209026/auth.xml");

fn census_rules() -> Rules {
    Rules {
        string: Some(StringEncoding::Utf16Terminated),
        cstring: Some(CStringEncoding::Terminated),
        array_var_small: Some(CountPrefix::U8),
        array_var_large: Some(CountPrefix::U16),
        buffer_var_small: Some(CountPrefix::U8),
        buffer_var_large: Some(CountPrefix::U16),
        buffer_fixed_is_size_bytes: true,
    }
}

/// Server 8 in its name-select form: request, zero first value, the
/// character value in field 2 (consumed by the type-0x15 branch), all other
/// fields zero/empty. Encoded through the schema so the layout cannot drift.
fn name_select_8(server8: &Message, request: u32, character: Guid) -> io::Result<BytesMut> {
    let mut values = Vec::new();
    for (index, f) in server8.fields.iter().enumerate() {
        let value = match (index, f.kind.as_str()) {
            (0, "Dword") => Value::U32(request),
            (2, "Guid") => Value::Guid(character),
            (_, "Dword") => Value::U32(0),
            (_, "Guid") => Value::Guid(Guid([0; 16])),
            (_, "Byte") => Value::U8(0),
            (_, "BufferFixed") => Value::Bytes(vec![0; f.size.unwrap_or(0) as usize]),
            (_, "ArrayVarSmall") => Value::Array(vec![]),
            _ => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "unexpected server 8 field kind",
                ));
            }
        };
        values.push((f.name.clone(), value));
    }
    let mut out = BytesMut::new();
    8u16.encode(&mut out);
    dynamic::encode_message(server8, &census_rules(), &Value::Struct(values), &mut out)?;
    Ok(out)
}

fn validate_character(bytes: &[u8]) -> io::Result<()> {
    let invalid = || io::Error::new(ErrorKind::InvalidData, "invalid character record fixture");
    if bytes.len() > CHARACTER_BODY_LIMIT {
        return Err(invalid());
    }
    let message = Message {
        id: 17,
        name: String::new(),
        addr: None,
        fields: vec![
            field("Guid", None),
            field("Dword", None),
            field("Qword", None),
            field("Qword", None),
            field("String", Some(20)),
            field("BufferVarLarge", Some(512)),
        ],
    };
    let rules = Rules {
        string: Some(StringEncoding::Utf16Terminated),
        buffer_var_large: Some(CountPrefix::U16),
        ..Rules::default()
    };
    let mut remaining = bytes;
    let Value::Struct(fields) =
        dynamic::decode_message(&message, &rules, &mut remaining).map_err(|_| invalid())?
    else {
        return Err(invalid());
    };
    let Value::Text(name) = &fields[4].1 else {
        return Err(invalid());
    };
    let Value::Bytes(summary) = &fields[5].1 else {
        return Err(invalid());
    };
    if !remaining.is_empty()
        || name.is_empty()
        || !SummaryV11::parse(summary)
            .map_err(|_| invalid())?
            .passes_preview_sex_check()
    {
        return Err(invalid());
    }
    Ok(())
}

fn field(kind: &str, size: Option<u32>) -> Field {
    Field {
        kind: kind.into(),
        name: String::new(),
        size,
        type_name: None,
        fields: vec![],
    }
}

fn messages() -> Vec<Message> {
    [
        (1, vec![field("Dword", None)]),
        (
            2,
            vec![field("String", Some(32)), field("String", Some(32))],
        ),
        (
            3,
            vec![field("Dword", None), field("BufferFixed", Some(16))],
        ),
        (
            10,
            vec![
                field("Dword", None),
                field("Guid", None),
                field("Guid", None),
                field("String", Some(64)),
                field("String", Some(28)),
            ],
        ),
    ]
    .into_iter()
    .map(|(id, fields)| Message {
        id,
        name: id.to_string(),
        addr: None,
        fields,
    })
    .collect()
}

pub struct AuthProbe {
    pending: BytesMut,
    messages: Vec<Message>,
    fixture: ReplyFixture,
    replied: bool,
    play: Option<PlayPhase>,
}

struct PlayPhase {
    route: PlayRoute,
    client: Vec<Message>,
    server8: Message,
}

impl AuthProbe {
    pub fn new(fixture: ReplyFixture) -> Self {
        Self {
            pending: BytesMut::new(),
            messages: messages(),
            fixture,
            replied: false,
            play: None,
        }
    }

    /// Enable the opt-in Play phase after the login reply.
    pub fn with_play_route(mut self, route: PlayRoute) -> io::Result<Self> {
        let invalid = |_| io::Error::new(ErrorKind::InvalidData, "invalid embedded auth schema");
        let parsed = schema::parse(AUTH_SCHEMA).map_err(invalid)?;
        let group = parsed
            .protocols
            .into_iter()
            .find(|p| p.name == "3")
            .and_then(|p| p.groups.into_iter().next())
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "auth schema lacks protocol 3"))?;
        let server8 = group
            .server
            .iter()
            .find(|m| m.id == 8)
            .cloned()
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "auth schema lacks server 8"))?;
        self.play = Some(PlayPhase {
            route,
            client: group.client,
            server8,
        });
        Ok(self)
    }

    /// Second phase: answer client 22 with server 8 + 3 and client 29 with
    /// server 20. An undecodable frame disables the phase instead of aborting
    /// the capture.
    fn push_play(&mut self, bytes: &[u8]) -> io::Result<Option<Bytes>> {
        let Some(play) = &self.play else {
            return Ok(None);
        };
        if self.pending.len() + bytes.len() > 65536 {
            self.play = None;
            eprintln!("Play phase disabled: input exceeds bound");
            return Ok(None);
        }
        self.pending.extend_from_slice(bytes);
        let rules = census_rules();
        let mut reply = BytesMut::new();
        while !self.pending.is_empty() {
            let mut cursor = &self.pending[..];
            let id = match dynamic::decode_frame(&play.client, &rules, &mut cursor) {
                Ok((message, _)) => message.id,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => {
                    eprintln!("Play phase disabled: {e}");
                    self.play = None;
                    self.pending.clear();
                    break;
                }
            };
            let used = self.pending.len() - cursor.len();
            let frame = self.pending.split_to(used);
            eprintln!("Play phase parsed client message {id} ({used} bytes)");
            match id {
                22 => {
                    let mut body = &frame[2..];
                    let request = <u32 as gw2_proto::Decode>::decode(&mut body)?;
                    reply.extend_from_slice(&name_select_8(&play.server8, request, play.route.character)?);
                    NameSelectComplete3 {
                        request,
                        scalar: 0,
                        address: play.route.address,
                        result: [0; 4],
                    }
                    .encode_frame(&mut reply);
                }
                29 => {
                    let mut input = &frame[..];
                    let route = RouteRequest29::decode_frame(&mut input)?;
                    RouteResult20 {
                        request: route.request,
                        values: [0; 3],
                        address: play.route.address,
                        scalar: 0,
                    }
                    .encode_frame(&mut reply);
                }
                _ => {}
            }
        }
        Ok((!reply.is_empty()).then(|| reply.freeze()))
    }

    /// Parse only through the first complete ID 10. Incomplete reads retain all
    /// bytes; future messages remain capture-only after the explicit reply.
    pub fn push(&mut self, bytes: &[u8]) -> io::Result<Option<Bytes>> {
        if self.replied {
            return self.push_play(bytes);
        }
        if self.pending.len() + bytes.len() > 65536 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "auth probe input exceeds bound",
            ));
        }
        self.pending.extend_from_slice(bytes);
        let rules = Rules {
            string: Some(StringEncoding::Utf16Terminated),
            buffer_fixed_is_size_bytes: true,
            ..Rules::default()
        };
        while !self.pending.is_empty() {
            let mut cursor = &self.pending[..];
            let (message, value) = match dynamic::decode_frame(&self.messages, &rules, &mut cursor)
            {
                Ok(result) => result,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
                Err(e) => return Err(e),
            };
            let used = self.pending.len() - cursor.len();
            let id = message.id;
            let _ = self.pending.split_to(used);
            eprintln!("Auth probe parsed client message {id} ({used} bytes)");
            if id == 10 {
                let Value::Struct(fields) = value else {
                    unreachable!()
                };
                let Value::U32(request) = fields[0].1 else {
                    unreachable!()
                };
                let mut reply = BytesMut::new();
                8u16.encode(&mut reply);
                request.encode(&mut reply);
                reply.extend_from_slice(&self.fixture.body);
                // Configuration must arrive while this request is still pending.
                // Explicit empty fixture; no recovered account configuration yet.
                10u16.encode(&mut reply);
                request.encode(&mut reply);
                reply.extend_from_slice(&0u16.to_le_bytes());
                if let Some(record) = &self.fixture.character {
                    17u16.encode(&mut reply);
                    request.encode(&mut reply);
                    reply.extend_from_slice(record);
                }
                7u16.encode(&mut reply);
                request.encode(&mut reply);
                reply.extend_from_slice(&[0; 6]); // Two result fields, four status fields.
                self.replied = true;
                self.pending.clear();
                return Ok(Some(reply.freeze()));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gw2_proto::{Decode, Guid};

    fn character_record(name: &str, sex: u8) -> Bytes {
        let mut record = BytesMut::new();
        Guid([0x55; 16]).encode(&mut record);
        300u32.encode(&mut record);
        0x1234_5678_9abc_def0u64.encode(&mut record);
        0xfedc_ba98_7654_3210u64.encode(&mut record);
        for unit in name.encode_utf16().chain(std::iter::once(0)) {
            unit.encode(&mut record);
        }
        // A structural fixture only; no content-valid map/appearance claim.
        let mut summary = [0u8; 159];
        summary[0] = 11;
        summary[4] = 1;
        summary[5] = 1;
        summary[0x23] = sex;
        159u16.encode(&mut record);
        record.extend_from_slice(&summary);
        record.freeze()
    }

    #[test]
    fn character_fixture_rejects_truncation_trailing_data_and_bad_summary() {
        let record = character_record("🦀🦀🦀🦀🦀🦀🦀🦀abc", 1); // 19 UTF-16 units.
        validate_character(&record).unwrap();
        let mut maximum = record.to_vec();
        maximum.splice(16..18, [0xff, 0xff, 0xff, 0xff, 0x0f]);
        assert_eq!(maximum.len(), CHARACTER_BODY_LIMIT);
        validate_character(&maximum).unwrap();
        for cut in 0..record.len() {
            assert!(validate_character(&record[..cut]).is_err(), "cut {cut}");
        }
        let mut trailing = record.to_vec();
        trailing.push(0);
        assert!(validate_character(&trailing).is_err());
        assert!(validate_character(&character_record("", 0)).is_err());
        assert!(validate_character(&character_record("🦀🦀🦀🦀🦀🦀🦀🦀abcd", 0)).is_err());
        assert!(validate_character(&character_record("Fixture", 2)).is_err());
        let mut bad_version = record.to_vec();
        let start = bad_version.len() - 159;
        bad_version[start] = 10;
        assert!(validate_character(&bad_version).is_err());
        assert!(validate_character(&vec![0; CHARACTER_BODY_LIMIT + 1]).is_err());
    }

    #[test]
    fn character_record_is_correlated_and_precedes_completion_for_every_split() {
        let record = character_record("Fixture", 0);
        let body = Bytes::from_static(&[0x44; 2]);
        let mut input = BytesMut::new();
        10u16.encode(&mut input);
        123456u32.encode(&mut input);
        Guid([0x11; 16]).encode(&mut input);
        Guid([0x33; 16]).encode(&mut input);
        input.extend_from_slice(&[0; 4]);
        let mut expected = BytesMut::new();
        8u16.encode(&mut expected);
        123456u32.encode(&mut expected);
        expected.extend_from_slice(&body);
        10u16.encode(&mut expected);
        123456u32.encode(&mut expected);
        expected.extend_from_slice(&[0; 2]);
        17u16.encode(&mut expected);
        123456u32.encode(&mut expected);
        expected.extend_from_slice(&record);
        7u16.encode(&mut expected);
        123456u32.encode(&mut expected);
        expected.extend_from_slice(&[0; 6]);
        for cut in 0..input.len() {
            let fixture = ReplyFixture::new(body.clone(), Some(record.clone())).unwrap();
            let mut probe = AuthProbe::new(fixture);
            assert!(probe.push(&input[..cut]).unwrap().is_none());
            assert_eq!(probe.push(&input[cut..]).unwrap().unwrap(), expected);
            assert!(probe.push(&input).unwrap().is_none());
        }
    }

    #[test]
    fn fragmented_auth_request_replies_once_with_matching_request_id() {
        let mut input = BytesMut::new();
        2u16.encode(&mut input);
        input.extend_from_slice(&[65, 0, 0, 0, 66, 0, 0, 0]);
        3u16.encode(&mut input);
        300u32.encode(&mut input);
        input.extend_from_slice(&[0x22; 16]);
        1u16.encode(&mut input);
        42u32.encode(&mut input);
        10u16.encode(&mut input);
        123456u32.encode(&mut input);
        Guid([0x11; 16]).encode(&mut input);
        Guid([0x33; 16]).encode(&mut input);
        input.extend_from_slice(&[0, 0, 0, 0]);
        for cut in 0..input.len() {
            let mut probe =
                AuthProbe::new(ReplyFixture::new(Bytes::from_static(&[0x44; 2]), None).unwrap());
            assert!(probe.push(&input[..cut]).unwrap().is_none());
            let reply = probe.push(&input[cut..]).unwrap().unwrap();
            let mut cursor = &reply[..];
            assert_eq!(u16::decode(&mut cursor).unwrap(), 8);
            assert_eq!(u32::decode(&mut cursor).unwrap(), 123456);
            assert_eq!(&cursor[..2], &[0x44; 2]);
            cursor = &cursor[2..];
            assert_eq!(u16::decode(&mut cursor).unwrap(), 10);
            assert_eq!(u32::decode(&mut cursor).unwrap(), 123456);
            assert_eq!(&cursor[..2], &[0, 0]);
            cursor = &cursor[2..];
            assert_eq!(u16::decode(&mut cursor).unwrap(), 7);
            assert_eq!(u32::decode(&mut cursor).unwrap(), 123456);
            assert_eq!(cursor, &[0; 6]);
            assert!(probe.push(&input).unwrap().is_none());
        }
    }

    fn login_input() -> BytesMut {
        let mut input = BytesMut::new();
        10u16.encode(&mut input);
        123456u32.encode(&mut input);
        Guid([0x11; 16]).encode(&mut input);
        Guid([0x33; 16]).encode(&mut input);
        input.extend_from_slice(&[0; 4]);
        input
    }

    fn play_input() -> BytesMut {
        let mut input = BytesMut::new();
        22u16.encode(&mut input);
        500u32.encode(&mut input);
        for unit in "Fixture".encode_utf16().chain(std::iter::once(0)) {
            unit.encode(&mut input);
        }
        RouteRequest29 {
            request: 501,
            game_type: 4,
            reserved: 0,
            map_key: 15,
            argument: 0,
            hash: 0xdead_beef,
            context: Guid([0x5a; 16]),
        }
        .encode_frame(&mut input);
        input
    }

    fn play_probe() -> AuthProbe {
        let fixture = ReplyFixture::new(Bytes::from_static(&[0x44; 2]), None).unwrap();
        let route = PlayRoute::new("127.1.0.3:6112".parse().unwrap(), Guid([0x55; 16])).unwrap();
        let mut probe = AuthProbe::new(fixture).with_play_route(route).unwrap();
        assert!(probe.push(&login_input()).unwrap().is_some());
        probe
    }

    #[test]
    fn play_phase_answers_22_and_29_for_every_split() {
        let input = play_input();
        let server = schema::parse(AUTH_SCHEMA).unwrap().protocols[0].groups[0].server.clone();
        for cut in 0..=input.len() {
            let mut probe = play_probe();
            let mut replies = BytesMut::new();
            for part in [&input[..cut], &input[cut..]] {
                if let Some(reply) = probe.push(part).unwrap() {
                    replies.extend_from_slice(&reply);
                }
            }
            let mut cursor = &replies[..];
            let (m8, v8) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
            assert_eq!(m8.id, 8, "cut {cut}");
            let Value::Struct(f8) = v8 else { panic!() };
            assert_eq!(f8[0].1, Value::U32(500));
            assert_eq!(f8[2].1, Value::Guid(Guid([0x55; 16])));
            let (m3, v3) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
            assert_eq!(m3.id, 3);
            let Value::Struct(f3) = v3 else { panic!() };
            assert_eq!(f3[0].1, Value::U32(500));
            assert_eq!(f3[2].1, Value::Address("127.1.0.3:6112".parse().unwrap()));
            let (m20, v20) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
            assert_eq!(m20.id, 20);
            let Value::Struct(f20) = v20 else { panic!() };
            assert_eq!(f20[0].1, Value::U32(501));
            assert_eq!(f20[4].1, Value::Address("127.1.0.3:6112".parse().unwrap()));
            assert!(cursor.is_empty(), "cut {cut}");
        }
    }

    #[test]
    fn play_phase_disables_on_undecodable_frame_and_is_opt_in() {
        let mut probe = play_probe();
        assert!(probe.push(&[0xe7, 0x03, 0]).unwrap().is_none()); // ID 999
        assert!(probe.push(&play_input()).unwrap().is_none());

        let fixture = ReplyFixture::new(Bytes::from_static(&[0x44; 2]), None).unwrap();
        let mut plain = AuthProbe::new(fixture);
        assert!(plain.push(&login_input()).unwrap().is_some());
        assert!(plain.push(&play_input()).unwrap().is_none());
        assert!(PlayRoute::new("10.0.0.1:6112".parse().unwrap(), Guid([0; 16])).is_err());
    }
}
