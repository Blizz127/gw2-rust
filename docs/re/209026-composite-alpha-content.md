# Build 209026: composite content, alpha, and partial preview rendering

An empty-clamped composite operation skips pixel work. Completing that
operation's bookkeeping does not synthesize the missing image content.
The inspected nonempty composer has a separate compressed alpha plane and
can copy primary-source alpha directly into the destination. These facts
make texture content a concrete candidate for a partially rendered model,
but do not prove that the missing body is transparent or rule out material
or geometry selection.

This is a bounded static REA investigation of frozen client SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
It follows the known CPU composite/allocation and component-dispatch paths,
without launching or attaching a client, accessing an archive, changing a
probe, or writing target state. Root separately reported run 135's final
appearance image as severely incomplete, with retained fragments rather
than a blank preview. Its supplied image SHA-256 is
`1ab91b81497bde52844a8e4c0d68495e7cc3530e2898376468a088489dc54b1c`.
This source lane did not inspect that image; visual provenance and the
successful native readiness/setup observations belong to
[the pipeline audit](209026-creation-model-pipeline.md).

## What an empty operation leaves unresolved

The complete `ProcessComposite` path at `140a747c0` clamps the source
rectangle against actual source/destination dimensions. The empty result
at `140a74ce5` selects the diagnostic/return path and bypasses producer
`140b3d3b0`, callback `140b3d4e0`, and pixel work `140c20d80`. That path does
not stretch a 512-by-128 source into a 1024-square acquisition request, fill
the originally requested rectangle, or reconstruct omitted texels.

The guarded runner completion changes only unsubmitted-job bookkeeping.
It permits native cleanup and subsequent work, whose readiness can later
succeed. It does not establish the contents of an untouched region. The
source/dimension discrepancy remains the separate relationship described
in [the authored-rectangle note](209026-composite-authored-rectangles.md).

## The inspected format-26 composer includes an alpha plane

`140c20d80` prepares block-aligned source/destination coordinates through
`140c20f40` and invokes `140c1f7b0 → 140c1ecf0` for block rows. The preparation
uses raw format **26** for primary/destination row sizes and **27** for
masks. Its coordinate-to-byte addressing divides X/Y by four, advances
primary/destination block columns by **16 bytes**, and mask columns by
**eight bytes**. The raw format-classification entries are `0xb1` for 26
and `0xa1` for 27; this note does not assign every classification bit or
claim a public format-enum name.

The format-26 block is handled as two eight-byte portions:

- For pass mode **`0xc8`**, `140c1ee1c..140c1ee20` copies the first Qword
  from the primary block to the destination block.
- The single-pass color-copy path at `140c1f077` copies the primary Qword at
  **block +8** to destination **block +8**. The color-transform/multi-pass
  paths likewise write that second portion.
- A pass-mode **`0x82`** path invokes `140c1e220`, which reads the first
  eight bytes of source and destination, expands two byte endpoints plus
  sixteen three-bit indices, combines the expanded values, and writes a
  recompressed result through `140c1f9d0`.

`140c1e090` builds the eight-value endpoint palette, with the familiar
seven-step versus five-step-plus-zero/full branches. The related decoder
`140bffaf0` inserts those values into the high byte of sixteen output
Dwords, retaining the other three bytes. This establishes the inspected
compressed alpha representation directly from the client. In particular,
**an all-zero first eight bytes selects alpha zero for all sixteen samples**:
both endpoints and every three-bit index are zero. That statement is about
this image representation, not proof of the eventual shader's opacity or
alpha-test policy.

The expanded authored route `1412dd1d0` builds pass zero with mode `0xc8`
at `1412dd302`. Its nonempty blocks therefore have a source-defined path
that copies primary alpha. Actual route/pass identity and texture bytes
were not captured in runs 134/135, and other dispatcher routes were not
expanded here. Image mask passes are not automatically mesh-visibility
masks; the component-selection masks around `1412d7f20` have no established
hidden-mesh or alpha semantics in this slice.

## Allocation is not proof of transparent zero-filled scratch pixels

