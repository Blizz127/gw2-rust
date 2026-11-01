# Build 209026 mission route and join

Static REA analysis, 2026-11-01, of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No mission connection has been exercised at runtime. The existing User Agreement
decision remains pending; this work does not advance the UI.

## Route lookup and completion

Transaction constructor `0x140239a00` sets type `0xc0000013` and vtable
`0x141923720`. Its send method `0x14023e490` calls `0x140241f40`, which sends
auth protocol-3 client ID 29: six Dword varints followed by a GUID. The first
Dword is the transaction ID. The remaining request semantics are unresolved.

Server ID 20 has descriptor `0x1425b1360` and handler `0x140240ac0` in the
auth registration table. Its fields are four Dwords, Address, Dword. The handler
passes these to `0x14023ae60`, which matches field 0 to transaction +0x30,
requires low transaction type 0x13, copies the address to +0x78, final Dword to
+0x94, and fields 1–3 to +0x40/+0x44/+0x48, then sets state +0x20 to 1.
This handler does not additionally check that the prior state is 3.

The transaction completion method `0x14023e100` emits event 0x23 with its
96-byte result. Consumer `0x141417e50`, branch `0x1414180ca`, tests the result's
error Word and, on zero, calls `0x141417b80` with the returned address and
parameters. That routine stores mission state and calls `0x14023c990`, which
constructs a `0xc0000014` game-join transaction with vtable `0x141923748`.
The exact triggering UI action and valid route values remain unverified.

The character-creation route is now identified statically. Character-select
routine `0x1402a0720` obtains the view interface and calls vtable slot +0x38.
The table at `0x141b7e818` maps that slot to `0x14094b0b0`. On its allowed
branch, this initializes creation context through slot +0xc8, then calls
`0x141410e40` -> `0x141417d10` -> `0x14023cc70`. The last routine zeroes
the transaction's 96-byte payload and sets +0x40 to 1. Consequently ID 29
contains `[request_id, 1, 0, 0, 0, 0]` followed by a zero GUID on this path.
An alternate path's assertion at `0x1422dd648`, reached when its mode equals 1,
names this value `GAME_TYPE_CHARCREATE`. The UI event labels and allowed-branch
predicate are not yet identified, and this request has not been seen at runtime.

The intended static sequence is therefore creation route lookup (auth ID 29),
route result (auth ID 20), protocol-0 game join, and later creation-data request
(mission ID 415). It does not follow that replying to ID 29 alone is enough
to initialize the creation scene.

## Game join

The join start method `0x14023e740` is named by the client string
`Gc::GcTransGameJoin::OnStart`. It creates an 84-byte preamble, then calls
`0x14023f1e0` with the transaction ID, destination address and that preamble.
The preamble contains a first Dword `0x000c0500`, build 209026, product 101,
and a Dword `0x48` at offset 12. It also copies transaction fields, two
16-byte account globals and another 16-byte global; their precise wire roles
are not all established. Offset 20 was not assigned in the inspected start
routine, so this is not yet a complete reproducible preamble specification.

Connection callback `0x14023f280`, case 1, creates the protocol-0 connection
with factory `0x140fefb00`, a 20-byte parameter block and encryption callback
`0x14023f130`. That callback is named `Gc::GameSrvEncryptCallback`. It calls
`0x14023b5c0` with the join transaction ID: that helper requires a matching
transaction in state 3, sets state 1, copies the eight-byte result and succeeds
only when the error Word is zero. On success, the callback attaches the
connection and sets the connected flags. Join completion emits event 0x25.

This traces prerequisites for the separately documented
[creation request](209026-character-creation.md); it does not prove a complete
mission bootstrap or successful character creation.

### Observer preparation

The connection job copies exactly the supplied preamble length into its queued
buffer (`0x140243320`). The mission callback obtains the 136-byte parameter blob
at `0x142111220` through `0x140ff45b0`, then uses the same `0x140fefb00`
factory as the auth transport. The factory emits the type-0 key record with
two-byte header and at most 64 public-value bytes, aligned to four bytes. This
supports reusing the observer's cipher exchange after consuming the mission
preamble; runtime mission interoperability is still untested.

An explicit mission observer mode can therefore consume 84 bytes, validate the
four established Dwords (kind, build, product and length field), and leave the
remaining preamble opaque. It must not infer identities from those bytes or
apply the protocol-3 auth reply fixture to a protocol-0 connection. The default
16-byte auth preamble behavior stays available for existing replay wrappers.
No mission connection is launched while the User Agreement decision is pending.

