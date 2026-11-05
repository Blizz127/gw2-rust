# Build 209026: creator gates after model finalization

Bit 35 at model `+0x90` is only the first readiness gate. The exact concrete
model predicate `0x140cf4d10`, vtable `0x141ee0310` slot `+0x540`, additionally
requires a skeleton, animation readiness, and usually a ready graphics object.
This source-only slice follows those known edges and the creator-owned
consumer observed in run 122. It does not establish a final draw or explain
the remaining blank preview by itself.

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All new source captures read this frozen executable only. No client, working
archive, renderer call, or target mutation was used for the investigation.

## Ordered full-readiness gates

For the consumer's actual argument zero, `0x140cf4d10` requires:

| Order | Source condition |
| --- | --- |
| 1 | Model Qword `+0x90`, bit 35 set |
| 2 | Model `+0x238` skeleton pointer non-null |
| 3 | `0x140ce9ce0(model)` returns nonzero |
| 4 | Unless model Qword `+0x90` bit 56 is set, model `+0x200` is non-null and `0x140a805d0` returns nonzero |

Other callers can use argument bits zero/one to skip gates three/four and
bit two to check descendants. Those are not the arguments passed at the
specific consumer call examined here.

Animation predicate `0x140ce9ce0` first accepts cached model flag bit one.
Otherwise it checks every animation-bank file at bank `+0x10` for state 4/5,
following next-bank `+8`, and checks the model `+0x180` list for node flags
`+0x40 & 0x200`. It caches success in model flag bit one. A read-only probe
must observe this state or the native result rather than call this predicate.

The graphics predicate asserts object type nine at `+0x14`. Flags `+0x38`
bit zero accept cached readiness; bit one returns false. Otherwise it walks
`+0x58` / count `+0x64`, stride `0xe8`, checking each entry's `+0xd0` handle
through `0x140aa82d0`, then caches success. A non-null graphics pointer alone
does not prove readiness. The helper below reads only the type and, for
type nine, flags/count; it does not walk these entries or invoke the getter.

## Skeleton binding precedes the finalized store

Finalizer `0x140cf1a80` calls `0x140cf7cd0` at `0x140cf1ba2` before storing
bit 35 at `0x140cf1bf4`. If the skeleton already exists, that helper retains
it and rebinds animation banks. Otherwise, the inspected chain is:

`bank = [model+0x158]` → `file = [bank+0x10]` → `data = [file+0x10]` →
`skeletonEntry = [data+0xc8]` → `root = [skeletonEntry]` →
`model+0x238 = [root+8]` at `0x140cf7e2a`.

Named assertions require the animation bank, ready root file, data root at
`+0x98`, and skeleton entry. The resulting pointer is immediately used later
in that path. Normal completion therefore expects a bound skeleton, but this
is not a global invariant against later changes; the full readiness function
still checks it independently. A finalized model with a null skeleton would
justify inspecting this exact field chain, not forcing the pointer or flag.

## Creator-owned consumer and remaining draw boundary

The [earlier readiness observation](209026-creation-preview-readiness.md)
matched the model passed to `+0x540` to the exact current scene `+0x88` /
manager `+0xaf0` model. Its retained sampled caller was `0x140a5d2d0`.
This is a runtime ownership relation; this slice has not established the
consumer's complete construction/binding path from the creator UI.

Source now resolves the next steps in that consumer:

1. It reads model `consumer+0x2a0`, skips initialization if that is null or
   Dword `consumer+0x43c` is already nonzero, and calls model `+0x540(0)` at
   `0x140a5d2fb`. **`0x140a5d301`** has the native EAX result and RDI=consumer.
2. Zero returns immediately. Nonzero enters the region named by marker
   `jjcStreamModels`, binds resource state through model virtual `+0x6a8`
   (`0x140cfae70`), performs other setup, then calls `0x140a5c920`.
3. It stores `consumer+0x43c = 1` at `0x140a5d460`.
   **`0x140a5d46a`** is after that store, with RDI still the current consumer,
   before the no-op call. This proves setup reached that point, not a draw.
4. Outer update `0x140a5bfd0` calls this function when lifecycle Dword
   `consumer+0x4dc == 1`. Its nonzero result leads to `+0x4dc = 2` at
   `0x140a5c029`. Since null/already-initialized cases can also return one,
   lifecycle two alone is not proof of a successful fresh readiness call.

