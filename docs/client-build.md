# Frozen client: build 209026

Frozen 2026-10-31T19:03:33Z with `tools/freeze-client.sh build-209026`.

- **Source:** ArenaNet's public installer run on the devbox by Devbox (Devbox tasks
  145d51d8f62e and e357f5475e6e). No login was used.
- **Location:** `/home/blizz/gw2-client/build-209026/`. It is read-only, and a write attempt
  was denied.
- **Manifest:** `/home/blizz/gw2-client/build-209026.manifest.txt`. Total size
  89,311,004,697 bytes.

| File | Bytes | SHA256 | What it is |
| --- | --- | --- | --- |
| `Gw2-64.new.exe` | 43,065,736 | `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621` | **The game client, build 209026.** This is the REA target. |
| `Gw2-64.tmp` | 43,065,736 | same as `Gw2-64.new.exe` | Installer staging copy. |
| `Gw2-64.exe` | 29,340,840 | `6ff33600c3bb29041d78b4050e0ba8e1d58520236d79a8e382373f9e80746539` | Installer bootstrap. Identical to `Gw2Setup-64.exe`. |
| `Gw2.dat` | 89,165,623,112 | `c452556901bf283a1f1fb1c0056ec0bb3a0d6dba719638e135788d7a14a4f425` | Game data archive. |
| `bin64/*` | 0 each | empty | Placeholders. The client unpacks these from `Gw2.dat` on first run. |

**How the build was identified:**
- **Observed:** the installer's own crash log (`ArenaNet.crash1.log`) names
  `App: Gw2-64.tmp`, `ProgramId: 101`, `Build: 209026`.
- **Observed:** the hashes above show `Gw2-64.tmp` = `Gw2-64.new.exe`.
- The web API also reported build 209026 on 2026-10-31.

**Why the freeze was taken as-is:** the baseline is the installer's exact output. The
client normally renames `Gw2-64.new.exe` to `Gw2-64.exe` and fills `bin64/` on its first
launch. That happens only in a separate **writable working copy** inside the
`--network none` sandbox, never in this folder.

**Hypothesis, not yet checked:** the installer crashed once under Wine with
`Assertion: _beginthreadex` (`ExeThread.cpp(639)`, 48 processors reported). If the client
repeats this, try limiting the container's CPUs (`--cpuset-cpus`).

## Gw2.dat checks (our own readers, non-REA format parsing of game data)

| Check | Result |
| --- | --- |
| `gw2-dat-ls Gw2.dat 10 --sample 25` | 1,124,232 entries, 50 of 50 sample reads OK (0.37 s). Compressed entries decompress to `PF` packfiles. Uncompressed entries are `ATEX` textures exactly 4 bytes shorter than stored, which confirms the per-block 4-byte trailer and the decompressor. |
| `gw2-dat-scan` (head-only type scan, 7 min) | Top types: `PF:MODL` 372,835, `PF:ASND` 91,328, `PF:ABNK` 75,265, `ATEU` 48,570, `ATEP` 44,400, `PF:AMSP` 40,855, `DDS` 19,081, `PF:hvkC` 16,550, `strs` 11,285, `PF:mapc` 562, `PF:cntc` 64, `MZ` 23 (embedded PE binaries, likely the bin64 DLLs). |
| `gw2-dat-pf` on mapc 189364, 186397, 187611, 192711 | Every chunk decodes with the **208039 reference schema**: parm, trn v15, area v4, laye, msn, cube, havk v16, zon2 v22, octm, prp2 v21, env v78, lght v19 and others. `MapCollide` validates all cross-references on real data, e.g. Queensdale: 1,318 meshes, 212,072 triangles, 8,301 props, 4,381 zone models. |
| `gw2-dat-scan --map-rects` + `tools/match-map-rects.py` | `MapParam.rect` uses the same units as API `map_rect`. All 1,086 API maps match at least one of the 562 mapc files. Rects are often shared (e.g. Kessex Hills matches 18 files), so a rect match alone is ambiguous. |

### Queensdale (API map 15)

- Map file **192711**. File 3782379 is content-identical: same parm GUID, same size for
  every chunk.
- `MapParam.rect` = (-43008, -27648, 43008, 30720), which equals the API `map_rect`.
- Collision bounds are about (-1424.7, -1049.0, -252.6) .. (1372.2, 1066.6, 481.0).
- **Unknown:** how collision/world units relate to API map units. The ratio is about
  30× on x and about 26× on y, so it is not a single uniform scale against the rect;
  most likely the collision only covers the playable area. The packet captures will settle it.

Raw outputs (private) are in `~/re/gw2/notes/`: `mapc-rects-209026.tsv` and
`mapc-to-api-209026.tsv`.
