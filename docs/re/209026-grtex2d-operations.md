# Build 209026: GrTex2d readiness and queued operations

This note follows the concrete texture implementation observed in creator run
126. It is static source evidence, not a new runtime observation or a proposed
bypass. The frozen executable SHA-256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Runtime identity, probe bounds, and the subsequent run-128 field values are in
[the creation model pipeline note](209026-creation-model-pipeline.md).

The observed vtable `0x141bfc700` has readiness method `0x140a73860` at slot
`+0x48`. That method returns true only when texture-data flags `+0x60` bit
`0x10000` is set **and** the tagged head at `+0xa8` is empty. This second field
is now source-named **`m_opList.Head()`**. It is a pending operation list, not
merely a numeric GPU-fence value.

## Source-backed field and flag names

| Texture-data location | Established meaning |
| --- | --- |
| `+0x40` | `m_bits` |
| `+0x48` | `m_dataDims` (two 32-bit dimensions) |
| `+0x60` | `m_flags` |
| `+0x64` | `m_format` |
| `+0x68` | `m_levels` |
| `+0x98` | `m_opList`, intrusive link offset 8 |
| `+0xa8` | Operation-list head; zero or low-bit-tagged means empty |
| Flags `0x2` | `GR_TEXTURE_BLIT_SOURCE` |
| Flags `0x4` | `GR_TEXTURE_CPU_ONLY` |
| Flags `0x100` | `GR_TEXTURE_NOT_READY_ON_CREATE` |
| Flags `0x10000` | Required readiness bit; exact enum name remains unknown |

The names come from assertions in `GrTex2d.cpp`, notably `0x140a740c0`,
`0x140a70ae0`, and constructor callers around `0x140a71990`. The constructor
`0x140a6be30` installs the observed vtable and initializes `m_opList` with link
offset 8 at `0x140a6bfb1`. The same class is explicitly named in the assertion
`!CIGrTexture2dData::CheckDimsError(dims)`.

## Bit-setting paths

`0x140a71800` constructs the texture-data object, calls `0x140a740c0` to
prepare its CPU backing, and sets `m_flags |= 0x10000` at **`0x140a71950`**
when preparation succeeds and caller flags bit 8 is clear. If that caller bit
is set, this path leaves the readiness bit unchanged. The later bounded trace
identifies this flag as `GR_TEXTURE_NOT_READY_ON_CREATE`; see below.

`0x140a740c0` requires CPU-only or blit-source flags. It validates format,
dimensions, and mip-level count. If `m_bits` is absent it calls the backing
allocator at `0x140b3e7a0`, stores the resulting bits object and these metadata
fields, and returns whether `m_bits` is non-null. Existing backing must match
the supplied metadata. This is concrete CPU-data preparation, not a GPU-fence
wait. The readiness bit can therefore be set before all later queued blits
finish; `m_opList` is a separate part of the readiness predicate.

Other inspected constructor paths set the same bit at `0x140a71ae4` and
`0x140a71d2f` in their CPU branches. Those sites establish additional producers,
not an exhaustive inventory of every producer in the client. Reset-like
`0x140a76480` and destructor `0x140a6cab0` clear it on their resource cleanup
paths. No run-126 observation establishes which constructor or bit writer
produced a particular failed texture.

## Deferred blit insertion

`0x140a70ae0` is the source-backed CPU blit entry. Its assertions require
2D texture wrappers, CPU-only source and destination, and a blit-source flag
on the source. It can process immediately through **`0x140a743d0`** when the
source is ready and the destination has no queued operations. The callee's
assertion names `source.IsReady()` and its diagnostic names `ProcessBlt()`.

The defer branch at **`0x140a70da6`** allocates a 0x68-byte operation and calls
**`0x140a6b940`**. The immediate path also uses that constructor if processing
returns asynchronous work. The operation stores destination texture-data at
`+0x28` and source texture-data at `+0x30` and installs its own vtable
**`0x141bfc780`**. This is a distinct table immediately after the texture table;
its slots must not be mistaken for additional texture methods.

The constructor then:

1. Appends the operation to destination `m_opList` (`destination+0x98`) via
   generic insert type 2 at **`0x140a6b9e3`**.
2. Appends/relinks the destination in global **`s_textureOpTargets`**, list
   object **`0x14268b760`**, via the same insert type at **`0x140a6b9fb`**.
3. Adds the destination to the source's dependent-texture array, unless already
   present, using source fields `+0x20` / pointer `+0x28` / count `+0x34`.

The global list's head is Qword `0x14268b770`; its first Dword supplies the
embedded link offset to generic list operations. That offset has not been
assigned a numeric value from this slice. Generic insert first detaches an
already-linked node before placing it at the requested position, so repeated
queueing can move a target without adding a second intrusive link.

