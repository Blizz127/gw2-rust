//! Offline vertical-surface diagnostic for primary-map build-209026 geometry.
use gw2_dat::{
    archive::Archive,
    mapc::MapCollide,
    packfile::Packfile,
    pfread::{Limits, PfValue, read_chunk},
    pfschema::{PackfileSchema, find_all, parse},
    scene_bounds::{Bounds3, SceneBounds209026},
    surface::{VerticalHit, vertical_segment_hit},
    terrain::{TerrainHeights209026, TerrainTiles209026},
    water::MapWater209026,
};
use std::{
    io::{Error, ErrorKind, Result},
    process::ExitCode,
};

const USAGE: &str = "usage: gw2-map-probe <Gw2.dat> <map_file_id> <physics_x> <physics_y> <lower_z> <upper_z>\nOffline identity-primary-map geometry; hits are not walkability or spawn validation.";
fn bad(s: &str) -> Error {
    Error::new(ErrorKind::InvalidData, s)
}
#[derive(Debug, Clone, Copy)]
enum Source {
    Terrain {
        cell: [u32; 2],
        triangle: usize,
        surface: u16,
    },
    Prop {
        placement: usize,
        mesh: u32,
        triangle: usize,
    },
    Zone {
        placement: usize,
        mesh: u32,
        triangle: usize,
    },
}
impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Terrain {
                cell,
                triangle,
                surface,
            } => write!(
                f,
                "terrain cell={cell:?} triangle={triangle} surface={surface}"
            ),
            Self::Prop {
                placement,
                mesh,
                triangle,
            } => write!(
                f,
                "prop placement={placement} mesh={mesh} triangle={triangle}"
            ),
            Self::Zone {
                placement,
                mesh,
                triangle,
            } => write!(
                f,
                "zone placement={placement} mesh={mesh} triangle={triangle}"
            ),
        }
    }
}
fn consider(
    best: &mut Option<(VerticalHit, Source)>,
    tri: [[f32; 3]; 3],
    xy: [f32; 2],
    range: [f32; 2],
    source: Source,
) {
    // Cheap conservative XY bounds before the full finite/barycentric checks.
    for axis in 0..2 {
        let lo = tri.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min);
        let hi = tri
            .iter()
            .map(|p| p[axis])
            .fold(f32::NEG_INFINITY, f32::max);
        if xy[axis] < lo || xy[axis] > hi {
            return;
        }
    }
    if let Some(hit) = vertical_segment_hit(tri, xy, range)
        && best.as_ref().is_none_or(|(old, _)| hit.z > old.z)
    {
        *best = Some((hit, source));
    }
}

fn read_geometry_chunk(
    schemas: &[PackfileSchema],
    name: &str,
    version: u16,
    bytes: &[u8],
    ptr64: bool,
) -> Result<PfValue> {
    if !matches!(
        (name, version),
        ("parm", 0) | ("trn", 14 | 15) | ("havk", 16)
    ) {
        return Err(bad(&format!("unsupported {name} version {version}")));
    }
    let schema = find_all(schemas, "mapc", name)
        .into_iter()
        .find_map(|s| s.version(u32::from(version)))
        .ok_or_else(|| bad("required schema missing"))?;
    let mut value = read_chunk(bytes, schema, ptr64, Limits::default())?;
    if name == "trn" && version == 14 {
        // Build 209026's v15 upgrade callback (0x140cd6900) initializes the
        // appended +0x38 field to 32. Decode v14's own layout first; a missing
        // field in a v15 chunk must still fail. See the Norn terrain RE note.
        let PfValue::Struct(fields) = &mut value else {
            return Err(bad("terrain schema is not a struct"));
        };
        fields.push(("verticesPerChunkSide".into(), PfValue::U32(32)));
    }
    Ok(value)
}

