//! Build-specific engine-agent creation semantics established by client RE.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode, Point3};

/// Build-209026 server message 37, including its frame ID on the wire.
///
/// This preserves framing, not valid world state. All fields are explicit:
/// there are no inferred spawn, identity, flag or dimension defaults. Float
/// bit patterns and sentinel facing words are retained by the frame codec.
/// See `docs/re/209026-agent-creation.md` and its semantic follow-up notes.
#[derive(Debug, Clone)]
pub struct AgentCreation209026 {
    pub agent_index: u16,
    pub remote_tick: u32,
    pub command_flags: u8,
    pub behavior_value: u32,
    pub behavior_data: Vec<u8>,
    pub agent_type: u8,
    pub identity: u32,
    /// Fields 7, 8 and 9 in wire order; meanings depend on the creation path.
    pub creation_flags: [u8; 3],
    pub position: Point3,
    /// Signed pitch/yaw words, in that order; 0x7fff is the sentinel.
    pub facing: [u16; 2],
    /// Field 13, in wire order; not all content semantics are established.
    pub parameters: [f32; 8],
    pub radius: f32,
}

impl AgentCreation209026 {
    pub const MESSAGE_ID: u16 = 37;
    pub const MAX_BEHAVIOR_BYTES: usize = 250;

    /// Append one frame. On an invalid buffer length, leave output unchanged.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if self.behavior_data.len() > Self::MAX_BEHAVIOR_BYTES {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "agent behavior exceeds 250 bytes",
            ));
        }
        Self::MESSAGE_ID.encode(output);
        self.agent_index.encode(output);
        self.remote_tick.encode(output);
        self.command_flags.encode(output);
        self.behavior_value.encode(output);
        (self.behavior_data.len() as u8).encode(output);
        output.extend_from_slice(&self.behavior_data);
        self.agent_type.encode(output);
        self.identity.encode(output);
        self.creation_flags.encode(output);
        self.position.encode(output);
        self.facing.encode(output);
        self.parameters.encode(output);
        self.radius.encode(output);
        Ok(())
    }

    /// Consume one frame, leaving the input slice unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != Self::MESSAGE_ID {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected agent-creation message 37",
            ));
        }
        let agent_index = u16::decode(&mut cursor)?;
        let remote_tick = u32::decode(&mut cursor)?;
        let command_flags = u8::decode(&mut cursor)?;
        let behavior_value = u32::decode(&mut cursor)?;
        let length = u8::decode(&mut cursor)? as usize;
        if length > Self::MAX_BEHAVIOR_BYTES {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "agent behavior exceeds 250 bytes",
            ));
        }
        if cursor.len() < length {
            return Err(ErrorKind::UnexpectedEof.into());
        }
        let behavior_data = cursor[..length].to_vec();
        cursor = &cursor[length..];
        let frame = Self {
            agent_index,
            remote_tick,
            command_flags,
            behavior_value,
            behavior_data,
            agent_type: u8::decode(&mut cursor)?,
            identity: u32::decode(&mut cursor)?,
            creation_flags: <[u8; 3]>::decode(&mut cursor)?,
            position: Point3::decode(&mut cursor)?,
            facing: <[u16; 2]>::decode(&mut cursor)?,
            parameters: <[f32; 8]>::decode(&mut cursor)?,
            radius: f32::decode(&mut cursor)?,
        };
        *input = cursor;
        Ok(frame)
    }
}

/// Decode build-209026 server-37 fields 11/12 into map-space facing.
///
/// The words are signed pitch/yaw scaled by pi/32766. A 0x7fff in either
/// position produces no direction; the client helper returns zero, which
/// fails the subsequent creation-path unit-vector assertion. This function
/// therefore returns `None` for that sentinel instead of inventing a facing.
///
/// Collision facing negates Z without scaling. Host trig need not match the
/// Windows CRT bit for bit. See `docs/re/209026-agent-facing.md`.
pub fn facing_209026(pitch: u16, yaw: u16) -> Option<[f32; 3]> {
    if pitch == 0x7fff || yaw == 0x7fff {
        return None;
    }
    let angle = |word: u16| (word as i16) as f32 * std::f32::consts::PI / 32766.;
    let p = angle(pitch);
    let y = angle(yaw);
    let cp = p.cos();
    let facing = [y.cos() * cp, y.sin() * cp, p.sin()];
    let length = (facing[1] * facing[1] + facing[0] * facing[0] + facing[2] * facing[2]).sqrt();
    Some(facing.map(|v| v / length))
}

