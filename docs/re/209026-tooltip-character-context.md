# Build 209026: character-backed tooltip context and fixture inputs

Bounded static REA, 2026-11-06, on frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This follows [literal amount facts](209026-mesmer-tooltip-facts.md). No runtime,
working archive, gameplay or existing source files were changed.

**A concrete skill-description adapter reads the second core-stat level and the
stored effective-attribute word selected by the fact.** For selector 6 the word
is coreStats +0xc8. This connects the evaluator's explicit inputs to known fixture
construction; it does not establish a retail attribute default or prove which
adapter was active in run 144.

## Adapter construction and typed methods

Constructor `0x14088de30..0x14088de9b` installs vtable `0x141b67f38`,
stores its character argument at adapter +0x20, and asserts `m_character`.
The source label is `SkillDescription/SdInt.cpp`. It installs a listener at
+0x18 and subscribes through character virtual +0x558.

| Adapter slot | Function | Demonstrated result |
| --- | --- | --- |
| +0x08 | `0x14026d910..0x14026d914` | Character pointer at adapter +0x20. |
| +0x10 | `0x14088ede0..0x14088ee1a` | Character's coreStats virtual +0x70(selector), or zero if character is null. |
| +0x18 | `0x14088f140..0x14088f175` | Character's coreStats virtual +0x10, or one if character is null. |

Both coreStats accesses use character +8 interface virtual +0xb0.
The installed character interface `0x142161250` binds this to
`0x1411d75f0..0x1411d7635`, which asserts a nonnull component and returns
interface +0x380, equivalent to whole character +0x388.

CoreStats vtable `0x142168da0` resolves the remaining methods:

- +0x70 is `0x141207f90..0x141207fed`. It requires selector != 0 and
  selector < 12, then returns Dword `coreStats +0xb0 + 4*selector`.
  The assertion explicitly names `m_effectiveAttributes`. Selector 6 therefore
  reads +0xc8. This identifies the array, **not the name of attribute 6**.
- +0x10 is `0x140282470..0x140282476`, returning Dword +0xe0 directly.
  This is the second core-stat level from server 182/183. The earned-level
  getters' conditional return of 80 is not used by this adapter method.

The amount formatter's signed conversion of the attribute and unsigned conversion
of the level are unchanged from the preceding note. The null-character fallback
is also source behavior, not evidence that a real player's character was absent.

## How this adapter reaches the formatter

The complete root updater `0x14088cad0..0x14088d051` has a character argument
in R8. If nonnull and different from the character returned by the current
adapter's virtual +8, it allocates 0x28 bytes and calls `0x14088de30` at
`0x14088cb64`. It stores the resulting adapter at root +0x68.

Other branches matter: a separate nonnull R9 argument can construct a different
adapter, and the no-character/no-alternate path constructs the base interface.
This slice does not label every alternate adapter or assert that all tooltip
contexts have the character-backed implementation.

Root description-building code `0x14088b5f0..0x14088b91d` passes root +0x68
to description children. Fact update `0x1408966f0` retains its supplied
context at fact-widget +0x58 and passes that same referenced object to
`0x140891f50` at `0x140896841`. The complete wrapper
`0x140896c60..0x140896d41` independently constructs the same character-backed
adapter and forwards it to that fact update. Its sibling wrappers
`0x140896d50..0x140896e09` and `0x140896e10..0x140896ef8` forward an
already supplied adapter.

These are concrete construction and forwarding paths. The actual hover caller,
adapter pointer and live core-stat values from run 144 were not captured here;
the screenshot's displayed values remain independent observational evidence.

## Initial fixture values and what remains server supplied

CoreStats constructor `0x141207a10..0x141207c40` zero-fills the 0x30-byte
array at +0xb0 through `0x1409b7550` at `0x141207ae0`. That helper sets the
fill value to zero before tail-calling its memory-fill implementation. Thus
constructor state contains zero at selector 6, alongside the other eleven words.

The complete second-level setter `0x141209100..0x1412091d8` stores its explicit
argument at +0xe0, then calls `0x1412083c0` and `0x1412084b0`.
Those complete notification loops update the cached arrays at +0x1a0 and +0x204;
the latter reads +0xb0 but does not recalculate or replace it.

