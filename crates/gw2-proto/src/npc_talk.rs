//! Build-209026 NPC talk frames: conversation server 756–764 and client
//! 275/276, the NPC talk/interact flags 378/379, and the client-23 interact
//! request payload. See `docs/re/209026-npc-talk-loop.md`.
//!
//! The codec writes and reads the wire layout only. It does not enforce the
//! client's ordering rules (one active conversation; page 760, then its
//! responses, then 761), which the note documents and callers must follow.
//! Fields whose meaning is not established keep ordinal names.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::agent_creation::{AgentCreation209026, facing_words_209026};
use crate::agent_scene::AgentWorldEntry209026;
use crate::character_setup::CharacterSetupFrame209026;
use crate::core_stats::CoreStatsCreation209026;
use crate::health::{HealthFrame209026, HealthOperation209026, HealthValues209026};
use crate::{Decode, Encode, Point3};

/// Schema `String` sizes, in UTF-16 units including the zero terminator.
const LINE_UNITS: usize = 512;
const SPEAKER_UNITS: usize = 64;
const EXTRA_UNITS: usize = 128;

fn invalid_input(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidInput, message)
}

fn invalid_data(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

/// UTF-16 units for a terminated string of schema size `size`.
fn text_units(text: &str, size: usize) -> Result<Vec<u16>> {
    let units: Vec<u16> = text.encode_utf16().collect();
    if units.contains(&0) {
        return Err(invalid_input("conversation text contains a zero unit"));
    }
    if units.len() >= size {
        return Err(invalid_input("conversation text and terminator exceed Size"));
    }
    Ok(units)
}

/// Plain (unencrypted) English text whose whole template is `%str1%`. Found by
/// a scan of every language-0 string pack in the frozen Gw2.dat (80 such IDs;
/// this is the lowest). See `docs/re/209026-npc-talk-loop.md` section 4.
pub const LITERAL_TEMPLATE_TEXT_ID: u32 = 46217;

/// Coded text (the form `TextValidateCoded` accepts) that renders `text`
/// literally: the `%str1%` template, STR1 literal-parameter marker 0x107, the
/// literal units, and the parameter terminator 1. Text IDs are base-0x7f00
/// digits offset by 0x100, bit 15 marking a following digit (`0x1410e53f0`).
pub fn coded_literal(text: &str) -> Result<String> {
    if text.encode_utf16().any(|unit| unit < 0x0a) {
        return Err(invalid_input("literal text contains a control unit below 0x0a"));
    }
    let mut digits = Vec::new();
    let mut id = LITERAL_TEMPLATE_TEXT_ID;
    loop {
        digits.push((id % 0x7f00) as u16);
        id /= 0x7f00;
        if id == 0 {
            break;
        }
    }
    let last = digits.len() - 1;
    let mut units: Vec<u16> = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, digit)| 0x100 + digit + if i < last { 0x8000 } else { 0 })
        .collect();
    units.push(0x107);
    units.extend(text.encode_utf16());
    units.push(1);
    String::from_utf16(&units).map_err(|_| invalid_input("literal text is not valid UTF-16"))
}

fn put_units(units: &[u16], output: &mut BytesMut) {
    units.iter().for_each(|unit| unit.encode(output));
    0u16.encode(output);
}

fn take_text(input: &mut &[u8], size: usize) -> Result<String> {
    let mut units = Vec::new();
    loop {
        if units.len() >= size {
            return Err(invalid_data("conversation text exceeds Size without a terminator"));
        }
        let unit = u16::decode(input)?;
        if unit == 0 {
            break;
        }
        units.push(unit);
    }
    String::from_utf16(&units).map_err(|_| invalid_data("conversation text is not UTF-16"))
}

/// Server-to-client conversation messages (registration table `0x14223a5c0`).
///
/// Every message after `Open` reaches only the client's *active* conversation,
/// matched by `conversation`; most also require `stage` to match the stage set
/// by the latest `Open`/`Page`.
#[derive(Debug, Clone, PartialEq)]
pub enum ConversationFrame209026 {
    /// Message 758: Creates the conversation. If none is active it becomes active at
    /// once; otherwise the client closes the active one (replying 276) and
    /// leaves this one pending, where later messages do not reach it.
    Open {
        /// Stored separately from the key; likely the NPC agent (inferred).
        agent: u32,
        conversation: u32,
        stage: u32,
        byte_3: u8,
        /// Used as the sender name when 759 echoes to chat.
        speaker: String,
    },
    /// Message 760: Starts a page: sets `stage`, clears ready/selection and the
    /// response list, and (unless `flags` bit 0) builds the line UI.
    /// `flags` 0x80 also prints `text` to chat as `speaker`.
    Page {
        conversation: u32,
        stage: u32,
        /// Client countdown; values over 250 are reduced by 250.
        timer_ms: u32,
        /// Not read by the build-209026 handler.
        float_3: f32,
        byte_4: u8,
        flags: u32,
        dword_6: u32,
        dword_7: u32,
        dword_8: u32,
        byte_9: u8,
        /// Not read by the build-209026 handler.
        dword_10: u32,
        dword_11: u32,
        text: String,
        speaker: String,
        extra: String,
    },
    /// Message 756: Appends a response option to the current page.
    AddResponse {
        conversation: u32,
        stage: u32,
        /// Echoed by client 275 when chosen.
        response: u32,
        /// Nonzero values are resolved as a content definition (inferred cost).
        definition: u32,
        dword_4: u32,
        dword_5: u32,
        dword_6: u32,
        byte_7: u8,
        text: String,
    },
    /// Message 757: Response option with client defaults for the omitted fields.
    AddShortResponse {
        conversation: u32,
        stage: u32,
        response: u32,
        definition: u32,
        dword_4: u32,
        byte_5: u8,
        text: String,
    },
    /// Message 759: Records the chosen/previous response text; `flags` 0x200 echoes it
    /// to chat. Checks `stage` only.
    ChosenResponse {
        conversation: u32,
        stage: u32,
        dword_2: u32,
        response: u32,
        dword_4: u32,
        dword_5: u32,
        text: String,
        flags: u32,
    },
    /// Message 761: Marks the page ready once its responses have been sent.
    PageReady { conversation: u32, stage: u32 },
    /// Message 762: Closes the conversation; the client answers with 276.
    Close { conversation: u32 },
    /// Message 763: Passes `value` to one response (inferred vote/enabled state).
    UpdateResponse {
        conversation: u32,
        stage: u32,
        response: u32,
        value: u32,
    },
    /// Message 764: Sets the selected response.
    SetSelection {
        conversation: u32,
        stage: u32,
        response: u32,
    },
}

