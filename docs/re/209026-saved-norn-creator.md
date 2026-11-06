# Build 209026 saved Norn Mesmer creator experiment

Saved Norn character creation is **enabled live** since **2026-11-06T03:29:33Z**,
currently on `/home/blizz/gw2-rust-c2cb6fd` (deployed03:56:12Z).
The owner chose activation in the native publisher session at03:29:03Z;
root independently verified the latest configuration, backup, kit hashes and
health at04:27:55Z. See the [activation receipt](209026-saved-norn-live-activation.md).
The store was empty at03:43 and had one saved record at04:01; contents were
not read. Saved mode hides the prepared Banshee Mesmer.
Initial creation is durable; world XP, loot and heart progress remain
process-local and reset on restart. The retail starter goal is incomplete.

The [tested metadata increment](209026-texture-remapall-private-candidate.md)
resolved the previously demonstrated rendering blockers. Run142 rendered
complete male Norn Mesmer preview with no new assertion. Run144 then passed
real UI creation, durable save, visible direct world entry, combat/heart,
defeat and same-GUID waypoint return with the then-offered runner0e6ac30
and metadata900, without requiring runner63. Earlier runs126/127 established
initial save and restart relog; run134 established same-GUID progress carry.
These private checks are the activation's acceptance basis, not proof of
all retail encounter/skill behavior.

The sole publisher applied the existing v3 mode/store plan after the owner
answered its native roster question. The previous unresolved-choice hold
is historical. The mode-only rollback restores prepared mode while keeping
later-created records and metadata900; the existing kit rollback is
`/home/blizz/gw2-rust-82d8d04`. Plan
`~/re/gw2/notes/saved-norn-activation-handoff-v3-20261106.json`, SHA-256
`6b7567c330463dfc85a7d7ed4dd512d5d4c14c90b15a9601e982d3d5d5c098bf`,
remains preserved alongside the actual backup/readback receipt. Root did not
answer the native prompt or change host configuration. The separate earlier
heart preflight fixd175bd5 shipped in018cb96; it did not itself clear the
then-existing renderer hold.

The later respawn rejoin carries XP/level, configured loot drops, heart
progress and waypoint position through one process-local slot. This is separate
from durable initial-creation storage and is not progress persistence across
server restarts. Runs182/183 tested the prepared profile. Actual saved-mode
respawn must still preserve the selected GUID in the new connection; the
creation redirect in run126 and auth relog in run127 do not establish that
different transition. Run134 subsequently verified it: actual client210 and
1065 rejoined the same saved GUID with XP2500/level2, one loot drop and heart
progress1/5 preserved. The original UI waypoint click missed; one separately
recorded, ownership-checked click completed the flow. The 142-byte character
record remained unchanged. This establishes same-process carry, not durable
world progress. Run142 resolves the private preview rendering test separately;
it does not by itself enable live saved mode or establish durable world progress.
Release `d7252a8` also preserves whether the heart reward was already issued,
preventing its panel and bonus XP from repeating after a waypoint rejoin.
The focused regression reproduced the duplicate on `bc997f0` and passed with
`1e4e3e1`, cherry-picked by the integrator as `d7252a8`. Startup smoke, zero-human
deployment, 3.4-second readiness and local/remote hashes are in the release
receipt. This fix changes only volatile carry state; it adds no durable progress.

## Run144: combined flow with the current launcher runner

On 2026-11-06, `run144-norn-creator-remapall-live-runner-full-flow`
completed in 373.22 seconds with wrapper and UI exit zero. It used the exact
offered runner0e6ac30 plus the unchanged passive observer407, server cd4c3b6,
the full compatible remap metadata rooted at2130706900, a copied completed
run137 prefix and a fresh private character store. No runner63 repair was
present. The prepared live server configuration was not changed.

The real UI displayed the complete male Norn Mesmer on appearance page4/10,
submitted one124-byte client415, received1048 and continued through430/1065
to map379. Two420/412/1036 sequences bracketed combat, defeat and actual
client210 waypoint use. Kill, loot and heart-progress events each occurred
once; the rejoin carried XP2500/level2, one loot drop and heart progress1,
with heart done/rewarded both false. The final screenshot shows the visible
returned character and health388. This is the diagnostic fixture, not an
authored Great Hunt completion.

The sole154-byte character record remained unchanged, SHA-256
`267d4be2786a0ddd1b1e3a253e299359dfd0403aac3b01d36bb95c9990558e76`.
Its envelope and filename/GUID consistency passed. Both successful joins
resolve that sole saved GUID by the pinned server's required lookup from
preamble bytes48..64. This is a source-backed identity inference: no-capture
mode discards the preambles, so a direct comparison of two retained join
GUIDs is unavailable. World progress carry remains process-local.

The passive observer recorded19 valid records with no observer errors;
the completed appearance owner reached scene3/state7 with finalized model
and zero scratch state, followed by scene9. The crash log remained exactly
the staged baseline, SHA-256
`5b599267cc9c73e1c07167152f98d44250f13080e2a6852e69655bdf583f91ee`:
zero new bytes. Cleanup verified the shared container absent, both executable
hashes unchanged and the archive index at1,124,232 entries. Window22 was
returned at2026-11-06T00:13:51.618582Z.

- Runtime: `ev_21ebc75ec9cf556588812ee7c372190c02e07f1c7bc26b0aba84975e4be5707c`.
- Final safe audit: `ev_bcede89a23a79a525d76cd014f02252e73a1490a055404b53e4a5dbc69a045a4`.
- Independent cleanup/image admission: `ev_ad1c32f271bda1574f8b1c08cf97f93684effedb3409fbb4d878b320c2ad9acd`.
- Creator image, root viewed:988475 bytes, SHA-256
  `a27ff082e1e440d8357ff5b7c4aa26a1ae8839efc20949bd61e053bfb48b001d`.
- Returned-world image, root viewed:1250725 bytes, SHA-256
  `225ee9a3b8b955d81f314e73c5cdfd91f92ef6d773d4ff39b56e7101ba3acb2b`.
- Private controller bundle:
  `/home/blizz/re/gw2/notes/209026-norn-creator-run144-live-runner-full-flow-20261106.json`,
  5451060 bytes, SHA-256
  `2b90d0df1c8325bbd181d825c9cf9beba07e012e0280a24bafbb8e060057d155`.

## Implemented local behavior

`--character-store DIR` requires no-capture mode and excludes a static record.
The private directory stores each exact client415 request and a stable random
GUID. Publication uses a synced temporary file and atomic non-overwriting
hard link; directory synchronization completes publication. Thread/process
locks protect uniqueness. Exact retries return the same record, conflicting
names do not overwrite it, and corrupt records fail closed and remain intact.
Names are private, and ordinary logs expose neither names nor request contents.

Admission currently permits Norn/profession7/level1, the established scalar
bounds, one Mesmer mask answer187/188/189, and colors representable as Words.
This is a local supported profile, not complete retail content/name validation.
All submitted appearance bytes, answers and dye rows are retained unchanged.

Auth login and client25 refresh read current records from disk. Name selection
returns the selected GUID. The world resolves that GUID from mission preamble
bytes48..64, without a last-selected fallback. Explicit world settings provide
the remaining fixture state. Equipment adds the biography mask and then applies
submitted dyes; startup preflights capacity for every supported mask/dye value.

The creation scene sends1049, answers410 with1050 and accepts415 after411.
Successful durable creation yields1048;430 uses the source-supported1065 local
continuation to map379. A conflicting saved name returns1051 code0x19 and
permits a corrected-name retry. This continuation is locally tested, not
observed from a retail server. Sources:
[binding](209026-creation-mission-binding.md),
[error codes](209026-creation-error-codes.md), and
[biography equipment](209026-mesmer-biography-equipment.md).

Tests cover fragmentation/coalescing, reopen/retry/conflict, independent
processes, exact selected GUID, independently expected appearance/equipment,
malformed input and private storage. The observer suite passed82 tests with
three expected ignores before the final equipment preflight addition; six
focused saved-population tests passed after that addition. Creation-response
codec tests passed5/5. These checks do not establish a usable client creator.

## Actual client observations

Frozen client executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Both runs used the synthetic account in Docker with network=none; no live
server was changed. Both ended normally, with container removal and unchanged
frozen/working executable hashes verified. The archive retained1,124,232
entries. REA also records inaccessible host process metadata as residual
unknowns; the container's explicit cleanup check is the runtime boundary.

- Run119 opened the real Select Race scene. Server1049, client410, server1050
  and client411 completed; client416 followed. No415 was submitted and no
  character was saved. Preview/navigation controls were absent at1280x720.
