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
//! An explicit quest fixture adds 527/532/528 once after that startup sequence.
//! Goal selection is intentionally not repeated: selecting an existing active
//! goal can complete it (docs/re/209026-quest-lifecycle.md).
use bytes::{Bytes, BytesMut};
use gw2_proto::{
    Guid,
    agent_control::AgentControl209026,
    character_summary::SummaryV11,
    dynamic::{self, CStringEncoding, CountPrefix, Rules, StringEncoding},
    map_load::{self, ClientMapEvent, LoadMap1060, MapData1063},
    player_startup::PlayerStartupFrame209026,
    population::{self, PopulationPlan, PopulationProfession209026},
    quest::{QuestLifecycleFrame209026, QuestLifecycleOperation209026},
    schema::{self, Message},
};
use std::io::{self, ErrorKind};
use std::time::{Duration, Instant};

const MISSION_SCHEMA: &str = include_str!("../../../../../data/protocol/209026/mission.xml");
/// Group 0 (AgMsg.cpp) client messages: the agent traffic the client sends once
/// in the world, e.g. client 17 (delay report). Extracted by
/// ~/re/gw2/notes/extract-agent-client.py; kind 15 is Optional (a presence
/// byte, then the nested fields), verified from the client's field codec.
const AGENT_CLIENT_SCHEMA: &str = include_str!("../../../../../data/protocol/209026/agent-client.xml");
/// The auth connection's client table (it precedes the auth server table 0x141923fe0).
const AUTH_CLIENT_TABLE: &str = "ctable-0x141923f00";
/// The game connection's main client table (it precedes server table 0x14216e940).
const GAME_CLIENT_TABLE: &str = "ctable-0x14216e260";
/// Every client registration table of the frozen client (one group per table;
/// ~/re/gw2/notes/extract-client-tables.py). The owner's first map load sent client 278,
/// which no loaded group framed, so the phase disabled and the population was never sent.
const CLIENT_TABLES_SCHEMA: &str = include_str!("../../../../../data/protocol/209026/client-tables.xml");
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
    quest: Option<QuestFixture>,
    equipment: Option<super::equipment::EquipmentFixture>,
    skills: Option<super::skills::SkillFixture>,
}

#[derive(Clone, Copy)]
struct QuestFixture {
    quest_id: u32,
    flags: u8,
    goal_id: u32,
}

impl PopulationFixture {
    /// `spawn` is an operator-chosen packet-space position; valid spawn
    /// coordinates are not established (docs/re/209026-map-coordinate-spaces.md).
    pub fn new(record: &[u8], spawn: [f32; 3]) -> io::Result<Self> {
        Self::with_profession(record, spawn, PopulationProfession209026::Guardian, [24, 100, 103, 30])
    }

