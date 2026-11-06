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
