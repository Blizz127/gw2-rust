# Build 209026: composite component source acquisition

`0x1412d8950` acquires source texture handles and dispatches component
composition. Its incomplete return is not a poll of texture readiness.
Allocated source textures can proceed to queued blits while their data remains
unready. This distinction matters before attributing an early composite index
to a stalled file or worker.

This is static REA evidence for frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No run-129 late state, actual failing component, or runtime pool condition is
inferred here. No runtime, archive, or patch action occurred.

## Loop ordinal, component slot, and source records

Caller `0x1412d7f20` reads the loop ordinal from composite `+0xb40`. It maps
that ordinal through the fifteen signed Dwords at **`0x142185c18`**:

```
0, 1, 2, 3, 5, 6, 7, 4, 8, 9, 10, 11, 14, 13, 12
```

Thus ordinal four selects **slot five**, not slot four. The helper's first
argument is the component state
`C = composite + 0x18 + 0x48 * slot`; its second is the slot ID. The caller
skips absent definitions and slots outside its masks before calling the
helper at `0x1412d81ce`. A helper zero return keeps the ordinal unchanged and
makes this caller return one at `0x1412d8214`. A helper one return allows the
ordinal to advance. The caller's status meanings therefore must not be
confused with the helper's Boolean result.

| Component state field | Use |
| --- | --- |
| `C+0` | Component definition pointer |
| `C+8` | Optional variant texture wrapper |
| `C+0x10 .. C+0x40`, stride 8 | Seven source texture wrappers |

The helper resolves a composite-data descriptor with `0x1412d9da0`, passing
the Qword at `composite+0xa40`, the first Qword of the definition pointed to
by `C`, and zero. Its assertions name the first two arguments `raceToken`
and `compositeToken`. The descriptor lookup can use a
race fallback. A null descriptor is not a handled incomplete result in this
helper: the code subsequently dereferences it. Do not classify such a pointer
as ordinary pending file I/O.

The component definition's byte `+0x66` selects a 40-byte blit-rectangle set
via `0x1412da170`; the assertion calls it `blitRectSet`. The set's Dwords
`+8/+0xc` provide requested source dimensions. Caller flag
`composite+0xb1c & 8` halves both dimensions and changes source flags from
`0x1032` to `0x10b2`.

The optional variant lookup uses `0x1412daca0(raceToken, low16(composite+0x6e4))`.
Zero selector or an empty race variant table returns null. Otherwise it
selects a 48-byte record using `(selector-1) % recordCount`. The helper maps
component slots 0..7 to optional filename-pointer offsets:

| Slot | Optional record offset |
| --- | --- |
| 0 | `+0` |
| 1 | `+0x10` |
| 2 | `+0x18` |
| 3 | `+0x20` |
| 4 | None |
| 5 | `+8` |
| 6 | None |
| 7 | `+0x28` |

Other slots have no optional pointer selected by this switch. Slot five's
optional input therefore comes from variant-record `+8`.

The seven main source filename pointers come from these packed offsets of
the resolved composite-data descriptor, in channel order:
`+0x4a, +0x42, +0x22, +0x2a, +0x32, +0x3a, +0x52`.
Their requested formats, read through `0x1412dd7e0` from the exact seven-Dword
table at `0x142186490`, are **26, 27, 27, 27, 27, 27, 30**. The optional
variant requests format 27. Null filename pointers are absent inputs and
are skipped. These are source pointers passed to a function whose assertion
names `filename`; this trace does not assign archive IDs or human asset names.

## Exact incomplete and dispatch conditions

For normal returns, the helper returns zero in three cases:

1. `*C` is null at `0x1412d89b0`.
2. A non-null optional filename requires acquisition, but `0x140a72530`
   returns null at `0x1412d8b2c..0x1412d8b2f`.
3. A non-null channel filename requires acquisition, but `0x140a72530`
   returns null at `0x1412d8c0b..0x1412d8c0e`. This path releases the earlier
   channel handles and the optional handle before returning zero.

Existing non-null handles are reused by the helper. For newly acquired
handles, it calls `0x140a720a0`. That function reads **texture-data flags
bit `0x20000`**, not readiness bit `0x10000` and not `m_opList`. Its exact
enum name remains unestablished here. The helper accumulates that flag and,
if any checked input has it set, skips composition, releases its handles,
and still returns one. This is a skip-on-flag branch, not an incomplete return.

With the aggregate flag clear, the helper calls **`0x1412dd5e0`** at
**`0x1412d8c9e`**, passing both destination scratch wrappers and the source
handle array. It releases its handles and returns one afterward. There is no
source `IsReady()` call or file-state poll in the complete helper body.

