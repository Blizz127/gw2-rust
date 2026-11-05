# Build 209026: Norn authored sector polygons

The first Great Hunt objective's sector now has client-authored polygon
coordinates. This continues the [entry-geography trace](209026-norn-entry-geography.md).
It establishes an objective area, **not a retail starting position or facing**.
All observations below use frozen build 209026 and target-free REA process
captures on 2026-11-05.

## Link from tutorial map to polygon

The previous trace identified the WorldMapModel outline reader
`0x1409843c0`: sector point pointer +0x30, count +0x38, stride
8-byte Float2, followed by map-to-continent conversion `0x1413f98a0`.

Its caller `0x14096d940` obtains a content hierarchy through
`0x1413f9420`. This uses global service +0x1b0, whose getter is
`0x1413f6970`. The MapLayout context vtable `0x1422db110`
binds virtual +0x08 to `0x1413f6340`, which forwards to the embedded
cache at +0xa8 and `0x1413f9d00`. The cache's continent collection
loader `0x1413fa2a0` iterates numeric content type **280**.

The archive hierarchy is made of content records:

`280 continent → 281 floor → 282 region → 283 map → 289 sector polygon`

The first three child arrays use +0x30/count +0x38. Map sectors instead
use **+0x80/count +0x88**. Map +0x28 points to a type-277 definition,
whose +0x48 is the type-45 map definition. Sector polygon +0x28 points
to a type-279 definition, whose +0x28 points to the type-63 sector
definition. The inspected WorldMapModel caller walks these exact links
and passes the sector polygon and map definition to `0x140984160`.

Tutorial map **379** has a type-283 layout record at container
**1282852**, content offset **4461360**, linked through type-277
1282854:4424888 to map 379 at 1282833:3863896. Its three sector entries
are **200, 512, 201**. They share polygon records with the inspected
Wayfarer map-28 layouts. Quest step sector **202** exists in the map-28
layouts but is absent from this tutorial layout's three-entry list.
That difference alone establishes no transfer/progression rule.

## Exact first-objective area

Quest 154 / goal 1 / step 1 references sector **201**, as established in
the linked entry-geography note. Its authored polygon is type 289 at
**1282852:5725912**, with 11 Float2 vertices beginning at
**1282852:5725976**:

```text
 -7427.000000, -34732.000000
  2757.620117, -25841.000000
   504.434998, -24509.300781
 -1427.130005, -22918.099609
 -3657.489990, -22234.199219
 -8911.000000, -18440.000000
-11109.000000, -18437.000000
-21384.300781, -20781.500000
-22133.000000, -31784.000000
-12978.000000, -31811.000000
-10977.000000, -34682.000000
```

These are the unconverted map-coordinate points consumed by the client
outline renderer. They have no height or facing component.

| Sector | Polygon record offset in 1282852 | Points | Map X bounds | Map Y bounds |
| ---: | ---: | ---: | --- | --- |
| 201 | 5725912 | 11 | −22133 to 2757.620117 | −34732 to −18437 |
| 200 | 6291696 | 11 | −11578 to 9437.450195 | −40565 to −25085 |
| 202 | 6337720 | 15 | −23718.199219 to −4426 | −20695.800781 to −285.384003 |
| 512 | 5886256 | 14 | −8907 to 7639 | −25951.599609 to −12731 |

For comparison with the existing terrain probe's 32-unit conversion,
sector 201's bounds divided by 32 are X **−691.656250 to 86.175629**,
Y **−1085.375000 to −576.156250**. This arithmetic does not select a
spawn, prove a walkable point, or certify that the entire bounding box is
inside the irregular polygon.

Both the tutorial layout and inspected Wayfarer layout use map rectangle
**[−21504, −55296, 21504, 55296]** at +0xc0 and continent rectangle
**[54528, 27648, 56320, 32256]** at +0xb0. Helper
`0x1413f8150` extracts these rectangles; `0x1413f98a0` normalizes
map XY by the map rectangle, scales by the continent rectangle, and
reverses its Y axis. This confirms these Float2 values are geographic
outline coordinates rather than unclassified metadata.

A deliberate diagnostic point chosen inside sector 201 can now be
justified as lying within the first objective's authored area. Its exact
XY would remain an analyst choice, with terrain/collision and visual
inspection still required. Retail entry position and facing remain untraced.

## Verified interior diagnostic surface

The deliberately chosen map XY **[−3200, −25600]** is inside sector 201's
actual polygon. A horizontal ray crosses its edges once; the shortest
point-to-edge distance is **3083.801279 map units**, or **96.368790 physics
units**. This verifies the polygon itself, not just its bounding rectangle.

The polygon's area centroid was also verified inside, but its measured
terrain normal Z was 0.29433134, so it was not selected. A bounded check of
four eastern interior points found the chosen point nearly level and without
a selected prop/zone triangle hit along the tested vertical interval.

The fixed real-archive probe was run through REA as:

```text
gw2-map-probe /home/blizz/gw2-client/build-209026/Gw2.dat 187611 -100 -800 -1000 1000
```

