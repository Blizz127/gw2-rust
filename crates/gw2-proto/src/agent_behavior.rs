//! Build-209026 server-84/client-23 behavior frames and packed health transitions.
//! See `docs/re/209026-health-transitions.md`; framing does not validate game state.
//! Serializer 0x14103df90 selects client 23 for context type 1, otherwise server 84.
//! Client payloads remain opaque; the health-transition helpers apply separately.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBehaviorFrame209026 {
    pub agent_index: u16,
    /// Sender's world tick. The field name preserves the original server API.
    pub remote_tick: u32,
    /// On server 84, mask 0x10 selects the traced character transition dispatcher.
    /// Client 23 retains all raw flags without assigning payload semantics.
    /// Scheduling and lifecycle requirements remain caller decisions.
    pub command_flags: u8,
    pub behavior_value: u32,
    pub behavior_data: Vec<u8>,
}

impl AgentBehaviorFrame209026 {
    /// Existing server-to-client message ID.
    pub const MESSAGE_ID: u16 = 84;
    pub const CLIENT_MESSAGE_ID: u16 = 23;
    pub const MAX_BEHAVIOR_BYTES: usize = 250;

    /// Append server 84, leaving output unchanged for an invalid length.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        self.encode_frame_with_id(Self::MESSAGE_ID, output)
    }

    /// Append client 23 with the same raw fields and byte-counted payload.
    /// Output is unchanged for an invalid length.
    pub fn encode_client_frame(&self, output: &mut BytesMut) -> Result<()> {
        self.encode_frame_with_id(Self::CLIENT_MESSAGE_ID, output)
    }

    fn encode_frame_with_id(&self, message_id: u16, output: &mut BytesMut) -> Result<()> {
        if self.behavior_data.len() > Self::MAX_BEHAVIOR_BYTES {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "behavior exceeds 250 bytes",
            ));
        }
        message_id.encode(output);
        self.agent_index.encode(output);
        self.remote_tick.encode(output);
        self.command_flags.encode(output);
        self.behavior_value.encode(output);
        (self.behavior_data.len() as u8).encode(output);
        output.extend_from_slice(&self.behavior_data);
        Ok(())
    }

    /// Consume one server-84 frame. Input is unchanged on every decoding error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        Self::decode_frame_with_id(input, Self::MESSAGE_ID)
    }

    /// Consume one client-23 frame without interpreting the payload.
    /// Input is unchanged on every decoding error, including a server-84 ID.
    pub fn decode_client_frame(input: &mut &[u8]) -> Result<Self> {
        Self::decode_frame_with_id(input, Self::CLIENT_MESSAGE_ID)
    }

    fn decode_frame_with_id(input: &mut &[u8], message_id: u16) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != message_id {
            return Err(Error::new(
                ErrorKind::InvalidData,
                if message_id == Self::MESSAGE_ID {
                    "expected behavior message 84"
                } else {
                    "expected behavior message 23"
                },
            ));
        }
        let agent_index = u16::decode(&mut cursor)?;
        let remote_tick = u32::decode(&mut cursor)?;
        let command_flags = u8::decode(&mut cursor)?;
        let behavior_value = u32::decode(&mut cursor)?;
        let length = u8::decode(&mut cursor)? as usize;
        if length > Self::MAX_BEHAVIOR_BYTES {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "behavior exceeds 250 bytes",
            ));
        }
        if cursor.len() < length {
            return Err(ErrorKind::UnexpectedEof.into());
        }
        let frame = Self {
            agent_index,
            remote_tick,
            command_flags,
            behavior_value,
            behavior_data: cursor[..length].to_vec(),
        };
        *input = &cursor[length..];
        Ok(frame)
    }
}

/// Shared packed fields of dead/downed payloads. Names remain deliberately raw
/// where the client trace establishes an event argument but not its meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthTransitionDetails209026 {
    /// Downed's event reads bit 0; the packed codec retains all bits.
    pub detail_word: u16,
    pub detail_byte: u8,
    pub content_key: u32,
    pub agent_id: u32,
    /// The event converts this byte to a nonzero/zero boolean.
    pub trailing_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthTransitionPayload209026 {
    Dead(HealthTransitionDetails209026),
    Downed(HealthTransitionDetails209026),
    Alive {
        detail_byte: u8,
        /// The event converts this byte to a nonzero/zero boolean.
        trailing_byte: u8,
    },
}

