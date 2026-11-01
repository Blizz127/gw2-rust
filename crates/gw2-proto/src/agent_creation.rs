//! Build-specific engine-agent creation semantics established by client RE.

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
