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