## Processing versus cleanup

The concrete blit-operation table is:

| Slot | Target | Established behavior |
| --- | --- | --- |
| `+0` | `0x140a6d0e0` | Destructor path |
| `+8` | `0x140a6fc50` | Performs `ProcessBlt` or polls its existing work |
| `+0x10` | `0x140a73790` | Tests source texture `+0x30` with the same readiness predicate |
| `+0x18` | `0x140a75190` | Clears the source's dependent-texture array |

Normal per-texture processor **`0x140a6ff50`** walks `m_opList`. Each operation
must pass virtual `+0x10`. It also compares operation rectangles to preserve
ordering where earlier operations overlap. An eligible operation runs virtual
`+8`; a zero result keeps it queued, and success invokes its destructor. The
texture is unlinked from `s_textureOpTargets` only after `m_opList` is empty.
Thus a ready destination backing can still remain unready because a blit's
source is unready or that operation has not completed.

Two direct callers were found in the bounded GrTex2d interval:

- **`0x140a6e6d0`** advances targets from a context list at `+0x68`, calling
  the per-texture processor at `0x140a6e723` / `0x140a6e732`. It also advances
  file/resource work and asserts
  `advancedSomething || !s_textureOpTargets.Head()` at its tail.
- **`0x140a70150`** visits the dependent-texture array and invokes the same
  per-texture processor at `0x140a70193`.

The inspected interval did not expose an upstream direct caller of
`0x140a6e6d0`. Its registration/calling context and execution frequency remain
unknown here. The context's `+0x68` list must not simply be equated with the
static global list without further source evidence.

Separate **`0x140a6e4e0`** drains `s_textureOpTargets` by detaching each target,
calling operation virtual `+0x18`, and deleting operations. It does not follow
the normal predicate/virtual-`+8` path. A local direct call appears at
`0x140a72e16`. This cleanup/drain is not evidence for normal frame processing
and is not proposed as a repair.

## Scope, next evidence, and provenance

The trace used known vtable methods and their direct callees. An exact
reference query for the observed vtable was limited to
`0x140a6b000..0x140a6e000`; exact direct-call/global-list reference queries
were limited to `0x140a6b000..0x140a77000` (48 KiB). There was no image-wide
field-offset scan. Negative caller findings apply only to this interval.

The creator probe `8a487...` subsequently ran in run 128: all four sampled
textures had flags `0x3104`, with the readiness bit clear, and three had empty
operation lists. See the pipeline note for exact timing and sampling limits.
If operations are pending, a later bounded observation can identify the head
operation's concrete table and source texture state; if the bit is clear,
the actual preparation/completion path needs observation. These are diagnostic
branches, not grounds to force either condition true.

Key static evidence:

- `ev_844f3e0fd58d4777b6d54d4d8057b5e0b42e24d872a16dd72428028598e28523`
  and `ev_d218e917463f0f18aa88cb719a528df05d9525d3516398ed0ff07103cc81e4df`:
  exact vtable constructor and list initialization.
- `ev_e7d4027ae9b4b8eef2c4ff8e42df8d05473e3e0583656725ce95610735d43bb3`
  and `ev_3f54903b1ce1feac39e24c162fe6ab268af6a5709773061961af0582f2d7e65c`:
  named flags/fields, readiness setters, blit entry, preparation, operation insertion.
- `ev_801ea784cedb95bae65b49af66992ac804237494cae2741eca07d9a941c86b97`
  and `ev_d1603b8f44ec588ff16eddd9430f8e338a0ec4ebd15ede54416ec67dd17646ca`:
  exact global-list references, context advance, cleanup/drain.
- `ev_907af654965623d8e2fd9ddde43e3dbf68ffd505c954c94432f76bb6f5c0efe4`
  and `ev_1c2dcbf4e0e432493fa9a8e9ccf7e4fdbe7d717c2037cbc5d2de3dc047ed8a66`:
  normal operation loop, rectangle-order gate, removal on completion.
- `ev_80522751b9a8040beddac26746036530e509c25715afc934157ce7128bf8e2b3`,
  `ev_403a4ffac310361296e8e685dff38d32c66c17fe9708137a4cc6fd359055e691`,
  and `ev_9bf1f0efd4e70076b262b9fd47cc005a6d102d147fbf748fb6c8efaa8fbd615c`:
  operation table, cleanup hook, dependent caller.
- `ev_55b0f644a9b76a58ca92d2a298b1bab8f75f392d6aef8a3120b2367d2c9cc50c`:
  bounded caller query and its coverage limit.

Private complete export:
`/home/blizz/re/gw2/notes/209026-grtex2d-operation-producers-20261105.json`.
No runtime was launched or attached, and frozen run inputs were not edited.