- Run120 changed the private display/profile to1280x900. Double-clicking the
  visible race, gender and profession selections reached Norn/Mesmer appearance.
  The 3D preview and navigation controls remained absent. Enter and a diagnostic
  click near the lower-right region did not advance. Four416 reports were
  observed, with no415 or saved character. The unchanged appearance screenshot
  hash was `f35ac13b7fc2031e71e86a38f09dd407727c9111934ed44cf4c4d6ad568eec34`.

At that stage, the taller display did not resolve the issue. Source checks identify416 as a
scene report, not a proven response wait. Navigation creation/layout has no
identified server-readiness or preview gate; model construction is local.
Do not add an invented acknowledgment. The next bounded diagnostic is a
passive inspection of the existing scene/model/UI state, using the anchors in
[client416](209026-creation-client416.md) and
[navigation](209026-creation-navigation.md).

Run125 subsequently identified the cause: the 1280×900 game child was inside
an undersized 1272×866 X parent. The footer at Y868–900 was clipped. Resizing
that parent restored Back, Next and Skip To End. The recorded click reached
Next's owner handler, and `creator-after-next.png` shows the Mesmer mask
biography page, 5/10. No server packet or client UI-handler change was needed.
The distinct missing model preview now has an observed Gr texture readiness
failure on earlier queued models; see the [model pipeline](209026-creation-model-pipeline.md).

Run125 exited zero, UI status was zero, the owned container was removed, and
both executable hashes were unchanged. Runtime evidence is
`ev_bf4614290cee173087c24c54f7b06007f19d5bfbbd4891cf04e0c97df4b3921d`;
the explicit resize is
`ev_3afdd740d1c0453b757f6db23dc5018978e325fb444523f9ae63209e4ab2aec4`.
Screenshot/intervention audit
`ev_9f217a62edb22db3399a61add9e987a7665bc8f21eddfcfc7675b67659769053`
binds the resulting 1280×900 mask-page image to SHA-256
`520fb9134b9ff10834314d2dedeb8557e00003eb85bafd590892f835bff6a01e`.
Complete private evidence: `/home/blizz/re/gw2/notes/209026-norn-creator-run125-20261105.json`.

## Run126: actual creation and world transfer

The dynamic X-parent helper repeated the successful resize without fixed
window IDs. From Norn/male/Mesmer appearance, Next opened the mask biography
and Skip To End opened the character-name page. A synthetic name was entered
through the visible UI. The client sent its actual 116-byte415; the server
persisted one 146-byte, mode0600 record and returned1048. Client430 followed,
then the1065 continuation to map379. The new world connection reached420
MapLoaded, population,412 AgentsLoaded and1036 BeginPlaying.

The saved record has SHA-256
`74a17e292ca395074e5ad6555628f27b44da9b0a6ac4f98602e4c13b8703e52d`.
The envelope build is209026, its declared request length matches, and the
same record is retained privately for process-restart validation. No name,
GUID or request contents are copied into this note.

The final screenshot shows the world HUD, health388 and skills; the player
model is invisible. This is not a complete visual acceptance result or a
retail tutorial claim. The UI script exited5 because its bounded120-second
wait for the controller's completion marker expired; the controller did not
mark that wait complete. The outer wrapper exited0 after cleanup and hash
checks, so wrapper success must not be mistaken for a full UI-script pass.
No relog occurred in this run.

Evidence:

- Submission and persisted-record count: `ev_763e31e0a0f42cd3e661caefc6e7d6f0fe6679237dac40693740cec685f17247`.
- Record envelope and intermediate world loading: `ev_3a1b0b5bf2ba97ff549c9220f151252201f82f21a9eeeb9e06e2088a55ddeeb4`.
- Completed runtime, container cleanup and unchanged executable hashes: `ev_90784c5436259666c2d1fcc0e3297edbfdf88dbb49d769f4dddd88f9dbed2593`.
- Final checkpoints/UI status/image hash: `ev_d459456b99e81d4c085676ae58201dda28ece80b10041ed98d63cf55b34adcca`.

`shots/shot-015.png` SHA-256:
`6d9c419a50cd04a402383cc9c92d2518eea9fe68fafe2c6ea24786b36c132485`.
Private root bundle: `/home/blizz/re/gw2/notes/209026-norn-creator-run126-20261105.json`.
The live release's mode switch is separate from this offline evidence; no
publication is implied by these runtime results.

## Run127: process-restart relog

A fresh client and fresh auth/world processes reopened the exact run126
store through a separate mount. No record was copied or synthesized. The
saved level1 Norn Mesmer appeared in character select; selecting its card
entered map379 through420,412 and1036. The UI sent a bounded walking input
and completed with status0. The final screenshot shows the player model
visibly. Thus a client restart/relog is an observed workaround for run126's
invisible direct-entry avatar; the underlying texture-loading cause remains
under investigation.

The record count remained1 and the146-byte record's SHA-256 stayed
`74a17e292ca395074e5ad6555628f27b44da9b0a6ac4f98602e4c13b8703e52d`.
Wrapper and UI both exited0. Container removal, unchanged client executable
hashes and archive entry count1,124,232 were verified. This establishes
creation-profile persistence and selected-GUID world entry, not persistence
of later combat, movement or quest progress.

Runtime evidence: `ev_73d5ad0c0ac17b3cc1f7edaed5b37aff3757d6e1efea3679ddf4ba230e830bf4`.
Final UI/record/image audit: `ev_512eece42f8e2d651417b8b7d3b4ab5108c97e359df58d59996994fb5bac1d92`.
Final `saved-relog-after-walk.png` SHA-256:
`efc18d255e009181fe356fe196212851c3e54589e8f417809b6b1a978df9a3f1`.
Private bundle: `/home/blizz/re/gw2/notes/209026-norn-saved-relog-run127-20261105.json`.

## Run228: saved stage0 entry, incomplete tutorial route

On 2026-11-06, `run228-saved-norn-hunt` used the exact9326c08-vendor
binary/profile, the unchanged synthetic record created by run144, and normal
hunt stage0. It preserved current actor positions and used no prepared
character fallback, stage override, packet injection or owner record.
The record remained154 bytes, SHA-256
`267d4be2786a0ddd1b1e3a253e299359dfd0403aac3b01d36bb95c9990558e76`.

The client entered map379 and interacted with Thora. The server logs one
BeginPlaying, initial wallet0 and step1 completion. No trophy kill/step2,
later story step, reward claim or map28 transfer was observed. The admitted
first-unmet-step screenshot shows the character among nearby friendly NPCs,
the Greet prompt and distant red labels; no selected hostile is visible.
The image hash is
`f733a62fd7c7bce9dfec5431fbb812803967dcb68dae7782f248d834d1f079f8`.

The single designated native controller claimed the paused UI at08:17:05Z
and captured an image. Its full evidence output caused a context compaction
during the short control window. It issued no movement, attack, click or
release after claiming. The UI and wrapper correctly exited12 on controller
timeout. This is a failed test-harness handover, not a successful full route
or proof of a gameplay defect. A subsequent attempt must retain full evidence
privately and return only compact status during the active control window.
No retry was performed under this grant.

Cleanup, console privacy, unchanged frozen/working executable hashes and
container absence were verified. Window34 was returned at
**08:19:24.565915Z** with the failed UI outcome preserved. Cleanup status
`complete` refers only to resource return. The final manifest remains
`cd7dfcbf71f056a82959bd7f32839932134eab3e5d8c3a9ced56dc34017f74e3`.
The return helper's no-signal check was corrected before use to accept REA's
observed signal0 as well as null; the original helper was retained. No
signal-terminated capture is accepted by that change.

Private REA receipts under `~/re/gw2/notes/`:

| Receipt | Evidence |
| --- | --- |
| `209026-run228-window34-verification-20261106.json` | `ev_5a7c44ebdeb3eb8f185223d5de7440619b114c67211488ee6b40dc1a2e13e922` |
| `209026-run228-saved-norn-hunt-launch-20261106.json` | `ev_a83daf2a6b2695f52a37fe4cbe48feb7d51f88180bf759607a9491a2af2ef600` |
| `209026-run228-saved-norn-hunt-readback-20261106.json` | `ev_20d24de545c1353977dcd171692582ee434c6ea07b4fe263ca9d5f66fd6cccbc` |
| `run228-native-controller-bind-20261106-evidence.json` | `ev_544930014bee3e8cd0f08c9d55b57fd11cd98245dbad8cd24c185198bc2efe16` |
| `209026-run228-window34-return-20261106.json` | `ev_5adc0312b7445129a50a82c908be045f7eefa23eba2b3f8844ca8353250ae628` |

