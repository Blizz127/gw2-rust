//! Build-209026 auth protocol-3 frames on the existing-character Play path:
//! client 22 (name selection), server 3 (its completion), client 29 (route
//! request) and server 20 (route result). Layouts and transaction roles are in
//! `docs/re/209026-play-route.md`, `docs/re/209026-character-creation.md` and
//! `docs/re/209026-mission-join.md`. These are framing types only: valid
//! server values (addresses, scalars, result codes) are not established, and
//! no frame here has been exchanged with the client.

use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;

use bytes::BytesMut;

use crate::{
    Decode, Encode, Guid,
    dynamic::{self, Rules, StringEncoding, Value},
    schema::{Field, Message},
};

/// `GAME_TYPE_CHARCREATE`; the Play path asserts its game type differs.
pub const GAME_TYPE_CHARCREATE: u32 = 1;
/// `GAME_TYPES`: valid game types are below this bound.
pub const GAME_TYPES: u32 = 0x16;

fn expect_id(input: &mut &[u8], id: u16, what: &str) -> Result<()> {
    if u16::decode(input)? != id {
        return Err(Error::new(ErrorKind::InvalidData, format!("expected {what}")));
    }
    Ok(())
}

/// Client 22: request ID and the selected character's name (UTF-16, ≤ 20 units
/// including the terminator, per the schema bound).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameSelect22 {
    pub request: u32,
    pub name: String,
}

impl NameSelect22 {
    pub const MESSAGE_ID: u16 = 22;

    /// Consume exactly one frame; leave input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        expect_id(&mut cursor, Self::MESSAGE_ID, "name selection message 22")?;
        let message = Message {
            id: Self::MESSAGE_ID,
            name: String::new(),
            addr: None,
            fields: vec![field("Dword", None), field("String", Some(20))],
        };
        let rules = Rules {
            string: Some(StringEncoding::Utf16Terminated),
            ..Rules::default()
        };
        let Value::Struct(fields) = dynamic::decode_message(&message, &rules, &mut cursor)? else {
            unreachable!("message decodes to a struct")
        };
        let (Value::U32(request), Value::Text(name)) = (&fields[0].1, &fields[1].1) else {
            return Err(Error::new(ErrorKind::InvalidData, "unexpected field kinds"));
        };
        let decoded = Self {
            request: *request,
            name: name.clone(),
        };
        *input = cursor;
        Ok(decoded)
    }
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

/// Server 3: completion of the pending name transaction. The handler matches
/// `request`, stores `address`/`scalar` in the transaction and copies the low
/// Words of `result` as the result code (all zero = success).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameSelectComplete3 {
    pub request: u32,
    pub scalar: u32,
    pub address: SocketAddr,
    pub result: [u32; 4],
}

impl NameSelectComplete3 {
    pub const MESSAGE_ID: u16 = 3;

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.request.encode(output);
        self.scalar.encode(output);
        self.address.encode(output);
        for value in self.result {
            value.encode(output);
        }
    }
}

/// Client 29: game route request. Wire order from send method `0x14023e490`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteRequest29 {
    pub request: u32,
    pub game_type: u32,
    pub reserved: u32,
    pub map_key: u32,
    pub argument: u32,
    pub hash: u32,
    pub context: Guid,
}

impl RouteRequest29 {
    pub const MESSAGE_ID: u16 = 29;

    /// Consume exactly one frame; leave input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        expect_id(&mut cursor, Self::MESSAGE_ID, "route request message 29")?;
        let decoded = Self {
            request: u32::decode(&mut cursor)?,
            game_type: u32::decode(&mut cursor)?,
            reserved: u32::decode(&mut cursor)?,
            map_key: u32::decode(&mut cursor)?,
            argument: u32::decode(&mut cursor)?,
            hash: u32::decode(&mut cursor)?,
            context: Guid::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(decoded)
    }

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        for value in [
            self.request,
            self.game_type,
            self.reserved,
            self.map_key,
            self.argument,
            self.hash,
        ] {
            value.encode(output);
        }
        self.context.encode(output);
    }

    /// The creation path sends game type 1; the Play path asserts otherwise.
    pub fn is_creation(&self) -> bool {
        self.game_type == GAME_TYPE_CHARCREATE
    }

    /// Mirrors the client's own `gameType < GAME_TYPES` assertion.
    pub fn game_type_in_range(&self) -> bool {
        self.game_type < GAME_TYPES
    }
}

/// Server 20: route result. Fields 1–3 are copied to transaction +0x40/+0x44/
/// +0x48 and `scalar` to +0x94; their meanings are unresolved, hence the
/// neutral names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteResult20 {
    pub request: u32,
    pub values: [u32; 3],
    pub address: SocketAddr,
    pub scalar: u32,
}