## Exact upstream-reference query and entry validation

An additional authorized exact-address query covered executable sections for
E8 calls, E9 jumps, and REX.W RIP-relative LEA references to `0x140a6e6d0`,
and readable non-executable sections for aligned absolute-Qword pointers.
It found zero candidates in those encodings. A separate exact RVA32 and
self-relative32 data query found only the `.pdata` start-RVA record at
`0x142a1eafc`, not a registration.

The entry was then verified independently: `.pdata` gives
`0x140a6e6d0..0x140a6e94a`, unwind info `0x1423f712c`, version 1, flags zero,
28-byte prolog, and twelve unwind codes. There is no chained-unwind parent.
The preceding function returns at `0x140a6e6c1` and is followed by padding.
Thus this result is not explained by a chained interior `.pdata` fragment.
It still does not prove that the function or texture processing never executes;
other encodings, indirect construction, and other processing paths remain
outside this query.

Evidence:
`ev_4377d6db6409e37c6db4497976f33aa72c7b17efad9a8899ac5fdd209628fa4d`
(exact instruction/absolute-pointer encodings),
`ev_2efce4e6309deb0dc0e6fe2e3aeca0d3adad8dd1992e8fb92e7205e75dbd257c`
(exact RVA/self-relative data references), and
`ev_f3c7590726d0c4fcf08aca5a8f277d6530f1b0fb733374696c7ce550b5a0e498`
(entry and unwind validation). Runtime run 128 subsequently found all sampled
readiness bits clear, with three operation lists empty; see the pipeline note
before attributing the current failure to this operation-list servicing path.

## Intentionally unready composite scratch textures

The assertion at `0x140a7316e` names flag `0x100` precisely:
`!bits || !(flags & GR_TEXTURE_NOT_READY_ON_CREATE)`. Thus the run-128 value
`0x3104` includes CPU-only and intentionally-unready creation flags. A bounded
instruction inventory in `0x140a6b000..0x140a77000` included direct and
register-fed flag stores plus bit operations; it was not a claim to enumerate
every readiness producer in the image.

The composite creation path supplies exactly these flags:

1. `CpsComposite.cpp` function **`0x1412d21a0`** obtains two wrappers with
   calls to **`0x1412dd540`** at `0x1412d2212` and `0x1412d2229`. It stores
   them at composite **`+0xb58`** and **`+0xb60`**. Assertions explicitly
   name the latter `m_scratchNormal`; the former's exact source name is not
   established by this slice. Creation is deferred if global counter
   `0x1428a970c >= 4`; successful creation increments that counter.
2. `0x1412dd540` requests dimensions **1024 by 1024**, or **512 by 512**
   when its first argument is nonzero. The composite passes its flags
   `+0xb1c` bit 3 for that argument. The first texture uses format selector
   0 and the second selector 6. Table `0x142186490` supplies formats 26 and
   30; the getter remaps 30 to **31**, so the effective pair is **26, 31**.
   It supplies flags **`0x1104`** and mip argument zero to pool lookup
   **`0x1412dd930`**.
3. The lookup matches pool dimensions, flags, and mip argument. Pool
   constructor **`0x141562800`** stores those flags at pool `+0x38`.
   Pool acquisition **`0x141562910`** adds `0x2000`, preserving the existing
   flags, and calls **`0x140a71800`** with **`0x3104`**. That constructor
   prepares CPU backing but deliberately skips its immediate ready-bit
   store. A reusable existing pool entry instead uses its reset virtual;
   this trace does not assign every reset-path effect.
4. When composite flag `+0xb1c & 0x4000` is set, `0x1412d2358` passes both
   scratch wrappers to model virtual **`+0x8a8`**. For the retained model
   table `0x141ee0310`, the concrete target is **`0x140cfb1c0`**. It matches
   supplied blit-source keys and stores the wrapper at each model blit
   entry's **`+0x10`**, at **`0x140cfb309`**. This is exactly the alternate
   entry field selected in all four run-128 failed predicates.

This is a concrete source path explaining both `0x3104` and selected entry
`+0x10`. Runtime identity equality between each of the four run-128 heads'
alternate textures and its creating composite was not captured. Run 129 later
established that equality for its owned initial scene-0 model; see the pipeline
note for the bounded early samples and failed late diagnostic. The scratch
targets are allocated CPU images, not archive-file records: this constructor
chain does **not** supply a file or asset ID. Format selectors and blit-source
keys must not be reported as file IDs.

## Scratch completion requests and the next passive boundary

Composite processing **`0x1412d25e0`** advances `+0xb44` to 5 after scratch
setup returns 2, then calls **`0x1412d7f20`** at `0x1412d2717`. That function
walks component work with index `+0xb40` before reaching completion stage
`+0xb3c`:

| Stage | Source action |
| --- | --- |
| 0 | Call **`0x140a71e80`** for `+0xb58` at `0x1412d8222`; store stage 1 |
| 1 | Poll that wrapper through `0x140ab7e30`; then call `0x140a71e80` for `+0xb60` at `0x1412d8252`; store stage 2 |
| 2 | Poll `+0xb60`; then store stage 3 |
| 3 | Return 2 to the composite processor |

The second argument controls whether these stages continue in the same call
despite a not-ready poll; with that argument zero they return and retry later.
The caller derives it from the inverse of composite flag `+0xb1c` bit 4.

`0x140a71e80` is a CPU-only completion-operation submission. It allocates a
0x58-byte object, installs table **`0x141bfc7e0`**, stores destination
texture-data at operation `+0x28`, zeros group `+0x30` and work-array metadata,
and appends it to the texture's operation list and global targets list.
If the prior operation list was empty it immediately calls **`0x140a75110`**;
otherwise normal list processing must reach it. This operation is distinct
from ordinary blit table `0x141bfc780` and multi-source blit table
`0x141bfc7c0`. Its completion/destructor path includes a ready-bit store at
**`0x140a6c9fd`**; the separate completion trace establishes its work and
polling prerequisites. An empty operation list observed later does not by
itself prove that this submission or destructor ever ran.

A targeted passive observation should therefore first bind **one owned
composite**, its model, and its two scratch wrappers. Record only tags,
dimensions, format, flags, and stages `+0xb44`, `+0xb40`, `+0xb3c`; compare
the model's alternate blit wrappers to the scratch wrappers. Source-guarded
sites `0x1412d2217`, `0x1412d222e`, `0x1412d8222`, and `0x1412d8252`
can record creation returns and completion requests with native conditions
on the owned composite in RBX. One late marker-gated snapshot, before enabling
internal completion sites, can distinguish unfinished component work from
missing submission or work completion. If ownership becomes known only after
creation, report that early history as unobserved rather than reconstructing
it from the late fields. Internal work/destructor observations should filter
by those same two texture-data identities and stop after bounded transitions;
no continuous pump tracing, target calls, or state writes are required.

## Runner full-texture patch: bounded control-flow check

The owned runner's `apply_full_textures` changes the conditional branch at
**`0x140a6f57a`** into an unconditional jump to **`0x140a6f67a`**. The skipped
region computes reduced dimensions, may generate additional mip levels via
`0x140b4f870`, and adjusts/replaces image backing via `0x140b3f2d0`.
Both the original no-reduction route and patched route still reach the common
tail **`0x140a6f75d`**, which stores **1 at this load task's `+0x48`**.
For a CPU-only destination the tail is reached directly from the flag test
at `0x140a6f686`.

This does not support a direct claim that the patch skips the completion
operation's work-count initialization or ready-bit destructor. This load
task's `+0x48` is a different object/field from completion-operation `+0x4c`.
It also does not prove the patch harmless: changed source dimensions/backing
may have indirect effects outside this slice. No patch removal, ready-bit
override, or queue-policy change follows from this static observation.

Creation-side evidence:

- `ev_b6736ca4abd397f5f3ee4a0179293b77e8f5d7ef6e67dbe5312b135b0fa50164`
  and `ev_79ca76b7131be11ad3c51882c8eb3817354671eeee87b2c0143b08057a371212`:
  named flag and bounded writer inventory.
- `ev_08856c1b4c8c7f8b6b33e499e29f192c3edb4c1f7684948072505b4b0ed3bd68`,
  `ev_f11370e341a2761797fb5d2af7b44f1e395880ff0432d12ce5edaddfc9972626`,
  and `ev_87268dadc2d145a3f72efd39e63db4c2f2a3921c5d0c75deab23b221b6909dc1`:
  operation constructor, validated direct callers, composite stages, pool acquisition.
- `ev_b375453b6dd3879ec303479b1645773d5a146f54d2a03ca21f347725479e053e`,
  `ev_2dd64ada67905d602fa98bf795e4606afbcd10fb0398c46ac6c9776c142ca0e5`,
  `ev_04fa044b2feae613546884cb402298cd2f98f40ae1b183f0057c63556b88ebbc`,
  and `ev_e34df7f34abe7e11c190690b6f719eef367d893a6a3aaa60f42b2b19633b8df7`:
  scratch getters, pool matching/construction, and composite binding.
- `ev_3482cc7aac46bdfb811afd837131e0c5ba628ecfc376abe3972b2d6f1893f5bf`,
  `ev_82439a72ac3a53f309146a1df531ed3ff1848245a3ab9762ee824f8eeda653c5`,
  and `ev_682f1e3ddf60f85c91795d404e42922441d7606d39c27b0473950301a339651c`:
  concrete table slot, format table, composite progression, alternate-texture store.
