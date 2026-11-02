//! Build-209026 protocol-0 mission (game server) frames for map loading:
//! server 1063 (MapData, logged by the client as "MsCli OnRecvMapData"), server
//! 1060 (LoadMap, "OnRecvLoadMap"), server 1061 (LoadAgents, empty), server
//! 1036 (BeginPlaying, empty), client 423 (language preferences), client 420
//! (MapLoaded, empty), client 418 (FailedToLoad, empty) and client 412
//! (AgentsLoaded, empty). Layouts and handler roles are in
//! `docs/re/209026-mission-transfer.md`, `docs/re/209026-map-ready.md` and
//! `docs/re/209026-mission-entry.md` (1063 field roles and ordering). These
//! are framing types only: valid server values are not established, and no
//! frame here has been exchanged with the client.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// `LANGUAGES`: the client asserts each language preference is below this bound.
pub const LANGUAGES: u8 = 6;

/// Server 1061: ask the client to load agents (empty body).
pub const LOAD_AGENTS_1061: u16 = 1061;
/// Server 1036: begin playing (empty body). The client asserts loader
/// `STATE_READY` when it receives this frame, so it must only be sent after the
/// client has sent 412 (`AGENTS_LOADED_412`).
pub const BEGIN_PLAYING_1036: u16 = 1036;
/// Client 420: map loaded (empty body).
pub const MAP_LOADED_420: u16 = 420;
/// Client 418: failed to load (empty body).
pub const FAILED_TO_LOAD_418: u16 = 418;
/// Client 412: agents loaded (empty body).
pub const AGENTS_LOADED_412: u16 = 412;

fn expect_id(input: &mut &[u8], id: u16, what: &str) -> Result<()> {
    if u16::decode(input)? != id {
        return Err(Error::new(ErrorKind::InvalidData, format!("expected {what}")));
    }
    Ok(())
}

/// Encode a frame that has no body: just the message ID.
pub fn encode_empty(id: u16, output: &mut BytesMut) {
    id.encode(output);
}

/// Consume exactly one body-less frame with message ID `id` (2 bytes); leave
/// input unchanged on any error.
pub fn decode_empty(input: &mut &[u8], id: u16) -> Result<()> {
    let mut cursor = *input;
    expect_id(&mut cursor, id, &format!("empty message {id}"))?;
    *input = cursor;
    Ok(())
}

/// Server 1063: map data. Wire: Dword, Dword, Dword, Dword, Byte, Byte. Field
/// roles come from the client's handler; the neutral names mark values whose
/// meaning is unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapData1063 {
    /// Stored at mission context +0x22c, which the client reads back as
    /// `MissionCliGetPlayerId()` (`0x141411190`, named by the assertion
    /// `m_ownerPlayerId == MissionCliGetPlayerId()`): the local player ID.
    pub player_id: u32,
    /// The full raw Dword; the client masks it, see [`Self::map_id`].
    pub map: u32,
    /// Copied to a client global; meaning unknown.
    pub global_value: u32,
    /// Stored at mission context +0x1d0; meaning unknown.
    pub context_value_2: u32,
    /// Nonzero sets mission context +0x228 bit 0x20.
    pub flag_a: u8,
    /// Nonzero sets mission context +0x228 bit 0x10.
    pub flag_b: u8,
}

impl MapData1063 {
    pub const MESSAGE_ID: u16 = 1063;

    /// A frame for `map_id` with every other field zero. `None` when `map_id`
    /// has bit 31 or 30 set (those bits are flags, see [`Self::high_flags`]).
    pub fn for_map(map_id: u32) -> Option<Self> {
        if map_id > 0x3fff_ffff {
            return None;
        }
        Some(Self {
            player_id: 0,
            map: map_id,
            global_value: 0,
            context_value_2: 0,
            flag_a: 0,
            flag_b: 0,
        })
    }

