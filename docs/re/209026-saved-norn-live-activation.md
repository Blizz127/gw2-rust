# Saved Norn creation: live activation and independent readback

Saved character creation is live, currently on **e624c8b** at
`/home/blizz/gw2-rust-e624c8b`, independently verified10:41:12Z.
The [latest increment](#verified-skill-admission-release-e624c8b)
rejects skills absent from the explicit availability list before combat mutation.
It retains no-clone Ether Feast healing/recharge and guarded same-process
health/cooldown carry with original-roster relog routing. Saved mode was first
activated on82d8d04-cdn900.
The sole publisher records the owner's
native choice at **2026-11-06T03:29:03Z** and activation at **03:29:33Z**.
Root independently verified the configuration, live hashes and service at
**03:43:51Z**. The earlier unanswered roster prompt is historical.

## Verified skill admission release: e624c8b

The sole publisher deployed the exact tested **e624c8b** artifact at
**2026-11-06T10:35:43Z** to `/home/blizz/gw2-rust-e624c8b`.
Rollback is `/home/blizz/gw2-rust-7c26f45`. The
[admission correction](209026-skill-caster-ownership.md#explicit-skill-availability-e624c8b-verified-live)
has an executed failure-before regression,17 passing focused tests, a static
musl build and two saved-profile no-listener startup checks. Profile, supervisor
and data remain identical to7c26f45. The publisher's separate40fef7a merge into
the quest162 branch is not included in this exact artifact.

Root's independent readback at **10:41:12.080308Z** verifies all current and
rollback kit hashes, selected/running executables and four owned healthy
listeners. Saved-norn mode, one record in the private0700 store and the unchanged
activation backup are verified without reading record contents. Zero humans
and bots were present. Ether Feast, damage30, the17-byte skill fixture and
existing world/vendor/heart/POI settings remain enabled; legacy autoattack/echo
remain absent. `story_branch` is `none` and frozen metadata remains900.
No launcher feed, audience, frozen client, owner store or schema was changed.

Binary SHA-256: `723b9c36947df9a747010334fe999a0e98cfd4308d120015751aab2f36089453`.
Manifest SHA-256: `689e5e4db2fb4c45090d9dd68a135b0b092f438e7b86d5e018bbff43c44f9269`.
Private verification receipt: `e624c8b-release-verification-20261106.json`,
SHA-256 `f89f806c43f7052fc8311c1de13efdf988f295c546ef0dcc3c54ae9c4b00a9fe`.
Full readback: `209026-e624c8b-release-readback-20261106.json`, SHA-256
`1f7f96952146d69c3f557dd884119ec672216e2a72eca192c37a3679f4c8fd2f`, REA
`ev_7e3d18e854a7cfe1475a88ec89955cee540689bfdf5b8da1a1cf6ab3f4f1dd48`.
Evidence is untruncated and owned cleanup is verified; one unrelated process
environment permission failure remains recorded and untouched.

## Binary baseline release: c2cb6fd

The sole publisher deployed **c2cb6fd at 2026-11-06T03:56:12Z**. This
cherry-picks the Ether Feast damage-exclusion guard onto the previous live
source, preserving the working diagnostic attack setting. It does not
implement healing or recharge. The publisher's release record reports
run204's attack/kill/XP/loot/heart smoke passed and the cycle ran with zero
players. Rollback is `/home/blizz/gw2-rust-82d8d04-cdn900`.

Root independently verified the following at **04:01:16Z**:

- Current link selects `/home/blizz/gw2-rust-c2cb6fd` and all kit hashes pass.
- Game SHA-256:
  `1da9300828542334c8407cbd758cef65ab850a58cb26562e6da8e03012cf5e1b`.
- Manifest SHA-256:
  `d795c56b487d9b5dc5d9c9430a18b86e7b67192b7186a03818b8687d9bc02e17`.
- Ready/running, four owned listeners, zero humans and bots.
- Saved mode remains enabled; the private store now has one `.char` record.
  No character contents were read. The original activation backup still
  matches its recorded hash.
- Metadata900 and b128 feed hashes remain unchanged from the receipts below.

Evidence: `~/re/gw2/notes/209026-live-c2cb6fd-readback-20261106.json`,
SHA-256 `dba1a309674ae5afe1669d10e25c0212cd9f3801115927da28b21d988f107004`,
REA `ev_8884999c8b201612e3e4d1a511c82661f64a9d860f2966ee73ecbfae2b847bba`.
The helper exited0 without truncation and verified owned cleanup. The
shared `latest_live_release.sha256sums_sha256` still contained the old
manifest hash at the first post-release inspection; the independent hash
above and publisher's release note agree on the new manifest.

After run148 cleanup, a fresh independent readback at **04:27:55Z** again
verified c2cb6fd, every kit hash, metadata900, ready/running with four owned
listeners and zero connections, saved mode and one private record, and the
activation backup. The b128 feed was unchanged at04:27:56Z. Receipt:
`~/re/gw2/notes/209026-live-c2cb6fd-post-run148-20261106.json`,
REA `ev_5169156df88f4a24f6335e868a5418cebc99aca9154746f209d5c2b86e150072`.
The read-only helper exited0 with complete output and verified owned cleanup;
unrelated processes it could not inspect were left untouched.

## Activation behavior

The next login presents the saved roster, initially empty at readback, so
the owner can create a Norn Mesmer. The prepared Banshee Mesmer is hidden.
Initial creation records persist; XP, loot, heart and waypoint carry remain
process-local and reset on a server restart. The retail Great Hunt and
Mesmer skill effects are still incomplete. Run144 is the prior complete
private creator/save/world/waypoint acceptance; this readback is not a new
owner-client playtest.

## Read-only verification

The readback used the existing SSH identity and published probe helpers,
without changing configuration, service state, the store or an archive.
It read only the two selected configuration values and store metadata,
not character records or other configuration values:

- `GW2_CHARACTER_MODE=saved-norn`.
- `GW2_CHARACTER_STORE=/home/blizz/gw2-rust-state/characters-norn-owner`;
  directory mode0700, zero `.char` records at readback.
- User service `gw2-server.service`: active/running; ActiveEnterTimestamp
  `Thu 2026-11-05 21:29:33 CST`, equivalent to03:29:33 UTC onNov6.
- All44 kit manifest entries verified; game binary SHA-256
  `10bd95cf90858ace19de8040311d7ac74bce403dfee92bf93b10f997035eee29`;
  manifest SHA-256
  `ca037744d43485eb298ba3385965fb35a5ac043424c704d54fe16365bfa84741`.
- Four listeners belonged to their own processes; health ready/running;
  zero human and zero bot connections.
- Selected metadata remained`209026_0_0_2130706900_276`; root and main
  metadata hashes matched their retained values.
- Activation backup:
  `/home/blizz/gw2-rust-state/backups/saved-norn-ZFhCINlm.tar`, SHA-256
  `c87e40f578544e71ef47dc28dab524bd8c0201fc9c6f27cdf1e10d4dc303aa52`.

The publisher's mode rollback restores the prior configuration while
preserving saved records; root did not perform rollback. The existing kit
rollback remains`/home/blizz/gw2-rust-82d8d04`; a client metadata downgrade
has not been tested by this verification.

The launcher feed separately read back **b128** at03:43:52Z, feed SHA-256
`4094162f23023a3ed34527a9e8f903f8cd4bb63893209701915fc3a2e8266f9a`,
package SHA-256
`79f198f05cb0dcdd3cbe44404f1cbb85a95505197898b715a63d782bb0342b24`,
size116357624 bytes. This query checked feed metadata, not the package's
runner pin or a changed audience/access arrangement. Root published no
launcher artifact and did not clear the historical owner-only feed constraint.

## Current resource and release state

At03:41:35Z, the same native publisher session6f90f24c was alive with
status`shell`, and its pane reported activation and preparation of5516cb4's
combat smoke. This supersedes the03:18 unanswered-prompt observation.
The publisher's existing window23 had been granted at03:29:03Z; because
run145 had not launched, root returned it **unused at03:42:49Z** as requested
for publisher priority run203. The shared cleanup record is written. No
client was stopped, no running test was changed, and root needs a fresh
window24 before launching run145. The publisher subsequently granted
window24 at03:56:36Z for one bounded creator run and cleanup, with a
45-minute limit. [Run148](209026-sword-slot-selection-runtime.md) subsequently
completed the bounded observation and cleanup; window24 was returned at
04:25:32Z. Existing run145 inputs remain unchanged and unlaunched. Run148
found a null selected SkillDef on all four controlled sword activations;
it did not establish working sword combat or a replacement startup input.
The earlier log inspection below establishes
that run203 ran and reported cleanup, but does not establish a combat pass.

The later cumulative [caster-ownership candidate45c0731](209026-skill-caster-ownership.md)
remains withheld. Its coordinator handoff was refused and not delivered.
The corrected independent9451cf0 handoff was submitted to the existing
native publisher at04:33:23Z; submission alone is not a deployment receipt.

## Subsequent combat smoke readback

At **03:52:25Z**, root inspected the existing run203 logs through REA,
without launching a client, accessing an archive or taking a resource lease.
`run.out` reports world startup, container cleanup, unchanged executable
hashes and a passed credential-safe console check. The mission log contains
16 slot inputs `[29 05]`, one `[2a 00]`, one `[29 00]`, and exactly one
typed SkillBegin: Ether Feast10176, controlled agent1, target0. There is no
sword SkillBegin or post-startup server response in that retained log.
The client log contains no `assert` text; that bounded text check does not
establish successful combat or an error-free client.

Run203 therefore does **not establish the required combat smoke pass**.
The input/admission cause is unresolved; these observations alone do not
prove a particular client or server defect. Both 5516cb4 and the cumulative
45c0731 candidate remain undeployed in the available release records. The
independent Ether Feast guard was released separately as c2cb6fd above.
The slot-input corrections must not be replaced with guessed damage merely
to make the smoke appear to pass. Window24 was subsequently returned after
run148; any new runtime now requires a fresh resource window.

The retained [activation contract](209026-skill-input-action-lifecycle.md#activation0x2a-validation-then-explicit0x23)
narrows the next observation upstream of accepted-order timing. Given a
nonnull ordinary sword definition, `0x1411f5250` emits `[2a 05]` before
downstream admission checks; cached sword10170/10171/10172 satisfy its
early flag tests. The missing observation is slot5's actual selected
SkillDef and whether activation reaches `0x140316230 → 0x1411f51e0 →
0x1411f5250`. Repeated `[29 05]` proves only an end/deactivation path.
Missing slot resolution, an earlier gate and a different input path remain
possible; target or range is not established as the cause. Run145's current
downstream heal/request-producer hooks alone cannot distinguish this gap.
The later run148 observed the previously missing boundary and proved its
selected SkillDef was null; see the linked runtime note for the remaining
cache-population dependency.

Retained readback:
`~/re/gw2/notes/209026-run203-status-readback-20261106.json`,
REA `ev_4bdb7bfba8b10a547fe727c9d86bf309237eb8de720422d484c906f198f56797`.
The read-only helper exited0 without truncation, with owned cleanup verified;
an unrelated process could not be inspected and was left untouched. Input
`server4.log` SHA-256:
`632b5182a306194e9141b24e4921b13f23fabe1dbe8dadf313b1a0d26d66b424`.
`run.out` SHA-256:
`78f7058b97263806c7a62e1c99576c26c2bcc7769e13c13e60e0c57ea748e9da`.

## Evidence

Complete REA readback:
`~/re/gw2/notes/209026-live-saved-norn-readback-20261106.json`, SHA-256
`7cdd8288f14b179398458d1d44f23a499782d69b39cf8bc886855815fcb5e531`;
REA`ev_15a4b489f75b6215d6842c7a525baad7f39d9d0cd6039b50a7564e8e4226405e`.
Capture exited0 without truncation and verified owned-helper cleanup.
Native availability receipt:
`~/re/gw2/notes/publisher-native-check-20261106-034135.json`.
The shared publisher receipt and owner decision remain in
`creator-lane-peer-handoff.json#/latest_live_release` and
`#/saved_norn_reassessment/owner_decision`.

## Spawn-area ambient increment: c2cb6fd-amb

The sole publisher records **2026-11-06T04:30:49Z** deployment of
`/home/blizz/gw2-rust-c2cb6fd-amb`. This profile-only change adds a neutral
boar, wolf and two ravens within about600 units of the diagnostic Norn
spawn. They are placeholders, not the authored Great Hunt population.
Publisher receipt commit5f482ca records run205 with all7 ambient and4
hostile NPCs spawned, visible wolf/ravens, no asserts, and a zero-human
watched cycle ready in3.3s. That smoke was performed by the publisher.
Rollback is `/home/blizz/gw2-rust-c2cb6fd`.

Root independently verified the selected kit at **04:36:02Z**: all manifest
hashes pass, health is ready/running with4 owned listeners and0 humans/bots,
saved-norn remains enabled with one record, store permissions remain0700,
and the activation backup hash is unchanged. No character contents were read.
The binary remains SHA-256
`1da9300828542334c8407cbd758cef65ab850a58cb26562e6da8e03012cf5e1b`.
The ambient kit's actual manifest SHA-256 is
`c5f0a3aa8fe412c4c945175d14d51a656a05cc427feaa2197963f9e3ba8c5b9f`;
the shared publisher summary still carries the obsolete82d8d04 manifest
hash and must not be used as this kit's checksum. CDN900 root/main hashes
pass. Feed metadata remains b128 at04:36:03Z; its runner pin and audience
were not reverified. No root deployment or launcher publication occurred.

Private receipt: `~/re/gw2/notes/209026-live-c2cb6fd-amb-readback-20261106.json`,
REA `ev_bdbc1fe42c4f02fad060caa78cf5928a8ffdd0ef51bcbb12fc5544b6dd3e4c65`.
Readback exited0 without truncation; owned-helper cleanup was verified.

## Caster guard and Eir appearance: 4b3c4c2

The sole publisher records **2026-11-06T04:46:24Z** deployment of
`/home/blizz/gw2-rust-4b3c4c2`, with rollback at
`/home/blizz/gw2-rust-c2cb6fd-amb`. It includes the independent9451cf0 caster
guard as e788d58, quest542/544 codecs and server377 composite NPC support.
The research steps-on-kill option is off. Eir Stegalkin uses her species3172
composite appearance near the diagnostic spawn; her position is manually
chosen, she is neutral, and dialogue is absent. This is not proof of the
authored Great Hunt encounter.

The publisher's release note records112 observer tests, the377 codec test,
saved-mode checks and a zero-human watched cycle. Run208 rendered Eir and
exercised autoattack/kill after the coded-name correction; run207's earlier
TextParser assertion is retained as a failed attempt. Run206 artificially
advanced quest154 through542 and displayed a completion/reward panel.
The retained screenshots show blank title/objective text; the first forced
step changes the icon to a checkmark. The final panel shows a weapon icon
and72 copper. The harness explicitly supplies step order1,2,3,6,4,5, so this
does not establish authentic conditions, reward acceptance or inventory
delivery. Retained screenshot admission Evidence:
`ev_7ba42da8069ef30491be7abccc08173c3dade989eff6bfac3772fb91142935aa`.
Root did not rerun these publisher smokes.

Root's independent **04:53:03Z** readback verifies all kit hashes, four
healthy owned listeners, zero humans/bots, saved-norn mode, one unchanged
store record count and0700 permissions, the original activation backup
hash, and metadata900 root/main hashes. No saved record contents were read.

- Game SHA-256:
  `8b75cceecc196e39872d5ca85d31c543fcfaacfcbbc78e2d55858cefeb2ee979`.
- Manifest SHA-256:
  `e429d37155ddadf06287f8bbec24263af5c01cc757aae27ace5d32de0166432b`.
- Receipt: `~/re/gw2/notes/209026-live-4b3c4c2-readback-20261106.json`.
- REA: `ev_2660994d1fd6a86819ecd754b0cc4f3d21532df495f14f602bab3b2d1cff6ab5`.

The helper exited0 without truncation and verified owned cleanup; unrelated
processes with inaccessible ownership metadata were left untouched. Feed
metadata remains b128 at04:53:04Z. Runner pin and audience were not
reverified; root made no server or launcher publication.

## Great Hunt story actions and regeneration: cf13a3b

The sole publisher records deployment at **2026-11-06T05:32:09Z** to
`/home/blizz/gw2-rust-cf13a3b`, rollback
`/home/blizz/gw2-rust-4b3c4c2`. Its release record reports talkable Thora,
Ido and Eir, a stand-in wurm boss, ordered story-action progression and
out-of-combat regeneration. NPC placement, Thora/Ido appearances, boss,
dialogue and regeneration pacing include reconstruction or placeholders.
Publisher client smokes exercised talking to Thora, a trophy kill and
regeneration; later story steps have unit coverage but no completed client
smoke. The earlier completion-panel observation does not prove reward
acceptance/delivery or authentic authored action conditions. Hunt state is
connection-local and restarts on login or waypoint rejoin. Ether Feast
healing/recharge and full retail combat remain unfinished.

Root independently verified the selected kit at **05:38:38Z**: all kit
hashes pass, four owned listeners are healthy, zero humans/bots are present,
saved-norn mode remains enabled with one record and0700 store permissions,
the original activation backup hash is unchanged, and CDN900 root/main
hashes pass. No character contents were read and no configuration changed.

- Game SHA-256:
  `944a7a6747e9db21de0aae010c78ba13cc469795c96fce2cf20f4ada6236a8f0`.
- Manifest SHA-256:
  `33d573479e8de6e39ad1e43b06787f59777c6aab0acf1d031553b81122666e05`.
- Receipt: `~/re/gw2/notes/209026-live-cf13a3b-readback-20261106.json`.
- REA: `ev_d626d70589872c880095b542358cd6222f1706c66b10e4977bc4b68de468f0e5`.

Readback exited0 without truncation. Feed metadata remains b128 at05:38:39Z;
runner pin and audience were not reverified. The availability correction
from root's separate run214 is **not live**: its request had target0 and
did not demonstrate damage. No competing publication occurred.

The bounded retained audit of `run214-great-hunt-4` observes completed step
IDs1 and2, seven Thora/agent21 conversations and no Ido/Eir conversations.
The final screenshot shows defeat, not quest completion. In released code,
opening the relevant conversation advances a stage; dialogue completion,
a trophy item and trophy turn-in are not checked. The authored definitions
establish IDs, text references and sectors, not these reconstructed action
conditions or their sequence. Release prose must not treat that mapping as
proof of retail encounter rules. The run wrapper and release prose also
disagree on the boss position; neither establishes a verified retail spawn.

Private audit: `209026-run214-great-hunt-ledger-audit-20261106.json`, SHA-256
`626771245cb26a8040088206470cea617d5ed27fa384ed5af614c2546e83c6cc`,
REA `ev_4f88606d925b60eb5392437873b19b40eff4e27a61467b5cbd6d2a356a85a1c6`.

## Great Hunt waypoint carry: 4c46cad

The sole publisher deployed **2026-11-06T05:43:40Z** to
`/home/blizz/gw2-rust-4c46cad`, rollback
`/home/blizz/gw2-rust-cf13a3b`. It carries Great Hunt stage through a
same-process waypoint rejoin, recreates the quest and replays completed
steps in order. A finished hunt is not recreated. This does not add durable
restart storage or implement the displayed item/currency reward.

The publisher records115 observer tests including the carry regression;
its `run215-hunt-carry` exercised Thora, a trophy kill and regeneration
without asserts. That reported client smoke does not itself document a
completed waypoint round trip. The release cycle ran at zero humans and
became ready in3.4s. Root did not rerun the publisher's tests.

Independent readback at **05:45:12Z** confirms the selected kit, all manifest
hashes, four healthy owned listeners, saved-norn mode with one0700 record
store, unchanged activation backup and CDN900 root/main hashes. **One human
and zero bots were present.** That later active-player observation was sent
to the publisher; no root restart or deployment occurred.

- Game SHA-256:
  `229e0276e9fc84517d52b65d39c829fa418fea3a08e197f70d97da5fc3aa560a`.
- Manifest SHA-256:
  `180dd68505fd8e0ca64aff1767651384dd114ad21e4ee0d4872771a8feec99e9`.
- Receipt: `~/re/gw2/notes/209026-live-4c46cad-readback-20261106.json`.
- REA: `ev_6a2ceb040168d1399c230f1246ab54e1880e20a23e73e8c625d47c1918435d11`.

Readback exited0 without truncation. Feed metadata remains b128; this query
does not reverify the runner pin or clear the audience restriction.

A fresh **06:13:47Z** readback after the successful root216 sword smoke
still selects4c46cad with the same game/manifest hashes, healthy listeners,
saved record and backup. It reports **zero humans and zero bots**,
superseding the earlier active-player observation; rollout still requires
its own current player check. The tested skills/profile increment was
handed to the sole publisher for prompt small-release integration, with
no deployment claim. Receipt
`209026-live-4c46cad-post216-readback-20261106.json`, REA
`ev_763119c1fac80f762329211b5635c7475099eaa5a99388c96c24e6c56026521f`.

## Real sword request availability: 4c46cad-sk

The sole publisher deployed **2026-11-06T06:16:37Z** to
`/home/blizz/gw2-rust-4c46cad-sk`, rollback
`/home/blizz/gw2-rust-4c46cad`. This small increment adds605(10170,player1)
to the skills batch and explicitly clears effective `GW2_AUTOATTACK_SKILL`.
Damage30 remains enabled and diagnostic echo remains absent. All other
profile values are unchanged, including NPC positions; the nearby wolf
from root216 was not deployed. The binary is unchanged from4c46cad.

The [root216 smoke](209026-sword-slot-selection-runtime.md#run216-targeted-ordinary-request-and-one-damage-response-pass)
verified fresh hostile selection, a matching controlled10170 request, one
30-point255 response and no extra order/hit on release. This is the evidence
for the small correction, not full retail combat or authored NPC placement.
The separate world-handover/carry-identity work is not included in this kit.

Root independently verified at **06:19:28Z**:

- All kit hashes and four owned healthy listeners; zero humans/bots.
- Exactly17 skill bytes, SHA-256
  `4ceabe189b2b1fa26310a1339424b882cadfc03cd8480f26c7a55be0b2280678`.
- The running game process has no autoattack/echo option, uses the verified
  skills file and retains damage30.
- Saved-norn mode, one0700 record store and the unchanged activation backup.
- Retained rollback binary, manifest and original12-byte skills hashes.
- CDN900 root/main hashes and unchanged b128 feed metadata.

Game SHA-256:
`229e0276e9fc84517d52b65d39c829fa418fea3a08e197f70d97da5fc3aa560a`.
Manifest SHA-256:
`6530f73395ae9391a5275a4cd203c72c99600b937f56f476d38832ee38ffc2cb`.
Receipt: `209026-live-4c46cad-sk-readback-v3-20261106.json`, REA
`ev_096b6297087934b101279699a460e9600fcfb8de1df32bb0327435303fa12354`.
The helper exited0 without truncation. Its initial version failed locally
before SSH because of a quoting error; that failed receipt is retained.
The final comparison proves only the autoattack **value** changed; raw
profile text also has formatting/comment differences. No character contents
were read and root made no deployment. Feed runner/audience constraints
were not reverified or cleared.

## Private world-handover observation; not in 4c46cad-sk

The publisher's retained `run217-world-handover` uses prepared-character
mode and `--mission-hunt-start-stage 4` onf0a45a4. It observes a boss kill,
step5, the completion panel, one1065 transfer to map28, and a second
MapLoaded/AgentsLoaded/1036 sequence. The character renders afterward;
scripted M opens a map titled Wayfarer Foothills. No deliberate movement or
combat after transfer was exercised. This is not a full authored Great
Hunt playthrough or a saved-character runtime admission test.

The mission log records restored loot2 and XP5000/level3. Heart56's carried
tuple retains2/5. Its bootstrap log prints a literal0/5 regardless of state;
exactf0a45a4 source restores matching heart state on MapLoaded, then emits
1089 for nonzero progress after creating counters on AgentsLoaded. That
source path would send2 for heart56, without completion. Actual fractional
heart progress in the UI was not inspected, so the log's0/5 is not a reset
finding. The completion panel displays a weapon icon and72 copper; no
Accept click or actual reward-credit proof exists.

Arrival uses configured Hero's Moot waypoint coordinates, with no proven
authored tutorial-arrival selector. Source tests cover saved-GUID admission
separately. The publisher integrated root's carry-identity guard as3135b29
and the independent arrival regressions as76af245. These changes were
outside4c46cad-sk; the subsequent deployment is independently verified below.

Private corrected audit:
`209026-run217-world-handover-audit-v2-20261106.json`, SHA-256
`e7b8d9f1b7817783550cd623622f0ca15b365026a92e162f7c8908cec011b057`.
Complete retained evidence:
`209026-run217-world-handover-evidence-20261106.json`, SHA-256
`06e9259ac86f3f3cf6c232277674d59892f1603fbd93cadd4368fe4cba14ad8d`.

## World-handover release 76af245

The sole publisher deployed `/home/blizz/gw2-rust-76af245` at
**2026-11-06T06:21:31Z**, retaining `/home/blizz/gw2-rust-4c46cad-sk` as
rollback. Root's independent readback at **06:25:42Z** passed all kit sums,
four owned listeners, saved-norn mode, one private saved record and the
unchanged activation backup. Zero humans/bots were present. The actual
process uses map28 arrival `(597.3,-30352.1,-1563.2)`, 14 heart NPCs,
five world hostiles, four ambient NPCs, the 17-byte skills file and damage30.
Legacy autoattack and order echo are absent.

Game SHA-256:
`acf59ef479b37a92230432eedd5e95ae14247a7476c726c4cfd946566d2babf8`.
Manifest SHA-256:
`2d519c188e05bb071b13c7ac5021571ecb9215024beec8f62d2fd7888dfc0832`.
Rollback binary/manifest/skills hashes pass. CDN900 root/main hashes pass;
b128 feed metadata is unchanged. Its runner and audience were not
reverified, and this readback clears no launcher publication restriction.

Receipt: `209026-live-76af245-readback-20261106.json`, REA
`ev_1f17f3987e20ee61e5a9d35ac2bf9ca3cf18e5a2c5f853bc3552704ea45373a3`.
The helper exited0; root did not deploy or modify the saved store.

Retained run218 uses a prepared character at hunt stage4 and proves transfer
plus 14 NPC/384 pairs; no NPC conversation was exercised. Run219 remains
in the tutorial: four10170 SkillBegin requests and four30-point hits
118→88→58→28→0. The release prose's255-hit claim does not describe these
retained logs. These runs do not establish saved-mode runtime admission
through the combined transfer.

Current heart behavior credits each hostile kill to every unfinished heart
containing the player, toward a configured target5. The authored titles,
polygons and centres are recovered; action/species conditions are not.
NPC names reuse mail-sender text, appearances are substitutes and placement
uses area centres. Coins30, karma50 and uniform5147 XP remain placeholders.
The exact Raven-heart56 completion text matches mail95, supplying a bounded
next source target, not a proof of authentic progress conditions.

Scope receipt: `209026-wayfarer-heart-scope-and-raven-mail-seed-20261106.json`,
SHA-256 `6d5a811d17528e8ae84a05002d3541536f1737236880bdfaf6d6c41e98377861`.

### Raven-heart completion letter: recovered delivery contract

Server1001 provides a direct type43 content route for mail95. Handler
0x1413eb660 resolves the second Dword through content virtual198, stores the
definition on the parcel, appends/notifies the inbox and clears incoming
staging. Its six Dwords are parcel ID, type43 key, coins, unnamed parcel+84,
parcel flags, and unnamed parcel+8c. The definition supplies body30728,
sender30730 and subject26289. Body/subject formatting also requires the
local player, character and coreStats. No delivery has been attempted.

Ordinary content text requires `(parcel_flags & 0x0e) == 0`, which does not
establish the full flags value. Mail95's definition flag4 is separate:
copying it into parcel flags selects a gem-store branch. Both unnamed
fields initialize to zero, but this is not proof of their retail delivery
values. Server1004 clears+84; virtual98 exposes+8c without establishing a
timestamp meaning. A fresh parcel is item-empty; server996 can independently
add attachments, so absence of authored attachments is not established.

The literal alternative stages997 body **before**999 final header, with
the same parcel ID and no interleaved parcel. Exact reward amounts,
attachments, completion trigger, replay and persistence remain unresolved.
No defaults or gameplay sends were introduced.

Private contracts: `209026-raven-heart-mail-contract-20261106.json`, SHA-256
`ce9283a2dcf37e5c30576fc81adf294a89542981abfce35a71b8985ca62184be`, and
`209026-raven-mail-fields-followup-20261106.json`, SHA-256
`b2fc5d10a8fbb824c3d5e53ef2d506ba04c140b76b75a0c42c35803dc0963230`.
The complete REA contract evidence is retained privately as
`209026-raven-heart-mail-contract-evidence-20261106.json`, SHA-256
`ca11692b025566be020ec299f419fda993f56db882138703c6f4663d4e3b4b03`.

## Independent fix: interleaved character transfers

Commit **98bae4f7539beda97703778b69578328b9dd06af**, based on live0763c47,
fixes a second in-memory carry loss. The single global pending slot let a
newer A transfer erase B's pending state. The regression reproduces B losing
7000 XP/level4 before the fix. The replacement retains the newest pending
state for each character, checking destination and consuming only a matching
entry once. Unrelated/wrong-destination joins preserve pending state.

Eight focused tests, isolated musl build and four unchanged-profile
no-listener configurations passed. Only mission.rs changes; original415/
record bytes and the profile remain unchanged. No restart persistence or
disk format is added. Binary SHA-256:
`a99d389615ed94c5169c30cb7f6bd139de2e65bf4195834d6c50b00ae626c3d1`.
Verification receipt:
`interleaved-carry-0763c47-20261106-checks/verification-receipt.json`, SHA-256
`f5d91b1fb200ee1862d44bed53d201b449b5fc9f328065b7f03fb444ca1e59d0`.

The delegated lane submitted this artifact once to the verified native
publisher. At07:06:01Z integration/publication remained unconfirmed. The
unpublished discovery branch's third waypoint producer must use the new
queue helper during integration; this is documented in the handoff and is
not a dependency of the live-base fix. Submission receipt:
`interleaved-carry-native-publisher-handoff-20261106.json`, SHA-256
`f32724d4e62aa8141aaed086e38c009d5fd89a89832f768b0ea4cc0277e8d251`.

## Verified live release b2d6d24

The sole publisher integrated98bae4f and adapted the discovery waypoint
producer to the same queue helper. Release **b2d6d249bb02e1654e76dcd60f2caa6b29b4b6e6**
was deployed **2026-11-06T07:17:51Z** at
`/home/blizz/gw2-rust-b2d6d24`, retaining0763c47 as rollback. It also enables
the publisher's in-memory wallet and preserves the earned-attribute increment.
Vendor and POI discovery implementations are compiled but their flags are
absent from the observed live process. This is not a quest154 payout release.

Independent read-only REA verification at **07:23:22Z** passed all current
and rollback manifest sums, four owned listeners, zero humans/bots, effective
default-curve/damage30/17-byte skills/world28 settings, saved-norn mode,
one0700 saved record and unchanged activation backup. No character contents
were read. Legacy autoattack/echo remain absent;14 heart NPCs,5 hostiles and
4 ambient actors remain configured. Wallet is enabled exactly once.

Game SHA-256:
`7b8fa261ee871ef65e5a6129ec4863ec6f6e2bc90cb55cc1b168d9bdb38cb66c`.
Manifest SHA-256:
`225c4cd41a2ce7d3e167b1a9ac6d36fbbedcfe68c5fe8a44773ea83a9124a435`.
Effective profile SHA-256:
`c4fb0754613e3ab77984424729fe6c105166634b234d738b500e57a7c2880fe3`.
Receipt: `209026-live-b2d6d24-carry-wallet-readback-20261106.json`, SHA-256
`a404eb994a888d1273591799abc9c4cd5d18b695cb89f36c646abd298ce81922`,
REA `ev_f4f56181c1c7d3aee54dfc713bc3af01154566705cd205e7f0ad602a0a23d2bb`.
The capture exited0 without truncation and verified owned cleanup.
This readback does not establish client reward interaction, full retail
economy, restart persistence or any launcher audience change.

## Verified live release b2d6d24-poi

The sole publisher deployed the profile/data/supervisor increment at
**2026-11-06T07:25:26Z**, kit `/home/blizz/gw2-rust-b2d6d24-poi`, rollback
`/home/blizz/gw2-rust-b2d6d24`. The game binary remains the exact7b8fa261 hash
above. Supervisorf26865e forwards the configured map28 POI file; wallet stays
enabled and vendor stays disabled. Discovery radius/threshold policy remains
inferred, so this does not prove full retail discovery rules.

Independent readback at **07:28:02Z** passed full current/rollback manifest
sums, four owned listeners, zero humans/bots, saved-norn/one0700 record,
unchanged activation backup and the earlier attribute/combat/world/skills
settings. Exact configured POI file SHA-256:
`16ab02c8cb0aafe63c9865cf34090959adbd9c0b00211195318c7f85f13e485c`.
New manifest SHA-256:
`e2aee95ea30a2b6665d1ed92b853759402c005cb41a771bb3908b9004e27caf6`.
Effective profile SHA-256:
`ab67b28beccb0eee389c9b26dc64b63f6d12a8158b69e92e86e04e8d501bc61b`.

The bounded readback of `run224-discovery-move` observes one map28 transfer,
two MapLoaded/BeginPlaying cycles and one Hero's Moot waypoint371 discovery
with1140/1143. The script moves and opens the map; it does not click a
waypoint. No client208 or waypoint1065 rejoin is present. Waypoint travel
therefore retains offline-test coverage only. Run logs record completed
cleanup, unchanged PE hashes and console privacy pass. Absence of assertion
text is a bounded log result, not a general gameplay correctness proof.

Receipt: `209026-live-b2d6d24-poi-and-run224-readback-20261106.json`, SHA-256
`2b8e4ee40db9c3f80c216afefcc49d1e6ae1c178447a3352ba6418b1843b4ab6`.
Live REA: `ev_c5867ecd65863879dc027c84d9d3f72cd98e0390bafd06cadbd0e82b37dddb3a`.
Run224 REA: `ev_92cff1c4debbfc5d0dc4dc64f07f654ec633e4b84fafb4939a8e619b9818fc43`.
Both read-only captures exited0 with owned cleanup. Unrelated processes whose
environment could not be read remain unverified and were untouched.

## Raven-heart task conditions: prose locator, server-authored behavior missing

The retained decoded text catalog's entry84428 mentions finding raven eggs,
returning them to roosts, defeating skelk and destroying nests. Entry25956
describes a fallen Raven Egg;621628 mentions returning an egg to a nest in
Wayfarer Foothills. These are useful locators, but the old catalog lacks
per-row raw/width hashes. Exact packed-record revalidation remains pending
an authorized archive window. No typed heart56-to84428, species or task-credit
binding was recovered, so these entries do not authorize guessed kill rules.

Fresh cached-record REA checks confirm that the exact type72/287 records
expose title/mail/level and polygon references, with type72+44..54 zero.
The native657 handler14123fb40 calls14123e1d0, which copies two received UTF16
strings to subregion+10/+30 and stores a runtime objective ID at+6c. The
subregion table's+8 getter independently derives the title from definition+30;
getters+10/+18 return the received strings. Objective counters arrive from
the server. Neither these records nor this UI consumer supplies egg/roost/
skelk/nest instance bindings, credit weights or completion thresholds.

The generic kill-based heart fixture therefore remains explicitly incomplete.
The next source dependency is concrete authored action/instance binding and
credit rules, rather than another generic kill handler. This investigation
made no archive access, gameplay change or runtime launch.

Private findings: `209026-raven-heart-task-conditions-20261106.json`, SHA-256
`4a77b563aee144c39ac5ce8af372cf13f72fc749d6493f7e82041ffb118298fb`.
Complete REA bundle:
`209026-raven-heart-task-conditions-evidence-20261106.json`, SHA-256
`d381abeb55b9b5806e24e135bf60dcae568651b21ff8273fcdcf38a301c62cc2`.

## Verified live release 9326c08

The sole publisher deployed **9326c08b19fdb375fa24e4ae97cbec1782bf0f4f** at
**2026-11-06T07:45:14Z**, kit `/home/blizz/gw2-rust-9326c08`, rollback
`/home/blizz/gw2-rust-b2d6d24-poi`. Source ancestry contains exact reward
9914a835 and the publisher's wildlife-cap changebe1f777. This is not a
reward-only binary. The profile now configures20 world hostiles and12 ambient
actors; this does not authenticate their retail identities/placements.

Independent REA readback at **07:46:42Z** verifies all current/rollback
manifest sums, four owned listeners, zero humans/bots, saved-norn/one0700
record and unchanged activation backup. Default-curve, damage30,17-byte
skills, wallet, POIs and heart data remain effective. Legacy autoattack/echo
and vendor remain absent. No character contents or configuration were changed.

Game SHA-256:
`48eb3ab86335a042f1efe196435f99d32c78527e4f87d552c0b5f43663a1bbbd`.
Manifest SHA-256:
`dea42c447de9e77843e378e62105ee4c14ccc3d12359488017dc2c17e5a2a6ca`.
Profile SHA-256:
`a22103983932fe53fce1c75abe2b128351eec1f8048aef539c1d6fc783f2825a`.
Receipt: `reward-wildlife-9326c08-release-verification-20261106.json`, SHA-256
`22a950bf47889a88dd3d55f4a46a0419a4d76a39bbf1bba7e169af3cde7a3e22`.
Full REA receipt: `209026-reward-wildlife-release-readback-20261106.json`,
SHA-256 `849da57bc66b05aadd270aee6c3828b336c7bd40f076b0ed863c68f812b0bd3e`,
evidence `ev_bf047a5dd912cfa8ed0d15a49d7d740a49116d2b0c9e50c1e9d89234ea1d2f7a`.

This verifies deployment/source/profile/health, not a replayed native reward
click. The publisher's retained payout smoke is audited separately below.
The [fixed-payout record](209026-quest154-content.md) preserves its focused
tests, native-default item choices, unchanged XP and process-local limits.
Original character files remain intact; gameplay restart persistence is absent.

### Retained publisher payout smoke

The separate `run226-hunt-payout` uses prepared auth/mission records, no
character store and hunt stage4. It is distinct from root's Ether Feast
`run226-skill10176-finish`. One client205 claim is logged, followed by one
credit log and a successful33-byte network write. The selected key is not
logged; selected0 follows from the exact9326 source guard, not decoded wire.
The959/276/204/526 batch is established by pinned code plus its successful
write. Raw frame capture was disabled, so it is not independently decoded.

Admitted screenshot inspection shows a completion panel with reward icon,
72 copper and Accept; after acceptance the panel is closed and a72-copper
toast is visible. Inventory shows2/20 slots, the matching weapon icon and72
copper. After approximately12.393 seconds the server transfers379→28, applies
the carry, restores two allocations and reports wallet72/0. A later image
shows the avatar in the new world; its wallet/item are not visible there.
Thus native pretransfer presentation and logged post-transfer restoration
have distinct evidence boundaries.

No duplicate claim, arbitrary disconnect, restart or saved-mode restore was
exercised. Run logs retain cleanup, executable-hash and console privacy checks.
The audit capture verifies its own process cleanup; an unrelated EACCES
process probe remains unverified and was untouched.

Receipt: `publisher-run226-payout-scope-20261106.json`, SHA-256
`aafd38ad43f5a89a20c62cb5a6915a902f965df1fed09b4657227ef3e4fd285e`.
Full REA: `209026-publisher-run226-payout-audit-20261106.json`, SHA-256
`3ea560aa1924fdaf81b60cc17617822eb0f314d32e83c611a834d96386ab6c17`,
evidence `ev_ade464c54f1eb60e6edc36974a8804b2528142b2e937d14d39d2a5e6d7d2dba5`.

## Verified merchant profile release: 9326c08-vendor

The sole publisher deployed `/home/blizz/gw2-rust-9326c08-vendor` at
**2026-11-06T07:55:30Z**. This enables the tutorial merchant in the profile
without changing the preceding9326c08 binary. Rollback is
`/home/blizz/gw2-rust-9326c08`.

Root independently verified at **08:05:25Z**:

- Current and rollback manifests and every listed file match their pins.
- All four listeners belong to the selected kit's processes; health is
  ready/running, with zero human and bot connections at readback.
- Saved-norn mode, one private record and mode0700 remain intact; the
  activation backup hash is unchanged. No character record contents were read.
- The exact merchant argument and profile match the staged kit. Wallet,
  default-curve attributes, damage30,17-byte skills, map28 POIs and
  14 heart NPCs/20 hostiles/12 ambient actors remain configured. Legacy
  autoattack and skill-order echo remain absent.

The merchant uses the Eir composite as a stand-in. Its appearance and
placement are implementation choices, not recovered retail merchant data.
The retained drag-sale smoke is audited below; this deployment
readback does not itself replay that interaction. No client feed,
audience, access rule, schema or server configuration was changed by root.

Game SHA-256:
`48eb3ab86335a042f1efe196435f99d32c78527e4f87d552c0b5f43663a1bbbd`.
Manifest SHA-256:
`236443586b07020349ae2e0489acaddf6d5a43d2008f5729173cee0193352a75`.
Profile SHA-256:
`d564afee8cacaf1a35aa2c92b91670ed4d173f69aed99d11738533e4ab018f0f`.
Private receipt: `vendor-9326c08-release-verification-20261106.json`, SHA-256
`90b4cea819768ac93622babd399ebdf667420e9564c9828a6241e091c41c6f41`.
Full REA: `209026-vendor-release-readback-corrected-20261106.json`, evidence
`ev_d6a85c250e5f0d6c17ea6b07fee66f1ed5f43f046c408066472456620f27c218`.
The helper exited0 with verified cleanup. An earlier invocation rejected a
mistyped rollback digest locally, before SSH; its failed evidence is retained.

### Retained merchant sale smoke

The separate prepared-character `run228-vendor-drag` contains one decoded
510 Sell request for item instance28432, count1 and expected total2. The
server identifies item key19526, removes slot0, credits2 copper and logs
295/204/1279(result1), followed by a successful24-byte write. No plaintext
response capture exists; the response batch is supported by exact source,
typed logs and successful write. Hash-admitted native screenshots show the
inventory changing from1/20 with0 copper to0/20 with2 copper and a2-coin toast.

The retained wrapper records cleanup, privacy and unchanged executable hash
checks. It does not retain a numeric client exit code; its runner wait uses
`|| true`. This audit made no fresh client, container, executable or archive
query. There is no duplicate-sale, buy/buyback, saved-character transfer,
relog or restart observation. The single item's expected price agrees with
the server price; this does not establish every merchant price or catalog.

Private receipt: `publisher-run228-vendor-drag-scope-20261106.json`, SHA-256
`8797d6070307f9f74d2b82bf5d56c9daf6db4a9219e5ab1baf2f9ad678f960b3`.
Complete six-capture REA bundle:
`209026-publisher-run228-vendor-drag-audit-20261106.json`, SHA-256
`32655685600c0e98a9f7278122cc4bda1a8171d88aa7ceaadedbb5e0502e79a6`.
Primary log evidence:
`ev_0e26eaaf63037474a58e9c2ed0ad5d1cebf2d32604940c004a317756ccd56b97`.
All audit processes exited0 with complete normalized frames and verified
owned cleanup. Raw evidence remains private.

## Verified carried-XP order release: a223a90

The sole publisher deployed the exact root-reviewed artifact as
`/home/blizz/gw2-rust-a223a90` at **2026-11-06T08:41:57Z**. Rollback is
`/home/blizz/gw2-rust-9326c08-vendor`. This changes the order of existing
map-rejoin frames: carried XP/earned stats precede replayed quest completion,
so an immediately initialized reward panel reads the restored XP.
The [source binding and regression](209026-quest154-content.md#reward-initialization-timing-and-carried-xp-ordering-fix)
record the failure-before test,21 distinct passing focused tests, musl build
and four startup checks. No XP award policy or storage schema changes.

Root independently verified at **08:43:03.537619Z** that the current link,
exact executable, full current/rollback manifests and all listed files match.
All four listeners belong to the selected kit's processes; health is ready
and running, with zero humans/bots at readback. Saved-norn mode, one private
record, directory mode0700 and the activation backup remain verified. The
reader does not open character records or mutate configuration/service state.

The profile is unchanged: vendor and wallet enabled, default-curve attributes,
damage30,17-byte skills, map28 POIs and14 heart NPCs/20 hostiles/12 ambient
actors; legacy autoattack and skill-order echo remain absent. Existing fixture
XP and encounter limitations remain. No new native map-rejoin test or durable
progression is claimed. The publisher's separate quest162 branch is not live
in this exact artifact, and no launcher channel/audience changed.

Game SHA-256:
`d1d09f9bf4c91c547ccfca37a61719dd0394d609b22fe662553b2fe4aef89a6e`.
Manifest SHA-256:
`98c371dc836457d8749cd639ec0e8060645db1d1e333af5ebc97414d49fe40bb`.
Profile SHA-256:
`d564afee8cacaf1a35aa2c92b91670ed4d173f69aed99d11738533e4ab018f0f`.
Private verification receipt: `a223a90-release-verification-20261106.json`,
SHA-256 `b4d9ec1447407b2de8e97f57cabbc76580dc7cca0508de05f2f292362acc4063`.
Full REA: `209026-a223a90-release-readback-20261106.json`, SHA-256
`d28601b6dc1edec3281135ba2d754148774180772cda047b627eaaf7cd18d6b4`,
evidence `ev_9728d30a086bee2365a644f295adbad80a94a8b31e5d95ea9fe8b32428a00d97`.
The read-only verifier exited0 with verified owned cleanup and complete output.

## Verified Ether Feast release: 7c26f45

The sole publisher deployed the exact root-reviewed artifact as
`/home/blizz/gw2-rust-7c26f45` at **2026-11-06T09:24:16Z**. Rollback is
`/home/blizz/gw2-rust-a223a90`. This enables an owned no-clone Ether Feast
self-heal and20-second recharge, guarded health/cooldown carry, and a corrected
original-roster login redirect to the configured carried destination.
The [implementation record](209026-ether-feast-reconstruction.md) preserves
the chosen scheduling policy, focused tests, actual TCP failure-before/pass-after
relog regression, musl build and eight off/on startup checks.

Root independently verified at **09:27:43.186242Z** that the current link,
exact executable, current and rollback manifests and all listed files match.
All four listeners belong to the selected kit's processes; health is ready
and running, with zero humans/bots at readback. Saved-norn mode, one private
record, directory mode0700 and the original activation backup remain verified.
Character contents were not read and the verifier changed no live state.
The effective Ether Feast option, paired supervisor and unchanged17-byte skills
fixture match. Vendor, wallet, damage30, default-curve attributes, map28 POIs
and14 heart NPCs/20 hostiles/12 ambient actors remain configured; legacy
autoattack and skill-order echo remain absent.

The first readback exited1 at `exact_profile`: the publisher changed comments
and blank lines in the tested profile. Root then established that every other
line was byte-identical and in the same order, retained that comparison, and
reran with the exact published profile hash. The corrected readback exited0
with complete output and verified owned cleanup. The earlier failed check is
retained as a failed expectation check, not relabeled as a pass.

Game SHA-256:
`89d68f855ba34a42eebe963578a25aac3191f8f53d668f2d190da08b06347088`.
Manifest SHA-256:
`b392b579d0f1c466cd342653bbd3e4c6b3c65b78539daf7fa530b79c1d63a0b3`.
Published profile SHA-256:
`4dd3519314b77ff7abbd117ab71cc6198da27df614161e04c2d95b0a2193c180`.
Private verification receipt: `7c26f45-release-verification-20261106.json`,
SHA-256 `925766c70246eebc3d200858a04f5220f6593c2909b5e3196f9a5285b45a8f29`.
Full REA: `209026-7c26f45-release-readback-corrected-profile-20261106.json`,
SHA-256 `d3c73babcf88a861ea00ac6a411e9aff40a7a3fbf626ba0bd5baea11edb0f884`,
evidence `ev_f55126920cf34a21d00b0614726c8f68c05c3390626d294ec74db0d537d1866a`.
Profile comparison: `7c26f45-profile-comment-comparison-20261106.json`,
SHA-256 `d5f911f6e9e2df3a39d15c7cf1211c12a3c9cd8f9407bc32d2ef460838c7f81e`.

This readback establishes deployment and service health. The separate retained
run233 audit below establishes a bounded native cooldown smoke, but no positive
health gain. Manual cancellation, clones/conditions/modifiers, effective-level scaling
and restart progression remain unsupported. The separate quest162 branch is not
included in this artifact. No launcher feed, audience, frozen client or schema
changed. The pending owner storage exception and full retail completion goal
remain open.

### Retained run233 cooldown smoke

The independent **09:35:13Z** REA audit of the publisher's retained
`run233-ether-feast` uses the same game SHA-256
`89d68f855ba34a42eebe963578a25aac3191f8f53d668f2d190da08b06347088`.
Its prepared synthetic character sent two controlled10176/slot0 requests.
Three scripted key6 inputs produced a visible19 countdown, a middle retry with
**“Skill recharging.”** and16, then a later request and new19 countdown.
The middle retry has input/release records but no10176 SkillBegin in the retained
server log. This is positive native cooldown presentation/retry/recast evidence.

It is **not a positive-heal smoke**: server4.log:134 records regeneration350→388
before the first10176 request at147, and the before/after screenshots all show388.
Four generic scheduled-update records do not identify four heals. The matching
source's full-health branch emits630 and later84/45, without positive255; that is
source interpretation rather than a plaintext outbound-wire capture. Exact
native effect timing, health gain, saved relog/carry and retail equivalence are
not established. See the [full bounded audit](209026-ether-feast-reconstruction.md#retained-run233-native-cooldown-retry-and-recast).

The retained wrapper reports cleanup, executable hash preservation and console
privacy checks; no assertion/crash was found in the bounded client/wine scan.
No fresh container/live check or owner-store read was made. The audit reader
exited0 with complete output and verified owned cleanup.

Private files under `/home/blizz/re/gw2/notes/`:

- `209026-run233-ether-feast-audit-20261106.json`, SHA-256
  `d8821c5a8a6c9dc2c8030da25b1c752c96acfc8c653cf00752d62ab3c4962b29`;
  REA `ev_e8b76dbfee11394ca6140bcc88c74294c455ef6632dbe7f1b5e4b7d875af0987`.
- `209026-run233-ether-feast-audit-receipt-20261106.json`, SHA-256
  `fb59d10e7a7dcd81b4403dfcee9d7e7b8b973c759658e581e886b5281a326a73`;
  exact binary, source, log and four screenshot pins are retained there.

## Verified sword-chain release: 7cfe9c4

The sole publisher deployed the exact reviewed7cfe9c4 artifact to
`/home/blizz/gw2-rust-7cfe9c4` at **2026-11-06T11:38:42Z**, retaining
`/home/blizz/gw2-rust-e624c8b` for rollback. Live was still the exact e624c8b
base, so this did not overwrite newer live work. The publisher's later quest162
branch remains separate and must preserve this increment when integrated.

The [opt-in implementation](209026-mesmer-sword-chain-reconstruction.md) advances
the three sword selections after accepted hits and resets with the actual
predecessor/count0 on an admitted heal or loss of alive state. Rejected requests
leave combat and pending healing unchanged. The candidate passed29 focused
checks, musl build and two saved-profile no-listener startup checks. Only the
explicit chain option and two additional605 grants change the effective profile.
Hit-driven progression/reset are reconstruction policy; strike timing, damage,
expiry and full retail mechanics remain unresolved.

Root's independent **11:43:05.582617Z** readback verifies the selected kit,
running executable identities, every current/rollback manifest entry, four
owned healthy listeners, both chain and Ether Feast flags and the exact27-byte
skills fixture. Saved-norn mode, one private record, directory mode0700 and the
original activation backup remain unchanged. Record contents were not read.
Zero humans/bots were present. Damage30/default-curve, vendor/wallet/POIs and
14 heart NPCs/20 hostiles/12 ambient actors remain configured. Autoattack and
echo remain absent; `story_branch` is `none`.

| Deployed input | SHA-256 |
| --- | --- |
| Game,1834928 bytes | `b411840dea19760e77f826681bd0a2fbe759b9313b1e556b04ff62250f031df2` |
| Manifest | `c0bdd0f4a70a2d32ffd1d05b59ca2a07fd6111db4d5b04c28dfbf9cb5741c190` |
| Profile | `c62fa699d43c7e1be49f8c4b6e1331e168b6658e87bf8bd3574a662ca8be2ac8` |
| Supervisor | `831ea70a1f3bdb2628248e805775c83ff102ccedbe2b3cd15e21fa0dec9629ce` |
| Skills,27 bytes | `7a8f8c7be4e52f646aec90b0edfaf66feceb6a800d52354cf78fad8f9809d7b1` |

Private receipt `7cfe9c4-release-verification-20261106.json`, SHA-256
`7f37451e9295022898eb21c640820492933687be39da78cb9809a16840d220a7`.
Full REA `209026-7cfe9c4-release-readback-20261106.json`, SHA-256
`b246755e246155f2850e06833699c640763940649add9bc42b993364682e1b92`,
evidence `ev_cd2a4c15dbd90c1ab71e9f50e7c6a7cc66b1ee1af1c156da1da8174fdbc1b069`.
The read-only verifier exits0, untruncated, with verified owned cleanup.
No launcher/feed, audience, frozen client, owner record or schema changes occur.
The native three-tap smoke was pending at that release readback. Subsequent
[run241](209026-mesmer-sword-chain-reconstruction.md#run241-native-three-tap-cycle)
observes the exact live artifact's three native skill requests, corresponding
hits and slot icon cycle; its sandbox returned cleanly at11:51:19Z. That prepared
fixture observation does not establish saved-mode/full-hunt or retail mechanics.

## Verified movement-diagnostic release: 403ac37

The sole publisher deployed exact403ac37 to `/home/blizz/gw2-rust-403ac37`
at **2026-11-06T11:54:07Z**, retaining7cfe9c4 for rollback. This increment adds
`--mission-movement-log-every N`, accepting1..1000 and defaulting to20. The
default predicate remains exactly count%20==1, including wraparound; N1 logs
each report. The option changes log selection only, leaving movement decoding,
replies, timers, gameplay state, placements and client hooks unchanged. No
supervisor/profile option enables denser logs on live.

Root reviewed the three-file diff and11 pinned receipt inputs. Four focused tests
cover default/wrap compatibility, dense/max selection, invalid options and
unchanged decoded movement/replies;15 binary CLI/no-listener checks and the
static musl build pass. The common phase application follows prepared/saved
destination selection; the creator connection has no world phase. The newer
publisher story branch remains separately integrated and unpublished by this
exact artifact.

Independent readback at **11:56:13.566234Z** verifies the selected/running binary,
all current and rollback files, four owned healthy listeners, saved mode/one
private record/mode0700 and unchanged activation backup. It checks that the
movement-log override is absent, preserving the reviewed default20. The exact
profile, supervisor,27-byte skills, chain/healing options and existing world,
vendor, wallet and POI inputs match7cfe9c4. Zero humans/bots were present.
No record contents were read or configuration changed.

Game:1834968 bytes, SHA-256
`123d90c3cd5ea8635d919d232306e5180e32c2044a75333870e8448c5ce26f08`.
Manifest SHA-256:
`3a01dab51b0f8132ee60cc5bd94a36660c24136744243abe46bf01c9a53ff082`.
Implementation receipt `movement-log-every-7cfe9c4-20261106/receipt.json`,
SHA-256 `6da6488d9452ca413df0149faa63272dc281bf060937c879790e0af903d05818`.
Release receipt `403ac37-release-verification-20261106.json`, SHA-256
`fc5dd8dc67245510f66fe72212f8203dea9d5e22904652c40004d88514b5c506`.
Full REA `209026-403ac37-release-readback-20261106.json`, SHA-256
`078926bc5df58278b8bc3e41270f7484d17491c7d089bc9974e2a0f2a8f95d70`,
evidence `ev_f74d29a186b520deaa5798fcebad81479634a1fb07908d16a4e965148d0da163`.
All receipts remain private under `~/re/gw2/notes/`; readback exits0 with complete
output and verified owned cleanup. Run241's native combo evidence is specifically
on the preceding7cfe9c4 artifact. No403 native route traversal, full-retail,
durable-progression or launcher-feed claim follows from this diagnostic release.

## Verified sword2-observation release: f926ce0

The sole publisher deployed exactf926ce0 to `/home/blizz/gw2-rust-f926ce0`
at **2026-11-06T13:00:46Z**, retaining403ac37 for rollback. Its default-off
`--mission-observe-mesmer-sword2` option is fixture observation tooling: it
grants10334 after an actual earned-level188 reaches2, then rebuilds the skill
bar once. Chain admission still rejects10334 before combat effects. The
option is **disabled live**. Profile, supervisor, skills and gameplay defaults
remain unchanged. [Implementation and checks](209026-mesmer-starter-skill-requirements.md#earned-level-observation-increment-f926ce0-verified-live-disabled).

Root independently verified the release at **13:03:57.637653Z**: exact running
artifact, all current/rollback manifest sums, four healthy owned listeners,
saved Norn mode, one private record/mode0700 and unchanged activation backup.
Zero humans/bots were present. Both diagnostic flags are absent, preserving
movement-log cadence20 and sword2 observation disabled. Ether Feast, sword chain,
wallet/vendor/POI, world route and population inputs remain as reviewed.
No record contents were read or configuration changed.

Game:1846128 bytes, SHA-256
`bffb89e2d20b68cc234520017395006940e79ee96d0bf4f73091b309e4a16d3f`.
Manifest SHA-256:
`015d69b6b61715e5348b0f4611c6482afc0b503fbd7e3af215823a01ef443446`.
Private scalar receipt `f926ce0-release-verification-20261106.json`, SHA-256
`bf057d5e7ee5391175e8a9c21c345ab84c5e0c735ee37c2a968d65ab6a9a02a7`.
Full REA `209026-f926ce0-release-readback-20261106.json`, SHA-256
`25484b6b6921e3ef23818e40a8823170d89bae397f0a5e502e312e3fd67922c8`,
evidence `ev_d65e78a4b604542f85b156827281c9e796c462ff82c1a2b157e0352d24aa5466`.
Readback exits0, untruncated, with verified owned cleanup. This establishes
publication of the tested binary, not native10334 execution, full starter
completion, durable progression or a launcher-feed update.
