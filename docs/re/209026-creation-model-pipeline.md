# Build 209026 creator model submission and completion

This continues [the run-122 readiness finding](209026-creation-preview-readiness.md).
Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The source slice and probe preparation on 2026-11-05 launch no client, attach no
additional debugger, and change no target state.

**The clear model flag in run 122 is `IsLoadFinalized()`, and the missing
`+0x238` object is `m_skeleton`.** The client names both in initialization
assertions in `0x140cf5130`. Source also identifies `+0x240` as `m_geomFile`
and `+0x158` as `m_animationBanks`. File requests, queue polling, model
finalization, and composite completion are distinct observable stages.

## Reused post-load NPC result

The independent native-peer investigation already traced the request/enqueue
and pump chain in its post-load NPC runs 171/172. Run 172 observed the NPC's view
being advanced while its model never reached the finalizer in the observed
interval; load-time player models did reach it. This supersedes run 171's
stronger initial inference that no request/enqueue occurred. The sources are
retained at:

- `/home/blizz/dev/gw2-rust-talkloop/docs/re/209026-npc-talk-loop.md`, runs 171–172.
- `/home/blizz/re/gw2/notes/swarm-20261105/post-load-spawn.md`.
- `/home/blizz/re/gw2/notes/swarm-20261105/post-load-spawn-2.md`.

Creator run 122 and the post-load NPC have the same unfinalized-model symptom.
These observations do not yet establish a shared scheduling cause. In
particular, the map loader's forced-drain path alone does not prove that a
normal-frame budget prevents this model from completing.

## Submission, polling, and callback

Model constructor `0x140ce56d0` installs vtable `0x141ee0310`. Its relevant
slots are `+0x218 → 0x140cea1d0` and `+0x540 → 0x140cf4d10`.

`0x140cea1d0` receives RCX=model, EDX=geometry-reference count, and R8=the pointer
array. It replaces previous component-file inputs, attempts a combined-file
lookup through `0x140d029c0`, or requests each input through `0x140d02a20`.
The resulting combined resource is at model `+0x248`; individual pending file
handles are at pointer/count `+0x258/+0x264`. It clears finalized bit 35 if set
and enqueues the model through `0x140d00c90`. These are geometry submissions,
not proof that archive bytes were read or accepted.

The model's initial setup `0x140cf5130` requests the base file through
`0x140d02a20`, stores it at `+0x240`, and retains its handle at
`[model+0x158]+0x10`. A copy of the initial reference is stored at
`[model+0x158]+0x30`. The probe accepts only the strict encoded archive-ID
representation documented in [file references](209026-content-file-references.md);
unsupported forms are reported without printing strings.

Queue helper `0x140d00c90` operates on manager list `+0x170`. Pump
`0x140d01c30` traverses that list; for an unfinalized model it calls
**`0x140cf4e40`** at `0x140d01fb5`. The continuation **`0x140d01fba`** has
RBX=model and EAX=result. A zero result skips finalization for that model.

The done predicate first checks these file handles' Dword `+0x70` states:

1. Each pending file in model `+0x258/+0x264`.
2. Optional combined file `+0x248`.
3. Base geometry file `+0x240`.
4. Required child-model readiness checks, if present.
5. Animation-bank model file `[model+0x158]+0x10`.

File state **4 or 5** passes these source tests; the distinction between those
two states is not assigned here. Capturing all direct file states as 4/5 would
not by itself prove that child-model requirements pass.

When the predicate succeeds, pump `0x140d01ff3` calls finalizer
**`0x140cf1a80`**. That function first calls `0x140cf6150`, which can initiate
combined-file work and return zero, and has later texture/resource gates.
Reaching the finalizer's entry therefore does not guarantee completion.
Successful processing sets bit 35 at **`0x140cf1bf4`**; continuation
**`0x140cf1bfb`** observes the completed store. Later it walks model listeners
and invokes slot zero at **`0x140cf1e91`**, passing RCX=listener and RDX=model.

Composite listener vtable `0x142185d50`, installed at manager `+8`, has slot
zero **`0x1412d6310`**. That leaf clears `0x4000` at `[RCX+0xb14]`, which is
manager `+0xb1c`: `COMPOSITE_PLAYER_WAITING_ON_MODEL_GEO`. This connects successful
model finalization to the composite wait flag seen in run 122.

The model-file pump is called unconditionally near the start of
`0x140cf63a0` (`ModelAdvanceFiles`) before that function tests its float delta
argument. `0x140cf6c40` also calls it. The inspected normal-frame call site
`0x14094d361` passes its flags in R8D. Runtime observations are still needed to
establish which calls occur and whether the owned creator model is visited.

## Prepared bounded passive probe

Private helper:
`/home/blizz/re/gw2/notes/creator-model-pipeline-probe-20261105.py`.
Entrypoint `arm_creator_model_pipeline(inf, slide)`, output prefix
`GW2_CREATOR_MODEL_PIPELINE`, SHA-256
`74ba8e90bf1656844bfde5d0c1d8af5aac0620cdf37df1004dbcb89b06c45c7c`.

Its eight sites bind the current UI model, record UI and pump heartbeats,
geometry submissions, the owned pump done-check result, finalizer entry,
finalized-bit continuation, and the composite callback. GDB conditions exclude
unrelated model calls before Python accounting. The pump heartbeat intentionally
counts the shared pump separately. Existing source proves initial enqueue; the
probe does not need to capture a second global enqueue stream.

Limits are 90 seconds from first owned post-build, 16,000 matched callbacks,
256 output records, and 64 post-build observations. File/reference arrays are
capped at 32 entries with explicit truncation; that is an observer resource
limit, not a client domain claim. State changes and two-second heartbeats retain
late progress. A model whose construction predates binding can still expose its
current direct file handles, states, and copied initial reference.

The next evidence should distinguish: shared pump absent; pump active but owned
model not visited; owned done predicate false with specific unfinished direct
file handles; done predicate true but finalization incomplete; or finalization
and listener callback both complete. None of those results is assumed in this
source-only note. No new server packet or client patch is justified yet.

## Evidence

Private REA bundle:
`/home/blizz/re/gw2/notes/209026-creator-model-pipeline-20261105.json`.

| Evidence | Coverage |
| --- | --- |
| `ev_6101cca16688259d3baa8f3091a0cf3c2a74f43179db0e3101a8ab433e8a76d4` | Named model initialization assertions and initial file/reference storage |
| `ev_41692c15cfdee20b66ffcc0c1fc315dbbdcbeda4ca47a3508a386111c1c48e9a` | Model constructor vtable assignment |
| `ev_7429ee0b15cf13080ce45427376d456afe5173f6759f213a85abdb523bcd1454` | Exact model slots and composite listener slot zero |
| `ev_f75ca2e607d8cf1b51eaa8b508e4ac42fd444a5c3d3b1e70ac64811e50be8f1c` | Geometry submission and complete file-done predicate |
| `ev_bd1340cba27b0ad52475981037a60628c73dfa96090db8d75c38f33435e468ff` | Finalizer's combined-resource prerequisite and listener registration |
| `ev_99a63df274a28c70082877cf7ae7ff87ce6b436e12bb5300d80b75f563631082` | Finalized-bit store, listener dispatch, queue helper |
| `ev_2a334a637b3202b8e2a4151d0d9a25d366619af4e26c8e11c3f289958ae9cf65` | Composite callback clears waiting flag |
| `ev_c936663156ede73f1daddfe3ac49ff13be5b001730c98184b25dbc0a1675e258` | Pump traversal and owned done-check/finalizer call sites |
| `ev_07a913b6e6a58241186c6c200408457823cd06d79dac770f83b9d1ffb506b566` | ModelAdvanceFiles pump calls |
| `ev_87abdf8b9744d1056dd481296450b97de77060e312c6ea20a966613a22dcabed` | Syntax and twelve host-only mock checks, exit zero, cleanup verified |

