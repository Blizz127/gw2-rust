# Build 209026 fixture: validate the requesting skill caster

## Explicit skill availability: e624c8b ready for the publisher

On7c26f45, an owned SkillBegin for an unavailable skill could retarget and
damage a hostile, while the heal-interruption path rejected the same request.
An encoded10171 request reproduced target3→4 and hostile health118→88.
The pending heal and combat paths therefore disagreed about admission.

Isolated commit **e624c8bfcdb62ed51726383c5450a0868afd3c71** adds ten
production lines after the existing caster guard. Once gameplay has begun,
damage-enabled sessions with an explicit skill list reject keys absent from
that list before target tracking or combat replies. Granted10170 still
attacks and cancels a pending heal; rejected requests preserve that heal and
its eventual effect. Granted10176 and the existing foreign-caster guard are
preserved. Fixtures without a skill list and non-damage research modes retain
their prior behavior. This does not implement universal retail admission,
sword-chain state, new grants, strike timing or damage policy.

The regression failed before the fix;17 focused tests pass afterward,
including three new admission controls and existing heal/carry/relog/caster
checks. The isolated static musl build and two saved-profile auth/world
`--check-config` runs pass without binding listeners. Profile,17-byte skill
input, damage30, Ether Feast, supervisor, store/schema and carry/lease ordering
are unchanged. Root reviewed the complete diff and verified the source commit,
artifact and retained test/build log hashes. No client runtime was needed for
this encoded-request regression.

The tested artifact is
`/home/blizz/scratch/gw2-skill-admission-7c26-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`,
1830360 bytes, SHA-256
`723b9c36947df9a747010334fe999a0e98cfd4308d120015751aab2f36089453`.
Private receipt: `skill-admission-7c26-tested-receipt-20261106.json`, SHA-256
`def7e099725c323f1e865213ab525cc1901fd42d4ae04ea7aae289c8dbce1420`.
Source branch: `codex/skill-admission-7c26-20261106` in
`/home/blizz/scratch/gw2-skill-admission-7c26-20261106-src`.

The existing native peer helper accepted the handoff to the sole publisher;
the coordinator status reply is queued. The handoff requests prompt release
while preserving newer publisher work, saved mode, active players and rollback.
**This candidate is not yet verified live.** Shared record:
`creator-lane-peer-handoff.json#/root_skill_admission_ready_20261106`.

## Independent fix released through the sole publisher

**9451cf084cd1896824bac165b3ebf9adc7b20291** applies the same seven-line
SkillBegin ownership guard directly to then-live source **c2cb6fd**. It preserves
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

The corrected candidate handoff was submitted through the existing native
peer helper at **2026-11-06T04:33:23Z** to sole publisher session
`6f90f24c-c2e0-4d00-803a-853401bd55ce`, under its existing task
`f45083e2793a`. Native submission succeeded without a new task or publisher.
The receipt is
`~/re/gw2/notes/caster-ownership-existing-publisher-handoff-20261106.json`;
`creator-lane-peer-handoff.json#/root_caster_live_base_ready_20261106`
records that distinction. The coordinator status reply is queued.
At04:38:54Z, source inspection confirmed the publisher's commit
`e788d586a5bd6d3828d2cedf49cf6022ffd17fff` contains exactly the same119
inserted guard/test lines. That inspection established source integration;
the private receipt is `caster-ownership-publisher-source-integration-20261106.json`.
Keep the publisher's current profile, saved store/mode, metadata900, player
checks and rollback; do not overwrite later NPC additions with the earlier
staged profile.
The publisher subsequently recorded the integrated fix live in **4b3c4c2**
at **04:46:24Z**. Root independently verified its binary/manifest hashes,
health and saved mode at **04:53:03Z**. The candidate's original standalone
binary was not published unchanged; the released binary includes the
publisher's other tested changes. See the
[live release receipt](209026-saved-norn-live-activation.md#caster-guard-and-eir-appearance-4b3c4c2).
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

## Checked cumulative artifact (withheld)

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
artifact remains intact. Historical preparation record:
`creator-lane-peer-handoff.json#/root_caster_ownership_ready_20261106`.
Its release-ready flag is false. The coordinator's attempted handoff was
refused because task `f45083e2793a` was already active; task
`task237f1f15ce02` is blocked and was **not delivered**. Earlier wording
that this cumulative candidate had been handed to the publisher overstated
the evidence. The successful native submission above instead identifies
9451cf0 as the independent candidate and explicitly withholds45c0731.

**Ether Feast healing remains unimplemented.** Both5516cb4 and this build
exclude that skill from generic attack damage/retaliation; neither restores
health or implements recharge. A publisher summary saying it “heals instead”
was corrected in the handoff. At that historical checkpoint, the latest
independently verified live kit was4b3c4c2, with
[saved mode enabled](209026-saved-norn-live-activation.md).
This cumulative artifact remains **withheld and not deployed**.

The publisher's subsequent run203 smoke used5516cb4. Its retained logs
show world startup and cleanup, but no sword SkillBegin or combat response;
the only typed skill request is10176 with target0. This does not establish
a combat pass for that candidate or this cumulative artifact. The
[03:52 readback](209026-saved-norn-live-activation.md#subsequent-combat-smoke-readback)
records the exact evidence and unresolved admission/input cause. Focused
ownership tests and startup checks remain passed; release readiness must
also account for this unresolved changed-combat smoke.
