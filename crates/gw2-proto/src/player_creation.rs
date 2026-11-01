//! Build-209026 server-428 framing; see `docs/re/209026-player-creation-fields.md`.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode, Guid};

/// Explicit player-creation fields, without account or bootstrap validation.
/// No Debug implementation: names and portal identities need not be logged.
#[derive(Clone, PartialEq, Eq)]
pub struct PlayerCreation209026 {
    pub player_index: u32,
    /// Raw UTF-16 units without the zero terminator; text validity is unchecked.
    pub name_units: Vec<u16>,
    pub portal_user_id: Guid,
    /// Bit 0 selects initial state 2 instead of 3 and inserts input selector 4.
    /// Preserve all bits; the constructor's other bits have no observed use.
    pub flags: u32,
}

impl PlayerCreation209026 {
    pub const MESSAGE_ID: u16 = 428;
    pub const MAX_NAME_UNITS: usize = 31;
    pub const MAX_FRAME_SIZE: usize = 92;

    /// Append a complete frame; invalid name framing leaves output unchanged.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if self.name_units.len() > Self::MAX_NAME_UNITS || self.name_units.contains(&0) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "player name must fit 31 nonzero UTF-16 units",
            ));
        }
        Self::MESSAGE_ID.encode(output);
        self.player_index.encode(output);
        for unit in &self.name_units {
            unit.encode(output);
        }
        0u16.encode(output);
        self.portal_user_id.encode(output);
        self.flags.encode(output);
        Ok(())
    }

    /// Consume one frame, leaving input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::MESSAGE_ID {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected player-creation message 428",
            ));
        }
        let player_index = u32::decode(&mut cursor)?;
        let mut name_units = Vec::with_capacity(Self::MAX_NAME_UNITS);
        loop {
            let unit = u16::decode(&mut cursor)?;
            if unit == 0 {
                break;
            }
            if name_units.len() == Self::MAX_NAME_UNITS {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "player name exceeds terminated bound",
                ));
            }
            name_units.push(unit);
        }
        let frame = Self {
            player_index,
            name_units,
            portal_user_id: Guid::decode(&mut cursor)?,
            flags: u32::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hand-authored framing only: includes a supplementary character and an
    // unpaired surrogate, neither a proposed account name nor a text-policy test.
    fn golden_frame() -> Vec<u8> {
        vec![
            0xac, 0x01, 0xac, 0x02, // ID 428, player index 300.
            0x41, 0, 0x3d, 0xd8, 0, 0xde, 0, 0xd8, 0, 0, // Name + terminator.
            0x78, 0x56, 0x34, 0x12, 0xbc, 0x9a, 0xf0, 0xde, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, // GUID wire order.
            0xff, 0xff, 0xff, 0xff, 0x0f, // Preserve all flag bits.
        ]
    }

    #[test]
    fn player_creation_golden_preserves_name_guid_and_flags() {
        let golden = golden_frame();
        let mut input = golden.as_slice();
        let frame = PlayerCreation209026::decode_frame(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(frame.player_index, 300);
        assert_eq!(frame.name_units, [0x41, 0xd83d, 0xde00, 0xd800]);
        assert_eq!(frame.portal_user_id.0, golden[14..30]);
        assert_eq!(frame.flags, u32::MAX);
        let mut encoded = BytesMut::from(&b"prefix"[..]);
        frame.encode_frame(&mut encoded).unwrap();
        assert_eq!(&encoded[6..], golden);
    }

    #[test]
    fn player_creation_rejects_bad_framing_transactionally() {
        let golden = golden_frame();
        for cut in 0..golden.len() {
            let mut input = &golden[..cut];
            assert!(PlayerCreation209026::decode_frame(&mut input).is_err());
            assert_eq!(input, &golden[..cut]);
        }
        let mut wrong_id = golden.clone();
        wrong_id[0] = 0xab;
        let mut overflowing_flags = golden.clone();
        overflowing_flags[34] = 0x10;
        let overflowing_index = vec![0xac, 1, 0xff, 0xff, 0xff, 0xff, 0x10];
        let mut unterminated = vec![0xac, 1, 1];
        for _ in 0..32 {
            unterminated.extend([0x41, 0]);
        }
        for malformed in [wrong_id, overflowing_flags, overflowing_index, unterminated] {
            let mut input = malformed.as_slice();
            let error = PlayerCreation209026::decode_frame(&mut input)
                .err()
                .unwrap();
            assert_eq!(error.kind(), ErrorKind::InvalidData);
            assert_eq!(input, malformed);
        }
        let mut input = golden.as_slice();
        let mut frame = PlayerCreation209026::decode_frame(&mut input).unwrap();
        for bad in [vec![0x41; 32], vec![0x41, 0, 0x42]] {
            frame.name_units = bad;
            let mut output = BytesMut::from(&b"prefix"[..]);
            assert_eq!(
                frame.encode_frame(&mut output).unwrap_err().kind(),
                ErrorKind::InvalidInput
            );
            assert_eq!(output.as_ref(), b"prefix");
        }
    }

    #[test]
    fn player_creation_name_bounds_and_following_frame() {
        for length in [0, 31] {
            let mut wire = vec![0xac, 1, 0xff, 0xff, 0xff, 0xff, 0x0f];
            for _ in 0..length {
                wire.extend([0x41, 0]);
            }
            wire.extend([0, 0]);
            wire.extend([0xa5; 16]);
            wire.extend([0xff, 0xff, 0xff, 0xff, 0x0f]);
            let frame_length = wire.len();
            if length == 31 {
                assert_eq!(frame_length, PlayerCreation209026::MAX_FRAME_SIZE);
            }
            let following = golden_frame();
            wire.extend_from_slice(&following);
            let mut input = wire.as_slice();
            let frame = PlayerCreation209026::decode_frame(&mut input).unwrap();
            assert_eq!(frame.name_units.len(), length);
            assert_eq!(frame.player_index, u32::MAX);
            assert_eq!(input, following);
            let mut encoded = BytesMut::new();
            frame.encode_frame(&mut encoded).unwrap();
            assert_eq!(encoded, &wire[..frame_length]);
            PlayerCreation209026::decode_frame(&mut input).unwrap();
            assert!(input.is_empty());
        }
    }
}
