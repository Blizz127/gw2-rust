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

## Existing hunt placements: measured surfaces, grounding unresolved

On 2026-11-06 at 10:52:21Z, the same unchanged probe queried the five existing
hunt actor XY positions against frozen 187611. This reads the configured
positions; it does not recover retail placements. Every configured Z is -3800.
With server Z = -32 × physics Z, their configured origins are mathematically
above the selected terrain by the following amounts:

| Species | Configured XY | Selected terrain map Z | Origin above selected terrain, map units |
| ---: | --- | ---: | ---: |
| 1544 | [-300,-26800] | -3522.4514 | 277.5486 |
| 4079 | [-200,-26300] | -3734.9336 | 65.0664 |
| 8013 | [-800,-26750] | -3540.0080 | 259.9920 |
| 7200 | [-700,-26200] | -3792.3060 | 7.6940 |
| 4083 | [-600,-27300] | -3358.4644 | 441.5356 |

All five queries use physics Z interval [-1000,1000] and find no selected prop
or zone triangle at their XY. Coverage is 1,032,192 terrain,1,498,209 prop and
29,798 zone triangles, with 23 empty and 3 missing prop selections. These are
selected static surfaces, not walkability, line-of-sight or model-foot tests.
The model origin, body offset and client grounding behavior remain unresolved;
the numbers alone do not establish visibly floating actors or justify changing
their heights. The nearest diagnostic hostile, species 7200, is only 7.694 units
above the selected terrain, so this comparison alone does not explain the
failed run237 route.

The retained collision audit narrows this uncertainty: the current NPC fixture's
creation flags`[0,1,0]` select the full collider. Its ordinary capsule is
bottom-relative, while its entered-scene position getter applies an 8-map-unit
body/agent offset for the fixture dimensions. That offset does not establish
terrain correction or model-foot alignment. The demonstrated ground-query
helper belongs to the simple collider and cannot establish automatic snapping
for these full-collider NPCs. See the existing
[creation flags](209026-agent-creation-flags.md),
[dimension offsets](209026-agent-dimension-offsets.md) and
[position update](209026-agent-position-update.md) evidence. The bounded audit
is`norn-hostile-origin-grounding-findings-20261106.json`, SHA-256
`b8cc4e6779d35316f33e05a7787565e9917e4fe46e2093393f3456b8f529dfd6`;
its 13 complete retained REA records are in
`209026-norn-hostile-origin-retained-evidence-20261106.json`, SHA-256
`b47d170753561e030fe4bd19155ca2ec2807d65e634b3c0ded428a6e61408eff`.

The subsequent complete source read of`1410a4c30(this,1,1)` shows requested
point synchronization followed by a rigid-body update to resolved position
plus the height-origin offset. Its immediate helper`1410a4930` normalizes a
vector. Neither body directly queries terrain. This closes the immediate
setter question, while later physics advancement and actual model-foot
alignment remain unknown. No height, flag or position change follows from
this slice. Complete-source evidence:
`ev_4459e7ae09d037c6d6e36245cbb62c0a039f95d82489c1b99591ec85ce432a7b`
and`ev_6761c56a2e796cc81ff511116f4cd0669c942cc9737744c41bceb737cf253c0d`.
Private source bundle:`209026-full-collider-grounding-source-20261106.json`,
SHA-256`a7b14ce88a4466a0edfa33029e9d0ed0dff1fde0ed6b83b5f0cdb1fa1e2a402e`;
findings:`full-collider-grounding-findings-20261106.json`, SHA-256
`eeb989de47833efa54699fb7398db0ea3568ef85dd9f01b61d71245eee7a33c6`.
The helper capture omits one rendered-state frame; all raw and normalized
source text is retained. All captures exit0 with verified owned cleanup.

Frozen archive identity and decoder/probe hashes were checked before and after;
decoded187611 retains the hash above. No working archive, game runtime, owner
record or actor placement was changed by this survey. The private result is
`norn-hostile-surfaces-20261106.json`, SHA-256
`b39e57dc238d1f0fcaa411a29a533e1a17af29071e46f5a8e5d8f5d7141df6e5`.
Complete REA bundle:`209026-norn-hostile-surfaces-20261106.json`, SHA-256
`ded15f0e2442bd3386f5bbce1f64df1cf4e6793b2ff1746e0d444a983a34ad43`,
evidence`ev_e1b517994577fb9dfa4b2f40fe61965ca0e1205fef56f35c6c2ba0fff7bd2b46`.
Exit0 and owned cleanup are verified. One rendered terminal frame is truncated;
all 9,314 raw bytes and normalized frames, plus the complete result JSON, are
retained. Seven unrelated inaccessible process environments were left untouched.

## Local geometry around the failed hall approaches

