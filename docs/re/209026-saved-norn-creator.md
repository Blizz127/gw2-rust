# Build 209026 saved Norn Mesmer creator experiment

The observer has opt-in, single-account persistence and a creation scene.
Support first deployed in owner release `56242d1`. The integrator's latest
receipt is `82d8d04-cdn900` at `/home/blizz/gw2-rust-82d8d04-cdn900`, live at
2026-11-05 23:56:31 UTC in prepared mode, with rollback
`/home/blizz/gw2-rust-82d8d04`; saved mode is not enabled in the host config.
The profile retains the Hero's Moot respawn point from `9961b56-hm`;
the [release receipt](209026-combat-release-3cf2483.md) records hashes and checks.
Run126 observed actual UI creation, durable save and transfer to a playable
world connection. Run127 passed process-restart relog using the same record,
and its final screenshot shows the visible player model. Run125
resolved the test harness's missing navigation controls. Run134, using
the tested empty-composite runner fix, now shows the newly created avatar
on direct world entry and after saved-GUID waypoint rejoin. Run135 showed a
partially rendered preview. **Run142 subsequently rendered the complete male
Norn Mesmer preview**, with native graphics/animation readiness, no invalid
textures and no new assertion, after restoring the complete compatible
historical texture-remap metadata set. See the
[tested metadata increment](209026-texture-remapall-private-candidate.md).
The sole publisher deployed it after run202 verified prepared-profile
character selection, world entry, combat, loot and heart progress with the
currently offered runner0e6ac30. Root independently verified live metadata,
all kit hashes and four owned listeners at 23:56:53 UTC. **Run144 then passed
the full creation/save/world/defeat/waypoint flow with the actual offered
runner0e6ac30 and that metadata**, without the runner63 composite repair.
This removes the demonstrated need for runner63 in this tested flow; it does
not activate saved mode. The result was handed to the sole server publisher.
Runner63bc59a owner objects are published and verified,
but the last launcher-feed readback at 23:56 UTC still advertised b126 with
the old runner pin. At00:14:31 UTC the publisher explicitly marked the renderer
dependency resolved, then requested an owner choice about replacing the
prepared roster with an initially empty saved roster and process-local world
progress. The coordinator now confirms that native Claude publisher session
6f90f24c is waiting at that owner question: enable saved characters, hiding
the prepared Banshee Mesmer behind an empty roster, or keep the prepared
character. Initial character creation persists; XP/heart progress resets at
server restart. The coordinator surfaced the exact choice and has received
no owner answer. The native prompt must not be auto-answered or bypassed.
The previously accepted native agreement is separate from this unanswered
roster decision. Preserve the sole publisher's baton; delivered fix handoffs
remain separate, with no newer release receipt or resource lease established.

The live integrator held activation under the owner's exclusion of known
broken builds: the creator preview and direct-entry avatar must render.
Run144 satisfies those rendering checks on the currently offered runner;
it is private acceptance evidence, not an owner-mode deployment receipt.
The concrete private activation plan uses `GW2_CHARACTER_MODE=saved-norn` and
`GW2_CHARACTER_STORE=/home/blizz/gw2-rust-state/characters-norn-owner`, an
initially empty private owner store, stopped-service backup and rollback that
preserves later-created records. No synthetic character is imported. Plan:
`/home/blizz/re/gw2/notes/saved-norn-activation-handoff-v3-20261106.json`
(SHA-256 `6b7567c330463dfc85a7d7ed4dd512d5d4c14c90b15a9601e982d3d5d5c098bf`),
reviewed against current `82d8d04-cdn900`; v2 and the original remain preserved.
All44 local kit files matched the manifest. The plan preserves the complete
current profile, including revised hostile positions and ambient ravens, and
incorporates run144's acceptance within its exact fixture scope. A mode-only
rollback keeps CDN900 and restores prepared mode while retaining every saved
record; it does not assume a tested client metadata downgrade. This is a
prepared plan, not permission to answer the native prompt. Activation awaits
the owner's saved-mode choice, then revalidation of the then-current kit,
host mode and player state through the sole publisher.
The separate heart preflight/combination-coverage fix `d175bd5` shipped in
`018cb96`; it did not by itself clear the then-existing rendering hold.

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
