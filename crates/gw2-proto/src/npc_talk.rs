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

use crate::{Decode, Encode};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, CountPrefix, Rules, StringEncoding, Value};
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
}