## Run 123: direct files complete, then owned done-check visits stop

Run 123, `/home/blizz/gw2-sandbox/runs/run123-norn-creator-render-submit`, uses
the guarded helper above in the existing isolated runner. Read-only REA
extraction after the appearance soak finds **an active pump, successful initial
file completion, and one deferred finalizer entry, followed by no further
owned-model done-check visits even after the combined file becomes ready**.
This narrows the outstanding failure to progress after combined-file creation;
it does not yet distinguish list removal from starvation behind other work.

The preview's copied initial reference decodes to archive file **29213**, suffix
zero. At elapsed 16.450 seconds, the owned model receives geometry submission
count **9**, with these references in order, all suffix zero:

`39997, 39853, 39911, 39845, 39565, 39167, 39168, 39169, 39171`.

These are observed submitted archive references. No conclusion about their
rendering semantics or complete transitive dependencies is implied. Earlier
preview objects reference 29193 and 30126; the final appearance object remains
the same owned model throughout the interval below.

All elapsed times are seconds from the probe's first owned post-build callback.

| Time | Observation |
| --- | --- |
| 16.118 | Appearance scene 3 begins; model unfinalized, base and pending file states 1 |
| 16.450 | Nine geometry references submitted |
| 16.462 | Composite state 4; nine pending files; done-check returns zero |
| 34.231–35.309 | Direct files progress through states 1/2 to 4; by 35.309 all nine pending files plus base/animation-bank file are 4 |
| 35.548 | Owned done-check succeeds and finalizer entry is observed with argument zero |
| 35.551 | Pending count becomes zero; new combined file is present in state 3; model remains unfinalized |
| 35.764 | Last observed owned done-check returns zero with combined file still state 3 |
| 36.189 | Combined file reaches state 4; base and animation-bank file also remain 4 |
| 64.046 | Same direct files remain ready, pending count zero, bit 35 clear, skeleton null, composite waiting flag set |

The combined-file creation and clearing of pending inputs match the inspected
`0x140cf6150` path, which returns zero after starting this work when its argument
is zero. The probe records entry and resulting state, rather than that helper's
return instruction directly.

Shared pump calls continue with observed **flags 8**. Between the combined
file's state-4 observation at 36.189 seconds and the final heartbeat 27.857
seconds later, the pump count advances **169 → 291**. The owned done-check count
stays **164**, finalizer-entry count stays **1**, and no finalized-bit continuation
or composite callback is recorded. UI-frame count reaches 291, with the same
owned model and manager and no later post-build/rebinding. Thus neither absence
of the pump nor a continuing direct-file state-1/2/3 wait explains the late
observation. It remains possible that queue position, budget, or another
scheduling condition prevents the model from reaching its next done check.
The child-dependency walk itself is not reevaluated by this probe.

No probe failure, stop limit, or array truncation is reported. There are 113
output records, including 25 sampled done-check records (24 zero, one nonzero),
one submission, and one finalizer entry. These output-record counts differ from
the unsampled callback counters. The post-soak marker exists. The last scene-3
observation is 47.928 seconds after its first recorded appearance entry.

The next targeted observation should inspect the model-manager `+0x170` list's
membership/order and per-pass stop location around the already observed model,
including whether work ahead repeatedly consumes the normal budget. No new
server message or permanent client change follows from this evidence alone.

Private final log `client.out`: 216707 bytes, SHA-256
`6b792c283214e1f7a69e5e632cc9a992a6c6e0a99925d4d3305242a44a88e2bb`.
Safe-prefix extractor:
`/home/blizz/re/gw2/notes/creator-model-run123-filter-20261105.py`.
Private REA bundle:
`/home/blizz/re/gw2/notes/209026-creator-run123-model-pipeline-20261105.json`.

- `ev_6b6ae929ae2b2a270496020c613cf832663642cac327cc9535af81c5ab1e133a`
  retains the first/last scene records, exact submission, deferred finalizer,
  and final file-state snapshot.
- `ev_b56f6e2c902228027f122f53b139e40c18e6a1564855fc87438e90e490b6bb95`
  retains the stable final log hash and compact state-transition timeline.

Both extractions exited zero without truncation. This audit read only the
existing log; it did not launch or attach to the client or change its state.

## Queue contract and sparse follow-up

Additional static inspection makes the defer contract explicit. The Qword at
`0x1427f9010` holds the file-manager pointer. Let `L = manager + 0x170`:

| Location | Meaning in the inspected intrusive-list implementation |
| --- | --- |
| Dword `L` | Signed offset of the link embedded in each model |
| Qword `L+0x10` | First model pointer, or a terminal value with low bit set |
| Qword `model+linkOffset+8` | Next model pointer, or low-bit terminal value |
| Detached link | First link Qword points to itself; second equals link address OR 1 |

`0x14023b630` relinks a node after first unlinking it from its old position.
The model enqueue wrapper `0x140d00c90` supplies type 2 and a null reference
node, inserting before the sentinel (at the tail). Wrapper `0x140d00cb0`
supplies type 1, inserting after the sentinel (at the head). The helper's
assertion names `LIST_LINK_AFTER` and `LIST_LINK_BEFORE` and validates the link
offset. These operations do not imply a separate queue class for a new model.

Unlink wrapper **`0x140d06750`** selects the same manager `+0x170` list and
jumps to generic unlink `0x140a08570`. Finalizer `0x140cf1a80` invokes this
wrapper at **`0x140cf1b82`**, after its preparation and texture/resource gates.
The deferred combined-file path returns from the preparation helper with zero
and exits before that unlink. The inspected defer path therefore expects the
model to remain on the list for a later pump; it does not require a new model
enqueue or the composite-completion callback to resume. This is not an
exhaustive exclusion of unrelated destructor/cancellation paths.

Every pump pass restarts from `manager+0x180`, the list head. It caches the next
node before processing the current one. Already-finalized nodes are removed;
non-ready nodes are skipped; ready nodes enter the finalizer. A deferred ready
node is not rotated to the tail by this loop. After finalizer calls, flags bit 1
or bit 0 bypass the budget check; otherwise the cached elapsed counter is
compared with the Qword budget at **`0x1425928e0`** and the branch at
**`0x140d0200d`** can end the pass. Observed run-123 flags 8 have both bypass
bits clear. The time counter is sampled between iterations; the check is not a
precise duration measurement of the just-completed finalizer.

This creates a concrete possible failure mode: earlier ready models repeatedly
defer, retain their places, and consume enough budget to prevent later models
from being visited. Static code establishes that possibility and the expected
retained membership after a defer. Run 123 does not establish that this mechanism
occurred, which node blocked progress, or whether the owner was instead unlinked
by another path.

**Timing remains confounded by instrumentation.** Run 123 used frequent owned
UI/pump callbacks and the runner still had generic diagnostics enabled,
including its default movement trace. Breakpoint stops and host-side inspection
can amplify budget effects. The earlier missing preview in run 120 is not a
controlled long-soak comparison. These results do not justify changing the
client's queue policy.

The next helper is
`/home/blizz/re/gw2/notes/creator-model-queue-probe-20261105.py`, entry
`arm_creator_model_queue_probe(inf, slide)`, SHA-256
`0286dfd9025b9614fe885544d8954810cee496446db00fd982c8cc8998963442`.
It is intended to replace the previous pipeline probe while generic continuous
traces are disabled. It has no pump-entry breakpoint. The UI heartbeat uses
GDB `ignore_count=30` and checks only for the exact marker
`/sandbox/logs/creator-quiet-soak-complete`. The planned controller writes this
after a 45-second quiet appearance soak.

