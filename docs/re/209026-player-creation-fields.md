# Build 209026: player name, portal identity and initial state

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This extends [player population](209026-character-population.md).
No client was launched and no player records were transmitted.

## Server 428 field storage

Handler `0x141254ec0` forwards decoded fields through `0x1411b7280` into
player constructor `0x1411b9cc0`. The complete constructor is
`0x1411b9cc0..0x1411ba112`; its object allocation is 0xa180 bytes.

| Field | Constructor use |
| --- | --- |
| Dword 0 | Player index, stored at player +0x74. |
| String(32) 1 | UTF-16 name copied into the owned string at +0x60; pointer at +0x68. |
| Guid 2 | Copied to +0x78; named `m_portalUserId` by a downstream assertion. |
| Dword 3 | Low bit selects initial player state and input-binding activation, below. |

The string copy calls UTF-16 length helper `0x1409b90f0`, adds the
terminator, then calls `0x1402e45c0`, which allocates/copies twice that
unit count. Player virtual +0xd8 resolves to `0x140257e70`, returning the
stored string pointer. Display-name helper `0x1411badc0` can prefer the
owned character's name; its fallback uses +0x68 and explicitly asserts
`TextValidateLiteral(m_name.Ptr())` at `0x14215ef28` before formatting
nonempty text. This establishes a literal player name, not a requirement
to reproduce a real account display name. The established String(32)
wire bound includes its UTF-16 zero terminator.

Player virtual +0xf0 resolves to `0x140680300`, returning +0x78. At
`0x1411bc490`, a comparison against that stored GUID reaches assertion
`ownerUserId != m_portalUserId` (`0x14215efa0`). Thus the GUID's client
name is established more precisely than merely “lookup key.” This note
does not establish equality with any particular STS response field or
authorize using a real account identifier. Offline fixtures still use
synthetic identities consistently within their traced dependencies.

## Initial state is not an arbitrary ignored Dword

At `0x1411ba089` the constructor tests only bit 0 of the final Dword:

```text
if flags & 1:
    player[0x100] = 2
    input_binding_context.virtual_50(4)
else:
    player[0x100] = 3
```

The complete constructor does not use the other bits. Predicate
`0x1411bb660`, player virtual +0x170, returns `state != 3`. The input
context comes from `0x1413ac960`; its source literal identifies
`Game/InputBinding/IbContext.cpp`. Its vtable is `0x142256688`:
virtual +0x50 is `0x1413ab1f0`, which asserts selector absence and inserts
it into a sorted collection; virtual +0x58 is `0x1413ab0e0`, which asserts
presence and removes it. Both recompute bindings around the mutation.
This is a selector set, not a reference counter. Constructor bit 0 set
therefore has an input-binding side effect involving selector 4.

The [direct consumer trace](209026-player-spectating.md) now connects this
state machinery to spectating. Individual numeric state names and selector
4's binding membership remain unresolved. Calling it a cosmetic flag, a
cinematic flag or an unconditional global input lock would exceed this evidence.
Clearing bit 0 specifically avoids the constructor's selector insertion;
it does not independently prove a ready or valid player bootstrap.

## Related messages 459 and 460

The same state has explicit later setters on the context's selected
player (obtained through context virtual +0x70):

- Server **459**, descriptor `0x1425c9f60`, handler `0x141255e60`, has
  three Dwords. Setter `0x1411bc280` resolves field 0 as an optional
  engine-agent reference, resolves field 1 as content type 0x14e, and
  stores field 2 at player +0x100. If the previous predicate was false,
  it first inserts input selector 4. It then notifies player/context
  listeners.
- Server **460**, descriptor `0x1425c9ec0`, handler `0x141255ea0`, is
  empty. Setter `0x1411bc350` restores state 3, clears the associated
  content/reference, notifies listeners and removes selector 4.

These handlers and the player destructor demonstrate why choosing
initial bit 0 blindly can add a dependency on later messages. They do
not establish a need to send 459/460 during ordinary map entry, nor a
reason to send a removal message when the selector was never inserted.
The direct consumer identifies content type 334 as the spectate camera
and the agent reference as the spectate target; both must not resolve
simultaneously. Other state values and full transition conditions remain open.

## Typed frame scope

The server-428 frame codec uses the established Dword / String(32) / Guid /
Dword order and a little-endian message ID. The name is retained as raw
UTF-16 units, without its zero terminator, so framing can be inspected
without silently repairing unpaired surrogates. Encoding rejects embedded
zero units and more than 31 name units before appending any bytes. Decoding
requires the terminator within 32 units and advances the caller's input
only after the complete frame succeeds. All GUID bytes and final-Dword
bits remain explicit. No text-policy, account-identity or bootstrap-state
validation is implied by accepting a frame.

The maximum frame size is 92 bytes: 2 for the ID, 5 for each Dword, 64
for the terminated name and 16 for the GUID. This is an offline framing
helper; adding it does not send a player record to the client.

Implemented as `gw2_proto::player_creation::PlayerCreation209026`.
Three focused tests pass: a hand-authored wire fixture preserves GUID byte
order, supplementary/unpaired UTF-16 units and all flag bits; every fixture
truncation and malformed ID/varint/name bound leaves the input unchanged;
empty/maximum names preserve a following frame. Invalid output names leave
an existing output prefix unchanged. `cargo build -p gw2-proto --lib` also
passes. This is local codec validation, not a client bootstrap replay.

## Evidence

Private bundle: `~/re/gw2/notes/209026-player-creation-fields-1.json`
(930 records, 21,436,980 bytes). Source-only findings; no runtime validation.

| REA evidence | Observation |
| --- | --- |
| `ev_4d94922ea28ebbe445df198a41e39894ee46b597bc44c280b5be64ceac512833` | Handler arguments, object allocation, player index/GUID stores. |
| `ev_66390c98a9c119d8fbf01797cf3d934ee507a35a0e0f457e80c7a20c7a2aa1c1` | Complete constructor tail: name copy, low-bit branch and destructor state check. |
| `ev_09574865c995ac52d4ebdb0bce2263d007005be43693191862b8ffb258da6fb5` | Player vtable, state predicate and input-context accessor. |
| `ev_34f029e6ed6455aea3b7d5ead23300f3a8790bffb235606437c081b1276c33ed` | Getter bindings, state setter and input-context source name. |
| `ev_f67896cfcd26262613d6fb5369a628c0dcb4a657d91033b5f5373f979b5f5367` | Display-name fallback, state clear and direct setter callers. |
| `ev_98e4003e616062b7bf90a21bdfcca67feafa81d36819569eb61ad27faad313f8` | Name/portal-user assertions and protocol registration pointers. |
| `ev_f90bbcdc4e693e3636f2e7a40eca32cfd3b511bfb90690458f39ce38ae441a25` | Hash-guarded 459/460 descriptors and input-context vtable installation. |
| `ev_2c52d4ebecbc248b567e3ec50fba2de48fb71b75517dd55ffb949d508e987889` | Input method pointers, handlers and UTF-16 string copy. |
| `ev_c81d4209fe9ee10805885ed7fee7eecfcf0b4e4b816b88cf135de6760d0018e9` | Selector insertion/removal and listener dispatch. |
| `ev_166e28995aa48b8b80687e2b90df987d28ba446119861cf0458fb3932e8a917f` | Selector assertions and binding recomputation. |
