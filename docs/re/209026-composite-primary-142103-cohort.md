# Build 209026: complete source cohort for composite primary 142103

Source-only follow-up to run 139's reported `Composite mask smaller than
source RGB texture` assertion, 2026-11-05. This pass reads only the retained
composite asset **154681**, version **19**, SHA-256
`94796c36a36c506ed4d616d798d8c90ae44dfed49372b21ff66a37f3299ab7f5`.
It does not read the archive, launch or attach to the client, inspect runtime
state, or change a manifest.

**Every retained descriptor whose primary is 142103 uses mask 142101.**
There are four matching descriptors: two variants in `nornmale` and the same
two variants in `dwarfmale`. Their normal reference depends on the variant.
This supplies the authored dependency cohort; it does not identify the
descriptor, compacted pass, or mask that raised the runtime assertion.

## Exact descriptors and seven source fields

The [component consumer](209026-composite-component-source-inputs.md) reads
seven filename fields in order **`+0x4a`, `+0x42`, `+0x22`, `+0x2a`,
`+0x32`, `+0x3a`, `+0x52`**. These are primary, five mask inputs, and normal
input. Requested formats are **26, 27, 27, 27, 27, 27, 30**; they are not
independently measured asset-header formats.

Offsets below are relative to the `.comp` payload. Add **28** to obtain
absolute offsets in `appearance-comp-154681.pf`.

| Profile / token | Profile offset | Component / token | Descriptor offset | Kind / rectset |
| --- | ---: | --- | ---: | --- |
| `nornmale` / `0x0000002b02d749ee` | 11048 | `skinbc` / `0x0000000006272573` | 4925346 | 0 / 2 |
| `nornmale` / `0x0000002b02d749ee` | 11048 | `skinbc1` / `0x1000000006272573` | 4925758 | 0 / 2 |
| `dwarfmale` / `0x000005605a6906e4` | 13288 | `skinbc` / `0x0000000006272573` | 9011826 | 0 / 2 |
| `dwarfmale` / `0x000005605a6906e4` | 13288 | `skinbc1` / `0x1000000006272573` | 9012238 | 0 / 2 |

The token display suffix `1` represents the token's high nibble; the exact
64-bit token is the lookup identity.

| Channel | Descriptor field | `skinbc`, both profiles | `skinbc1`, both profiles |
| ---: | --- | ---: | ---: |
| 0, primary | `+0x4a` | **142103** | **142103** |
| 1, first mask | `+0x42` | **142101** | **142101** |
| 2, second mask | `+0x22` | null | null |
| 3, third mask | `+0x2a` | null | null |
| 4, fourth mask | `+0x32` | null | null |
| 5, fifth mask | `+0x3a` | null | null |
| 6, normal | `+0x52` | **142105** | **142125** |

Every non-null reference decodes directly from a relocation-checked compact
filename and has suffix byte zero. The retained encoded-string offsets are
11439604 for 142103, 11439598 for 142101, 11439610 for 142105, and 11439688
for 142125. Null channels are authored null pointers, not missing extractions.
No neighboring-ID rule was used.

## Lookup and manifest boundary

The bounded traversal examined all **29** profiles and **82,785** component
records in this retained file and found exactly the four rows above. Each
matching component token occurs once within its matching profile. Both
profiles have fallback token zero. Following the existing profile fallback
contract for these tokens from every retained profile yields only the four
direct resolutions above, with zero fallback hops. This is a uniqueness claim
for these matching keys, not for all component keys in the file.

The registration and fallback layout is the same one established in the
[Eir preset source note](209026-eir-preset-assets.md): profile stride `0xe0`,
component count/pointer `+0x68/+0x6c`, descriptor stride `0x67`, and profile
fallback token `+0xc0`.

The reported run 139 load of remapped primary **142103 → 142104**, followed
by a mask-size assertion, motivates checking the exact companion **142101**
and the selected variant's normal input. The source cohort alone does not
prove the active profile or variant, a particular failing mask, current
loaded dimensions, or which file metadata/cache path was used for that mask.
Its remapped target must come from authoritative type-0 metadata under the
[remapped-file contract](209026-texture-remapped-file-selection.md), not an
inferred `ID + 1`. The two normal references are separate authored choices;
there is no source basis here to substitute one for the other.

The already-retained [multi-source consumer](209026-grtex2d-multi-source-blit.md)
checks each non-null mask's dimensions against the primary at
`0x140a74a21` / `0x140a74a2c`. A smaller mask selects diagnostic/assertion
code before rectangle clamping. It is a distinct validation from the later
empty-rectangle producer-skip path. Asset references and requested dimensions
alone do not establish that these runtime checks pass.

## All 17 overlay originals: authored dependency closure

A second pass uses each of the existing 17 overlay originals as a possible
primary at descriptor `+0x4a`, across every retained profile and variant.
It finds **31** descriptors whose primary is one of four IDs. The other
13 supplied originals have no primary-field match in this file.

| Matched primary | Matching descriptors | Union of all seven source fields |
| ---: | ---: | --- |
| 139381 | 5 | 139375, 139377, 139379, 139381, 139383 |
| 140123 | 11 | 140115, 140117, 140119, 140121, 140123, 140125 |
| 140521 | 11 | 140515, 140517, 140519, 140521, 140523 |
| 142103 | 4 | 142101, 142103, 142105, 142125 |

The complete union contains **20** original IDs. Subtracting the supplied
17 gives exactly this canonical sorted missing-ID list:

```json
[142101, 142105, 142125]
```

There are **no additional missing companions** in the seven authored fields
for these matching descriptors. All matched profile/token pairs occur once
within their profile and resolve directly with zero fallback hops. This is
closure only over the specified primary set and these seven fields. It does
not cover other creator components, geometry references, separately chosen
assets, later appearances, or runtime-selected variants.

The full private report preserves all 31 profile names/tokens/offsets,
component names/tokens, descriptor offsets, kind/rectset values, ordered
filename fields and their encoded-string offsets, plus the exact input set,
union, and missing IDs:
`~/re/gw2/notes/composite-remap17-cohorts-20261105.json` (**59,731 bytes**),
SHA-256 `bb125d96bdd53b74d870e3716cf34aea79e62e140c6925d48cfc5aa47145193f`.
This is an original-ID dependency report; it deliberately supplies no
unverified remapped targets.

## Evidence

Final complete capture:
`ev_d67323e5a1ffc1149f4df8468ca21a02f38f8fe03992953ad239815a954d019d`.
It exited 0 without truncation, with owned-process-group cleanup verified.
The capture also reports unrelated process-environment permission failures;
no claim about those other processes is made.

Private reader:
`~/re/gw2/notes/composite-primary-142103-cohort-20261105.py`, SHA-256
`741e9ff16a5d135028f9951e4ea1d1c6a651a29dd6d5ebbd382fef9bab7d5b17`.
It checks the full retained-file hash, PF/chunk/version/bounds, relocation
slots, bounded zero tail padding, compact filename encoding, profile key
uniqueness, and fallback cycles. It has no archive/subprocess fallback.

Complete final evidence:
`~/re/gw2/notes/209026-composite-primary-142103-cohort-20261105.json`, SHA-256
`e3f36800ea187d79e2d2fcf4f0dd271f336613ced7e64b86d65d0371f041d948`.
The reader initially rejected the retained six-byte zero tail padding and
then unrelated duplicate component keys; the final reader accepts only the
bounded zero padding and reports matching-key multiplicity explicitly.
Those initial failed reads do not supply the result above.

The all-17 extension is
`~/re/gw2/notes/composite-remap17-cohorts-20261105.py`, SHA-256
`89c2e5c5a755f9f57490044475c1c10455c978ff7722ccbf03fa80542e3f33c5`.
It retains the same input/hash/offset checks and writes only a new private
report, refusing to overwrite it. Capture
`ev_bb3a2b2ce91822465e266dc5b41ff5133c78d6c65edc3152d6de09f2b8e30443`
exited 0 without truncation and with owned cleanup verified. Complete evidence:
`~/re/gw2/notes/209026-composite-remap17-cohorts-20261105-evidence.json`,
SHA-256 `1c37ab609d692ec4251b8586ca1cef3fe4567d0bfbaeb42363913bf3429e8acf`.
