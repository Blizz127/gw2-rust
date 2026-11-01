# Build 209026 collision water records

REA static analysis and offline Queensdale archive audit, frozen executable
SHA-256 `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This supplies the water input used by the [character height routine](209026-character-ray-mask.md).

## Loader

Scene loader calls `0x1410cffa0` with its `+0x6e0` water collection and the
havk v16 data. It reads pack flags at `+0x18`, global `waterSurfaceZ` at
`+0x1c`, and water-volume count/pointer at `+0x74/+0x78`.

If pack flag bit 0 is set, it inserts an implicit global volume with XY bounds
`[-FLT_MAX,+FLT_MAX]`, Z range `[waterSurfaceZ,+FLT_MAX]`, and runtime flag
bit 1 set to bypass polygon checks. It records this global volume's index;
without that flag the index is -1. Global surface presence must therefore
not be inferred merely from a stored float being present or nonzero.

Explicit volume records have 40-byte stride: GUID `+0`, name pointer `+8`,
flags `+0x10`, vertical range `+0x14/+0x18`, vertex count/pointer `+0x1c/+0x20`.
The loader copies Float2 vertices unchanged and computes their XY extrema.
It copies both vertical-range floats unchanged to runtime bounds. These fields
remain in map units, unlike cached collision-mesh vertices. Runtime flag bit 0
copies source flag bit 0; other source bits are not copied by this loader.
The loader builds inter-volume references with `0x1410ce780` after loading.
Their overlap/exclusion semantics are not yet fully traced.

## Height query scope

`0x1410cfb40` tests XY bounds and polygon containment (except implicit-global
flag bit 1), selects vertical boundaries according to flag bit 0, and consults
`0x1410d0410` before accepting a candidate boundary. It can return sentinel
1,000,000,000 when no applicable boundary exists. This is not simply a maximum
of `verticalRange` values. A complete explicit-volume query is not implemented.

For a collection containing only the implicit global volume, the query's
boundary selection returns its global surface for finite positions within
its finite bounds. The typed reader exposes the flag-controlled global
surface and raw explicit volumes separately. Its API does not claim a complete
water-height or containment query.

## Queensdale

File 192711 has havk flags **1**, global water surface **0 map units**, and
**zero explicit water volumes**. Thus the loader creates one implicit global
water volume. The earlier geometric probe did not include water as a solid
triangle surface; that distinction remains intentional. The CLI reports
water metadata alongside its geometric hits.

The typed reader uses the bounded schema output and validates field
types, finite coordinates and ordered vertical ranges. Unknown raw flag bits
are preserved. Synthetic records cover explicit volumes because Queensdale
cannot validate that nonempty archive path.

## Evidence

Private bundle: `~/re/gw2/notes/209026-water-volumes-1.json`.

- Loader call site: `ev_f3166adac26b88b0a09520dc326cad611d84af91cf1c43b83f9f0341200425f1`.
- Water loader: `ev_152cb6c25056c10e2704d4863fdae0a5087ddd0b2e77d128b91c7f610cead62b`.
- Boundary query: `ev_fb8663f6f25784782b42cdff2d24f261f2d15c988500acc3f74998e8d0773950`.
- Queensdale raw-schema audit: `ev_dfe65391e453a4cd7690dff6698fefde9ddd84d6810c2a60b403f2651a43de7e`.

## Implementation verification

`water::MapWater209026` decodes the metadata and provides `global_surface_z()`
with the flag gate. `gw2-map-probe` prints the global surface in map/collision
units and the number of explicit volumes. Two focused synthetic tests pass,
covering enabled/disabled global water, raw explicit records, and malformed or
nonfinite inputs. The library and probe build pass. REA's real archive smoke
reports global map Z 0, collision Z -0, zero explicit volumes, and the unchanged
terrain hit/triangle counts. No nonempty real-volume archive case was run.

- Updated CLI smoke: `ev_c6df9102b5501cd6ec5c91bbff579ecb5c03317c21425fa261d3318a7d89cd73`.