impl ConversationFrame209026 {
    pub const MESSAGE_IDS: [u16; 9] = [756, 757, 758, 759, 760, 761, 762, 763, 764];

    pub fn message_id(&self) -> u16 {
        use ConversationFrame209026::*;
        match self {
            AddResponse { .. } => 756,
            AddShortResponse { .. } => 757,
            Open { .. } => 758,
            ChosenResponse { .. } => 759,
            Page { .. } => 760,
            PageReady { .. } => 761,
            Close { .. } => 762,
            UpdateResponse { .. } => 763,
            SetSelection { .. } => 764,
        }
    }

    /// Append one frame. Output is unchanged if any text is too long or
    /// contains a zero unit.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        use ConversationFrame209026::*;
        // Validate every string before writing anything.
        let texts: Vec<Vec<u16>> = match self {
            Open { speaker, .. } => vec![text_units(speaker, SPEAKER_UNITS)?],
            Page {
                text,
                speaker,
                extra,
                ..
            } => vec![
                text_units(text, LINE_UNITS)?,
                text_units(speaker, SPEAKER_UNITS)?,
                text_units(extra, EXTRA_UNITS)?,
            ],
            AddResponse { text, .. } | AddShortResponse { text, .. } | ChosenResponse { text, .. } => {
                vec![text_units(text, LINE_UNITS)?]
            }
            PageReady { .. } | Close { .. } | UpdateResponse { .. } | SetSelection { .. } => Vec::new(),
        };
        self.message_id().encode(output);
        match *self {
            Open {
                agent,
                conversation,
                stage,
                byte_3,
                ..
            } => {
                agent.encode(output);
                conversation.encode(output);
                stage.encode(output);
                byte_3.encode(output);
                put_units(&texts[0], output);
            }
            Page {
                conversation,
                stage,
                timer_ms,
                float_3,
                byte_4,
                flags,
                dword_6,
                dword_7,
                dword_8,
                byte_9,
                dword_10,
                dword_11,
                ..
            } => {
                conversation.encode(output);
                stage.encode(output);
                timer_ms.encode(output);
                float_3.encode(output);
                byte_4.encode(output);
                flags.encode(output);
                dword_6.encode(output);
                dword_7.encode(output);
                dword_8.encode(output);
                byte_9.encode(output);
                dword_10.encode(output);
                dword_11.encode(output);
                texts.iter().for_each(|units| put_units(units, output));
            }
            AddResponse {
                conversation,
                stage,
                response,
                definition,
                dword_4,
                dword_5,
                dword_6,
                byte_7,
                ..
            } => {
                conversation.encode(output);
                stage.encode(output);
                response.encode(output);
                definition.encode(output);
                dword_4.encode(output);
                dword_5.encode(output);
                dword_6.encode(output);
                byte_7.encode(output);
                put_units(&texts[0], output);
            }
            AddShortResponse {
                conversation,
                stage,
                response,
                definition,
                dword_4,
                byte_5,
                ..
            } => {
                conversation.encode(output);
                stage.encode(output);
                response.encode(output);
                definition.encode(output);
                dword_4.encode(output);
                byte_5.encode(output);
                put_units(&texts[0], output);
            }
            ChosenResponse {
                conversation,
                stage,
                dword_2,
                response,
                dword_4,
                dword_5,
                flags,
                ..
            } => {
                conversation.encode(output);
                stage.encode(output);
                dword_2.encode(output);
                response.encode(output);
                dword_4.encode(output);
                dword_5.encode(output);
                put_units(&texts[0], output);
                flags.encode(output);
            }
            PageReady { conversation, stage } => {
                conversation.encode(output);
                stage.encode(output);
            }
            Close { conversation } => conversation.encode(output),
            UpdateResponse {
                conversation,
                stage,
                response,
                value,
            } => {
                conversation.encode(output);
                stage.encode(output);
                response.encode(output);
                value.encode(output);
            }
            SetSelection {
                conversation,
                stage,
                response,
            } => {
                conversation.encode(output);
                stage.encode(output);
                response.encode(output);
            }
        }
        Ok(())
    }

    /// Consume one complete frame; input is unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use ConversationFrame209026::*;
        let mut c = *input;
        let cursor = &mut c;
        let frame = match u16::decode(cursor)? {
            756 => AddResponse {
                conversation: u32::decode(cursor)?,
                stage: u32::decode(cursor)?,
                response: u32::decode(cursor)?,
                definition: u32::decode(cursor)?,
                dword_4: u32::decode(cursor)?,
                dword_5: u32::decode(cursor)?,
                dword_6: u32::decode(cursor)?,
                byte_7: u8::decode(cursor)?,
                text: take_text(cursor, LINE_UNITS)?,
            },
            757 => AddShortResponse {
                conversation: u32::decode(cursor)?,
                stage: u32::decode(cursor)?,
                response: u32::decode(cursor)?,
                definition: u32::decode(cursor)?,
                dword_4: u32::decode(cursor)?,
                byte_5: u8::decode(cursor)?,
                text: take_text(cursor, LINE_UNITS)?,
            },
            758 => Open {
                agent: u32::decode(cursor)?,
                conversation: u32::decode(cursor)?,
                stage: u32::decode(cursor)?,
                byte_3: u8::decode(cursor)?,
                speaker: take_text(cursor, SPEAKER_UNITS)?,
            },
            759 => ChosenResponse {
                conversation: u32::decode(cursor)?,
                stage: u32::decode(cursor)?,
                dword_2: u32::decode(cursor)?,
                response: u32::decode(cursor)?,
                dword_4: u32::decode(cursor)?,
                dword_5: u32::decode(cursor)?,
                text: take_text(cursor, LINE_UNITS)?,
                flags: u32::decode(cursor)?,
            },
            760 => Page {
                conversation: u32::decode(cursor)?,
                stage: u32::decode(cursor)?,
                timer_ms: u32::decode(cursor)?,
                float_3: f32::decode(cursor)?,
                byte_4: u8::decode(cursor)?,
                flags: u32::decode(cursor)?,
                dword_6: u32::decode(cursor)?,
                dword_7: u32::decode(cursor)?,
                dword_8: u32::decode(cursor)?,
                byte_9: u8::decode(cursor)?,
                dword_10: u32::decode(cursor)?,
                dword_11: u32::decode(cursor)?,
                text: take_text(cursor, LINE_UNITS)?,
                speaker: take_text(cursor, SPEAKER_UNITS)?,
                extra: take_text(cursor, EXTRA_UNITS)?,
            },
            761 => PageReady {
                conversation: u32::decode(cursor)?,
                stage: u32::decode(cursor)?,
            },
            762 => Close {
                conversation: u32::decode(cursor)?,
            },
            763 => UpdateResponse {
                conversation: u32::decode(cursor)?,
                stage: u32::decode(cursor)?,
                response: u32::decode(cursor)?,
                value: u32::decode(cursor)?,
            },
            764 => SetSelection {
                conversation: u32::decode(cursor)?,
                stage: u32::decode(cursor)?,
                response: u32::decode(cursor)?,
            },
            _ => return Err(invalid_data("expected conversation message 756..=764")),
        };
        *input = c;
        Ok(frame)
    }
}