Fresh CPU backing creation follows
`140a740c0 → 140b3e7a0 → 140b3ed40 → 1409cd5f0`.
The caller at `140a741a6` supplies zero as `140b3e7a0`'s seventh argument;
that becomes `140b3ed40`'s eighth argument. The latter derives the memory
allocator's zero-fill Boolean from that argument, so this inspected call
passes **false**. `1409cd5f0` invokes the zero-fill helper only when that
Boolean is nonzero. `140b3e240` builds the mip-pointer table and zeroes its
unused pointer slots; it does not clear the associated pixel ranges.

There is a distinct fallback in `140a74280`: if destination backing is
absent, it allocates backing and, when the requested rectangle does not
cover the complete destination, clears each mip's byte range through
`1409b7550`. With existing backing, it skips that allocation/clear branch.
`1409b7550` is a zero-fill wrapper around `1418e9640`.

Consequently **neither a newly requested scratch texture nor an empty
operation proves its unwritten blocks are zero**. Backing can already
exist, pool storage can be reused, and earlier composite/copy work may have
written it. The frozen source does not justify filling pixels, forcing
opaque alpha, assigning every missing region to transparency, or treating
completion as an image-quality repair.

## Smallest next observation: current component authoring and source tuple

The existing call site **`1412d8c9e`**, immediately before dispatcher
`1412dd5e0`, contains all the required authoring pointers in the live
`1412d8950` frame. A probe can match `R14 = C` to the exact current manager's
component array (`manager + 0x18 + 0x48*slot`, slot 0..14), then sample the
following without retaining source objects for later dereference:

| Field | Live source at `1412d8c9e` |
| --- | --- |
| Component definition | `Qword[C]` |
| Rectangle-set ID | `Byte[definition + 0x66]` |
| Resolved descriptor | `Qword[RSP + 0x78]`, equal to outgoing `Qword[RSP + 0x20]` |
| Descriptor kind | `Byte[descriptor + 8]` |
| Rectangle set | `Qword[RSP + 0x70]`, equal to outgoing `Qword[RSP + 0x28]` |
| Authored dimensions | Dwords `rectset +8/+0xc` |
| Half-size input | Dword `RSP +0x1a0`, equal to outgoing Dword `RSP +0x48` |
| Effective requested dimensions | Two Dwords `RSP +0x68/+0x6c` |
| Acquisition flags | Dword `RSP +0x60` |
| Source-wrapper array | Outgoing `Qword[RSP +0x30] == C +0x10`; seven Qword entries |
| Optional wrapper | `Qword[C +8]`, separate from those seven channels |

The seven descriptor filename Qwords, in channel order 0..6, are at
**`+0x4a, +0x42, +0x22, +0x2a, +0x32, +0x3a, +0x52`**. These are packed
fields; do not impose an invented eight-byte alignment condition.
The helper copies that exact order into its acquisition loop. Their
compact numeric IDs can be decoded with the existing
[bounded filename contract](209026-composite-source-filename.md), without
outputting paths. At acquisition calls `1412d8b23/1412d8c01`, RCX also
contains the incoming filename directly. That earlier observation is an
alternative when pairing the exact thread/stack call and return is needed.

Reaching dispatch proves the helper's aggregate error-related flag was
clear, **not that every allocated source wrapper was ready**. For each
current wrapper, retain the concrete texture/readiness guards and record
actual dimensions, levels, format, backing presence and the current
ready predicate. A pending acquisition can still carry requested
dimensions before loading establishes different actual dimensions.
A dispatch-time filename-to-wrapper relation is therefore preferable to
later guessing from an already released load-task holder. Null or
unsupported inputs must remain explicit.

The exact dispatch-site 16-byte guard is
`e83d490000498b4e08e8749aaeff49c7`.
A finite first-sample-per-slot/current-owner probe of this tuple can establish
which actual descriptor/rectangle-set/half-size/source IDs participate in
the mismatch. It does not require adding pixel reads, changing readiness,
or tracing a global worker.

