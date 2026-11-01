# Build 209026: collision scene entry and server 49

Static REA findings on the frozen executable identified in
[client-build.md](../client-build.md). No client launch or replay occurred.

The collision flag bit 1 required by [server 87](209026-agent-position-state.md)
and the destination methods of [server 64](209026-agent-point-adjustment.md)
is **COLLIDER_FLAG_ENTERED_SCENE**. This name comes directly from the
assertions at `0x14212cfe0` and `0x14212d010`, whose tests read that bit.
The shared collision base constructor `0x1410c0850`, called with the
secondary interface at whole collision +0x10, clears its +0x48 flag word
(whole collision +0x58).

## Scene entry methods

Both collision vtables expose scene entry at **virtual +0x108**:

| Class | Vtable entry | Method |
| --- | --- | --- |
| Full | `0x14212ca10` | `0x1410a2a00` |
| Simple | `0x14212d2d0` | `0x1410a7a60` |

Both methods assert that ENTERED_SCENE is clear. The full method calls
`0x1410b9730` for its primary +0x100 body and optional +0x108 body. The
simple method asserts a nonnull body at +0x160 and calls the same helper
for it. Both then call `0x1410c1710` with **this +0x10**. That shared
method sets secondary +0x48 mask 2, which is whole collision +0x58 bit 1,
and performs optional scene bookkeeping through secondary +0x50/+0x58.
The full path then notifies secondary virtual +0x48; the simple path
calls `0x1410a5a30` on the secondary interface.

The paired exit methods are full `0x1410a2a70` and simple `0x1410a7ae0`
at virtual +0x110. They assert ENTERED_SCENE is set. Their shared helper
`0x1410c1760` clears bit 1 and sets bit 2 on its immediate branch; when
scene +0x210 bit 0 is set it instead marks bit 5 and schedules work.
The rest of deferred removal is not traced here. A repeated scene-entry
request must not be treated as idempotent: the client explicitly asserts
that entry has not happened already.

## Server 49 drives the entry wrapper

Server **49** (decimal transport ID, distinct from behavior opcode 0x49)
has descriptor `0x1425b9540`, handler `0x141033d00`, and these fields:

| Field | Encoding | Decoded offset |
| --- | --- | --- |
| Agent index | Word, fixed u16 LE | +2 |
| Remote tick | Dword, varint u32 | +4 |
| Command flags | Byte | +8 |

The handler allocates 0x70 bytes, constructs through `0x141039000`, and
submits using `0x14102fcf0` with the packet index and tick. Construction
calls the common command initializer with **kind 0x33** and flags
**incoming byte OR 0x12000**, then installs vtable `0x14211cd80`.
Its +0x10 executor is `0x14103ace0`. The executor obtains the bound agent
from command +0x20, asserts nonnull, and directly reaches `0x14102b800`
with that bound interface and the command.

`0x14102b800` temporarily sets bound-agent +0x30 bit 6, calls
`0x1410331c0`, completes the command with `0x14103ea90`, then clears bit 6.
The wrapper `0x1410331c0` asserts bound-agent +0x30 mask 0x20 is clear
(the previously identified agent REVEALED bit). It obtains the collision
object at bound +0x38 (whole agent +0x50) and tail-jumps to virtual +0x108.
That reaches the entry methods above and enables the named collision flag.

This command still goes through normal binding/admission/queue dispatch.
Its fixed flags do not include server 37's missing-agent allowance or its
forced immediate-dispatch bit; the incoming byte and world state remain
relevant. No default flag byte or timestamp is inferred.

## Bootstrap scope and search limits

The server-37 executor directly constructs the base agent; the inspected
constructor tail invokes the collision +0x20 method and game component
+0x10 callback. It does not itself directly call the scene-entry wrapper.
Other traced wrapper callers include `0x14102b580`, which conditionally
enters based on command +0x80 bit 1, and two other construction/command
paths. Their complete packet/lifecycle relationships remain open. Thus
server 49 is a verified entry operation, **not proof that every initial
bootstrap must send it separately**. Game/component callbacks and other
entry paths must be accounted for before choosing an initial sequence.

The direct virtual-call search initially missed tail jumps. Adding FF /4
candidates located the real `0x1410331c0` wrapper. Other +0x108 calls
inspected nearby were on different interfaces; a matching slot alone is
not a call-graph edge. The broad byte-candidate search is not an exhaustive
indirect-call analysis. Only the complete functions and validated edges
above support the conclusion.

## Typed framing scope

A build-specific server-49 transport frame can safely expose just explicit
agent index, remote tick and command flags, retaining the shared fixed Word,
varint Dword and Byte encoding. Decoding should consume one frame
transactionally and encoding should append it. Such an API must not imply
that the agent exists, that scene entry has not already happened, or that
an incoming flag/tick choice is valid. No runtime sender is introduced.

The `AgentSceneEntry209026` codec now implements that scope. Two focused
transport tests and the library build pass: literal 6/7/10-byte goldens,
zero/max unsigned values, prefix and concatenated frame boundaries, every
truncated representative frame, wrong ID and overflowing tick rejection.
Errors leave the input slice unchanged. No frame was transmitted.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-scene-entry-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_4350a95431501a0109ea9a882b230bba610c0f856a5b667c3d5aa9e600fa6e81` | Shared constructor zeroes the secondary flag word. |
| `ev_f08dfe5e1fe62c660cd287ca57a0fb8d75abf8601134a5a7480809435a38599b` | Bounded OR/BTS candidate search identifies shared bit setter. |
| `ev_d9ceb274cb990edc976ea6711f9c6de36c06311fa1d56b575228102632fc9100` | Shared entry flag write and immediate/deferred exit branches. |
| `ev_cb9c8fb91e9350ebe517372e923b96d12682c3b38090b0d038dded49d7dbc408` | Complete simple entry/exit and full exit methods. |
| `ev_fa0d647a9a9beadadd3d1eddb95791df106ab6cc0b06015b89b627035cf5c5de` | ENTERED_SCENE assertion strings. |
| `ev_c380357bc8eb2294f7e9e573e6c6d0baa4284a714ab6f3c4896e9e24ee92d8ec` | Entry and exit vtable entries. |
| `ev_02c2c05eb352a667d618ee6a03910daf74b59b9985b6ac5b0927efb156b95d9e` | Complete full entry method; nearby same-slot call belongs to another interface. |
| `ev_44200b07e27fe1ca263d12d1e62c9a2537f9519ed71d35abeb2dfa2168c08751` | Complete bound-agent entry wrapper and tail virtual call. |
| `ev_11636fa67c9838323da142dcdf26ab111b8aabe52426ef78e1cef094e6842604` | Complete bound entry command method and other direct callers. |
| `ev_6b93905080788424706113596d5a8e368253c1bcc8ac8a11782c082cdb4ddc64` | Complete command constructor and executor. |
| `ev_d1208a2e73f12394c6ee302485562a5ff12d0b7db19c9f827084ef2721a9f9c4` | Allocation, constructor call and index/tick submission. |
| `ev_53e3b5ee6ee52263c91847f1261d47cc479e01bbafcdf5a300e821eb2de2e116` | Registered server-49 descriptor and three fields. |
| `ev_81eb5c8a7320c4519cdc9f7737c6c13cb6534946e5cad56f04b1f5bbbe3be171` | Complete server-37 executor and conditional other entry call. |
| `ev_f1c4888542ab7578517779adb1a5159c6a64d89c5f2344b3261129cd705a61aa` | Base-agent constructor tail after factory selection. |
