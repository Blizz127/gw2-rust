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
use std::io::{self, ErrorKind, Write};
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::character_store::CharacterStore;

pub const CHARACTER_BODY_LIMIT: usize = 238;

/// Explicit local fixtures. No Debug implementation: bodies can contain names.
#[derive(Clone)]
pub struct ReplyFixture {
    body: Bytes,
    character: Option<Bytes>,
    character_store: Option<CharacterStore>,
    agreement: Option<AgreementState>,
}

/// The one account's accepted User Agreement version (owner decision 2026-11-04: the
/// server remembers it so the client stops asking every login). Client 15 {Byte} carries
/// the version the player accepted (`0x140241690`, docs/re/209026-home-world.md); the
/// login reply's server 8 Dword4 low byte is the account's accepted version (handler
/// `0x140240850` passes it to record +0x1e8), and a match with the client's current
/// 'eula' content version (`0x1413e6420`) hides the panel. One byte in a 0600 file.
#[derive(Clone)]
pub struct AgreementState {
    path: PathBuf,
}

impl AgreementState {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// The stored version, if a valid one-byte record exists.
    pub fn load(&self) -> Option<u8> {
        match std::fs::read(&self.path) {
            Ok(bytes) if bytes.len() == 1 => Some(bytes[0]),
            _ => None,
        }
    }

    /// Replace the stored version (temporary file, then rename).
    pub fn store(&self, version: u8) -> io::Result<()> {
        use std::os::unix::fs::OpenOptionsExt;
        let tmp = self.path.with_extension("tmp");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)?;
        file.write_all(&[version])?;
        file.sync_all()?;
        std::fs::rename(&tmp, &self.path)
    }
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
        Ok(Self { body, character, character_store: None, agreement: None })
    }

    /// List and select saved characters for the explicitly configured local account.
    pub fn with_character_store(mut self, store: CharacterStore) -> io::Result<Self> {
        if self.character.is_some() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "saved characters and a static character fixture are mutually exclusive",
            ));
        }
        self.character_store = Some(store);
        Ok(self)
    }

    /// Remember the accepted User Agreement version in `state` (see [`AgreementState`]).
    pub fn with_agreement(mut self, state: AgreementState) -> Self {
        self.agreement = Some(state);
        self
    }

    /// The login body as sent now: with a stored agreement version, Dword4 carries it.
    fn login_body(&self) -> io::Result<Bytes> {
        let Some(version) = self.agreement.as_ref().and_then(AgreementState::load) else {
            return Ok(self.body.clone());
        };
        edit_login_body(&self.body, |fields| match &mut fields[4].1 {
            Value::U32(accepted) => {
                *accepted = u32::from(version);
                Ok(())
            }
            _ => Err(io::Error::new(ErrorKind::InvalidData, "server 8 Dword4 is not a Dword")),
        })
    }

    /// Set server 8's Dword6 (home world) and Dword7 (current world) of the login
    /// body to [`HOME_WORLD`] where they are zero (docs/re/209026-home-world.md:
    /// `0x14023a770` stores Dword7 at record +0x1a4 and Dword6 at +0x1a8, which
    /// `0x14023d96d`-`0x14023d986` copy to the current and home world). The server
    /// keeps no account state, so without this the client starts every session with
    /// no world: world "Unknown" and PLAY greyed until the Initial Home World Select
    /// dialog is confirmed (owner, 2026-11-04 01:51Z). The body is decoded and
    /// re-encoded through the auth schema, so it must be a complete server 8 body.
    pub fn with_home_world(mut self) -> io::Result<Self> {
        self.body = edit_login_body(&self.body, |fields| {
            for index in [6, 7] {
                match &mut fields[index].1 {
                    Value::U32(world) if *world == 0 => *world = HOME_WORLD,
                    Value::U32(_) => {}
                    _ => return Err(io::Error::new(ErrorKind::InvalidData, "server 8 world is not a Dword")),
                }
            }
            Ok(())
        })?;
        Ok(self)
    }
}

/// Decode a login body (server 8 after its request Dword) through the auth schema, edit
/// its fields and re-encode it. Every other byte is kept.
fn edit_login_body(
    body: &[u8],
    edit: impl FnOnce(&mut [(String, Value)]) -> io::Result<()>,
) -> io::Result<Bytes> {
    let invalid = || io::Error::new(ErrorKind::InvalidData, "auth body is not a server 8 body");
    let server8 = auth_server8()?;
    // The body follows the request Dword; frame it with request 0 (one varint byte).
    let mut framed = BytesMut::new();
    0u32.encode(&mut framed);
    framed.extend_from_slice(body);
    let mut cursor = &framed[..];
    let Value::Struct(mut fields) =
        dynamic::decode_message(&server8, &census_rules(), &mut cursor).map_err(|_| invalid())?
    else {
        return Err(invalid());
    };
    if !cursor.is_empty() || fields.len() != server8.fields.len() {
        return Err(invalid());
    }
    edit(&mut fields)?;
    let mut out = BytesMut::new();
    dynamic::encode_message(&server8, &census_rules(), &Value::Struct(fields), &mut out)?;
    Ok(out.freeze().slice(1..))
}

