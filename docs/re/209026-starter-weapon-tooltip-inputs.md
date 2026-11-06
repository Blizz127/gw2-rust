# Build 209026: starter sword strength inputs to damage tooltips

Bounded frozen-client REA, 2026-11-06. No runtime, working archive, server
behavior or packet defaults were changed. This follows the
[character tooltip adapter](209026-tooltip-character-context.md),
[equipment delivery](209026-item-equipment-delivery.md), and
[map-context gate](209026-skill-context-map-gate.md).

**The inspected ordinary player/item calculation returns strength 105–117,
mean 111, for equipped starter item 33355.** This is conditional on selecting
that equipped item and taking the static item-strength branch described below.
It is not the item's raw authored pair 105–116, a captured tooltip value, or a
server damage rule. Caller slot selection and an alternate player-state branch
must remain explicit when applying it to a particular fact display.

## From the character adapter to the equipped item

The constructor-established SdInt character adapter has vtable `0x141b67f38`
and character pointer +0x20. Its relevant slots are:

| Slot | Concrete method | Source result |
| --- | --- | --- |
| +0x58 | `0x14088f320` | Current inventory primary item, then its item definition. |
| +0x60 | `0x14088f380` | Current inventory secondary item, then its item definition. |
| +0x50 | `0x14088f430..0x14088f4a3` | Another inventory lookup, passing character primary virtual +0x1b0 to inventory virtual +0x2f0, then item definition. Its selection semantics are not named here. |
| +0x70 | `0x14088f540` | Character +8 interface virtual +0x68, the established IsPlayer predicate. |
| +0x78 | `0x140168aa0` | Constant false. |

The first two methods get inventory through character +8 virtual +0xc8.
The installed table `0x142161250` binds that to `0x1411d7f60`, which asserts
`m_inventory` and returns whole-character +0x3f0. Inventory vtable
`0x142163228` binds +0x18/+0x20 to `0x14128f4a0`/`0x14128f4d0`.
These obtain active equipment slots through +0xf0/+0xf8 and read the
equipment pointer array at inventory +0x160; slot 68 means no item.

The established active-set-4 mapping gives slots 29 and 30. The explicit
fixture places item 33355 in slot 29, leaving slot 30 empty. Weapon constructor
`0x1413cb4f0` installs table `0x1422602c0`; its +8 method is the complete
`0x1413c4930..0x1413c496c`, which asserts `m_itemDef` and returns item +0x40.
Thus this path uses the equipped item's definition, not its skin or instance key.

The complete chooser `0x140895550..0x1408958bf` selects these adapter methods
using its explicit R9D selector. Its jump table begins at `0x1408958c0`:

- Selectors 5, 6 and 7 request the primary item.
- Selectors 8 and 9 request the secondary item.
- Selectors 10 and 11 do not force either of those selections.
- Selectors 12 through 18 request the +0x50 lookup.

The chooser also has no-item/fallback paths and a non-player condition. This
note does not replace those with a universal primary-weapon default. Its final
adapter +0x78 query could replace the lower result with the upper result; the
character adapter's constant-false method prevents that replacement here.
The damage fact's own zero selector preserves its caller-supplied selector;
the particular display's incoming slot lineage is a separate dependency.

## Exact authored item and the selected formula

The cached type-35 item row is file 1282834, content offset 3014352. The
bounded reader required the cached container to exist before using the content
reader, and checked its exact type/key and weapon kind.

| Field | Frozen value |
| --- | --- |
| Item key / kind | 33355 / 24 |
| Item +0x4c / +0x60 | Level 0 / rarity 1 |
| Weapon subtype location | Same file, offset 3014528 |
| Subtype +0x0c | Weapon type 0 |
| Subtype +0x10 | Authored pair 105, 116 |
| Subtype +0x30 | `scaleType` 1 |

`0x140895460..0x14089554d` supplies the item definition, item level, rarity,
adapter level, inverse IsPlayer predicate and an ability-controlled multiplier
switch to `0x1413cdbb0`. The complete logical calculation continues through
`0x1413cddc0`; the unwind entry's first fragment is not the whole function.
Its source assertion names `scaleType`. Only scaleType 0 copies the raw pair.
**ScaleType 1 selects the explicit item level**, which is zero for this item,
rather than substituting character level 1.

For a player, helper `0x1413cf4b0..0x1413cf536` chooses
`0x1412df6e0`, the player-item-power lookup at `0x1421d7060`.
The alternative inverse-IsPlayer branch uses the monster-power lookup, not
this table. The player table's index 0 is 170.

