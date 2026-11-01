//! Build-209026 world/scene-entry transport, documented in
//! `docs/re/209026-agent-world-entry.md` and
//! `docs/re/209026-agent-scene-entry.md`.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode, Point3};

/// Server 46's explicit world-entry inputs. Client world/scene state and
/// option validity are separate from transport encoding.
#[derive(Debug, Clone)]
pub struct AgentWorldEntry209026 {
    pub agent_index: u16,
    pub remote_tick: u32,
    pub command_flags: u8,
    pub collision_flags: u8,
    pub entry_options: u8,
    pub position: Point3,
    /// Signed pitch/yaw words in wire order; preserve 0x7fff sentinels.
    pub facing: [u16; 2],
}

impl AgentWorldEntry209026 {
    pub const MESSAGE_ID: u16 = 46;

    /// Append one frame without interpreting its options or destination.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.agent_index.encode(output);
        self.remote_tick.encode(output);
        self.command_flags.encode(output);
        self.collision_flags.encode(output);
        self.entry_options.encode(output);
        self.position.encode(output);
        self.facing.encode(output);
    }

    /// Consume one frame, leaving input unchanged on every decoding error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::MESSAGE_ID {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected agent-world-entry message 46",
            ));
        }
        let frame = Self {
            agent_index: u16::decode(&mut cursor)?,
            remote_tick: u32::decode(&mut cursor)?,
            command_flags: u8::decode(&mut cursor)?,
            collision_flags: u8::decode(&mut cursor)?,
            entry_options: u8::decode(&mut cursor)?,
            position: Point3::decode(&mut cursor)?,
            facing: <[u16; 2]>::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

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

#[cfg(test)]
mod world_tests {
    use super::*;

    const WIRE: &[u8] = &[
        0x2e, 0, 0x34, 0x12, 0xac, 2, 0xa5, 0x96, 0x3c, 0, 0, 0x80, 0x3f, 0, 0, 0, 0xc0, 0, 0,
        0x60, 0x40, 0x80, 1, 0xff, 0x7f, 0, 0x80,
    ];

    #[test]
    fn world_entry_frame_golden_and_concatenation() {
        let frame = AgentWorldEntry209026 {
            agent_index: 0x1234,
            remote_tick: 300,
            command_flags: 0xa5,
            collision_flags: 0x96,
            entry_options: 0x3c,
            position: Point3([1.0, -2.0, 3.5], 128),
            facing: [0x7fff, 0x8000],
        };
        let mut output = BytesMut::from(&b"prefix"[..]);
        frame.encode_frame(&mut output);
        assert_eq!(&output[..6], b"prefix");
        assert_eq!(&output[6..], WIRE);
        frame.encode_frame(&mut output);
        output.extend_from_slice(&[0xcc]);
        let mut input = &output[6..];
        for _ in 0..2 {
            let decoded = AgentWorldEntry209026::decode_frame(&mut input).unwrap();
            assert_eq!(decoded.agent_index, frame.agent_index);
            assert_eq!(decoded.remote_tick, frame.remote_tick);
            assert_eq!(decoded.command_flags, frame.command_flags);
            assert_eq!(decoded.collision_flags, frame.collision_flags);
            assert_eq!(decoded.entry_options, frame.entry_options);
            assert_eq!(decoded.position, frame.position);
            assert_eq!(decoded.facing, frame.facing);
        }
        assert_eq!(input, &[0xcc]);
    }

    #[test]
    fn world_entry_frame_preserves_bits_and_unsigned_boundaries() {
        let bits = [0x8000_0000, 0x7fc1_2345, 0xff80_0000];
        let frame = AgentWorldEntry209026 {
            agent_index: u16::MAX,
            remote_tick: u32::MAX,
            command_flags: u8::MAX,
            collision_flags: u8::MAX,
            entry_options: u8::MAX,
            position: Point3(bits.map(f32::from_bits), u32::MAX),
            facing: [u16::MAX; 2],
        };
        let wire = [
            0x2e, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff, 0xff, 0xff, 0, 0, 0, 0x80,
            0x45, 0x23, 0xc1, 0x7f, 0, 0, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff, 0xff,
            0xff, 0xff,
        ];
        let mut output = BytesMut::new();
        frame.encode_frame(&mut output);
        assert_eq!(&output[..], wire);
        assert_eq!(wire.len(), 33);
        let mut input = wire.as_slice();
        let decoded = AgentWorldEntry209026::decode_frame(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(decoded.agent_index, u16::MAX);
        assert_eq!(decoded.remote_tick, u32::MAX);
        assert_eq!(decoded.command_flags, u8::MAX);
        assert_eq!(decoded.collision_flags, u8::MAX);
        assert_eq!(decoded.entry_options, u8::MAX);
        assert_eq!(decoded.position.0.map(f32::to_bits), bits);
        assert_eq!(decoded.position.1, u32::MAX);
        assert_eq!(decoded.facing, [u16::MAX; 2]);

        let mut zero_wire = [0; 25];
        zero_wire[0] = 46;
        let mut input = zero_wire.as_slice();
        let zero = AgentWorldEntry209026::decode_frame(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(zero.agent_index, 0);
        assert_eq!(zero.remote_tick, 0);
        assert_eq!(zero.command_flags, 0);
        assert_eq!(zero.collision_flags, 0);
        assert_eq!(zero.entry_options, 0);
        assert_eq!(zero.position.0.map(f32::to_bits), [0; 3]);
        assert_eq!(zero.position.1, 0);
        assert_eq!(zero.facing, [0; 2]);
        output.clear();
        zero.encode_frame(&mut output);
        assert_eq!(&output[..], zero_wire);
    }

    #[test]
    fn world_entry_frame_errors_are_transactional() {
        fn rejected(bytes: &[u8], kind: ErrorKind) {
            let mut input = bytes;
            assert_eq!(
                AgentWorldEntry209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(input, bytes);
        }
        for end in 0..WIRE.len() {
            rejected(&WIRE[..end], ErrorKind::UnexpectedEof);
        }
        let mut wrong_id = WIRE.to_vec();
        wrong_id[0] = 49;
        rejected(&wrong_id, ErrorKind::InvalidData);
        let mut bad_tick = WIRE[..4].to_vec();
        bad_tick.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x10]);
        bad_tick.extend_from_slice(&WIRE[6..]);
        rejected(&bad_tick, ErrorKind::InvalidData);
        let mut bad_tag = WIRE[..21].to_vec();
        bad_tag.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x10]);
        bad_tag.extend_from_slice(&WIRE[23..]);
        rejected(&bad_tag, ErrorKind::InvalidData);
    }
}
