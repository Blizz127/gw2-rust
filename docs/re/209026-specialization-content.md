# Build 209026: specialization catalog and trait membership

Offline archive census and static REA analysis, 2026-11-01, for the frozen
client identified in [mission transfer](209026-mission-transfer.md).
This supplies content evidence for the [640–643 message layouts](209026-specialization-messages.md),
without choosing a playable build or sending client messages.

## Complete base-container census

All 32 base content containers, file IDs 1282830..1282861, were scanned.
Their hashes accompany the private results. Both selected types use the
already decoded metadata tuple `[0, 20, 40, 24, 0]`: GUID at +0,
UID at +0x14, key at +0x28, name pointer at +0x18, tracking flag zero.

| Content | Records | Unique keys | Key range | Zero/duplicate keys |
| --- | --- | --- | --- | --- |
| Type 386 specialization | 81 | 81 | 1..81, contiguous | None |
| Type 77 trait | 2,425 | 2,425 | 1..2445, with gaps | None |

Every specialization's pointer array at +0x58, count +0x60, contains
**nine non-null references to type-77 traits**. All 729 references resolve.
Each specialization has three traits with +0x70 tier 1, three with tier 2,
and three with tier 3. That is 243 references per tier across the catalog.
This array contains choices; it is not the three selected keys supplied
for one specialization slot in server 642.

The generic content lookup `0x1412e6820` constructs a hash key from content
type and the input key's low 22 bits, searches up to two runtime sources,
and returns null if neither resolves. Its call to `0x1412e9680` checks the
high bits, but the inspected caller does not branch on that result. The
base census's lack of key zero does not establish that every runtime
source lacks it, or authorize using zero as a universal clearing convention.
The wire codec therefore preserves raw u32 keys and supplies no default.

## Discriminators used by Guardian

