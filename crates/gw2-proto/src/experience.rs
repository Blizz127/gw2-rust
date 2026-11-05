//! Build-209026 character experience and earned-level frames (server 191 and
//! 188) and the client's experience-for-level table. See
//! `docs/re/209026-npc-talk-loop.md` ("XP").
//!
//! Server 191's handler `0x14124e2e0` calls coreStats setter `0x1412091e0`,
//! which stores the experience Dword at coreStats +0xe4 capped at
//! `EXPERIENCE_FOR_LEVEL[80]` (15 in some modes) and notifies listeners with
//! the two trailing fields. Server 188 sets the earned level (`0x141208f50`).

use bytes::BytesMut;

use crate::Encode;

/// Total experience required for each level 0..=80: the frozen client's
/// `s_experienceForLevelLut` (`0x1421d59a0`, read by `0x1412df420`).
pub const EXPERIENCE_FOR_LEVEL: [u32; 81] = [
    0, 0, 2000, 4110, 6489, 9163, 12162, 15505,
    19224, 23353, 27920, 32964, 38517, 44628, 51333, 58687,
    66729, 75519, 84789, 94559, 104859, 115759, 127159, 139259,
    151959, 165359, 179459, 194359, 210059, 226559, 243959, 262259,
    281559, 301959, 323459, 346059, 369959, 395059, 421559, 449459,
    478859, 509859, 542559, 577059, 613359, 651659, 691959, 734459,
    779259, 826459, 876259, 928759, 984059, 1042359, 1103759, 1168459,
    1236659, 1308559, 1384359, 1464159, 1548259, 1636959, 1730459, 1828959,
    1932959, 2041959, 2156959, 2278959, 2406959, 2541959, 2683959, 2833959,
    2991959, 3158959, 3334959, 3519959, 3714959, 3919959, 4136959, 4364959,
    4605959,
];

/// The level reached with `experience` total, clamped to 1..=80.
pub fn level_for_experience(experience: u32) -> u8 {
    let reached = EXPERIENCE_FOR_LEVEL.iter().rposition(|&need| experience >= need).unwrap_or(1);
    reached.clamp(1, 80) as u8
}

/// Server 191: total experience. `dword_2` and `byte_3` reach the experience
/// listeners unnamed (inferred: change source / reason); 0 is used here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExperienceFrame209026 {
    pub character_index: u32,
    pub experience: u32,
    pub dword_2: u32,
    pub byte_3: u8,
}

impl ExperienceFrame209026 {
    pub const MESSAGE_ID: u16 = 191;

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.character_index.encode(output);
        self.experience.encode(output);
        self.dword_2.encode(output);
        self.byte_3.encode(output);
    }
}

/// Server 188: earned level (players 1..=80, asserted by the setter).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EarnedLevelFrame209026 {
    pub character_index: u32,
    pub level: u8,
}

impl EarnedLevelFrame209026 {
    pub const MESSAGE_ID: u16 = 188;

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.character_index.encode(output);
        self.level.encode(output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, Rules};
    use crate::schema;

    #[test]
    fn table_matches_known_thresholds_and_levels() {
        assert_eq!(&EXPERIENCE_FOR_LEVEL[..4], &[0, 0, 2000, 4110]);
        assert_eq!(EXPERIENCE_FOR_LEVEL[80], 4_605_959);
        assert!(EXPERIENCE_FOR_LEVEL.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(level_for_experience(0), 1);
        assert_eq!(level_for_experience(1999), 1);
        assert_eq!(level_for_experience(2000), 2);
        assert_eq!(level_for_experience(u32::MAX), 80);
    }

    #[test]
    fn frames_match_the_client_schema() {
        let server = schema::parse(include_str!("../../../data/protocol/209026/server-tables.xml")).unwrap();
        let messages: Vec<_> = server
            .protocols
            .into_iter()
            .filter(|p| p.name == "0")
            .flat_map(|p| p.groups)
            .filter(|g| g.name == "table-0x14216e940")
            .flat_map(|g| g.server)
            .collect();
        let rules = Rules::default();
        let mut out = BytesMut::new();
        ExperienceFrame209026 { character_index: 1, experience: 2500, dword_2: 0, byte_3: 0 }.encode_frame(&mut out);
        EarnedLevelFrame209026 { character_index: 1, level: 2 }.encode_frame(&mut out);
        let mut input = &out[..];
        assert_eq!(dynamic::decode_frame(&messages, &rules, &mut input).unwrap().0.id, 191);
        assert_eq!(dynamic::decode_frame(&messages, &rules, &mut input).unwrap().0.id, 188);
        assert!(input.is_empty());
    }
}
