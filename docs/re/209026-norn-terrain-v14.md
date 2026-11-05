# Build 209026: Norn terrain v14 surface diagnostic

The Norn map archive **187611**, shared by map keys 379 and 28, has `parm`
version 0, **`trn` version 14**, and `havk` version 16. The offline map probe
previously required terrain version 15 and rejected this archive before
querying geometry. The probe now decodes v14 with its own schema and applies
the client's explicit field initialization. Other unsupported versions still
fail with the chunk name and version in the error.

## Client compatibility evidence

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

The `trn` descriptors at `0x141e511b0`, `0x141e512d0` and `0x141e51460`
all reference a 16-entry version table at **`0x141e50a30`**. Its version-14
row references schema **`0x141ecb320`**; version 15 references
**`0x141ecb400`** and callback **`0x140cd6900`**.

The v14 and v15 schemas have identical first six fields, including identical
array/nested-schema references. Their packed sizes are 56 and 60 bytes.
The sole additional v15 field is the Dword **`verticesPerChunkSide`** at
**+0x38**. The complete callback at `0x140cd6900..0x140cd690d` tests the
output pointer in r8 and, when nonnull, writes **32** to `[r8+0x38]`.
The terrain loader requests version **15** at `0x140c43356` and reads this
field at `0x140c43385`; its existing zero-to-32 rule is unchanged.

The probe mirrors this compatibility step only after decoding a v14 chunk.
It does not treat a missing field in arbitrary input as permission to invent
a default. The strict terrain readers and v15 path remain unchanged.

## Frozen archive and diagnostic result

Decoded archive 187611 SHA-256:
`2dc9c92929362a10cdb22b5350957eac764ccc1d53c8a97b01c5798223211bbc`.
Terrain payload SHA-256:
`17db1f9f87956992815bc97e5a79bd13b49376d6da232e0fc68f9dc1a92e4e30`.
Its dimensions are **448 × 1152** cells, with **617,400** resident height
floats, **32,256** surface-selector words and **504** chunk records. These
counts agree with 14 × 36 chunks, each with a 35 × 35 bordered height grid.

Running the corrected probe at physics XY **[0,0]**, with vertical interval
[-1000,1000], returned:

| Archive | Terrain physics Z | Server-37 surface point | Terrain source |
| ---: | ---: | --- | --- |
| 187611, Norn | 61.488403 | `[0,0,-1967.6289]`, tag 0 | cell [223,575], triangle 1, surface 2 |
| 192711, Queensdale | -0.23037942 | `[0,0,7.3721414]`, tag 0 | cell [447,319], triangle 1, surface 0 |

Both surface points are inside the queried character and expanded character
bounds. Neither query hit a selected prop or zone triangle at that XY.
Norn has three prop placements without a selected animation and 23 with empty
collision selections; the existing probe reports these coverage limits.

This is an offline surface query, **not a retail start location, validated
character origin, walkability result or in-client spawn replay**. It excludes
fallback boxes, blockers, margins, scene clipping and runtime filters.

## Verification and retained evidence

Two focused tests pass: decoding v14 then sampling the terrain/surface metadata,
preserving a nondefault v15 cell size, rejecting unsupported versions and
rejecting truncated arrays. The probe build and both real-archive REA probe
runs pass. `rustfmt --check` and `git diff --check` pass.

Private bundle: `~/re/gw2/notes/209026-norn-terrain-v14-20261105.json`.
Archive identity capture:
`~/re/gw2/notes/209026-norn-terrain-archive-20261105.json`.

| REA evidence | Scope |
| --- | --- |
| `ev_1c23548fa46d4b912230fda1bec17e21646a56babeb329be472860f57f8ee1b8` | Hash-bound v14/v15 schema rows and shared references. |
| `ev_c808cb153a1c96c529402eff42bd920769f7a06455571b964ae966f8ddda9b1f` | Chunk/version table binding and requested version 15. |
| `ev_159d2ce9a2f263e32d804e2c631a3092a6880863a1d7e0795c088dc8880d8c8b` | Complete initialization callback and bounded loader instructions. |
| `ev_b203cb77cdac50ab296a05fcd81d638207092cfae9a7a14b984366ff76732987` | Decoded archive/chunk hashes, versions and terrain counts. |
| `ev_ffd9c33ee4decce0255e952dcd58c404779fa1f20dc5650d7bacb495037daad3` | Corrected Norn probe, exit 0. |
| `ev_80eacec8b24467dc9a209fcf1d607ce493a4b036ff111b17e10f0f0684f68f42` | Queensdale v15 probe, exit 0. |

The first generic archive inspection also decoded all three required chunks,
but exited 1 because the unrelated `octm` chunk exceeded its element budget
(`ev_930650e79862b6d3bd1aa51ea042820a35bd3ecd3f96ca01a06bda4dc9334a09`).
That inspection was not counted as a passing whole-archive test. An initial
PE-unwind lookup could not find the small callback's boundary; the subsequent
bounded instruction capture above covers its complete leaf body.

All final captures exited 0 with verified cleanup of their owned process
groups. Unrelated inaccessible `/proc` entries were left untouched. Static
captures asserted the frozen executable hash; the archive capture records
the selected decoded asset hash and reader identity. No game client or game
server was started by this investigation.
