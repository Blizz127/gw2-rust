# Build 209026: client message registration tables

Static extraction from the frozen `Gw2-64.exe` by `~/re/gw2/notes/extract-client-tables.py`:
- **Scan:** runs of 8-byte descriptor pointers, where each descriptor starts with a message-id field.
- **Output:** `data/protocol/209026/client-tables.xml` (local, gitignored): 34 tables, 525 messages, sha256 `b0f026b4…`, REA `ev_3ada7c78…`.

**Module column.** It is the last source path string in the 0x300 bytes before each table (the same pattern as AgMsg.cpp, see [agent client messages](209026-agent-client-messages.md)).
- That makes it the module that most likely registers the table. This is inferred from adjacency.
- A table without a path in that window shows `?`.
- Id ranges come from the table, in id order.

**How the game observer uses it** (`game_observe/mission.rs`):
- It loads mission groups 39/40 and AgMsg group 0 first.
- It then loads the main game table `0x14216e260`, which is authoritative and ends exactly at its server table `0x14216e940`.
- Then it loads every other table except the auth connection's `0x141923f00`, which is identical to `auth.xml`.
- Result: 428 more ids, none ambiguous.

| table | ids | n | module (inferred) |
| --- | --- | --- | --- |
| `0x141923b70` | 1..2 | 2 | `Net\Cli\GcApi.cpp` |
| `0x141923f00` | 1..40 | 28 | `Net\Cli\GcAuthCmd.cpp` |
| `0x141927cc0` | 445..1146 | 11 | `Game\Portal\PlWatch.cpp` |
| `0x141b7b9b0` | 508..1268 | 7 | `Vendor\Cli\VendCliContext.cpp` |
| `0x141b812f0` | 514..1280 | 3 | `Vote\Cli\VoteCliVote.cpp` |
| `0x141b8e2e0` | 516..523 | 7 | `Game\Wvw\WvwCommon.cpp` |
| `0x14211bdb0` | 3..26 | 23 | `Engine\Agent\AgMsg.cpp` |
| `0x1421590f0` | 462..1233 | 2 | `AssetEncryption\Cli\GaeCliContext.cpp` |
| `0x14215dce0` | 27..89 | 2 | `BigBattle\Cli\BigbCliContext.cpp` |
| `0x14216e260` | 28..255 | 219 | `Game\Char\CharForce.cpp` |
| `0x1421800a0` | 260..715 | 5 | `Chat\Cli\CtCliMsg.cpp` |
| `0x142183680` | 256..258 | 3 | `Claimable\Cli\ClmCliClaimableGuild.cpp` |
| `0x142185290` | 265..277 | 3 | `?` |
| `0x142239a50` | 267..750 | 9 | `Game\ContentFunction\ConFunContext.cpp` |
| `0x14223a5b0` | 275..756 | 3 | `Conversation\Cli\ConvCliContext.cpp` |
| `0x14223bf20` | 278..286 | 9 | `DebugInfo\Cli\DiCliContext.cpp` |
| `0x142246880` | 289..293 | 5 | `?` |
| `0x1422518c0` | 294..864 | 37 | `Game\Guild\GldCommon.cpp` |
| `0x1422557f0` | 332..904 | 9 | `Homestead\Cli\HsCliHomestead.cpp` |
| `0x14225b6a0` | 340..906 | 11 | `Game\Instance\InsConst.cpp` |
| `0x1422d6a08` | 352..979 | 2 | `Leaderboard\Cli\LbCliContext.cpp` |
| `0x1422d7368` | 353..980 | 5 | `Lfg\Cli\LfgCliContext.cpp` |
| `0x1422d8d30` | 357..362 | 6 | `?` |
| `0x1422d9290` | 363..984 | 7 | `Loot\Cli\LootCliContext.cpp` |
| `0x1422d9df0` | 392..996 | 15 | `Game\Mail\MailConst.cpp` |
| `0x1422db6f0` | 405..1017 | 2 | `Game\MapLayout\MlCache.cpp` |
| `0x1422ddaa0` | 12..438 | 41 | `Mission\Cli\MsCliContext.cpp` |
| `0x1422e7490` | 376..1203 | 6 | `PvpExternal\Cli\PexCliContext.cpp` |
| `0x1422e96c0` | 381..391 | 11 | `PvpGame\Cli\PvpCliContext.cpp` |
| `0x1422eab98` | 457..458 | 2 | `QualityAssurance\Cli\QaCliContext.cpp` |
| `0x1422ee6a0` | 470..1241 | 11 | `Squad\Cli\SquadCliContext.cpp` |
| `0x1422ef300` | 480..1245 | 17 | `Game\Tournament\TournamentConst.cpp` |
| `0x1425a3b38` | 1 | 1 | `?` |
| `0x1425a3ba0` | 1 | 1 | `?` |

**Notes:**
- Client **278**, the first message of the owner's map load at 17:07:08Z, is in the `DebugInfo\Cli\DiCliContext.cpp` table.
  - Its layout: two Bytes, a Dword, a Word, String(13), Dword, String(128), Words and Dwords, a Guid, String(256).
  - That reads like a client system and debug-info report sent once at connect (inferred). We send no reply.
- The id ranges show that the high ids (700–1280) in several tables belong to the same modules as their low ids.
  - So each module registers more than one block of ids.
- **Open:** which of these client messages are requests that wait for a server reply. Framing only keeps the connection alive.
  - A missing reply shows up as a UI that waits, not as a disconnect.
  - The owner-run report lists each in-world client id with its table and module, so the first unanswered request can be read off his log.

**Requests and replies:** see [request/reply](209026-request-reply.md).
- Only the auth connection has a verified request-id completion list.
- Client 278 waits on nothing.
- Game-side waits found so far are Mission state steps, already handled by the mission phase.
