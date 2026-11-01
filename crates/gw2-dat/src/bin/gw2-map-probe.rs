//! Offline vertical-surface diagnostic for primary-map build-209026 geometry.
use gw2_dat::{
    archive::Archive,
    mapc::MapCollide,
    packfile::Packfile,
    pfread::{Limits, PfValue, read_chunk},
    pfschema::{find_all, parse},
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
    if let Some(hit) = vertical_segment_hit(tri, xy, range) {
        if best.as_ref().is_none_or(|(old, _)| hit.z > old.z) {
            *best = Some((hit, source));
        }
    }
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
        let (name, version) = match &c.name() {
            b"parm" => ("parm", 0),
            b"trn\0" => ("trn", 15),
            b"havk" => ("havk", 16),
            _ => continue,
        };
        if c.version() != version {
            return Err(bad("unsupported parm/trn/havk version"));
        }
        if values.iter().any(|(n, _)| *n == name) {
            return Err(bad("duplicate required chunk"));
        }
        let candidates = find_all(&schemas, "mapc", name);
        let schema = candidates
            .iter()
            .find_map(|s| s.version(version as u32))
            .ok_or_else(|| bad("required schema missing"))?;
        values.push((
            name,
            read_chunk(c.bytes(), schema, pf.is_64bit(), Limits::default())?,
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
                for (triangle, indices) in m.indices.chunks_exact(3).enumerate() {
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
