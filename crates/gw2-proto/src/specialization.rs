//! Build-209026 specialization/major-trait messages 640–643.
//! Wire IDs are content UIDs (+0x14), not data keys (+0x28).
//! See `docs/re/209026-specialization-messages.md` for state prerequisites.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecializationOperation209026 {
    Assign {
        specialization_uid: u32,
        slot: u8,
    },
    AssignMajorTrait {
        trait_uid: u32,
        specialization_slot: u8,
        major_trait_slot: u8,
    },
    Replace {
        specialization_uids: [u32; 3],
        /// Indexed by specialization slot, then major-trait slot.
        trait_uids: [[u32; 3]; 3],
        /// Raw final byte passed to notifications; meaning unresolved.
        notification: u8,
    },
    Swap {
        first_slot: u8,
        second_slot: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecializationFrame209026 {
    pub player_index: u32,
    pub operation: SpecializationOperation209026,
}

impl SpecializationFrame209026 {
    /// Append a frame, preserving raw UIDs, slots and notification values.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use SpecializationOperation209026::*;
        let id: u16 = match self.operation {
            Assign { .. } => 640,
            AssignMajorTrait { .. } => 641,
            Replace { .. } => 642,
            Swap { .. } => 643,
        };
        id.encode(output);
        self.player_index.encode(output);
        match self.operation {
            Assign {
                specialization_uid,
                slot,
            } => {
                specialization_uid.encode(output);
                slot.encode(output);
            }
            AssignMajorTrait {
                trait_uid,
                specialization_slot,
                major_trait_slot,
            } => {
                trait_uid.encode(output);
                specialization_slot.encode(output);
                major_trait_slot.encode(output);
            }
            Replace {
                specialization_uids,
                trait_uids,
                notification,
            } => {
                for uid in specialization_uids {
                    uid.encode(output);
                }
                for row in trait_uids {
                    for uid in row {
                        uid.encode(output);
                    }
                }
                notification.encode(output);
            }
            Swap {
                first_slot,
                second_slot,
            } => {
                first_slot.encode(output);
                second_slot.encode(output);
            }
        }
    }

    /// Consume one complete frame transactionally, without state validation.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use SpecializationOperation209026::*;
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !(640..=643).contains(&id) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected specialization message 640–643",
            ));
        }
        let player_index = u32::decode(&mut cursor)?;
        let operation = match id {
            640 => Assign {
                specialization_uid: u32::decode(&mut cursor)?,
                slot: u8::decode(&mut cursor)?,
            },
            641 => AssignMajorTrait {
                trait_uid: u32::decode(&mut cursor)?,
                specialization_slot: u8::decode(&mut cursor)?,
                major_trait_slot: u8::decode(&mut cursor)?,
            },
            642 => {
                let mut specialization_uids = [0; 3];
                for uid in &mut specialization_uids {
                    *uid = u32::decode(&mut cursor)?;
                }
                let mut trait_uids = [[0; 3]; 3];
                for row in &mut trait_uids {
                    for uid in row {
                        *uid = u32::decode(&mut cursor)?;
                    }
                }
                Replace {
                    specialization_uids,
                    trait_uids,
                    notification: u8::decode(&mut cursor)?,
                }
            }
            643 => Swap {
                first_slot: u8::decode(&mut cursor)?,
                second_slot: u8::decode(&mut cursor)?,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> [(Vec<u8>, SpecializationFrame209026); 4] {
        use SpecializationOperation209026::*;
        [
            (
                "80 02 ff ff ff ff 0f 80 80 01 ff",
                u32::MAX,
                Assign {
                    specialization_uid: 16_384,
                    slot: 255,
                },
            ),
            (
                "81 02 00 ff ff ff ff 0f 02 01",
                0,
                AssignMajorTrait {
                    trait_uid: u32::MAX,
                    specialization_slot: 2,
                    major_trait_slot: 1,
                },
            ),
            (
                "82 02 ac 02 01 80 01 80 80 01
                 00 7f 81 01 ff 7f 81 80 01 ff ff ff ff 0f
                 02 ff 01 80 80 04 fe",
                300,
                Replace {
                    specialization_uids: [1, 128, 16_384],
                    trait_uids: [[0, 127, 129], [16_383, 16_385, u32::MAX], [2, 255, 65_536]],
                    notification: 254,
                },
            ),
            (
                "83 02 80 01 02 00",
                128,
                Swap {
                    first_slot: 2,
                    second_slot: 0,
                },
            ),
        ]
        .map(|(text, player_index, operation)| {
            (
                text.split_whitespace()
                    .map(|b| u8::from_str_radix(b, 16).unwrap())
                    .collect(),
                SpecializationFrame209026 {
                    player_index,
                    operation,
                },
            )
        })
    }

    #[test]
    fn specialization_golden_layouts_preserve_values() {
        for (golden, frame) in fixtures() {
            let mut input = golden.as_slice();
            assert_eq!(
                SpecializationFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            assert!(input.is_empty());
            let mut output = BytesMut::from(&b"prefix"[..]);
            frame.encode_frame(&mut output);
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
        }
    }

    #[test]
    fn specialization_consecutive_frames_preserve_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(b, _)| b.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, frame) in fixtures {
            assert_eq!(
                SpecializationFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn specialization_bad_frames_leave_input_unchanged() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(SpecializationFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        for id in [639u16, 644] {
            let mut bytes = id.to_le_bytes().to_vec();
            bytes.extend_from_slice(&[0; 20]);
            let mut input = bytes.as_slice();
            assert_eq!(
                SpecializationFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
        // Zero-valued layouts have one byte per Dword. Inject invalid varints
        // at every Dword, including every element of both fixed bulk arrays.
        for (id, length, dwords) in [(640u16, 5, 2), (641, 6, 2), (642, 16, 13), (643, 5, 1)] {
            let mut base = id.to_le_bytes().to_vec();
            base.resize(length, 0);
            for position in 2..2 + dwords {
                for bad in [
                    &b"\xff\xff\xff\xff\x10"[..],
                    &b"\x80\x80\x80\x80\x80\x00"[..],
                ] {
                    let bytes = [&base[..position], bad, &base[position + 1..]].concat();
                    let mut input = bytes.as_slice();
                    assert_eq!(
                        SpecializationFrame209026::decode_frame(&mut input)
                            .unwrap_err()
                            .kind(),
                        ErrorKind::InvalidData
                    );
                    assert_eq!(input, bytes);
                }
            }
        }
    }
}
