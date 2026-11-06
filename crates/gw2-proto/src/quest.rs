//! Build-209026 quest lifecycle (527/528/530/532), report (538), and steps (542/544).
//! See `docs/re/209026-quest-lifecycle.md` for the client consumers and limits,
//! `docs/re/209026-quest-step-state.md` for step consumers, and
//! `docs/re/209026-content-direction-startup.md` for report ordering.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// Wire values only; quest selection, lifetime and ordering belong to the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestLifecycleOperation209026 {
    /// 527 replaces the active quest with the specified type-57 definition.
    Create {
        quest_data_id: u32,
        /// Stored at quest +0x88. Bit 0 is completion; other meanings and the
        /// retail startup value remain unresolved. No bits are masked here.
        raw_byte: u8,
    },
    /// 528 notifies the manager's listeners; the client requires an active quest.
    Notify,
    /// 530 removes the active quest; the client requires a matching data ID.
    Remove { quest_data_id: u32 },
    /// 532 selects a goal ID from the active quest definition, not an array index.
    /// The client requires an incomplete quest and a matching goal definition.
    SelectGoal { goal_id: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuestLifecycleFrame209026 {
    /// Index of an existing player, not a character index or quest content key.
    pub player_index: u32,
    pub operation: QuestLifecycleOperation209026,
}

impl QuestLifecycleFrame209026 {
    /// Append one complete frame without selecting content or validating state.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use QuestLifecycleOperation209026::*;
        let id: u16 = match self.operation {
            Create { .. } => 527,
            Notify => 528,
            Remove { .. } => 530,
            SelectGoal { .. } => 532,
        };
        id.encode(output);
        self.player_index.encode(output);
        match self.operation {
            Create {
                quest_data_id,
                raw_byte,
            } => {
                quest_data_id.encode(output);
                raw_byte.encode(output);
            }
            Notify => {}
            Remove { quest_data_id } => quest_data_id.encode(output),
            SelectGoal { goal_id } => goal_id.encode(output),
        }
    }

    /// Consume one complete frame, leaving input unchanged on every error.
    /// The caller is responsible for the client's player and active-quest state.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use QuestLifecycleOperation209026::*;
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !matches!(id, 527 | 528 | 530 | 532) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected quest lifecycle message 527, 528, 530 or 532",
            ));
        }
        let player_index = u32::decode(&mut cursor)?;
        let operation = match id {
            527 => Create {
                quest_data_id: u32::decode(&mut cursor)?,
                raw_byte: u8::decode(&mut cursor)?,
            },
            528 => Notify,
            530 => Remove {
                quest_data_id: u32::decode(&mut cursor)?,
            },
            532 => SelectGoal {
                goal_id: u32::decode(&mut cursor)?,
            },
            _ => unreachable!(),
        };
        *input = cursor;
        Ok(Self {
            player_index,
            operation,
        })
    }
}

