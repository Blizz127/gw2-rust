//! Build-209026 server 538: report an existing player's quest manager.
//! See `docs/re/209026-content-direction-startup.md` for startup ordering.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

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