The launch and reader retain unrelated inaccessible process probes; those
processes were untouched. This run is distinct from the publisher's prepared
`run228-vendor-drag`, which exercised a sale.

## Run229: native control and targeting boundary

The fresh `run229-saved-norn-hunt` retains the same9326c08-vendor build,
profile, stage0, actor positions and unchanged synthetic144 record. It
corrects run228's controller-delivery failure: full REA evidence is saved
privately before compact output. Login starts after30 seconds, supported by
the previous ready-login image; the35-second world-rendering cushion remains.
The unsuccessful blind attack rounds are removed. The total405-second
runtime ceiling is unchanged, with a110-second native-input cap and an
earlier release target that reserves continuation time.

The controller claimed the paused UI at08:32:44Z, sent11 native navigation,
targeting or click actions, and released normally with14 helper seconds
remaining. Sixteen complete evidence records and four admitted screenshots
are retained. The binding capture omitted one duplicate rendered frame,
while retaining all886 raw bytes,10 raw frames and every normalized frame.

Gameplay still stops at step2. The only selected target is client227's
agent45; the actual10170/slot5 request also targets45. The staged source
maps45 to the sixth neutral ambient actor, species6045, matching the visible
Juvenile Raven. Tab produces no new target message. The hostile-only damage
guard therefore has no eligible target; this does not establish a combat
implementation defect.

Fourteen sparse movement samples show the character move approximately from
`[-3200,-25600,-3742]` to `[-2213,-25313,-3759]`, then back to
`[-2485,-25310,-3745]`. Requested D holds were150ms and500ms. The first view
after movement does not establish the intended heading; continued forward
input brought the view against the hillside. No yaw, effective keybinding,
camera transform or per-key timestamped movement evidence proves whether D
turned, strafed or was ignored. The exact current server `Moved` branch
generates heart/POI effects rather than a rotation/position correction.
No collision-query failure is inferred from the image alone.

The next navigation attempt needs an isolated before/after direction check
and a verified hostile227 selection before attacking. The original positions
must remain intact; moving enemies closer would test a different route.
No further run or resource grant was created by this finding.

UI/wrapper exit13 records the incomplete sequence `[1]`. No reward claim,
payout or map28 transfer occurred. Cleanup, privacy, unchanged frozen/working
executable hashes and original synthetic record passed; window35 was returned
**08:35:16.924249Z**. This is a successful controller handover and resource
return, not a completed tutorial.

Private evidence under `~/re/gw2/notes/`:

| Receipt | Evidence |
| --- | --- |
| `209026-run229-window35-verification-20261106.json` | `ev_98d384a6456d9211107458094bc8a3f2b23045ddd86f7ebc0217e37254044330` |
| `209026-run229-saved-norn-hunt-launch-20261106.json` | `ev_69eec34c55b725b6e1ed1d9bed16a9f14f0d210e4aa49f637b6ca4740f6134e3` |
| `209026-run229-saved-norn-hunt-readback-20261106.json` | `ev_96dcdd1acde9b6fe6966e0ceeedcb1347905bbf28f87d4bbc75ecfb6e9584022` |
| `209026-run229-window35-return-20261106.json` | `ev_2bbc09925769ff34a4775177bc536a637274ab8c4f70924ffad1dc16f20e63f5` |
| `run229-navigation-evidence-20261106.json` | `ev_97ff494bd940748a609dc1894ef175cbca60b9509e763ae2f2ed9a01ca4b225e` |

Controller receipt: `run229-native-controller-outcome-20261106.json`, SHA-256
`18fa6e771b91d6303a7048a8b6182a7f8d2be431910b2a8b55e2858fcd4aef4c`.
Navigation finding: `run229-navigation-findings-20261106.json`, SHA-256
`80c5fe8080420466889c5a4b735225621427df07b066317fd7e30ea75a0dbbff`.
No owner store, archive content, source fixture placement or live service
was changed by the controller. Inaccessible unrelated process probes remain
unverified and were untouched.

## Run230: isolated input observation, no hostile attack

`run230-saved-norn-heading` used source
`a223a90f4e929a3469e93eac47503f818317abaa`, game SHA-256
`d1d09f9bf4c91c547ccfca37a61719dd0394d609b22fe662553b2fe4aef89a6e`,
and the unchanged vendor profile
`d564afee8cacaf1a35aa2c92b91670ed4d173f69aed99d11738533e4ab018f0f`.
The 1,836-input manifest is
`8ec5c1115dbbc61570594c45e4f2630f9860196e39ec9634a224f2c6c12aa1ab`.
Normal saved stage0, actor positions and the synthetic144 record were preserved;
there was no owner-store access or actor relocation.

The sole controller claimed at08:58:31Z. The initial view already had no open
dialogue or selected target, so no unnecessary Escape or world click was sent.
It captured the stationary view, held D for500ms, and captured the view again
before any other input. The first sparse movement report was
`[-3200,-25607,-3742]`, compared with configured spawn
`[-3200,-25600,-3742]`; nearby actors shifted left in the view while the player
had a lateral walking pose. This is consistent with lateral movement, not
proof of an effective keybinding or a measured yaw. The seven-unit difference
is the first sparse report, **not total displacement**. Trees and a large
building continued appearing between images; asset streaming changes must
not be attributed to the D input or camera rotation.

One Tab input produced no client227 target. There was no visible selected
hostile, and no attack was sent. Progress remained `[1]`, with step2 unmet.
The controller released all inputs at08:59:22.860–23.379Z, with37 helper seconds
remaining, before its80-second release target. No W/A/S/Q/E chain, packets,
memory writes, server-state edits or resource controls were used. Seven
complete REA records were saved privately before compact output. The binder
retained all888 raw bytes,10 raw frames and every normalized frame; only one
rendered frame was omitted.

The image pair is retained privately as `controller-heading-before.png`
(SHA-256 `c3a52e73cd209e09846bb5223777508c0b448448ae563b387bdf6ed08494f33e`)
and `controller-heading-after-d.png`
(`6a9ea2369097f0153639b296170ef0db603731b3ec4691869ba95001ed46322e`).
Root separately checked those hashes and viewed both images.

Wrapper/UI exit0 means the bounded diagnostic capture completed, **not a
successful hostile attack or full tutorial**. Root verified container absence,
console privacy, both client executable hashes unchanged
(`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`),
and the original154-byte synthetic record unchanged
(`267d4be2786a0ddd1b1e3a253e299359dfd0403aac3b01d36bb95c9990558e76`).
Window36 was returned at **09:00:04.871304Z**. No payout or map28 transfer was
attempted. Cleanup and resource return remain separate from gameplay coverage.

Private receipts under `~/re/gw2/notes/`:

| Receipt | Evidence |
| --- | --- |
| `209026-run230-window36-verification-20261106.json` | `ev_03df814212f6bc7993e7dbd08c932e30bd73591d4e75f60ca17f0849262446cc` |
| `209026-run230-saved-norn-heading-launch-20261106.json` | `ev_e55304ccdc1284b2b0ceb21839485091b9e84e153a094ee3c1379cbf016fdc66` |
| `209026-run230-saved-norn-heading-readback-20261106.json` | `ev_83de2d6c2034073fdbc1306d58d17684f68930cc743b1be1a46c71329734c086` |
| `209026-run230-window36-return-20261106.json` | `ev_e2b04e21abe868b33c02b5b1bb965e03a2ee3d8b1e618c1dbce4577f90543eda` |

`run230-native-controller-outcome-20261106.json` pins all controller captures
and images, SHA-256
`4c3f6def2cfbb625223394706c37f87c70496e1296a57193c2c80a4515d2065e`.
The staging receipt is `run230-saved-norn-heading-staging-receipt-20261106.json`,
SHA-256 `22bf2b0af1285fe7321cc5ca795b6fa17623e56efbccf8bff1f3e473b3e0fe45`.

## Run234: General Options reached, Controls still unobserved

`run234-norn-controls` used exact7c26f45, its tested Ether Feast profile,
the unchanged154-byte native-created synthetic record, normalstage0 and original
positions. The sole publisher granted window37 at09:34:17Z; root verified it
at09:34:55Z. The single controller sent only observed menu clicks and Escape.
It reached the Game Menu and General Options. Root separately hash-admitted and
viewed those images: General Options displays `[F11]`, a mouse-shaped category
icon and FPS2. The shortcut was not invoked and the icon's category was not
opened. No Controls label, movement binding or camera behavior was established.
Capture labels such as `controller-controls` describe intended observations;
that image actually shows the Game Menu.

