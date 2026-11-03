//! Opt-in offline mission (game server) phase, not a game server.
//! Ordering and field roles: docs/re/209026-mission-entry.md.
//!
//! The client completes the game join by itself on the type-1 key reply, so
//! the phase opens with server 1063 (MapData) for the route's map key, then
//! 1060 (LoadMap) with placeholder values. Client frames are then framed
//! through the mission schema and logged by ID only. It never sends 1267
//! (unnecessary: the join transaction has already completed). It never sends
//! 1036 BeginPlaying before client 412 AgentsLoaded: the client asserts loader
//! READY on it, and READY needs population data, which is opt-in (below).
//!
//! With an explicit [`PopulationFixture`] the phase additionally sends the
//! candidate population sequence (`gw2_proto::population`) once client 420
//! MapLoaded arrives, and empty server 1036 BeginPlaying followed by the
//! payload-free 431 player startup once client 412 AgentsLoaded arrives after
//! that. 431's direct body needs the populated player but checks no loader or
//! view state (docs/re/209026-mission-entry.md); its order after 1036 follows
//! the bootstrap candidate order. Both are candidates from static analysis
//! (docs/re/209026-bootstrap-sequence.md), not a validated bootstrap.
use bytes::{Bytes, BytesMut};
use gw2_proto::{
    Guid,
    character_summary::SummaryV11,
    dynamic::{self, CStringEncoding, CountPrefix, Rules, StringEncoding},
    map_load::{self, ClientMapEvent, LoadMap1060, MapData1063},
    player_startup::PlayerStartupFrame209026,
    population::{self, PopulationPlan},
    schema::{self, Message},
};
use std::io::{self, ErrorKind};

const MISSION_SCHEMA: &str = include_str!("../../../../../data/protocol/209026/mission.xml");
/// Group 0 (AgMsg.cpp) client messages: the agent traffic the client sends once
/// in the world, e.g. client 17 (delay report). Extracted by
/// ~/re/gw2/notes/extract-agent-client.py; kind-15 (Optional) fields stay
/// unverified and fail framing explicitly.
const AGENT_CLIENT_SCHEMA: &str = include_str!("../../../../../data/protocol/209026/agent-client.xml");
const PENDING_LIMIT: usize = 65536;

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

/// Client frames the phase recognizes; everything else is framed by schema
/// and reported by ID only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    Map(ClientMapEvent),
    Other(u16),
}

/// Explicit synthetic population input, built from the same server-17
/// character record the auth phase serves. No Debug: it holds a name.
#[derive(Clone)]
pub struct PopulationFixture {
    plan: PopulationPlan,
    map_key: u16,
}

impl PopulationFixture {
    /// `spawn` is an operator-chosen packet-space position; valid spawn
    /// coordinates are not established (docs/re/209026-map-coordinate-spaces.md).
    pub fn new(record: &[u8], spawn: [f32; 3]) -> io::Result<Self> {
        let invalid = |message: &str| io::Error::new(ErrorKind::InvalidInput, message.to_owned());
        let (name, summary) = super::auth::character_parts(record)?;
        let summary = SummaryV11::parse(&summary)?;
        // Only the Guardian (profession 1) creator, server 477, is implemented.
        if summary.profession() != 1 {
            return Err(invalid("population fixture supports only profession 1 (Guardian)"));
        }
        let plan = PopulationPlan {
            player_index: 1,        // placeholder: valid value not established
            character_index: 1,     // placeholder: valid value not established
            name,
            portal_user_id: Guid([0; 16]), // placeholder: synthetic identity
            position: spawn,
            facing: [1.0, 0.0, 0.0], // placeholder: unit vector, direction unestablished
            appearance: summary
                .appearance()
                .try_into()
                .map_err(|_| invalid("summary appearance is not 45 bytes"))?,
        };
        plan.validate()?;
        population::population_stream(&plan)?;
        Ok(Self {
            plan,
            map_key: summary.map_definition_key(),
        })
    }

