# Build 209026 terrain height lookup

Static REA trace of the frozen client and Queensdale archive file 192711.
Executable SHA-256: `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This documents the resident `mapc/trn` height query in map-local coordinates;
it does not establish mission packet coordinates or complete walkable ground.
No client session or User Agreement progression was involved.

## Layout and coordinates

Queensdale `parm` v0 rectangle is `[-43008,-27648,43008,30720]`.
The `trn` v15 fields decode with the historical 208039 schema:
`dims=[896,608]`, `verticesPerChunkSide=32`, 651,700 height floats,
34,048 tile flag words and 532 chunk records. Despite its schema name,
`verticesPerChunkSide` is the number of **cells** per chunk in this query.

The loader at `0x140c43385` reads that field at pack offset `+0x38`,
substituting 32 when zero. Constructor `0x140c3d690` divides the grid dimensions
by it, passes it to the height-cache constructor `0x140c41a50`, and stores
integer `3072 / cellsPerChunk` as cell spacing at object `+0x1cc`.
Vtable `0x141d62668` slots `+0x90` and `+0xb0` read cells per chunk and spacing
through `0x140282080` and `0x140c3da90` (interface pointer is object `+0x20`).
Queensdale therefore has 28×19 chunks and 96-unit sample spacing.

The layout vtable `0x141d62ca0` points to `0x140c425d0` for row stride:
`cellsPerChunk + 3`. Its first method `0x140c42590` returns stride squared.
The resident loader at `0x140c42116` copies floats unchanged from:

```
heightMapArray + (chunkY * chunksX + chunkX) * stride²
```

Cache acquisition `0x140c42de0` stores a pointer advanced by `stride + 1`
floats. Thus each 32-cell chunk has 35×35 samples including a one-sample
border around its 33×33 vertices. Sample X increases rightward and sample Y
downward from the rectangle's maximum Y. Chunk-origin helper `0x140c45fc0`
uses `x = minX + chunkX*3072`, `y = maxY - chunkY*3072`.

## Point query

`0x140c471b0` accepts only `minX <= x < maxX` and `minY <= y < maxY`.
It computes:

```
dx = x - minX; dy = maxY - y
ix = floor(dx / spacing); iy = floor(dy / spacing)
cx = min(ix / cellsPerChunk, chunksX - 1)
cy = min(iy / cellsPerChunk, chunksY - 1)
lx = ix - cx*cellsPerChunk; ly = iy - cy*cellsPerChunk
base = (cy*chunksX + cx)*stride² + (ly+1)*stride + lx+1
fx = (dx - ix*spacing) / spacing
fy = (dy - iy*spacing) / spacing
```

The floor helper `0x1409bef60` rounds nonnegative floats using the
12,582,912 bias, then subtracts one when the rounded integer exceeds the input.
It asserts input at most 4,194,303. The four heights are at `base`, `base+1`,
`base+stride`, `base+stride+1` (h00, h10, h01, h11).

Interpolation splits on the northeast–southwest diagonal, using float arithmetic:

```
fx + fy < 1: (h10-h00)*fx + h00 + (h01-h00)*fy
otherwise:  (h01-h11)*(1-fx) + h11 + (h10-h11)*(1-fy)
```

The diagonal takes the second branch. The maximum-Y edge is excluded and
minimum-Y edge included: the latter uses the final chunk's `ly=32`, supported
by its stored border. The query returns the raw height without scale/sign
conversion. Wrapper `0x140c443d0` transforms input XY using the inverse map
matrix before calling it; the primary map's traced load matrix is identity.
Tile flags are not consulted by this height-only query. Holes, prop collision,
water, bridges and actual walkability require separate handling.

## Archive check and limitations

All 651,700 Queensdale height floats are finite, ranging from -5768.9424 to
1701.4622. Comparing the three overlapping rows/columns of adjacent chunks
produces 106,785 comparisons: 106,782 exact matches and three differences.
All three are between chunks (14,0) and (15,0), in stored row 0 (outside the
rectangle above its maximum Y), with maximum difference 99.038086. Preserve
chunk-local samples; do not deduplicate or require every border copy to match.
The initial audit's zero-difference assertion failed; the follow-up localized
and recorded these three differences. No in-client height replay has run.

## Evidence

Private REA bundle: `~/re/gw2/notes/209026-terrain-height-1.json` (759 records).
The private Rust helper `check-terrain-layout.rs` reads only archive file 192711
and was executed through REA. Main records:

- Query and resident float copy: `ev_8ce0cbcaf082cfad892c0d0027dd6f5924ac905f68b578b88632f538e558f66b`.
- Cache border offset: `ev_be76166ef719cc14dc30f2453ea17b4d560ae79f6b0a4101253a5d08d1bb705c`.
- Layout methods: `ev_30aca0ed504cbd7af8dafa408571c611a1c091435756e7b58572872998a66502`.
- Terrain constructor/loader: `ev_4cb1aaf59655f5aacd285b783f97bdf42f0b49e6771cb91875d87ed3fcef90fa`.
- Floor helper/constants: `ev_66ddd3a45a434e1a1d93cb48667e396107d39ec5cd01af201df18bd5377b7ab7`.
- Coordinate wrapper: `ev_5171eea42ffc18f9dad06583956c3035e43d45efd8e91dab20011b2438d3a9de`.
- Archive border audit: `ev_a315622262bec5c4e4981228d05d8627c4d61dc99ab05bb6763e2c3713a30a38`.
