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
