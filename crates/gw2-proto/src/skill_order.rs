//! Build-209026 skill-order research codec: the client-23 SkillBegin request
//! (packed opcode 0x23), server 34 (the agent behavior-order message whose
//! command carries flag bit 12) and the packed 0x78 active-skill order.
//!
//! Layouts follow `docs/re/209026-mesmer-skill-command-path.md` and
//! `docs/re/209026-npc-talk-loop.md` section 6. Whether server 34 + 0x78 is
//! the retail way to execute a skill is an untested hypothesis; nothing here
//! validates game state.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

/// Client-23 behavior data `[0x23][u32 target][u32 skill key][u8 slot]`
/// (producer `0x141284202`). Little-endian fixed fields, 10 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillBeginRequest209026 {
    /// Supplied target object's +0xc, or 0 for none.
    pub target: u32,
    pub skill_key: u32,
    /// Skillbar slot (not the physical key).
    pub slot: u8,
}

impl SkillBeginRequest209026 {
    pub const OPCODE: u8 = 0x23;

    pub fn from_behavior_data(data: &[u8]) -> Result<Self> {
        let [Self::OPCODE, t0, t1, t2, t3, k0, k1, k2, k3, slot] = *data else {
            return Err(invalid("not a 10-byte SkillBegin 0x23 payload"));
        };
        Ok(Self {
            target: u32::from_le_bytes([t0, t1, t2, t3]),
            skill_key: u32::from_le_bytes([k0, k1, k2, k3]),
            slot,
        })
    }
}

/// Packed 0x78 active-skill order, 13 bytes (order factory `0x14127ffb0`,
/// constructor `0x14127da80`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillOrder209026 {
    /// Stored at order +0x14; separate from the outer command flags.
    pub order_flags: u8,
    pub skill_key: u32,
    pub slot: u8,
    /// Stored at order +0xf4.
    pub raw_7: u8,
    /// Agent identifier resolved through `0x141024060`.
    pub agent: u32,
    /// Bit 0 adds order flag 0x8000; bit 1 selects the action variant.
    pub byte_12: u8,
}

impl SkillOrder209026 {
    pub const OPCODE: u8 = 0x78;

    pub fn to_behavior_data(&self) -> [u8; 13] {
        let mut out = [0u8; 13];
        out[0] = Self::OPCODE;
        out[1] = self.order_flags;
        out[2..6].copy_from_slice(&self.skill_key.to_le_bytes());
        out[6] = self.slot;
        out[7] = self.raw_7;
        out[8..12].copy_from_slice(&self.agent.to_le_bytes());
        out[12] = self.byte_12;
        out
    }
}

/// Server 34: the behavior header of 84 plus a trailing Word. Its handler
/// `0x1410333e0` builds command kind 2 (`0x141035880`) with flags
/// `| 0x801000` (bit 12 set, bit 17 clear) and stores the Word at +0x70.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBehaviorOrderFrame209026 {
    pub agent_index: u16,
    pub remote_tick: u32,
    pub command_flags: u8,
    pub behavior_value: u32,
    pub behavior_data: Vec<u8>,
    /// Stored at command +0x70 = `m_durationMs`. For a non-controlled agent
    /// it must be nonzero (AgBase.cpp:822 asserts, sandbox run 127).
    pub word: u16,
}

impl AgentBehaviorOrderFrame209026 {
    pub const MESSAGE_ID: u16 = 34;
    pub const MAX_BEHAVIOR_BYTES: usize = 250;

    /// Append one frame; output is unchanged for an oversized payload.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if self.behavior_data.len() > Self::MAX_BEHAVIOR_BYTES {
            return Err(Error::new(ErrorKind::InvalidInput, "behavior exceeds 250 bytes"));
        }
        Self::MESSAGE_ID.encode(output);
        self.agent_index.encode(output);
        self.remote_tick.encode(output);
        self.command_flags.encode(output);
        self.behavior_value.encode(output);
        (self.behavior_data.len() as u8).encode(output);
        output.extend_from_slice(&self.behavior_data);
        self.word.encode(output);
        Ok(())
    }

    /// Consume one frame; input is unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::MESSAGE_ID {
            return Err(invalid("expected agent behavior-order message 34"));
        }
        let agent_index = u16::decode(&mut cursor)?;
        let remote_tick = u32::decode(&mut cursor)?;
        let command_flags = u8::decode(&mut cursor)?;
        let behavior_value = u32::decode(&mut cursor)?;
        let length = u8::decode(&mut cursor)? as usize;
        if length > Self::MAX_BEHAVIOR_BYTES {
            return Err(invalid("behavior exceeds 250 bytes"));
        }
        if cursor.len() < length {
            return Err(ErrorKind::UnexpectedEof.into());
        }
        let behavior_data = cursor[..length].to_vec();
        cursor = &cursor[length..];
        let word = u16::decode(&mut cursor)?;
        *input = cursor;
        Ok(Self {
            agent_index,
            remote_tick,
            command_flags,
            behavior_value,
            behavior_data,
            word,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, CountPrefix, Rules, StringEncoding};
    use crate::schema;

    #[test]
    fn run125_skill_begin_bytes_decode() {
        let data = [0x23, 0, 0, 0, 0, 0xc0, 0x27, 0, 0, 0];
        assert_eq!(
            SkillBeginRequest209026::from_behavior_data(&data).unwrap(),
            SkillBeginRequest209026 { target: 0, skill_key: 10176, slot: 0 }
        );
        assert!(SkillBeginRequest209026::from_behavior_data(&data[..9]).is_err());
        assert!(SkillBeginRequest209026::from_behavior_data(&[0x14, 2, 0, 0, 0]).is_err());
    }

    #[test]
    fn order_34_carries_a_13_byte_0x78_order_and_matches_the_schema() {
        let order = SkillOrder209026 { order_flags: 0, skill_key: 10176, slot: 0, raw_7: 0, agent: 1, byte_12: 0 };
        assert_eq!(order.to_behavior_data(), [0x78, 0, 0xc0, 0x27, 0, 0, 0, 0, 1, 0, 0, 0, 0]);
        let frame = AgentBehaviorOrderFrame209026 {
            agent_index: 1,
            remote_tick: 77,
            command_flags: 3,
            behavior_value: 0x8000_0009,
            behavior_data: order.to_behavior_data().to_vec(),
            word: 0,
        };
        let mut out = BytesMut::new();
        frame.encode_frame(&mut out).unwrap();
        assert_eq!(AgentBehaviorOrderFrame209026::decode_frame(&mut &out[..]).unwrap(), frame);
        let server = schema::parse(include_str!("../../../data/protocol/209026/server-tables.xml")).unwrap();
        let messages: Vec<_> = server
            .protocols
            .into_iter()
            .filter(|p| p.name == "0")
            .flat_map(|p| p.groups)
            .filter(|g| g.name == "table-0x14211be70")
            .flat_map(|g| g.server)
            .collect();
        let rules = Rules {
            string: Some(StringEncoding::Utf16Terminated),
            buffer_var_small: Some(CountPrefix::U8),
            ..Rules::default()
        };
        let mut input = &out[..];
        let (message, _) = dynamic::decode_frame(&messages, &rules, &mut input).unwrap();
        assert_eq!(message.id, 34);
        assert!(input.is_empty());
        for end in 0..out.len() {
            let mut cut = &out[..end];
            assert!(AgentBehaviorOrderFrame209026::decode_frame(&mut cut).is_err());
            assert_eq!(cut.len(), end);
        }
    }
}
