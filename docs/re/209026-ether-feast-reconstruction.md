# Opt-in no-clone Ether Feast reconstruction

Corrected commit **7c26f45** (including5a6337b) is **live**, deployed by the
existing sole publisher at **2026-11-06T09:24:16Z** to
`/home/blizz/gw2-rust-7c26f45`. Root independently verified the exact reviewed
artifact, all current/rollback hashes, published profile, effective option,
saved mode and service health at **09:27:43Z**. Rollback isa223a90.
The published profile differs from the tested profile only in comments and blank
lines; comparison and corrected exact-hash readback are retained in the
[release record](209026-saved-norn-live-activation.md#verified-ether-feast-release-7c26f45).

This increment is based on a223a90f4e929a3469e93eac47503f818317abaa.
It implements one ordinary owned10176 self-heal; it does not recover the whole
retail server. Enable explicitly with `--mission-ether-feast`, forwarded by
`GW2_ETHER_FEAST=1`. Default behavior and existing skill fixture bytes are unchanged.

## Integration into newer story work

The exact checked standalone increment was published overa223a90 while newer
story work remained on its own branch. Integrating newer story work must preserve
this increment. Every added quest162/story
transfer producer must preserve the enriched carry, retire the source's pending
cast, enqueue carry **before** releasing its active-character lease, and release
before returning1065. The three a223 transfer producers are covered here. Maps
392/792 are admitted alongside379/28 because their exact installed-map recharge
context is1; this does not port those later story-map implementations.

## Client inputs versus reconstruction choices

The retained primary10176 locale0 data supplies execute1067, action1350,
base recharge20000, and the literal amount expression
`R(f32(L)^2 * .85) + R(f32(A6) * 1) + 120`, with separate native rounding.
The existing `LiteralAmountFact209026` evaluator is used without changing it.
L is the unsigned second coreStats level; A6 is signed effective attribute6.
The present server initializes that level to1 and leaves A6 at its constructor0;
its earned188/187 updates do not change either. Thus this supported profile
heals121 before capping. This is not an assertion of retail scaling at higher
levels. Future writers of190 or189 selector6 must also update these runtime inputs.

Using1067ms as the effect deadline,1350ms as the admitted typed78 presentation
duration, and a20000ms category0 cooldown beginning at the successful effect are
explicit server reconstruction policies grounded in those fields. They are not
recovered retail server timestamps. A successful full-health cast still consumes
cooldown; only positive effective gain emits255. Positive255 uses self-source,
skill10176, second delta0, kind0 and Word0;630 uses category0 and Word0. These raw
Word/kind choices are explicit compatibility choices, not recovered retail enums.
The presentation ends with correlated84 packed45; no53 or44 result is invented.

The live17-byte skills fixture already contains605(10176),605(10170), and612
palette366/slot0/locale0. Admission requires that explicit10176 grant and primary
palette, Mesmer profession7, supported map, normal health and a rejoin route.
Generic research echo and NPC-cast modes are mutually exclusive with this option.
The existing saved creator admits Norn Mesmer only. Prepared non-Mesmer profiles
retain their old behavior when the new option is disabled.

## Lifecycle and volatile state

Foreign, dead, unavailable, wrong-slot, non-primary-flag, pending, cooling and
replayed requests cannot start another heal. Behavior serial comparisons are an
ordered-connection replay policy, not a recovered protocol counter enum. Selected
enemies are never healed, damaged or newly aggroed by this branch. A validated
replacement attack, death, parser disable, disconnect or transfer cancels pending
work. Raw00 and normal29/2a input release do not cancel: ordinary deactivation
is29; raw00 has no accepted-cast correlation. Physical/manual cancellation,
clones, conditions and modifiers remain unsupported.

The effect and ordinary damage/death/transfer work are selected by deadline,
including a late loop where both are overdue. Equal deadlines favor ordinary
work. The old scheduler is unchanged when this option is off. Existing damage,
downed, rally, regeneration and normal defeat respawn policies are preserved.

Health and successful cooldown travel only in the existing GUID/destination-bound
in-memory carry. A living reconnect preserves health; an existing death-respawn
reset preserves successful cooldown. Pending casts never survive transfer. A
same-character incompatible-profile or unconfigured-destination join is rejected
rather than replacing carried cooldown with fresh state. Carry consumed at MapLoaded is
requeued if the join aborts before BeginPlaying. Unrelated early handshake failures
create no carry, and retired sources cannot overwrite newer state.

An opted-in active-character lease prevents parallel joins of the same GUID.
It is acquired before carry consumption; carry is queued before releasing it.
Only lease ownership uses infallible identity-only Drop cleanup; actual carry is
queued by the normal connection teardown hook. No GUIDs are logged. No durable
record/schema/account changes are made, and process restart still loses this state.

## Provenance and verification

Three existing source modules were ported byte-identically from the root checkout:

- `gw2-dat/src/skill_fact_content.rs`: SHA256
  ed5f7bcfd674fb972be3448cace79da202581240b506c6db7308f2af64a8d123.
- `gw2-proto/src/skill_recharge.rs`: SHA256
  fc4d2e087664867341f8362106e58d560ce8a570e1786aa94a97687f3bbe533d.
- `gw2-proto/src/skill_order_update.rs`: SHA256
  369ad281653270d6185c996d5aef686fc40c42d72c3902274d3ebd05dae32e97.

No new third-party dependency version is introduced; gw2-net adds the existing
workspace gw2-dat dependency. Private source contract:
`/home/blizz/re/gw2/notes/ether-feast-successful-no-clone-policy-proposal-20261106.md`,
SHA256 d70479610758ad9da415bb9f6a95715065e9dfdd41ec5740936c2c685c8f6f6b.
Its source-listener bundle is
`209026-ether-feast-start-step-listeners-20261106.json`, SHA256
7e750ab8e26dc2937f83523514de8154f20531cab385fbad5d9ebe8b545d7aa9.
The map392/792 context bundle is
`209026-ether-feast-story-map-contexts-20261106.json`, SHA256
ed01cbd18b2e6068591452141303cf98b57f3166c5d925654e2176c28d5c79b7,
with exact-row REA ev_b97f14d7146a2efa7ced94931a3b2e7017549fc16fa7653270e4d414bea7db9f
and fallback ev_5f16dc30900807fe1a478314bc34fe7750b469b07391c00b1ec64aace0433dac.
These are retained source findings, not a new native gameplay observation.

Private verification receipt and artifact/profile pins are under
`/home/blizz/re/gw2/notes/ether-feast-a223a90-20261106-checks/receipt.json`.
Focused checks passed:11 healing/guard and actual TCP relog tests,16 compatibility/config tests,
4 further dispatch/config checks (one overlaps),8 literal-fact tests,8 recharge/order-update tests,3 supervisor tests; musl release
build and8 exact-profile no-listener checks (option off/on, prepared/saved,
auth/world). The saved regression covers two characters, duplicate live joins,
wrong destination, interrupted join, transfer, retired-source rejection and
unchanged original415/auth record bytes. These checks do not claim a native client
run; deployment is separately established by the release readback above.

## Original-roster relog and configured destination resume

Auth29 still echoes the requested map; saved summaries remain immutable379.
Auth/world are separate processes, so the world process cannot change that roster
through its volatile CARRY. A normal original-map login now detects a matching-GUID
pending destination only if that destination is explicitly configured by
`with_next_map`. It leaves carry unconsumed, holds the active lease, and permits
only map readiness messages in this temporary phase. After412 it emits the normal
1036/431/83 readiness followed by1065 to that configured destination. It does not
replay constructor quest, inventory, wallet or XP. The temporary phase retires
and releases its lease before the network write; abort before that point also
releases without queueing constructor state. The destination alone consumes carry.

The frozen source contract uses generic1065 redirect/commonjoin, not a rewritten
1063/1060 map (the loader requires agreement with the route). Source bundle:
`/home/blizz/re/gw2/notes/209026-heal-reconnect-route-contract-20261106.json`,
SHA256 bc029debb8edc07139b36f339fae2e62b94910a13f0195b3405a8a6828969b65;
findings `heal-reconnect-route-contract-findings-20261106.md`, SHA256
d53109a58b841b0df75e8742a5fc0c4f34d28d67ad97037c35da37b0610efbb0.
This uses the already-observed post-BeginPlaying timing; no earlier loader-time
acceptance is assumed. The real TCP regression failed on the previous rejection,
then passed through an aborted379 join, a fresh379 login, its received28 redirect,
and the correct28 join with health321, active cooldown, XP5000 and immutable saved
bytes. No native client test is claimed.

The newer quest162 branch must also expose each configured story destination to
this resume lookup; admitting its map for recharge context is insufficient by
itself. Do not infer a destination from the general heal map allowlist or accept
an arbitrary client-supplied map. This integration obligation is separate from
preserving the extended carry at its additional transfer producers.
