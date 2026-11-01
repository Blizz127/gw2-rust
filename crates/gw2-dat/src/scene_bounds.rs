//! Build-209026 bounds on the default identity-primary-map load path.
//! See `docs/re/209026-scene-bounds.md`; membership is not spawn validation.

use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds3 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Bounds3 {
    /// Inclusive XYZ membership. Nonfinite points are rejected.
    pub fn contains(&self, point: [f32; 3]) -> bool {
        (0..3).all(|i| point[i].is_finite() && self.min[i] <= point[i] && point[i] <= self.max[i])
    }

    fn valid(self) -> bool {
        (0..3).all(|i| {
            self.min[i].is_finite() && self.max[i].is_finite() && self.min[i] <= self.max[i]
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneBounds209026 {
    /// Full-character creation clamps the resolved origin to these bounds.
    pub character: Bounds3,
    /// Simple-character creation checks these expanded bounds.
    pub expanded: Bounds3,
    /// Unmodified archive collision bounds.
    pub archive: Bounds3,
}

impl SceneBounds209026 {
    /// Construct bounds for the default request rectangle and identity map.
    /// `rect` uses map units; `archive` already uses collision units.
    /// Zero cells-per-chunk defaults to 32. Other transforms/request rectangles,
    /// missing havk and the client's zero-minimum fallback are unsupported.
    pub fn primary_map(rect: [f32; 4], cells_per_chunk: u32, archive: Bounds3) -> Result<Self> {
        let bad = |message| Error::new(ErrorKind::InvalidData, message);
        if !archive.valid() || !rect.iter().all(|v| v.is_finite()) {
            return Err(bad("invalid primary-map bounds"));
        }
        if archive.min == [0.; 3] {
            return Err(bad("zero-minimum havk fallback is unsupported"));
        }
        let cells = if cells_per_chunk == 0 {
            32
        } else {
            cells_per_chunk
        };
        if cells > 3072 {
            return Err(bad("invalid cells per chunk"));
        }
        let inset = (3072 / cells) as f32 / 32.;
        let character = Bounds3 {
            min: [rect[0] / 32. + inset, rect[1] / 32. + inset, archive.min[2]],
            max: [rect[2] / 32. - inset, rect[3] / 32. - inset, archive.max[2]],
        };
        let expanded = Bounds3 {
            min: archive.min.map(|v| v - 192.),
            max: archive.max.map(|v| v + 192.),
        };
        if !character.valid()
            || !expanded.valid()
            || character.min[0] >= character.max[0]
            || character.min[1] >= character.max[1]
        {
            return Err(bad("empty or invalid primary-map bounds after inset"));
        }
        Ok(Self {
            character,
            expanded,
            archive,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separates_rectangle_clamp_from_expanded_archive_bounds() {
        let archive = Bounds3 {
            min: [-20., -15., -4.],
            max: [25., 30., 8.],
        };
        let bounds =
            SceneBounds209026::primary_map([-320., -640., 960., 1280.], 0, archive).unwrap();
        assert_eq!(
            bounds.character,
            Bounds3 {
                min: [-7., -17., -4.],
                max: [27., 37., 8.]
            }
        );
        assert_eq!(
            bounds.expanded,
            Bounds3 {
                min: [-212., -207., -196.],
                max: [217., 222., 200.]
            }
        );
        assert!(bounds.character.contains([-7., 37., 8.]));
        assert!(!bounds.character.contains([-8., 0., 0.]));
        assert!(bounds.expanded.contains([-8., 0., 0.]));
        assert!(!bounds.character.contains([0., 0., 8.01]));
        assert!(!bounds.expanded.contains([f32::NAN, 0., 0.]));
        // The getter performs integer division, not rectangle-derived spacing.
        let odd =
            SceneBounds209026::primary_map([-320., -640., 960., 1280.], 100, archive).unwrap();
        assert_eq!(odd.character.min[0], -10. + 30. / 32.);
    }

    #[test]
    fn rejects_unsupported_or_invalid_bounds() {
        let archive = Bounds3 {
            min: [-1.; 3],
            max: [1.; 3],
        };
        let rect = [-320., -320., 320., 320.];
        assert!(SceneBounds209026::primary_map(rect, 3073, archive).is_err());
        assert!(SceneBounds209026::primary_map([0., 0., 100., 100.], 32, archive).is_err());
        assert!(
            SceneBounds209026::primary_map([f32::INFINITY, 0., 320., 320.], 32, archive).is_err()
        );
        assert!(
            SceneBounds209026::primary_map(
                rect,
                32,
                Bounds3 {
                    min: [-0., 0., 0.],
                    ..archive
                }
            )
            .is_err()
        );
        assert!(
            SceneBounds209026::primary_map(
                rect,
                32,
                Bounds3 {
                    max: [-2., 1., 1.],
                    ..archive
                }
            )
            .is_err()
        );
        assert!(
            SceneBounds209026::primary_map(
                rect,
                32,
                Bounds3 {
                    min: [f32::NAN, -1., -1.],
                    ..archive
                }
            )
            .is_err()
        );
    }
}
