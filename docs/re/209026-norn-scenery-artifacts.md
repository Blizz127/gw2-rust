# Build 209026: Norn distant scenery artifact diagnostic

The sharp-edged blue planes in run 115 remain unresolved. The captured run
does **not** establish missing CDN assets or a correct retail scene. The most
useful next comparison is the same stationary view with only `lodDistance`
changed from `low` to `high`, followed, if needed, by a separate `shaders`
comparison. This isolates the distant-detail and material paths before changing
map packets or assets.

## Captured observations

Run directory:
`/home/blizz/gw2-sandbox/runs/run115-norn-mesmer-initial-equipment`.

Visual inspection of `shots/norn-mesmer-gear-ready.png` and
`shots/norn-mesmer-gear-soak.png` shows the same large blue planes above and
beyond the camp. Their hard rectangular/polygonal edges persist while the
character's idle pose changes. They are not merely a transient loading frame.
The images alone do not identify the owning mesh, material, fog volume or
texture. In particular, their appearance is not proof of an impostor asset.

The captured `cdn.log` has only two lines: a successful `/latest64/101`
request and connection close. It contains no file fetch, HTTP asset error or
retry. The loader trace in `client.out` progresses through map streaming,
model streaming, map-asset streaming and agent streaming, reaching
`STATE_READY_WAIT` at 02:05:39Z. This does not prove that every material or
archive asset is valid; it only rules out a recorded CDN file failure in this
run and shows that the loader completed these stages.

The saved `GFXSettings.Gw2-64.exe.xml` reports:

| Setting | Recorded value |
| --- | --- |
| `environment` | `low` |
| `lodDistance` | `low` |
| `shaders` | `low` |
| `textureDetail` | `low` |
| `sampling` | `native` |
| `reflections` | `none` |

`client.out` records DXVK 3.10 and llvmpipe (LLVM 19.1.7, 256 bits), with
D3D feature level 11.1. It also records `full-texture patch applied`. These
are relevant variables, not evidence that a specific renderer defect caused
the planes. The graphics XML is a saved profile, not a live read of every
renderer variable.

The prefix's `ArenaNet.log` starts with the older `m_kennel` crash dated
2026-11-04 17:08:26Z. It is not evidence of a run-115 scenery failure. The
captured console warnings concern Wine services/authentication, display
metadata, external-memory support and interface queries; no console line
identifies the blue scenery's material or asset.

## Source-supported diagnostic site

The frozen executable contains `Material '%S' is missing textures.` at
`0x141c32e38`. The bounded material path selects that string at
`0x140aa8713` or `0x140aa872a`, reaches `0x140aa898f`, puts the material name
in `r8`, sets `ecx=2`, and calls `0x1409bb390` at **`0x140aa8997`**.

A read-only breakpoint immediately before that call, conditional on
`rdx == 0x141c32e38` (adjusted for image slide), can record the bounded
material-name string in `r8`. This would provide a specific asset lead if
the missing-texture branch is actually involved. The current console and
fatal-error hooks do not establish that this branch was absent. The bounded
source trace does not establish a blue fallback color or a texture ID.

The full-texture runner patch changes the branch at `0x140a6f57a` to jump
to `0x140a6f67a`, bypassing the texture-size/mip-reduction block. The captured
instructions show that branch and its target. The patch's presence alone
does not explain rectangular scenery; retain it during the first graphics
comparison so the experiment changes one variable.

A textual `m_impostors` search led to a UI text-editing path, not a proven
scenery renderer. That lead is discarded. No map/scene field has been tied
to these visible planes in this slice.

## Next bounded experiment

1. Hold map, position, facing, camera and runner constant; capture the current
   view, set only `lodDistance=high` through the existing graphics option, and
   capture after it settles. Record whether the planes disappear, move farther
   away, or remain unchanged. The saved profile explicitly lists `high` as
   a supported value.
2. If unchanged, restore the distance setting and compare only `shaders=high`.
   A positive result isolates a graphics path; it does not yet identify the
   shader/material bug. A negative result leaves archive/material and scene
   initialization investigations open.
3. Capture the material-warning site above during a subsequent controlled
   load. An emitted name gives a concrete next archive lookup. If neither
   option comparison nor the warning hook explains the artifact, a draw-call
   capture identifying its mesh, bound textures and blend/depth state is the
   next useful boundary; guessing a missing server packet is not supported.

No game process was launched, configuration changed, or code edited by this
investigation.

## Follow-up: prepared isolated LOD comparison

The subsequent preparation task authorized one configuration change in a copy
of the finished run-115 prefix. The client source confirms that
`GFXSettings.Gw2-64.exe.xml` is also an input: filename helper
`0x141376b80` constructs `GFXSettings.%s.xml`; `0x141376c70` reads and parses
it, finds `GAMESETTINGS`, and iterates `OPTION` nodes through
`0x141377070`. That parser reads `Type`, `Name` and `Value` attributes and
dispatches the `Enum` case to `0x1413773a0`.

