# Build 209026: initial equipment and model construction order

2026-11-05. Static follow-up to run114, where the explicit armor/weapon
batch was accepted after BeginPlaying and a weapon skill icon appeared,
but the character remained unclothed. The source supports a focused
ordering experiment: send the existing explicit creation/equip/set batch
**immediately before finalization 170**, after the existing character
components, so initial model construction can read the populated slots.
This trace does not itself establish runtime rendering success or a
complete explanation of the late-update behavior.

No packet was injected by this investigation. The existing
[item delivery](209026-item-equipment-delivery.md) and
[model input](209026-character-model-inputs.md) constraints still apply.

## Report 283 is not a confirmed view refresh

The report helper `0x1411dac70` asserts character +0x3f0 inventory,
then dispatches character listeners at +0x1e0 using callback
`0x1411d3d00`. That callback is exactly a trampoline to listener virtual
**+0x1a0**. The ordinary view's retained character-listener vtable
**0x142141558** has +0x1a0 equal to **0x140163990**, a `ret 0` no-op.
The table's neighboring +0x198 and +0x1a8 entries are also no-ops.

The ordinary view registration and retention of this table were already
established in [view reveal](209026-character-view-reveal.md).
Thus 283 does not directly refresh this view through its report callback.
Other report subscribers were not exhaustively enumerated; this is not
proof that the report has no effects anywhere in the client.

Equip 288 uses a different notifier: inventory +0x10, with owner at +0x70.
Its conditional mapped-slot event uses callback `0x14026df54`, listener
virtual +0x128; the final event uses `0x14026e848`, virtual +0x20.
The complete propagation from these events to a late model rebuild remains
unresolved. Sending 283 or arbitrary wardrobe values to force that rebuild
is not justified by this trace.

## The default outfit already selects the intended slots

Inventory constructor `0x1411ebdf0` writes Qword **11** at +0x474.
Consequently the Dword at **+0x478 starts at zero**, while +0x474 is 11.
It separately initializes the active weapon set at +0x46c to 11.

Model builder `0x141166580` obtains inventory and iterates seven model
rows at `0x141166914–0x1411669f1`. Inventory virtual +0x128 resolves to
`0x14128f440`, which passes +0x478 into slot mapper `0x1411c4e10`.
Its mode-zero table at `0x1411c4fcc` maps rows as follows:

| Model row | Equipment slot for outfit selector 0 |
| ---: | ---: |
| 0 | 1 |
| 1 | 2 |
| 2 | 3 |
| 3 | 4 |
| 4 | 0 or 5, selected by the inspected hide/character predicates |
| 5 | 6 |
| 6 | 7 |

Therefore the source template's armor slots **2, 3 and 6** are already
in the selected family. Virtual +0x130 (`0x14128f410`) performs this
mapping and then calls the ordinary equipment getter, returning null
only for sentinel 68 or an empty selected slot. The model checks slot
visibility through virtual +0x370 and an optional wardrobe substitution
path before adding the item's attachment data.

Server 297 reaches `0x141290d50`: it stores the requested value at +0x478
and dispatches listener +0x90, asserting against value 4 (`CHAR_OUTFITS`).
Sending 297=0 would redispatch the already-selected family; it is not a
source-supported correction of a wrong constructor default. The related
298/299 gathering and weapon selectors are separate fields.

## Armor's model eligibility is not item kind zero

Before calling armor attachment helper `0x141165f50`, the model builder
checks item virtual +0, **`0x1413c5410`**. This is a predicate, not the
item-kind getter. It rejects virtual +0x40 value 2; otherwise it permits
the item when virtual +0x108 is zero, or uses +0x110 when nonzero.

For armor, +0x40 is `0x1413c48c0`, whose result for raw flags 0 is 0.
Virtual +0x108 (`0x1413c5880`) tests whether item definition +0x68 is
nonnull. Hash-checked records for **6465, 6467 and 6464** all have null
+0x68. Thus these three items with the experiment's raw flags 0 pass
this particular model predicate. This does not establish retail flags
or bypass other model/asset conditions.

The attachment helper independently checks definition +0x2c equals 0
(armor), obtains the effective skin interface through item +0x200 and
color interface through +0x228, and conditionally calls composite-model
virtual +0x70. The source skins and color rows are documented in the
item-delivery note; asset loading was not observed here.

## Pre-finalization prerequisites for the explicit batch

The complete 288 setter `0x1411ec7b0–0x1411eca87` asserts a nonnull item,
slot <68 and an empty slot. It stores the pointer, sets its item location,
registers the inventory as an item listener, handles optional upgrades,
runs a gated player follow-up, then dispatches inventory listeners.
There is no unconditional character-view, model or skill-bar dereference
in this direct body.

