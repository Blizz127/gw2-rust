//! Build-209026 renown heart (task region) research codec: the ChCliTask
//! messages 651/657/655/656/654 (char table `0x14216e940`), the heart NPC link
//! 384 and the counter objective 1090/1089 (objective table `0x1422e03b0`).
//!
//! Layouts and asserts follow the static note
//! `~/re/gw2/notes/swarm-20261105/renown-heart.md` (summarised in
//! `docs/re/209026-npc-talk-loop.md`). Dwords are varints; the heart key is a
//! fixed Word in 654-657 and a Dword in 384. Region and objective ids are
//! server-chosen; the heart key is a type-72 content key.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Encode, Guid};

/// One renown heart message. `player` is the player index (as in the quest
/// messages), `character` the NPC's character index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenownFrame209026 {
    /// 651: add the active task region; must precede 657/654 (the client
    /// dereferences a missing region) and must not repeat (asserts).
    AddRegion { player: u32, region: u32 },
    /// 657: add the heart's sub-region with its counter objective `objective`.
    AddSubRegion { player: u32, region: u32, objective: u32, heart: u16 },
    /// 655: the player is inside the heart; the tracker shows it.
    EnterSubRegion { player: u32, heart: u16 },
    /// 656: the player left the heart; the tracker hides it.
    LeaveSubRegion { player: u32, heart: u16 },
    /// 654: complete: removes the sub-region, sets the completed bit and hides
    /// the tracker (first Dword unnamed, 0).
    CompleteSubRegion { player: u32, region: u32, heart: u16 },
    /// 384: tie an NPC character to the heart (heart marker on the NPC). Send
    /// at most once per NPC while the heart is active (asserts otherwise).
    HeartNpc { character: u32, heart: u32 },
    /// 1090: create a plain counter objective (flags 0). `target` must be
    /// nonzero; the progress percent divides by it.
    CreateCounter { objective: u32, target: u32 },
    /// 1089: set the counter; a plain counter asserts `0 <= value <= target`.
    SetCounter { objective: u32, value: u32 },
    /// 556 type 14 (CHAR_REWARD_TASK_SUB_REGION): the heart completion reward
    /// panel. `reward` must be unique per session (asserting lookup).
    /// Currencies are `(content type 14 key, amount)`; key 1 coin and 2 karma
    /// are inferred (low confidence). Optional #21 carries the heart key; all
    /// other optionals are absent. Display only: it does not credit a wallet.
    HeartReward { reward: u32, heart: u32, currencies: Vec<(u32, u32)> },
}

impl RenownFrame209026 {
    pub fn message_id(&self) -> u16 {
        match self {
            Self::AddRegion { .. } => 651,
            Self::AddSubRegion { .. } => 657,
            Self::EnterSubRegion { .. } => 655,
            Self::LeaveSubRegion { .. } => 656,
            Self::CompleteSubRegion { .. } => 654,
            Self::HeartNpc { .. } => 384,
            Self::CreateCounter { .. } => 1090,
            Self::SetCounter { .. } => 1089,
            Self::HeartReward { .. } => 556,
        }
    }

