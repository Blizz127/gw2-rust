# Build 209026: bounded Guardian palette eligibility

Static REA follow-up, 2026-11-01, to the [palette census](209026-skill-palette-content.md)
and [character locale](209026-character-locale.md). No client replay or
initial skill selection was performed.

## Six concrete entry records

A hash-checked archive helper follows each candidate palette's row and
entry pointers, then the referenced type-64 skill. All six Guardian rows
have one entry and zero palette flags, as in the earlier census.

| Palette key | Skill key | Row category | Level threshold +0x1c | Mode selector +0x14 | Skill flags +0x38 |
| --- | --- | --- | --- | --- | --- |
| 127 | 9083 | 2 | 1 | 3 | 68182022 |
| 21 | 12440 | 2 | 1 | 0 | 67112966 |
| 2 | 12417 | 17 | 11 | 0 | 67108870 |
| 4 | 12337 | 17 | 11 | 0 | 67112966 |
| 1 | 12343 | 0 | 31 | 0 | 67112964 |
| 14 | 12323 | 0 | 31 | 0 | 67108864 |

All six skill flags include bit 26 and exclude bits 28/29: their inspected
locale filter permits locale 0 or 3, but not 1/2. Their entry fields share:

- Null classified references at +8, +0x30, +0x40 and +0x50.
- +0x24/+0x28 both 23; this pair skips the corresponding scalar comparison.
- +0x48/+0x4c both 24; each skips its comparison after the context getter.
- +0x3c and +0x64 zero; these skip the matching scalar and content-flag checks.
- +0x60 equals 8. For ordinary selection this calls `0x141213430`, which
  returns true for their null +0x50 reference. It does not call the separate
  +0x58 helper on this branch.

The whole entry predicate is `0x141212fe0..0x14121342b`. Selector +0x14
value 3 skips its mode check; other non-2 values must equal context virtual
+0x60. The stack adapter forwards that call to the skill-bar interface,
whose method `0x1411f7420` converts `0x14140f5d0` to a boolean. This is why
palette 127 has one fewer condition than the other five. The global
predicate reads a context-linked record and compares a scalar with 16;
its gameplay name is not established here.

The six entries still require a passing slot check, level threshold and
separate skill availability. Null optional references alone do not make
an entry unconditional. The entry mode that skips availability is not
used for these Guardian standard rows: resolver `0x141213650` supplies
that mode only for profession 10, slot 21 or row category 8.

## Standard-slot membership is not recursive eligibility

The slot predicate `0x1411d5670` first checks whether this is the player's
finalized owned character. Player virtual +0x80 (`0x1411bada0`) returns
player +0x20 only when character +0x178 has finalized bit 0x10. Other
characters return true from this particular slot predicate early. The
owned-character path has transformation/inventory alternatives; a newly
constructed inventory's selector 11 proceeds to the normal path.

For a non-downed associated player and standard slots 0–4, the normal
path checks stored palette membership. Character virtual +0x370
(`0x1411d9280`) identifies downed state by character +0x188 == 2. With
the ordinary call's zero fourth argument, membership checks only the
requested slot, across locale selectors 0 through 3.

Player skill virtual +0xb0 (`0x1412288f0`) selects the stored palette
using the same +0x60/+0x88 arrays as message 612, then calls
`0x141229360`. The latter scans **every row and entry** in that palette
and compares its first skill pointer. It does not recursively invoke the
entry predicate, profession filter, availability test or level check.
A matching skill returns true from the slot predicate before its final
slot-level fallback. This establishes a direct membership path, while the
outer entry predicate still applies its own level and availability checks.

Consequently, for the inspected ordinary path, storing palette 127 in
standard slot 0 makes its skill 9083 discoverable by this membership
check. Combined with locale 0, level getter at least 1, passing availability
and the already traced component prerequisites, its entry passes the
reduced conditions above. This is a **conditional static candidate**,
not evidence of a rendered skill bar, usable action or complete bootstrap.

## Separate availability and fallback levels

The existing type-64 availability bitset remains separate from palette
membership. `0x1412284b0` returns true when the skill's data-key bit is
set, alongside mode/account/transformation alternatives. It still obtains
owned-character transformation/context state before its final result;
setting a bit does not remove those component lifetime dependencies.
Messages 605/609/613 and their ordering are documented in
[skill availability](209026-skill-availability.md).

