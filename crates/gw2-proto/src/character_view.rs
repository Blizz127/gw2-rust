//! Build-209026 server-585 transport. See
//! `docs/re/209026-character-view-suppression.md` for observed lifecycle effects.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// Character view suppression input, with the original byte preserved.
/// This frame does not establish finalization or other client prerequisites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterViewSuppression209026 {
    pub character_index: u32,
    /// The client treats every nonzero value as true; preserve raw transport.
    pub value: u8,
}

impl CharacterViewSuppression209026 {
    pub const MESSAGE_ID: u16 = 585;

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.character_index.encode(output);
        self.value.encode(output);
    }

    /// Consume one frame; leave input unchanged on every decoding error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::MESSAGE_ID {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected character-view-suppression message 585",
            ));
        }
        let frame = Self {
            character_index: u32::decode(&mut cursor)?,
            value: u8::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_suppression_frame_literals_and_stream_boundaries() {
        let cases: &[(&[u8], u32, u8)] = &[
            (&[0x49, 2, 0, 0], 0, 0),
            (&[0x49, 2, 0xac, 2, 1], 300, 1),
            (
                &[0x49, 2, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff],
                u32::MAX,
                255,
            ),
        ];
        let mut output = BytesMut::from(&b"prefix"[..]);
        for &(wire, character_index, value) in cases {
            let frame = CharacterViewSuppression209026 {
                character_index,
                value,
            };
            let start = output.len();
            frame.encode_frame(&mut output);
            assert_eq!(&output[start..], wire);
        }
        assert_eq!(&output[..6], b"prefix");
        let mut input = &output[6..];
        for &(_, character_index, value) in cases {
            assert_eq!(
                CharacterViewSuppression209026::decode_frame(&mut input).unwrap(),
                CharacterViewSuppression209026 {
                    character_index,
                    value
                }
            );
        }
        assert!(input.is_empty());
    }

    #[test]
    fn view_suppression_frame_errors_are_transactional() {
        let wire = [0x49, 2, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff];
        for end in 0..wire.len() {
            let original = &wire[..end];
            let mut input = original;
            assert!(CharacterViewSuppression209026::decode_frame(&mut input).is_err());
            assert_eq!(input, original);
        }
        for original in [
            &[0x48, 2, 0, 0][..],                               // Wrong message ID.
            &[0x49, 2, 0xff, 0xff, 0xff, 0xff, 0x10, 0][..],    // Dword overflow.
            &[0x49, 2, 0x80, 0x80, 0x80, 0x80, 0x80, 0, 0][..], // Too long.
        ] {
            let mut input = original;
            assert!(CharacterViewSuppression209026::decode_frame(&mut input).is_err());
            assert_eq!(input, original);
        }
    }
}
