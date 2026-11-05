//! Explicit initial skill availability and standard-palette selection.
//! Content membership and retail selection remain the caller's responsibility.

use bytes::Bytes;
use gw2_proto::{
    skill_availability::{SkillAvailabilityFrame209026, SkillAvailabilityOperation209026},
    skill_palette::SkillPaletteFrame209026,
};
use std::{collections::BTreeSet, io};

pub const SKILL_BYTES_LIMIT: usize = 4096;
// Observer resource policy: 605 grows a bitset indexed by the supplied key.
// This is not a client content maximum or a gameplay eligibility rule.
const MAX_FIXTURE_SKILL_KEY: u32 = 1_000_000;

#[derive(Clone)]
pub struct SkillFixture(Bytes);

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

impl SkillFixture {
    /// Zero or more unique 605 availability grants, then at least one 612
    /// selection. Restrict selection to standard slots 0..4 and locales 0..3.
    /// The two arrays alias locales 0/2 and 1/3; each destination is set once.
    pub fn parse(bytes: &[u8], player_index: u32) -> io::Result<Self> {
        if bytes.is_empty() || bytes.len() > SKILL_BYTES_LIMIT {
            return Err(invalid("skill fixture must contain 1..4096 bytes"));
        }
        let mut cursor = bytes;
        let mut skills = BTreeSet::new();
        let mut destinations = BTreeSet::new();
        while !cursor.is_empty() {
            let id = cursor
                .get(..2)
                .ok_or_else(|| invalid("truncated skill frame ID"))?;
            match u16::from_le_bytes([id[0], id[1]]) {
                605 => {
                    if !destinations.is_empty() || skills.len() >= 64 {
                        return Err(invalid(
                            "at most 64 availability grants must precede all palette selections",
                        ));
                    }
                    let frame = SkillAvailabilityFrame209026::decode_frame(&mut cursor)?;
                    let SkillAvailabilityOperation209026::InitialSet(key) = frame.operation else {
                        unreachable!("605 decoder returns InitialSet");
                    };
                    if frame.player_index != player_index {
                        return Err(invalid(
                            "skill player index differs from population fixture",
                        ));
                    }
                    if key == 0 || key > MAX_FIXTURE_SKILL_KEY || !skills.insert(key) {
                        return Err(invalid(
                            "skill availability keys must be unique and within 1..1000000",
                        ));
                    }
                }
                612 => {
                    let frame = SkillPaletteFrame209026::decode_frame(&mut cursor)?;
                    if frame.player_index != player_index {
                        return Err(invalid(
                            "palette player index differs from population fixture",
                        ));
                    }
                    if frame.palette_key == 0 || frame.slot > 4 || frame.context > 3 {
                        return Err(invalid(
                            "initial palette requires a nonzero key, standard slot 0..4 and locale 0..3",
                        ));
                    }
                    if !destinations.insert((frame.slot, frame.context & 1)) {
                        return Err(invalid(
                            "duplicate palette destination, including aliased locales",
                        ));
                    }
                }
                _ => {
                    return Err(invalid(
                        "skill fixture allows only initial availability 605 and palette selection 612",
                    ));
                }
            }
        }
        if destinations.is_empty() {
            return Err(invalid(
                "skill fixture needs at least one palette selection to rebuild the skill bar",
            ));
        }
        Ok(Self(Bytes::copy_from_slice(bytes)))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
pub(super) fn fixture_for_test() -> Vec<u8> {
    // Independent wire fixture: 605(10176,1), 612(366,slot0,locale0,player1).
    vec![0x5d, 2, 0xc0, 0x4f, 1, 0x64, 2, 0xee, 2, 0, 0, 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    fn grant(key: u32, player_index: u32) -> Vec<u8> {
        let mut out = BytesMut::new();
        SkillAvailabilityFrame209026 {
            player_index,
            operation: SkillAvailabilityOperation209026::InitialSet(key),
        }
        .encode_frame(&mut out);
        out.to_vec()
    }

    fn select(key: u32, slot: u8, context: u8, player_index: u32) -> Vec<u8> {
        let mut out = BytesMut::new();
        SkillPaletteFrame209026 {
            palette_key: key,
            slot,
            context,
            player_index,
        }
        .encode_frame(&mut out);
        out.to_vec()
    }

    #[test]
    fn initial_skills_preserve_frames_and_reject_invalid_state_or_order() {
        let good = fixture_for_test();
        assert_eq!(SkillFixture::parse(&good, 1).unwrap().bytes(), good);
        let palette = select(366, 0, 0, 1);
        assert!(
            SkillFixture::parse(&palette, 1).is_ok(),
            "selection may use already available content"
        );
        for bytes in [
            grant(10176, 1),
            [grant(0, 1), palette.clone()].concat(),
            [grant(MAX_FIXTURE_SKILL_KEY + 1, 1), palette.clone()].concat(),
            [grant(10176, 2), palette.clone()].concat(),
            [grant(10176, 1), grant(10176, 1), palette.clone()].concat(),
            [palette.clone(), grant(10176, 1)].concat(),
            select(0, 0, 0, 1),
            select(366, 5, 0, 1),
            select(366, 0, 4, 1),
            select(366, 0, 0, 2),
            [palette.clone(), palette.clone()].concat(),
            [palette.clone(), select(366, 0, 2, 1)].concat(),
            [select(366, 0, 1, 1), select(366, 0, 3, 1)].concat(),
            [good.clone(), vec![0x5e, 2, 1]].concat(), // 606 reset is not initialization.
        ] {
            assert!(
                SkillFixture::parse(&bytes, 1).is_err(),
                "accepted {bytes:?}"
            );
        }
        for cut in 0..good.len() {
            assert!(SkillFixture::parse(&good[..cut], 1).is_err());
        }
        assert!(SkillFixture::parse(&vec![0; SKILL_BYTES_LIMIT + 1], 1).is_err());
    }

    #[test]
    fn distinct_palette_arrays_and_availability_bound_are_preserved() {
        let selections = [select(366, 0, 0, 1), select(366, 0, 1, 1)].concat();
        assert_eq!(
            SkillFixture::parse(&selections, 1).unwrap().bytes(),
            selections
        );
        let grants: Vec<u8> = (1..=64).flat_map(|key| grant(key, 1)).collect();
        assert!(SkillFixture::parse(&[grants.clone(), selections.clone()].concat(), 1).is_ok());
        assert!(SkillFixture::parse(&[grants, grant(65, 1), selections].concat(), 1).is_err());
    }
}