/// Client-to-server conversation messages (registration table `0x14223a5b0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationReply209026 {
    /// Message 275: The player chose `response` (the 756/757 response id).
    Choose { conversation: u32, response: u32 },
    /// Message 276: The client tore down its active conversation. It is also the
    /// client's reply to server 762, so it is an acknowledgement, not a request.
    Closed { conversation: u32 },
}

impl ConversationReply209026 {
    pub fn encode_frame(&self, output: &mut BytesMut) {
        match *self {
            Self::Choose {
                conversation,
                response,
            } => {
                275u16.encode(output);
                conversation.encode(output);
                response.encode(output);
            }
            Self::Closed { conversation } => {
                276u16.encode(output);
                conversation.encode(output);
            }
        }
    }

    /// Consume one complete frame; input is unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        let frame = match u16::decode(&mut cursor)? {
            275 => Self::Choose {
                conversation: u32::decode(&mut cursor)?,
                response: u32::decode(&mut cursor)?,
            },
            276 => Self::Closed {
                conversation: u32::decode(&mut cursor)?,
            },
            _ => return Err(invalid_data("expected conversation reply 275 or 276")),
        };
        *input = cursor;
        Ok(frame)
    }
}

/// NPC flag messages that make an `IsMonster` character selectable for
/// interaction (character flag bits 13/14). Send only after server 37 has
/// created the character; the setters assert `IsMonster()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcInteractFlag209026 {
    /// Message 378 (Dword resolved by the same character lookup as 151): nonzero `talk` sets character flag bit 13; `byte_2` sets bit 20.
    Talk { character_index: u32, talk: u8, byte_2: u8 },
    /// Message 379: Nonzero `interact` sets character flag bit 14.
    Interact { character_index: u32, interact: u8 },
}

impl NpcInteractFlag209026 {
    pub fn encode_frame(&self, output: &mut BytesMut) {
        match *self {
            Self::Talk {
                character_index,
                talk,
                byte_2,
            } => {
                378u16.encode(output);
                character_index.encode(output);
                talk.encode(output);
                byte_2.encode(output);
            }
            Self::Interact { character_index, interact } => {
                379u16.encode(output);
                character_index.encode(output);
                interact.encode(output);
            }
        }
    }

