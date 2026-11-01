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
  derived component state. Named consumers establish this as **profession**
  (see the getter trace below).
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

## Profession getter trace

The component's primary vtable at `0x142168da0`, virtual +0x28, points to
`0x141027030`, which reads the Dword at +0x12c. Two independent consumers
identify this value:

- The profession-icon widget gets character coreStats through virtual +0xb0,
  then reads virtual +0x28 at `0x1403bb3b1`. Nearby assertions name
  `AsProfessionIcon.cpp` and `coreStats`. The value reaches `0x1411c6040`,
  which recognizes integers 1 through 9, then the icon-selection path.
  An unrecognized value can be replaced by a separate UI lookup; that is not
  evidence that an arbitrary wire value is valid.
- The trait initializer reads the same getter at `0x14120ebaa` and passes
  it to character-context virtual 0. That function, `0x1411b5270`, looks up
  context +0x310 indexed by the value, returning null for indexes >= 11.
  The caller's assertion explicitly names
  `professionDef->GetTraitLineCount() == lineCount`.

This establishes the field's meaning without assigning names to individual
profession numbers. The component setter itself does not establish a valid
content definition for every byte value.

The two level fields remain separately identified by storage and getters:
virtual 0 (`0x141207eb0`) and +8 (`0x141207f20`) normally return +0xac but
can return 80 under player-state conditions; virtual +0x10 (`0x140282470`)
reads +0xe0 directly. Base/effective-level names still need a named consumer.

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

Getter follow-up bundle:
`~/re/gw2/notes/209026-character-core-scalars-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_02f7b0482c7233716a3ff82c217d2e7c6cdc3db0acddbfe9afc09f303a1b728b` | Component initialization and dependent scalar calculations. |
| `ev_5e55721e28ad033a5ff881ca8e274468032dcab0a505519bd6bc3bebaf757e9e` | Primary vtable and +0x12c getter. |
| `ev_485aea3ea2333ffbf313a8649a428ac1d686891657f21bf3824dda8bc2530de1` | Hash-guarded getter-consumer search with named icon/trait assertions. |
| `ev_06b3d5ae719782fa1bd830877e96afb3a40f10fd91e8f45bc804a728e5d901e0` | Bounded profession-icon data flow. |
| `ev_82bd2af93f4f8a87f05446ff3520d0c8f8c16581cbd79a1a809f7795b2ed4e75` | Context virtual 0 and 1..9 predicate. |
| `ev_14cc742d4eb4f48e92af23e0f4eac334e09babcf63f981ff460c80d71e4a4371` | Context profession-definition lookup and appearance getters. |
| `ev_13a15aec3d9eb243234ccb90fc7a4542c25fd83ad2a153782622b0716a1c1c77` | Level getters and conditional 80 results. |
