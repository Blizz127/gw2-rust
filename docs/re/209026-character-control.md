# Build 209026: controlled-character assignment

Static REA analysis, 2026-11-01, on frozen build 209026 (identity in
[mission transfer](209026-mission-transfer.md)). No client session or
population-message replay was performed. The User Agreement hold remains.

## Registration and wire layout

`0x14020a2b0` registers protocol 0, group 0 through `0x140ff3d30`, with
219 client definitions at `0x14216e260` and 624 server definition/handler
pairs at `0x14216e940`. This is a separate group from the mission-management
messages already documented in groups 39 and 40.

Server **444** is registered at pair `0x14216fef0`, with descriptor
`0x1425c8ef0` and handler `0x141255710`. Its fields are three **Dwords**,
using the existing variable-length unsigned encoding. The descriptor ends
after the third field. Decoded offsets +2, +6 and +0xa are not wire offsets.

Call the fields A, B and C in the raw codec. B is the player index in
both branches (A equals B in the first); C is the character index. A's
meaning beyond selecting the branch remains unresolved:

- When A equals B, the handler obtains a player entry using A and
  character-context virtual +0x118. It asserts that the entry exists,
  then passes C to `0x1411bc8e0`.
- When A differs from B, it obtains the player entry using B, asserts
  that it exists, and passes C to `0x1411bc9e0` instead. This branch does
  not call the controlled-character setter in the inspected function.

Character-context construction at `0x1411b1de0` installs its primary
vtable `0x14215dfc8`. Slot +0x118 is `0x1411b77f0`: it returns the indexed
entry from context +0x80 if the index is below the count at +0x8c, else null.
The message handler's source diagnostic is `ChCliMsg.cpp`; the direct
control helper checks an assertion named `m_charControlled`.

## Existing character and finalization

`0x1411bc8e0` resolves C using `0x1411b4500`: an indexed lookup in the
character pointer array at context +0x60, bounded by count +0x6c. It stores
the result at player +0x18, requires a nonnull result, then calls
`0x1411b8550` to assign context +0x98. See the setter and notification
details in [map readiness](209026-map-ready.md).

The primary vtable's +0x68 getter is `0x1411b4f10`. It returns context
+0x98 only when nonnull **and** character +0x178 mask 0x10 is set;
otherwise it returns null. That is the same flag named IsFinalized in the
setter's assertion. Therefore AGENT_STREAM cannot advance merely because
a raw controlled-character pointer was installed.

This identifies an assignment path, not a complete bootstrap. The player
and character entries must already exist, finalization must have happened,
and later streaming checks must still pass. Equality of A and B is an observed
branch condition, not proof of the fields' account/ownership semantics.

Follow-up [population analysis](209026-character-population.md) identifies
server 428's player insertion, the type-0 character factory and server 170's
finalization call. Further [component analysis](209026-character-finalization-dependencies.md)
establishes partial startup ordering; a playable bootstrap remains open.

## Control branches and notifications

Follow-up static trace on the same frozen executable, 2026-11-01:

For **A == B**, `0x1411bc8e0` first stores the resolved character in
player +0x18, then calls context setter `0x1411b8550`. That setter returns
if its old/new pointers match. Otherwise it stores context +0x98, asserts
that a nonnull new character is finalized, and calls `0x1411e0a40` to
enable control. It dispatches context +0x230 listeners through virtual
+0x40, then disables control on the old character, if any.

`0x1411e0a40` uses primary-character virtual +0x350 (`0x1411d9210`),
which reads flags +0x178 bit 3. Assertion strings `!IsControlled()` at
`0x142161c50` and `IsControlled()` at `0x142161c18` identify that bit.
The helper sets/clears mask **0x8**, stores 1/0 at character +0x4f0,
stores its player-index override at +0x220, and calls `0x14127c130` on
embedded character +0x400. That last helper only clears its +0x38 low
three bits when disabling. It then dispatches character +0x1e0 listeners
through virtual +0x60 followed by +0x68. These listener bodies are not
exhaustively traced here.

The context setter supplies a nonzero player-index override only when
context +0xa0 already has a player and the new character differs from
that player's owned character (+0x20); otherwise it supplies zero.
After it returns, `0x1411bc8e0` installs the player at context +0xa0
through `0x1411b8620` **only if the prior player +0x18 was null**.
That installation registers three embedded context listeners:

| Player getter slot | Embedded player object | Constructor / vtable | Subscribe method |
| --- | --- | --- | --- |
| +0x18, `0x1411bb240` | +0x9760 | `0x141231660` / `0x14216c780` | +0x30, `0x140320a90` |
| +0x1b0, `0x1411bace0` | +0x218 | `0x1412158c0` / `0x14216a2d8` | +0x188, `0x141217b70` |
| +0x3e8, `0x1411bb460` | +0xa090 | `0x141209da0` / `0x1421693a0` | +0x90, `0x14120afe0` |

Player constructor `0x1411b9cc0` constructs all three objects. The
subscription bodies insert list entries without an immediate listener
callback; the latter two assert on duplicate subscriptions. The context
setter then stores +0xa0 and dispatches its +0x230 listeners through
virtual +0x48. This does not establish the dependencies of those listeners.

If the newly controlled character equals player +0x20, the equal-ID helper
also obtains its coreStats **without a null guard**, reads virtual +0x48
(`0x141207d70`, byte +0x35, the previously mapped sex field), and compares
it with 1. If that boolean differs from `0x1410dcb30`, it calls
`0x1410dcc30` and `0x141070dc0`. Their broader effect is not established
here. Thus owned-character control has a direct coreStats dependency,
already included in the known fresh-player finalization prerequisites.