    /// Consume exactly one frame; leave input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        expect_id(&mut cursor, Self::MESSAGE_ID, "map data message 1063")?;
        let decoded = Self {
            player_id: u32::decode(&mut cursor)?,
            map: u32::decode(&mut cursor)?,
            global_value: u32::decode(&mut cursor)?,
            context_value_2: u32::decode(&mut cursor)?,
            flag_a: u8::decode(&mut cursor)?,
            flag_b: u8::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(decoded)
    }

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.player_id.encode(output);
        self.map.encode(output);
        self.global_value.encode(output);
        self.context_value_2.encode(output);
        self.flag_a.encode(output);
        self.flag_b.encode(output);
    }

    /// The map ID the client uses: `map & 0x3fff_ffff`.
    pub fn map_id(&self) -> u32 {
        self.map & 0x3fff_ffff
    }

    /// `(bit 31 set, bit 30 set)` of [`Self::map`]. The client turns bit 31 into
    /// mission context flag 0x200 and bit 30 into 0x400; their meanings are
    /// unknown.
    pub fn high_flags(&self) -> (bool, bool) {
        (self.map & 0x8000_0000 != 0, self.map & 0x4000_0000 != 0)
    }
}

/// Server 1060: load map. Wire: Float3, Float2, Float2, Word, Byte, Qword,
/// Dword. Floats are carried as raw bit patterns (NaN payloads survive a
/// decode/encode round trip).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoadMap1060 {
    /// Forwarded to the loader at +0x2a8; physical role unresolved.
    pub position: [f32; 3],
    /// The two Float2 values, copied as 16 consecutive bytes to mission
    /// context +0x1d8.
    pub vectors: [[f32; 2]; 2],
    /// Only logged by the client.
    pub diagnostic: u16,
    /// The client converts it to radians as `2 * value * pi / 255`, see
    /// [`Self::angle_radians`].
    pub angle: u8,
    /// Stored at loader +0x318.
    pub loader_value: u64,
    /// Passed to a client function that is not yet resolved.
    pub final_value: u32,
}

impl LoadMap1060 {
    pub const MESSAGE_ID: u16 = 1060;

    /// Consume exactly one frame; leave input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        expect_id(&mut cursor, Self::MESSAGE_ID, "load map message 1060")?;
        let decoded = Self {
            position: <[f32; 3]>::decode(&mut cursor)?,
            vectors: [
                <[f32; 2]>::decode(&mut cursor)?,
                <[f32; 2]>::decode(&mut cursor)?,
            ],
            diagnostic: u16::decode(&mut cursor)?,
            angle: u8::decode(&mut cursor)?,
            loader_value: u64::decode(&mut cursor)?,
            final_value: u32::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(decoded)
    }

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.position.encode(output);
        for vector in self.vectors {
            vector.encode(output);
        }
        self.diagnostic.encode(output);
        self.angle.encode(output);
        self.loader_value.encode(output);
        self.final_value.encode(output);
    }

    /// The client's byte-to-radians conversion: `2 * angle * pi / 255`.
    pub fn angle_radians(&self) -> f32 {
        2.0 * self.angle as f32 * std::f32::consts::PI / 255.0
    }
}

/// Client 423: language preferences, text then audio. The client asserts each
/// is below [`LANGUAGES`]; the enum labels are not mapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguagePrefs423 {
    pub text: u8,
    pub audio: u8,
}

impl LanguagePrefs423 {
    pub const MESSAGE_ID: u16 = 423;

    /// Consume exactly one frame; leave input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        expect_id(&mut cursor, Self::MESSAGE_ID, "language preferences message 423")?;
        let decoded = Self {
            text: u8::decode(&mut cursor)?,
            audio: u8::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(decoded)
    }

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.text.encode(output);
        self.audio.encode(output);
    }

    /// Mirrors the client's own `< LANGUAGES` assertion on both values.
    pub fn in_range(&self) -> bool {
        self.text < LANGUAGES && self.audio < LANGUAGES
    }
}

