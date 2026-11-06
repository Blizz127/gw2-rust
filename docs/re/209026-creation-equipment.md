# Build 209026: creation equipment templates

Static REA analysis, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). No client replay or creation request
was performed.

Creation preview builder `0x14129b500` selects a template through
`0x1411c3e40`. Its arguments include race, profession, sex and level.
The creation vtable at `0x14217dca8` resolves +0x50 to `0x14129e450`
(context data +0x30, level), +0x80 to `0x14129e7e0` (data +0xa0,
profession), and +0x98 to `0x14129ec30` (context +0x8a4 bit 0).
When the builder's input flags contain bit 2, it supplies zero for the
level argument instead of calling +0x50. Other mode arguments remain unnamed.

The selector has two mode-specific branches using embedded GUIDs. The first
requires content type **139 (0x8b)** directly; the second requires
**150 (0x96)** and passes record+0x30 to `0x1412e38c0`. The branch at
`0x1411c443e` instead obtains the
race/sex type-137 definition and walks its pointer array at **+0x58**, count
**+0x60**. It matches pointed-record **+0x2c to profession** and **+0x28
to the requested level**. The inspected nested loop decrements a loop
counter but still compares +0x28 against the original level; it must not be
described as a proven lower-level fallback. The complete mode-selection
policy and runtime branch remain unresolved.

The preview builder walks 16-byte entries from the selected template +0x38,
count +0x40. It consumes each entry's pointer at +0 and nonzero Dword +8.
For item +0x2c equal to zero, `0x14129b0c0` obtains item +0x30 and asserts
the armor subrecord. With armor +0x68 nonzero, it follows armor +0 to a skin,
requires the skin's +0x80 branch to be zero, then uses skin +0x88 as the
`skinArmor` structure seen in the summary preview trace. The skinArmor's
first pointer supplies a model, while armor +8 indexes the eight 0x88-byte
preview entries. The helper also retains the originating item pointer.

Additional preview passes can override this selection using the creation
context's +0x70 virtual accessor. Thus the template alone is not a complete
description of final rendered equipment.

The template entry's +8 Dword is an **equipment slot**, established by
following it through `0x14129b0c0` into virtual +0x38, `0x14129e1b0`.
That color helper reads `context +0x460 +16*slot +4*colorSlot`, the same
matrix used by the creation color setter. The initial private extraction
used a provisional `quantity` field name; the corrected private artifact
below names it `equipment_slot`.

`0x1411bf460` maps armor category to equipment slot as follows:

| Armor category | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Equipment slot | 2 | 6 | 4 | 5 | 0 | 3 | 7 | 5 |

These numeric mappings do not establish localized slot names or the ordering
of the six summary equipment records.

## Extracted templates

A relocation-checked traversal across seven base containers found **18
type-139 template references per race/sex definition**: one for each
profession at level 1 and one at level 0. All ten definitions share the
same **nine distinct level-1 templates**. The level-0 entries were counted,
but their items were not extracted.

Across the 90 race/sex/profession level-1 combinations, 80 templates have
four item entries and ten have six. The referenced items are content type
35. Each combination has three armor entries in equipment slots **2, 3,
and 6**, with categories 0, 5 and 1 respectively. Ten combinations also
have an armor entry for category 4/slot 0, which the inspected nonzero-slot
template loop skips. Other entries have item +0x2c value 24, outside the
inspected armor branch.

Every extracted armor entry has nonzero armor +0x68, a type-66 skin, skin
+0x80 equal to zero and a positive skin data ID within signed-Dword range.
There are ten distinct skin IDs across these armor references. Equipment
slots are unique within each level-1 template. This supplies concrete skin
and color-row associations, but does not validate the full preview, later
overrides or a server-side character creation transaction.

Corrected private extraction:
`~/re/gw2/notes/creation-equipment-slots-209026.json`, 139,015 bytes,
mode 0600, SHA-256
`d82e9eadd1f57112dc49a441ccfb21bcb58688764a75dc4fd96b79fa25d6f05b`.
It retains source-container hashes, template locations and item/skin data IDs.

## Archive reference relocation

