# Build 209026: client→server requests and the replies that complete them (static, partial)

Static analysis by a read-only helper (2026-11-03). The lead re-checked: 0x141319930 (client 278's sender) has one direct call site, 0x141314e0d, behind `test byte [rbx+0x38], 4`. The per-message sender list is data/protocol/209026/client-senders.tsv (local, gitignored): 525 rows, 418 senders resolved.

Read-only static RE of `Gw2-64.exe` (rizin, Python scans; no edits, no client run). Raw dumps stayed in the session scratchpad and are not kept.

## Bottom line

1. **Only the auth connection has a request-id completion mechanism.** It is verified (below).
2. **No shared request-id/pending-callback completer was found on the game connection (CharForce main table and the module tables), but this is a direct-call scan only; indirect dispatch (vtable calls, the `0x14026cfa0` event callbacks, `call r9`) was not followed, so a completer hidden behind a vtable cannot be ruled out.** The `0x1409b1fb0` TLS-context getter appears among the first two direct calls of 947 of the 1253 handlers; the samples read (Mail 996–1006, AgMsg 66–80) fetch a module object from a context slot and call a module method with the message fields. The waits that ARE known on the game connection are Mission state-machine steps, listed in the next table.
3. **DebugInfo client 278 does not wait for any reply.** Verified (below). Frame it, send nothing back.
4. The documented game-side waits are Mission handshake steps (table below), keyed on Mission state, not on a request id. The map-load stall is probably not a missing reply to 278.

## Mechanism A (auth connection): outstanding transaction list, verified

- Global intrusive list head `0x14266bfc0`, link offset `[0x14266bfb0]`. A transaction object has request id Dword at **+0x30**, state at **+0x20** (3 = sent and waiting, 1 = complete), type at **+0x18** (word).
- Each reply handler calls a helper that walks the list with `cmp dword [rax+0x30], <D0>`, requires state 3 (`cmp dword [rax+0x20], 3`), checks the type, applies the result, and sets state 1 (`mov dword [rax+0x20], 1`).
- A heuristic scan of all 1253 server handlers (handler plus direct callees; backward-jump loop with a `cmp dword [reg+disp], reg`) flagged 42 handlers. The auth ones are exactly the transaction walkers; the rest are not requests (see Mechanism B).

| auth server id | handler | helper | what it requires | evidence |
| --- | --- | --- | --- | --- |
| 3 | `0x140240750` | `0x14023add0` | state 3; calls the transaction's virtual +0x20, sets state 1 | verified (disassembly) |
| 4 | `0x1402407f0` | `0x14023b5c0` | state 3 → 1 | verified |
| 7 | `0x1402407a0` | `0x14023a9e0` | state 3, type **7**; sets state 1 | verified; doc `209026-auth-messages.md` |
| 8 | `0x140240850` | `0x14023a770` | state 3, type 7 (stores account data) or type 0x15 branch (character value) | verified |
| 10 | `0x140240990` | `0x14023a630` | state 3, type 7; stores config buffer | verified |
| 16 | `0x1402409e0` | `0x14023a960` | state 3, type 7 | verified |
| 17, 18 | `0x140240a00`, `0x140240a50` | `0x14023aa50`, `0x14023ab70` | state 3; type word 7 or 0x18 (17) | verified |
| 20 | `0x140240ac0` | `0x14023ae60` | type **0x13** (no state-3 check seen); sets state 1 | verified |
| 28 | `0x140240b00` | `0x14023aef0` | type 0x13 | verified |
| 1267 (table `0x141923b80`) | `0x14023eeb0` | `0x14023b5c0` | state 3 → 1, D0 is the id | verified |

Request side: transaction send methods (vtable) call the auth senders. Known sender→client id (from `209026-auth-messages.md`, `209026-home-world.md`, `209026-play-route.md`, `209026-character-creation.md`, plus my sender map):

| client id | sender | role | reply (server ids) | status |
| --- | --- | --- | --- | --- |
| 10 `{req, Guid, Guid, S64, S28}` | `0x1402416f0` | account/session request (type 7) | 8, 10 (config), 7 (completes, D0 = req) | verified (existing doc; helpers above) |
| 14 `{req, world, byte}` | `0x140241950` | home-world select | 7 | verified (home-world doc) |
| 22 `{req, S20}` | `0x140241b10` | character name (Play) | 8 (type 0x15), 3 (completes) | verified (play-route doc) |
| 29 `{req, 5×D, Guid}` | `0x140241f40` | route request | 3 (completes); creation route also 1048..1051 on game side | verified (play-route doc) |
| 20, 21, 23, 24, 25, 26, 36–40, 16, 12 (first Dword) | see TSV | likely transactions | 20/28 (type 0x13) are the candidates; the type→client mapping is **not traced** | inferred only |

## Mechanism B (game connection): no generic request id

Evidence:
- Handler shape, e.g. Mail server 996..1006 `0x1413eb510..0x1413eb880`: `call 0x1409b1fb0; mov rcx,[rax+0x1a0]` (mail module) then a method call with D0.. as arguments. Same shape for AgMsg handlers `0x1410347a0..` (allocate a command, queue to the agent).
- The 42-handler scan hits in the game tables were checked on samples and are not request completions:
  - server 18/19 (table `0x14213fe70`, `0x1410f6430`/`0x1410f6610` → `0x1410f79c0`/`0x1410f77b0`): binary search in a sorted 12-byte-entry table keyed by D0 (a flag/value set).
  - server 994 `0x1413e9630` → `0x1413e8bb0`, 1022/1027, 910/911, 182/183/188/190/193: lookups of an entity or slot by id in a module array, then state update (`0x141209440` stores a value and fires event `0x14026cfa0`).
  - not read individually: 209, 304, 305, 374, 483, 524, 526, 542–547, 671, 783, 805, 884, 1290, 1292 (same scan hit; treat as lookups, unverified).
- Table `0x142370e90` (server 11–16, `CStr64` items, `0x1417ed1c0` compares `[rcx+0x30]`) is a console/variable table; I did not trace it.
- The one genuine request-id service in the binary (`0x1402ee5a0`/`0x1402efc10`, request ids 1..0x1fffffff, XML requests such as `GetLimits` / `GemStoreSearch` over the portal HTTP service) is separate from the game protocol (see `209026-quest-store-request.md`); replies arrive over HTTPS, not as game messages.


## Game-connection request→reply pairs already established (Mission module; not request-id based)

All are documented elsewhere in the repo (RE by earlier sessions); I did not re-derive them, I only cross-checked 278 and the generic scan.

| client message (module) | server message that completes / follows it | status | source |
| --- | --- | --- | --- |
| join (auth 29 route request, auth connection) → game-connection key exchange | server type-1 reply, then **1063 MapData** and **1060 LoadMap** (1063 must precede 1060; 1063's map id must equal the route map key) | verified (static) | `209026-mission-entry.md` §2–3, `209026-mission-join.md` |
| **423** (Mission, languages) is the client's ack that join completed AND 1063 arrived | none needed | verified (static) | `209026-mission-entry.md` "1063 triggers client 423" (`0x141418df0`) |
| **420 MapLoaded** (sent when the loader's map-stream stage completes; 418 FailedToLoad on error) | population sequence starting **428** (player create), then 37, 443, 182/183, 282, profession, 151, 247/248, 565, 538, 605, 170, 618, 444 | ordering inferred (emulator sends it after 420; AGENT_STREAM polls for the character) | `209026-map-ready.md`, `209026-mission-entry.md` population step, `209026-bootstrap-sequence.md` |
| **412 AgentsLoaded** (after READY and connection/view guards) | **1036 BeginPlaying**, then **431** (1036 then 431 is the bootstrap candidate order) | verified static for 412→1036 gating (the emulator sends 1036 only after 412 and only if population was sent); 431 order inferred | `209026-mission-entry.md`, `209026-map-ready.md` |
| creation path: server **1049** (empty) and join complete → client **410**; server **1050** (empty) → client **411** | the reverse direction: the server pushes first, the client acks | verified (static) | `209026-character-creation.md` "Creation-scene startup exchange" |

These are what a UI "stuck waiting" on map load most plausibly traces to: 420 without a population (no 428…), 412 without 1036. In-world after 1036, no further reply is documented as required (STATUS 2026-11-03: no game-connection keepalive needed).

## DebugInfo client 278: no reply is awaited (verified)

- Sender `0x141319930` writes message word `0x116` (278), fills it from system/video info (calls `0x140a68e60` ×6, `0x140a68bd0`) and sends with `0x140ff0110` (size 0x56) at `0x141319ade`.
- Its only caller is `0x141314e0d`, inside the DiCli connect routine (`0x141314dc0`): `or [ctx+0x38], 1; …; if !(flags&4) { flags|=4; send 278 }`. Immediately afterwards it asserts bit 1, then loops over a queue at `ctx+0x100` flushing buffered debug items through virtual `+0x48` and frees it. It stores no request id, sets no pending state, and nothing after the send blocks.
- The other DebugInfo client messages 279–286 are plain senders (`0x141319570`, `0x1413195e6`, `0x141319830`, `0x1413198d0`, `0x141319b10`): fire-and-forget log/diagnostic uploads. Server DebugInfo ids 770–781 are debug-console commands, not replies.
- The stall seen after 278 in the 17:07Z run was the framing failure plus whatever the Mission module still waits for (population 420 etc.), not a reply to 278.

## Likely in the first minutes (what I can and cannot say)

- Mission: see the pair table above. The remaining Mission client messages are empty (409–412, 417, 418, 420, 422, 428–430, 434, 437, 439, 442, 443; 12, 60, 215, 264, 308, 369, 425, 438). My byte scan did not resolve their senders (not verified why; the likely reason is a generic empty-message sender, see "Not done").
- Agent, CharForce, Chat, Guild, Mail, Squad, Loot: **no pending-request structure was found by the direct-call scan; indirect dispatch was not followed.** Mail client messages 392–402 begin with a Dword, and the server handlers 996–1006 update mail module state; I could not tell whether that Dword is a request key or a mailbox handle (not traced to the mail UI). Treat these modules as "unknown", not "no replies needed".

## Suggested emulator rule of thumb

- Auth: implement the transaction pairs above (correlate on D0 = request id; keep state-3 semantics: send 8/10 before 7, as the existing doc says).
- Game: no reply is needed for 278 or any DebugInfo message. For Mission follow the pair table (1063→1060; 420→428…; 412→1036→431). For other modules, if a UI hangs, look first at that module's connect-time server pushes (e.g. Mail 996/997/1263.., Mission 1034..1081); a request-id pairing there is not excluded.

## Not done / limits

- No per-module audit of "what the UI waits for" beyond DebugInfo; ~45 min budget.
- The 14 unread scan hits and the auth type→client mapping for ids 20–26, 36–40 are open.
- **Concrete next step for the 107 unresolved client senders:** the size-2 send at `0x14023f620` (inside `0x14023f5d0`) looks like the generic empty-message sender. Find its callers (`call 0x14023f5d0`) and read the immediate loaded into the id argument; that should resolve most Mission empty ids.
- Not followed: virtual/indirect dispatch from game handlers (`call r9`, `call [rax+0x48]`, event callbacks via `0x14026cfa0`), so any pending-callback store reached only through a vtable is unseen.
- Sender map used a byte scan for `mov word [rsp+x], id`; 107 client ids unresolved (generic empty-message senders and unusual encodings, e.g. 278 was patched in by hand).

## Loading-screen path: waits beyond the direct-call scan (lead, 2026-11-03)
The loader was traced earlier, including its vtable and service calls, in [map ready](209026-map-ready.md) and [bootstrap sequence](209026-bootstrap-sequence.md). On that path the waits are these, and none is a request-id reply:
- **SERVER_WAIT:** needs 1060; it is sent.
- **AGENT_STREAM:** waits with **no timeout** for a finalized controlled character. That character comes from the population sequence after 420 (428 … 170, 618, 444).
- **World readiness:** a soft wait. It times out after about **120 s** (5 s with client setting 0x3d), then logs `Map load hang on STATE_AGENT_STREAM detected: IsWorldReady: %d, CharLoadingCount %d` and continues.
- **READY_WAIT countdown, then READY:** after READY, 412 is sent, and the server answers with 1036 and 431.

So after 0edbe1f, a loading screen that clears after about 2 minutes is the world-readiness timeout, not a hang. One that never clears means the controlled character was never finalized, which is a population problem.

`tools/owner-run-report.py` prints both diagnostics if the client writes them to ArenaNet.log (whether it does is not verified).
