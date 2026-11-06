# Build 209026: remove casts and extra hits from slot release

The observer previously interpreted packed `[0x29, slot]` as `RepeatStart`.
With the research autoattack option enabled, that manufactured a new typed78
order and optionally dealt another hit. A regression reproduced a real
SkillBegin reducing hostile health from 118 to 108, followed by release
incorrectly reducing it to 98.

The [complete input lifecycle trace](209026-skill-input-action-lifecycle.md)
now proves the ordinary input path: activation can send 0x2a before admission,
an admitted ordinary cast sends explicit SkillBegin0x23, and deactivation or
ending a previous slot can send 0x29. Creating a cast from either slot input
would bypass admission or duplicate an explicit cast. This source finding
supersedes the earlier RepeatStart interpretation; it does not supply a retail
repeat scheduler or a server acceptance policy.

## Implemented correction

Both exact two-byte inputs are retained as `Seen::SkillSlotInput`, preserving
opcode, raw slot and the complete outer behavior header. They produce no cast,
damage, implicit target choice, retaliation, kill reward or loot. The existing
explicit SkillBegin path remains the current fixture's hit trigger. No guessed
cancel, finish, chain increment, cooldown or repeating timer was substituted.

The obsolete autoattack state, nearest-target fallback and response branch are
removed. Direct `--mission-autoattack-skill` use returns a specific retirement
error, and the option is removed from usage. The supervisor keeps older
profiles bootable: a configured `GW2_AUTOATTACK_SKILL` emits one constant
deprecation message and is no longer forwarded to the binary. New profiles
should omit that setting. Deploy the corrected supervisor with the binary;
the old supervisor still forwards the retired argument.

The previous **c862325 / 834c723f…** static artifact is retained as historical
evidence and marked **superseded**, not the latest releasable fix. Its proof
about incoming typed78 admission mask0x10 remains valid separately; the
request-to-cast mapping was wrong. No live kit, publisher process or held
run145 input was changed during this work.

## Focused verification

The new release regression first failed on the old branch with the extra
order/hit above, then passed after removal. Three focused slot tests pass:

- Full framed `0x2a → SkillBegin0x23 → 0x29 → repeated0x29` yields exactly
  one existing fixture hit, from SkillBegin.
- Both slot opcodes preserve full header values and raw slots 0/5/23/255,
  without health, target, aggro, XP or loot changes. Malformed lengths remain
  unclassified.
- A release after an explicit hit preserves health, selected target, aggro
  and the already scheduled NPC attack deadline, returning no order.

The 34-test mission-module run passed 33 tests and exposed one old test that
still expected 0x2a to be unclassified. That fixture was changed to unrelated
raw opcode0x2b; its focused rerun passed. Production parsing was unchanged
by that expectation correction. Both shared creator/combat configuration
tests also pass. `bash -n` passes, and executing the exact supervisor
compatibility block with an absent and a configured legacy setting verifies
zero forwarded arguments and the configured-only deprecation notice.

## Remaining gameplay boundary

This fixes duplicate/invalid execution, not complete retail combat. The
damage-enabled SkillBegin branch originally returned before the separate
`skill_order_echo` diagnostic. The later
[reply composition fix](209026-skill-reply-composition.md) permits both
explicit fixtures in one response; it does not establish an accepted retail
animation lifecycle. The removed release response must not be restored to
hide this gap. Actual cast acceptance, hit timing, automatic
repetition, sword-chain progression, healing and cooldown execution remain
separate work. Tooltip calculations and animation event quantities alone
do not prove those server rules.

The sole publisher's unanswered saved-versus-prepared roster prompt and the
fresh shared-runtime lease remain preserved. An offline build/check receipt
does not establish deployment; the last verified live kit remains
82d8d04-cdn900 until a newer publisher receipt exists.

## Replacement artifact, not deployed

The later [87ef495 cumulative artifact](209026-skill-reply-composition.md)
includes this correction and the explicit diagnostic response composition.
The artifact below is retained unchanged as a tested earlier candidate.

Source commit **00599af** was built in isolated worktree
`/home/blizz/scratch/gw2-slot-release-20261106-src`, with its own target
directory. The offline release build for `x86_64-unknown-linux-musl` passes.
The static-PIE observer is:

- Path: `/home/blizz/scratch/gw2-slot-release-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`.
- Size: **1581800 bytes**.
- SHA-256: **a31f6a21dca5f26f1fe4fe6afa2a1f6fb8ec8c8995b7f843372a068f89edadc2**.
- Required paired supervisor: `/home/blizz/scratch/gw2-slot-release-20261106-src/deploy/gw2-server.sh`.
- Supervisor SHA-256: **673a6cea9caa1d364204e16143f2378a5676273eedb035ba4dc117fe5c91eb10**.

At **2026-11-06 02:21:24 UTC**, the exact binary passed prepared auth and
Norn mission `--no-capture --check-config` checks, both confirming no listener
started. The mission check uses the surviving explicit SkillBegin damage
fixture, without the retired option. The actual binary rejects that retired
option separately; both legacy-setting supervisor checks and shell syntax pass.
Eight pinned fixture files and the run145 manifest remain unchanged.

The four ignored schema files were copied from the already tested root inputs
with hashes verified; no schema change was made. Private receipts:
`~/re/gw2/notes/slot-release-build-inputs-20261106.json` and
`~/re/gw2/notes/slot-release-build-checks-20261106.json`.
Reproducible preflight wrapper:
`~/re/gw2/notes/check-slot-release-build-20261106.py`.
No client, working archive, shared runtime, saved store, online listener or
publisher process was started or changed. Retain the previous live kit for
rollback and deploy this binary with the corrected supervisor through the
existing publisher once the actual owner prompt/holds permit it.
