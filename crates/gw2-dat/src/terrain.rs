//! Resident build-209026 `mapc/trn` terrain heights in map-local coordinates.
//!
//! Chunk borders and triangle interpolation follow the frozen client's point
//! query. This is not a walkability query: tile flags, props and water are not
//! considered. See `docs/re/209026-terrain-height.md`.

use crate::pfread::PfValue;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone)]
pub struct TerrainHeights209026 {
    rect: [f32; 4],
    chunks: [usize; 2],
    cells: usize,
    stride: usize,
    spacing: f32,
    heights: Vec<f32>,
}

fn bad(message: &str) -> Error {
    Error::new(ErrorKind::InvalidData, format!("trn: {message}"))
}

impl TerrainHeights209026 {
    /// Validate resident heights, retaining every chunk's border samples.
    /// `dims` counts cells; zero `cells_per_chunk` defaults to 32 as in the client.
    /// Rectangle dimensions must agree with the client's integer cell spacing.
    pub fn new(
        rect: [f32; 4],
        dims: [u32; 2],
        cells_per_chunk: u32,
        heights: Vec<f32>,
    ) -> Result<Self> {
        let cells = if cells_per_chunk == 0 {
            32
        } else {
            cells_per_chunk
        };
        if cells > 3072
            || dims
                .iter()
                .any(|&n| n == 0 || n > 4_194_303 || n % cells != 0)
        {
            return Err(bad("invalid grid dimensions or cells per chunk"));
        }
        let spacing = (3072 / cells) as f32;
        if !rect.iter().all(|v| v.is_finite())
            || rect[2] - rect[0] != dims[0] as f32 * spacing
            || rect[3] - rect[1] != dims[1] as f32 * spacing
        {
            return Err(bad("rectangle does not match grid dimensions"));
        }
        let chunks = [(dims[0] / cells) as usize, (dims[1] / cells) as usize];
        let cells = cells as usize;
        let stride = cells + 3;
        let count = chunks[0]
            .checked_mul(chunks[1])
            .and_then(|n| n.checked_mul(stride))
            .and_then(|n| n.checked_mul(stride));
        if count != Some(heights.len()) || !heights.iter().all(|v| v.is_finite()) {
            return Err(bad("invalid resident height count or nonfinite height"));
        }
        Ok(Self {
            rect,
            chunks,
            cells,
            stride,
            spacing,
            heights,
        })
    }

    /// Decode named fields from `parm` v0 and `trn` v15 schema values.
    /// Callers select the chunk versions before using this typed view.
    pub fn from_values(parm: &PfValue, trn: &PfValue) -> Result<Self> {
        let rect = match parm.get("rect") {
            Some(PfValue::Floats(v)) if v.len() == 4 => [v[0], v[1], v[2], v[3]],
            _ => return Err(bad("missing Float4 rectangle")),
        };
        let dims = match trn.get("dims") {
            Some(PfValue::Dwords(v)) if v.len() == 2 => [v[0], v[1]],
            _ => return Err(bad("missing Dword2 dimensions")),
        };
        let cells = match trn.get("verticesPerChunkSide") {
            Some(PfValue::U32(v)) => *v,
            _ => return Err(bad("missing cells per chunk")),
        };
        let heights = match trn.get("heightMapArray") {
            Some(PfValue::Array(v)) => v
                .iter()
                .map(|v| match v {
                    PfValue::F32(h) => Ok(*h),
                    _ => Err(bad("height is not a Float")),
                })
                .collect::<Result<Vec<_>>>()?,
            _ => return Err(bad("missing resident height array")),
        };
        Self::new(rect, dims, cells, heights)
    }

