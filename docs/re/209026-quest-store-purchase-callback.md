# Build 209026: quest store Purchase request result callback

Static REA follow-up, 2026-11-01, to the
[item-result continuation](209026-quest-store-item-followup.md). That note
left the Purchase request's result handling untraced. Result: **a
successful Purchase response is discarded by its request-ID callback and
emits no store event**. Only the error path posts internal event 0x50.
The client remained stopped; no request, purchase, account access or
agreement acceptance occurred.

## Request entries

The Purchase request built by `0x140ff8aa0` has primary table
`0x142111680`. Its interface table at `0x142111688` supplies:

| Interface slot | Function | Behavior |
| --- | --- | --- |
| +0x20, success | `0x140ff6c70` | Gets XML through response virtual +0x58. On null XML, builds error 0x3f through `0x140e339d0` and invokes its own +0x28. On nonnull XML with a callback at whole +8, calls callback virtual +0x20 with the response and XML. The last two arguments are zero; no auxiliary array is passed. Then returns. |
| +0x28, error | `0x140ff6a50` | Forwards a nonnull callback to virtual +0x18. |

This mirrors the search request's success entry `0x140ff72f0` documented in
the [callback trace](209026-quest-store-callbacks.md). The success entry
emits nothing beyond the callback.

## Callback table `0x141924898`

Adapter `0x1402451d0` allocates a 16-byte callback. It stores table
`0x141924898` and, at +8, the request ID supplied by store method
`0x14025adc0`. The table has five entries. The next table,
`0x1419248c0`, is the search request's callback and begins at +0x28.

| Slot | Purchase callback | Search callback (`0x1419248c0`) |
| --- | --- | --- |
| +0 | `0x1402443d0`: deleting destructor (frees 0x10 bytes) | Same |
| +8 | `0x140246e90`: invokes own +0 with flag 1 (self-release) | Same |
| +0x10 | `0x140246f20`: posts event **0x50**, payload {request ID, 8-byte error} | Same |
| +0x18 | `0x140247420`: converts the error through `0x140e345d0`, then calls own +0x10 | Same |
| +0x20 | **No-op `0x140163990`** | `0x140247460`: parses `Items`/`Packages` and posts 0x42/0x43 |

A successful Purchase response therefore reaches a no-op. It does not post
item (0x42) or package (0x43) results and notifies no store listener. A
failed Purchase, whether a transport error or null XML (error 0x3f), submits
event 0x50 with the Purchase request ID through `0x141069d20`. The process
gates documented in the callback trace still apply. That event follows the
[dispatch](209026-quest-store-dispatch.md) error branch.

## Effect on the quest continuation

On the guarded path, quest callback `0x141200560`:
- tracks the Purchase ID as a null-valued entry in the static request tracker (`0x1412005c4` → tracker +0x18)
- then unregisters its own store listener

Combined with the [tracker reader sweep](209026-quest-store-tracker-readers.md):

| Purchase outcome | Static consequence |
| --- | --- |
| Success | No event. The null tracker entry stays until process exit. No reader treats it as pending. |
| Error | Event 0x50. If the service's own +0x608 bookkeeping does not claim the ID, ordinary +0x160 delivery reaches the tracker, which removes the entry. The quest listener is already unsubscribed. |

On this static evidence, neither outcome feeds back into quest state through
these callbacks. An offline emulator is therefore not required by this path
to answer the Purchase request for startup to continue.

This does not establish:
- the server's purchase semantics or any entitlement or wallet effect
- the 0x50 error branch's retry path
- whether other content notifications follow a successful purchase through separate server messages

Runtime behavior is unobserved.

## Evidence

Private bundle: `~/re/gw2/notes/209026-quest-store-purchase-callback-1.json`
(17 Rizin records, including the tracker sweep's). Frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`,
unchanged after each call.

The previous lead began this trace before the 2026-11-01 host reboot. Those
four records are retained in
`~/re/gw2/notes/209026-quest-store-item-callback-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_0618ea8d8cc073ae86a58c0ba8824aa1928dd0937e3f3afea91ca118d435d1e8` (item-callback bundle) | Callback table entries and Purchase request +0x20/+0x28 entries. |
| `ev_a72c213660433b03a4edf715d7a25fe52588b60bf8a87862ca9d09b61ebd1898` (item-callback bundle) | Purchase success entry `0x140ff6c70`. |
| `ev_c5174471a1ab3c411271437922bf04cf664401a68fb471da402355fb77f8024c` (item-callback bundle) | Purchase error entry `0x140ff6a50`. |
| `ev_295b08adadc1fce3f47b421fbc5ba8871617726e0ba9de0d8e9bace5efe2a535` | Adjacent Purchase and search callback tables. |
| `ev_4c11f7cebd44f53746354544ea177aed7dd22a4032c885f2af955e862d5a888f` | Callback destructor, release, event-0x50 poster, error converter, search +0x20 prefix, and the adapter that installs `0x141924898` with the request ID. |
