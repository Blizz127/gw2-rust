# Build 209026: server 585 changes character-view lifetime

Static REA continuation of [view finalization](209026-character-view-finalization.md).
No client launch, packet replay or agreement interaction was performed.
“View suppression” describes an observed effect; an original protocol name
has not been established.

## Wire and handler

The registered server pair at `0x1421707a0` has descriptor `0x1425c55a0`
and handler `0x14125aad0`. The terminated descriptor contains ID **585**,
then a **Dword character index** and **Byte value**. The index uses the
existing unsigned variable-length encoding; the byte is raw.

The complete handler resolves the character through `0x1411b4500`,
asserts nonnull, normalizes the byte to `value != 0`, and calls
`0x1411e13e0`. That setter asserts character finalization, sets or clears
bit 33 (mask 0x200000000) in the character's flags at +0x178, then
dispatches `(character, normalized value)` through `0x1411b6250`.
That dispatcher uses context +0x230 and thunk `0x14026e8bc`, listener
virtual **+0x28**.

The setter also has side effects before its flag store. For a nonzero
value on a nonplayer character, it queries the embedded +0x50 object
with key 2 and may call `0x141023870` with key 2 and value 15000. Zero
calls `0x1411e2c00`, which removes key 2 through `0x141023cc0` and may
perform further cleanup under character flag bit 35. Those helpers'
broader semantics are not established. This is not merely a rendering
boolean and must not be used as an initial-spawn shortcut.

## AvContext listener

The [confirmed AvContext subscription](209026-character-view-finalization.md)
uses vtable `0x1421424d8`. Its +0x28 entry is `0x141110620`:

- **Nonzero:** call `0x14110f380`. If the character's view exists, this
  notifies AvContext's +0x480 listeners, updates selected-view references
  through `0x141110480`, then calls view virtual +0x18 with argument 1.
  For the ordinary view, this resolves to `0x141158aa0`, which removes
  the view from its registry, runs base cleanup and frees the 0xc00-byte
  object when argument bit 0 is set.
- **Zero:** if the view lookup is absent, call ordinary view factory
  `0x14110f150` with third argument 1. This callback does **not** call
  the `0x141111ad0` eligibility predicate used by finalization. The
  setter's finalization requirement and all factory/component dependencies
  still apply. The third argument's semantics remain unnamed.

This operates on the client view's lifetime. It is distinct from the
agent REVEALED flag and collision scene-entry callback traced earlier,
and from the player's state at +0x100. The setter has no equality-return
guard: repeated values still invoke its helpers and dispatch the event.
Only the inspected AvContext callback checks whether a view exists.
No network fixture has been validated against the client.

## Offline transport validation

`CharacterViewSuppression209026` in `crates/gw2-proto/src/character_view.rs`
now appends/decodes explicit 585 frames, preserving the raw byte and
leaving input unchanged on errors. Two focused `view_suppression_frame`
tests pass: independent literal fixtures cover index 0/300/u32::MAX,
values 0/1/255, append and concatenation; malformed cases cover every
truncation, wrong ID, Dword overflow and overlong varints. The
`gw2-proto` library build also passes. This validates offline framing,
not client state or acceptance.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-view-suppression-1.json`
(1,548 records; 30,284,191 bytes). Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| REA evidence | Observation |
| --- | --- |
| `ev_04388b68530b11cbe4a078817198e8318f914a606ed4b96fc4138a8119090406` | Registered descriptor/handler pair. |
| `ev_089af6431b087e04d3443d2a352563ad89b10c00317faf6f483d8de289b560af` | Complete terminated 585 descriptor. |
| `ev_e3638477bb44ad9e17f43818e68177602a76542f50804e43f69331c8de4818be` | Complete handler and byte normalization. |
| `ev_888b804e059957da579157707fe0189b696c3186620ddbbc081c50d3e8f5eff9` | Complete character setter and view-removal helper. |
| `ev_3a84e16577a204501a51dcd6e78f0625754ee1063c7597edfe0f05dc4f9d759c` | +0x28 event thunk. |
| `ev_c76e7120429b49f2f30a6cf964dcb0a802adea18a3251ec42d48a9e2dba9ce50` | Context +0x230 dispatcher and subscription. |
| `ev_6965ea811f6d0e615b9eeffb4b38ab75bf75dcd4765f9ddda56e3c64cfc4096b` | Complete AvContext callback, including ungated zero branch. |
| `ev_0dab8fa1f226304c58ba56b7e451b76e46e0bb23cb997295a9c588af13ad42d6` | Ordinary view deletion and interface-getter entries. |
| `ev_885a0cbf8ec3b1035f077bf3ce657ae7ce6b2507fa0bb3b2e0ba4fe797747c2c` | View-reference updates, getter, destructor opening and zero-path cleanup. |
| `ev_74bf0360f2964f212a9d0956fc4cc4f15c19ddd60e2a79c5a75baac239969ecf` | Destructor completion, registry removal and conditional object free. |
