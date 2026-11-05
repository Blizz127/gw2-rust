# Build 209026: authored composite rectangles and loaded source dimensions

The traced composite path gets its requested dimensions and rectangle list
from composite data. It forms source coordinates relative to the selected
rectangles' common origin; it does not first rescale that geometry to the
loaded source image. The later image-composition path clamps against actual
texture dimensions. This separates the origin of run 132's request from the
still-unresolved reason its source image was smaller than the requested
rectangle extent.

This is a bounded static REA trace of frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No client was launched or attached, no archive accessed, and no helper or
runner patch changed. Runtime observations and their lifetime limits remain
in [the model-pipeline note](209026-creation-model-pipeline.md).

## Rectangle set and half-size mode

Component helper `0x1412d8950` reads byte **`+0x66`** of its component
definition and passes it to `0x1412da170`. The latter indexes a loaded
composite-data table using **40-byte** records. Its assertion is
`blitRectSet < table->blitRectCount`; the table comes from the existing
service/context chain, not from an image dimension query in this function.
This trace does not assign the table an archive type or an asset ID.

The returned rectangle set's Dwords **`+8/+0xc`** supply the dimensions used
for source texture acquisition. When the incoming composite half-size flag
is clear, the helper requests those dimensions with flags `0x1032`. When
set, it halves both dimensions and uses `0x10b2`. The flag is the previously
traced composite `+0xb1c` bit `8`; it is passed through the component call.

Rectangle-list builder `0x1412dc050` selects one or more categories according
to the resolved descriptor's byte `+8`, component-definition flags and the
descriptor's additional selection bits. It passes each selected category to
`0x1412dbe90`. That function:

1. Reads the rectangle set's pointer at **`+0x14`** and uses adjacent byte
   entries for the category's start/end indices.
2. Reads the rectangle pointer at **`+0x20`**; each rectangle is four Dwords,
   16 bytes.
3. Appends those records to the selected list. In half-size mode it applies
   a logical right shift by one to all four coordinates before appending.

Thus the explicit half-size path scales acquisition dimensions and rectangle
coordinates together. There is no loaded texture-width/height lookup in
these complete list-selection/copy routines. Actual runtime selection bits,
rectangle-set ID and descriptor byte were not retained in run 132.

## A complete dispatch route and coordinate construction

Dispatcher `0x1412dd5e0` switches on descriptor byte `+8`. Exact table data:

| Descriptor value | Routine |
| --- | --- |
| 0, 1, 2, 3, 5, 7 | `0x1412dd1d0` |
| 4, 6 | `0x1412dcc70` |
| 8 through 14 | `0x1412dc660` |

This bounded investigation expands the complete **`0x1412dd1d0`** route.
The other two routes were identified, not claimed to have the same complete
implementation. The run-132 descriptor value was not sampled, so this route
is a source-backed candidate consistent with the numerical observations,
not a proven runtime dispatch identity.

The route receives the rectangle-set pointer and the component's acquired
source-wrapper array. Its primary source is array entry zero, ultimately
component state `C+0x10`. It constructs multi-pass descriptors from that
array and the optional source; the submission entry may compact inactive
mask passes as documented in
[the multi-source operation note](209026-grtex2d-multi-source-blit.md).

After building the selected rectangle list, the routine finds the minimum
left/top coordinates across that list, seeded with the set's requested
width/height (halved when that mode is active). For each selected rectangle
`[left, top, right, bottom]`, it passes to `0x140a71180` at
**`0x1412dd401`**:

```
destination_point = [left, top]
source_rectangle  = [left - origin_x, top - origin_y,
                     right - origin_x, bottom - origin_y]
```

The common origin is derived from the selected data rectangles. No actual
loaded-source dimension query or proportional rescale occurs between that
selection and this advanced-blit submission. The routine's later ordinary
blit submission uses the same kind of origin subtraction for its selected
rectangle list. This does not remove the later checks and clamp in
`ProcessComposite` at `0x140a747c0`.

## What the run-132 numbers support

The observed nonempty call and marker-bound request imply the same candidate
origin:

| Observation | Destination point | Source left/top | Implied origin |
| --- | --- | --- | --- |
| Nonempty early call | `[384,768]` | `[0,0]` | `[384,768]` |
| Marker-bound pending request | `[896,896]` | `[512,128]` | `[384,768]` |

The marker-bound request's complete source rectangle was
`[512,128,640,256]`, while its ready primary and masks were **512 by 128**.
The early empty call's clamped rectangle was `[512,128,512,128]`.
These values are consistent with the subtraction formula and later bounds
clamp. They do not prove which dispatch route or rectangle set produced the
request. They also do not establish continuity between the early job tag
and the later marker-bound allocation; the run-132 probe did not cover
retirement throughout that interval.

Requested pool dimensions of 1024-square and actual source dimensions of
512 by 128 describe different stages. The acquisition request does not, in
this trace, prove that the source file's loaded pixels were 1024-square or
that the loaded image was meant to be upscaled. This lane did not identify
the file/asset key, inspect its pixels or establish why the selected data
expects a larger coordinate extent. The existing runner texture-loading
patch is a separate investigation; nothing here proves it caused the
mismatch.

The missing binding is concrete: actual descriptor byte `+8`, rectangle-set
ID, half-size flag and selected rectangles at the same owned submission,
linked to the loaded source identity. No broad field scan, automatic
coordinate repair, readiness override or new runtime follows from this
source note.

## Evidence

Complete private export:
`/home/blizz/re/gw2/notes/209026-composite-authored-rectangles-20261105.json`.
All captures exited zero, were nontruncated and verified owned reader-process
cleanup. `.pdata` fragments were followed with bounded instruction windows
where needed; jump-table bytes were decoded as data.

- `ev_9c9a3016498e8f738dd7d30f1c948cbcabb555a3c9504a17ece21930a655b4ec`:
  complete representative routine, dispatch entry and rectangle-set lookup.
- `ev_aaf09bcb0714c46d395f7a6ae271384bbe218ef6bf30799bcdfb8f690094dce7`:
  component submission arguments, complete dispatcher and list selection.
- `ev_98e7450817406312a7f80cbaa4f6681ebb95baafd32de06b4d581450df73d37f`
  and complete continuation
  `ev_6f3d248c82932b432b14bd5ab7f8d969d68ff185dd4acb6d8b63c1fcf4879838`:
  category byte ranges, rectangle copying/halving and acquisition dimensions.
- `ev_96ff378bee9e3de6de082fe4f7d67c1289c516620035036f56c9747b01791b36`:
  exact descriptor dispatch table and explicitly inferred numerical origin.
