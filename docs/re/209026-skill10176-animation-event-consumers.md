# Build 209026: Ether Feast animation-event consumers

Static REA, 2026-11-06, against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This follows the complete event loop retained in
[typed0x78 completion](209026-typed78-completion-release.md) and the exact
[10176 animation records](209026-mesmer-heal-animation-data.md).
Only the frozen PE and retained evidence were read; no runtime, container,
archive, deployment, or gameplay state changed.

**The concrete installed ordinary-view consumers establish visual-effect
notifications and local view state, not an authoritative healing or success
moment.** In particular, primary event type 1 at raw quantity 1067 selects
visual effects, type 2 at the same quantity clears retained effect handles,
and type 3 can set a view flag. There is no basis here to schedule a server
heal at 1067, the displayed tooltip value 1, the type-0 value 1350, or the
local predicted command's 1000.

## Timed dispatcher and exact character-listener slots

The complete loop `0x141286a70..0x141286cf3` is retained by the preceding
completion investigation. Its nine-byte event accessor returns type at +0
and raw quantity at +1. For ordinary processing, it computes the due clock
from order +0xf8 plus a conversion of:

```text
float(event.quantity) / character.primary_virtual_2b0()
```

Conversion helper `0x1409bedf0..0x1409bee70` is a bounded float-to-integer
conversion using a bias and mantissa mask. The event loop compares the
resulting due clock with `0x141023500`'s local world clock. A future event
stops the current pass and schedules through `0x141023990` on order +0x40.
The nonzero force argument bypasses the future-time comparison; thus even
the local dispatch is not unconditionally tied to the raw event quantity.
The prior [clock trace](209026-agent-command-clock.md) establishes the local
clock unit; this pass does not infer a server clock or policy.

Character primary table `0x142160be0` binds +0x2b0 to the complete
`0x1411d8660..0x1411d86c5`. It normally returns character float +0x250.
It returns 1 when that field equals 1, or when the skill obtained from the
character's embedded +0x4b0 interface has kind-0 body flag +0x70 mask
0x10000 set. No current modifier value or runtime selected skill was sampled.
The retained 10176 body flag word is zero, which does not establish the
character's current +0x250 value.

The exact 14-entry switch table at `0x141286cf4` and complete notification
wrappers establish these primary-record dispatches:

| Primary event (type, quantity) | Notification helper | Character listener slot | Additional argument or direct order effect |
| --- | --- | --- | --- |
| (4, 0) | `0x1411dbb30` | +0x380 | Selector 1 and matched-index Boolean |
| (5, 867) | `0x1411dbb30` | +0x380 | Selector 2 and matched-index Boolean |
| (1, 1067) | `0x1411dbac0` | +0x360 | Pointer from order +0x108 |
| (3, 1067) | `0x1411dbb30` | +0x380 | Selector 0 and matched-index Boolean |
| (2, 1067) | `0x1411dbb00` | +0x378 | No extra event value |
| (0, 1350) | `0x1411dba50` | +0x348 | Sets order +0x14 mask 0x400 |

All wrappers notify the character's collection at +0x1e0. Their callback
arguments are listener, owner character, and skill-definition reference;
type 1 adds the order +0x108 pointer. Its domain is not renamed here.
Types 3–5 add selector `type - 3` and a Boolean. The dispatcher code
`0x1411d1e50..0x1411d200c` and `0x1411d1a70..0x1411d1c4d` confirms those
argument identities, including the fifth Boolean argument for +0x380.
The two-argument form uses `0x1411d1700..0x1411d18ab`.

The Boolean is true when the unsigned current step is at least the
animation-wrapper virtual +0x48 result and the current event index equals
wrapper virtual +0x20's result. That case also clears order flag bit 0.
For the unchanged primary one-segment action, those getter results are 0
and 3; at step 0 the event of type 3 is the matching index. This is a
conditional derivation from the retained record, not observed runtime state.
The loop increments the event index after each notification, so equal-time
events follow their authored order: type 1, then 3, then 2 for the primary.

## Concrete installed ordinary-view consumer

The [view registration trace](209026-character-view-reveal.md) establishes
that AvCharBase registers the listener at whole view +0x28, table
`0x142141558`, with its character. The ordinary derived view retains that
table. Exact entries for the slots above are:

| Listener slot | Bound consumer | Immediate effect |
| --- | --- | --- |
| +0x348 | `0x14110ba80..0x14110bace` | Forwards the skill key to view helper `0x141143fc0`. |
| +0x360 | `0x14110bb50..0x14110bbb6` | Calls `0x141130e40` on whole view +0x2b8 with skill and order +0x108 pointer. |
| +0x378 | Leaf `0x14110bc60` | Tail-calls `0x141130eb0` on whole view +0x2b8. |
| +0x380 | `0x14110bc70..0x14110bcaf` | When the fifth argument is nonzero, sets whole view +0xb78 mask 0x10. |

The nontrivial owner-checked callbacks compare the passed character with
listener +0xb10, or whole view +0xb38. The type-2 leaf does not perform
that comparison itself. The +0x380 consumer does not inspect the selector
or mutate health; the meaning and readers of its view flag remain outside
this slice.