    /// Consume one complete frame; input is unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        let frame = match u16::decode(&mut cursor)? {
            378 => Self::Talk {
                character_index: u32::decode(&mut cursor)?,
                talk: u8::decode(&mut cursor)?,
                byte_2: u8::decode(&mut cursor)?,
            },
            379 => Self::Interact {
                character_index: u32::decode(&mut cursor)?,
                interact: u8::decode(&mut cursor)?,
            },
            _ => return Err(invalid_data("expected NPC flag message 378 or 379")),
        };
        *input = cursor;
        Ok(frame)
    }
}

/// The packed client-23 behavior data sent by order producer `0x141282d90`
/// (order vtable `0x142179b90` +0x148): opcode then a little-endian target
/// id: the target agent's +0xc identifier. Which server-side id (server 37
/// agent index or character index) it equals is unverified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractRequest209026 {
    /// Opcode 0x14: ordinary interaction (an NPC character always takes this).
    Interact { target: u32 },
    /// Opcode 0x15: a gadget whose interaction is channelled.
    Channel { target: u32 },
}

impl InteractRequest209026 {
    pub const INTERACT_OPCODE: u8 = 0x14;
    pub const CHANNEL_OPCODE: u8 = 0x15;

    pub fn to_behavior_data(&self) -> [u8; 5] {
        let (opcode, target) = match *self {
            Self::Interact { target } => (Self::INTERACT_OPCODE, target),
            Self::Channel { target } => (Self::CHANNEL_OPCODE, target),
        };
        let mut data = [opcode, 0, 0, 0, 0];
        data[1..].copy_from_slice(&target.to_le_bytes());
        data
    }

    /// Recognise a complete interact payload. Other opcodes (skills, pickup,
    /// …) and any other length are rejected so callers can keep them raw.
    pub fn from_behavior_data(data: &[u8]) -> Result<Self> {
        let [opcode, a, b, c, d] = *data else {
            return Err(invalid_data("interact behavior data is not 5 bytes"));
        };
        let target = u32::from_le_bytes([a, b, c, d]);
        match opcode {
            Self::INTERACT_OPCODE => Ok(Self::Interact { target }),
            Self::CHANNEL_OPCODE => Ok(Self::Channel { target }),
            _ => Err(invalid_data("behavior opcode is not interact 0x14/0x15")),
        }
    }
}

/// An explicitly chosen test NPC: an ordinary nonplayer character
/// (`docs/re/209026-npc-character-foundation.md`) made talkable by 378.
///
/// No value here is a retail spawn. The species key must exist in content
/// type 67 or the client's species lookup fails; no tutorial species key is
/// established (`docs/re/209026-great-hunt-species-link-audit.md`).
#[derive(Debug, Clone, PartialEq)]
pub struct TalkNpcPlan209026 {
    /// Server-37 Word agent index; also the Dword character index of
    /// 151/256/170/378. Must differ from every other agent.
    pub agent_index: u16,
    pub species_key: u32,
    /// Map units, transform tag zero (same space as the player's spawn).
    pub position: [f32; 3],
    /// Unit map-space facing.
    pub facing: [f32; 3],
    /// Server-151 attitude toward the player: 0 friendly / 3 neutral pass the
    /// interaction gate's talk test; 1/2 are the attackable pair (inferred
    /// labels: hostile/indifferent). Must be below 4.
    pub attitude: u32,
    /// Send 378 {index, 1, 0} (talk flag, bit 13). Off for enemies.
    pub talkable: bool,
}

impl TalkNpcPlan209026 {
    /// Nonplayer identity: high nibble 2 plus a 28-bit species key.
    pub fn identity(&self) -> Result<u32> {
        if self.species_key == 0 || self.species_key >= 0x1000_0000 {
            return Err(invalid_input("species key must be nonzero and below 0x10000000"));
        }
        Ok(0x2000_0000 + self.species_key)
    }

