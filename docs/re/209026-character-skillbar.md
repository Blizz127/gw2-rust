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