/// Server 8 as the embedded auth schema (protocol 3) defines it.
fn auth_server8() -> io::Result<Message> {
    let invalid = |_| io::Error::new(ErrorKind::InvalidData, "invalid embedded auth schema");
    schema::parse(AUTH_SCHEMA)
        .map_err(invalid)?
        .protocols
        .into_iter()
        .find(|p| p.name == "3")
        .and_then(|p| p.groups.into_iter().next())
        .and_then(|g| g.server.into_iter().find(|m| m.id == 8))
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "auth schema lacks server 8"))
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
    creation_address: Option<SocketAddr>,
}

impl PlayRoute {
    pub fn new(address: SocketAddr, character: Guid) -> io::Result<Self> {
        if !address.ip().is_loopback() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "play-route address must be loopback",
            ));
        }
        Ok(Self {
            address,
            character,
            creation_address: None,
        })
    }

    /// Route game type 1 (character creation) to a separate local listener.
    pub fn with_creation_address(mut self, address: SocketAddr) -> io::Result<Self> {
        if !address.ip().is_loopback() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "creation-route address must be loopback",
            ));
        }
        self.creation_address = Some(address);
        Ok(self)
    }
}

const AUTH_SCHEMA: &str = include_str!("../../../../../data/protocol/209026/auth.xml");

/// The one offered home world: 1001 is present in the client's world content (type 373).
const HOME_WORLD: u32 = 1001;

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
    character_parts(bytes).map(|_| ())
}

