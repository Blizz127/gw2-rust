# Build 209026 zone-model collision placements

REA static trace of executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`,
plus Queensdale archive file 192711. No runtime replay or agreement progression.

## Placement and selection

The collision loader's loop at `0x1410b8950` handles `havk.zoneModels`:
count at pack `+0x68`, pointer `+0x6c`, record stride 32. Fields are scale
at `+0`, translation Float3 at `+4`, rotation Float3 at `+0x10`, geometry
index at `+0x1c`. There is no per-placement sequence token.

After geometry/scene overlap checking, the loader creates an object through
`0x1410becf0` and calls body builder `0x1410bfa40` with the stored scale,
translation, rotation and map transform. At `0x1410bfadc`, the builder loads
**fixed sequence token `0x83e502b9fa`** from `0x14212ec48` and passes it to
`0x1410cbee0`, the same first-exact-match geometry animation lookup used by
props. This token must not be replaced with the prop default `zeropose` or
with the first animation. Its textual token name is not assigned here.

The normal zone body uses collision filter value 13 (`0x1410bfad7`). A null
shape switches to value 15 and the same small-box fallback documented in
[prop fallback](209026-prop-fallback.md). No Queensdale zone selection is empty.

## Primary-map transform

In the identity-map branch (`0x1410bfb0c`), rotation goes through the same
`0x1410bd8f0` quaternion helper as props. Translation is multiplied by 1/32
and Z sign reversed at `0x1410bfb23`. Scale was passed unchanged to the
shape lookup. Therefore the existing primary-map prop matrix formula also
applies to zone collision vertices:

```
world = rotate(qy(ry)*qx(rx)*qz(-rz), scale*meshVertex)
        + [translateX/32, translateY/32, -translateZ/32]
```

There is no second unit conversion on cached mesh vertices. Nonidentity map
handling follows a different branch and is not implemented by this formula.

## Queensdale audit

A private helper selects the fixed sequence in each zone geometry and applies
the existing matrix implementation, comparing against independent double-
precision sequential axis rotations. All 4,381 zones have a matching nonempty
selection. They contribute 62,148 vertex instances and 92,367 triangle
instances. Scales range from 0.5 to 1.528125. All matrices/transformed vertices
are finite; maximum coordinate difference is 0.00006124218 collision units.
This verifies archive coverage and reconstruction algebra, not a client replay.

## Evidence

Private bundle `~/re/gw2/notes/209026-zone-collision-1.json`.

- Zone loader, record offsets, fixed token and object constructor:
  `ev_732dd19be275869701e66bba42a9899f6a4a5efb5b485d3972f11ba21efee54b`.
- Body builder, sequence lookup, identity transform and fallback:
  `ev_9314f17f72725c999853618007a3ad40b301d57f2df159b4811ad90b2ca08d95`.
- REA execution of private `check-zone-world-matrix.rs`:
  `ev_2d4da3df8fca81d9c5653b6fc6bf2bb40c62ccce9d7b759bb4e68b9ff7ae6469`.