The controller released at09:40:04.416–09:40:05.050Z, with3 helper seconds left.
It stayed inside the90-second hard limit but **missed the80-second target**.
No preference, binding, movement or attack input was sent. Twelve controller
captures were retained. The binder has complete883-byte raw/normalized output
and10 raw frames; one rendered frame is omitted and process sampling reaches
its40-entry limit. Other controller captures are untruncated. Full Evidence was
saved privately before compact output or image delivery.

Wrapper/UI exit0 establishes a completed bounded observation only. Story remains
at step1. Root verified cleanup, shared-container absence, privacy, unchanged
frozen/working executable hashes and the original synthetic record, then returned
window37 at **09:40:45.981259Z**. Unrelated processes with denied environment
reads were untouched. The next probe can use the now-observed F11 label and
category layout, with fewer action/capture round trips and an earlier navigation
cutoff. A fresh resource window is required for any new run.

Private root outcome: `run234-norn-controls-outcome-20261106.json`, SHA-256
`12b26825d23c17e26a80b07a521e85246fb0613a0df7f0de4799380e148030e2`.
Controller outcome: `run234-controls-controller-outcome-20261106.json`, SHA-256
`6de430600e39db3e49a466a0082919e7128bcca5d7e49de93f3189ea6a90e9a8`.

| Retained REA | Evidence |
| --- | --- |
| `209026-run234-window37-verification-20261106.json` | `ev_0ab4aea6fa7bb3456ab5244faa792df11f1d02d8673708afcdde40ee73b005d1` |
| `209026-run234-norn-controls-launch-20261106.json` | `ev_7491aa3963848dece6e434a9b4eb3b002b206c6500a47cfede7625b1b6e49640` |
| `209026-run234-norn-controls-readback-20261106.json` | `ev_04be95ad71b0db1b4a431d5fb65bfc38f00d80e24ea7afd99c2047d3603c632e` |
| `209026-run234-window37-return-20261106.json` | `ev_1c2fbbdcf53338f86cd85c481667f11467572d6d03ba98dd14552e378186e1b4` |

## Run235: displayed movement and turn bindings established

`run235-norn-controls-direct` retains the exact7c26f45 game, tested profile,
normalstage0, original positions and unchanged native-created synthetic record.
The publisher granted window38 at09:43:52Z; root verified it09:47:05Z. The
controller used the displayed F11 shortcut, clicked the observed category tab,
then scrolled one notch inside the list to move a tooltip away. Each menu action
included a two-second render pause and screenshot in the same capture.

The resulting page is explicitly titled **Control Options**. Root independently
checked the admitted image hash and viewed these rows:

| Action | Primary | Secondary |
| --- | --- | --- |
| Move Forward | W | Up Arrow |
| Move Backward | S | Down Arrow |
| Strafe Left | A | Left Arrow |
| Strafe Right | D | Right Arrow |
| Turn Left | Q | blank |
| Turn Right | E | blank |
| Dodge | V | blank |
| Autorun | R | Num Lock |
| Jump / Swim Up / Fly Up | Space | blank |
| Weapon Skill 1 | 1 | blank |
| Healing Skill | 6 | blank |

These are this fixture's **displayed bindings**, not universal defaults or
executed turn/camera behavior. They explain why D was unsuitable for the earlier
turn attempt. The camera section was not inspected. No binding editor,
preference change, gameplay movement or attack was used. Story remains at step1.

The controller released09:51:23.482Z, about56 seconds after claim, inside the
80-second target and90-second hard limit. Five controller captures are complete
and untruncated. Root's readback verified wrapper/UI exit0, cleanup, console
privacy, unchanged executable hashes and original154-byte record. Window38
returned at **09:52:33.524904Z**. The full hunt remains unproven; its next run can
use Q/E after observing their actual response and must verify the complete
objective/reward/transfer sequence separately.

Private root outcome: `run235-norn-controls-direct-outcome-20261106.json`, SHA-256
`701d7281ad88859b28f0d90b567cbdda0e500116cf50b579107360e1c870b3a9`.
Controller outcome: `run235-controls-controller-outcome-20261106.json`, SHA-256
`02f94a22c5bcd09aac6dd9887a6c7f429f1981c8c0ed53312de39d6e415d6406`.
Readback: `209026-run235-norn-controls-direct-readback-20261106.json`, SHA-256
`188e5c78b0c104c819952c5ce50b6acb5d810b91c18e919f7e0ad60b193f3c2d`,
REA `ev_e9d5bd296d134f6df749321aac24bb5fe6ee1be1813c2d5006c5d5eb635d7e41`.
Return: `209026-run235-window38-return-20261106.json`, REA
`ev_eb6c1c825eaf3e5cd60697b0ceeeae7af5d53396968c493cdff6d2abad55fbbe`.
The admitted `controller-controls-bindings-2.png` has SHA-256
`be4acce1c9bc95d3f3e3f10f869998fda34aea273f8a7a23046cd30400cc5400`.

## Run236: turning observed, full hunt route fails

Run236 uses exact7c26f45, the original154-byte synthetic saved record, normal
stage0 and unchanged actor positions. An85-second rendering allowance precedes
the first controller image, which shows the detailed hall. E500ms visibly turns
the view right; Q250ms later turns it back left. Three W1500ms holds approach
and enter the hall/cooking room. The view faces the oven and back wall; no
onward passage or hostile is demonstrated. One S1500ms hold backs toward the
entrance. Root independently hash-admitted and viewed the initial, E-turn,
Q-alignment and interior images.

Only Thora/step1 completes. There is no target227, attack/heal input, hostile
death, payout, reward claim or map28 transfer. UI and launch exit13 preserve
the failed full-hunt predicate at step2. Successful controller helper exits
do not mean the hunt passed. The failure concerns the selected route; it does
not establish broken movement, invalid placement or globally unreachable
hostiles. The exterior route remains untested.

Controller release completes at10:11:57.878Z, within the210-second target and
240-second hard limit. Several inter-capture gaps are20–46seconds and include
image interpretation, planning and tool turns; they cannot all be attributed
to rendering. The source logs movement only when `moves_seen % 20 == 1`
(`game_observe/mission.rs:3005` at7c26f45). Thus the last recorded position
`[-2512,-26213,-3757]` is a sparse sample, not the exact final pose or yaw.

The existing REA CLI captured the foreground launch under a730-second scenario
through a yielded exec session. No host configuration change or detached
client was required. Complete Evidence was saved privately before compact
output. Strict readback and window return pass: the sandbox is absent, frozen
and working PE hashes and original synthetic record are unchanged, and console
privacy is verified. Window39 returned **2026-11-06T10:16:44.368582Z**. No
preferences, stage, owner store, server implementation or release changed.

Launch Evidence is untruncated with owned-process-group cleanup verified;
45 unrelated process-environment permission failures remain recorded and
untouched. Readback reports overall truncation because one rendered terminal
frame has unknown size, while all5336 raw terminal bytes and both normalized
frames are retained. Its parsed JSON and fresh strict readback agree. The first
root summary validator rejected the blanket truncation flag before any writes;
the corrected validator checks this narrower completeness claim without
repeating the runtime or readback.

Private root outcome: `run236-saved-norn-hunt-turn-outcome-20261106.json`, SHA-256
`3aaf7c5dbe693b535908c67c2a2d044a9fa7c082b2b5152bb210cbdf5cce6a00`.
Controller outcome: `run236-hunt-controller-outcome-20261106.json`, SHA-256
`e2ce913742876e99705117f1302cad2205325b456258e58a5fd3cb31d1b9e0bd`.
Launch: `209026-run236-saved-norn-hunt-turn-launch-20261106.json`, REA
`ev_f4f2011f6d644ed23b7b2210564e395bc94804d976be17f15ce042bc3eb5af4b`.
Readback: `209026-run236-saved-norn-hunt-turn-readback-20261106.json`, SHA-256
`450a972d14fe6a43b6236d27f27cf9cbae4a63e639632487ea88759eadab826b`, REA
`ev_542867b857b12275bc47b597de4a1a23584de7bd8274a8b14435e650c789468c`.
Return: `209026-run236-window39-return-20261106.json`, SHA-256
`19da2ad1b40a0ef76ef6b9b8a7230cccdeebf08b998435e77a16232a5c7ae1d7`, REA
`ev_eb1bbadab713ff74398483e46143f54f43897ad741da8e881f1a25e38249e14f`.

