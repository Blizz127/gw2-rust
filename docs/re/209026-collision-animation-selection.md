# Build 209026: collision prop animation selection

Static REA and read-only Queensdale archive audit, 2026-11-01. Frozen target
identity: [client-build.md](../client-build.md). No client run or physics
simulation occurred.

The typed `ModelPlacement` previously omitted `havk.propModels.sequence`.
It now preserves that token, and `prop_animation_209026` selects the
matching animation. The `geometry_meshes` convenience method still returns
the first animation, explicitly documented as a different operation.

## Client lookup

`0x1410cc590` builds animation objects from the archive's animations array,
preserving each sequence token and collision/blocker references. It builds
each geometry's ordered animation list from that geometry's index array.

The prop constructor `0x1410be5e0` receives the stored sequence token and
saves it at prop object +0x28/+0x30. A zero input is replaced by
`0x2cdf07c8ba` (the packed token `zeropose`). The separate
`0x2b9e7718cf` token decodes to `offngone`; neither special token is an
instruction to use the first geometry animation here.

On initial body construction, `0x1410bf740` passes +0x30 to shape lookup
`0x1410cbee0`. That helper scans the geometry's animation list in order,
comparing each animation's token at +8. It uses the **first exact token
match**. Only a zero lookup key falls back to the first animation after
no exact match; prop construction has already normalized zero to
`zeropose`, so that generic fallback is not the normal prop path.

An unknown nonzero token returns no shape. The caller has separate shape
fallback handling, which is not reproduced by an animation lookup method.
Missing sequence data should therefore not silently choose an arbitrary
animation in the Rust reader.

## Queensdale audit and fix

Archive file 192711 contains 8,301 prop placements. Comparing each raw
sequence token to the ordered animation list of its referenced geometry:

| Check | Result |
| --- | --- |
| Stored zero sequence tokens | 0 |
| Props with a matching animation | **8,301** |
| Props whose match is not the first animation | **819** |
| Missing matches | 0 |
| Props with duplicate matching tokens | 0 |
| Distinct stored sequence tokens | 13 |
| Props selecting a different collision mesh list than the first animation | **212** |

`ModelPlacement.sequence` is optional because zone placements lack this
field. The new prop-animation lookup normalizes stored zero to `zeropose`,
then finds the first exact match. It returns None for missing sequence,
geometry or token and does not implement a fallback shape.

Six focused map-reader tests and the library/inspector build pass. The
selection regression checks a non-first `zeropose`, stored zero handling,
first-match behavior with duplicate tokens, an explicit other sequence,
missing token, missing sequence and invalid geometry. A REA smoke compares
every typed token and lookup against an independent raw `PfValue` lookup:
all 8,301 agree. No runtime simulation result is claimed.

Private audit helper: `~/re/gw2/notes/check-map-animation-selection.rs`.
The audit reads raw `PfValue` tokens independently of the typed field and
confirms that selected plus missing counts equal all placements.

## Evidence

Private bundle: `~/re/gw2/notes/209026-collision-animation-selection-1.json`.

| REA record | Observation |
| --- | --- |
| `ev_84fb317a61b9cf4d1bddcb73799e5e7f9074721374767bc72ad1a7faba0ee99b` | Animation/geometry list construction and matching-token lookup helper. |
| `ev_4dcba000b8e4b6877e37b3d4e6e2998a8467d1e9c69fc516a6342431366eed9d` | Prop constructor normalizes zero and stores selected sequence. |
| `ev_6aae461ec44e31f2b6aabc08c8e57ddb5d661bc6ea959036f6abef18dec4cb6e` | Shape lookup matches sequence, with first-entry fallback only for zero. |
| `ev_1c0da3a018c5dd799e942196dffe42cd69bb9a523f26d74fdf493790776ea463` | Initial body creation uses prop +0x30 and separate missing-shape fallback. |
| `ev_5cf5e3e8d0304c8ad913616053826279a50fd0936af24185a72ca62984241a19` | All 8,301 sequence selections, including 819 non-first matches. |

Rust smoke bundle: `~/re/gw2/notes/209026-collision-animation-rust-1.json`,
record `ev_d9d405b09433ef3fed100caa9d3ff8ad2ce075340646d1ce04855329e3c8d2b6`.
Private helper: `~/re/gw2/notes/check-map-animation-selection-rust.rs`.