    /// 37 create, 183 coreStats, 282 inventory, 151 attitude, 256 health,
    /// 170 finalize, 46 world entry, then 378 talk flag. Send after the player
    /// is populated and in the world.
    pub fn spawn_frames(&self) -> Result<BytesMut> {
        let identity = self.identity()?;
        if self.attitude >= 4 {
            return Err(invalid_input("attitude must be below 4"));
        }
        if !self.position.iter().all(|v| v.is_finite()) {
            return Err(invalid_input("NPC position must be finite"));
        }
        let facing = facing_words_209026(self.facing)
            .ok_or_else(|| invalid_input("NPC facing must be a finite unit vector"))?;
        let character_index = u32::from(self.agent_index);
        let position = Point3(self.position, 0);
        let mut out = BytesMut::new();
        AgentCreation209026 {
            agent_index: self.agent_index,
            remote_tick: 0, // placeholder, as in the player population
            command_flags: 0,
            behavior_value: 0,
            behavior_data: Vec::new(),
            agent_type: 0, // character factory
            identity,
            // As the player: field 8 = 1 satisfies the full factory's F & 1
            // assertion (docs/re/209026-agent-creation-flags.md).
            creation_flags: [0, 1, 0],
            position,
            facing,
            // Speed and turn 0 (the NPC does not move); height 16 stays above
            // both minimums for radius 1 (docs/re/209026-agent-dimensions.md).
            // placeholder: no NPC values are established from content.
            parameters: [0.0, 0.0, 0.0, 16.0, 0.0, 0.0, 0.0, 0.0],
            radius: 1.0,
        }
        .encode_frame(&mut out)?;
        // coreStats: without it a character consumer asserts `m_coreStats`
        // (ChCliCharacter.h:565) as the NPC enters the world (sandbox run 121).
        // Levels 1..=100 are the nonplayer bounds
        // (docs/re/209026-character-core-stats.md). placeholder: level,
        // appearance and profession scalar are not established for NPCs.
        CoreStatsCreation209026 {
            character_index,
            appearance: [0; 45],
            earned_level: 1,
            level_2: 1,
            profession: 0,
            extended: None,
        }
        .encode_frame(&mut out);
        // With coreStats present, 170 requires inventory
        // (docs/re/209026-character-finalization-dependencies.md).
        CharacterSetupFrame209026::CreateInventory { character_index }.encode_frame(&mut out);
        // 170 requires attitude below 4; 0 (friendly, inferred) also passes
        // the interaction gate's friendly-or-neutral test.
        CharacterSetupFrame209026::Attitude {
            character_index,
            attitude: self.attitude,
        }
        .encode_frame(&mut out);
        // Alive: the gate needs alive (or flag bit 49). placeholder meter.
        HealthFrame209026 {
            character_index,
            operation: HealthOperation209026::CreateOrUpdate {
                state: 0,
                values: HealthValues209026 {
                    current: 100.0,
                    maximum: 100.0,
                    second_current: 0.0,
                    second_maximum: 0.0,
                    extra: 0.0,
                },
                rate: 0.0,
            },
        }
        .encode_frame(&mut out);
        CharacterSetupFrame209026::Finalize { character_index }.encode_frame(&mut out);
        AgentWorldEntry209026 {
            agent_index: self.agent_index,
            remote_tick: 0,
            // No client control or action slot: the NPC is not controlled.
            command_flags: 0,
            collision_flags: 0,
            // Bit 1 clear so the collision scene entry (REVEALED) is reached.
            entry_options: 0,
            position,
            facing,
        }
        .encode_frame(&mut out);
        if self.talkable {
            NpcInteractFlag209026::Talk {
                character_index,
                talk: 1,
                byte_2: 0,
            }
            .encode_frame(&mut out);
        }
        Ok(out)
    }
}

/// Server side of one NPC's conversation, following the client rules in
/// `docs/re/209026-npc-talk-loop.md`: one active conversation, each page sent
/// as 760, its responses, then 761, and 276 treated as an acknowledgement.
///
/// Dialogue text is caller-supplied test text; retail lines are server data
/// the client does not hold.
#[derive(Debug, Clone)]
pub struct TalkSession209026 {
    npc_agent: u32,
    speaker: String,
    pages: Vec<String>,
    next_conversation: u32,
    /// (conversation, stage) of the conversation the client has active.
    active: Option<(u32, u32)>,
}

impl TalkSession209026 {
    /// Response id that advances to the next page.
    pub const CONTINUE: u32 = 1;
    /// Response id that closes the conversation.
    pub const GOODBYE: u32 = 2;

    pub fn new(npc_agent: u32, speaker: String, pages: Vec<String>) -> Result<Self> {
        if pages.is_empty() {
            return Err(invalid_input("a talk session needs at least one page"));
        }
        // The client only renders coded text (sandbox run 123 showed blank
        // fields for plain UTF-16), so every string is stored coded.
        let speaker = coded_literal(&speaker)?;
        text_units(&speaker, SPEAKER_UNITS)?;
        let pages = pages.iter().map(|page| coded_literal(page)).collect::<Result<Vec<_>>>()?;
        for page in &pages {
            text_units(page, LINE_UNITS)?;
        }
        Ok(Self {
            npc_agent,
            speaker,
            pages,
            next_conversation: 1,
            active: None,
        })
    }

    pub fn active(&self) -> Option<(u32, u32)> {
        self.active
    }

    /// Frames for an interact request on `target`, or `None` when the target
    /// is not this NPC. A still-active conversation is closed first (762), so
    /// the new 758 is promoted immediately instead of left pending.
    pub fn on_interact(&mut self, target: u32) -> Result<Option<BytesMut>> {
        if target != self.npc_agent {
            return Ok(None);
        }
        let mut out = BytesMut::new();
        if let Some((conversation, _)) = self.active.take() {
            ConversationFrame209026::Close { conversation }.encode_frame(&mut out)?;
        }
        let conversation = self.next_conversation;
        self.next_conversation = self.next_conversation.wrapping_add(1).max(1);
        ConversationFrame209026::Open {
            agent: self.npc_agent,
            conversation,
            stage: 1,
            byte_3: 0,
            speaker: self.speaker.clone(),
        }
        .encode_frame(&mut out)?;
        self.page(conversation, 1, &mut out)?;
        Ok(Some(out))
    }

    /// Frames answering client 275/276, if any.
    pub fn on_reply(&mut self, reply: ConversationReply209026) -> Result<Option<BytesMut>> {
        match reply {
            ConversationReply209026::Closed { conversation } => {
                if self.active.is_some_and(|(active, _)| active == conversation) {
                    self.active = None;
                }
                Ok(None)
            }
            ConversationReply209026::Choose {
                conversation,
                response,
            } => {
                let Some((active, stage)) = self.active else { return Ok(None) };
                if active != conversation {
                    return Ok(None);
                }
                let mut out = BytesMut::new();
                if response == Self::CONTINUE && (stage as usize) < self.pages.len() {
                    self.page(conversation, stage + 1, &mut out)?;
                } else {
                    // Goodbye, or anything else: close. The client acks with 276.
                    self.active = None;
                    ConversationFrame209026::Close { conversation }.encode_frame(&mut out)?;
                }
                Ok(Some(out))
            }
        }
    }