    /// The summary's map key, which the client also sends in route 29.
    pub fn map_key(&self) -> u16 {
        self.map_key
    }
}

pub struct MissionPhase {
    map_key: u32,
    client: Vec<Message>,
    pending: BytesMut,
    enabled: bool,
    population: Option<PopulationFixture>,
    populated: bool,
    began: bool,
}

impl MissionPhase {
    /// `map_key` must equal the map key the auth phase returned in route
    /// result 20 field 3 (normally the character summary's +1 Word); the
    /// client asserts both resolve to one definition.
    pub fn new(map_key: u32) -> io::Result<Self> {
        if MapData1063::for_map(map_key).is_none() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "map key must fit in 30 bits",
            ));
        }
        let invalid = |_| io::Error::new(ErrorKind::InvalidData, "invalid embedded mission schema");
        let parsed = schema::parse(MISSION_SCHEMA).map_err(invalid)?;
        let client = parsed
            .protocols
            .into_iter()
            .find(|p| p.name == "0")
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "mission schema lacks protocol 0"))?
            .groups
            .into_iter()
            .filter(|g| g.name == "39" || g.name == "40")
            .flat_map(|g| g.client)
            .collect::<Vec<_>>();
        if client.is_empty() {
            return Err(io::Error::new(ErrorKind::InvalidData, "mission schema lacks client messages"));
        }
        let mut client = client;
        let agent = schema::parse(AGENT_CLIENT_SCHEMA).map_err(invalid)?;
        let agent_client = agent
            .protocols
            .into_iter()
            .find(|p| p.name == "0")
            .and_then(|p| p.groups.into_iter().find(|g| g.name == "0"))
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "agent schema lacks group 0"))?
            .client;
        for message in agent_client {
            if client.iter().any(|m| m.id == message.id) {
                return Err(io::Error::new(ErrorKind::InvalidData, "agent and mission client IDs collide"));
            }
            client.push(message);
        }
        Ok(Self {
            map_key,
            client,
            pending: BytesMut::new(),
            enabled: true,
            population: None,
            populated: false,
            began: false,
        })
    }

    /// Enable the population step. Its summary map key must equal the
    /// phase's map key: the client derives both from the same summary, and a
    /// mismatch would trip the loader's `m_mapDef == mapDef` assertion.
    pub fn with_population(mut self, fixture: PopulationFixture) -> io::Result<Self> {
        if u32::from(fixture.map_key()) != self.map_key {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "character summary map key differs from --mission-map-key",
            ));
        }
        self.population = Some(fixture);
        Ok(self)
    }

    /// Server frames owed for one framed client message: population after
    /// MapLoaded (once), then BeginPlaying after AgentsLoaded (once, and only
    /// after population). Nothing without a population fixture.
    pub fn respond(&mut self, seen: &Seen) -> io::Result<Option<Bytes>> {
        let Some(fixture) = &self.population else {
            return Ok(None);
        };
        match seen {
            Seen::Map(ClientMapEvent::MapLoaded) if !self.populated => {
                self.populated = true;
                eprintln!("Mission phase: sending candidate population sequence");
                Ok(Some(population::population_stream(&fixture.plan)?.freeze()))
            }
            Seen::Map(ClientMapEvent::AgentsLoaded) if self.populated && !self.began => {
                self.began = true;
                eprintln!("Mission phase: sending server 1036 BeginPlaying and 431 player startup");
                let mut out = BytesMut::new();
                map_load::encode_empty(map_load::BEGIN_PLAYING_1036, &mut out);
                PlayerStartupFrame209026.encode_frame(&mut out);
                Ok(Some(out.freeze()))
            }
            _ => Ok(None),
        }
    }

    #[cfg(test)]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Server 1063 then 1060, sent right after the type-1 key reply. 1063
    /// carries the map ID and, with population enabled, the local player ID
    /// (`MissionCliGetPlayerId()`) set to the plan's player index (inferred to
    /// be the same index space); every other value is a placeholder.
    pub fn opening(&self) -> Bytes {
        let mut out = BytesMut::new();
        MapData1063 {
            player_id: self
                .population
                .as_ref()
                .map_or(0, |fixture| fixture.plan.player_index),
            ..MapData1063::for_map(self.map_key).expect("validated in new")
        }
        .encode_frame(&mut out);
        LoadMap1060 {
            position: [0.0; 3],
            vectors: [[0.0; 2]; 2],
            diagnostic: 0,
            angle: 0,
            loader_value: 0,
            final_value: 0,
        }
        .encode_frame(&mut out);
        out.freeze()
    }

    /// Frame decrypted client bytes. Incomplete input is retained; an
    /// unframeable message disables the phase (capture continues).
    pub fn push(&mut self, bytes: &[u8]) -> Vec<Seen> {
        let mut seen = Vec::new();
        if !self.enabled {
            return seen;
        }
        if self.pending.len() + bytes.len() > PENDING_LIMIT {
            eprintln!("Mission phase disabled: input exceeds bound");
            self.disable();
            return seen;
        }
        self.pending.extend_from_slice(bytes);
        let rules = rules();
        while !self.pending.is_empty() {
            let mut cursor = &self.pending[..];
            let id = match dynamic::decode_frame(&self.client, &rules, &mut cursor) {
                Ok((message, _)) => message.id,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => {
                    eprintln!("Mission phase disabled: {e}");
                    self.disable();
                    break;
                }
            };
            let used = self.pending.len() - cursor.len();
            let frame = self.pending.split_to(used);
            let mut input = &frame[..];
            let event = match ClientMapEvent::decode_frame(&mut input) {
                Ok(event) if input.is_empty() => Seen::Map(event),
                _ => Seen::Other(id),
            };
            match event {
                Seen::Map(ClientMapEvent::LanguagePrefs(prefs)) => eprintln!(
                    "Mission phase: client 423 language preferences (in range: {})",
                    prefs.in_range()
                ),
                Seen::Map(ClientMapEvent::MapLoaded) => eprintln!("Mission phase: client 420 MapLoaded"),
                Seen::Map(ClientMapEvent::FailedToLoad) => {
                    eprintln!("Mission phase: client 418 FailedToLoad")
                }
                Seen::Map(ClientMapEvent::AgentsLoaded) => {
                    eprintln!("Mission phase: client 412 AgentsLoaded")
                }
                Seen::Other(id) => eprintln!("Mission phase: client message {id} ({used} bytes)"),
            }
            seen.push(event);
        }
        seen
    }

    fn disable(&mut self) {
        self.enabled = false;
        self.pending.clear();
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use gw2_proto::{Encode, map_load::LanguagePrefs423};

    fn client_input() -> BytesMut {
        let mut input = BytesMut::new();
        LanguagePrefs423 { text: 0, audio: 0 }.encode_frame(&mut input);
        // Mission control client 2 (group 40): one Dword.
        2u16.encode(&mut input);
        40u32.encode(&mut input);
        420u16.encode(&mut input);
        input
    }

    #[test]
    fn opening_is_map_data_then_load_map_for_the_route_key() {
        let phase = MissionPhase::new(15).unwrap();
        let opening = phase.opening();
        let mut input = &opening[..];
        let data = MapData1063::decode_frame(&mut input).unwrap();
        assert_eq!(data, MapData1063::for_map(15).unwrap());
        LoadMap1060::decode_frame(&mut input).unwrap();
        assert!(input.is_empty());
        // Never 1036 or 1267.
        assert!(!opening.windows(2).any(|w| w == [0x0c, 0x04] || w == [0xf3, 0x04]));
        assert!(MissionPhase::new(0x4000_0000).is_err());
    }

    #[test]
    fn push_frames_client_messages_for_every_split() {
        let input = client_input();
        let expected = vec![
            Seen::Map(ClientMapEvent::LanguagePrefs(LanguagePrefs423 { text: 0, audio: 0 })),
            Seen::Other(2),
            Seen::Map(ClientMapEvent::MapLoaded),
        ];
        for split in 0..=input.len() {
            let mut phase = MissionPhase::new(15).unwrap();
            let mut seen = phase.push(&input[..split]);
            seen.extend(phase.push(&input[split..]));
            assert_eq!(seen, expected, "split {split}");
            assert!(phase.enabled());
        }
    }

    /// Synthetic server-17 record: Guardian, Queensdale key 15, zero appearance.
    pub(crate) fn guardian_record(map_key: u16, profession: u8) -> Vec<u8> {
        let mut record = vec![0; 16]; // Synthetic character GUID.
        record.push(0); // Opaque Dword.
        record.extend_from_slice(&[0; 16]); // Opaque Qwords.
        for unit in "Fixture".encode_utf16().chain(std::iter::once(0)) {
            record.extend_from_slice(&unit.to_le_bytes());
        }
        record.extend_from_slice(&159u16.to_le_bytes());
        let mut summary = [0u8; 159];
        summary[0] = 11;
        summary[1..3].copy_from_slice(&map_key.to_le_bytes());
        summary[4] = 1;
        summary[5] = profession;
        record.extend_from_slice(&summary);
        record
    }

    fn populated_phase() -> MissionPhase {
        let fixture = PopulationFixture::new(&guardian_record(15, 1), [0.0, 0.0, 0.0]).unwrap();
        MissionPhase::new(15).unwrap().with_population(fixture).unwrap()
    }

    /// One group-0 client frame built from the extracted schema with zero values.
    fn agent_frame(id: u16) -> BytesMut {
        let parsed = schema::parse(AGENT_CLIENT_SCHEMA).unwrap();
        let message = parsed.protocols[0].groups[0]
            .client
            .iter()
            .find(|m| m.id == id)
            .unwrap()
            .clone();
        let values = message
            .fields
            .iter()
            .map(|f| {
                let value = match f.kind.as_str() {
                    "Byte" => dynamic::Value::U8(0),
                    "Word" => dynamic::Value::U16(250),
                    "Dword" => dynamic::Value::U32(0x1234),
                    "Float" => dynamic::Value::F32(1.5),
                    "Float3" => dynamic::Value::Floats(vec![1.0, 2.0, 7.4]),
                    other => panic!("test fixture lacks kind {other}"),
                };
                (f.name.clone(), value)
            })
            .collect();
        let mut out = BytesMut::new();
        id.encode(&mut out);
        dynamic::encode_message(&message, &rules(), &dynamic::Value::Struct(values), &mut out).unwrap();
        out
    }

    #[test]
    fn in_world_agent_messages_frame_for_every_split_and_keep_the_phase() {
        // Client 17 (delay report, one Word) and 18 (Dword Dword Float3 Dword Byte x3).
        let mut input = agent_frame(17);
        input.extend_from_slice(&agent_frame(18));
        input.extend_from_slice(&agent_frame(17));
        for split in 0..=input.len() {
            let mut phase = populated_phase();
            let mut seen = phase.push(&input[..split]);
            seen.extend(phase.push(&input[split..]));
            assert_eq!(seen, vec![Seen::Other(17), Seen::Other(18), Seen::Other(17)], "split {split}");
            assert!(phase.enabled(), "split {split}");
            assert!(phase.respond(&Seen::Other(17)).unwrap().is_none(), "no reply to agent traffic");
        }
    }

    /// Agent traffic between 420 MapLoaded and 412 AgentsLoaded (the client's
    /// delay report can start before 412) must not disable the phase: before the
    /// agent schema was loaded it did, and 1036 was then never sent.
    #[test]
    fn agent_traffic_between_map_loaded_and_agents_loaded_still_begins_playing() {
        let mut input = BytesMut::new();
        map_load::encode_empty(map_load::MAP_LOADED_420, &mut input);
        input.extend_from_slice(&agent_frame(17));
        input.extend_from_slice(&agent_frame(18));
        map_load::encode_empty(map_load::AGENTS_LOADED_412, &mut input);
        for split in 0..=input.len() {
            let mut phase = populated_phase();
            let mut replies = Vec::new();
            for part in [&input[..split], &input[split..]] {
                for seen in phase.push(part) {
                    if let Some(reply) = phase.respond(&seen).unwrap() {
                        replies.push((seen, reply));
                    }
                }
            }
            assert!(phase.enabled(), "split {split}");
            assert_eq!(replies.len(), 2, "population once, then 1036 + 431 once (split {split})");
            assert_eq!(replies[0].0, Seen::Map(ClientMapEvent::MapLoaded), "split {split}");
            assert_eq!(replies[1].0, Seen::Map(ClientMapEvent::AgentsLoaded), "split {split}");
            let mut begin = BytesMut::new();
            map_load::encode_empty(map_load::BEGIN_PLAYING_1036, &mut begin);
            PlayerStartupFrame209026.encode_frame(&mut begin);
            assert_eq!(replies[1].1, begin.freeze(), "split {split}");
        }
    }

    #[test]
    fn unverified_optional_field_disables_explicitly() {
        // Client 4 carries a kind-15 (Optional) field whose encoding is unverified.
        let mut phase = populated_phase();
        let mut input = BytesMut::new();
        4u16.encode(&mut input);
        input.extend_from_slice(&[0; 16]);
        assert!(phase.push(&input).is_empty());
        assert!(!phase.enabled());
    }

    #[test]
    fn population_follows_map_loaded_and_begin_playing_follows_agents_loaded() {
        let mut phase = populated_phase();
        let agents = Seen::Map(ClientMapEvent::AgentsLoaded);
        let loaded = Seen::Map(ClientMapEvent::MapLoaded);
        // 412 before population: never 1036.
        assert!(phase.respond(&agents).unwrap().is_none());
        assert!(phase.respond(&Seen::Other(2)).unwrap().is_none());
        let mut opening = &phase.opening()[..];
        assert_eq!(MapData1063::decode_frame(&mut opening).unwrap().player_id, 1);
        let sent = phase.respond(&loaded).unwrap().unwrap();
        let plan = phase.population.as_ref().unwrap().plan.clone();
        assert_eq!(sent, population::population_stream(&plan).unwrap());
        assert!(phase.respond(&loaded).unwrap().is_none(), "population is sent once");
        assert_eq!(&phase.respond(&agents).unwrap().unwrap()[..], &[0x0c, 0x04, 0xaf, 0x01]);
        assert!(phase.respond(&agents).unwrap().is_none(), "1036 is sent once");
    }

    #[test]
    fn without_population_the_phase_never_replies() {
        let mut phase = MissionPhase::new(15).unwrap();
        for seen in [
            Seen::Map(ClientMapEvent::MapLoaded),
            Seen::Map(ClientMapEvent::AgentsLoaded),
        ] {
            assert!(phase.respond(&seen).unwrap().is_none());
        }
    }

    #[test]
    fn population_fixture_checks_profession_map_key_and_spawn() {
        assert!(PopulationFixture::new(&guardian_record(15, 2), [0.0; 3]).is_err());
        assert!(PopulationFixture::new(&guardian_record(15, 1), [f32::NAN, 0.0, 0.0]).is_err());
        assert!(PopulationFixture::new(&guardian_record(15, 1)[..40], [0.0; 3]).is_err());
        let other_map = PopulationFixture::new(&guardian_record(16, 1), [0.0; 3]).unwrap();
        assert!(MissionPhase::new(15).unwrap().with_population(other_map).is_err());
    }

    #[test]
    fn unframeable_input_disables_the_phase() {
        let mut phase = MissionPhase::new(15).unwrap();
        assert!(phase.push(&[0xff, 0xff, 0]).is_empty());
        assert!(!phase.enabled());
        assert!(phase.push(&client_input()).is_empty());
    }
}
