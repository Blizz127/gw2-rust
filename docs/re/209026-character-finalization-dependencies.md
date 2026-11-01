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
| Server 182 or 183: coreStats | Character exists; component not already created. |
| Server 443: owned-character assignment | Player entry and character exist. |
| Server 151: attitude | Character exists; an earlier update would be ignored. |
| Server 170: fresh owned-player finalization | Player association, attitude, owned-character pointer, owned coreStats, inventory and a profession component. |
| Server 444: equal-ID controlled assignment | Player entry and finalized character exist. |

Appearance/content validity and further component dependencies remain open.

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
