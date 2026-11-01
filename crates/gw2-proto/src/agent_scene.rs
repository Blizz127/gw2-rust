//! Build-209026 collision scene-entry transport, documented in
//! `docs/re/209026-agent-scene-entry.md`.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// Server 49 with explicit command inputs. This frame does not establish that
/// the agent exists or that its collider has not already entered the scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentSceneEntry209026 {
    pub agent_index: u16,
    pub remote_tick: u32,
    pub command_flags: u8,
}

impl AgentSceneEntry209026 {
    pub const MESSAGE_ID: u16 = 49;

    /// Append one frame without interpreting its lifecycle or timing.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.agent_index.encode(output);
        self.remote_tick.encode(output);
        self.command_flags.encode(output);
    }

    /// Consume one frame, leaving input unchanged on every decoding error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::MESSAGE_ID {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected agent-scene-entry message 49",
            ));
        }
        let frame = Self {
            agent_index: u16::decode(&mut cursor)?,
            remote_tick: u32::decode(&mut cursor)?,
            command_flags: u8::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIRE: &[u8] = &[0x31, 0, 0x34, 0x12, 0xac, 2, 0xa5];

    #[test]
    fn scene_entry_frame_golden_and_boundaries() {
        let cases = [
            (
                AgentSceneEntry209026 {
                    agent_index: 0x1234,
                    remote_tick: 300,
                    command_flags: 0xa5,
                },
                WIRE,
            ),
            (
                AgentSceneEntry209026 {
                    agent_index: 0,
                    remote_tick: 0,
                    command_flags: 0,
                },
                &[0x31, 0, 0, 0, 0, 0][..],
            ),
            (
                AgentSceneEntry209026 {
                    agent_index: u16::MAX,
                    remote_tick: u32::MAX,
                    command_flags: u8::MAX,
                },
                &[0x31, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff][..],
            ),
        ];
        let mut output = BytesMut::from(&b"prefix"[..]);
        for (frame, wire) in cases {
            let offset = output.len();
            frame.encode_frame(&mut output);
            assert_eq!(&output[offset..], wire);
        }
        output.extend_from_slice(&[0xcc]);
        assert_eq!(&output[..6], b"prefix");
        let mut input = &output[6..];
        for (frame, _) in cases {
            assert_eq!(
                AgentSceneEntry209026::decode_frame(&mut input).unwrap(),
                frame
            );
        }
        assert_eq!(input, &[0xcc]);
    }

    #[test]
    fn scene_entry_frame_errors_are_transactional() {
        fn rejected(bytes: &[u8], kind: ErrorKind) {
            let mut input = bytes;
            assert_eq!(
                AgentSceneEntry209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(input, bytes);
        }
        for end in 0..WIRE.len() {
            rejected(&WIRE[..end], ErrorKind::UnexpectedEof);
        }
        // 0x49 is a different transport ID, despite the health behavior opcode.
        rejected(&[0x49, 0, 0, 0, 0, 0], ErrorKind::InvalidData);
        rejected(
            &[0x31, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x10, 0],
            ErrorKind::InvalidData,
        );
    }
}
