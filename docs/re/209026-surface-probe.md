# Offline primary-map surface probe

This is an emulator diagnostic built from the documented
[terrain triangles](209026-terrain-collision-basis.md),
[prop transforms](209026-collision-rotation.md),
[prop sequences](209026-collision-animation-selection.md), and
[zone sequences](209026-zone-collision.md). Its vertical triangle-intersection
algorithm is our own geometry code, not a traced client query implementation.

The command is:

```
gw2-map-probe <Gw2.dat> <map_file_id> <physics_x> <physics_y> <lower_z> <upper_z>
```

It reads only `parm` v0, `trn` v15 and `havk` v16 in a `mapc` packfile,
with the existing bounded schema reader. For each source (terrain, props,
zone models), it reports the highest triangle intersection in the inclusive
vertical interval, normal and source indices. Positions use collision units;
for the identity primary map these correspond to map-local `[x/32,y/32,-z/32]`.
These coordinates are not established mission packet positions.

The probe considers both triangle faces, preserves normals from vertex winding,
and uses double precision barycentric arithmetic with a 1e-9 edge tolerance.
Terrain chunks marked collision-excluded are skipped. Selected prop and zone
mesh triangles use their verified placement matrices. Missing/empty selections
are counted explicitly. Fallback boxes, blockers, collision margins, dynamic
state, scene clipping and runtime query/filter predicates are outside its scope.
A reported hit is a geometric surface candidate, not a safe spawn or walkability
claim. No network, client launch or UI action is needed.

Real archive execution must go through REA. Synthetic geometry tests and build/
CLI argument checks use ordinary development tools. Results are recorded below.

## Verification

Two focused synthetic geometry tests pass, covering slopes, winding, edges,
interval bounds and invalid input. The library and CLI build pass. Both help
forms and six invalid argument cases return the expected exit codes without
opening an archive.

REA ran the CLI against Queensdale file 192711 in the frozen archive. Each
query scanned 1,089,536 terrain, 911,224 prop and 92,367 zone triangles.
No terrain chunks were excluded. There were 62 empty prop selections, zero
missing prop selections, and no empty or missing zone selections.

| Query physics XY | Z interval | Hit | Source |
| --- | --- | --- | --- |
| `[0,0]` | `[-1000,1000]` | terrain `-0.23037942` | cell `[447,319]`, triangle 1, surface 0 |
| `[1277.1185,-195.63243]` | `[103.37971,103.39971]` | prop `103.38995` | placement 0, mesh 899, triangle 0 |
| `[756.1263,402.12057]` | `[64.88541,64.90541]` | zone `64.89612` | placement 0, mesh 77, triangle 0 |

The two mesh query points were selected by the private helper
`check-surface-probe.rs` from triangle centroids. Their steep normals amplify
rounding of the XY centroid; the returned Z values are intersections at the
rounded XY input, not exact centroid Z. Each other source reported no hit in
that query interval. These are coverage smoke checks, not client movement
validation.

Private evidence bundle: `~/re/gw2/notes/209026-surface-probe-1.json`.
The frozen executable hash remained unchanged for both REA commands.

- Center query: `ev_eb9d7943fa86e2b49054b81bab732450cc68e1cbfadc0ce539413c582649f0b0`.
- Prop and zone queries: `ev_f2cf685c6677ffc70e684276d974188ac76194efaf0e033f24615a7bb5b09e46`.
