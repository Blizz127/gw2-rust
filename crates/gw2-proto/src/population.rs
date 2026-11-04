//! Candidate build-209026 character population sequence for one fresh owned
//! player character, as concatenated protocol-0 server frames.
//!
//! **This is a candidate fixture sequence, not a validated bootstrap.** It
//! follows the "Phase 4" table and "Candidate order" item 3 of
//! `docs/re/209026-bootstrap-sequence.md` (plus Phase 5's server 46), using
//! only the existing encoders in this crate. No wire layout is defined here.
//! The builder is pure, offline and deterministic: nothing has been exchanged
//! with the client, and nothing here sends a frame.
//!
//! Every value is either required by a constraint in a `docs/re/209026-*.md`
//! note (cited in a comment as `docs/re/<file>.md: <reason>`) or marked
//! `placeholder:` in a comment, meaning a valid value is not established. A
//! placeholder is a framing-valid choice that satisfies every documented
//! constraint, not evidence of a useful gameplay value.
//!
//! The caller owns the values with no documented default: the identities, the
//! spawn position/facing and the 45 appearance bytes ([`PopulationPlan`]).
//! Scene bounds for the position are not checked here; for the primary map
//! see `docs/re/209026-scene-bounds.md` (the full-character clamp works in
//! collision units, map units divided by 32, and clamps rather than rejects).
//!
//! Gap: no note states that server 37's Word agent index equals the Dword
//! character index used by 443/183/282/477/151/170/444. This module follows
//! the task's tie (the same index everywhere) and therefore limits the index
//! to the Word range.

use std::io::{Error, ErrorKind, Result};

use bytes::{Bytes, BytesMut};

use crate::agent_creation::{AgentCreation209026, facing_words_209026};
use crate::agent_scene::AgentWorldEntry209026;
use crate::character_setup::CharacterSetupFrame209026;
use crate::core_stats::CoreStatsCreation209026;
use crate::endurance::{EnduranceFrame209026, EnduranceOperation209026};
use crate::guardian::{GuardianFrame209026, GuardianMeter209026, GuardianOperation209026};
use crate::health::{HealthFrame209026, HealthOperation209026, HealthValues209026};
use crate::player_creation::PlayerCreation209026;
use crate::quest::QuestManagerReportFrame209026;
use crate::reward_track::RewardTrackCreationFrame209026;
use crate::skill_availability::{SkillAvailabilityFrame209026, SkillAvailabilityOperation209026};
use crate::skillbar::{SkillbarFrame209026, SkillbarOperation209026};
use crate::wardrobe::{WardrobeFrame209026, WardrobeOperation209026};
use crate::{Guid, Point3};

/// Candidate message order. 256 (health) and 217 (endurance) create the
/// character's health and endurance components; without 256 the owner's client
/// (2026-11-04 01:26Z and 02:08Z, device ArenaNet.log) asserted `m_health`
/// (ChCliCharacter.h:575) from the per-frame ViewAdvanceDevice path
/// (`0x14094d22c` -> `0x14131a777` -> `0x14131ab10` -> health getter
/// `0x1411d7cb0`) once a controlled character existed. Both are sent before 444.
/// 183 is chosen over 182 because 182 and 183 are
/// alternatives (`docs/re/209026-character-core-stats.md`) and 182 would need
/// twelve attribute rows, a Dword and three floats that have no established
/// values. The constructor zero-fills the 182-only fields on the 183 path
/// (`docs/re/209026-character-finalization-dependencies.md`); whether 183
/// alone suffices for the controlled player is still open.
pub const POPULATION_ORDER: &[u16] = &[
    428, 37, 443, 183, 282, 477, 151, 256, 217, 247, 248, 565, 538, 605, 170, 618, 444, 46,
];

