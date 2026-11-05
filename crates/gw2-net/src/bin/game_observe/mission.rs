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
    agent_behavior::{AgentBehaviorFrame209026, HealthTransitionDetails209026, HealthTransitionPayload209026},
    agent_control::AgentControl209026,
    bags::{
        BagSlotFrame209026, InventoryItemFrame209026, InventorySlotCountFrame209026, STARTER_BACKPACK_ITEM,
        encode_minimal_item_creation, encode_stack_item_creation,
    },
    character_summary::SummaryV11,
    dynamic::{self, CStringEncoding, CountPrefix, Rules, StringEncoding},
    experience::{EarnedLevelFrame209026, ExperienceFrame209026, level_for_experience},
    renown::RenownFrame209026,
    health::{HealthFrame209026, HealthOperation209026, HealthValues209026, SourcedHealthFrame209026},
    map_load::{self, ClientMapEvent, LoadMap1060, MapData1063},
    npc_talk::{ConversationReply209026, InteractRequest209026, LEVEL1_NPC_HEALTH, TalkNpcPlan209026, TalkSession209026},
    skill_order::{AgentBehaviorOrderFrame209026, SkillBeginRequest209026, SkillOrder209026},
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

pub(super) fn rules() -> Rules {
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
    /// Client 23 carrying the packed interact request (opcode 0x14/0x15).
    Interact(InteractRequest209026),
    /// Client 275/276 conversation replies.
    Conversation(ConversationReply209026),
    /// Client 23 carrying SkillBegin 0x23, with its outer behavior header.
    Skill(SkillSeen),
    /// Client 23 2-byte `[0x29][skill-bar slot]` (inferred SkillRepeatStart; key 1
    /// sends slot 5, the weapon's first skill), with the header to echo.
    RepeatStart(SkillSeen),
    /// Client 227 `{Dword agent}` (sender `0x1412646b0`, CharForce.cpp): the
    /// selected target's agent id (object +0xc, the same field SkillBegin's
    /// target uses), 0 when none. Run 168: sent after Tab.
    Target(u32),
    /// Client 18/19 (AgMsg) `{Dword, Dword, Float3 position, ...}`: the
    /// player's movement report. The wire position is collision space
    /// `[X/32, Y/32, -Z/32]` (run 186: spawn 0,-26300,-3800 reported as
    /// 0,-822,118); this holds it converted back to whole map units.
    Moved { id: u16, position: [i32; 3] },
    Other(u16),
}

/// A SkillBegin request and the client-23 header fields an answer echoes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillSeen {
    pub agent_index: u16,
    pub remote_tick: u32,
    pub command_flags: u8,
    pub behavior_value: u32,
    pub request: SkillBeginRequest209026,
}

/// Opt-in talkable test NPC (docs/re/209026-npc-talk-loop.md). Spawned once
/// with the player's BeginPlaying batch; answers interact with test dialogue.
#[derive(Clone)]
pub struct TalkNpc {
    plan: TalkNpcPlan209026,
    session: TalkSession209026,
}

/// Heart 56's NPC and lines, quoted from the client's own English text table
/// (language-0 plain strings; ids in comments). Retail dialogue is server
/// data; these are the client strings that belong to this heart.
const HEART_56_SPEAKER: &str = "Shaman Sigrytha"; // text 30730
const HEART_56_ACTIVE: [&str; 2] = [
    "Honor the Spirit of Raven.", // text 45566 (the heart's title)
    "Raven, clever and wise, guides us with truths that others fear. He bestows a far-seeing clarity of mind. In my vision, he spoke of riddles and opened my eyes to secrets that few others can see.", // text 27970
];
const HEART_56_DONE: [&str; 2] = [
    "Shaman Sigrytha thanks you.",           // text 84427
    "You have honored the spirit of Raven.", // text 84429
];

impl TalkNpc {
    /// Replace the conversation (speaker and pages) unless one is open.
    fn set_dialogue(&mut self, speaker: &str, pages: &[&str]) -> io::Result<()> {
        if self.session.active().is_some() {
            return Ok(());
        }
        let pages = pages.iter().map(|p| (*p).to_owned()).collect();
        self.session = TalkSession209026::new(u32::from(self.plan.agent_index), speaker.to_owned(), pages)?;
        Ok(())
    }

    /// Agent index of the test NPC. The player's placeholder character index
    /// is 1; `respond` refuses to spawn on a collision.
    pub const AGENT_INDEX: u16 = 2;

    /// Placeholder pages, marked as test text: retail dialogue is server data.
    pub fn new(species_key: u32, position: [f32; 3], speaker: String) -> io::Result<Self> {
        let plan = TalkNpcPlan209026 {
            agent_index: Self::AGENT_INDEX,
            species_key,
            position,
            facing: [1.0, 0.0, 0.0], // placeholder: unit vector, direction unestablished
            attitude: 0,
            talkable: true,
        };
        plan.spawn_frames()?;
        let pages = vec![
            "[test] Well met, traveler.".to_owned(),
            "[test] This is placeholder dialogue from the private test server.".to_owned(),
        ];
        let session = TalkSession209026::new(u32::from(Self::AGENT_INDEX), speaker, pages)?;
        Ok(Self { plan, session })
    }
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
            initial_health: 1645.0, // Legacy research value; explicit override below.
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

