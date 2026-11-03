//! The typed in-world server frames, checked against the client's own server
//! registration tables (data/protocol/209026/server-tables.xml, extracted from the
//! frozen build-209026 client; docs/re/209026-agent-client-messages.md). Each sample
//! frame must decode to its exact length under the descriptor the client registers for
//! that id in the game tables. Framing only: value validity is not checked.
use bytes::BytesMut;
use gw2_proto::{
    Point3,
    agent_position::{AgentPosition209026, AgentPositionState209026},
    agent_scene::{AgentSceneEntry209026, AgentWorldEntry209026},
    dynamic::{self, CStringEncoding, CountPrefix, Rules, StringEncoding},
    schema,
};

const SERVER_TABLES: &str = include_str!("../../../data/protocol/209026/server-tables.xml");
const GAME: [&str; 4] = ["table-0x14216e940", "table-0x14211be70", "table-0x1422ddbf0", "table-0x141923b80"];

fn rules() -> Rules {
    Rules {
        string: Some(StringEncoding::Utf16Terminated),
        cstring: Some(CStringEncoding::Terminated),
        array_var_small: Some(CountPrefix::U8),
        array_var_large: Some(CountPrefix::U16),
        buffer_var_small: Some(CountPrefix::U8),
        buffer_var_large: Some(CountPrefix::U16),
        buffer_fixed_is_size_bytes: true,
    }
}

/// Decode one frame under the first game table that registers its id; exact length.
fn check(name: &str, frame: &BytesMut) -> u16 {
    let groups = schema::parse(SERVER_TABLES).unwrap().protocols[0].groups.clone();
    let id = u16::from_le_bytes([frame[0], frame[1]]);
    let table = GAME
        .iter()
        .find_map(|g| groups.iter().find(|t| t.name == *g && t.server.iter().any(|m| m.id == id)))
        .unwrap_or_else(|| panic!("{name}: server {id} has no handler in the client's game tables"));
    let mut cursor = &frame[..];
    dynamic::decode_frame(&table.server, &rules(), &mut cursor)
        .unwrap_or_else(|e| panic!("{name}: server {id} misfits {}: {e}", table.name));
    assert!(cursor.is_empty(), "{name}: server {id} leaves {} bytes under {}", cursor.len(), table.name);
    id
}

#[test]
fn agent_position_and_scene_frames_match_client_layouts() {
    let point = Point3([1.0, 2.0, 7.4], 0);
    let mut frames = Vec::new();
    let mut f = BytesMut::new();
    AgentPosition209026 { agent_index: 1, remote_tick: 300, command_flags: 0, position: point }.encode_frame(&mut f);
    frames.push(("86 position", f));
    let mut f = BytesMut::new();
    AgentPositionState209026 {
        agent_index: 1,
        remote_tick: 300,
        command_flags: 0,
        position: [1.0, 2.0, 7.4, 0.0],
        transform_tag: 0,
        second_vector: [0.0; 4],
        third_vector: [0.0, 0.0, 0.0, 1.0],
        state_bytes: [0, 0],
    }
    .encode_frame(&mut f);
    frames.push(("87 position state", f));
    let mut f = BytesMut::new();
    AgentWorldEntry209026 {
        agent_index: 1,
        remote_tick: 300,
        command_flags: 0,
        collision_flags: 0,
        entry_options: 0,
        position: point,
        facing: [0, 0],
    }
    .encode_frame(&mut f);
    frames.push(("46 world entry", f));
    let mut f = BytesMut::new();
    AgentSceneEntry209026 { agent_index: 1, remote_tick: 300, command_flags: 0 }.encode_frame(&mut f);
    frames.push(("49 scene entry", f));
    let ids: Vec<u16> = frames.iter().map(|(n, f)| check(n, f)).collect();
    assert_eq!(ids, vec![86, 87, 46, 49]);
}

#[test]
fn health_and_endurance_frames_match_client_layouts() {
    use gw2_proto::endurance::{EnduranceFrame209026, EnduranceOperation209026 as E};
    use gw2_proto::health::{HealthFrame209026, HealthOperation209026 as H, HealthValues209026};
    let values = HealthValues209026 { current: 100.0, maximum: 100.0, second_current: 0.0, second_maximum: 0.0, extra: 0.0 };
    let health = [
        H::CreateOrUpdate { state: 0, values, rate: 0.0 },
        H::Currents { current: 90.0, second_current: 0.0 },
        H::Remove,
        H::Values(values),
        H::Rate(1.0),
    ];
    let mut ids = Vec::new();
    for operation in health {
        let mut f = BytesMut::new();
        HealthFrame209026 { character_index: 1, operation }.encode_frame(&mut f);
        ids.push(check("health", &f));
    }
    let endurance = [
        E::Create { points: 100, maximum: 100, effective_maximum: 100, segment_divisor: 2 },
        E::Remove,
        E::Points(50),
        E::Maxima { maximum: 100, effective_maximum: 100 },
        E::SegmentDivisor(2),
    ];
    for operation in endurance {
        let mut f = BytesMut::new();
        EnduranceFrame209026 { character_index: 1, operation }.encode_frame(&mut f);
        ids.push(check("endurance", &f));
    }
    assert_eq!(ids, vec![256, 257, 258, 259, 260, 217, 218, 219, 220, 221]);
}

