# Build 209026: creator graphics readiness and material validity

The next bounded discriminator after run 135 is the current model's material
state, not another general readiness gate. The source allows a material
initializer to record failure while its first readiness call returns one;
the enclosing graphics object can then cache readiness. This establishes a
reason to inspect the material fields. It does **not** establish that such a
failure occurred in run 135 or caused the incomplete appearance.

The root's visual review of the final run-135 appearance image reported head,
hair, hands, belt/cloth edges, and boots, with most of the body absent. Its
SHA-256 is
`1ab91b81497bde52844a8e4c0d68495e7cc3530e2898376468a088489dc54b1c`.
This supersedes treating the final image as wholly blank. The earlier image
remained blank. Partial visible geometry does not distinguish texture alpha,
material validity, mesh selection, masking, or other rendering defects.

All new binary queries read only the frozen executable, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
There was no game launch, working-archive access, target call, or mutation.

## Compact lifecycle boundary

The [post-finalization note](209026-creator-post-finalization-gates.md)
establishes the exact consumer and model gates. Run 135 subsequently observed
the current appearance model returning one from native `+0x540(0)` at
`0x140a5d301`, then setup at `0x140a5d46a` with consumer `+0x43c = 1` and
lifecycle `+0x4dc = 1`. Its current graphics object had type nine, twelve
entries, and cached readiness. The retained runtime evidence is
`ev_a37c206bc2f7076dfcfd4d42b05979156a6d0de4cb5ab02afba0406c4035aa5b`.
The shared consumer address tag is an address label, not allocation continuity.

Outer update `0x140a5bfd0` calls the setup function with lifecycle one.
At `0x140a5c025`, EAX is that function's result and RSI is the current
consumer. A nonzero result stores lifecycle two at `0x140a5c029`, followed
by `0x140a5c033`. Null-model and already-initialized cases can also return
one, so lifecycle two alone would not prove fresh initialization or a draw.
The final partial image makes a broad draw-submission search a lower priority;
no final renderer submission was established by this bounded source slice.

## Exact current graphics-to-material relationship

Use the already verified current creator owner → model relationship first.
Let `G = Qword[model+0x200]`. The graphics predicate `0x140a805d0`
validates `Dword[G+0x14] == 9`. Its noncached path uses:

| Field | Source meaning within this path |
| --- | --- |
| `Dword[G+0x38]` | Bit zero accepts cached readiness; bit one returns false |
| `Qword[G+0x58]` | Entry-array base |
| `Dword[G+0x64]` | Entry count |
| `0xe8` | Entry stride |
| `Qword[entry+0xd0]` | Argument `P` passed to material predicate `0x140aa82d0` |

The array instructions are `0x140a80655..0x140a80687`; successful traversal
caches `G+0x38` bit zero at `0x140a8069c`. A separate concrete material
vtable or complete C++ class identity was not established. A passive reader
must obtain `P` through this current type-nine array, not classify an
unrelated pointer from its apparent fields.

Immediate material initializer `0x140aa84c0` contains `GrMat.cpp` diagnostics
for texture-count mismatches, missing textures, and failed material loading.
This provides the material interpretation of this exact path; individual
member names below remain unrecovered.

## Two material fields are necessary

`0x140aa82d0` first tests **Dword `P+0x50`**. Zero returns zero before any
texture-list access. With a nonzero value it reads `T = Qword[P+0x10]`:

| Field | Source behavior |
| --- | --- |
| `Dword[T+0x1c]` | Texture-wrapper count |
| `Qword[T+0x20]` | Array of wrapper pointers, stride eight |
| `Dword[T+0x28]` | Cached success of the texture readiness loop |
| `Dword[P+0x30]` | Nonzero accepts completed initialization after the texture gate |

The texture loop takes each wrapper's `+0x10` object and invokes its
virtual `+0x48`. If all return nonzero, it stores `T+0x28 = 1`.
A passive observer must not call these getters.

When `P+0x30` is zero, the predicate follows `Qword[P+0x18]`, then its
`+0x50` pointer and that object's `+0x38` resource. A present resource with
nonzero Dword `+0x50` defers initialization. Otherwise it calls
`0x140aa84c0` and returns **Dword `P+0x30`** at `0x140aa8380`.

