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
- `0x141208f50` with the first scalar byte: stores earned level at +0xac
  (named-consumer evidence below).
- `0x141209100` with the second scalar byte: stores another level at +0xe0.
  Both setters assert level >= 1 and level <= 80 for players, <= 100 for
  nonplayers. The second level's gameplay name remains unresolved.
- `0x141209440` with the third scalar byte: stores +0x12c and updates
  derived component state. Named consumers establish this as **profession**
  (see the getter trace below).
- `0x1411da600`: checks the component exists and dispatches a character
  event; it does not set the character's finalized flag.

Server 182 additionally supplies a Dword, three floats and twelve records
of three Words. Its handler iterates record indexes **1 through 11**, skipping
record 0, and passes each record to three setters. The **third Word** is
sign extended; the first two are zero extended. This corrects the earlier
note's mistaken attribution to the middle Word. The verified stores and
remaining naming limits are recorded in the follow-up below.

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

The two level fields are separately identified by storage and getters:
virtual 0 (`0x141207eb0`) and +8 (`0x141207f20`) normally return +0xac but
can return 80 under player-state conditions; virtual +0x10 (`0x140282470`)
reads +0xe0 directly. The first getter is used as earned level by the
named consumer below; an equally direct name for the second remains open.

### Earned-level consumer

At `0x1406e5331`, the leaderboard-reward path gets coreStats through the
character +8 interface's virtual +0xa8, calls coreStats virtual 0 with
argument zero, and passes the result as the first argument to
`0x1411c29a0`. This wrapper forwards to `0x1411c29c0`, whose first-argument
bound assertion explicitly names
`playerEarnedLevel <= CHAR_MAX_LEVEL_PLAYER` (`0x14215fbf0`).

The interface chain is resolved: character constructor `0x1411d2810`
installs `0x142161250` at +8; its +0xa8 entry is `0x1411d7640`. This getter
asserts `m_coreStats` and returns interface +0x380, equivalent to whole
character +0x388. CoreStats virtual 0 is `0x141207eb0`, whose usual result
is +0xac. This establishes earned-level usage without relying on which
of two otherwise unnamed level bytes appears first in the message.

The getter's override is material: with its argument zero, a player and
the checked player-state predicate can cause it to return 80 instead.
A nonzero argument bypasses that override. Virtual +8 has a separate
additional context predicate for returning 80. These conditions have not
been fully named; the wire byte is stored unchanged at +0xac after its
level-range checks. The second byte is separately stored at +0xe0 and is
not assigned a base/effective-level label by this trace.

Private bundle: `~/re/gw2/notes/209026-core-earned-level-1.json`
(956 records, 21,817,044 bytes), static REA only.

| REA evidence | Observation |
| --- | --- |
| `ev_746ada4d530eea3bdc97392df5220f02329236e88da27c88564dfe137b4e3cfb` | Named first-argument assertion and both level getter override paths. |
| `ev_a95cab6946dbdfa033f2f8fc8a0110741e563bd8e11e051088df179e2e44f450` | Forwarding wrapper. |
| `ev_f8bee6de0126db5826f2c7a3d6169143afd0824436618bb7ea0a3752c47fed84` | Reward consumer gets the first level and supplies both calculation arguments. |
| `ev_ea3a8c48e2f723a33ffffa0f30d678978fa13c0382c1d34d8d7f56d24e3af60e` | Character +8 interface installation. |
| `ev_cbec641779bc4fde4a24d2676885d21b8ab8a1fe23599191b565362dc91c4aca` | Interface getter pointer and direct second-level getter. |
| `ev_685304f56d4dd1556c6ffa56c265e007c61d780f9b33dc413605f550dbac269e` | CoreStats getter and complete level setters. |
| `ev_d81bd2559bad547831312b58160451d6b4115c72693d1302cf0ba7f2068dced1` | Exact named assertions and subsequent update loops. |

## Appearance and extended-record follow-up

The appearance setter `0x141208c70` copies exactly 45 bytes to component
+0x18. Primary virtual +0x30 resolves to `0x14026f760`, returning that
same address. The scalar getters preserve the previously established
appearance offsets:

| Appearance offset | Component getter | Behavior |
| --- | --- | --- |
| 27 | `0x141207d50` | Byte at component +0x33, race. |
| 28 | `0x141207d60` | Byte at +0x34, converted by `0x1411c30c0`. |
| 29 | `0x141207d70` | Byte at +0x35, sex. |

