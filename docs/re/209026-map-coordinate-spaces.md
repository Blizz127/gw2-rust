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
The transform is supplied by the map-load request. The primary-map path
below supplies identity; other queued entries need not do so.

`0x1409bf1f0` checks bounds ±2,097,151, adds float 14,680,064, takes the
low 22 bits of the float representation, subtracts 2,097,152 and converts
the resulting integer to float. Under the usual nearest-even SSE rounding
mode this rounds to an integer. No scale conversion occurs here; the
runtime rounding mode was not captured.

The no-packfile fallback divides dimensions by float **3072** at
`0x141c7d4d8`, checks exact chunk multiples and constructs centered bounds
using +3072/-3072. Assertions name `chunkDims` and `chunkWorldSize`.
This agrees arithmetically with 32 cells times 96 units.

## Primary map's transform path

The default-view load path identified by `VdfLoad.cpp` at
`0x14095225e` initializes a 48-byte matrix through `0x1409c6dd0`.
That helper copies the on-disk global at `0x142584848`, whose three rows
are `[1,0,0,0]`, `[0,1,0,0]`, `[0,0,1,0]`.

The loader appends a 56-byte entry containing the current map-file reference
and that identity matrix to an existing request list. Its loop passes each
entry's matrix at +8 to `0x140c342a0` as the fifth argument. The chain is:

1. `0x140c342a0` forwards it as argument six to `0x140c392c0`.
2. `0x140c392c0` copies it into request object +0x230.
3. `0x140c39582` passes that matrix to map constructor `0x140c34850`.
4. The map constructor copies it into map +0x14 and +0x44, then processes
   the second copy with `0x1409c74b0` (inverse-like math; not fully traced).
5. Map vtable `0x141d61b68` virtual +0x180 is `0x140c35310`, returning
   map +0x14. This is the matrix used by the `parm` rectangle loader.

Thus the appended primary map receives identity on this static path;
no additional rectangle scale, axis flip or offset is introduced by that
matrix. The pre-existing queued entries can carry other transforms.
This does not establish API landmark orientation, physics mesh units or
the coordinates expected by mission messages, and has not been exercised
in a live zone.

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
were not included. The runtime prop translation conversion is traced below;
mesh-local vertex units and rotation conventions remain unresolved.

The nine `msn` interest points also have coordinates on the larger scale;
their descriptive labels do not establish exact correspondence to cached
API landmarks. They are not proven player spawn positions.

## Prop translation into the collision scene

`CoScene.cpp` loader `0x1410b5b8c` obtains the `havk` chunk. It reads
the stored bounds directly, while multiplying map-rectangle XY values by
the float at `0x14192df28`: **0.03125 = 1/32**. It passes the chunk and
the map's virtual +0x180 matrix to `0x1410b8180`.

That consumer iterates `propModels` using count +0x5c, pointer +0x60 and
48-byte records. This matches the schema: token +0, sequence +8, scale
+0x10, translation +0x14, rotation +0x20, geometry index +0x2c.
It tests the map matrix with `0x140c915e0`, which compares all twelve
elements against the identity matrix.

For identity, `0x1410b8648` loads translation XYZ, flips Z's sign using
`0x80000000` at `0x141936e00`, multiplies all three by 1/32, and packs
`[x/32, y/32, -z/32, 0]` into XMM7. For a nonidentity matrix,
`0x1410b868b` first computes each row's affine product:

```
t[i] = M[i][0]*x + M[i][1]*y + M[i][2]*z + M[i][3]
collision_translation = [t[0]/32, t[1]/32, -t[2]/32]
```

At `0x1410b87f2` it writes this translation into the transform passed
to prop constructor `0x1410be5e0`; model scale is passed separately.
Consequently the prop's own scale does not multiply its placement origin.
This is a traced conversion for **prop translations**, not permission to
apply the same operation indiscriminately to all `havk` fields. The stored
bounds are already in the collision scene's smaller units. Rotations and
mesh-local vertex conversion still require their own traces.

