# Build 209026 ray clipping and result coordinates

Static REA analysis of frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Continues the [query-construction trace](209026-ray-query-construction.md).

## Scene-bound clipping

Helper `0x1410bd5b0` receives the scene bounds, two mutable collision-space
endpoints and an output distance. If both endpoints are inclusively inside
all three bounds, it returns true and zero distance without changing them.
Otherwise it starts a parametric interval `[0,1]`; for each axis with nonzero
direction, it intersects that interval with the two plane fractions, sorted
by direction. It returns false when the resulting lower fraction exceeds
the upper fraction. Otherwise it replaces both endpoints with the clipped
positions and reports the distance from the original first endpoint to its
replacement.

An exact zero direction skips that axis in the slow path, without a separate
parallel-outside-slab rejection. Consequently this helper alone is not a
complete rejection test for a segment parallel to, and outside, a scene slab.
The later world traversal can still return no hit. This observed detail should
not be silently replaced by a textbook slab test and called client-equivalent.
No runtime replay or client patch was performed.

Collectors compute `clipped_start_distance + hit_fraction * clipped_length`.
The fraction remains relative to the clipped segment; the distance includes
the removed prefix. This matters when comparing distance and fraction fields.

## Generic 0x58-byte result

`0x141096740` fills the generic output used by these collectors. Constants
`0x14192d3f0 = 32.0` and `0x14192df28 = 0.03125` verify the inverse unit
conversion between scene wrappers and result positions.

| Offset | Observed generic value |
| --- | --- |
| `0x00` | Result of associated object's first virtual method; numeric type discriminator |
| `0x08` | Initially null, populated for some discriminator branches |
| `0x10` | Prefix-adjusted distance multiplied by 32 |
| `0x14` | Raw hit fraction of the clipped segment |
| `0x18..0x20` | First supplied hit position converted to `[32*x,32*y,-32*z]` |
| `0x24` | Position tag, initially zero |
| `0x28..0x30` | Second supplied position, same conversion |
| `0x34` | Second position tag, initially zero |
| `0x38..0x40` | Hit normal `[nx,ny,-nz]`, with no factor of 32 |
| `0x48` | Associated object's virtual `+0x10` result |
| `0x50..0x53` | Initially zero, then two 16-bit values for certain type branches |

Both ray collectors pass the same interpolated hit point for the two position
arguments, so their generic values initially coincide. The helper also supports
other callers; the two fields should not be merged in a general result layout.
Discriminators 4/5 can resolve an associated transform and rewrite both position
fields and their tags through `0x1410be440`. Discriminators 11 and 14 take
separate metadata lookup paths. These discriminator numbers have not been
identified as collision-layer numbers; they must not be conflated.

This trace establishes engine result coordinates, not mission packet position
encoding. It does not prove a hit is walkable or a safe spawn.

## Evidence

Private bundle: `~/re/gw2/notes/209026-ray-results-1.json`.

- Complete clipping and result helper: `ev_4c2129206474fbe8e308a67a7752c0b5fc334e702482ba9a05c648fb3d9837ce`.
- Unit constants: `ev_df3e2149a06d713f44d7f02246b57d2f01399e845e8ca2d745e359d45faa9028`.
- Collector distance/fraction data flow: `ev_ee55b579b0084afe6eea2abaf79d78da476583b24453132edf1356d65c1ee32d`.