Setter `0x1412092f0` names byte 28 through assertion
`scale <= CHAR_APPEARANCE_SCALE_MAX` (`0x142169190`) and compares against
15. The conversion helper returns 1.0 for values above 15; for 0..15 its
integer instructions recover the original value, then separate f32
multiply/add instructions compute:

```text
scale = f32(value) * f32_bits(0x3c5a7411) + f32_bits(0x3f666666)
      ≈ value * 0.013333333 + 0.9
```

The multiply constant is at `0x142160410`, the add constant at
`0x141933f58`, and the fallback 1.0 at `0x141924c5c`. This identifies
the appearance scale conversion. It does not establish a formula for
server-37 capsule height/radius or verify rendering. The content checks
for the 45-byte appearance are documented separately in
[appearance validation](209026-appearance-validation.md).

Server 182's nested descriptor `0x1425dcf80` contains exactly three Words.
Its handler uses a six-byte stride and these offsets for rows 1..11:

| Word in wire order | Extension | Setter | Stored Dword |
| --- | --- | --- | --- |
| 0, offset +0 | Unsigned | `0x141209080` | coreStats +0xb0 + 4*row |
| 1, offset +2 | Unsigned | `0x141208e90` | coreStats +0x7c + 4*row |
| 2, offset +4 | Signed | `0x141208d00` | coreStats +0x48 + 4*row |

The decisive instruction at `0x14124def1` is `movsx r8d, word [rax+rdi+4]`;
the other two loads are `movzx`. Record 0 is present on the wire but skipped
by this handler. Bounds assertions name attribute arrays, but the +0x48
getter/setter use different `m_earnedAttributes` / `m_effectiveAttributes`
assertion strings. This note therefore retains concrete offsets rather than
assigning an unsupported semantic name to each column.

The additional Dword is passed to `0x1412091e0` with arguments 0 and 0x13;
it is stored at +0xe4 after a conditional cap. The trailing three floats
go to `0x141209340` and are stored, in wire order, at +0x120/+0x124/+0x128.
Their gameplay names remain open. These stores do not establish whether
183 alone is sufficient for the locally controlled player.

Follow-up private bundle: `~/re/gw2/notes/209026-core-stats-appearance-1.json`
(948 records, 21,731,440 bytes). No executable code was changed or client
launched for this follow-up.

| REA evidence | Observation |
| --- | --- |
| `ev_b26be48cce6fe74074ae60e28980e0284e91d2c7226a742158b589d80d83676f` | Appearance copy, primary vtable and attribute stores. |
| `ev_3f0940ec069f4208360ff0ce9a97b4fe48e44c0e387fbc331bdb2b2f9a0ceea6` | Race/scale/sex getters and attribute assertion strings. |
| `ev_10f6c004f3a8e4bf203e0348a625a7d552c51c5e2c6e9997d5ad27b1c82b5575` | Appearance pointer, scale conversion, complete 182/183 handlers. |
| `ev_45660f76758671a8a58320389a7bdfb084cd561d4b820e3e114633818a6ed4ab` | Hash-guarded 182 descriptor, remaining stores and scale constants. |
| `ev_1edec003e99c01c483f57aefa2fa3e2026bd7c5e6959dd07d6c1dfc18e2bebfc` | Three-Word nested descriptor, attribute notifications and named scale assertion. |

## Offline validation

### Typed creation-frame scope

The typed core-stat creation codec supports the two established alternatives:
server 183 has only the common fields, and server 182 additionally has the
Dword, twelve three-Word records and three floats. It preserves row 0 even
though the client handler skips it, represents each third Word as signed
16-bit data, and preserves all float bits. The common fields remain an
explicit character index, 45 appearance bytes, earned level, second level
and profession. No content defaults, valid-level claim or choice of variant
for the controlled player is supplied. Decoding consumes exactly one frame
transactionally, including its ID; fixed arrays impose fixed memory bounds.

Implemented as `gw2_proto::core_stats::CoreStatsCreation209026`, with an
explicit optional `ExtendedCoreStats209026`. Three focused tests and the
library build pass. Hand-authored 52-byte common and 141-byte extended
fixtures verify exact wire bytes and following-frame boundaries; distinct
attribute columns check signedness at -32768, -1 and 32767; float checks
preserve negative zero, a NaN payload and infinity. Every truncation and
malformed ID/overflowing Dword leaves the input unchanged. These intentionally
include values unsuitable for a live player and establish framing only.

### Descriptor/census validation

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
