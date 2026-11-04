//! Build-209026 server 83 (AgMsg.cpp, group 0): grant client control of an agent.
//!
//! Handler `0x141034f80`: **Word agent index, Dword N**. It calls `0x14102fb00`, which
//! stores world +0x1c4 = index and +0x1c8 = N, clears bit 0x40 on the previous agent's
//! +0xe8, ORs 0x40 into the new agent's +0xe8, then calls `0x141025460`. Bit 0x40 is
//! the agent's client-control flag (assertion `ValidateClientControl()`, `0x14211b360`,
//! AgChar.cpp). Without it, command submission `0x14102fbd0` destroys every locally
//! issued agent command for a character agent in client mode (`0x14102fc4d`): the
//! character plays its run animation but never moves and sends no AgMsg traffic
//! (owner device and sandbox runs 92-97, 2026-11-04). Index 0 is skipped, and the
//! slot is dereferenced unchecked, so the index must name an existing character agent.
//!
//! Ordering: once the flag is set, the client drops later server commands for that
//! agent unless their command flags carry 0x10 (`0x14102fdf3` / `0x14102fdfd`), so 83
//! must follow the scheduled world entry (46). The client is expected to answer with
//! client 5 (Word, Dword tick, Dword N). N is echoed back; its meaning is not
//! established (a grant tick, inferred).

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentControl209026 {
    pub agent_index: u16,
    pub value: u32,
}

impl AgentControl209026 {
    pub const ID: u16 = 83;

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::ID.encode(output);
        self.agent_index.encode(output);
        self.value.encode(output);
    }

    /// Consume exactly one frame; leave the input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::ID {
            return Err(Error::new(ErrorKind::InvalidData, "expected agent control message 83"));
        }
        let frame = Self { agent_index: u16::decode(&mut cursor)?, value: u32::decode(&mut cursor)? };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_rejects_truncation() {
        let frame = AgentControl209026 { agent_index: 1, value: 300 };
        let mut out = BytesMut::new();
        frame.encode_frame(&mut out);
        assert_eq!(&out[..], &[83, 0, 1, 0, 0xac, 0x02]);
        let mut input = &out[..];
        assert_eq!(AgentControl209026::decode_frame(&mut input).unwrap(), frame);
        assert!(input.is_empty());
        for cut in 0..out.len() {
            let mut short = &out[..cut];
            assert!(AgentControl209026::decode_frame(&mut short).is_err());
            assert_eq!(short.len(), cut);
        }
    }
}