/// Raw step operations; the caller owns active-quest state and gameplay triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestStepOperation209026 {
    /// 542 sets the step's completion bit and evaluates goal completion.
    /// Requires an active incomplete quest, active goal and matching step ID.
    Complete { step_id: u32 },
    /// 544 replaces step state without notifying or evaluating goal completion.
    /// Requires an active incomplete quest, active goal and matching step ID.
    Restore {
        step_id: u32,
        /// Complete wire byte, stored at runtime step +0x10 without masking.
        raw_flags: u8,
        /// Runtime step +0x14, named m_objectiveId by the client assertion.
        objective_id: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuestStepFrame209026 {
    /// Index of an existing player, not a character index or quest content key.
    pub player_index: u32,
    /// Step IDs belong to the active goal's definitions, not array indices.
    pub operation: QuestStepOperation209026,
}

impl QuestStepFrame209026 {
    /// Append one complete frame without choosing a quest or validating state.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use QuestStepOperation209026::*;
        let id: u16 = match self.operation {
            Complete { .. } => 542,
            Restore { .. } => 544,
        };
        id.encode(output);
        self.player_index.encode(output);
        match self.operation {
            Complete { step_id } => step_id.encode(output),
            Restore {
                step_id,
                raw_flags,
                objective_id,
            } => {
                step_id.encode(output);
                raw_flags.encode(output);
                objective_id.encode(output);
            }
        }
    }

    /// Consume one complete frame, leaving input unchanged on every error.
    /// State prerequisites, step membership and duplicate handling are external.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use QuestStepOperation209026::*;
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !matches!(id, 542 | 544) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected quest step message 542 or 544",
            ));
        }
        let player_index = u32::decode(&mut cursor)?;
        let step_id = u32::decode(&mut cursor)?;
        let operation = match id {
            542 => Complete { step_id },
            544 => Restore {
                step_id,
                raw_flags: u8::decode(&mut cursor)?,
                objective_id: u32::decode(&mut cursor)?,
            },
            _ => unreachable!(),
        };
        *input = cursor;
        Ok(Self {
            player_index,
            operation,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuestManagerReportFrame209026 {
    /// Index of an existing player, not a character index or quest content key.
    pub player_index: u32,
}

impl QuestManagerReportFrame209026 {
    /// Append the report frame without validating the client's player state.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        538u16.encode(output);
        self.player_index.encode(output);
    }

    /// Consume one complete frame, leaving input unchanged on error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != 538 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected quest-manager report message 538",
            ));
        }
        let frame = Self {
            player_index: u32::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, Rules, Value};
    use crate::schema;

    fn lifecycle_fixtures() -> [(QuestLifecycleFrame209026, &'static [u8]); 9] {
        use QuestLifecycleOperation209026::*;
        // Literal descriptor-derived fixtures; numeric values are codec probes,
        // not a selection of real quest definitions or initial state.
        [
            (
                QuestLifecycleFrame209026 {
                    player_index: 0,
                    operation: Create {
                        quest_data_id: 0,
                        raw_byte: 0,
                    },
                },
                &[0x0f, 0x02, 0, 0, 0],
            ),
            (
                QuestLifecycleFrame209026 {
                    player_index: 300,
                    operation: Create {
                        quest_data_id: 16384,
                        raw_byte: 128,
                    },
                },
                &[0x0f, 0x02, 0xac, 0x02, 0x80, 0x80, 0x01, 0x80],
            ),
            (
                QuestLifecycleFrame209026 {
                    player_index: u32::MAX,
                    operation: Notify,
                },
                &[0x10, 0x02, 0xff, 0xff, 0xff, 0xff, 0x0f],
            ),
            (
                QuestLifecycleFrame209026 {
                    player_index: 0,
                    operation: Remove {
                        quest_data_id: u32::MAX,
                    },
                },
                &[0x12, 0x02, 0, 0xff, 0xff, 0xff, 0xff, 0x0f],
            ),
            (
                QuestLifecycleFrame209026 {
                    player_index: u32::MAX,
                    operation: Create {
                        quest_data_id: u32::MAX,
                        raw_byte: 255,
                    },
                },
                &[
                    0x0f, 0x02, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff,
                ],
            ),
            (
                QuestLifecycleFrame209026 {
                    player_index: 300,
                    operation: Remove { quest_data_id: 0 },
                },
                &[0x12, 0x02, 0xac, 0x02, 0],
            ),
            (
                QuestLifecycleFrame209026 {
                    player_index: 300,
                    operation: SelectGoal { goal_id: 1 },
                },
                &[0x14, 0x02, 0xac, 0x02, 1],
            ),
            (
                QuestLifecycleFrame209026 {
                    player_index: 0,
                    operation: SelectGoal { goal_id: u32::MAX },
                },
                &[0x14, 0x02, 0, 0xff, 0xff, 0xff, 0xff, 0x0f],
            ),
            (
                QuestLifecycleFrame209026 {
                    player_index: u32::MAX,
                    operation: SelectGoal { goal_id: 0 },
                },
                &[0x14, 0x02, 0xff, 0xff, 0xff, 0xff, 0x0f, 0],
            ),
        ]
    }

    #[test]
    fn quest_lifecycle_golden_values_and_stream_boundaries() {
        let fixtures = lifecycle_fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(_, bytes)| bytes.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (expected, golden) in fixtures {
            let mut output = BytesMut::from(&b"prefix"[..]);
            expected.encode_frame(&mut output);
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
            assert_eq!(
                QuestLifecycleFrame209026::decode_frame(&mut input).unwrap(),
                expected
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn quest_lifecycle_matches_extracted_client_schema() {
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
        for (expected, golden) in lifecycle_fixtures() {
            let mut input = golden;
            let (message, value) =
                dynamic::decode_frame(&messages, &Rules::default(), &mut input).unwrap();
            assert!(input.is_empty());
            let (id, fields) = match expected.operation {
                QuestLifecycleOperation209026::Create {
                    quest_data_id,
                    raw_byte,
                } => (
                    527,
                    vec![
                        ("0".into(), Value::U32(expected.player_index)),
                        ("1".into(), Value::U32(quest_data_id)),
                        ("2".into(), Value::U8(raw_byte)),
                    ],
                ),
                QuestLifecycleOperation209026::Notify => {
                    (528, vec![("0".into(), Value::U32(expected.player_index))])
                }
                QuestLifecycleOperation209026::Remove { quest_data_id } => (
                    530,
                    vec![
                        ("0".into(), Value::U32(expected.player_index)),
                        ("1".into(), Value::U32(quest_data_id)),
                    ],
                ),
                QuestLifecycleOperation209026::SelectGoal { goal_id } => (
                    532,
                    vec![
                        ("0".into(), Value::U32(expected.player_index)),
                        ("1".into(), Value::U32(goal_id)),
                    ],
                ),
            };
            assert_eq!(messages.iter().filter(|m| m.id == id).count(), 1);
            assert_eq!(message.id, id);
            assert_eq!(value, Value::Struct(fields));
        }
    }

    #[test]
    fn quest_lifecycle_incomplete_or_invalid_frames_are_atomic() {
        for (_, golden) in lifecycle_fixtures() {
            for cut in 0..golden.len() {
                let original = &golden[..cut];
                let mut input = original;
                assert_eq!(
                    QuestLifecycleFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::UnexpectedEof
                );
                assert_eq!(input, original);
            }
        }
        for id in [526u16, 529, 531, 533, 538] {
            let original = id.to_le_bytes();
            let mut input = &original[..];
            assert_eq!(
                QuestLifecycleFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, original);
        }
        // Exercise both Dword positions, including five-byte overflow and an
        // unterminated varint. Trailing bytes must not hide either error.
        for id in [527u16, 528, 530, 532] {
            for malformed in [[0xff, 0xff, 0xff, 0xff, 0x10], [0x80; 5]] {
                for content_field in [false, true] {
                    if content_field && id == 528 {
                        continue;
                    }
                    let mut original = id.to_le_bytes().to_vec();
                    if content_field {
                        original.push(0);
                    }
                    original.extend_from_slice(&malformed);
                    original.extend_from_slice(&[0, 0, 0]);
                    let mut input = original.as_slice();
                    assert_eq!(
                        QuestLifecycleFrame209026::decode_frame(&mut input)
                            .unwrap_err()
                            .kind(),
                        ErrorKind::InvalidData
                    );
                    assert_eq!(input, original);
                }
            }
        }
    }

    fn step_fixtures() -> [(QuestStepFrame209026, &'static [u8]); 7] {
        use QuestStepOperation209026::*;
        // Descriptor-derived literal bytes; these values select no encounter.
        [
            (
                QuestStepFrame209026 {
                    player_index: 0,
                    operation: Complete { step_id: 0 },
                },
                &[0x1e, 0x02, 0, 0],
            ),
            (
                QuestStepFrame209026 {
                    player_index: 300,
                    operation: Complete { step_id: 16384 },
                },
                &[0x1e, 0x02, 0xac, 0x02, 0x80, 0x80, 0x01],
            ),
            (
                QuestStepFrame209026 {
                    player_index: u32::MAX,
                    operation: Complete { step_id: u32::MAX },
                },
                &[
                    0x1e, 0x02, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff, 0xff, 0xff, 0xff, 0x0f,
                ],
            ),
            (
                QuestStepFrame209026 {
                    player_index: 0,
                    operation: Restore {
                        step_id: 0,
                        raw_flags: 0,
                        objective_id: 0,
                    },
                },
                &[0x20, 0x02, 0, 0, 0, 0],
            ),
            (
                QuestStepFrame209026 {
                    player_index: 300,
                    operation: Restore {
                        step_id: 16384,
                        raw_flags: 128,
                        objective_id: 268435456,
                    },
                },
                &[
                    0x20, 0x02, 0xac, 0x02, 0x80, 0x80, 0x01, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01,
                ],
            ),
            (
                QuestStepFrame209026 {
                    player_index: u32::MAX,
                    operation: Restore {
                        step_id: u32::MAX,
                        raw_flags: 255,
                        objective_id: u32::MAX,
                    },
                },
                &[
                    0x20, 0x02, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0x0f,
                ],
            ),
            (
                QuestStepFrame209026 {
                    player_index: 1,
                    operation: Restore {
                        step_id: 42,
                        raw_flags: 10,
                        objective_id: 300,
                    },
                },
                &[0x20, 0x02, 1, 42, 10, 0xac, 0x02],
            ),
        ]
    }

    #[test]
    fn quest_step_golden_values_and_stream_boundaries() {
        let fixtures = step_fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(_, bytes)| bytes.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (expected, golden) in fixtures {
            let mut output = BytesMut::from(&b"prefix"[..]);
            expected.encode_frame(&mut output);
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
            assert_eq!(
                QuestStepFrame209026::decode_frame(&mut input).unwrap(),
                expected
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn quest_step_matches_extracted_client_schema() {
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
        for (expected, golden) in step_fixtures() {
            let mut input = golden;
            let (message, value) =
                dynamic::decode_frame(&messages, &Rules::default(), &mut input).unwrap();
            assert!(input.is_empty());
            let (id, fields) = match expected.operation {
                QuestStepOperation209026::Complete { step_id } => (
                    542,
                    vec![
                        ("0".into(), Value::U32(expected.player_index)),
                        ("1".into(), Value::U32(step_id)),
                    ],
                ),
                QuestStepOperation209026::Restore {
                    step_id,
                    raw_flags,
                    objective_id,
                } => (
                    544,
                    vec![
                        ("0".into(), Value::U32(expected.player_index)),
                        ("1".into(), Value::U32(step_id)),
                        ("2".into(), Value::U8(raw_flags)),
                        ("3".into(), Value::U32(objective_id)),
                    ],
                ),
            };
            assert_eq!(messages.iter().filter(|m| m.id == id).count(), 1);
            assert_eq!(message.id, id);
            assert_eq!(value, Value::Struct(fields));
        }
    }

    #[test]
    fn quest_step_raw_flags_preserve_every_byte() {
        for raw_flags in 0..=u8::MAX {
            let frame = QuestStepFrame209026 {
                player_index: 0,
                operation: QuestStepOperation209026::Restore {
                    step_id: 0,
                    raw_flags,
                    objective_id: 1,
                },
            };
            let golden = [0x20, 0x02, 0, 0, raw_flags, 1];
            let mut output = BytesMut::new();
            frame.encode_frame(&mut output);
            assert_eq!(output.as_ref(), golden);
            let mut input = &golden[..];
            assert_eq!(
                QuestStepFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            assert!(input.is_empty());
        }
    }

    #[test]
    fn quest_step_incomplete_or_invalid_frames_are_atomic() {
        for (_, golden) in step_fixtures() {
            for cut in 0..golden.len() {
                let original = &golden[..cut];
                let mut input = original;
                assert_eq!(
                    QuestStepFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::UnexpectedEof
                );
                assert_eq!(input, original);
            }
        }
        for id in [527u16, 532, 538, 541, 543, 545, 546, 547] {
            let original = id.to_le_bytes();
            let mut input = &original[..];
            assert_eq!(
                QuestStepFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, original);
        }
        // Corrupt each Dword independently. Valid trailing bytes cannot hide
        // overflow/unterminated values or cause a partially consumed frame.
        for id in [542u16, 544] {
            let dword_count = if id == 544 { 3 } else { 2 };
            for field in 0..dword_count {
                for malformed in [[0xff, 0xff, 0xff, 0xff, 0x10], [0x80; 5]] {
                    let mut original = id.to_le_bytes().to_vec();
                    original.extend(std::iter::repeat_n(0, field));
                    if field == 2 {
                        original.push(0xff); // The preceding raw flags byte.
                    }
                    original.extend_from_slice(&malformed);
                    original.extend_from_slice(&[0, 0, 0]);
                    let mut input = original.as_slice();
                    assert_eq!(
                        QuestStepFrame209026::decode_frame(&mut input)
                            .unwrap_err()
                            .kind(),
                        ErrorKind::InvalidData
                    );
                    assert_eq!(input, original);
                }
            }
        }
    }

    #[test]
    fn quest_report_wire_values_and_stream_boundaries() {
        // Literal descriptor-derived fixtures include multi-byte and full-width indices.
        let fixtures: &[(u32, &[u8])] = &[
            (0, &[0x1a, 0x02, 0]),
            (300, &[0x1a, 0x02, 0xac, 0x02]),
            (u32::MAX, &[0x1a, 0x02, 0xff, 0xff, 0xff, 0xff, 0x0f]),
        ];
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(_, b)| b.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for &(player_index, golden) in fixtures {
            let frame = QuestManagerReportFrame209026 { player_index };
            let mut output = BytesMut::from(&b"prefix"[..]);
            frame.encode_frame(&mut output);
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
            assert_eq!(
                QuestManagerReportFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn quest_report_incomplete_or_invalid_frames_are_atomic() {
        let longest = [0x1a, 0x02, 0xff, 0xff, 0xff, 0xff, 0x0f];
        for cut in 0..longest.len() {
            let original = &longest[..cut];
            let mut input = original;
            assert_eq!(
                QuestManagerReportFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::UnexpectedEof
            );
            assert_eq!(input, original);
        }
        for original in [
            &[0x19, 0x02, 1][..],
            &[0x1b, 0x02, 1][..],
            &[0x1a, 0x02, 0xff, 0xff, 0xff, 0xff, 0x10][..],
            &[0x1a, 0x02, 0x80, 0x80, 0x80, 0x80, 0x80][..],
        ] {
            let mut input = original;
            assert_eq!(
                QuestManagerReportFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, original);
        }
    }
}
