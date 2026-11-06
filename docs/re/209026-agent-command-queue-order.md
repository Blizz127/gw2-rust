# Build 209026: scheduled command ordering

Static REA, 2026-11-06, extending the [command-clock trace](209026-agent-command-clock.md).
The timer heap orders due commands by target milliseconds, then by an
unsigned locally assigned ordering word. This establishes a conditional
queue-order contract; it does **not** establish a retail skill duration,
an accepted skill order, or an unconditional start/finish packet sequence.
No gameplay code or runtime was changed.

## Timer insertion and dispatch

After admission and delay estimation, network submission
`0x14102fe78..0x14102fed7` increments world+0x1bc and passes that value
to timer setter `0x1410239e0`. Its other inputs are the elapsed clock plus
the clamped estimated delay, and command-flags bit14. The complete setter
stores:

```text
timer+0x18 = min(target_ms, 0x7fffffff)       // unsigned comparison
timer+0x1c = sequence | (bit14 ? 0 : 0x80000000)
```

An already linked timer is removed and reinserted through `0x1410236c0`.
A new timer reaches `0x14102ff50`, which inserts it into world+0x150 using
`0x1410232a0`. The world constructor sets this priority queue's link offset
to zero; the array and count are world+0x160 and +0x16c.

Complete insertion `0x1410232a0..0x1410234ff` appends an entry, then sifts
it toward the root. At `0x141023373..0x141023398`, unequal targets are
compared by the sign of their 32-bit difference. Equal targets use an
**unsigned** comparison of the ordering words. The insertion's terminal
helper `0x14025d3b0..0x14025d437` stores the queue owner and heap index at
timer+8/+0x10. The normal removal/reheapification path
`0x1402668c0..0x140266d17` uses the same target/ordering comparison; its
assertion tails are not needed to infer the normal comparison.

World update `0x14102f760..0x14102f978` first advances its clocks and runs
agent updates. It then repeatedly examines the timer heap's root. If
`elapsed_ms - target_ms` has no sign bit, it removes that root and calls
the timer's virtual+8. The ordering word's top bit also selects a service
context around dispatch. This is a heap with a secondary key, not a general
FIFO list.

For two retained timers with the same deadline and the same bit14 class,
increasing sequence values consequently give increasing dispatch order
**while the effective ordering keys remain strictly increasing**. This
qualification matters: the sequence is a wrapping Dword, and ORing in
0x80000000 discards a distinction at bit31. No guarantee across such a
boundary, equal keys, rescheduling, cancellation or an intervening state
change follows. Target arithmetic also retains the native 32-bit limits.

## What this says about a skill start followed by finish

The already traced constructors add masks0x801000 for server34 and
0x12000 for server84. Both leave bit14 and the immediate-execution bit7
clear; their received low flag byte cannot change bit14. If both received
bytes also leave bit7 clear, submission schedules both through the heap.
Their actual deadlines still depend on the estimator and current clock.
An ordered network batch by itself does not prove equal deadlines or
successful order admission.

The separately captured bound-agent queue `0x141026390..0x1410264db`
contains two mechanisms that can mark older pending commands. One requires
all of mask0x880000 on the incoming command; neither constructor's fixed
mask supplies that full mask. The other additionally requires received
low-byte bit0. A deliberately chosen low byte0x10 therefore skips both
marking branches for these two command types. This is a narrow statement
about that queue function, not a recommendation to normalize arbitrary
request flags or evidence that every other lifecycle gate passes.

The [existing typed78 completion contract](209026-typed78-completion-release.md)
already proves that a correlated server84 packed0x45 defers release while
the matched order is active, and releases directly when it is inactive.
Those findings were rechecked during this investigation; they are not new
behavior. The new heap trace explains one scheduling boundary preceding
that handler, but does not prove that server34 creates the matching order
before any chosen finish packet in a real session. Agent updates, order
admission, prediction, matching and teardown remain relevant.

No automatic finish was added to the diagnostic echo. Choosing such a
sequence would still be a server policy, and cannot recover heal execution,
recharge, authored duration or sword-chain progression. In particular,
the existing explicit1000ms diagnostic duration remains unrelated to the
separate authored execute1067, action1350 and segment2067 quantities.

## Evidence

Frozen PE SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Private cumulative bundle:
`~/re/gw2/notes/209026-agent-command-queue-order-20261106.json`,
7607786 bytes, SHA-256
`e1707b2d10b811a88f052b02dea07db083baea9342ad34902009753c3ee5c037`.
It includes earlier session records; only the following captures support
the new queue trace.

| Evidence | Scope |
| --- | --- |
| `ev_3459fc8f41f4708c0bb31264ca3a87783571d6b5b9b57ed758ccaa44196202ea` | Complete bound-agent queue; network submission prefix. |
| `ev_69cae37639503994940756bf2cbd15b616bd5e59783992711c935a97fd45f423` | Submission scheduling continuation and complete world update. |
| `ev_d49ba88b4682c9df2c088c7c95da5068b6381dcf4fac9e14175886030bf06c63` | Complete timer setter and world constructor. |
| `ev_c5ca759a8ffa6cc89e9a75651c80369cbc17cf9461a5064ac483a72b60269c30` | Heap insertion gateway, rescheduling and normal removal/reheapification. |
| `ev_61a8548aa817901cd4c9cf38e2e9026dfc13e5bf2dcd3932867490fc37f305bd` | Complete heap insertion and comparison. |
| `ev_6fa9fcfd0657a5549187af824311af8e39cffa79be7de617f42bf734967a0c69` | Complete queue-owner/index setter and bounded command predicates. |

All six helpers exited zero with complete raw output and verified owned
process-group cleanup. Neighboring instructions outside the named bodies
are not treated as complete-function evidence. No client, archive,
listener, lease, saved data, or publisher was changed; no runtime ordering
observation is claimed.
