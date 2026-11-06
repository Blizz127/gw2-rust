# Build209026: sword activation receives no selected skill

Run148 observed the immediate boundary behind the cumulative candidate's
failed sword combat: **all four slot5 activations received a null SkillDef**.
The controlled character and primary skill bar were validated at each typed
site. The activation wrapper and body both ran, but the ordinary SkillBegin
call site was not reached. This identifies missing slot selection; it does
not yet explain which upstream palette, equipment or unlock state caused it.

The frozen client was build209026, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The offline sandbox used run203's exact server arguments and5516cb4 binary,
SHA-256 `eafed1d81abd58613c44df7783cfd53d475c6ac57ef0397118f9f06a790027dd`,
with metadata900 and the production0e6 runner plus the bounded observer.
No autoattack or diagnostic order echo was added. This was a prepared
character fixture and did not access the owner's saved character records.

## Observations

The UI issued four key1 presses, with Tab before the second, and one key6
control. A35-second delay after world readiness matched the earlier smoke.
For each sword attempt the trace recorded:

- `0x140316230` activation entry for slot5.
- `0x1411f51e0` wrapper and `0x1411f5250` body entry, both with
  `owner_status=controlled_current` and `selection.status=absent`.
- The body argument matched the current cached null reference.
- Body, wrapper and outer return with the selection still absent;
  `ordinary_request_call_seen=false` and no observation errors.