## Run237: exterior route reaches a narrow strip, hunt still fails

Run237 keeps exact7c26f45, normal stage0, actor positions, profile and the
original154-byte synthetic record. The controller uses the right exterior
route and bounded W/S holds up to3000ms, with other holds capped at1500ms.
E750 leaves the hall on the left. Forward motion crosses the apron and then
follows the outside wall; one approach obscures the camera in wall geometry,
followed by backward/strafe corrections. The later images show a narrow
leaf-covered strip beside the wall and trees/slope to the right. The final
W750 produces nearly unchanged scenery and no new sparse movement sample.
Root independently hash-admitted and viewed those four route checkpoints.

The last logged position is `[-2408,-27319,-3629]`, ordinal321. Camera and
foliage obscure the avatar, and movement is sampled every20 messages, so the
specific cause of the stop and exact final pose remain unknown. This does not
prove globally unreachable hostiles or a server movement defect. No hostile
was identified or selected; no attack, heal, kill, payout or transfer occurred.
Only step1 completed. Launch/UI exit13 retains the failed full-hunt predicate.

The original binder invocation completed and saved its full Evidence after
its execution handle was lost from compacted controller context. The controller
recovered that receipt and admitted the first image before input; no binder or
claim was repeated. Thirteen complete REA captures retain twelve admitted
images and eleven bounded holds. Physical holds total14.9 seconds; action
captures total52.55 seconds. Release completed at **10:36:03.229Z**,182.701
seconds after the first image, with56 seconds left before the hard deadline.

Strict readback, privacy, unchanged PE and synthetic-record checks pass. The
sandbox is absent, and window40 returned **2026-11-06T10:37:33.515754Z**.
Launch/readback/return Evidence is untruncated and owned cleanup is verified;
46 launch and one readback process-environment permission failures remain
recorded and untouched. The next investigation needs an overview or geometric
route evidence; this run does not justify another blind movement sequence.

Private root outcome: `run237-saved-norn-exterior-hunt-outcome-20261106.json`,
SHA-256 `0c98e19e3bb2cac8516593b6a3980cee9a2f359afe158af7cb4c84864f188adf`.
Controller: `run237-exterior-hunt-controller-outcome-20261106.json`, SHA-256
`833faec2980dbf2b37b928561a8db2a3ef25750a36e9847de199d622f7170139`.
Launch REA: `ev_d43ce632b5790b61c7e41aa6b6f4ce71a93a9099142edff312beae740adf3289`.
Readback: `209026-run237-saved-norn-exterior-hunt-readback-20261106.json`,
SHA-256 `f51ca145c7987accb99ed2e53a792357e310484c1a5b7f3b7597d70e9935eeeb`,
REA `ev_2ab566c8a46e28b804e3d0836171f33d75fcf342f0788c37c48396c9ccb90fe1`.
Return: `209026-run237-window40-return-20261106.json`, SHA-256
`e573c9e4f0348c08b0e208a9e23593e267623df18943f59a845ce5bd02446186`,
REA `ev_bb91922b18ed850d14154520074373fe88a56b4e5cac262ea63ef02c320f0547`.

## Run238: native map overview observed, route still unresolved

Run238 uses exact e624c8b, the unchanged original synthetic 144 record, normal
stage 0 and existing placements. Fresh window 41 authorizes one stationary map
observation. There is no Thora interaction or movement after roster bootstrap.
The sole controller opens M, uses two wheel-down ticks at an observed map
canvas coordinate, then closes M. The first close image is still transitioning;
one passive follow-up confirms the native world returned. Controller release
at 11:06:28.717Z meets the 75-second target; all six controller captures retain
complete, untruncated Evidence before five hash-admitted images.

Root independently viewed map-open, map-wide and close-followup. A player
marker and broad painted terrain/water are visible. The wider view includes
the neighboring label “Hoelbrak (Lodge Level)”; that text does not establish
the current map's identity. No readable hall footprint, road connection,
hostile marker, numeric scale or verified north direction was obtained.
The overview therefore supplies no usable route around the hall. The first
and final world images differ in loaded scenery; map toggling is not proven
to cause that change.

Launch/UI 0 means this observation sequence completed. Strict readback records
no quest steps, targeting, SkillBegin, NPC damage, story interaction or transfer.
Container absence, privacy, frozen/working PE hashes and the original synthetic
record are verified. Window41 returned at 11:07:01.604977Z. This is no full-hunt
pass or new release, and no further use of that window is authorized.

Private root outcome:`run238-map379-overview-outcome-20261106.json`, SHA-256
`e19ed10f6bed4de4833e9a98089f7665762e8860badd05ad4990ad91bfcce12b`.
Controller:`run238-map379-controller-outcome-20261106.json`, SHA-256
`bd0b2ad8b48cd6382f05ef2c1393ad508534c2031e6f8f65624400642a9edf5c`.
Launch:`209026-run238-map379-overview-launch-20261106.json`, SHA-256
`20ce08a7041ada797b815def728bc29db44cb2152bea2b0c2ceadcd688934b83`,
REA`ev_a657f0aacb91c18fafad8e395c92f77834da326aaf74ee16bdb8704fd79453c5`.
Readback:`209026-run238-map379-overview-readback-20261106.json`, SHA-256
`eafbd1d5fe2c51c979470119676635c9e1e9ebe8c022a9e69e7e1fdc371a5366`,
REA`ev_0538ea02ef640d421438d10a1c814f9ac2f9ce176dc02c6d50c0ab1dee904a9a`.
Return:`209026-run238-window 41-return-20261106.json`, SHA-256
`fa1777a0ef98372527f6bee4da42fce9ebd210b288f2b6a7847fd184dc60c920`,
REA`ev_87201eb14a7d077e8aa7aa8650a4086e66dc819de7b507c89fb5ddb032b9dd22`.
Those three captures are untruncated and have verified owned cleanup; unrelated
inaccessible process environments remain untouched.

## Run242: normal spawn observed; initial position sample missing

Run242 uses exact live `403ac37` with diagnostic movement logging every report,
the unchanged original154-byte synthetic144 record, fresh145 prefix and normal
stage0. Root reviewed the admitted first and passive-follow-up images: the
character is visible beside the hall, with health388 and no selected target.
Neither image supplies a measured position or camera bearing.

No movement report arrived while stationary. The controller's required initial
position anchor therefore prevented its first calibration. It claimed at
12:12:58.475026Z and released incomplete at12:13:22.794203Z, after24.319s,
without movement, turn, targeting, skill, interaction or story input. Launch/UI
exit13 is incomplete observation; this supplies no evidence of an obstructed
route or a movement failure. Source geometry still does not prove walkability.

Root returned window44 at12:14:15.810309Z after verifying container absence,
unchanged frozen/working executable hashes, original synthetic record and
console privacy. Full launch, readback and return captures have verified owned
cleanup; unrelated inaccessible process observations were retained and untouched.
The next fresh fixture will allow one explicitly authorized movement of at most
100ms after visual review of clear normal spawn. Its configured spawn will be
labelled an inferred reference; further movement must await a fresh actual
position sample. No active242 helper or returned lease was reused.

Private evidence under `~/re/gw2/notes/`:

- `209026-run242-norn-upper-route-launch-20261106.json`, SHA-256
  `3d77fa344af8445c1ae07cea7eddbb48274ff439364be4a5fb07267454222e48`,
  REA `ev_30bfeafd9290ee2304e8921075ff761446d29c6d9cb0efb79d623e85d086d701`.
- `209026-run242-norn-upper-route-readback-20261106.json`, SHA-256
  `54eb18fed07f4151b37cfe932d39d83da57a51fb4e91e56d58ab04d123622d80`,
  REA `ev_415c7e037d21f774be4cdd9186c2f79a20adc934d65d96edb1b6f4683af1bf04`.
- `209026-run242-norn-upper-route-return-20261106.json`, SHA-256
  `456142b99e22728ff5bed47cf70c105e199177bb825ed8c2f767915f5d783836`,
  REA `ev_74a32b620c5ff42c9728042ce31652112520b26c0df21eac00a043e3f65d976f`.
- Root outcome `run242-norn-upper-route-outcome-20261106.json`, SHA-256
  `87403e5899b8064d124247989e041259a9169c528f2bafbc49c44ed8a257248b`.

## Run244: one initial input sent; no movement sample observed

