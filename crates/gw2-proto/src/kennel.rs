//! Build-209026 server 332: kennel component creation.
//!
//! Handler `0x141251e70` (protocol 0, group 0): **Dword character index, Byte, Byte,
//! ArrayVarSmall (U8 count, at most 32) of Dwords**. It asserts the character exists,
//! copies the array (`count * 4` bytes) into a small vector and calls `0x1411d6540`
//! with both Bytes and that vector. The helper asserts `!m_kennel` (character
//! +0x3f8), allocates 0x1b8 bytes and constructs the kennel through `0x141277790`,
//! which has no assertion of its own.
//!
//! Every character the client controls needs one: without it the owner's client
//! asserted `m_kennel` (ChCliCharacter.h:579) once the map reached READY_WAIT after
//! 1036/431 (sandbox run 84, 2026-11-04). The Bytes' and entries' meanings (pet
//! slots, presumably) are not established; an empty kennel is the minimal value.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// Largest entry count the descriptor admits.
pub const KENNEL_ENTRIES_MAX: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KennelCreation209026 {
    pub character_index: u32,
    /// Field 1, passed to the constructor as its third argument. Meaning open.
    pub first: u8,
    /// Field 2, passed to the constructor as its fourth argument. Meaning open.
    pub second: u8,
    /// Field 3, up to [`KENNEL_ENTRIES_MAX`] Dwords. Meaning open.
    pub entries: Vec<u32>,
}

impl KennelCreation209026 {
    pub const ID: u16 = 332;

    /// Append one complete frame. Fails if there are more than 32 entries.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if self.entries.len() > KENNEL_ENTRIES_MAX {
            return Err(Error::new(ErrorKind::InvalidInput, "kennel admits at most 32 entries"));
        }
        Self::ID.encode(output);
        self.character_index.encode(output);
        self.first.encode(output);
        self.second.encode(output);
        (self.entries.len() as u8).encode(output);
        for entry in &self.entries {
            entry.encode(output);
        }
        Ok(())
    }

    /// Consume exactly one frame; leave the input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::ID {
            return Err(Error::new(ErrorKind::InvalidData, "expected kennel message 332"));
        }
        let character_index = u32::decode(&mut cursor)?;
        let first = u8::decode(&mut cursor)?;
        let second = u8::decode(&mut cursor)?;
        let count = usize::from(u8::decode(&mut cursor)?);
        if count > KENNEL_ENTRIES_MAX {
            return Err(Error::new(ErrorKind::InvalidData, "kennel entry count exceeds 32"));
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(u32::decode(&mut cursor)?);
        }
        *input = cursor;
        Ok(Self { character_index, first, second, entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_rejects_bad_frames() {
        for frame in [
            KennelCreation209026 { character_index: 1, first: 0, second: 0, entries: vec![] },
            KennelCreation209026 { character_index: 300, first: 2, second: 7, entries: vec![1, 70_000, u32::MAX] },
        ] {
            let mut out = BytesMut::new();
            frame.encode_frame(&mut out).unwrap();
            let mut input = &out[..];
            assert_eq!(KennelCreation209026::decode_frame(&mut input).unwrap(), frame);
            assert!(input.is_empty());
            for cut in 0..out.len() {
                let mut short = &out[..cut];
                assert!(KennelCreation209026::decode_frame(&mut short).is_err());
                assert_eq!(short.len(), cut, "input unchanged on error");
            }
        }
        let too_many = KennelCreation209026 { character_index: 1, first: 0, second: 0, entries: vec![0; 33] };
        assert!(too_many.encode_frame(&mut BytesMut::new()).is_err());
        let mut wrong = &[0x4d, 0x01, 0x01, 0, 0, 0][..];
        assert!(KennelCreation209026::decode_frame(&mut wrong).is_err());
    }
}
