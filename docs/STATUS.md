# Status

Lead: Claude session `gw2-rust` (Devbox task f7f99ff037fe). Updated 2026-10-31.

## Blocker

- **Client copy in transit** (Devbox task 159ac9597aba, started 2026-10-31 10:09 CT).
  - Source: Blizz's **Steam** install (Steam build id 23352508, about 88 GB).
  - Destination: `/home/blizz/gw2-client/incoming/`, at about 2.7 MB/s, so 9–10 h.
  - A final rsync pass runs after he closes the game.
- **Faster source** (Devbox task 145d51d8f62e): ArenaNet's public installer is downloading
  build 209026 into `/home/blizz/gw2-client/arenanet-download` (no login, ETA ~14:00 CT).
  `incoming/` is the fallback.
- **Do not read either folder until Devbox's "complete" note names which one to freeze.**
  Then run `tools/freeze-client.sh build-209026 /home/blizz/gw2-client/<that folder>`.
- Platform decided (Blizz, 2026-10-31): the client runs on the devbox under Wine
  (wine-10.0, no GPU, software Vulkan), firewalled.

## Stages

| # | Stage | State |
| --- | --- | --- |
| 1 | Freeze client, capture traffic, message list for our build | blocked: no client |
| 2 | Gw2.dat reader | `gw2-dat` written (ascalon port, hardened); 18 synthetic tests; unverified on real data |
| 3 | Login stub: redirect, secure login, our own key | blocked: no client |
| 4 | Character select / creation | not started |
| 5 | One zone (Queensdale, map 15) load-in and spawn | Queensdale POI/waypoint/heart coordinates cached; continent↔map frame in `gw2_api::floor`. The map → in-world coordinate mapping is unknown. |
| 6 | Movement, a few NPCs/mobs, basic attacks (web API data) | `gw2-api` importer done. Build 209026 cache: maps 1086, skills 4763, specializations 81, professions 9, races 5, continents 2, Queensdale floor (47 POIs, 17 hearts, 7 hero challenges, 26 sectors) |
| 7 | Combat, events, loot | later |

## Crates (workspace tests: 58 pass, clippy clean)

| Crate | State |
| --- | --- |
| `gw2-net` | RC4 key derivation, LZ4 server frames, DH module (own server key, 136-byte param blob). Byte order and the secret/block combination are provisional. |
| `gw2-proto` | Primitive encodings (varint Dword, Guid, Point3, Address, Optional…), the `protocols.xml` loader, and `dynamic`: a schema-driven decoder/encoder. Unconfirmed kinds (String, ArrayVar*, Buffer*, CString) are rejected until `Rules` are set from REA. `gw2-schema-census <xml>`: 1440 of 1770 messages in the 208039 reference decode with confirmed kinds only. |
| `gw2-dat` | AN/MFT archive, Huffman/LZ decompressor, PF chunks. Also `pfschema`, a loader for the 208039 reference `packfiles.xml` (6 packfile types, 52 chunks, 372 versions, vendored and labeled), and `pfread`, a safe schema-driven chunk decoder (packed structs, slot-relative 4/8-byte pointers, bounded depth and elements). Arrival tools: `gw2-dat-ls --sample`, then `gw2-dat-pf <Gw2.dat> <ids>` to decode real packfiles. `gw2-dat-pf` tries each schema candidate for names the reference lists twice (`mapc`/`nm15`). `mapc::MapCollide` is a typed view of `havk` collision: triangle meshes, geometry→animation→mesh links, prop/zone placements, validated cross-references. It is the basis for ground height and mob pathing. Placement transform conventions are unknown. The navmesh (`nm15`) payload is opaque Havok bytes. |
| `gw2-login` | STS/1.0 codec (from public notes, **unverified**) + `gw2-login-observe`: logs STS messages, then hex-dumps once the stream leaves STS. Never replies. Stage 3 starts by pointing the sandboxed client at it. |
| `gw2-api` | `gw2-api-import build / fetch <ep>... / fetch-map <id>`. Cache in gitignored `data/api/<build>/`. |