    /// Raw terrain height for map-local XY, with the client's half-open bounds.
    /// Both minimum edges are included; both maximum edges are excluded.
    /// Returns `None` outside the rectangle or for nonfinite input/result.
    /// Apply any inverse map transform before calling. No physics-unit or Z-sign
    /// conversion is applied here, and a height does not imply walkable ground.
    pub fn height_at(&self, [x, y]: [f32; 2]) -> Option<f32> {
        if !x.is_finite()
            || !y.is_finite()
            || x < self.rect[0]
            || x >= self.rect[2]
            || y < self.rect[1]
            || y >= self.rect[3]
        {
            return None;
        }
        let dx = x - self.rect[0];
        let dy = self.rect[3] - y;
        let ix = (dx / self.spacing).floor() as usize;
        let iy = (dy / self.spacing).floor() as usize;
        let cx = (ix / self.cells).min(self.chunks[0] - 1);
        let cy = (iy / self.cells).min(self.chunks[1] - 1);
        let lx = ix - cx * self.cells;
        let ly = iy - cy * self.cells;
        let base = (cy * self.chunks[0] + cx) * self.stride * self.stride
            + (ly + 1) * self.stride
            + lx
            + 1;
        let h00 = *self.heights.get(base)?;
        let h10 = *self.heights.get(base + 1)?;
        let h01 = *self.heights.get(base + self.stride)?;
        let h11 = *self.heights.get(base + self.stride + 1)?;
        let fx = (dx - ix as f32 * self.spacing) / self.spacing;
        let fy = (dy - iy as f32 * self.spacing) / self.spacing;
        let h = if fx + fy < 1.0 {
            (h10 - h00) * fx + h00 + (h01 - h00) * fy
        } else {
            (h01 - h11) * (1.0 - fx) + h11 + (h10 - h11) * (1.0 - fy)
        };
        h.is_finite().then_some(h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_two_triangles_and_preserves_diagonal() {
        let mut h = vec![0.; 25];
        h[6] = 0.;
        h[7] = 4.;
        h[11] = 8.;
        h[12] = 20.;
        let t = TerrainHeights209026::new([0., 0., 3072., 3072.], [2, 2], 2, h).unwrap();
        assert_eq!(t.height_at([384., 2688.]), Some(3.));
        assert_eq!(t.height_at([1152., 1920.]), Some(13.));
        assert_eq!(t.height_at([768., 2304.]), Some(6.));
    }

    #[test]
    fn uses_selected_chunk_and_interior_offset() {
        // Deliberately unequal chunk copies: each lookup must use its own chunk.
        let mut h = vec![-999.; 100];
        for chunk in 0..4 {
            for y in 1..5 {
                for x in 1..5 {
                    h[chunk * 25 + y * 5 + x] = (chunk * 100 + y * 10 + x) as f32;
                }
            }
        }
        let t = TerrainHeights209026::new([0., 0., 6144., 6144.], [4, 4], 2, h).unwrap();
        assert_eq!(t.height_at([0., 4608.]), Some(21.));
        assert_eq!(t.height_at([3072., 4608.]), Some(121.));
        assert_eq!(t.height_at([0., 3072.]), Some(211.));
        assert_eq!(t.height_at([3072., 0.]), Some(331.));
        for xy in [
            [6144., 100.],
            [100., 6144.],
            [-1., 100.],
            [100., -1.],
            [f32::NAN, 0.],
            [0., f32::INFINITY],
        ] {
            assert_eq!(t.height_at(xy), None);
        }
    }

    #[test]
    fn rejects_invalid_layout_and_nonfinite_data() {
        let rect = [0., 0., 3072., 3072.];
        for (dims, cells, len) in [
            ([0, 2], 2, 25),
            ([3, 2], 2, 25),
            ([2, 2], 4000, 25),
            ([2, 2], 2, 24),
            ([u32::MAX, 2], 2, 25),
        ] {
            assert!(TerrainHeights209026::new(rect, dims, cells, vec![0.; len]).is_err());
        }
        assert!(
            TerrainHeights209026::new([0., 0., 3000., 3072.], [2, 2], 2, vec![0.; 25]).is_err()
        );
        assert!(TerrainHeights209026::new(rect, [2, 2], 2, vec![f32::NAN; 25]).is_err());
        let t = TerrainHeights209026::new(rect, [32, 32], 0, vec![7.; 1225]).unwrap();
        assert_eq!(t.height_at([150., 150.]), Some(7.));
    }

    #[test]
    fn decodes_required_schema_fields() {
        let parm = PfValue::Struct(vec![(
            "rect".into(),
            PfValue::Floats(vec![0., 0., 3072., 3072.]),
        )]);
        let mut trn = PfValue::Struct(vec![
            ("dims".into(), PfValue::Dwords(vec![2, 2])),
            ("verticesPerChunkSide".into(), PfValue::U32(2)),
            (
                "heightMapArray".into(),
                PfValue::Array(vec![PfValue::F32(-42.); 25]),
            ),
        ]);
        assert_eq!(
            TerrainHeights209026::from_values(&parm, &trn)
                .unwrap()
                .height_at([100., 200.]),
            Some(-42.)
        );
        let PfValue::Struct(fields) = &mut trn else {
            unreachable!()
        };
        fields[2].1 = PfValue::Array(vec![PfValue::U32(42); 25]);
        assert!(TerrainHeights209026::from_values(&parm, &trn).is_err());
        assert!(TerrainHeights209026::from_values(&PfValue::Null, &trn).is_err());
    }
}
