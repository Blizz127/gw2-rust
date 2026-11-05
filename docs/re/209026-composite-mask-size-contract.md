# Build 209026: exact failing composite-mask operands

The `Composite mask smaller than source RGB texture` assertion compares the
current operation's primary and each active non-null mask before rectangle
clamping. A failure-only observation at **`0x140a74a32`** can capture the
actual operands and operation within that native call. Earlier file-load
events with matching dimensions do not identify this failing pair.

This is a bounded static trace of frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
It was prompted by the runtime lane's run-140 report: primary 142103→142104
and mask 142101→142102 were each observed after transfer at 512×256, yet a
later size assertion still occurred. This source pass does not attribute the
failure to either ID. No client, archive, runtime helper, or target state was
accessed or changed.

## Exact condition and failure site

The size check belongs to **`0x140a747c0`** (`ProcessComposite`), after its
preparation call to `0x140a74280` returns. Its complete `.pdata` range is
`[0x140a747c0, 0x140a74e40)`.

The source loads primary dimensions from data `+0x48/+0x4c` into EBX/ESI,
then iterates the compacted mask array. For each non-null mask it verifies
backing, readiness, format `0x1b`, and nonzero levels before these comparisons:

```text
140a74a16  RDI = Qword[current_mask + 0x48]
140a74a1a  Qword[RSP + 0x78] = RDI
140a74a1f  compare EDI, EBX             ; mask width, primary width
140a74a21  JB 140a74a32                ; unsigned width failure
140a74a23  RAX = RDI
140a74a26  RAX >>= 32
140a74a2a  compare EAX, ESI             ; mask height, primary height
140a74a2c  JAE 140a74bf2               ; unsigned height success
140a74a32  first instruction of size-failure diagnostic
```

Therefore the failure predicate is
`mask.width < primary.width || mask.height < primary.height`, using unsigned
32-bit dimensions. There is no selected-mip adjustment, requested-size
substitution, rectangle intersection, or proportional scaling in these
comparisons. Larger masks pass this check; equal masks also pass it.

The failure path formats source and mask diagnostics and calls the assertion
at `0x140a74bc3`, supplying source line **2169** (`0x879`) and the quoted
message. Diagnostic calls overwrite EBX, ESI, and R15 before that final call.
The proposed site **`0x140a74a32`** precedes those changes and is reached only
after a dimension failure. If assertion handling returns, the code restores
loop registers and proceeds at `0x140a74bf2`; this does not make the assertion
an ordinary successful skip.

## Live fields at 0x140a74a32

Let **S** be current RSP. Values below are source-proven for this site, not
for the later assertion call or the separate empty-rectangle branch.

| Value | Current native location |
| --- | --- |
| Primary texture-data pointer | R14 |
| Current non-null mask texture-data pointer | RBP |
| Zero-based **compacted** pass index | R12D |
| Active pass count | R15D, also `Dword[S+0x380]` |
| Primary width / height operands | EBX / ESI |
| Packed mask width / height operand | RDI: low / high Dword |
| Primary packed dimensions copied before loop | `Qword[S+0x70]` |
| Mask packed dimensions copied immediately before comparison | `Qword[S+0x78]` |
| Primary width / height backup copies | `Dword[S+0x68]` / `Dword[S+0x60]` |
| Destination texture-data pointer | `Qword[S+0x88]` |
| Incoming mask-pointer array | `Qword[S+0x388]` |
| Work-group / job passed to this call | `Qword[S+0xb0]` / `Qword[S+0xa8]` |

**RDI is packed mask dimensions here, not the destination pointer.** The
compacted pass index must not be labelled the original descriptor channel;
the submission entry removes inactive null-mask passes.

## Same-call operation and lifetime guard

The established starter **`0x140a75020`** keeps its operation in RBX and
calls `0x140a747c0` at `0x140a750ed`, returning at **`0x140a750f2`**.
The callee pushes eight registers, starting with RBX, then subtracts `0x308`
from RSP. At the proposed failure site:

- `Qword[S+0x348]` is that return address; require exact `0x140a750f2`
  adjusted for the image slide.
- `Qword[S+0x340]` is the caller's saved RBX, hence the current operation
  for this guarded caller.
- Require operation vtable **`0x141bfc7c0`**, adjusted for the slide. The
  [established constructor](209026-grtex2d-multi-source-blit.md)
  `0x140a6bc40` installs this table and retains its source pointers. Four-byte
  alignment is valid; eight-byte alignment is not a source requirement.
- Require `op+0x28 == Qword[S+0x88]`, `op+0x30 == R14`, and
  `Qword[S+0x388] == op+0x58`.
- Require `Dword[op+0x50] == R15D == Dword[S+0x380]`, count in **1..5**,
  `R12D < count`, and `Qword[op+0x58+8*R12D] == RBP`.