`0x1412e7ef0` walks `externalOffsets` records `(relocOffset, targetFileIndex)`.
For the base-container load (third argument zero), it adds the target
container's content base to the Qword stored at the source content offset.
The target index selects the 32-entry base list; it is not a file-reference
table index. This supplies the cross-container pointer rule needed to follow
the race/sex template array offline.

## Evidence

Private bundle: `~/re/gw2/notes/209026-creation-equipment-static-1.json`.
Content extraction and slot audit:
`~/re/gw2/notes/209026-creation-equipment-content-2.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_44c8cfb4d95e880617cf97e3e8acacc34ef00927a6048676aef60e51084cb65b` | Preview template call, argument getters, 16-byte item loop and later override passes. |
| `ev_af915ee7bfbba73312febfceab6980af7c20fc9c24092a6b109e3baeef5ce7aa` | Mode-specific template lookup and item-to-skin helper. |
| `ev_15f118ddf223b4fb7a2ed44b94a0b358d67aaf089fa36960029cf311b62c7c0e` | Race/sex template list and profession/level comparisons. |
| `ev_a5a940c7b4a30dedae5cd2c97d7b9d9d9adbe6ea3d36aad2535af4b4cab430d0` | External content-reference relocation. |
| `ev_f735c7e9b4ad0770c4d9850c40ada50ed9f67f29613d56b8aacdad78a158cc09` | Creation getter vtable slots. |
| `ev_ad11c8c0eff7740e6eb41bd3c3b0e902a5cdc43cd095de773b2c7fb8050688af` | Level, profession and flag getters. |
| `ev_18cd44e716b922a70bc3575c713534cc8a0505cd7f9fed486fa479fb9f370c66` | Relocation-checked template/item/skin extraction. |
| `ev_ce6408c844d9f57fb64d0980d725d07b95be1653c3fb1bfa44f631286d95101e` | Shared templates and item/armor/skin counts. |
| `ev_bd2dc4e8e30c81c9bfccd2d4a6ec1be6e5592c924bb83f675dbf9d57bd1e28d5` | Armor-category mapping function and color-helper vtable. |
| `ev_5deff320aaee446b721d1817f9fefbd855dd366234fafe4a5a21adbc51828d3d` | Category jump table and equipment-slot color-matrix arithmetic. |
| `ev_28c8ffd64caddce5432aa993945c42ff9c1a1156118d4021423ee42cfbfb5081` | Corrected slot labels, category/slot pairs and per-template uniqueness. |

## Typed reader and Mesmer archive check (2026-11-05)

`gw2_dat::creation_equipment_content` now provides bounded views for the
type-137 equipment-template reference list, type-139 templates, their
16-byte item/slot entries, and type-35 item identity/kind. Traversal takes
an explicitly ordered container set and caller-supplied element limits.
Returned templates, entries and items retain their container index and
content offset; callers associate those locations with archive IDs and
digests. Local/external references and target record types are checked.
Template order and duplicates, null item references, and equipment slot
zero are preserved. Null template references are rejected.

Three focused tests pass, covering cross-container provenance, slot zero,
empty versus missing references, wrong types, truncation, overflowing
offsets and traversal limits. Debug and optimized library builds pass.

A compiled checker ran through REA against the frozen `Gw2.dat`, inspecting
the four explicit Human/Norn male/female type-137 records (content keys
55/54 and 57/56 respectively). Each lists 18 templates and has exactly one
entry with level 1 and profession 7. All four point to template **UID 210**,
archive file **1282832**, content offset **250800**, with ordered
`(item data key, equipment slot)` pairs:
`(6465, 2), (6467, 6), (6464, 3), (33355, 29)`.
The four used source-container hashes match the retained extraction above.
The optimized check exited successfully in approximately 2.9 seconds;
an earlier debug check was terminated by its 30-second capture timeout
and supplies no passing evidence.

This verifies archived references and reader behavior, not the runtime
mode-selection branch, final equipment after overrides, item gameplay
properties, or successful equipment delivery to the client. The reader
does not choose a default template or infer a lower-level fallback.

