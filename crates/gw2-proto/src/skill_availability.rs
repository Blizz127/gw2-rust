//! Build-209026 player skill availability, IDs 605, 606, 609 and 613.
//! See `docs/re/209026-skill-availability.md` for effects and dependencies.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillAvailabilityOperation209026 {
    /// 605 sets a bit without an immediate skill-bar rebuild.
    InitialSet(u32),
    /// 606 clears availability storage and a separate keyed table.
    Reset,
    /// 609 clears a bit and rebuilds the existing owned skill bar.
    Remove(u32),
    /// 613 sets a bit and rebuilds; preserve the raw notification flags.
    SetAndNotify { skill_key: u32, flags: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillAvailabilityFrame209026 {
    pub player_index: u32,
    pub operation: SkillAvailabilityOperation209026,
}

impl SkillAvailabilityFrame209026 {
    /// Append a frame without validating content keys or the player's state.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use SkillAvailabilityOperation209026::*;
        match self.operation {
            InitialSet(key) => {
                605u16.encode(output);
                key.encode(output);
            }
            Reset => 606u16.encode(output),
            Remove(key) => {
                609u16.encode(output);
                key.encode(output);
            }
            SetAndNotify { skill_key, flags } => {
                613u16.encode(output);
                skill_key.encode(output);
                flags.encode(output);
            }
        }
        self.player_index.encode(output);
    }

    /// Consume one frame transactionally; preserve raw flags and key values.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use SkillAvailabilityOperation209026::*;
        let mut cursor = *input;
        let operation = match u16::decode(&mut cursor)? {
            605 => InitialSet(u32::decode(&mut cursor)?),
            606 => Reset,
            609 => Remove(u32::decode(&mut cursor)?),
            613 => SetAndNotify {
                skill_key: u32::decode(&mut cursor)?,
                flags: u8::decode(&mut cursor)?,
            },
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "expected player skill-availability message 605, 606, 609 or 613",
                ));
            }
        };
        let player_index = u32::decode(&mut cursor)?;
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

    fn fixtures() -> [(Vec<u8>, SkillAvailabilityFrame209026); 4] {
        use SkillAvailabilityOperation209026::*;
        [
            ("5d 02 80 80 01 ac 02", 300, InitialSet(16_384)),
            ("5e 02 ff ff ff ff 0f", u32::MAX, Reset),
            ("61 02 ff ff ff ff 0f 00", 0, Remove(u32::MAX)),
            (
                "65 02 00 fe 80 01",
                128,
                SetAndNotify {
                    skill_key: 0,
                    flags: 254,
                },
            ),
        ]
        .map(|(text, player_index, operation)| {
            (
                text.split_whitespace()
                    .map(|b| u8::from_str_radix(b, 16).unwrap())
                    .collect(),
                SkillAvailabilityFrame209026 {
                    player_index,
                    operation,
                },
            )
        })
    }

    #[test]
    fn skill_availability_golden_layouts_preserve_values() {
        for (golden, frame) in fixtures() {
            let mut input = golden.as_slice();
            assert_eq!(
                SkillAvailabilityFrame209026::decode_frame(&mut input).unwrap(),
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
    fn skill_availability_consecutive_frames_preserve_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(b, _)| b.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, frame) in fixtures {
            assert_eq!(
                SkillAvailabilityFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn skill_availability_bad_frames_leave_input_unchanged() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(SkillAvailabilityFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        // Reject neighboring IDs and the holes between supported messages.
        for id in [604u16, 607, 608, 610, 611, 612, 614] {
            let mut bytes = id.to_le_bytes().to_vec();
            bytes.extend_from_slice(&[0; 12]);
            let mut input = bytes.as_slice();
            assert_eq!(
                SkillAvailabilityFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
        // Overflow/overlong encoding at every Dword position in every layout.
        for (prefix, middle) in [
            (&b"\x5d\x02"[..], &b""[..]),
            (&b"\x61\x02"[..], &b""[..]),
            (&b"\x65\x02"[..], &b"\xff"[..]),
        ] {
            for bad in [
                &b"\xff\xff\xff\xff\x10"[..],
                &b"\x80\x80\x80\x80\x80\x00"[..],
            ] {
                for bytes in [
                    [prefix, bad, middle, &[0]].concat(),
                    [prefix, &[0], middle, bad].concat(),
                    [&b"\x5e\x02"[..], bad].concat(),
                ] {
                    let mut input = bytes.as_slice();
                    assert_eq!(
                        SkillAvailabilityFrame209026::decode_frame(&mut input)
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
