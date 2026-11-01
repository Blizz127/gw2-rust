# Build 209026: ordinary view callbacks during control assignment

Static REA follow-up to [controlled-character assignment](209026-character-control.md)
and [view finalization](209026-character-view-finalization.md). No client
or control-assignment replay occurred.

## Skill bar required by the subscribed ordinary view

The AvCharBase character listener at whole-view +0x28 has vtable
`0x142141558`. Its control-change virtual +0x60 resolves through entry
`0x1421415b8` to **`0x141109d00`**. The callback asserts that its character
argument matches whole-view +0xb38.

When enabling control, it requires the stored skill-bar pointer at
whole-view +0xb48 to be absent, obtains character primary virtual
**+0xc8**, and stores the result there. That getter is **`0x1411d86d0`**,
returning whole-character **+0x520**, the skill bar created by server 618.
The callback immediately dereferences the result for virtual +0x1e8,
passing the view's listener at whole-view +0x40. There is no null guard.

The skill-bar primary vtable `0x142164cd8` resolves +0x1e8 to
`0x1411f9000`. Its complete body asserts a nonnull listener and no existing
registration, then inserts into the collection at skill bar +0xf8.
It does not immediately invoke the new listener. The matching +0x1f0
entry is `0x1411f9130`; the view calls it on disable when its stored
pointer is nonnull, then clears whole-view +0xb48.

Thus **when this ordinary view listener is subscribed, skill-bar creation
must precede the enable-control event reached by equal-ID server 444**.
This refines the previous direct-setter trace, which had not exhausted
its listeners. It does not make 618 an unconditional prerequisite of
server 170: the already traced constructor supports registration before
or after finalization. For the candidate ordinary-view sequence, 618
must be prepared before 444, retaining its profession/coreStats/inventory
requirements. Other control-change listeners remain unexhausted.

After subscription, the callback calls `0x141108e30`, then model virtual
+0x88 and further view-component helpers. The ordinary model entry is
`0x14116afe0`: enabling control sets model +0x20 mask 0x400; disabling
does not clear it in this method. The complete `0x141108e30` body checks
IsControlled and schedules a global-service callback using the view's
agent interface; that scheduled callback and the remaining component
helpers are not exhausted here.

The same character listener's +0x68 entry and AvContext's player-listener
+0xc0 entry both resolve to `0x140163990`, a return-only stub. These
specific no-ops do not describe all listeners on those events.

## Context selection callback

AvContext's character-context listener at whole-context +0x10, vtable
`0x1421424d8`, has virtual +0x40 entry `0x142142518` pointing to
`0x141110750`. Its complete body looks up the context's current finalized
controlled character, then looks up its existing view through
`0x141107840`. A missing view is converted to null; this callback contains
no call to the view factory.

For a player controlled character it obtains coreStats, explicitly
asserts nonnull, subscribes through coreStats virtual +0xd0, and calls
`0x1412d88d0` with coreStats virtual +0xa8's result. It also conditionally
unsubscribes a prior character's coreStats using +0xd8 and context flag
bit 5. The callback updates selected-view pointers and emits further
context notifications. These observations reinforce the coreStats
requirement and distinguish selecting an existing view from creating one;
they do not establish all downstream appearance or notification effects.

## Effect preparation after control enable

The next unconditional helper, `0x14112dc80` on whole-view +0x2b8,
is identified by assertion source **AvCharEffect.cpp** at `0x142146af0`.
Its enable branch first calls `0x141134420`. That helper obtains the
character through the component's parent view, checks IsControlled and
component +0x3e0 bit 4, then sets that bit for first preparation. It
retrieves the skill bar through the same character primary +0xc8 and
immediately dereferences it. Thus this is a second skill-bar consumer in
the control-enable chain.

It visits **23 slots (0–22)** through skill-bar virtual +0xe8. Null slot
results are skipped. Each nonnull record goes to `0x141134530`, which
iterates its +0x40/+0x48 array in 32-byte strides and passes each entry's
first pointer to global-service virtual +0x58. No network skill-availability
mutation occurs in these inspected bodies.

After the slots, it requires constant-registry key **504 / type 64**.
The assertion at `0x142146db8` names **levelUpSkillDef**. Initializer
`0x1402155ee` writes key 504, type 64 and the four GUID words
`1f04abf0 440556f6 ab85d1b1 e11c3bb9` into the registry entry at
`0x1425ef740`. A hash-checked base-content census found exactly one
matching type-150 indirection record: file 1282848, offset 0x2cc60.
Its declared type is 64; following its +0x38 reference reaches type-64
data key **1080** in file 1282842 at offset **0x434a88**. The target has
one effect-list entry pointing to type 183, file 1282860, offset 0x86628.
This establishes the recorded reference and content presence; runtime
indirection selection and the global effect loader are not exhausted.