impl RouteResult20 {
    pub const MESSAGE_ID: u16 = 20;

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.request.encode(output);
        for value in self.values {
            value.encode(output);
        }
        self.address.encode(output);
        self.scalar.encode(output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;

    fn auth_messages(server: bool) -> Vec<Message> {
        let xml = include_str!("../../../data/protocol/209026/auth.xml");
        let schema = schema::parse(xml).unwrap();
        let group = &schema.protocols[0].groups[0];
        if server {
            group.server.clone()
        } else {
            group.client.clone()
        }
    }

    fn rules() -> Rules {
        Rules {
            string: Some(StringEncoding::Utf16Terminated),
            ..Rules::default()
        }
    }

    #[test]
    fn name_select_22_matches_schema_and_is_transactional() {
        let mut wire = BytesMut::new();
        22u16.encode(&mut wire);
        300u32.encode(&mut wire);
        for unit in "Fixture".encode_utf16().chain(std::iter::once(0)) {
            unit.encode(&mut wire);
        }
        wire.extend_from_slice(b"tail");
        let mut input = &wire[..];
        let decoded = NameSelect22::decode_frame(&mut input).unwrap();
        assert_eq!(decoded, NameSelect22 { request: 300, name: "Fixture".into() });
        assert_eq!(input, b"tail");

        // The schema-driven census decoder agrees on the boundary.
        let client = auth_messages(false);
        let mut census = &wire[..];
        let (message, _) = dynamic::decode_frame(&client, &rules(), &mut census).unwrap();
        assert_eq!(message.id, 22);
        assert_eq!(census, b"tail");

        for cut in 0..wire.len() - 4 {
            let original = &wire[..cut];
            let mut input = original;
            assert!(NameSelect22::decode_frame(&mut input).is_err(), "cut {cut}");
            assert_eq!(input, original);
        }
        let mut other = &[29u8, 0, 1][..];
        assert!(NameSelect22::decode_frame(&mut other).is_err());
        assert_eq!(other, &[29, 0, 1]);
    }

    #[test]
    fn route_request_29_round_trips_and_matches_schema() {
        let play = RouteRequest29 {
            request: 0x1234,
            game_type: 4,
            reserved: 0,
            map_key: 15,
            argument: 0,
            hash: 0xdead_beef,
            context: Guid([0x5a; 16]),
        };
        let mut wire = BytesMut::new();
        play.encode_frame(&mut wire);
        let mut input = &wire[..];
        assert_eq!(RouteRequest29::decode_frame(&mut input).unwrap(), play);
        assert!(input.is_empty());
        assert!(!play.is_creation() && play.game_type_in_range());

        let client = auth_messages(false);
        let mut census = &wire[..];
        let (message, value) = dynamic::decode_frame(&client, &rules(), &mut census).unwrap();
        assert_eq!(message.id, 29);
        assert!(census.is_empty());
        let Value::Struct(fields) = value else { panic!() };
        assert_eq!(fields.len(), 7);
        assert_eq!(fields[3].1, Value::U32(15));

        // The documented creation fixture: [req, 1, 0, 0, 0, 0] + zero value.
        let creation = RouteRequest29 {
            request: 7,
            game_type: GAME_TYPE_CHARCREATE,
            reserved: 0,
            map_key: 0,
            argument: 0,
            hash: 0,
            context: Guid([0; 16]),
        };
        let mut bytes = BytesMut::new();
        creation.encode_frame(&mut bytes);
        assert_eq!(&bytes[..], &[[29, 0, 7, 1, 0, 0, 0, 0].as_slice(), &[0; 16]].concat()[..]);
        assert!(creation.is_creation());

        for cut in 0..wire.len() {
            let original = &wire[..cut];
            let mut input = original;
            assert!(RouteRequest29::decode_frame(&mut input).is_err(), "cut {cut}");
            assert_eq!(input, original);
        }
    }

    #[test]
    fn server_3_and_20_encodings_match_schema_layouts() {
        let address: SocketAddr = "127.1.0.3:6112".parse().unwrap();
        let mut wire = BytesMut::new();
        NameSelectComplete3 { request: 300, scalar: 9, address, result: [0; 4] }
            .encode_frame(&mut wire);
        RouteResult20 { request: 301, values: [1, 2, 3], address, scalar: 4 }.encode_frame(&mut wire);
        let messages = auth_messages(true);
        let mut input = &wire[..];
        let (first, value) = dynamic::decode_frame(&messages, &rules(), &mut input).unwrap();
        assert_eq!(first.id, 3);
        let Value::Struct(fields) = value else { panic!() };
        assert_eq!(fields[0].1, Value::U32(300));
        assert_eq!(fields[2].1, Value::Address(address));
        let (second, value) = dynamic::decode_frame(&messages, &rules(), &mut input).unwrap();
        assert_eq!(second.id, 20);
        let Value::Struct(fields) = value else { panic!() };
        assert_eq!(fields[4].1, Value::Address(address));
        assert_eq!(fields[5].1, Value::U32(4));
        assert!(input.is_empty());
    }
}