The subsequent atlas exports selected triangles in map XY
[-4096,512] × [-28160,-24576] from the same frozen decoded187611. One authorized
extraction is retained as
`ev_073857d8410273efa4a45de88c6237344bd64d6ecf35702cd875f364d5791d62`;
the export is
`ev_037f5b59cb8ac9c5dc1f816064ad23b5c6fb0f5bbcf464f564d805d2b56e3751`.
All later sections and route calculations use cached exports only. The original
probe and shared binaries are unchanged; the exporter uses an isolated source
snapshot and target directory.

The clipped crop contains3846 terrain,11666 prop and10 zone triangles. It
retains277 exactly vertical original prop faces that the earlier vertical-ray
probe cannot intersect. The grid contains4161 points and13589 all-layer hits,
including shared-edge duplicates. Three focused geometry tests pass; independent
barycentric calculations agree with the existing spawn/target terrain probes.
Root viewed the hash-admitted atlas and horizontal-section PNGs.

Prop3491/mesh373 has a large hall-like XY projection across
[-2862.443,-1690.582] × [-27212.342,-25266.931]. The sparse run236 samples enter
that projection; run237 passes its lower end. At H=-mapZ=3637, the final retained
run237 XY[-2408,-27319] is2.336 map units from a section segment of
prop2893/mesh391, near[-2408.2242,-27321.3252]. This is a geometric association,
not proof of the actual collision/contact that stopped movement. Sparse sample
connections are not verified continuous paths.

Four section heights3637,3750,3765,3808 use the relevant retained pose or fixture
height plus8. Exact e624 player and NPC fixtures supply radius1 and height16;
these are compatibility placeholders, not recovered Norn dimensions. The extra8
is only half that configured height, not a measured collision-contact offset.
Neither thin sections nor the selected triangle soup model a swept character.

The bounded species-dimension audit finds no supported replacement values.
Callback1411dc6e0 reaches1410a30b0 through the supplied agent/full collider and
updates collider+2e4, not height/radius/body-origin fields. Its definition lookup
uses a context separately identified with RidingCliContext; that does not name
the result's+64 field or supply ordinary species dimensions. The actual dimension
replacement method1410a1d20 still needs a typed caller/content binding. No actor
size, placement or server37 input changes follow from these observations.
Private findings `species-dimensions-fidelity-findings-20261106.json`, SHA-256
`e570dc81965d6053baef10bc83643a2d704ba62ea84794ed1a3593b5144e4097`,
pin14 retained REA records and nine artifacts.

## Upper route prefix for the next observation

Cached geometry rejects the lower detour as a generous corridor: opposite
64-unit offset probes intersect the hall and curved structure, and the terrain
steepens nearby. A more promising observation prefix goes around the upper end:

`[-3200,-25600] → [-3200,-25536] → [-3008,-25280] → [-2816,-25024] → [-2688,-25088] → [-2100,-25180]`.

The prefix is1442.3 map units long. With samples at most32 units apart, the
minimum sampled local horizontal-section separation is90.07 units, maximum
top-terrain face slope5.73°, and maximum along-path grade3.06°. The nine tested
terrain-following lines, at lateral offsets0/±64 and heights+1/+8/+16, have no
selected-triangle intersections. These sparse lines are not a swept body or a
collision-free guarantee. The placeholder radius1 is not used to admit a tiny
gap. Beyond the endpoint, terrain steepens and later probes intersect other
props; the full route remains unvalidated. Any native attempt should stop there
for a fresh scene assessment before further movement.

Coverage still omits23 empty/3 missing prop selections,92 global selected
animation-blocker references, fallback boxes, convex margins, actor footprint,
scene clipping and runtime filters. Highest selected terrain may not be the
native support layer. No visual landmark, map-north or camera heading is inferred
from the source XY axes. Original spawn, target and all actors remain unchanged.

Private receipts under `~/re/gw2/notes/map379-local-atlas-20261106/`:

- `receipt.json`, SHA-256
  `06f56f53714b9fa3a48c2629b9a649bad0a39a19b11c87d33bdc771c09272740`.
- `sections/receipt.json`, SHA-256
  `d98087e25afefbbfad67191e407b35fc2978d68e99fd8dee67ce21644704a492`.
- `routes/receipt.json`, SHA-256
  `3cf03959bce25d5eb6bdd8c820c70affb0f2330e300ddd32576653d473d212a8`.
- `routes/candidate-routes.md`, SHA-256
  `a02440656559fa046d39db876af17f64dcda0c74643dca8450b13c292f20ed88`.

The receipts retain complete arrays, calculations, plots, source pins and
limitations. Section and route arithmetic checks pass. No native route traversal
or full-hunt success is established by this source work.
