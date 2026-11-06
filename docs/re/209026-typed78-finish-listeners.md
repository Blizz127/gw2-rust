# Build209026 typed78 finish notification consumers

Static REA, 2026-11-06, frozen executable SHA256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This follows [typed78 completion/release](209026-typed78-completion-release.md)
without repeating its0x45 timing and release conditions. No runtime, archive,
server state, packet, or production code was changed.

**The recovered finish notification does not establish automatic sword-chain
advancement or another skill request.** Its installed ordinary-view consumer
clears local view/effect state. The constructor-established skillbar interface
has a no-op at the corresponding listener slot; this slice does not prove
when that interface is subscribed. Other dynamic listeners and later update
loops remain outside this bounded conclusion.

## Exact notification and callback arguments

Complete **0x1411db540..0x1411db586** dispatches through the character's
collection at **+0x1e0**, using leaf **0x1411d3fa0..0x1411d3fa8**. The leaf
tail-calls listener **virtual+0x370**. Complete collection dispatcher
**0x1411d0ae0..0x1411d0cd5** establishes the callback arguments:

| Callback argument | Source |
| --- | --- |
| RCX | Current listener from the collection. |
| RDX | Owner character, originally notification RCX. |
| R8 | Original notification RDX; the typed78 call supplies order+0x60. |
| R9 | Skill-definition reference copied from notification R8; the completion call supplies order+0x78 through a reference. |
| Fifth, Dword | Original notification R9D, the stored raw result from order+0xe8. |
| Sixth | Original notification fifth argument, the animation-wrapper pointer. |

The raw result's success/error names are still unknown. Neither the generic
dispatcher nor its +0x370 thunk updates the skillbar's chain pair or constructs
a behavior request in its own body; the concrete listener determines effects.

## Installed ordinary-view listener

The established [view registration](209026-character-view-reveal.md) installs
listener table **0x142141558** at whole view+0x28 and subscribes it to the
character. Its **+0x370 →0x14110bbc0..0x14110bc4f** callback asserts
`character == m_character` against listener+0xb10, obtains the skill key
through the supplied reference, and follows these exact edges:

| Edge | Recovered immediate effect |
| --- | --- |
| `0x1411070c0(whole_view)` | Clears whole-view+0xb70, copies +0xb6c to+0xb74, writes Qword0x17 at+0xb68, unlinks/clears the local node at+0xb08/+0xb10/+0xb18, and clears +0xb78 mask0x10. |
| `0x141143fe0(whole_view+0x730, skill_key)` | Resets a nested field via leaf0x141123aa0, calls0x141120dc0, and clears this component's +0x64 mask8. The complete immediate body does not use the supplied key. |
| Conditional `0x14113dc80([whole_view+0x728], skill_key)` | Applies local predicate/flag checks, may call0x1411402a0, then clears component+0x20 masks1/2. This immediate body also does not use the key. |
| `0x141130eb0(whole_view+0x2b8)` | Clears retained AvCharEffect handles through the already traced0x14112bc70 path. |
| Tail `0x14114ff20(whole_view+0x9d0, original_R8)` | Resolves a view from the supplied object, checks view/component and character predicates, and conditionally stores float8.0 at its component+0x38. |

These component-relative fields must not be confused with same-numbered
character or skillbar fields. The effect-handle operation is established by
[the animation-event consumer note](209026-skill10176-animation-event-consumers.md).
The newly inspected0x1411402a0 walks local 40-byte rows, updates a float and
invokes model virtual operations;0x141120dc0 changes local fields and forwards
values to nested interfaces before tail-calling0x141120cf0. Those deeper
virtual implementations and the latter tail are not exhaustively traced here.
Thus this is a concrete local cleanup path, not proof that every indirect
consequence of every possible listener is presentation-only.

The ordinary-view +0x370 callback does not read its fifth raw-result or sixth
animation-wrapper arguments. No direct skillbar access,628 setter call,
0x23/0x29 producer call, or new-container construction occurs in that complete
callback or the complete immediate helpers described above.

## Skillbar interface and subscription limit

