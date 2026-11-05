# Build 209026 saved Norn Mesmer creator experiment

The observer has opt-in, single-account sandbox persistence and a creation
scene. It is not deployed in owner release b7fbd8c. Actual UI creation,
save-to-world entry and relog have not passed runtime validation.

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

The taller window did not resolve the issue. Source checks identify416 as a
scene report, not a proven response wait. Navigation creation/layout has no
identified server-readiness or preview gate; model construction is local.
Do not add an invented acknowledgment. The next bounded diagnostic is a
passive inspection of the existing scene/model/UI state, using the anchors in
[client416](209026-creation-client416.md) and
[navigation](209026-creation-navigation.md).

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
