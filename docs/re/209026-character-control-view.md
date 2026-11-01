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
