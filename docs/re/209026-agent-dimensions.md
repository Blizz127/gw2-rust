# Build 209026: creation dimensions and capsule origin

Static REA findings, 2026-11-01, using the frozen executable identified in
[client-build.md](../client-build.md). This extends the [server-37 field
layout](209026-agent-creation.md) and [position trace](209026-agent-position.md).
No new packet fixture or client replay was performed.

## Radius and array dataflow

Server 37 field 14, the final float, is **collision radius in map units**.
Command +0xc4 reaches base-agent constructor `0x141024aa0` as argument 11.
It is forwarded to `0x141031290`, which stores it at its subobject +0x68
(whole base agent +0x80). The base constructor requires it to be greater
than zero; the assertion at `0x14211b1d8` says `radiusCollision > 0`.

Field 13's eight floats are copied unchanged from decoded array storage
into command +0xa4..+0xc3, then base agent +0x138..+0x157. The following
table uses zero-based array indices and whole-object offsets. These are
observed uses, not proposed names for all eight values.

| Input | Base-agent offset | Full collision-character initial use |
| --- | --- | --- |
| Array 0 | +0x138 | Base movement speed; also initializes current/previous speed at +0x15c/+0x160. |
| Array 1 | +0x13c | Turn-command scale; not passed to the collision factory here. |
| Array 2 | +0x140 | Stored; not passed to the collision factory here. |
| Array 3 | +0x144 | Height input; after factory adjustment and /32, collision +0x29c. |
| Array 4 | +0x148 | /32 to collision +0x2ac; participates in a second height minimum and an auxiliary shape's height. |
| Array 5 | +0x14c | -value/64 to collision +0x2a8. Full meaning unresolved. |
| Array 6 | +0x150 | /32 to collision +0x2a4; used as another shape parameter on a flagged branch. |
| Array 7 | +0x154 | -value/32 to collision +0x454. Full meaning unresolved. |
| Final float | +0x80 | /32 to collision +0x2a0, the radius used by the capsule and six-point ray sample. |

The subsequent [movement-parameter trace](209026-agent-movement-parameters.md)
establishes the first two roles and records the third scalar's limited
consumer evidence without assigning it a semantic name.

`0x14109ca00` receives flags, height, radius and array 6 in the four
register argument slots. Stack arguments include constant 50, array 4,
array 5, array 7, position and the agent interface. It forwards these to
`0x14109a900`. The constructor field writes are at `0x14109aa59` onward.
The radius getter `0x14109e1e0` multiplies collision +0x2a0 by 32, matching
the map-unit radius used by [six-point sampling](209026-character-ray-mask.md).

## Height adjustments

For finite ordinary inputs, both full factory `0x14109ca00` and simple
factory `0x1410a5e20` perform the same map-unit adjustment:

```text
if height < 2 * radius:
    height = 2 * radius + 0.1
```

The full constructor divides by 32 and may raise the result again. On
the branch where `(collision flags & 0x800800000000) == 0`, it applies:

```text
height_collision = max(height_collision,
                       max(array4 / 32, 2 * radius_collision) + 0.1)
```

The latter 0.1 is in collision units, unlike the earlier 0.1 in map units.
Both use float constant `0x141933f44` (`0x3dcccccd`). This distinction
matters when reproducing dimensions; a single map-unit clamp is insufficient.
No broad malformed-float acceptance claim follows from these comparisons.

## Ordinary capsule and position origin

With the above full-constructor flag mask clear, the primary shape path
calls `0x1410b6030(radius_collision, height_collision)` at `0x14109aef8`.
That function caches a shape keyed by the two floats through
`0x1410953c0`, or constructs it via `0x141096260`:

```text
endpoint A = [0, 0, radius_collision]
endpoint B = [0, 0, height_collision - radius_collision]
capsule radius = radius_collision
```

The final constructor is `0x1416c2e90`, vtable `0x1423381e0`; its RTTI
names `hkpCapsuleShape`. Its AABB method `0x1416c2f20` transforms the two
endpoints and expands their component bounds by the radius plus tolerance.
Thus, with an identity body orientation and zero extra tolerance, this
capsule extends from local Z=0 to Z=height_collision, not equally above
and below the origin. The [resolved/clamped packet position](209026-agent-position.md)
is passed into the character controller construction. This establishes
the bottom-relative shape convention on this branch, not successful
standing contact at an arbitrary triangle intersection.

The flagged branch can instead call `0x1410b60f0` → `0x141096310` →
`0x1416c3a60`, whose RTTI names `hkpCylinderShape`. Its supplied endpoints
are `[0,0,0]` and `[0,0,height_collision]`; it receives a separate convex
margin, initially 0.05 at `0x14263a6f4`. Another branch calls `0x1410b6260`
with additional parameters and has not been fully traced here.

The full constructor also creates a box through `0x1410b6000` and
`0x141095380`: half-extents are approximately
`[radius/sqrt(2), radius/sqrt(2), height/2]`. It uses a separate physics
object, so this centered box alone must not be used to infer the main
capsule's position origin. Auxiliary shape placement and later updates
remain open.

## Simple path and limits

The simple factory takes array 3, final radius and array 6, together with
constant 55. Its constructor stores the adjusted height/32 at +0x17c,
radius/32 at +0x180, and array6/32 at +0x184. It may cap the radius later;
the threshold and complete simple-character behavior are not established
here. Do not apply the full capsule convention to this path by assumption.

The full factory asserts `createFlags & COLLIDER_CHAR_CREATE_FLAG_PLAYER`.
The base constructor derives those flags from creation bytes and can also
alter path selection through a runtime query. Correct byte values, the
remaining array semantics, content-specific dimensions and complete
bootstrap state still need evidence. No guessed dimensions were installed.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-dimensions-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_30932644a3ad7ebcf1804bf81fe9cb390d818a8418e85544a1e853a4a3734795` | Array copy, radius assertion, base constructor and factory arguments. |
| `ev_2a8db278af0a5c520e0cb24cc628cbb7f1e90a2b514729202cad6a62128bd648` | Full/simple factory height adjustment and forwarding. |
| `ev_4bf75002cce11ccee01f10458ae5730503a8809b1d12af18d3d733d68adc3d28` | Constructor dimensions and simple path. |
| `ev_85371b5adab45fc49ef94d9c0b3441ca1c54f46d5911ab86fbb8ebb4d73cb9d5` | Height minimum, primary/auxiliary shape choices and controller input. |
| `ev_d881fd4476c582da9b7a0541969ae1a3c7f34ebe454260b912bdc93b91eb801a` | Radius assertion text, constants and shape caches. |
| `ev_a82f523ac3afea69e2f099ef79d509f14353bfe36be4c6e96c022c7952fedf36` | Radius storage, shape endpoints, box half-extents and player assertion. |
| `ev_2fd89d036f96ca2f1f9aeb14943b960e753fc66f4f8487678924c49e077b32f3` | Shape constructors/AABB and constants. |
| `ev_bed4dc26628c69797e54019bc14aedceaf56c85fd32d9f94a0ecb45530fcd900` | RTTI locator/type RVAs and radius getter. |
| `ev_56aa6405c23dbb4ac1d02ce8f6504de777ddbee03487b3f3f62e48ad89fd039b` | Capsule/cylinder RTTI names and second height minimum. |