- For additional same-call confirmation, match `op+0x100/+0x108` to the
  group/job arguments at `S+0xb0/+0xa8`. No job payload read is needed.

These checks recover the operation from the native call frame; they do not
guess that the texture's current list head is the operation being started.
The starter, constructor retention, and current stack establish the objects
needed for a single observation. Read them only while stopped at this native
site and discard all pointers afterward. No cached pointer follow-up,
generation inference, destructor hook, or target function call is needed for
that one snapshot. Other callers or unsupported tables remain unclassified.

For each of destination, primary, and current mask, require the known
texture-data table **`0x141bfc700`** before interpreting the layout. A bounded
snapshot can then report numeric width `+0x48`, height `+0x4c`, format `+0x64`,
levels `+0x68`, flags `+0x60`, backing `+0x40` presence, ready-bit state, and
operation-head `+0xa8` classification without following a chain. Check that
the current primary/mask dimensions agree with the native operands and local
copies, and report which inequality failed. Original operation point `+0x38`
and rectangle `+0x40` are optional context, not inputs to this size test.

File IDs are optional provenance. The bounded
[holder/compact-filename contract](209026-composite-source-filename.md) may
classify current high/low holders when their known type guards pass. Retired
or null holders leave the file identity unknown. Their absence must not
silently turn a generated scratch texture into a particular file-backed
descriptor input. The failure hook should not filter by the earlier 142103
or 142101 IDs; doing so could exclude the actual mismatch.

A small finite capture of the first few failure-site entries is sufficient.
There is no need for per-frame tracing, broad successful-pass observations,
or a change to readiness, pixel data, assertion behavior, or queue policy.

## Kind-0 route and the optional source outside the seven fields

The independently captured exact dispatcher `0x1412dd5e0` sends kind **0**
through `0x1412dd1d0`, calling it at `0x1412dd666` and returning at
`0x1412dd66b`. At its advanced submission `0x1412dd401`, R8 is source-array
entry **0**, from component state `C+0x10`. RCX is the separate destination
scratch wrapper. Thus this demonstrated route does not substitute the
destination scratch for its primary input.

The route constructs four requested 24-byte pass records:

| Requested pass | Mask wrapper | Color record | Flags |
| ---: | --- | --- | --- |
| 0 | null | 0 | `0xc8` |
| 1 | **Optional component `C+8`** | 1 | `0x81` |
| 2 | Source-array entry 2 (`C+0x20`) | 2 | `0x81` |
| 3 | Source-array entry 1 (`C+0x18`) | null | `4` |

The optional `C+8` input is selected through the existing race-variant path
`0x1412daca0`; it is **outside the seven descriptor filename fields** audited
in [the 20-ID cohort closure](209026-composite-primary-142103-cohort.md).
It may be absent. The submission entry compacts inactive null-mask passes,
while the first explicitly maskless pass remains active. Therefore requested
pass numbers above are not automatically the runtime compacted indices.
This is a concrete limit on that cohort closure, not evidence that the
optional input caused run 140's assertion. The one-site snapshot establishes
the actual operands but does not itself bind them to a creator descriptor.

Exact source evidence supplied by the parallel source lane:
`ev_656fa30fadb1915c08da1f44270340c057ebdaede2639998a22d747fa2074d91`
(dispatcher) and
`ev_ea215c6af7f4a3fc38ef74aea3c194d62b49dc2afa1b9a35a14e892732537967`
(complete kind-0 routine). Private bundle:
`~/re/gw2/notes/209026-kind0-composite-input-pairing-20261105.json`, SHA-256
`e0ad7badcf40439cd8a67bb971b9e0a74ecb434d083076a0668185eb7b8f5ee1`.

## Exact source guards and evidence

| Site | First 16 bytes |
| --- | --- |
| ProcessComposite entry `0x140a747c0` | `405355565741544155415641574881ec` |
| Local stack allocation `0x140a747cd` | `4881ec08030000488b05e5f3b1014833` |
| Mask dimension load `0x140a74a16` | `488b7d4848897c24783bfb720f488bc7` |
| Width branch `0x140a74a21` | `720f488bc748c1e8203bc60f83c00100` |
| Height branch `0x140a74a2c` | `0f83c0010000498b56584885d2750949` |
| **Failure-only observation `0x140a74a32`** | **`498b56584885d27509498b56504885d2`** |
| Assertion argument setup `0x140a74bb0` | `4c8d05d1831801ba79080000488d0d75` |
| Starter `0x140a75020` | `48895c24084889742410574883ec7048` |
| Caller `0x140a750ed` | `e8cef6ffff4c8d5c2470498b5b10498b` |
| Return `0x140a750f2` | `4c8d5c2470498b5b10498b7318498be3` |
| Operation constructor `0x140a6bc40` | `48895c240848896c2410488974241857` |

