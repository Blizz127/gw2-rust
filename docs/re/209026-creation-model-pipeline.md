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
