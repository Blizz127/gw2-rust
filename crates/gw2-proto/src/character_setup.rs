//! Build-209026 character setup frames: 151, 170, 282, 443 and 444.
//! See `docs/re/209026-character-setup-frames.md` for layout and state limits.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterSetupFrame209026 {
    Attitude {
        character_index: u32,
        attitude: u32,
    },
    Finalize {
        character_index: u32,
    },
    CreateInventory {
        character_index: u32,
    },
    AssignOwned {
        player_index: u32,
        character_index: u32,
    },
    /// Equal A/B selects the observed controlled-character path; retain both.
    AssignControl {
        a: u32,
        b: u32,
        character_index: u32,
    },
}

impl CharacterSetupFrame209026 {
    /// Append a frame without validating values or component prerequisites.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use CharacterSetupFrame209026::*;
        match *self {
            Attitude {
                character_index,
                attitude,
            } => {
                151u16.encode(output);
                character_index.encode(output);
                attitude.encode(output);
            }
            Finalize { character_index } => {
                170u16.encode(output);
                character_index.encode(output);
            }
            CreateInventory { character_index } => {
                282u16.encode(output);
                character_index.encode(output);
            }
            AssignOwned {
                player_index,
                character_index,
            } => {
                443u16.encode(output);
                player_index.encode(output);
                character_index.encode(output);
            }
            AssignControl {
                a,
                b,
                character_index,
            } => {
                444u16.encode(output);
                a.encode(output);
                b.encode(output);
                character_index.encode(output);
            }
        }
    }

    /// Consume one complete frame; leave input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use CharacterSetupFrame209026::*;
        let mut cursor = *input;
        let frame = match u16::decode(&mut cursor)? {
            151 => Attitude {
                character_index: u32::decode(&mut cursor)?,
                attitude: u32::decode(&mut cursor)?,
            },
            170 => Finalize {
                character_index: u32::decode(&mut cursor)?,
            },
            282 => CreateInventory {
                character_index: u32::decode(&mut cursor)?,
            },
            443 => AssignOwned {
                player_index: u32::decode(&mut cursor)?,
                character_index: u32::decode(&mut cursor)?,
            },
            444 => AssignControl {
                a: u32::decode(&mut cursor)?,
                b: u32::decode(&mut cursor)?,
                character_index: u32::decode(&mut cursor)?,
            },
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "expected character setup message 151, 170, 282, 443 or 444",
                ));
            }
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> [(Vec<u8>, CharacterSetupFrame209026); 6] {
        use CharacterSetupFrame209026::*;
        [
            (
                "97 00 ac 02 ff ff ff ff 0f",
                Attitude {
                    character_index: 300,
                    attitude: u32::MAX,
                },
            ),
            (
                "aa 00 80 01",
                Finalize {
                    character_index: 128,
                },
            ),
            ("1a 01 00", CreateInventory { character_index: 0 }),
            (
                "bb 01 01 ac 02",
                AssignOwned {
                    player_index: 1,
                    character_index: 300,
                },
            ),
            (
                "bc 01 01 02 80 80 01",
                AssignControl {
                    a: 1,
                    b: 2,
                    character_index: 16_384,
                },
            ),
            (
                "bc 01 ff ff ff ff 0f ff ff ff ff 0f 00",
                AssignControl {
                    a: u32::MAX,
                    b: u32::MAX,
                    character_index: 0,
                },
            ),
        ]
        .map(|(text, frame)| {
            (
                text.split_whitespace()
                    .map(|b| u8::from_str_radix(b, 16).unwrap())
                    .collect(),
                frame,
            )
        })
    }

    #[test]
    fn character_setup_golden_layouts_preserve_distinct_fields() {
        for (golden, frame) in fixtures() {
            let mut input = golden.as_slice();
            assert_eq!(
                CharacterSetupFrame209026::decode_frame(&mut input).unwrap(),
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
    fn character_setup_consecutive_frames_preserve_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(b, _)| b.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, frame) in fixtures {
            assert_eq!(
                CharacterSetupFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn character_setup_bad_frames_leave_input_unchanged() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(CharacterSetupFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        for id in [150u16, 152, 169, 171, 281, 283, 442, 445] {
            let mut bytes = id.to_le_bytes().to_vec();
            bytes.extend_from_slice(&[0; 15]);
            let mut input = bytes.as_slice();
            assert_eq!(
                CharacterSetupFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
        // Each position is independently malformed, not merely the first Dword.
        for (id, count) in [(151u16, 2), (170, 1), (282, 1), (443, 2), (444, 3)] {
            for field in 0..count {
                for bad in [
                    &b"\xff\xff\xff\xff\x10"[..],
                    &b"\x80\x80\x80\x80\x80\x00"[..],
                ] {
                    let mut bytes = id.to_le_bytes().to_vec();
                    for i in 0..count {
                        bytes.extend_from_slice(if i == field { bad } else { &[0] });
                    }
                    let mut input = bytes.as_slice();
                    assert_eq!(
                        CharacterSetupFrame209026::decode_frame(&mut input)
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