- Shared item-location setter **`0x1413c66c0`**, used by both armor and
  weapon, requires a nonnull inventory and location slot <4095. It stores
  the location and emits an item notification when changed. Fresh item
  construction clears its listener list. Item listener registration
  **`0x1413c5a90`** inserts a listener and does not immediately invoke it.
- The upgrade branches test primary/secondary results and iterate the
  additional list. The retained fixture uses no upgrades; the positive
  upgrade path is not a new prerequisite for this experiment.
- Player follow-up **`0x1412909a0`** checks owner/player identity, then
  requires character +0x178 bit 4 before its player/PvP-specific calls.
  Finalizer **`0x1411da260` first sets that bit at `0x1411da33d`**.
  Hence the fresh pre-170 path does not enter that later follow-up.
- Inventory construction clears its notifier lists. The identified
  coreStats subscription at **`0x141208920`** is invoked during 170,
  after the finalized bit is set; it is not a reason to defer initial
  equipment until after model construction.

The complete 299 setter **`0x141290dc0–0x141290e98`** asserts against
weapon sets 11 and 6, obtains optional old primary/secondary item locations,
stores the new set at +0x46c and dispatches inventory listeners. Old null
item results are explicitly handled. It has no unconditional finalized
view or skill-bar dependency. The source weapon's slot 29/set 4 mapping
is established in the item-delivery note.

For the same fresh fixture, the concrete experiment is therefore:

```text
existing player/character/components, including 183, 282, profession and wardrobe
→ existing explicit 960/971 item creation batch
→ existing explicit 288 equips
→ existing explicit 299 set selection
→ 170 finalization and initial model construction
→ existing 618 / 444 and later readiness flow
```

Maintain unique runtime item IDs, empty destination slots, supplied content
keys/flags/colors and once-only delivery. This is an ordering change to
an already explicit batch, not a retail-default claim. Runtime-dependent
notifier membership is not exhausted by the direct-body trace; playback
must still check assertions and the actual model. No new report or wardrobe
message is established as necessary here.

## Evidence

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Armor container 1282841 SHA-256:
`62ef0c04651578ca4cf6a4322fe4cb792bd865b614069d639858986b13594e39`.

Private REA bundle:
`~/re/gw2/notes/209026-initial-equipment-model-order-20261105.json`.

| Evidence | Observation |
| --- | --- |
| `ev_51dc08a19e000ad35ff26bb2da9017a6c01d8ca3f314d9131c903cd677d8eae1` | Report helper and +0x1a0 trampoline; initial model builder. |
| `ev_c60ce0ae6b78c3eff2d489e73dbeb955d993b48dcca4bb14f90421143461f631` | View report no-op; inventory model getters and equip-event trampoline. |
| `ev_9299039be44eceb3b631ce5b986e0269a5bc0290fb3f86af47b60850687671f9` | Equip setter front half, ordinary model initializer and seven-row inventory consumption. |
| `ev_e2efa29dbf37245e34d21e6b116a1507448045b6794f1f12a6f27513b00c0d63` | Outfit mapping branches, equipment getter and finalized-gated player follow-up. |
| `ev_50a5b1cd4d0696ef46d9cd5f15d2698d5abf96ba842549bd99fe17888324e6f2` | Constructor +0x474/+0x478 values and outfit setter/assertion. |
| `ev_2921fba75f6c13748b5b23dfbde6c7fb85406edf34739f233e378546ef5c9013` | Armor vtable installation and attachment/color helper. |
| `ev_86b3cb36e3521844c3ab93f230d1995904c3af33d467ee42cbd3f57e5f114406` | Item predicate slots and exact 297 handler. |
| `ev_5ab9b7daf9f7e8479e2777bb2126a7be4b1d4d0bb58a9ea24370f87de8aa2c43` | CoreStats inventory registration during finalization. |
| `ev_8d8e5e785e9864e865ca61119beb02c3d63f7a16e96ab9a58f813f1f35b9df36` | Complete remaining equip body, complete set selector and model predicate. |
| `ev_0f338f99ab10ed0ae5f9324501e87d556cc0cf10afdf39f7617f9f17dbea69bb` | Item-location/registration bodies, eligibility leaves and empty inventory listeners. |
| `ev_7a9e909ed216dd77094dd762bf7603931133acf00940b5b7e73afa1196cce2c5` | Finalized-bit write, item listener initialization, exact mode-zero jump table and null item-definition +0x68 fields. |

REA ran hash-bound read-only Python/objdump helpers, not the game. All
captures exited zero with verified owned-process cleanup; unrelated
unreadable `/proc` entries were left untouched. Disassembly output includes
clipped boundary bytes and jump-table data in some exploratory ranges;
conclusions use aligned function bodies and explicit table reads. This
investigation changed only this note and retained private evidence.
