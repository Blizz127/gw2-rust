//! Typed view of a map's collision chunk (`mapc` / `havk`, PackMapCollide), built
//! on [`crate::pfread`] values.
//!
//! Field names follow the 208039 reference schema (PackMapCollideV16 family).
//! Lookups are by name, so other versions that share the names also work, and
//! missing arrays read as empty. Version 16 decodes on build 209026 Queensdale;
//! index checks pass. Its collision bounds and prop translations use different
//! coordinate scales. 4,113 matched prop translations equal render positions
//! exactly. Prop translations apply the map matrix, then divide XYZ by 32
//! and negate Z for the collision scene. Cached mesh vertices are already in
//! collision units; local prop rotation is `qy(ry) * qx(rx) * qz(-rz)`.
//! See `docs/re/209026-map-coordinate-spaces.md` and
//! `docs/re/209026-collision-rotation.md` for scope and remaining limitations.
//!
//! Server use: collision triangles give ground height, line-of-sight and
//! blocked areas for NPC/mob movement in a zone (stage 5–6).

use std::io::{Error, ErrorKind, Result};

use crate::pfread::PfValue;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapCollide {
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub water_surface_z: Option<f32>,
    pub collisions: Vec<CollisionMesh>,
    pub blockers: Vec<Vec<[f32; 3]>>,
    pub animations: Vec<CollideAnimation>,
    pub geometries: Vec<CollideGeometry>,
    pub prop_models: Vec<ModelPlacement>,
    pub zone_models: Vec<ModelPlacement>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollisionMesh {
    pub vertices: Vec<[f32; 3]>,
    /// Triangle list (3 indices per triangle) into `vertices`.
    pub indices: Vec<u16>,
    pub surfaces: Vec<u16>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollideAnimation {
    pub sequence: u64,
    pub collision_indices: Vec<u32>,
    pub blocker_indices: Vec<u32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollideGeometry {
    /// Indices into [`MapCollide::animations`].
    pub animations: Vec<u32>,
    pub nav_mesh_index: Option<u16>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelPlacement {
    pub token: Option<u64>,
    pub scale: f32,
    pub translate: [f32; 3],
    pub rotate: [f32; 3],
    pub geometry_index: u32,
}

impl ModelPlacement {
    /// Convert a `propModels` placement origin to build-209026 collision space.
    ///
    /// `map_transform` is the map's row-major 3×4 affine matrix. The primary
    /// map uses identity on the traced default-view load path. Apply that
    /// matrix before dividing by 32 and reversing Z. The model's own scale
    /// and rotation do not change its placement origin.
    ///
    /// This operation does not transform mesh vertices or rotations, and has
    /// not been established for other placement tables or mission packets.
    pub fn prop_physics_translation_209026(&self, map_transform: [[f32; 4]; 3]) -> [f32; 3] {
        let [x, y, z] = self.translate;
        let [tx, ty, tz] = map_transform.map(|row| {
            row[0] * x + row[1] * y + row[2] * z + row[3]
        });
        [tx * 0.03125, ty * 0.03125, -tz * 0.03125]
    }

    /// Local collision quaternion `[x, y, z, w]` for a build-209026 prop.
    ///
    /// Stored angles are radians. The Hamilton product is
    /// `qy(ry) * qx(rx) * qz(-rz)`. This is the final rotation for an identity
    /// map; a nonidentity map needs additional rotation composition. Computed
    /// in double precision as in the client, without a bit-exact CRT guarantee.
    pub fn prop_local_physics_rotation_209026(&self) -> [f32; 4] {
        let [rx, ry, rz] = self.rotate;
        let (sa, ca) = (f64::from(rx) * 0.5).sin_cos();
        let (sb, cb) = (f64::from(ry) * 0.5).sin_cos();
        let (sc, cc) = (f64::from(rz) * -0.5).sin_cos();
        [
            cc * cb * sa + sc * sb * ca,
            cc * sb * ca - sc * cb * sa,
            sc * cb * ca - cc * sb * sa,
            sc * sb * sa + cc * cb * ca,
        ].map(|v| v as f32)
    }
}

fn bad(what: &str) -> Error {
    Error::new(ErrorKind::InvalidData, format!("havk: {what}"))
}

fn arr<'a>(v: &'a PfValue, name: &str) -> Result<&'a [PfValue]> {
    match v.get(name) {
        None | Some(PfValue::Null) => Ok(&[]),
        Some(PfValue::Array(a)) => Ok(a),
        Some(_) => Err(bad(&format!("{name} is not an array"))),
    }
}

fn f3(v: &PfValue) -> Result<[f32; 3]> {
    match v {
        PfValue::Floats(f) if f.len() == 3 => Ok([f[0], f[1], f[2]]),
        _ => Err(bad("expected Float3")),
    }
}

fn field_f3(v: &PfValue, name: &str) -> Result<[f32; 3]> {
    v.get(name).map(f3).transpose().map(|o| o.unwrap_or_default())
}

fn field_f32(v: &PfValue, name: &str) -> Option<f32> {
    match v.get(name) {
        Some(PfValue::F32(x)) => Some(*x),
        _ => None,
    }
}

fn field_u32(v: &PfValue, name: &str) -> Option<u32> {
    match v.get(name) {
        Some(PfValue::U32(x)) => Some(*x),
        Some(PfValue::U16(x)) => Some(*x as u32),
        Some(PfValue::U8(x)) => Some(*x as u32),
        _ => None,
    }
}

fn field_u64(v: &PfValue, name: &str) -> Option<u64> {
    match v.get(name) {
        Some(PfValue::U64(x)) => Some(*x),
        _ => None,
    }
}

fn u16s(v: &PfValue, name: &str) -> Result<Vec<u16>> {
    arr(v, name)?.iter().map(|x| match x {
        PfValue::U16(n) => Ok(*n),
        _ => Err(bad(&format!("{name} element is not a Word"))),
    }).collect()
}

fn u32s(v: &PfValue, name: &str) -> Result<Vec<u32>> {
    arr(v, name)?.iter().map(|x| match x {
        PfValue::U32(n) => Ok(*n),
        _ => Err(bad(&format!("{name} element is not a Dword"))),
    }).collect()
}

fn f3s(v: &PfValue, name: &str) -> Result<Vec<[f32; 3]>> {
    arr(v, name)?.iter().map(f3).collect()
}

fn placement(v: &PfValue) -> Result<ModelPlacement> {
    Ok(ModelPlacement {
        token: field_u64(v, "token"),
        scale: field_f32(v, "scale").unwrap_or(1.0),
        translate: field_f3(v, "translate")?,
        rotate: field_f3(v, "rotate")?,
        geometry_index: field_u32(v, "geometryIndex").ok_or_else(|| bad("placement without geometryIndex"))?,
    })
}

impl MapCollide {
    pub fn from_value(v: &PfValue) -> Result<Self> {
        let out = Self {
            bounds_min: field_f3(v, "boundsMin")?,
            bounds_max: field_f3(v, "boundsMax")?,
            water_surface_z: field_f32(v, "waterSurfaceZ"),
            collisions: arr(v, "collisions")?
                .iter()
                .map(|c| {
                    Ok(CollisionMesh {
                        vertices: f3s(c, "vertices")?,
                        indices: u16s(c, "indices")?,
                        surfaces: u16s(c, "surfaces")?,
                    })
                })
                .collect::<Result<_>>()?,
            blockers: arr(v, "blockers")?.iter().map(|b| f3s(b, "vertices")).collect::<Result<_>>()?,
            animations: arr(v, "animations")?
                .iter()
                .map(|a| {
                    Ok(CollideAnimation {
                        sequence: field_u64(a, "sequence").unwrap_or(0),
                        collision_indices: u32s(a, "collisionIndices")?,
                        blocker_indices: u32s(a, "blockerIndices")?,
                    })
                })
                .collect::<Result<_>>()?,
            geometries: arr(v, "geometries")?
                .iter()
                .map(|g| {
                    Ok(CollideGeometry {
                        animations: u32s(g, "animations")?,
                        nav_mesh_index: field_u32(g, "navMeshIndex").map(|n| n as u16),
                    })
                })
                .collect::<Result<_>>()?,
            prop_models: arr(v, "propModels")?.iter().map(placement).collect::<Result<_>>()?,
            zone_models: arr(v, "zoneModels")?.iter().map(placement).collect::<Result<_>>()?,
        };
        out.validate()?;
        Ok(out)
    }

    /// Cross-reference checks: every triangle index and every table index in range.
    pub fn validate(&self) -> Result<()> {
        for (i, c) in self.collisions.iter().enumerate() {
            if c.indices.len() % 3 != 0 {
                return Err(bad(&format!("collision {i}: {} indices is not a triangle list", c.indices.len())));
            }
            if let Some(&ix) = c.indices.iter().find(|&&ix| ix as usize >= c.vertices.len()) {
                return Err(bad(&format!("collision {i}: index {ix} >= {} vertices", c.vertices.len())));
            }
        }
        for (i, a) in self.animations.iter().enumerate() {
            if a.collision_indices.iter().any(|&c| c as usize >= self.collisions.len()) {
                return Err(bad(&format!("animation {i} references a missing collision")));
            }
        }
        for (i, g) in self.geometries.iter().enumerate() {
            if g.animations.iter().any(|&a| a as usize >= self.animations.len()) {
                return Err(bad(&format!("geometry {i} references a missing animation")));
            }
        }
        for p in self.prop_models.iter().chain(&self.zone_models) {
            if p.geometry_index as usize >= self.geometries.len() {
                return Err(bad(&format!("placement references missing geometry {}", p.geometry_index)));
            }
        }
        Ok(())
    }

    /// Collision meshes used by a geometry (through its first animation, i.e. the
    /// rest pose; using the first one is an assumption).
    pub fn geometry_meshes(&self, geometry: u32) -> impl Iterator<Item = &CollisionMesh> {
        self.geometries
            .get(geometry as usize)
            .and_then(|g| g.animations.first())
            .and_then(|&a| self.animations.get(a as usize))
            .into_iter()
            .flat_map(|a| a.collision_indices.iter())
            .filter_map(|&c| self.collisions.get(c as usize))
    }

    pub fn triangle_count(&self) -> usize {
        self.collisions.iter().map(|c| c.indices.len() / 3).sum()
    }
}

impl CollisionMesh {
    pub fn triangles(&self) -> impl Iterator<Item = [[f32; 3]; 3]> + '_ {
        self.indices.as_chunks::<3>().0.iter().map(|t| {
            [self.vertices[t[0] as usize], self.vertices[t[1] as usize], self.vertices[t[2] as usize]]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pfread::{Limits, read_chunk};
    use crate::pfschema::{find, parse};

    fn s(fields: Vec<(&str, PfValue)>) -> PfValue {
        PfValue::Struct(fields.into_iter().map(|(n, v)| (n.to_string(), v)).collect())
    }
    fn fl(a: [f32; 3]) -> PfValue {
        PfValue::Floats(a.to_vec())
    }

    fn sample() -> PfValue {
        s(vec![
            ("boundsMin", fl([-10.0, -10.0, -1.0])),
            ("boundsMax", fl([10.0, 10.0, 5.0])),
            ("flags", PfValue::U32(0)),
            ("waterSurfaceZ", PfValue::F32(-0.5)),
            ("collisions", PfValue::Array(vec![s(vec![
                ("indices", PfValue::Array([0u16, 1, 2, 0, 2, 3].map(PfValue::U16).to_vec())),
                ("vertices", PfValue::Array(vec![fl([0.0, 0.0, 0.0]), fl([1.0, 0.0, 0.0]), fl([1.0, 1.0, 0.0]), fl([0.0, 1.0, 0.0])])),
                ("surfaces", PfValue::Array(vec![PfValue::U16(7)])),
                ("moppCodeData", s(vec![("moppData", PfValue::Array(vec![]))])),
            ])])),
            ("blockers", PfValue::Array(vec![])),
            ("animations", PfValue::Array(vec![s(vec![
                ("sequence", PfValue::U64(0)),
                ("collisionIndices", PfValue::Array(vec![PfValue::U32(0)])),
                ("blockerIndices", PfValue::Array(vec![])),
            ])])),
            ("geometries", PfValue::Array(vec![s(vec![
                ("quantizedExtents", PfValue::U8(0)),
                ("animations", PfValue::Array(vec![PfValue::U32(0)])),
                ("navMeshIndex", PfValue::U16(3)),
            ])])),
            ("obsModels", PfValue::Array(vec![])),
            ("propModels", PfValue::Array(vec![s(vec![
                ("token", PfValue::U64(99)),
                ("sequence", PfValue::U64(0)),
                ("scale", PfValue::F32(2.0)),
                ("translate", fl([5.0, 6.0, 7.0])),
                ("rotate", fl([0.0, 0.0, 1.57])),
                ("geometryIndex", PfValue::U32(0)),
            ])])),
            ("zoneModels", PfValue::Array(vec![])),
            ("waterVolumes", PfValue::Array(vec![])),
        ])
    }

    #[test]
    fn typed_view_and_cross_references() {
        let m = MapCollide::from_value(&sample()).unwrap();
        assert_eq!(m.triangle_count(), 2);
        assert_eq!(m.water_surface_z, Some(-0.5));
        assert_eq!(m.prop_models[0].token, Some(99));
        assert_eq!(m.prop_models[0].scale, 2.0);
        assert_eq!(m.geometries[0].nav_mesh_index, Some(3));
        let meshes: Vec<_> = m.geometry_meshes(0).collect();
        assert_eq!(meshes.len(), 1);
        let tris: Vec<_> = meshes[0].triangles().collect();
        assert_eq!(tris[1], [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]]);
        assert_eq!(m.geometry_meshes(5).count(), 0);
    }

    #[test]
    fn prop_translation_applies_map_transform_before_collision_units() {
        let prop = ModelPlacement {
            translate: [32.0, 64.0, -96.0],
            scale: 7.0,
            rotate: [0.3, 0.7, 1.2],
            ..Default::default()
        };
        let identity = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
        assert_eq!(prop.prop_physics_translation_209026(identity), [1.0, 2.0, 3.0]);
        // Rotate XY, scale Z and translate in map units. Model scale is separate.
        let affine = [[0.0, -1.0, 0.0, 32.0], [1.0, 0.0, 0.0, 64.0], [0.0, 0.0, 2.0, 96.0]];
        assert_eq!(prop.prop_physics_translation_209026(affine), [-1.0, 3.0, 3.0]);
    }

    #[test]
    fn prop_rotation_axis_signs_and_noncommuting_order() {
        let h = std::f32::consts::FRAC_PI_2;
        let s = std::f32::consts::FRAC_1_SQRT_2;
        for (angles, expected) in [
            ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]),
            ([h, 0.0, 0.0], [s, 0.0, 0.0, s]),
            ([0.0, h, 0.0], [0.0, s, 0.0, s]),
            ([0.0, 0.0, h], [0.0, 0.0, -s, s]),
            ([h, h, 0.0], [0.5, 0.5, -0.5, 0.5]),
            ([h, h, h], [0.0, s, -s, 0.0]),
        ] {
            let prop = ModelPlacement { rotate: angles, ..Default::default() };
            let actual = prop.prop_local_physics_rotation_209026();
            for (a, e) in actual.into_iter().zip(expected) {
                assert!((a - e).abs() < 2e-7, "angles={angles:?}, actual={actual:?}");
            }
        }
    }

    #[test]
    fn rejects_broken_references() {
        let mut v = sample();
        // Point the prop at a geometry that doesn't exist.
        if let PfValue::Struct(f) = &mut v
            && let Some((_, PfValue::Array(props))) = f.iter_mut().find(|(n, _)| n == "propModels")
            && let PfValue::Struct(pf) = &mut props[0]
        {
            pf.iter_mut().find(|(n, _)| n == "geometryIndex").unwrap().1 = PfValue::U32(9);
        }
        assert!(MapCollide::from_value(&v).is_err());
        assert!(MapCollide::from_value(&PfValue::U8(1)).unwrap() == MapCollide::default());
    }

    #[test]
    fn decodes_from_raw_bytes_with_reference_schema() {
        // Minimal real-layout V16 payload: bounds, flags, water Z, and every ArrayPtr empty.
        let schemas = parse(include_str!("../reference/packfiles-208039.xml")).unwrap();
        let v16 = find(&schemas, "mapc", "havk").unwrap().version(16).unwrap();
        let mut b = Vec::new();
        for x in [-1.0f32, -2.0, -3.0, 1.0, 2.0, 3.0] {
            b.extend_from_slice(&x.to_le_bytes());
        }
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0.25f32.to_le_bytes());
        let arrays = v16.fields.iter().filter(|f| f.kind == "ArrayPtr").count();
        assert_eq!(arrays, 8);
        for _ in 0..arrays {
            b.extend_from_slice(&[0u8; 12]); // u32 count + 8-byte null pointer
        }
        let v = read_chunk(&b, v16, true, Limits::default()).unwrap();
        let m = MapCollide::from_value(&v).unwrap();
        assert_eq!(m.bounds_max, [1.0, 2.0, 3.0]);
        assert_eq!(m.water_surface_z, Some(0.25));
        assert_eq!(m.triangle_count(), 0);
    }
}
