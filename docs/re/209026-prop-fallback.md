# Build 209026 static prop shape fallback

REA static analysis of executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This resolves the 62 empty mesh selections found by the
[prop transform audit](209026-collision-rotation.md). No client runtime replay.

## Empty shape selection

`0x1410cbee0` selects the animation by sequence and calls `0x1410cbd20`.
The latter returns null immediately when the selected runtime animation has
zero collision entries (`+0x10`), before considering scale. Thus an existing
sequence with an empty collision list reaches the missing-shape branch, just
as an absent sequence does. This does not mean the prop is simply omitted.

The static `havk.propModels` loader calls `0x1410b9020` at `0x1410b8912`.
That helper calls body builder `0x1410bf740` with collision filter value 11
and motion argument zero. On a null shape at `0x1410bf835`, the builder uses
float `0x3d4ccccd` (0.05) from `0x141ae16c0`, changes the filter value to 15,
and calls `0x1410b5fd0`. Values 6 or 7 use a separate larger fallback and
retain their filter value; that is not the static-map-prop branch.

## Fallback geometry: box, not sphere

Following the complete factory chain corrects the earlier provisional sphere
interpretation. `0x1410b5fd0` calls `0x141095340`, which multiplies the supplied
size by float `0x3f3504f3` (~sqrt(0.5)) at `0x141d5a564`, repeats it on XYZ,
and constructs a three-float shape-cache key via `0x141095290`.
Cache factory `0x1410b59a0` calls `0x1410961e0`, which passes those three
values and convex radius 0.05 (`0x14263a6f4`) to `0x1416c2790`.

That constructor installs vtable `0x142338138`, stores the convex radius at
`+0x20`, and the half-extents at `+0x30`. Support method `0x1416c27f0` picks
signed half-extents independently on each axis: this is a box shape. For the
static prop fallback, each half-extent is approximately 0.03535534 collision
units, with convex radius 0.05. It retains the prop's body rotation and
converted origin, but the fallback dimensions do not use the model scale.
No sphere or triangulated-box approximation has been added to ground queries.

## Initial collision layer matrix

Scene initialization `0x1410b8bb0` constructs its group filter through
`0x1416d04f0`, initially filling 32 layer rows at `+0x4c` with all ones.
It clears pairings among layers 1–31 via `0x1416d0620` with both masks
`0xfffffffe`, then symmetrically enables these mask pairs via `0x1416d05e0`:

| First mask | Second mask |
| --- | --- |
| `0x7830` (4,5,11,12,13,14) | `0x10008` (3,16) |
| `0x200` (9) | `0xc6` (1,2,6,7) |
| `0x400` (10) | `0x4` (2) |
| `0x8` (3) | `0x6` (1,2) |

Layer 0 retains its all-layer pairings. None of the enable masks includes 15,
so fallback layer 15 initially pairs only with layer 0. Static prop meshes
(layer 11) and terrain (layer 12) pair with layers 0,3,16. These numeric layer
relationships do not by themselves identify an actor or a ground-query mask.

Custom pair filter `0x1410bd3d0` applies a further predicate `0x1410bd190`,
then masks both inputs to their low five bits before calling base lookup
`0x1416d08f0`, so this custom body-pair path always uses the layer matrix.
The base function also has a shared nonzero high-group path, reachable from
other callers including the world-ray wrapper. Custom ray filters impose
additional mask/tagged-pointer checks; see [ray filter trace](209026-ray-filters.md).
Consequently, the small fallback's existence is not proof that it supports
walking or should be included by every query. Runtime filter changes and the
actual movement-query masks remain to be traced.

## Queensdale count

All 8,301 prop sequences match existing animation entries. Exactly 62 selected
entries have empty collision lists, across five sequence tokens:
`0x7160f`→2, `0x1c55be42`→5, `0x22656d82e`→46,
`0x2cdf07c8ba`→6, `0x44d2c7b4a4`→3. These selections also have zero blocker
references. This is archive coverage; scene clipping/load lifetime can still
affect which bodies are instantiated.

## Evidence

Private bundle `~/re/gw2/notes/209026-prop-fallback-1.json`.
Private audit helper `check-empty-prop-shapes.rs` executed through REA.

- Null shape and fallback branches: `ev_9314f17f72725c999853618007a3ad40b301d57f2df159b4811ad90b2ca08d95`.
- Zero-entry return, constants, call sites: `ev_8cb14ae448f9de08b1a17330ee8064508c30a776a691c51cbe868f24e252e67a`.
- Static prop caller, scale key: `ev_72909e019c92c181767b8f2eb4a218cd3b6f9e2cd21e05c1b7440a653559e3a8`.
- Factory chain and loader: `ev_f77ab4a4bc3b6bf41a3a3820bdc6b2e95273481cc75dec7776f5cc6fa205a89e`.
- Shape constructor call: `ev_9559b9077343729e1542f4446f572cf1f340d927ea4dfe8ead67c8e3cc1847af`.
- Box support and dimensions: `ev_01af58c66da32107efbc2a1e0153d366900255a7b8a9329fa8a0c21b6d822e82`.
- Archive counts and scene filter setup: `ev_d418fedf88e8efca8199d59e2464c7b31071ea12c358a708dcf83cff64795e1d`.
- Filter bit operations: `ev_b18d1d1a6ab406e0901e2133a8c0868bb624cf965c0fecf1d3baed6236acb679`.
- Custom/base filter predicates: `ev_2481ab878783bbedd6c99bd91e3a18826e301fcc4e38db11cebfae6a3e8fa3e7`.
