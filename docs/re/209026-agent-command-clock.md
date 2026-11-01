# Build 209026: agent command timestamps and delay

REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This resolves the units of server
37 field 1 and the scheduling arithmetic described in
[agent creation](209026-agent-creation.md). No game session or packet
replay was performed.

## Two clocks

The packet's field 1 is a **remote simulation-tick timestamp**, compared
with a local tick counter whose steps are **40 ms**. It is not directly
added to an elapsed-millisecond clock.

Agent-world initialization `0x14102f680` registers the world with collision
scene virtual +0x98, implementation `0x1410b94e0`, at priority zero. The
scene retains callbacks as 16-byte entries in its +0x168 array. In
`0x1410b4050`, each simulation iteration increments scene +0x214, then
calls each callback's virtual +0 with EDX=40 and R8D=that tick counter.
The physics-step constant at `0x141b416b0` is f32 0.04 seconds.

For the agent world, virtual +0 is `0x14102f760` (vtable `0x14211b4e8`).
It adds EDX to world +0x1d4, an elapsed-millisecond clock, and stores R8D
at world +0x1d0, the current simulation tick. Its agent updates receive
the same 40 ms interval. The timer assertion at `0x14211b6b8` explicitly
names `TimerGetTargetMs()`.

The scene constructor initializes its tick counter to zero. The agent-world
constructor `0x14102e1e0` initializes its last supplied tick to 0xffffffff
and elapsed milliseconds to zero; the callback supplies the first actual
tick. This note does not establish a shared server/client tick epoch or
that their counters start simultaneously. The estimator below measures
an offset between them.

## Estimator state

Network command submission `0x14102fcf0` calls `0x14102ffb0` with the
local tick and the packet's field 1. On the traced network-context branch,
the estimator occupies world +0x178:

| Relative offset | Initial value | Observed role |
| --- | --- | --- |
| +0x00 | 400 | Maximum estimated delay, milliseconds. |
| +0x04 | 320 | Interval controlling downward offset adjustment, divided by 40. |
| +0x08 | 120 | Baseline delay, divided by 40 to give 3 ticks. |
| +0x0c | 0 | Last returned delay in milliseconds. |
| +0x10 | 0 | Not used by this leaf. |
| +0x14 | 0 | Estimated local-minus-remote tick offset plus buffering. |
| +0x18 | 0 | Previous local tick; zero selects initialization. |
| +0x1c | 0 | Previous scheduled target tick. |
| +0x20 | 0 | Last delay-report tick, managed by caller. |
| +0x24 | 0 | Accumulated ticks for downward adjustment. |

Initialization `0x14102f680` clears +0x0c through +0x24 but preserves the
three parameters. The following is a behavioral description for ordinary
counter ranges; the native instructions use wrapping 32-bit arithmetic
and a mixture of signed, unsigned and sign-flag comparisons.

1. Compute `difference = local_tick - remote_tick`.
2. If previous-local-tick is zero, set offset to `difference + 3` with
   the default baseline. Otherwise, raise offset toward a larger
   difference by at most 2 ticks per command. For a smaller difference,
   accumulate elapsed local ticks; each time that accumulator reaches
   8 ticks with the default interval, decrement offset by one and clear
   the accumulator.
3. Tentative delay is the nonnegative offset-minus-difference. Reset
   offset/delay to the baseline if delay exceeds the maximum (10 ticks
   by default), or if local tick is below 1500 and delay exceeds baseline.
4. Keep the resulting scheduled target tick from moving backward relative
   to the previous target, using the native subtraction sign test. This
   can increase the delay after the preceding reset.
5. Store and return `min(delay_ticks * 40, maximum_delay_ms)`.

The arithmetic does not independently authenticate, reject, or validate
remote timestamps. In particular, an arbitrary first timestamp can still
produce the baseline delay. Repeating a constant timestamp later is not
equivalent to a correctly advancing remote clock.

## Scheduling and client delay report

