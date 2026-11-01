//! Build-209026 server-86 position frames. See
//! `docs/re/209026-agent-position-update.md` for execution gates and bounds.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode, Point3};

/// Explicit transport values, without inferred flags or valid destinations.
/// Float bits are preserved; collision conversion belongs to the consumer.
#[derive(Debug, Clone)]
pub struct AgentPosition209026 {
    pub agent_index: u16,
    pub remote_tick: u32,
    pub command_flags: u8,
    pub position: Point3,
}

impl AgentPosition209026 {
    pub const MESSAGE_ID: u16 = 86;

    /// Append one complete frame.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.agent_index.encode(output);
        self.remote_tick.encode(output);
        self.command_flags.encode(output);
        self.position.encode(output);
    }

    /// Consume one frame, leaving input unchanged on every decoding error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::MESSAGE_ID {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected agent-position message 86",
            ));
        }
        let frame = Self {
            agent_index: u16::decode(&mut cursor)?,
            remote_tick: u32::decode(&mut cursor)?,
            command_flags: u8::decode(&mut cursor)?,
            position: Point3::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixed LE Word/float fields, varint tick 300 and tag 128.
    const WIRE: &[u8] = &[
        0x56, 0, 0x34, 0x12, 0xac, 2, 0xa5, 0, 0, 0x80, 0x3f, 0, 0, 0, 0xc0, 0, 0, 0x60, 0x40,
        0x80, 1,
    ];

    #[test]
    fn position_frame_golden_and_concatenation() {
        let frame = AgentPosition209026 {
            agent_index: 0x1234,
            remote_tick: 300,
            command_flags: 0xa5,
            position: Point3([1.0, -2.0, 3.5], 128),
        };
        let mut output = BytesMut::from(&b"prefix"[..]);
        frame.encode_frame(&mut output);
        assert_eq!(&output[..6], b"prefix");
        assert_eq!(&output[6..], WIRE);
        frame.encode_frame(&mut output);
        output.extend_from_slice(&[0xcc]);
        let mut input = &output[6..];
        for _ in 0..2 {
            let decoded = AgentPosition209026::decode_frame(&mut input).unwrap();
            assert_eq!(decoded.agent_index, frame.agent_index);
            assert_eq!(decoded.remote_tick, frame.remote_tick);
            assert_eq!(decoded.command_flags, frame.command_flags);
            assert_eq!(decoded.position, frame.position);
        }
        assert_eq!(input, &[0xcc]);
    }

    #[test]
    fn position_frame_retains_raw_float_bits_and_unsigned_boundaries() {
        let bits = [0x8000_0000, 0x7fc1_2345, 0xff80_0000];
        let frame = AgentPosition209026 {
            agent_index: u16::MAX,
            remote_tick: u32::MAX,
            command_flags: u8::MAX,
            position: Point3(bits.map(f32::from_bits), u32::MAX),
        };
        let wire = [
            0x56, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff, 0, 0, 0, 0x80, 0x45, 0x23,
            0xc1, 0x7f, 0, 0, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f,
        ];
        let mut output = BytesMut::new();
        frame.encode_frame(&mut output);
        assert_eq!(&output[..], wire);
        let mut input = wire.as_slice();
        let decoded = AgentPosition209026::decode_frame(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(decoded.agent_index, u16::MAX);
        assert_eq!(decoded.remote_tick, u32::MAX);
        assert_eq!(decoded.command_flags, u8::MAX);
        assert_eq!(decoded.position.0.map(f32::to_bits), bits);
        assert_eq!(decoded.position.1, u32::MAX);

        let zero_wire = [0x56, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut input = zero_wire.as_slice();
        let zero = AgentPosition209026::decode_frame(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(zero.agent_index, 0);
        assert_eq!(zero.remote_tick, 0);
        assert_eq!(zero.command_flags, 0);
        assert_eq!(zero.position.0.map(f32::to_bits), [0; 3]);
        assert_eq!(zero.position.1, 0);
        output.clear();
        zero.encode_frame(&mut output);
        assert_eq!(&output[..], zero_wire);
    }

    #[test]
    fn position_frame_errors_are_transactional() {
        fn rejected(bytes: &[u8], kind: ErrorKind) {
            let mut input = bytes;
            assert_eq!(
                AgentPosition209026::decode_frame(&mut input)
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
        wrong_id[0] = 64;
        rejected(&wrong_id, ErrorKind::InvalidData);

        let mut bad_tick = WIRE[..4].to_vec();
        bad_tick.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x10]);
        bad_tick.extend_from_slice(&WIRE[6..]);
        rejected(&bad_tick, ErrorKind::InvalidData);
        let mut bad_tag = WIRE[..19].to_vec();
        bad_tag.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x10]);
        rejected(&bad_tag, ErrorKind::InvalidData);
    }
}
