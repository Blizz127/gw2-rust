# Status

Lead: Claude session `gw2-rust` (Devbox task f7f99ff037fe). Updated 2026-10-31.

## Blocker

- **No client on the devbox.** The request is in [CLIENT-NEEDED.md](CLIENT-NEEDED.md).
  Stages 1, 3 and later wait on it.
- Platform decided (Blizz, 2026-10-31): the client runs on the devbox under Wine
  (wine-10.0, no GPU, software Vulkan), firewalled.

## Stages

| # | Stage | State |
| --- | --- | --- |
| 1 | Freeze client, capture traffic, message list for our build | blocked: no client |
| 2 | Gw2.dat reader | in progress (written from ascalon MIT reference; synthetic tests only) |
| 3 | Login stub: redirect, secure login, our own key | blocked: no client |
| 4 | Character select / creation | not started |
| 5 | One zone (Queensdale, map 15) load-in and spawn | not started |
| 6 | Movement, a few NPCs/mobs, basic attacks (web API data) | web API import in progress |
| 7 | Combat, events, loot | later |

## Known facts and their sources

Keep observed, inferred and unknown separate. No REA observations exist yet, because
there is no binary.

**Observed (outside the client):**
- Live build 209026 on 2026-10-31 (`GET https://api.guildwars2.com/v2/build`).
- ascalon at commit `03fb60b` (cloned to `~/re/gw2/refs/ascalon`, MIT): message
  layouts are for build **208039**, not ours.

**Inferred from ascalon source (reference only, not yet confirmed for our build):**
- Transport is RC4 both ways. The 20-byte RC4 key is a single SHA-1 compression round
  over the folded session key (`rc4_hash`). Server→client frames are
  `u16 compressed_len, u16 raw_len, LZ4 block` and the whole frame is RC4'd.
  Client→server is RC4 only.
- The DH parameters sit in `.rdata` as a 136-byte blob starting `u32 1, u32 4`
  (`research/dh_params.py`). This is the "server key baked into the exe".
- The message schema is registered at runtime through four `register messages`
  functions (build 208039 RVAs 0xFF32C0, +0xC0, +0x180, +0x210) and dispatched at
  0xFEF760. Those RVAs will differ in our build.
- Gw2.dat: `AN\x1A` v151 header → `Mft\x1A` table → entry 2 is the file-id index;
  compressed entries carry an 8-byte header (`08 00 01 80` + u32 size) and use a
  Huffman/LZ scheme. There is a 4-byte trailer per 64 KiB block.
- Network ints: `u32` varint (7-bit groups), `u16`/`u64`/`f32` little-endian.

**Unknown:**
- Whether `-authsrv <ip>` still works on the current client.
- The current login flow (portal/SRP via launcher), and where the session key comes from.
- Where runtime-key-encrypted text keys come from.

## First REA probes when Gw2-64.exe arrives

1. Pin SHA256. `open_binary` (Ghidra provider). Snapshot to `~/re/gw2/notes/gw2-64.snapshot`.
2. `search_bytes` for the `(1, 4)` DH header in `.rdata` and confirm the 136-byte layout
   by xrefs into the handshake code.
3. `search_bytes` for the register-messages prologue (`48 89 6c 24 08 48 89 74 24 10 48 89 7c 24 18`)
   and the dispatch prologue (`40 53 55 56 48 81 ec b0 00 00 00`). xrefs + decompile
   the four variants, then read the static message-def arrays their callers pass in,
   and regenerate `protocols.xml` for our build **statically**. A runtime Frida
   agent is the labeled fallback.
4. `search_strings` for `authsrv`, `portal`, `nopatchui`, `assetsrv` and auth hostnames.
   Then trace the login flow.
5. Packfile schema: locate the schema tables that `research/packfiles.py` walks
   (type ids 1..37) for our build.

## Log

- 2026-10-31: lane accepted, CLIENT-NEEDED.md written, ascalon studied, workspace started.
