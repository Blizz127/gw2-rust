# Build 209026: body-position and water-level dimension offsets

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This extends the [creation dimensions](209026-agent-dimensions.md).
No client launch or packet replay was performed. Array indices are zero-based;
collision offsets below are relative to the whole full-character object.

## Array 5: vertical offset on the body-backed position path

The constructor stores `-array5 / 64` at collision +0x2a8, and separately
stores `array5 / 32` at +0x2e4. Getter `0x14109e870` multiplies the latter
by 32. Setter `0x1410a30b0` updates +0x2e4, notifying interface +0x10
virtual +0x48 when its value changes; that setter does not update +0x2a8.
The dimension replacement routine around `0x1410a1d4f` does update +0x2a8.
These two stored values must not be treated as a permanently aliased field.

Position getter `0x14109ea70` has two paths:

- With collision flags +0x58 bit 1 clear, it reads collision +0x140 and
  converts collision XYZ to map `[32*x, 32*y, -32*z]`.
- With bit 1 set, it obtains the physics-body position through
  `0x1410b4710(scene +0x70, body +0x60, out)`, subtracts a Z-only vector,
  then performs the same map conversion. The offset is
  `height_collision / 2 + field[0x2a8]`, equivalently
  `(adjusted_height_map - array5) / 64` subject to float operation order.

`0x1410b4710` interpolates the body's swept transform at scene time,
including its rotated local center offset; it is not just a raw vector
load. Another getter, `0x14109e210`, subtracts the same Z-only offset
from body +0x1a0 before producing map position, alongside orientation
and velocity outputs.

The inverse direction is visible at `0x1410a3bf4..0x1410a3cd4`: the body
position supplied to `0x1417244a0` adds this Z-only offset to the selected
collision-space position. The helper dispatches to body +0x150 virtual
+0x70 and refreshes body state. This establishes array 5's participation
in body/character vertical-origin conversion on this path, without
assigning an unsupported content name or choosing a value.

The ordinary capsule's local geometry remains bottom-relative, as the
earlier note establishes. That shape-local fact does not make every
physics body position interchangeable with the packet position: the
bit-1 path has an additional origin conversion. Activation of that path
and content-specific defaults remain unresolved.

## Array 7: offset to the water level used by movement

The constructor stores `-array7 / 32` at collision +0x454. Movement-input
preparation `0x1410a11e0` writes, at `0x1410a1849..0x1410a187e`:

```text
movement_input[0xfc] = collision[0x160] + collision[0x454]
                    = water_z_collision - array7 / 32
```

Collision +0x160 is the water surface already established by the
[character height trace](209026-character-ray-mask.md): scene helper
`0x1410b7650` queries scene virtual +0x80, whose implementation
`0x1410b76f0` enters water collection `0x1410cfb40`. It converts map Z
to collision Z by negation and division by 32. The assignment to +0x160
is independently visible at `0x1410a125d`.

The adjusted level is consumed, not merely stored. Routine
`0x1410c6970` receives that movement input in R8 and compares +0xfc minus
1.25 with position Z at +0x98 before selecting movement states 5 or 6,
subject to other flags and predicates. Related state routines also use
the same field. Their full transition semantics and names are not
claimed here.

Movement update `0x1410c6170` uses the adjusted level in vertical output
rules near its end. One conditional branch limits output Z according
to the difference between `(level - 1.3)` and input position Z, scaled
by 0.98 and 25. Another, separately gated branch predicts Z using
input +0x88 (initialized to 0.04), and adds a vertical correction toward
`level - 0.25`, multiplied by input +0x8c (initialized to 25).
These are observations of the selected branches, not a complete swimming
implementation. Positive array 7 lowers this movement reference level
in the collision Z-up basis. It does not change the map's actual water.

## Limits and evidence

The observations identify uses of arrays 5 and 7 but supply no race,
model or player-specific default dimensions. No guessed defaults or
movement implementation were added. Array 4 and 6 still have only the
shape/height uses recorded by the earlier dimension note.

Private bundle: `~/re/gw2/notes/209026-agent-dimension-offsets-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_830370478739f15ec9cb3e040bf97fce01ca53863e539702c72fb9f7519ed560` | Bounded constructor/consumer search for dimension fields. |
| `ev_3651ea2a64857ce6c2cb0c7f4ce8e9648bc335d471e1d869ba35a4bf0cd07025` | Position getter, inverse offset, constructor's second array-5 store. |
| `ev_fe8a814a99ebff73ea634623ae6801dceacda2899f6cb14113ced92331953471` | Water-level write, movement-input field, independent +0x2e4 getter/setter. |
| `ev_294de2eed537514ea739e3b0162f8fefff6325e8e266c615a778d6c6ee32d84f` | Water conversion, body interpolation and position setter dispatch. |
| `ev_ebb4add5cf730ee5a0a6b1a322e7ece5ffe48a96fe85ab7b1ed207b0c1ff2edb` | Movement-level consumers and body position/orientation/velocity getter. |
| `ev_40a3dd9cbb0c6825e9b47e9d2d94981823a88bd2720d2ff3d04f4ecddbaf3ac9` | Movement-input binding, conditional vertical rules and constants. |
