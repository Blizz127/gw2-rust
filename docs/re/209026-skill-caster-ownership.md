# Build 209026 fixture: validate the requesting skill caster

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
was corrected in the handoff. Latest independently verified live is still
82d8d04-cdn900, with [saved mode enabled](209026-saved-norn-live-activation.md).
This new artifact is checked and handed off, **not claimed deployed**.

The publisher's subsequent run203 smoke used5516cb4. Its retained logs
show world startup and cleanup, but no sword SkillBegin or combat response;
the only typed skill request is10176 with target0. This does not establish
a combat pass for that candidate or this cumulative artifact. The
[03:52 readback](209026-saved-norn-live-activation.md#subsequent-combat-smoke-readback)
records the exact evidence and unresolved admission/input cause. Focused
ownership tests and startup checks remain passed; release readiness must
also account for this unresolved changed-combat smoke.
