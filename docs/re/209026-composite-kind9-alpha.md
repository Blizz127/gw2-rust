# Build 209026: kind-9 null passes and destination alpha

For the run-136 Norn slot-9 input tuple, the native submission code drops
the null mode-4 first pass and the null last pass. The three remaining
mode-`0x81` passes preserve the destination's compressed alpha plane when
the nonempty block worker runs normally. They do not independently copy
the primary source's alpha or synthesize a default alpha pass. Earlier
ordinary copies and other writes to the destination therefore matter.

This is a conditional source result, not an observation of completed
runtime passes, texture pixels or the cause of the partial Norn preview.
It follows frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`
through REA. No client, container or archive was accessed and no helper or
target state changed. The actual dispatch identity and file tuple are in
[the kind-9 rectangle note](209026-composite-kind9-rectangles.md).

## Exact compaction, with no synthetic default

`140a71180` iterates the requested 24-byte descriptors. Flag **`0x40`** is
source-named `passIsActiveButMaskless`; it permits a null mask only for
original pass zero. At `140a71410`, a null mask without that flag branches
directly to the next descriptor, without incrementing the active count.
Non-null masks are retained, and active flags, masks and 20-byte color
records are copied to compacted arrays. Bit `0x80` requires the color record.

Applying that exact rule to the five requested kind-9 descriptors gives:

| Original pass | Source channel / slot-9 ID | Requested mode | Source-derived result |
| --- | --- | --- | --- |
| 0 | Channel 1, null | `0x04` | Omitted; `0x04` does not contain `0x40` |
| 1 | Channel 2, 140515 | `0x81` | Compacted pass 0, original color record 0 |
| 2 | Channel 3, 140517 | `0x81` | Compacted pass 1, original color record 1 |
| 3 | Channel 4, 140519 | `0x81` | Compacted pass 2, original color record 2 |
| 4 | Channel 5, null | `0x81` | Omitted, including its color record |

The primary image remains channel 0, file **140521**, independently of this
mask array. The complete submission routine inserts no replacement pass;
if the compacted count is zero, it returns without allocating an operation.
For this tuple the expected active count is **three**, but run 136 observed
the component inputs while pending, not the resulting operation count.

The existing operation constructor retains these compacted arrays.
Producer `140b3d3b0` copies each raw flag into the job at `+0xe4 + 4*i`;
callback `140b3d4e0` passes that array into `140c20d80`. Worker preparation
`140c20480` constructs color transforms from the 20-byte records and reads
flag bit `0x80`; its complete body does not rewrite the mode array.

## What the all-`0x81` worker writes

The format-26 representation has a first eight-byte compressed alpha plane
and a second eight-byte color plane, as established in
[the earlier alpha representation trace](209026-composite-alpha-content.md).
That earlier note's representative `0xc8` authored route is not the
current kind-9 route.

Complete block worker `140c1ecf0` has these distinct paths:

- **Exact first mode `0xc8`:** `140c1ee1c..140c1ee20` copies the primary
  block's first Qword to the destination's first Qword.
- **Exact active mode `0x82`:** the mask scan can select the alpha-combination
  helper `140c1e220` at `140c1f015`.
- **Only modes `0x81`:** neither alpha path is selected. Mask values choose
  which color transformations contribute; a block with no contributing
  passes can skip color output as well. None of these cases adds a default
  primary-alpha copy.

The color-output address is destination block **`+8`**, established at
`140c1edfc`. The single-effective-pass transform `140c200f0` writes only
two Dwords at its destination `+0/+4`. The general path calls encoder
`140c018d0` with both alpha/output-mode arguments zero. It skips its alpha
encoders and emits two endpoint Words plus one index Dword: eight bytes,
again starting at destination block `+8`. Helpers `140c20920` and
`140c1e770` prepare and combine stack-local color samples; they do not write
the destination alpha plane.

Consequently, **for a valid nonempty invocation with the exact compacted
all-`0x81` flags, the block worker leaves destination bytes 0–7 unchanged**.
This is not a claim that the whole operation leaves all backing unchanged:
the separate `140a74280` preparation may allocate and clear backing when it
was absent, and other operations may write the same destination. It also
does not establish whether the eventual material interprets that plane as
opacity.

## Earlier ordinary B copies are part of the causal account

The actual kind-9 route requests ordinary primary copies at `1412dc930`
before its advanced calls at `1412dcaba`. Its lists have different extra
category selections:

```
B = base category 0 plus (descriptor.mask5e & ~descriptor.mask62)
C = base category 0 plus descriptor.mask5e
```

Thus an advanced C rectangle can inherit primary alpha from an earlier
ordinary B copy covering that destination region. The advanced `0x81`
worker preserves that alpha rather than reconstructing it from its primary.
Extra categories selected by `mask5e & mask62` may be absent from B, but
base category zero remains present in both lists and rectangles from
different categories can overlap. The masks alone therefore do not prove
that any particular C pixel lacked a prior copy. Earlier components and
reused scratch contents are additional possible sources of existing data.

The ordinary `ProcessBlt` path also clamps its rectangle against actual
source/destination dimensions; it can skip an entirely out-of-bounds copy.
A requested B copy is not proof that it wrote its originally requested
extent. Neither the compaction result nor alpha preservation proves a
transparency defect, justifies adding a maskless pass, or supports forcing
opaque pixels.

## Smallest conditional discriminator

Run 137 subsequently supplied a different current tuple: channel 1 was
non-null file 140121, while channel 5 remained null. The same compaction
rule therefore predicts `[0x04, 0x81, 0x81, 0x81]`, not run 136's three
passes. Mode 4 does not select either alpha-writing branch above; the
conditional alpha-preservation result still applies to the inspected
nonempty worker. This remains a source prediction, not a measurement of
the completed operation's pass count. See the
[paired run-137 observations](209026-creation-model-pipeline.md) and the
[actual file-selection contract](209026-texture-remapped-file-selection.md).

The paired kind-9 advanced-rectangle observation remains useful as staged;
no change to frozen run 137 follows from this slice. If its actual relative
extents fit the matched ready source, the next bounded questions are:

1. Record the same invocation's raw `mask5e/mask62` and a bounded selected
   B rectangle list, then compare its destination coverage with the observed
   C rectangles. Include category-zero coverage and rectangle overlap;
   missing a category bit is not equivalent to missing a pixel region.
2. Confirm the current operation's compacted count/modes before applying
   the all-`0x81` conclusion to runtime. The source calculation predicts
   three passes; that is not a substitute for a live operation observation.
3. Only if needed, summarize a few matched source/destination format-26
   alpha blocks after a native completion observation, while the live
   operation still retains both textures and before destruction/finalizer
   release. Compare a B-covered control with a candidate C-only region,
   using the source-defined coordinate mapping. Aggregate decoded alpha
   counts/minimum/maximum suffice; no image dump or target decoder call is
   needed. Explicitly retain unreadable/pending/unsupported cases.

Such a sample can distinguish source alpha from destination content at
specific coordinates. Zero destination alpha alongside nonzero primary
alpha would establish a content difference there, not by itself the
authoring intent, responsible earlier operation or eventual shader effect.
Do not dereference cached wrappers after their lifetime or infer completion
from a job handle alone.

## Evidence

Complete private export:
`/home/blizz/re/gw2/notes/209026-composite-kind9-alpha-20261105.json`.

- `ev_43870469b51aac666b1de75659dbc90dd50e23c2f0601c7e32201e5baae2700a`:
  complete submission, maskless-flag assertion, null omission and compaction.
- `ev_a4e1e6e199e535927fae365fc41a2175b95559d2f0f3ece9df2a95ede9bb86f0`:
  complete block worker, exact alpha branches and color-output addresses.
- `ev_91c02466a414a59cd0044643f4e5e698184a252fc428367b1325662a4a79c9d6`:
  complete job producer/callback and worker argument flow.
- `ev_160a5a75dd452a6e5bc22e94b524525d3ffd4808f77281f25b7de073d351146b`:
  complete color-transform preparation without mode-array writes.
- `ev_631e7fbfbf079aa53e3112da18e1f025f10e4311e90369f3af691ee5efc650db`:
  exact eight-byte fast color output, encoder entry and complete ordinary
  blit clamp/copy paths.
- `ev_a5d52cb9ca9493ee764a4a984a31c5166132ddd7077eb74f534784cb07ae2991`:
  general color encoder's zero-mode tail and ordinary kind-9 call site.
- `ev_491013c8e4dd7837730d7e5ac7030a240e7f7bf39a57470bb1db1609cef78a59`:
  complete color-sample preparation and mask blending helpers.

All listed captures exited zero with complete raw output and verified owned
reader-process cleanup. `.pdata` fragments were followed with bounded
windows; partial instruction boundaries were not interpreted. Unrelated
process-environment permission failures remain separate limitations.
