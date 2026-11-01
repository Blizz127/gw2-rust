# Build 209026: Queensdale coordinate spaces

Static REA and read-only archive comparisons, 2026-11-01. Target identity is
recorded in [client-build.md](../client-build.md). No client launch, mission
join or packet-coordinate validation occurred.

## Rectangle and terrain

Queensdale archive file **192711** has a `parm` v0 rectangle
`[-43008, -27648, 43008, 30720]`, identical to the cached API map-15
rectangle. Its `trn` v15 dimensions are `[896, 608]`, with 32 vertices per
chunk side and 532 chunks. Rectangle width and height divided by those
dimensions both give **96**. Also, `(896 / 32) * (608 / 32) = 532`.
These are archive/arithmetic observations, not a verified terrain sampler.

The map-parameter loader around `0x140c3c5bc` requests `parm`, copies its
rectangle to object +8 and +0x18, flags to +0x28 and GUID to +0x2c.
It then obtains a transform through the owner object's virtual +0x180 and
calls `0x140c3c720` on the +8 rectangle. That helper transforms all four
corners at Z=0, takes XY bounds and applies `0x1409bf1f0` to each bound.
The transform's producer and the final helper's numeric semantics remain
untraced; do not assume the transform is identity.

The no-packfile fallback divides dimensions by float **3072** at
`0x141c7d4d8`, checks exact chunk multiples and constructs centered bounds
using +3072/-3072. Assertions name `chunkDims` and `chunkWorldSize`.
This agrees arithmetically with 32 cells times 96 units.

## Collision placements are not collision bounds

The `havk` v16 bounds are approximately
`[-1424.68, -1048.97, -252.60]..[1372.17, 1066.63, 480.97]`.
Their smaller magnitude does **not** justify scaling every field in this
chunk by 32.

A comparison joins `prp2` v21 `propArray`/`propAnimArray` GUIDs to `havk`
`propModels` tokens, using the historical schema field names:

| Check | Result |
| --- | --- |
| Direct render placements | 6,216 |
| Duplicate GUIDs in those two arrays | 0 |
| Collision prop placements | 8,301 |
| Joined collision placements | 4,113 |
| Joined XYZ position/translation values exactly equal | **4,113** |

For example, both tables store
`[22945.0859375, -2267.200439453125, -1821.687744140625]` for one matched
placement. This is equality of decoded float values, without a scale,
translation or axis flip. Instance-array transforms and unmatched props
were not included. Mesh-local vertex units, bounds units, rotations and
runtime transform composition remain unresolved.

The nine `msn` interest points also have coordinates on the larger scale;
their descriptive labels do not establish exact correspondence to cached
API landmarks. They are not proven player spawn positions.

## Implementation consequence and evidence

Keep the API continent↔map conversion as it is. Do not add a blanket
map↔collision/world conversion based on rectangle size or bounds alone.
Next useful evidence is the map transform producer, physics placement
consumer, or a controlled mission-position capture after the UI hold clears.

Private helper: `~/re/gw2/notes/check-map-placement-scale.rs`; its compiled
binary was run through REA. It asserts all 4,113 joins are exact and that
the render GUIDs are unique. No extracted assets are committed.

Private bundle: `~/re/gw2/notes/209026-map-coordinate-spaces-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_3dbb6fb6bd09436e43982105935540f60465d7dd80f1ee6f378ff0ac66df0b6a` | Rectangle, terrain dimensions and chunk counts. |
| `ev_82a14710d89f686be12f1632d34d05837e2b88dc5858f47a7f1fb01c2e820fa3` | MapParams source-string references. |
| `ev_00047d7636e4dcd061cd2d354c37020c708c5765e88b7a0458a258f875a275cd` | Rectangle loader, transform helper and chunk-size constants. |
| `ev_272ce5767270e2038c4fdbef120e3434eff878d6ef4b58edac3875c2e9ff37e3` | Queensdale collision/render chunk versions and counts. |
| `ev_40cc99e704365e20cacdb26830222819355d5cd860aaf91fa1e0c7c8671ffa4f` | Exact matched-placement assertions and mission points. |