Scene construction registers listener `scene+0x60`, whose slot eight is
`0x14029cca0`. Its call to `0x14110e340(2, currentManager)` first tests
composite readiness and gathers models for further virtual calls. This was
not established as the final renderer submission path. No broad indirect
slot scan or assumed pixel-rendering semantics is used here.

## New minimal passive observer for run 135

Private helper:
`/home/blizz/re/gw2/notes/creator-post-finalization-probe-20261105.py`.
SHA-256 `e24a1c0378bf778580524c2069705bde0f3a4719d23aa3b4902b59ff7c62a5bd`.
Entry `arm_creator_post_finalization(inf, slide)`; prefix
`GW2_CREATOR_POST_FINALIZATION`.

Its four sites are the existing creator post-build and sparse UI callbacks,
the native readiness return `0x140a5d301`, and setup continuation
`0x140a5d46a`. Every consumer observation verifies active UI global
`0x1425595d8` → scene → manager/model, manager `+0xaf0`, the concrete model
table and `+0x540` target, and current RDI consumer `+0x2a0 == model`.
It never reads a remembered consumer on a later UI callback. Native records
carry bounded opaque consumer-address tags, explicitly marked as address
labels rather than allocation/lifetime identities. Sampling is per owner,
not per consumer: a zero, one, and setup record can come from different
consumers sharing the model, and their tags must not be silently merged.

It records the first native zero and one per owner, first setup continuation,
and changed-state UI snapshots. Fields include skeleton presence, graphics
type/flags/count, full model flags, cached animation readiness, graphics
bypass flag, and the native consumer's initialization/lifecycle fields.
Graphics offsets beyond the type are read only for the source-established
type nine. Setup records explicitly state `draw_submission_observed=false`.

Limits are eight owner bindings and 128 output records; the 210-second clock
starts at the first binding, not arming. Normal initial/late marker paths and
`initial_soak_snapshot_before_rearm` are retained. An eight-second late
followup ends observation, with a twenty-second late hard limit checked on
callbacks. Limits and read/guard failures report partial coverage. Repeated
native results are filtered before Python; UI callbacks use native ignore
count 30. The helper makes no target calls, writes, flag forcing, or renderer
changes. Twelve host fixture tests passed, including ownership rejection,
rebind/value sampling, no cached consumer reads, marker/timer behavior,
unknown graphics types, different consumers sharing one model, and explicit
limits. No run-135 result is claimed.

## Evidence

Sixteen complete source records are retained mode 0600 in
`/home/blizz/re/gw2/notes/209026-creator-post-finalization-gates-20261105.json`,
SHA-256 `628966b93b319f5a54af728eee0d68cb5d96c807230a80dadb43fd32c38a38f2`.
Some PE unwind chunks ended before the branch continuation; the bounded
follow-up windows preserved those continuations rather than treating the
chunk boundary as a complete function. Every capture exited zero without
tool-output truncation and verified owned-process cleanup.

| Evidence | Finding |
| --- | --- |
| `ev_328380b05aafe6e7fef13c5703705fc07d99465020df862f8031d3fe03d2f478` | Complete model readiness gates |
| `ev_6a8a5f39963052f3ccba7eda098528eb38a6e259c948059ba32043955422bbe6` | Full consumer readiness/setup continuation and finalizer ordering |
| `ev_87441b607078a521beb3865b2a505db16846d9b68d93db3dd16b9f0d9580aaff` | Complete animation predicate and graphics-ready continuation |
| `ev_c8d9cda52751bec31088e8baaffba3819a13c675b3cce256e4a3b54a5c4eda4e` | Consumer lifecycle transition and concrete resource-binding method |
| `ev_3b01ade5a81ef834849d03582fd825565da79a752fccd40aaef3e22fba1a7e44` | Exact consumer-address reference query; limited listed encodings |
| `ev_d87f694cb3afb5f4ad01b8380c00322c8d2f24e5ae18b0714b63eee03d548d35` | Exact hook guard bytes |

The independent skeleton-writer slice is retained at
`/home/blizz/re/gw2/notes/209026-model-skeleton-writer-20261105.json`,
SHA-256 `a9e61c2d8b4f3f0471f0def2004ccf254d64db0e4a1f50d03f3e5f02cda0cca0`;
its source records are
`ev_5ef386d53ccf99e24922ac7491c534e795773d159ee1ea5f334f8f13a4c0dc70`
and `ev_1d04e16fc304d78f5d962078ba384526337638001dcc523051c612ad290b4b19`.