The observer ended after four attempts:26 records,24 callbacks,0 rejections,
no open invocation and a fresh marker. Its return values were not interpreted
as success/failure. The observed lack of a definition is consistent with the
[source ordering](209026-skill-input-action-lifecycle.md#activation0x2a-validation-then-explicit0x23):
an ordinary nonnull sword definition would emit0x2a before later admission
checks. This run cannot establish a target/range rejection or server hit timing.

The server received four `[29 05]` slot-end inputs, no `[2a 05]` and no
sword SkillBegin. The heal control produced `[2a 00]`, typed10176 with
target0, then `[29 00]`. The cumulative server's existing exclusion still
does not implement a heal. These results do not justify responding to0x29
with guessed damage or granting arbitrary skills.

The next dependency is how startup equipment/palette/unlock state populates
the primary bar's slot5 cache (`bar+0x1d0+5*8`). This run proves the cache
was absent at these four invocations; it does not select a corrective
startup message, grant or retail default.

Retained source identifies the normal cache writer: rebuild `0x1411fa340`
computes the slot address, calls resolver `0x141212cd0` at `0x1411fa7ac`,
reads its result at `0x1411fa7b1`, and writes the changed cached pointer at
`0x1411fa879`. Evidence `ev_6a005a85f8e5fdeaa3ecfd82498786cc01f734d080f8626cf64a2f437d038ff2`
is retained in `209026-skillbar-lifecycle-1.json`. The
[weapon selection source](209026-mesmer-starter-skill-requirements.md)
routes slot5 through the current primary item and its palette. Source
item33355 supplies palette90 and its profession7/category3 candidate10170;
this run did not observe which actual item/palette that resolver received.

The smallest missing observation is the owned slot5 resolver result at
`0x1411fa7b1`, paired with its actual primary item/palette. That separates
missing equipment/palette from failed entry eligibility. If palette90 is
present,10170's `0x141212fe0` predicate is the next boundary. The existing
startup inserts equipment before finalization, which rebuilds the bar;
server612 rebuilds it again but selects standard slots0–4. A605 grant for
10170 supplies one availability branch, not proof of the omitted retail
initialization. No additional startup packet is justified by the null cache
alone. No further runtime was performed after returning window24.

A read-only audit of the retained run148 fixture and current Rust startup
found no equipment-order discrepancy: item33355/instance1004 is created,
equipped in slot29, then set4 is selected. The full equipment batch precedes
170 finalization; saved mode retains the weapon/set bytes while adding mask
and dyes. The explicit skill inputs grant10176 and1080, not10170, but that
absence does not establish a missing retail grant because other selection
branches remain possible. Actual runtime item/palette and entry eligibility
remain the required observations; no packet or grant was changed.

The next observer is prepared offline in
`~/re/gw2/notes/slot5-resolver-observer-20261106.py` (SHA-256
`f703d89d7b5b3aba84444e8fe84f21530415f248c380490d8b52bc5a3cb94ddd`).
Twenty focused synthetic checks and syntax checks pass. Its five paired
sites observe the owned slot5 lookup, primary item, item definition, chosen
palette and resolver result with thread/stack/ownership guards and bounded
records/attempts. It requires no UI marker, allowing controlled startup
rebuilds to be observed. The result precedes possible remapping and cache
write, so it cannot by itself establish final selection or cast admission.
Receipt `slot5-resolver-observer-20261106-receipt.json`, SHA-256
`ad3bee0886e584eff5bc8fb0ac93734eefeb6def130a587962be309e810923eb`,
retains source/guard Evidence and limits. No native observation was run;
fresh harness staging and a new publisher resource window remain necessary.

## Bounded observation and cleanup

The private helper observes seven source-guarded entry, return and call sites.
It retains thread/stack identities and scalar results, not heap pointers,
and does not call target methods or change gameplay fields. Debugger
breakpoints affect timing; recorded elapsed times are not retail skill timings.
Its21 host mock checks passed before native use. UI success and successful
observer completion are **not a combat smoke pass**.

| Attempt | Result | Cleanup |
| --- | --- | --- |
| run146 | Exit5; new private entry script lacked execute permission, so the game did not start. | Container absent; executable hashes unchanged. Retained unchanged. |
| run147 | Entry mode corrected to0700; observer armed and world loaded, but UI deadline stopped before any skill input (exit8). | Observer reported0 attempts and inferior exit; container absent; executable hashes unchanged. |
| run148 | UI deadline160s and outer observation240s; four sword attempts and heal control completed, exit0. | UI0, observer terminal, container absent, credential-safe console check and executable hashes passed. |

Each attempt used a fresh directory. Run148 reused pinned pre-client inputs
from146; no prior runtime output or mutated prefix was reused. File-mode
checks now accompany byte hashes for its1,832 manifest entries. Run145
remains untouched and unlaunched; its old staged entry mode600 requires
fresh corrected staging before a future launch.

Window24 was returned **complete at2026-11-06T04:25:32Z**, before its04:41:36Z
expiry, with no shared container remaining. The publisher's existing run205
queue can proceed. No new runtime lease is established by this result.
Live service and launcher publication remained with the sole publisher.

## Evidence

Private paths are under `~/re/gw2/notes/` unless otherwise stated:

- Helper `sword-activation-observer-20261106.py`, SHA-256
  `258c4012f846f97d40d8179a794de9377661f3a44fda2da30852a9be9bdf9f06`.
  Its receipt records21 host checks and the exact source/ABI guards.
- Run148 input manifest:
  `/home/blizz/gw2-sandbox/runs/run148-sword-activation-observation/probe-inputs.json`,
  SHA-256 `4a7d4ad5dcea0f1f4086e3541442a7688d78e45d7544031b07bf9a71d8ddf117`.
- `209026-run148-sword-launch-20261106.json`,
  REA `ev_5d3dc072e19692efac768d73f2e79c4c396e305244962477d5044fe3f7bddd6c`.
- `209026-run148-sword-readback-20261106.json`, SHA-256
  `1bbe277fd0066ec4b42d57f3d2d952927dc8473f144fbf1859506632a356e2a1`,
  REA `ev_2dab1c0470fbf4dcace8fc5e0c8840a2b2b026275758d91559e4dfc1366516ad`.
- Raw client log SHA-256
  `09b60523d47a200fcbde4618aa331eda800ad418fa08b60f04ad30619945f008`;
  mission log SHA-256
  `6ccdf72f5f0255ccaddf4fc0b13baaf0ea2be779e190542dd35ff9276a4016d4`.
- Earlier failure receipts: `209026-run146-sword-launch-20261106.json`,
  `209026-run146-startup-error-20261106.json`,
  `209026-run147-sword-launch-20261106.json` and
  `209026-run147-sword-readback-20261106.json`.

REA launch/readback exited0 with complete retained output and verified owned
cleanup. It reported unrelated processes whose ownership it could not inspect;
they were left untouched. Raw evidence and debugger source remain private.
