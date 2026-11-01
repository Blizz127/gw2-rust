# Build 209026: local prop rotation and collision mesh units

Static REA analysis, 2026-11-01, frozen executable identified in
[client-build.md](../client-build.md). This follows the
[prop translation trace](209026-map-coordinate-spaces.md). No client launch
or physics simulation was performed.

## Local rotation

The prop loader passes record +0x20 (three rotation floats) to
`0x1410bd8f0`. The helper promotes each component to double and uses
half-angles `a = rx/2`, `b = ry/2`, `c = -rz/2`. Constants at
`0x141b91930` and `0x141b919a8` are +0.5 and -0.5.

It calls `0x140e675a0` for cosine and `0x140e67f20` for sine. Their
exception paths lead to the literal names `cos` and `sin` at
`0x1420c86b4` and `0x1420c86a8`; the small-input paths also agree with
those functions. Thus these angles are radians, with no degrees conversion.

Writing `ca = cos(a)`, `sa = sin(a)` and similarly for b/c, the output
four floats are:

```
qx = cc*cb*sa + sc*sb*ca
qy = cc*sb*ca - sc*cb*sa
qz = sc*cb*ca - cc*sb*sa
qw = sc*sb*sa + cc*cb*ca
```

This is Hamilton quaternion product `qy(ry) * qx(rx) * qz(-rz)`, stored
as `[x,y,z,w]`. The output is used directly in the identity-map branch.
For a nonidentity map the caller decomposes the map matrix and combines
rotations separately; the local helper alone does not implement that path.
The client computes products/sums in double and converts each result to
float. A Rust reconstruction need not be bit-identical to its CRT trig
implementation; comparisons should use a small numerical tolerance.

The prop constructor `0x1410be5e0` stores quaternion at object +0x40,
translation at +0x50, transform tag at +0x60 and model scale at +0x70.
New map props receive transform tag zero. `0x1410bf740` copies that
transform for body creation. Its scene helper `0x1410b7ce0` leaves the
zero-tag transform unchanged; the alternate tagged-object path is outside
this trace.

`ModelPlacement::prop_local_physics_rotation_209026` now implements the
local quaternion calculation. Five focused map-reader tests and the
library/inspector build pass. Axis quarter-turns check signs; combined
quarter-turns check the noncommuting multiplication order.

A REA smoke applied the compiled Rust helper to all 8,301 Queensdale props.
All quaternions were finite. Maximum squared-norm error was `8.17e-8`;
rotating a probe vector with the quaternion agreed with independent
sequential Rz(-rz), Rx(rx), Ry(ry) matrices within `3.20e-7` per component.
This validates the reconstruction's algebra over the archive corpus, not
a comparison against a live client physics simulation.

## Mesh data and scale

`0x1410cc590` iterates `havk.collisions` (count +0x20, pointer +0x24,
48-byte records). It copies triangle indices, Float3 vertices and surface
indices unchanged into the cache allocated by `0x1410cba30`.

Body creation reads the prop's stored model scale at +0x70 and passes it
through `0x1410cbee0` → `0x1410cbd20` → `0x1410cc170`.
The latter supplies the cached vertex pointer/count, index pointer/count
and the same scale to `0x1410bd7c0`. That helper constructs a triangle
subpart with 12-byte vertex stride and a uniform XYZ scale. There is no
additional 1/32 or sign inversion on this traced cached-mesh path.
Consequently mesh vertices and prop translations have different stored
conventions: apply model scale to collision vertices, but convert the
stored prop origin through the map matrix and `[x/32,y/32,-z/32]`.

This does not resolve shape fallback behavior, dynamic movement, blockers,
terrain sampling, or mission packet positions. Subsequent
[sequence-selection work](209026-collision-animation-selection.md) adds
an explicit prop lookup; `geometry_meshes` still means the first animation
and should not substitute for that lookup.

## Evidence

Private bundle: `~/re/gw2/notes/209026-collision-rotation-static-1.json`.

| REA record | Observation |
| --- | --- |
| `ev_816850fe7f3dc218c5924ee04303c005000afa226fe7acab19a198611ed31327` | Complete local quaternion helper and unchanged vertex/index copies. |
| `ev_4dcba000b8e4b6877e37b3d4e6e2998a8467d1e9c69fc516a6342431366eed9d` | Half-angle constants, trig entry points and prop transform storage. |
| `ev_edf8c42e9a3300f5acdb35630d91412ccf201c49a9aa77c3bd158541c902b166` | Trig small-input behavior and mesh cache layout. |
| `ev_6aae461ec44e31f2b6aabc08c8e57ddb5d661bc6ea959036f6abef18dec4cb6e` | Trig exception dispatch and shape scale forwarding. |
| `ev_07fc9a4470e2addd83e60a85ef841d02e22bf4fa0d7906b01dccefe914ed810e` | Cos/sin names and cached mesh construction. |
| `ev_85d88108eb9dfdf0dddd2ed2db08962206de31f8975723449f083674f31ffd6e` | Triangle subpart data, uniform scale and prop insertion. |
| `ev_1c0da3a018c5dd799e942196dffe42cd69bb9a523f26d74fdf493790776ea463` | Stored prop scale passed to shape lookup; transform passed to body creation. |
| `ev_8279e9f77be7078c447e4cd3ba26e6170ac9a9f3fae31cfe57744962e08e1463` | Zero-tag scene transform is copied unchanged. |

Rust smoke bundle: `~/re/gw2/notes/209026-collision-rotation-rust-1.json`,
record `ev_c687dee8ffa898014d5a746f0b23217d5118367f5f600d40f4327f2ccafff8ed`.
Private helper: `~/re/gw2/notes/check-map-prop-rotation-rust.rs`.