/// Caller-supplied inputs. No `Debug`: names and portal identities need not
/// be logged (same stance as [`PlayerCreation209026`]).
#[derive(Clone, PartialEq)]
pub struct PopulationPlan {
    /// Player-table index. Must be nonzero (`docs/re/209026-agent-creation.md`
    /// and `209026-character-finalization-dependencies.md`: a player index of
    /// zero is absent for GetPlayer) and below `0x1000_0000` (server 37's
    /// identity is `0x3000_0000 + player_index`; the high nibble must stay 3).
    pub player_index: u32,
    /// Character index, shared by 37 (as its Word agent index), 443, 183,
    /// 282, 477, 151, 170, 444 and 46. No note forbids zero. Must fit a Word
    /// because 37 and 46 carry it as one (see the module-level gap).
    pub character_index: u32,
    /// Player name for server 428: 1 to 31 nonzero UTF-16 units
    /// (`docs/re/209026-player-creation-fields.md`; `TextValidateLiteral` is
    /// asserted on nonempty text, so no embedded zero units).
    pub name: String,
    /// Synthetic portal identity (`m_portalUserId`). Not established as any
    /// STS field; use a synthetic value, never a real account identifier.
    pub portal_user_id: Guid,
    /// Spawn position in map units, with transform tag zero. Used by both 37
    /// and 46. Must be finite; bounds are the caller's concern.
    pub position: [f32; 3],
    /// Map-space facing direction for 37 and 46. Must be finite and within
    /// the client's unit-vector tolerance (`docs/re/209026-agent-facing.md`).
    pub facing: [f32; 3],
    /// 45 appearance bytes for 183. Byte 27 (race) < 5, byte 28 (scale) <= 15
    /// and byte 29 (sex) <= 1 (`docs/re/209026-appearance-validation.md`).
    /// Content defaults are not available to this crate.
    pub appearance: [u8; 45],
}

impl PopulationPlan {
    /// Check every plan constraint stated in the notes.
    pub fn validate(&self) -> Result<()> {
        if self.player_index == 0 || self.player_index >= 0x1000_0000 {
            return Err(invalid("player index must be 1..0x0fff_ffff"));
        }
        if self.character_index > u32::from(u16::MAX) {
            return Err(invalid("character index must fit the Word agent index"));
        }
        let units = self.name.encode_utf16().count();
        if units == 0
            || units > PlayerCreation209026::MAX_NAME_UNITS
            || self.name.contains('\0')
        {
            return Err(invalid("player name must be 1..=31 nonzero UTF-16 units"));
        }
        if !self.position.iter().all(|v| v.is_finite()) {
            return Err(invalid("position must be finite"));
        }
        if facing_words_209026(self.facing).is_none() {
            return Err(invalid("facing must be finite and unit length"));
        }
        let [race, scale, sex] = [self.appearance[27], self.appearance[28], self.appearance[29]];
        if race >= 5 || scale > 15 || sex > 1 {
            return Err(invalid("appearance race/scale/sex bytes out of range"));
        }
        Ok(())
    }
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidInput, message)
}

/// Remote simulation tick for 37 and 46.
/// placeholder: valid value not established
/// (`docs/re/209026-agent-command-clock.md` shows an arbitrary first timestamp
/// still yields the baseline delay, but no shared epoch is established).
const REMOTE_TICK: u32 = 0;

/// Profession byte of 183 and the 477 creator: Guardian is 1
/// (`docs/re/209026-character-profession.md`: "prepared local character
/// candidate is profession 1"; 477's getter returns 1).
const PROFESSION_GUARDIAN: u8 = 1;

/// Both level bytes of 183. The setters assert 1..=80 for players
/// (`docs/re/209026-character-core-stats.md`). placeholder: the value 1 and
/// the meaning of the second byte are not established.
const LEVEL: u8 = 1;

/// Attitude of 151: must be below 4 (`docs/re/209026-character-core-stats.md`,
/// signed bound `AFFINITY_ATTITUDES`). placeholder: valid value not
/// established; the enum labels and lower bound are unassigned.
const ATTITUDE: u32 = 0;

/// Server-37 collision radius. Must be greater than zero
/// (`docs/re/209026-agent-dimensions.md`). placeholder: valid value not
/// established.
const RADIUS: f32 = 1.0;

/// Server-37 field 13. Index 0 (base speed) must be >= 0 (the simple
/// collision path asserts `speed >= 0`,
/// `docs/re/209026-agent-movement-parameters.md`). Index 3 (height) is kept
/// above both height minimums of `docs/re/209026-agent-dimensions.md`: the
/// map-unit one (`2 * RADIUS`) and the full constructor's second one
/// (`max(array4 / 32, 2 * radius / 32) + 0.1` collision units, about 0.1625,
/// which is 5.2 map units for this radius). Every other element, and the
/// chosen speed and height, are placeholder: valid values not established.
const PARAMETERS: [f32; 8] = [0.0, 0.0, 0.0, 16.0, 0.0, 0.0, 0.0, 0.0];

