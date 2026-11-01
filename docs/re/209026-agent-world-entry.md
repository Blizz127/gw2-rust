# Build 209026: server 46 world entry and optional collision entry

Static REA findings on the frozen executable identified in
[client-build.md](../client-build.md). No messages were sent to the client.
This extends [collision scene entry](209026-agent-scene-entry.md) and
separates two lifecycle flags that must not be conflated.

## Wire and command mapping

Server **46**, descriptor `0x1425b9100`, handler `0x141033b20`, has:

| Field | Encoding | Decoded offset | Command use |
| --- | --- | --- | --- |
| Agent index | Word | +2 | Binding/submission |
| Remote tick | Dword | +4 | Submission timing |
| Command flags | Byte | +8 | +0x50, OR 0x12000 |
| Collision flags | Byte | +9 | +0x84 |
| Entry options | Byte | +0xa | +0x80 |
| Position | Point3 | +0xb | +0x70 |
| Pitch | Word | +0x1b | Signed facing conversion |
| Yaw | Word | +0x1d | Signed facing conversion |

The two Byte names above describe observed uses, not recovered protocol
symbol names. The handler allocates 0x98 bytes, calls `0x141038c60` and
submits via `0x14102fcf0`. The constructor uses **kind 0x30**, vtable
`0x14211ccd8`, copies the Point3, zero-extends the two extra bytes, and
converts the signed pitch/yaw words through `0x141039590` to the command
vector at +0x88. As in [creation facing](209026-agent-facing.md), 0x7fff
is a sentinel; this trace does not assign it a useful initial direction.

Command virtual +0x10 is `0x14103ac00`, which requires a bound agent and
tail-calls its virtual +0x258. For the base-agent bound interface, the
entry at `0x14211b138` is **`0x14102b580`**. This is an operation on the
existing agent and collider, not construction of a replacement collider.

## Existing-collider updates

The complete `0x14102b580` method temporarily sets bound-agent +0x30 bit
4, then performs these operations in order:

1. Entry-option bit 2 controls bound +0xd0 bit 7; option bit 3 controls
   bit 8. The method also consults/sets corresponding saved bits at +0xd4
   and calls the game component's +0x110 notification when values change.
   Their gameplay names remain unresolved.
2. OR the collision-flags byte into bound +0xd0 (whole agent +0xe8).
3. If the command facing vector exceeds the small squared-magnitude
   predicate, call collision virtual +0x498 with that vector.
4. Call collision virtual **+0x4a8** with the Point3 and third argument 1,
   reaching the position setters documented for
   [server 86](209026-agent-position-update.md).
5. Call collision +0x4b8 with bound +0x120; when bound +0xf0 bit 3 is set,
   call +0x4d0 with 1. Invoke `0x14102dd30`, then query collision +0xf0
   and conditionally call `0x141032780`. The complete side effects of
   these intermediate helpers are not established here.
6. Prepare an extent vector from bound +0x6c as `[r, r, 2*r]`, using
   `[r, r, r]` when bound +0xd0 bit 2 is set. Entry-option **bit 0** sets
   bound +0xd0 mask 0x20 and replaces this vector with zero.
7. Call the world-entry helper `0x1410330c0` with that vector and mask
   **0xfffff85f OR ((bound[+0xd0] & 1) << 8)**.
8. Unless entry-option **bit 1** is set, call the collision scene-entry
   wrapper `0x1410331c0`.
9. Complete the command through `0x14103ea90` and clear the temporary bit 4.

These observations describe the method's branches; they do not prove
valid extents, helper state or packet choices for a playable character.

## Two separate entry flags

`0x1410330c0` asserts that bound-agent +0x30 mask **0x08** is clear. The
assertion at `0x14211bbc0` explicitly names **AGENT_FLAG_ENTERED_WORLD**.
It calls collision virtual +0x118 only if **all three extent components
compare unequal to zero** (unordered/NaN comparisons also take that
branch), forwarding the mask and extent vector. It then sets the agent
ENTERED_WORLD bit regardless of whether the extent call was skipped,
and notifies an optional context object through virtual +0x20.

