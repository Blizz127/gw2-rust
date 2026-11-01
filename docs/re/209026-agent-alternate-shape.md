# Build 209026: alternate convex character shape

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This completes the bounded shape-builder
branch left open by [creation dimensions](209026-agent-dimensions.md).
No client launch, packet replay or gameplay validation was performed.

## Full-character bit-47 branch

When collision flags +0x58 bit 47 selects the alternate shape, constructor
`0x14109a900` calls scene cache `0x1410b6260`. That cache packs six floats
through `0x1410954b0`, keys them by their bytes, and constructs a missing
shape through `0x1410963f0`. The initial constructor passes:

| Key offset | Collision-space value |
| --- | --- |
| +0x00 | R = final packet radius / 32 |
| +0x04 | Y = array 6 / 32 |
| +0x08 | H = adjusted height / 32 |
| +0x0c | A = array 4 / 32 |
| +0x10 | S = R - (A - A/2) / tan(50 degrees) |
| +0x14 | B = A/2 |

The creation factory's map-unit height adjustment still applies. The
additional full-constructor height minimum described in the earlier note
is skipped on this branch. The angle is the initial factory argument;
the later dimension replacement path obtains it from controller state.

`0x1410963f0` resolves `Y == 0` to R and uses scale 1 in that case;
otherwise its scale is Y/R. It generates three rings with 12 points
each, using angle `i / 12 * f32_two_pi`, for i=0..11:

```text
top:     [R*cos(angle), Y_resolved*sin(angle), H]
middle:  [R*cos(angle), Y_resolved*sin(angle), A]
lower:   [S*cos(angle), (S*scale)*sin(angle), B]
```

It appends `[0,0,0]`, then submits all **37** points at 16-byte stride to
`0x1416c5530`. Its vtable `0x142338438` has RTTI
`hkpConvexVerticesShape`; the constructor's diagnostic path names
`Shape\\Convex\\ConvexVertices\\hkpConvexVerticesShapeConstructor.cpp`.
Configuration helper `0x1416c54f0` supplies the convex margin from
`0x14263a6f4`, initially 0.05 collision units, along with other hull
construction settings. The point list is input to that constructor;
it is not asserted to be the final hull vertices, facets or exact
collision support surface after convex-radius processing.

This establishes **array 6 as the alternate shape's Y radius**, with
zero meaning use the main radius on this branch. Array 4 sets the
middle/lower heights and contributes to the smaller ring's X radius.
Their broader content names and player-specific values remain unknown.
No positivity or successful-hull claim is made for arbitrary supplied
values. The creation-flag eligibility check is recorded separately in
[creation flags](209026-agent-creation-flags.md).

## Simple-character radius cap

Simple constructor `0x1410a54f0` first stores height/32 at +0x17c,
radius/32 at +0x180 and array6/32 at +0x184. At `0x1410a5649` it compares
the scaled radius against `0x141ae16c0` (f32 0.05) and replaces a greater
radius with 0.05. Thus for finite positive inputs the stored radius is
`min(radius_map / 32, 0.05)`, at most **1.6 map units**.

The dimension replacement method `0x1410a7970` repeats that cap.
The initial factory height adjustment occurred before the cap and used
the original input radius. It must not be recomputed from the capped
value when matching constructor behavior.

The simple constructor then obtains its cached box through
`0x1410b6000(capped_radius, height_collision)`. Packing helper
`0x141095380` supplies half-extents
`[capped_radius/sqrt(2), capped_radius/sqrt(2), height_collision/2]`.
It does not pass array 6 to this box builder. The bounded search did not
establish array 6's later simple-character use; offset matches in a method
whose this pointer is interface +0x10 must not be confused with whole
object +0x184.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-alternate-shape-1.json`
(914 records, 21,201,462 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_146a151bb05e2a84ad898c047fc8dc3631077838ff63993e54103e9675f72bc2` | Shape cache, contact-setup use of array 4, simple constructor cap. |
| `ev_b311fd4e87f8147fa254e81abf85f060270f0e82fdc798c5643dbe645d76dfce` | Six-float key, all 37 generated points, simple shape and radius constant. |
| `ev_7c438277767d63525be2d699036fe6de6af0de40440fec7df08e1364884393ac` | Convex constructor/configuration, 12-point ring constant, bounded simple-field search. |
| `ev_50989a19b1affefa5027d6dfb7102333562f86e7822e266d7d2d7c5279cdf1d5` | Full-constructor arguments, hull source literal, margin constant and RTTI locator. |
| `ev_fc183bc1b05a807da39f86b1ff0f9df6fda575d5f79128135648ecc9e541f439` | RTTI type RVA, repeated simple radius cap and box half-extents. |
| `ev_b0ff79fa044d1745a0efc11dc68902fec54980b3675e663cd0b2824add897d04` | RTTI name, reciprocal-sqrt-two constant and simple replacement entry. |