/// Encode a map-space creation direction as build-209026 [pitch, yaw] words.
///
/// Requires finite components and the client's creation-path unit-length
/// tolerance (squared length 0.9801..=1.0201). Does not normalize XYZ: pitch
/// uses clamped Z, while yaw uses the normalized XY projection. Rejects zero
/// instead of emitting the sentinel rejected by creation. Half ties round
/// to even; host trig can differ from the Windows CRT near word boundaries.
pub fn facing_words_209026([x, y, z]: [f32; 3]) -> Option<[u16; 2]> {
    let xy_squared = y * y + x * x;
    let squared_length = xy_squared + z * z;
    if ![x, y, z].iter().all(|v| v.is_finite()) || !(0.9801..=1.0201).contains(&squared_length) {
        return None;
    }
    let (nx, ny) = if xy_squared < 1e-12 {
        (0., 0.)
    } else {
        let inverse_length = 1. / xy_squared.sqrt();
        (x * inverse_length, y * inverse_length)
    };
    let word =
        |angle: f32| ((angle / std::f32::consts::PI * 32766.).round_ties_even() as i16) as u16;
    Some([word(z.clamp(-1., 1.).asin()), word(ny.atan2(nx))])
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independently specified wire words exercise every field boundary. This
    // deliberately includes values unsuitable for a live creation message.
    fn frame_fixture() -> Vec<u8> {
        "25 00 34 12 ac 02 a5 80 01 03 10 20 30 00 81 80 80 80 03 01 02 04
         00 00 80 3f 00 00 00 80 45 23 c1 7f ff ff ff ff 0f 00 80 ff 7f
         00 00 00 00 00 00 80 3f 00 00 80 bf 00 00 00 3f
         00 00 00 40 00 00 00 80 00 00 80 7f 21 43 c5 7f 00 00 80 ff"
            .split_whitespace()
            .map(|s| u8::from_str_radix(s, 16).unwrap())
            .collect()
    }

    #[test]
    fn creation_frame_golden_bytes_preserve_fields_and_float_bits() {
        let wire = frame_fixture();
        let mut input = wire.as_slice();
        let frame = AgentCreation209026::decode_frame(&mut input).unwrap();
        assert!(input.is_empty());
        assert_eq!(frame.agent_index, 0x1234);
        assert_eq!(frame.remote_tick, 300);
        assert_eq!(frame.command_flags, 0xa5);
        assert_eq!(frame.behavior_value, 128);
        assert_eq!(frame.behavior_data, [0x10, 0x20, 0x30]);
        assert_eq!(frame.agent_type, 0);
        assert_eq!(frame.identity, 0x30000001);
        assert_eq!(frame.creation_flags, [1, 2, 4]);
        assert_eq!(
            frame.position.0.map(f32::to_bits),
            [0x3f800000, 0x80000000, 0x7fc12345]
        );
        assert_eq!(frame.position.1, u32::MAX);
        assert_eq!(frame.facing, [0x8000, 0x7fff]);
        assert_eq!(
            frame.parameters.map(f32::to_bits),
            [
                0, 0x3f800000, 0xbf800000, 0x3f000000, 0x40000000, 0x80000000, 0x7f800000,
                0x7fc54321,
            ]
        );
        assert_eq!(frame.radius.to_bits(), 0xff800000);
        let mut encoded = BytesMut::from(&b"prefix"[..]);
        frame.encode_frame(&mut encoded).unwrap();
        assert_eq!(&encoded[6..], wire);
    }

    #[test]
    fn creation_frame_truncation_and_invalid_fields_are_transactional() {
        let wire = frame_fixture();
        for end in 0..wire.len() {
            let mut input = &wire[..end];
            assert!(
                AgentCreation209026::decode_frame(&mut input).is_err(),
                "end={end}"
            );
            assert_eq!(input, &wire[..end]);
        }
        for (offset, bad) in [(0, 36), (9, 251), (38, 0x10)] {
            let mut malformed = wire.clone();
            malformed[offset] = bad;
            let mut input = malformed.as_slice();
            assert_eq!(
                AgentCreation209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, malformed);
        }
        let mut input = wire.as_slice();
        let mut frame = AgentCreation209026::decode_frame(&mut input).unwrap();
        frame.behavior_data = vec![0; 251];
        let mut output = BytesMut::from(&b"prefix"[..]);
        assert_eq!(
            frame.encode_frame(&mut output).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(output.as_ref(), b"prefix");
    }

    #[test]
    fn creation_frame_buffer_limits_and_following_frame() {
        let golden = frame_fixture();
        for length in [0u8, 250] {
            let mut wire = golden[..9].to_vec();
            wire.push(length);
            wire.extend((0..length).map(|v| v ^ 0xa5));
            wire.extend_from_slice(&golden[13..]);
            let first_length = wire.len();
            wire.extend_from_slice(&golden);
            let mut input = wire.as_slice();
            let frame = AgentCreation209026::decode_frame(&mut input).unwrap();
            assert_eq!(frame.behavior_data.len(), length as usize);
            assert_eq!(input, golden);
            let mut encoded = BytesMut::new();
            frame.encode_frame(&mut encoded).unwrap();
            assert_eq!(encoded, &wire[..first_length]);
            AgentCreation209026::decode_frame(&mut input).unwrap();
            assert!(input.is_empty());
        }
    }

    #[test]
    fn facing_field_order_signed_axes_and_half_turn() {
        for (p, y, expected) in [
            (0i16, 0i16, [1., 0., 0.]),
            (0, 16383, [0., 1., 0.]),
            (0, -16383, [0., -1., 0.]),
            (16383, 0, [0., 0., 1.]),
            (-16383, 0, [0., 0., -1.]),
            (0, 32766, [-1., 0., 0.]),
        ] {
            let actual = facing_209026(p as u16, y as u16).unwrap();
            for i in 0..3 {
                assert!(
                    (actual[i] - expected[i]).abs() < 1e-6,
                    "{p} {y}: {actual:?}"
                );
            }
        }
    }

    #[test]
    fn facing_sentinel_and_signed_extremes() {
        assert_eq!(facing_209026(0x7fff, 0), None);
        assert_eq!(facing_209026(0, 0x7fff), None);
        assert_eq!(facing_209026(0x7fff, 0x7fff), None);
        let extreme = facing_209026(0, 0x8000).unwrap();
        assert!(extreme[0] < -0.99999);
        // -32768 is slightly past -pi, not a sentinel or a clamped half turn.
        assert!((0.00018..0.00021).contains(&extreme[1]));
        for (p, y) in [(0x8000, 0x8001), (0xffff, 0xffff), (8191, 8192)] {
            let v = facing_209026(p, y).unwrap();
            assert!(v.iter().all(|x| x.is_finite()));
            assert!((v.iter().map(|x| x * x).sum::<f32>() - 1.).abs() < 1e-6);
        }
        assert!(facing_209026(0, 0xffff).unwrap()[1] < 0.);
    }

    #[test]
    fn facing_encoder_axes_signed_zero_and_half_ties() {
        for (direction, expected) in [
            ([1., 0., 0.], [0i16, 0i16]),
            ([0., 1., 0.], [0, 16383]),
            ([0., -1., 0.], [0, -16383]),
            ([0., 0., 1.], [16383, 0]),
            ([0., 0., -1.], [-16383, 0]),
            ([-1., 0., 0.], [0, 32766]),
            ([-1., -0., 0.], [0, -32766]),
            ([0.5f32.sqrt(), 0.5f32.sqrt(), 0.], [0, 8192]),
            ([0.5f32.sqrt(), -0.5f32.sqrt(), 0.], [0, -8192]),
        ] {
            assert_eq!(
                facing_words_209026(direction),
                Some(expected.map(|v| v as u16))
            );
        }
    }

    #[test]
    fn facing_encoder_creation_bounds_and_vertical_projection() {
        for bad in [
            [0., 0., 0.],
            [2., 0., 0.],
            [0.98, 0., 0.],
            [1.02, 0., 0.],
            [f32::NAN, 0., 1.],
            [1., f32::INFINITY, 0.],
            [1., 0., f32::NEG_INFINITY],
        ] {
            assert_eq!(facing_words_209026(bad), None);
        }
        // XY below the client threshold is discarded, including its signs.
        assert_eq!(facing_words_209026([-1e-7, -1e-7, 1.]), Some([16383, 0]));
        assert_eq!(facing_words_209026([0., 0., 1.005]), Some([16383, 0]));
        // Accepted approximate unit vectors use Z as supplied, not normalized.
        let encoded = facing_words_209026([0., 0., 0.995]).unwrap();
        assert!(encoded[0] < 16383);
    }

    #[test]
    fn facing_encoder_round_trip_preserves_direction() {
        for pitch in [-16000i16, -8192, -1, 0, 1, 8192, 16000] {
            for yaw in [-32766i16, -24574, -8192, -1, 0, 1, 8192, 24574, 32766] {
                let direction = facing_209026(pitch as u16, yaw as u16).unwrap();
                let words = facing_words_209026(direction).unwrap();
                assert!(!words.contains(&0x7fff));
                let decoded = facing_209026(words[0], words[1]).unwrap();
                for i in 0..3 {
                    assert!((decoded[i] - direction[i]).abs() < 0.0002);
                }
            }
        }
    }
}