#[test]
fn profession_skill_and_view_frames_match_client_layouts() {
    use gw2_proto::character_view::CharacterViewSuppression209026;
    use gw2_proto::guardian::{GuardianFrame209026, GuardianMeter209026, GuardianOperation209026 as G};
    use gw2_proto::skill_availability::{SkillAvailabilityFrame209026, SkillAvailabilityOperation209026 as A};
    use gw2_proto::skill_palette::SkillPaletteFrame209026;
    use gw2_proto::skillbar::{SkillbarFrame209026, SkillbarOperation209026 as S};
    let meter = GuardianMeter209026 { maximum: 100.0, rate: 0.0, current: 0.0 };
    let mut ids = Vec::new();
    for operation in [
        G::Create { meter, values: [0, 0] },
        G::Remove,
        G::Meter { meter, trailing: [0.0, 0.0] },
        G::Values([0, 0]),
    ] {
        let mut f = BytesMut::new();
        GuardianFrame209026 { character_index: 1, operation }.encode_frame(&mut f);
        ids.push(check("guardian", &f));
    }
    for operation in [S::Create, S::Notify, S::Remove, S::ClearSlotFlag(0), S::AutoAttackSlot(255), S::AutoAttackMask(0)] {
        let mut f = BytesMut::new();
        SkillbarFrame209026 { character_index: 1, operation }.encode_frame(&mut f);
        ids.push(check("skillbar", &f));
    }
    let mut f = BytesMut::new();
    SkillPaletteFrame209026 { palette_key: 0, slot: 0, context: 1, player_index: 1 }.encode_frame(&mut f);
    ids.push(check("skill palette", &f));
    for operation in [A::InitialSet(1), A::Reset, A::Remove(1), A::SetAndNotify { skill_key: 1, flags: 0 }] {
        let mut f = BytesMut::new();
        SkillAvailabilityFrame209026 { player_index: 1, operation }.encode_frame(&mut f);
        ids.push(check("skill availability", &f));
    }
    let mut f = BytesMut::new();
    CharacterViewSuppression209026 { character_index: 1, value: 0 }.encode_frame(&mut f);
    ids.push(check("view suppression", &f));
    assert_eq!(ids, vec![477, 478, 479, 480, 618, 619, 620, 621, 622, 623, 612, 605, 606, 609, 613, 585]);
}

#[test]
fn core_stats_and_specialization_frames_match_client_layouts() {
    use gw2_proto::core_stats::{AttributeRecord209026, CoreStatsCreation209026, ExtendedCoreStats209026};
    use gw2_proto::specialization::{SpecializationFrame209026, SpecializationOperation209026 as P};
    let mut ids = Vec::new();
    for extended in [
        Some(ExtendedCoreStats209026 {
            value: 0,
            attributes: [AttributeRecord209026 { value_0: 0, value_1: 0, value_2: -1 }; 12],
            values: [0.0; 3],
        }),
        None,
    ] {
        let mut f = BytesMut::new();
        CoreStatsCreation209026 { character_index: 1, appearance: [0; 45], earned_level: 1, level_2: 0, profession: 1, extended }
            .encode_frame(&mut f);
        ids.push(check("core stats", &f));
    }
    for operation in [
        P::Assign { specialization_uid: 1, slot: 0 },
        P::AssignMajorTrait { trait_uid: 1, specialization_slot: 0, major_trait_slot: 0 },
        P::Replace { specialization_uids: [0; 3], trait_uids: [[0; 3]; 3], notification: 0 },
        P::Swap { first_slot: 0, second_slot: 1 },
    ] {
        let mut f = BytesMut::new();
        SpecializationFrame209026 { player_index: 1, operation }.encode_frame(&mut f);
        ids.push(check("specialization", &f));
    }
    assert_eq!(&ids[..2], &[182, 183]);
    assert_eq!(ids.len(), 6);
}

#[test]
fn quest_reward_progress_wardrobe_and_behavior_frames_match_client_layouts() {
    use gw2_proto::agent_behavior::AgentBehaviorFrame209026;
    use gw2_proto::progress::{BitMaskProgressFrame209026, BitProgressFrame209026, CounterProgressFrame209026};
    use gw2_proto::quest::QuestManagerReportFrame209026;
    use gw2_proto::reward_track::RewardTrackCreationFrame209026;
    use gw2_proto::wardrobe::{WardrobeFrame209026, WardrobeOperation209026 as W};
    let mut ids = Vec::new();
    let mut f = BytesMut::new();
    QuestManagerReportFrame209026 { player_index: 1 }.encode_frame(&mut f);
    ids.push(check("quest report", &f));
    let mut f = BytesMut::new();
    RewardTrackCreationFrame209026 { player_index: 1, track_ids: [0; 4], slot_bytes: [0; 4] }.encode_frame(&mut f);
    ids.push(check("reward track", &f));
    let mut f = BytesMut::new();
    BitProgressFrame209026 { progress_key: 1, channel_mask: 1 }.encode_frame(&mut f);
    ids.push(check("bit progress", &f));
    let mut f = BytesMut::new();
    BitMaskProgressFrame209026 { progress_key: 1, words: vec![0, 1] }.encode_frame(&mut f).unwrap();
    ids.push(check("bitmask progress", &f));
    let mut f = BytesMut::new();
    CounterProgressFrame209026 { progress_key: 1, values: vec![0, 1, 2, 3] }.encode_frame(&mut f).unwrap();
    ids.push(check("counter progress", &f));
    for operation in [W::AssignType23 { content_id: 1 }, W::UpdateWords { values: [0; 4] }] {
        let mut f = BytesMut::new();
        WardrobeFrame209026 { character_index: 1, operation }.encode_frame(&mut f);
        ids.push(check("wardrobe", &f));
    }
    let mut f = BytesMut::new();
    AgentBehaviorFrame209026 { agent_index: 1, remote_tick: 300, command_flags: 0x10, behavior_value: 0, behavior_data: vec![] }
        .encode_frame(&mut f)
        .unwrap();
    ids.push(check("agent behavior", &f));
    assert_eq!(ids.len(), 8);
    assert_eq!(*ids.last().unwrap(), 84);
}
