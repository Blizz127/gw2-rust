# Build 209026: attitude and core-stat messages

Static REA analysis, 2026-11-01, on the frozen executable identified in
[mission transfer](209026-mission-transfer.md). No client replay occurred.
These messages belong to protocol 0, group 0's character registry.

## Attitude: server 151

Descriptor `0x1425d54f0`, handler `0x14124b300`, contains two **Dwords**:
character index and attitude. The handler looks up the raw character; if
it is missing, it returns success without applying the update. Otherwise
it calls `0x1411da150` with field 1.

The setter checks `attitude < Content::AFFINITY_ATTITUDES` (signed upper
bound 4), writes character +0xc0, and notifies listeners. Finalization's
`m_attitudeTowardControlled` assertion reads that same field. Thus an
attitude message sent before creation would not initialize the later
character. The enum labels and valid lower bound remain unassigned.

## Core-stat creation versus appearance update

| Server ID | Descriptor | Fields after character-index Dword |
| --- | --- | --- |
| 183 | `0x1425dccf0` | ArrayFixed(45) Byte; Byte; Byte; Byte |
| 182 | `0x1425dd0f0` | Same as 183; Dword; ArrayFixed(12) of three Words; Float; Float; Float |
| 185 | `0x1425dd440` | ArrayFixed(45) Byte |

Handlers `0x14124dd10` (183) and `0x14124dde0` (182) require an existing
character and call `0x1411d60a0`. That helper checks **!m_coreStats**, then
allocates/constructs the 0x288-byte component at character +0x388 through
`0x141207a10`. They get the component through the character's +8 interface
virtual +0xb0 and check the explicit **coreStats** assertion.

Both then invoke:

- `0x141208c70` with the 45 bytes: copies them to coreStats +0x18,
  compares against the prior bytes at +0x140, and notifies on change.
- `0x141208f50` with the first scalar byte: stores a level at +0xac.
- `0x141209100` with the second scalar byte: stores another level at +0xe0.
  Both setters assert level >= 1 and level <= 80 for players, <= 100 for
  nonplayers. Their exact base/effective-level naming is not established here.
- `0x141209440` with the third scalar byte: stores +0x12c and updates
  derived component state. Its meaning remains unresolved.
- `0x1411da600`: checks the component exists and dispatches a character
  event; it does not set the character's finalized flag.

Server 182 additionally supplies a Dword, three floats and twelve records
of three Words. Its handler iterates record indexes **1 through 11**, skipping
record 0, and passes each record to three setters. The middle Word is sign
extended. Their content meanings are not assigned here.

Server 185's handler `0x14124df50` requires an existing coreStats component,
then only applies the 45-byte update through `0x141208c70`. It does not
allocate the component or perform the initialization event above.

**Ordering implication:** 182 and 183 are alternative component-creation
paths, not two initialization packets to send consecutively to one fresh
character. 185 cannot replace that creation step. This does not establish
which creation variant is appropriate for a local player, valid appearance
content, or the complete set of components needed before server 170.

## Offline validation

The hash-guarded extractor emitted these four messages and the already
mapped owned-character message 443 to private mode-0600
`data/protocol/209026/character-core-stats.xml` (1516 bytes, SHA-256
`c7c79f92b93124ba6a8785e365c4318a8e73ca9887ce6d89bf5b4f4a3babb43b`).

The existing census parsed five hand-authored frames completely: IDs
151/183/182/185/443, lengths 5/52/138/49/5, total 249 bytes. This checks
fixed arrays, a multibyte character index and consecutive boundaries.
The concatenation deliberately includes both creation variants solely as
parser input; it is not a valid initialization sequence and was not sent.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-core-stats-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_a77862083dfcfdf17f269cbefd1d2fef972f0569a8c2ea041dc1a3654630994e` | Attitude setter and assertion. |
| `ev_af235f48d31f970cb38e29db3c93ee0d21d4c31916814659bf9754014b59e4d7` | Attitude handler ignores missing characters and registration pair. |
| `ev_7e568d24da673b1e8780d15f413f0b988373a2a7e54ac15f5a3f7362ef5094ae` | Server 151 descriptor. |
| `ev_ef64c6d12f48fc122ca53786b4657cf3b6c68c3b43e16848357f3ad32e298e11` | Hash-guarded extraction of 45-byte appearance descriptors. |
| `ev_9e369ac98fa470c726d8838c54dd103a397316f2b44ea1b14f8ce072df9324df` | Handlers and nested Byte/three-Word descriptors. |
| `ev_defb70213395a1e63fd42dcaef7f3d1001c2a5a7dcad9ce2a599f96ebb777977` | Component allocation, appearance copy and scalar setters. |
| `ev_65ec76d63e356fefae39028d95f9ea243d93c0043d65666761a89e4a35a1ad1c` | Named coreStats/level assertions, bounds and initialization event. |
| `ev_c892199aa16cf2d825ec4c9abac7ffa701926e10a576822c354e2c679366b22d` | Hash-guarded five-message schema extraction. |