Implemented as `gw2-game-observe --mission-preamble`, using a separately chosen
loopback listener and the existing explicit local identity. The option rejects
`--auth-reply-body`. Focused checks pass: all truncated prefixes, exact
consumption before a following key record, invalid established header fields,
and synthetic loopback DH/RC4 exchanges for both prefix modes. The loopback
check receives a compressed server frame, sends encrypted client bytes and
verifies that only their plaintext reaches a mode-0600 capture. The existing
fragmented auth-reply test also passes (three tests total), the observer builds,
and the CLI fixture-conflict check rejects before identity loading/binding.
These are local test peers, not a frozen-client mission replay.

REA evidence: `ev_5509884917cbe400b5ce9e7327858c44e7a105edaa5643ba53eb9d8513914105`
(queued preamble and protocol-0 factory call),
`ev_a13e50a1dd75cbeb1662c654e568c897886de17e5ca3e5461039782f916c5273`
(shared parameter getter and key-record construction),
`ev_1bc91ab7300f8203dfc5c4984c745b3d2137796b2c1c7b2ad5fa7759bc13d743`
(84-byte preamble construction). Private export: `209026-mission-static-8.json`.

## Small mission control group

Protocol-0 group 40 is registered at `0x1401a6f90`, with two client descriptors
at `0x141923b70` and six server descriptor/handler pairs at `0x141923b80`.
These are distinct from auth protocol-3 group 40.

| Server ID | Descriptor fields | Handler behavior |
| --- | --- | --- |
| 1 | Four Dwords | `0x14023ede0` copies their low Words into the connection error, if no error is already set, and returns false. |
| 2 | Two Dwords, String(128) | `0x14023ee20` saves text and a message ID; diagnostic format names DisconnectEx, msgId and prevMsgId. |
| 3 | Empty | `0x14023ed60` sends client ID 2 with one Dword: 1000 divided by a local setting, or 40 if that setting is zero. |
| 4 | Dword | `0x14023edc0` calls `0x14023b150`; values at most 5000 enter a ten-entry history and an event is emitted. Meaning unresolved. |
| 5 | Float, Word, Float, Word, Word, Word | `0x14023ee70` copies six values into account/game globals through `0x14023b110`. Meanings unresolved. |
| 1267 (0x4f3) | Five Dwords | `0x14023eeb0` passes field 0 as transaction ID and the low Words of fields 1–4 as a result to `0x14023b5c0`. |

These handlers identify control traffic and one automatic client response. They
do not establish which messages a server must send first or which scene-loading
messages are required. Float remains outside the capture census's admitted
primitive set until its wire representation is separately verified.

## Address wire byte order

Schema kind 12 dispatches to encoder `0x140ff0b2f`. It copies all 28 bytes
from the decoded address directly to the output, with no byte-order conversion
or length prefix. The IPv4 constructor `0x1409c0080` zeroes 28 bytes, writes
family 2, stores the network-order address at +4, and calls imported `htons`
before storing the port at +2. The port setter `0x1409c0be0` does the same;
its IPv6 branch reaches `0x1409ccbb0`, also calling `htons` at port +2.
Predicate `0x1409ccc10` identifies IPv6 by family 23. The formatter reads the
IPv4 port through imported `ntohs`.

REA resolves IAT `0x141915270` to `WS2_32.dll!htons` and `0x141915258` to
`ntohs`. Thus family is little-endian and port is **big-endian** for both
families. IPv4 `127.1.0.2:6112` begins `02 00 17 e0 7f 01 00 02`, followed
by 20 zero padding bytes. The existing Rust codec's little-endian port was a
historical-reference assumption; round-trip tests could not detect that error.
The corrected codec passes an exact-byte IPv4/IPv6 test, including every
truncation and preservation of the following field. `gw2-game-observe` and
`gw2-capture-census` build successfully. No runtime route reply has been sent.
No claim is made here about
independent verification of nonzero IPv6 flow-info or scope fields.