Complete skillbar constructor **0x1411f4720..0x1411f4c6c** installs table
**0x142165158 at skillbar+0x20**. Its exact +0x370 entry is
**0x140163990**, whose complete body is **`ret 0`**. If this interface is
subscribed to the character's collection, that slot performs no chain-state
mutation and sends no request.

The constructor and finalization registration helper
**0x1411f9530..0x1411f9685** establish the combatant, coreStats, inventory,
transformation and global-context subscriptions already documented in
[skillbar lifetime](209026-character-skillbar.md). They do not establish a
subscription of skillbar+0x20 to character+0x1e0 in these bodies. The
constructor's owner virtual+0x4e8 is0x1411d9750, a forwarding thunk to the
embedded combatant's virtual+0xc8, not a direct character-listener registration.
The global-context listener table at skillbar+0x10 has six no-op entries and
two concrete callbacks,0x1411f9220/0x1411f92a0; those bodies do not supply the
missing direct character registration either. Their deeper calls are not a
global subscription census.

This keeps the negative conclusion narrow: no installed **active** skillbar
finish consumer has been demonstrated, and the specific constructor-established
interface's matching slot is a no-op. It is not proof that the entire client
has no other listener, indirect chain writer, or automatic-repeat mechanism.

## Consequence for future server work

The [628 chain contract](209026-mesmer-sword-chain.md) remains separate:
skillbar+0x1c8/+0x1c0 hold the predecessor reference/count used by palette
selection. This finish slice supplies no event or policy for changing them.
It does not justify advancing the sword chain, choosing a result byte, or
sending another cast when0x45 releases an order.

Release helper0x14127ff00 can separately service an already queued local order,
as established in the completion note. That conditional queue operation must
not be relabeled an automatically generated repeat request from +0x370.
The next useful dependency is a concrete non-no-op subscriber or an exact
later update/input caller, not a guessed response sequence. No missing raw
codec or server-handler requirement is proved by this slice.

## Evidence

Private complete bundle:
`/home/blizz/re/gw2/notes/209026-typed78-finish-listeners-20261106.json`,
SHA256 `43cf2a91ec71b09be9c6b23659b6a887cd60c9a428d5992b6ebb5e158b2d9375`.
All16 captures exited0 with complete raw terminal frames. One capture omitted
a repeated rendered frame because of its display budget. Initial fragments,
an unsuccessful pdata leaf lookup and an unaligned continuation start are
retained; later complete bodies support the claims above.

| Evidence | Scope |
| --- | --- |
| `ev_6b92125c2129cabf6f737a5265c66ebef08b92ef05bef0905f03817e267034a3`, `ev_4f2bc7fde128f2523d31b993fd9e34f116d24092e1ec42e839c88cff342f8399` | Notification wrapper and exact+0x370 dispatch leaf. |
| `ev_ff7b696c876a17cc552f94bdb84f93cf0645a39e358f61ded6ecb56f33c5c89a` | Complete argument dispatcher and no-op body; bounded skillbar field-reference results. |
| `ev_7eacda18053d566a347605f25efcd5249f7c5e5332f329ff181695637bf4b17c`, `ev_55939f7c94b105315628478c7710292bb3df9e54b5a9647cab8368bd977597e6` | Installed view entry and complete callback. |
| `ev_2173526c67fe56ac32de0b8485a51e8b3099a980a736a08b295c7fe8f10b4d79`, `ev_c98ddfbee8c763c788a7c4c19b20122df48d59b531e728a13a4c78f19236f588` | Immediate view helpers, nested reset leaf and bounded deeper operations. |
| `ev_2f31a79b3cc3efc8fec80065e1eaf51066643f64d8e5ec04570c6b7e1128cdd0`, `ev_a14c0d5e35221c77f910abd7f8d3847b0d9802f1c6e3143c99fd44b745a94852` | Skillbar construction/registration and exact+0x370 table entry. |
| `ev_b46df7feed90b28b9b8c78e83d373bcc00ab533989a7dbde80334f4496570d2d`, `ev_947bedda7ad824c226e0b237b4faffd1b64463678deefcc723f5512d5b663970`, `ev_cfdf3e085234fd52a6824032da0f3fa71bc2674e6be119222d22eeac00836509` | Owner thunk, context table and complete two context callbacks; view helper tail boundary. |
