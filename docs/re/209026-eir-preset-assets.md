# Build 209026: registered Eir composite preset and asset references

Source-only follow-up to [species 3172's alternate appearance](209026-eir-alternate-appearance.md),
2026-11-05. Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Only that executable and previously extracted content were read. No archive,
client, runtime fixture or production code was accessed or changed.

**The exact `nornfemale/preir` pair is registered by the client, and all eight
authored preset components resolve directly in that profile.** This closes the
preset-existence gap. It does not establish a complete server 377 message,
successful rendering, NPC placement or Great Hunt membership.

## Registration and exact retained records

The retained composite asset is **154681**, `.comp` version **19**, SHA-256
`94796c36a36c506ed4d616d798d8c90ae44dfed49372b21ff66a37f3299ab7f5`.
The [appearance validation note](209026-appearance-validation.md) establishes
the startup load and its frozen file reference. Native loader **`0x1412dba00`**
iterates profile records of stride `0xe0`. For each profile it registers:

- Components: count/pointer at `+0x68/+0x6c`, stride `0x67`, through
  `0x1412db4b0`, using profile `+8` and component `+0` as the key.
- Presets: count/pointer at `+0xc8/+0xcc`, stride `0x39`, through
  **`0x1412db760`**, using profile `+8` and preset `+0` as the key.

`0x1412db760` calls `0x1412d9130`, which stores the two tokens at hash-node
`+0/+8`; it then stores the authored preset pointer at node `+0x40`.
This is the same exact pair and pointer checked/returned by `0x1412db100`.
Component lookup `0x1412d9da0` uses the equivalent pair and supports profile
fallback through profile `+0xc0`. No fallback is needed for these eight entries.

All offsets below are **composite payload offsets**; add **28** for absolute
offsets in the retained `appearance-comp-154681.pf` file.

| Record | Exact value |
| --- | --- |
| Profile | `nornfemale`, token `0x0000ac0b4a6749ee`, offset **11944** |
| Preset | `preir`, token `0x0000000001249650`, offset **6574195** |
| Matching preset count | Exactly **1** among this profile's **330** presets |
| Preset component array | Offset **10427144**, count **8**, stride **28** |
| Profile fallback token | Zero |

`0x1412d5bb0` consumes this preset's component array and packed colors. It
sets the appropriate component slots from descriptor byte `+8`, and reads
the authored preset color fields at `+0x18/+0x1d/+0x22/+0x27/+0x2c`.
The component entries also contain their own four five-byte color records.
Those bytes are retained in the evidence; no replacement colors were invented.
The absent kind-4 component reaches `0x1412da060` with index zero, which
returns a zero token rather than selecting the first profile entry.

## Component file references

The [source-input consumer](209026-composite-component-source-inputs.md)
establishes seven filename fields in this order: **`+0x4a`, `+0x42`, `+0x22`,
`+0x2a`, `+0x32`, `+0x3a`, `+0x52`**. They are primary, five mask inputs,
and normal input, with requested native formats **26, 27, 27, 27, 27, 27, 30**.
These are requested formats, not independently read asset-header formats.
All references below use the strict encoded archive-ID representation with
zero suffix. A dash is an authored null filename.

| Component token | Kind / rectset | Descriptor offset | Primary | Masks in the five-field order above | Normal |
| --- | --- | --- | --- | --- | --- |
| `cultmadic` (`0x34902da32a3`) | 9 / 2 | 6302206 | 144239 | 144237, 144231, 144233, 144235, — | 144241 |
| `cultmadib` (`0x24902da32a3`) | 8 / 2 | 6302103 | 144227 | 144225, 144219, 144221, 144223, — | 144229 |
| `cultmadig` (`0x74902da32a3`) | 10 / 2 | 6302309 | 144251 | 144249, 144243, 144245, 144247, — | 144253 |
| `cultmadil` (`0xc4902da32a3`) | 12 / 2 | 6302412 | 144263 | 144261, 144255, 144257, 144259, — | 144265 |
| `cultmadis` (`0x134902da32a3`) | 14 / 2 | 6302618 | 144275 | 144273, 144267, 144269, 144271, — | 144277 |
| `sigridfa` (`0x988991d33`) | 5 / 2 | 6558470 | 148761 | 148759, —, —, —, — | 148763 |
| `defaultea` (`0x12d195098a4`) | 7 / 3 | 6555792 | — | —, —, —, —, — | — |
| `eirha` (`0x144925`) | 6 / 3 | 6561457 | 148849 | 148847, 148843, 148845, —, — | 148851 |

Additional encoded filenames occur in descriptor `+0x12/+0x1a`. Their
individual roles were not established by this slice; they must not yet be
labelled particular geometry variants merely from their numeric IDs.

| Component | `+0x12` | `+0x1a` |
| --- | --- | --- |
| `cultmadic` | 37854 | — |
| `cultmadib` | 37852 | 37853 |
| `cultmadig` | 37856 | 37855 |
| `cultmadil` | 37857 | — |
| `cultmadis` | 37858 | — |
| `sigridfa` | 38187 | — |
| `defaultea` | 38177 | — |
| `eirha` | 38241 | — |

All eight descriptors have null `+0x0a`. No referenced component asset was
extracted or decoded in this pass; the table proves references, not asset
availability or rendered output.

## Profile base model and packet boundary

The profile's default model reference at **`+0x8c`** decodes to **30126**.
`0x1412daa70(profile, selector)` first tries a nonzero selector in the
profile's `+0xd4/+0xd8` override array, then uses `+0x8c`, then profile
fallback if necessary. This profile has **69** override entries. The preset
consumer passes composite `+0xa18` as that selector; the default is therefore
conditional on the selector not matching an override. Its result reaches
`0x1412d32b0` and then model loader `0x140cf7000`. The earlier
[bounded archive presence check](209026-creator-model-archive-presence.md)
already verified file 30126 independently; it was not reread here.

For server **377**, only these appearance words are established:

- Optional field 3, Qword 1: profile **`0x0000ac0b4a6749ee`**.
- Optional field 3, Qword 2: preset **`0x0000000001249650`**.
- Install that composite record **once, before 170 finalization**; the
  setter asserts both `!IsFinalized()` and `!m_compositeData`.

**Qword 0 remains unresolved.** The zero auxiliary field at species-entry
`+8` has not been linked to this word. It does not justify a source-backed
Qword-0 default. The character index, name, metadata flags and second optional
record also need explicit valid inputs. Consequently this result is an
authentic preset/asset selector contract, not a complete safe 377 fixture.
The next packet-specific dependency is a typed consumer or captured provenance
for compositeData `+0`, together with the intended existing metadata. No
placement or story-membership inference follows from this appearance proof.

## Evidence

Private reader: `~/re/gw2/notes/eir-preset-retained-components-20261105.py`.
It disables the content helper's subprocess/archive fallback before use,
hash-checks retained data, checks relocation slots, and requires unique
profile/preset/component matches.

Complete private bundle:
`~/re/gw2/notes/209026-eir-preset-assets-20261105.json`, SHA-256
`43ac6c824a30a13ecc0e29bcfa8214a43c3fe738bfa30604eb83eea5b08df4be`.
All 12 captures exited 0 with owned-process cleanup. One intermediate record
has rendered-terminal truncation; the final complete read below supersedes it.

| Evidence | Scope |
| --- | --- |
| `ev_48dbc613737ca5f167c53ab05a45b1671f12597669f809780ac106eee382e6f3` | Retained loader, `.comp` v19, exact component/preset registration loops; in `209026-appearance-comp-counts-1.json` |
| `ev_cc4e709e445e73a335b3c67dae3c690e2ed519e1167549f0d3a8094d222eddec` | Preset registration and component lookup/fallback |
| `ev_cc2b4da799fb3e37fd14f8c7e3d9fd514d9b71b2d3d69d37ec887ed71b4575c4` | Key-node constructor stores both exact tokens |
| `ev_c60df009de9b6189fed6e4f1d95e164eb77b5f663bf46392923ef40a72b41186` | Preset components/colors consumer; prior Eir bundle |
| `ev_eea033000c8ad9934282befbfdb9f4c4829bec426cadb85d4dd24e9eb3da2df6` | Zero-index kind-4 lookup and component-slot assignment |
| `ev_196dd945c64367304e16b23706a31909603a4d22552f259396c00fa799d94aab` / `ev_d0c8b41e3ecf6006174364f4d0641b6421e11b940bf5c47ddf8ce37edbf79aea` | Preset consumer continuation and model-load call |
| `ev_0651166a4dd1d2b2c927689beea59d13defad9efbd48d31dd00aae092c12f8d0` / `ev_50f53cf68f684e8a0e04777a67cf9fc19f6a0f18217c2a79b1c155c3a3e2670b` | Profile base/override lookup and forwarding to model loader |
| `ev_d198888a59d405e715b8f400e4c5183f5c64c0923511ddc55b8855e00d449d34` | Complete hash-checked preset, all eight resolved descriptors, exact filename IDs and profile default-model reference |