    /// Explicit research fixture. The supplied state must match the summary;
    /// neither raw Mesmer values nor selected reward tracks have established
    /// retail startup defaults. See docs/re/209026-mesmer-profession.md.
    pub fn with_profession(
        record: &[u8],
        spawn: [f32; 3],
        profession: PopulationProfession209026,
        reward_track_ids: [u32; 4],
    ) -> io::Result<Self> {
        let invalid = |message: &str| io::Error::new(ErrorKind::InvalidInput, message.to_owned());
        let (name, summary) = super::auth::character_parts(record)?;
        let summary = SummaryV11::parse(&summary)?;
        if summary.profession() != profession.profession_id() {
            return Err(invalid("population profession state must match character summary"));
        }
        let plan = PopulationPlan {
            profession,
            reward_track_ids,
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
            quest: None,
            equipment: None,
            skills: None,
        })
    }

    /// Explicit research input, not a retail quest selector. The caller must
    /// establish that the definition contains the goal. Bit 0 means complete,
    /// which the client's goal setter forbids; other flag semantics are open.
    pub fn with_quest(mut self, quest_id: u32, flags: u8, goal_id: u32) -> io::Result<Self> {
        if flags & 1 != 0 {
            return Err(io::Error::new(ErrorKind::InvalidInput, "initial quest goal requires completion flag bit 0 clear"));
        }
        self.quest = Some(QuestFixture { quest_id, flags, goal_id });
        Ok(self)
    }

    /// Explicit item frames with checked creation/reference ordering. Content
    /// definition validity and retail selection remain the caller's responsibility.
    pub fn with_equipment(mut self, frames: &[u8]) -> io::Result<Self> {
        self.equipment = Some(super::equipment::EquipmentFixture::parse(frames, self.plan.character_index)?);
        Ok(self)
    }

    /// Explicit availability grants followed by standard palette selections.
    /// Content existence and eligibility are not checked by the wire parser.
    pub fn with_skills(mut self, frames: &[u8]) -> io::Result<Self> {
        self.skills = Some(super::skills::SkillFixture::parse(frames, self.plan.player_index)?);
        Ok(self)
    }

    fn population_stream(&self) -> io::Result<Bytes> {
        let Some(equipment) = &self.equipment else {
            return Ok(population::population_stream(&self.plan)?.freeze());
        };
        let mut out = BytesMut::new();
        let mut inserted = false;
        for (id, frame) in population::population_frames(&self.plan)? {
            if id == 170 {
                if inserted {
                    return Err(io::Error::new(ErrorKind::InvalidData, "duplicate character finalization"));
                }
                // Item state must exist for the initial model build at170.
                // All character/inventory/profession/wardrobe prerequisites
                // already precede this point. Setters have no unconditional
                // finalized-view/skillbar dependency; see the visibility trace.
                out.extend_from_slice(equipment.bytes());
                inserted = true;
            }
            out.extend_from_slice(&frame);
        }
        if !inserted {
            return Err(io::Error::new(ErrorKind::InvalidData, "equipment requires a character finalization point"));
        }
        Ok(out.freeze())
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
    /// Diagnostic hold between client 420 and the population (0: send at once).
    population_delay: Duration,
    /// The population sequence and when it is due, while held.
    held: Option<(Instant, Bytes)>,
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
        // Then every other client table: add ids not yet framed. The auth connection's own
        // table is skipped, and the game's main table (next to its 624-entry server table
        // 0x14216e940) goes first, so it wins the ids it shares with smaller tables. An id
        // that the remaining tables define with different layouts is left out (framing it
        // wrongly is worse).
        let tables = schema::parse(CLIENT_TABLES_SCHEMA).map_err(invalid)?;
        let mut groups: Vec<_> = tables
            .protocols
            .into_iter()
            .flat_map(|p| p.groups)
            .filter(|g| g.name != AUTH_CLIENT_TABLE)
            .collect();
        groups.sort_by_key(|g| g.name != GAME_CLIENT_TABLE);
        let mut extra: Vec<Message> = Vec::new();
        let mut ambiguous: Vec<u16> = Vec::new();
        let before = client.len();
        for group in groups {
            let authoritative = group.name == GAME_CLIENT_TABLE;
            for message in group.client {
                if client.iter().any(|m| m.id == message.id) || ambiguous.contains(&message.id) {
                    continue;
                }
                if authoritative {
                    client.push(message);
                    continue;
                }
                if let Some(seen) = extra.iter().position(|m| m.id == message.id) {
                    if extra[seen].fields != message.fields {
                        extra.remove(seen);
                        ambiguous.push(message.id);
                    }
                    continue;
                }
                extra.push(message);
            }
        }
        eprintln!(
            "Mission phase: {} more client ids from the client tables; ambiguous ids left unframed: {:?}",
            client.len() - before + extra.len(),
            ambiguous
        );
        client.extend(extra);
        Ok(Self {
            map_key,
            client,
            pending: BytesMut::new(),
            enabled: true,
            population: None,
            populated: false,
            began: false,
            population_delay: Duration::ZERO,
            held: None,
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

    /// Hold the population for `delay` after MapLoaded instead of sending it in the
    /// same write. Diagnostic: the client streams the map's textures from 420 on, so
    /// a crash in that second can't tell map assets from population data apart
    /// (owner run 2026-11-04 03:23Z, GrTex2d/ImgMip null write). AGENT_STREAM waits
    /// for the controlled character without a timeout, so a short hold is safe.
    pub fn with_population_delay(mut self, delay: Duration) -> Self {
        self.population_delay = delay;
        self
    }

    /// When the held population is due, if one is held.
    pub fn due_at(&self) -> Option<Instant> {
        self.held.as_ref().map(|(due, _)| *due)
    }

    /// The held population once `now` has reached its due time (once).
    pub fn take_due(&mut self, now: Instant) -> Option<Bytes> {
        if self.due_at().is_some_and(|due| now >= due) {
            eprintln!("Mission phase: sending the held population sequence");
            return self.held.take().map(|(_, bytes)| bytes);
        }
        None
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
                let stream = fixture.population_stream()?;
                if !self.population_delay.is_zero() {
                    eprintln!(
                        "Mission phase: holding the population sequence for {} ms",
                        self.population_delay.as_millis()
                    );
                    self.held = Some((Instant::now() + self.population_delay, stream));
                    return Ok(None);
                }
                eprintln!("Mission phase: sending candidate population sequence");
                Ok(Some(stream))
            }
            Seen::Map(ClientMapEvent::AgentsLoaded) if self.populated && self.held.is_none() && !self.began => {
                self.began = true;
                eprintln!("Mission phase: sending server 1036 BeginPlaying, 431 player startup and 83 client control");
                let mut out = BytesMut::new();
                map_load::encode_empty(map_load::BEGIN_PLAYING_1036, &mut out);
                PlayerStartupFrame209026.encode_frame(&mut out);
                // 83 grants client control of the player's agent (gw2_proto::agent_control):
                // without it every local movement command is destroyed. After 412 the
                // scheduled world entry (46) has run, so the flag can't make it drop 46.
                AgentControl209026 { agent_index: fixture.plan.character_index as u16, value: 1 }
                    .encode_frame(&mut out);
                if let Some(skills) = &fixture.skills {
                    // 612 immediately rebuilds and needs the owned character's
                    // profession and skill bar. 605 grants precede selections.
                    // See 209026-player-skill-palettes.md and skill-availability.md.
                    out.extend_from_slice(skills.bytes());
                    eprintln!("Mission phase: sending explicit initial skills once");
                }
                if let Some(QuestFixture { quest_id, flags, goal_id }) = fixture.quest {
                    // All character/player components now exist. 532 is not
                    // idempotent: it completes an old active goal. Send exactly
                    // once after a fresh 527, then notify with 528. See the
                    // quest-lifecycle source note; this remains opt-in.
                    for operation in [
                        QuestLifecycleOperation209026::Create { quest_data_id: quest_id, raw_byte: flags },
                        QuestLifecycleOperation209026::SelectGoal { goal_id },
                        QuestLifecycleOperation209026::Notify,
                    ] {
                        QuestLifecycleFrame209026 { player_index: fixture.plan.player_index, operation }
                            .encode_frame(&mut out);
                    }
                    eprintln!("Mission phase: sending explicit quest Create, SelectGoal and Notify once");
                }
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

    /// A fixture value for one schema field; `present` selects every Optional. Covers
    /// every field kind in the client schemas the phases load.
    pub(crate) fn fixture_value(f: &schema::Field, present: bool) -> dynamic::Value {
        use dynamic::Value as V;
        let element = |f: &schema::Field| match &f.type_name {
            Some(t) => fixture_value(&schema::Field { kind: t.clone(), name: String::new(), size: None, type_name: None, fields: vec![] }, present),
            None => V::Struct(f.fields.iter().map(|g| (g.name.clone(), fixture_value(g, present))).collect()),
        };
        match f.kind.as_str() {
            "Byte" => V::U8(0),
            "Word" => V::U16(250),
            "Dword" => V::U32(0x1234),
            "Qword" => V::U64(1),
            "Float" => V::F32(1.5),
            "Float3" => V::Floats(vec![1.0, 2.0, 7.4]),
            "Float4" => V::Floats(vec![0.0, 0.0, 0.0, 1.0]),
            "Point3" => V::Point3(gw2_proto::Point3([1.0, 2.0, 7.4], 0)),
            "Guid" => V::Guid(Guid([0x5a; 16])),
            // Short enough for the smallest Size (terminator included).
            "String" => V::Text("Fixture"[..f.size.map_or(7, |n| (n as usize).saturating_sub(1).min(7))].into()),
            "CString" => V::Bytes(b"Fixture"[..f.size.map_or(7, |n| (n as usize).saturating_sub(1).min(7))].to_vec()),
            "Float2" => V::Floats(vec![1.0, 2.0]),
            "Address" => V::Address("127.0.0.1:6112".parse().unwrap()),
            "BufferFixed" => V::Bytes(vec![0; f.size.unwrap_or(0) as usize]),
            "BufferVarSmall" | "BufferVarLarge" => V::Bytes(vec![1, 2, 3]),
            "ArrayFixed" => V::Array((0..f.size.unwrap_or(0)).map(|_| element(f)).collect()),
            "ArrayVarSmall" | "ArrayVarLarge" => V::Array(vec![element(f)]),
            "Optional" if present => V::Optional(Some(Box::new(element(f)))),
            "Optional" => V::Optional(None),
            other => panic!("test fixture lacks kind {other}"),
        }
    }

    /// Encode one client message from a schema list with fixture values.
    pub(crate) fn schema_frame(messages: &[Message], id: u16, present: bool) -> BytesMut {
        let message = messages.iter().find(|m| m.id == id).unwrap();
        let values = message.fields.iter().map(|f| (f.name.clone(), fixture_value(f, present))).collect();
        let mut out = BytesMut::new();
        id.encode(&mut out);
        dynamic::encode_message(message, &rules(), &dynamic::Value::Struct(values), &mut out).unwrap();
        out
    }

    /// Every client message the mission phase loads (groups 39, 40 and agent group 0), with
    /// Optionals absent and present, is dispatched under its own id and keeps the phase.
    #[test]
    fn every_loaded_client_message_is_dispatched_under_its_own_id() {
        let phase = MissionPhase::new(15).unwrap();
        let ids: Vec<u16> = phase.client.iter().map(|m| m.id).collect();
        assert!(ids.len() > 66 && ids.contains(&278), "all client tables are loaded ({})", ids.len());
        for present in [false, true] {
            for &id in &ids {
                let frame = schema_frame(&phase.client, id, present);
                let mut live = populated_phase();
                let seen = live.push(&frame);
                let got: Vec<u16> = seen
                    .iter()
                    .map(|s| match s {
                        Seen::Other(i) => *i,
                        Seen::Map(ClientMapEvent::LanguagePrefs(_)) => 423,
                        Seen::Map(ClientMapEvent::MapLoaded) => 420,
                        Seen::Map(ClientMapEvent::FailedToLoad) => 418,
                        Seen::Map(ClientMapEvent::AgentsLoaded) => 412,
                    })
                    .collect();
                assert_eq!(got, vec![id], "client {id} (optionals present: {present})");
                assert!(live.enabled(), "client {id} disabled the phase");
            }
        }
    }

    /// One group-0 client frame built from the extracted schema.
    fn agent_frame_with(id: u16, present: bool) -> BytesMut {
        let parsed = schema::parse(AGENT_CLIENT_SCHEMA).unwrap();
        let message = parsed.protocols[0].groups[0]
            .client
            .iter()
            .find(|m| m.id == id)
            .unwrap()
            .clone();
        let values = message.fields.iter().map(|f| (f.name.clone(), fixture_value(f, present))).collect();
        let mut out = BytesMut::new();
        id.encode(&mut out);
        dynamic::encode_message(&message, &rules(), &dynamic::Value::Struct(values), &mut out).unwrap();
        out
    }

    fn agent_frame(id: u16) -> BytesMut {
        agent_frame_with(id, false)
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
            AgentControl209026 { agent_index: 1, value: 1 }.encode_frame(&mut begin);
            assert_eq!(replies[1].1, begin.freeze(), "split {split}");
        }
    }

    /// The ids the auth table and the game's main table both define (30, 34, 36-40) and 89
    /// (also in 0x14215dce0) take the game table's layout, and nothing is left unframed.
    #[test]
    fn ids_shared_with_other_tables_take_the_game_tables_layout() {
        let phase = MissionPhase::new(15).unwrap();
        let tables = schema::parse(CLIENT_TABLES_SCHEMA).unwrap();
        let game = tables.protocols.iter().flat_map(|p| &p.groups).find(|g| g.name == GAME_CLIENT_TABLE).unwrap();
        for id in [30, 34, 36, 37, 38, 39, 40, 89] {
            let loaded = phase.client.iter().find(|m| m.id == id).unwrap_or_else(|| panic!("client {id} unframed"));
            let own = game.client.iter().find(|m| m.id == id).unwrap();
            assert_eq!(loaded.fields, own.fields, "client {id}");
        }
        for group in tables.protocols.iter().flat_map(|p| &p.groups).filter(|g| g.name != AUTH_CLIENT_TABLE) {
            for m in &group.client {
                assert!(phase.client.iter().any(|l| l.id == m.id), "client {} ({}) unframed", m.id, group.name);
            }
        }
        let mut ids: Vec<u16> = phase.client.iter().map(|m| m.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "one layout per id");
    }

    /// The owner's first map load (2026-11-03 17:07Z): client 278 arrived right after the
    /// opening, no loaded group framed it, the phase disabled and the population was never
    /// sent. It is in table 0x14223bf20 (client-tables.xml) and must now keep the phase.
    #[test]
    fn client_278_during_the_map_load_keeps_the_phase_and_the_population_follows() {
        let phase = MissionPhase::new(15).unwrap();
        let mut input = schema_frame(&phase.client, 278, false);
        map_load::encode_empty(map_load::MAP_LOADED_420, &mut input);
        map_load::encode_empty(map_load::AGENTS_LOADED_412, &mut input);
        for split in 0..=input.len() {
            let mut live = populated_phase();
            let mut seen_all = Vec::new();
            let mut replies = 0;
            for part in [&input[..split], &input[split..]] {
                for seen in live.push(part) {
                    replies += live.respond(&seen).unwrap().is_some() as usize;
                    seen_all.push(seen);
                }
            }
            assert!(live.enabled(), "split {split}");
            assert_eq!(seen_all[0], Seen::Other(278), "split {split}");
            assert_eq!(replies, 2, "population, then 1036 + 431 (split {split})");
        }
    }

    /// Kind 15 is Optional (docs/re/209026-agent-client-messages.md): one presence byte,
    /// then the nested fields only when present. Client 4 nests two more Optionals.
    /// Every server frame the deployed mission phase emits (opening 1063/1060, the
    /// population sequence, 1036 + 431) must be a message the client registers a handler
    /// for and must decode to its exact length under the client's own descriptors (groups
    /// 0, 39 and 40 extracted from its handler tables). An unregistered id is
    /// DISCONNECT_MISSING_HANDLER on the client; a misfit frame desynchronizes the stream.
    #[test]
    fn every_emitted_server_frame_matches_a_client_handler_descriptor() {
        // Every server registration table the client has (one group per table).
        const SERVER_TABLES: &str = include_str!("../../../../../data/protocol/209026/server-tables.xml");
        let tables = schema::parse(SERVER_TABLES).unwrap().protocols[0].groups.clone();
        assert!(tables.len() >= 40);
        let mut phase = populated_phase();
        let streams = [
            ("opening", phase.opening()),
            ("population", phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap().unwrap()),
            ("begin", phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap().unwrap()),
        ];
        let mut census = Vec::new();
        for (name, bytes) in streams {
            let mut cursor = &bytes[..];
            while !cursor.is_empty() {
                let before = cursor.len();
                let id = u16::from_le_bytes([cursor[0], cursor[1]]);
                // Every table that registers this id; the frame must decode exactly under one,
                // and the frame boundary is taken from that decode.
                let fits: Vec<(String, Option<String>, usize)> = tables
                    .iter()
                    .filter_map(|t| {
                        let mut probe = cursor;
                        let (m, _) = dynamic::decode_frame(&t.server, &rules(), &mut probe).ok()?;
                        Some((t.name.clone(), m.addr.clone(), before - probe.len()))
                    })
                    .collect();
                assert!(!fits.is_empty(), "{name}: server {id} at +{} decodes under no client table", bytes.len() - before);
                // The game connection's own tables, in the order prior RE attributes them: the
                // 624-entry group-0 table, AgMsg.cpp's server table, mission groups 39 and 40.
                // Ids also present in another table (other protocols) are only reported.
                const GAME: [&str; 4] = ["table-0x14216e940", "table-0x14211be70", "table-0x1422ddbf0", "table-0x141923b80"];
                let (table, addr, len) = GAME
                    .iter()
                    .find_map(|g| fits.iter().find(|f| f.0 == *g).cloned())
                    .unwrap_or_else(|| panic!("{name}: server {id} is registered only outside the game tables: {fits:?}"));
                let others: Vec<String> = fits.iter().filter(|f| f.0 != table).map(|f| format!("{}@{}", f.0, f.2)).collect();
                census.push((name, id, addr, len, table, others));
                cursor = &cursor[len..];
            }
        }
        for (name, id, addr, len, table, others) in &census {
            eprintln!("audit {name}: server {id} ({len} bytes) {table} handler {} also {others:?}", addr.as_deref().unwrap_or("?"));
        }
        assert!(census.iter().any(|c| c.1 == 1063) && census.iter().any(|c| c.1 == 1036) && census.iter().any(|c| c.1 == 431));
    }

    /// Deterministic xorshift bytes (no extra dependency).
    fn noise(seed: u64, len: usize) -> Vec<u8> {
        let mut x = seed | 1;
        (0..len)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x as u8
            })
            .collect()
    }

    /// Malformed client input never panics the mission phase: every truncated prefix of
    /// valid traffic is retained (not misparsed), junk after valid frames disables the phase
    /// cleanly, and 500 seeded random inputs (including split delivery) neither panic nor
    /// make the phase reply more than once per event.
    #[test]
    fn malformed_client_input_never_panics_and_disables_cleanly() {
        let mut valid = BytesMut::new();
        map_load::encode_empty(map_load::MAP_LOADED_420, &mut valid);
        valid.extend_from_slice(&agent_frame_with(4, true));
        valid.extend_from_slice(&agent_frame(17));
        map_load::encode_empty(map_load::AGENTS_LOADED_412, &mut valid);
        // Truncated prefixes: never more events than complete frames, phase still enabled.
        for cut in 0..valid.len() {
            let mut phase = populated_phase();
            let seen = phase.push(&valid[..cut]);
            assert!(seen.len() <= 4 && phase.enabled(), "prefix {cut}");
        }
        // A junk tail after valid frames: earlier events still delivered, then disabled.
        let mut tail = valid.clone();
        tail.extend_from_slice(&[0xff, 0xff, 0x01]); // id 65535: no descriptor
        let mut phase = populated_phase();
        assert_eq!(phase.push(&tail).len(), 4);
        assert!(!phase.enabled());
        assert!(phase.push(&valid).is_empty(), "a disabled phase stays silent");
        // Seeded random input, whole and split: no panic; at most one population and one begin.
        for seed in 0..500u64 {
            let input = noise(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15), (seed as usize % 300) + 1);
            let mut phase = populated_phase();
            let split = input.len() / 2;
            let mut replies = 0;
            for part in [&input[..split], &input[split..]] {
                for seen in phase.push(part) {
                    if phase.respond(&seen).unwrap().is_some() {
                        replies += 1;
                    }
                }
            }
            assert!(replies <= 2, "seed {seed}");
        }
    }

    #[test]
    fn optional_fields_frame_absent_and_present_for_every_split() {
        let absent = agent_frame_with(4, false);
        let present = agent_frame_with(4, true);
        // The present form adds the nested fields: three Float4 (16 bytes each), a Dword,
        // two Bytes and the two inner presence bytes.
        assert!(present.len() > absent.len() + 3 * 16);
        let mut input = absent.clone();
        input.extend_from_slice(&present);
        input.extend_from_slice(&agent_frame(17));
        for split in 0..=input.len() {
            let mut phase = populated_phase();
            let mut seen = phase.push(&input[..split]);
            seen.extend(phase.push(&input[split..]));
            assert_eq!(seen, vec![Seen::Other(4), Seen::Other(4), Seen::Other(17)], "split {split}");
            assert!(phase.enabled(), "split {split}");
        }
        // A presence byte the client never writes (its encoder uses setne: 0 or 1) is rejected.
        let mut bad = absent.clone();
        let last = bad.len() - 1;
        bad[last] = 2;
        let mut phase = populated_phase();
        assert!(phase.push(&bad).is_empty());
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
        assert_eq!(&phase.respond(&agents).unwrap().unwrap()[..], &[0x0c, 0x04, 0xaf, 0x01, 0x53, 0x00, 0x01, 0x00, 0x01], "1036, 431, then 83 client control for agent 1");
        assert!(phase.respond(&agents).unwrap().is_none(), "1036 is sent once");
    }

    #[test]
    fn explicit_quest_starts_once_after_population_and_agent_readiness() {
        let fixture = PopulationFixture::new(&guardian_record(15, 1), [0.0; 3]).unwrap();
        assert!(fixture.clone().with_quest(154, 1, 1).is_err());
        assert!(fixture.clone().with_quest(154, 255, 1).is_err());
        // Distinct synthetic IDs/flags check transport; no content selection.
        let fixture = fixture.with_quest(16384, 2, 300).unwrap();
        let mut phase = MissionPhase::new(15).unwrap().with_population(fixture).unwrap()
            .with_population_delay(Duration::from_secs(1));
        let agents = Seen::Map(ClientMapEvent::AgentsLoaded);
        let loaded = Seen::Map(ClientMapEvent::MapLoaded);
        assert!(phase.respond(&agents).unwrap().is_none());
        assert!(phase.respond(&loaded).unwrap().is_none());
        assert!(phase.respond(&agents).unwrap().is_none());
        assert!(phase.take_due(phase.due_at().unwrap()).is_some());
        let reply = phase.respond(&agents).unwrap().unwrap();
        assert_eq!(&reply[..9], &[0x0c, 0x04, 0xaf, 0x01, 0x53, 0x00, 0x01, 0x00, 0x01]);
        let mut tail = &reply[9..];
        for operation in [
            QuestLifecycleOperation209026::Create { quest_data_id: 16384, raw_byte: 2 },
            QuestLifecycleOperation209026::SelectGoal { goal_id: 300 },
            QuestLifecycleOperation209026::Notify,
        ] {
            assert_eq!(QuestLifecycleFrame209026::decode_frame(&mut tail).unwrap(),
                QuestLifecycleFrame209026 { player_index: 1, operation });
        }
        assert!(tail.is_empty());
        for seen in [loaded, agents, agents] {
            assert!(phase.respond(&seen).unwrap().is_none(), "never reselect an active goal");
        }
    }

    #[test]
    fn explicit_equipment_precedes_finalization_and_skills_and_quest_wait_for_readiness() {
        let equipment = super::super::equipment::fixture_for_test();
        let skills = super::super::skills::fixture_for_test();
        let fixture = PopulationFixture::new(&guardian_record(15, 1), [0.0; 3]).unwrap()
            .with_equipment(&equipment).unwrap().with_skills(&skills).unwrap().with_quest(154, 0, 1).unwrap();
        let mut phase = MissionPhase::new(15).unwrap().with_population(fixture).unwrap();
        let loaded = Seen::Map(ClientMapEvent::MapLoaded);
        let agents = Seen::Map(ClientMapEvent::AgentsLoaded);
        assert!(phase.respond(&agents).unwrap().is_none());
        let population = phase.respond(&loaded).unwrap().unwrap();
        let baseline_frames = population::population_frames(&phase.population.as_ref().unwrap().plan).unwrap();
        let before_finalization: usize = baseline_frames.iter().take_while(|(id, _)| *id != 170).map(|(_, bytes)| bytes.len()).sum();
        let baseline = population::population_stream(&phase.population.as_ref().unwrap().plan).unwrap();
        assert_eq!(&population[..before_finalization], &baseline[..before_finalization]);
        assert_eq!(&population[before_finalization..before_finalization + equipment.len()], equipment);
        assert_eq!(&population[before_finalization + equipment.len()..], &baseline[before_finalization..]);
        assert_eq!(&population[before_finalization + equipment.len()..before_finalization + equipment.len() + 3], &[0xaa, 0, 1], "170 finalizes character1 after equipment");
        let reply = phase.respond(&agents).unwrap().unwrap();
        assert_eq!(&reply[..9], &[0x0c, 0x04, 0xaf, 0x01, 0x53, 0x00, 0x01, 0x00, 0x01]);
        assert_eq!(&reply[9..9 + skills.len()], skills, "612 requires the already populated owned skill bar");
        let mut quest = &reply[9 + skills.len()..];
        assert!(matches!(QuestLifecycleFrame209026::decode_frame(&mut quest).unwrap().operation,
            QuestLifecycleOperation209026::Create { quest_data_id: 154, raw_byte: 0 }));
        assert!(phase.respond(&loaded).unwrap().is_none());
        assert!(phase.respond(&agents).unwrap().is_none(), "never recreate items or repeat initial selections");
    }

    /// With a delay the population is held after 420 and released once due (once);
    /// 412 during the hold never produces 1036 early.
    #[test]
    fn held_population_goes_out_once_due_and_1036_waits_for_it() {
        let mut phase = populated_phase().with_population_delay(Duration::from_secs(15));
        let agents = Seen::Map(ClientMapEvent::AgentsLoaded);
        let loaded = Seen::Map(ClientMapEvent::MapLoaded);
        assert!(phase.due_at().is_none());
        assert!(phase.respond(&loaded).unwrap().is_none(), "held, not sent with 420");
        let due = phase.due_at().expect("held");
        assert!(phase.take_due(due - Duration::from_millis(1)).is_none());
        assert!(phase.respond(&agents).unwrap().is_none(), "no 1036 before the population");
        let plan = phase.population.as_ref().unwrap().plan.clone();
        assert_eq!(phase.take_due(due).unwrap(), population::population_stream(&plan).unwrap());
        assert!(phase.take_due(due).is_none() && phase.due_at().is_none(), "released once");
        assert!(phase.respond(&loaded).unwrap().is_none());
        assert_eq!(&phase.respond(&agents).unwrap().unwrap()[..], &[0x0c, 0x04, 0xaf, 0x01, 0x53, 0x00, 0x01, 0x00, 0x01], "1036, 431, then 83 client control for agent 1");
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
    fn mesmer_fixture_requires_matching_summary_and_explicit_state() {
        let state = PopulationProfession209026::Mesmer { values: [3, 0] };
        // Synthetic distinct IDs test transport, not retail track selection.
        let tracks = [1, 2, 3, 4];
        assert!(PopulationFixture::new(&guardian_record(28, 7), [0.0; 3]).is_err());
        assert!(PopulationFixture::with_profession(&guardian_record(28, 1), [0.0; 3], state, tracks).is_err());
        for (map, race) in [(379, 3), (28, 3), (138, 2), (15, 2)] {
            let mut record = guardian_record(map, 7);
            let summary_start = record.len() - 159;
            record[summary_start + 0x21] = race;
            let fixture = PopulationFixture::with_profession(&record, [0.0; 3], state, tracks).unwrap();
            assert_eq!(fixture.map_key(), map);
            assert_eq!(fixture.plan.appearance[27], race);
            assert_eq!(fixture.plan.reward_track_ids, tracks);
            let frames = population::population_frames(&fixture.plan).unwrap();
            assert!(!frames.iter().any(|(id, _)| *id == 477));
            assert_eq!(frames.iter().filter(|(id, _)| *id == 481).count(), 1);
            let mut phase = MissionPhase::new(u32::from(map)).unwrap().with_population(fixture).unwrap();
            assert!(phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap().is_none());
            assert!(phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap().is_some());
            assert!(phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap().is_some());
            assert!(phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap().is_none());
        }
    }

    #[test]
    fn unframeable_input_disables_the_phase() {
        let mut phase = MissionPhase::new(15).unwrap();
        assert!(phase.push(&[0xff, 0xff, 0]).is_empty());
        assert!(!phase.enabled());
        assert!(phase.push(&client_input()).is_empty());
    }
}