The outer helper then reads the current map definition without a null
guard and compares its +0x28 against five table entries. Matching
GUID-selected records are prepared only when lookup returns nonnull.
The executable's initial comparison values are 226, 212, 212, 212, 212;
this read does not prove that no other initialization can change them.
Finally it lazily obtains three global resource handles. These are
client content/service dependencies, not newly identified component
creation packets. Disabling control tail-calls `0x141137210`, which is
outside this enable-path follow-up.

The scheduled callback `0x141106bb0` from `0x141108e30` obtains the view's
agent interface, asserts nonnull and forwards the supplied output through
agent virtual +0x1d0. The callback body has no character-component lookup.

The optional whole-view +0x728 helper `0x14113c160` visits 12 internal
entries, toggles their flag bit 0, and calls virtual +0x6f0 only for
nonnull per-entry objects. The tail helper on whole-view +0x9d0,
`0x14114e760`, registers with a static service on enable and calls
`0x14114de30`. That complete refresh body has early guards for its
view-derived model and local +0x68 resource, and conditional lookups of
another character/view. Its nested virtual methods and global service
callbacks remain open; no overall control-readiness claim follows from
these bounded bodies.

Follow-up bundle: `~/re/gw2/notes/209026-control-effect-dependencies-1.json`
(1,630 records; 31,153,581 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_d67bae50ae13526e603cf8464d0393359232948282c18da937a70f555c77e5d2` | Complete effect enable, optional-entry update, tail wrapper and scheduled callback. |
| `ev_7d970e4ebfedfae21fe9a6ffd6b048132e6a8c4c2e0d3f08dffec41828ea9611` | Complete skill-slot preparation and refresh opening. |
| `ev_b11d4e5c293e8c2f4eaacab39cf67bb79cf9f38e2d5a14e32966a5e7ef224b7b` | AvCharEffect.cpp assertion source. |
| `ev_8734a9b59be2970c799685c5b2523dc1b4d565f74fcca9351e11ae67461f1cf8` | Refresh remainder and complete effect-list preparation. |
| `ev_795c4a70c97859153cf6e0ec5aed0388129ebd17e69fc865a27483d2359f01fa` | levelUpSkillDef assertion literal. |
| `ev_b1cf46614df9b2124aaad1816f90dfa50f70da365d851df6bfbca04ece6ae786` | Constant 504 registry initializer. |
| `ev_0c914cc9d917dc8ae5d7c26c4e0795c1e194d613c97a4831796b75fe38cf06e7` | Hash-checked GUID census and referenced type-64/effect records. |
| `ev_edd8b329fb4b42afa28bd2b3b3246186577d360afcf364c02263575f7bac190d` | Initial map-comparison Dwords (low halves of displayed Qwords). |

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-control-view-1.json`
(1,620 records; 31,050,729 bytes), executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| REA evidence | Observation |
| --- | --- |
| `ev_5bb02d085c9dd1b303e4aa496ad0a0a4397f8609388b09f184bd5e8109b8072f` | Character/context/player listener entries. |
| `ev_72eb248c636170f501f77e1b42f33129ac71a7c80f293ffbd19e8d953f9b3dba` | Complete control-change and context-selection callbacks; return-only stub. |
| `ev_579fc9f96830999326aefd19d0733aa1f2d713f6120f6454afeef0351614f55c` | Character skill-bar getter and model control-event entries. |
| `ev_9395e9acdf40500cfac079b1ba699362815495e746d79f92340129837a4770cd` | Getter returns character +0x520; model flag update and controlled-view helper. |
| `ev_377b27c277c67c570d02ea596de4e78c76469df4157a30296a0bb8d6cd50ab71` | Skill-bar subscribe/unsubscribe entries. |
| `ev_543b7ee056479227778da96c380d7d6214f1d13d5af12c38f7226a68e5d0e627` | View constructor's stored skill-bar field initialization site. |
| `ev_025c0ac34613692841d91c1275eb65e7e7f3ef22a3ce52d14d51fe0d9c1d2539` | Complete skill-bar listener insertion, with no immediate replay. |
