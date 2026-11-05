# Build 209026: demonstrated kind-9 composite rectangle route

Descriptor kind nine dispatches `0x1412dc660`. Its complete body constructs
source rectangles by subtracting a common authored origin. It does not read
loaded source dimensions, proportionally rescale coordinates, or rotate the
rectangles before submission. The next useful observation is the exact
same-invocation request at `0x1412dcaba`, linked to the current component.

This extends, rather than reattributes, the earlier representative
[`0x1412dd1d0` trace](209026-composite-authored-rectangles.md).
All new source queries read frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
There was no runtime, archive access, target call, or mutation in this lane.

## Actual tuple and evidence separation

Root's run-136 current appearance sample identified owner seven, scene three,
component slot nine, descriptor kind nine, rectangle set three, half-size
flag zero, and acquisition request 1024×1024. Its IDs were primary 140521,
masks 140515/140517/140519, and normal 140523. Root's separate immutable
header check found all five files declare **512×512**; formats were DXT5,
DXTA, and 3DCX respectively. That archive evidence is
`ev_9581d7c770e57533c5f05ee1f0e31c930d78c8c0bdb4e1acc19bd8ca78f270b8`.

The old run-132 ready 512×128 source and its out-of-range request belonged
to an initial-scene owner. Neither those dimensions nor that rectangle are
attributed to the Norn appearance tuple. Pending 1024×1024 metadata at
dispatch is also not a measurement of the eventual ready runtime dimensions.

## Three lists and the shared origin

Let `D` be the resolved descriptor and `R` the rectangle-set record.
`0x1412dc660` gets its seed width/height from `R+8/+0xc`; half-size mode
shifts both by one. Its three calls to existing list builder `0x1412dc050`
use these selections:

| List | Build call | Extra category mask for kind nine | Use |
| --- | --- | --- | --- |
| A | `0x1412dc794` | `Dword[D+0x5e]` | Origin reference |
| B | `0x1412dc7ec` | `Dword[D+0x5e] & ~Dword[D+0x62]` | Ordinary primary and normal copies |
| C | `0x1412dc840` | `Dword[D+0x5e]` | Advanced color blits |

The exact kind-nine table entry in `0x1412dc050` is `0x1412dc0d5`, which
selects base category zero. Each list includes that base category plus its
mask-selected categories. The builder's other input flag and optional-input
Boolean do not alter kind nine's base-category branch. Thus A and C use
the same selections for this kind. The route's extra `0x200` bit applies
only to kinds eleven and thirteen, not nine.

Category selection reads index ranges through `R+0x14`, and copies 16-byte
rectangles from `R+0x20`, as previously sourced. Half-size mode shifts every
coordinate by one at that copy boundary. Run 136's flag zero selects neither
the dimension nor coordinate halving path.

Each submission loop computes its origin as the minimum left/top over A,
seeded by the authored width/height. For selected rectangle `[l,t,r,b]`:

```
destination_point = [l, t]
source_rectangle  = [l-origin_x, t-origin_y, r-origin_x, b-origin_y]
```

A 512×512 asset can be appropriate when the selected relative extents fit
within that area even though the acquisition request was 1024×1024. If a
linked relative request starts at or beyond the ready source's width/height,
the downstream clamp can make it empty. This source slice does not decide
whether any such selected region is authored intentionally, whether the
actual Norn request has that extent, or why its assets have their dimensions.

## Exact submissions and differing pass layout

| Call site | Callee | Primary source | Destination |
| --- | --- | --- | --- |
| `0x1412dc930` | `0x140a70ae0` | Source array `[0]` | Color scratch |
| `0x1412dcaba` | `0x140a71180` | Source array `[0]` | Color scratch |
| `0x1412dcbab` | `0x140a70ae0` | Source array `[6]`, if non-null | Normal scratch |

The advanced call supplies five 24-byte pass records. The first takes
source array `[1]`, a null color pointer, and mode **4**. The remaining four
take source array `[2..5]`, successive color records at offsets
`0/0x14/0x28/0x3c`, and mode **0x81**. These are the actual route's values;
the earlier representative route's mode `0xc8` must not be substituted.
Null/inactive inputs may subsequently be compacted by the submission layer;
five requested pass records do not prove five active sources or completed work.

## Minimal linked submission observation

Pair entry into this exact routine with the already verified current owner,
descriptor kind nine, component/source array, destinations, rectangle set,
and caller. At **`0x1412dcaba`**, before its native call:

| Register/field | Meaning at this instruction |
| --- | --- |
| RCX | Current color destination wrapper |
| RDX | Destination point, also `RSP+0x60` |
| R8 | Current primary source wrapper |
| R9 | Source rectangle, also `RBP-0x18` |
| R15D / ESI | Origin x / y |
| `R14-8` | Current selected rectangle before subtraction |
| `Qword[RBP+0x110]` | Descriptor pointer |
| `Qword[RBP+0x118]` | Rectangle-set pointer |
| `Qword[RBP+0x120]` | Seven-source array pointer, component `C+0x10` |
| `Dword[RBP+0x130]` | Half-size flag |