The fixed 28-byte copy also permits the metadata-only capture census to admit
Address fields using the corrected codec. This extension reports message
boundaries and IDs only; it must not print destinations or other field values.
The census extension passes all three focused tests and builds. Handwritten
creation (86-byte) and route (36-byte) fixtures parse in consecutive pairs;
every nonempty truncation, invalid address family, and absent/oversize fixed
array count is rejected. The saved run-58 capture still consumes all six
messages and 233 bytes, with unchanged boundaries
(`ev_a5c47782ac3dec6276661ad3601ab52dfbe9930a5a3435178fd97383117437a5`).
This is fixture/capture analysis, not runtime mission validation. The added
evidence is exported privately in `209026-mission-static-2.json`.

## Evidence

Private bundles: `~/re/gw2/notes/209026-mission-static-1.json` through
`209026-mission-static-3.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_efb0c852ec226cba26970a168ec167d64fc6f37408e1049a8c55cd8bbf32e233` | Protocol-0 group-40 registration. |
| `ev_968a8ba79cbd482fa3496e57a5bf23675c940c48a68ba9e96289ff00869788b6` | Control handlers and descriptors for IDs 1, 2, 3, 4 and 1267. |
| `ev_db6b73b11687b7d1c4b69561068e512a0bd740673946a4cda357bf6a58067444` | ID-5 descriptor, scalar storage, bounded history and diagnostic text. |
| `ev_366c7c8e6cf6a30b9c8dce6765d875967671ec5c63ce19141318876a298d8776` | Route constructors, including zero payload plus mode 1. |
| `ev_31788c6ddd4992bc634ef461df2ba23b74d3d4952705ab869cd785d391290116` | Creation route setup and general route setup. |
| `ev_20b9425269c9ba6121e6c4010c7497e7407633fd5c571f664953d76293fdf66d` | View call path and GAME_TYPE_CHARCREATE assertion. |
| `ev_a5e305fa68b209ea356b18d3d35c670148a7a071d7c43d3fea5dc38c7a026e38` | View vtable slot and mode-1 comparison. |
| `ev_af809e5875e85a09b74987715c1b02f7e3151bdc81ce560b0153996a67002a4d` | Character-select scene calls creation view slot. |
| `ev_17ea319db572bb3fe9f12a6b2493e1a4eb185bbdae16a1531fce2f6aa4ba5ef5` | Route/join transaction vtables and construction. |
| `ev_c7a18d00178716cf541c903e3a200cba374d80f0e47c72b1c017a5590d8c5c25` | Auth ID-29 sender. |
| `ev_a6a6b01af93a755c24a3634c144fa0115570487bab833229444aca10cf77a1c3` | ID-20 handler arguments. |
| `ev_3b783184ecda927779e9c6a1f911d09ea8e5f6b421d1f45c62f8dc71dce5c890` | Correlated route response and transaction storage. |
| `ev_148fb9cd3408f4114e91ea6d71a6a305534b23077af8df567718d614a71cf26c` | Events 0x23–0x26 and route completion emitter. |
| `ev_7561ef1a1fa28264ed87b3218e5d7e578df1cbd317bf44c64485438e2e4d14f8` | Route result consumption into join. |
| `ev_3144f81e925f25f632c9cc77ea7c78c7ce7a21cea4437cecbb426fe395d0b81b` | Named join start and preamble construction. |
| `ev_fabb68529af93d22ff3cfa9f1b08f28320be12b205c36e1c273e2c7de8eb00b9` | Build/product getters. |
| `ev_d21d23aa0de3ca643915608267ac2406fc2f8d3228ae6bf93e2c6a659c6e453a` | Connection factory and callback registration. |
| `ev_c8959237887fb37b2c01945c161050ba77e8143f54f408352063b46856a2c020` | Encryption callback and connection flags. |
| `ev_778e29940fba81820e3f4f15d49d88350a47e16baf640fe9f81c57805d99d73d` | Transaction completion helper and named callback. |
| `ev_b0a1737b7e2511e44eff92c3b2002576d9243cdacf35b96e1f219596916a93c9` | Raw Address encoder and auth handler registration. |
| `ev_ff0c4ee8091b1c79401ef60aa1bb7c24008a5ac4502c7eee9a1ea3d76135c0db` | ID-20 descriptor. |
| `ev_4b9c4e14e86efc123a2b8aa579ab2eb134a940aa6daf4bce040fb3c3b60355dc` | Address setter, parser and formatter. |
| `ev_338869311dd5f4fa2738d0014b30d8ab8189db3083da986cdf5c00a31a068510` | htons/ntohs imports, IPv4 constructor and IPv6 port setter. |
