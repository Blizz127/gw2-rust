# Build 209026 character ray mask and height sampling

Static REA analysis of executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No character runtime replay or movement implementation is claimed.

## Concrete character wrappers

The `CoChar.cpp` region has nearest-ray wrapper `0x1410a1c10`; its vtable
entry is `0x14212cc30`, slot `+0x328` of table `0x14212c908`.
`CoCharSimpleCli.cpp` has the related wrapper `0x1410a7860`.
Both convert XYZ by `[x/32,y/32,-z/32]`, preserve position tags, and call
`0x141097be0` with fixed caller mask **`0x7830`**: layers 4,5,11,12,13,14.

The first reads full filter-info from the character's body pointer `+0x100`,
collidable `+0x20`, filter `+0x2c` (or zero if absent). It uses scene `+0x70`
and no restricted-world argument. The simple-client wrapper reads filter-info
from pointer `+0x60`, field `+0x4c`, and passes pointer `+0x160` as the optional
restricted-world argument. Unlike generic scene wrappers, character queries
can therefore carry nonzero full filter-info as well as the mask.

The mask includes static props (11), terrain (12) and zone meshes (13).
It excludes fallback layer 15. The collector rechecks the original mask even
when scene prefiltering is disabled, so these wrappers cannot accept a
layer-15 fallback through that switch alone. This is a statement about these
traced ray paths, not every possible character collision operation.

## Six-ray height routine

`0x14109d380` builds six vertical segments from character position `+0x140`.
It converts that position into map units and scales six XY offsets by virtual
`+0x218`; for the observed vtable, getter `0x14109e1e0` returns field `+0x2a0`
multiplied by 32. The stored offset table at `0x1425b99b0` is a center point
and five surrounding points:

```
(0,0), (1,0), (0.30901697,0.95106), (0.30901697,-0.95106),
(-0.809,0.588), (-0.809,-0.588)
```

These decimal values are approximate labels; the exact f32 bit patterns remain
in the private evidence. Each segment increases map Z by 10,000 (collision Z
falls by 312.5). The routine calls batch helper `0x141098670` with count 6,
mask `0x7830` and the same body filter-info as the ordinary character wrapper.
The helper's ungrouped path issues nearest queries and retains one 0x58-byte
result or an empty result per segment.

Successful hit Z values are resolved from any position tags, then compared
relative to virtual `+0x260`. Getter `0x14109e8c0` returns `-32 * field[0x160]`.
That field is populated through scene helper `0x1410b7650`, which delegates
to the water-volume height query `0x1410cfb40` and converts its result back
to collision units. The water loader `0x1410cffa0` proves the source: havk
`waterSurfaceZ` at `+0x1c` and `waterVolumes` at `+0x74/+0x78`.
Thus this routine combines geometry with water context; it is not just a
nearest solid-terrain lookup.

It tracks maximum/minimum relative hit Z and averages normals. For positive
relative Z it substitutes map-space normal `[0,0,-1]`. It normalizes the mean
and reverses Z when saving it at character `+0x2c0`. If no valid height was
found, it tries the character's `+0x328` nearest-ray wrapper using a segment
extending 200 map units in the opposite Z direction. Remaining misses use
its reference height. The caller at `0x14109f9f5` further converts/clamps
these results before using them in character state.

This trace identifies a real character query mask and sampling pattern.
Water volume rules, shape margins, body filter state, step/slope acceptance
and the complete caller state machine still prevent treating it as a full
walkability specification. The offline probe remains a geometry diagnostic.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-ray-mask-1.json`.

- Source-path references: `ev_c8546fe317613ed2d2cd932ac9efbfb22cdb319a93d99a331b3df983a3b5e145`, `ev_d2fcda170178f7fd05101d63851a5219d3369a96d4fdbfd339a7ee5c247fa0cd`.
- Character wrapper and batch masks: `ev_931276ab789ad8eb0119c97e2551a21e968ca65b83abf92708bdf5cdbd72938e`.
- Six-ray aggregation and simple wrapper: `ev_94abfb9280ad33208f0735cbb98ea796c67e63132692e7d0c8b1354002e7186b`.
- Offset constants/getters: `ev_c662bdae4d28ef3a9a42faa2a6137890c174edd14a40f6e722baf3c62c9a8e26`, `ev_d27468924582d8189b96e772e6c4fdb9699969fd79cc83a3b213d076b833291a`.
- Reference height data flow: `ev_f7b72a7b7af1701be252233a9762f8913af34f7f7a4573da1eb882e1d2dcec42`, `ev_6a8fca15f1cff0f725da948cd4e50747a918baa36d26f3334a6d9a9b562e142e`.
- Water-height query and loader: `ev_fb8663f6f25784782b42cdff2d24f261f2d15c988500acc3f74998e8d0773950`, `ev_152cb6c25056c10e2704d4863fdae0a5087ddd0b2e77d128b91c7f610cead62b`.
