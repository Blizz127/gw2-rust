# Build 209026: existing-character Play route (auth ID 29)

Static REA, 2026-11-01. This fills the main gap listed in the
[bootstrap sequence](209026-bootstrap-sequence.md): how the client requests
a game route when an existing character is played, rather than created.
The client was not run; the Run59 agreement hold stays in place.

## How Play reaches the selection callback

Character-select UI handlers call view slot +0x60 (table `0x141b7e818`,
entry `0x141b7e878` = `0x14094b180`) with the selected character's name:
- `0x1402a12c0` calls it from its selected-character branch, unless
  `0x1410f6a10()` and a flag bit 0x2 are set. In that case it first shows a
  text-`0xc27b` dialog and re-dispatches UI event 0x1c.
- `0x1402a07c0` calls it on UI event 0x1c after reading a non-empty name.
  The same handler sends event 0x1b to the creation slot +0x38.

`0x14094b180` is the name transaction already documented in
[character creation](209026-character-creation.md): auth **client 22**
(request ID + name). Its completion follows:

1. Server **8**: the type-0x15 branch writes the character's 16-byte value
   while the transaction is pending in state 3.
2. Server **3**: address/scalar/result; sets state 1.
3. Completion event **0x5d** (`0x140950d40` → `0x140950eba`): looks up the
   character record by name, then calls `0x14094db50`.

The dialog plus 0x1c re-dispatch is a **candidate** for the Run59
User Agreement step, which sits before client 22. That is not confirmed;
the panel was not exercised.

## Selection callback and branch

`0x14094db50`, called from `0x140950f09`, receives a selection result
pointer in RDX. A nonzero first Word posts UI event `0x10000008` with the
error and stops. A zero Word continues:

1. Posts UI event `0x10000009` and notifies the view.
2. Stores the selected character's context fields in mission context
   (global +0x1c8):

   | Setter | Field |
   | --- | --- |
   | `0x141410c20` | 16-byte value at +0x20, taken from R8 |
   | `0x141410c50` | 20-byte string at +0x30, taken from R9 |
   | `0x141410c80` | Copies a summary into +0x58 through `0x141419e80` (a Dword and a pointer from stack arguments) |

3. Branches:

   | Condition | Path |
   | --- | --- |
   | `0x1414113e0` (ctx +0x228 bit 0x2) is set **and** view +0x74 is 0–3 | `0x141410f30`: the creation continuation to empty mission ID 430 ([mission join](209026-mission-join.md)) |
   | Otherwise | **`0x14094f4d0`: the Play route** |

Context +0x228 mask 0x2 is the same mission-connection bit that client
412 requires ([map ready](209026-map-ready.md)). From character select,
with no mission connection, the static branch is therefore the Play route.

## Destination map selection (`0x14094f4d0`)

The function asserts that a default type-0x2d (map-definition) record exists
(content key `0x249`). It then chooses map key EBX:

1. **Overrides.** If `0x1410e9aa0(0x31)` is true, a parsed settings
   structure is checked for bits 0x14, 0x15 and 0x13: lookup by name,
   lookup by a second key, or a raw key at +0x69c. These look like
   command-line or debug overrides and are not otherwise identified.
2. **Character summary.** Otherwise `0x141410bf0` returns ctx +0x58,
   or null when its first byte is 0xff, and the **summary's +1 Word** is
   the map key. The [character protocol](209026-character-protocol.md)
   already identifies that Word as the map-definition key; Queensdale is
   key 15 ([map definition](209026-map-definition-content.md)). The record
   is looked up (type 0x2d, virtual +0x70) and checked by
   `0x14140ff50(record, 0x141411190())`. A failed check falls back to the
   default record's +0x28.
3. **Fallback.** A zero key, or a key whose map type is `0x14`
   (MAP_TYPES, invalid), produces the values {0, 0} for the first two
   route arguments.

A third argument, EDI, is a seeded table hash (`0x14026a4d0`, seed
`0x325d1eae`, table `0x141b90df0`). It hashes a string obtained through
context `0x140266200` virtual +0x200, then +0x110. The meaning of that
string is not identified. The function posts UI event `0x1000000c`, then
calls `0x141410e80(mapKey, 0, hash)`.

## Request construction

`0x141410e80` derives the game type from the destination map:

| Step | Detail |
| --- | --- |
| Map type | `0x1412e19c0(mapKey)`: content map-type lookup through `0x1428aeb00`. Asserts `mapType != Content::MAP_TYPES` (0x14). |
| Game type | `0x1414162d0(mapType)`: scans 18 `{gameType, mapType}` pairs at `0x1422dda10`; returns 0x16 if no pair matches. Asserts `gameType != GAME_TYPE_CHARCREATE` (1) and `gameType < GAME_TYPES` (0x16). |
| Route setup | General route setup `0x141417d50` (redirect type 1): sets ctx +0x150 bit 0x100, clears the 28-byte `gameRedirect` at +0x1b4, stores type 1, the game type at +4, 0 at +8, mapKey at +0xc, the second argument at +0x10 and the hash at +0x18. |
| Builder | `0x14023cb30`. Copies the 20-byte ctx +0x30 string to global `0x14266c060` and the 16-byte ctx +0x20 value to `0x14266c050`. Fills a new 0x98-byte transaction, then submits it through the same queue loop as the creation builder `0x14023cc70`. |