The option registration at `0x141372b79..0x141372b9b` binds `lodDistance`
to options-object +0xec and the four-entry table at `0x14224c150`:
`low=0`, `medium=1`, `high=2`, `ultra=3`. The import path maps the same
name to +0xec at `0x141373d80..0x141373d92`, then retrieves values in the
loop at `0x141373f60..0x141373f96`. Thus the XML value is a supported
setting input; a runtime comparison must still confirm that this prepared
value remains selected after entry.

Prepared, **not launched by this agent**:
`/home/blizz/gw2-sandbox/runs/run117-norn-mesmer-lod-high/run.sh`.
The preparation compared hashes/symlinks for 1,802 copied input and prefix
entries. Exactly two copied files differ: the XML's single
`lodDistance` value (`low` to `high`) and `run.sh`'s two run-directory
substitutions. Local.dat, all other graphics options, UI script, GDB package,
fixtures, server arguments, spawn, equipment and quest inputs match run 115.
The baseline was rehashed unchanged; no run-116 prefix was used.
`preparation.json` in run 117 retains complete before/after manifests.
Both launch and UI scripts pass `bash -n`.

Focused evidence bundle:
`~/re/gw2/notes/209026-run117-lod-high-preparation-20261105.json`.
Source captures are `ev_bce5f062104d8c6ab85e546ad2a334de2b5594d227719aa81e6821e0e376ffae`,
`ev_31ebbc329335e3a87d96a326da528f75185ed3c7034cbd97d81056aa2ae29ee0`,
and `ev_983794ac106b6cd55f1edeb297a85b6528dfac53564d2ba2f29a69dacea10e50`.
Preparation, exact diffs and unchanged baseline:
`ev_e1e4e88d621012aed73996398ccf010dc258f87506c4f0aec41fe32f723584d7`.
All four captures exited 0 with verified owned-process cleanup.

## Evidence

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Private bundle:
`~/re/gw2/notes/209026-norn-scenery-artifacts-20261105.json`.

| REA evidence | Scope |
| --- | --- |
| `ev_419321b27aee33cd675c089c5fbd5c8cb946c8427d2320c9bea7989373af15c4` | Hash-bound run logs, complete two-line CDN log and bounded filtered console excerpts. |
| `ev_bb67e19d18a529d3a067764c88c5f600f83bd2d9cbd10124147b3553850649d8` | Saved graphics options, renderer/loader console lines, older crash date and screenshot hashes. |
| `ev_d56c9d670eaa2630870242f5ba0f2ddcd78f39bbd9e0c9da3cf564e88d237059` | Hash-bound client strings, including the missing-material-textures diagnostic. |
| `ev_8755dfac2b3a2d1a7aab45b830793ce7ffc3874134b220187e66d3c85df62d6a` | Bounded material diagnostic branch/call and discarded UI impostor lead. |

Those captures exit 0 with verified cleanup of their owned process groups.
An additional full-listing reference search timed out after providing partial
references and the texture-branch disassembly
(`ev_36a2d9a3089d9006ced15f66f25364e1acd9795a8f4ca0ea84328552f11478f7`);
its process group was verified cleaned up. Its partial output is retained,
but it is not counted as a completed global reference census. Unrelated
inaccessible `/proc` entries were left untouched.

## Runtime comparison: run117

The root lane launched the prepared run117 after its release build completed.
It retained run115's packets/UI and changed only the isolated XML level-of-detail
setting. After the run, the XML still reported `lodDistance=high`.
The inspected ready screenshot still shows the large blue planes, so this
setting change does **not** resolve the artifact. No successful scenery fix
is claimed. Shader/material investigation remains separate; user-facing
character creation is now the immediate implementation priority.

Run evidence:
`ev_0312ab9cdef8062d7062d1926fdd244476429b8cc047541d6350b8e5704baac6`.
Post-run audit:
`ev_c756a3b4c96a8407f8e33a516a48a9f2b1a3ae87a46b98d9f1d9fe07c176fd34`.
Both captures exited0. The exact sandbox container was removed, executable
hashes and archive retention passed, and no captured assertion/fatal marker
was found. Ready screenshot SHA-256:
`238e1fd40010ffe109ff27ce448b00966bc75f324c04ef1060fb2ece05ab15c3`.
Soak SHA-256:
`8a4ecbf8449e5fd2d69da0d8cf68d52f9a921fb10b0fa077c3d00c6834b6a2b5`.
Private bundle: `~/re/gw2/notes/209026-norn-mesmer-run117-20261105.json`.
