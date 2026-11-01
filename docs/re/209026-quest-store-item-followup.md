# Build 209026: quest store item-result continuation

Static REA follow-up, 2026-11-01, to
[store result dispatch](209026-quest-store-dispatch.md). A matching item
result with zero TotalPrice can cause the subscribed quest listener to
build a separate `Purchase` request with quantity 1 and `AsContent=1`,
track that new request ID, and unsubscribe itself. This is a conditional
client code path, not an executed operation or a validated emulator
response. The client remained stopped; no request, purchase, account
access or agreement acceptance occurred.

## Item notification arguments

The context +0xc8 service receives decoded item results through its
whole +0x10 interface, callback `0x1412d0a40`. The inspected prefix stores
the item in its whole +0x48 map, keyed by the GUID at decoded item +2.
It invokes listener virtual +8 using trampoline `0x140255560` and helper
`0x1412cee30`, on the listener list rooted at whole +0x18:

| Callback argument | Decoded item source |
| --- | --- |
| RDX | Pointer to GUID at +2 |
| R8D | Dword at +0x28 |
| R9D | Dword at +0x2c |
| Fifth argument | Byte at +0x30 |

The helper dereferences these scalar pointers before calling the
trampoline. A second notification uses virtual +0x10 and item +0x22/+0x24.
Additional content iteration later in the item callback is not classified
here; this trace does not claim to cover all item-result effects.

Quest constructor `0x1411fd630` installs whole +0x20 interface table
**`0x1421662c8`**. Its +8 entry is **`0x141200560`**, while slots +0,
+0x10 and +0x18 are the no-op `0x140163990`. The startup callback's
conditional service registration uses this +0x20 interface.

## Quest guards and unsubscription

`0x141200560` returns immediately when R8D is nonzero. Otherwise it gets
the existing built-in-398 definition through `0x141466a80` virtual +0,
requires it nonnull, and compares all four GUID Dwords to the notification
GUID. The [startup content trace](209026-startup-subscribers.md) resolves
that definition to frozen type-68 key 156.

Only a match reaches context +0xc8 virtual +0xa8 with that GUID. The
callback passes the result to the static request tracker's virtual +0x18,
then unregisters its +0x20 interface through service virtual +0xe0 and
clears whole quest +0xf0 mask 0x10. It performs this unsubscription even
if the returned ID was zero; the tracker rejects zero separately.
It does not remove the original search request ID on this path.

## Follow-up operation

The service call chain resolves through these primary-table entries:

| Slot / function | Behavior |
| --- | --- |
| +0xa8 / `0x1412d1070` | Supply R8D=1 and tail-call +0xa0. |
| +0xa0 / `0x1412d1080` | Look up cached item by GUID; return zero if absent. |
| +0x98 / `0x1412d11b0` | Check service gate; if enabled, call store service +0x148. |
| Store +0x148 / `0x14025adc0` | Allocate a nonzero request ID and invoke the request adapter. |

The cache lookup copies the stored 49-byte decoded item. Its +0x12 GUID
becomes the follow-up GUID, quantity is 1, its +0x28 Dword is passed as
the next scalar, and the final flag is 1. The notification guard above
requires the same +0x28 Dword to be zero. The zero/cache-miss and disabled
service branches can still prevent a request.

Store method `0x14025adc0` advances its +0x1c8 counter, masks it to
0x1fffffff and skips zero. Adapter `0x14023d670` jumps to
`0x1402451d0`, which allocates a request-ID callback with table
`0x141924898`, distinct from the search callback. Factory/submission
`0x140ff8aa0` builds a 0x50-byte request with primary table `0x142111680`
and interface +0x10 table `0x142111688`; builder **`0x141007580`** supplies
selector **1** to `0x141007650`.

This uses the same selector map already traced for initial `GetLimits`:
table pairs at `0x142114080` map 1 to **`Purchase`** at `0x1421140b8`
and 2 to `GetLimits`. The builder emits literal fields `Guid`, `Quantity`,
`BuildId`, `Language`, `AsContent`, and `TotalPrice`. On this specific
guarded path the proven values are **Quantity=1, AsContent=1,
TotalPrice=0**, with the cached item +0x12 GUID. No currency, server
acceptance or entitlement effect is inferred from those values.

The [Purchase result callback](209026-quest-store-purchase-callback.md)
discards success and posts event 0x50 only on error. Package-result effects,
subsequent content notifications and readiness implications remain untraced. A
future offline fixture must account for this possible continuation;
this finding does not select or implement such a fixture.

## Evidence

Private bundle: `~/re/gw2/notes/209026-quest-store-item-followup-1.json`
(1,936 records), frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All artifact reads used REA; no client state or source behavior changed.
Earlier service constructor and callback-prefix evidence is preserved in
`209026-quest-store-dispatch-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_24ba121a5ceb3186c88fa19ccd740f97282cbbdb516ce66f098e88bb5f6d867f` | Quest interface constructor and item notification sites. |
| `ev_85b2f47b79cd0b281c9783471f3253e81242f1b46d5a6ac1ac029b305bc80ec5` | Quest listener callback/no-op entries. |
| `ev_38b85b66dc7837a4f5472c87876265e6b575faef569793e20e7b26f6de430f94` | Complete quest item callback and unsubscription. |
| `ev_a10af5b99c01fa7aec8388a30857484371a77cdd0f5f75c03cf95e112a02e092` / `ev_89be5974f8a80a1f0f65b363decf5f8f8073916e0802607d1febb5c3b94cde71` | Service-chain table entries. |
| `ev_35aa1e9fa18fe67e6287e4be4f2a798b8d1e60fa49b2a10a9327721fa8e5d22f` | Cached item lookup, scalar forwarding, service gate and listener argument dereferences. |
| `ev_3fc45d3a139f00c0fc2ed6c19824a6b059c3acdbf3d61533b7b3816a7101284c` | Nonzero follow-up request ID and adapter call. |
| `ev_f84141c4d199f04acd26ed69c3f5188e69d734cbdd9cf3ddfad88b9c91c821aa` / `ev_979364926eca747e09b8b7900f49203671aa8e58bfbecae293776c852e33087f` | Adapter and distinct request-ID callback construction. |
| `ev_c5acc2bf7701b99abc4de626a53353af4d4db122290035d06532f9e479c98034` | Request construction, builder and submission call. |
| `ev_4093c502ebb429cb31a4fe59bc7c7137834b68f921eb29be04013a46bc1a8c81` / `ev_cf242b2d786efce19ecd5e58e0b82761a404533758498c1c4938b5f5cb386553` | Selector 1 builder, field arguments and literal names/map pairs. |
