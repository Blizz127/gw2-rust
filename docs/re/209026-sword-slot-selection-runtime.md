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

### Run209: equipment and palette are present

Run209 used fresh pinned pre-client inputs and the five-site observer under
the publisher's window25. Two complete owned slot5 lookups observed active
weapon set4, a present primary item33355 with kind24, and native palette90.
Both returned a null SkillDef with the cache still null. Both paired
attempts have no observer errors. There were11 callbacks, one rejected
callback, and a terminal inferior-exited record with no open attempt.
The rejection does not invalidate the two complete records, nor identify
the state of an unqualified call.

This rules out absent equipment or palette for those two lookups. It does
not establish which profession/category row was returned, whether10170
was evaluated, or which eligibility predicate rejected a candidate. The
[exact base-sword map](209026-mesmer-starter-skill-requirements.md#exact-base-sword-eligibility-boundary)
defines the next bounded observation. No skill input, grant or gameplay
change was made; a null resolver result before remapping/cache write is
not a combat result.

UI and outer run exited0, world initialization was recorded, no assert text
was found, the credential-safe console and frozen/working executable hashes
passed, and container absence was independently verified. Window25 was
returned complete at **2026-11-06T04:52:21.503066Z**; it authorizes no further
root runtime after that return. Existing runs remain unchanged.

- Launch Evidence: `ev_8913c172cdd705c924db92d0ed251098f65badbda69fafed8035a35cc7e29ce3`.
- Readback: `~/re/gw2/notes/209026-run209-slot5-readback-20261106.json`,
  REA `ev_a5f28e677232aa2446d57a3d84e9aa626f92c1b1e49a7b3970e38a23e44c8383`.
- Cleanup/return: `ev_023a2037714edb73d14312d0a20a7ea82e456b73f9e862f285d13d158be9ed40`.
- Input manifest SHA-256:
  `b846bece584d5c5fc5abb997767da00f7534dc2adabfc7be34d8f5f3053645df`.

### Run211: base-sword availability rejects selection

The nine-site observer followed two owned slot5 lookups through actual
palette-row selection and every candidate. Both selected palette90 row16,
profession7/category3 with four entries. Candidates45230,10172 and10171
returned false; their detailed rejection predicates were outside this
probe's scope. Base10170 was then evaluated with its availability bypass
clear. Its slot predicate returned1, and its unsigned level returned1
against minimum1. Its availability predicate returned0, immediately
rejecting the candidate. The locale scalar was0, but the locale predicate
was not reached. Both outer lookups and caches remained null.

There were30 records and29 callbacks, with one rejected callback and no
errors inside either complete attempt. The terminal record reports no
open lookup or candidate. UI and outer run exited0; no assert text was
found. Executable hashes, credential-safe console and container cleanup
passed. Window26 was returned complete at **2026-11-06T05:15:07.388549Z**.
This is a negative selection observation, not successful casting or proof
of a full retail availability policy.

The isolated correction candidate is
`~/re/gw2/notes/skill10170-availability-candidate-20261106/skills-10170.bin`,
17 bytes, SHA-256
`4ceabe189b2b1fa26310a1339424b882cadfc03cd8480f26c7a55be0b2280678`.
It preserves605(10176,player1) and612(palette366,slot0,locale0,player1),
inserting only605(10170,player1). Typed encode/decode, independent literal
and extracted-schema checks, SkillFixture ordering/identity checks and a
no-listener configuration check pass. No live profile or saved data changed.

The next smoke must omit `--mission-autoattack-skill`, not set it to0.
Live4b3c4c2 otherwise responds both to real SkillBegin and to legacy29
release input. Keeping both after the grant risks duplicate damage. With
the option absent, real controlled10170 requests retain the existing
damage-on-skill30 response and29 produces none. The old echo remains off;
its Word0 and unmodified request flags do not establish accepted execution.
This isolates the candidate from the cumulative45 changes while leaving
actual cast lifecycle, chain progression and retail effects unresolved.

Evidence under `~/re/gw2/notes/`:

- Launch: `209026-run211-slot5-launch-20261106.json`,
  `ev_da26361418f29d9906bfb920ab4957b0caee53cfdaee0ce6fdb20dfeb80b661f`.
- Full readback: `209026-run211-slot5-readback-20261106.json`,
  `ev_b4237b14fccd74f8df3111192fbc1d35c4bfaea94b31dfb4b7119ef88fb421de`.
  Its complete29402-byte raw/normalized output is retained, but three
  rendered frames are missing. This is recorded as partial rendering.
- Compact readback: `209026-run211-slot5-compact-readback-20261106.json`,
  `ev_e90ad18a50cd17e92cd4754aafe69be513d22592fa4742e0c7383946c36ea16d`.
  It rechecks identical input-log hashes and exits0 without truncation.
- Return/cleanup: `209026-run211-window-return-20261106.json`,
  `ev_020ebabdaaacc72c331e9e36455df952594f8f15dee324c06c8030ff05b4af1b`.
- Client log SHA-256:
  `58aced651204f63d64c882d0d7809cc643ed06f636eab3a695f2836e35005949`.
- Mission log SHA-256:
  `1cc158c7528e0a5138f7752b540dc40faa875c1d646a8695470b20a1f34343b1`.

### Run214: availability passes, targeted combat still unproved

Root's `run214-skill10170-availability-smoke` used exact4b3c4c2 with the
17-byte availability batch above, damage30, and both legacy autoattack and
diagnostic echo omitted. This is distinct from the publisher's
`run214-great-hunt-4`. Window27 was explicitly activated05:30:30Z and
verified05:33:12Z before the one bounded launch.

The first startup lookup still failed availability. In the second, all
four predicates passed: slot, level, availability and locale0. The native
10170 candidate returned1. The final outer lookup was unavailable to the
observer, so a final cache-selection claim is not established. The client
sent one owned10170/slot5 request, but its target was0. A release input
followed, with no legacy order, echo or extra hit. **No damage occurred;
this is not a combat smoke pass or a release-ready profile change.**
The next correction is to the bounded UI targeting step.

The retained before/after screenshots show that the harness's unconditional
Escape opens the menu. There is no client227 target selection in this run.
The final observer error is an `identity_or_pair_guard` at `lookup_result`;
it must not be recast as a null selection after the candidate passed.
The next harness removes Escape and requires a fresh hostile227 selection
before its one key1 press, then checks the request's matching target and
damage. Passive final-cache coverage remains a separate observation.

UI and outer run exited0; no assert text was found. Console and executable
hash checks passed, and container absence was independently verified before
returning window27 complete at **2026-11-06T05:37:58.599062Z**. Cleanup does
not resolve the partial observer coverage or the missing damage result.

An isolated exact4b regression passed one test: an encoded targeted10170
request causes one30-point hit; subsequent29/2a input causes no extra
response, damage or target/retaliation mutation when autoattack is absent.
The legacy-enabled positive control reproduces the extra order/hit. Its
receipt is `4b3c4c2-autoattack-omission-regression-20261106.json`, SHA-256
`fe63bc7b7558e60e57ab5117d1181666f462f0e8c5feefd14fd442537365bfdb`.
This synthetic regression does not supply the missing targeted client smoke.
The same focused regression subsequently passed on currentcf13a3b
(one test,117 filtered out). Its attack handlers and option handling are
byte-identical across the two releases; surrounding additions include
lethal-hit story callbacks and timed regeneration. Private receipt:
`cf13a3b-autoattack-omission-regression-20261106.json`, SHA-256
`6bf895687af1fa007c5688bdfc597a3ffcd7c6c9cf5a35ec497cdf6e04644e9a`.

Private evidence:

- Launch: `209026-run214-availability-launch-20261106.json`,
  `ev_afe393f16027e8fb89d2d8b95bbba185c842bf100108ad776a0e34362da35561`.
- Compact readback: `209026-run214-availability-compact-readback-20261106.json`,
  `ev_e01fb0ec1d22aaf85763534123b185101405bcc8a7a88a0f61bc07d5a9dada23`.
- Window return: `209026-run214-window-return-20261106.json`,
  `ev_ea5ba815d98b3da8abe6baf1e72417b3ddbc09af12240e6c52852610e194e49a`.
- Client log SHA-256:
  `3e92903926fb0e8fee3ba53a0f4eda15efc6f60fc50ae22a746fc08e52c09a69`.
- Mission log SHA-256:
  `0332d110886c3c7477ff1ad027cd148abc19a2350564191231b76df6f96d3641`.

### Run215: no target acquired; no cast issued

`run215-skill10170-targeted-smoke` used the exactcf13a3b binary, unchanged
availability grant and passive observer, with autoattack/echo absent. The
UI removed Escape, required a fresh hostile227 selection before one key1
press, and allowed four targeting stages with at most six seconds of
forward movement. Ten focused reader checks and two no-listener startup
checks passed before the run. Publisher window28 was verified05:47:58Z.

All four10170 predicates again passed in the second startup lookup, with
candidate result1 and the same partial final-return guard. The targeting
stages produced **no227 event**, so the harness correctly issued no skill
press. UI/outer exit9 is a failed combat smoke; no damage occurred. Cleanup,
console and executable-hash checks passed. Window28 was returned complete
at **2026-11-06T05:52:57.618120Z**, with no shared container remaining.

Retained screenshots show the first world-ready capture was still loading.
Later target captures show the world without the Escape menu. Movement
advanced from approximately(-3188,-25600,-3742) to(-2605,-25643,-3750), then
stalled nearx=-2598. The later camera is occluded by a wagon/wheel ahead of
spawn. Thus repeated forward input is not a demonstrated route to the
distant hostiles. The next focused test will use an explicitly diagnostic
nearby hostile placement and bounded world-settle time, preserving the
real client selection/request/damage checks. No live population changes
or retail placement claim follow from that test fixture.

Private receipts:

- Launch: `209026-run215-targeted-launch-20261106.json`,
  `ev_7225e4a200ee2f8276d49df9808a1f8288a2588de2fc145b061ee70b8aaf2634`.
- Compact readback: `209026-run215-targeted-compact-readback-20261106.json`,
  `ev_8dce571811fec1c597a53ec11d376b6371b0e48ed7e77b71e2fa52aae7e0aa20`.
  The earlier verbose readback was truncated; this bounded readback is
  complete and pins both logs below.
- Window return: `209026-run215-window-return-20261106.json`,
  `ev_3a08abfc9e9a31b692bec6018082a38e465f793bd471858bf6525b091679a852`.
- Client log SHA-256:
  `057ac8d1237b907c0b7ca16b44ffdbb87d5a9e6d4e6d0ab382511832f78c0c1b`.
- Mission log SHA-256:
  `519a9285b1e844be1f7c6dd0fd23679937ea2a083546e1eb38ef5638f61dd4db`.

The separate return-guard review found no proven success-path ABI change
or invalid alignment check. Cached10170 definition/body offsets satisfy
the reader under an aligned synthetic mapping. The coarse exception still
cannot distinguish parent/owner, output selection or cached selection.
It is not evidence of a null definition after candidate success. Receipt
`slot5-return-guard-review-20261106.json`, SHA-256
`ce7a2086e86094ad8e89e4522469b5fe5c179420e37a126780c389a587fb0766`.

### Run216: targeted ordinary request and one damage response pass

`run216-skill10170-near-target-smoke` completed on exactcf13a3b with the
17-byte grant, damage30, and autoattack/echo absent. It used a fresh146
pre-client prefix. Only diagnostic wolf NPC4 moved to the existing nearby
test position(-3050,-25450,-3745); no movement input was used. The125s
settle accounts for retained cold model loading. This placement is not an
authored spawn or part of the release handoff.

The changed-feature smoke **passes**:

1. A fresh client227 selects hostile4 before the skill press.
2. The controlled player1 sends one10170/slot5 request to target4.
3. The server sends one255 response, lowering that target from118 to88.
4. One subsequent29 release produces no legacy order, extra hit or echo.

UI and outer run exit0; no assert text is found. Console and executable
hash checks pass. Container absence was independently verified, then
window29 returned complete **2026-11-06T06:10:56.781678Z**. The native10170
predicates all pass in the second startup lookup; final-cache instrumentation
still reports the known guard failure. That partial observation is retained
separately from the actual selected-target/request/damage evidence.
This verifies the small availability correction and removal of duplicate
release-triggered damage, not retail damage, accepted lifecycle, attack
chain, healing, cooldowns or the full Mesmer combat implementation.

The concrete release handoff was submitted through the existing sole
publisher's native inbox: install the17-byte skills batch and explicitly
set effective kit-profile `GW2_AUTOATTACK_SKILL=` empty; retain damage30,
echo off, and all newer profile/content/saved-mode changes. Merely deleting
the assignment could inherit host configuration;0 is not disabled. The
then-current4c46cad batch matches the original12-byte baseline
`bf7e83481d73b5acb4836a742072e5118f7001c8a065766273b64c16c5b9e55e`.
The publisher must merge the grant if its newer batch differs. Root does
not publish or claim the candidate live from this submission.

Private evidence:

- Manifest SHA-256:
  `0c389f213fef7fcae61b100d38e78793512ebe0c702e6bc33c0aa800149fdfb4`.
- Staging: `run216-skill10170-near-target-staging-receipt-20261106.json`,
  eleven reader checks, two no-listener checks and unchanged observer tests
  reused by hash; shell/composition and executable-mode checks pass.
- Launch: `209026-run216-near-target-launch-20261106.json`,
  `ev_794476afaa6d1167f686229a41e18b086559ffc5a23ec4369ac47c8a4f3c5f48`.
- Complete compact readback: `209026-run216-near-target-compact-readback-20261106.json`,
  `ev_b0bc4fb7262a4ff39d25dc25d05367779b4ff4ba7c7b051ee5553ad177780ba6`.
- Window return: `209026-run216-window-return-20261106.json`,
  `ev_a24e52e37aaaaea40c7259c2e881a9aae34989f814ac4dce89d17497826e54c0`.
- Native submission: `run216-release-native-handoff-20261106.json`.
- Client log SHA-256:
  `898fd3624e45d57f38275e035a860f722439690a83c66014bbfde11646295d98`.
- Mission log SHA-256:
  `81041dfa705740a395f609ee60ee66399c0ba8472b34ea788d23ecdde20dc1b1`.

### Earlier activation observations

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