The subsequent wrapper `0x1410331c0` asserts the agent REVEALED bit 0x20
is clear and calls collision virtual +0x108. That sets the distinct
**COLLIDER_FLAG_ENTERED_SCENE**, bit 1 of collision +0x58, as traced in
[server 49](209026-agent-scene-entry.md). Therefore:

- Server 46 can perform agent world entry **and** collision scene entry.
- Its option bit 1 skips only that final collision-entry call; world entry
  and preceding updates still occur.
- Server 49 reaches collision scene entry only. Its method does not call
  the world-entry helper and is not a substitute for those preceding updates.
- These entry operations have assertions, not idempotent guards. A 46 that
  takes its collision-entry branch followed by 49, without an intervening
  exit, encounters the repeated collision-entry assertion.

## First-creation callback follow-up

At the end of the base-agent constructor, the full/simple collision +0x20
entries (`0x14212c928`, `0x14212d1e8`) both resolve to **`0x140163990`**,
a plain `ret 0`. The type-0 character game interface instead uses vtable
**`0x142161670`**: its +0x10 entry at `0x142161680` resolves to
`0x1411dc6e0`. The previous use of `0x142161710` as the table base was
incorrect. The [corrected callback trace](209026-character-entry-callbacks.md)
shows an immediate return for identity high nibble 0x3 (including candidate
0x30000001), and a conditional nested float-setting call for other identities.
Only the collision callbacks are unconditional no-ops.

This narrows the candidate bootstrap relationship: creation and the two
entry states are distinct, and 46 is a verified route that supplies both
entry operations. It does not establish a complete packet sequence or
valid initial flags, nor exhaust every nested constructor or context
callback. No additional runtime fixture or inferred defaults are added.

The [body follow-up](209026-entry-collision-bodies.md) confirms constructor
allocation of the entry bodies and identifies the full +0x118 extent
operation as separate visibility-phantom allocation. Its rigid-body
prerequisite is supplied by the full constructor on successful allocation.

## Typed framing scope

An explicit server-46 frame type can carry index, tick, command flags,
collision flags, entry options, Point3 and the two raw facing Words. It
should retain every float bit, raw tag, option and sentinel facing word,
append encoding and decode one frame transactionally. It supplies no
initial option defaults and performs no lifecycle validation or sending.

`AgentWorldEntry209026` now implements this scope. Three focused tests and
the library build pass: literal 27/33-byte goldens and the 25-byte minimum,
all explicit fields, sentinel and signed-facing bit patterns, raw float
bits, unsigned maxima, adjacent frames, every truncated golden and
transactional wrong-ID/overflow rejection. No frame was sent to the client.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-world-entry-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_55730229be82c2f9f1cf10e44300e85ff57f47ac5c9306bd514ce6c4e8e88d72` | Final creation callback vtable entries. |
| `ev_c1b732834c7c633c4c71443bc6e8ed0b189a5d425cd8873fd3658ca537d7ef4b` | Complete 46 executor and plain-return callback target. |
| `ev_fc764314f4c2da8f52431c373981de9809b45281d52b61d78d43d1f10a6c8d2a` | Executor slot in command vtable. |
| `ev_988920cb20c22545bd20f9f46acb54ff51a8b5dd4f1ec5d59166369a340d2fe5` | Bound-agent +0x258 implementation entry. |
| `ev_a2ead9193349613918830d7285f77d0d904d8a87551a0d8b056dd6db0cfa0407` | Complete constructor and first portion of bound operation. |
| `ev_81eb5c8a7320c4519cdc9f7737c6c13cb6534946e5cad56f04b1f5bbbe3be171` | Remainder of bound operation through return. |
| `ev_1ff05732f14a2646906a68dd357689d0395ad25b84ce68a91dcc56a797d7110d` | Registered server-46 descriptor and eight fields. |
| `ev_225f5c55a6020b51dabb451da62cbdadedfb56c87d0eff09bcc1eeee25f36576` | Complete handler and world-entry helper; partial intermediate mode helper only. |
| `ev_4b9882557b0ae82f3bf3fb64405835cfddf45f0c3c96f4589e30de45dd5a57f9` | Named ENTERED_WORLD and REVEALED assertions. |
