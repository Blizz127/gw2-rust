//! Build-209026 server 565: create and initialize a reward-track manager.
//! See `docs/re/209026-reward-track-messages.md` for state prerequisites.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewardTrackCreationFrame209026 {
    pub player_index: u32,
    /// Numeric IDs passed to content-service virtual +0x70 for type 353.
    pub track_ids: [u32; 4],
    /// Raw pin value per slot (`m_pinnedTracks`); accepted values unresolved.
    pub slot_bytes: [u8; 4],
}

impl RewardTrackCreationFrame209026 {
    /// Append a frame without interpreting content IDs or slot bytes.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        565u16.encode(output);
        self.player_index.encode(output);
        self.track_ids.encode(output);
        self.slot_bytes.encode(output);
    }

    /// Consume one complete frame transactionally, without state validation.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != 565 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected reward-track creation message 565",
            ));
        }
        let frame = Self {
            player_index: u32::decode(&mut cursor)?,
            track_ids: <[u32; 4]>::decode(&mut cursor)?,
            slot_bytes: <[u8; 4]>::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> [(Vec<u8>, RewardTrackCreationFrame209026); 2] {
        [
            (
                "35 02 ac 02 00 80 01 80 80 01 ff ff ff ff 0f 00 01 80 ff",
                RewardTrackCreationFrame209026 {
                    player_index: 300,
                    track_ids: [0, 128, 16_384, u32::MAX],
                    slot_bytes: [0, 1, 128, 255],
                },
            ),
            (
                "35 02 ff ff ff ff 0f 7f 81 01 ff 7f 81 80 01 ff fe 02 00",
                RewardTrackCreationFrame209026 {
                    player_index: u32::MAX,
                    track_ids: [127, 129, 16_383, 16_385],
                    slot_bytes: [255, 254, 2, 0],
                },
            ),
        ]
        .map(|(hex, frame)| {
            (
                hex.split_whitespace()
                    .map(|b| u8::from_str_radix(b, 16).unwrap())
                    .collect(),
                frame,
            )
        })
    }

    #[test]
    fn reward_track_creation_golden_layout_preserves_arrays() {
        for (golden, frame) in fixtures() {
            let mut input = golden.as_slice();
            assert_eq!(
                RewardTrackCreationFrame209026::decode_frame(&mut input).unwrap(),
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
    fn reward_track_creation_consecutive_frames_preserve_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(b, _)| b.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, frame) in fixtures {
            assert_eq!(
                RewardTrackCreationFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn reward_track_creation_invalid_frames_preserve_input() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert_eq!(
                    RewardTrackCreationFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::UnexpectedEof
                );
                assert_eq!(input, &golden[..cut]);
            }
        }
        for id in [564u16, 566] {
            let mut bytes = id.to_le_bytes().to_vec();
            bytes.extend_from_slice(&[0; 9]);
            let mut input = bytes.as_slice();
            assert_eq!(
                RewardTrackCreationFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
        // A zero-valued frame uses one byte for each of its five Dwords.
        // Overflow/overlength must fail at every array element as well as player.
        let base = [0x35, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        for position in 2..7 {
            for invalid in [
                &b"\xff\xff\xff\xff\x10"[..],
                &b"\x80\x80\x80\x80\x80\x00"[..],
            ] {
                let bytes = [&base[..position], invalid, &base[position + 1..]].concat();
                let mut input = bytes.as_slice();
                assert_eq!(
                    RewardTrackCreationFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::InvalidData
                );
                assert_eq!(input, bytes);
            }
        }
    }
}