The first late snapshot walks the list and records owner membership/index,
self-detachment, four head models, and direct file/finalization state **before**
enabling internal finalizer/budget probes. The walk stops at 128 nodes, and
truncation or a cycle cannot be reported as proven owner absence. Later probes
allow at most three taken budget exits, four observations at each preparation,
texture-result, and finalizer gate, and two owner-unlink calls. The next sparse
UI followup uses `ignore_count=10` and disables all probe sites. Independent
limits are 15 seconds for that internal window, 120 seconds overall, 96 records,
and 64 post-build observations. Ignored software-breakpoint traps still have
some overhead; the late snapshot is a stronger timing control, not a claim of
zero instrumentation cost.

New source evidence:
`ev_314ec01dcd7d1f4423162bf36c87cf6efa62400e85e0f3ed4afab3ba38b292f6`
(list insert/unlink and combined builder) and
`ev_9a9744fbf702fdca5a1b9addbea188d4c525c4006ddd1816e4c194346cccc2f9`
(exact queue wrapper, unlink location, pump budget path). The private bundle is
`/home/blizz/re/gw2/notes/209026-creator-model-queue-20261105.json`.
The final helper passed syntax checking and twelve host-only synthetic tests,
including marker ordering, membership versus truncated/cyclic walks, head gate
samples, and observation limits. No client was launched for this preparation.
Final-helper mock evidence:
`ev_1b0dcb093782e9b559946a2bbd330c6f2e74938dfd3732b71e5991feda39249b`
(exit zero, nontruncated; owned-process cleanup verified).

## Run 124: early membership retained; late probe stopped before marker

Run 124 disabled the runner's generic movement, stage, send, command, and texture
diagnostic traces and used the marker-gated queue helper above. Its queue probe
**stopped before the quiet-soak marker**, so this run supplies no late queue
snapshot, internal finalizer-gate result, or budget-exit observation.

The early observations remain useful:

- At first ownership, the list has four models and the owner is index 3.
- The actual link offset is **`0x40`**; the sampled budget is 20000 ticks.
- At appearance entry, elapsed 16.545 seconds, the complete list has six models
  and the new owner is **index 5**, with its link attached.
- The first three models are earlier preview owners (identified by the retained
  object tags and scene transitions). Each has base and combined file state 4,
  no pending inputs, finalized bit clear, null skeleton, and two blit resources.
- The new owner has base file state 4 and eleven pending inputs at that instant.

At elapsed **25.983 seconds**, the helper emits `finished` with reason
`120_heartbeat_limit`, 121 UI callbacks, and 22 post-build callbacks. Its
internal-probe counts are empty. The final log later shows that the quiet-soak
marker exists, but there is no `late_snapshot_before_internal_sites` event.
The early ready-but-unfinalized queue heads are not evidence that they remain
in that state after the full soak or that a budget exit blocks the owner.

The helper's synthetic tests validated assignment of `ignore_count`, not its
interaction with an actual GDB Python `Breakpoint.stop()` callback. The observed
callback rate did not provide the intended sparse observation. Before another
client test, a host-only GDB test should establish native ignore/condition versus
Python-callback ordering and validate the replacement mechanism. Simply raising
the callback cap would not establish a quiet timing control. The exact retained
`0286...` helper is unchanged for reproducibility.

Final private log:
`/home/blizz/gw2-sandbox/runs/run124-norn-creator-quiet-input/client.out`,
75076 bytes, SHA-256
`63669a567f98f3fe0bb7fd1e4dc56fe81180b1cc908a52a88822944252b56295`.
Safe-prefix filter:
`/home/blizz/re/gw2/notes/creator-model-run124-filter-20261105.py`.
Final read-only extraction:
`ev_f6c795a62eda576189e728f53a92b8898b48cb1521d0126d94222610d1852fce`
(exit zero, nontruncated; owned-process cleanup verified).
Private bundle:
`/home/blizz/re/gw2/notes/209026-creator-run124-queue-20261105.json`.


## Replacement probe: verified native GDB filtering

A separate owned C fixture under the installed **GDB 16.3** settles the
run-124 sampling defect. A Python subclass's overridden `Breakpoint.stop()`
ran on all 1000 calls despite condition zero and an ignore count of 100.
A plain public `gdb.Breakpoint(..., internal=False)` with a native command list
behaved as required: condition zero produced no Python callbacks; ignore 100
produced 900; resetting ignore to 99 inside the callback produced ten callbacks
at calls 0, 100, through 900. Installing the plain breakpoint and its command
list inside an existing `Arm.stop()` also passed. This validates the debugger
mechanism on the owned fixture; it does not yet observe the replacement in GW2.

The command list is installed with `gdb.execute` while the breakpoint is
disabled, using `silent`, `python cmn_dispatch(...)`, `continue`, and `end`.
The new helper contains no overridden `stop()` method. All seven sites remain
disabled until installation succeeds; an installation failure deletes every
breakpoint created by that attempt.

Private replacement helper:
`/home/blizz/re/gw2/notes/creator-model-queue-native-probe-20261105.py`,
entry **`arm_creator_model_queue_native(inf, slide)`**, 11725 bytes, SHA-256
`f6f4f2f8e8cb4b0563ae81056c6e2539ef4a5efbb2b8ed229f3ca18f1de43cee`.
The original `0286...` helper remains unchanged; the first native-command
revision `6274...` is retained privately as
`creator-model-queue-native-probe-20261105-62741234.py`.

The queue heartbeat is at **`0x14029ccd3`**, distinct from the navigation
helper's plain-command breakpoint at UI function entry `0x14029ccd0`.
The only preceding instruction is the three-byte `mov rax,rsp`; at `ccd3`,
`mov [rax+0x18],rbx` begins, and RCX still holds the scene. RCX is first
modified at `0x14029ccef`. As for every site, installation verifies twelve
live bytes against the hash-checked frozen image. This avoids competing
`continue` command lists at the same address. Targeted instruction evidence:
`ev_b680eb877cf430c699e8373592d0c24ade0f4388cc206257a7fbf1bd4379be12`.

Before the exact `/sandbox/logs/creator-quiet-soak-complete` marker, the new
helper reads only post-build ownership and the sparse marker heartbeat.
**There is no early queue walk.** Native `ignore_count=30` controls the
heartbeat, and the erroneous 120-heartbeat cap is removed. The first marker
snapshot still precedes all internal probe activation. The bounded list walk,
head/owner gates, three budget exits, two owner-unlink observations, one late
followup after ignore 10, and the 15-second/120-second/output limits remain as
specified above. Native ignored software-breakpoint hits still incur debugger
trap overhead; this change does not establish completely uninstrumented timing.

The final helper passed syntax parsing and seventeen host-only synthetic checks,
including no pre-marker queue walk, marker-before-internal-site ordering,
partial-installation cleanup, distinct UI heartbeat placement, capped or cyclic
list uncertainty, and observation limits. Real GDB filtering evidence is
`ev_9f168c2562ac00b661da4151c5d903e303bf748c03deabd07a3b04b220517b32`;
the owned C fixture, driver, results, and complete evidence are retained beside
`/home/blizz/re/gw2/notes/gdb-breakpoint-semantics-oiuknhd1/README.md`.
Final-helper mock evidence is
`ev_206de85a5663b0d443fd17fb6f866b3c40ec190263d2ac0784f5436f64e47d24`
(exit zero, nontruncated, owned process group verified). No GW2 process was
launched for this preparation. The private export is
`/home/blizz/re/gw2/notes/209026-creator-queue-native-probe-20261105.json`.