/// Health for 256: full first meter, no regeneration, empty second meter, state
/// 0 (alive; `docs/re/209026-character-health-endurance.md`: a non-alive player
/// gets character flag +0x178 mask 2). The maximum setter asserts
/// `valueMax >= 0` and current is clamped to [0, maximum]. placeholder: the
/// level-1 Guardian's real health, the second meter and F5 are not established.
const HEALTH: f32 = 1645.0;
const HEALTH_STATE_ALIVE: u8 = 0;

/// Endurance for 217: points, maximum, effective maximum, segment divisor
/// (`docs/re/209026-character-health-endurance.md`: points <= effective maximum
/// is asserted by the setter; divisor 0 reads as 1). placeholder: 100 points in
/// two segments is the familiar default, not established from content.
const ENDURANCE: u32 = 100;
const ENDURANCE_SEGMENTS: u32 = 2;

/// Type-64 data key for 605. Type-64 data keys span 1..=81357
/// (`docs/re/209026-content-numeric-ids.md`) and an unresolved key trips the
/// handler. Key 1080 is the `levelUpSkillDef` record, a type-64 data key that
/// exists in base content (`docs/re/209026-character-control-view.md`).
/// placeholder: which skills are available at startup is not established.
const SKILL_KEY: u32 = 1080;

/// Four type-353 reward-track data keys for 565. Type-353 keys span 1..=529
/// and all 529 tracks have a progress reference
/// (`docs/re/209026-reward-track-content.md`). 24, 100, 103 and 30 are the
/// Guardian specialization tracks listed there. placeholder: the initial
/// track selection is not established.
const REWARD_TRACKS: [u32; 4] = [24, 100, 103, 30];

/// Pin values for 565's four slots (`docs/re/209026-reward-track-slots.md`
/// names them `m_pinnedTracks`). placeholder: the accepted domain is open.
const PINNED: [u8; 4] = [0; 4];

/// Type-23 data key for 247. Keys span 1..=155 and zero is skipped by the
/// index builder (`docs/re/209026-content-numeric-ids.md`); there is no null
/// null assertion on the result, so an unresolved key is stored as null
/// rather than rejected (`docs/re/209026-wardrobe-startup.md`).
/// placeholder: which type-23 record belongs here is not established.
const WARDROBE_TYPE23_KEY: u16 = 1;

fn frame(encode: impl FnOnce(&mut BytesMut) -> Result<()>) -> Result<Bytes> {
    let mut out = BytesMut::new();
    encode(&mut out)?;
    Ok(out.freeze())
}

