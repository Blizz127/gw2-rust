//! Build-209026 server 516: player PvP manager creation.
//!
//! Handler `0x141257c60` (protocol 0, group 0): **Dword player index, Dword, Dword,
//! Byte, Dword, Dword, Dword, ArrayFixed of 4 Dwords, Dword**. It looks the player up
//! through context virtual +0x118 and asserts it exists, then calls `0x1411baa20`,
//! which asserts `!m_pvpMgr` (player +0x97b8), allocates 0xd8 bytes and constructs the
//! manager (`0x1411fbcb0`). It then applies the fields:
//! - `0x1411fc340(field 2, field 1, field 3)`;
//! - field 6 resolves as a type-0x36 content key, fields 5 and 4 and the four array
//!   entries as type-0x23 keys, each passed to a setter. Key 0 resolves to null, and
//!   no setter asserts on null. The array setter `0x1411fc4a0` asserts only its slot
//!   index (`upgrade < arrsize(m_weaponUpgrades)`, 0..=3, supplied by the handler).
//!
//! Without it the client asserted `m_pvpMgr` (ChCliPlayer.h:308) at the end of
//! READY_WAIT (sandbox run 85, 2026-11-04). Field meanings beyond those lookups
//! are not established; all-zero values are the minimal manager.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PvpManagerCreation209026 {
    pub player_index: u32,
    pub values: [u32; 2],
    pub flag: u8,
    /// Fields 4, 5 (type-0x23 keys) and 6 (type-0x36 key).
    pub keys: [u32; 3],
    /// Field 7: four type-0x23 keys (weapon upgrades, by the setter's assertion).
    pub upgrades: [u32; 4],
    pub last: u32,
}

impl PvpManagerCreation209026 {
    pub const ID: u16 = 516;

    /// The minimal manager for `player_index`: every other field zero.
    pub fn empty(player_index: u32) -> Self {
        Self { player_index, values: [0; 2], flag: 0, keys: [0; 3], upgrades: [0; 4], last: 0 }
    }

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::ID.encode(output);
        self.player_index.encode(output);
        self.values.encode(output);
        self.flag.encode(output);
        self.keys.encode(output);
        self.upgrades.encode(output);
        self.last.encode(output);
    }

    /// Consume exactly one frame; leave the input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::ID {
            return Err(Error::new(ErrorKind::InvalidData, "expected PvP manager message 516"));
        }
        let frame = Self {
            player_index: u32::decode(&mut cursor)?,
            values: <[u32; 2]>::decode(&mut cursor)?,
            flag: u8::decode(&mut cursor)?,
            keys: <[u32; 3]>::decode(&mut cursor)?,
            upgrades: <[u32; 4]>::decode(&mut cursor)?,
            last: u32::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_rejects_truncation() {
        for frame in [
            PvpManagerCreation209026::empty(1),
            PvpManagerCreation209026 { player_index: 9, values: [1, 300], flag: 2, keys: [4, 5, 6], upgrades: [7, 8, 9, u32::MAX], last: 70_000 },
        ] {
            let mut out = BytesMut::new();
            frame.encode_frame(&mut out);
            let mut input = &out[..];
            assert_eq!(PvpManagerCreation209026::decode_frame(&mut input).unwrap(), frame);
            assert!(input.is_empty());
            for cut in 0..out.len() {
                let mut short = &out[..cut];
                assert!(PvpManagerCreation209026::decode_frame(&mut short).is_err());
                assert_eq!(short.len(), cut);
            }
        }
    }
}
