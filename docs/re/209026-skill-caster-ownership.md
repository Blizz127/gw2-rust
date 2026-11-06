# Build 209026 fixture: validate the requesting skill caster

## Independent release candidate on the current live source

**9451cf084cd1896824bac165b3ebf9adc7b20291** applies the same seven-line
SkillBegin ownership guard directly to live source **c2cb6fd**. It preserves
the existing diagnostic autoattack, profile, supervisor and Ether Feast
damage exclusion. It avoids the cumulative candidate's unresolved sword
activation regression. Only `mission.rs` differs from c2cb6fd; removing the
guard and its tests reproduces that baseline file byte-for-byte.

Two wire-driven regressions fail before this guard and pass afterward.
Eight focused tests pass in total: ownership2, autoattack2, echo1,
kill/respawn1, counterattack1 and Ether Feast exclusion1. The isolated offline
musl release build passed. Four no-listener configurations passed at
**2026-11-06T04:03:38Z**: prepared/saved auth and prepared/saved mission,
using the unchanged live profile. All44 baseline kit hashes and the paired
supervisor match. No shared client, container or archive was accessed.

- Binary:
  `/home/blizz/scratch/gw2-caster-ownership-c2cb6fd-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`
- Size:1582032 bytes; SHA-256:
  `721ad5fed8171983cf6a7e69fe8203979b462ce85cc60ec184fbe9d87af39b18`.
- Source worktree:
  `/home/blizz/scratch/gw2-caster-ownership-live-base-20261106-src`.
- Unchanged supervisor SHA-256:
  `6bbecb3d0042dab27bb3252d6b933f81553e090acd50d94860369a4b4fdf0ca6`.
- Unchanged profile SHA-256:
  `3e5699820e0fc7c752485f81a04eddd52994eb6f40dc71971ed788022d6d043f`.
- Preparation receipt:
  `~/re/gw2/notes/caster-ownership-c2cb6fd-preparation-receipt-20261106.json`,
  SHA-256 `2b2e14824623e90a0d7e403e924b5802a610d409a20386a93ccc35ebf1836119`.
- Startup receipt:
  `~/re/gw2/notes/caster-ownership-live-base-startup-20261106.json`,
  SHA-256 `f15605cdbadcb6ea2a19b14aa17f1f40cf788b50183e0ad903668ec96400e464`.

This independently checked candidate is handed to the sole publisher via
`creator-lane-peer-handoff.json#/root_caster_live_base_ready_20261106`;
the coordinator notification is queued. It is **not claimed deployed**.
Keep the existing saved store/mode, metadata900, player checks and rollback.
The guard covers `Seen::Skill` only; the legacy `RepeatStart` and other
action classes remain outside its scope. This preserves the current fixture
while the upstream sword investigation continues; it does not establish
retail combat accuracy.

## Original cumulative implementation

Commit **45c07310077968ec74e4f4bb176ad22851ed97ae** rejects a client
SkillBegin when its outer agent index differs from the character controlled
by that connection's population fixture. The check precedes target tracking,
the Ether Feast exclusion/echo, damage and NPC-cast responses.

Previously a foreign agent could kill a fixture NPC, award XP/loot/heart
progress and provoke retaliation as the connection's player. The echo path
could also issue an order using the foreign index. This was reproducible
with actual encoded client23 frames, not only a direct Rust event.

The [client producer](209026-predicted-skill-order-lifecycle.md) checks
`m_isControlled`; the fixture grants control with server83 to its own
`plan.character_index`. This fix enforces that existing connection boundary.
It does not add control of clones, vehicles or other agents, or select new
retail skill timing, damage, healing, cooldowns or unlock rules.

## Focused verification

Both new regression tests failed before the guard and passed afterward.
They exercise foreign indices0/3/65535 and sword10170/heal10176 across
damage+echo, NPC-cast, echo-only and target-only configurations. They verify
no reply and no target/health/aggro/retaliation/respawn/NPC-order/XP/loot/heart
mutation. A separate fixture controls agent7: agent1 is rejected, while
agent7 still receives its order and hits the NPC. This guards against
hard-coding the usual player index1.

Nine focused tests passed: the two ownership regressions, five existing
order-echo tests, the Ether Feast damage exclusion and independent NPC
kill/respawn. The private test receipt records observed tool results and
failure excerpts; it is explicitly not a retained raw terminal log:
`~/re/gw2/notes/skill-caster-ownership-tests-20261106.json`, SHA-256
`de7b51008a7e0883ad91d950cd169e6a64dc2c6d0dcc86e553c750b41f3a5359`.
The tested `mission.rs` SHA-256 is
`6eafe3015ca57a173939191c8cb4356f0f535bdc8f100c830c47c50950b4c8f4`.

An isolated offline musl release build passed in12.62s. Four no-listener
startup checks passed at **2026-11-06T03:45:50Z**: auth, prepared default
curve, prepared fixed37 and saved mode with a private empty store. The
retired CLI rejection, paired supervisor syntax/compatibility checks and
eight pinned fixture inputs also passed. No shared client or archive ran.

## Checked artifact for the sole publisher

- Binary:
  `/home/blizz/scratch/gw2-caster-ownership-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`
- Size: **1582824 bytes**; SHA-256:
  **1359b524960f89284d2f292a349e38c0c5d5f241c7d0f5df1781f037154cc4e3**.
- Source worktree: `/home/blizz/scratch/gw2-caster-ownership-20261106-src`.
- Required paired `deploy/gw2-server.sh` SHA-256:
  `c006b89524e011039a9b5bdb4df913d2505e82a9f44f7e69a31cd02b8735b6ad`.
- Existing candidate profile:
  `~/re/gw2/notes/mission-profile-attribute-curve-20261106.conf`, SHA-256
  `d7e25b1da72a9354f7cf84c7d46ad893158fac3c24a1cda13a68d73a6a9ee3ef`.
- Complete startup/input receipts:
  `~/re/gw2/notes/caster-ownership-build-checks-20261106.json` and
  `caster-ownership-build-inputs-20261106.json`.

This cumulative candidate includes5516cb4 and adds no profile setting or
schema. Preserve the now-live saved mode/store, publisher baton, active test
inputs and normal player/backup/rollback/hash checks. The earlier5516cb4
artifact remains intact. Handoff:
`creator-lane-peer-handoff.json#/root_caster_ownership_ready_20261106`.
The coordinator notification was queued; that is not a deployment receipt.

**Ether Feast healing remains unimplemented.** Both5516cb4 and this build
exclude that skill from generic attack damage/retaliation; neither restores
health or implements recharge. A publisher summary saying it “heals instead”
was corrected in the handoff. Latest independently verified live is
c2cb6fd, with [saved mode enabled](209026-saved-norn-live-activation.md).
This new artifact is checked and handed off, **not claimed deployed**.

The publisher's subsequent run203 smoke used5516cb4. Its retained logs
show world startup and cleanup, but no sword SkillBegin or combat response;
the only typed skill request is10176 with target0. This does not establish
a combat pass for that candidate or this cumulative artifact. The
[03:52 readback](209026-saved-norn-live-activation.md#subsequent-combat-smoke-readback)
records the exact evidence and unresolved admission/input cause. Focused
ownership tests and startup checks remain passed; release readiness must
also account for this unresolved changed-combat smoke.