## Blit-resource gate: interpreting the bounded probe

The finalizer's blit gate is separate from its model-file completion predicate.
A bounded inspection of its direct wrappers establishes this ordered path:

1. `0x140cf1a80` calls preparation helper `0x140cf6150`. A zero result at
   `0x140cf1a91` exits immediately, before any blit test, even on the forced
   path. This includes the previously observed combined-file defer.
2. With preparation successful, model flags `+0x90` bit 41 must be clear for
   the normal blit loop to run. If set, the aggregate gate becomes zero.
3. The loop reads the array at `model+0x1c0`, count `+0x1cc`, stride 24 bytes.
   For each entry it selects Qword `entry+0x10` when non-null, otherwise
   Qword `entry+8`, and calls **`0x140ab7e30`**. The first zero return, observed
   at `0x140cf1b04`, ends the loop with a zero aggregate gate.
4. After every entry passes (or an empty loop), flags bit 58 must be clear.
   The aggregate result at `0x140cf1b2a` is otherwise zero.
5. Unlink/finalization proceeds when that aggregate result is nonzero or
   `R14D` is nonzero. The latter is set only when the finalizer argument is
   nonzero **and** model flags bit 10 is clear. It bypasses this aggregate
   gate, not the earlier preparation helper.

`0x140ab7e30` asserts `texturePtr` against the embedded source path
`Engine/Gr/GrTex.cpp`, then loads `texturePtr+0x10` and tail-dispatches through
that object's **virtual slot `+0x48`**. Thus a failing `texture_result` is
specifically a Gr texture readiness predicate, not the model file-state
predicate. The inspected wrapper does not identify its concrete implementation
or prove whether failure means an undecoded texture, deferred upload, GPU
completion, another dependency, or an explicit resource error. File state 4/5
on the model's base/combined/animation files does not settle this texture gate.

The earlier **`0x140a805d0` is not the direct blit predicate**. It belongs to the
later model `IsReady` path, reached through the model's `+0x200` object. It
checks Gr object type 9; flags `+0x38` bit 0 return ready, bit 1 return not ready;
otherwise it walks `+0x58` / count `+0x64` with stride `0xe8`, calling
`0x140aa82d0` on each record's `+0xd0` object and caching success. That callee
has its own nonzero `+0x50` prerequisite, tests a list of texture objects via
the same nested-object virtual slot `+0x48`, and has additional resource
initialization conditions. This is a later aggregate readiness path, not proof
that it caused the queued model to fail finalization. No numeric type name or
GPU-specific interpretation is assigned from this slice.

For the planned probe, `prepare_gate=0` localizes failure before textures;
`texture_result=0` localizes it to one selected Gr texture; and
`finalizer_gate=0` with bypass zero localizes the aggregate block. A missing
texture event is not itself a texture failure: bit 41 may skip the loop, the
array may be empty, or the bounded sample cap may have been reached. Several
sampled successful textures do not establish that every array entry passed.
The frozen `f6f4...` helper already captures these gates and model bits 41/58,
and remains unchanged.

If a zero texture result is actually observed, the next minimal passive datum
would be the preferred-image address of the concrete function: read the nested
object pointer at `selectedTexture+0x10`, read its vtable pointer, then read the
function pointer at `vtable+0x48`, alongside the resource identity already tagged
by this helper. That identifies a concrete
implementation for a bounded followup. It is a proposed followup, not a new
probe, runtime observation, or reason to change queue policy.

