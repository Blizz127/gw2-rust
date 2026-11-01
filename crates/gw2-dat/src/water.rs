//! Raw build-209026 collision-water metadata; not a complete containment query.
use crate::pfread::PfValue;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct MapWater209026 {
    /// Raw havk flags; bit 0 enables the implicit global volume.
    pub flags: u32,
    /// Stored map-space Z, whether or not the global volume is enabled.
    pub water_surface_z: f32,
    pub volumes: Vec<WaterVolume209026>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaterVolume209026 {
    pub guid: u64,
    /// Raw flags, preserved even though the traced loader copies only bit 0.
    pub flags: u32,
    /// Map-space Z bounds, copied without collision-unit conversion.
    pub vertical_range: [f32; 2],
    /// Map-space XY polygon vertices, in stored order.
    pub vertices: Vec<[f32; 2]>,
}

fn bad(message: &str) -> Error {
    Error::new(ErrorKind::InvalidData, format!("havk water: {message}"))
}
fn u32_field(value: &PfValue, name: &str) -> Result<u32> {
    match value.get(name) {
        Some(PfValue::U32(v)) => Ok(*v),
        _ => Err(bad(&format!("missing or invalid {name}"))),
    }
}
fn array<'a>(value: &'a PfValue, name: &str) -> Result<&'a [PfValue]> {
    match value.get(name) {
        Some(PfValue::Array(v)) => Ok(v),
        _ => Err(bad(&format!("missing or invalid {name}"))),
    }
}
fn float2(value: &PfValue) -> Result<[f32; 2]> {
    match value {
        PfValue::Floats(v) if v.len() == 2 && v.iter().all(|x| x.is_finite()) => Ok([v[0], v[1]]),
        _ => Err(bad("expected finite Float2")),
    }
}

impl MapWater209026 {
    /// Decode water fields from a bounded, schema-decoded havk v16 chunk.
    /// Rejects malformed/nonfinite fields and reversed explicit Z bounds.
    /// Volume names are not needed for geometry and are not retained.
    pub fn from_value(value: &PfValue) -> Result<Self> {
        let flags = u32_field(value, "flags")?;
        let water_surface_z = match value.get("waterSurfaceZ") {
            Some(PfValue::F32(v)) if v.is_finite() => *v,
            _ => return Err(bad("missing or nonfinite waterSurfaceZ")),
        };
        let mut volumes = Vec::new();
        for v in array(value, "waterVolumes")? {
            let guid = match v.get("guid") {
                Some(PfValue::U64(guid)) => *guid,
                _ => return Err(bad("missing or invalid volume GUID")),
            };
            let flags = u32_field(v, "flags")?;
            let vertical_range = float2(
                v.get("verticalRange")
                    .ok_or_else(|| bad("missing verticalRange"))?,
            )?;
            if vertical_range[0] > vertical_range[1] {
                return Err(bad("reversed verticalRange"));
            }
            let vertices = array(v, "vertices")?
                .iter()
                .map(float2)
                .collect::<Result<Vec<_>>>()?;
            volumes.push(WaterVolume209026 {
                guid,
                flags,
                vertical_range,
                vertices,
            });
        }
        Ok(Self {
            flags,
            water_surface_z,
            volumes,
        })
    }

    /// Surface of the flag-enabled implicit global volume, in map units.
    /// Explicit volumes can alter water behavior at a position; this is metadata,
    /// not a position-dependent water-height or containment result.
    pub fn global_surface_z(&self) -> Option<f32> {
        (self.flags & 1 != 0).then_some(self.water_surface_z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn s(fields: Vec<(&str, PfValue)>) -> PfValue {
        PfValue::Struct(fields.into_iter().map(|(n, v)| (n.into(), v)).collect())
    }
    fn sample(flags: u32, z: f32, range: Vec<f32>, point: Vec<f32>) -> PfValue {
        s(vec![
            ("flags", PfValue::U32(flags)),
            ("waterSurfaceZ", PfValue::F32(z)),
            (
                "waterVolumes",
                PfValue::Array(vec![s(vec![
                    ("guid", PfValue::U64(42)),
                    ("flags", PfValue::U32(0x81)),
                    ("verticalRange", PfValue::Floats(range)),
                    ("vertices", PfValue::Array(vec![PfValue::Floats(point)])),
                ])]),
            ),
        ])
    }
    #[test]
    fn global_presence_is_flag_controlled_and_raw_units_are_preserved() {
        let mut raw = sample(0x80, 12.5, vec![-5., 20.], vec![32., -64.]);
        let map = MapWater209026::from_value(&raw).unwrap();
        assert_eq!(map.global_surface_z(), None);
        assert_eq!(map.water_surface_z, 12.5);
        assert_eq!(
            map.volumes[0],
            WaterVolume209026 {
                guid: 42,
                flags: 0x81,
                vertical_range: [-5., 20.],
                vertices: vec![[32., -64.]],
            }
        );
        let PfValue::Struct(fields) = &mut raw else {
            unreachable!()
        };
        fields[0].1 = PfValue::U32(0x81);
        fields[1].1 = PfValue::F32(0.);
        fields[2].1 = PfValue::Array(vec![]);
        let map = MapWater209026::from_value(&raw).unwrap();
        assert_eq!(map.global_surface_z(), Some(0.));
        assert!(map.volumes.is_empty());
    }
    #[test]
    fn malformed_water_metadata_is_rejected() {
        for raw in [
            PfValue::Null,
            sample(1, f32::NAN, vec![0., 1.], vec![0., 0.]),
            sample(1, 0., vec![2., 1.], vec![0., 0.]),
            sample(1, 0., vec![0., f32::INFINITY], vec![0., 0.]),
            sample(1, 0., vec![0., 1.], vec![0., f32::NAN]),
            sample(1, 0., vec![0., 1.], vec![0.]),
        ] {
            assert_eq!(
                MapWater209026::from_value(&raw).unwrap_err().kind(),
                ErrorKind::InvalidData
            );
        }
    }
}