Complete current captures:

- `ev_0ca3060b9f57e9b8cf77e41c8de493335876e85597afb166569932419a551b80`:
  complete ProcessComposite and starter, assertion strings and exact operands.
- `ev_f1f68bf27815a4a2d124252c729e3acb854b77ce21d862d86a63ccf4336697c1`:
  guards, containing `.pdata` ranges, concrete operation/texture tables.

Both exited 0 without truncation, with owned cleanup verified. The first
also reports permission failures observing unrelated process environments;
those processes are outside the claim. Private guard reader:
`~/re/gw2/notes/composite-mask-size-guards-20261105.py`.
Complete bundle:
`~/re/gw2/notes/209026-composite-mask-size-contract-20261105.json`, SHA-256
`8670dea4f072a5635e2c9ed422f9b38d4e21bc08f7ad0a07192ea7dfbaafe8a3`.

## Private observer handoff

Prepared without runtime execution or staging:
`~/re/gw2/notes/creator-composite-size-failure-probe-20261105.py`, SHA-256
`d1cec0d3c6a3d7b0984d2ebecf8d7adab9043da79d5f04eb52e0ff28a7a5d92d`.
Entry: `arm_creator_composite_size_failure(inf, slide)`; prefix:
`GW2_CREATOR_COMPOSITE_SIZE_FAILURE`.

The observer uses one plain public command-list breakpoint at the
failure-only site and disables after the first qualified snapshot. It permits
at most four unqualified entries, producing at most five records including
the arm record. It keeps no object pointers after a callback. Output is
scalar operand dimensions, pass index/count/flags, bounded texture fields,
comparison results and scope indicators; optional filename IDs are omitted.
Image, file/code bytes, architecture, inferior and all-stop checks follow the
existing private observer pattern.

Twelve host tests cover same-call/source/group/job relations, four-byte
operation alignment and image slide, pass bounds before array reads, texture
type guards before field interpretation, every dimension-copy relation,
unsigned width/height failure, equality rejection, bounded read failures and
rejection cap, code/environment guards, one plain command-list breakpoint,
and no further reads after success. All passed in
`ev_dbc23954f735d20e8c743350690e79301c96941a11741c1e244956b3b773b208`;
the extra stack-allocation guard was independently read in
`ev_4a4083032840c02cd1c17cdec5c450105100d1f9959a31b06b399ba5dbb85330`.
These checks validate the observer against its source contract; they do not
supply a runtime failure attribution.

### Run 141 preparation: separate v2 with optional original IDs

A separate v2 preserves the same native site, operation/operand guards and
sampling bounds, adding only the conservative current-holder decoding
described in the [filename note](209026-composite-source-filename.md).
Private helper `creator-composite-size-failure-v2-probe-20261105.py` has
SHA-256 `d1a63ac12baa23fc92cbe3672a9d5691f79185e0b754052d2b5bbbd1f56799cf`.
The v1 source and its prepared manifest/GDB/wrapper were preserved separately.

`primary_holders` and `mask_holders` each contain `low` and `high` results:
absent, recognized original ID with encoding size/optional byte, or unknown
with a bounded reason. A failed optional read leaves the scalar failure
qualified. `file_identity_observed` reports whether any holder original was
recognized; `file_identity_scope` is `current_holder_original_ids_only` and
`selected_file_identity_observed` remains false. No ordinary text, raw
pointers, target calls/writes, or cached-object observation was added.

All 19 host tests passed in
`ev_8c95eead8cdc5485644326711391b3da39345d81f69458873e2960d016b9e27e`.
The new coverage checks holder tables, interface/back-reference/key-class,
six/eight-byte decoding, ID equality, local unknown handling and preservation
of other holder results. The initial malformed test fixture was corrected
because its first word was actually valid compact encoding; no helper change
was needed for that test correction.

Run 141 was staged from the completed run-137 prefix, with unchanged remap20
CDN files, server/UI semantics, kind-9/load observers and production runner.
The separate v2 staging capture
`ev_635960b3646fcfb33b4619d4527dbcd87e5ef9a2e95376fc2d6f1b1244ee4e5d`
verifies 1,838 other manifest entries unchanged, 11 disjoint native command
list sites, and source-equivalent production code after removing observer
insertions. The prepared manifest SHA-256 is
`f545a3128d339deb783d4104d20cee63f316cf1a8252f9571c4bb4b3027fe4a3`.
This paragraph records preparation only; the resource gate remained present
and no client, container or working archive was accessed by this lane.

Complete private source/test/preparation receipt:
`~/re/gw2/notes/209026-run141-size-failure-v2-preparation-20261105.json`,
SHA-256 `63de2c90209f0565d2ad1aa2ea3f9e1066fdff21ef0304958d6cdb0107b56775`.