Its binary SHA-256 was
`f778a152a2817cbef32929a627b7289861c5873456327ff87e4b8d524d8bec0d`.
The measured terrain hit has physics Z **116.8381118774414**, normal
**[0.012950949, −0.004526348, 0.9999059]** (about **0.7861°** from
horizontal), source cell **[190,842]**, triangle **1**, surface selector
**0**. The surface point is inside both reported character-origin and
simple-character bounds. No selected prop or zone triangle hit occurred at
this XY in physics Z [−1000,1000].

Using the existing probe's identity-primary-map transform:

```text
server37.x = physics.x * 32
server37.y = physics.y * 32
server37.z = -physics.z * 32
transform_tag = 0

exact float32 surface XYZ = [-3200, -25600, -3738.819580078125]
float32 bits             = [0xc5480000, 0xc6c80000, 0xc569ad1d]
```

The root agent selected explicit run-114 packet XYZ **[−3200, −25600,
−3742]**. This is an analyst-selected character placement **3.180419921875
map units** (0.099388122559 physics units) above the measured terrain
surface. This note does not claim that run was observed here. Neither
that clearance nor the polygon area is a retail starting-position rule.

Probe coverage was 1,032,192 terrain, 1,498,209 selected prop and 29,798
selected zone triangles, with zero excluded terrain chunks. It reported
**23 empty and 3 missing prop selections**, zero empty/missing zone
selections, global water surface physics Z 0 and zero explicit water
volumes. The probe excludes fallback boxes, blockers, margins, scene
clipping and runtime filters. A point hit and shallow terrain slope do not
validate the character's full footprint, collision adjustment, movement,
facing or retail origin.

Private captured commands and results:
`~/re/gw2/notes/209026-norn-sector201-diagnostic-surface-20261105.json`.

| REA evidence | Scope |
| --- | --- |
| `ev_f3ca9919b3c441e5c8dd48d7b6c35fee7fdf2ef3226e7535a0668d3691a2a2f6` | Polygon centroid/interior check and steep initial terrain result. |
| `ev_5d7aaf6ce5db3a7e8dec11b484c080df6d1c9acd0e36b296d58c79f656a144fb` | Four verified interior XY candidates and actual archive terrain/prop/zone probes. |
| `ev_28d24922c95000675ae697c58b468f726dbe1a60686189ae31c8086a80914e29` | Chosen point ray-crossing proof, edge distance and exact float32 coordinate transform. |

## Evidence

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

Relevant container SHA-256 values:

| Container | SHA-256 |
| ---: | --- |
| 1282852 | `54de38b4b7c4fdf69c18154fb254a3f5f346c1b031b68a8416153c4f751ac5d3` |
| 1282861 | `c4060c28ffc167b9e4d9d7fa617789569ff11b3625048b06405a474d8b7c40ba` |
| 1282846 | `1dffd2ad00d632d83661c653cfdbeaf9a52516672f66d684182273d0475d894f` |

Private REA scenarios/output:
`~/re/gw2/notes/209026-norn-sector-polygon-evidence-20261105.json`.
Full exact polygon arrays, reference links and bounds:
`~/re/gw2/notes/209026-norn-sector-polygons-20261105.json`.

| REA evidence | Scope |
| --- | --- |
| `ev_1254ca0f424855c8a27b7fa3836855b5460e071852a5f5ab9c1bf0a9177372b1` | Outline caller's hierarchy traversal and full map-to-continent transform. |
| `ev_0e1781d64296837b13d45898a9ec326f5c5e80580d44f2274bd568ebac7cb780` | Caller entry, hierarchy getter and rectangle extraction. |
| `ev_8c032e44b4bef79bee6a89ade84f369ab8fd96923911e806cfff1ed52570791f` | MapLayout service getter and virtual +0x08 forwarding. |
| `ev_f811d081373099448fe7e79f1f6106d1c85cae0acae393fbbaceefea5f770191` | MapLayout context vtable slots. |
| `ev_d22e73c0483f5fff6baf82c57f9652c53e248950aedd1a0ee78d91bfef16b866` | Embedded cache getter. |
| `ev_af3ffb7bb3122f9b35148fcc6e2a0c40c175cec38442bc9a34c8457cf670d805` | Continent collection iterates numeric type 280. |
| `ev_ed3cd2ef4a25a4c6892805b5dc079ec580c2f3f902364585e305b2fb4124a07f` | Two type-280 base records and their array pointers. |
| `ev_3e9dda21368fff4634c4129a9a939766d0ab1b1712a194f42857517dc5416d88` | Tutorial/Wayfarer type-283 records, map links and exact sector lists. |
| `ev_327d4f412d16a6191a66fd9a6f63528606ec6bc76142c6b323825c0d92f61acd` | Hash-checked sector polygons, reference identities, map rectangles and bounds. |

All retained final captures exited zero with verified owned-process cleanup.
The artifact identity is the helper interpreter; scripts explicitly bind
the executable hash and the imported content reader checks archive container
hashes. The evidence bundle preserves the commands for review. This slice
changed no gameplay code and launched no client.