After estimation, submission computes a timer target from:

```text
elapsed_ms + min(estimated_delay_ms, world.absorb_clamp_ms)
```

The separate clamp at world +0x20c defaults to 1,000,000. Configuration
code `0x141415630` reads the literal **AgentAbsorbClampMs** at
`0x1422dd9b0` and forwards it through `0x141024420`/`0x14102ffa0`.
Timer setup `0x1410239e0` stores a target capped at 0x7fffffff and a
locally incremented ordering value. That ordering value is separate
from the packet timestamp.

Server 37 has command mask 0x80 set and therefore invokes its execution
method immediately **after** this timer bookkeeping. The 120 ms baseline
does not mean that initial agent allocation waits 120 ms. Other command
types can use the scheduled path. Admission, binding and lifecycle checks
still apply.

When unsigned `local_tick - last_report_tick > 12`, the network-context
caller can send **client 17**, descriptor `0x1425b7260`: one Word field.
It reports the estimated delay in milliseconds, saturating values at
0xfffe when they are at least 0xffff. Thus this report is not a copy of
the remote timestamp. Its registration belongs to protocol 0, group 0,
alongside the other agent messages.

## Isolated calculation check

A private, hash-guarded harness copied only the 266-byte leaf
`0x14102ffb0..0x1410300ba` into process-local executable memory and called
it with the Windows x64 ABI through a small C adapter. The inspected leaf
has no calls or image-relative references. It receives only a ten-u32
state buffer and two counters. This did not run the launcher, connect a
socket or resume the held client.

An independent Python model matched the returned delay and all ten state
words for **12,005 calls**: five fresh-state examples plus six deterministic
2,000-command sequences covering repeated/irregular ticks, remote
reordering, the 1500-tick boundary and u32 wraparound. This is sampled
calculation evidence, not exhaustive integer verification or a valid
bootstrap fixture. Fresh examples `(local, remote)` of `(1,0)`,
`(1,500000)`, `(10000,0)`, `(10000,9999)` and `(0xffffffff,0)` all returned
120 ms under untouched defaults.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-command-clock-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_2234c0ea0512e8aad69679d821cc8ea4f96e26bd5de15fdd43afab12d8dd52a5` | Submission, estimator arithmetic and delay report. |
| `ev_aa6a43d0761c52d13e5595291814b4912ba03c5e46e76688c54b761b76c80689` | Default state, clock writes, packet-field forwarding and timer setup. |
| `ev_edc0ecd3072602a43c479432c384d574f1497c24ddc48c4db8be16411d1e591e` | Agent update vtable and TimerGetTargetMs assertion. |
| `ev_2e3e0a7645a4ad33319efc260a10a96d92f79b355dddf5777905f77d7f04d3d4` | Initialization clears estimator history and registers scene callback. |
| `ev_e4d32432ce2f94b02302f7b7dbb360535ad63114d0276ce051b237a1d4d8ce6d` | Collision-scene vtable and clamp configuration call. |
| `ev_9dc111d75c4281485f115a7321b6e0c2e56552caa7d443c0130fe350d5e11e3f` | Registration slot and AgentAbsorbClampMs literal. |
| `ev_865488287d72a45c566efdde6b97427e07e842687d507d29341fcf06445b0599` | Callback array registration. |
| `ev_49d855f34092f084f627019a156076b373883592f415a2b38350c3895e982519` | Simulation loop supplies 40 ms plus incrementing tick. |
| `ev_f19a9f560c811924a78ca4411a35b87b5b2357646af1e1d90233c33091304c5f` | Counter initialization/update, agent forwarding and client definition table. |
| `ev_1ccd2637455e438cae0aad9a742704bd4d4376e38681b758e8a8984fa0e50688` | 0.04-second constant and creation flags. |
| `ev_05132dc63e4058b1af5a1c985a21e1e6eae714a947f211df8f457a710a91b98f` | 12,005 native-leaf/model comparisons and client-17 descriptor extraction. |
