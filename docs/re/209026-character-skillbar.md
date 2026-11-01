# Build 209026: skill-bar lifetime and initialization

Static REA findings, 2026-11-01, on the frozen executable identified in
[mission transfer](209026-mission-transfer.md). No client replay occurred.

## Creation, notification and removal

All three messages are protocol 0, group 0, and contain exactly one
**Dword character index**. Their registration pairs start at `0x1421709b0`.

| ID | Descriptor | Handler | Effect |
| --- | --- | --- | --- |
| 618 | `0x1425d8310` | `0x14125bc60` | Raw-character lookup, then `0x1411d6610`: requires `!m_skillbar`, allocates 0x298 bytes through `0x1411f4720`, stores character +0x520. |
| 619 | `0x1425d8460` | `0x14125bcc0` | Calls `0x1411dbc00`: requires `m_skillbar`, emits a character event. |
| 620 | `0x1425d85d0` | `0x14125bdc0` | Calls `0x1411d6a60`: requires the component, invokes its deleting destructor through the +0x50 interface, clears character +0x520. |

The assertions are directly named by literals `0x142142138` and
`0x142142148`. The source diagnostic at `0x1421658d0` identifies
`ChCliSkillbar.cpp`. Character construction starts the pointer at null.
618 has no skill IDs, palette entries or slot data in its wire layout.

619's event uses dispatcher `0x1411d4030`, which invokes listener virtual
+0x3d0 through the character event collection at +0x1e0. The inspected
helper does not allocate a component, set character finalization flags or
directly call the skill-bar registration routine. Its complete downstream
listener effects remain open; calling it a second character-finalization
message would overstate the evidence.

## Registration timing and dependencies

Character finalization `0x1411da260` calls `0x1411f9530` **only if**
character +0x520 is nonnull. That helper registers skill-bar listeners with:

- Character primary +0x108: the embedded +0x40 combatant interface.
- Character +0x388: coreStats, immediately passed to its listener method.
- Character +0x3f0: inventory, immediately dereferenced for virtual +0x540.
- Character +0x548: the embedded transformation object.

The stored registration fields are +0x158/+0x160/+0x168/+0x178. Assertions
name registeredCombatant, registeredCoreStats and registeredInventory.
After subscription, it invokes skill-bar virtual +0x1f8.

The skill-bar constructor checks the owning character's finalized flag
(+0x178 mask 0x10) at `0x1411f4a10`. If already set, it performs equivalent
registration and calls the rebuild itself; otherwise it skips that block.
Therefore this code supports registration on either side of character
finalization. It does **not** establish a strict requirement that 618 must
precede 170. It also does not prove every later consumer tolerates an absent
skill bar or that every creation ordering is valid.

The primary skill-bar vtable `0x142164cd8` +0x1f8 points to
`0x1411fa2d0`. This chooses `0x1411fa910` for a downed player and
`0x1411fa340` otherwise. The latter reads character +0x510 as its profession
pointer. Although some branches check for null, the loop for slot values
12–18 calls profession virtual +0x10 without a null check, and its tail at
`0x1411fa8ce` calls profession virtual +0x170 unconditionally. Thus the
normal rebuild requires the separately created profession component.
Creating a skill bar on an already-finalized character reaches this path
inside its constructor. A pre-finalization skill bar reaches it when
character finalization invokes the callback.

The profession state setter `0x1412408e0` supplies another dependency:
when its owning character equals the context's controlled character, it
loads character +0x520, asserts **skillbar**, and calls `0x1411f9500`, a
wrapper for skill-bar virtual +0x1f8. An active controlled-character
profession-state update therefore cannot rely on the finalizer's null
guard to make skill-bar absence safe.

Controlled-character assignment itself calls `0x1411e0a40`, which changes
flags, updates an embedded object and emits events. This trace does not
claim that the direct setter alone unconditionally dereferences the skill
bar; the event-listener graph remains incomplete.

## Slot updates and auto-attack selection

Registration continues at `0x1421709e0`. Every message below starts with
a Dword character index and requires an existing skill bar through
character primary virtual +0xc8. These are field mutations, not skill-bar
allocation or proof of useful initial skill choices.

| ID | Descriptor / handler | Remaining fields and inspected effect |
| --- | --- | --- |
| 621 | `0x1425d8740` / `0x14125bd20` | Byte slot. Calls `0x1411fb680` with value zero to clear that slot's bit in component +0x128. |
| 622 | `0x1425d8830` / `0x14125be20` | Byte auto-attack slot. 255 maps to signed -1; other values are zero extended. Calls `0x1411f9e90`. |
| 623 | `0x1425d8950` / `0x14125bec0` | Dword auto-attack mask. Calls `0x1411f9ef0`. |
| 624 | `0x1425d8a70` / `0x14125bf50` | Dword content key. Nonzero resolves type 299; zero supplies null. Virtual +0x260 stores component +0xc8 and notifies. Record's gameplay role unresolved. |
| 625 | `0x1425d8bb0` / `0x14125c010` | Dword content reference, resolved through content virtual +0x230 and passed to skill-bar virtual +0x268. Full effect unresolved. |
| 626 | `0x1425d8dc0` / `0x14125c0d0` | Two Dwords: content reference and engine-agent index. `0x1411fb240` stores resolved content at +0x130, updates the agent reference at +0x138 and triggers normal rebuild. |
| 627 | `0x1425d8e90` / `0x14125c1a0` | Float, passed to `0x1411fb280`, which updates a field through the object at skill-bar +0x150. Gameplay meaning unresolved. |