- `ev_974dc96f2303c53e9a5bde4069a1ef07f6834bca31a8e0f62cf159c111ff6c73`
  and `ev_512875f3fdcae3a02a03d3a90004d5ad9dade03759c1095047c04f829da10642`:
  original mip-reduction branch and common load-task completion tail.

## Direct servicing gates: no demonstrated model-readiness cycle

The exact-reference query found one direct caller of `0x1412d7f20`, at
`0x1412d2717` inside `0x1412d25e0`. The dispatch table at `0x1412d2a5c`
maps composite states **2 and 5** directly to `0x1412d26e4`: assert a nonzero
component-invalid mask, derive the second argument from flag bit 4, and call
`0x1412d7f20`. State **4** enters `0x1412d26cd`, calls scratch setup
`0x1412d21a0`, and advances to state 5 only if that returns 2. It then falls
through into the same component call. The `+0xb1c & 0x4000` wait is tested
**after** `0x1412d7f20` returns 2, at `0x1412d2725`. Thus this direct call
does not wait for the model's finalization callback before requesting scratch
work. No dependency cycle follows from these gates alone.

Component index `+0xb40` can still prevent reaching completion stages.
`0x1412d7f20` returns early if component helper `0x1412d8950` returns zero;
the inspected helper resolves/loads component textures and submits their
blits, rather than testing the owned model's finalizer flag. Source-texture
loading, scratch copying, and scheduling remain separate possible blockers.

Two direct callers of `0x1412d25e0` were found:

- Wrapper **`0x1412d7db0`**, call `0x1412d7df6`, skips processing only when
  the component-invalid mask is zero and geometry-invalid flag `0x2000` is
  clear. It does not first require model readiness.
- Global composite processor **`0x1412d2410`**, call `0x1412d253f`, walks
  a list rooted at `0x1428a9730` / head `0x1428a9740`. Its per-pass boolean
  starts true. Once an earlier processed composite returns 1, that boolean
  becomes false; later composites in states **4 through 6** are skipped
  at `0x1412d24fd..0x1412d2513` for the remainder of that pass. Ready-model
  virtual `+0x540` checks earlier in this function service a separate array
  and are not a readiness gate around this list's call.

The scratch limit's exact source name is now established by this processor's
assertion: global Dword **`0x1428a970c` is `s_scratchTextureSetCount`**.
Together, the four-set limit and per-pass skip explain why observing only a
late appearance owner in state 4 could capture a downstream consequence of
earlier work. Neither behavior has yet been attributed to a runtime sample.

Evidence: exact caller query
`ev_aabb08b299fb2419ed8cf853bd594ea768bcd803d67e7b14c7eabfecce35b267`,
dispatch table
`ev_6f634acf349b512b513f42c217566e024b7ae24153d9e5e85cc5db72208a9b1e`,
component helper
`ev_5d792af3b00a272f13c6dec5bbc6b28d7306f900196e1dc1821e1b5d7bbe7590`,
processor callers
`ev_419e2fc7292bf1cc1caaf89c0d1a029eeae233e63cca1c54d32447714d789e73`,
and their bodies
`ev_dc6e0e4e7a890840f86a6da07bda75924a6d34e931c9466359c4aaf834b87a98`.
The complete CPU-work completion chain is in
[the separate completion note](209026-grtex2d-cpu-work-completion.md).

## Frozen lifecycle probe for the two-soak observation

New private helper:
`/home/blizz/re/gw2/notes/creator-scratch-lifecycle-probe-20261105.py`.
Entrypoint **`arm_creator_scratch_lifecycle(inf, slide)`**; safe output prefix
**`GW2_CREATOR_SCRATCH_LIFECYCLE`**. Frozen size **16,448 bytes**, SHA-256
**`f995b17f2534a9a115240a2cb5b963e92a478715d81acce2d0bb8c633f99af57`**.
All earlier helpers and run inputs remain unchanged. This replaces earlier
texture/queue helpers rather than being installed alongside shared sites.

The probe binds the first available creation scene's manager/model after its
local build, with pre-binding creation history explicitly unobserved. It
records the first four owner bindings and retains their tags plus the current
binding. Later scene changes do not end observation or reread stale objects;
the final current owner is sampled even when it has no scratch pair. Identity
associations are reported explicitly when the same texture pointer was
observed with multiple owner bindings; these associations are not a proof of
allocation generations or exclusive ownership.

