# Build 209026: Norn tutorial entry geography

Read-only REA investigation, 2026-11-05, continuing the
[Norn content/navigation audit](209026-norn-content-navigation.md).
No retail spawn is established. The terrain diagnostic at physical XY (0, 0)
is an explicit research placement only.

## Great Hunt step links identify sectors

The [quest lifecycle investigation](209026-quest-lifecycle.md) identifies
type-57 quest 154, first goal 1, as directly linked to map 379.
Its six 96-byte type-343 step definitions begin at container 1282830,
content offset 4146312. Their +0x38 references resolve as follows:

| Step array index | Step ID (+0x14) | Sector key | Container 1282846 content offset |
| ---: | ---: | ---: | ---: |
| 0 | 1 | 201 | 3289648 |
| 1 | 2 | 200 | 3233008 |
| 2 | 3 | 201 | 3289648 |
| 3 | 6 | null | — |
| 4 | 4 | 202 | 3179240 |
| 5 | 5 | 202 | 3179240 |

These targets are **type-63 sector definitions**. The content-service
virtual +0x228 resolves to `0x1412e1e80`, which dispatches numeric type
0x3f. Callers `0x141252370` and `0x1413fa460` assert `sectorDef`
after that lookup. All three definitions themselves reference open-world
map **28** through +0x38. This is consistent with tutorial geography sharing
the Wayfarer asset, but does not choose a character entry point.

The quest agent found no other relocated pointer in the six inline steps.
Goal +0x18 is null with count +0x20 zero. Step +0x18, +0x20, +0x30,
+0x40, +0x50 and +0x58 are all zero. Raw step +0x48 values are
0x851e, 0x851f, 0x8520, 0x8521, 0x8521 and 0x8522; their meaning is
untraced. No inspected field supplies an authored starting coordinate.

A census of incoming relocated references found 28 references to the three
sectors: 22 type-343 steps, three type-279 records at +0x28, and three
within an indexed type-317 region whose immediate owning-record structure
has not been established. The type-279 embedded data is not identified as
world coordinates. Reinterpreting its numeric fields as a spawn is unjustified.

## Tutorial map-layer shapes are audio effects

The tutorial's selected namespace contains 19 type-106 records, each linked
at +0x38 to a type-257 shape. Their inspected consumer
`0x1413ef8e0` comes from `MlrCliContext.cpp` and asserts that the
effect type is `MAP_LAYER_AUDIO_EFFECT_ENABLE` or
`MAP_LAYER_AUDIO_EFFECT_DISABLE`. It resolves type 106, reads the shape
pointer, obtains a representative point through `0x1403a27e0`, and
uses the record UID with an enable/disable path. These records participate
in spatial audio effects. Their coordinate-like shape data is not evidence
of an entry spawn.

The all-base type-307 census also rules out another easy map-key guess:
node definition +0x38 contains only 0 (8,570 records), 1 (3,031), or
2 (212). Neither +0x28 nor +0x38 is a Norn map ID. Neither census
establishes spawn semantics for the node's +0x2c vector.

## Separate sector polygon reader

WorldMapModel constructor `0x140984160` stores a map definition at
object +0x90 and a separate sector object at +0x98, then invokes
`0x1409843c0`. That reader accesses the sector object's point pointer
at **+0x30** and count at **+0x38**, iterates 8-byte Float2 points,
and passes each to **`0x1413f98a0`** with the map definition's
+0x28 key. It computes polygon winding and bounds after that conversion.
The relevant source assertion identifies `WmmSectorOutline.cpp`.

This sector object differs from the type-63 definition above, whose +0x38
is a map pointer. Finding the polygon object's loader/owner is the next
bounded source step. A proven polygon for sector 201 could constrain the
first tutorial objective's geography; it would still not establish a retail
spawn or facing.

## Evidence and limits

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

Private captured commands, output and cleanup:
`~/re/gw2/notes/209026-norn-entry-geography-20261105.json`.
Incoming-reference inventory:
`~/re/gw2/notes/209026-norn-sector-reference-candidates-20261105.json`.
Each retained final capture exited zero with verified owned-process cleanup.
These are static archive/executable observations, with no client launch,
runtime spawn observation or gameplay mutation.

| REA evidence | Scope |
| --- | --- |
| `ev_38781654b454545ebff182fb280ca8f64e3d035f3abc51bc5a1f78d2367232c0` | Quest agent: exact six step definitions and absence of other relocated pointers. |
| `ev_bd829e56c84a8ac5d4f7981399af722ba0216ee6180444940814814ce4f01d61` | Independent target record identities, type-63 keys and map links. |
| `ev_9893191eebbd8653a250275c827df1b4ec7e8c0c3fd392d8bc2d1cb332b677f2` | Content virtual +0x228 binds the sector lookup to numeric type 63. |
| `ev_4198838afb2c0ba00858a182571e2c675d63408067c87e7cf82ba0a11f553446` | Incoming-reference census for sectors 201/200/202. |
| `ev_e2a7925ae83785ddbb2799e388e4954e29859021e726ee2fd12500258758ce18` | Type-279 candidate records and unclassified embedded data. |
| `ev_ecadd55579953bb82746815ded7afbfe9cb433de9a66fc4643c2670c658b131c` | Map-layer audio assertion/source strings. |
| `ev_9e353bd60aaf11e71982d71361c13b21e3195ffc282d95145f3b085b13e94e24` | Type-106 audio consumer and bounded shape helper. |
| `ev_bb306b995cfc4b5f5ca0b5efb432b04a30e2e7d1f25cd9f58dbaf592b0bf8d56` | All-base type-307 +0x38 value census. |
| `ev_813aeb62098361156226e633f8e211af9b3d93fdbaff0340b7295caef6201b90` | Sector-outline constructor, stored object types and source assertion. |
| `ev_5171b16ac07ecd9e6a3a5b06e079fdaab030aa0bc1c6fa03c37229d42accaadf` | Polygon point array, map-key conversion and bounds consumer continuation. |

The interpreter identity alone does not bind imported helper source. The
retained scripts assert the frozen executable hash and the content reader
checks archived source hashes. Partial PE unwind fragments are not treated
as complete functions; the outline reader continuation was captured explicitly.
