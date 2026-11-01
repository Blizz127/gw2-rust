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
}
