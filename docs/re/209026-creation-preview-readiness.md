# Build 209026 creation preview readiness

Frozen client `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Static source inspection and bounded passive observations from isolated creator
runs 121/122 on 2026-11-05. Raw runtime logs and evidence remain private.

**Run 122 constructs the Norn Mesmer preview objects, advances composite
processing beyond the rebuild state, and repeatedly fails the underlying
model's first readiness gate during the appearance-screen soak.** This identifies
a concrete local blocker; it does not yet identify a missing resource, failed
archive read, or missing server message.

## Local construction and readiness gates

Creation UI `0x14029eb30` calls creation-context virtual `+0xd8`, implemented by
`0x14129cd40`. It stores the returned composite manager at scene `+0x80` and the
output model pointer at `+0x88`; `0x14029ec21` follows these stores. The context
builds a profile from the current appearance and race/sex content. It reuses a
matching manager or creates one through `0x1412d84b0 → 0x1412d16b0`.
The underlying model is created on the path through
`0x1412d58d0 → 0x1412d32b0 → 0x140cf7000`, stored at manager `+0xaf0`.

Composite readiness `0x1412d4fd0`, virtual `+0x48`, requires all of:

1. Texture-invalid mask at manager `+0xae4` equals zero.
2. Manager flags `+0xb1c` have `0x2000` clear
   (`COMPOSITE_PLAYER_COMPONENT_GEO_INVALID`).
3. Manager model pointer `+0xaf0` is non-null.
4. Model virtual `+0x540(0)` returns nonzero.

The actual model's `+0x540` target was verified as `0x140cf4d10` against the
frozen image before the run-122 probe enabled readiness observations. Its ordered
gates are:

| Gate | Source condition |
| --- | --- |
| First | Model Qword flags `+0x90`, bit 35, must be set |
| Second | Model Qword `+0x238` must be non-null |
| Third | Unless argument bit 0 is set, `0x140ce9ce0(model)` must return nonzero |
| Fourth | Unless model byte `+0x97` bit 0 or argument bit 1 is set, `+0x200` must be non-null and `0x140a805d0` must return nonzero |
| Optional descendants | Argument bit 2 also checks child models |

Both outcomes pass `0x140cf4e1a` with EAX equal to zero or one. At that address
RSP is entry RSP minus `0x28`, which the probe uses to pair sampled entries and
returns by thread and stack. No target function is called by the probe.

The observed caller is `0x140a5d2d0`: it takes the model at its own `+0x2a0`,
passes argument zero through virtual `+0x540`, and returns immediately at
`0x140a5d305` when the result at continuation `0x140a5d301` is zero. The probe
allows every caller; sampling is limited to one entry/return pair per current
model per second. Only this caller appears in the retained sampled pairs.

## Run 121 limitation and run 122 result

Run 121 showed non-null scene/manager/model objects, but its samples followed UI
rebuilds. Readiness sites exhausted their raw-stop limits on unrelated contexts;
there was no usable late model-readiness observation. The state-3 snapshots from
that run therefore did not prove processing remained in state 3 during the soak.

Run 122 uses `creator-preview-late-probe-20261105.py`, SHA-256
`7f466bd312bf5eb62571c3e8eb12a72b61d31399ca195fddad72368ead9e6ece`.
It binds GDB conditions to the current scene, composite, and model, and follows
UI frame `0x14029ccd0`, composite processing `0x1412d25e0`, and model readiness.
It emits state changes and two-second heartbeats, with limits of 90 seconds,
16,000 matched callbacks, 256 output records, 64 post-build callbacks, and eight
outstanding sampled returns. Ten host-only mock checks passed before injection.

The final retained run-122 log has 103 preview records, no malformed preview
JSON, and no probe failure/limit event. The UI's post-soak marker exists.
Appearance scene 3 is observed from 18.229 through 47.608 seconds after the first
owned post-build callback: a 29.379-second interval.

| Observation | Rebuild / first scene-3 sample | Late appearance sample |
| --- | --- | --- |
| Composite/model | Non-null; manager model matches UI model | Same owned objects, still matching |
| Composite state `+0xb44` | 3 | 4 |
| Composite flags `+0xb1c` | `0x17007` | `0x5007` |
| Geometry-invalid `0x2000` | Set | Clear |
| Waiting-on-model-geo `0x4000` | Set | Set |
| Texture-invalid `+0xae4` | 32767 | 32767 |
| Model flags `+0x90` | `0x8881000700400` | Same |
| Model flag bit 35 | Clear | Clear |
| Model fields `+0x238`, `+0x200` | Both null | Both null |

At elapsed 19.343 seconds the composite processing function is entered for the
appearance manager with choice zero. By 19.388 seconds the UI-frame sample sees
state 4 and flags `0x5007`. There are 26 sampled appearance-model readiness
returns, all zero, between 19.383 and 47.140 seconds. Five earlier scene-2 returns
are also zero. The final UI heartbeat records 208 UI-frame callbacks,
140 readiness-entry callbacks, 31 sampled readiness returns, 22 post-build
callbacks, and 70 composite-processing callbacks. The composite count remains
70 after that first scene-3 processing call while UI/model callbacks continue.

The earliest failed model-readiness test is therefore the **clear bit 35**, not
an inferred texture or draw failure. Null `+0x238/+0x200` values are additional
observed state; the sampled readiness calls return before reaching their tests.
These observations cover a bounded, instrumented run and do not show that the
model can never become ready or establish the cause of the unready state.

## Narrow source follow-up

`0x1412d25e0` state 3 calls geometry update `0x1412d7770`, sets state 4 at
`0x1412d26b3`, and returns one when its choice argument is zero. Geometry update
clears `0x2000` at `0x1412d7d70`; that source path explains the observed transition
without implying model readiness. For the base model it sets the waiting flag
`0x4000` at `0x1412d7bf6`.

Geometry submission calls model virtual `+0x218` at **`0x1412d7d0b`** or
**`0x1412d7d27`**, with RCX=model, EDX=count, and R8=the pointer array of geometry
references. The manager's base geometry reference is `+0xa18`; component
geometry references are assembled earlier in the same function. This is a
specific next passive observation boundary for the owned model. Reference
representation must be resolved before interpreting values as archive file IDs.

State 4's inspected path calls `0x1412d21a0`: it acquires paired scratch texture
objects at manager `+0xb58/+0xb60`, performs source-texture copies when needed,
and can pass those objects to model virtual `+0x8a8` while waiting flag `0x4000`
is set. Run 122 does not sample that function or prove it was entered. Follow-up
should distinguish geometry submission/load completion and the writer of model
flag bit 35 from composite scheduling/scratch-resource progress. The present
result does not justify changing server 1050 or inventing a response to client
416; see [the client-416 source trace](209026-creation-client416.md) and the
independent [navigation-control investigation](209026-creation-navigation.md).

## Evidence and reproduction scope

Private artifact paths:

- `/home/blizz/gw2-sandbox/runs/run122-norn-creator-paint-probe/client.out`,
  143078 bytes, SHA-256
  `3a08d7b056b7f0459fafb76d73ae68509c36afb7e4ab5ea35fcb13800ba45c52`.
- `/home/blizz/re/gw2/notes/creator-preview-late-probe-20261105.py` and
  `creator-preview-late-probe-mock-20261105.py`.
- Safe-prefix filter `creator-preview-run122-filter-20261105.py`; it prints only
  probe JSON, aggregate counts, marker presence, and whole-log hash.
- Exported REA bundle
  `/home/blizz/re/gw2/notes/209026-creator-run122-preview-20261105.json`.

Key REA records:

| Evidence | Coverage |
| --- | --- |
| `ev_914ed50d14aa8c5f1ced997ef4d30026aad059eec6a4871bc957f8e49c2e6207` | UI preview stores and context call |
| `ev_6eddbad742fa6218af87280b500f73bfb5f90995afa11278a2f1685fe280eb57` | Context construction/reuse path |
| `ev_2906f80f57eef05fc88d7481bbc408b5d1797bbb30674c367f6062f5b0359610` | Composite readiness predicate |
| `ev_83857e83b1585feb306c7451d9f340c596ec1fea76b549d0ccc38dd2f12755c9` | UI frame entry and scene argument |
| `ev_d52f0de84f1bee94a5bdc4fea4776651b45abf3cd77092000e7aa5f60c898b98` | Syntax and ten synthetic probe checks |
| `ev_8ac5b24b174080ef32d6b0fb4bcaf02b9cf8d2c739f8615c84818b60474a9de4` | Observed caller, complete model-readiness predicate, composite state transition |
| `ev_7f5586294a21c4faf476844a985cb6973bce95d1c723fd635ce10ed1f20aedbc` | Geometry submission and state-4 scratch-resource path |
| `ev_0488ef0e5a2dd4c996ee1ce193821bcc34ea6f41077fa06f5d4f03f4217a522a` | Full retained appearance-event extraction |
| `ev_f984e0d9f2f463249039b55a941cb329621c1a1dbe35d3cbdb71a3c0dd67f2eb` | Stable final log hash, post-soak marker, compact final result |

The source and filter captures exited zero without output truncation. REA
verified their owned-process cleanup; some captures also report unrelated
`/proc/.../environ` permission unknowns. This audit only read the existing log
and frozen executable. It did not attach another debugger, launch a client,
change target state, or inspect credentials.
