# Build 209026: primary-map character bounds

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). No client launch or mission replay.

## Three bounds pairs

On the ordinary `havk` load path, `0x1410b5aa0` initializes three distinct
collision-space AABBs. For a primary map with identity transform, default
request rectangle and a nonzero stored minimum vector:

| Scene offsets | Minimum | Maximum | Observed use |
| --- | --- | --- | --- |
| +0x10 / +0x20 | `[rect.minX/32 + inset, rect.minY/32 + inset, havk.minZ]` | `[rect.maxX/32 - inset, rect.maxY/32 - inset, havk.maxZ]` | Full-character creation clamp `0x1410b6a60`. |
| +0x30 / +0x40 | Stored `havk` minimum unchanged | Stored `havk` maximum unchanged | Basis for the expanded bounds below. |
| +0x50 / +0x60 | `havk.minimum - [192,192,192]` | `havk.maximum + [192,192,192]` | Simple-character creation assertion. |

Here `rect` is the loaded/transformed `parm` rectangle, and
`inset = floor(3072 / cellsPerChunk) / 32`. Zero cells-per-chunk defaults
to 32. Queensdale uses 32, giving an inset of **3 collision units**.
The inset affects only X and Y; stored Z bounds retain collision units
and are not divided or negated again.

For Queensdale's rectangle `[-43008,-27648,43008,30720]`, the full-character
clamp's XY interval is **[-1341,-861]..[1341,957]**. Stored Z limits are
approximately [-252.60,480.97]. The origin is inside, but that alone
does not establish walkability, clearance or a valid spawn.

## Rectangle provenance

Scene construction `0x141095070` gets map module 16, calls its virtual
+0x48, and copies the returned rectangle into the loader's argument 3.
Module descriptor `0x141d623e0` identifies module 16 as `Params` and
factory `0x140c3c400`. The installed vtable is `0x141d62428`; +0x48 is
`0x1402cfd80`, returning object +8. The `parm` loader fills and transforms
that rectangle as recorded in [coordinate spaces](209026-map-coordinate-spaces.md).

The scene loader's argument 4 is a request rectangle. It first intersects
the two rectangles (maximum minima, minimum maxima); if either resulting
axis has no positive width, it falls back to the Params rectangle.

The default-view load loop explicitly zeros the request rectangle at
`0x1409524e4` before calling `0x140c342a0`. It reaches request +0x274 via
`0x140c392c0`, then module-construction options +8 via `0x140c396db`.
Scene construction passes that +8 rectangle to `0x1410b5aa0`. A zero
rectangle cannot produce a positive-area intersection, so this traced
primary-map path uses the full Params rectangle. Other callers can supply
a real clipping rectangle and are outside the offline helper's scope.

## Inset provenance

At `0x1410b5cf4`, the scene gets module 23 and calls virtual +0xb0.
Descriptor `0x141d62610` identifies module 23 as `Terrain`, factory
`0x140c3d660`. The archive path calls `0x140c432f0`, constructs through
`0x140c3d690`, then returns the whole object +0x20 interface.

That interface's vtable `0x141d62668` has +0xb0 = `0x140c3da90`, returning
interface +0x1ac, i.e. whole object +0x1cc. Constructor `0x140c3d852`
sets this field to unsigned integer `3072 / cellsPerChunk`. Thus the
returned value is cell spacing, not chunk count or character radius.
The scene converts this integer to float, divides by 32 and adds/subtracts
it from XY at `0x1410b5d49` onward.

The fixed expansion comes from vector `0x14212eac0`, whose four float
words are `0x43400000` (192). The loader clears the W lane, subtracts it
from stored minima and adds it to stored maxima. The expansion is already
in collision units; it is not 192 map units.

## Supported offline scope

An offline helper can reproduce these bounds for the default identity
primary-map path, reject nonfinite/reversed or empty inset bounds, and
report whether a probe hit lies within each inclusive XYZ pair. Bounds
membership is separate from clipping the probe's ray, resolving tags,
collision filters, shape overlap and movement validity.

The client has a special branch when all three stored minimum components
equal zero: it substitutes other bounds, including fixed Z constants.
This branch, the missing-`havk` fallback `0x1410b5e50`, nonidentity maps,
and nondefault request rectangles are not implemented by this helper.
Rejecting the zero-minimum case prevents silently reporting ordinary-path
bounds for a different client path.

## Evidence

Private bundle: `~/re/gw2/notes/209026-scene-bounds-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_e566464e94339c70c4fd933d7f81428928a893fa2a4c67fe7cc51d88c35a70a2` | Havk bounds, zero-minimum special branch, inset call and bounds writes. |
| `ev_1fe3d6be43ec518cb6581c463be01008e3f6697f34794f354af306cc93366135` | Rectangle intersection/fallback and scene construction caller. |
| `ev_019870c667723a43088acd9cd3b97fed3843ba5c0363ba3b9acaed00f7dac2f3` | Scene constructor obtains module 16 rectangle. |
| `ev_7e6f0ccdad917a8640fe399a9f24384652ead6228afe0abec2e14b02ea831abb` | Map module getter indexes its table by module ID. |
| `ev_47ff8fc7a22ce6daef9664b6d56a58687ef7502199a1746e3f614fa0b9bf4a02` | Module options copy and dispatch through registered factories. |
| `ev_24171442331622d4910c3ff547f9dbb3b4d01efc2892ddc451a4f15c3d3fc784` | Params/Terrain descriptors, factories and getter pointers. |
| `ev_689288b2cc16229878dc787a87eba9994b47a1a53c906bb5b8a8bf20037eb829` | Module names, terrain getter and returned interface. |
| `ev_7e2e25772aece11b56e4c74147951f35fd866fc549f868393357f3af1e77a441` | Integer cell spacing, Params getter and request rectangle copy. |
| `ev_fc13f4b3364547b2fbb9a41ae50a9404a643f525a8ed57a84252896e6d5deb19` | Default-view identity map entry and zero request rectangle. |
| `ev_bed4dc26628c69797e54019bc14aedceaf56c85fd32d9f94a0ecb45530fcd900` | Expansion constant 192. |

The [agent position trace](209026-agent-position.md) supplies the two
character consumers and their inclusive comparisons/clamp behavior.

## Implementation and verification

`gw2-dat::scene_bounds::SceneBounds209026` now constructs the supported
primary-map bounds and exposes inclusive finite-point membership. The
probe prints both character bounds and membership for each reported hit;
it still does not clip rays or triangles to those bounds.

Two focused tests pass: separate rectangle/archive roles, inclusive edges,
unmodified Z, integer cell-spacing division and rejection of nonfinite,
reversed, empty-inset or unsupported zero-minimum inputs. The probe builds.
The real Queensdale center smoke through REA reports:

```text
full min [-1341, -861, -252.59863], max [1341, 957, 480.96875]
simple min [-1616.6771, -1240.9697, -444.59863]
simple max [1564.1681, 1258.6284, 672.96875]
```

The center surface hit is inside both pairs. Prior hit XYZ, normal and
triangle counts are unchanged. Smoke evidence:
`ev_5353f08e978bae7571e9a6aed33b43395bed03e3b90d718dff344be013a91eba`.