The complete prologue establishes original entry stack `RBP+0xe8` and
current `RSP == RBP-0x100`. The original return address must equal
`0x1412dd701 + slide`, immediately after the dispatcher's exact call.
Use same-call stack pairing and revalidate the current owner and pointers;
address equality across separate lifetimes is insufficient. Optional list
inspection must bound A/C counts and validate readable ranges, with explicit
null/read-failure/truncation status. No getters or target calls are needed.

For executable/source guards, the sixteen bytes at the advanced call are
`e8c14679ff4d8d7610498d46f8493bc5`; the entry bytes are
`4c8bdc5541554156498dab18ffffff48`. These bytes supplement full frozen-image
and function/caller validation; they do not establish object ownership.

## Evidence

Private complete bundle:
`/home/blizz/re/gw2/notes/209026-composite-kind9-rectangles-20261105.json`,
SHA-256 `5890a3bbe1da724572258b0ef4bbdf7ba38a1b76550690c6fffdeb08a89f0342`.
All three source captures exited zero, retained untruncated output, and
verified owned-process cleanup.

- `ev_3dfc0fbfded25772d75104d81921685837c57a1c559126526885d14a6aef58c5`:
  complete routine through its final return, all lists and submissions.
- `ev_961467e9ee105b540ce9afafec013914f11b7831cdbd103717c75b5044e27bba`:
  exact kind-nine selection table entry and bounded instruction guards.
- Retained `ev_aaf09bcb0714c46d395f7a6ae271384bbe218ef6bf30799bcdfb8f690094dce7`:
  component arguments, dispatcher, and complete category-list selection;
  available in the earlier authored-rectangles evidence bundle.

## Paired passive observer prepared for run 137

The complete parent/child stack proof is retained separately in
`/home/blizz/re/gw2/notes/209026-kind9-parent-frame-pairing-20261105.json`,
SHA-256 `cb823eb55d5b3a6d915a3fab2977c88f2b34f79e25279d162a983bfde67d4baa`.
Let `A` be RSP at parent `1412d8c9e`. At `1412dcaba`, current
RSP is `A-0x250` and RBP is `A-0x150`. The two required returns are
`Qword[A-8] == 1412d8ca3+slide` and
`Qword[A-0x68] == 1412dd701+slide`. At `1412d8ca3`, RSP is again A
and R14 is the component; this continuation precedes the first wrapper
release. A return reached without a matching observed parent call supplies
no authority. Evidence:
`ev_147c45d0f3a8474ae793743478fcabbe751a26a806b79f5ad8919babe9cb2b2e`
and `ev_096a972be307e6d98a643952cce45f0ba50437ac5d2a622808b7bd48db5dfa01`.

Private helper `creator-kind9-rectangles-probe-20261105.py`, SHA-256
`70729da1925b2ae6e7761822fe5768d96a13d97b57938496430b7995657a76a1`,
uses five sites: post-build ownership, sparse UI state, parent dispatch,
advanced submission and the matched parent return. It admits the first
kind-nine slot-9 dispatch per current owner in scenes 2/3. It records at
most sixteen advanced submissions from that invocation, then retains the
return observation to retire the pair. Reentry before a matched return
abandons the pair instead of treating a reused stack address as the same
invocation. Owner rebinds likewise retire the pair.

Every advanced observation revalidates the current creator relation,
thread, parent/child frame, both return addresses, descriptor/rectangle-set/
source-array arguments and current component. The color destination must
match current manager `+0xb58`, RCX and R12; saved normal destination must
match current manager `+0xb60`. R8 must equal the current primary wrapper.
The selected rectangle must be a sixteen-byte entry in the current list C,
with count bounded to 64. The observed destination and source rectangles
must exactly match the source-defined 32-bit subtraction formula. Overflow
is preserved as unsigned source arithmetic, not silently corrected.

The record includes compact file IDs, current dimensions/readiness of all
source wrappers, destination fields, selected rectangle, common origin and
relative source rectangle. Pending dimensions remain explicitly pending;
the probe does not read pixels, call getters, alter target state or claim a
completed composition. UI snapshots retain the bounded material fields.
Eight owners, 128 output records and 210 seconds from first binding bound
the observer; late snapshots end after their existing bounded followup.

Fifteen host checks cover pairing, owner/thread/frame rejection, each return
address, current destinations/source relation, selected-list membership,
rectangle formula, unsigned arithmetic, pending inputs, reentry, record
limits and cleanup before release. Two configuration preflights, both GDB
Python blocks and five shell-script syntax checks passed. No runtime result
is claimed by this preparation note. The run's input manifest is
`5a174c6f14f743f6703486984bb96dee856e45f280120f646c60e3289a6f4e31`.
Before launch, the reserved exclusive window also permits read-only
comparison of the six known working-archive assets to their frozen hashes.