impl HealthTransitionPayload209026 {
    /// Encode packed memory layout, including opcode. Inner u32s are fixed
    /// little-endian, unlike the varints in the outer message.
    pub fn encode_payload(&self) -> Vec<u8> {
        let (opcode, details) = match *self {
            Self::Dead(details) => (0x33, details),
            Self::Downed(details) => (0x34, details),
            Self::Alive {
                detail_byte,
                trailing_byte,
            } => {
                return vec![0x49, detail_byte, trailing_byte];
            }
        };
        let mut bytes = Vec::with_capacity(13);
        bytes.push(opcode);
        bytes.extend_from_slice(&details.detail_word.to_le_bytes());
        bytes.push(details.detail_byte);
        bytes.extend_from_slice(&details.content_key.to_le_bytes());
        bytes.extend_from_slice(&details.agent_id.to_le_bytes());
        bytes.push(details.trailing_byte);
        bytes
    }

    /// Parse exactly one behavior payload; rejects trailing bytes, unknown
    /// opcodes and every length other than the client's asserted 13 or 3.
    pub fn decode_payload(bytes: &[u8]) -> Result<Self> {
        match bytes.first().copied() {
            Some(0x49) if bytes.len() == 3 => Ok(Self::Alive {
                detail_byte: bytes[1],
                trailing_byte: bytes[2],
            }),
            Some(opcode @ (0x33 | 0x34)) if bytes.len() == 13 => {
                let details = HealthTransitionDetails209026 {
                    detail_word: u16::from_le_bytes(bytes[1..3].try_into().unwrap()),
                    detail_byte: bytes[3],
                    content_key: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
                    agent_id: u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
                    trailing_byte: bytes[12],
                };
                Ok(if opcode == 0x33 {
                    Self::Dead(details)
                } else {
                    Self::Downed(details)
                })
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "invalid health-transition payload",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dynamic, schema};
    use dynamic::{CountPrefix, Rules, Value};

    #[test]
    fn client_behavior_golden_concatenation_and_direction() {
        let wire = [23, 0, 0x34, 0x12, 0xac, 2, 0xf3, 0x80, 1, 3, 0xa5, 0, 0xff];
        let expected = AgentBehaviorFrame209026 {
            agent_index: 0x1234,
            remote_tick: 300,
            command_flags: 0xf3,
            behavior_value: 128,
            behavior_data: vec![0xa5, 0, 0xff],
        };
        let mut output = BytesMut::from(&b"prefix"[..]);
        expected.encode_client_frame(&mut output).unwrap();
        assert_eq!(&output[..6], b"prefix");
        assert_eq!(&output[6..], wire);

        let mut server_wire = BytesMut::new();
        expected.encode_frame(&mut server_wire).unwrap();
        assert_eq!(&server_wire[..2], &[84, 0]);
        assert_eq!(&server_wire[2..], &wire[2..]);
        let mut wrong_direction = wire.as_slice();
        assert!(AgentBehaviorFrame209026::decode_frame(&mut wrong_direction).is_err());
        assert_eq!(wrong_direction, wire);
        let mut wrong_direction = server_wire.as_ref();
        assert!(AgentBehaviorFrame209026::decode_client_frame(&mut wrong_direction).is_err());
        assert_eq!(wrong_direction, server_wire.as_ref());

        let mut stream = wire.to_vec();
        stream.extend_from_slice(&wire);
        stream.push(0xa5);
        let mut input = stream.as_slice();
        for _ in 0..2 {
            assert_eq!(
                AgentBehaviorFrame209026::decode_client_frame(&mut input).unwrap(),
                expected
            );
        }
        assert_eq!(input, [0xa5]);
        for n in 0..wire.len() {
            let mut input = &wire[..n];
            assert!(AgentBehaviorFrame209026::decode_client_frame(&mut input).is_err());
            assert_eq!(input, &wire[..n]);
        }
    }

    #[test]
    fn client_behavior_bounds_and_errors_are_transactional() {
        for length in [0, 250] {
            let data: Vec<_> = (0..length).map(|x| x as u8).collect();
            let frame = AgentBehaviorFrame209026 {
                agent_index: u16::MAX,
                remote_tick: u32::MAX,
                command_flags: 0xff,
                behavior_value: u32::MAX,
                behavior_data: data.clone(),
            };
            let mut wire = vec![
                23,
                0,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0x0f,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0x0f,
                length as u8,
            ];
            wire.extend_from_slice(&data);
            let mut output = BytesMut::new();
            frame.encode_client_frame(&mut output).unwrap();
            assert_eq!(output.as_ref(), wire);
            let mut input = wire.as_slice();
            assert_eq!(
                AgentBehaviorFrame209026::decode_client_frame(&mut input).unwrap(),
                frame
            );
            assert!(input.is_empty());
        }
        let oversized = AgentBehaviorFrame209026 {
            agent_index: 0,
            remote_tick: 0,
            command_flags: 0,
            behavior_value: 0,
            behavior_data: vec![0; 251],
        };
        let mut output = BytesMut::from(&b"keep"[..]);
        assert_eq!(
            oversized
                .encode_client_frame(&mut output)
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(output.as_ref(), b"keep");
        let invalid: &[&[u8]] = &[
            &[84, 0, 0, 0, 0, 0, 0, 0],                            // server direction
            &[23, 0, 0, 0, 0, 0, 0, 251],                          // excessive count
            &[23, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x1f, 0, 0, 0], // tick overflow
            &[23, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x1f, 0], // behavior overflow
        ];
        for original in invalid {
            let mut input = *original;
            assert!(AgentBehaviorFrame209026::decode_client_frame(&mut input).is_err());
            assert_eq!(input, *original);
        }
    }

    #[test]
    fn behavior_directions_match_extracted_schemas() {
        let rules = Rules {
            buffer_var_small: Some(CountPrefix::U8),
            ..Rules::default()
        };
        let expected = Value::Struct(vec![
            ("0".into(), Value::U16(0x1234)),
            ("1".into(), Value::U32(300)),
            ("2".into(), Value::U8(0xf3)),
            ("3".into(), Value::U32(128)),
            ("4".into(), Value::Bytes(vec![0xa5, 0, 0xff])),
        ]);
        for (client, xml, group) in [
            (
                true,
                include_str!("../../../data/protocol/209026/agent-client.xml"),
                "0",
            ),
            (
                true,
                include_str!("../../../data/protocol/209026/client-tables.xml"),
                "ctable-0x14211bdb0",
            ),
            (
                false,
                include_str!("../../../data/protocol/209026/server-tables.xml"),
                "table-0x14211be70",
            ),
        ] {
            let messages: Vec<_> = schema::parse(xml)
                .unwrap()
                .protocols
                .into_iter()
                .filter(|p| p.name == "0")
                .flat_map(|p| p.groups)
                .filter(|g| g.name == group)
                .flat_map(|g| if client { g.client } else { g.server })
                .collect();
            let id = if client { 23 } else { 84 };
            assert_eq!(messages.iter().filter(|m| m.id == id).count(), 1);
            let wire = [
                id as u8, 0, 0x34, 0x12, 0xac, 2, 0xf3, 0x80, 1, 3, 0xa5, 0, 0xff,
            ];
            let mut input = wire.as_slice();
            let (message, value) = dynamic::decode_frame(&messages, &rules, &mut input).unwrap();
            assert_eq!(message.id, id);
            assert_eq!(message.fields[4].kind, "BufferVarSmall");
            assert_eq!(message.fields[4].size, Some(250));
            assert_eq!(value, expected);
            assert!(input.is_empty());
            let mut output = BytesMut::new();
            message.id.encode(&mut output);
            dynamic::encode_message(message, &rules, &value, &mut output).unwrap();
            assert_eq!(output.as_ref(), wire);
        }
    }

    #[test]
    fn behavior_frame_golden_and_concatenation() {
        // Header Dwords are varints; agent Word and inner words are fixed LE.
        let wire = [
            0x54, 0, 0x34, 0x12, 0xac, 2, 0x10, 0x80, 1, 3, 0x49, 0xa5, 0xff,
        ];
        let expected = AgentBehaviorFrame209026 {
            agent_index: 0x1234,
            remote_tick: 300,
            command_flags: 0x10,
            behavior_value: 128,
            behavior_data: vec![0x49, 0xa5, 0xff],
        };
        let mut output = BytesMut::from(&b"prefix"[..]);
        expected.encode_frame(&mut output).unwrap();
        assert_eq!(&output[6..], wire);
        let mut stream = wire.to_vec();
        stream.extend_from_slice(&wire);
        stream.push(0xa5);
        let mut input = stream.as_slice();
        assert_eq!(
            AgentBehaviorFrame209026::decode_frame(&mut input).unwrap(),
            expected
        );
        assert_eq!(
            AgentBehaviorFrame209026::decode_frame(&mut input).unwrap(),
            expected
        );
        assert_eq!(input, [0xa5]);
        for n in 0..wire.len() {
            let mut input = &wire[..n];
            assert!(AgentBehaviorFrame209026::decode_frame(&mut input).is_err());
            assert_eq!(input, &wire[..n]);
        }
    }

    #[test]
    fn behavior_frame_bounds_and_errors_are_transactional() {
        for length in [0, 250] {
            let data: Vec<_> = (0..length).map(|x| x as u8).collect();
            let frame = AgentBehaviorFrame209026 {
                agent_index: u16::MAX,
                remote_tick: u32::MAX,
                command_flags: 0xff,
                behavior_value: u32::MAX,
                behavior_data: data.clone(),
            };
            let mut wire = vec![
                0x54,
                0,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0x0f,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0x0f,
                length as u8,
            ];
            wire.extend_from_slice(&data);
            let mut out = BytesMut::new();
            frame.encode_frame(&mut out).unwrap();
            assert_eq!(out.as_ref(), wire);
            let mut input = wire.as_slice();
            assert_eq!(
                AgentBehaviorFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            assert!(input.is_empty());
        }
        let frame = AgentBehaviorFrame209026 {
            agent_index: 0,
            remote_tick: 0,
            command_flags: 0,
            behavior_value: 0,
            behavior_data: vec![0; 251],
        };
        let mut output = BytesMut::from(&b"keep"[..]);
        assert!(frame.encode_frame(&mut output).is_err());
        assert_eq!(output.as_ref(), b"keep");
        let bad: &[&[u8]] = &[
            &[42, 0, 0, 0, 0, 0, 0, 0],                            // wrong outer ID
            &[84, 0, 0, 0, 0, 0, 0, 251],                          // excessive count
            &[84, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x1f, 0, 0, 0], // tick overflow
            &[84, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x1f, 0], // behavior overflow
        ];
        for original in bad {
            let mut input = *original;
            assert!(AgentBehaviorFrame209026::decode_frame(&mut input).is_err());
            assert_eq!(input, *original);
        }
    }

    #[test]
    fn health_payload_golden_exact_lengths_and_raw_fields() {
        let details = HealthTransitionDetails209026 {
            detail_word: 0x8134,
            detail_byte: 0xa5,
            content_key: 0x12345678,
            agent_id: 0xfedcba98,
            trailing_byte: 0xff,
        };
        let fixtures = [
            (
                HealthTransitionPayload209026::Dead(details),
                vec![
                    0x33, 0x34, 0x81, 0xa5, 0x78, 0x56, 0x34, 0x12, 0x98, 0xba, 0xdc, 0xfe, 0xff,
                ],
            ),
            (
                HealthTransitionPayload209026::Downed(details),
                vec![
                    0x34, 0x34, 0x81, 0xa5, 0x78, 0x56, 0x34, 0x12, 0x98, 0xba, 0xdc, 0xfe, 0xff,
                ],
            ),
            (
                HealthTransitionPayload209026::Alive {
                    detail_byte: 0xa5,
                    trailing_byte: 0xff,
                },
                vec![0x49, 0xa5, 0xff],
            ),
        ];
        for (payload, wire) in fixtures {
            assert_eq!(payload.encode_payload(), wire);
            assert_eq!(
                HealthTransitionPayload209026::decode_payload(&wire).unwrap(),
                payload
            );
            for n in 0..wire.len() {
                assert!(HealthTransitionPayload209026::decode_payload(&wire[..n]).is_err());
            }
            let mut bad = wire.clone();
            bad.push(0);
            assert!(HealthTransitionPayload209026::decode_payload(&bad).is_err());
            bad = wire;
            bad[0] = 0x35;
            assert!(HealthTransitionPayload209026::decode_payload(&bad).is_err());
        }
    }
}
