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
`+0x10`. Runtime identity equality between each sampled head's alternate
texture and its creating composite has not yet been captured. The scratch
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