    /// Append one frame; output is unchanged on an invalid value.
    pub fn encode_frame(&self, out: &mut BytesMut) -> Result<()> {
        if let Self::CreateCounter { objective, target } = self
            && (*objective == 0 || *target == 0)
        {
            return Err(Error::new(ErrorKind::InvalidInput, "counter needs a nonzero uid and target"));
        }
        if let Self::HeartReward { reward, currencies, .. } = self
            && (*reward == 0 || currencies.len() > 255)
        {
            return Err(Error::new(ErrorKind::InvalidInput, "reward needs a nonzero id and at most 255 currencies"));
        }
        self.message_id().encode(out);
        let empty_string = |out: &mut BytesMut| 0u16.encode(out);
        if let Self::HeartReward { reward, heart, currencies } = self {
            reward.encode(out);
            14u8.encode(out); // reward type: task sub-region
            0u32.encode(out);
            (currencies.len() as u8).encode(out);
            for (key, amount) in currencies {
                key.encode(out);
                amount.encode(out);
            }
            0u8.encode(out); // no items
            0u8.encode(out); // no items
            for optional in 6..=25 {
                if optional == 21 {
                    1u8.encode(out);
                    heart.encode(out);
                } else {
                    0u8.encode(out);
                }
            }
            return Ok(());
        }
        match *self {
            Self::AddRegion { player, region } => {
                player.encode(out);
                region.encode(out);
                0u8.encode(out);
            }
            Self::AddSubRegion { player, region, objective, heart } => {
                player.encode(out);
                region.encode(out);
                objective.encode(out);
                0u8.encode(out); // empty Dword array
                empty_string(out);
                empty_string(out);
                heart.encode(out);
            }
            Self::EnterSubRegion { player, heart } | Self::LeaveSubRegion { player, heart } => {
                player.encode(out);
                heart.encode(out);
            }
            Self::CompleteSubRegion { player, region, heart } => {
                player.encode(out);
                0u32.encode(out);
                region.encode(out);
                heart.encode(out);
            }
            Self::HeartNpc { character, heart } => {
                character.encode(out);
                heart.encode(out);
            }
            Self::CreateCounter { objective, target } => {
                objective.encode(out);
                empty_string(out);
                0u32.encode(out); // flags: plain counter
                target.encode(out);
                0u32.encode(out); // initial
                0u32.encode(out);
                0u32.encode(out); // soft type-0xf4 def key: none
                Guid([0; 16]).encode(out);
                0u32.encode(out); // soft type-0x13c def key: none
            }
            Self::SetCounter { objective, value } => {
                objective.encode(out);
                value.encode(out);
            }
            Self::HeartReward { .. } => unreachable!("encoded above"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, CountPrefix, Rules, StringEncoding};
    use crate::schema;

    #[test]
    fn every_frame_matches_the_client_schema() {
        let server = schema::parse(include_str!("../../../data/protocol/209026/server-tables.xml")).unwrap();
        let messages: Vec<_> = server
            .protocols
            .into_iter()
            .filter(|p| p.name == "0")
            .flat_map(|p| p.groups)
            .filter(|g| g.name == "table-0x14216e940" || g.name == "table-0x1422e03b0")
            .flat_map(|g| g.server)
            .collect();
        let rules = Rules {
            string: Some(StringEncoding::Utf16Terminated),
            array_var_small: Some(CountPrefix::U8),
            ..Rules::default()
        };
        let frames = [
            RenownFrame209026::AddRegion { player: 1, region: 1 },
            RenownFrame209026::CreateCounter { objective: 0x38, target: 5 },
            RenownFrame209026::AddSubRegion { player: 1, region: 1, objective: 0x38, heart: 56 },
            RenownFrame209026::EnterSubRegion { player: 1, heart: 56 },
            RenownFrame209026::SetCounter { objective: 0x38, value: 3 },
            RenownFrame209026::LeaveSubRegion { player: 1, heart: 56 },
            RenownFrame209026::CompleteSubRegion { player: 1, region: 1, heart: 56 },
            RenownFrame209026::HeartNpc { character: 2, heart: 56 },
            RenownFrame209026::HeartReward { reward: 1, heart: 56, currencies: vec![(1, 30), (2, 50)] },
        ];
        for frame in frames {
            let mut out = BytesMut::new();
            frame.encode_frame(&mut out).unwrap();
            let mut input = &out[..];
            let (message, _) = dynamic::decode_frame(&messages, &rules, &mut input).unwrap();
            assert_eq!(message.id, frame.message_id(), "{frame:?}");
            assert!(input.is_empty(), "{frame:?} left {} bytes", input.len());
        }
        let mut keep = BytesMut::from(&b"x"[..]);
        assert!(RenownFrame209026::CreateCounter { objective: 1, target: 0 }.encode_frame(&mut keep).is_err());
        assert!(RenownFrame209026::CreateCounter { objective: 0, target: 1 }.encode_frame(&mut keep).is_err());
        assert_eq!(&keep[..], b"x");
    }
}