The explicit effective-attribute setter `0x141209080..0x1412090fd` writes
`+0xb0 + 4*selector`. A bounded exact-address direct-call census found:

- `0x14124df0a`, in the already established server-182 extended initialization;
- `0x14124e21d`, in complete handler `0x14124e1a0..0x14124e242`, which
  supplies an unsigned Word value and Byte selector from a received message.

The latter handler also sends a separate signed Word to `0x141208d00`.
No packet number is assigned by this slice. The direct-call census does not
exclude indirect calls or all possible writes elsewhere in the client.

At reviewed repository commit `085aa994b4f7865f5bb1c3581cd74de23eda007e`,
`crates/gw2-proto/src/population.rs` chooses server 183 with `extended: None`
and both level bytes set to explicit fixture constant 1. Consequently
**(level 1, selector-6 value 0) is grounded in the inspected constructor/message
path**, instead of being inferred from health 388 or the visible number 121.
The existing literal evaluator gives primary 121 and per-clone 0 for those inputs.

This is a fixture-construction model. It does not prove those words remain
unchanged after every possible listener or later server message, or establish
retail starter attributes, gear-derived modifiers or server healing execution.
No automatic attribute default has been added to the evaluator.

## Character context selector remains conditional

Character primary vtable `0x142160be0`, +0x228, resolves to
`0x1411d7c30..0x1411d7c4f`. It calls the character's +0x220 method and
returns that result unless it is 4; result 4 invokes `0x14140f1d0`.

The complete +0x220 implementation `0x1411d7b50..0x1411d7c26`, across its
pdata fragment, depends on `0x141411600`, an optional related-character
+0x268 path, and three queries with numeric selectors 23/24/25. Those queried
states can produce 1, 2, 3 or fallback 4; their names and fixture-state values
are not established here.

The fallback `0x14140f1d0..0x14140f22d` uses the definition returned by
`0x141411100` (the map-definition chain is retained in
[character model inputs](209026-character-model-inputs.md)).
It returns 3 for the selected +0x2c cases; otherwise it may inspect the
+0x100 then +0x30 reference chain to choose 1 or 2, with 1 as the absent-data
fallback. This slice does not resolve that chain for map 379 or sample the
preceding character overrides. It therefore does not choose the base 20000
versus context-2/3 recharge overrides 25000 from the amount-fact note.

## Retained evidence

Complete private bundle:
`/home/blizz/re/gw2/notes/209026-tooltip-character-context-20261106.json`,
674,658 bytes, SHA-256
`daa8a39780fe7151ea9230dfcaf38d3191a0f89bf1e23a7aff52a1ebfc51acf5`.

Twenty captures exited zero with complete raw PTY and normalized frames; some
duplicate rendered terminal frames were omitted. Instructions outside verified
function boundaries, including disassembled jump-table bytes, are not used as
source claims. Key records:

| Evidence | Scope |
| --- | --- |
| `ev_ea776ee9337455281042e72a0cfc5a8f1b8df4ad2619e0d86a194acaf1385443` | Complete adapter constructor. |
| `ev_39372800ecf1c6b5487214ed9b08d5ec1780d641bb7c0573608fd2edc2d9fd9c` | Vtable methods and exact source/assertion names. |
| `ev_fa8aff6ebe1d537637280654e93c3f8e69de6b67f54c667086d768fb63bc4c53` | Complete attribute method and exact constructor call sites. |
| `ev_b5d37dd0965e2a65abd6856fd65fd0800919b692071887d181bcd9b8484a8ec6` | Core attribute getter and complete root adapter selection/forwarding. |
| `ev_42db29facd7e8b9c557e7dcb2e40d0bd1113088695f23eaf9fdde9672c71f59c` | Core constructor, level setter and both notification loops. |
| `ev_ba80f9ae6948ab30627ce152379b5c9488eaa9f8aeb1a75c4e5ab2e2cbc787a4` | Exact effective-attribute setter calls and direct second-level getter. |
| `ev_b1d6cc457f80431145cd60c57c14208c1b9c8ee625d9c59d7f5b19ac004d2a5c` | Character core interface, received-attribute update and +0x228. |
| `ev_704e93b4daf6bad6bb4cbdf8d43856eb9a13dc06cf707d98d8d9389a4cf09093` | Complete character selector and global fallback. |