## On arrival checklist (client copy)

1. `tools/freeze-client.sh build-209026 <folder>`. This moves the folder Devbox names, makes it read-only, and writes
   a SHA256 manifest. Copy the key hashes into `docs/client-build.md`.
2. `gw2-dat-ls Gw2.dat 20 --sample 25`, then `gw2-dat-pf Gw2.dat <PF ids from the sample>`: opens the archive and reads 25 compressed and
   25 uncompressed entries, printing each one's length and magic. That tests the
   block-trailer guess (probably a per-block CRC) and the decompressor on real data.
3. REA probes below.
4. Sandbox image is already built and preflighted ([SANDBOX.md](SANDBOX.md)); run `sandbox/run.sh client <build>`.
5. Run `gw2-login-observe` inside the sandbox. Launch the client with `-authsrv 127.0.0.1`
   if REA shows the flag still exists. Commit the capture summary under `docs/re/` and
   keep the raw logs in `~/re/gw2/notes/`.

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

1. `tools/rea-import.sh <frozen>/Gw2-64.exe` (run in the background). It refuses paths
   under `incoming/` or `arenanet-download/` and writable files. It runs a cold Ghidra
   import with a 1 h startup budget and writes
   `~/re/gw2/notes/gw2-64-<sha12>.{snapshot,analyze.json,import.log}`. Then open it in MCP
   with `open_binary(path, provider_id: "ghidra", snapshot_path)`.
   - `inspect_pe_imports` defaults to 64 MiB, so raise `max_file_bytes` if the exe is bigger.
   - Pipeline dry run 2026-10-31 on a read-only copy of Wine's `winver.exe`:
     - the script imported it in 34 s and wrote the snapshot
     - MCP `open_binary` with that snapshot answered `search_strings` (`ev_200eb91d…`) without re-importing
     - outputs were moved to `~/re/gw2/notes/dryrun/`
2. `search_bytes` for the `(1, 4)` DH header in `.rdata` and confirm the 136-byte layout
   by xrefs into the handshake code.
3. Message schema. The register-messages prologue (`48 89 6c 24 08 48 89 74 24 10 48 89 7c 24 18`)
   is a generic MSVC shadow-space save that matches thousands of functions, so filter
   the hits by **adjacency**: four small register functions in a row (spaced +0,
   +0xC0, +0x180, +0x210 in 208039; expect the spacing to drift).
   - Cross-check with the more specific dispatch prologue (`40 53 55 56 48 81 ec b0 00 00 00`)
     and its xrefs from the receive path (the RC4/LZ4 decode loop).
   - Fallback: scan the data sections for arrays of pointers to defs whose u16 at +0x10
     runs through sequential message ids. That is the def layout ascalon's agent reads.
   - Then decompile, read the static def arrays, and regenerate `protocols.xml`
     **statically**. A runtime Frida agent under Wine is the labeled fallback.
4. `search_strings` for `authsrv`, `portal`, `nopatchui`, `assetsrv` and auth hostnames.
   Then trace the login flow.
5. Steam provider. This is a Steam install, so the launcher normally passes `-provider Steam`
   and logs in with a Steam ticket via `steam_api64.dll`.
   - `search_strings` for `provider`, `Steam`, `Portal` and `authsrv`.
   - Inspect PE imports and `bin64/` for steam_api.
   - Find which login path (portal SRP vs Steam ticket) needs the fewest in-memory patches
     to reach our server with no Steam client running.
6. Packfile schema: locate the schema tables that `research/packfiles.py` walks
   (type ids 1..37) for our build.

## Log

- 2026-10-31: lane accepted, CLIENT-NEEDED.md written, ascalon studied, workspace started.
- 2026-10-31: Wine chosen by owner; docker `--network none` sandbox preflight passed;
  gw2-net/proto/dat/api landed; public 2012–13 protocol notes in `docs/re/`.