/// The client frames of the map-load handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientMapEvent {
    LanguagePrefs(LanguagePrefs423),
    MapLoaded,
    FailedToLoad,
    AgentsLoaded,
}

impl ClientMapEvent {
    /// Consume exactly one frame, dispatching on its ID; leave input unchanged
    /// on any error, including an ID that is not one of these four.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut peek = *input;
        match u16::decode(&mut peek)? {
            LanguagePrefs423::MESSAGE_ID => {
                LanguagePrefs423::decode_frame(input).map(Self::LanguagePrefs)
            }
            MAP_LOADED_420 => decode_empty(input, MAP_LOADED_420).map(|()| Self::MapLoaded),
            FAILED_TO_LOAD_418 => {
                decode_empty(input, FAILED_TO_LOAD_418).map(|()| Self::FailedToLoad)
            }
            AGENTS_LOADED_412 => {
                decode_empty(input, AGENTS_LOADED_412).map(|()| Self::AgentsLoaded)
            }
            id => Err(Error::new(
                ErrorKind::InvalidData,
                format!("not a map-load client message: {id}"),
            )),
        }
    }

    pub fn encode_frame(&self, output: &mut BytesMut) {
        match self {
            Self::LanguagePrefs(prefs) => prefs.encode_frame(output),
            Self::MapLoaded => encode_empty(MAP_LOADED_420, output),
            Self::FailedToLoad => encode_empty(FAILED_TO_LOAD_418, output),
            Self::AgentsLoaded => encode_empty(AGENTS_LOADED_412, output),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dynamic::{self, Rules, Value},
        schema::{self, Message},
    };

    /// Messages of the `<Messages Name="39">` group, selected by name.
    fn mission_messages(server: bool) -> Vec<Message> {
        let xml = include_str!("../../../data/protocol/209026/mission.xml");
        let schema = schema::parse(xml).unwrap();
        let group = schema.protocols[0]
            .groups
            .iter()
            .find(|group| group.name == "39")
            .expect("mission group 39");
        if server {
            group.server.clone()
        } else {
            group.client.clone()
        }
    }

    fn sample_1063() -> MapData1063 {
        MapData1063 {
            player_id: 0x1234,
            map: 0xc000_0000 | 15,
            global_value: 300,
            context_value_2: 7,
            flag_a: 1,
            flag_b: 0,
        }
    }

    fn sample_1060() -> LoadMap1060 {
        LoadMap1060 {
            position: [-1.5, f32::from_bits(0x7fc0_1234), 1024.25],
            vectors: [[0.5, -0.25], [-8.0, 3.0]],
            diagnostic: 0xbeef,
            angle: 128,
            loader_value: 0x0123_4567_89ab_cdef,
            final_value: 5,
        }
    }

    fn floats(value: &Value) -> Vec<u32> {
        let Value::Floats(values) = value else { panic!("not floats: {value:?}") };
        values.iter().map(|v| v.to_bits()).collect()
    }

    #[test]
    fn map_data_1063_round_trips_and_matches_schema() {
        let frame = sample_1063();
        let mut wire = BytesMut::new();
        frame.encode_frame(&mut wire);
        let mut input = &wire[..];
        assert_eq!(MapData1063::decode_frame(&mut input).unwrap(), frame);
        assert!(input.is_empty());

        let server = mission_messages(true);
        let mut census = &wire[..];
        let (message, value) = dynamic::decode_frame(&server, &Rules::default(), &mut census).unwrap();
        assert_eq!(message.id, 1063);
        assert!(census.is_empty());
        let Value::Struct(fields) = value else { panic!() };
        assert_eq!(fields.len(), 6);
        assert_eq!(fields[0].1, Value::U32(frame.player_id));
        assert_eq!(fields[1].1, Value::U32(frame.map));
        assert_eq!(fields[2].1, Value::U32(frame.global_value));
        assert_eq!(fields[3].1, Value::U32(frame.context_value_2));
        assert_eq!(fields[4].1, Value::U8(1));
        assert_eq!(fields[5].1, Value::U8(0));
    }

    #[test]
    fn map_data_1063_map_id_and_high_flags() {
        let frame = sample_1063();
        assert_eq!(frame.map_id(), 15);
        assert_eq!(frame.high_flags(), (true, true));
        let only_31 = MapData1063 { map: 0x8000_0001, ..frame };
        assert_eq!((only_31.map_id(), only_31.high_flags()), (1, (true, false)));
        let only_30 = MapData1063 { map: 0x4000_0002, ..frame };
        assert_eq!((only_30.map_id(), only_30.high_flags()), (2, (false, true)));

        let plain = MapData1063::for_map(0x3fff_ffff).unwrap();
        assert_eq!(plain.map_id(), 0x3fff_ffff);
        assert_eq!(plain.high_flags(), (false, false));
        assert_eq!(
            plain,
            MapData1063 {
                player_id: 0,
                map: 0x3fff_ffff,
                global_value: 0,
                context_value_2: 0,
                flag_a: 0,
                flag_b: 0,
            }
        );
        assert!(MapData1063::for_map(0x4000_0000).is_none());
        assert!(MapData1063::for_map(0x8000_0000).is_none());
    }

    #[test]
    fn map_data_1063_truncation_and_wrong_id_leave_input_unchanged() {
        let mut wire = BytesMut::new();
        sample_1063().encode_frame(&mut wire);
        for cut in 0..wire.len() {
            let original = &wire[..cut];
            let mut input = original;
            assert!(MapData1063::decode_frame(&mut input).is_err(), "cut {cut}");
            assert_eq!(input, original);
        }
        let mut other = vec![0x25, 0x04, 1, 2, 3];
        other.extend_from_slice(&wire[2..]);
        let original = other.clone();
        let mut input = &other[..];
        assert!(MapData1063::decode_frame(&mut input).is_err());
        assert_eq!(input, &original[..]);
    }

    #[test]
    fn load_map_1060_round_trips_bitwise_and_matches_schema() {
        let frame = sample_1060();
        let mut wire = BytesMut::new();
        frame.encode_frame(&mut wire);
        let mut input = &wire[..];
        let decoded = LoadMap1060::decode_frame(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(decoded.position.map(f32::to_bits), frame.position.map(f32::to_bits));
        assert!(decoded.position[1].is_nan());
        assert_eq!(decoded.position[1].to_bits(), 0x7fc0_1234);
        for (got, want) in decoded.vectors.iter().zip(frame.vectors) {
            assert_eq!(got.map(f32::to_bits), want.map(f32::to_bits));
        }
        assert_eq!(decoded.diagnostic, frame.diagnostic);
        assert_eq!(decoded.angle, frame.angle);
        assert_eq!(decoded.loader_value, frame.loader_value);
        assert_eq!(decoded.final_value, frame.final_value);

        let server = mission_messages(true);
        let mut census = &wire[..];
        let (message, value) = dynamic::decode_frame(&server, &Rules::default(), &mut census).unwrap();
        assert_eq!(message.id, 1060);
        assert!(census.is_empty());
        let Value::Struct(fields) = value else { panic!() };
        assert_eq!(fields.len(), 7);
        assert_eq!(floats(&fields[0].1), frame.position.map(f32::to_bits));
        assert_eq!(floats(&fields[1].1), frame.vectors[0].map(f32::to_bits));
        assert_eq!(floats(&fields[2].1), frame.vectors[1].map(f32::to_bits));
        assert_eq!(fields[3].1, Value::U16(0xbeef));
        assert_eq!(fields[4].1, Value::U8(128));
        assert_eq!(fields[5].1, Value::U64(0x0123_4567_89ab_cdef));
        assert_eq!(fields[6].1, Value::U32(5));
    }

    #[test]
    fn load_map_1060_is_42_bytes_with_one_byte_final_value() {
        for final_value in [0, 5, 127] {
            let mut wire = BytesMut::new();
            LoadMap1060 { final_value, ..sample_1060() }.encode_frame(&mut wire);
            // 2 id + 12 + 8 + 8 + 2 + 1 + 8 + 1
            assert_eq!(wire.len(), 42, "final_value {final_value}");
        }
        let mut wire = BytesMut::new();
        LoadMap1060 { final_value: 128, ..sample_1060() }.encode_frame(&mut wire);
        assert_eq!(wire.len(), 43);
    }

    #[test]
    fn load_map_1060_truncation_and_wrong_id_leave_input_unchanged() {
        let mut wire = BytesMut::new();
        sample_1060().encode_frame(&mut wire);
        for cut in 0..wire.len() {
            let original = &wire[..cut];
            let mut input = original;
            assert!(LoadMap1060::decode_frame(&mut input).is_err(), "cut {cut}");
            assert_eq!(input, original);
        }
        let mut other = wire.to_vec();
        other[0] ^= 1;
        let original = other.clone();
        let mut input = &other[..];
        assert!(LoadMap1060::decode_frame(&mut input).is_err());
        assert_eq!(input, &original[..]);
    }

    #[test]
    fn load_map_1060_angle_radians() {
        let zero = LoadMap1060 { angle: 0, ..sample_1060() };
        assert_eq!(zero.angle_radians(), 0.0);
        let full = LoadMap1060 { angle: 255, ..sample_1060() };
        assert!((full.angle_radians() - std::f32::consts::TAU).abs() < 1e-5);
        let half = LoadMap1060 { angle: 128, ..sample_1060() };
        assert!((half.angle_radians() - 2.0 * 128.0 * std::f32::consts::PI / 255.0).abs() < 1e-6);
    }

    #[test]
    fn load_map_then_load_agents_concatenate() {
        let mut wire = BytesMut::new();
        LoadMap1060 { final_value: 1, ..sample_1060() }.encode_frame(&mut wire);
        encode_empty(LOAD_AGENTS_1061, &mut wire);
        assert_eq!(wire.len(), 44);

        let mut input = &wire[..];
        LoadMap1060::decode_frame(&mut input).unwrap();
        assert_eq!(input.len(), 2);
        decode_empty(&mut input, LOAD_AGENTS_1061).unwrap();
        assert!(input.is_empty());

        let server = mission_messages(true);
        let mut census = &wire[..];
        let (first, _) = dynamic::decode_frame(&server, &Rules::default(), &mut census).unwrap();
        assert_eq!(first.id, 1060);
        assert_eq!(census.len(), 2);
        let (second, value) = dynamic::decode_frame(&server, &Rules::default(), &mut census).unwrap();
        assert_eq!(second.id, 1061);
        assert_eq!(value, Value::Struct(vec![]));
        assert!(census.is_empty());
    }

    #[test]
    fn begin_playing_1036_encodes_exactly() {
        let mut wire = BytesMut::new();
        encode_empty(BEGIN_PLAYING_1036, &mut wire);
        assert_eq!(&wire[..], &[0x0c, 0x04]);

        let server = mission_messages(true);
        let mut census = &wire[..];
        let (message, _) = dynamic::decode_frame(&server, &Rules::default(), &mut census).unwrap();
        assert_eq!(message.id, 1036);
        assert!(census.is_empty());
    }

    #[test]
    fn empty_frames_are_exact_and_transactional() {
        let mut wire = BytesMut::new();
        encode_empty(AGENTS_LOADED_412, &mut wire);
        wire.extend_from_slice(b"x");
        let mut input = &wire[..];
        decode_empty(&mut input, AGENTS_LOADED_412).unwrap();
        assert_eq!(input, b"x");

        let original = &wire[..];
        let mut input = original;
        assert!(decode_empty(&mut input, MAP_LOADED_420).is_err());
        assert_eq!(input, original);
        let mut short = &wire[..1];
        assert!(decode_empty(&mut short, AGENTS_LOADED_412).is_err());
        assert_eq!(short.len(), 1);
    }

    #[test]
    fn client_events_decode_and_match_schema() {
        let client = mission_messages(false);
        let cases: [(&[u8], ClientMapEvent, u16); 4] = [
            (
                &[0xa7, 0x01, 1, 2],
                ClientMapEvent::LanguagePrefs(LanguagePrefs423 { text: 1, audio: 2 }),
                423,
            ),
            (&[0xa4, 0x01], ClientMapEvent::MapLoaded, 420),
            (&[0xa2, 0x01], ClientMapEvent::FailedToLoad, 418),
            (&[0x9c, 0x01], ClientMapEvent::AgentsLoaded, 412),
        ];
        for (wire, event, id) in cases {
            let mut input = wire;
            assert_eq!(ClientMapEvent::decode_frame(&mut input).unwrap(), event);
            assert!(input.is_empty());

            let mut encoded = BytesMut::new();
            event.encode_frame(&mut encoded);
            assert_eq!(&encoded[..], wire);

            let mut census = wire;
            let (message, _) = dynamic::decode_frame(&client, &Rules::default(), &mut census).unwrap();
            assert_eq!(message.id, id);
            assert!(census.is_empty());
        }

        let mut census = &[0xa7u8, 0x01, 1, 2][..];
        let (_, value) = dynamic::decode_frame(&client, &Rules::default(), &mut census).unwrap();
        let Value::Struct(fields) = value else { panic!() };
        assert_eq!(fields[0].1, Value::U8(1));
        assert_eq!(fields[1].1, Value::U8(2));
    }

    #[test]
    fn client_events_concatenate_and_reject_unknown_ids() {
        let mut wire = BytesMut::new();
        ClientMapEvent::MapLoaded.encode_frame(&mut wire);
        ClientMapEvent::FailedToLoad.encode_frame(&mut wire);
        assert_eq!(wire.len(), 4);
        let mut input = &wire[..];
        assert_eq!(ClientMapEvent::decode_frame(&mut input).unwrap(), ClientMapEvent::MapLoaded);
        assert_eq!(ClientMapEvent::decode_frame(&mut input).unwrap(), ClientMapEvent::FailedToLoad);
        assert!(input.is_empty());

        let mut server_frame = BytesMut::new();
        sample_1060().encode_frame(&mut server_frame);
        let original = &server_frame[..];
        let mut input = original;
        assert!(ClientMapEvent::decode_frame(&mut input).is_err());
        assert_eq!(input, original);

        let mut short = &[0xa7u8, 0x01, 1][..];
        assert!(ClientMapEvent::decode_frame(&mut short).is_err());
        assert_eq!(short, &[0xa7, 0x01, 1]);
        let mut none = &[][..];
        assert!(ClientMapEvent::decode_frame(&mut none).is_err());
    }

    #[test]
    fn language_prefs_in_range_and_truncation() {
        assert!(LanguagePrefs423 { text: 0, audio: 5 }.in_range());
        assert!(!LanguagePrefs423 { text: 6, audio: 0 }.in_range());
        assert!(!LanguagePrefs423 { text: 0, audio: LANGUAGES }.in_range());

        let mut wire = BytesMut::new();
        LanguagePrefs423 { text: 3, audio: 4 }.encode_frame(&mut wire);
        for cut in 0..wire.len() {
            let original = &wire[..cut];
            let mut input = original;
            assert!(LanguagePrefs423::decode_frame(&mut input).is_err(), "cut {cut}");
            assert_eq!(input, original);
        }
    }
}
