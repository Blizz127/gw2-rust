# Build 209026: home-world list and selection (auth protocol)

This is static analysis of the frozen client. Nothing here has been confirmed live yet.

**Owner symptom (2026-11-03 16:58Z).** After a full login, character select showed a
modal "Initial Home World Select" dialog with an empty list. PLAY stayed greyed out.

## Client 15 (one Byte): User Agreement accepted, not a world request
- It is sent by `0x140241690`, from `0x14023bed0`, which stores the byte at `0x14266bee4`
  and expects no reply.
- The byte is the version of a content file tagged `'eula'`: `0x1410f4370` asserts
  `territory < TERRITORIES` in `AcctCliApi.cpp`. The "version" label is inferred; the
  equality check at `0x1413e6420` is verified.
- At login the stored value comes from the low byte of server 8 Dword4, via
  `0x14023a770` (record +0x1e8). Echoing it there later would hide the agreement.

## World list: server 23, 25, 24 (handlers in table `0x141923fe0`)
- **23** (no fields, `0x140240b60`) clears the staging list at `0x14266c470`.
- **25** (4 Dwords, `0x140240ba0`) adds one entry:
  - D0 is the world id (verified: the search key, and both the Name and Location columns
    use it).
  - D1 is the population, 0–3 (verified: `0x140870e00` asserts on any other value).
  - D2 is probably a transfer cost; use 0 (inferred).
  - D3 is probably hours (×3600); use 0 (inferred).
- **24** (no fields, `0x140240c90`) publishes the list (`[0x14266c458]`, count
  `0x14266c464`). The UI reads it through `0x14023d080` and `0x14023d8e0`
  (`ShardSelect/Ssd*.cpp`).
- **The grid fills only once.** `0x14086b9e0` is called only from the grid's create
  handler `0x14086b3dc`, so the list must arrive with the login reply, before the dialog
  opens.
- **Location filter:** defaults to 9 ("all"). Region and territory are derived from the
  world id (`0x1409b1b20` and `0x1409b1b60`).
- **Content check:** when server 8 Dword3 is 2, a row needs its world id in content
  type 373. Ids 1001–1059 and 2001–2039 are present there, so 1001 passes.

## Selection: client 14, then server 7
- **Select** (`0x140869070` → `0x14023bf00`, send `0x140241950`) sends **client 14** =
  {Dword request id, Dword world id, Byte 0}.
- The client waits for **server 7** (`0x1402407a0` → `0x14023a9e0`):
  - D0 must equal the request id;
  - the low 16 bits of D3 are the status, where 0 means success.
- On success, `0x14023deb0` sets the current world (`0x14266c150`) and the home world
  (`0x14266c154`), then posts event 0x20.
- **Not traced:** which check then enables PLAY.

**Login worlds (static, in use since kit 102bad3).**
- Server 8's handler `0x140240850` passes the message fields to `0x14023a770`:
  - Dword7 (message +0x36) arrives in r9d and is stored at record +0x1a4;
  - Dword6 (+0x32) arrives as the first stack argument and is stored at +0x1a8;
  - Dword8 (+0x3a) is stored at +0x1ac.
- `0x14023d96d`–`0x14023d986` copy +0x1a4 to the current world (`0x14266c150`) and +0x1a8
  to the home world (`0x14266c154`), and +0x1ac to `0x14266c158`.
- Caveat: when `0x1410f4be0` returns nonzero and `0x1410e9aa0(0x60)` is set
  (`0x14023d944`), the home world is stored as 0 whatever Dword6 says. Not traced.
- Owner run 2026-11-04 01:51Z (cd93d58): no dialog and no client 14, so character select
  showed world "Unknown" with PLAY greyed. The server keeps no account state, so each
  session started with no world.

**Client 12** {Dword request, BufferVarLarge ≤ 4096} (sender `0x1402418b0` through vtable
slot `0x14023e3d0` at `0x141923708`) was sent at 01:51:51Z (841 bytes) after the owner
"clicked it again" at character select. It is a different transaction from 14: the
neighbouring send slot `0x14023e3f0` goes to `0x1402419d0`, and its completion
`0x14023de70` posts UI event 0x18 with the result at +0x24. Unanswered; not world-related.

## Emulator (`game_observe/auth.rs`)
- The login reply now sends 23, 25 {1001, 0, 0, 0} and 24 right after server 8.
- The play phase answers client 14 with server 7 {request, 0, 0, 0, 0, 0, 0}.
- Since 102bad3, gw2-game-observe sets the login body's Dword6/Dword7 to 1001 where zero
  (`ReplyFixture::with_home_world`; test `login_body_carries_home_and_current_world`, and
  the deployed-fixture audit checks it).
- Tests: `home_world_selection_is_confirmed_with_server_7`, decoded under the client's
  own auth table. The deployed-fixture audit `every_auth_reply_frame_matches_the_client_auth_table`
  now expects 8, 23, 25, 24, 10, 17, 7, then 8, 3, 20.
