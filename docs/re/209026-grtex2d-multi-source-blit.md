# Build 209026: multi-source CPU composite operation

The concrete operation table `0x141bfc7c0` retains one primary texture and
up to five mask passes. Input readiness, asynchronous job completion, and
operation removal are separate conditions. Starting one such operation
increments the destination texture's outstanding counter once; its pass count
does not determine that counter. Its destructor does **not** set the texture
ready bit.

This is a bounded static trace of frozen
`/home/blizz/gw2-client/build-209026/Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The motivating run-130 observation was supplied by the runtime lane: initial
scene 0 at marker +54.847 s had this table at the first scratch texture's
operation head, destination `+0x94 = 4`, and a four-byte-aligned operation
whose address was four modulo eight. Neither its pass count nor its job was
sampled there. See [the model pipeline](209026-creation-model-pipeline.md)
for runtime provenance and bounds. This investigation launched no client,
attached to no process, and accessed no archive.

## Concrete methods and object layouts

| Vtable slot | Target | Established role |
| --- | --- | --- |
| `+0` | `0x140a6d3e0` | Deleting wrapper; calls destructor `0x140a6c720`, object size `0x110` |
| `+8` | `0x140a6fdc0` | Starts work or polls the existing job |
| `+0x10` | `0x140a737c0` | Checks every retained input's readiness |
| `+0x18` | `0x140a751d0` | Clears retained inputs' dependent-texture arrays during cleanup/drain |

| Object | Offset | Established use |
| --- | --- | --- |
| Multi-source operation | `+0x28` | Retained destination texture data |
| Multi-source operation | `+0x30` | Retained primary source texture data |
| Multi-source operation | `+0x38` | Destination point, two Dwords |
| Multi-source operation | `+0x40` | Source rectangle, four Dwords |
| Multi-source operation | `+0x50` | Active pass count, at most five |
| Multi-source operation | `+0x58 + 8*i` | Retained mask texture-data pointers |
| Multi-source operation | `+0x80 + 20*i` | Copied color-shift data |
| Multi-source operation | `+0xe4 + 4*i` | Copied per-pass flags |
| Multi-source operation | `+0xf8` | Additional request scalar; meaning not established here |
| Multi-source operation | `+0x100`, `+0x108` | Work-group handle and single composite-job pointer |
| Composite job | `+0` | Function callback, **not a vtable** |
| Composite job | `+0xf8` | Completion word, zero before submission and one after callback work |
| Destination texture data | `+0x94` | Aggregate outstanding work; incremented once by each start |

These offsets must not be combined with the CPU mip-operation layout at
`0x141bfc7e0`. In particular, this operation's `+0x50` is a pass count, not
a completed-job counter. Texture flags `+0x60`, ready bit `0x10000`, and
operation head `+0xa8` belong to texture data, not either operation or job.

## Construction and retained source identities

Entry `0x140a71180` asserts an original pass count of one through five using
`compositePassCount && (compositePassCount <= IMG_COMPOSITE_MAX_PASS_COUNT)`.
Each original descriptor is 24 bytes: mask wrapper pointer at `+0`, color-shift
pointer at `+8`, and flags Dword at `+0x10`. Flag `0x40` means the assertion's
`passIsActiveButMaskless`; it is permitted only for original pass zero and
requires a null mask. Other null-mask descriptors are omitted. Active
descriptors are compacted. If none remain, no operation is allocated.

The entry validates destination and primary source as format `0x1a`, allowing
the unset-format sentinel `0x26` at this stage; masks similarly permit `0x1b`
or that sentinel. Assertions name `IMG_COMPOSITE_DEST_AND_SOURCE_FORMAT` and
`IMG_COMPOSITE_MASK_FORMAT`. Flag `0x80` requires a color-shift pointer and
copies its 20 bytes; without it that pointer must be null. These are raw
format values, not a new mapping to public format names.

Constructor `0x140a6bc40` copies the compacted pass data, retains the primary
source and each non-null mask, and initializes group/job to zero. It queues
the operation in destination `m_opList` and the destination in global
`s_textureOpTargets` at `0x14268b760`. It also registers the destination in
each input's dependent-texture array, avoiding duplicate entries. Thus
operation `+0x30` and the bounded mask array are the exact texture-data
identities to observe; no filename, asset key, or current pool ownership can
be inferred from an operation's mere presence.

After construction, `0x140a71180` calls starter `0x140a75020` immediately only
if the primary source and masks were ready and the destination previously
had no queued operation. Otherwise the operation remains queued for normal
processing. The base intrusive-operation implementation accepts four-byte
alignment; a probe must not reject this concrete operation for failing
eight-byte alignment.

## Readiness, processing, and completion

Readiness method `0x140a737c0` tests the primary source first, then mask zero
if non-null, then masks one through `count-1`. Those later masks are required
non-null. Every visited input must have `flags & 0x10000` and an empty
operation head: head zero or low bit set. Any failed input returns zero.
This method does not test the destination's ready bit or outstanding count.

The normal destination processor described in
[the operations note](209026-grtex2d-operations.md) first calls this readiness
method and enforces ordering against earlier overlapping rectangles. Only
an eligible operation reaches virtual `+8`.

`0x140a6fdc0` has two paths:

- With a non-null group `+0x100`, it reads job `+0x108` through
  `0x14026f540`, which returns exactly the job's Dword `+0xf8`. Nonzero means
  this processing call returns success; zero leaves the operation queued.
- Without a group, it checks each retained input for raw flag `0x20000`.
  A set flag returns success without starting work. Otherwise it calls
  `0x140a75020` and returns zero. The symbolic name of this error-related flag
  is not established here. Normal entry still has the preceding readiness
  gate; inspecting this method alone does not remove that gate.

Starter `0x140a75020` creates a group through `0x140dd4340` and one job through
`0x140dd4780`, stores them at `+0x100/+0x108`, and increments destination
`+0x94` at `0x140a75055`. A zero-to-one transition inserts the destination in
the context list at `[0x14268b840]+0x68`. It then calls
`0x140a747c0`, whose diagnostic names `ProcessComposite()`.

That function requires actual destination/source format `0x1a`, mask format
`0x1b`, ready source inputs with CPU bits, nonzero mask levels, and mask
dimensions at least as large as the primary source. It obtains backing-bit
pointers, bounds the source/destination rectangles, and normally calls
`0x140b3d3b0` at `0x140a74e17`.

The job producer clears job `+0xf8`, copies the work inputs, and installs
callback **`0x140b3d4e0`** through `0x140168610` at `0x140b3d493`.
The latter helper simply writes the callback pointer to job `+0`.
Submission occurs through `0x140dd43a0` at `0x140b3d4be`.
Callback `0x140b3d4e0` calls image composite work `0x140c20d80`, then writes
**job `+0xf8 = 1` at `0x140b3d568`**. The JobManager scheduling mechanism and
its actual run-130 activity were not traced in this slice.

There is one bounded edge to preserve when interpreting handles: an entirely
out-of-bounds source rectangle logs
`ProcessComposite(): Source rect entirely outside bounds of file %S` and
returns without calling the job producer. The starter has already allocated
handles and incremented the texture counter. A non-null job alone therefore
does not establish submission or initialized completion metadata. This is
static control flow, not evidence that run 130 took that edge.

## Deletion and cleanup are separate from texture readiness

Destructor `0x140a6c720` checks whether group `+0x100` is non-null. If so it
requires the job pointer, calls group cleanup `0x140dd4380`, releases the job
through `0x140dd4870`, decrements destination `+0x94`, and removes the
destination from the context list on transition to zero. Job release tail
`0x140dd5490` decrements its reference count and returns its allocation to the
pool when the count reaches zero.

Both destructor paths release the primary source, all retained masks, and
the destination, then unlink the operation. **No ready-bit write occurs in
the complete destructor.** The later CPU mip-operation completion path has
that separate write; see
[the CPU work-completion note](209026-grtex2d-cpu-work-completion.md).
The virtual `+0x18` cleanup method merely frees/clears each input's dependent
array; it is not normal readiness or processing.

Consequently a destination counter of four establishes aggregate outstanding
work, not four passes, four masks, or four unfinished jobs. Completed jobs
can remain represented until their operations are processed and destroyed.
Other queued operations can have work already started; the observed head
does not summarize the whole destination list.

## Minimal bounded observation

Match an operation's concrete table to `0x141bfc7c0` and its destination
`+0x28` to an already bound scratch texture before reading this layout.
Require pass count `1..5`; reject malformed counts instead of reading an
unbounded array. Read the primary input and at most five masks, permitting
null only at mask zero and deduplicating the resulting maximum six input
identities. For each input, guard concrete texture table `0x141bfc700`, then
record flags `+0x60` and operation-head emptiness at `+0xa8`. The previously
validated texture dimensions/format/levels may contextualize those results;
no recursive operation-list or file-resource walk is needed.

For a non-null job, require job callback `+0 == 0x140b3d4e0` before labeling
`+0xf8` as composite completion. Label this a callback-guarded snapshot:
the allocator recycles jobs, so callback equality alone does not prove the
current operation passed submission. A post-start observation, producer or
callback visit can supply stronger lifecycle evidence. Never dereference a
cached job after its operation has been destroyed; revalidate the live
operation-to-job relation for each event.

| Hook | Owned identity at entry/site | Exact first 16 bytes |
| --- | --- | --- |
| Readiness `0x140a737c0` | RCX = operation | `4c8bc9488b4930f74160000001000f84` |
| Process `0x140a6fdc0` | RCX = operation | `4883ec284883b900010000007419488b` |
| Start `0x140a75020` | RCX = operation | `48895c24084889742410574883ec7048` |
| After start's processing call `0x140a750f2` | RBX = operation | `4c8d5c2470498b5b10498b7318498be3` |
| Destructor `0x140a6c720` | RCX = operation, before release | `48895c2418574883ec204883b9000100` |
| Job callback `0x140b3d4e0` | RCX = job | `48895c24084889742410574883ec500f` |
| Completion writer `0x140b3d568` | RSI = job, **before** write of one | `c786f800000001000000488b74246848` |

Use only already matched destinations, native breakpoint filtering, finite
event/identity caps, and no forced method invocation. These hooks were
provided to the runtime-probe author; this static investigation ran none.
The unknowns remain the actual retained inputs, their readiness, whether the
head's job was submitted/completed, and whether normal processing serviced
that completed job. No patch follows from the current evidence.

## Evidence

Complete private export, mode 0600, nine REA records:
`/home/blizz/re/gw2/notes/209026-grtex2d-multi-source-blit-20261105.json`.
SHA-256: `b366f0bfaf8892e18e6cbcec4d3b00a708af531d1138037660d469f1bdd0c5ed`.
All records exited zero, were nontruncated, and verified owned-process-group
cleanup. Some records report unrelated `/proc/*/environ` sampling permission
failures; those processes were not touched. `.pdata` can split logical
functions, so demonstrated continuations were captured as bounded instruction
windows through their returns. No broad field-offset scan was used.

| Finding | REA Evidence ID |
| --- | --- |
| Four exact vtable targets | `ev_b8be3dcf61df17c36f46123c85d2d852b8ffc7e3f6efc0f85b027b1542360a1c` |
| Methods and complete entry | `ev_d7f4485b8cdd292cff6af451f7d85870eeb288d3a65bb0e2f45bd5c12ab97645` |
| Constructor, start, ProcessComposite | `ev_c74507ae63057828be2e4558b106143d228ce89cc3ffba87f823832445eb19d5` |
| Complete readiness/destructor and exact completion getter | `ev_810af90eeeafbb763a90a2a0e23c5c914293e594c382cc2b0e9246d780985a44` |
| Initial producer/allocation chunks | `ev_1e85f4ec051517c0f7f0c00098856cf974204901bb80491b9401994a0f6a8952` |
| Complete producer and job-allocation continuation | `ev_4e35bd41440f305db817c6b44a8361e03a1ab80e57a746daefe4d8027331ddc5` |
| Worker completion writer and job release | `ev_55908254c49c67d10f72a7c113dd182f5b164ba9823abe97820607849a344b1c` |
| Exact hook guards | `ev_fa69d9b1e66a3b484050dcd68e34bfe89e5d1a8c425d19638708e3f0a7f33826` |
| Callback-pointer setter | `ev_507544b376efcc1b01e9f5a2c9ee26fd10c2efcb64ae1aabd78c09bf260cfcc6` |
