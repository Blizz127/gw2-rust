# Build 209026: file-derived texture dimensions and holder retirement

The load-completion path explicitly replaces a texture's provisional
dimensions with dimensions from its decoded file task. A 1024-square
acquisition followed by a 512-square texture is therefore supported by the
implementation; that size difference alone is not evidence of a defect or
of an intended rescale. The current Norn component's selected relative
rectangles still need to be compared with its actual ready source geometry.

This bounded trace read frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`
through REA. It used only the retained load-task, texture-transfer and
directly connected completion routines. No archive, client runtime or
container was accessed, and no probe or target state was changed.

## Exact writeback chain

The established decoder continuation `140a6f490` copies decoder format
`+0x44`, levels `+0x48`, and packed two-Dword dimensions `+0x38` into file
task `+0x58`, `+0x64`, and `+0x68/+0x6c`. The later optional reduction step
is separate; see [its existing analysis](209026-grtex2d-operations.md).
CPU-only textures skip this continuation's later GPU-format conversion.

The task processor `140a6db40` uses the table at `140a6df10`. State 6 maps
to `140a6de1b`; after removing the task from the pending list, it sets state
7 and calls **`140a70450` at `140a6de7a`**, passing the task's loaded format,
packed dimensions and levels. This is the exact bridge from decoded
metadata to texture data:

| Site | Operation |
| --- | --- |
| `140a70450` entry | RCX is file task, EDX format, R8 packed dimensions, R9D levels |
| `140a704a5` | Calls `140a75cb0` with destination `task+0x28` and image backing `task+0x50` |
| `140a75d0c` | Writes texture levels `+0x68` |
| `140a75d0f` | Writes texture format `+0x64` |
| **`140a75d12`** | Writes the loaded two-Dword dimensions to **texture `+0x48/+0x4c`** |
| `140a75d46` | CPU-only/blit-source branch calls `140a75de0` using that metadata |
| `140a76035` | That CPU path sets texture readiness bit `0x10000` |

The CPU branch validates/adopts the input backing or allocates/copies using
the newly assigned dimensions; it may generate additional mip levels.
The inspected transfer routine does not restore the earlier acquisition's
atlas size. `140b3f3a0`, called before adopting backing, identifies itself
as `ImgMemValidate`; its downstream validator was not expanded in this
slice. A readiness bit does not by itself imply an empty operation list or
correct pixels.

For fresh input flags `0x1032`, the previously established conversion to
`0x41036` and derived task flags `0x420/0x440` selects the original
no-reduction branch. The runner's full-texture branch patch does not upscale
the image. Cached or subsequently changed tasks require their own observed
history; this static flow is not a measurement of run 136's completed load.

## High/low holders can disappear after successful completion

The existing names are exact: texture `+0x50` is `m_fileLowRes`, `+0x58`
is `m_fileHighRes`. A file task retains its copied filename at `+8`, compact
numeric ID at `+0x30`, and key flags at `+0x10`. The transfer's rejection
assertion identifies key bit 0 as `TEXTURE_KEY_HIGH_RES`. These are task
metadata, not proof that an arbitrary later texture selected a different
high-resolution archive asset.

After the successful task-transfer path, `140a6debd` tests texture flags
**`0x2000`**. When set, the continuation at `140a6dedc..140a6df06` invokes
each nonnull high/low file task's deleting virtual and clears **both holder
pointers**. This supplies a concrete reason that an already-loaded source
can have no retained filename holder. It does not prove that a particular
runtime null was produced by this branch, but prevents interpreting null
holders alone as missing-file evidence. Run 136's pending source flags
`0x3036` included this bit; its completed holder lifetime was not sampled.

The empty-composite diagnostic prefers the high holder and falls back to
low, as [previously documented](209026-composite-source-filename.md).
That preference is not a provenance record for the current backing.
This slice does not establish the complete alternate-asset lookup or cache
key policy and does not assign the file predicate `140d90480` new semantics.

## Smallest followup and interpretation boundary

Run 136's Norn slot 9 identifies primary **140521**, masks
**140515/140517/140519**, and channel 6 **140523**. Their separately extracted
frozen headers are all 512-square. The older run-132 512-by-128 ready source
belonged to an unidentified initial-scene owner and cannot be substituted
for this tuple. See [the exact asset headers](209026-creator-texture-asset-headers.md).

The smallest next observation remains the current kind-9 submission's
selected rectangle, common origin, relative source rectangle and matched
current texture fields. The [kind-9 source route](209026-composite-kind9-rectangles.md)
subtracts the origin without loaded-dimension scaling. Relative extents
within 512-square are compatible with the smaller source even though its
atlas request was 1024-square. Extents outside a matched ready source could
explain clipping, but do not alone determine whether the authoring,
selection, cache/load history or another condition is wrong.

Only if that comparison leaves the loaded identity/dimensions in question
is a file-transfer observation useful: at `140a70450`, match
`task+0x28` to the currently owned source, then record compact task ID,
key flags, incoming dimensions/format/levels and holder relation before
retirement. A cached source may not execute a new transfer. No such extra
hook, automatic rescale, pixel replacement or readiness override follows
from this source result.

## Evidence

Complete private bundle:
`/home/blizz/re/gw2/notes/209026-texture-loaded-dimensions-writeback-20261105.json`.

- `ev_f0d2e1c1633b885ca08594b09db76964b042b1865220ffa932a1ce11669de421`:
  complete quality-flag helper and decoder continuation through return.
- `ev_e4418adf1f531ecbb4d8b63cb951709f5f43b073104c696d61a1ea43548c1395`:
  task constructor, stored file ID/key flags and decoder-completion call.
- `ev_23035a76dad31ffdf5bad5dc2df186bed51b2f19eff265162f72be2a5049eba7`:
  task processor completion block and high/low reload handling.
- `ev_aada35395dcd94b712bff8ba4aa0d70d8fbf18e25bb9ef309055ce9cf7e798d4`:
  exact task-to-texture transfer arguments and high-resolution key assertion.
- `ev_0f45fa66feb61c3897fac31d8b86e809cbe7d953a0422f66ea0fc5e5d45ecf44`:
  complete dimension/format/level writeback routine.
- `ev_8f6d27b3d26cbeb5f7d95579c4758ede7a0d5eed0e1d3062d5043b1cd4c98076`:
  CPU backing path, readiness store, and holder-clearing tail.
- `ev_1dcb5885622861c9ae3ed5d269637245d1a871bcd015ce400e1f26ac2314d805`:
  exact state table decoded as data and transfer-site guards.
- `ev_979f80815822df5c0a943859a08752ee469028db2b86dd6f0769eb321a78a0aa`:
  directly called backing validator and its source name.

Captures exited zero with nontruncated raw output and verified owned
reader-process cleanup. `.pdata` fragments were followed with bounded
windows; incidental table bytes in instruction output were not treated as
code. Unrelated unreadable process environments remain separate limitations.
