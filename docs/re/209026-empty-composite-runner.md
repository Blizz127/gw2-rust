# Build 209026: persistent empty-composite completion handler

The runner now has a stateless handler for the concrete `0x141bfc7c0`
operation's fresh, unsubmitted empty job. It writes only that job's completion
Dword `+0xf8` from zero to one, leaving normal polling, cleanup, other work,
and the later texture-ready writer unchanged. Set
`GW2_RUNNER_EMPTY_COMPOSITE=0` to leave this path untouched.

This is a production-scoped candidate for private testing, not evidence that
the remaining blank Norn preview is fixed. Run 133's bounded creator-only
helper observed four write/readback-one → actual native poll-one → destructor
entry chains, and its initial owner reached finalized/state seven. The final
Norn appearance remained blank; its timer expired before the late state
observation. The reported runtime evidence includes
`ev_ce597059d8fd566a20942a1c75984d3c9b677c2105ebaac00dd109efd9a8c8d4`.
Destructor entry establishes retirement before release, not completed cleanup
or visual success. The private run-133 helper and inputs remain unchanged.

## Source scope

Target: frozen build 209026, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The [operation analysis](209026-grtex2d-multi-source-blit.md) and
[repair contract](209026-empty-composite-repair-contract.md) establish:

- Starter `0x140a75020` allocates the group/job and increments destination
  outstanding work before calling `ProcessComposite` at `0x140a750ed`.
- Empty bounded rectangles skip the sole producer `0x140b3d3b0`; no pixel
  lies inside that computed intersection. This does not establish that the
  authored rectangle itself is correct.
- The allocator leaves generic job state five and reference count one.
  Fresh group fields are `+0=1`, `+8=0`, `+0x10=1`, `+0x20=0`,
  `+0x28=self`, `+0x30=1`. The completion word may retain recycled data;
  callback/link values are not evidence of submission for this allocation.
- For that fresh state, normal group wait/release and job release require
  no worker invocation. The ordinary operation destructor decrements the
  texture's outstanding count and releases references. It does not set the
  texture-ready bit.

Retained source records include
`ev_c74507ae63057828be2e4558b106143d228ce89cc3ffba87f823832445eb19d5`
(complete branch body),
`ev_946f2d461642a86279ea87338b3b97c7970712960a192cbef4fdad48434094bc`
(fresh group/job release), and
`ev_5d87882f928caad17c2fd381d0e96a2e2a0c0d2e3090d23023df82a5b359626e`
(complete prologue and caller guards). Complete records remain in the private
multi-source-blit, repair-contract, and same-call-guards bundles cited by
those notes.

## Exact hook and admission

The public plain GDB breakpoint is at **`0x140a74ced`**, first instruction of
the empty-only block. The existing `test eax,eax` / `je 0x140a74da7` sends
all nonempty rectangles around it. EAX remains one and the source-established
RDI/R14/R13 and stack relations are intact. This also avoids a trap on each
nonempty composite. The later jump at `0x140a74da5` is unsuitable for those
same register predicates because the diagnostic overwrites EDI at
`0x140a74d25`.

The installer runs after the existing image digest, mapped-image identity,
inferior, and relocation checks. It validates frozen file and memory bytes
for the complete prologue/caller/branch and normal poll/destructor anchors.
Native breakpoint conditions require EAX one and the exact saved return
address. The callback then requires the bound x86-64 inferior and all-stop
mode, rechecks code, and verifies:

- Operation `[RSP+0x340]`, return `[RSP+0x348]`, and concrete table `7c0`;
- operation destination/source equal RDI/R14, and operation group/job equal
  `[RSP+0xb0]` / `[RSP+0xa8]`, with distinct aligned non-null allocations;
- `R13 == [RSP+0x378] == RSP+0x3b0` and an unsigned empty rectangle;
- the complete fresh group tuple, job state five/reference one/completion
  zero, and a positive destination outstanding count;
- concrete CPU texture backing/format and bounded ready primary/mask input
  predicates, with the source-allowed null first mask only.

Every checked dynamic field is captured again before writing, and the two
complete snapshots must agree. The sole possible target write is four bytes
at the current job's `+0xf8`, followed by readback. No target calls, register
changes, ready-bit changes, counter changes, instruction patches, executable
allocations, or cached operation pointers are used. Unexpected or unreadable
state receives no write; failed write/readback invokes the runner's controlled
failure path rather than retrying an uncertain mutation.

There is no UI owner, lifetime history, elapsed-time limit, or repair-count
limit. At most eight constant status lines plus one suppression line are
printed; exhausting that log budget leaves the handler active. The semantic
generalization to world composites follows from the exact operation/starter
contract, not from assuming every world/UI path was observed in run 133.
Submitted, synchronous/null-job, nonempty, differently typed, and unknown
cases remain unchanged. Preserving the existing full-width texture diagnostic
from `cba9759` is covered by a focused check.

## Checks and remaining runtime scope

`python3 -B tools/gw2-runner/test-empty-composite.py` passes twelve host tests
with multiple rejection subcases. They cover exact four-byte mutation,
relocation, all fresh fields, wrong frame/input/allocation, submitted/nonempty
cases, actual primary-plus-two-mask shape, second-capture changes, write and
read failures, install cleanup and opt-out. A single fixture performs 260
fresh operations after the log cap and verifies all complete, with no growing
lifetime state. Both embedded GDB Python blocks compile and the diff passes
whitespace checks. These fixtures read no game/archive resources.

A separate owned native fixture under GDB 16.3 ran the exact extracted
installer, public breakpoint command list, dispatcher, completion write,
readback, and logging code. It observed 180 qualified four-byte writes,
540 nonempty/wrong-EAX/wrong-caller invocations filtered before Python,
171 writes after log saturation, and 52 after operation 128. All 252 other
bytes in each synthetic job remained unchanged. Its 720 code-guard reads
returned original bytes under enabled software breakpoints. REA evidence:
`ev_c8124beae74d812c30c5da1b3720769b0a166e28df5da2a49f1237e40b76738d`.
Complete evidence and fixture are retained at
`/home/blizz/re/gw2/notes/empty-composite-production-gdb-n4crluo7/`.
This fixture substituted its own addresses and `capture()` predicate;
it verifies native GDB mechanics, not the complete GW2 data/frame guards.
Only its tiny directory was mounted, networking was disabled, and its
distinct container was removed. No GW2 process was launched or attached.

The requested next private smoke is visible Norn Mesmer preview → actual
`415/1048` durable save → `430/1065` transfer → the same saved character's
first tutorial-379 world model/equipment and movement. Existing constant
server milestones and durable record counts provide evidence without names
or raw requests in logs. This change has not itself been run against GW2 or
published during preparation, and the bookkeeping checks alone do not prove
that the outstanding visual problem is resolved.
