# Build 209026: archived skill palettes and eligibility

Read-only REA archive census and static executable analysis, 2026-11-01.
Target identity is in [client-build.md](../client-build.md); record layout
and message 612 are traced in [player skill palettes](209026-player-skill-palettes.md).
No client replay or skill selection was performed.

## Complete frozen base census

The private helper `~/re/gw2/notes/inspect-skill-palettes.py` scanned all
32 frozen base containers, archive IDs 1282830–1282861, in four bounded
batches. It used the existing content fixup reader, checked pointer and
array bounds, retained each source hash, and verified the type at each
selected record and referenced skill.

| Item | Count |
| --- | ---: |
| Type-65 palettes | 7,769 |
| Distinct palette keys | 7,769 |
| Palette rows, stride 0x18 | 20,990 |
| Entries, stride 0x70 | 27,320 |
| Nonnull type-64 skill references | 27,320 |
| Guardian-specific rows (profession 1) | 185 |

There were no duplicate palette keys or null first skill references in
this base scan. This does not establish overlay/override behavior outside
these containers. Type-65 metadata is `[0,20,40,24,0]`: GUID +0, UID +0x14,
data key +0x28, name +0x18 and reference-tracking byte zero. Type 64 has
the same four offsets and reference-tracking byte one.

Guardian rows include 11 category-2 rows, 51 category-17 rows and 27
category-0 rows, alongside weapon/other categories. The standard slot
mapper uses category 2 for slot 0, category 17 for slots 1–3 and category
0 for slot 4. The following single-entry examples were inspected to
bound the next eligibility work; they are **not validated initial choices**:

| Palette key | Palette kind | Row category | Skill key | Entry +0x1c |
| --- | ---: | ---: | ---: | ---: |
| 21 | 4 | 2 | 12440 | 1 |
| 127 | 4 | 2 | 9083 | 1 |
| 2 | 9 | 17 | 12417 | 11 |
| 4 | 9 | 17 | 12337 | 11 |
| 1 | 1 | 0 | 12343 | 31 |
| 14 | 1 | 0 | 12323 | 31 |

Palette 127 is in file 1282837; the other five are in 1282848. All six
have zero palette flags and null entry references at +8/+0x30/+0x40.
Other entry fields differ or invoke additional checks, so those nulls
do not make an entry unconditional.

## Level threshold and character locale

The entry predicate compares unsigned entry +0x1c against adapter virtual
+0x50; values above the returned scalar fail. The adapter forwards to
`0x1411f7300`. Its ordinary path calls coreStats getter `0x141207f20`,
which returns stored earned level (+0xac) or 80 under its override
conditions. A separate character-context branch can use another object's
level getter. Consequently +0x1c is a level threshold, but testing only
the raw first level byte would not reproduce every client context.

The selector passed through character +0xd8 is named **charLocale** by
the diagnostic at `0x142160098`, in consumer `0x1411be3c0`. That consumer
checks type-64 skill flags at +0x38: selector 0 tests bit 26, selector 1
bit 28, selector 2 bit 29, and selector 3 succeeds unconditionally.
The numeric locale names remain unassigned. This establishes another
skill-content constraint beyond the palette's profession/category row.

## Availability is separate from palette selection

For the ordinary predicate path, adapter virtual +0x128 reaches
`0x1411f7b60`. For a player character it calls `0x1412286a0` on the
associated player's embedded skill object. That routine can succeed from
inventory-supplied palettes, profession virtual +0x118, transformation
state, or its final call to skill virtual +0xa8 (`0x1412284b0`).

The latter tests a bit indexed by the type-64 skill key (+0x28) in a
bitset at skill-object +0x28, bounded by its Dword count at +0x34. It also
has mode/player/transformation alternatives. The constructor initializes
that bitset empty. Message 612 writes palette arrays at +0x60/+0x88;
its inspected body does not populate this separate bitset. The messages
that initialize or change the bits remain to be traced. Thus a 612 update
alone does not prove the selected skill will pass availability checks.

Character skill-slot eligibility is also checked through primary virtual
+0x168 (`0x1411d5670`), with inventory, transformation, player palette and
level branches. This census deliberately does not implement a partial
predicate as if it were the complete client's decision.

## Evidence

Private census outputs: `~/re/gw2/notes/skill-palettes-209026-{0-8,8-16,16-24,24-32}.json`,
mode 0600. Private evidence bundle:
`~/re/gw2/notes/209026-skill-palette-census-1.json`
(1,052 records, 23,360,809 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_d35cd2334cac2f63ebdf79879ed3726f2e3de23369c55286a080b4f066e0cf0e` | Base containers 0–7: 2,070 palettes, 6,620 entries. |
| `ev_501427ee45c4e38e902fc328433e9e9e510f8ffef7de8308411689d536942886` | Containers 8–15: 1,829 palettes, 6,287 entries. |
| `ev_06e900bad14548dc73fcb30a4d48c7b18d1f60068136ea14b0a50e5f2ce212f2` | Containers 16–23: 1,525 palettes, 6,142 entries. |
| `ev_3d3bf517d901e2b555147e88eb4955612f7d33ab0cc7fa36b5dd3145ae55424d` | Containers 24–31: 2,345 palettes, 8,271 entries. |
| `ev_9294c75f6875561c69e6b382b8de25230916467664239fa65dfba1cade81bdb4` | Aggregate counts, key uniqueness, selected entry fields and adapter targets. |
| `ev_3010a37ca69fb16e9ba1ff94f49689f889df6014f7a1c9d9ad85816e977b5637` | Level/context getters and entry +0x50 optional-reference predicate. |
| `ev_6a59c7a4bad084701dc909c6a430f941d3e115940020d5ca4784256905ae8825` | Level override branch and locale-specific skill flag checks. |
| `ev_ef2fdfd35b578f0c4dd9e63e93fdb9bf02f478a2c5dc548347d5f8e55b0976ad` | Availability dispatch, charLocale literal and getter fallback. |
| `ev_44bb002fb43ca09b419381e0f4324629e40578855ec7416f9a713587609de697` | Complete inventory/profession/transformation availability search. |
| `ev_471ae047a47b660a8208acdf954d9e006067dd2d01ee40d86ab6088df00d10b3` | Skill-key bitset lookup and complete character slot-eligibility tail. |