    /// Full health for an explicitly chosen profile; no additional modifiers
    /// are inferred. Rate, second meter and F5 remain separate fixture values.
    pub fn with_initial_health(mut self, value: f32) -> io::Result<Self> {
        self.plan.initial_health = value;
        self.plan.validate()?;
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

pub(super) fn client_messages() -> io::Result<Vec<Message>> {
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
    Ok(client)
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
    talk: Option<TalkNpc>,
    /// Opt-in hostile test NPCs (attitude 1, no talk flag) for combat research.
    /// Agent indices 3, 4, ... in the order added (at most MAX_HOSTILE_NPCS).
    hostiles: Vec<HostileNpc>,
    /// The hostile NPC (agent index) the player last hit with a SkillBegin.
    last_target: Option<u16>,
    /// Hostile NPCs the player hit that are still alive, oldest hit first; the
    /// last one is the aggro target that counter-attacks the player.
    aggro: Vec<u16>,
    /// Research: answer SkillBegin with server 34 carrying a 0x78 order.
    skill_order_echo: bool,
    /// Research: on each player SkillBegin, order the hostile NPC to cast this
    /// skill (server 34 + 0x78, slot 23), with its own behavior-value counter.
    npc_cast: Option<(u32, u32)>,
    /// Research: on each player SkillBegin aimed at a hostile NPC, lower its
    /// health by this amount through server 257.
    damage_on_skill: Option<f32>,
    /// Research: answer `[0x29][slot]` with server 34 + 0x78 for this skill key,
    /// aimed at the hostile NPC (auto-attack hypothesis).
    autoattack_skill: Option<u32>,
    /// Research: experience per NPC kill, running total and earned level.
    xp: Option<(u32, u32, u8)>,
    /// Research: item-creation message used to give the Starter Backpack.
    starter_bag: Option<u16>,
    /// Research (post-load spawn trace): one neutral NPC spawned in the
    /// AgentsLoaded batch, after BeginPlaying, the path whose model never
    /// streams (docs/re/209026-npc-talk-loop.md, render root cause).
    late_npc: Option<TalkNpcPlan209026>,
    /// Research: one renown heart shown at spawn whose counter advances on
    /// each NPC kill (docs/re/209026-npc-talk-loop.md, renown heart).
    heart: Option<Heart>,
    /// Research (run 178): send 169 {char, 1} on defeat so "Return to a
    /// Waypoint" sends client 210. Opt-in: with it, run 178 later asserted
    /// ChCliInventory.cpp:3079 `inventorySlot < m_inventorySlots.Count()`.
    respawn_flag: bool,
    /// Research: where a respawn places the player (server 86), e.g. a
    /// waypoint's client POI position. None revives in place.
    respawn_point: Option<[f32; 3]>,
    /// Research (run 182): answer the respawn request with 1065 to this game
    /// address so the client rejoins the map at `respawn_point`, carrying the
    /// character's progress over to the next connection (CARRY).
    rejoin_route: Option<std::net::SocketAddr>,
    /// Loot drops restored from CARRY, resent at AgentsLoaded.
    restore_loot: u32,
    /// Experience restored from CARRY, resent at AgentsLoaded (total, level).
    restore_xp: Option<(u32, u8)>,
    /// Count of client 18/19 movement reports (sparse logging).
    moves_seen: u32,
    /// Research: on each kill, the next KILL_LOOT_TABLE item goes into the next
    /// backpack slot (968 + 276). Holds the number of drops so far.
    kill_loot: Option<u32>,
    /// Research: the hostile NPC hits the player while both are alive.
    npc_attack: Option<NpcAttack>,
    /// Research: send combat health changes as 255 with this change kind
    /// (skill and attacker attached) instead of 257.
    sourced_damage: Option<u8>,
}

/// Most hostile NPCs one mission phase spawns (agent indices 3..=18).
pub const MAX_HOSTILE_NPCS: usize = 16;

/// One hostile test NPC with its own health and revival timer.
struct HostileNpc {
    plan: TalkNpcPlan209026,
    health: f32,
    /// When the killed NPC revives (RESPAWN_DELAY after death).
    respawn: Option<Instant>,
}

/// One combat health change from `previous` to `current`: 257 Currents, or
/// 255 (a delta) with kind, skill and source.
fn encode_health_change(
    out: &mut BytesMut,
    sourced: Option<u8>,
    target: u32,
    (previous, current): (f32, f32),
    skill_key: u32,
    source_agent: u32,
) -> io::Result<u16> {
    match sourced {
        Some(kind) => {
            let delta = current - previous;
            // Research (run 165): the Word carries the magnitude, hypothesised
            // to be the floater's displayed amount (with 0 no hit floater painted).
            let raw_word = delta.abs().round().min(f32::from(u16::MAX)) as u16;
            SourcedHealthFrame209026 { character_index: target, delta, second_delta: 0.0, raw_word, kind, skill_key, source_agent }
                .encode_frame(out)?;
            Ok(255)
        }
        None => {
            HealthFrame209026 { character_index: target, operation: HealthOperation209026::Currents { current, second_current: 0.0 } }
                .encode_frame(out);
            Ok(257)
        }
    }
}

/// Research NPC counter-attack: after the player's first hit, server 257 lowers
/// the player's health every NPC_ATTACK_INTERVAL until the NPC dies. Damage,
/// timing and the stop-before-death rule are placeholders, not retail values.
#[derive(Clone, Copy)]
struct NpcAttack {
    damage: f32,
    player_health: f32,
    max_health: f32,
    next: Option<Instant>,
    /// Set when the player goes down: when the defeat transition is due.
    defeat_at: Option<Instant>,
    /// The player was defeated; their attacks no longer land.
    defeated: bool,
}

/// Placeholder time between the player going down and being defeated.
const DOWNED_TO_DEFEAT: Duration = Duration::from_secs(10);

/// Replace the player's first health meter (the downed bar reads only meter 1,
/// `DwnHealth.cpp` 0x1404f4130): 259 values, then 260 rate (units/s; the
/// client applies a negative rate locally). Second meter and extra stay 0, as
/// in the player's 256 creation.
fn encode_health_pool(out: &mut BytesMut, character_index: u32, current: f32, maximum: f32, rate: f32) {
    let values = HealthValues209026 { current, maximum, second_current: 0.0, second_maximum: 0.0, extra: 0.0 };
    HealthFrame209026 { character_index, operation: HealthOperation209026::Values(values) }.encode_frame(out);
    HealthFrame209026 { character_index, operation: HealthOperation209026::Rate(rate) }.encode_frame(out);
}

/// Rally: the downed player killed the foe. Cancel defeat, send the packed
/// 0x49 alive transition and restore half health (placeholder amount) on the normal meter.
/// Returns whether the player was downed and rallied (the attack timer is then
/// stopped; the caller stops it otherwise when no hit NPC is left alive).
fn rally_on_kill(attack: &mut Option<NpcAttack>, character_index: u32, out: &mut BytesMut) -> io::Result<bool> {
    let Some(attack) = attack.as_mut() else { return Ok(false) };
    if attack.defeat_at.take().is_none() {
        return Ok(false);
    }
    attack.next = None;
    AgentBehaviorFrame209026 {
        agent_index: character_index as u16,
        remote_tick: 0,
        command_flags: 0x10,
        behavior_value: 0x4000_0500,
        behavior_data: HealthTransitionPayload209026::Alive { detail_byte: 0, trailing_byte: 0 }.encode_payload(),
    }
    .encode_frame(out)?;
    let restored = attack.max_health / 2.0;
    // Swap the downed pool back to the normal meter at half health.
    encode_health_pool(out, character_index, restored, attack.max_health, 0.0);
    attack.player_health = restored;
    eprintln!("Mission phase: player rallied (server 84 packed 0x49), health -> {restored}");
    Ok(true)
}

/// Progress carried from one game connection to the next by a waypoint
/// rejoin (research; one player per observer process).
#[derive(Clone, Copy, Debug)]
struct CarryOver {
    spawn: [f32; 3],
    xp: Option<(u32, u8)>,
    loot_drops: Option<u32>,
    /// Progress, completion, and whether the one-time reward was issued.
    heart: Option<(u32, bool, bool)>,
}

static CARRY: std::sync::Mutex<Option<CarryOver>> = std::sync::Mutex::new(None);

/// Server 169 `{Dword character, Byte}` (handler `0x14124be30` ->
/// `0x1411da200`): nonzero sets character flag bit 1 (defeated, may respawn),
/// zero clears it.
fn encode_respawn_flag(out: &mut BytesMut, character_index: u32, on: bool) {
    use gw2_proto::Encode;
    169u16.encode(out);
    character_index.encode(out);
    u8::from(on).encode(out);
}

/// Respawn the defeated player in place: packed 0x49 alive, the normal health
/// meter at full, the respawn flag cleared. Retail moves the player to the
/// chosen waypoint; that teleport is not implemented.
fn respawn_player(attack: &mut NpcAttack, character_index: u32, out: &mut BytesMut) -> io::Result<()> {
    AgentBehaviorFrame209026 {
        agent_index: character_index as u16,
        remote_tick: 0,
        command_flags: 0x10,
        behavior_value: 0x4000_0600,
        behavior_data: HealthTransitionPayload209026::Alive { detail_byte: 0, trailing_byte: 0 }.encode_payload(),
    }
    .encode_frame(out)?;
    encode_health_pool(out, character_index, attack.max_health, attack.max_health, 0.0);
    encode_respawn_flag(out, character_index, false);
    attack.player_health = attack.max_health;
    attack.defeated = false;
    attack.defeat_at = None;
    attack.next = None;
    Ok(())
}

/// A packed health transition (0x34 downed / 0x33 dead) on `agent` through
/// server 84 (docs/re/209026-health-transitions.md), `killer` as the agent.
fn encode_transition(out: &mut BytesMut, agent: u16, downed: bool, killer: u32, behavior_value: u32) -> io::Result<()> {
    let details = HealthTransitionDetails209026 { detail_word: 0, detail_byte: 0, content_key: 0, agent_id: killer, trailing_byte: 0 };
    let payload = if downed { HealthTransitionPayload209026::Downed(details) } else { HealthTransitionPayload209026::Dead(details) };
    AgentBehaviorFrame209026 { agent_index: agent, remote_tick: 0, command_flags: 0x10, behavior_value, behavior_data: payload.encode_payload() }
        .encode_frame(out)
}

const NPC_ATTACK_INTERVAL: Duration = Duration::from_secs(2);
const NPC_ATTACK_FIRST_DELAY: Duration = Duration::from_millis(1500);

/// The player's hit aggroes the NPC: schedule its first attack once. A
/// downed or defeated player (health 0) is not attacked again.
fn start_npc_attack(attack: &mut Option<NpcAttack>) {
    if let Some(attack) = attack.as_mut()
        && attack.next.is_none()
        && attack.player_health > 0.0
    {
        attack.next = Some(Instant::now() + NPC_ATTACK_FIRST_DELAY);
    }
}

/// Research renown heart: type-72 key, kills needed, kills so far, player
/// index (set when shown), and whether it completed.
#[derive(Clone, Copy)]
struct Heart {
    key: u16,
    target: u32,
    progress: u32,
    player: Option<u32>,
    done: bool,
    /// The completion reward (556 + bonus XP) has been sent.
    rewarded: bool,
    /// Whether the player is inside the heart's area (heart 56 only; other
    /// keys have no polygon here and count as always inside).
    inside: bool,
}

/// Heart completion reward. Coins and karma are placeholders (the client
/// holds no heart reward values); XP is the client's level-2 content-XP base
/// (`s_experienceForContentLut[3] - [2]` = 7354) times the 70% factor of the
/// client Reward.cpp formula, a rough client-derived scale (swarm note
/// rewards-waypoint.md section A).
const HEART_COINS: u32 = 30;
const HEART_KARMA: u32 = 50;
const HEART_XP: u32 = 7354 * 7 / 10;

/// Heart 56's area polygon (x, y map units), read from its client type-287
/// record (content 1282852 offset 4224016: +0x40 Float2 array, +0x48 count
/// 16). Centre 465.05, -26226.8, -3820.56.
const HEART_56_POLYGON: [(f32, f32); 16] = [
    (-4290.5, -26895.6), (-2586.4, -27380.6), (-1511.5, -27177.6), (-1238.0, -26547.5),
    (-1437.1, -25318.5), (-2893.7, -24242.6), (-1184.0, -23906.0), (3026.0, -26504.0),
    (2627.0, -28787.0), (1782.4, -29510.9), (19.0, -29976.0), (-1734.0, -30403.6),
    (-2612.3, -29939.8), (-3751.3, -29076.1), (-4833.2, -28165.5), (-4800.0, -27501.4),
];

/// Even-odd point-in-polygon test on the x/y plane.
fn inside_polygon(polygon: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Server-chosen ids for the single research heart.
const HEART_REGION: u32 = 1;
const HEART_OBJECTIVE: u32 = 0x38;

/// Show the heart: region, counter, sub-region, enter (651, 1090, 657, 655).
fn show_heart(heart: &mut Heart, player: u32, out: &mut BytesMut) -> io::Result<()> {
    for frame in [
        RenownFrame209026::AddRegion { player, region: HEART_REGION },
        RenownFrame209026::CreateCounter { objective: HEART_OBJECTIVE, target: heart.target },
        RenownFrame209026::AddSubRegion { player, region: HEART_REGION, objective: HEART_OBJECTIVE, heart: heart.key },
    ] {
        frame.encode_frame(out)?;
    }
    if heart.inside {
        RenownFrame209026::EnterSubRegion { player, heart: heart.key }.encode_frame(out)?;
    }
    heart.player = Some(player);
    eprintln!("Mission phase: renown heart {} shown, 0/{} (server 651, 1090, 657, 655)", heart.key, heart.target);
    Ok(())
}

/// A kill advances the heart counter (1089); at the target it completes (654).
fn advance_heart(heart: &mut Option<Heart>, out: &mut BytesMut) -> io::Result<()> {
    let Some(heart) = heart.as_mut() else { return Ok(()) };
    let Some(player) = heart.player else { return Ok(()) };
    if heart.done {
        return Ok(());
    }
    heart.progress = (heart.progress + 1).min(heart.target);
    RenownFrame209026::SetCounter { objective: HEART_OBJECTIVE, value: heart.progress }.encode_frame(out)?;
    eprintln!("Mission phase: renown heart {} progress {}/{} (server 1089)", heart.key, heart.progress, heart.target);
    if heart.progress == heart.target {
        RenownFrame209026::CompleteSubRegion { player, region: HEART_REGION, heart: heart.key }.encode_frame(out)?;
        heart.done = true;
        eprintln!("Mission phase: renown heart {} completed (server 654)", heart.key);
    }
    Ok(())
}

/// Starter Backpack 8932 capacity (subtype +0x28 = 20); loot fills these.
const STARTER_BAG_SLOTS: u32 = 20;
/// m_inventorySlots size sent with 305: the client scans `bag-slot count (5)
/// << 5` = 160 inventory slots with an asserting getter (GpDownedManager on
/// defeat, `0x1411ee490`), so the array must cover 5 x 32 even though only
/// the backpack's 20 are usable (run 178, swarm note inventory-slot-assert.md).
const INVENTORY_SLOT_ARRAY: u32 = 5 * 32;
/// Research instance IDs for the backpack and drops (registry is sparse).
const STARTER_BAG_INSTANCE: u32 = 0x6f00;
const LOOT_INSTANCE_BASE: u32 = 0x6f10;

/// Kill drops, in rotation: level-1 trophies and crafting materials from the
/// client's item definitions (swarm note loot-items.md). The client holds no
/// drop tables, so the choice and the order are ours; each is a stack of 1
/// created by 968 and placed by 276 into the next backpack slot.
const KILL_LOOT_TABLE: [(u32, &str); 6] = [
    (19526, "Broken Claw"),
    (24346, "Tiny Claw"),
    (19538, "Broken Fang"),
    (87275, "Branded Shard"),
    (19561, "Patch of Fur"),
    (24352, "Tiny Fang"),
];

/// On an NPC kill: create the next table item and place it with 276 in the
/// next backpack slot (docs/re/209026-npc-talk-loop.md, loot).
fn award_kill_loot(loot: &mut Option<u32>, character_index: u32, out: &mut BytesMut) -> io::Result<()> {
    let Some(drops) = loot.as_mut() else { return Ok(()) };
    if *drops >= STARTER_BAG_SLOTS {
        eprintln!("Mission phase: backpack full, no loot");
        return Ok(());
    }
    let slot = *drops;
    let (item_key, name) = KILL_LOOT_TABLE[slot as usize % KILL_LOOT_TABLE.len()];
    let instance = LOOT_INSTANCE_BASE + slot;
    encode_stack_item_creation(instance, item_key, 1, out)?;
    InventoryItemFrame209026 { character_index, item_instance: instance, reason: 0, slot }.encode_frame(out)?;
    *drops += 1;
    eprintln!("Mission phase: loot {name} (item {item_key}) -> backpack slot {slot} (server 968 + 276)");
    Ok(())
}

/// Seconds between a test NPC's death and its revival (placeholder).
const RESPAWN_DELAY: Duration = Duration::from_secs(10);

/// On an NPC kill: add experience, send 191, and 188 when the client's
/// experience table crosses a level (docs/re/209026-npc-talk-loop.md, XP).
fn award_kill_xp(xp: &mut Option<(u32, u32, u8)>, character_index: u32, out: &mut BytesMut) {
    let Some(per_kill) = xp.map(|(per_kill, ..)| per_kill) else { return };
    award_xp(xp, per_kill, "kill", character_index, out);
}

/// Add `amount` experience for `reason`: 191 with the new total, and 188 when
/// the client's experience table crosses a level.
fn award_xp(xp: &mut Option<(u32, u32, u8)>, amount: u32, reason: &str, character_index: u32, out: &mut BytesMut) {
    let Some((_, total, level)) = xp.as_mut() else { return };
    *total = total.saturating_add(amount);
    ExperienceFrame209026 { character_index, experience: *total, dword_2: 0, byte_3: 0 }.encode_frame(out);
    let reached = level_for_experience(*total);
    eprintln!("Mission phase: {reason} experience +{amount} -> {total} (server 191)");
    if reached > *level {
        *level = reached;
        EarnedLevelFrame209026 { character_index, level: reached }.encode_frame(out);
        eprintln!("Mission phase: level up to {reached} (server 188)");
    }
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
        let client = client_messages()?;
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
            talk: None,
            hostiles: Vec::new(),
            last_target: None,
            aggro: Vec::new(),
            skill_order_echo: false,
            npc_cast: None,
            damage_on_skill: None,
            autoattack_skill: None,
            xp: None,
            starter_bag: None,
            late_npc: None,
            heart: None,
            respawn_flag: false,
            respawn_point: None,
            rejoin_route: None,
            restore_loot: 0,
            restore_xp: None,
            moves_seen: 0,
            kill_loot: None,
            npc_attack: None,
            sourced_damage: None,
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

    /// Spawn a talkable test NPC after BeginPlaying (needs a population).
    pub fn with_talk_npc(mut self, npc: TalkNpc) -> Self {
        self.talk = Some(npc);
        self
    }

    /// Add a hostile test NPC with the population. The first gets agent index 3
    /// (the talk NPC keeps 2, the player is 1), each further call the next index,
    /// up to MAX_HOSTILE_NPCS. Research input: it only fights back when
    /// `with_npc_attack` is set; it lets the client's attack requests be observed.
    pub fn with_hostile_npc(mut self, species_key: u32, position: [f32; 3]) -> io::Result<Self> {
        if self.hostiles.len() >= MAX_HOSTILE_NPCS {
            return Err(io::Error::new(ErrorKind::InvalidInput, "too many hostile NPCs"));
        }
        let plan = TalkNpcPlan209026 {
            agent_index: 3 + self.hostiles.len() as u16,
            species_key,
            position,
            facing: [1.0, 0.0, 0.0], // placeholder
            attitude: 1,             // attackable (inferred: hostile)
            talkable: false,
        };
        plan.spawn_frames()?;
        self.hostiles.push(HostileNpc { plan, health: LEVEL1_NPC_HEALTH, respawn: None });
        Ok(self)
    }

    /// Research hypothesis (docs/re/209026-npc-talk-loop.md section 6): answer
    /// each SkillBegin with server 34 carrying the packed 0x78 order.
    pub fn with_skill_order_echo(mut self) -> Self {
        self.skill_order_echo = true;
        self
    }

    /// Research: make the hostile NPC cast `skill_key` whenever the player
    /// sends SkillBegin (docs/re/209026-npc-talk-loop.md section 6).
    pub fn with_npc_cast(mut self, skill_key: u32) -> Self {
        self.npc_cast = Some((skill_key, 0x4000_0000));
        self
    }

    /// Research: player skills aimed at the hostile NPC lower its health by
    /// `amount` via server 257 (the NPC spawns with 100). Clamped at 0.
    pub fn with_damage_on_skill(mut self, amount: f32) -> Self {
        self.damage_on_skill = Some(amount);
        self
    }

    /// Research: answer the weapon repeat-start signal with a 0x78 order.
    pub fn with_autoattack_skill(mut self, skill_key: u32) -> Self {
        self.autoattack_skill = Some(skill_key);
        self
    }

    /// Research: award `per_kill` experience on every NPC kill (level 1 start).
    pub fn with_xp_per_kill(mut self, per_kill: u32) -> Self {
        self.xp = Some((per_kill, 0, 1));
        self
    }

    /// Research: create the Starter Backpack with item-creation `message_id`
    /// after BeginPlaying and equip it into bag slot 0 (server 300).
    pub fn with_sourced_damage(mut self, kind: u8) -> Self {
        self.sourced_damage = Some(kind);
        self
    }

    /// Requires a hostile NPC and the player's starting health.
    pub fn with_npc_attack(mut self, damage: f32, player_health: f32) -> Self {
        self.npc_attack = Some(NpcAttack { damage, player_health, max_health: player_health, next: None, defeat_at: None, defeated: false });
        self
    }

    /// Requires with_starter_bag (276 needs 305's slots).
    pub fn with_kill_loot(mut self) -> Self {
        self.kill_loot = Some(0);
        self
    }

    /// Requires a respawn point: the respawn request then rejoins the map there.
    pub fn with_rejoin_route(mut self, route: std::net::SocketAddr) -> Self {
        self.rejoin_route = Some(route);
        self
    }

    pub fn with_respawn_point(mut self, point: [f32; 3]) -> Self {
        self.respawn_point = Some(point);
        self
    }

    pub fn with_respawn_flag(mut self) -> Self {
        self.respawn_flag = true;
        self
    }

    /// `target` kills complete renown heart `key` (a type-72 content key).
    pub fn with_heart(mut self, key: u16, target: u32) -> io::Result<Self> {
        let spawn = self.population.as_ref().map(|f| f.plan.position);
        if target == 0 {
            return Err(io::Error::new(ErrorKind::InvalidInput, "heart target must be nonzero"));
        }
        let inside = key != 56 || spawn.is_none_or(|p| inside_polygon(&HEART_56_POLYGON, p[0], p[1]));
        self.heart = Some(Heart { key, target, progress: 0, player: None, done: false, rewarded: false, inside });
        Ok(self)
    }

    /// Agent 20, attitude 3 (neutral), not talkable.
    pub fn with_late_npc(mut self, species_key: u32, position: [f32; 3]) -> io::Result<Self> {
        let plan = TalkNpcPlan209026 { agent_index: 20, species_key, position, facing: [1.0, 0.0, 0.0], attitude: 3, talkable: false };
        plan.spawn_frames()?;
        self.late_npc = Some(plan);
        Ok(self)
    }

    pub fn with_starter_bag(mut self, message_id: u16) -> Self {
        self.starter_bag = Some(message_id);
        self
    }

    /// Slot of the living hostile NPC with this agent index.
    fn living_hostile(&self, agent: u32) -> Option<usize> {
        self.hostiles.iter().position(|h| u32::from(h.plan.agent_index) == agent && h.health > 0.0)
    }

    /// Auto-attack target (the client names none): the NPC the player last hit
    /// with a SkillBegin if alive, else the living hostile NPC nearest `origin`
    /// (the player spawn; the first added wins a tie), else nothing.
    fn autoattack_target(&self, origin: [f32; 3]) -> Option<usize> {
        if let Some(slot) = self.last_target.and_then(|agent| self.living_hostile(u32::from(agent))) {
            return Some(slot);
        }
        let distance = |p: [f32; 3]| (0..3).map(|i| (p[i] - origin[i]).powi(2)).sum::<f32>();
        self.hostiles
            .iter()
            .enumerate()
            .filter(|(_, h)| h.health > 0.0)
            .min_by(|(_, a), (_, b)| distance(a.plan.position).total_cmp(&distance(b.plan.position)))
            .map(|(slot, _)| slot)
    }

    /// One player hit on hostile NPC `slot`: lower its health, send the change,
    /// make it the aggro target, and on death send the kill, XP, loot and
    /// rally frames and start its respawn timer. Needs `damage_on_skill`.
    fn hit_hostile(&mut self, slot: usize, player: u32, skill_key: u32, remote_tick: u32, what: &str, out: &mut BytesMut) -> io::Result<()> {
        let Some(amount) = self.damage_on_skill else { return Ok(()) };
        let npc = self.hostiles[slot].plan.agent_index;
        let health = self.hostiles[slot].health;
        let remaining = (health - amount).max(0.0);
        self.hostiles[slot].health = remaining;
        self.aggro.retain(|&a| a != npc);
        self.aggro.push(npc);
        start_npc_attack(&mut self.npc_attack);
        let id = encode_health_change(out, self.sourced_damage, u32::from(npc), (health, remaining), skill_key, player)?;
        eprintln!("Mission phase: {what} hit NPC {npc}: health {health} -> {remaining} (server {id})");
        if remaining == 0.0 && health > 0.0 {
            // Packed 0x33 dead transition through server 84; flags mask 0x10
            // selects the character transition dispatcher
            // (docs/re/209026-health-transitions.md). Killer = the player.
            let dead = HealthTransitionPayload209026::Dead(HealthTransitionDetails209026 {
                detail_word: 0,
                detail_byte: 0,
                content_key: skill_key,
                agent_id: player,
                trailing_byte: 0,
            });
            AgentBehaviorFrame209026 {
                agent_index: npc,
                remote_tick,
                command_flags: 0x10,
                behavior_value: 0x4000_0100,
                behavior_data: dead.encode_payload(),
            }
            .encode_frame(out)?;
            eprintln!("Mission phase: NPC {npc} died (server 84 packed 0x33)");
            award_kill_xp(&mut self.xp, player, out);
            advance_heart(&mut self.heart, out)?;
            if let Some(heart) = self.heart.as_mut()
                && heart.done
                && !heart.rewarded
            {
                heart.rewarded = true;
                RenownFrame209026::HeartReward {
                    reward: 1,
                    heart: u32::from(heart.key),
                    currencies: vec![(1, HEART_COINS), (2, HEART_KARMA)],
                }
                .encode_frame(out)?;
                eprintln!("Mission phase: renown heart {} reward panel (server 556 type 14)", heart.key);
                award_xp(&mut self.xp, HEART_XP, "renown heart", player, out);
            }
            if self.heart.is_some_and(|h| h.done && h.key == 56)
                && let Some(talk) = self.talk.as_mut()
            {
                talk.set_dialogue(HEART_56_SPEAKER, &HEART_56_DONE)?;
            }
            award_kill_loot(&mut self.kill_loot, player, out)?;
            self.hostiles[slot].respawn = Some(Instant::now() + RESPAWN_DELAY);
            // The dead NPC stops attacking; another hit NPC that is still alive
            // takes over as the attacker (the aggro list's new last entry).
            self.aggro.retain(|&a| a != npc);
            if self.last_target == Some(npc) {
                self.last_target = None;
            }
            let rallied = rally_on_kill(&mut self.npc_attack, player, out)?;
            if let Some(attack) = self.npc_attack.as_mut()
                && (rallied || self.aggro.is_empty())
            {
                attack.next = None;
            }
        }
        Ok(())
    }

    /// When the held population is due, if one is held.
    pub fn due_at(&self) -> Option<Instant> {
        let held = self.held.as_ref().map(|(due, _)| *due);
        let attack = self.npc_attack.and_then(|a| a.next);
        let defeat = self.npc_attack.and_then(|a| a.defeat_at);
        let respawns = self.hostiles.iter().filter_map(|h| h.respawn);
        [held, attack, defeat].into_iter().flatten().chain(respawns).min()
    }

    /// The held population once `now` has reached its due time (once).
    pub fn take_due(&mut self, now: Instant) -> Option<Bytes> {
        if self.held.as_ref().is_some_and(|(due, _)| now >= *due) {
            eprintln!("Mission phase: sending the held population sequence");
            return self.held.take().map(|(_, bytes)| bytes);
        }
        if let Some(attack) = self.npc_attack.as_mut()
            && attack.defeat_at.is_some_and(|due| now >= due)
        {
            attack.defeat_at = None;
            attack.defeated = true;
            let character_index = self.population.as_ref()?.plan.character_index;
            let npc = u32::from(self.aggro.last().copied().or_else(|| self.hostiles.first().map(|h| h.plan.agent_index))?);
            let mut out = BytesMut::new();
            encode_transition(&mut out, character_index as u16, false, npc, 0x4000_0400).ok()?;
            // 169 {char, 1} sets character flag bit 1, the guard that lets the
            // defeat screen's "Return to a Waypoint" send client 210 (swarm note
            // rewards-waypoint.md; the 0x33 transition does not set it).
            if self.respawn_flag {
                encode_respawn_flag(&mut out, character_index, true);
            }
            eprintln!("Mission phase: player defeated by NPC {npc} (server 84 packed 0x33)");
            return Some(out.freeze());
        }
        if let Some(attack) = self.npc_attack.as_mut()
            && attack.next.is_some_and(|due| now >= due)
        {
            let character_index = self.population.as_ref()?.plan.character_index;
            // The attacker is the NPC the player hit most recently and that is still alive.
            let Some(&npc) = self.aggro.last() else {
                attack.next = None;
                return None;
            };
            let npc = u32::from(npc);
            let remaining = attack.player_health - attack.damage;
            if remaining < 1.0 {
                // The hit takes the player to 0, then the packed 0x34 downed
                // transition; defeat (0x33) follows after DOWNED_TO_DEFEAT.
                // Retail downed health and downed skills are not modelled.
                let previous = attack.player_health;
                attack.player_health = 0.0;
                attack.next = None;
                attack.defeat_at = Some(now + DOWNED_TO_DEFEAT);
                let mut out = BytesMut::new();
                let id = encode_health_change(&mut out, self.sourced_damage, character_index, (previous, 0.0), 0, npc).ok()?;
                encode_transition(&mut out, character_index as u16, true, npc, 0x4000_0300).ok()?;
                // Downed pool on meter 1, draining to 0 over DOWNED_TO_DEFEAT
                // (placeholder size = the normal maximum).
                let pool = attack.max_health;
                encode_health_pool(&mut out, character_index, pool, pool, -pool / DOWNED_TO_DEFEAT.as_secs_f32());
                eprintln!("Mission phase: NPC {npc} hit player for {previous}: health -> 0 (server {id}), player downed (server 84 packed 0x34)");
                return Some(out.freeze());
            }
            attack.player_health = remaining;
            attack.next = Some(now + NPC_ATTACK_INTERVAL);
            let mut out = BytesMut::new();
            let id = encode_health_change(&mut out, self.sourced_damage, character_index, (attack.player_health + attack.damage, remaining), 0, npc).ok()?;
            eprintln!("Mission phase: NPC {npc} hit player for {}: health -> {remaining} (server {id})", attack.damage);
            return Some(out.freeze());
        }
        if let Some(hostile) = self.hostiles.iter_mut().find(|h| h.respawn.is_some_and(|due| now >= due)) {
            hostile.respawn = None;
            hostile.health = LEVEL1_NPC_HEALTH;
            let npc = hostile.plan.agent_index;
            let mut out = BytesMut::new();
            // Packed 0x49 alive transition (docs/re/209026-health-transitions.md),
            // then full health; the same agent revives, so its model stays loaded.
            AgentBehaviorFrame209026 {
                agent_index: npc,
                remote_tick: 0,
                command_flags: 0x10,
                behavior_value: 0x4000_0200,
                behavior_data: HealthTransitionPayload209026::Alive { detail_byte: 0, trailing_byte: 0 }.encode_payload(),
            }
            .encode_frame(&mut out)
            .ok()?;
            HealthFrame209026 {
                character_index: u32::from(npc),
                operation: HealthOperation209026::Currents { current: LEVEL1_NPC_HEALTH, second_current: 0.0 },
            }
            .encode_frame(&mut out);
            eprintln!("Mission phase: NPC {npc} respawned (server 84 packed 0x49, health {LEVEL1_NPC_HEALTH})");
            return Some(out.freeze());
        }
        None
    }

    /// Server frames owed for one framed client message: population after
    /// MapLoaded (once), then BeginPlaying after AgentsLoaded (once, and only
    /// after population). Nothing without a population fixture.
    pub fn respond(&mut self, seen: &Seen) -> io::Result<Option<Bytes>> {
        if self.rejoin_route.is_some()
            && !self.populated
            && matches!(seen, Seen::Map(ClientMapEvent::MapLoaded))
            && let Some(carry) = CARRY.lock().ok().and_then(|mut c| c.take())
        {
            if let Some(fixture) = self.population.as_mut() {
                fixture.plan.position = carry.spawn;
            }
            if let (Some((_, total, level)), Some((t, l))) = (self.xp.as_mut(), carry.xp) {
                (*total, *level) = (t, l);
                self.restore_xp = Some((t, l));
            }
            if let (Some(drops), Some(n)) = (self.kill_loot.as_mut(), carry.loot_drops) {
                *drops = n;
                self.restore_loot = n;
            }
            if let (Some(heart), Some((progress, done, rewarded))) = (self.heart.as_mut(), carry.heart) {
                heart.progress = progress;
                heart.done = done;
                heart.rewarded = rewarded;
                heart.inside = heart.key != 56 || inside_polygon(&HEART_56_POLYGON, carry.spawn[0], carry.spawn[1]);
            }
            eprintln!("Mission phase: rejoin carry-over applied {carry:?}");
        }
        let Some(fixture) = &self.population else {
            return Ok(None);
        };
        // Auto-attack follows the NPC last attacked with a SkillBegin, whatever
        // the damage setting (or a defeat) does with the skill itself.
        if self.began
            && let Seen::Skill(skill) = seen
            && let Some(slot) = self.living_hostile(skill.request.target)
        {
            self.last_target = Some(self.hostiles[slot].plan.agent_index);
        }
        match seen {
            Seen::Map(ClientMapEvent::MapLoaded) if !self.populated => {
                self.populated = true;
                let mut stream = fixture.population_stream()?;
                // Test NPCs go out with the population, inside the loader's agent and
                // model streaming: spawned after BeginPlaying their models never
                // streamed in and they stayed invisible (sandbox runs 122-135; run 136/
                // 137 render). docs/re/209026-npc-talk-loop.md section 6.
                let mut npcs = BytesMut::new();
                let talk = self.talk.as_ref().map(|npc| (&npc.plan, "talk"));
                let hostile = self.hostiles.iter().map(|h| (&h.plan, "hostile"));
                for (plan, kind) in talk.into_iter().chain(hostile) {
                    if u32::from(plan.agent_index) == fixture.plan.character_index {
                        eprintln!("Mission phase: {kind} NPC agent index collides with the player; not spawned");
                        continue;
                    }
                    npcs.extend_from_slice(&plan.spawn_frames()?);
                    eprintln!(
                        "Mission phase: spawning {kind} NPC agent {} (species {}) with the population",
                        plan.agent_index, plan.species_key
                    );
                }
                if !npcs.is_empty() {
                    let mut joined = BytesMut::from(&stream[..]);
                    joined.extend_from_slice(&npcs);
                    stream = joined.freeze();
                }
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
                if let Some(message_id) = self.starter_bag {
                    let instance = STARTER_BAG_INSTANCE;
                    encode_minimal_item_creation(message_id, instance, STARTER_BACKPACK_ITEM, &mut out)?;
                    BagSlotFrame209026 { character_index: fixture.plan.character_index, bag_slot: 0, item_instance: instance }
                        .encode_frame(&mut out);
                    // 300 does not size m_inventorySlots; 305 grows it to the
                    // backpack's 20 slots (item 8932 def +0x4c).
                    InventorySlotCountFrame209026 { character_index: fixture.plan.character_index, slot_count: INVENTORY_SLOT_ARRAY }
                        .encode_frame(&mut out)?;
                    eprintln!("Mission phase: Starter Backpack via server {message_id} + 300 bag slot 0 + 305 {INVENTORY_SLOT_ARRAY} slots");
                }
                if let Some(plan) = &self.late_npc {
                    out.extend_from_slice(&plan.spawn_frames()?);
                    eprintln!("Mission phase: late NPC agent {} (species {}) after BeginPlaying", plan.agent_index, plan.species_key);
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
                for slot in 0..self.restore_loot.min(STARTER_BAG_SLOTS) {
                    let (item_key, _) = KILL_LOOT_TABLE[slot as usize % KILL_LOOT_TABLE.len()];
                    let instance = LOOT_INSTANCE_BASE + slot;
                    encode_stack_item_creation(instance, item_key, 1, &mut out)?;
                    InventoryItemFrame209026 { character_index: fixture.plan.character_index, item_instance: instance, reason: 0, slot }
                        .encode_frame(&mut out)?;
                }
                if let Some((total, level)) = self.restore_xp {
                    ExperienceFrame209026 { character_index: fixture.plan.character_index, experience: total, dword_2: 0, byte_3: 0 }
                        .encode_frame(&mut out);
                    EarnedLevelFrame209026 { character_index: fixture.plan.character_index, level }.encode_frame(&mut out);
                }
                if self.restore_loot > 0 || self.restore_xp.is_some() {
                    eprintln!("Mission phase: restored {} loot items and experience {:?} after rejoin", self.restore_loot, self.restore_xp);
                }
                if let Some(heart) = self.heart.as_mut() {
                    show_heart(heart, fixture.plan.player_index, &mut out)?;
                    if heart.progress > 0 {
                        RenownFrame209026::SetCounter { objective: HEART_OBJECTIVE, value: heart.progress }.encode_frame(&mut out)?;
                    }
                    if heart.done {
                        RenownFrame209026::CompleteSubRegion { player: fixture.plan.player_index, region: HEART_REGION, heart: heart.key }
                            .encode_frame(&mut out)?;
                    }
                    // 384 ties the talk NPC to the heart: the client puts the
                    // heart marker (map, compass, "!" bang) on it. The sub-region
                    // already exists (657 above), so the marker is created now.
                    // Sent once: a second 384 asserts while the heart is active.
                    if let Some(talk) = self.talk.as_mut() {
                        if heart.key == 56 {
                            talk.set_dialogue(HEART_56_SPEAKER, &HEART_56_ACTIVE)?;
                        }
                        let character = u32::from(talk.plan.agent_index);
                        RenownFrame209026::HeartNpc { character, heart: u32::from(heart.key) }.encode_frame(&mut out)?;
                        eprintln!("Mission phase: renown heart {} on NPC {character} (server 384)", heart.key);
                    }
                }
                Ok(Some(out.freeze()))
            }
            Seen::Other(id @ (208 | 210)) if self.npc_attack.is_some_and(|a| a.defeated) => {
                // 210 {}: "Return to a Waypoint" with no known waypoint; 208
                // {poi}: a waypoint picked on the map. Both mean respawn here.
                let mut out = BytesMut::new();
                let character_index = fixture.plan.character_index;
                if let (Some(route), Some(point)) = (self.rejoin_route, self.respawn_point) {
                    // Rejoin the map at the waypoint on a new connection (1065,
                    // as the creator's creation -> world redirect). Fields:
                    // destination, unused, join scalar 0, map Word, 0, unused.
                    let carry = CarryOver {
                        spawn: point,
                        xp: self.xp.map(|(_, total, level)| (total, level)),
                        loot_drops: self.kill_loot,
                        heart: self.heart.map(|h| (h.progress, h.done, h.rewarded)),
                    };
                    if let Ok(mut slot) = CARRY.lock() {
                        *slot = Some(carry);
                    }
                    use gw2_proto::Encode;
                    1065u16.encode(&mut out);
                    route.encode(&mut out);
                    0u32.encode(&mut out);
                    0u32.encode(&mut out);
                    (self.map_key as u16).encode(&mut out);
                    0u32.encode(&mut out);
                    0u32.encode(&mut out);
                    eprintln!("Mission phase: client {id} respawn -> rejoin map {} at {point:?} via 1065 {route} (carry {carry:?})", self.map_key);
                    return Ok(Some(out.freeze()));
                }
                if let Some(attack) = self.npc_attack.as_mut() {
                    respawn_player(attack, character_index, &mut out)?;
                }
                if let Some(point) = self.respawn_point {
                    // Server 86 position update (docs/re/209026-agent-position-update.md);
                    // research: whether it moves the controlled player.
                    gw2_proto::agent_position::AgentPosition209026 {
                        agent_index: character_index as u16,
                        remote_tick: 0,
                        command_flags: 0,
                        position: gw2_proto::Point3(point, 0),
                    }
                    .encode_frame(&mut out);
                    eprintln!("Mission phase: respawn at {point:?} (server 86)");
                }
                eprintln!("Mission phase: client {id} respawn request -> player alive at full health (84 packed 0x49, 259/260, 169 0)");
                Ok(Some(out.freeze()))
            }
            Seen::Moved { position, .. } if self.began => {
                // 655/656 follow the player across heart 56's boundary; the
                // client never tests the polygon itself (renown-heart note).
                let Some(heart) = self.heart.as_mut() else { return Ok(None) };
                let (Some(player), false, 56) = (heart.player, heart.done, heart.key) else { return Ok(None) };
                let inside = inside_polygon(&HEART_56_POLYGON, position[0] as f32, position[1] as f32);
                if inside == heart.inside {
                    return Ok(None);
                }
                heart.inside = inside;
                let mut out = BytesMut::new();
                let frame = if inside {
                    RenownFrame209026::EnterSubRegion { player, heart: heart.key }
                } else {
                    RenownFrame209026::LeaveSubRegion { player, heart: heart.key }
                };
                frame.encode_frame(&mut out)?;
                eprintln!("Mission phase: player {} heart 56 area at {position:?} (server {})", if inside { "entered" } else { "left" }, frame.message_id());
                Ok(Some(out.freeze()))
            }
            Seen::Target(agent) if self.living_hostile(*agent).is_some() => {
                // The player's selection drives auto-attack (it carries no target).
                self.last_target = u16::try_from(*agent).ok();
                Ok(None)
            }
            Seen::Interact(request) if self.began => {
                let Some(npc) = self.talk.as_mut() else { return Ok(None) };
                let InteractRequest209026::Interact { target } = *request else { return Ok(None) };
                let reply = npc.session.on_interact(target)?;
                eprintln!(
                    "Mission phase: client interact target {target} ({})",
                    if reply.is_some() { "talk NPC: opening conversation" } else { "not the talk NPC" }
                );
                Ok(reply.map(BytesMut::freeze))
            }
            Seen::RepeatStart(start) if self.began && self.autoattack_skill.is_some() && !self.hostiles.is_empty() => {
                let player = fixture.plan.character_index;
                // The client sends no target with auto-attack (repeat start).
                let Some(slot) = self.autoattack_target(fixture.plan.position) else { return Ok(None) };
                let npc = self.hostiles[slot].plan.agent_index;
                let skill_key = self.autoattack_skill.unwrap();
                let order = SkillOrder209026 {
                    order_flags: 0,
                    skill_key,
                    slot: start.request.slot,
                    raw_7: 0,
                    agent: u32::from(npc),
                    byte_12: 0,
                };
                let mut out = BytesMut::new();
                AgentBehaviorOrderFrame209026 {
                    agent_index: start.agent_index,
                    remote_tick: start.remote_tick,
                    command_flags: start.command_flags,
                    behavior_value: start.behavior_value,
                    behavior_data: order.to_behavior_data().to_vec(),
                    word: 1000, // placeholder duration (m_durationMs)
                }
                .encode_frame(&mut out)?;
                eprintln!(
                    "Mission phase: repeat-start slot {} answered with server 34 0x78 skill {skill_key} on NPC {npc}",
                    start.request.slot
                );
                if self.damage_on_skill.is_some() && !self.npc_attack.is_some_and(|a| a.defeated) {
                    self.hit_hostile(slot, player, skill_key, start.remote_tick, "auto-attack", &mut out)?;
                }
                Ok(Some(out.freeze()))
            }
            Seen::Skill(skill)
                if self.began
                    && self.damage_on_skill.is_some()
                    && !self.npc_attack.is_some_and(|a| a.defeated)
                    && self.living_hostile(skill.request.target).is_some() =>
            {
                let player = fixture.plan.character_index;
                let slot = self.living_hostile(skill.request.target).unwrap();
                let mut out = BytesMut::new();
                let what = format!("skill {}", skill.request.skill_key);
                self.hit_hostile(slot, player, skill.request.skill_key, skill.remote_tick, &what, &mut out)?;
                Ok(Some(out.freeze()))
            }
            Seen::Skill(skill) if self.began && self.npc_cast.is_some() && !self.hostiles.is_empty() => {
                // The targeted hostile NPC casts, else the first one.
                let slot = self.living_hostile(skill.request.target).unwrap_or(0);
                let npc = self.hostiles[slot].plan.agent_index;
                let (skill_key, value) = self.npc_cast.unwrap();
                self.npc_cast = Some((skill_key, value.wrapping_add(1)));
                let order = SkillOrder209026 {
                    order_flags: 0,
                    skill_key,
                    // 23 bypasses the skill-bar slot mutation (the NPC has none).
                    slot: 23,
                    raw_7: 0,
                    agent: u32::from(npc), // placeholder: self-target
                    byte_12: 0,
                };
                let mut out = BytesMut::new();
                AgentBehaviorOrderFrame209026 {
                    agent_index: npc,
                    remote_tick: skill.remote_tick,
                    command_flags: 0,
                    behavior_value: value,
                    behavior_data: order.to_behavior_data().to_vec(),
                    // m_durationMs (command +0x70): a non-controlled agent's
                    // command asserts it nonzero (AgBase.cpp:822, run 127).
                    // placeholder: 1 s, not the skill's content duration.
                    word: 1000,
                }
                .encode_frame(&mut out)?;
                eprintln!("Mission phase: ordering hostile NPC {npc} to cast skill {skill_key} (server 34 0x78, value {value:#x})");
                Ok(Some(out.freeze()))
            }
            Seen::Skill(skill) if self.began && self.skill_order_echo => {
                let request = skill.request;
                let order = SkillOrder209026 {
                    order_flags: 0,
                    skill_key: request.skill_key,
                    slot: request.slot,
                    raw_7: 0,
                    // Target 0 (none): the caster itself (placeholder choice).
                    agent: if request.target == 0 { fixture.plan.character_index } else { request.target },
                    byte_12: 0,
                };
                let mut out = BytesMut::new();
                AgentBehaviorOrderFrame209026 {
                    agent_index: skill.agent_index,
                    remote_tick: skill.remote_tick,
                    command_flags: skill.command_flags,
                    behavior_value: skill.behavior_value,
                    behavior_data: order.to_behavior_data().to_vec(),
                    word: 0,
                }
                .encode_frame(&mut out)?;
                eprintln!(
                    "Mission phase: answering SkillBegin (skill {}, slot {}, value {:#x}) with server 34 0x78 order",
                    request.skill_key, request.slot, skill.behavior_value
                );
                Ok(Some(out.freeze()))
            }
            Seen::Conversation(reply) if self.began => {
                let Some(npc) = self.talk.as_mut() else { return Ok(None) };
                let answer = npc.session.on_reply(*reply)?;
                eprintln!("Mission phase: conversation reply {reply:?}; active now {:?}", npc.session.active());
                Ok(answer.map(BytesMut::freeze))
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
                _ => Self::talk_event(id, &frame).unwrap_or(Seen::Other(id)),
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
                Seen::Interact(request) => eprintln!("Mission phase: client 23 interact {request:?}"),
                Seen::Conversation(reply) => eprintln!("Mission phase: client conversation {reply:?}"),
                Seen::Skill(skill) => eprintln!("Mission phase: client 23 {skill:?}"),
                Seen::RepeatStart(start) => eprintln!(
                    "Mission phase: client 23 repeat-start [29 {:02x}] value {:#x}",
                    start.request.slot, start.behavior_value
                ),
                // Short fixed-layout messages without text (17 Word, 47 Byte,
                // 248): log their bytes for research. Never dump others; they
                // may carry typed text.
                Seen::Other(id @ (17 | 47 | 248)) if used <= 8 => {
                    let hex: Vec<String> = frame.iter().map(|b| format!("{b:02x}")).collect();
                    eprintln!("Mission phase: client message {id} ({used} bytes) [{}]", hex.join(" "))
                }
                Seen::Target(agent) => eprintln!("Mission phase: client 227 target agent {agent}"),
                Seen::Moved { id, position } => {
                    // Sparse research log of the player's reported position.
                    self.moves_seen = self.moves_seen.wrapping_add(1);
                    if self.moves_seen % 20 == 1 {
                        eprintln!("Mission phase: client {id} movement #{} at {position:?}", self.moves_seen);
                    }
                }
                Seen::Other(id) => eprintln!("Mission phase: client message {id} ({used} bytes)"),
            }
            seen.push(event);
        }
        seen
    }

    /// Client 23 with an interact payload, or 275/276. Other client-23
    /// payloads (skills, movement signals) stay `Other` and are logged with
    /// their leading opcode so new request kinds can be identified.
    fn talk_event(id: u16, frame: &[u8]) -> Option<Seen> {
        let mut input = frame;
        match id {
            18 | 19 => {
                use gw2_proto::Decode;
                let _ = u16::decode(&mut input).ok()?;
                let _ = u32::decode(&mut input).ok()?;
                let _ = u32::decode(&mut input).ok()?;
                let position = [f32::decode(&mut input).ok()?, f32::decode(&mut input).ok()?, f32::decode(&mut input).ok()?];
                position
                    .iter()
                    .all(|v| v.is_finite() && v.abs() < 1.0e7)
                    .then(|| {
                        let map = [position[0] * 32.0, position[1] * 32.0, -position[2] * 32.0];
                        Seen::Moved { id, position: map.map(|v| v.round() as i32) }
                    })
            }
            227 => {
                use gw2_proto::Decode;
                let _ = u16::decode(&mut input).ok()?;
                let agent = u32::decode(&mut input).ok()?;
                input.is_empty().then_some(Seen::Target(agent))
            }
            AgentBehaviorFrame209026::CLIENT_MESSAGE_ID => {
                let behavior = AgentBehaviorFrame209026::decode_client_frame(&mut input).ok()?;
                if let [0x29, slot] = behavior.behavior_data[..] {
                    return Some(Seen::RepeatStart(SkillSeen {
                        agent_index: behavior.agent_index,
                        remote_tick: behavior.remote_tick,
                        command_flags: behavior.command_flags,
                        behavior_value: behavior.behavior_value,
                        request: SkillBeginRequest209026 { target: 0, skill_key: 0, slot },
                    }));
                }
                if let Ok(request) = SkillBeginRequest209026::from_behavior_data(&behavior.behavior_data) {
                    return Some(Seen::Skill(SkillSeen {
                        agent_index: behavior.agent_index,
                        remote_tick: behavior.remote_tick,
                        command_flags: behavior.command_flags,
                        behavior_value: behavior.behavior_value,
                        request,
                    }));
                }
                match InteractRequest209026::from_behavior_data(&behavior.behavior_data) {
                    Ok(request) => Some(Seen::Interact(request)),
                    Err(_) => {
                        // Skill/target bytes only (no account data); at most 250.
                        let hex: Vec<String> = behavior.behavior_data.iter().map(|b| format!("{b:02x}")).collect();
                        eprintln!(
                            "Mission phase: client 23 flags {:#04x} value {} data [{}]",
                            behavior.command_flags,
                            behavior.behavior_value,
                            hex.join(" ")
                        );
                        None
                    }
                }
            }
            275 | 276 => ConversationReply209026::decode_frame(&mut input).ok().map(Seen::Conversation),
            _ => None,
        }
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
                        Seen::Interact(_) | Seen::Skill(_) | Seen::RepeatStart(_) => 23,
                        Seen::Target(_) => 227,
                        Seen::Moved { id, .. } => *id,
                        Seen::Conversation(ConversationReply209026::Choose { .. }) => 275,
                        Seen::Conversation(ConversationReply209026::Closed { .. }) => 276,
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
            assert_eq!(seen.len(), 3, "split {split}");
            assert_eq!(seen[0], Seen::Other(17), "split {split}");
            assert!(matches!(seen[1], Seen::Moved { id: 18, .. }), "split {split}");
            assert_eq!(seen[2], Seen::Other(17), "split {split}");
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
            assert_eq!(fixture.plan.initial_health, 1645.0, "legacy observer value is unchanged without an override");
            assert!(fixture.clone().with_initial_health(f32::NAN).is_err());
            let fixture = fixture.with_initial_health(388.0).unwrap();
            assert_eq!(fixture.plan.initial_health, 388.0);
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

    fn talk_phase() -> MissionPhase {
        let npc = TalkNpc::new(3172, [5.0, 0.0, 0.0], "Eir".to_owned()).unwrap();
        populated_phase().with_talk_npc(npc)
    }

    fn client_23(data: &[u8]) -> BytesMut {
        let mut out = BytesMut::new();
        AgentBehaviorFrame209026 {
            agent_index: 1,
            remote_tick: 77,
            command_flags: 3,
            behavior_value: 0,
            behavior_data: data.to_vec(),
        }
        .encode_client_frame(&mut out)
        .unwrap();
        out
    }

    #[test]
    fn talk_npc_spawns_with_the_population_and_answers_interact_and_replies() {
        use gw2_proto::npc_talk::{ConversationFrame209026, NpcInteractFlag209026};
        let mut phase = talk_phase();
        let agents = Seen::Map(ClientMapEvent::AgentsLoaded);
        let interact = Seen::Interact(InteractRequest209026::Interact { target: 2 });
        // Before BeginPlaying an interact is not answered.
        assert!(phase.respond(&interact).unwrap().is_none());
        let population = phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap().unwrap();
        // The NPC spawn (ending in 378) follows the player's population, inside map loading.
        let plan = phase.population.as_ref().unwrap().plan.clone();
        let player = population::population_stream(&plan).unwrap();
        let spawn = phase.talk.as_ref().unwrap().plan.spawn_frames().unwrap();
        assert_eq!(&population[..player.len()], &player[..]);
        assert_eq!(&population[player.len()..], &spawn[..]);
        assert_eq!(
            NpcInteractFlag209026::decode_frame(&mut &population[population.len() - 5..]).unwrap(),
            NpcInteractFlag209026::Talk { character_index: 2, talk: 1, byte_2: 0 }
        );
        // BeginPlaying is unchanged: 1036, 431 and 83 only.
        let begin = phase.respond(&agents).unwrap().unwrap();
        assert_eq!(&begin[..], &[0x0c, 0x04, 0xaf, 0x01, 0x53, 0x00, 0x01, 0x00, 0x01]);

        // Client 23 [0x14][2] reaches the phase as an interact and opens a conversation.
        let seen = phase.push(&client_23(&[0x14, 2, 0, 0, 0]));
        assert_eq!(seen, [interact]);
        let open = phase.respond(&interact).unwrap().unwrap();
        let mut input = &open[..];
        assert!(matches!(
            ConversationFrame209026::decode_frame(&mut input).unwrap(),
            ConversationFrame209026::Open { agent: 2, conversation: 1, stage: 1, .. }
        ));
        // Another target is ignored.
        let other = Seen::Interact(InteractRequest209026::Interact { target: 1 });
        assert!(phase.respond(&other).unwrap().is_none());

        // 275 Goodbye on the last page closes with 762; the 276 ack needs no answer.
        let mut reply = BytesMut::new();
        ConversationReply209026::Choose { conversation: 1, response: TalkSession209026::CONTINUE }
            .encode_frame(&mut reply);
        let seen = phase.push(&reply);
        assert!(matches!(seen[..], [Seen::Conversation(_)]));
        let next = phase.respond(&seen[0]).unwrap().unwrap();
        assert!(matches!(
            ConversationFrame209026::decode_frame(&mut &next[..]).unwrap(),
            ConversationFrame209026::Page { stage: 2, .. }
        ));
        let bye = Seen::Conversation(ConversationReply209026::Choose {
            conversation: 1,
            response: TalkSession209026::GOODBYE,
        });
        let close = phase.respond(&bye).unwrap().unwrap();
        assert_eq!(&close[..], &[0xfa, 0x02, 0x01]);
        let ack = Seen::Conversation(ConversationReply209026::Closed { conversation: 1 });
        assert!(phase.respond(&ack).unwrap().is_none());
        assert!(phase.enabled());
    }

    #[test]
    fn other_client_23_payloads_stay_raw_and_keep_the_phase() {
        let mut phase = talk_phase();
        // A 2-byte 0x2a signal is neither interact, skill nor repeat start.
        assert_eq!(phase.push(&client_23(&[0x2a, 0])), [Seen::Other(23)]);
        assert!(matches!(phase.push(&client_23(&[0x29, 5]))[..], [Seen::RepeatStart(_)]));
        // SkillBegin is recognised but unanswered without the research flag.
        let seen = phase.push(&client_23(&[0x23, 0, 0, 0, 0, 0xc0, 0x27, 0, 0, 0]));
        assert!(matches!(seen[..], [Seen::Skill(SkillSeen { request: SkillBeginRequest209026 { skill_key: 10176, .. }, .. })]));
        phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap();
        phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap();
        assert!(phase.respond(&seen[0]).unwrap().is_none());
        assert!(phase.enabled());
    }

    #[test]
    fn skill_order_echo_answers_with_server_34_and_the_request_value() {
        let mut phase = populated_phase().with_skill_order_echo();
        phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap();
        phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap();
        let seen = phase.push(&client_23(&[0x23, 0, 0, 0, 0, 0xc0, 0x27, 0, 0, 0]));
        let reply = phase.respond(&seen[0]).unwrap().unwrap();
        let frame = AgentBehaviorOrderFrame209026::decode_frame(&mut &reply[..]).unwrap();
        assert_eq!((frame.agent_index, frame.remote_tick, frame.command_flags, frame.behavior_value), (1, 77, 3, 0));
        assert_eq!(frame.behavior_data, [0x78, 0, 0xc0, 0x27, 0, 0, 0, 0, 1, 0, 0, 0, 0]);
    }

    #[test]
    fn talk_npc_colliding_with_the_player_is_not_spawned() {
        let mut npc = TalkNpc::new(3172, [5.0, 0.0, 0.0], "Eir".to_owned()).unwrap();
        npc.plan.agent_index = 1;
        let mut phase = populated_phase().with_talk_npc(npc);
        let population = phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap().unwrap();
        let plan = phase.population.as_ref().unwrap().plan.clone();
        assert_eq!(population, population::population_stream(&plan).unwrap(), "no NPC frames appended");
    }

    fn hostile_phase(damage: f32) -> MissionPhase {
        let mut phase = populated_phase()
            .with_hostile_npc(100, [50.0, 0.0, 0.0])
            .unwrap()
            .with_hostile_npc(101, [5.0, 0.0, 0.0])
            .unwrap()
            .with_damage_on_skill(damage)
            .with_autoattack_skill(77);
        phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap();
        phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap();
        phase
    }

    fn skill_at(target: u32) -> Seen {
        Seen::Skill(SkillSeen {
            agent_index: 1,
            remote_tick: 5,
            command_flags: 0,
            behavior_value: 0,
            request: SkillBeginRequest209026 { target, skill_key: 10176, slot: 1 },
        })
    }

    fn repeat_start() -> Seen {
        Seen::RepeatStart(SkillSeen {
            agent_index: 1,
            remote_tick: 6,
            command_flags: 0,
            behavior_value: 0,
            request: SkillBeginRequest209026 { target: 0, skill_key: 0, slot: 1 },
        })
    }

    fn health_of(phase: &MissionPhase, agent: u16) -> f32 {
        phase.hostiles.iter().find(|h| h.plan.agent_index == agent).unwrap().health
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|w| w == needle)
    }

    fn dead_frame(agent: u16, skill_key: u32, tick: u32) -> Vec<u8> {
        let dead = HealthTransitionPayload209026::Dead(HealthTransitionDetails209026 {
            detail_word: 0,
            detail_byte: 0,
            content_key: skill_key,
            agent_id: 1,
            trailing_byte: 0,
        });
        let mut out = BytesMut::new();
        AgentBehaviorFrame209026 { agent_index: agent, remote_tick: tick, command_flags: 0x10, behavior_value: 0x4000_0100, behavior_data: dead.encode_payload() }
            .encode_frame(&mut out)
            .unwrap();
        out.to_vec()
    }

    #[test]
    fn two_hostile_npcs_spawn_with_agent_indices_3_and_4() {
        let mut phase = populated_phase().with_hostile_npc(100, [50.0, 0.0, 0.0]).unwrap().with_hostile_npc(101, [5.0, 0.0, 0.0]).unwrap();
        let indices: Vec<u16> = phase.hostiles.iter().map(|h| h.plan.agent_index).collect();
        assert_eq!(indices, [3, 4]);
        let population = phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap().unwrap();
        let player = population::population_stream(&phase.population.as_ref().unwrap().plan).unwrap();
        let first = phase.hostiles[0].plan.spawn_frames().unwrap();
        let second = phase.hostiles[1].plan.spawn_frames().unwrap();
        assert_eq!(&population[..player.len()], &player[..]);
        assert_eq!(&population[player.len()..player.len() + first.len()], &first[..]);
        assert_eq!(&population[player.len() + first.len()..], &second[..]);
        assert!(phase.hostiles.iter().all(|h| h.plan.attitude == 1 && h.health == LEVEL1_NPC_HEALTH));
    }

    #[test]
    fn at_most_sixteen_hostile_npcs() {
        let mut phase = populated_phase();
        for i in 0..MAX_HOSTILE_NPCS {
            phase = phase.with_hostile_npc(100, [i as f32, 0.0, 0.0]).unwrap();
        }
        assert_eq!(phase.hostiles.last().unwrap().plan.agent_index, 18);
        assert!(phase.with_hostile_npc(100, [0.0; 3]).is_err());
    }

    #[test]
    fn killing_one_hostile_npc_leaves_the_other_and_respawns_independently() {
        let mut phase = hostile_phase(59.0);
        // Skill hits on NPC 3: 118 -> 59 -> 0.
        let hit = phase.respond(&skill_at(3)).unwrap().unwrap();
        assert!(!contains(&hit, &dead_frame(3, 10176, 5)));
        assert_eq!((health_of(&phase, 3), health_of(&phase, 4)), (59.0, LEVEL1_NPC_HEALTH));
        let kill = phase.respond(&skill_at(3)).unwrap().unwrap();
        assert!(contains(&kill, &dead_frame(3, 10176, 5)), "kill frame for agent 3");
        assert_eq!((health_of(&phase, 3), health_of(&phase, 4)), (0.0, LEVEL1_NPC_HEALTH));
        assert!(phase.hostiles[0].respawn.is_some() && phase.hostiles[1].respawn.is_none());
        // A dead NPC takes no more hits and the live one is still hittable.
        assert!(phase.respond(&skill_at(3)).unwrap().is_none());
        phase.respond(&skill_at(4)).unwrap().unwrap();
        assert_eq!(health_of(&phase, 4), 59.0);
        // Kill NPC 4 too, then make only NPC 3's timer due.
        let kill = phase.respond(&skill_at(4)).unwrap().unwrap();
        assert!(contains(&kill, &dead_frame(4, 10176, 5)));
        let now = Instant::now();
        let first_due = phase.hostiles[0].respawn.unwrap();
        assert!(phase.hostiles[1].respawn.unwrap() >= first_due);
        assert_eq!(phase.due_at(), Some(first_due));
        assert!(phase.take_due(now).is_none(), "nothing is due before the delay");
        phase.hostiles[1].respawn = Some(first_due + Duration::from_secs(5));
        let late = first_due + Duration::from_secs(1);
        let revive = phase.take_due(late).unwrap();
        assert_eq!(health_of(&phase, 3), LEVEL1_NPC_HEALTH);
        assert_eq!(health_of(&phase, 4), 0.0, "NPC 4 stays dead");
        assert!(phase.hostiles[0].respawn.is_none() && phase.hostiles[1].respawn.is_some());
        let mut alive = BytesMut::new();
        AgentBehaviorFrame209026 {
            agent_index: 3,
            remote_tick: 0,
            command_flags: 0x10,
            behavior_value: 0x4000_0200,
            behavior_data: HealthTransitionPayload209026::Alive { detail_byte: 0, trailing_byte: 0 }.encode_payload(),
        }
        .encode_frame(&mut alive)
        .unwrap();
        assert!(contains(&revive, &alive));
        assert_eq!(phase.due_at(), phase.hostiles[1].respawn);
        assert!(phase.take_due(late).is_none());
        phase.take_due(first_due + Duration::from_secs(6)).unwrap();
        assert_eq!(health_of(&phase, 4), LEVEL1_NPC_HEALTH);
        assert!(phase.due_at().is_none());
    }

    #[test]
    fn auto_attack_hits_the_last_skill_target_else_the_nearest_living_hostile() {
        let mut phase = hostile_phase(10.0);
        // No SkillBegin yet: the nearest NPC to the spawn (agent 4 at x=5) is hit.
        phase.respond(&repeat_start()).unwrap().unwrap();
        assert_eq!((health_of(&phase, 3), health_of(&phase, 4)), (LEVEL1_NPC_HEALTH, 108.0));
        // A SkillBegin on the far NPC makes it the auto-attack target.
        phase.respond(&skill_at(3)).unwrap().unwrap();
        phase.respond(&repeat_start()).unwrap().unwrap();
        assert_eq!((health_of(&phase, 3), health_of(&phase, 4)), (98.0, 108.0));
        // Once it is dead the nearest living one is hit again.
        phase.hostiles[0].health = 5.0;
        phase.respond(&repeat_start()).unwrap().unwrap();
        assert_eq!(health_of(&phase, 3), 0.0);
        phase.respond(&repeat_start()).unwrap().unwrap();
        assert_eq!(health_of(&phase, 4), 98.0);
        // Nothing alive: nothing to attack.
        phase.hostiles[1].health = 0.0;
        assert!(phase.respond(&repeat_start()).unwrap().is_none());
    }

    #[test]
    fn auto_attack_follows_a_skill_target_without_the_damage_setting() {
        let mut phase = populated_phase()
            .with_hostile_npc(100, [50.0, 0.0, 0.0])
            .unwrap()
            .with_hostile_npc(101, [5.0, 0.0, 0.0])
            .unwrap()
            .with_autoattack_skill(77);
        phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap();
        phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap();
        let order_for = |agent: u32| {
            let order = SkillOrder209026 { order_flags: 0, skill_key: 77, slot: 1, raw_7: 0, agent, byte_12: 0 };
            order.to_behavior_data().to_vec()
        };
        let reply = phase.respond(&repeat_start()).unwrap().unwrap();
        assert!(contains(&reply, &order_for(4)), "nearest NPC first");
        assert!(phase.respond(&skill_at(3)).unwrap().is_none());
        let reply = phase.respond(&repeat_start()).unwrap().unwrap();
        assert!(contains(&reply, &order_for(3)), "then the last SkillBegin target");
    }

    #[test]
    fn counter_attacker_is_the_last_hit_living_npc() {
        let mut phase = hostile_phase(59.0).with_npc_attack(10.0, 100.0);
        phase.respond(&skill_at(3)).unwrap().unwrap();
        phase.respond(&skill_at(4)).unwrap().unwrap();
        assert_eq!(phase.aggro, [3, 4]);
        let due = phase.npc_attack.unwrap().next.expect("attack scheduled");
        phase.take_due(due).unwrap();
        assert_eq!(phase.npc_attack.unwrap().player_health, 90.0);
        // Killing the attacker hands the attack to the other hit NPC.
        phase.respond(&skill_at(4)).unwrap().unwrap();
        assert_eq!(phase.aggro, [3]);
        assert!(phase.npc_attack.unwrap().next.is_some(), "NPC 3 keeps attacking");
        // Killing the last hit NPC stops the attacks.
        phase.respond(&skill_at(3)).unwrap().unwrap();
        assert!(phase.aggro.is_empty());
        assert!(phase.npc_attack.unwrap().next.is_none());
        assert!(phase.due_at().is_some_and(|d| phase.hostiles.iter().any(|h| h.respawn == Some(d))));
    }

    #[test]
    fn a_downed_player_rallies_on_a_kill_and_a_defeated_player_is_ignored() {
        let mut phase = hostile_phase(59.0).with_npc_attack(100.0, 100.0);
        phase.respond(&skill_at(3)).unwrap().unwrap();
        phase.respond(&skill_at(4)).unwrap().unwrap();
        // NPC 4 (the attacker) downs the player.
        let due = phase.npc_attack.unwrap().next.unwrap();
        phase.take_due(due).unwrap();
        let attack = phase.npc_attack.unwrap();
        assert_eq!(attack.player_health, 0.0);
        assert!(attack.next.is_none() && attack.defeat_at.is_some());
        // The downed player kills NPC 4 and rallies; NPC 3 is still hit and alive.
        phase.respond(&skill_at(4)).unwrap().unwrap();
        let attack = phase.npc_attack.unwrap();
        assert!(attack.defeat_at.is_none() && !attack.defeated && attack.next.is_none());
        assert_eq!(attack.player_health, 50.0);
        // The next (non-lethal) hit on NPC 3 restarts its attack.
        phase.hostiles[0].health = LEVEL1_NPC_HEALTH;
        phase.respond(&skill_at(3)).unwrap().unwrap();
        assert!(phase.npc_attack.unwrap().next.is_some());
        // Defeat: the player's hits are ignored.
        phase.npc_attack.as_mut().unwrap().defeated = true;
        phase.hostiles[1].health = LEVEL1_NPC_HEALTH;
        assert!(phase.respond(&skill_at(4)).unwrap().is_none());
        assert_eq!(health_of(&phase, 4), LEVEL1_NPC_HEALTH);
    }

    #[test]
    fn heart_area_crossings_and_waypoint_rejoin_use_current_position() {
        // The process-global handoff must not be consumed by parallel tests.
        const CHILD: &str = "GW2_TEST_HEART_AREA_REJOIN_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "mission::tests::heart_area_crossings_and_waypoint_rejoin_use_current_position", "--nocapture"])
                .env(CHILD, "1")
                .output()
                .unwrap();
            assert!(output.status.success(), "isolated heart area lifecycle failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            return;
        }
        assert!(CARRY.lock().unwrap().is_none());
        const OUTSIDE: [f32; 3] = [-3200.0, -25600.0, -3742.0];
        const WAYPOINT: [f32; 3] = [597.3, -30352.1, -1563.2];
        const INSIDE: [f32; 3] = [-500.0, -26500.0, -3800.0];
        let fresh = |spawn, respawn| {
            let mut phase = populated_phase();
            phase.population.as_mut().unwrap().plan.position = spawn;
            phase.with_heart(56, 2).unwrap()
                .with_npc_attack(10.0, 100.0)
                .with_respawn_point(respawn)
                .with_rejoin_route("127.1.0.4:6112".parse().unwrap())
        };
        let enter_world = |phase: &mut MissionPhase| {
            phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap().unwrap();
            phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap().unwrap()
        };
        let moved = |position: [f32; 3]| Seen::Moved { id: 18, position: position.map(|v| v.round() as i32) };
        let wire = |frame: RenownFrame209026| {
            let mut out = BytesMut::new();
            frame.encode_frame(&mut out).unwrap();
            out.freeze()
        };
        let entered = wire(RenownFrame209026::EnterSubRegion { player: 1, heart: 56 });
        let left = wire(RenownFrame209026::LeaveSubRegion { player: 1, heart: 56 });
        let tables = schema::parse(include_str!("../../../../../data/protocol/209026/server-tables.xml")).unwrap();
        let game_tables: Vec<_> = [
            "table-0x14216e940", "table-0x14211be70", "table-0x1422ddbf0",
            "table-0x141923b80", "table-0x1422e03b0",
        ].iter().map(|name| tables.protocols.iter().flat_map(|p| &p.groups)
            .find(|g| g.name == *name).unwrap()).collect();
        // Inspect actual frame boundaries, not byte patterns inside other frames.
        let ids = |bytes: &[u8]| {
            let mut input = bytes;
            let mut ids = Vec::new();
            while !input.is_empty() {
                let id = u16::from_le_bytes([input[0], input[1]]);
                let table = game_tables.iter().find(|t| t.server.iter().any(|m| m.id == id)).unwrap();
                ids.push(dynamic::decode_frame(&table.server, &rules(), &mut input).unwrap().0.id);
            }
            ids
        };
        let heart_ids = |bytes: &[u8]| ids(bytes).into_iter()
            .filter(|id| [651, 1090, 657, 655, 656, 1089, 654, 556].contains(id)).collect::<Vec<_>>();
        let redirect = |phase: &mut MissionPhase| {
            phase.npc_attack.as_mut().unwrap().defeated = true;
            let reply = phase.respond(&Seen::Other(210)).unwrap().unwrap();
            assert_eq!(ids(&reply), [1065]);
        };

        let mut phase = fresh(OUTSIDE, WAYPOINT);
        assert!(phase.respond(&moved(INSIDE)).unwrap().is_none(), "movement before BeginPlaying is ignored");
        let opening = enter_world(&mut phase);
        assert_eq!(heart_ids(&opening), [651, 1090, 657], "outside spawn does not enter the heart");
        assert_eq!(phase.respond(&moved(INSIDE)).unwrap(), Some(entered.clone()));
        assert!(phase.respond(&moved(INSIDE)).unwrap().is_none(), "same-side movement must not repeat 655");
        assert_eq!(phase.respond(&moved(OUTSIDE)).unwrap(), Some(left.clone()));
        assert!(phase.respond(&moved(OUTSIDE)).unwrap().is_none(), "same-side movement must not repeat 656");
        assert_eq!(phase.respond(&moved(INSIDE)).unwrap(), Some(entered.clone()));
        advance_heart(&mut phase.heart, &mut BytesMut::new()).unwrap();

        // Rejoin outside after leaving a connection whose last position was inside.
        // The fresh fixture deliberately starts inside, so only the carried
        // waypoint can determine the correct opening state.
        redirect(&mut phase);
        let mut phase = fresh(INSIDE, INSIDE);
        let opening = enter_world(&mut phase);
        assert!(CARRY.lock().unwrap().is_none());
        assert_eq!(phase.population.as_ref().unwrap().plan.position, WAYPOINT);
        assert!(phase.heart.is_some_and(|h| !h.inside && h.progress == 1 && !h.done && !h.rewarded));
        assert_eq!(heart_ids(&opening), [651, 1090, 657, 1089]);
        assert!(phase.respond(&moved(WAYPOINT)).unwrap().is_none());
        assert_eq!(phase.respond(&moved(INSIDE)).unwrap(), Some(entered.clone()));

        // The reverse rejoin must enter immediately, despite an outside fixture.
        redirect(&mut phase);
        let mut phase = fresh(OUTSIDE, WAYPOINT);
        let opening = enter_world(&mut phase);
        assert!(CARRY.lock().unwrap().is_none());
        assert_eq!(phase.population.as_ref().unwrap().plan.position, INSIDE);
        assert!(phase.heart.is_some_and(|h| h.inside && h.progress == 1 && !h.done && !h.rewarded));
        assert_eq!(heart_ids(&opening), [651, 1090, 657, 655, 1089]);
        assert!(phase.respond(&moved(INSIDE)).unwrap().is_none());
        assert_eq!(phase.respond(&moved(OUTSIDE)).unwrap(), Some(left));
    }

    #[test]
    fn heart_reward_is_issued_once_across_waypoint_rejoins() {
        // CARRY belongs to the observer process. Run this lifecycle in its own
        // process so parallel mission/TCP tests cannot consume its handoff.
        const CHILD: &str = "GW2_TEST_HEART_REWARD_REJOIN_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "mission::tests::heart_reward_is_issued_once_across_waypoint_rejoins", "--nocapture"])
                .env(CHILD, "1")
                .output()
                .unwrap();
            assert!(output.status.success(), "isolated heart lifecycle failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            return;
        }
        assert!(CARRY.lock().unwrap().is_none());
        let fresh = || populated_phase()
            .with_hostile_npc(100, [5.0, 0.0, 0.0]).unwrap()
            .with_damage_on_skill(LEVEL1_NPC_HEALTH)
            .with_xp_per_kill(10)
            .with_npc_attack(10.0, 100.0)
            .with_heart(56, 2).unwrap()
            .with_respawn_point([1.0, 2.0, 3.0])
            .with_rejoin_route("127.1.0.4:6112".parse().unwrap());
        let enter = |phase: &mut MissionPhase| {
            phase.respond(&Seen::Map(ClientMapEvent::MapLoaded)).unwrap().unwrap();
            phase.respond(&Seen::Map(ClientMapEvent::AgentsLoaded)).unwrap().unwrap()
        };
        let tables = schema::parse(include_str!("../../../../../data/protocol/209026/server-tables.xml")).unwrap();
        let game_tables: Vec<_> = [
            "table-0x14216e940", "table-0x14211be70", "table-0x1422ddbf0",
            "table-0x141923b80", "table-0x1422e03b0",
        ].iter().map(|name| tables.protocols.iter().flat_map(|p| &p.groups)
            .find(|g| g.name == *name).unwrap()).collect();
        // Decode boundaries using the frozen game schemas, so a byte pattern
        // inside another frame cannot masquerade as a missing/extra reward.
        let frames = |bytes: &[u8]| {
            let mut input = bytes;
            let mut found = Vec::new();
            while !input.is_empty() {
                let before = input;
                let id = u16::from_le_bytes([input[0], input[1]]);
                let table = game_tables.iter().find(|t| t.server.iter().any(|m| m.id == id))
                    .unwrap_or_else(|| panic!("unregistered server frame {id}"));
                let (message, _) = dynamic::decode_frame(&table.server, &rules(), &mut input).unwrap();
                found.push((message.id, before[..before.len() - input.len()].to_vec()));
            }
            found
        };
        let xp_frames = |totals: &[u32]| totals.iter().map(|&experience| {
            let mut out = BytesMut::new();
            ExperienceFrame209026 { character_index: 1, experience, dword_2: 0, byte_3: 0 }.encode_frame(&mut out);
            (191, out.to_vec())
        }).collect::<Vec<_>>();
        let rejoin = |phase: &mut MissionPhase| {
            phase.npc_attack.as_mut().unwrap().defeated = true;
            let redirect = phase.respond(&Seen::Other(210)).unwrap().unwrap();
            assert_eq!(frames(&redirect).iter().map(|f| f.0).collect::<Vec<_>>(), [1065]);
            let mut next = fresh();
            let opening = frames(&enter(&mut next));
            assert!(CARRY.lock().unwrap().is_none(), "MapLoaded consumes the carry-over once");
            assert_eq!(next.population.as_ref().unwrap().plan.position, [1.0, 2.0, 3.0]);
            assert!(!opening.iter().any(|f| f.0 == 556), "rejoin must not issue a reward panel");
            (next, opening)
        };

        let mut phase = fresh();
        enter(&mut phase);
        let first = frames(&phase.respond(&skill_at(3)).unwrap().unwrap());
        assert!(!first.iter().any(|f| f.0 == 556));
        assert_eq!(first.into_iter().filter(|f| f.0 == 191).collect::<Vec<_>>(), xp_frames(&[10]));

        // Incomplete progress survives, and the first completion still pays.
        let (mut phase, opening) = rejoin(&mut phase);
        assert_eq!(opening.into_iter().filter(|f| f.0 == 191).collect::<Vec<_>>(), xp_frames(&[10]));
        assert!(phase.heart.is_some_and(|h| h.progress == 1 && !h.done));
        let completion = frames(&phase.respond(&skill_at(3)).unwrap().unwrap());
        let mut reward = BytesMut::new();
        RenownFrame209026::HeartReward {
            reward: 1, heart: 56, currencies: vec![(1, HEART_COINS), (2, HEART_KARMA)],
        }.encode_frame(&mut reward).unwrap();
        assert_eq!(completion.iter().filter(|f| f.0 == 556).cloned().collect::<Vec<_>>(), [(556, reward.to_vec())]);
        assert_eq!(completion.into_iter().filter(|f| f.0 == 191).collect::<Vec<_>>(), xp_frames(&[20, 20 + HEART_XP]));

        // A completed, rewarded heart survives another connection. Its next
        // kill retains ordinary kill XP without replaying panel or heart XP.
        let (mut phase, opening) = rejoin(&mut phase);
        assert_eq!(opening.into_iter().filter(|f| f.0 == 191).collect::<Vec<_>>(), xp_frames(&[20 + HEART_XP]));
        assert!(phase.heart.is_some_and(|h| h.progress == 2 && h.done));
        let next_kill = frames(&phase.respond(&skill_at(3)).unwrap().unwrap());
        assert!(!next_kill.iter().any(|f| f.0 == 556), "completed heart reward must not repeat after respawn");
        assert_eq!(next_kill.into_iter().filter(|f| f.0 == 191).collect::<Vec<_>>(), xp_frames(&[30 + HEART_XP]));
        assert_eq!(phase.xp.unwrap().1, 30 + HEART_XP);
    }
}

#[cfg(test)]
mod heart_area_tests {
    use super::*;

    #[test]
    fn heart_56_area_contains_its_centre_but_not_the_spawn_or_waypoints() {
        // The client polygon dips north-east of the spawn: the heart starts
        // outside it, as in retail, and is entered by walking east.
        assert!(!inside_polygon(&HEART_56_POLYGON, -3200.0, -25600.0), "Norn Mesmer spawn");
        assert!(!inside_polygon(&HEART_56_POLYGON, 597.3, -30352.1), "Hero's Moot Waypoint");
        assert!(inside_polygon(&HEART_56_POLYGON, -500.0, -26500.0), "inside near the centre");
        assert!(inside_polygon(&HEART_56_POLYGON, 465.05, -26226.8), "heart centre");
        assert!(!inside_polygon(&HEART_56_POLYGON, -9057.7, -24303.5), "Horncall Waypoint");
    }
}
