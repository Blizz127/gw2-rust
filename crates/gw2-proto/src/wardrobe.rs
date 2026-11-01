//! Small build-209026 wardrobe messages 247/248; both lazily create the manager.
//! Field semantics and prerequisites: `docs/re/209026-wardrobe-startup.md`.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WardrobeOperation209026 {
    /// A Word used for content type 23 lookup; gameplay meaning unresolved.
    AssignType23 { content_id: u16 },
    /// Four separate Words stored by message 248; gameplay meanings unresolved.
    UpdateWords { values: [u16; 4] },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WardrobeFrame209026 {
    pub character_index: u32,
    pub operation: WardrobeOperation209026,
}

impl WardrobeFrame209026 {
    /// Append a frame, preserving raw field values without content validation.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        let id: u16 = match self.operation {
            WardrobeOperation209026::AssignType23 { .. } => 247,
            WardrobeOperation209026::UpdateWords { .. } => 248,
        };
        id.encode(output);
        self.character_index.encode(output);
        match self.operation {
            WardrobeOperation209026::AssignType23 { content_id } => content_id.encode(output),
            WardrobeOperation209026::UpdateWords { values } => values.encode(output),
        }
    }

    /// Decode exactly one frame; leave input unchanged on any failure.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !matches!(id, 247 | 248) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected wardrobe message 247/248",
            ));
        }
        let character_index = u32::decode(&mut cursor)?;
        let operation = match id {
            247 => WardrobeOperation209026::AssignType23 {
                content_id: u16::decode(&mut cursor)?,
            },
            248 => WardrobeOperation209026::UpdateWords {
                values: <[u16; 4]>::decode(&mut cursor)?,
            },
            _ => unreachable!(),
        };
        *input = cursor;
        Ok(Self {
            character_index,
            operation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> [(Vec<u8>, WardrobeFrame209026); 3] {
        use WardrobeOperation209026::*;
        [
            ("f7 00 ac 02 80 01", 300, AssignType23 { content_id: 384 }),
            (
                "f8 00 ff ff ff ff 0f 00 00 80 00 34 12 ff ff",
                u32::MAX,
                UpdateWords {
                    values: [0, 128, 0x1234, u16::MAX],
                },
            ),
            (
                "f8 00 00 00 00 00 00 00 00 00 00",
                0,
                UpdateWords { values: [0; 4] },
            ),
        ]
        .map(|(hex, character_index, operation)| {
            (
                hex.split_whitespace()
                    .map(|b| u8::from_str_radix(b, 16).unwrap())
                    .collect(),
                WardrobeFrame209026 {
                    character_index,
                    operation,
                },
            )
        })
    }

    #[test]
    fn wardrobe_golden_layout_keeps_words_fixed_width() {
        for (golden, frame) in fixtures() {
            let mut input = golden.as_slice();
            assert_eq!(
                WardrobeFrame209026::decode_frame(&mut input).unwrap(),
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
    fn wardrobe_consecutive_frames_preserve_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(b, _)| b.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, frame) in fixtures {
            assert_eq!(
                WardrobeFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn wardrobe_invalid_frames_preserve_input() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert_eq!(
                    WardrobeFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::UnexpectedEof
                );
                assert_eq!(input, &golden[..cut]);
            }
        }
        for id in [246u16, 249] {
            let mut bytes = id.to_le_bytes().to_vec();
            bytes.extend_from_slice(&[0; 9]);
            let mut input = bytes.as_slice();
            assert_eq!(
                WardrobeFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
        for id in [247u16, 248] {
            for invalid in [
                &b"\xff\xff\xff\xff\x10"[..],
                &b"\x80\x80\x80\x80\x80\x00"[..],
            ] {
                let bytes = [&id.to_le_bytes()[..], invalid, &[0; 8]].concat();
                let mut input = bytes.as_slice();
                assert_eq!(
                    WardrobeFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::InvalidData
                );
                assert_eq!(input, bytes);
            }
        }
    }
}
