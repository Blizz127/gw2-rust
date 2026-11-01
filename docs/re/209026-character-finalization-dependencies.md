# Build 209026: finalization dependencies and inventory creation

Static REA analysis, 2026-11-01, on the frozen executable identified in
[mission transfer](209026-mission-transfer.md). This extends the
[player/finalization](209026-character-population.md) and
[core-stat](209026-character-core-stats.md) traces. No client replay occurred.

## Owned character is needed during player finalization

Server 170's finalizer `0x1411da260` calls `0x14120e380` with
`player +0x9f58` on its player branch. That helper reads its argument
minus 0x9f38: **player +0x20**, the owned-character pointer populated by
server 443. The player constructor `0x1411b9cc0` initially clears it.

The helper then reads owned-character +0x388 (coreStats), stores the result
as its registered component, and calls `0x141208700` to add a listener.
Neither the owned-character dereference nor that listener registration
guards a null component. Its assertion names `!m_registeredCoreStats` in
`ChCliTrait.cpp`. This is a concrete initialization dependency beyond the
finalizer's initial `GetPlayer()` assertion.

For a fresh player whose owned character is being initialized, server 443
and coreStats creation must therefore precede server 170. Server 443's
setter uses the raw character lookup and checks `!m_charOwned`; it does
not require the finalized flag. This fits the ordering without a cycle.
It does not establish the behavior of every ownership/control transition.
The [coreStats initialization callback](209026-character-core-stats.md#initialization-already-requires-player-ownership)
adds a stricter order: fresh owned-player coreStats creation itself requires
the associated player and owned-character pointer. Its earned-level setter
reads owned coreStats before any finalized-flag guard. Creation stores the
new component before that setter, so 443 must precede 182/183 for this path.

## Inventory creation: server 282

Protocol 0, group 0 server **282** has descriptor `0x1425e3160`, registration
pair `0x14216f5e0`, and handler `0x141250500`. Its sole field is a **Dword
character index**. The handler requires that raw character lookup succeeds
and calls `0x1411d64c0`.

That helper allocates a 0x490-byte component through `0x1411ebdf0` when
character +0x3f0 is null. If the component already exists, it returns.
The constructor stores the owning character at component +0x70.
This message creates the inventory component; it contains no item records.

The coreStats finalization callback `0x141208920` obtains inventory through
the character's +8 interface, virtual +0xc8. The vtable entry resolves to
`0x1411d7f60`, which checks **m_inventory** and returns interface +0x3e8,
equivalent to whole-character +0x3f0. The callback stores the pointer at
coreStats +0x130 and immediately calls inventory virtual +0x540 to subscribe.
Its prior-state assertion names `!m_registeredInventory`.

Thus a character with coreStats needs inventory before finalization;
skipping server 282 merely because no items are present is unsupported.
Whether an empty inventory is sufficient for all later consumers is untested.

### Empty equipment and the inspected skill paths

The complete inventory constructor `0x1411ebdf0..0x1411ec12d` installs
primary vtable `0x142163228`. It zeroes the 0x220-byte equipment pointer
array at +0x160 (68 slots) and the 0x80-byte bag pointer array at +0x380
(16 slots). Equipment getter `0x14128fb10` asserts
`equipSlot < arrsize(m_equipmentSlots)` with bound 68. The initial
active weapon-set field at +0x46c is **11**; the getter at primary +0xe8
(`0x14128f500`) returns that value.

The weapon-slot helpers `0x14128fdc0` and `0x14128fe70` both map weapon
set 11 to sentinel **68**. Their switch diagnostic names **weaponSet**;
the exact case targets are verified by table entries `0x14128fe6c` and
`0x14128ff1c`. Inventory virtual +0x18/+0x20 route through these helpers
and return null for that sentinel. Thus the [skill-bar palette getters](209026-character-skillbar.md)
for slots 5–7 and 8–9 see no equipment item in the newly constructed state.
The slot-19 route uses inventory virtual +0xd8, which obtains the slot
from +0x468 (initially 34), then reads its null equipment pointer.

The [availability search](209026-skill-palette-content.md) also checks
inventory virtual +0x160 (equipment pointer +0x230, slot 26), slot 33,
and slots 34/35/36/60/61/62. These all lie in the cleared array. Their
inspected consumers test null before following item content. This gives
a bounded reason those particular searches tolerate absent equipment;
it does not prove the whole skill bar or playable client is ready.

Constructor field +0x440 starts at five bag slots. Its final helper
`0x141291c40` updates player bag flags through `0x14122ba40`; it does not
populate the equipment array or change the initial weapon-set selector.
Source diagnostics identify `ChCliItemStorageEquipment.cpp` and
`ChCliItemStorageBags.cpp`.

Private bundle: `~/re/gw2/notes/209026-inventory-empty-state-1.json`
(1,064 records, 23,549,514 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_e4ff87fba5e2c19201fa96e8fdceadc18c69b2b590e07d069998381f6a5bab04` | Constructor field initialization and array zeroing. |
| `ev_acf5e65462b8fc5ca9de46396a7ef81d5d868f75e62d8ddd650723ea6c08a308` | Vtable entries and constructor tail. |
| `ev_4bc8da6fa9a54704c1a7f444a45e83cce5d3aac5741931d88400a097bcded55b` | Inventory getters and bag-count helper. |
| `ev_22b5d48a1e1309c6d077843fbfa8b53eac673df63e008a2c33b1001e85b61d1a` | Equipment bound, slot helper targets and bag flag updater. |
| `ev_2c4bd05dc910fbe047d8b178233205dbf6df6d094d418703ad6ad929f24f42fb` | Complete selected-slot helpers and equipment assertion. |
| `ev_9231415cd93c2b67847f6f8f7107d1a9af8ffbdacb4ca6b888723cd1a70ecc06` | Weapon-set mapping functions and availability getter target. |
| `ev_503068ec7460627820e322e446cb30f6c70b07336b34c29719e1b2b4cf3265f2` | Exact sentinel targets, six-slot constants and source/weaponSet literals. |

## CoreStats constructor: embedded transformation listener

The unconditional registration at `0x141207bfb..0x141207c26` gets
character primary virtual +0xd8, stores its result at coreStats +0x138,
and calls result virtual +0x40. This initially looks like another nullable
component dependency, but the resolved chain establishes embedded storage:

- Primary character vtable `0x142160be0` +0xd8 is `0x1411d88c0`, which
  returns the address **character +0x548**, not a pointer loaded from it.
- Character construction initializes that address through `0x141287a80`
  at `0x1411d2d12`, installing vtable `0x14217a190` and empty listener state.
- That object's +0x40 method is `0x141288410`, which adds the supplied
  listener to its collection. Its assertion source explicitly identifies
  `Game/Char/Cli/ChCliTransformation.cpp` (`0x14217a1e0`). CoreStats destruction
  unsubscribes through virtual +0x48.

Therefore this constructor path registers with an already constructed
transformation object. It does not establish an additional network
component-creation prerequisite. The separately allocated inventory remains
a prerequisite at finalization, as described above.

The complete coreStats constructor also zeroes the three twelve-Dword
attribute arrays (+0x48/+0x7c/+0xb0), the three floats at +0x120 and both
Dwords at +0xe0/+0xe4. The zero helper `0x1409b7550` forwards to the byte-fill
routine with fill value zero and the supplied byte count. Thus server 183
leaves the extra fields absent from its layout initialized, rather than
uninitialized. This does not establish that zero values are useful gameplay
values, that later listeners cannot change them, or that 183 is sufficient
for the locally controlled player.

Private bundle: `~/re/gw2/notes/209026-core-construction-dependencies-1.json`
(963 records, 21,911,700 bytes). Static analysis only; no client launch.

| REA evidence | Observation |
| --- | --- |
| `ev_955d29cde26226585e4dcb5bb6103bb14a8d215f2f7b373e60bf639ef9e741cd` | Constructor zeroing and unconditional component getter call. |
| `ev_b4090e77a322d3371aeebe6d9a177be6c0f4112e578e28e6fdb69a98244a5564` | Getter vtable entry, complete registration/destruction and zero-helper wrapper. |
| `ev_8ad7ecce214fcab8ea3c4ed458610ff25fd5076879a0ee933dd062c575d1e2e4` | Getter returns embedded +0x548 address. |
| `ev_880c8660c1325ccc659e3fc5a87ee9412feef450550924d666701cd2fe2488d2` | Character initializes the embedded object; inventory registration at finalization. |
| `ev_ac98ca10a5c389aadf314ac813e4040066ce7700e5cb38008331144859517035` | Embedded object vtable and listener-state initialization. |
| `ev_e29068dd57b162e7fb1f648874ee6205ce6d359ca672c8abedfc5b34af9d443e` | Listener method pointers and zero-fill implementation. |
| `ev_13cccb9031d259e5450348d4ea1712a8b5e6f820efb7b83733325a2ccc764e7f` | Listener registration body with direct transformation source reference. |

## Minimum known ordering constraints

The [chatter-registration follow-up](209026-character-profession.md) adds
a separate **profession component** prerequisite for player finalization.
This is character +0x510, distinct from the profession scalar in coreStats.
Nine alternative creation handlers are mapped in that follow-up.

The [skill-bar follow-up](209026-character-skillbar.md) maps creation 618,
notification 619 and removal 620. Skill-bar registration can occur during
character finalization or during creation on an already-finalized character.
Its normal rebuild needs profession, coreStats and inventory. A controlled
character's profession-state update requires a skill bar; the current trace
does not make skill-bar creation a strict prerequisite of server 170 itself.

These are dependencies, not a complete playable bootstrap or an exhaustive
ordering of event listeners and content setup.

| Operation | Prerequisite established by this trace and linked findings |
| --- | --- |
| Server 282: inventory component | Character exists through server 37. |
| Server 182 or 183: coreStats | Character exists; component not already created. Player path also needs associated player and owned coreStats; for a fresh owned character, 443 precedes this creation, which stores its component before the level callback. |
| Server 443: owned-character assignment | Player entry and character exist. |
| Server 477: fresh Guardian component | Character exists and profession component is absent. Its inspected initialization adds no coreStats/inventory/skill-bar dependency. |
| Server 217: endurance creation | Character exists and endurance is absent. Constructor registers with the global settings service; no direct additional character-component dependency was found. |
| Server 256: fresh health creation | Character exists. Direct constructor adds no other character-component requirement, but the health event precedes the state-byte store and its listeners remain untraced. |
| Server 151: attitude | Character exists; an earlier update would be ignored. |
| Server 605: initial skill availability | Player entry exists and type-64 skill key resolves; direct path does not require an owned character. |
| Server 565: reward-track manager creation | Player entry exists and manager is absent; four type-353 IDs and four raw slot bytes initialize it. Content validity and runtime listener effects remain open; see [reward tracks](209026-reward-track-messages.md). |
| Server 612: palette selection | Player entry, owned character and skill bar exist; immediate rebuild has profession/coreStats/inventory dependencies. |
| Server 609/613: live skill availability change | Player entry, owned character and skill bar exist; type-64 skill key resolves. |
| Server 640/642: specialization update | Player entry; changed non-null specialization requires owned character during progress recalculation and conditionally reward-track manager. Any existing skill bar is rebuilt; see [specialization trace](209026-specialization-messages.md). |
| Server 170: fresh owned-player finalization | Player association, attitude, owned-character pointer, owned coreStats, inventory and a profession component. |
| Server 444: equal-ID controlled assignment | Player entry and finalized character exist; owned-character branch also directly dereferences coreStats. Local-player subscriptions use embedded objects; control-change listener dependencies remain open. See [control branches](209026-character-control.md). |

Appearance/content validity and further component dependencies remain open.

### Remaining direct finalizer callbacks

The player branch of `0x1411da260` calls `0x14120e380` on player +0x9f58,
which has the owned-coreStats dependency above. Its calls on embedded
player +0x9cc0, +0x4f20 and conditionally +0x18c8 all target `0x140163990`,
a `ret 0` stub in this frozen executable. Those three direct calls do not
introduce additional component allocation requirements. The lookup before
each call treats player index zero as absent; a player record at index zero
does not satisfy this path or the already documented GetPlayer assertion.

The finalizer then calls `0x1411b61f0`, which dispatches the character to
context +0x230 listeners through virtual +8, and tail-calls `0x1411e3ab0`.
The latter reads character flags, the stored health state and context
services, and updates character +0x230. Its inspected health checks read
the stored state directly, not a health-component pointer. This is not an
exhaustive proof of listener behavior or of all component requirements;
the context event dispatch remains a runtime-dependent boundary.

Private bundle: `~/re/gw2/notes/209026-finalization-tail-1.json`
(1,069 records, 23,621,602 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_dc9a7a3d716dae2c7dc8a58a72dfc2f0ed68ddb79f3c66bd9b1851116cc86c83` | Finalizer's full player branch and direct tail calls; runtime-function entries split this function. |
| `ev_ff79b3a2fef83dc29107b8baf574d79b39d23d2d9d4cf2b7b6941931b137d413` | No-op target, context dispatcher and complete state-update tail. |
| `ev_aac9e274019f8ab586de720520bb1ebce00eed4fc80a2e00a05a89f6585ab9dd` | Constructor-installed interfaces and tail getter targets. |
| `ev_3aa34414d0c8b22199e0fc38a53f3f2d084858554437d4cc3680e344948e06e2` | Flag/stored-health getters and event virtual +8 trampoline. |
| `ev_4f9012f2ebcbc00d02858f0d20ae0eb1c869fb04f1687f8aebb8ba9a443bab95` | Interface-relative health getter and event-dispatch setup. |

## Offline framing check

The hash-guarded extractor wrote private mode-0600
`data/protocol/209026/character-inventory.xml` (310 bytes, SHA-256
`fc4ebd61a11b7aa4d7faca0cd93c9037331de6c16299dcad7cc42969157bb57b`).
The existing census parsed two hand-authored ID-282 frames of lengths 3
and 4, consuming all seven bytes. Removing the last varint byte was
rejected. No messages were sent and no new build was needed.

## Evidence

Private bundle:
`~/re/gw2/notes/209026-character-finalization-dependencies-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_5921847d25d2c1ee62d961f94ae38198493f4054ee31a67715bd6c4a55c997af` | Finalizer's player-component call and ordering. |
| `ev_0b5c8d6cceabca92c9ca4be6f15d41db7a367315e439d917eea1e9ad90a9e0ba` | Owned-character/coreStats dereferences and inventory subscription. |
| `ev_c268f7fdfc7df3966c420c6eac94c5716d557dfbc37c95928431baf061f450f0` | Named registered-component assertions, vtable, listener registration and owned setter. |
| `ev_072d1239bc3d9a4eaa10350373e2602b4c90ec9408569dcdc7d3fe60d1caca8e` | Inventory getter and initially null owned-character pointer. |
| `ev_4822bbdbbe93c1c69e143349d4d6e05547c3d624eb308ded02b48d9df3ea7644` | Inventory allocation, null initialization and m_inventory assertion. |
| `ev_55aacfaa52c9fb082fdc924385da88807ab81438e0f2ca2eabf5e873c2b78362` | Inventory allocator's handler call site. |
| `ev_7d2834447258349907994a1ea50dc4ce0c54a00321c3378489fd361e3b5e353f` | Handler's raw lookup and registration pair. |
| `ev_5719e2f51a7e3ff55480696b50d9862f187b9aba37319284e9faac5135ade32f` | Hash-guarded server-282 definition and inventory constructor. |
| `ev_7308c417bb286b493256205b87b90a6ea045a7c312364c7e8f215965c0516cd6` | Hash-guarded single-message schema extraction. |