Fresh244 retains exact403ac37/N1, original synthetic144, normalstage0 and a
fresh145 prefix. Its reviewed controller adds one explicitly permitted50–100ms
translation after visual spawn attestation. The configured spawn is an inferred
reference, never substituted for an actual position. Seventeen controller and
six reader checks, both startup checks and all1837 manifest rows passed.

After the initially incomplete scene rendered, the sole controller sent one
**S100ms** probe. Immediate and single passive follow-up readbacks both contain
zero movement rows and no position/ordinal. Root reviewed the settled, post-probe
and passive images and identifies no clear displacement. Successful xdotool
execution does not prove application delivery; duration, focus, bindings and
obstruction remain unproven causes. No second movement, turn or gameplay input
was sent. Controller claim12:22:57.238183Z, probe marker12:23:55.238464Z,
release12:24:21.368640Z:84.130s of control, launch/UI13 incomplete.

Window45 returned12:25:30.662980Z with container absence, unchanged frozen/working
PE and original record, and privacy verified. The next dependency is a bounded
comparison with retained successful movement inputs and a concrete delivery
observation, before another route attempt. No244 or returned lease is reused.

Private launch/readback/return records under `~/re/gw2/notes/`:

- `209026-run244-norn-upper-route-launch-20261106.json`, SHA-256
  `47f9acc93b79fed73274ab2125ea3cd2ac789f7ab1236ef87029c2587ca07cbe`,
  REA `ev_de00ea738bda18b020f25e549f65a94119168d0f3dd5f74a825e9a4fc749ba89`.
- `209026-run244-norn-upper-route-readback-20261106.json`, SHA-256
  `7b8c6924a40a697747352373be0789b759fa542733ef503b3763b02fa5bfd534`,
  REA `ev_d476296ca35ea6d35b867473e1844b597ea6e6127f20bfc74a26809085e68da0`.
- `209026-run244-norn-upper-route-return-20261106.json`, SHA-256
  `53e9656c1bfa178b05f499d0afbad766b15be4e572880bea8058b431fa602baf`,
  REA `ev_4a9f0a927e8f587da103cb33ae910d2d94297918004443a7678f2479dcff3eee`.
- Root outcome `run244-norn-upper-route-outcome-20261106.json`, SHA-256
  `a10e0361a8469e8dbef41fc79851d2c42741ea32d3e4212a2437ea68fd715e44`.

The retained input comparison confirms the same xdotool keydown/sleep/keyup
structure and2s settle in236/237/244. Successful earlier S inputs requested750
or1500ms;244 requested100ms. Earlier first translations also followed turns,
and those UI scripts attempted F/conditional Escape before controller handoff.
None recorded numeric input focus or actual application key delivery. These
differences do not establish a duration, focus, binding or gameplay-state cause.
Comparison receipt `run244-input-delivery-comparison-20261106.json` has SHA-256
`4c8916434fc7bdbf22de7f1f20db0be0083504b02e3d98b6f2b8d9d47bc9ac01`.

A subsequent strict scalar read of the retained59-line log reports generic
messages5,191,278,280,424 and one each412/420, but still no movement. There is
no pre-probe baseline or per-line timestamp, so these totals cannot establish
post-probe activity. Complete receipt `209026-run244-input-counts-20261106.json`,
SHA-256 `fff26b18876bb5d78f8f13a6b7e82d43c6589e8854c72b713f78fb2f9e1061bb`,
REA `ev_5fa4fcbe1f0a8a59574e2b3560f280e6d54456f5c77152e56336dce16970ffbc`.
The next scoped proposal is one S750ms delivery observation with numeric
owned-window/focus metadata and append-only scalar log snapshots before/after,
without route continuation. No new run or resource request is staged here.

## Run246: PointerRoot guard stops before input

Fresh246 retains exact403ac37/N1, original synthetic144 and normalstage0.
Its controller planned one S750ms input after visual attestation and passive
numeric focus/process checks. The focus observer returned raw1 and refused
`focus_not_selected_pair` before completing process identity admission. No
pulse began, no movement was sent and no post-pulse delta exists. Baseline own
log has53 lines and zero movement rows. Controller claim12:46:13.273265Z and
release12:46:45.107208Z bound control to31.834s; launch/UI13 is incomplete.

