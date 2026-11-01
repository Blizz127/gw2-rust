# Build 209026 terrain surface selectors and collision exclusion

REA static trace of frozen build 209026, executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This extends [terrain height lookup](209026-terrain-height.md). No client replay
or agreement progression was performed.

## Surface selectors

Despite its historical schema name, `trn.tileFlagArray` stores two-bit surface
selectors in the traced query, not individual collision-hole flags. Loader
`0x140c42938` checks `wordCount == dimsX*dimsY >> 4` and copies the words.
Accessor `0x140c427d0` uses the global cell coordinates, in the same left-to-right,
top-to-bottom order as the height grid:

```
i = cellY*dimsX + cellX
selector = (tileFlagArray[i/16] >> (2*(i%16))) & 3
```

This accessor permits a coordinate equal to its dimension by decrementing it
before indexing. Its world-coordinate caller `0x140c44160` has a different
boundary path: it transforms XY, converts to integer grid coordinates, and
rejects grid coordinates outside `[0,dims)` before calling the accessor.
Do not assume the height query's boundary policy applies to every terrain API.

The caller computes `chunkXY = cellXY / cellsPerChunk`, passes the selector
into `0x140c42720`, and returns a 16-bit entry from that chunk's four-entry
surface table. Loader `0x140c429e0` accepts surface table counts 0 or 4 and
copies the words to runtime chunk offset `+4` (runtime chunk stride 12).
The table is zero-initialized at construction, so an absent table returns 0.
The numeric surface IDs are preserved; their material names are not inferred.
`surfaceTokenArray` is not consumed by this traced loader.

## Chunk collision exclusion

The loader maps raw `chunkFlags` to runtime flags as:

```
runtime = ((raw & 1) == 0 ? 1 : 0) | (raw & (2|4|8|32))
```

`0x140c423c0` reads those runtime flags by chunk coordinate. Terrain interface
vtable `0x141d62668`, slot `+0x30`, dispatches through `0x140c445d0` to it.
Collision-scene terrain builder `0x1410b63a0` obtains that terrain interface
from map component `0x17`, walks the grid in cells-per-chunk increments, calls
slot `+0x30` at `0x1410b660e`, and **skips the chunk when bit `0x4` is set**.
Because the loader preserves bit 4, the archive's raw flags suffice for this
specific exclusion rule. Bit 0 is inverted and must not be treated as the same
raw/runtime value. Other flag meanings remain unassigned.

For an included chunk, the builder also requires intersection with the scene's
XY rectangle before calling terrain-body constructor `0x1410be760`.
That constructor configures a sampled heightfield with horizontal scale
`spacing/32` and vertical scale `1/32`, and attaches a height iterator. Its
body transformation includes an additional quaternion composition, not yet
fully traced. The exclusion bit alone does not establish a full world-space
collision surface, walkability, slope limits, water behavior or prop coverage.

## Queensdale audit

Archive file 192711 (`trn` v15), through REA:

- 532 chunks: 520 have raw flags 0, 12 have raw flags 1; none set bit 4.
- Every chunk has four surface indices and an empty surface-token array.
- Cell-selector counts for 0,1,2,3: 218843, 35862, 79147, 210916.
- Resolved surface-ID counts: 0→220192, 1→92751, 2→220398,
  3→10244, 4→961, 5→201, 6→21. No selected table slot is absent.

## Evidence

Private bundle `~/re/gw2/notes/209026-terrain-flags-1.json` and private helper
`check-terrain-flags.rs` (executed through REA).

- Packed selector, surface accessor, flag/table loader:
  `ev_6685aab4ef50bfc0a7de10cf36efca84029169ed83278ec6e122fb8c916f9841`.
- World-coordinate surface query:
  `ev_78e51c114ab2347f555bf884361b3783433c0af642d63ca7b264700818e43d78`.
- Chunk getter and vtable:
  `ev_4178df9da84ce3f78a8a54bfd65bea9d445b1bcba893fd5255d12b7b46b23f50`,
  `ev_2c90c112f170041026ab0302463d962136df99caa6483223da9a698624840b77`.
- Collision builder skip condition:
  `ev_c55191f3c8fa29fbbed710336bb4c79ed4ff4917fceab11d5839a1207e1bd7f6`.
- Terrain body construction:
  `ev_82036e3a46541ba1860797484941761cd7dad64f1331d098be683d65eb2a8bc6`.
- Archive counts:
  `ev_64b1bd2569ab5a772026cd847391fbb6e0b06a4a290f9250b8574b6ea3e08e4f`.

## Typed implementation

`terrain::TerrainTiles209026` decodes the packed surface selectors and chunk
metadata separately from the height sampler. Integer cell surface lookup
rejects out-of-grid coordinates; chunk exclusion reports only the traced bit-4
rule. Absent chunk/surface tables default to zero as in the client. Counts and
types are validated before indexing, and unknown raw flag bits are retained.

All six focused terrain tests and the library/inspector build pass. A REA run
of private `check-terrain-flags-rust.rs` compared all 544,768 cell surface IDs
and all 532 chunk exclusions against decoded archive fields, with exact matches.
Evidence `ev_990c3c1243ce91181fa53a755d6c0b80e8442630e65120b4d420d1d584afdf2f`
is saved in private bundle `209026-terrain-flags-rust-1.json`.