fn run(path: &str, id: u32, xy: [f32; 2], range: [f32; 2]) -> Result<()> {
    let archive = Archive::open(path)?;
    let pf = Packfile::new(archive.read(id)?)?;
    if pf.r#type() != *b"mapc" {
        return Err(bad("expected mapc packfile"));
    }
    let schemas = parse(include_str!("../../reference/packfiles-208039.xml"))
        .map_err(|e| bad(&format!("embedded schema: {e}")))?;
    let mut values = Vec::new();
    for c in pf.chunks() {
        let c = c?;
        let name = match &c.name() {
            b"parm" => "parm",
            b"trn\0" => "trn",
            b"havk" => "havk",
            _ => continue,
        };
        if values.iter().any(|(n, _)| *n == name) {
            return Err(bad("duplicate required chunk"));
        }
        values.push((
            name,
            read_geometry_chunk(&schemas, name, c.version(), c.bytes(), pf.is_64bit())?,
        ));
    }
    let get = |name| {
        values
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v)
            .ok_or_else(|| bad("required chunk missing"))
    };
    let trn = get("trn")?;
    let terrain = TerrainHeights209026::from_values(get("parm")?, trn)?;
    let tiles = TerrainTiles209026::from_value(trn)?;
    let map = MapCollide::from_value(get("havk")?)?;
    let water = MapWater209026::from_value(get("havk")?)?;
    let Some(PfValue::Dwords(dims)) = trn.get("dims") else {
        return Err(bad("missing dimensions"));
    };
    let dims = [dims[0], dims[1]];
    let cells = match trn.get("verticesPerChunkSide") {
        Some(PfValue::U32(0)) => 32,
        Some(PfValue::U32(v)) => *v,
        _ => return Err(bad("missing cells per chunk")),
    };
    let rect = match get("parm")?.get("rect") {
        Some(PfValue::Floats(v)) if v.len() == 4 => [v[0], v[1], v[2], v[3]],
        _ => return Err(bad("missing rectangle")),
    };
    let bounds = SceneBounds209026::primary_map(
        rect,
        cells,
        Bounds3 {
            min: map.bounds_min,
            max: map.bounds_max,
        },
    )?;
    drop(values);
    let mut best = [None; 3];
    let mut counts = [0usize; 3];
    let mut excluded = 0usize;
    for cy in 0..dims[1] / cells {
        for cx in 0..dims[0] / cells {
            if tiles.chunk_excluded_from_collision([cx, cy]) == Some(true) {
                excluded += 1;
                continue;
            }
            for y in cy * cells..(cy + 1) * cells {
                for x in cx * cells..(cx + 1) * cells {
                    let pair = terrain
                        .primary_map_physics_triangles([x, y])
                        .ok_or_else(|| bad("terrain cell unavailable"))?;
                    let surface = tiles
                        .surface_index([x, y])
                        .ok_or_else(|| bad("surface cell unavailable"))?;
                    for (triangle, tri) in pair.into_iter().enumerate() {
                        counts[0] += 1;
                        consider(
                            &mut best[0],
                            tri,
                            xy,
                            range,
                            Source::Terrain {
                                cell: [x, y],
                                triangle,
                                surface,
                            },
                        );
                    }
                }
            }
        }
    }
    let mut empty = [0usize; 2];
    let mut missing = [0usize; 2];
    let mut invalid_vertices = 0usize;
    for (kind, placements) in [&map.prop_models, &map.zone_models].into_iter().enumerate() {
        for (placement, p) in placements.iter().enumerate() {
            let animation = if kind == 0 {
                map.prop_animation_209026(p)
            } else {
                map.zone_animation_209026(p)
            };
            let Some(animation) = animation else {
                missing[kind] += 1;
                continue;
            };
            if animation.collision_indices.is_empty() {
                empty[kind] += 1;
                continue;
            }
            let matrix = if kind == 0 {
                p.prop_primary_map_physics_matrix_209026()
            } else {
                p.zone_primary_map_physics_matrix_209026()
            };
            if !matrix.iter().flatten().all(|v| v.is_finite()) {
                return Err(bad("nonfinite placement matrix"));
            }
            for &mesh in &animation.collision_indices {
                let m = &map.collisions[mesh as usize];
                let vertices: Vec<_> = m
                    .vertices
                    .iter()
                    .map(|v| matrix.map(|r| r[0] * v[0] + r[1] * v[1] + r[2] * v[2] + r[3]))
                    .collect();
                invalid_vertices += vertices
                    .iter()
                    .filter(|v| !v.iter().all(|v| v.is_finite()))
                    .count();
                for (triangle, indices) in m.indices.as_chunks::<3>().0.iter().enumerate() {
                    let tri = [
                        vertices[indices[0] as usize],
                        vertices[indices[1] as usize],
                        vertices[indices[2] as usize],
                    ];
                    let source = if kind == 0 {
                        Source::Prop {
                            placement,
                            mesh,
                            triangle,
                        }
                    } else {
                        Source::Zone {
                            placement,
                            mesh,
                            triangle,
                        }
                    };
                    counts[kind + 1] += 1;
                    consider(&mut best[kind + 1], tri, xy, range, source);
                }
            }
        }
    }
    if invalid_vertices != 0 {
        return Err(bad("nonfinite transformed mesh vertices"));
    }
    println!("file={id} physics_xy={xy:?} z_interval={range:?} identity_primary_map=true");
    println!(
        "character_origin_bounds={:?} simple_character_bounds={:?}",
        bounds.character, bounds.expanded
    );
    println!(
        "water global_surface_map_z={:?} global_surface_physics_z={:?} explicit_volumes={} (metadata only)",
        water.global_surface_z(),
        water.global_surface_z().map(|z| -z / 32.),
        water.volumes.len(),
    );
    println!(
        "triangles terrain={} props={} zones={} excluded_terrain_chunks={excluded}",
        counts[0], counts[1], counts[2]
    );
    println!(
        "selections props_empty={} props_missing={} zones_empty={} zones_missing={}",
        empty[0], missing[0], empty[1], missing[1]
    );
    for (name, hit) in ["terrain", "props", "zones"].into_iter().zip(best) {
        match hit {
            Some((h, source)) => {
                println!("{name}: z={} normal={:?} source={source}", h.z, h.normal);
                let collision_point = [xy[0], xy[1], h.z];
                println!(
                    "  inside_character_bounds={} inside_simple_character_bounds={}",
                    bounds.character.contains(collision_point),
                    bounds.expanded.contains(collision_point)
                );
                // Server 37, build 209026, before character bounds handling.
                // This is the surface point, not a validated character origin.
                let point = [xy[0] * 32., xy[1] * 32., -h.z * 32.];
                if point.iter().all(|v| v.is_finite()) {
                    println!("  server37_surface_point_xyz={point:?} transform_tag=0");
                } else {
                    println!("  server37_surface_point_xyz=unrepresentable transform_tag=0");
                }
            }
            None => println!("{name}: no triangle hit in interval"),
        }
    }
    println!(
        "coverage: selected triangles only; excludes fallback boxes, blockers, margins, scene clipping and runtime filters"
    );
    Ok(())
}
fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let parsed = (|| {
        if args.len() != 6 {
            return None;
        }
        let id = args[1].parse::<u32>().ok()?;
        if id == 0 {
            return None;
        }
        let nums = args[2..]
            .iter()
            .map(|s| s.parse::<f32>().ok())
            .collect::<Option<Vec<_>>>()?;
        if !nums.iter().all(|v| v.is_finite()) || nums[2] > nums[3] {
            return None;
        }
        Some((id, [nums[0], nums[1]], [nums[2], nums[3]]))
    })();
    let Some((id, xy, range)) = parsed else {
        eprintln!(
            "{USAGE}\nRequire a positive file ID, finite coordinates and lower_z <= upper_z."
        );
        return ExitCode::from(2);
    };
    match run(&args[0], id, xy, range) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("map probe failed: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terrain_bytes(version: u16, cells: u32) -> Vec<u8> {
        let header = if version == 14 { 56 } else { 60 };
        let samples = (cells as usize + 3).pow(2);
        let words = cells as usize * cells as usize / 16;
        let mut bytes = vec![0; header + samples * 4 + words * 4];
        bytes[0..4].copy_from_slice(&cells.to_le_bytes());
        bytes[4..8].copy_from_slice(&cells.to_le_bytes());
        bytes[12..16].copy_from_slice(&(samples as u32).to_le_bytes());
        bytes[16..24].copy_from_slice(&((header - 16) as i64).to_le_bytes());
        bytes[24..28].copy_from_slice(&(words as u32).to_le_bytes());
        bytes[28..36].copy_from_slice(&((header + samples * 4 - 28) as i64).to_le_bytes());
        if version == 15 {
            bytes[56..60].copy_from_slice(&cells.to_le_bytes());
        }
        for sample in bytes[header..header + samples * 4].chunks_exact_mut(4) {
            sample.copy_from_slice(&(-64_f32).to_le_bytes());
        }
        bytes
    }

    #[test]
    fn upgrades_v14_and_preserves_explicit_v15_cell_size() {
        let schemas = parse(include_str!("../../reference/packfiles-208039.xml")).unwrap();
        let parm = PfValue::Struct(vec![(
            "rect".into(),
            PfValue::Floats(vec![0., 0., 3072., 3072.]),
        )]);
        for (version, cells) in [(14, 32), (15, 16)] {
            let trn = read_geometry_chunk(
                &schemas,
                "trn",
                version,
                &terrain_bytes(version, cells),
                true,
            )
            .unwrap();
            assert_eq!(trn.get("verticesPerChunkSide"), Some(&PfValue::U32(cells)));
            let heights = TerrainHeights209026::from_values(&parm, &trn).unwrap();
            let tiles = TerrainTiles209026::from_value(&trn).unwrap();
            assert_eq!(heights.height_at([150., 150.]), Some(-64.));
            assert_eq!(tiles.surface_index([cells - 1, cells - 1]), Some(0));
            assert_eq!(tiles.surface_index([cells, 0]), None);
            assert_eq!(
                heights.primary_map_physics_triangles([0, 0]).unwrap()[0][2],
                [96. / cells as f32, 96., 2.]
            );
        }
    }

    #[test]
    fn rejects_unknown_versions_and_truncated_terrain() {
        let schemas = parse(include_str!("../../reference/packfiles-208039.xml")).unwrap();
        for (name, version) in [("trn", 13), ("trn", 16), ("parm", 1), ("havk", 15)] {
            let error = read_geometry_chunk(&schemas, name, version, &[], true).unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("unsupported {name} version {version}")
            );
        }
        for version in [14, 15] {
            let mut bytes = terrain_bytes(version, 32);
            bytes.pop();
            assert!(read_geometry_chunk(&schemas, "trn", version, &bytes, true).is_err());
        }
    }
}
