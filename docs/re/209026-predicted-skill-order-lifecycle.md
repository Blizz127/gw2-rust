# Build 209026: predicted skill order lifecycle and a corrected gate

Static REA, 2026-11-05, against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This corrects the interpretation of the player gate in the
[NPC talk-loop note](209026-npc-talk-loop.md#order-start-gate-correction-0x14127ef80)
and follows the [skill command trace](209026-mesmer-skill-command-path.md).
No client, live archive or container was accessed, and no gameplay reply was sent.

**There is no demonstrated requirement to acknowledge predicted order `0x53`
with a new server message.** The client constructs a command for it locally,
with a separate behavior container from the subsequent `SkillBegin` request.
The previous claim that every controlled player's skill without flag `0x80000`
rejects a server `0x78` start was incorrect. The tested gate belongs to a
different, still unnamed component flag.

## Controlled state is not component +0x58 bit 0

In the complete request producer `0x141283f90..0x1412842da`:

- `0x141283fcf` checks **Dword component +0x40**, and the failure at
  `0x141283ff3` names `m_isControlled` at `0x14217a050`.
- The earlier test at `0x141283fc5` checks **component +0x58 bit 0**.
  When set, a skill lacking `SkillDef +0x38 & 0x80000` returns before
  constructing the predicted order or sending `SkillBegin`.
- Order receiver `0x14127f19c..0x14127f249` checks that same +0x58 bit.
  Only when it is set does the receiver reject a packed `0x78` for a skill
  lacking `0x80000`.

Component constructor `0x14127da40..0x14127da7e` initializes both fields to
zero. Component vtable `0x142179b90` slot +0x238 points to
`0x141286ef0..0x141286f08`, which sets/clears only +0x58 bit 0 from a Boolean
argument. This slice does not establish that setter's caller or a retail name
for the flag. No unconditional association with control is justified.

The retained type-64 record for skill 10176 still has flags `0x14000006`,
without `0x80000`. Its previously captured outgoing `SkillBegin` therefore
does not support the claim that the same request path was suppressed by this
flag. This is a correction of the source interpretation, not a reconstruction
of the runtime value of the flag or the cause of run 126's lack of animation.

## A locally constructed command precedes the skill request

When the selected action resolves, the producer allocates predicted order
`0x53` through `0x14127de10..0x14127df32`. The constructor installs vtable
`0x142179a28`, stores the skill at +0xa0 and slot at +0xa8, initializes its
action object at +0x40, and calls `0x141023f10` for its own behavior container
at +0x20. The producer queues this object at component +0x38 and calls
`0x14127ecf0..0x14127ef7c` to attempt admission to active slot 0 or 1.

The predicted order's virtual +0x18, `0x141286750..0x141286816`, produces
exactly three packed bytes:

`[0x53, order_flags & 0x3f, 0x02]`

It passes that existing container to base-agent virtual +0x98,
`0x141025630..0x14102570e`, with **literal duration 1000** and low command flags
derived from the order's slot bit plus `0x06`. That calls constructor
`0x141035b10..0x141035ba0`: command kind 3, fixed flags `0x8801000`, duration
at +0x70, and a copy of the supplied behavior container. **1000 is the locally
constructed command's duration; it is not evidence of 10176's retail cast time,
healing interval or recharge.**

The kind-3 serializer `0x14103b4f0..0x14103b6de` chooses message **client 7**
for context type 1, otherwise server 35. This binds a serialization route;
this static slice does not assert that a particular historical packet used it.

After attempting the predicted order, the request producer separately calls
`0x141023f10` again and creates the ten-byte `SkillBegin` payload for client 23.
The complete initializer `0x141023f10..0x141023f55` increments the context's
behavior counter and applies the context-type-1 high bit. The two containers
must not be treated as having the same correlation value. Nor is there a
source-supported rule here to recover the predicted value by subtracting one
from a captured request value: other container creation and conditional paths
must remain accounted for.

## Start, cancellation and completion use the common order lifecycle

The following character-bound slots are proven by table `0x142161670`:

| Bound slot | Thunk | Order-component target |
| --- | --- | --- |
| +0x80 | `0x1411dc060` | `0x14127ebe0..0x14127ece0`, cancellation |
| +0x88 | `0x1411dc070` | `0x14127ef80..0x14127f474`, start |
| +0x90 | `0x1411dc080` | `0x14127fe10..0x14127fefb`, completion |

Common command start `0x141025e80..0x141025fe8` calls the bound start slot when
bit 12 is set, bit 17 is clear and its other guards pass. Common command end
`0x1410261c0..0x141026382` uses the same bit guards and selects cancellation
when command bit 8 is set, otherwise completion.

All three order paths resolve the slot from outer bit 5. The matching helper
`0x1412874a0..0x141287531` first compares the behavior value, then **asserts
that the packed kind equals the existing order kind**. A matching existing
`0x53` can therefore reach its typed lifecycle callbacks. A mismatched
behavior value follows the receiver's replacement/cancellation path instead;
it is not confirmation of the existing order.

| Predicted vtable slot | Target and observed action |
| --- | --- |
| +0x00 | `0x141285350..0x1412853b2`: conditionally changes agent state, then dispatches `0x1411db5c0`. |
| +0x08 | `0x1412858e0..0x141285902`: dispatches `0x1411db590` with the stored skill. |
| +0x10 | `0x141285da0..0x141285e08`: conditionally changes agent state, dispatches `0x1411db5c0`, returns 1. |
| +0x18 | `0x141286750..0x141286816`: constructs the local command described above. |

Completion clears the active flag and, because +0x10 returns 1, releases the
order through `0x14127ff00..0x14127ffa2`. This is a typed lifecycle, not a
demonstration of a heal amount or server acknowledgement sequence.

**Do not send standalone packed `0x53` to create an order.** The factory table
at `0x1412816fc` maps `0x53` to the fatal unsupported-kind branch
`0x1412816e2..0x1412816fa`; the local constructor is separate. The generic
receiver can reuse an existing matching order, but that conditional fact does
not establish an appropriate server response. Server-84 `0x44/0x45/0x47`
remain explicitly restricted to an existing `0x78` and cannot be relabeled as
predicted-order confirmations.

## Exact 10176 action-selector data reached in this slice

The existing cached container 1282860 was hash-checked to
`05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23`.
Skill 10176 is at offset 4074696. Its +0x58 tag is zero and relocated +0x60
points within that container to body offset 4074944. The complete selector
`0x14129a4d0..0x14129a662` passes these opaque Qword keys to action-context
virtual +0x30, reached through `0x1411757e0`:

| Body offset | Exact stored value | Selector use |
| --- | --- | --- |
| +0x00 | `0x0000b309585954f7` | Primary action key. |
| +0x08 | `0x000000215a908ef7` | Alternate action key when the caller's selector is nonzero. |
| +0x10 | 0 | Second lookup argument for the alternate branch. |
| +0x18 | 0 | Second lookup argument for the primary branch. |
| +0x70 | 0 | Flags tested by the predicted-order lifecycle at masks `0x400` and `0x8000`. |

These keys are unrelocated scalar values, not proven file IDs or addresses.
No action table, action timing or healing formula was recovered here. The next
bounded content edge is the demonstrated action-context lookup for these exact
two keys. The context selector, complete successful server order sequence,
heal amount and recharge category/Word/duration remain unresolved.

## Evidence and scope

Private complete Evidence bundle:
`~/re/gw2/notes/209026-predicted-skill-order-lifecycle-20261105.json`, SHA-256
`6814e6b247b38dd4b95ce45685d4c09bdf97bae18d93f046d21ecf1917322253`.
It contains 22 captures. Some aggregate truncation flags concern duplicate
rendered terminal frames; the raw captures used above are retained. Disassembly
windows sometimes include adjacent code, which is not claimed as a typed edge.
PE exception entries split several logical bodies; the ranges above follow
their actual branch continuations and returns rather than treating a single
exception entry as the whole function.

| Evidence | Relevant finding |
| --- | --- |
| `ev_f2ce7d01798371752316661550bff761c8dd1de0ec4e1e1d7fa78540a1a49ad0`, `ev_f73287465434986acb2dd086cb5845eab252513633615fc72b17c4af28ff15dc` | Complete request producer and literal `m_isControlled` assertion. |
| `ev_edaee31c09fabd887d20b4580ebdcc0a73c0e745905a61b2068d2c6279775d8b`, `ev_abaa35632c6f1b3dce9386af112216273be837239c634ec7930566f01c1b0cce` | Component initialization, local queue admission and independent +0x58 flag setter. |
| `ev_366905933b9e64031a2d7fca86c0a3f01e11755aa2d30553a385aab0102be22f`, `ev_7edbb3dfd8be90e008edbe1a770145c13c7710e54e5b39c791d0b01095668416`, `ev_2cad593e1db7dad5b26c364399579331cb89a6eb7800349ce2b0a4c6e8c0a0a9` | Predicted constructor and exact vtable functions. |
| `ev_6c3141ed929f205e324fa150de423ade000c755cbcc8105d9e652c27b9b2522d`, `ev_54000555b438c2da6bade948530ff48c153e19c7fb5d9fff23e63669f2e76dac`, `ev_ed8ad35bf0936e2e7a450188746087147666ac873c5e02414680c8af43d861c5` | Complete order start/comparison, common command end and bound slots. |
| `ev_41bf707cfbbb808d36044fa18058f38d81e2f6f0833d641d9b65aedb580160e6`, `ev_f3e7a8251a4eec16469eaf47ef3730b7bd8dd418fe16f6ddd4412ec79f9efc92`, `ev_161ce8110455e086d7c2117454be7cde7f46c7841b2ba6ff4db1b9a5de919574`, `ev_911da4b88330727ccafa5e7efee950843067388e5ab6276475f8120fbbf7c241` | Local command construction, separate behavior IDs, serializer and unsupported standalone `0x53`. |
| `ev_28401bedf5e7368b1cf1356088c0bee159f3a10f89a666ae376d6dc18b2ba824`, `ev_add2c3d87a2503c108be20574275975109857cf11148b97cdb9d8add07dfdea7`, `ev_2afaee8ad9a24e8745e5d961a108edf99ad95a574fc9d816c2b2da4af34d761f` | Exact 10176 body, opaque keys and complete action selector. |
