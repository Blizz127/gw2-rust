# Saved Norn creation: live activation and independent readback

Saved character creation is live, currently on **4b3c4c2** at
`/home/blizz/gw2-rust-4b3c4c2`, independently verified04:53:03Z.
The [latest increment](#caster-guard-and-eir-appearance-4b3c4c2) ships the
caster guard and Eir's composite appearance. Saved mode was first
activated on82d8d04-cdn900.
The sole publisher records the owner's
native choice at **2026-11-06T03:29:03Z** and activation at **03:29:33Z**.
Root independently verified the configuration, live hashes and service at
**03:43:51Z**. The earlier unanswered roster prompt is historical.

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