If actual selected rectangles are still needed later, the established
`1412dd1d0` route calls `140a71180` at **`1412dd401`** with RCX=destination
wrapper, R8=primary wrapper, RDX=two-Dword destination point and R9=four-Dword
source rectangle. Let A be RSP at the parent `1412d8c9e`: this route's
submission RSP is **A −0x1d0**; `[RSP+0x168]` is return `1412dd66b` and
`[RSP+0x1c8]` is return `1412d8ca3`. These follow the captured dispatcher
and child prologues. Same-thread/current-owner/frame checks and bounded
samples would be required. This identifies only that route, not the two
other dispatcher branches, and is not a request to expand the next probe.

## Conditional later texel discriminator

Only if the authoring/source tuple leaves the cause unresolved, a separate
bounded passive sample can examine a few **matched live format-26 blocks**.
Texture data `+0x40` is the mip-pointer table, whose first Qword addresses
level-zero backing. For the established four-by-four/16-byte layout and
suitably bounded divisible-by-four dimensions, a block at pixel `(x,y)` has
byte offset `16 * ((y/4)*(width/4) + x/4)` from that level-zero pointer.
Use source-derived requested/affected coordinates and a few control blocks,
not an unbounded texture dump. Record aggregate zero/nonzero alpha counts
or bounded decoded alpha summaries, not copyrighted image content.

Sample only a revalidated live owner/binding and a source-defined quiescent
point; do not dereference a wrapper after finalizer release, infer allocation
continuity from an address tag, or call a decoder in the target. Nonzero
alpha in a few blocks does not prove the whole atlas is valid, and all-zero
alpha does not prove the material samples that atlas region. A material
validity/binding or mesh-selection observation remains a distinct
alternative; the parallel graphics/material source lane owns that contract.

## Evidence

Complete private source export:
`/home/blizz/re/gw2/notes/209026-composite-alpha-content-20261105.json`.
All listed REA captures verified the pinned executable, exited zero with
nontruncated output, and verified owned reader-process cleanup. Unrelated
process-environment permission failures remain separate. `.pdata` fragments
were followed with bounded instruction windows; partial boundary lines
were not treated as complete instructions.

- `ev_2c3d722ef0a7d6c212d7b393ebb909dcd307c7a1c06ff71cb9704da5144a3b3a`,
  `ev_13e6aa2435fc95883460be22a0721aaec104349495670fabed41fe1dab2d1128`,
  `ev_9ff9dbcad2a158ee37d971c8e10282f85aec17eac8808e3d180e9a1357c2d8eb`:
  composer preparation, block addressing, fallback clearing and zero-fill wrapper.
- `ev_55ca459ed66c3dc91b3636cb88266bfa2147151a205c5460fcafdb9185533f2c`,
  `ev_f0c9ee67b07d9aa39c7117680902dd611446f43900679d277884edbefa57afec`,
  `ev_0602026a67cac7d19d3a01ae76814f600021ce784018a9da16d536a8dfc9fe47`:
  complete block composition, alpha combination/palette/decoder and mip-pointer setup.
- `ev_e6d7c9f4272655d163a88d8788427654b89b7c7f11e204fa4e25bc134a4995f5`,
  `ev_c09ec755cbcdc30d4a0571484e08b217b64f6aba2539dff442c443372cd973ed`,
  `ev_2f135f7f2a779697cb4c4c7a70ad893397e162f6156b352242f377f8a6fb2d98`:
  backing allocation arguments and image-memory allocator.
- `ev_53ea10badbd7e135b51d127138e5cf5c8acc3657f7ed4bf400aeea7626e649c5`:
  complete component helper, current authoring fields and ordered filename references.
- `ev_d8f7be6d94083d41f52afc78ffda9bf11ab5c2fe56aa75264a0aa51900431f6e`,
  `ev_fabdbd18615bda1992f5181c379dd196cb9cedbfd682d1eb62d95030ca09fb47`:
  authored pass mode and exact parent/submission frame relationship.
- `ev_35768c398e03ec41ee57278fe91e184935c876d09f5c143fbf5ea6c2cee3263e`:
  exact raw format-table values and focused code guards.
