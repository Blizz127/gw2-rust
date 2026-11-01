//! Geometry diagnostics for vertical segments; not client movement semantics.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerticalHit {
    pub z: f32,
    /// Unit normal from the supplied winding; either face can be hit.
    pub normal: [f32; 3],
}

/// Intersect either face of a triangle with an inclusive vertical segment.
/// Rejects nonfinite input, reversed bounds, and zero projected XY area.
/// Double-precision barycentric arithmetic uses a 1e-9 edge tolerance.
pub fn vertical_segment_hit(
    triangle: [[f32; 3]; 3],
    xy: [f32; 2],
    z_range: [f32; 2],
) -> Option<VerticalHit> {
    if !triangle
        .iter()
        .flatten()
        .chain(xy.iter())
        .chain(z_range.iter())
        .all(|v| v.is_finite())
        || z_range[0] > z_range[1]
    {
        return None;
    }
    let [a, b, c] = triangle.map(|p| p.map(f64::from));
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let dx = f64::from(xy[0]) - a[0];
    let dy = f64::from(xy[1]) - a[1];
    let det = u[0] * v[1] - u[1] * v[0];
    if det == 0. {
        return None;
    }
    let s = (dx * v[1] - dy * v[0]) / det;
    let t = (u[0] * dy - u[1] * dx) / det;
    if s < -1e-9 || t < -1e-9 || s + t > 1. + 1e-9 {
        return None;
    }
    let z = a[2] + s * u[2] + t * v[2];
    if z < f64::from(z_range[0]) || z > f64::from(z_range[1]) {
        return None;
    }
    let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], det];
    let length = n.iter().map(|n| n * n).sum::<f64>().sqrt();
    Some(VerticalHit {
        z: z as f32,
        normal: n.map(|n| (n / length) as f32),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sloped_surface_segment_edges_and_winding() {
        let tri = [[0., 0., 2.], [2., 0., 4.], [0., 2., 6.]];
        let hit = vertical_segment_hit(tri, [0.5, 0.5], [0., 10.]).unwrap();
        assert_eq!(hit.z, 3.5);
        let length = 6f32.sqrt();
        for (a, b) in hit
            .normal
            .into_iter()
            .zip([-1. / length, -2. / length, 1. / length])
        {
            assert!((a - b).abs() < 1e-6);
        }
        let reverse =
            vertical_segment_hit([tri[0], tri[2], tri[1]], [0.5, 0.5], [3.5, 3.5]).unwrap();
        assert_eq!(reverse.z, hit.z);
        assert_eq!(reverse.normal, hit.normal.map(|n| -n));
        assert_eq!(
            vertical_segment_hit(tri, [1., 1.], [0., 10.]).unwrap().z,
            5.
        );
        assert!(vertical_segment_hit(tri, [1.01, 1.01], [0., 10.]).is_none());
        assert!(vertical_segment_hit(tri, [0.5, 0.5], [0., 3.]).is_none());
        assert!(vertical_segment_hit(tri, [0.5, 0.5], [4., 10.]).is_none());
    }
    #[test]
    fn rejects_degenerate_nonfinite_and_reversed_inputs() {
        assert!(vertical_segment_hit([[0.; 3]; 3], [0.; 2], [0., 1.]).is_none());
        assert!(
            vertical_segment_hit(
                [[0., 0., 0.], [0., 0., 1.], [0., 1., 0.]],
                [0.; 2],
                [0., 1.]
            )
            .is_none()
        );
        let tri = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        assert!(vertical_segment_hit(tri, [0.; 2], [1., 0.]).is_none());
        assert!(vertical_segment_hit(tri, [f32::NAN, 0.], [0., 1.]).is_none());
        assert!(vertical_segment_hit(tri, [0.; 2], [0., f32::INFINITY]).is_none());
    }
}