Transaction payload, at offsets in the new object:

| Offset | Value |
| --- | --- |
| +0x40 | Game type |
| +0x44 | 0 |
| +0x48 | Map key |
| +0x4c | Second argument (0 on this path) |
| +0x54 | 16 bytes from ctx +0x20 |
| +0x64 | Hash |
| +0x68 | 16 bytes from ctx +0xdd |

Send method `0x14023e490` serializes the transaction through sender
`0x140241f40` (message Word `0x1d`):

| Wire field | Transaction source | Play value |
| --- | --- | --- |
| Dword 0 | +0x30 | Request ID |
| Dword 1 | +0x40 | Game type |
| Dword 2 | +0x44 | 0 |
| Dword 3 | +0x48 | Map key |
| Dword 4 | +0x4c | 0 |
| Dword 5 | +0x64 | Hash |
| 16 bytes | +0x68 | ctx +0xdd value |

The selected character's 16-byte value (+0x54, from ctx +0x20) and its
20-byte name are **not** serialized; the builder copies them only to client
globals `0x14266c050`/`0x14266c060`. This matches the six-Dword + 16-byte
descriptor and the creation fixture `[req, 1, 0, 0, 0, 0]` + zero.

The table, as `mapType → gameType`:

| mapType | 0 | 1 | 2 | 3 | 4 | 5 | 7 | 8 | 9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **gameType** | 4 | 1 | 0xb | 6 | 4 | 3 | 9 | 5 | 7 |

| mapType | 0xa | 0xb | 0xc | 0xe | 0xf | 0x10 | 0x11 | 0x12 | 0x13 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **gameType** | 0xe | 0xd | 0xc | 0x10 | 0x11 | 0x12 | 0x13 | 0x15 | 0x14 |

## Consequences for the emulator

Statically, Play from character select requires this exchange before the
game route:

client 22 (name) → server 8 (type-0x15 character value, while pending) →
server 3 (completion) → client 29 → server 20 → game join.

Client 29 carries a non-creation game type derived from the summary map's
type, the summary map key (15 for the Queensdale fixture), the hash and
the ctx +0xdd value. A server parser should follow the wire table above.

Still open:
- Queensdale's map type, and therefore its game-type value; the content field read by `0x1412e2cd0` is not decoded.
- The source of ctx +0xdd and of the hashed string.
- Whether the `0xc27b` dialog branch is the User Agreement panel.
- The meaning of the override settings (0x31 gate, bits 0x13–0x15).

## Evidence

Private bundle: `~/re/gw2/notes/209026-play-route-1.json` (53 Rizin
records, including the session's earlier sweeps). Frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`,
unchanged after each call.

| REA evidence | Observation |
| --- | --- |
| `ev_7b91860cae2f91cb48d5557b7ce4b5e65ed47bdb58a0b01dd5fdd42593ee194a` | Creation and general route setup, creation builder, caller list. |
| `ev_eb38bc27af9e21cb038f621f310309748318d9ffbd395f61603c60a3429bb1ea` | `0x141410e80` request function, general builder `0x14023cb30`, single caller. |
| `ev_d6aff9106a9be6a436ba8b2e1e878506ca664c90db26df938ae822bb56f9d6e6` / `ev_da629188361f854d3ab41c22238cd3f33205df308e8d5c33f1d48940e59504b4` | Assertion strings `mapType != Content::MAP_TYPES`, `gameType < GAME_TYPES`. |
| `ev_105f2f5f4ab3a167cec9b0bc9cfd1d686d39afb84a33b152fb8a96a585956749` | Map-type lookup wrapper and game-type table scan. |
| `ev_004e159c5a3486c938c16d4a55ae3dfec13244c2ca639127c8098fffbe0623fd` | The 18 `{gameType, mapType}` pairs. |
| `ev_07f9934037f73126c225779a99e835e18da58b70521b694c8c03bfd87e73524e` | Destination selection `0x14094f4d0`. |
| `ev_c9a944a00ed969f26de024061abb41ae91eb2ccf7b1a68844fe80b5513996e4d` | Summary getter (+0x58), +0x20/+0x30 setters, hash helper. |
| `ev_82c0745192c1db55c424084d537f8f48b5b8e8d5cf0b2452a5ac12c52ba7100a` | Selection callback `0x14094db50` and its branch. |
| `ev_5bfca4ecf8951b0d51f405810834b9a50559a7f785f3539b79c5145b7eabfde4` | Summary store into +0x58; ctx +0x228 bit 0x2 predicate. |
| `ev_a555955567d4056480ad2ecdcc3e272a66b97b6e6633e22df84dbc4d54645b3a` | View-getter call sites dispatching slot +0x60. |
| `ev_16310fd81ad8446c5f9e647bafe0c1606de973264731304abb8534cef935efad` | Character-select handlers `0x1402a07c0` / `0x1402a12c0` calling slot +0x60. |
| `ev_731e01e18d7cdd33c9227df995f0b072275a7d63f912022286ad4c1879d09bb8` | Table entry `0x141b7e878` → `0x14094b180`. |
| `ev_10a09a2bb61bdf1105a7022805252c0b4c89e47f938529ffa322f5271d8f1780` / `ev_e2aa249f485cacf733f328dfc185a93d0cd868dd898c98597ff3e1fe7747bd66` | ID-29 sender layout and transaction-to-wire mapping. |