    fn page(&mut self, conversation: u32, stage: u32, out: &mut BytesMut) -> Result<()> {
        let text = self.pages[stage as usize - 1].clone();
        ConversationFrame209026::Page {
            conversation,
            stage,
            timer_ms: 0,
            float_3: 0.0,
            byte_4: 0,
            // No 0x80 chat echo: the chat path requires coded text and asserts
            // `TextValidateCoded(codedName)` (CtApi.cpp:141) on a plain speaker
            // name (sandbox run 122). Coded dialogue text is open.
            flags: 0,
            dword_6: 0,
            dword_7: 0,
            dword_8: 0,
            byte_9: 0,
            dword_10: 0,
            dword_11: 0,
            text,
            speaker: self.speaker.clone(),
            extra: String::new(),
        }
        .encode_frame(out)?;
        if (stage as usize) < self.pages.len() {
            ConversationFrame209026::AddShortResponse {
                conversation,
                stage,
                response: Self::CONTINUE,
                definition: 0,
                dword_4: 0,
                byte_5: 0,
                text: coded_literal("Continue.")?,
            }
            .encode_frame(out)?;
        }
        ConversationFrame209026::AddShortResponse {
            conversation,
            stage,
            response: Self::GOODBYE,
            definition: 0,
            dword_4: 0,
            byte_5: 0,
            text: coded_literal("Goodbye.")?,
        }
        .encode_frame(out)?;
        ConversationFrame209026::PageReady { conversation, stage }.encode_frame(out)?;
        self.active = Some((conversation, stage));
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, CountPrefix, Rules, StringEncoding, Value};
    use crate::agent_creation::AgentCreation209026;
    use crate::schema;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
    }

    fn page() -> ConversationFrame209026 {
        ConversationFrame209026::Page {
            conversation: 300,
            stage: 1,
            timer_ms: 0,
            float_3: 0.0,
            byte_4: 0,
            flags: 0x80,
            dword_6: 0,
            dword_7: 0,
            dword_8: 0,
            byte_9: 0,
            dword_10: 0,
            dword_11: 0,
            text: "Hi".into(),
            speaker: "Eir".into(),
            extra: String::new(),
        }
    }

    fn all_frames() -> Vec<ConversationFrame209026> {
        use ConversationFrame209026::*;
        vec![
            Open {
                agent: 7,
                conversation: 300,
                stage: 1,
                byte_3: 0,
                speaker: "Eir".into(),
            },
            page(),
            AddResponse {
                conversation: 300,
                stage: 1,
                response: 2,
                definition: 0,
                dword_4: 1,
                dword_5: 2,
                dword_6: 3,
                byte_7: 4,
                text: "Tell me more.".into(),
            },
            AddShortResponse {
                conversation: 300,
                stage: 1,
                response: 3,
                definition: 0,
                dword_4: 0,
                byte_5: 0,
                text: "Goodbye.".into(),
            },
            ChosenResponse {
                conversation: 300,
                stage: 1,
                dword_2: 0,
                response: 2,
                dword_4: 0,
                dword_5: 0,
                text: "Tell me more.".into(),
                flags: 0x200,
            },
            PageReady {
                conversation: 300,
                stage: 1,
            },
            Close { conversation: 300 },
            UpdateResponse {
                conversation: 300,
                stage: 1,
                response: 2,
                value: 9,
            },
            SetSelection {
                conversation: 300,
                stage: 1,
                response: 2,
            },
        ]
    }

    #[test]
    fn golden_open_page_and_close_bytes() {
        let mut out = BytesMut::new();
        all_frames()[0].encode_frame(&mut out).unwrap();
        assert_eq!(hex(&out), "f6 02 07 ac 02 01 00 45 00 69 00 72 00 00 00");
        out.clear();
        page().encode_frame(&mut out).unwrap();
        assert_eq!(
            hex(&out),
            "f8 02 ac 02 01 00 00 00 00 00 00 80 01 00 00 00 00 00 00 \
             48 00 69 00 00 00 45 00 69 00 72 00 00 00 00 00"
        );
        out.clear();
        ConversationFrame209026::Close { conversation: 300 }
            .encode_frame(&mut out)
            .unwrap();
        assert_eq!(hex(&out), "fa 02 ac 02");
    }

    #[test]
    fn every_frame_round_trips_and_concatenates() {
        let mut stream = BytesMut::new();
        for frame in all_frames() {
            frame.encode_frame(&mut stream).unwrap();
        }
        let mut input = &stream[..];
        for frame in all_frames() {
            assert_eq!(ConversationFrame209026::decode_frame(&mut input).unwrap(), frame);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn frames_match_the_client_schema() {
        let server = schema::parse(include_str!("../../../data/protocol/209026/server-tables.xml")).unwrap();
        let client = schema::parse(include_str!("../../../data/protocol/209026/client-tables.xml")).unwrap();
        let group = |s: schema::Schema, name: &str, server_side: bool| -> Vec<schema::Message> {
            s.protocols
                .into_iter()
                .filter(|p| p.name == "0")
                .flat_map(|p| p.groups)
                .filter(|g| g.name == name)
                .flat_map(|g| if server_side { g.server } else { g.client })
                .collect()
        };
        let mut server_messages = group(server.clone(), "table-0x14223a5c0", true);
        server_messages.extend(group(server, "table-0x14216e940", true));
        let client_messages = group(client, "ctable-0x14223a5b0", false);
        let rules = Rules {
            string: Some(StringEncoding::Utf16Terminated),
            array_var_small: Some(CountPrefix::U8),
            buffer_var_small: Some(CountPrefix::U8),
            ..Rules::default()
        };
        let check = |messages: &[schema::Message], bytes: &[u8]| {
            let mut input = bytes;
            let (message, value) = dynamic::decode_frame(messages, &rules, &mut input).unwrap();
            assert!(input.is_empty(), "message {} left bytes", message.id);
            (message.id, value)
        };

        for frame in all_frames() {
            let mut out = BytesMut::new();
            frame.encode_frame(&mut out).unwrap();
            let (id, _) = check(&server_messages, &out);
            assert_eq!(id, frame.message_id());
        }
        let (_, value) = check(&server_messages, &{
            let mut out = BytesMut::new();
            page().encode_frame(&mut out).unwrap();
            out
        });
        let Value::Struct(fields) = value else { panic!("struct") };
        assert_eq!(fields[12].1, Value::Text("Hi".into()));
        assert_eq!(fields[13].1, Value::Text("Eir".into()));

        for flag in [
            NpcInteractFlag209026::Talk {
                character_index: 7,
                talk: 1,
                byte_2: 0,
            },
            NpcInteractFlag209026::Interact {
                character_index: 7,
                interact: 1,
            },
        ] {
            let mut out = BytesMut::new();
            flag.encode_frame(&mut out);
            check(&server_messages, &out);
            assert_eq!(NpcInteractFlag209026::decode_frame(&mut &out[..]).unwrap(), flag);
        }
        for reply in [
            ConversationReply209026::Choose {
                conversation: 300,
                response: 2,
            },
            ConversationReply209026::Closed { conversation: 300 },
        ] {
            let mut out = BytesMut::new();
            reply.encode_frame(&mut out);
            check(&client_messages, &out);
            assert_eq!(ConversationReply209026::decode_frame(&mut &out[..]).unwrap(), reply);
        }
    }

    #[test]
    fn oversized_or_zero_text_leaves_output_unchanged() {
        let mut out = BytesMut::from(&b"keep"[..]);
        let long = ConversationFrame209026::Open {
            agent: 1,
            conversation: 1,
            stage: 0,
            byte_3: 0,
            speaker: "x".repeat(64),
        };
        assert!(long.encode_frame(&mut out).is_err());
        let ConversationFrame209026::Open { speaker, .. } = &long else { unreachable!() };
        let fits = ConversationFrame209026::Open {
            agent: 1,
            conversation: 1,
            stage: 0,
            byte_3: 0,
            speaker: speaker[..63].to_string(),
        };
        let mut page = page();
        if let ConversationFrame209026::Page { extra, .. } = &mut page {
            *extra = "a\0b".into();
        }
        assert!(page.encode_frame(&mut out).is_err());
        assert_eq!(&out[..], b"keep");
        fits.encode_frame(&mut out).unwrap();
    }

    #[test]
    fn truncated_or_foreign_input_is_left_unchanged() {
        let mut out = BytesMut::new();
        page().encode_frame(&mut out).unwrap();
        for end in 0..out.len() {
            let mut input = &out[..end];
            assert!(ConversationFrame209026::decode_frame(&mut input).is_err());
            assert_eq!(input.len(), end);
        }
        let mut foreign: &[u8] = &[0x97, 0x00, 0x01, 0x00];
        assert!(ConversationFrame209026::decode_frame(&mut foreign).is_err());
        assert!(ConversationReply209026::decode_frame(&mut foreign).is_err());
        assert!(NpcInteractFlag209026::decode_frame(&mut foreign).is_err());
        assert_eq!(foreign.len(), 4);
    }

    #[test]
    fn interact_payload_is_opcode_then_little_endian_target() {
        let request = InteractRequest209026::Interact { target: 0x0102_0304 };
        let data = request.to_behavior_data();
        assert_eq!(data, [0x14, 0x04, 0x03, 0x02, 0x01]);
        assert_eq!(InteractRequest209026::from_behavior_data(&data).unwrap(), request);
        assert_eq!(
            InteractRequest209026::from_behavior_data(&[0x15, 7, 0, 0, 0]).unwrap(),
            InteractRequest209026::Channel { target: 7 }
        );
        // The 10-byte skill request (0x23) and other lengths stay unrecognised.
        assert!(InteractRequest209026::from_behavior_data(&[0x23, 0, 0, 0, 0, 0, 0, 0, 0, 0]).is_err());
        assert!(InteractRequest209026::from_behavior_data(&[0x14, 1, 0, 0]).is_err());
        assert!(InteractRequest209026::from_behavior_data(&[0x1f, 1, 0, 0, 0]).is_err());
    }

    fn server_messages() -> Vec<schema::Message> {
        let server = schema::parse(include_str!("../../../data/protocol/209026/server-tables.xml")).unwrap();
        server
            .protocols
            .into_iter()
            .filter(|p| p.name == "0")
            .flat_map(|p| p.groups)
            .filter(|g| ["table-0x14223a5c0", "table-0x14216e940", "table-0x14211be70"].contains(&g.name.as_str()))
            .flat_map(|g| g.server)
            .collect()
    }

    /// Split a server stream through the client schema; return the frame ids.
    fn ids(messages: &[schema::Message], bytes: &[u8]) -> Vec<u16> {
        let rules = Rules {
            string: Some(StringEncoding::Utf16Terminated),
            array_var_small: Some(CountPrefix::U8),
            array_var_large: Some(CountPrefix::U16),
            buffer_var_small: Some(CountPrefix::U8),
            buffer_var_large: Some(CountPrefix::U16),
            buffer_fixed_is_size_bytes: true,
            ..Rules::default()
        };
        let mut input = bytes;
        let mut out = Vec::new();
        while !input.is_empty() {
            out.push(dynamic::decode_frame(messages, &rules, &mut input).unwrap().0.id);
        }
        out
    }

    fn npc() -> TalkNpcPlan209026 {
        TalkNpcPlan209026 {
            agent_index: 2,
            species_key: 3172,
            position: [10.0, 20.0, 30.0],
            facing: [1.0, 0.0, 0.0],
            attitude: 0,
            talkable: true,
        }
    }

    #[test]
    fn npc_spawn_is_create_core_stats_inventory_attitude_health_finalize_enter_then_talk_flag() {
        let messages = server_messages();
        let bytes = npc().spawn_frames().unwrap();
        assert_eq!(ids(&messages, &bytes), [37, 183, 282, 151, 256, 170, 46, 378]);
        let mut input = &bytes[..];
        let created = AgentCreation209026::decode_frame(&mut input).unwrap();
        assert_eq!((created.agent_index, created.agent_type, created.identity), (2, 0, 0x2000_0000 + 3172));
        let tail = &bytes[bytes.len() - 5..];
        assert_eq!(
            NpcInteractFlag209026::decode_frame(&mut &tail[..]).unwrap(),
            NpcInteractFlag209026::Talk { character_index: 2, talk: 1, byte_2: 0 }
        );
        for bad in [0, 0x1000_0000] {
            assert!(TalkNpcPlan209026 { species_key: bad, ..npc() }.spawn_frames().is_err());
        }
        assert!(TalkNpcPlan209026 { facing: [0.0, 0.0, 0.0], ..npc() }.spawn_frames().is_err());
        assert!(TalkNpcPlan209026 { attitude: 4, ..npc() }.spawn_frames().is_err());
        // An enemy: attitude 1 and no talk flag.
        let enemy = TalkNpcPlan209026 { attitude: 1, talkable: false, ..npc() }.spawn_frames().unwrap();
        assert_eq!(ids(&messages, &enemy), [37, 183, 282, 151, 256, 170, 46]);
    }

    #[test]
    fn session_pages_continue_and_goodbye_follow_the_client_ordering() {
        let messages = server_messages();
        let mut session =
            TalkSession209026::new(2, "Eir".into(), vec!["First.".into(), "Second.".into()]).unwrap();
        assert!(session.on_interact(9).unwrap().is_none());

        let open = session.on_interact(2).unwrap().unwrap();
        assert_eq!(ids(&messages, &open), [758, 760, 757, 757, 761]);
        assert_eq!(session.active(), Some((1, 1)));

        let reply = |response| ConversationReply209026::Choose { conversation: 1, response };
        // A stale conversation id is ignored.
        assert!(session
            .on_reply(ConversationReply209026::Choose { conversation: 7, response: 1 })
            .unwrap()
            .is_none());
        let next = session.on_reply(reply(TalkSession209026::CONTINUE)).unwrap().unwrap();
        // Last page: no Continue option.
        assert_eq!(ids(&messages, &next), [760, 757, 761]);
        assert_eq!(session.active(), Some((1, 2)));

        let bye = session.on_reply(reply(TalkSession209026::GOODBYE)).unwrap().unwrap();
        assert_eq!(ids(&messages, &bye), [762]);
        assert_eq!(session.active(), None);
        // The client's 276 ack after our 762 changes nothing.
        assert!(session.on_reply(ConversationReply209026::Closed { conversation: 1 }).unwrap().is_none());
    }

    #[test]
    fn reinteracting_closes_the_active_conversation_first() {
        let messages = server_messages();
        let mut session = TalkSession209026::new(2, "Eir".into(), vec!["Only.".into()]).unwrap();
        session.on_interact(2).unwrap().unwrap();
        let again = session.on_interact(2).unwrap().unwrap();
        assert_eq!(ids(&messages, &again), [762, 758, 760, 757, 761]);
        assert_eq!(session.active(), Some((2, 1)));
        // A client-side close (276) drops the active conversation.
        session.on_reply(ConversationReply209026::Closed { conversation: 2 }).unwrap();
        assert_eq!(session.active(), None);
        assert!(TalkSession209026::new(2, "Eir".into(), Vec::new()).is_err());
        assert!(TalkSession209026::new(2, "x".repeat(64), vec!["a".into()]).is_err());
    }

    #[test]
    fn coded_literal_wraps_text_in_the_plain_str1_template() {
        let units: Vec<u16> = coded_literal("Hi").unwrap().encode_utf16().collect();
        // 46217 = 1 * 0x7f00 + 13705: digits 0x101 (continued) and 0x3689.
        assert_eq!(units, [0x8101, 0x3689, 0x107, 0x48, 0x69, 0x0001]);
        assert!(coded_literal("a\tb").is_err());
        // Still a valid terminated schema String: no zero unit.
        assert!(!units.contains(&0));
    }
}