Whole view +0x2b8 is the already identified **AvCharEffect** component.
Complete helper `0x141130e40..0x141130ea4` obtains a context value from
component +0x430's embedded +8 interface, then invokes
`0x141134030..0x14113441d` with selection mask **0xf8**. Its diagnostic
source explicitly names `Game/AgentView/AvCharEffect.cpp`.

The complete latter helper reads 32-byte effect rows at skill +0x40,
count +0x48, and row flag +0x1c. It selects flags intersecting the supplied
mask. A nonnull optional context can supply a replacement row list after a
typed discriminator check; remaining selections use the skill's rows.
Selected rows call `0x141133210`, and rows matching mask 0x180050 retain
returned handles in the component's 24-byte-entry collection at +0x210.
The nested effect-instantiation helper was inspected only through its
initial guards, not exhausted. This is the source-proven visual-effect
selection boundary, not a claim that every possible nested effect or
subscriber has been enumerated.

Type 2 follows `0x141130eb0` directly to the complete
`0x14112bc70..0x14112bce3`. It walks that collection using pointer +0x218
and count +0x224, calls virtual +0x30 on each nonnull handle, releases the
entries through `0x14027abb0`, then zeroes the count. The exact virtual
operation's name is not recovered; the handle calls and list clearing are
established without labeling this a gameplay heal, cancel, or success.

Type 0's complete leaf `0x141143fc0` loads a nested object, supplies value
1 to `0x141123a00`, and ignores the incoming skill-key argument. That
complete helper stores the value at one of two nested Dword fields when
the associated pointer exists. It has no health-meter update in its body;
the fields' semantic names remain unresolved.

## Execution boundary

The selected ordinary-view consumers do not calculate healing, remove
conditions, update a health meter, choose recharge, or establish a retail
success result. Their event delivery is driven by local order timing and
can be forced by the separate completion path. The independently traced
[positive health message](209026-health-gain-delivery.md) applies an
explicit server-supplied delta; no demonstrated edge here supplies that
message's amount or authoritative send time.

This is not a census of all dynamically registered character listeners.
Unexamined subscribers, deeper visual effect operations, the opaque order
+0x108 context, and the view flag's readers remain open. The safe conclusion
is the concrete client visual/local contract above, not a universal claim
that animation events can never participate in other client behavior.
Authoritative effect timing and success policy remain outside the recovered
client contract. No gameplay or default packet sequence was added.

## Evidence

New private complete bundle:
`~/re/gw2/notes/209026-skill10176-animation-event-consumers-20261106.json`,
356239 bytes, SHA-256
`b0274d0d5b3cc2bcaacd766078c1ab14b1bdaff9adec5a2c676424719118bf76`.
All 12 process readers exited zero with complete raw PTY frames. Some
repeated rendered frames were omitted by display budgets. Initial bounded
ranges that stopped mid-function are completed by subsequent captures where
the text claims a complete body; the deeper `0x141133210` prefix remains
explicitly partial. Leaf tail jumps without pdata entries were followed.

| REA evidence | Supported observation |
| --- | --- |
| `ev_0bc6e10b60d854e65d607ac0ffe69532c864d5022afd293025321d6f1cb4206f` | Notification bounds and exact event table. |
| `ev_fece07c3145910a0dc21050e531db85409f3d9c7f7ebe379e52c4af59e3ac641`, `ev_64092b9085960ccd7d661e99d28997e302f953e236df495f65c9538c562a3a15` | Notification wrappers, listener slots and argument dispatch. |
| `ev_bd197f3b67ba4cf0852fd12a0b79a1ac23da6483562abb6e7286e2445e43fe5f`, `ev_e451501f1feb9c391bea51c10d2ed2f72445c5f826f7cbec564aee21d2b0c3c9` | Installed view entries, complete callbacks and dispatcher tails. |
| `ev_3fb27811ed41c105f6169d967c359b5dc30b49a84efb81128de3e69ea8010777`, `ev_91a478f493d28a69c35c4534406859deae57b4091db0533f9325219f0904e516` | Character timing slot, visual-effect bridge and two-argument dispatcher. |
| `ev_b8f937e155539c35c4d35f321c41c0ea53f74e31787aec2b25eb063d0c91a787`, `ev_daf5f94246de8903734cd35d114abef4f4fc3a4f272edef677721c0daf6b83cc` | Timing-scale getter, complete effect-row selection, handle-list clearing and local type-0 stores. |
| `ev_7cd3219456f88181959b6af170f7f5f73b217c8610794c58886e06f77d10b26b`, `ev_dad14437ddbfb0d3dee1d4f61a6e72c86e31740b32c888b884b5e86016b612db` | Named AvCharEffect source, complete numerical conversion and bounded nested helper prefix. |

The complete timed loop is reused from
`~/re/gw2/notes/209026-typed78-completion-release-20261106.json`, SHA-256
`bde90fb73167bdcca6798e9941c554abee36d82ab1ae5026d2f7bcb867676301`,
capture `ev_7867b8d41bb333320e60b16f45fae7e7418f876eb55b2aa9bd0626f4096511b7`.