The initializer's failure paths reach `0x140aa89bc`, which clears
**Dword `P+0x50`**. Its common continuation nevertheless stores
**`P+0x30 = 1`** at `0x140aa89c9`. Thus that first predicate invocation
can return one after failure; the graphics traversal can cache success.
A subsequent direct material predicate call would reject the cleared
`P+0x50`, but cached graphics readiness does not necessarily repeat it.
This is source control flow, not an observed run-135 failure.

## Minimal passive snapshot contract

Revalidate the active creator owner/model and current `model+0x200`; require
type nine before reading the entry array. Bound count to 32 for the planned
diagnostic and report excess count as explicit rejected/truncated coverage.
Check pointer arithmetic, readable ranges, and each pointer separately.
For each admitted entry record its ordinal, the two raw Dwords `P+0x50`
and `P+0x30`, and optionally `T`'s count/cache pair. Use distinct statuses
for null, read failure, unrecognized type, and budget rejection; never turn
an unreadable field into zero or a false material result.

The smallest useful output does not need texture contents, material strings,
file IDs, a texture-array walk, or deeper resource fields. `P+0x50 == 0`
would establish rejection by the native material predicate at that instant,
not the reason or time it became zero. Both fields being nonzero would rule
out that specific current validity failure but would not prove correct
texels, alpha, mesh membership, or final screen output.

## Temporary scratch release is expected

The source-backed scratch binding remains model virtual `+0x8a8`, concrete
`0x140cfb1c0`: it matches the key in each 24-byte model blit entry and stores
the supplied scratch wrapper at entry `+0x10` (`0x140cfb309`). The array is
model `+0x1c0`, count `+0x1cc`; entry `+8` is the alternative source used by
the finalizer when `+0x10` is null.

Finalizer `0x140cf1a80` explicitly releases and clears every temporary
entry `+0x10` at `0x140cf1b51..0x140cf1b65`, before setting model bit 35.
Consequently a null alternate binding after finalization is expected and
must not be labeled a missing material. The complete connection between
these blit entries and each graphics material's final texture slots remains
unresolved. This slice establishes neither a particular slot as body alpha
nor a hidden-mesh mask. The independent composite-output lane examines
texel content; the material fields above supply a separate discriminator.

## Evidence and boundaries

Private complete source bundle:
`/home/blizz/re/gw2/notes/209026-creator-graphics-material-binding-20261105.json`,
SHA-256 `94c11d2a81457b098b1db2ec7235c49c937071a6a25e99fc0327004162377ce7`.
Its ten records include some intentionally partial callee windows; no
semantics are inferred from untraced continuations. Captures exited zero
without output truncation and verified owned-process cleanup. Some captures
also reported unrelated unreadable process-environment entries; those were
not investigated or treated as owned processes.

Independent material bundle:
`/home/blizz/re/gw2/notes/209026-grmat-readiness-followup-20261105.json`,
SHA-256 `cbe9c111c3162b9c130157abc2cfd132fc219b99621b4656d5a0ace0618411e9`.

| Evidence | Finding |
| --- | --- |
| `ev_c8d9cda52751bec31088e8baaffba3819a13c675b3cce256e4a3b54a5c4eda4e` | Retained exact outer lifecycle transition |
| `ev_87441b607078a521beb3865b2a505db16846d9b68d93db3dd16b9f0d9580aaff` | Exact graphics array traversal and cached readiness |
| `ev_a1d574b5b510e30abc87fff4d5f8d7eb64c7b6af5a73252cf7e18f5bb09dd8d0` | Complete material predicate and field order |
| `ev_5def71787e3b9d963fb93889f632f1b91d668b09204bb76d425ef54065106b3d` | Initializer checks and failure branches |
| `ev_13118f63d5d0f0922c14c882ec953b36da3d1855d037a65b9635799290052e15` | Failure clear and common initialization-complete store |
| `ev_66c013ed0ef826bb0484c0e16ba0a3ab8b1630744afd9cdd709caf75e79a397d` | GrMat.cpp and texture-count/missing-texture diagnostics |
| `ev_57140f29209c356f8fb2bd35fb0ee0dd4c1599f509f6c410370bd630a67413ee` | Failed material-loading diagnostics |
| `ev_647f8ab8559bb0ba55751e28889acd22e9727dad485d3508ae42e2d03ad55f37` | Complete exact scratch-wrapper binding method |
| `ev_6a8a5f39963052f3ccba7eda098528eb38a6e259c948059ba32043955422bbe6` | Finalizer clears alternate scratch fields before bit 35 |

