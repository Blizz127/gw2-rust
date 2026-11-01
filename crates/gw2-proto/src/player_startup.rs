//! Build-209026 server 431: payload-free player startup event.
//! Component prerequisites are documented in `docs/re/209026-player-startup-frame.md`.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// Transport only; constructing this value does not establish client readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStartupFrame209026;

impl PlayerStartupFrame209026 {
    pub const MESSAGE_ID: u16 = 431;

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
    }

    /// Consume exactly one frame; leave input unchanged on any decoding error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::MESSAGE_ID {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected player startup message 431",
            ));
        }
        *input = cursor;
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quest::QuestManagerReportFrame209026;

    #[test]
    fn player_startup_literal_and_mixed_stream_boundaries() {
        let mut output = BytesMut::from(&b"prefix"[..]);
        PlayerStartupFrame209026.encode_frame(&mut output);
        assert_eq!(&output[..], b"prefix\xaf\x01");

        // Report player 300, startup, then report player 1. Parsing startup
        // must neither consume the following frame nor require a payload.
        let wire = [0x1a, 2, 0xac, 2, 0xaf, 1, 0x1a, 2, 1];
        let mut input = wire.as_slice();
        assert_eq!(
            QuestManagerReportFrame209026::decode_frame(&mut input)
                .unwrap()
                .player_index,
            300
        );
        assert_eq!(
            PlayerStartupFrame209026::decode_frame(&mut input).unwrap(),
            PlayerStartupFrame209026
        );
        assert_eq!(input, &wire[6..]);
        assert_eq!(
            QuestManagerReportFrame209026::decode_frame(&mut input)
                .unwrap()
                .player_index,
            1
        );
        assert!(input.is_empty());
    }

    #[test]
    fn player_startup_rejects_truncation_and_other_ids_transactionally() {
        for original in [&[][..], &[0xaf][..]] {
            let mut input = original;
            assert_eq!(
                PlayerStartupFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::UnexpectedEof
            );
            assert_eq!(input, original);
        }
        for id in [0u16, 430, 432, 538, u16::MAX] {
            let [lo, hi] = id.to_le_bytes();
            let bytes = [lo, hi, 0xaf, 1];
            let mut input = bytes.as_slice();
            assert_eq!(
                PlayerStartupFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
    }
}
