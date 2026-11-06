//! Build-209026 research codecs for correlated server-84 skill-order updates.
//!
//! These are packed payloads for `agent_behavior::AgentBehaviorFrame209026`.
//! They choose no outer flags, correlation value, timing, result meaning or
//! gameplay sequence. See `docs/re/209026-typed78-completion-release.md`.

use std::io::{Error, ErrorKind, Result};

/// The source-supported packed 0x44, minimal 0x45, and 0x47 forms.
///
/// The native handlers require an existing typed 0x78 order with a matching
/// behavior value. Parsing bytes does not establish that runtime state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillOrderUpdatePayload209026 {
    /// Stores the raw byte at order +0xe8 and emits a notification.
    /// No success/error enumeration is established.
    Result { raw_result: u8 },
    /// Minimal opcode-only form. Defers release while order flag 0x100 is set,
    /// otherwise follows the native release path. No send timing is implied.
    Finish,
    /// Updates the step index and preserves the complete raw flag byte.
    /// Bits 1/0 control order masks 0x4000/0x8000; higher bits remain raw.
    Step { raw_step_index: u8, raw_flags: u8 },
}

impl SkillOrderUpdatePayload209026 {
    pub fn encode_payload(&self) -> Vec<u8> {
        match *self {
            Self::Result { raw_result } => vec![0x44, raw_result],
            Self::Finish => vec![0x45],
            Self::Step {
                raw_step_index,
                raw_flags,
            } => vec![0x47, raw_step_index, raw_flags],
        }
    }