The chooser reads ability-body +0x70 bit 31. If clear, the strength helper
uses the weapon-type multiplier; if set it retains a separate default 1.1.
Independent cached-row evidence `ev_1139614cc70d841fc2a761ec769eb5e6b3f728ddec017c5ef4e0a23aeae2433e`
finds +0x70 equal to zero for skills 10170, 10171 and 10172. Thus the clear-bit
case applies to those exact ability rows; it is not inferred from the skill's
separate flags word.

| Input | Frozen value |
| --- | --- |
| Player item-power table, level 0 | 170 |
| Weapon type-0 multiplier, `0x142260c80` | f32 1.0 |
| Rarity-1 multiplier, `0x142260e54` | f32 0.6499999761581421 |
| Weapon type-0 spread, `0x142260c84` | f32 0.05000000074505806 |

Preserving the instruction-order f32 rounding gives base
110.49999237060547, spread 5.524999618530273 and pre-conversion endpoints
104.97499084472656 and 116.02499389648438. Each endpoint passes through
`0x1409bec80`, whose complete bounded-positive behavior is upward integer
conversion. The result is **105, 117**, mean **111**. This endpoint helper
is different from the damage formatter's add-0.5/truncate rounding helper.

As a control on the multiplier dependency, retaining default f32 1.1 instead
would produce 116, 128, mean 122. It is not the selected branch for the three
cached sword ability rows above.

## Map and player-state qualifications

The chooser's player level/rarity overrides are `0x141476250` and
`0x141476270`. Both call `0x14140f5d0`, which reads the current map definition
through `0x141411100`, follows map +0x100 and then referenced record +0x30,
and tests the reached type against 16. With frozen map 379 installed,
map +0x100 is null, so the predicate is false. The helpers return level 0
and rarity sentinel 8; the chooser replaces those with the equipped item's
level 0 and rarity 1. The null reference is independently retained in the
map-context note; this conclusion does not assume a particular global setting.

Before its static calculation, `0x140895460` checks a separate player object:
character +8 virtual +0xe0, player virtual +0x20, then that object's virtual 0.
The installed player getter `0x1411bb3d0` returns player +0x9bd0. Constructor
`0x141274990` installs table `0x1421735c0` and stores Dword +8 = 2;
its virtual-0 predicate `0x1412749b0` returns whether that word differs from 2.
Constructor state therefore takes the static strength path. A nonzero predicate
instead asks the component returned by character +8 virtual +0xa8 for its
virtual +0xa0 result. That alternate calculation and later state changes were
not expanded in this bounded slice. Constructor state is not proof of all live
states or all tooltip callers.

The existing character-context source supplies the other damage-fact inputs:
adapter +0x18 reads the second core-stat level, and +0x10(selector 1) reads
signed effective-attribute Dword coreStats +0xb4. The inspected level-1 fixture
construction has level 1 and initially zero effective attributes; no later
runtime attribute observation is claimed here. Health 388 does not establish
attribute 1. This note provides explicit numerical inputs and branch conditions,
not a complete authoritative damage or combat implementation.

## Retained evidence

Private bundle:
`/home/blizz/re/gw2/notes/209026-starter-weapon-tooltip-inputs-20261106.json`,
538,068 bytes, SHA-256
`3903e1530ec2d8c786006d9c95259c8096a8d158d21fe1137ebefc730a0ab8c4`.
It contains 20 new bounded captures and the reused complete chooser capture
`ev_4fe68cbd78a93d4f900626a191a7fba3f284be96048b3acca76cbd260e835b75`.
All exited zero and verified cleanup of their owned process groups. Some
captures report unrelated process-environment permission limits; no cleanup of
those unrelated processes is claimed.

The cached-item read is
`ev_5b18a8d6846b44ad41368f34fe4f196317398cc2baa28bc707821e947fd528d9`;
the exact item-definition getter is
`ev_1a201295c5aeef364fb87da2f938bf8e602a54eeba52bdb630962920f3eaa6a5`;
the hash-bound f32 arithmetic is
`ev_a9fd8ff4a0c889dc6c6ee3f783ff6c10a9fa61484052c3f05a9b38c836489d84`.
Private helpers retain the exact scalar inputs and calculation; no raw content
or executable bytes were added to the repository.

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Cached item-container SHA-256:
`fc03eee83e799747ebed6f13c5445c353e5bfd1198105b1bc0048c8359013232`.
