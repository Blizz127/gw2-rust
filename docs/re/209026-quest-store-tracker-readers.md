# Build 209026: quest store request tracker readers

Static REA follow-up, 2026-11-01, to the
[tracker error cleanup](209026-quest-store-tracker.md). Question: does any
client path read the static request tracker's pending entries or count,
for example as a zone-readiness wait? If it did, the quest startup
consumer's null-valued request entry could hold that path until a store
response arrives. The sweep below finds **no reader of pending
state through this object**, apart from its own insert, guard and remove
methods and an exit-time destructor. The client remained stopped; no
request, purchase, account access or agreement acceptance occurred.

## Object layout and direct references

A full Intel disassembly of the frozen executable was searched for every
RIP-relative operand in 0x14255a8c0–0x14255a9ff. The tracker object at
`0x14255a8f0` has only three direct referents:

| Site | Role |
| --- | --- |
| `0x1402eea80` | Getter: `lea rax, [0x14255a8f0]; ret`. |
| `0x1401ab610` | Static initializer. Stores a `0x1409b5010` result at +0x5c and registers `0x1418ef020` as an exit handler. |
| `0x1418ef020` | Exit destructor. Resets the tables, releases each occupied slot's nonnull value through its virtual +0, clears the slot array, then destroys the +0x18 list. |

The destructor fixes the map fields: +0x48 is the slot capacity (reset to
8), +0x4c the occupied count, and +0x50 the 24-byte slot array. This matches
the earlier note's slot layout. No other instruction addresses these
fields RIP-relatively. The other hits in the window (0x14255a9a0 and up)
belong to a separate neighbouring object. A byte search finds no stored
pointer to the getter.

The three installed tables are:

| Offset | Table | Live entries |
| --- | --- | --- |
| +0 | `0x14193fd30` | Six methods, below. |
| +8 | `0x14193fd60` | Only +0x160 (`0x1402efef0`, the error callback). Every other slot is the no-op `0x140163990`, **including item +0x68 and package +0x70**. |
| +0x10 | `0x14193fec8` | +0x38 `0x1402efdb0` and +0x40 `0x1402efe00`. The other ten slots are no-ops; the table ends at +0x58. |

The store's success notifications (item and package results) therefore
reach no-op slots in this tracker. They cannot remove an entry here.

## Primary methods

| Slot | Function | Behavior |
| --- | --- | --- |
| +0 | `0x1402efb70` | Registers the +8 interface with `0x140266200`'s virtual +0, and the +0x10 interface with `0x1412c5560`'s virtual +8. |
| +8 | `0x1402efbc0` | The matching unregistrations (virtual +8 and +0x10). |
| +0x10 | `0x1402ee7d0` | Tracks an ID with a nonnull value (previously documented). |
| +0x18 | `0x1402effc0` | Tracks an ID with a null value (previously documented). |
| +0x20 | `0x1402efc70` | Appends a caller pointer to the tracker's own list at +0x18 (head +0x20, tail +0x28, free list +0x30) unless it is already present. |
| +0x28 | `0x1402efd40` | Unlinks that pointer from the same list. |

None of these reads the map count or tests whether entries are pending.
The +0x20/+0x28 list holds the tracker's own observers. The error callback
notifies them only for a nonnull stored value and a positive error Word.
That branch is skipped for the quest startup entry.

The +0x10 interface entries only insert:
- `0x1402efdb0` converts its pointer argument through `0x140320d30` and forwards to primary +0x10.
- `0x1402efe00` allocates an 8-byte value with table `0x14193fca8` and forwards to primary +0x10.

## Every getter call site

All 23 calls to `0x1402eea80` dispatch only through the primary table:

| Slot | Call sites |
| --- | --- |
| +0 | `0x1402818db` (a run of subsystem +0 calls) |
| +8 | `0x140281b86` (the matching run of +8 calls) |
| +0x10 | `0x14035b1a2`, `0x14047f813`, `0x140482fe0`, `0x14054a053` |
| +0x18 | `0x14054ecbb`, `0x140563fd4`, `0x140564093`, `0x14067872a`, `0x14069f9c7`, `0x1407a5d8f`, `0x1407e2dc4`, `0x14080be49`, `0x141200531`, `0x1412005c4`, `0x1412a4de8`, `0x1413ddc44`, `0x1413df0ef`, `0x141529684` |
| +0x20 | `0x1402ace10` (argument: context +0x128) |
| +0x28 | `0x1402aa362`, `0x1402af5fd` (argument: context +0x128) |

The +0x18 sites include the quest startup consumer at `0x141200531` and the
Purchase follow-up at `0x1412005c4`. Twelve further producers insert
null-valued request IDs the same way. None of the 23 sites reads tracker
state after the call.

## Map helpers

Insert helper `0x1402ee5a0` and remove helper `0x1402ee410` are generic
hash-map routines shared with many unrelated maps: 19 and 4 call sites
respectively. Within the tracker, the observed callers are:
- the two track methods (`0x1402ee877`, `0x1402f0006`)
- the error callback's removal (`0x1402eff63`)

The remaining call sites lie outside the tracker's methods and were
not individually inspected. The getter is the only way to reach the object's address, and
all of its callers were classified above.

## Conclusion and limits

The null-valued entry that the quest startup consumer inserts:
- leaves only through a matching ordinary error delivery
- is otherwise released at process exit

**No reader in this object treats the entry as pending work, and no
tracker-based readiness wait was found.** By this tracker alone, an
emulator would not have to answer the GemStoreSearch request before zone
load.

This is a bounded static claim. It does not cover:
- pointers to the object copied out of the classified methods and used indirectly
- side effects of the two registration services (`0x140266200`, `0x1412c5560`)
- readiness gates elsewhere, such as the quest object's +0xf0 mask 0x10, the context +0xc8 service, or the store service itself

Runtime ordering remains unobserved.

## Evidence

Private bundle: `~/re/gw2/notes/209026-quest-store-tracker-readers-1.json`
(12 records). Frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All reads went through REA's Rizin provider; the executable digest was
unchanged after each call.

The first three bundle records are Ghidra startup timeouts with no
results. They are retained only as the record of that failed attempt.

The helper scripts are `~/re/gw2/notes/disasm-and-refs.py` and
`getter-call-windows.py`.

| REA evidence | Observation |
| --- | --- |
| `ev_33414370010e7bbcf03325686cbf9e94650c2fae7cc8439cdcbc980a1504e255` | Every RIP-relative reference to 0x14255a8c0–0x14255a9ff. |
| `ev_0e95eaf2ddebfbca95c6b212f2105390347c773810a8f17e68885e2ddff11a00` | Destructor, static initializer, getter callers and table references. |
| `ev_53cd49d02a69580429af78d68469f7aaea56b609b2654dc9445d6502e1a75bd1` | All 23 getter call windows and their dispatched slots. |
| `ev_beef7de8f9e6dd3f8b7f6be3dcc5b22adaf0180de98fe771c87da34e7241d95a` / `ev_aa7dd8a37f749794a4fd1e544158cdca964ff4096558c8aded3352074e9fcd5b` / `ev_06be2293c79290aef18050a48779db08826dae818fa7f522e4e538bc5146baaa` | Primary, +8 and +0x10 table contents. |
| `ev_2e67c6a51eaadc89560928d358c3bf480cc67bbf7f64cd285b8ba0c6300d8891` | Primary +0/+8/+0x20/+0x28 and +0x10 interface entry bodies. |
| `ev_798a2674eff2a6e563831e4f6cb5113ebfe8152995847dcc120c1fc04cea81e7` | Insert/remove helper and callback call sites. |
| `ev_6aa9fff9e0317eecc74df9f17f40fd6e718314c380fa49dc7950aacd2b833fc8` | No stored pointer to the getter. |