If membership does not succeed, the slot predicate can reach
`0x1411c85d0`, which resolves configurable level thresholds rather than
using literal level constants. The table maps slots 0/1/2/3/4 to setting
IDs 0x271/0x27c/0x27d/0x27e/0x270. The [settings follow-up](209026-skill-slot-levels.md) resolves their frozen
archive values to 0/11/15/19/31, distinct from executable defaults and
entry +0x1c.

## Bounded reader scope

`gw2_dat::skill_palette_content` exposes type-65 UID/key/flags/kind, its 0x18-byte
rows (profession/category), each row's 0x70-byte entries (skill reference,
minimum level and raw context-mode selector), and type-64 UID/key/flags.
Array resolution retains the source container and checks caller-supplied
limits before returning the target bytes. Classified zero-offset references are
valid targets; only unclassified zero pointers are null. The skill view requires type 64 before interpreting its flags. These views
do not implement an eligibility predicate.

Three focused reader tests pass: cross-container definition/row/entry/skill
chains, unequal UID/key values, every truncated record extent, wrong types,
array limits, missing containers, conflicting fixups and classified zero.
The optimized `gw2-skill-palette-check` build and four CLI rejection cases
pass. A REA archive smoke independently matches all earlier counts:
7,769 palettes, 20,990 rows, 27,320 entries and resolved type-64 references,
and 185 Guardian rows. All six printed candidate records match the Python
helper. The checker uses an explicit 100,000-element limit per array;
no initial palette assignment or runtime fixture was added.

## Evidence

Private bundle: `~/re/gw2/notes/209026-guardian-palette-eligibility-1.json`.
Private helper: `inspect-guardian-palette-entries.py`.

| REA evidence | Observation |
| --- | --- |
| `ev_89b9c77fd04d1c348a5af86c5922f0f58620286cd3f1c7134e7fdd8384cdf5a9` | Entry predicate prefix and locale filter. |
| `ev_1c8e5ec233346ab7c2854242714f46c1ba6d9ff53ce95a7dc6718e4a4778cab6` | Entry predicate tail and adapter vtable. |
| `ev_117f07314336001e0b40fc91d9fe14b57b657b101c693f51bd4f7797f614b440` | Six actual entries/skill flags, mode adapters and optional-reference helpers. |
| `ev_59e105ac2e56a12e1540309e5b1e0a077d6559d2861f75967ca210fd24a7e76a` | Skill-bar interface construction and complete palette resolver. |
| `ev_54c3f8fbf52b4415b3c5878abde16b429a6f1bd4d87ffb3c1583363d5ab56ef8` | Interface vtable and stack adapter binding. |
| `ev_cdec8b680691ca812e4ff760678825a4e00b660420435b6a48ad249e23431886` | Slot-predicate prefix and adapter argument forwarding. |
| `ev_3060133328079cf8f17cc917c77d8b37263a16492119d7b47c8a0884a84baaa3` | Slot-predicate tail, numeric slot classifiers and global mode predicate. |
| `ev_7127001dfebd5801804c40b897f3da387530f78b7257b8ff462bc86db8662460` | Slot-level setting selection and membership virtual target. |
| `ev_30cb878d0ecb7cb7e8583aa9f079af35f5f62e53795356bc5ca9b56ee1bc5309` | Membership palette selection and exact standard-slot setting table. |
| `ev_48b52a4d509243af96ad661ff25d395f533c6b00395d1b0105748d4e7c460e20` | Complete nonrecursive pointer-membership scan. |
| `ev_628062962eaee3d792c2a86ae6ba2ee41b1c6ba7ed77dfdd2595b9a6fc1c6c8f` | IsPlayer/GetPlayer/downed virtual targets and player vtable installation. |
| `ev_4d2bea3355be3b7a350b5109f6127af04c2c51fab4fbe0b6ae1996a794510edf` | Downed predicate, owned getter target and availability result conditions. |
| `ev_ad5d42273f0782fdb37f49551e6d5e134e4f5b6e2dea3c3e6ba5f966259462d4` | Complete finalized-owned-character getter. |
| `ev_0c6fe34dc426da3c5ac7d5d8e276d587eb1749bbe6b070a07f9077e461223f95` | Optimized Rust archive check matches the complete palette census and six candidate records. |