For **A != B**, `0x1411bc9e0` updates player +0x18 and allows the raw
character lookup to return null. If changed, it calls `0x1411e0b30` on a
nonnull new character with `(1, player +0x74)`, then on a nonnull old
character with `(0, 0)`. That helper stores character +0x220, calls the
same embedded +0x400 helper, and dispatches character listeners through
virtual +0x68. It does **not** toggle IsControlled, install context +0x98
or +0xa0, or directly check finalization. The exact protocol convention
for an absent character index remains unproven.

Both player helpers finish a changed assignment by dispatching player
+0x90 listeners through virtual +0xc0 (`0x14026dfa8`). The comparison,
pointer stores and notification order are observed; the full semantics
of A and all runtime listener dependencies remain open.

This also names the conditional requirement in
[specialization progress](209026-specialization-messages.md): its owned
character +0x350 test is **IsControlled()**. A changed nonnull specialization
on a controlled owned character requires the reward-track manager; moving
messages earlier does not prove that the resulting game state is valid.

## Offline validation

The hash-guarded private extraction script found exactly one server 444
entry in the registered table and emitted only that message into
`data/protocol/209026/character-control.xml` (ignored, mode 0600).
The 368-byte schema SHA-256 is
`7334d2840cadb6b8f899ec1b48a052f7533b931f06c17dd5dc0f8c019918f68f`.

The existing `gw2-capture-census` binary parsed a hand-authored concatenation
of fields `[1,1,300]` and `[1,128,4294967295]` completely: two messages,
6 and 10 bytes, total 16 bytes. This checks frame boundaries and descriptor
translation only; the fixture was not sent and is not valid game state.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-control-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_eb72020aa46971eb7888d8ae91b59db811a399b92a6b3a31557947068dca09a3` | Handler and registered descriptor/handler pair. |
| `ev_2b398b5c0b3ab066b35deaac1d6371d5e9ef877a8cb9e999e15502fc6ffaa18a` | Three Dword fields, both branches, character lookup. |
| `ev_20d26b6b42744aea44a50b4973ffa39c43129de481c67fc20714b3b3eff3ae40` | Descriptor terminator. |
| `ev_2e7e934e4f47e60736e5ede9b7aad0b8852a97ec66cb836c9769e0956548ceff` | Protocol/group registration and table counts. |
| `ev_0e790445e3d311ef0d39dadf616046710afb917fadab702db99da19ca22d7838` | Registration helper and table shapes. |
| `ev_a3b46744bcb2347fb0349efcd0e96a4e1760d6308b882ed07c1ef0e66ff69273` | Context vtable entries and message/control diagnostics. |
| `ev_75560894b9bff9807ee51cb43fbd18ef5abb18c6302146ced46b24582bcabdd7` | Constructor establishes primary vtable address. |
| `ev_0ea1055a18429a627e5b28ace55d69606b956a2bec4d64e6b9be885b275c9c57` | Player-index lookup. |
| `ev_a18545b5a703c3875fdfb2aeee6d74f580b442f231b59a6d10aea400e6ddf79d` | Getter explicitly requires finalization. |
| `ev_ad42bb2bba9322dfe084cd047326f784fa3c1a78b15f5f45d1d8d3f574f69c7f` | Hash-guarded schema extraction. |

Follow-up private bundle:
`~/re/gw2/notes/209026-control-assignment-branches-1.json`
(1,145 cumulative records). No client execution or replay was performed.

| REA evidence | Observation |
| --- | --- |
| `ev_15d94275cabc32400f66aaef2a6f1c04a14a7733cc3bd452e8834092c13ac212` | Handler branches and nonlocal association helper. |
| `ev_55df56c7a9cb8c42c387fdf4ca000d35748a9e7471a61975847320e87177ac4f` | Context controlled setter and embedded movement helper. |
| `ev_cf28bffc1bef9e00ec5beb20fd44cf00013c0a07db1e9d5bda03bd1b9d72d55b` | Complete control, player-context and assignment helpers. |
| `ev_eec1fa7d2ca1b3da3514c313e927c4b5f65d6f51b9166002090a356a8ad15e6b` | IsControlled assertions, getter slots and event trampolines. |
| `ev_421ce28b26186b5a1c97747a77ae9321a0e7246700a607550c8e815ce0767132` | Embedded-object getters, sex getter and local-player installation. |
| `ev_945cc87703d679e567b4be50c395e814e9eeb7ca23b8797ecb287bfc2a3c2ce9` | Both complete player assignment helpers, including coreStats dereference. |
| `ev_9488dfbd28c11388eab1ca482ca4827ec7198d715bf8fec73dbb6ae42413199f` | Player constructs all three subscription objects. |
| `ev_1fcf36fb7837fbb460a7f1c76c65aa276c067f4fa07b5b78a93fb73c14840c50` | Embedded-object vtables and complete character control/association helpers. |
| `ev_978b31e5663ba28c3a476cf2e60fae936974aac3bdeb1bb4d525ea850b0f3874` | Subscription slots and bit-3 getter. |
| `ev_6a000f128699de4d9029834b9ab9e974b3a21eb8bf59e637446364b0f80688fa` | Subscription insertion bodies. |
| `ev_47dca363957757446deb501e460e732cc0cd1b6e4d80397c5eb806719131747c` | Remaining list branches and duplicate assertions. |
| `ev_30ed7fcced231894342acf0cdf002ce8b97d260cb160fd80171a2c354dc4c0b6` | Last two subscription epilogues. |
