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

## Minimum known ordering constraints

These are dependencies, not a complete playable bootstrap or an exhaustive
ordering of event listeners and content setup.

| Operation | Prerequisite established by this trace and linked findings |
| --- | --- |
| Server 282: inventory component | Character exists through server 37. |
| Server 182 or 183: coreStats | Character exists; component not already created. |
| Server 443: owned-character assignment | Player entry and character exist. |
| Server 151: attitude | Character exists; an earlier update would be ignored. |
| Server 170: fresh owned-player finalization | Player association, attitude, owned-character pointer, owned coreStats, and inventory for the finalized coreStats. |
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
