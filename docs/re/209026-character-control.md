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

Call the fields A, B and C until their full meanings are established:

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
and later streaming checks must still pass. Their creation messages and
required ordering remain unresolved. Equality of A and B is an observed
branch condition, not proof of the fields' account/ownership semantics.

Follow-up [population analysis](209026-character-population.md) identifies
server 428's player insertion, the type-0 character factory and server 170's
finalization call. Initial agent creation and component setup remain open.

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
