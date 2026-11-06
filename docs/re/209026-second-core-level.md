# Build 209026: second core-stat level and the scaling boundary

2026-11-06. Bounded static REA against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No runtime, archive extraction, decoded-container reads, or gameplay changes.

**The second level has an explicit received update, server190, but this
slice does not establish a named map-scaling rule or a value to send for
map379.** It also corrects a misleading specialization-eligibility label:
that path reads the earned field with a conditional override, not the
second level. Nothing here supports automatically making the two levels
equal or advancing effective attributes when earned level changes.

## Exact received update and bounds

One registration was checked directly:

| Property | Value |
| --- | --- |
| Registration | `0x14216f140` |
| Descriptor | `0x1425ddaa0` |
| Handler | `0x14124e250..0x14124e2dc` |
| Message ID | server190 |
| Fields after ID | Dword character index; Byte level |

The handler resolves the character, requires its coreStats component,
zero-extends the received byte at decoded message +6, and calls
`0x141209100` at `0x14124e2c8`. There is no map lookup or level calculation
in this complete handler.

The complete setter `0x141209100..0x1412091d8`:

1. Checks unsigned level >=1.
2. Uses the owning character's player predicate. It checks level <=80
   for a player, or <=100 otherwise.
3. Stores the supplied Dword at coreStats +0xe0.
4. If the notification cache at +0x234 differs, dispatches through the
   collection at +0xe8 using trampoline `0x14027259c`, then updates the
   cache. The trampoline jumps to listener virtual +0x30.
5. Calls the previously traced attribute-notification helpers
   `0x1412083c0` and `0x1412084b0`.

The named assertions are only `level >= 1`,
`level <= CHAR_MAX_LEVEL_PLAYER`, and `level <= CHAR_MAX_LEVEL_MONSTER`.
The player bound is a setter validity check, not evidence of a map-specific
cap or of the policy selecting a value within that bound.

An exact E8/E9-reference query found three validated direct calls to this
setter: `0x14124ddae` in creation183, `0x14124de89` in creation182, and
`0x14124e2c8` in update190. The first two were already traced as received
initialization bytes. This bounded query excludes neither indirect calls
nor independent writes elsewhere; it is not a complete writer census.

The [tooltip-context trace](209026-tooltip-character-context.md) already
establishes that the two notification helpers update caches and notify;
they do not replace the effective-attribute array at +0xb0 with values from
the default curve. Server189 remains a separate explicit attribute update.

## Typed consumers and the corrected specialization claim

The existing coreStats table `0x142168da0` binds virtual +0x10 to
`0x140282470`, the Dword getter for +0xe0. Established typed consumers
remain useful:

- The character-backed `SkillDescription/SdInt.cpp` adapter obtains this
  getter through character coreStats and supplies it as the fact evaluator's
  level. Its separate attribute method reads `m_effectiveAttributes`.
- The [baseline-health consumer](209026-mesmer-health-speed-baseline.md)
  also reads virtual +0x10, then uses the level-indexed default attribute
  and health tables.

These prove concrete uses of the second field. The name of the attribute
array does not independently name the level field, nor does using the
field for a displayed calculation establish its server selection policy.

The prior [specialization note](209026-specialization-content.md) described
an effective-level eligibility test. Complete predicate
`0x141224f00..0x14122507b` instead loads character +0x388 and directly calls
`0x141207f20` at `0x141224fba`. It passes that result as ECX and the slot
as EDX to `0x1411ce820` at `0x141224fc3`.

`0x141207f20..0x141207f8f` returns **earned field +0xac**, except for
its player/context predicates that select constant80. It never reads
+0xe0. The complete eligibility helper compares that supplied level
against a content-selected threshold; no threshold is assigned a fixed
retail unlock level here. The earlier label is corrected at its source.

## What the bounded follow-up did not establish

The second-level notifier is a concrete subscription boundary. The
inspected skill-bar registration subscribes its +0x28 listener through
`0x141208700`; the ordinary-view context also subscribes a listener through
coreStats virtual +0xd0. This slice does not assign their +0x30 callback
implementations or claim any callback computes a new second level.

Exact ASCII-name queries for effective/scaled/combat-level spellings and
MapLevel/LevelCap found no naming anchor. A bounded source-filename query
found several level-related UI classes, but no typed connection from those
filenames to this field. These negative results cover only the stated
spellings and source-filename pattern. The generic getter address also
has unrelated direct users; its address alone does not type their receivers
as coreStats, so those users were not promoted to character-level evidence.

The retained [map379 context-gate evidence](209026-skill-context-map-gate.md)
binds map379 to type45 at container1282833 offset3863896, with map type7,
Dword +0x9c=1 and null +0x100. The proved consumer of bit13 in +0x9c controls
skill-context overrides. Neither this gate nor the retained sector polygons
establishes a level range, level cap, or scaling formula. No unnamed map
scalar was relabeled as a level input.

**Implementation boundary:** keep the explicit earned progression separate.
Selecting a second level or new effective189 values still requires an
authored scaling input with a typed consumer, or authoritative server
policy. A server190 codec could preserve the proved raw fields, but its
existence would not supply that missing policy; none was added here.

## Evidence

Complete private bundle:
`/home/blizz/re/gw2/notes/209026-second-core-level-20261106.json`,
SHA-256 `68dbf5ed06e2f27dae19f70bdfdf7356a9a4420089a5b012159a42f5e324d78f`.
It retains all eleven captures, including bounded searches that yielded
no useful name. Each exited zero with untruncated captured output and
verified owned-process-group cleanup. Unrelated processes reported as
unverifiable by REA were left untouched. No cold decompiler was started.

| REA evidence | Exact scope |
| --- | --- |
| `ev_d981216a3acdbacf6c07da6bb44edf0c3964ae4535ff05cd94edccb423ded6d7` | Complete second-level setter, specialization predicate, and level/slot helper. |
| `ev_bc8832c1cce90c015d49137a96dad35d2a70940e90501f61c6837bd6f8c242a0` | Three validated direct setter calls; exact-address E8/E9 scope. |
| `ev_2f7a31b6dca04599f2cb8a7d005c294fff46bf1e3257062dcc32f8ed0cee50ca` | Complete update190 handler and earned/override getter. |
| `ev_d280b2a52eb4bbe035e8ad7a032b3a0a43f64a61ec2deb6aeaa8385a963f0434` | Single registration and exact descriptor kinds4,2. |
| `ev_b882156408b2ba517d997ae4935c297e210d70001250ccada61ec1950f1c7169` | Notifier trampoline virtual+0x30 and coreStats vtable binding. |
| `ev_fe3dab071167535221804b1a41ee3410fd1e12dbf311bcd2453cd46f433e3bce`, `ev_abeb834e399728c27e969ed501d735732eab5120f1ab237d3d2495d1fb9fd62e` | Concrete subscription arguments; no new callback semantics claimed. |
| `ev_ba132af6bfc22644c1996f57acb73e245ae47c1874e59cb5954a68303e7ff191`, `ev_0591d13ad6da38a9f96993cb6297f0b418721fef434421524f892df462bf47f5` | Bounded name/source-filename queries and their limits. |

Instructions past the notifier's six-byte leaf and incomplete logical
functions ending at a pdata fragment are not used as complete-body proof.
Only the already established subscription behavior is carried forward.