621's setter explicitly asserts `skillbarSlot < CHAR_SKILLBAR_SLOTS` with
upper bound 23. The bit's gameplay meaning is not yet named. It can also
schedule a callback and notify listeners when the old value was set;
do not equate clearing it with deleting the skill in that slot.

For 622, a changed selection stores component +0x88, clears +0x8c, then
sets one bit in +0x8c unless selection is -1. For 623, a changed mask is
stored at +0x8c and +0x88 becomes its lowest set-bit index, or -1 for a zero
mask. The iterator constructor `0x1409be9a0` isolates the lowest bit and
`0x1409bf150` obtains its index using BSR. Both setters notify listeners.

Consumer `0x1411f63d0` calls `0x1410f62f0`, which reads the named setting
**EnableMultiAutoAttacks** at `0x14213f6a8`. With it enabled, the consumer
iterates +0x8c's set bits and indexes the skill pointers at +0x1d0. With it
disabled, it indexes the same array using +0x88. This establishes the
auto-attack role of 622/623 without assigning names to every skill slot.
No range sanitization is visible in these two setters; their ability to
store arbitrary bytes/masks is not proof those values are safe to use.

Private bundle: `~/re/gw2/notes/209026-skillbar-slot-updates-1.json`
(1,024 records, 22,915,210 bytes, mode 0600).

| REA evidence | Observation |
| --- | --- |
| `ev_60621a526d2641c72dda8efed24a2ad48f217c2b51650b04c40ef73dd9b4d6c8` | Registration pairs, 622–624 handlers and slot-flag setter. |
| `ev_5080c941ae3f22ff573c3219bea26008e3692c0003fb826358685db48aae57f3` | Hash-guarded 621–627 descriptors, selection/mask setters, 625–627 handlers. |
| `ev_ed4af17929eda91173312092de7160732ad994b4a4a5e295d9c8d52a114c4042` | Named setting search and bounded field-use census. |
| `ev_5ba7fd4244aa28d5add7bd6070300563912745959e9a2c2007e74dfdd32c4bdb` | Single/multiple-slot consumer, setting lookup, 624 setter and mask iterator. |
| `ev_71f25d63585fccaa1ab88f65c9362673fa3477eab3847905ed422b8a85428b0a` | Setting literal, bit-index helper, corrected consumer entry boundary and 626/627 setters. |

## Typed lifecycle and selection frames

`gw2_proto::skillbar::SkillbarFrame209026` covers IDs 618–623 with an
explicit character index and operation enum. It preserves the raw slot
byte (including 255) and all mask bits. It does not choose skill contents,
enforce slot ranges or component lifetime, or interpret 619 as character
finalization. IDs 624 onward remain outside this typed subset.

Three focused tests and `cargo build -p gw2-proto --lib` pass. Hand-authored
fixtures cover six layouts, the byte sentinel, a zero mask, the full u32
mask, raw out-of-gameplay-range slot retention, output prefixes and exact
boundaries in a concatenated stream. All truncations, invalid neighboring
IDs and overflowing/overlong Dwords fail without advancing input. These
are offline framing checks; no messages were sent.

## Limits and evidence

These findings add component lifetime and rebuild constraints, not valid
skill contents or a completed playable bootstrap. The pending User
Agreement hold is unchanged. No messages were sent; no new build was
needed for this static documentation increment.

Private bundle: `~/re/gw2/notes/209026-skillbar-lifecycle-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_408febce227bab694f1beec323431913a299ab982d7403e6c26cc3055e7cb2b1` | Registration body, skillbar literal and controlled-player setter. |
| `ev_6be7ab113ac73e740bd7430f39cc206679ad277e1ef7dc5f24458af7453736b1` | Allocation/null/removal stores, control setter and source diagnostic. |
| `ev_b1203f88c4c9417fd5cbdda6c5e0b8af40a5a1a53d3f1ed192cd767ceaa4056b` | Control-state update events and allocator range. |
| `ev_5eb10d008c9dbfb1f5323de97c290fcedd9099d554fff6bfa82c8e9e22c56486` | Creation caller, complete allocator entry and constructor initialization. |
| `ev_96420d44a32a7955763bbc7de5d463689bfdea5117f118a25269a49cc1a9ab62` | Registration pair, constructor finalized-flag branch and rebuild virtual slot. |
| `ev_01d4ef39d58bf02ab2b94730ae5cf586b4bddbccd201a4990608066bc956023f` | Hash-guarded 618–620 descriptors, constructor registration tail and rebuild dispatch. |
| `ev_6a005a85f8e5fdeaa3ecfd82498786cc01f734d080f8626cf64a2f437d038ff2` | 619 event helper and normal rebuild's unguarded profession uses. |
| `ev_31ae349fd14060efcd0274705f23baceac0a4f1aeae3d65365779a8144cb6415` | Exact removal registration, 620 handler, event virtual offset and named registration fields. |
| `ev_5f1cb3f216d48e3fd8a7d9cfc9a7c06ac8aff5739f410046ba2f817e8b8c3a56` | Profession-state setter requires skill bar for the controlled character. |
| `ev_dc04dafc4618ab4536860ce447e13dc922819ce9d9fd18b21ff9d5eeb2e464a4` | Character finalizer skips the skill-bar callback when absent. |