The dispatcher selects a composition routine by descriptor byte `+8`.
One traced routine, `0x1412dd1d0`, submits advanced and ordinary blits at
`0x1412dd401` (`0x140a71180`) and `0x1412dd4ee` (`0x140a70ae0`). The ordinary
blit's deferred operation processing is described in
[the GrTex2d operations note](209026-grtex2d-operations.md). Dispatch or helper
success does not prove these blits have completed. The other dispatcher
branches and their complete source-array layouts were not expanded here.

## Why an acquisition can return null

`0x140a72530` first searches the source texture cache. A cache miss calls the
supplied constructor callback **`0x1412d8930`**. That callback obtains a pool
from `0x1412dd930` and tail-calls **`0x141562910`**.

Pool acquisition first seeks a reusable texture whose wrapper reference count
and texture-data reference count are both one. If none is found, it compares
pool count `+0x1c` with limit `+0x44` at **`0x141562979`**. The branch at
`0x14156297f` returns null when **count is greater than the limit**. The
comparison is strict `>`; do not rewrite it as `>=`. This pool constructor
supplies limit eight. If the limit branch is not taken, a null result from
texture allocation also returns null. These are source-supported acquisition
failure causes; neither was observed in this investigation.

This per-source-pool limit is separate from the global four-scratch-set limit
at `0x1428a970c`, described in the operations note. A low global scratch-set
count does not establish that an individual source pool has reusable entries.

On a non-null new wrapper, `0x140a72530` clears its readiness bit, creates and
binds resource-load state, starts the load path, and returns the wrapper.
Therefore incomplete file loading alone does not make this acquisition return
null. Waiting for file/texture readiness can instead be represented by a
retained blit operation. A pool-cap failure and a queued blit waiting on an
unready source are different states and require different evidence.

## Minimal passive observation boundaries

All component hooks must match `C` to the owned composite's calculated slot,
use finite transition caps, and emit pointer tags/nullness rather than raw
filenames. Existing guarded texture layout checks must precede reading a
wrapper's `+0x10` texture data. The useful boundaries are:

| Address | Registers and timing |
| --- | --- |
| `0x1412d8a6b` | R14=`C`, EDI=slot, R15=resolved descriptor, RBP=optional filename; before source acquisition |
| `0x1412d8b28` | RAX=optional acquisition result, R14=`C`; before storing the result |
| `0x1412d8c06` | RAX=channel acquisition result, R14=`C`, R15D=channel 0..6; before storing |
| `0x1412d8c38` | All acquisitions reached their common boundary; R14=`C`, R12D=skip-flag aggregate |
| `0x1412d8c9e` | Composition dispatch; RCX/RDX are destination scratch wrappers, R14=`C` |
| `0x1412d8d64` | EAX=helper return, R14=`C`; before epilogue restores registers |

A non-null acquired wrapper with clear readiness supports a source-readiness
dependency only when the pending destination operation is shown to reference
that same source data. A null acquisition localizes failure before dispatch;
it does not establish pool exhaustion until the
owned acquisition's pool branch and count/limit are observed. A successful
helper can leave pending blits. The early loop ordinal alone proves none of
these outcomes.

Exact sixteen-byte guards for these sites and the pool branches were sent to
the single lifecycle-probe author and retained in the evidence bundle. No
probe was executed or helper changed in this source task.

## Evidence

Private complete export:
`/home/blizz/re/gw2/notes/209026-composite-component-source-inputs-20261105.json`,
SHA-256 `c07f3674869052a995507f27b40cd3da6ddd2d45af6416aa20c6791a15a8f65c`.
All seven captures exited zero with complete output and verified owned-process
cleanup; unrelated process-environment sampling permission failures remain
recorded. Selected function windows were used where `.pdata` fragments split
a body. Embedded jump tables are decoded as data, not as objdump's incidental
instruction renderings.

| Scope | REA Evidence ID |
| --- | --- |
| Complete component helper | `ev_a482e468b9dd30a28cff97facd018227cff20b9dd8c3ac3cef998c5a056d7760` |
| Caller, acquisition, flag test, constructor callback | `ev_9b4768a0405791917e6c6c1126e4be1d87c6620e7547ab0dfeb69c37a34ec418` |
| Race/composite descriptor and rectangle lookup | `ev_ea9b010d4c772e632c103fade74978e95472a7d91ef5232cf7112dc11d76321a` |
| Component order and optional-input switch data | `ev_daa3c2786e079f58f44b34ffd7e0b5bc7057886a69dbcdaf12f04b0336ca8a67` |
| Full pool lookup, variant selection, composition dispatcher | `ev_0e7d3b5381d8d20a12cdd646bbc2a8a6ec9cc8252641f785085df591e245f41e` |
| Pool acquire/constructor, format lookup, representative blit path | `ev_1baaff50e3c3b7929d67e08ee00583966e395be1b8ccc7adbc725db483ea0046` |
| Exact channel formats and hook byte guards | `ev_4b79142466d3d815a768d4a7a28173ad8e10dd24be507523e8fdf3362f954ab4` |
