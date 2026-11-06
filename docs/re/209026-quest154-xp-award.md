# Build209026: quest154 completion XP reconstruction

This increment adds the missing XP portion to the existing Mesmer quest154
item32138/72-copper claim. It uses the current server's normal-mode model; it
does not recover retail payout timing or change the quest's reconstructed
actor/action rules. There is no profile, wire-schema or durable-store change.

## Source contract and chosen sampling policy

The [retained reward expression](209026-quest154-content.md#exact-xp-display-expression-no-new-xp-payout-policy)
uses quest154 level field2, minimum0 and level-table total2000. For ordinary
stored XP in mode0, its displayed amount is `max(2000 - total_XP, 0)`.
The player component constructor selects mode0; the current server does not
send its mode-changing122 update. That is a source/model condition, not a
sampled native runtime mode. Nonzero modes are outside this implementation.

On the eligible automatic-panel path, final542 synchronously reaches quest
completion `1412006a0`, GpReward listener `1402c94c0`, widget initialization
and the expression `14082f4c0`. The implementation freezes the amount after
preceding kill XP and immediately before final542. It pays that exact amount
on the existing205/0 claim. Completion-time sampling and claim-time credit
are explicit reconstruction policy, not recovered retail server scheduling.

Delayed opening or a recreated panel can sample later XP. A subsequent kill
or rejoin never changes the frozen award; consequently this policy does not
promise an identical display after those events. The existing restoration
order remains intact: carried191/188/earned187 precede replayed542, so the
recreated panel sees actual carried XP, not constructor0.

Native subtraction wraps a Dword before a signed-positive test. Saturating
subtraction agrees only through total `0x800007d0`; higher inputs produce an
explicit unsupported Pending state. They cannot silently pay zero or trigger
automatic transfer. Normal supported amounts are0..2000. XP-disabled profiles
retain their old fixed-item/coin behavior with a zero award.

## Atomic claim and volatile carry

Pending stores its frozen amount in the existing process-local reward state.
Every existing carry producer copies it with the selected GUID and destination.
A reward-bearing carry with tracked XP cannot be consumed by an XP-disabled
destination. Existing quest/profession/economy and GUID/map admission remain
required. No original415, owner record, database or durable format is changed.

Claim validation checks the current XP total plus the frozen amount for overflow
before encoding or committing any reward. Item/inventory encoding and staged
XP/earned-level/earned-attribute updates precede the acknowledgement. Wallet,
bag, allocator, XP and Claimed commit together; invalid/full/overflow claims
preserve Pending. Positive XP crossing an earned threshold uses the existing
188/187 path and, only if already enabled, the diagnostic sword2 grant hook.
Effective189, second level190, health and live skill defaults remain unchanged.

The current live profile's2500 XP per kill means its final-kill quest reward
is already zero. Positive awards are covered by focused synthetic tests, not
by a new native gameplay observation. Ordinary disconnect carry remains the
existing opted-in Ether Feast lifecycle; restart persistence is not added.

## Evidence and checks

Frozen executable SHA256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

Private expression bundle:
`~/re/gw2/notes/209026-quest154-xp-expression-evidence-20261106.json`, SHA256
`391cf257f37130ea7ea4bd0a957e0b9c81a55906a7d0810856c73ee51f51c22e`.
Private sampling bundle:
`~/re/gw2/notes/209026-quest154-xp-sampling-evidence-20261106.json`, SHA256
`d139329c138005175b1d656753e79b7d2c800e281b79c20e32dbdb37ed2e1d04`.
The exact source and sampling limits remain in their linked findings; no new
client-source capture or runtime test was needed for this increment.

The isolated regression first fails because completion at1500 XP, followed by
another100 XP, claims no XP and leaves1600. The fix credits the frozen500,
produces total2100/earned2 and the literal191/188/187 wire sequence, once.
Additional focused checks cover zero/positive/domain boundaries, unavailable
XP sink and checked overflow, disabled/enabled diagnostic grant behavior,
two saved synthetic identities, incompatible destinations, disconnect,
consume-then-abort retry, Pending/Claimed restore and immutable original records.
Private build/startup/test receipt:
`~/re/gw2/notes/quest154-xp-award-ec69643-20261106-checks/receipt.json`.
Twenty-seven focused tests, the isolated musl build and two unchanged saved
auth/world no-listener checks passed. The sole publisher deployed the exact
artifact as fccd1b0 at15:30:21Z; independent15:35:13Z verification confirms
the running bytes, rollback, service health and unchanged asset feed.
[Deployment evidence](209026-saved-norn-live-activation.md#verified-quest154-xp-release-fccd1b0).
No positive native reward observation is claimed.