/// The candidate sequence as `(message id, one complete frame)` pairs, in
/// [`POPULATION_ORDER`]. Validates the plan first.
pub fn population_frames(plan: &PopulationPlan) -> Result<Vec<(u16, Bytes)>> {
    plan.validate()?;
    let player = plan.player_index;
    let character = plan.character_index;
    // Validated above: a finite unit direction and a Word index.
    let facing = facing_words_209026(plan.facing).ok_or_else(|| invalid("facing"))?;
    let agent_index = character as u16;
    let position = Point3(plan.position, 0);
    let mut frames = Vec::with_capacity(POPULATION_ORDER.len());

    // 428: Dword player index, String(32), Guid, Dword flags.
    frames.push((
        428,
        frame(|out| {
            PlayerCreation209026 {
                player_index: player,
                name_units: plan.name.encode_utf16().collect(),
                portal_user_id: plan.portal_user_id,
                // docs/re/209026-player-creation-fields.md and
                // 209026-character-finalization-dependencies.md: bit 0 clear
                // selects state 3 (ordinary view creation at 170 is gated on
                // state 3) and avoids input selector 4; other bits are unused.
                flags: 0,
            }
            .encode_frame(out)
        })?,
    ));

    // 37: type-0 player character agent.
    frames.push((
        37,
        frame(|out| {
            AgentCreation209026 {
                agent_index,
                remote_tick: REMOTE_TICK,
                // docs/re/209026-agent-creation.md: the constructor ORs
                // 0x18080 into this byte, so creation is immediate for any
                // value. placeholder: valid value not established.
                command_flags: 0,
                // docs/re/209026-agent-behavior-data.md: a zero Dword and an
                // empty buffer have no payload dependency on first type-0
                // creation.
                behavior_value: 0,
                behavior_data: Vec::new(),
                // docs/re/209026-agent-creation.md: type 0 is the character
                // factory.
                agent_type: 0,
                // docs/re/209026-agent-creation.md: IsPlayer needs high
                // nibble 3 and the player index is identity - 0x30000000.
                identity: 0x3000_0000 + player,
                // Fields 7, 8, 9 (docs/re/209026-agent-creation-flags.md):
                // field 7 must leave masks 0x08 and 0x20 clear (also keeps
                // 46's ENTERED_WORLD assertion safe). Field 8 = 1 gives
                // factory flag F & 1 (the full factory asserts it; 0x10 and
                // 0x02 must stay clear). Field 9 = 0. The doc's own A=1, B=0
                // example is not bootstrap evidence: constraint-satisfying;
                // otherwise placeholder, valid value not established.
                creation_flags: [0, 1, 0],
                // docs/re/209026-agent-position.md: tag zero selects no extra
                // transform; XYZ are map units from the plan.
                position,
                // docs/re/209026-agent-facing.md: unit length required; the
                // direction comes from the plan.
                facing,
                parameters: PARAMETERS,
                radius: RADIUS,
            }
            .encode_frame(out)
        })?,
    ));

    // 443: owned-character assignment (player index, then character index).
    // docs/re/209026-character-core-stats.md and
    // 209026-character-finalization-dependencies.md: 428, 37, 443, then
    // 182/183, and before 170. Layout: docs/re/209026-agent-creation.md.
    frames.push((
        443,
        frame(|out| {
            CharacterSetupFrame209026::AssignOwned {
                player_index: player,
                character_index: character,
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 183: coreStats creation (alternative to 182; see POPULATION_ORDER).
    frames.push((
        183,
        frame(|out| {
            CoreStatsCreation209026 {
                character_index: character,
                appearance: plan.appearance,
                earned_level: LEVEL,
                level_2: LEVEL,
                // docs/re/209026-character-core-stats.md: the profession
                // scalar; must match the 477 creator.
                profession: PROFESSION_GUARDIAN,
                extended: None,
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 282: inventory component, required before 170
    // (docs/re/209026-character-finalization-dependencies.md).
    frames.push((
        282,
        frame(|out| {
            CharacterSetupFrame209026::CreateInventory {
                character_index: character,
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 477: Guardian profession creator, required before 170
    // (docs/re/209026-character-profession.md).
    frames.push((
        477,
        frame(|out| {
            GuardianFrame209026 {
                character_index: character,
                operation: GuardianOperation209026::Create {
                    // docs/re/209026-character-profession.md: the ratio getter
                    // divides by the maximum without a zero guard, so it must
                    // be nonzero and finite; current is clamped to it.
                    // placeholder: the maximum, rate and current values are
                    // not established.
                    meter: GuardianMeter209026 {
                        maximum: 1.0,
                        rate: 0.0,
                        current: 0.0,
                    },
                    // placeholder: valid values not established (an
                    // unconfigured Guardian never reaches the first-byte
                    // comparison, docs/re/209026-character-profession.md).
                    values: [0, 0],
                },
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 151: attitude. docs/re/209026-character-core-stats.md: the character must
    // exist (an earlier update is ignored) and the value must be below 4.
    frames.push((
        151,
        frame(|out| {
            CharacterSetupFrame209026::Attitude {
                character_index: character,
                attitude: ATTITUDE,
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 256: health component creation (allocates when absent). Required before
    // 444: once the character is controlled, the per-frame device view reads its
    // health and asserts m_health.
    frames.push((
        256,
        frame(|out| {
            HealthFrame209026 {
                character_index: character,
                operation: HealthOperation209026::CreateOrUpdate {
                    state: HEALTH_STATE_ALIVE,
                    values: HealthValues209026 {
                        current: HEALTH,
                        maximum: HEALTH,
                        second_current: 0.0,
                        second_maximum: 0.0,
                        extra: 0.0,
                    },
                    rate: 0.0,
                },
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 217: endurance component creation (asserts that none exists; sent once).
    frames.push((
        217,
        frame(|out| {
            EnduranceFrame209026 {
                character_index: character,
                operation: EnduranceOperation209026::Create {
                    points: ENDURANCE,
                    maximum: ENDURANCE,
                    effective_maximum: ENDURANCE,
                    segment_divisor: ENDURANCE_SEGMENTS,
                },
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 247/248: wardrobe manager. docs/re/209026-wardrobe-startup.md: both
    // lazily create it; needed before 170's model path and required before 431.
    frames.push((
        247,
        frame(|out| {
            WardrobeFrame209026 {
                character_index: character,
                operation: WardrobeOperation209026::AssignType23 {
                    content_id: WARDROBE_TYPE23_KEY,
                },
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));
    frames.push((
        248,
        frame(|out| {
            WardrobeFrame209026 {
                character_index: character,
                // docs/re/209026-wardrobe-startup.md: a first all-zero 248
                // leaves the freshly cleared fields unchanged and dispatches
                // nothing. Not a verified wardrobe state.
                operation: WardrobeOperation209026::UpdateWords { values: [0; 4] },
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 565: reward-track manager (player index). docs/re/209026-reward-track-messages.md:
    // needs an existing player (after 428) and no existing manager.
    frames.push((
        565,
        frame(|out| {
            RewardTrackCreationFrame209026 {
                player_index: player,
                track_ids: REWARD_TRACKS,
                slot_bytes: PINNED,
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 538: quest-manager report (player index).
    // docs/re/209026-content-direction-startup.md: the player must exist and
    // the report must precede 431.
    frames.push((
        538,
        frame(|out| {
            QuestManagerReportFrame209026 { player_index: player }.encode_frame(out);
            Ok(())
        })?,
    ));

    // 605: initial skill availability (player index; the encoder writes the
    // key first). docs/re/209026-skill-availability.md: needs the player
    // entry and a resolvable type-64 key; no owned character is required.
    frames.push((
        605,
        frame(|out| {
            SkillAvailabilityFrame209026 {
                player_index: player,
                operation: SkillAvailabilityOperation209026::InitialSet(SKILL_KEY),
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 170: finalize. docs/re/209026-character-finalization-dependencies.md
    // ("Server 170" row): needs the player (428), attitude (151), owned
    // character (443), coreStats (183), inventory (282) and a profession
    // component (477, docs/re/209026-character-profession.md).
    frames.push((
        170,
        frame(|out| {
            CharacterSetupFrame209026::Finalize {
                character_index: character,
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 618: skill bar. docs/re/209026-character-control-view.md: must precede
    // equal-ID 444 when the ordinary view is subscribed; it carries no
    // skill records (docs/re/209026-character-skillbar.md).
    frames.push((
        618,
        frame(|out| {
            SkillbarFrame209026 {
                character_index: character,
                operation: SkillbarOperation209026::Create,
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 444: controlled-character assignment. docs/re/209026-character-control.md:
    // A == B (both the player index) selects the owned/controlled branch and C
    // is the character index; after 170 and 618.
    frames.push((
        444,
        frame(|out| {
            CharacterSetupFrame209026::AssignControl {
                a: player,
                b: player,
                character_index: character,
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    // 46: world entry. docs/re/209026-agent-world-entry.md and
    // 209026-bootstrap-sequence.md (Phase 5). Its position relative to 444 is
    // open; this candidate order places it last.
    frames.push((
        46,
        frame(|out| {
            AgentWorldEntry209026 {
                agent_index,
                remote_tick: REMOTE_TICK,
                // docs/re/209026-agent-world-entry.md: the constructor ORs
                // only 0x12000, not 0x80; docs/re/209026-agent-point-admission.md:
                // bit 7 then selects immediate over queued execution.
                // placeholder: valid value not established; open.
                command_flags: 0,
                // ORed into the agent collision flags. placeholder: valid
                // value not established.
                collision_flags: 0,
                // docs/re/209026-agent-world-entry.md: option bit 1 must stay
                // clear so the collision scene entry (and so REVEALED) is
                // reached. Bits 0, 2, 3 are placeholder: valid values not
                // established.
                entry_options: 0,
                // Same spawn as 37 (the plan's position and facing).
                position,
                facing,
            }
            .encode_frame(out);
            Ok(())
        })?,
    ));

    debug_assert_eq!(
        frames.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        POPULATION_ORDER
    );
    Ok(frames)
}

/// The frames of [`population_frames`] concatenated in order.
pub fn population_stream(plan: &PopulationPlan) -> Result<BytesMut> {
    let mut out = BytesMut::new();
    for (_, bytes) in population_frames(plan)? {
        out.extend_from_slice(&bytes);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_creation::facing_209026;
    use crate::dynamic::{self, CountPrefix, Rules, StringEncoding, Value};
    use crate::schema::{self, Message};

    fn plan() -> PopulationPlan {
        let mut appearance = [0u8; 45];
        // Placeholder fixture bytes within the documented bounds.
        appearance[27] = 2;
        appearance[28] = 7;
        appearance[29] = 1;
        PopulationPlan {
            player_index: 300,
            character_index: 0x1234,
            name: "Fixture".to_string(),
            portal_user_id: Guid([0x5a; 16]),
            position: [32.0, -64.0, 96.0],
            facing: [0.0, 1.0, 0.0],
            appearance,
        }
    }

    fn frames() -> Vec<(u16, Bytes)> {
        population_frames(&plan()).unwrap()
    }

    /// Decode one frame with its module decoder, re-encode it and return the
    /// bytes plus how much input was consumed.
    fn decode_reencode(id: u16, input: &mut &[u8]) -> Vec<u8> {
        let mut out = BytesMut::new();
        match id {
            428 => PlayerCreation209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out)
                .unwrap(),
            37 => AgentCreation209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out)
                .unwrap(),
            443 | 282 | 151 | 170 | 444 => CharacterSetupFrame209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            256 => HealthFrame209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            217 => EnduranceFrame209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            182 | 183 => CoreStatsCreation209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            477 => GuardianFrame209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            247 | 248 => WardrobeFrame209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            565 => RewardTrackCreationFrame209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            538 => QuestManagerReportFrame209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            605 => SkillAvailabilityFrame209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            618 => SkillbarFrame209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            46 => AgentWorldEntry209026::decode_frame(input)
                .unwrap()
                .encode_frame(&mut out),
            other => panic!("no decoder for {other}"),
        }
        out.to_vec()
    }

    #[test]
    fn frame_ids_match_population_order_and_lead_with_le_id() {
        let frames = frames();
        let ids: Vec<u16> = frames.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, POPULATION_ORDER);
        for (id, bytes) in &frames {
            assert_eq!(&bytes[..2], id.to_le_bytes(), "id {id}");
            assert!(bytes.len() > 2);
        }
    }

    #[test]
    fn every_frame_round_trips_through_its_decoder() {
        for (id, bytes) in frames() {
            let mut input = &bytes[..];
            let again = decode_reencode(id, &mut input);
            assert!(input.is_empty(), "id {id} left bytes");
            assert_eq!(again, &bytes[..], "id {id}");
        }
    }

    #[test]
    fn stream_splits_back_into_exactly_the_frames() {
        let frames = frames();
        let stream = population_stream(&plan()).unwrap();
        assert_eq!(
            stream.len(),
            frames.iter().map(|(_, b)| b.len()).sum::<usize>()
        );
        let mut input = &stream[..];
        for (id, bytes) in &frames {
            let before = input.len();
            let again = decode_reencode(*id, &mut input);
            assert_eq!(before - input.len(), bytes.len(), "id {id}");
            assert_eq!(again, &bytes[..], "id {id}");
        }
        assert!(input.is_empty());
    }

    fn only<T>(frames: &[(u16, Bytes)], id: u16, f: impl FnOnce(&mut &[u8]) -> T) -> T {
        let bytes = &frames.iter().find(|(i, _)| *i == id).unwrap().1;
        let mut input = &bytes[..];
        let value = f(&mut input);
        assert!(input.is_empty());
        value
    }

    #[test]
    fn indices_are_consistent_across_frames() {
        use CharacterSetupFrame209026::*;
        let plan = plan();
        let frames = frames();
        let character = plan.character_index;
        let player = plan.player_index;
        let setup = |id| only(&frames, id, |i| CharacterSetupFrame209026::decode_frame(i).unwrap());

        assert_eq!(
            setup(443),
            AssignOwned { player_index: player, character_index: character }
        );
        assert_eq!(setup(282), CreateInventory { character_index: character });
        assert_eq!(setup(170), Finalize { character_index: character });
        match setup(151) {
            Attitude { character_index, attitude } => {
                assert_eq!(character_index, character);
                assert!(attitude < 4);
            }
            other => panic!("{other:?}"),
        }
        match setup(444) {
            AssignControl { a, b, character_index } => {
                assert_eq!(a, b);
                assert_eq!(b, player);
                assert_eq!(character_index, character);
            }
            other => panic!("{other:?}"),
        }

        let created = only(&frames, 37, |i| AgentCreation209026::decode_frame(i).unwrap());
        assert_eq!(u32::from(created.agent_index), character);
        assert_eq!(created.identity, 0x3000_0000 + player);
        assert_eq!(created.identity & 0xf000_0000, 0x3000_0000);
        assert_eq!(created.agent_type, 0);
        assert_eq!(created.creation_flags[0] & 0x28, 0);
        assert_ne!(created.creation_flags[1] & 0x09, 0);
        assert_eq!(created.creation_flags[1] & 0x12, 0);
        assert_eq!(created.position, Point3(plan.position, 0));
        assert!(created.radius > 0.0);
        assert!(created.parameters[0] >= 0.0);
        let facing = facing_209026(created.facing[0], created.facing[1]).unwrap();
        let squared: f32 = facing.iter().map(|v| v * v).sum();
        assert!((0.9801..=1.0201).contains(&squared));

        let entry = only(&frames, 46, |i| AgentWorldEntry209026::decode_frame(i).unwrap());
        assert_eq!(u32::from(entry.agent_index), character);
        assert_eq!(entry.position, created.position);
        assert_eq!(entry.facing, created.facing);
        assert_eq!(entry.entry_options & 0b10, 0);

        let core = only(&frames, 183, |i| CoreStatsCreation209026::decode_frame(i).unwrap());
        assert_eq!(core.character_index, character);
        assert_eq!(core.appearance, plan.appearance);
        assert!((1..=80).contains(&core.earned_level));
        assert!((1..=80).contains(&core.level_2));
        assert_eq!(core.profession, 1);
        assert!(core.extended.is_none());

        let player_frame = only(&frames, 428, |i| PlayerCreation209026::decode_frame(i).unwrap());
        assert_eq!(player_frame.player_index, player);
        assert_ne!(player_frame.player_index, 0);
        assert_eq!(player_frame.flags & 1, 0);
        assert_eq!(player_frame.portal_user_id, plan.portal_user_id);
        assert_eq!(
            player_frame.name_units,
            plan.name.encode_utf16().collect::<Vec<_>>()
        );

        let guardian = only(&frames, 477, |i| GuardianFrame209026::decode_frame(i).unwrap());
        assert_eq!(guardian.character_index, character);
        match guardian.operation {
            GuardianOperation209026::Create { meter, .. } => {
                assert!(meter.maximum.is_finite() && meter.maximum > 0.0);
                assert!(meter.current <= meter.maximum);
            }
            other => panic!("{other:?}"),
        }

        for id in [247, 248] {
            let wardrobe = only(&frames, id, |i| WardrobeFrame209026::decode_frame(i).unwrap());
            assert_eq!(wardrobe.character_index, character);
            if let WardrobeOperation209026::AssignType23 { content_id } = wardrobe.operation {
                assert!((1..=155).contains(&content_id));
            }
        }
        let rewards = only(&frames, 565, |i| {
            RewardTrackCreationFrame209026::decode_frame(i).unwrap()
        });
        assert_eq!(rewards.player_index, player);
        assert!(rewards.track_ids.iter().all(|k| (1..=529).contains(k)));
        let quest = only(&frames, 538, |i| {
            QuestManagerReportFrame209026::decode_frame(i).unwrap()
        });
        assert_eq!(quest.player_index, player);
        let skills = only(&frames, 605, |i| {
            SkillAvailabilityFrame209026::decode_frame(i).unwrap()
        });
        assert_eq!(skills.player_index, player);
        assert!(matches!(
            skills.operation,
            SkillAvailabilityOperation209026::InitialSet(key) if (1..=81357).contains(&key)
        ));
        let bar = only(&frames, 618, |i| SkillbarFrame209026::decode_frame(i).unwrap());
        assert_eq!(bar.character_index, character);
        assert_eq!(bar.operation, SkillbarOperation209026::Create);
    }

    fn character_schema_messages() -> Vec<Message> {
        // Protocol 0, group 0, server messages only: mission.xml and auth.xml
        // reuse IDs 37, 428 and 443 in other groups, so they are excluded.
        let sources = [
            include_str!("../../../data/protocol/209026/agent-creation.xml"),
            include_str!("../../../data/protocol/209026/character-population.xml"),
            include_str!("../../../data/protocol/209026/character-core-stats.xml"),
            include_str!("../../../data/protocol/209026/character-inventory.xml"),
            include_str!("../../../data/protocol/209026/character-control.xml"),
            include_str!("../../../data/protocol/209026/character-health-endurance.xml"),
        ];
        sources
            .iter()
            .flat_map(|xml| {
                let schema = schema::parse(xml).unwrap();
                assert_eq!(schema.protocols[0].name, "0");
                schema.protocols[0].groups[0].server.clone()
            })
            .collect()
    }

    fn u32_fields(value: &Value) -> Vec<u32> {
        match value {
            Value::Struct(fields) => fields
                .iter()
                .filter_map(|(_, v)| match v {
                    Value::U32(n) => Some(*n),
                    _ => None,
                })
                .collect(),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn schema_covered_frames_decode_with_the_census_decoder() {
        let messages = character_schema_messages();
        let rules = Rules {
            string: Some(StringEncoding::Utf16Terminated),
            buffer_var_small: Some(CountPrefix::U8),
            buffer_fixed_is_size_bytes: true,
            ..Rules::default()
        };
        let plan = plan();
        let covered = [428u16, 37, 443, 183, 282, 151, 256, 217, 170, 444];
        for (id, bytes) in frames() {
            let mut input = &bytes[..];
            if !covered.contains(&id) {
                continue;
            }
            let (message, value) = dynamic::decode_frame(&messages, &rules, &mut input).unwrap();
            assert_eq!(message.id, id);
            assert!(input.is_empty(), "id {id} left bytes");
            let dwords = u32_fields(&value);
            let (player, character) = (plan.player_index, plan.character_index);
            match id {
                428 => assert_eq!(dwords[0], player),
                443 => assert_eq!(dwords, [player, character]),
                282 | 170 | 183 | 256 => assert_eq!(dwords[0], character),
                217 => assert_eq!(dwords, [character, ENDURANCE, ENDURANCE, ENDURANCE, ENDURANCE_SEGMENTS]),
                151 => assert_eq!(dwords[0], character),
                444 => assert_eq!(dwords, [player, player, character]),
                37 => assert_eq!(dwords.last(), Some(&(0x3000_0000 + player))),
                _ => unreachable!(),
            }
        }
        // Server 477 and the other frames have no schema entry here; they are
        // covered by their typed decoders above.
    }

    #[test]
    fn plan_validation_rejects_bad_inputs() {
        let bad = |edit: &dyn Fn(&mut PopulationPlan)| {
            let mut p = plan();
            edit(&mut p);
            assert_eq!(
                population_frames(&p).unwrap_err().kind(),
                ErrorKind::InvalidInput
            );
            assert_eq!(
                population_stream(&p).unwrap_err().kind(),
                ErrorKind::InvalidInput
            );
        };
        bad(&|p| p.player_index = 0);
        bad(&|p| p.player_index = 0x1000_0000);
        bad(&|p| p.character_index = 0x1_0000);
        bad(&|p| p.name.clear());
        bad(&|p| p.name = "a".repeat(32));
        bad(&|p| p.name = "ab\0cd".into());
        // 16 supplementary-plane chars are 16 chars but 32 UTF-16 units.
        bad(&|p| p.name = "\u{1F600}".repeat(16));
        bad(&|p| p.position[1] = f32::NAN);
        bad(&|p| p.position[2] = f32::INFINITY);
        bad(&|p| p.facing = [0.0, 0.0, 0.0]);
        bad(&|p| p.facing = [f32::NAN, 0.0, 1.0]);
        bad(&|p| p.appearance[27] = 5);
        bad(&|p| p.appearance[28] = 16);
        bad(&|p| p.appearance[29] = 2);
    }

    #[test]
    fn plan_validation_accepts_documented_bounds() {
        let mut p = plan();
        p.name = "a".repeat(31);
        p.player_index = 0x0fff_ffff;
        p.character_index = 0;
        p.appearance = [0; 45];
        population_frames(&p).unwrap();
        // 15 supplementary-plane chars are 30 UTF-16 units; one more char
        // would make 32.
        p.name = "\u{1F600}".repeat(15);
        population_frames(&p).unwrap();
        p.character_index = u32::from(u16::MAX);
        population_frames(&p).unwrap();
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(frames(), frames());
        assert_eq!(
            population_stream(&plan()).unwrap(),
            population_stream(&plan()).unwrap()
        );
        let mut other = plan();
        other.character_index += 1;
        assert_ne!(
            population_stream(&plan()).unwrap(),
            population_stream(&other).unwrap()
        );
    }
}
