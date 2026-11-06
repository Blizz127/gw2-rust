//! Build-209026 skill-chain state, server message 628.
//!
//! The client stores the resolved skill and raw count, then rebuilds the skill
//! bar if either changes. See `docs/re/209026-mesmer-sword-chain.md`.
//! This codec selects no progression, reset, count or timing policy.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillChainFrame209026 {
    /// Existing character with a skill bar, not an agent or player index.
    pub character_index: u32,
    /// Content key used as the current predecessor in palette-entry tests.
    pub skill_key: u32,
    /// Compared against each entry's threshold; preserve the full wire byte.
    pub raw_count: u8,
}

impl SkillChainFrame209026 {
    /// Append one complete frame, without validating gameplay state.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        628u16.encode(output);
        self.character_index.encode(output);
        self.skill_key.encode(output);
        self.raw_count.encode(output);
    }

    /// Consume one frame, leaving the input unchanged on any decoding error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != 628 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected skill-chain state message 628",
            ));
        }
        let value = Self {
            character_index: u32::decode(&mut cursor)?,
            skill_key: u32::decode(&mut cursor)?,
            raw_count: u8::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, Rules, Value};
    use crate::schema;

    fn fixtures() -> [(SkillChainFrame209026, &'static [u8]); 4] {
        [
            (
                SkillChainFrame209026 {
                    character_index: 300,
                    skill_key: 10170,
                    raw_count: 1,
                },
                &[0x74, 0x02, 0xac, 0x02, 0xba, 0x4f, 0x01],
            ),
            (
                SkillChainFrame209026 {
                    character_index: 0,
                    skill_key: u32::MAX,
                    raw_count: u8::MAX,
                },
                &[0x74, 0x02, 0, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff],
            ),
            (
                SkillChainFrame209026 {
                    character_index: u32::MAX,
                    skill_key: 0,
                    raw_count: 0,
                },
                &[0x74, 0x02, 0xff, 0xff, 0xff, 0xff, 0x0f, 0, 0],
            ),
            (
                SkillChainFrame209026 {
                    character_index: 128,
                    skill_key: 16384,
                    raw_count: 0x80,
                },
                &[0x74, 0x02, 0x80, 0x01, 0x80, 0x80, 0x01, 0x80],
            ),
        ]
    }

    #[test]
    fn chain_literal_frames_match_schema_and_keep_consecutive_boundaries() {
        let tables = schema::parse(include_str!(
            "../../../data/protocol/209026/server-tables.xml"
        ))
        .unwrap();
        let messages: Vec<_> = tables
            .protocols
            .into_iter()
            .filter(|p| p.name == "0")
            .flat_map(|p| p.groups)
            .filter(|g| g.name == "table-0x14216e940")
            .flat_map(|g| g.server)
            .collect();
        assert_eq!(messages.iter().filter(|m| m.id == 628).count(), 1);
        let stream: Vec<_> = fixtures()
            .iter()
            .flat_map(|(_, bytes)| bytes.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (expected, golden) in fixtures() {
            let mut output = BytesMut::from(&b"prefix"[..]);
            expected.encode_frame(&mut output);
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
            assert_eq!(
                SkillChainFrame209026::decode_frame(&mut input).unwrap(),
                expected
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
            let mut schema_input = golden;
            let (message, value) =
                dynamic::decode_frame(&messages, &Rules::default(), &mut schema_input).unwrap();
            assert_eq!(message.id, 628);
            assert_eq!(
                value,
                Value::Struct(vec![
                    ("0".into(), Value::U32(expected.character_index)),
                    ("1".into(), Value::U32(expected.skill_key)),
                    ("2".into(), Value::U8(expected.raw_count)),
                ])
            );
            assert!(schema_input.is_empty());
        }
        assert!(input.is_empty());
    }

    #[test]
    fn chain_preserves_every_raw_count_byte() {
        for raw_count in 0..=u8::MAX {
            let golden = [0x74, 0x02, 0, 0, raw_count];
            let expected = SkillChainFrame209026 {
                character_index: 0,
                skill_key: 0,
                raw_count,
            };
            let mut input = golden.as_slice();
            assert_eq!(
                SkillChainFrame209026::decode_frame(&mut input).unwrap(),
                expected
            );
            assert!(input.is_empty());
            let mut output = BytesMut::new();
            expected.encode_frame(&mut output);
            assert_eq!(output.as_ref(), golden);
        }
    }

    #[test]
    fn chain_rejects_truncation_wrong_ids_and_malformed_dwords_transactionally() {
        for (_, golden) in fixtures() {
            for cut in 0..golden.len() {
                let original = &golden[..cut];
                let mut input = original;
                assert_eq!(
                    SkillChainFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::UnexpectedEof
                );
                assert_eq!(input, original);
            }
        }
        for id in [0u16, 627, 629, 630, u16::MAX] {
            let original = id.to_le_bytes();
            let mut input = original.as_slice();
            assert_eq!(
                SkillChainFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, original);
        }
        for offset in [2, 3] {
            for malformed in [[0xff, 0xff, 0xff, 0xff, 0x10], [0x80; 5]] {
                let mut original = vec![0x74, 0x02, 0, 0, 0];
                original.splice(offset..offset + 1, malformed);
                let mut input = original.as_slice();
                assert_eq!(
                    SkillChainFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::InvalidData
                );
                assert_eq!(input, original);
            }
        }
    }
}
