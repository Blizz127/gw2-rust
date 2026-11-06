# Saved Norn creation: live activation and independent readback

Saved character creation is **live on 82d8d04-cdn900** at
`/home/blizz/gw2-rust-82d8d04-cdn900`. The sole publisher records the owner's
native choice at **2026-11-06T03:29:03Z** and activation at **03:29:33Z**.
Root independently verified the configuration, live hashes and service at
**03:43:51Z**. The earlier unanswered roster prompt is historical.

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
window24 before launching run145. The later log inspection below establishes
that run203 ran and reported cleanup, but does not establish a combat pass.

The later [caster-ownership candidate45c0731](209026-skill-caster-ownership.md)
is checked and handed to the sole publisher. Neither that handoff nor the
publisher's5516cb4 preparation is a newer deployment receipt.

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
45c0731 candidate remain undeployed in the available release records.
The slot-input corrections must not be replaced with guessed damage merely
to make the smoke appear to pass. Run145 still requires a fresh resource grant.

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
