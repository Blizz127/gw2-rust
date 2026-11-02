# Build 209026: mission connection entry (join completion and map data)

Static analysis, 2026-11-02, of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`. This
covers what a game (mission) server must do on the protocol-0 connection
after auth route result 20 ([mission join](209026-mission-join.md)). No mission
connection has been made at runtime. The Run59 User Agreement hold is
unchanged.

## 1. The join completes on the key exchange; no server message is needed

- **Callback registration.** The join connect callback (`0x14023f280`, case 1)
  passes `0x14023f130` (`Gc::GameSrvEncryptCallback`) as r8 to the connection
  factory at `0x14023f3f1`. The factory stores r8 at connection +0x110
  (`0x140fefb45`).
- **Constant result on the type-1 reply.** The type-1 key-reply handler
  `0x140fee7f0` sets connection state 3 and runs cipher setup. It then calls
  +0x110 (`0x140feea6a`) with r8 pointing at the constant eight bytes
  `{0x00300000, 0x0c471b58}`. The server sends nothing besides the 22-byte
  type-1 reply ([game handshake](209026-game-handshake.md)).
- **The result is a success.** The callback passes the join transaction ID
  (global `0x14266c334`, stored by join send `0x14023f1e0`) and that pointer to
  `0x14023b5c0`. The helper needs the matching transaction to be in state 3. It
  sets state 1, copies the eight bytes to tx+0x24, and returns success only
  when their low Word is zero. Here the low Word is `0x0000`, so on success the
  callback attaches the connection and sets the two connected globals
  (`0x14266c338`, `0x14266c33c`).
- **Completion event.** Join vtable `0x141923748` has slot +0x18 =
  `0x14023e0d0`, the counterpart of the route's `0x14023e100`. It copies
  tx+0x24 into the event payload at tx+0x38 and emits **event 0x25** (0x34
  bytes).
- **Connected flag.** The event-0x25 consumer `0x141417f97` reads the payload's
  first Word at event +0x10 (the same convention as the route consumer). When
  it is zero, it asserts `!(context->playerFlags & PLAYER_FLAG_CONNECTED)` and
  sets **ctx+0x228 bit 0x2 = PLAYER_FLAG_CONNECTED**. This is the bit the
  AgentsLoaded (412) sender requires ([map ready](209026-map-ready.md)). It
  calls view slot +0x178. Then, unless a deferred redirect (+0x150 mask 0x10)
  is pending, it sets **+0x150 mask 0x40** and calls `0x141418df0`.

**Consequences for a server:**
- After the 84-byte preamble and the key records, the join is complete. The
  server doesn't need to send any join acknowledgement.
- Mission control server **1267** also feeds `0x14023b5c0`. Once the
  transaction has left state 3 it matches nothing, and the handler
  `0x14023eeb0` ignores the helper's result and returns 1. A late 1267 is
  therefore an unnecessary no-op, not a hazard. The fixture does not send it.

## 2. Server 1063 is MapData

The handler is `0x141412e60`, with diagnostic `MsCli OnRecvMapData: Context Map
Id: %i` at `0x1422dd7e8`. Fields as decoded (after the u16 ID):

| Field | Wire | Client effect |
| --- | --- | --- |
| f0 | Dword | ctx+0x22c, the **local player ID**: getter `0x141411190` is named `MissionCliGetPlayerId()` by the vote assertion `m_ownerPlayerId == MissionCliGetPlayerId()` |
| f1 | Dword | `f1 & 0x3fffffff` → ctx+0x1f0, the **context map ID**. Bit 31 sets ctx+0x150 flag 0x200 and bit 30 sets 0x400 (meanings unknown). The masked ID is resolved through content slot +0x1a8 into **ctx+0x1e8**, the map definition. |
| f2 | Dword | ctx+0x1f4 and global `0x14266c150` (via `0x14023bfe0`), meaning unknown |
| f3 | Dword | ctx+0x1d0 (meaning unknown; the context constructor initializes it to -1). Its only reader in the client, through getter `0x1414110a0`, copies it into a record (`0x1411a6d68`); no index comparison was found |
| b4 | Byte | nonzero → ctx+0x228 bit 0x20 |
| b5 | Byte | nonzero → ctx+0x228 bit 0x10 |

The handler then sets **+0x150 mask 0x80** and calls `0x141418df0`.

### 1063 triggers client 423

`0x141418df0` acts only once both 0x40 (join completed) and 0x80 (1063) are
set. With mask 0x1 clear (the ordinary, non-creation branch), it reads the text
and audio languages, asserts `LanguageIsPermitted`, and sends **client 423**
via `0x141418bb0`. Either arrival order works. Because the join completes
during the key exchange, a server can send 1063 immediately after its type-1
reply. Client 423 then serves as the acknowledgement that both halves landed.

## 3. 1063 must precede 1060, and its map ID must match the route's map key

- **LoadMap compares two definitions.** The 1060 handler (`0x141412da0`,
  `MsCli OnRecvLoadMap: Load Map Id: %i, Context Map Id: %i`) forwards to
  `0x141418ee0`. That routine passes **ctx+0x1e8** as `mapDef` into view slot
  +0x190 → loader. The loader asserts `m_mapDef == mapDef`
  ([mission transfer](209026-mission-transfer.md)).
- **Where the loader's own definition comes from.** It is set earlier, by
  BeginMap (`0x140953670`): when route result 20 completes, `0x141417b80` →
  view slot +0x188 resolves the **route map key** (server 20 field 3, ctx
  +0x15c) through the same content slot +0x1a8 into loader +0x2b8
  (`0x1409536ba`).
- **1063 is the only source of ctx+0x1e8.** In the MsCliGame code range
  (`0x1413f0000`–`0x141420000`), the only 64-bit register stores to +0x1e8 are:
  - 1063's lookup store (`0x141412eec`)
  - zeroing in the context constructor (`0x141415093`)
  - zeroing in the reset `0x1414160d0` (`0x14141610e`)

  Route preparation sets the loader's definition, not ctx+0x1e8.

**Stated:** ctx+0x1e8 is null until 1063 arrives, so a 1060 sent before 1063
fails the loader assertion. 1063 must precede 1060.

**Inferred:** both values come from the same content slot +0x1a8. Equal keys
are assumed to return the same definition pointer. So 1063's masked f1 must
equal the map key the server returned in route result 20 field 3.

## 4. Minimal offline server sequence on the mission connection

1. Read the 84-byte preamble, then the client's type-0 key record. Send the
   type-1 reply. The join completes in the client (§1).
2. Send **1063** with f1 = the route map key; zero all other fields and leave
   bits 30/31 clear.
3. Send **1060**. Its Float3 position, two Float2s, angle, Qword and final Dword
   remain placeholders, because valid values are not established.
4. Expect **client 423** (after step 2). If map streaming succeeds, expect
   **client 420 MapLoaded**; on failure, **418 FailedToLoad**.
5. The loader then waits in AGENT_STREAM for a finalized controlled character
   (phase 4 of the [bootstrap sequence](209026-bootstrap-sequence.md)).
   Without population data, the honest stopping point is there.
6. **Never send 1036 BeginPlaying before client 412.** Its receiving view
   asserts loader STATE_READY, so sending it early triggers that client
   assertion. It is not a harmless no-op.

### Offline implementation

`gw2-game-observe --mission-preamble --mission-map-key KEY` implements steps 1–4.
KEY must equal the map key the auth phase echoed in route result 20. That
echo is the client's own route-29 map key, which normally comes from the
character summary inside `--auth-character-record`: the summary's Word at
+1, read little-endian ([play route](209026-play-route.md)). Settings
overrides or a failed record check change that choice. A mismatch would trip
the loader's `m_mapDef == mapDef` assertion.
- **Sends:** one server frame with 1063 for KEY, followed by an all-zero 1060.
- **Client traffic:** framed through `mission.xml` groups 39/40 and logged by
  message ID only.
- **Never sends:** 1267 or 1036.
- **Phase cutoff:** an unframeable message ends the phase, and capture
  continues.
- **Read timeout:** 600 s, because the client may stay silent while it loads.
- **Arguments:** the option requires `--mission-preamble` and rejects
  `--reply-frame`. Both checks run before the identity is loaded.

Tests:
- loopback: key exchange, then the encrypted opening, then captured
  423+420
- every-split client framing
- unframeable input disables the phase
- map-key bound

Not run against the client (Run59 hold).

**Population step (opt-in).** `--mission-population RECORD --mission-spawn X,Y,Z`
adds the candidate population sequence (`gw2_proto::population`).
- **RECORD:** the same server-17 character record body the auth phase serves.
  It supplies the name and the 45 appearance bytes. Only profession 1 is
  accepted, because only the Guardian creator, 477, is implemented.
- **Map-key check:** RECORD's summary map key must equal `--mission-map-key`,
  so the coupling above is now checked rather than just documented.
- **Spawn:** an operator choice. Packet-space spawn coordinates are not
  established.
- **When frames go out:** the sequence is sent once, after client 420
  MapLoaded. Server 1036 BeginPlaying is sent once, only after client 412
  AgentsLoaded, and only if population was already sent.
- **Inferred ordering:** sending population after 420 is a conservative choice
  (the map has streamed by then). AGENT_STREAM itself polls for the controlled
  character, so it would accept population earlier too
  ([map ready](209026-map-ready.md)).
- **Tests:** unit tests for the response rules, plus a loopback exchange (key
  reply → opening; 420 → sequence starting with 428; 412 → 1036).

Not run against the client.

### Player ID in 1063 (2026-11-02)

The readers of `MissionCliGetPlayerId()` are:
- vote code (`VoteCliVote.cpp`, `DlgGenericVote.cpp`), which compares a vote
  owner or candidate against it
- the Play-route map-record check

No population or control handler on the traced path reads it. So a zero f0
does not stop population. With population enabled, the observer now sends f0
= the plan's player index. That the player ID and 428's player index share
one index space is **inferred**.

Evidence: `ev_2dd3d867eb8f375c5343d4c07ea73c800a7e1592b990de7d3cbb087d56dda899`,
private `~/re/gw2/notes/209026-mission-player-id-1.json`.

### 1060's Qword is the map decryption key (2026-11-02)

LoadMap stores the 1060 Qword at loader +0x318 (`0x14095397a`). In
MAP_DOWNLOAD (`0x140952166`), the loader checks the map definition. If the
definition's +0x98 flags have **0x20000000** set and +0x318 is zero, it logs
`MapLoader: Completed STATE_MAP_DOWNLOAD with error (tried to enter an
encrypted map without a key)` and moves to state 6 (error). Otherwise the
Qword is passed with the map resource to `0x140c342a0`.

Queensdale's map-definition record has flags `0x50000200`: key 15, content
container 1282830 (SHA-256 `e5d11e4f…`), offset 3754040. **It is not
encrypted, so a zero key is valid for Queensdale.** Only 1 of the 1,554
type-45 map definitions has the encryption bit. Field 1060.loader_value
should be read as the map key.

Evidence: `ev_c9e5aba013435a64e4cd05d97f249e2a8fd65dff3d003d5e682bbe8a8b383ecc`
(Rizin-run `map-encryption-flag.py`, which also reads the content container),
private `~/re/gw2/notes/209026-map-key-1.json`.

### Server 431 timing (2026-11-02)

Handler `0x141255010` passes charContext to `0x1411b7700`. That body:
- dereferences the controlled character (charContext +0x98) without a null
  check
- runs the skill-bar and wardrobe helpers
- asserts `m_rewardTrackMgr` (player +0x9a60, from `ChCliPlayer.h`)
- sets charContext +0x208 bit 1
- tail-calls the listener notifier `0x1411b9680`

The direct body contains **no loader or view-state check**, so statically 431
depends on the populated player (444, 618, 247/248, 565, 538), not on READY or
1036. Listener-level dependencies remain as documented in the
[startup subscriber notes](209026-startup-subscribers.md).

The observer's population step now appends 431 to the 1036 reply after client
412. This keeps the bootstrap candidate order (1036 then 431); that order is a
choice, not traced.

Evidence: `ev_d77030d72abab0298fe697eb8d8bb281a87427ae4bd3500e0bf95e267c761522`,
private `~/re/gw2/notes/209026-startup-431-1.json`.

## Open

- Meanings of 1063 f0/f2/f3/b4/b5 and of the bit-30/31 flags.
- Physical roles of the 1060 vectors and final Dword. The Qword is the map decryption key (zero is valid for unencrypted maps such as Queensdale). The Float3 is the AGENT_STREAM streaming centre ([map ready](209026-map-ready.md)). The final Dword is exchanged into global `0x1425b99f8`, which is read in the CoScene code, meaning unknown.
- Whether the client sends any protocol-0 message before 423. Mission control
  server 3 makes the client send client 2, but nothing requires a server to
  send it.
- Population ordering relative to 1060 (bootstrap open item 3).

## Evidence

Private bundle `~/re/gw2/notes/209026-mission-entry-1.json` (mode 0600). Record
`ev_f9004655ce76ddc242932f45aed90742c3ad8556d1ae78ae80cb4700583ffc31` is a
Rizin-run `mission-entry-windows.py`. The script SHA-checks the frozen exe and
prints every window cited above:
- the callback registration and factory store
- the type-1 constant and call
- the encrypt callback and result helper
- the join send
- the join/route completion slots and both vtables
- the event-0x25 consumer
- the 423 gate
- the 1060/1063 handlers and LoadMap
- the BeginMap lookup
- the two zeroing stores
- the three diagnostic strings

The ctx+0x1e8 writer census came from a full objdump listing of the same
SHA-verified executable. It isn't a separate evidence record.