The Guardian consumer previously traced in
[profession initialization](209026-character-profession.md#specialization-dependent-byte-check)
compares specialization discriminator +0xc8 against 3 and 5; its fallback
is 1. The following shipped records have discriminator values 1..5, all
in file 1282840. The profession-definition follow-up below independently
confirms all nine belong to profession 1, Guardian. Display names remain
unassigned.

| Specialization key | Discriminator | Flags +0x38 | Type-353 progress-reference key |
| --- | --- | --- | --- |
| 13 | 1 | 0 | 24 |
| 16 | 1 | 0 | 100 |
| 42 | 1 | 0 | 103 |
| 46 | 1 | 0 | 30 |
| 49 | 1 | 0 | 45 |
| 27 | 2 | 1 | 85 |
| 62 | 3 | 1 | 417 |
| 65 | 4 | 1 | 466 |
| 81 | 5 | 1 | 521 |

For a concrete membership example, specialization **13** has tier-1
choices **588/581/633**, tier-2 choices **580/584/1684**, and tier-3 choices
**585/586/589**. These are candidate content keys, not a selected or
unlocked fixture. The Guardian secondary-method GUID reference
`6804e96f7e8035418c057debff649834` resolves to trait key **2200** in file
1282858. Its gameplay name and role remain unassigned.

## Eligibility checks are separate from wire assignment

Specialization method `0x141224db0` checks a proposed major trait against
the active or pending specialization, requires trait +0x70 to equal
`major_trait_slot + 1`, calls the player's trait eligibility method, and
searches the specialization's +0x58/+0x60 pointer array for that exact
trait. It also checks character/runtime state. Thus matching a key and
tier is necessary in this path but is not a complete eligibility test.

Specialization assignment predicate `0x141224f00` checks character state,
effective-level/slot eligibility through `0x1411ce820`, and whether the
specialization is already present. Content flag +0x38 bit 0 restricts the
specialization to slot 2. It obtains the owned coreStats profession, resolves
the profession definition through character-context virtual 0, and searches
that definition's +0xb8 pointer array, count +0xc0, for membership. This
profession-definition membership is now reconstructed offline below.

The level helper uses three content-setting selectors at `0x142160978`
and compares the supplied level with a resolved threshold. Conditional
runtime paths can substitute zero for that threshold. This trace does not
assign fixed unlock levels. These client request predicates do not replace
the distinct effects and prerequisites of incoming 640–643 messages.

## Profession membership

Loader `0x1411b3c90` enumerates **type 315 (0x13b)** through content virtual
+0x48. It indexes character-context +0x310 with record +0xa8, guarded by
`professionDef->GetType() < arrsize(m_professionLookup)` (bound 11).
The getter `0x1411b5270` returns this indexed pointer. This connects the
profession enum to the exact definition used by assignment eligibility.

A complete 32-container scan finds nine type-315 records, one for each
profession 1..9. Their metadata has **no numeric content-key offset**
(`0xffffffff`); the enum at +0xa8 is not a message content key. Each
+0xb8/+0xc0 array contains nine type-386 pointers. Across the nine records,
the 81 referenced specialization keys cover the complete catalog exactly
once, with no duplicate or omitted membership.

The lists below preserve content-array order. Profession labels follow
the already mapped [profession enum](209026-character-creation.md).

| Profession | Specialization keys |
| --- | --- |
| 1 Guardian | 42, 16, 13, 49, 46, 27, 62, 65, 81 |
| 2 Warrior | 4, 36, 22, 11, 51, 18, 61, 68, 74 |
| 3 Engineer | 6, 38, 47, 29, 21, 43, 57, 70, 75 |
| 4 Ranger | 8, 30, 33, 25, 32, 5, 55, 72, 78 |
| 5 Thief | 28, 35, 20, 54, 44, 7, 58, 71, 77 |
| 6 Elementalist | 31, 41, 26, 17, 37, 48, 56, 67, 80 |
| 7 Mesmer | 10, 1, 45, 23, 24, 40, 59, 66, 73 |
| 8 Necromancer | 53, 39, 2, 19, 50, 34, 60, 64, 76 |
| 9 Revenant | 14, 9, 12, 3, 15, 52, 63, 69, 79 |

Guardian's definition resides in file 1282842, content offset 3831408.
This proves profession membership; it does not bypass level, unlock,
slot, trait or runtime-state eligibility. No build selection was made.

Private follow-up outputs:
`profession-specializations-209026-{0-8,8-16,16-24,24-32}.json`.
Bundle `~/re/gw2/notes/209026-profession-specialization-membership-1.json`
contains 1,126 records, 24,224,037 bytes.

| REA evidence | Observation |
| --- | --- |
| `ev_76fe447668780dce7970d4a334fc5cabcd9b5790127b1513b95a40205b5811b9` | Bounded context-table search and level-setting selectors. |
| `ev_f2f762d3d4909500ca4e3ac30d9436480a1fa906c2c776e4080b5313d4a24255` | Full profession-definition loader and named bound assertion. |
| `ev_44f4e76a3bd9ff588018bec64ce3e467cd2c2219c5ba6506b1a843c920ee455d` | Definition scan 0..7, six profession lists. |
| `ev_de0722b86e1726b2dd8247afd4e7a8cb28f5cff0da1cc0a5cb039bf143944db2` | Definition scan 8..15, Guardian and Mesmer lists. |
| `ev_65b608ed3921003ce70dedac6112e5b2db1abc5b7ce5f794037f60345b8290dc` | Definition scan 16..23, no additional records. |
| `ev_a4aedf50407f919ceede159fd89e11491e8d3cf01fbc426e7e90b1812babc441` | Definition scan 24..31, Revenant list. |
| `ev_6a2120a8717e08b6ae286cd6aae8bece977c6b39276fc5f3410d8b49281d8d8b` | Exact nine-profession/81-specialization coverage audit. |

## Bounded reader scope

`gw2_dat::profession_content` exposes typed views of the verified type-315,
type-386 and type-77 fields through the existing `ContentIndex`.
It checks the stored content type and each accessed byte range, preserves
raw keys/flags/tiers, and returns classified references for profession and
major-trait arrays. Local array ranges must fit the current container;
external arrays remain explicit references whose destination bounds and
record types the caller must validate. String references are not valid
record-array targets. This reader will not select a build or infer unlock
state, and does not load or send network messages.

Three focused tests, the library build and the `gw2-profession-check`
binary build pass. Synthetic cases exercise nonzero record offsets, raw
scalar preservation, classified external zero offsets, wrong types,
truncated records, null nonempty arrays, local array extents, arithmetic
overflow and invalid reference classes. Empty arrays remain explicit.

The read-only checker, run through REA against all 32 frozen base
containers, reports **9 professions, 81 specializations, 2,425 traits,
81 specialization references and 729 major-trait references**, matching
the independent Python census. It rejects duplicate keys/enums but does
not resolve external targets or validate eligibility. Command:
`gw2-profession-check /path/to/frozen-209026/Gw2.dat`.

Smoke evidence `ev_7e6865593fe6e90352eeafd348791a3a3cca1c2d9e2d6a63ca5cf6b56f49dd37`;
private bundle `~/re/gw2/notes/209026-profession-reader-smoke-1.json`
(1,128 records, 24,231,497 bytes). The first shell-escape invocation was
rejected by Rizin and did not run the checker; the corrected invocation
above completed successfully.

`gw2_dat::content::resolve_reference` accepts an ordered slice of already parsed
containers. Local references use their source container; external references
use their explicit container index. It checks both container indexes,
offset conversion, range addition and destination byte bounds, and reject
string references. The caller still supplies the intended field/record
length and verifies the destination's content type. It does not discover
files, perform I/O or treat an external offset as belonging to the source.
Two focused tests and the library build pass, including local and external
zero offsets, source/target index errors, destination overruns, offset/length
overflow, string rejection and empty byte ranges.

## Evidence

Private census outputs: `specializations-209026-{0-8,8-16,16-24,24-32}.json`
and `specialization-traits-209026-{0-27,27-54,54-81}.json`, under
`~/re/gw2/notes/`, mode 0600. Private cumulative bundle:
`~/re/gw2/notes/209026-specialization-content-1.json`
(1,119 records, 24,172,145 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_c38fac1ff2d7cd5aec5acd761bc955b4b0c7c27b1f95653590306c0e180b5664` | Generic key lookup and two-source search. |
| `ev_445c4adcdc0e4e7107778bf1af6bb4bf8d00a791e7ab9b672660780ffb355cb2` | Containers 0..7. |
| `ev_f4f2906293441de5b1777e27fd7ac51201555d0e34173054f56b9481807df543` | Containers 8..15. |
| `ev_2900f8a91e9616853d37888a2575f766a4b9b96fd7a02035e53691629370be1f` | Containers 16..23. |
| `ev_4478671278bf197df3271657561bbff88ba488d9a4adcb05a01d30b585228291` | Containers 24..31. |
| `ev_1db5bbf53d953b7d6b21a08c975790270be7cab693f6eb651a8f5958c53ed9c1` | Aggregate counts, duplicates/zero audit, selected records and high-bit predicate. |
| `ev_3e36ef04ae7eedc7f075fa26c414f16dc8378dee3464ec6562bc0fc2acae1e09` | Major-trait eligibility and selected-trait getter. |
| `ev_75e63824070b5c9176a860778d2c1ef82c22b5da5a376a40230b999267be8df6` | Specialization assignment predicate and profession membership. |
| `ev_a4468e001c536e16e6108077ca2836a97933f8d191672e564736725b2aa3c5b1` | Level/slot threshold lookup. |
| `ev_7dea17fc30440ca0ed031b93437e53d4ea0f4cb507acd5eac5c46818c5fc3810` | First 27 specialization trait arrays and tiers. |
| `ev_a18367ce4854c23dfe816fb701f4a8e430b3d9f1964b85a6bbed955bcd165978` | Middle 27 trait arrays and tiers. |
| `ev_5670817c23d1636d8df80fcfdbd12e33c313c9b35c49141b837f8e3657e2fe70` | Final 27 trait arrays and tiers. |