Thirteen guarded sites include UI post-build and a sparse heartbeat,
`0x1412d25e0` entry, scratch-setup return `0x1412d26d5`, the `0x1412d7f20`
call, both scratch creation returns, both completion requests, and four
completion hooks supplied by the completion trace. Service-entry/call/return
samples are one-shot per tracked owner and sampling phase. Work hooks match
at most ten concrete texture-data identities, with one start, one producer
return, one pending and one completed poll, and one destructor join per
identity. A native nonzero-count condition repeats the source poll gate to
prevent zero/zero equality from repeatedly invoking Python. Destructor join
is labeled `post_completion_branch`, not as proof that the ready-bit writer
executed.

Snapshots include composite `+0xb1c`, `+0xb44`, `+0xb40`, `+0xb3c`, the
global scratch-set count, scratch dimensions/format/levels/flags and operation
head classification, plus exact alternate-entry equality with the model's
blit array. Deep operation-head reads occur only in marker snapshots. Heap
addresses are tags; no texture pixels, arbitrary strings, or speculative file
identifiers are emitted.

The initial marker is
**`/sandbox/logs/creator-initial-soak-complete`**. It produces an initial-soak
snapshot and rearms one service sample while allowing observation to continue.
The final marker remains
**`/sandbox/logs/creator-quiet-soak-complete`**. It samples the then-current
owner, rearms one final service sample, and ends after a sparse follow-up at
least five seconds later. The UI must wait for the safe-prefix
`initial_soak_snapshot_before_rearm` event before selecting Norn; writing the
marker and immediately changing scenes would not guarantee an initial-owner
snapshot. The records always include the actual sampled scene and owner.

Bounds are 210 seconds from first binding, 20 seconds after the final marker,
128 output records, 64 post-build callbacks, a 64-entry blit-array read limit,
and native heartbeat ignore count 30. Time limits are checked at callbacks,
not by an independent timer. No global pump tracing, target function calls,
explicit target writes, client launch, or attachment is performed by this
preparation. Native breakpoint traps still have cost; this is not an
uninstrumented timing baseline.

Syntax and 24 host-side checks passed, including both markers, early race
ownership, later-owner continuation, stale-object avoidance, producer
identity checks, pending/completed polls, partial-install cleanup, bounds,
pointer redaction, and real-image site guards. REA evidence:
`ev_00a59dcf3daf0395965f1cbb73ca1631ca72557a4900a177794081a1d589200b`.
The checks use mocked GDB/inferior memory; actual native command-list filtering
is supported by the earlier owned-GDB fixture, not newly claimed as tested
against a live GW2 process here.

Private source/probe export:
`/home/blizz/re/gw2/notes/209026-creator-scratch-lifecycle-20261105.json`.

## Normal advance caller resolved through a conditional tail branch

A later exact query of the known texture globals found the missing caller of
`0x140a6e6d0`. Leaf **`0x140a72d90`** loads `[0x14268b840]` into RCX, tests
it, and conditionally tail-branches to that function with **JNE** at
`0x140a72d9a`. A null context returns. The leaf has no containing `.pdata`
record, but its complete instruction sequence and surrounding padding were
read directly. The earlier E8/E9/LEA query did not include conditional branches;
its bounded negative result never established an absent caller.

The exact source-supported chain is:

| Site | Behavior |
| --- | --- |
| `0x14094d905` | Calls `0x140a70a90`, under source profiler label `ViewAdvanceTexture` |
| `0x140a70a90` | Leaf jump to `0x140ab8000` |
| `0x140ab8004` | Calls `0x140a72d90`; other texture work follows |
| `0x140a72d90..0x140a72da0` | Calls the normal context advance through a non-null conditional tail branch |
| `0x140a6e723` / `0x140a6e732` | Process each texture in context list `+0x68`, whose head is `+0x78` |

In the same `VdfContext.cpp` body, the profiler label `ViewAdvanceComposite`
and call `0x1412d8310` follow at `0x14094d90f..0x14094d91f`. This establishes
their order on this control-flow path. It does not yet prove that a creator
frame takes the enclosing path, that either call executed in run130, or that
an owned scratch texture was in the processed list. The enclosing view gates
and runtime servicing remain separate questions.

The normal advance walks active texture work before its resource-load and
deferred-release lists. Its final assertion is still
`advancedSomething || !s_textureOpTargets.Head()`. Source list membership,
an eligible head operation, actual worker completion, and a processing visit
must not be collapsed into one readiness observation.

The global query covered executable REX.W RIP-relative MOV load/store and LEA
references to `0x14268b760`, `0x14268b768`, `0x14268b770`, `0x14268b840`,
plus aligned absolute Qwords in readable non-executable sections. It returned
51 code candidates and no pointer candidates; 50 had decoded `.pdata`
instruction boundaries, and the remaining leaf was verified by the bounded
window above. Other encodings remain outside this query. Raw and normalized
terminal records were complete; cumulative rendered-terminal frames alone
were truncated in the global-query capture. All captures exited zero with
owned-process cleanup verified. No runtime, archive access, or patch occurred.