`ModelPlacement::prop_physics_translation_209026` accepts the map matrix
explicitly and implements only this translation operation. Its focused
test exercises identity and a matrix with axis mixing, translation and
Z scaling; the prop's own scale and rotation do not alter the origin.
All four focused map-reader tests and the library/inspector build pass.

A REA smoke of the compiled Rust helper converts all **8,301** Queensdale
prop origins to finite collision translations. All 8,301 lie within the
stored collision bounds. Origin containment is a diagnostic consistency
check; it does not prove mesh containment or ground-height accuracy.
The same smoke repeats the 4,113 exact render/collision placement joins.

## Implementation consequence and evidence

Keep the API continent↔map conversion as it is. Do not add a blanket
map↔collision/world conversion based on rectangle size or bounds alone.
An exact API/archive point correspondence or a controlled mission-position
capture after the UI hold clears is still needed for packet coordinates.

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

Transform follow-up bundle:
`~/re/gw2/notes/209026-map-coordinate-transform-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_3d9c4c1c2fd1ab7b59b18fd621a02cf71e6983ff221a70656fe53e895a95d4d7` | Rectangle rounding helper. |
| `ev_6a8c82e39f2354f8e65875f3754ea4f819aab305e0acdcfebe5a03125ec81642` | Rounding range and addend constants. |
| `ev_ee5269b112969ab745b6e65ccde5f534df164ee98889bdcc3eb56b7f57baac46` | Map vtable +0x180. |
| `ev_8088582bac2c5d5ce258c26b8940fda8f2616ba502b6ae0f9fcfbbda0326dda2` | Getter returns map +0x14. |
| `ev_335e168334de20ff561e474a662f35d44a76fc8ff334c233be0f3484fa5a65ee` | Map constructor copies supplied matrix. |
| `ev_ba2749ff107ba283b7b2d03ccbff922173ca24f8f2310b343aa4322065864c08` | Load-request matrix storage and constructor call. |
| `ev_a188a89c6cb0d82829560269073731f775a6cf6184296d9f593aa6c1cd945e28` | Constructor argument positions. |
| `ev_58b6d94c427fa44fc9ae1708ded90b5e272eef07353cec9080ff027c2caf817a` | Request wrapper forwarding. |
| `ev_b7f592b1b831252d35b4f2987aff84dcb0f174af058e36e2da0f2abb755f334a` | Primary-map entry append and request loop. |
| `ev_f4b580baacc50c6857eb0613ff81e8f6033706819d4f58d5ebb775c24e91f989` | Matrix initializer and current map-file reference. |
| `ev_95aa39f8ddb37cb09250ee69e8776c3cfdaad5dccbbb3556f6363ccb0596e5a0` | Identity global and VdfLoad source string. |

Collision follow-up bundle: `~/re/gw2/notes/209026-collision-translation-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_5e8a5c481b15697130b86c484322cb6e2af21518143085df672f9e3c03c84cf3` | Hash-guarded helper finds direct `havk` references after two shell-pipeline search timeouts. |
| `ev_6f618d0905bba6b6a1012cff1ba8db72dca0111f9cef32302c24090ed2128294` | Chunk load, bounds handling and map-matrix forwarding. |
| `ev_3a70f5f8783f3711888330d64b76f1460883f7cef73814642da3fdb0d441c928` | 1/32 constant, CoScene source path and prop-array layout. |
| `ev_90ec0300bde93c5e8ad461056de49fc7afb50f847cf6a56c27cd4482aa323eff` | Identity test, affine translation conversion, Z sign mask and prop construction. |

Rust smoke bundle: `~/re/gw2/notes/209026-collision-translation-rust-1.json`;
record `ev_baac54c10b9c703d6690afc00ac8a24333959dc4da93ecdbcaa0e63004b9177b`.
Private helper source: `~/re/gw2/notes/check-map-prop-translation-rust.rs`.