/// Parse a server-17 character record body (see `validate_character`) into
/// its name and summary blob. Values are never logged.
pub fn character_parts(bytes: &[u8]) -> io::Result<(String, Vec<u8>)> {
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
    Ok((name.clone(), summary.clone()))
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

    /// Whether the opt-in Play phase is still active.
    pub fn play_enabled(&self) -> bool {
        self.play.is_some()
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

    /// Second phase: answer client 22 with server 8 + 3, client 25 with
    /// saved records + completion, and client 29 with server 20. An undecodable
    /// frame disables the phase instead of aborting the capture.
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
            let (id, value) = match dynamic::decode_frame(&play.client, &rules, &mut cursor) {
                Ok((message, value)) => (message.id, value),
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
                    // The client schema bounds this name to String(20), including
                    // its terminator. Keep the decoded name out of diagnostics.
                    let Value::Struct(fields) = value else { unreachable!() };
                    let Value::U32(request) = fields[0].1 else { unreachable!() };
                    let Value::Text(name) = &fields[1].1 else { unreachable!() };
                    let character = if let Some(store) = &self.fixture.character_store {
                        store.find_name(name)?.ok_or_else(|| {
                            io::Error::new(ErrorKind::NotFound, "selected character is not saved")
                        })?.guid()
                    } else {
                        play.route.character
                    };
                    reply.extend_from_slice(&name_select_8(&play.server8, request, character)?);
                    NameSelectComplete3 {
                        request,
                        scalar: 0,
                        address: play.route.address,
                        result: [0; 4],
                    }
                    .encode_frame(&mut reply);
                }
                // Client 14 {request, world id, byte}: the home-world selection. The
                // client waits for server 7 echoing the request id; status word (D3) 0 is
                // success, after which it sets its current and home world to the choice.
                // Client 15 {Byte}: the User Agreement version the player accepted.
                // No reply; with --auth-agreement-state it is remembered.
                15 => {
                    let version = frame[2];
                    eprintln!("Play phase: User Agreement version {version} accepted");
                    if let Some(state) = &self.fixture.agreement
                        && let Err(e) = state.store(version)
                    {
                        eprintln!("Play phase: could not store the agreement version: {e}");
                    }
                }
                // Client 12 {request, buffer}: the account-preferences upload
                // (PrContext.cpp 0x14027da93 -> 0x14023bcc0, transaction vtable
                // 0x1419236f8). Server 7 is the generic completion: 0x14023a9e0 finds
                // any pending transaction by request id, marks it done and stores the
                // status words (0 = success). Nothing is stored server-side.
                12 => {
                    let mut body = &frame[2..];
                    let request = <u32 as gw2_proto::Decode>::decode(&mut body)?;
                    eprintln!("Play phase: preferences upload acknowledged");
                    7u16.encode(&mut reply);
                    for value in [request, 0, 0, 0, 0, 0, 0] {
                        value.encode(&mut reply);
                    }
                }
                14 => {
                    let mut body = &frame[2..];
                    let request = <u32 as gw2_proto::Decode>::decode(&mut body)?;
                    let world = <u32 as gw2_proto::Decode>::decode(&mut body)?;
                    eprintln!("Play phase: home world {world} selected; confirming");
                    7u16.encode(&mut reply);
                    for value in [request, 0, 0, 0, 0, 0, 0] {
                        value.encode(&mut reply);
                    }
                }
                25 if self.fixture.character_store.is_some() => {
                    // Client 25 starts type-0x18 list refresh. Server 17 accepts
                    // its request while pending; server 4 completes that request.
                    // docs/re/209026-character-protocol.md, "Explicit character-list refresh".
                    let Value::Struct(fields) = value else { unreachable!() };
                    let Value::U32(request) = fields[0].1 else { unreachable!() };
                    let store = self.fixture.character_store.as_ref().unwrap();
                    for record in store.list()? {
                        17u16.encode(&mut reply);
                        request.encode(&mut reply);
                        reply.extend_from_slice(&record.auth_body());
                    }
                    4u16.encode(&mut reply);
                    for value in [request, 0, 0, 0, 0] {
                        value.encode(&mut reply);
                    }
                }
                29 => {
                    let mut input = &frame[..];
                    let route = RouteRequest29::decode_frame(&mut input)?;
                    // Server 20's fields 1-3 overwrite the transaction slots
                    // client 29 filled (game type, 0, map key); echo them
                    // rather than clearing the map key. Inferred, not verified.
                    RouteResult20 {
                        request: route.request,
                        values: [route.game_type, route.reserved, route.map_key],
                        address: if route.game_type == 1 {
                            play.route.creation_address.unwrap_or(play.route.address)
                        } else {
                            play.route.address
                        },
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
                reply.extend_from_slice(&self.fixture.login_body()?);
                // The home-world list (docs/re/209026-home-world.md): server 23 clears the
                // client's staging list, each server 25 adds {world id, population 0-3,
                // transfer cost?, hours?}, server 24 publishes it. The Initial Home World
                // Select grid is filled once, when it is created, so the list must arrive
                // with the login reply. World 1001 exists in content type 373.
                23u16.encode(&mut reply);
                25u16.encode(&mut reply);
                for value in [HOME_WORLD, 0, 0, 0] {
                    value.encode(&mut reply);
                }
                24u16.encode(&mut reply);
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
                if let Some(store) = &self.fixture.character_store {
                    for record in store.list()? {
                        17u16.encode(&mut reply);
                        request.encode(&mut reply);
                        reply.extend_from_slice(&record.auth_body());
                    }
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
pub(crate) mod tests {
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
        world_list(&mut expected);
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
            let mut worlds = BytesMut::new();
            world_list(&mut worlds);
            assert_eq!(&cursor[..worlds.len()], &worlds[..], "home-world list follows server 8");
            cursor = &cursor[worlds.len()..];
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

    /// Decode a login body (after the request Dword) under the client's own auth
    /// server table and return its Dword fields 3-9.
    fn login_body_dwords(body: &[u8]) -> Vec<u32> {
        const SERVER_TABLES: &str = include_str!("../../../../../data/protocol/209026/server-tables.xml");
        let tables = schema::parse(SERVER_TABLES).unwrap().protocols[0].groups.clone();
        let table = tables.iter().find(|g| g.name == "table-0x141923fe0").unwrap();
        let server8 = table.server.iter().find(|m| m.id == 8).unwrap();
        let mut framed = BytesMut::new();
        77u32.encode(&mut framed);
        framed.extend_from_slice(body);
        let mut cursor = &framed[..];
        let Value::Struct(fields) = dynamic::decode_message(server8, &census_rules(), &mut cursor).unwrap() else {
            unreachable!()
        };
        assert!(cursor.is_empty(), "the body keeps its exact length");
        assert_eq!(fields[0].1, Value::U32(77));
        (3..=9)
            .map(|i| match fields[i].1 {
                Value::U32(v) => v,
                _ => panic!("field {i} is not a Dword"),
            })
            .collect()
    }

    /// The login body carries home world (Dword6) and current world (Dword7) 1001, so
    /// PLAY doesn't depend on the Initial Home World Select dialog; every other field
    /// is kept byte for byte, and a nonzero world is left alone.
    #[test]
    fn login_body_carries_home_and_current_world() {
        let server8 = auth_server8().unwrap();
        let body_with = |worlds: [u32; 2]| {
            let mut values = Vec::new();
            for (index, f) in server8.fields.iter().enumerate() {
                let value = match (index, f.kind.as_str()) {
                    (6, _) => Value::U32(worlds[0]),
                    (7, _) => Value::U32(worlds[1]),
                    (3, _) => Value::U32(2),
                    (_, "Dword") => Value::U32(index as u32 * 300),
                    (_, "Guid") => Value::Guid(Guid([index as u8; 16])),
                    (_, "Byte") => Value::U8(1),
                    (_, "BufferFixed") => Value::Bytes(vec![0x5a; f.size.unwrap() as usize]),
                    (_, "ArrayVarSmall") => Value::Array(vec![]),
                    _ => unreachable!(),
                };
                values.push((f.name.clone(), value));
            }
            let mut out = BytesMut::new();
            dynamic::encode_message(&server8, &census_rules(), &Value::Struct(values), &mut out).unwrap();
            out.freeze().slice(1..) // request 0
        };
        let fixture = ReplyFixture::new(body_with([0, 0]), None).unwrap().with_home_world().unwrap();
        assert_eq!(fixture.body, body_with([HOME_WORLD, HOME_WORLD]));
        assert_eq!(login_body_dwords(&fixture.body), [2, 1200, 1500, HOME_WORLD, HOME_WORLD, 2400, 2700]);
        let kept = ReplyFixture::new(body_with([1002, 0]), None).unwrap().with_home_world().unwrap();
        assert_eq!(kept.body, body_with([1002, HOME_WORLD]));
        assert!(ReplyFixture::new(Bytes::from_static(&[0x44; 2]), None).unwrap().with_home_world().is_err());
    }

    /// Selecting a home world (client 14 {request, world, byte}) is confirmed by server 7
    /// echoing the request with status 0, decoded under the client's own auth table.
    #[test]
    fn home_world_selection_is_confirmed_with_server_7() {
        const SERVER_TABLES: &str = include_str!("../../../../../data/protocol/209026/server-tables.xml");
        let tables = schema::parse(SERVER_TABLES).unwrap().protocols[0].groups.clone();
        let table = tables.iter().find(|g| g.name == "table-0x141923fe0").unwrap();
        let mut select = BytesMut::new();
        14u16.encode(&mut select);
        77u32.encode(&mut select);
        HOME_WORLD.encode(&mut select);
        select.extend_from_slice(&[0]);
        for cut in 0..=select.len() {
            let mut probe = play_probe();
            let mut replies = Vec::new();
            for part in [&select[..cut], &select[cut..]] {
                if let Some(reply) = probe.push(part).unwrap() {
                    replies.push(reply);
                }
            }
            assert_eq!(replies.len(), 1, "cut {cut}");
            let mut cursor = &replies[0][..];
            let (message, value) = dynamic::decode_frame(&table.server, &census_rules(), &mut cursor).unwrap();
            assert_eq!(message.id, 7);
            assert!(cursor.is_empty());
            let Value::Struct(fields) = value else { panic!() };
            assert_eq!(fields[0].1, Value::U32(77), "request echoed");
            assert_eq!(fields[3].1, Value::U32(0), "status 0 = success");
            assert!(probe.play_enabled());
        }
    }

    /// Client 15 stores the accepted agreement version; the next login's server 8 then
    /// carries it in Dword4 (record +0x1e8), with every other field unchanged. A missing
    /// or malformed state file leaves the body as configured.
    #[test]
    fn accepted_agreement_version_is_remembered_and_echoed_at_login() {
        let dir = std::env::temp_dir().join(format!("gw2-agreement-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let state = AgreementState::new(dir.join("agreement-version"));
        let _ = std::fs::remove_file(dir.join("agreement-version"));
        let server8 = auth_server8().unwrap();
        let mut values = Vec::new();
        for f in &server8.fields {
            values.push((f.name.clone(), match f.kind.as_str() {
                "Dword" => Value::U32(0),
                "Guid" => Value::Guid(Guid([3; 16])),
                "Byte" => Value::U8(0),
                "BufferFixed" => Value::Bytes(vec![0; f.size.unwrap() as usize]),
                "ArrayVarSmall" => Value::Array(vec![]),
                _ => unreachable!(),
            }));
        }
        let mut out = BytesMut::new();
        dynamic::encode_message(&server8, &census_rules(), &Value::Struct(values), &mut out).unwrap();
        let body = out.freeze().slice(1..);
        let fixture = ReplyFixture::new(body.clone(), None).unwrap().with_agreement(state.clone());
        assert_eq!(fixture.login_body().unwrap(), body, "nothing stored yet");
        std::fs::write(dir.join("agreement-version"), b"xy").unwrap();
        assert_eq!(fixture.login_body().unwrap(), body, "malformed state is ignored");

        let route = PlayRoute::new("127.1.0.4:6112".parse().unwrap(), Guid([0x55; 16])).unwrap();
        let mut probe = AuthProbe::new(fixture.clone()).with_play_route(route).unwrap();
        probe.push(&login_input()).unwrap().unwrap();
        let mut accept = BytesMut::new();
        15u16.encode(&mut accept);
        accept.extend_from_slice(&[7]);
        assert!(probe.push(&accept).unwrap().is_none(), "client 15 gets no reply");
        assert!(probe.play_enabled());
        assert_eq!(state.load(), Some(7));
        assert_eq!(login_body_dwords(&fixture.login_body().unwrap()), [0, 7, 0, 0, 0, 0, 0]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The preferences upload (client 12 {request, BufferVarLarge}, 841 bytes on the
    /// owner's device) is completed by server 7 echoing its request with status 0.
    #[test]
    fn preferences_upload_is_completed_with_server_7() {
        const SERVER_TABLES: &str = include_str!("../../../../../data/protocol/209026/server-tables.xml");
        let tables = schema::parse(SERVER_TABLES).unwrap().protocols[0].groups.clone();
        let table = tables.iter().find(|g| g.name == "table-0x141923fe0").unwrap();
        let mut upload = BytesMut::new();
        12u16.encode(&mut upload);
        91u32.encode(&mut upload);
        upload.extend_from_slice(&835u16.to_le_bytes());
        upload.extend_from_slice(&[0x5a; 835]);
        for cut in [0, 1, 3, 6, 400, upload.len()] {
            let mut probe = play_probe();
            let mut replies = Vec::new();
            for part in [&upload[..cut], &upload[cut..]] {
                if let Some(reply) = probe.push(part).unwrap() {
                    replies.push(reply);
                }
            }
            assert_eq!(replies.len(), 1, "cut {cut}");
            let mut cursor = &replies[0][..];
            let (message, value) = dynamic::decode_frame(&table.server, &census_rules(), &mut cursor).unwrap();
            assert_eq!(message.id, 7);
            assert!(cursor.is_empty());
            let Value::Struct(fields) = value else { panic!() };
            assert_eq!(fields[0].1, Value::U32(91), "request echoed");
            assert_eq!(fields[3].1, Value::U32(0), "status 0 = success");
            assert!(probe.play_enabled());
        }
    }

    /// The home-world list the login reply carries: 23, 25 {1001, 0, 0, 0}, 24.
    pub(crate) fn world_list(out: &mut BytesMut) {
        23u16.encode(out);
        25u16.encode(out);
        for value in [HOME_WORLD, 0, 0, 0] {
            value.encode(out);
        }
        24u16.encode(out);
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

    fn saved_store() -> (PathBuf, CharacterStore) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "gw2-auth-saved-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        let store = CharacterStore::open(path.clone()).unwrap();
        (path, store)
    }

    fn saved_fixture(store: CharacterStore) -> ReplyFixture {
        let frame = name_select_8(&auth_server8().unwrap(), 0, Guid([0; 16])).unwrap();
        ReplyFixture::new(frame.freeze().slice(3..), None)
            .unwrap()
            .with_character_store(store)
            .unwrap()
    }

    fn name_request(request: u32, name: &str) -> BytesMut {
        let mut input = BytesMut::new();
        22u16.encode(&mut input);
        request.encode(&mut input);
        for unit in name.encode_utf16().chain(std::iter::once(0)) {
            unit.encode(&mut input);
        }
        input
    }

    #[test]
    fn saved_characters_are_listed_before_completion_and_survive_relogin() {
        let (path, store) = saved_store();
        // Construct the fixture before saving: each login must read current records.
        let fixture = saved_fixture(store.clone());
        let first = store.create(&crate::character_store::test_request("Saved One")).unwrap();
        let second = store.create(&crate::character_store::test_request("Saved Ω🦀")).unwrap();
        assert_ne!(first.guid(), second.guid());
        let server = schema::parse(AUTH_SCHEMA).unwrap().protocols[0].groups[0].server.clone();
        for fixture in [fixture, saved_fixture(CharacterStore::open(path.clone()).unwrap())] {
            let mut probe = AuthProbe::new(fixture);
            let reply = probe.push(&login_input()).unwrap().unwrap();
            let mut cursor = &reply[..];
            let mut ids = Vec::new();
            let mut records = Vec::new();
            while !cursor.is_empty() {
                let (message, value) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
                ids.push(message.id);
                let Value::Struct(fields) = value else { panic!() };
                if matches!(message.id, 8 | 10 | 17 | 7) {
                    assert_eq!(fields[0].1, Value::U32(123456));
                }
                if message.id == 17 {
                    records.push((fields[1].1.clone(), fields[5].1.clone(), fields[6].1.clone()));
                }
            }
            assert_eq!(ids, [8, 23, 25, 24, 10, 17, 17, 7]);
            let expected: Vec<_> = store.list().unwrap().into_iter().map(|record| (
                Value::Guid(record.guid()),
                Value::Text(record.name().to_owned()),
                Value::Bytes(record.summary().to_vec()),
            )).collect();
            assert_eq!(records, expected);
            assert!(probe.push(&login_input()).unwrap().is_none());
        }
        assert!(ReplyFixture::new(Bytes::from_static(&[0x44]), Some(character_record("Fixture", 0)))
            .unwrap().with_character_store(store).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }


    #[test]
    fn saved_list_refresh_reads_new_records_and_correlates_before_completion() {
        const SERVER_TABLES: &str =
            include_str!("../../../../../data/protocol/209026/server-tables.xml");
        let server = schema::parse(SERVER_TABLES).unwrap().protocols
            .into_iter().flat_map(|protocol| protocol.groups)
            .find(|group| group.name == "table-0x141923fe0").unwrap().server;
        let refresh = |request: u32| {
            let mut input = BytesMut::new();
            25u16.encode(&mut input);
            request.encode(&mut input);
            input
        };
        let input = refresh(123456);
        for cut in 0..=input.len() {
            let (path, store) = saved_store();
            let fixture = saved_fixture(store.clone());
            let route = PlayRoute::new("127.1.0.3:6112".parse().unwrap(), Guid([0x55; 16])).unwrap();
            let mut probe = AuthProbe::new(fixture).with_play_route(route).unwrap();
            probe.push(&login_input()).unwrap().unwrap();

            // An empty account still needs completion to leave the refresh state.
            let reply = probe.push(&refresh(300)).unwrap().unwrap();
            let mut cursor = &reply[..];
            let (message, value) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
            assert_eq!(message.id, 4);
            let Value::Struct(fields) = value else { panic!() };
            assert_eq!(fields[0].1, Value::U32(300));
            assert!(fields[1..].iter().all(|(_, value)| *value == Value::U32(0)));
            assert!(cursor.is_empty());

            // Another handle creates records after login. Refresh must reread
            // the durable store, not reuse the original login's empty snapshot.
            let writer = CharacterStore::open(path.clone()).unwrap();
            writer.create(&crate::character_store::test_request("Refresh One")).unwrap();
            writer.create(&crate::character_store::test_request("Refresh Two")).unwrap();
            let expected = store.list().unwrap();
            let mut reply = BytesMut::new();
            for part in [&input[..cut], &input[cut..]] {
                if let Some(bytes) = probe.push(part).unwrap() {
                    reply.extend_from_slice(&bytes);
                }
            }
            let mut cursor = &reply[..];
            for record in &expected {
                let (message, value) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
                assert_eq!(message.id, 17);
                let Value::Struct(fields) = value else { panic!() };
                assert_eq!(fields[0].1, Value::U32(123456));
                assert_eq!(fields[1].1, Value::Guid(record.guid()));
                assert_eq!(fields[5].1, Value::Text(record.name().to_owned()));
                assert_eq!(fields[6].1, Value::Bytes(record.summary().to_vec()));
            }
            let (message, value) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
            assert_eq!(message.id, 4, "completion must follow all records");
            let Value::Struct(fields) = value else { panic!() };
            assert_eq!(fields[0].1, Value::U32(123456));
            assert!(fields[1..].iter().all(|(_, value)| *value == Value::U32(0)));
            assert!(cursor.is_empty());

            // Multiple refresh transactions on one connection remain correlated.
            let combined = [refresh(7).as_ref(), refresh(u32::MAX).as_ref()].concat();
            let reply = probe.push(&combined).unwrap().unwrap();
            let mut cursor = &reply[..];
            for request in [7, u32::MAX] {
                for id in [17, 17, 4] {
                    let (message, value) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
                    assert_eq!(message.id, id);
                    let Value::Struct(fields) = value else { panic!() };
                    assert_eq!(fields[0].1, Value::U32(request));
                }
            }
            assert!(cursor.is_empty());
            assert!(probe.play_enabled());
            std::fs::remove_dir_all(path).unwrap();
        }
    }


    #[test]
    fn saved_name_selection_returns_the_requested_guid_for_every_split() {
        let (path, store) = saved_store();
        let first = store.create(&crate::character_store::test_request("First Mesmer")).unwrap();
        let second = store.create(&crate::character_store::test_request("Second Ω🦀")).unwrap();
        let fixture = saved_fixture(store);
        let route = PlayRoute::new("127.1.0.3:6112".parse().unwrap(), Guid([0x55; 16])).unwrap();
        let server = schema::parse(AUTH_SCHEMA).unwrap().protocols[0].groups[0].server.clone();
        for record in [&second, &first] {
            let input = name_request(123456, record.name());
            for cut in 0..=input.len() {
                let mut probe = AuthProbe::new(fixture.clone()).with_play_route(route).unwrap();
                probe.push(&login_input()).unwrap().unwrap();
                let mut replies = BytesMut::new();
                for part in [&input[..cut], &input[cut..]] {
                    if let Some(reply) = probe.push(part).unwrap() {
                        replies.extend_from_slice(&reply);
                    }
                }
                let mut cursor = &replies[..];
                let (message, value) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
                assert_eq!(message.id, 8);
                let Value::Struct(fields) = value else { panic!() };
                assert_eq!(fields[0].1, Value::U32(123456));
                assert_eq!(fields[2].1, Value::Guid(record.guid()));
                let (message, value) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
                assert_eq!(message.id, 3);
                let Value::Struct(fields) = value else { panic!() };
                assert_eq!(fields[0].1, Value::U32(123456));
                assert!(fields[3..].iter().all(|(_, value)| *value == Value::U32(0)));
                assert!(cursor.is_empty());
            }
        }
        let mut probe = AuthProbe::new(fixture.clone()).with_play_route(route).unwrap();
        probe.push(&login_input()).unwrap().unwrap();
        let error = probe.push(&name_request(500, "Not Saved")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::NotFound);
        assert_eq!(error.to_string(), "selected character is not saved");
        let mut probe = AuthProbe::new(fixture).with_play_route(route).unwrap();
        probe.push(&login_input()).unwrap().unwrap();
        assert!(probe.push(&name_request(501, "12345678901234567890")).unwrap().is_none());
        assert!(!probe.play_enabled(), "name beyond String(20) must not be selected");
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn creation_route_uses_its_listener_and_other_routes_keep_world_listener() {
        let world = "127.1.0.3:6112".parse().unwrap();
        let creation = "127.1.0.4:6112".parse().unwrap();
        let original = PlayRoute::new(world, Guid([0x55; 16])).unwrap();
        let separate = original.with_creation_address(creation).unwrap();
        assert!(original.with_creation_address("192.0.2.1:6112".parse().unwrap()).is_err());
        let server = schema::parse(AUTH_SCHEMA).unwrap().protocols[0].groups[0].server.clone();
        for route in [original, separate] {
            for game_type in [1, 3, 9] {
                let fixture = ReplyFixture::new(Bytes::from_static(&[0x44; 2]), None).unwrap();
                let mut probe = AuthProbe::new(fixture).with_play_route(route).unwrap();
                probe.push(&login_input()).unwrap().unwrap();
                let mut input = BytesMut::new();
                RouteRequest29 {
                    request: 700,
                    game_type,
                    reserved: 17,
                    map_key: 379,
                    argument: 0,
                    hash: 0,
                    context: Guid([0; 16]),
                }.encode_frame(&mut input);
                let reply = probe.push(&input).unwrap().unwrap();
                let mut cursor = &reply[..];
                let (message, value) = dynamic::decode_frame(&server, &census_rules(), &mut cursor).unwrap();
                assert_eq!(message.id, 20);
                let Value::Struct(fields) = value else { panic!() };
                assert_eq!(fields[0].1, Value::U32(700));
                assert_eq!(fields[1].1, Value::U32(game_type));
                assert_eq!(fields[2].1, Value::U32(17));
                assert_eq!(fields[3].1, Value::U32(379));
                let expected = if game_type == 1 { route.creation_address.unwrap_or(world) } else { world };
                assert_eq!(fields[4].1, Value::Address(expected));
                assert!(cursor.is_empty());
            }
        }
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
            assert_eq!(f20[1].1, Value::U32(4));
            assert_eq!(f20[3].1, Value::U32(15));
            assert_eq!(f20[4].1, Value::Address("127.1.0.3:6112".parse().unwrap()));
            assert!(cursor.is_empty(), "cut {cut}");
        }
    }

    /// The client's later play-phase traffic, with the sizes Banshee logged on
    /// 2026-11-02: heartbeat 1 (6 bytes, a 4-byte varint clock), 4 (115 bytes:
    /// BufferVarSmall 96 + Guid) and 34 (3 bytes, a 1-byte varint).
    fn play_tail() -> BytesMut {
        let mut input = BytesMut::new();
        1u16.encode(&mut input);
        0x0fff_ffffu32.encode(&mut input);
        4u16.encode(&mut input);
        input.extend_from_slice(&[96]);
        input.extend_from_slice(&[0x61; 96]);
        Guid([0x77; 16]).encode(&mut input);
        34u16.encode(&mut input);
        3u32.encode(&mut input);
        input
    }

    #[test]
    fn play_phase_tail_after_route_gets_no_reply_and_stays_enabled() {
        let tail = play_tail();
        assert_eq!(tail.len(), 6 + 115 + 3, "frame sizes match the live log");
        for cut in 0..=tail.len() {
            let mut probe = play_probe();
            assert!(probe.push(&play_input()).unwrap().is_some(), "22/29 answered");
            for part in [&tail[..cut], &tail[cut..]] {
                assert!(probe.push(part).unwrap().is_none(), "no reply to 1/4/34 (cut {cut})");
            }
            assert!(probe.play_enabled(), "tail must not disable the phase (cut {cut})");
            // A second route request after the tail is still answered.
            let again = probe.push(&play_input()).unwrap();
            assert!(again.is_some(), "phase still live after the tail (cut {cut})");
        }
    }

    /// Every frame the auth server sends (the login reply 8/10/17/7 and the play replies
    /// 8+3 and 20) decodes to its exact length under the client's own auth server table
    /// (0x141923fe0, extracted from its registration tables), not only under auth.xml.
    /// Uses the DEPLOYED fixtures: set GW2_AUTH_FIXTURE_DIR to a kit's data/ directory
    /// (auth-account-body.bin, character-record.bin) and run with `--ignored`.
    #[test]
    #[ignore]
    fn every_auth_reply_frame_matches_the_client_auth_table() {
        let dir = std::path::PathBuf::from(std::env::var("GW2_AUTH_FIXTURE_DIR").expect("GW2_AUTH_FIXTURE_DIR"));
        let body = Bytes::from(std::fs::read(dir.join("auth-account-body.bin")).unwrap());
        let deployed_record = Bytes::from(std::fs::read(dir.join("character-record.bin")).unwrap());
        const SERVER_TABLES: &str = include_str!("../../../../../data/protocol/209026/server-tables.xml");
        let tables = schema::parse(SERVER_TABLES).unwrap().protocols[0].groups.clone();
        let table = tables.iter().find(|g| g.name == "table-0x141923fe0").expect("client auth table");
        // As gw2-game-observe loads it: home and current world filled in.
        let fixture = ReplyFixture::new(body, Some(deployed_record)).unwrap().with_home_world().unwrap();
        let dwords = login_body_dwords(&fixture.body);
        assert_eq!((dwords[3], dwords[4]), (HOME_WORLD, HOME_WORLD), "Dword6 home world, Dword7 current world");
        let route = PlayRoute::new("127.1.0.4:6112".parse().unwrap(), Guid([0x55; 16])).unwrap();
        let mut probe = AuthProbe::new(fixture).with_play_route(route).unwrap();
        let login = probe.push(&login_input()).unwrap().unwrap();
        let play = probe.push(&play_input()).unwrap().unwrap();
        let mut ids = Vec::new();
        for (name, bytes) in [("login", login), ("play", play)] {
            let mut cursor = &bytes[..];
            while !cursor.is_empty() {
                let (message, _) = dynamic::decode_frame(&table.server, &census_rules(), &mut cursor)
                    .unwrap_or_else(|e| panic!("{name} reply frame misfits the client auth table: {e}"));
                ids.push(message.id);
            }
        }
        assert_eq!(ids, vec![8, 23, 25, 24, 10, 17, 7, 8, 3, 20]);
    }

    /// Malformed client input never panics the auth probe: random bytes before login are
    /// an error or retained (bounded), and random bytes in the play phase disable it cleanly;
    /// every truncated prefix of the play input is retained without a reply.
    #[test]
    fn malformed_auth_input_never_panics() {
        let noise = |seed: u64, len: usize| {
            let mut x = seed | 1;
            (0..len)
                .map(|_| {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    x as u8
                })
                .collect::<Vec<u8>>()
        };
        let play = play_input();
        for cut in 0..play.len() {
            let mut probe = play_probe();
            let _ = probe.push(&play[..cut]).unwrap();
            assert!(probe.play_enabled(), "prefix {cut} must be retained, not misparsed");
        }
        for seed in 0..500u64 {
            let input = noise(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15), (seed as usize % 300) + 1);
            // Before login: an error or retained bytes, never a panic.
            let fixture = ReplyFixture::new(Bytes::from_static(&[0x44; 2]), None).unwrap();
            let mut fresh = AuthProbe::new(fixture);
            let _ = fresh.push(&input);
            // In the play phase: no panic; garbage either frames (ids 22/29 only reply) or disables.
            let mut probe = play_probe();
            let _ = probe.push(&input);
        }
    }

    /// The login phase frames clients 1, 2, 3 and 10 with its own hand-written schema
    /// (`messages()`). Frames encoded from the client-extracted auth.xml layouts must be
    /// accepted by it: no reply for 1, 2 and 3, exactly one reply once 10 is complete,
    /// for every split of the stream, and the hand-written field kinds must match auth.xml.
    #[test]
    fn login_phase_schema_matches_the_client_layouts() {
        let group = schema::parse(AUTH_SCHEMA).unwrap().protocols.into_iter().find(|p| p.name == "3").unwrap().groups.remove(0);
        for own in messages() {
            let client = group.client.iter().find(|m| m.id == own.id).unwrap();
            let kinds = |m: &Message| m.fields.iter().map(|f| (f.kind.clone(), f.size)).collect::<Vec<_>>();
            assert_eq!(kinds(&own), kinds(client), "hand-written schema for client {} differs from auth.xml", own.id);
        }
        let mut input = BytesMut::new();
        for id in [2u16, 3, 1, 10] {
            input.extend_from_slice(&crate::mission::tests::schema_frame(&group.client, id, false));
        }
        for cut in 0..=input.len() {
            let fixture = ReplyFixture::new(Bytes::from_static(&[0x44; 2]), None).unwrap();
            let mut probe = AuthProbe::new(fixture);
            let replies = [&input[..cut], &input[cut..]]
                .into_iter()
                .filter_map(|part| probe.push(part).unwrap())
                .count();
            assert_eq!(replies, 1, "cut {cut}");
        }
    }

    /// Every auth client message the play phase loads (auth.xml protocol 3) is framed
    /// without disabling the phase; only 12 (preferences upload), 14 (home-world select),
    /// 22 and 29 get replies.
    #[test]
    fn every_auth_client_message_frames_in_the_play_phase() {
        let group = schema::parse(AUTH_SCHEMA).unwrap().protocols.into_iter().find(|p| p.name == "3").unwrap().groups.remove(0);
        assert_eq!(group.client.len(), 28);
        for message in &group.client {
            let frame = crate::mission::tests::schema_frame(&group.client, message.id, false);
            let mut probe = play_probe();
            let reply = probe.push(&frame).unwrap();
            assert!(probe.play_enabled(), "client {} disabled the play phase", message.id);
            assert_eq!(reply.is_some(), matches!(message.id, 12 | 14 | 22 | 29), "client {} reply", message.id);
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

    /// Replays a private real capture (never committed): set
    /// GW2_AUTH_CAPTURE to a run-58/59 `*-0.decrypted.bin` and run with
    /// `--ignored`. The phase must survive the client's post-login frames.
    #[test]
    #[ignore]
    fn play_phase_survives_real_post_login_capture() {
        let path = std::env::var("GW2_AUTH_CAPTURE").expect("GW2_AUTH_CAPTURE");
        let capture = std::fs::read(path).unwrap();
        let fixture = ReplyFixture::new(Bytes::from_static(&[0x44; 2]), None).unwrap();
        let route = PlayRoute::new("127.1.0.3:6112".parse().unwrap(), Guid([0x55; 16])).unwrap();
        let mut probe = AuthProbe::new(fixture).with_play_route(route).unwrap();
        let mut replies = 0;
        for byte in capture.chunks(1) {
            if probe.push(byte).unwrap().is_some() {
                replies += 1;
            }
        }
        assert_eq!(replies, 1, "only the login reply; no 22/29 in this capture");
        assert!(probe.play_enabled(), "post-login frames must not disable the phase");
    }
}