Evidence: global references
`ev_2bf0b48b4f646d9e19759df6f3380470c505af820b1ddd1e17da081d5649a1f6`;
leaf window `ev_746579568faceea6f4abb730b299d5b4555c4a5eabca76e8030816811b80090c`;
exact wrapper callers `ev_5da9ec0483eb96d2d7877dcf0efdcf2dc0a97138a5774e9bee635e2cf6e7cd7f`,
`ev_4e96174efe6d374e0f0832e648005c5a0748e05afdeb80a9c67fa315f5ee9a26`,
and `ev_416d428becf88ad1bf5dc2700fdcdf3be03140e6de2023ca27b455ed00d74e38`;
wrapper bodies `ev_943a2c6d7cc2c31afe8f263586f6e9fa451b732cb7337173f940899381b35896`
and `ev_2f626fef1190f15a6b6c25fd7111f8e3aa40daba5ea37ca997e332df55a74907`;
view labels/order `ev_dd18d4d2d631bfb36b111e6231b537301c51f8ba2f3817ab3a5056a2d23f591f`;
complete context advance `ev_28a99e2b3de6ddcf7729552b9372c10e0612aefe8023429e06ec088b3e32423f`.
Private export: `/home/blizz/re/gw2/notes/209026-texture-view-advance-chain-20261105.json`.

## Enclosing view gates and the independent composite entry

The enclosing entry is **`0x14094cc70`**, established by chained unwind
records, not the apparent fragment start `0x14094d688`. Its four `.pdata`
fragments start at `0x14094cc70`, `0x14094cde3`, `0x14094d569`, and
`0x14094d688`; the latter fragments chain back to the same function. The
normal instructions end at `0x14094da49`, followed by switch-table data.

Within this function, the inspected view branches do **not** select a path
that skips the texture call while retaining the composite call:

| Gate | Verified continuation |
| --- | --- |
| Context `+0xb90 & 2`, earlier context `+0x74` cases, and the mode checks before `ViewAdvanceStress` | Optional setup converges before the advance pair |
| `context->GetView()` through virtual `+0x110` at `0x14094d001` | Values 4, 7, 8, 11, 12, 15, 17 enter optional stress setup at `0x14094d029`; the other values 0..17 go directly to `0x14094d108`; both groups converge there |
| Unsigned view value greater than 17 | Fatal assertion for `context->GetView()`, not a normal alternate advance path |
| `comiss xmm6,xmm13` at `0x14094d4e4`, with XMM6 from context `+0x84` and XMM13 zero | `JBE 0x14094d8d4` skips the middle subsystem block but still reaches both advances |
| Remaining middle-block branches | Converge at `0x14094d688`, then reach the same pair |

The common path calls texture advancement at `0x14094d905`, then composite
advancement at `0x14094d91f`, with no local conditional branch between them.
This is static reachability on normal return from preceding calls. It does
not establish that creator frames invoke this entry, that the calls return,
or that the independently checked texture context `[0x14268b840]` is non-null.
No numeric view value is assigned the meaning "creator" by this trace.

The composite wrapper **`0x1412d8310`** is a complete five-byte leaf jump
(`e9fba0ffff`) to **`0x1412d2410`**. That function walks the active composite
list headed by `[0x1428a9740]`, using the link offset at `[0x1428a9730]`.
For selected composites, it calls virtual `+8`, then checks dirty mask
`+0xae4` and geometry-invalid flag `+0xb1c & 0x2000`. Dirty work reaches
**`0x1412d25e0` at callsite `0x1412d253f`**. The per-pass processing flag
can defer later composites in states 4 through 6; list traversal is not
proof that every object was processed. This wrapper contains no texture
advance call.

There is also an independent object entry **`0x1412d7db0`**. It returns 2
when the same dirty mask and geometry-invalid flag are clear; otherwise it
calls **`0x1412d25e0` at `0x1412d7df6`**, forwarding R8D as the processor's
EDX argument. Its caller leaf **`0x1412d71c0`** sets R8D to 1 and tail-jumps
to it at `0x1412d71c6`. Neither entry calls texture advancement. The exact
direct-reference query found this leaf; binding it to a creator UI call or
virtual slot remains unproven. A runtime hit at `0x1412d25e0` alone therefore
does not identify its caller or demonstrate normal texture servicing.

Useful passive observation anchors are the enclosing entry `0x14094cc70`,
texture callsite `0x14094d905`, the null-context branch `0x140a72d9a`,
composite callsite `0x14094d91f`, and the two distinct processor callsites
`0x1412d253f` / `0x1412d7df6`. The present source result supports observing
these boundaries; it does not justify adding a pump call or changing flags.

