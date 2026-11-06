//! Build-209026 per-skill recharge frames, IDs 630 and 631.
//! See `docs/re/209026-mesmer-skill-command-path.md` for client consumers.
//! These codecs choose no category, duration, pause value or skill behavior.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillRechargeOperation209026 {
    /// 630 updates or removes a record. The client clamps remaining to total;
    /// this wire codec preserves the caller's complete values without clamping.
    Update {
        remaining: u32,
        total: u32,
        /// Stored at recharge-record +0x28; gameplay meaning is unresolved.
        raw_word: u16,
    },
    /// 631 changes the pause state of an existing skill/category record.
    /// Preserve the complete byte rather than invent a restricted enum.
    Pause { raw_selector: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillRechargeFrame209026 {
    /// Existing character with a skill bar, not an agent or player index.
    pub character_index: u32,
    /// Content key resolved by the client, not a skill-bar slot number.
    pub skill_key: u32,
    /// Full wire byte. Valid gameplay categories must be established separately.
    pub raw_category: u8,
    pub operation: SkillRechargeOperation209026,
}

impl SkillRechargeFrame209026 {
    /// Append one complete frame without selecting or validating gameplay state.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use SkillRechargeOperation209026::*;
        let id: u16 = match self.operation {
            Update { .. } => 630,
            Pause { .. } => 631,
        };
        id.encode(output);
        self.character_index.encode(output);
        self.skill_key.encode(output);
        self.raw_category.encode(output);
        match self.operation {
            Update {
                remaining,
                total,
                raw_word,
            } => {
                remaining.encode(output);
                total.encode(output);
                raw_word.encode(output);
            }
            Pause { raw_selector } => raw_selector.encode(output),
        }
    }

    /// Consume one frame, leaving the input unchanged on any decoding error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use SkillRechargeOperation209026::*;
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !matches!(id, 630 | 631) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected per-skill recharge message 630 or 631",
            ));
        }
        let character_index = u32::decode(&mut cursor)?;
        let skill_key = u32::decode(&mut cursor)?;
        let raw_category = u8::decode(&mut cursor)?;
        let operation = match id {
            630 => Update {
                remaining: u32::decode(&mut cursor)?,
                total: u32::decode(&mut cursor)?,
                raw_word: u16::decode(&mut cursor)?,
            },
            631 => Pause {
                raw_selector: u8::decode(&mut cursor)?,
            },
            _ => unreachable!(),
        };
        *input = cursor;
        Ok(Self {
            character_index,
            skill_key,
            raw_category,
            operation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, Rules, Value};
    use crate::schema;

    fn fixtures() -> [(SkillRechargeFrame209026, &'static [u8]); 3] {
        use SkillRechargeOperation209026::*;
        // Literal descriptor-derived bytes, deliberately not retail values.
        [
            (
                SkillRechargeFrame209026 {
                    character_index: 300,
                    skill_key: 16384,
                    raw_category: 0xfe,
                    operation: Update {
                        remaining: u32::MAX,
                        total: 0,
                        raw_word: 0x81ff,
                    },
                },
                &[
                    0x76, 0x02, 0xac, 0x02, 0x80, 0x80, 0x01, 0xfe, 0xff, 0xff, 0xff, 0xff, 0x0f,
                    0x00, 0xff, 0x81,
                ],
            ),
            (
                SkillRechargeFrame209026 {
                    character_index: u32::MAX,
                    skill_key: 0,
                    raw_category: 0,
                    operation: Update {
                        remaining: 0,
                        total: u32::MAX,
                        raw_word: 0,
                    },
                },
                &[
                    0x76, 0x02, 0xff, 0xff, 0xff, 0xff, 0x0f, 0, 0, 0, 0xff, 0xff, 0xff, 0xff,
                    0x0f, 0, 0,
                ],
            ),
            (
                SkillRechargeFrame209026 {
                    character_index: 0,
                    skill_key: u32::MAX,
                    raw_category: 0xff,
                    operation: Pause { raw_selector: 0x80 },
                },
                &[0x77, 0x02, 0, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff, 0x80],
            ),
        ]
    }

    #[test]
    fn recharge_literal_wire_values_and_consecutive_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<_> = fixtures
            .iter()
            .flat_map(|(_, b)| b.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (expected, golden) in fixtures {
            let mut output = BytesMut::from(&b"prefix"[..]);
            expected.encode_frame(&mut output);
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
            assert_eq!(
                SkillRechargeFrame209026::decode_frame(&mut input).unwrap(),
                expected
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn recharge_literals_match_the_extracted_client_schema() {
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
        for (expected, golden) in fixtures() {
            let mut input = golden;
            let (message, value) =
                dynamic::decode_frame(&messages, &Rules::default(), &mut input).unwrap();
            let mut fields = vec![
                ("0".into(), Value::U32(expected.character_index)),
                ("1".into(), Value::U32(expected.skill_key)),
                ("2".into(), Value::U8(expected.raw_category)),
            ];
            let id = match expected.operation {
                SkillRechargeOperation209026::Update {
                    remaining,
                    total,
                    raw_word,
                } => {
                    fields.extend([
                        ("3".into(), Value::U32(remaining)),
                        ("4".into(), Value::U32(total)),
                        ("5".into(), Value::U16(raw_word)),
                    ]);
                    630
                }
                SkillRechargeOperation209026::Pause { raw_selector } => {
                    fields.push(("3".into(), Value::U8(raw_selector)));
                    631
                }
            };
            assert_eq!(messages.iter().filter(|m| m.id == id).count(), 1);
            assert_eq!(message.id, id);
            assert_eq!(value, Value::Struct(fields));
            assert!(input.is_empty());
        }
    }

    #[test]
    fn recharge_preserves_all_category_and_pause_bytes() {
        for raw in 0..=255u8 {
            let golden = [0x77, 0x02, 0, 0, raw, 255 - raw];
            let expected = SkillRechargeFrame209026 {
                character_index: 0,
                skill_key: 0,
                raw_category: raw,
                operation: SkillRechargeOperation209026::Pause {
                    raw_selector: 255 - raw,
                },
            };
            let mut input = golden.as_slice();
            assert_eq!(
                SkillRechargeFrame209026::decode_frame(&mut input).unwrap(),
                expected
            );
            let mut output = BytesMut::new();
            expected.encode_frame(&mut output);
            assert_eq!(output.as_ref(), golden);
        }
    }

    #[test]
    fn recharge_invalid_and_incomplete_frames_do_not_consume_input() {
        for (_, golden) in fixtures() {
            for cut in 0..golden.len() {
                let original = &golden[..cut];
                let mut input = original;
                assert_eq!(
                    SkillRechargeFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::UnexpectedEof
                );
                assert_eq!(input, original);
            }
        }
        for id in [629u16, 632, 627, 618] {
            let original = id.to_le_bytes();
            let mut input = original.as_slice();
            assert_eq!(
                SkillRechargeFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, original);
        }
        // Replace each Dword in a zero-valued630 frame with overflow or an
        // unterminated varint. Its fixed Word must not hide malformed framing.
        for offset in [2, 3, 5, 6] {
            for malformed in [[0xff, 0xff, 0xff, 0xff, 0x10], [0x80; 5]] {
                let mut original = vec![0x76, 0x02, 0, 0, 0, 0, 0, 0, 0];
                original.splice(offset..offset + 1, malformed);
                let mut input = original.as_slice();
                assert_eq!(
                    SkillRechargeFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::InvalidData
                );
                assert_eq!(input, original);
            }
        }
    }
}