    /// Decode an entire packed payload without consuming or changing input.
    ///
    /// The 0x44/0x47 handlers assert exact lengths 2/3. The 0x45 dispatcher
    /// admits at least one byte and its handler reads no further payload; this
    /// typed codec recognizes only the minimal one-byte form. Extended forms
    /// are rejected rather than silently discarded and remain available in
    /// the existing raw behavior carrier.
    pub fn decode_payload(bytes: &[u8]) -> Result<Self> {
        match *bytes {
            [0x44, raw_result] => Ok(Self::Result { raw_result }),
            [0x45] => Ok(Self::Finish),
            [0x47, raw_step_index, raw_flags] => Ok(Self::Step {
                raw_step_index,
                raw_flags,
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "invalid or unsupported skill-order update payload",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_behavior::AgentBehaviorFrame209026;
    use crate::dynamic::{self, CountPrefix, Rules, Value};
    use crate::{Encode, schema};
    use bytes::BytesMut;

    fn fixtures() -> [(SkillOrderUpdatePayload209026, Vec<u8>); 3] {
        [
            (SelfPayload::Result { raw_result: 0xfe }, vec![0x44, 0xfe]),
            (SelfPayload::Finish, vec![0x45]),
            (
                SelfPayload::Step {
                    raw_step_index: 0xff,
                    raw_flags: 0xa5,
                },
                vec![0x47, 0xff, 0xa5],
            ),
        ]
    }

    type SelfPayload = SkillOrderUpdatePayload209026;

    fn outer(payload: Vec<u8>) -> AgentBehaviorFrame209026 {
        AgentBehaviorFrame209026 {
            agent_index: 0x1234,
            remote_tick: 300,
            command_flags: 0xf3,
            behavior_value: 128,
            behavior_data: payload,
        }
    }

    #[test]
    fn literal_packed_forms_preserve_raw_bytes() {
        for (payload, wire) in fixtures() {
            assert_eq!(payload.encode_payload(), wire);
            assert_eq!(SelfPayload::decode_payload(&wire).unwrap(), payload);
        }
        for raw_result in 0..=u8::MAX {
            let wire = [0x44, raw_result];
            assert_eq!(
                SelfPayload::decode_payload(&wire).unwrap().encode_payload(),
                wire
            );
        }
        // Every step/flag pair remains representable, including high flag bits
        // with no assigned client semantics. No boolean normalization occurs.
        for raw_step_index in 0..=u8::MAX {
            for raw_flags in 0..=u8::MAX {
                let wire = [0x47, raw_step_index, raw_flags];
                assert_eq!(
                    SelfPayload::decode_payload(&wire).unwrap().encode_payload(),
                    wire
                );
            }
        }
    }

    #[test]
    fn packed_forms_reject_truncation_extensions_and_other_opcodes() {
        for (_, wire) in fixtures() {
            for length in 0..wire.len() {
                assert_eq!(
                    SelfPayload::decode_payload(&wire[..length])
                        .unwrap_err()
                        .kind(),
                    ErrorKind::InvalidData
                );
            }
            let mut extended = wire.clone();
            extended.push(0);
            assert!(SelfPayload::decode_payload(&extended).is_err());
            for opcode in 0..=u8::MAX {
                if [0x44, 0x45, 0x47].contains(&opcode) {
                    continue;
                }
                let mut other = wire.clone();
                other[0] = opcode;
                assert!(SelfPayload::decode_payload(&other).is_err());
            }
        }
        // The raw carrier retains unsupported extensions losslessly.
        let frame = outer(vec![0x45, 0xaa]);
        let mut output = BytesMut::new();
        frame.encode_frame(&mut output).unwrap();
        let decoded = AgentBehaviorFrame209026::decode_frame(&mut output.as_ref()).unwrap();
        assert_eq!(decoded, frame);
        assert!(SelfPayload::decode_payload(&decoded.behavior_data).is_err());
    }

    #[test]
    fn shared_server84_golden_frames_concatenate_and_truncate_transactionally() {
        let mut stream = Vec::new();
        for (payload, packed) in fixtures() {
            // Outer Dwords are varints; the agent Word and payload bytes are
            // literal. No extra field or implicit flag is added by the codec.
            let mut wire = vec![84, 0, 0x34, 0x12, 0xac, 2, 0xf3, 0x80, 1];
            wire.push(packed.len() as u8);
            wire.extend_from_slice(&packed);
            let mut output = BytesMut::from(&b"prefix"[..]);
            outer(payload.encode_payload())
                .encode_frame(&mut output)
                .unwrap();
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], wire);
            for length in 0..wire.len() {
                let mut input = &wire[..length];
                assert!(AgentBehaviorFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &wire[..length]);
            }
            stream.extend_from_slice(&wire);
        }
        stream.push(0xa5);
        let mut input = stream.as_slice();
        for (expected, packed) in fixtures() {
            let frame = AgentBehaviorFrame209026::decode_frame(&mut input).unwrap();
            assert_eq!(frame, outer(packed));
            assert_eq!(
                SelfPayload::decode_payload(&frame.behavior_data).unwrap(),
                expected
            );
        }
        assert_eq!(input, [0xa5]);
    }

    #[test]
    fn packed_forms_match_frozen_server84_schema() {
        let messages: Vec<_> = schema::parse(include_str!(
            "../../../data/protocol/209026/server-tables.xml"
        ))
        .unwrap()
        .protocols
        .into_iter()
        .filter(|p| p.name == "0")
        .flat_map(|p| p.groups)
        .filter(|g| g.name == "table-0x14211be70")
        .flat_map(|g| g.server)
        .collect();
        let rules = Rules {
            buffer_var_small: Some(CountPrefix::U8),
            ..Rules::default()
        };
        for (payload, packed) in fixtures() {
            let mut encoded = BytesMut::new();
            outer(payload.encode_payload())
                .encode_frame(&mut encoded)
                .unwrap();
            let mut input = encoded.as_ref();
            let (message, decoded) = dynamic::decode_frame(&messages, &rules, &mut input).unwrap();
            assert_eq!(message.id, 84);
            assert!(input.is_empty());
            assert_eq!(
                decoded,
                Value::Struct(vec![
                    ("0".into(), Value::U16(0x1234)),
                    ("1".into(), Value::U32(300)),
                    ("2".into(), Value::U8(0xf3)),
                    ("3".into(), Value::U32(128)),
                    ("4".into(), Value::Bytes(packed)),
                ])
            );
            let mut schema_encoded = BytesMut::new();
            message.id.encode(&mut schema_encoded);
            dynamic::encode_message(message, &rules, &decoded, &mut schema_encoded).unwrap();
            assert_eq!(schema_encoded, encoded);
        }
    }
}