X11 defines1 as PointerRoot, whose effective focus is the pointer's screen
root. This is a missing observer case, not evidence of a foreign application
or a cause of run244's result. Run246 retained no pointer path, so its effective
owned-window association cannot be reconstructed. The next fresh helper checks
the pointer path through the selected root, parent and render child, plus the
existing PID/start-time guards; it neither changes focus nor moves the mouse.
[Primary Xlib semantics](https://raw.githubusercontent.com/mirror/libX11/master/man/XSetInputFocus.man).

Window46 returned12:48:14.680415Z after container absence, unchanged executable
hashes/original private record and console privacy were verified. Root did not
visually review246 images; their admitted hashes remain in the readback. No
route, delivery, walking failure or retail completion is claimed.

Private evidence under `~/re/gw2/notes/`:

- `209026-run246-norn-input-delivery-launch-20261106.json`, SHA-256
  `95110fe7a41db605a6b60127f416d397bc382c1509a3f80eb927da16ab04bf14`,
  REA `ev_13daba9a46aed3dc7eb95563015cb440fcad499e7a49ceff57d28d3c19e3d951`.
- `209026-run246-norn-input-delivery-readback-20261106.json`, SHA-256
  `9872ed2cf7c2d146309ec02c5330ef64e27465a3695963e18cd95419b0b6565d`,
  REA `ev_0320ecf3b6db6f1c8cb1a38359aeaa9de02f94b16c8b613495395966e735c061`.
- `209026-run246-norn-input-delivery-return-20261106.json`, SHA-256
  `180fbec64b0bb74e3173dc649e6412dc27c1123b0635205856155698282a1476`,
  REA `ev_b43a32792af16c27d4a21d270f5538eec376787e54595478d97f33cb9452f1de`.
- Root outcome `run246-norn-input-delivery-outcome-20261106.json`, SHA-256
  `8919b2143483cdccea44e9114d38dc4ba94d768b331fdf1eedfd0288fdde7604`.

## Run248: one input produces fresh movement reports

Fresh248 retains exact403ac37/N1, original synthetic154-byte record, stage0,
unchanged actors and production hooks. Its observer handles raw focus1 only
after two matching passive pointer paths through root→owned parent→render leaf,
same screen/in-bounds coordinates and matching PID/start ticks. Raw focus1,
effective screen root543 and owned leaf8391711 remain separate fields.
Before/after observations admit the same process/window identity; no focus or
mouse change is made during the probe.

The sole controller claims13:08:04.088967Z and sends one S750ms request from
13:08:14.193099Z to13:08:15.106459Z. The host command takes913.275441ms;
that is not the application's exact key-event duration. Append-only own-log
comparison finds39 new lines/2183 bytes, including29 new message18 movement
reports and one each message4/11. The last actual ordinal29 is
**[-3301,-25600,-3745]**. The pre-input snapshot has no position, so configured
spawn is not substituted for a measured starting point. This is a positive
bounded input-delivery observation, not a causal isolation of run244's result.

Control releases13:08:39.095272Z,35.006305s after claim, with no second pulse,
route, attack, interaction or story input. Root reviewed hash-admitted first
and post-input images: the avatar/immediate ground are visible and the later
scene has more streamed detail. Streaming change is kept separate from movement
evidence. Launch/UI0; window47 returns13:09:22.386747Z with container absence,
unchanged PE/original record and privacy verified. No full route is claimed.

The original delivery console exceeds its12KB limit and remains unchanged.
The complete persisted result is separately pinned; a later strict scalar
recovery and root final readback are complete and untruncated. Future action
output should retain full focus records privately and print compact metadata.

Private records under `~/re/gw2/notes/`:

- `209026-run248-norn-input-delivery-launch-20261106.json`, SHA-256
  `70206ef09e67b2c02f4800072bccc1a5e1172b565677e7a6e06e5eacf2027a85`,
  REA `ev_839fdb0e731f7cb4ca3feb74f90444ea5d41a536e488beb2601fc9387f15127c`.
- `209026-run248-norn-input-delivery-readback-20261106.json`, SHA-256
  `b513a676157f3dd20fe926a8d1029c37be9a03e6e7c33ce280afd631d4dbdee7`,
  REA `ev_a803c37e770a511dc5d98a3c94ecc942ef05a448922556ee0863bf34a17befa0`.
- `209026-run248-norn-input-delivery-return-20261106.json`, SHA-256
  `18e0ac696de28190d1bfb3772713919a7cf812e454309584c957ef85ef7ffefc`,
  REA `ev_a59e5a6256e01f126b55f12ef0c8d81b615a82bb36130dd6c87a5500ccfe7046`.
- Controller receipt `run248-controller-outcome-receipt-20261106.json`, SHA-256
  `4665784724041d543347052b2fe5e1930ded62e90d494aafa471e75b7cd1fff9`.
- Root outcome `run248-norn-input-delivery-outcome-20261106.json`, SHA-256
  `3e79060ad5e383174a5d75752e88c5ebb1657a1c6335f46ae1f06046c75669ff`.

A retained236/237 audit corrects the next route plan: pure E turns have no
position/ordinal before translation; later Q turns can retain the old ordinal.
Sparse20 logging limits the later-turn conclusion, and generic message13 cannot
be attributed to a turn without temporal baselines. Current Rust retains rounded
XYZ but no typed heading scalar. Consequently an unresolved translation blocks
all further holds, while an admitted pure turn requires a fresh image and
invalidates translation calibration; it need not manufacture a new position
report. The next clear-ground translation must yield fresh actual samples.
This plan was later staged as run252 below. Private audit
`209026-retained236237-turn-audit-20261106.json`, SHA-256
`92906734b6d026d8426ed796bde81358aa719bae67271230bfbf388db4d46172`,
REA `ev_59e7fefee222b6818f4024193b37a62d5fd9067d8a27a9ced221572ab0aad1c2`.
Corrected plan `run252-norn-upper-route-corrected-plan-20261106.json`, SHA-256
`b40356994eef8c9bafb8d4b39fc55acbcef2acaa0136d412813cc433d385e974`.

## Run252: mismatched helper run name stops before movement

Run252 used exact403ac377 with dense movement logging, the original154-byte
synthetic saved record, a fresh145 prefix and unchanged stage0/spawn/actors.
Launch and UI exit13: the owned-focus observer expected
`run252-norn-input-delivery`, while the staged run was
`run252-norn-upper-route`. It returned `owned_run_marker_mismatch` before
the requested S750 command. No movement input or position report occurred;
this is a test-helper naming defect, not evidence that movement failed.

The controller claimed at13:45:10.320172Z and released incomplete at
13:45:33.646748Z, after23.326576 seconds. There were no target, skill,
interaction, story, hit or transfer events. Root returned window49 at
13:46:34.042389Z after verifying container absence, all70 immutable non-prefix
entries, the original record, executable hashes and console privacy. All three
root captures have complete raw and normalized output. The failed run remains
unchanged; a fresh254 fixture is being prepared with the full run-name binding
and a focused check that rejects this exact252 mismatch. No later runtime or
route pass is implied by that preparation.

Private evidence under `~/re/gw2/notes/`:

- `209026-run252-norn-upper-route-launch-20261106.json`, SHA-256
  `eeb9cddf5e9f7ca4bad444a020d943518a1bda52b6177df150229b8c8f7c68da`,
  REA `ev_fdc89377f727e4ce91a7f2e220350ad9e918e0d228dd7a9a74d00360a8f67107`.
- `209026-run252-norn-upper-route-readback-20261106.json`, SHA-256
  `3934d2390a9ac4cb0146486122131490c0b8008006e5de1e304076d88293b918`,
  REA `ev_175b08547cb6a762f9371ee690809f19abc87bfe3df84ce3209dffa02c48c8e1`.
- `209026-run252-norn-upper-route-return-20261106.json`, SHA-256
  `7092781e52c4c2f99a64cc458b0867f3d31b18a52ff7b9f1b768fdbf48a4bdf0`,
  REA `ev_548810e01e2099b44ed15c7ba931bb3afab12ffabfb5bcd6c542e9161b7f1609`.
- Root outcome `run252-norn-upper-route-outcome-20261106.json`, SHA-256
  `3f5bf2b47155bd82fff93b1a46e015325a9d71449da88a37f8cd43626670f5de`.
- Controller outcome `run252-controller-outcome-receipt-20261106.json`, SHA-256
  `83a84fb5c5e7bc24fe4517968103819d1e1522706f64dc55cddd5591a7a04ac8`.

## Run254: five movements observed; upper route remains incomplete

Fresh254 corrects only the complete helper run-name binding, preserving the
exact403 comparison binary, dense logging, original144 synthetic record,
pristine145 prefix and stage0 placements. The staged observer, contract,
manifest and owned marker agree. Three focused binding tests include the
actual252 mismatch; unchanged controller/reader/focus tests are reused through
exact source-rename comparisons. Two startup checks and1839 input pins pass.

The native run admits the same PointerRoot/owned process identity before and
after each completed hold. Append-only logs contain107 fresh movement reports:

| Requested hold | New reports | Last actual map XYZ |
| --- | --- | --- |
| S750ms | 25 | [-3284,-25600,-3744] |
| A750ms | 24 | [-3284,-25442,-3747] |
| W750ms | 28 | [-2978,-25442,-3743] |
| A750ms | 19 | [-2978,-25319,-3746] |
| A750ms | 11 | [-2978,-25255,-3747] |

The first new sample is[-3204,-25600,-3742]; there is no observed pre-bootstrap
position, and configured spawn is not substituted. Final proximity is39.051
map units fromB2 and881.197 fromB5. Root viewed the pinned first/final images:
the character remains outdoors near the hall's tables and covered carts.
These observations establish bounded movement, not a collision cause, complete
walkability or B5 arrival. No turns, target, skill, combat, story or travel
inputs occur.

Claim13:59:48.327749Z to release14:02:57.228132Z is188.900383 seconds.
The sixth attempt began164.809251 seconds after claim, already outside the
required18-second admission margin before the180-second input cutoff. No
attempt/action record exists; those writes precede native input. It was
budget-ineligible before input, while the generic exception does not reveal
the precise failed assertion. Completed action captures total24.664 seconds;
intervening review gaps, rather than native hold durations alone, consumed much
of the budget. No retry or in-flight extension occurred.

Launch/UI13 retains the incomplete route result. Root returned window50 at
14:05:23.881468Z after verifying container absence, all70 immutable non-prefix
inputs, the original154-byte record, executable hashes and console privacy.
All root captures preserve complete raw and normalized output. A separate
read-only plan will consider review overhead and a longer bounded window;
this is not a new runtime grant.

Private evidence under `~/re/gw2/notes/`:

- Launch `209026-run254-norn-upper-route-launch-20261106.json`, SHA-256
  `4467ea93afad02ab6f69e442c750b91a7763979c68e05feaa568ff32215d1e44`,
  REA `ev_de12f0f9d526ad3565fa14dbfb7ad841b3cbb0763f036c05ab909c760ce85462`.
- Readback `209026-run254-norn-upper-route-readback-20261106.json`, SHA-256
  `b6f1cf21ccf4256a6a1efd95e7dd17cf9cda21f41c2e89297f1f4b913474dcb3`,
  REA `ev_e1030c79f80175503920b10a372b668887fa802ba799ff4ff8eedbf118849bf0`.
- Return `209026-run254-norn-upper-route-return-20261106.json`, SHA-256
  `5d119b76c7337eac1df72d2a1704aa4796a9b2154a3ec69a18aecb789c870a11`,
  REA `ev_93d32ec044cc3bf496c5c6cc7bd0c796fd4874121338f47c95cfb1057390d7b4`.
- Root outcome `run254-norn-upper-route-outcome-20261106.json`, SHA-256
  `74bde3988e077117bfd8709f72a1275b72318c950f2639f46db902ab4e2498bd`.
- Controller receipt `run254-controller-outcome-receipt-20261106.json`, SHA-256
  `c248a2949c3c1268f3ebc4b6b14dbc18084c1e1a317cbc0dee964128764ed8ce`.

## Run258: B5 route prefix observed

Fresh run258 retains the original154-byte synthetic record, pristine145 prefix,
unchanged normal-stage0 spawn/actors/profile, production client hooks and the
403ac37 comparison binary with movement-log cadence1. It extends the controller
input/release/hard bounds to300/330/360 seconds. The focus, fresh-position,
image, calibration and hold limits are unchanged. Seven changed-boundary
tests, exact run-name binding, compact-output and two startup checks pass.
Root verifies41 metadata pins and1839 manifest rows before launch.

The controller claims at14:47:52.984197Z and releases at14:51:39.434437Z,
after226.450240 seconds. Eleven completed holds produce235 fresh, sequential
movement reports. Ten holds request750ms; the final W hold requests600ms.
There are no turns, failed attempts, unresolved translations, target, combat,
story, interaction or transfer events. Every before/after focus check admits
the same owned client/window identity. Requested hold durations and host command
timings do not establish exact application key timing or camera yaw.

| Hold | Last new ordinal | Last actual position |
| --- | ---: | --- |
| S750 bootstrap | 17 | [-3263,-25600,-3743] |
| A750 | 39 | [-3263,-25456,-3746] |
| W750 | 62 | [-3016,-25456,-3743] |
| A750 | 82 | [-3016,-25327,-3746] |
| A750 | 104 | [-3016,-25183,-3749] |
| A750 | 122 | [-3013,-25068,-3750] |
| W750 | 150 | [-2708,-25068,-3751] |
| W750 | 174 | [-2472,-25068,-3753] |
| D750 | 195 | [-2472,-25205,-3746] |
| W750 | 215 | [-2261,-25205,-3757] |
| W600 | 235 | [-2085,-25205,-3761] |

These are retained observations, not a replay sequence. The bootstrap's first
actual sample is[-3204,-25600,-3742]; configured spawn is not treated as an
observed pre-input endpoint. Final XY is29.154759 units from B5. The controller
attests the current view, and root independently reviews the hash-admitted final
image: health388, character on narrow ground beside a hall corner, rock face
and foliage. Onward clearance is not visible. The bounded B5 proximity/image
predicate passes; complete walkability, a route to hostiles and the full hunt
remain unproven. The next retained plan concerns viewing around this corner.

Launch/UI0 and cleanup pass. Window52 returns at14:52:33.045497Z with70 immutable
non-prefix inputs, original record, both executable hashes, console privacy and
container absence verified. All38,427 raw readback bytes/10 frames and normalized
frames are preserved; only duplicated terminal rendering is partial. No live
profile, actor coordinates, record or schema changes result from this probe.

Private evidence under `~/re/gw2/notes/`:

- Launch `209026-run258-norn-upper-route-launch-20261106.json`, SHA256
  `f0761bc25c582ed6a1bbe8edb46a0cf077e6310322b67903f952878b0a6d3b28`,
  REA `ev_23f4bb3ab3a82465c4b335f69e3586ee6095210f0ad2f6f607b19f86d455f5c9`.
- Readback `209026-run258-norn-upper-route-readback-20261106.json`, SHA256
  `81d029635dac77af35e81d4d494b3bcfc483373b9ef96919c74ec77f56d41928`,
  REA `ev_6ca1b93769558d8935164fa465b2b64783c05775ac1e34d28b99c8af07f14568`.
- Return `209026-run258-norn-upper-route-return-20261106.json`, SHA256
  `a7b09935992ddd0214c1acc92e3e097cfd0c3a9948a1123ce4d617a90802156d`,
  REA `ev_955a974294255f4d0be121c9bb6d71ae46f7372fb3d2cb15f174374365e19dfb`.
- Root outcome `run258-norn-upper-route-outcome-20261106.json`, SHA256
  `3b621de958283303d804ef90b0c6b58a227452e66092962c0f0f908d2ed51837`.
- Controller receipt `run258-controller-outcome-receipt-20261106.json`, SHA256
  `71537197155d772e8baae3a83975b180fadb03375c1994e9f1e3ab36b11774ca`.

## Run262: corner view observed; onward clearance unresolved

Run 262 keeps the exact 403 comparison with movement-log cadence 1, original 154-byte
record, pristine 145 prefix, stage 0 placements and production client hooks.
The controller approaches B5 using fresh images and position feedback, then
performs one terminal 250 ms E-key view and releases. Root checks 46 preparation pins
and 1839 manifest entries. Four focused endpoint/UI cases, exact run-name
binding, syntax and two startup checks pass. An actual UI fixture catches and
corrects the host/container image-path alias before staging is frozen; unchanged
controller, reader, focus and timing checks are reused.

Ten translation holds produce 229 sequential movement reports. Nine request 750 ms
and the final W hold 600 ms. The first actual sample is [-3204,-25600,-3742]; final
actual [-2120,-25147,-3760] is 38.587563 XY units from B5. The terminal turn has
zero new position reports. Every completed action admits the same owned focus
identity. No further hold, targeting, combat, interaction, story or transfer
occurs. These are observed actions, not instructions for blind replay.

Root independently views the pinned terminal image: the male Norn in purple,
health 388, stands on a narrow grassy/rocky strip between steep rock on the left
and the hall's dark arched end/adjoining wood wall on the right. A large gray
foreground occlusion covers the left third. No continuous onward passage is
established. Exact yaw remains unknown, and an absent position report does not
prove a perfectly stationary position. The image establishes neither global
unwalkability nor a complete route to the hunt.

Controller claim/release output is collected at 15:37:39.683Z/15:41:26.039Z,
226.356 seconds apart; these are not exact target-event or marker timestamps.
All 13 controller captures exit 0 without truncation. Root independently checks
the 39 files pinned by the final controller receipt. Launch/UI 0 and cleanup pass;
window 54 returns at 15:42:22.993843Z and the publisher records it closed. All 70
immutable non-prefix inputs, original record, executable hashes, console
privacy and named-container absence pass. The readback preserves all 38,978 raw
bytes/10 frames and all normalized output; only duplicated terminal rendering
is partial. Unverified unrelated host processes remain untouched. No live
profile, saved mode, actor placement or schema changes result.

Private evidence under `~/re/gw2/notes/`:

- Launch `209026-run262-norn-corner-view-launch-20261106.json`, SHA256
  `23952bd9b2630aaa541b3cfc520972ac504859cae5f454819adf056b9a538072`,
  REA `ev_654ad826d612e295688e963bd36cac617fd6dcbee08c90ee556f12a7fbff1d79`.
- Readback `209026-run262-norn-corner-view-readback-20261106.json`, SHA256
  `958adaebc6c7c28029b1d4bec6fbd3ccf14e7443803fa31acc3d4858bebbf1aa`,
  REA `ev_5d2aa0c0ff0d46d88884864eab098b180a12aa9be4a522bea288aab9de3bb7f7`.
- Return `209026-run262-return-window54-20261106.json`, SHA256
  `f13adf171bc29ab2d2717b40e96eca825348f1c41a1fd117fb3ab1ab9e4eb1de`,
  REA `ev_a01e1d3722674d4450ef9af7b358bc3121bd88d1ad03953eb0410b99ac018a56`.
- Controller receipt `run262-controller-outcome-receipt-20261106.json`, SHA256
  `172baee556d072c5a9e4bafd4fee53a7480e04cfbcdcc0874e9ccc098849e70c`.
- Root outcome `run262-norn-corner-view-outcome-20261106.json`, SHA256
  `8322200dd32fef469689bf120e8549fa3e5ba1134466bd9ec5e26d5d0edbf095`.
- Terminal image `controller-corner-view.png`, 1,105,801 bytes, SHA256
  `d47c400ffc94d56e1bec6efd1798de8099f262e046ab5fb2b64db431bdc1f40e`.

## Retained REA evidence

- Run119: `ev_34c13ab8e5acd423ac493b6e304bf0468235eea3a6b6f562c7a7de4a7614bfcd`.
- Run119 audit: `ev_94566830af79d495b778d425fea1808070ed7e313baafeec56942e1c4a35f659`.
- Run120 preparation: `ev_774f83f49ed5654a409c8946a1a411f398ab20e3c112a55e77ad3f393328679c`.
- Run120: `ev_5396d64f24d287bff774fd29ba507e23fd7b5bbf28d571a2f266c24fa5338275`.
- Run120 audit: `ev_14b96dd97dca0084ba45fb2f77d31b5ecae2989ee89e62d726d19f0337e6d4af`.
- Double-click Norn/gender/Mesmer: `ev_96a157514f6834925d1981c4e77d9bf8aede4ef86108417dda7834a215292c64`,
  `ev_e97a982e81c1fe165dc0183ef94f74506006664f04939b7c03be0ef25c160b1a`,
  `ev_efb55027117090be2aedfeb75ac6e1a845b09bc695f6e005a3adb934e87c064b`.
- Enter/lower-right diagnostic: `ev_4c573537ab498916bb706bf7617f89b3c9a9e33bf9594aa4fe13db25292fa0cf`,
  `ev_6bb2734f8e7cb1952e5f817ab5a138f1b0a02d5fd9684153b96deef1ffe1af34`.

Private bundles: `/home/blizz/re/gw2/notes/209026-norn-creator-run119-20261105.json`
and `209026-norn-creator-run120-20261105.json`. Raw evidence/screenshots remain
outside the repository.
