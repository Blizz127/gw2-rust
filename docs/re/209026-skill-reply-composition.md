# Explicit skill-order diagnostics alongside fixture damage

The mission observer used mutually exclusive response branches for fixture
damage and the explicit `--mission-skill-order-echo DURATION_MS` diagnostic.
When both were configured, an ordinary SkillBegin hit returned only the
health response. The damage-enabled Ether Feast guard returned no response
at all. In both cases the requested diagnostic order was silently lost.

The shared encoder now appends the configured player order before applying
the existing damage fixture. It preserves the incoming header, skill, slot
and target, the existing target-zero fallback, and the existing explicit
admission flag. Duration still requires a caller-supplied nonzero u16; this
change supplies no duration default and enables no new profile setting.
Without the diagnostic option, damage responses remain unchanged.

The damage-enabled Ether Feast guard can return the explicit diagnostic
order while preserving its gameplay exclusions: no damage, healing, target
tracking, aggro, retaliation, rewards or recharge. This is not an implemented
healing effect. The separate NPC-cast research fixture keeps its existing
priority when no player hit is applied. With damage disabled it still uses
any SkillBegin, including Ether Feast, as an explicit NPC diagnostic trigger.

Both slot activation0x2a and deactivation0x29 remain silent. Only the
existing explicit SkillBegin path reaches the order encoder. See the
[slot-release correction](209026-skill-slot-release-correction.md) and
[source lifecycle trace](209026-skill-input-action-lifecycle.md).

## Validation and limits

Two new regressions failed before implementation: the combined cast test
received257 where34 was expected, and Ether Feast produced no response.
After implementation, six focused echo/configuration tests pass, followed
by all37 mission-module tests on the final code. The new
tests verify the exact order then health frames, a single hit across
`2a → 23 → 29 → 29`, unchanged existing NPC-cast precedence, and the
damage-enabled healing guard. Request headers and duration are checked
against explicit synthetic values.

These are Rust fixture checks, not a new shipped-client runtime observation.
The combined response ordering is a diagnostic implementation choice.
Client acceptance, cancellation/finish, retail hit timing, automatic repeat,
sword-chain progression, actual healing and cooldown scheduling remain
unimplemented or unverified. The source evidence already retained for the
order encoding does not supply those server policies.

The native publisher's unanswered saved-versus-prepared character prompt
and the shared-runtime lease hold remain in force. No client, working
archive, run145 input or online deployment was changed by this fix.

## Checked cumulative artifact, not deployed

Source **87ef495** includes the slot-release correction00599af and was
built offline in isolated worktree
`/home/blizz/scratch/gw2-skill-reply-composition-20261106-src`, with a distinct
target directory. The `x86_64-unknown-linux-musl` release build passes.

- Binary: `/home/blizz/scratch/gw2-skill-reply-composition-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`.
- Size: **1581984 bytes**, static PIE.
- SHA256: **45c52f1fb072cc7b02a02d64262c8cb1a924ec6bdbf8241db6420685c8642395**.
- Required paired supervisor: `/home/blizz/scratch/gw2-skill-reply-composition-20261106-src/deploy/gw2-server.sh`.
- Supervisor SHA256: **673a6cea9caa1d364204e16143f2378a5676273eedb035ba4dc117fe5c91eb10**.

At **2026-11-06T02:31:21.206487Z**, this exact binary passed prepared auth
and Norn mission `--no-capture --check-config`, both with no listener started.
The private mission preflight combines an explicitly requested1000ms echo
with the existing synthetic hostile/damage fixture; that value is not a
retail duration or a new deployment default. The retired autoattack CLI is
rejected, the supervisor's absent/configured legacy-setting checks pass,
and shell syntax passes. Eight pinned fixture files and the run145 manifest
are unchanged. Four ignored schema inputs match the previously tested pins.

Private receipts and reproducible wrapper:

- `~/re/gw2/notes/skill-reply-composition-build-inputs-20261106.json`.
- `~/re/gw2/notes/skill-reply-composition-build-checks-20261106.json`.
- `~/re/gw2/notes/check-skill-reply-composition-build-20261106.py`.

This checked candidate is now included in the newer
[d9d0bfe starter-attribute build](209026-starter-base-attributes.md#checked-cumulative-artifact-not-deployed).
The previous00599af
artifact remains intact as a tested earlier candidate; c862325 remains
withdrawn because it manufactured a cast from slot release. The last verified
live kit remains **82d8d04-cdn900**, with its existing rollback retained.
Publication still belongs to the sole publisher after the actual owner
prompt and resource holds permit it; no deployment is claimed here.
