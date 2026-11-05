# Build 209026: conditional empty-composite repair contract

This is a source-only contract, not an implemented patch or a finding that
run 132 took this path. It applies only if a bounded observation connects the
**same live owned operation/job** to the empty-rectangle producer skip and a
subsequent native completion poll of zero. A callback mismatch alone does not
satisfy that condition.

For that case, the inspected lifetime code supports changing only the
allocated, unsubmitted composite job's completion word `+0xf8` from zero to
one. The existing operation poll/destructor would then release its group,
job, references, and outstanding-work count normally. No texture readiness
write, worker invocation, operation unlink, or counter decrement belongs in
the repair itself. This is a control-flow inference from frozen source;
runtime behavior and visual correctness remain unverified.

Target: frozen `/home/blizz/gw2-client/build-209026/Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
See [the complete operation and branch analysis](209026-grtex2d-multi-source-blit.md)
for the producer skip, poll, and field layouts. No executable, runtime,
archive, or deployed artifact was modified for this contract.

## Fresh group/job state and release

`0x140dd3f00` allocates/reuses a `0x50`-byte group allocation and returns its
address plus `0x10`. It clears the group fields, initializes its wait/event
object at `+0x18`, and sets the following state. Wrapper `0x140dd4340` then
sets group `+0x10` to one.

| Group-relative offset | Fresh value | Established role |
| --- | --- | --- |
| `+0` Dword | 1 | Counter named `jobCount` by the destruction assertion; includes its initial sentinel |
| `+8` Qword | 0 | Group member-job list head |
| `+0x10` Dword | 1 | Active/enabled state set by `0x140dd4340` |
| `+0x20` Qword | 0 | Parent/group chain link |
| `+0x28` Qword | The group itself | Counter-owner/root pointer |
| `+0x30` Dword | 1 | Wait counter, including its initial sentinel |

Job allocator `0x140dd4780` returns its allocation plus `0x10`, initializing
generic job `+0x18 = 5` and reference count `+0x1c = 1`. It does not explicitly
initialize callback `+0`, job links `+8/+0x10`, or composite completion
`+0xf8`. Those links may contain recycled values and must not be treated as
a fresh-state zero requirement. The normal composite producer would set
callback/completion and change generic state to one before submission.

For the fresh, unsubmitted state above, normal operation destruction
`0x140a6c720` follows this path:

1. Non-null group requires non-null job (`m_job` assertion). It calls
   `0x140dd4380`, whose first action is group wait `0x140dd4510`.
2. Wait counter one skips the worker-claim loop. Its decrement returns one,
   so no event wait occurs. The group has no member jobs or parent chain.
   Wait processing subtracts the group counter's sentinel, then restores
   its normal sentinel state (`+0` and `+0x30` equal one) before returning.
3. `0x140dd4380` decrements group `+0` once and tail-calls `0x140dd4100`.
   The member list is empty and `jobCount` is now zero, satisfying its
   `jobCount == 0. jobCount: %u` assertion. It destroys the wait object and
   returns the allocation to the group pool.
4. `0x140dd4870` jumps to `0x140dd5490`, which atomically decrements the
   job reference count from one to zero and returns `job-0x10` to its pool.
   This complete release path neither invokes the callback nor checks
   callback/composite-completion/generic-state values.
5. The operation decrements destination texture `+0x94`; its assertion is
   `remainingCount != MAX_UNSIGNED`. A positive pre-deletion count avoids
   underflow. Zero removes the destination from the context's work list;
   a remaining positive count preserves other work. Retained texture
   references and the operation link are then released normally.

Thus the fresh empty group does not require a submitted job in order to be
released. JobManager state five and reference count one already come from
the allocator; no generic job-state repair is needed. The proposed completion
word would be read by `0x14026f540` through the existing `0x140a6fdc0` poll.
The destructor does not set texture readiness.

## Exact scope and guards for a possible implementation

The candidate site is the nonzero/empty outcome of `0x140a74ce5`, before
the diagnostic and jump past the producer. An implementation must preserve
the original branch, diagnostics, registers, flags, stack, and all nonempty
behavior. Its sole candidate data change is the current job's Dword
`+0xf8: 0 -> 1`, once per qualified allocation.

Require all of these facts together:

- Exact frozen-image and instruction guards. At `0x140a74ce5`, EAX equals
  one and the local rectangle satisfies the source's unsigned empty test.
  EAX zero must execute unchanged with no write.
- The invocation came from starter `0x140a75020` at call `0x140a750ed`,
  returning to `0x140a750f2`. An exact-reference query found only that call
  in the queried encodings; this negative scope is not proof of every
  possible indirect caller, so the implementation should still guard it.
- A currently owned operation has concrete table `0x141bfc7c0`, with
  destination `+0x28` matching the already bound creator scratch texture,
  primary source `+0x30` matching R14, group `+0x100` matching
  `[RSP+0xb0]`, and job `+0x108` matching `[RSP+0xa8]`. Group and job are
  non-null, distinct live allocations with the expected alignment.
- At this exact function frame, RDI is destination and R13 points to the
  local four-Dword rectangle. `[RSP+0x370]` holds its local destination
  point. The prologue saves caller RBX (the starter's operation) at
  `[RSP+0x340]`; the return address is `[RSP+0x348]`. These offsets depend
  on the guarded eight-register push and `0x308` stack allocation.
- Group fields equal the complete fresh values in the table above; job
  generic state is five, reference count is one, and composite completion
  is zero. A submitted/linked job, modified group, different caller, null
  pointer, stale operation, or unknown state must remain unchanged.
- Destination outstanding count is positive. Do not require it to be one:
  other operations can legitimately contribute to the same destination.
  The operation and allocation must not have been retired or reused.

These guards exclude synchronous/null-job calls and real submitted work even
if such callers exist outside the bounded reference query. Callback equality
is not a guard here: the skipped producer never installed its callback, and
the old callback value is not meaningful for this new allocation. No cached
job pointer may be used after the operation's destructor.

| Anchor | Exact first 16 bytes |
| --- | --- |
| `0x140a747c0` prologue | `405355565741544155415641574881ec` |
| `0x140a750ed` caller | `e8cef6ffff4c8d5c2470498b5b10498b` |
| `0x140a74ce5` empty test | `85c00f84ba000000498b56584885d275` |
| `0x140a74da5` producer-skip jump | `eb754d8b4e40488b8424980000004c8b` |

The first sixteen prologue bytes do not alone encode the entire stack
allocation; a future implementation must validate the complete prologue and
the relevant call/site bytes before relying on those stack offsets.

## Run-132 observer qualification limit and future same-call plan

Read-only review confirmed passive helper SHA-256
`fa0361504cc9d0ef41f17b78d8caf64b18e6d8dbf44657c1b7eb3419d49a90eb`.
Its poll hook correctly matches the current concrete operation/destination/job
and reads the native getter's EAX. Destructor entry retires marker-created
watches before release. However, an early rectangle event does not register
its operation for that destructor watch: `watch_multi` runs only during the
marker snapshots. The early-event-to-marker interval therefore lacks
continuous retirement coverage. Tags are keyed by address, so matching
operation/job tags on opposite sides of that interval cannot alone prove
the same allocation survived. Early rectangle records also do not capture
the fresh group/job state required above. **This helper cannot qualify the
repair by itself.** Its bounded branch and poll records remain useful when
those observation intervals are kept distinct.

A future passive observer can close the gap without a global queue scan:

1. At the exact `0x140a74ce5` empty branch, verify current creator ownership,
   the complete function/call guards, return address, and saved caller RBX.
   Recover the actual operation from `[RSP+0x340]`, rather than assuming the
   destination list's head is the operation currently being processed.
2. Validate operation/destination/primary/group/job equality and record the
   complete fresh-state table, job state/reference count, completion word,
   destination outstanding count, and bounded local rectangle/point. The
   caller's rectangle-copy setup further establishes
   `R13 == [RSP+0x378] == RSP+0x3b0` at this site. Reject mismatches or read
   failures rather than supplying defaults.
3. Before continuing this invocation, assign a new observer generation to
   this operation/job pair and arm finite native poll/destructor conditions.
   Record the first actual poll result and retire the generation at destructor
   entry, before any release. A later reuse of either address must acquire a
   new generation. If the observer reaches a limit, disables, or loses its
   lifetime coverage, that generation cannot qualify a later repair.
4. Only a continuously paired empty branch, fresh unsubmitted state, and
   actual zero poll establishes the conditional case addressed here. Absence
   of a callback event, an earlier unmatched rectangle record, or pointer-tag
   equality across a gap does not establish it.

If a later implementation is authorized after that observation, its write
should still qualify the **current invocation** at the empty branch using
all the guards above, rather than replaying a recorded pointer. Require
all-stop observation for the bounded ownership/state check and write; preserve
all nonempty and unknown cases. The only candidate mutation remains that
job's `+0xf8` zero-to-one transition. Keep the normal poll and destructor
observations afterward. This paragraph specifies a future guard design;
it does not authorize or implement a target write.

The complete 20-byte prologue is
`405355565741544155415641574881ec08030000`.
The 29 bytes at caller setup `0x140a750d5`, including its rectangle copy and
call, are
`488d4424604889442428488b433848894424200f29442460e8cef6ffff`.
They are verified by
`ev_5d87882f928caad17c2fd381d0e96a2e2a0c0d2e3090d23023df82a5b359626e`;
the stack relations follow from those bytes and the previously retained
complete function body. No helper or target was edited during this review.

## Pixel work and verification limits

The existing function deliberately does not submit image-composite work
when its bounded rectangle is empty. There is no pixel inside that computed
intersection for this operation to write. Backing allocation or clearing in
preparation has already occurred before this branch; completing bookkeeping
does not skip that preparation. It does not establish that the original
rectangle was authored correctly or that the eventual character appearance
will be correct.

All nonempty operations must keep their real producers and completion
checks. Later queued operations and CPU mip generation still have to run;
the normal ready-bit writer remains authoritative. If this conditional
repair is later implemented and authorized for a test, the discriminating
observations are native poll zero to nonzero, normal destructor entry,
balanced outstanding-count decrement, and subsequent ordinary pipeline
progress. No direct texture-ready write or successful-render claim follows
from this source contract.

## Evidence

Four additional complete REA records are retained, mode 0600, at
`/home/blizz/re/gw2/notes/209026-empty-composite-repair-contract-20261105.json`.
SHA-256 `0b89e8c4470aa202f8d00fbfd5b635dbe3218537f8f7d9a63d4ae188943305a2`.
All exited zero, were nontruncated, and verified owned-process cleanup.
Unrelated `/proc/*/environ` sampling failures did not cause those processes
to be touched. Earlier complete job allocation, wait, operation destruction,
and rectangle bodies remain in the two bundles cited by the operation note.

| Finding | REA Evidence ID |
| --- | --- |
| Complete group initialization/destruction and job release | `ev_946f2d461642a86279ea87338b3b97c7970712960a192cbef4fdad48434094bc` |
| Exact incoming-reference query for `0x140a747c0` | `ev_ff750cc24cb28966d82c428550ad6badfc78dced7b94fe6b35c85b9b3dae066a` |
| Exact guards and release assertion strings | `ev_a7292aaef74f7bb6dbe0b200a30c67d288a1fef04926828a4d4c1e1d9afd7e97` |

## Private run-133 diagnostic candidate

The earlier future-observer plan described how to establish the complete
unmodified lifetime. Run 132 supplied separate positive empty-branch and
zero-poll observations, with the continuity limitation above retained. The
root prepared a private conditional diagnostic at the **current invocation**
under the existing owner-authorized development scope. No new owner approval
or hold clearance was given or required for this isolated test. Its
prewrite zero is explicitly a memory observation; it does not claim that this
allocation previously produced a native zero poll. Native polls after the
candidate action are logged separately.

The separate private helper is
`/home/blizz/re/gw2/notes/creator-empty-composite-candidate-20261105.py`,
SHA-256 `48a01c74e03f97c1b26387f4cc2b4cac690b23f7fdf37a2aaac1e5a35eec5f22`.
It defaults to passive mode; the explicit diagnostic entry is
`arm_creator_empty_composite_candidate(inf, slide, repair=True)`.
It verifies the active UI global `0x1425595d8`, current ownership, complete
prologue/caller bytes, same-call operation/group/job pairing, the fresh-state
table, backing/format/readiness checks, and an unchanged second all-stop
capture. It arms generation-specific native poll and destructor watches
before its sole possible target write: four bytes at this job's `+0xf8`,
zero to one. No target function calls, pixel writes, readiness writes, or
counter changes are made.

Bounds are eight owner bindings, 128 qualified operations/writes, sixteen
simultaneous watches, 256 attempts, and 512 output records. Callback checks
enforce 210 seconds overall and twenty seconds after the late marker; the
normal late followup occurs after eight seconds. Limit events explicitly
report partial coverage. Destructor entry retires the generation before
release and reports `release_completion_observed=false`; it does not assert
that cleanup finished. Current scene/component context is recorded without
claiming the original component identity.

Sixteen host fixture tests passed, including submitted/nonempty/mismatched/
stale/null exclusions, the observed primary-plus-two-mask shape, write and
readback failures, lifetime retirement, and output/owner limits. A separate
owned native fixture under GDB 16.3 verified original instruction reads at
two enabled software breakpoints: three callbacks and six successful code
guard reads. Its REA record is
`ev_3276b02d34c59388e7f96cbe48bb76efbde078577f823765be59e06524804145`.
The fixture mounted only its own tiny directory, used no network, and its
distinct container was removed. Preparation did not launch or mutate GW2.

The complete artifact hashes, test command, markers, limits, and evidence
paths are retained in
`/home/blizz/re/gw2/notes/209026-empty-composite-candidate-handoff-20261105.json`.
This candidate remains a private diagnostic, not a publishable fix or a
successful-render claim.
