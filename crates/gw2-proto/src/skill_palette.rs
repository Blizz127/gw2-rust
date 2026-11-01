//! Build-209026 server 612: a player's skill palette selection.
//! See `docs/re/209026-player-skill-palettes.md` for ordering and value limits.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillPaletteFrame209026 {
    /// Type-65 content key; zero clears the selected palette pointer.
    pub palette_key: u32,
    /// Raw slot byte. Framing does not establish gameplay validity.
    pub slot: u8,
    /// Raw selector; the client uses 1/3 for one array and other values for another.
    pub context: u8,
    /// Player index, appearing last on the wire (not a character index).
    pub player_index: u32,
}

impl SkillPaletteFrame209026 {
    /// Append the frame without selecting content or validating client state.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        612u16.encode(output);
        self.palette_key.encode(output);
        self.slot.encode(output);
        self.context.encode(output);
        self.player_index.encode(output);
    }

    /// Consume exactly one complete frame, leaving input unchanged on error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != 612 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected player skill-palette message 612",
            ));
        }
        let frame = Self {
            palette_key: u32::decode(&mut cursor)?,
            slot: u8::decode(&mut cursor)?,
            context: u8::decode(&mut cursor)?,
            player_index: u32::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> [(Vec<u8>, SkillPaletteFrame209026); 3] {
        [
            (
                "64 02 80 80 01 04 03 ac 02",
                SkillPaletteFrame209026 {
                    palette_key: 16_384,
                    slot: 4,
                    context: 3,
                    player_index: 300,
                },
            ),
            (
                "64 02 00 00 01 00",
                SkillPaletteFrame209026 {
                    palette_key: 0,
                    slot: 0,
                    context: 1,
                    player_index: 0,
                },
            ),
            (
                "64 02 ff ff ff ff 0f fe ff ff ff ff ff 0f",
                SkillPaletteFrame209026 {
                    palette_key: u32::MAX,
                    slot: 254,
                    context: 255,
                    player_index: u32::MAX,
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
    fn skill_palette_golden_field_order_and_raw_values() {
        for (golden, frame) in fixtures() {
            let mut input = golden.as_slice();
            assert_eq!(
                SkillPaletteFrame209026::decode_frame(&mut input).unwrap(),
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
    fn skill_palette_consecutive_frames_preserve_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(b, _)| b.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, frame) in fixtures {
            assert_eq!(
                SkillPaletteFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn skill_palette_bad_frames_leave_input_unchanged() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(SkillPaletteFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        // Neighboring IDs and invalid varints in each Dword position.
        for bytes in [
            vec![0x63, 2, 0, 0, 0, 0],
            vec![0x65, 2, 0, 0, 0, 0],
            vec![0x64, 2, 0xff, 0xff, 0xff, 0xff, 0x10, 0, 0, 0],
            vec![0x64, 2, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0],
            vec![0x64, 2, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x10],
            vec![0x64, 2, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0],
        ] {
            let mut input = bytes.as_slice();
            assert_eq!(
                SkillPaletteFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
    }
}