Evidence: chained bounds, switch, strings, and guards
`ev_4667a2e961722b0002c86cbbe3d47881becb5e4d211dace5aa7ab7adc036604a`;
enclosing disassembly
`ev_735014fa34a2ad0902931971336e5ffa6ebeadba21afa6b6401c3492fb72b7a5`,
`ev_15bae4b4674378bf21e32a07fd1f706f1d85f16fb6e41bc7418e24548deca294`,
`ev_3f7db2e00782aa0296e91fc9394aaf267f5b2a0f0fea1966e0130c7be50c22b3`;
corrected instruction boundary across the initial split at `0x14094d050`
`ev_58804922ac7d4e2ff70af7e7bf846511b759c63b405c729ef6663af2a9e09cb9`;
global composite body
`ev_a0f2c1c06c7d50761940c5bf2c60132bae5b2779e05edfbd315176620b274f39`;
independent object body and exact caller
`ev_b94cf590588f357d87b00b0e49bcd5d8fb9f908be76407cee1778c77d368927c`,
`ev_1025986ba85c48bfe1875f0db104ed7876ae2ca518b0873b77ccc5527d3cb616`,
`ev_c4bff5ca3b6ad55eb25c011017af6aeba171c395bb2579a90575a4ff694c621b`.
All reader processes exited zero. Raw and normalized terminal records are
complete; some cumulative rendered frames were truncated. Unrelated process
permission limitations remain in the full private Evidence bundle. No client
runtime, working archive, or target mutation was used in this follow-up.

Private export:
`/home/blizz/re/gw2/notes/209026-vdf-texture-composite-gates-20261105.json`
(SHA256 `357ec55e117c7c71d90aa7928b5c912db94e7b27c96cb386bba5859154ad4ec3`).

## Alternate entry bound to creator construction

A subsequent exact-address follow-up resolves the creator binding left open
above. Constructor `0x1412d16cc..0x1412d16d6` installs composite vtable
**`0x142185c58`**. Its **`+0xe0`** slot at `0x142185d38` holds
**`0x1412d71c0`**, the leaf that sets R8D to 1 and jumps to `0x1412d7db0`.
The literal-pointer query found this one reference to the leaf; the immediate
code-reference query found only its jump to `0x1412d7db0`. These bounded
queries do not enumerate every possible indirect caller.

Creator UI build `0x14029eb30` calls creation-context virtual `+0xd8` at
`0x14029ec17`; its established implementation is `0x14129cd40`. Within that
context method, the new-manager path sets R12D to 1 at `0x14129ce2a`, calls
the composite factory at `0x14129cf32`, and stores the result at context
`+0x450`. At **`0x14129d104`**, a nonzero R12D leads to **`0x14129d118`**:
call manager virtual `+0xe0` with EDX=1. The existing-manager path sets
R12D=0 at `0x14129cdf7` and skips this call. Thus new preview construction
has the concrete path `14129d118 → 1412d71c0 → 1412d7db0 → 1412d25e0`,
with processor choice argument 1, independently of the normal Vdf advance
pair. This call precedes the UI's post-build observation point
`0x14029ec21`; a probe first binding there cannot observe this same initial
processor invocation. It does not explain later processor choice-0 samples.

The complete inspected UI frame callback `0x14029ccd0..0x14029d2ec` has no
direct call to either alternate entry or manager virtual `+0xe0`. No further
callgraph expansion or runtime attribution is needed for the construction
mapping. Whether the creator also executes the normal Vdf advance remains
a separate runtime question.

Evidence: exact references
`ev_f2f626e1edc5b0789e36d836a1c2f2ee6892bc4b63fa16f4a34a3b1fcc5819e8`,
`ev_02305e8cf665b0c41ba0e462696090f6815cd2c14e9e2b7ead02e62c1529060e`;
creator UI routines
`ev_60fc6da7d6ac36a0689469ff885b4868a4941ea6db337f25f95f68fb7a60af85`;
context branches and exact slot call / constructor install
`ev_0ae9051b6b667b804d8c09976701917e3969fed9484682bb2a381de7482627ca`,
`ev_57470240f6271cb7e006ea74e2ff6214c0f5e5b1825bae551b64c510dec059a5`.
The aggregate UI/constructor capture truncates later constructor output;
the relevant constructor install was recaptured in full separately. The
context's initial split instruction was likewise recaptured from a verified
boundary. All readers exited zero; no runtime or working archive was used.
Full private export:
`/home/blizz/re/gw2/notes/209026-creator-alternate-composite-binding-20261105.json`
(SHA256 `1cfca6e75390cae5ffdc270db904ea31099de39499e3c352ffe96ee655dd98c1`).