Private helper/source:
`~/re/gw2/notes/check-creation-equipment-rust-20261105{,.rs}`.
Private evidence bundle:
`~/re/gw2/notes/209026-creation-equipment-reader-20261105.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_9910aac9354013b47173b01c7936f46a9b3434c5915e26a450336a2011a5bd12` | Optimized Rust reader checks all four explicit Human/Norn Mesmer template paths, ordered item/slot pairs and source digests; exit 0. |

## Norn level-1 and level-80 template distinction (2026-11-06)

A bounded follow-up reconfirms that both Norn type-137 records (male
`1282852:113352`, UID4; female `1282852:143984`, UID8) reference template
UID 210 at `1282832:250800` through their **+0x58** arrays, index 3 of 18.
The template declares level 1/profession 7 and retains the ordered item/slot
pairs `(6465,2), (6467,6), (6464,3), (33355,29)` used by the starter fixture.

Their separate **+0x88** arrays, index 4 of 9, instead reference UID 251 at
`1282837:186032`, level 80/profession 7. It contains 15 equipment entries and
different items, including primary-slot29 item39742. All nine +0x88 targets
are type139 level 80 templates, one per profession. This does not establish
the named mode or runtime selection policy for that array.

UID210 and UID 251 have identical type65 references at
`+0x90/+0x98/+0xa0/+0xa8/+0xb0`: palettes366/357/358/386/4845. Equality of
these authored references does not establish their active skill-bar slots,
availability grants or initial server selection. It does not justify
substituting the level 80 equipment for the level 1 starter loadout.

This investigation recovered no Norn opening-cinematic key. Neither the
nine template headers nor their examined direct references supply a type7
or41 link. The separately traced +0x68 resource collection is consumed as
appearance option groups. A concrete creation/race/biography-to-cinematic
selector remains unbound; this bounded result is not proof of global absence.

The exact comparison is REA
`ev_af133eee0d4164532232f9b3ce61cfdd162143ebc508bfa60cde4b54e4f9a3c0`;
the nine-target classification is
`ev_49eb88c5a503e97926dcce23281ca032cf3a9c731dda443b5ac64ee7cef088ef`.
Complete private follow-up bundle:
`~/re/gw2/notes/209026-norn-opening-plus88-followup-20261106.json`, SHA256
`4528ca231ecd9246890636d37cbb6d424da831bafaa919afaee4c3990b702b51`.
Findings/coverage receipt:
`~/re/gw2/notes/209026-norn-opening-selector-boundary-20261106-findings.json`,
SHA256 `84653705c866e7ed8f8727953d142aae1d94ce283439c683b0556fd52322de4d`.
The newly decoded file1282837 has SHA256
`efabc1cd94ec8cbb72da0f2c35c5e569b29d9414c93974de9adbc710f48ae6fd`.
The authorized single-file frozen-archive read verified unchanged archive
identity and decoder hash before/after; other comparison inputs were cached.

A subsequent bounded PE read completed the missing selector branch from
`0x1411c4153` through the ordinary-path join at`0x1411c4446`. It corrects the
earlier description that all mode-specific branches directly require type139:
the second branch requires type150 at`0x1411c4428`, then invokes the helper
above. The retained caller binds its gate to creation virtual+0x98 and its
GUID-pair choice to`0x1410f3bb0`; their mode names and current runtime values
remain unknown. The joined selector contains no direct +0x88 read, although
a GUID lookup might resolve a record also listed there. No equality or
normal-creation behavior change is established, and the cinematic mapping
remains open.

REA`ev_288ecb789bee6b0c40ad8dc7c1802003ec41268668ea94a5ffb8f8e046d31484`
retains the complete read with unchanged frozen PE hash, exit0, untruncated
output and verified cleanup. Private bundle:
`209026-norn-template-selector-middle-evidence-20261106.json`, SHA-256
`3b13ac75815caf97ae55f268fe4c816c5290bebc59cba36f81a4437d6708a6c2`;
joined findings:`209026-norn-template-selector-middle-findings-20261106.json`,
SHA-256`cdfb078a7d9a605033761fc5eda9ffea9973327a287cf75fefd372252241c42c`.
