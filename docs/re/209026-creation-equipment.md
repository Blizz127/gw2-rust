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

The selector has mode-specific branches using embedded GUIDs and requiring
content type **139 (0x8b)**. The branch at `0x1411c443e` instead obtains the
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
