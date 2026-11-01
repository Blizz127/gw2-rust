# Build 209026 terrain collision vertices and triangles

Static REA trace of the frozen executable, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This follows [terrain heights](209026-terrain-height.md) and
[chunk collision flags](209026-terrain-flags.md), and applies to the primary
map's identity transform. No runtime replay or User Agreement progression.

## Heightfield construction

`0x1410be760` creates each included chunk's sampled heightfield. The config
at `0x1410be819` has local XYZ scale `[spacing/32, 1/32, spacing/32]`,
33×33 vertices for a 32-cell chunk, and a zero diagonal-flip setting.
Constructor `0x1416de7d0` copies scale to `+0x50`, dimensions to `+0x3c/+0x40`,
and the setting to `+0x48`. The custom heightfield vtable is `0x14212ed00`.

Slot `+0xa0` points through `0x1410c0820` to getter `0x1410c0750`.
For local node (x,z), it adds the chunk's global grid origin (iterator fields
`+0xa0/+0xa4`), divides by cells per chunk (`+0xb4`), and reads the selected
chunk's cached height through its row stride (`+0xb0`). At the entire grid's
right/bottom node boundary it selects the final chunk and local coordinate 32.
At internal boundaries it selects the next chunk's local coordinate 0.
The cached pointers already skip the one-sample stored border. Finally,
`0x1410c07e5` XORs the height's sign bit: the returned value is **-rawHeight**.

This node lookup differs from the point sampler's four-height load, which
reads all four values from the containing chunk. Queensdale's internal border
copies agree, but do not silently assume that for arbitrary archives.

## Body orientation and position

For identity maps, body position is the chunk's upper-left map-local origin,
converted to `[originX/32, originY/32, -originZ/32]` (originZ is zero here).
Its initial quaternion is `[0,0,0,1]` at constant `0x142634270`.

A static quaternion at `0x14289e790` is initialized by `0x1410becb1`:
axis `[1,0,0,0]` from `0x14192d420`, angle `0x3fc90fdb` (float π/2) from
`0x14192d3d0`, passed to `0x1415a1430`. That wrapper calls `0x1415a1470`,
which halves the angle (`0x142634190` contains four 0.5 floats), computes
sine/cosine through `0x141592ba0`, and builds the axis-angle quaternion.
The terrain body constructor composes this quaternion with the map rotation
at `0x1410bead9` before initializing the body.

Thus, for identity maps, the ideal quarter-turn maps local heightfield
`[localX*spacing/32, -rawHeight/32, localY*spacing/32]` to
`[localX*spacing/32, -localY*spacing/32, -rawHeight/32]`.
After adding the chunk origin, each terrain node has collision coordinates:

```
[(rect.minX + gridX*spacing)/32,
 (rect.maxY - gridY*spacing)/32,
 -rawHeight/32]
```

This is the mathematical axis mapping; direct extraction using it does not
claim bit-identical quaternion/trigonometric rounding to the client. Nonidentity
map composition and mission packet coordinates remain outside this result.

## Triangle indices and winding

Heightfield slot `+0xa8` calls `0x1410c0830`, which returns false for the
flip setting. Collection child-shape function `0x1416dff90` loads four nodes
in order `[NW, SW, NE, SE]`, via heightfield slot `+0xa0`, and scales them.
With flip false, at `0x1416e00a7` it chooses:

- Triangle key bit 0 clear: node indices `[0,1,2]` = `[NW,SW,NE]`.
- Triangle key bit 0 set: `[3,2,1]` = `[SE,NE,SW]`.

Both share the northeast–southwest diagonal used by the point sampler.
In collision coordinates, this winding has positive Z for a flat surface.
Chunk exclusion bit 4 and collision-scene rectangle clipping must still be
applied by a caller assembling the scene. Props, water and movement rules are
separate; triangles alone do not establish walkability.

## Evidence

Private bundle `~/re/gw2/notes/209026-terrain-collision-basis-1.json`.

- Body construction: `ev_82036e3a46541ba1860797484941761cd7dad64f1331d098be683d65eb2a8bc6`.
- Axis-angle initialization/vtable: `ev_17560735320fba8ecaeb1c846700fd19eb48ae13128b4a28a48860949e4339a0`.
- Quaternion constants/helper and flip getter: `ev_54ceb018d23a0eee4b5415893648dfb640183320c51cb9417e4525ed7ca781a7`.
- Global node lookup and sign flip: `ev_12af03a6611323d2da20effbea080675ecec756f22f935f44df4302e2e480148`.
- Heightfield constructor: `ev_8252bb4847e13d00ded0c18a6c86797a45a48ccf96c88c17687c766531031be6`.
- Triangle assembly: `ev_fd5a4235b6f6e76601b0b00c87502dc50c797488e95d36d44ce133cebf053fe6`.