New source evidence:
`ev_b10ba88cf797781d82a726432806dc7a2e8255d85ffc8e54ab41658fd61b16f8`
(texture wrapper, source strings, separate Gr aggregate entry),
`ev_820e442b9f2f81110e7bc4b604f0a7cb30aaa0a7f6e9881f06f931dcd3cd2d87`
(full ordered finalizer gates and aggregate loop),
`ev_953dad259ccf114d7521f51f34dfd0ed93a52e5360709003e4c879f8e6bc5619`
and `ev_c8d5edd7aae573fc35beb5df2608717a30062d88f6ceb338e2a8b9759d65c335`
(the aggregate's immediate readiness callee). All are hash-bound static slices;
no client was launched. Private export:
`/home/blizz/re/gw2/notes/209026-creator-blit-readiness-20261105.json`.


## Run 125: late retained queue and failing Gr texture predicates

Run 125 used the native-command helper `f6f4...` with the navigation heartbeat
at a different instruction and generic continuous traces disabled. Its first
late snapshot occurs at **75.465 seconds**, 59.474 seconds after the first
appearance ownership record, **before any internal queue/finalizer site was
enabled**. This time the marker-gated observation completed as intended.

The complete seven-node list has link offset `0x40`. The current appearance
owner (tag 4) is attached at **index 6**. Each of the first four heads and the
owner has base/combined file state 4, pending count zero, finalized bit clear,
null skeleton, two blit resources, and flags bits 41/58 clear. The snapshot
therefore establishes retained late membership and ready sampled model files;
it does not establish that every texture or other dependency is ready.

After the snapshot, the first two heads (tags 1 and 2, retained from earlier
creation scenes) each produce this sequence twice:

| Probe result | Observation |
| --- | --- |
| Preparation | `1` |
| Selected Gr texture predicate | `0`, resource tags 14 and 15 respectively |
| Aggregate finalizer gate | `0` |
| Bypass (`R14D`) | `0` |

This is the first direct observation of a blocking dependency: the earlier
heads pass preparation but fail the Gr texture readiness predicate. Bits 41
and 58 do not explain those sampled failures. The concrete texture virtual
implementation and its underlying reason are still unknown. The four-sample
caps were consumed by these two heads; the owner's own texture predicate was
not observed.

Three taken budget exits occur before the owner: twice after the second head
with elapsed counters 770006 and 790006 against budget 20000, then after the
fourth head with elapsed counter 20000. The first two include several internal
breakpoint stops and their large counters must not be treated as normal frame
cost. The third occurs after the four gate-sample caps have disabled those
sites, but still within the instrumented window. These exits demonstrate the
observed traversal cutoff, not an uninstrumented duration or a complete cause
of the original failure.

At **77.954 seconds**, the followup still has seven nodes and the same attached
owner at index 6, with the sampled readiness fields unchanged. No owner unlink
was recorded. The helper stops normally with `one_late_followup_complete`,
12 sparse heartbeat callbacks, 23 post-build callbacks, and 24 total records.
It ends while still observing scene 3, so it supplies **no coverage of later
Next-driven scene/owner changes**.

This run was not an untouched soak. At approximately 18:43:40 UTC, the parent
controller resized the X parent from 1272×866 to 1280×900 to match its render
child; evidence
`ev_3afdd740d1c0453b757f6db23dc5018978e325fb444523f9ae63209e4ab2aec4`
records that display/window-state intervention. Footer visibility and later
navigation are tracked separately. The distinction retained here is the first
late list snapshot before internal queue probes, not absence of all interaction
or debugger trap overhead.

Private log read twice with identical contents:
`/home/blizz/gw2-sandbox/runs/run125-norn-creator-filtered-input/client.out`,
92010 bytes, SHA-256
`b21a40d9a7830b07bd0fb6b48f8bc6e0db4a795ad757a7b74fa8dc1c79ebc338`.
Safe-prefix filter:
`/home/blizz/re/gw2/notes/creator-model-run125-filter-20261105.py`.
Read-only extractions:
`ev_4bfb7e5025b81dae77c36612f63552ce99802de184a148072df8c129a3e1e570`
and `ev_e854081ad87ec71ec5df105fe7a4e22c75ac8bf4d898b8bb0683aeedab3ed79e`
(exit zero and nontruncated; owned process group verified, with one unrelated
unreadable process environment listed on the second capture). Private export:
`/home/blizz/re/gw2/notes/209026-creator-run125-texture-gate-20261105.json`.


## Prepared followup: concrete texture predicate identity

A new private replacement helper is ready at
`/home/blizz/re/gw2/notes/creator-texture-predicate-probe-20261105.py`, entry
**`arm_creator_texture_predicate(inf, slide)`**, 12024 bytes, SHA-256
`ef497751bc146656272a2c74c8ba7a099b3b9739b1aaf01e2bbc42a37b056d6c`.
The run-125 `f6f4...` helper remains unchanged. Install this replacement alone,
not alongside the earlier queue helper.

It uses three plain public GDB command-list sites: ownership at `0x14029ec21`,
the distinct UI heartbeat at `0x14029ccd3`, and the texture result at
`0x140cf1b04`. Before `/sandbox/logs/creator-quiet-soak-complete`, only ownership
and native-ignore-30 marker checks run. The first late list snapshot precedes
enabling the texture site. The native condition admits only `EAX==0` for the
first four queued heads or current owner. After each observation, that model
is excluded from subsequent callbacks, allowing at most four failed models.

Each failure records the source-backed array index, selection of entry `+0x10`
or fallback `+8`, opaque texture/nested-object tags, and existing model/file
readiness fields. For a vtable in the frozen image, it records the preferred
vtable address and reads slot `+0x48`. For an executable target inside that
image, it records the preferred method address. It separately checks that the
live slot matches the relocated frozen slot and that sixteen method bytes
match. Out-of-image vtables or methods remain explicit unknowns; raw heap or
external addresses are not printed. No speculative nested-object readiness
offsets are read: the result labels that layout unknown until the concrete
method can be inspected.

The image hash, all breakpoint sites, the texture wrapper, and the preceding
entry-selection/call bytes are guarded before any breakpoint is installed.
Installation failure removes all owned sites. The helper stops after four
failed models, one sparse late followup, an owner change, or its bounds:
15 seconds for the texture window, 120 seconds overall, 64 ownership callbacks,
40 records, and 128 queue nodes. The blit-entry validation has a separate
4096-entry diagnostic bound; this is not a claim about a client content limit.
The helper makes no target function calls or explicit target-memory/register
writes and changes no queue policy; software breakpoint timing effects still
apply.

Twenty-one focused host checks pass, covering the actual pinned PE section
layout, relocated vtable/method guards, outside-image unknowns, fallback
selection, successful/repeated predicate exclusion, marker ordering, limits,
and transactional installation failure. The plain-command filtering mechanism
reuses the separately verified GDB 16.3 fixture. Preparation evidence:
`ev_ff67f8b60dc703bf00495e8df03b5e6c0594ae3aaa2af68c8aff98c90224e23b`
(exit zero, nontruncated; owned process group verified, with two unrelated
unreadable process environments in the census). No client was launched or
attached. Private export:
`/home/blizz/re/gw2/notes/209026-creator-texture-predicate-probe-20261105.json`.


## Run 126: resolved GrTex2d readiness implementation

Run 126 used the unchanged `ef497...` helper. At elapsed **72.197 seconds**,
55.821 seconds after its first appearance ownership record, the first late
snapshot walks a complete five-node queue. The owner is attached at **index 4**.
The first four heads and owner have base/combined file state 4, pending count
zero, finalized bit clear, null skeleton, two blit entries, and bits 41/58 clear.
The controller reports its X-parent resize before the 45-second soak; this
probe does not independently reconstruct that UI timing.

Four distinct earlier heads then fail **blit entry index 0**, each selecting
the non-null entry `+0x10` texture. Every selected texture's nested object has
vtable **`0x141bfc700`**, whose slot `+0x48` resolves to **`0x140a73860`**.
All four live vtable slots and first sixteen method bytes match the frozen
executable. The observations span 72.481–73.833 seconds; the helper stops
normally at the four-model limit with nine sparse heartbeat callbacks,
22 post-build callbacks, and twelve total records. The current owner's own
predicate is not sampled, and no later creation/save/world behavior is covered.

A hash-bound instruction slice resolves the entire leaf predicate through its
return at `0x140a73888`. For the nested object `T`, its result is:

```text
(Dword[T+0x60] & 0x10000) != 0
    AND (Qword[T+0xa8] == 0 OR (Qword[T+0xa8] & 1) != 0)
```

Bit 16 clear returns zero immediately. With that bit set, a nonzero, even
`+0xa8` value returns zero; zero or a low-bit-set value returns one. This method
makes no file API, resource submission, or GPU fence calls. That does not mean
its fields are independent of such work; their producer meanings remain
unresolved. The method reads `+0xa8` twice on one branch, so a later field
snapshot must also be distinguished from the exact values read during the call.

The same vtable's slot `+0x58` points to `0x140a73ed0`, whose embedded source
path is **`Engine/Gr/GrTex2d.cpp`** and whose assertion names `m_bits` at another
field. This binds the observed implementation to the texture subsystem without
assigning unsupported meanings to bit 16 or `+0xa8`. The numeric state of those
two readiness fields was deliberately absent from the generic identity probe;
run 126 cannot identify which subcondition failed. A minimal next passive
snapshot can now read Dword `+0x60` and classify Qword `+0xa8` as zero,
low-bit-set, or nonzero/even, only after matching the observed vtable/method.
No probe or queue policy was changed during this investigation.

Runtime prefix evidence:
`ev_ca5775cbcd069cde9da93228dd88fb2f3d8c15642255e01998f56c40bcf239d9`
and `ev_8eda345f0afd4246d535da712e2543849dcfc0ac4873dc98acea831c360cf7b9`.
At the latter read (19:01:16 UTC), private
`/home/blizz/gw2-sandbox/runs/run126-norn-creator-save/client.out` was
77952 bytes, SHA-256
`11ddf075c53d8844051efe9193cc2a932458dbf3f9934d619ce108b30eb27a95`;
this identifies that read, not a claim that the entire runtime log had finished.
Safe-prefix reader:
`/home/blizz/re/gw2/notes/creator-texture-run126-filter-20261105.py`.

Static evidence:
`ev_76d26ec127c8975a8066c85bf87183bbc1672d2965096e7b66236411b5f9c014`
(complete leaf),
`ev_d4e73208939246a78ef92ccb6dca0a6901b5497f489e57096e60913a740fbd7f`
(exact vtable entries), and
`ev_da7875a0299a2ed0a8b75c11ac39dc28b77c9554f28637335d721323d43f612e`
(same-vtable GrTex2d source context). These are bounded, hash-checked reads;
this lane launched or attached no client. Private export:
`/home/blizz/re/gw2/notes/209026-creator-run126-texture-identity-20261105.json`.


## Prepared concrete-state probe and bounded producer inspection

The new private helper is
`/home/blizz/re/gw2/notes/creator-texture-state-probe-20261105.py`, entry
**`arm_creator_texture_state(inf, slide)`**, prefix
`GW2_CREATOR_TEXTURE_STATE`, 13353 bytes, SHA-256
`8a4874c8685b3cf839a210286237bbfc57d0c041fe609b081af5bc8d7129f140`.
It replaces the previous probe; neither `ef497...` nor `f6f4...` was changed.
The marker, three sites, four-model cap, and other observation bounds remain
the same as the identity probe.

Only for the matched vtable `0x141bfc700` and method `0x140a73860`, with the
frozen slot and **complete 41-byte leaf** verified, it reads Dword `+0x60` and
Qword `+0xa8` from the nested texture object. Output includes `flags60`,
`flag60_bit16`, and `state_a8_class` (`zero`, `low_bit_set`, or `nonzero_even`).
The raw `+0xa8` value is never printed. It recomputes the source predicate and
reports whether these fields still match the observed zero result, explicitly
labeling the field sample **`post_return`**. A mismatch is retained, not forced
into the earlier call's result. Other implementations or guard mismatches
remain unsupported with unknown layout; no concrete fields are read.

A bounded inspection from the observed table and direct callees found a
readiness-bit clear site but did not establish a set site. The pointer at
vtable-relative `+0x78` leads to **`0x140a76480`**, which clears bit 16 at
`0x140a76494`, replaces fields `+0x64`, `+0x48`, and `+0x68`, and performs
reset-like cleanup. In its non-null `+0x18` branch it calls `0x140b13040`,
clears bit 16 again at `0x140a765ad`, and nulls `+0x18`. The exact semantic
names of those replacement fields are not established here.

Its direct callee **`0x140a6e660`** skips `+0xa8` when the low bit is set or
its value is zero. Otherwise it treats that value as an object pointer, follows
each object's tagged `+0x10` link, invokes virtual slot `+0x18`, then virtual
slot zero with argument 1. Thus `+0xa8` is a head for a tagged chain of objects;
it is not merely a numeric GPU-fence value. The objects' concrete types,
insertion producer, and completion mechanism remain unknown. The neighboring
bit-access methods inspected from the same source context do not establish
those producers. No broad offset scan or queue-policy change followed.

Producer-slice evidence:
`ev_88e4d44e07263d7f2be1ed890272970d215e2d85d250e45b417da950ae239ad8`
(clear/reset-like method),
`ev_2472254f0015fd8723bf8d6d64eddff729c89a014ec30a7226f9422b43218424`
and `ev_2f73742ef88badc460b89a8436c2bbf6bc45de90893f76fc0f5502cffd42669e`
(direct chain-walk helper). The new probe passes twenty-five focused host
checks, including every predicate branch, complete-leaf mismatch outside the
old sixteen-byte guard, post-return mismatch reporting, unsupported concrete
methods, and pointer redaction. Test evidence:
`ev_e609aa96d2827b744118c975f299829988159e5bf0417b0fd4ae2cf5b3d7eb71`
(exit zero, nontruncated; owned process group verified, with two unrelated
unreadable process environments listed). No client was launched or attached
for preparation. Private export:
`/home/blizz/re/gw2/notes/209026-creator-texture-state-probe-20261105.json`.


The subsequent bounded source trace resolves `+0xa8` as `m_opList.Head()`,
identifies readiness-bit setters and deferred CPU-blit insertion, and separates
normal processing from cleanup/drain. See [GrTex2d operation producers and processing](209026-grtex2d-operations.md) for the exact fields,
callers, evidence, and remaining runtime limits. Frozen probe inputs remain
unchanged.


## Run 128: common readiness-bit failure with mostly empty operation lists

Run 128 used the unchanged concrete-state helper `8a487...`. At elapsed
**76.445 seconds**, 59.846 seconds after the first appearance ownership record,
the first late snapshot walks a complete seven-node model queue. The owner is
attached at index 6. Sampled model base/combined files are state 4, pending
counts zero, model finalized bits clear, and skeleton pointers null.

Four different earlier heads then fail blit entry zero, selecting entry
`+0x10`. Every nested GrTex2d object matches vtable `0x141bfc700`, method
`0x140a73860`, the frozen slot, and the complete 41-byte leaf. Their immediate
**post-return** field snapshots are:

| Model tag | Elapsed seconds | `flags60` | Required bit 16 | `m_opList` head |
| --- | --- | --- | --- | --- |
| 1 | 76.677 | `0x3104` | Clear | Nonzero/even: nonempty |
| 2 | 76.697 | `0x3104` | Clear | Low-bit-tagged: empty |
| 9 | 76.925 | `0x3104` | Clear | Low-bit-tagged: empty |
| 3 | 77.177 | `0x3104` | Clear | Low-bit-tagged: empty |

Each recomputed source predicate is false and matches the observed zero return.
The common captured blocker is the clear readiness bit. Three empty operation
lists show that pending operations are not necessary to explain these sampled
failures. This does not establish the exact earlier values read inside the
call, which writer last changed the bit, or the current owner's own texture
state; that owner was behind the four sampled heads.

The common flags contain source-defined `GR_TEXTURE_CPU_ONLY` (`0x4`) and
bit 8 (`0x100`), subsequently source-named `GR_TEXTURE_NOT_READY_ON_CREATE`
in [the operation producer trace](209026-grtex2d-operations.md).
The inspected constructor path skips its immediate bit-16 set
when caller bit 8 is present. That is a concrete source-compatible path for the
captured state, not proof that this was each object's creation path or that a
later completion was never attempted. No flag or queue state was overridden.

The helper finishes normally at **77.178 seconds**, after four models,
11 sparse heartbeat callbacks, 23 ownership callbacks, and twelve records.
The runtime controller reports normal completion; later UI behavior is outside
this probe's observation interval.

Final private log:
`/home/blizz/gw2-sandbox/runs/run128-norn-creator-texture-state/client.out`,
76871 bytes, SHA-256
`ad33a1663d45196dfe82af15725718b97e49cdf0388f3030d29671a437b9eda7`.
Safe-prefix reader:
`/home/blizz/re/gw2/notes/creator-texture-run128-filter-20261105.py`.
Runtime evidence:
`ev_134530181899520424f9614f4c48201ee9774711514a5a8003a8fcde48751000`
and final unchanged read
`ev_b8642264c162962c42c21b35a64a1d1c840ae66c106522f29f6189f744f5eb15`
(exit zero, nontruncated, owned-process cleanup verified).
Private export:
`/home/blizz/re/gw2/notes/209026-creator-run128-readiness-bit-20261105.json`.

## Run 129: initial preview binding and bounded observation failure

This was a **failed diagnostic**, with useful early observations retained.
Neither these records nor the controller's initial race-chooser artwork
establish successful character rendering.

Run 129 used the frozen lifecycle helper
`f995b17f2534a9a115240a2cb5b963e92a478715d81acce2d0bb8c633f99af57`
and a 45-second initial-scene soak before Norn selection. The controller added
a bounded wait for `initial_soak_snapshot_before_rearm` after writing the
initial marker; without that event it must not treat the intended initial
snapshot as captured. The helper contract, guarded sites, and mock evidence
are in [the operations note](209026-grtex2d-operations.md).

The first available post-build record captured **scene 0**, owner record 1,
with the manager's model matching the UI model. This establishes the actual
scratch binding rather than inferring it from matching flags:

| Field | First binding and early service samples |
| --- | --- |
| Model alternate blit entry 0 | Exactly manager scratch `+0xb58` |
| Model alternate blit entry 1 | Exactly manager scratch `+0xb60` |
| Scratch dimensions | Both 1024 by 1024 |
| Scratch formats | 26 and 31, respectively |
| Scratch levels | Both 9 |
| Scratch flags | Both `0x3104`, readiness bit clear |
| CPU backing | Present for both |
| Operation heads | Both nonzero/even, hence nonempty |
| Outstanding group count `+0x94` | Both zero |
| Global scratch-set count | 1 |
| Composite state `+0xb44` | 5 |
| Component index `+0xb40` | 4 |
| Completion stage `+0xb3c` | 0 |
| Composite flags `+0xb1c` | `0x5007`, including model-geometry wait `0x4000` |
| Component-invalid mask `+0xae4` | `0x7fff` |
| Model finalized bit | Clear |

The owned `0x1412d25e0` entry was observed at **+0.249 seconds**, followed
by the owned call to `0x1412d7f20` at **+0.270 seconds**, both with second
argument 1. These are times relative to the first binding, not process launch.
They directly show that this initial preview receives composite and component
processing before model finalization. The four-set limit was not exhausted
in these samples. They do not establish whether every later processing pass
ran, because these sites intentionally become one-shot until a marker.

At **+57.925 seconds**, the helper stopped with
**`bounded_read_failure_at_ui_frame`** while attempting the initial marker
snapshot. It emitted no initial-soak snapshot, no later owner binding, and no
completion-request/work event. Its final counters were four post-build
callbacks, eleven sparse heartbeat callbacks, one owner, one component-call
sample, and five total safe-prefix records including arm/finish. The stop
record does not identify which field read failed; it must not be relabeled
as a specific target resource failure or as proof that an operation pointer
was corrupt. Earlier lightweight snapshots succeeded; the failed marker path
also attempted deep operation-head reads, but the available record does not
isolate that step.

Consequently there is no captured late state to compare with the initial
component index or to diagnose cap exhaustion. The absence of later request
events is bounded to this instrumented interval and filtering contract, not
a universal claim that the client never submits finishing work. The sampled
1024-square/nine-level targets also do not fit the source trace's specialized
one-level/zero-job edge; see [the CPU completion note](209026-grtex2d-cpu-work-completion.md).
No readiness override or queue-policy change was made.

Safe-prefix read at **2026-11-05 19:50:34 UTC**:
`/home/blizz/gw2-sandbox/runs/run129-norn-creator-lifecycle/client.out`,
74792 bytes, SHA-256
`c28bff6e37dfeb4270799e4473df506d93d073d9f36c738f8398754a124b4611`.
The initial marker existed and final appearance marker did not at that read.
This digest identifies that read, not an assertion about subsequent controller
cleanup or other log writes.

Reader: `/home/blizz/re/gw2/notes/creator-scratch-run129-filter-20261105.py`.
Runtime evidence:
`ev_4f94dfc3d06cb9b739e434465d5ef3652b44a65952407bfade299661d774f88e`
and `ev_9fdd4005ce3e84012f3f3aee86aae20feb4770c1ee6c4dd09c542c45eb7965ce`.
Both reader processes exited zero with owned-process cleanup verified; the
latter lists an unrelated `/proc/*/environ` permission failure in process
sampling. No client attachment, control action, or frozen-input edit was
performed by this audit.

### Observer failure audit and separate resilient revision

The frozen `f995...` helper has been preserved. Its UI callback first checks
the current scene's manager/model pointers, then evaluates the entire deep
snapshot as an argument to `emit`. A failure before that call means no partial
snapshot is written. The last record identifies only `ui_frame`, so the
following possibilities cannot be distinguished from run 129's output:

| Read stage | Conditions that can terminate the old snapshot |
| --- | --- |
| UI identity and composite/model fields | A bounded memory read fails or required scalar unpack fails |
| Model blit array | Count exceeds 64, nonzero count has null base, or an entry read fails |
| Scratch wrapper/data/predicate metadata | Wrapper, data, vtable, predicate code, or scalar metadata read fails |
| Optional nonempty operation head | Old eight-byte alignment guard rejects it, or a vtable/concrete-field read fails |

An unrecognized texture vtable or mismatching predicate guard normally returns
an explicitly unsupported texture; that branch alone is not a thrown failure.
No run-129 record identifies the missing value or failing address. There was
also no initial snapshot event before the finish record, so later rearming
after a successfully emitted initial snapshot does not explain this output.

One unsupported observer assumption was found: the old operation reader
required `operation & 7 == 0`, while base constructor `0x140a6b880` asserts
only `(operation+8) & 3 == 0`. The client therefore establishes four-byte
alignment, not the reader's stricter eight-byte requirement. The normal
processor at `0x140a6ff6c..0x140a6ffa6` takes `texture+0xa8` directly as the
operation base and calls its vtable; no missing subtraction of the intrusive
link offset was found. This corrects a source mismatch, **not** a proven
explanation for the actual read failure. Static evidence:
`ev_6d5c85c5f007bdefb31d6d8417b3156fcb85b43054cd9a662fad0017b09fca84`
and `ev_839e302801a540a95578a1222ecdbcd5a509f4eed3ec203851a8c3269ed0adc6`.

New private file:
`/home/blizz/re/gw2/notes/creator-scratch-lifecycle-resilient-probe-20261105.py`.
Entrypoint **`arm_creator_scratch_lifecycle_resilient(inf, slide)`**, output
prefix **`GW2_CREATOR_SCRATCH_LIFECYCLE_V2`**, size **23,147 bytes**, SHA-256
**`0a47d7f116712b33934e6b0b72218ca3c2e15ba59e4855624c3e9470117677b2`**.
It keeps the same thirteen sites and observation bounds and introduces no
target calls, explicit target writes, or additional global tracing.

The revision starts each snapshot with owner-record and phase metadata,
rereads current UI manager/model identities, and verifies manager `+0xaf0`
before following their fields. A changed, unreadable, or mismatched identity
is labeled and prevents speculative reads through the cached owner. Each
optional scalar, texture, and deep operation read preserves preceding fields
and returns a local `read_errors` entry with a fixed stage and safe error kind.
One failed texture no longer discards the other texture, composite state, or
available blit bindings. Operation alignment now follows the source's four-byte
requirement and records only its modulo-eight class, never the pointer.

Marker records can intentionally contain **`snapshot_status: partial`** and
an unverified `owner_identity`. Seeing the event is not equivalent to a complete
diagnostic: readers and the UI handoff must retain those labels. A failure
outside the optional snapshot readers emits `observation_error` with owner
tags, phase, stage, and error kind before stopping. Original exception messages,
which may contain heap addresses, are not printed.

Syntax and 33 focused host checks passed, including injected unreadable heads,
four-byte-aligned operations, wrong alignment, unreadable wrappers/core fields,
changed owner pointers, manager/model mismatch, partial marker output, and
preservation of the frozen `f995...` hash. REA evidence:
`ev_6167a05165ee9e8fd640a99f1554bdeeaccdb5ab538d8be481a6627da297107b`.
These are mocked inferior/GDB checks with real pinned-image code guards;
the revised helper has not been launched against the client by this audit.

### Source-aware V3 revision prepared for run 130

A separate file preserves both frozen predecessors:
`/home/blizz/re/gw2/notes/creator-scratch-lifecycle-source-probe-20261105.py`.
Entrypoint **`arm_creator_scratch_lifecycle_source(inf, slide)`**, prefix
**`GW2_CREATOR_SCRATCH_LIFECYCLE_V3`**, size **25,427 bytes**, SHA-256
**`8e9737a35d4c46c7449e7ca119a4fe33dbffc48d3f6d7416c2d6760e69d886c0`**.
The prepared run-130 UI handshake must match that prefix and the unchanged
`initial_soak_snapshot_before_rearm` event; its staging UI is owned by the
navigation lane, separately from this helper.

With the same thirteen sites and bounds, V3 extends only the marker snapshot
of an active head whose concrete table is ordinary blit `0x141bfc780`. It
reads that operation's retained source-data pointer at `+0x30` and samples
one object. Source fields are read only after matching texture-data vtable
`0x141bfc700`, readiness target `0x140a73860`, and its complete 41-byte frozen
code guard. It records flags/readiness, dimensions, format, levels, outstanding
groups, and operation-head classification. The source's own operation head is
never followed, and its identity is not added to the work-hook watch set.
Other operation or source types remain unknown; nulls and failed reads retain
local diagnostic labels and surrounding snapshot fields.

`predicate_from_current_fields` is a snapshot of the known readiness
conjunction, not an observed predicate call or proof of completed blit work.
It separates an unready retained source from other possible processing gates;
a ready source alone does not establish that a pending blit's workers have
finished or that normal processing should already have removed it.

Syntax and 40 focused host checks passed, including all source predicate
branches, null/unsupported sources, full-code-guard mismatch, a deliberately
unreadable source head proving no recursion, local source-field failure, and
unchanged `f995...` / `0a47...` hashes. REA evidence:
`ev_483fc49743d380867798ef0abe1fd29b7a2ec012a73759d807e0dc637aa95060`.
Private export:
`/home/blizz/re/gw2/notes/209026-creator-scratch-source-probe-20261105.json`.
No client run or attachment was performed during this preparation.

## Run 130: complete snapshots, initial multi-source work, late component wait

Run 130 installed the exact source-aware V3 helper `8e9737...` in
`/home/blizz/gw2-sandbox/runs/run130-norn-creator-lifecycle-v3`. The initial
snapshot handshake completed before race selection. All **16 snapshots** in
the 18 safe-prefix records report `snapshot_status: complete` and
`owner_identity: verified`; none contains a read error or observer error.
The helper finished normally with `late_followup_complete` at **+130.642
seconds**, after 20 sparse heartbeat callbacks, 19 post-build callbacks,
three owner bindings, five composite-entry samples, and five component-call
samples. Times are relative to the first owner binding.

The initial scene-0 binding again directly matched both model alternate blit
entries to its composite's two scratch wrappers. Both targets had CPU backing,
dimensions **1024 by 1024**, formats **26 and 31**, **nine levels**, and flags
**`0x3104`**. The initial global scratch-set count was **1**; model finalization
was clear. Composite flags were `0x5007`, state `+0xb44=5`, component index
`+0xb40=4`, and completion stage `+0xb3c=0`. Owned composite and component
processing calls were observed at **+0.175** and **+0.194 seconds**.

At the initial marker, **+54.847 seconds**, that same verified owner was still
scene 0 and had progressed to **component index 9**. Its state remained 5 and
completion stage remained zero. The scratch-set count was still 1.

| Initial owner's scratch at the marker | Slot 0 / format 26 | Slot 1 / format 31 |
| --- | --- | --- |
| Ready bit | Clear | Clear |
| Operation head | Nonempty | Low-bit-tagged empty |
| Concrete head | Multi-source blit, table `0x141bfc7c0` | No active head |
| Head alignment modulo 8 | **4** | Not applicable |
| Texture-data `+0x94` | **4** | **0** |

The real four-byte-aligned operation validates the revised reader's alignment
domain. It does not retroactively identify the missing value in failed run
129. The raw `+0x94=4` is an outstanding texture-work count; it must not be
equated with the multi-source operation's source count or its own job count.
No ordinary-blit table `0x141bfc780` head was sampled, so V3 correctly emitted
**no retained-source state**. The run therefore did not resolve source
readiness or completed-work status for the multi-source head.

The initial owner received further composite/component calls at **+55.016**
and **+55.048**. The next owner binding appeared only afterward, at
**+55.083**, establishing the intended initial-snapshot ordering. A third
owner appeared at **+59.399** and was observed in appearance scene 3 from
**+64.063** onward. The scratch-set count increased to 2 and then 3 as these
owners were bound; the helper does not retain arbitrary later reads through
the old owners.

The current appearance owner was sampled after its soak at **+123.860** and
again at **+130.642**. Both complete snapshots agree:

- State `+0xb44=2`, component index `+0xb40=0`, completion stage `+0xb3c=0`.
- Composite flags `0x7007`, component-invalid mask `0x7fff`, geometry-wait
  bit set, model finalization bit clear.
- Global scratch-set count **3**, below the source-backed limit of four.
- Both model alternate entries still match its own scratch wrappers.
- Both targets retain `0x3104`, 1024-square dimensions, formats 26/31 and
  nine levels; both operation heads are tagged empty and both `+0x94` counts
  are zero.

Despite that unchanged sampled state, owned composite and component calls
were observed at **+124.115** and **+124.146**. Thus these late samples do not
support absent composite servicing or four-set exhaustion as the current
owner's blocker. The observed boundary remains before finishing requests:
there are no captured `scratch*_request`, work-start/producer/poll/destructor,
or scratch-setup-return events within the probe's identity and one-shot
filters. That is bounded negative evidence, not a claim about all client
objects or execution history. The exact component acquisition/result causing
the repeated component index was not sampled; see
[the component input source trace](209026-composite-component-source-inputs.md)
before distinguishing a missing source, pool refusal, or completed dispatch.
The initial multi-source operation's later progress is also unobserved after
ownership changed. No flag, queue, or client behavior was overridden by this
audit.

Numeric geometry evidence records the owned initial X parent changing from
**1272 by 866** to **1280 by 900**, with render child and root already
1280 by 900. The appearance geometry already matched 1280 by 900; the second
geometry record reports no resize attempt. This controlled run includes the
initial geometry correction and passive breakpoint cost and is not an
uninstrumented timing baseline. Numeric geometry and model state alone do
not establish successful character rendering.

Final retained `client.out`: **90,583 bytes**, SHA-256
**`1a1da9d212d4d8ccf0fb0640c8374c80853a9c9ab447693f47441520877c6c21`**.
The independent safe status read confirms UI exit **0**. The first log line
is the exact known isolation banner, not a container ID; it contains no
64-hex token. This audit therefore does not derive an independent
container-ID absence check from that line. The controller separately reports
exit/cleanup/archive/privacy checks in
`ev_adc1d6564bee2565e7a3e854ed08aa66ab07a8d2efff7e73b7203971374ecf05`.

Audit evidence:

- `ev_7ed64aa65ecddf09ddc864b7b1d6bcce2dbb36da09b91f116c60eb19a3963a07`:
  early binding and service calls.
- `ev_386969a12abe23c43c806e766f349c0bf42618a4b838e0026cb415de7a12f223`:
  initial marker, four-byte-aligned multi-source head, and owner transitions.
- `ev_b515bf6358047faf49626cae6a0d7c7175f37e4ea74eca9c602de6f28ad95ed6`
  and final unchanged read
  `ev_7217f9ab3e02156b62f38d08fc91dbfdce339daa42dc01f982c5ccac3b5996bc`:
  complete late snapshots and normal helper finish.
- `ev_a643367464f58c91bff03a0fd9831008a9e7af9277fe39f085ea9b9beae4a4e2`:
  retained numeric geometry before/after the initial resize and at appearance.
- `ev_ae5c5dd01ab60f1a7a13e5669ebf4ee173465d5c767363f25559c0ced8292af6`:
  final digest, UI status, and safe first-line classification.

Private safe-prefix/geometry filters are
`creator-scratch-run130-filter-20261105.py`,
`creator-scratch-run130-ui-filter-20261105.py`, and
`creator-scratch-run130-final-filter-20261105.py` under the private notes
directory. Complete audit export:
`/home/blizz/re/gw2/notes/209026-creator-run130-lifecycle-audit-20261105.json`.
Reader processes exited zero and verified their owned-process cleanup; some
records list unrelated `/proc/*/environ` permission failures in process
sampling. This lane performed no client attachment, launch, control action,
archive access, or frozen-input edit during the audit.
