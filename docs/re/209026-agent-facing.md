# Build 209026: server-37 facing words

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This extends [agent creation](209026-agent-creation.md).
No client replay was performed.

## Word interpretation

Command constructor `0x141035c50` reads fields 11 and 12 from decoded
message +0x2e/+0x30 using signed 16-bit loads. For each non-sentinel word:

```text
angle = float(signed_word) * f32_pi / 32766.0f
```

The multiply and divide are separate f32 operations. Constants are
`0x14193fc84 = 0x40490fdb` (f32 pi) and
`0x14211cf80 = 0x46fffc00` (32766, not 32767 or 32768).
The special word **0x7fff** becomes positive infinity from `0x141b698cc`.
Other signed values, including -32768 and -32767, use the ordinary formula.

Helper `0x141039590` receives field 11's angle as its first scalar and
field 12's as its second. If either is positive infinity it returns the
zero vector. Otherwise, calling them pitch `p` and yaw `y` by their
observed mathematical roles, it computes and normalizes:

```text
map-space facing = [cos(y)*cos(p), sin(y)*cos(p), sin(p)]
```

The helper's `cosf` and `sinf` calls are `0x140e5bbb0` and `0x140e5c0b0`.
Their exceptional-input paths reference the literal names at
`0x1420c86ac` and `0x1420c86a0`. Squared length is summed as
`y*y + x*x + z*z`; below float `0x2b8cbccc` (approximately 1e-12),
the helper returns zero. Otherwise it divides by the square root through
`0x1409bf290`. The command stores XYZ at +0x98/+0x9c/+0xa0.

Examples before small floating-point residuals:

| Field 11 | Field 12 | Map-space direction |
| --- | --- | --- |
| 0 | 0 | +X |
| 0 | 16383 | +Y |
| 0 | -16383 | -Y |
| 16383 | 0 | +Z |
| -16383 | 0 | -Z |
| 0 | 32766 | -X |

These axis names do not assert compass bearings. Positive map-space Z
becomes negative collision-space Z, so positive pitch points downward
in the collision basis.

## Constructor and collision consumers

Execute `0x141039b10` passes the vector to base constructor
`0x141024aa0`. Before its optional setter call, that constructor asserts
`MathIsUnitVector(localFacing, COLLIDE_VECTOR_LENGTH_EPSILON)`.
It requires squared length within the f32 bounds at `0x141d61fd0` and
`0x141d61fd4`, approximately **0.9801..1.0201**.

Therefore the sentinel-produced zero vector is not established as an
acceptable initial facing: it reaches this assertion before the later
near-zero check can skip the setter. A framing-valid 0x7fff word must not
be confused with a valid creation fixture.

Collision virtual +0x498 dispatches to full setter `0x1410a3140` or simple
setter `0x1410a7ba0`. Both assert unit length, keep X/Y, negate Z and set
W=0. Neither divides a direction by 32. The full setter stores the
direction at +0x180, then creates a separately normalized XY projection
at +0x190 (mask `0x142325f80` clears Z). The simple setter stores the
converted direction at +0x110.

## Offline helper scope

A build-specific protocol helper can decode the two words into normalized
map-space facing, returning no direction for the sentinel. It should not
silently substitute a direction or claim the sentinel can create an agent.
Host Rust trigonometric results need not be bit-identical to the Windows
CRT; the field order, signed scaling, sentinel and coordinate convention
are the established contract.

`gw2-proto::agent_creation::facing_209026` now implements this offline
decode, returning `None` for either sentinel. Two focused tests pass:
field order/cardinal directions/exact half-turn denominator, and sentinel
handling/signed extremes/unit length. The library builds. No orientation
packet was sent, and no Windows-CRT bitwise equivalence test was run.

## Outgoing conversion and quantization

The server-37 command serializer `0x14103b760` calls inverse-direction
helper `0x1410396f0` on command +0x98, then scalar quantizer `0x1410394e0`
on each angle. Pitch goes to decoded message +0x2e and yaw to +0x30,
confirming the receiving field order independently.

For a nonzero finite input, the inverse helper computes XY squared length
as `y*y + x*x`. Below f32 `1e-12` (bits `0x2b8cbccc`) it replaces both
components with positive zero; otherwise it multiplies each by the
reciprocal square root. Pitch is `asinf(clamp(z, -1, 1))`; yaw is
`atan2f(normalized_y, normalized_x)`. The asinf exceptional-path literal
identifies `0x140e5fef0`; `0x140e5cfa8` implements the quadrant-sensitive
arctangent, including signed-zero and half-turn cases. The helper does
**not** normalize the full XYZ input before taking asin. An exactly zero
input instead produces two positive infinities, which quantize to the
0x7fff sentinel.

The scalar quantizer wraps angles outside [-pi, pi] by multiples of f32
2*pi, using ceiling helper `0x1409bebe0` -> `0x140e5e268`. Within that
interval it computes `(angle / f32_pi) * 32766.0`, with separate single
precision operations, and tail-calls `0x1409bebf0`. That helper asserts
the value lies within +/-0x1fffff, adds f32 **14680064** (bits `0x4b600000`),
takes the low 22 result bits and subtracts 0x200000. Under the ordinary
nearest-even floating-point mode this rounds to the nearest integer,
with half ties to even; truncation or Rust `round()` would differ.
The scaled facing range lies entirely within its asserted bounds.

The offline encoder accepts finite directions whose squared lengths
meet the creation constructor's bounds above, without silently normalizing
them. This is an API restriction for creation use, not a check inside the
client inverse helper. It preserves the near-vertical XY handling and
Z clamp. Its asin/atan2 results are already in the quantizer's principal
interval, so arbitrary-angle wrapping is unnecessary for this API. Host
trigonometry may differ at quantization boundaries; no bitwise CRT claim
or runtime orientation validation follows from this trace.

`gw2-proto::agent_creation::facing_words_209026` implements this creation-only
encoder. All five focused facing tests and the library build pass, covering
signed axes/zero, half ties, rejected inputs, vertical projection, Z clamp
and direction round trips. No packet was sent.

Additional private bundle: `~/re/gw2/notes/209026-agent-facing-encoding-1.json`
(902 records, 20,854,076 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_01495251aae13295e4405a9b16468bba418f43a00a63a15d2f1b413138cd469d` | Serializer calls and pitch/yaw destination order. |
| `ev_cf4001802dd1127b7c1d9a5c1d9a60d5eb228f39fa2eecabffbbf0ff0aeec00c` | Scalar quantizer, scaling and asin special cases. |
| `ev_8bd2eebec1eeca9f11cf8978a5ad42ad726bc6243fafa77e9d1ce4de4809affa` | Asinf literal, angle constants, XY threshold and rounding entry. |
| `ev_c9cc6535d8499bb22d1bdac9c952f4cddfd3b10ddbb4ba63effe7627ae95d5c7` | Bit-extraction rounding tail, ceiling helper and quadrant arctangent. |
| `ev_77becd8e3d8de030e8ea7fee914e50bff2adad380d5c5e2a5fc7d838c221e4fd` | Full inverse helper and rounding bias constant. |

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-facing-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_30932644a3ad7ebcf1804bf81fe9cb390d818a8418e85544a1e853a4a3734795` | Word sign extension/scaling, sentinel substitution and constructor checks. |
| `ev_b2cf1d171f721d9f6e0b11c485c82bd898f693099d9a0b37ff1075eaf5630da0` | Vector forwarding to base constructor. |
| `ev_b74a1c3292a2a261e8fb9fb8791a817bda419a65bcda492a788bd80d337c5f71` | Constants, facing helper and normalization. |
| `ev_99399efcd96b449c306d4f99bd1e43e65ae03e3e53dae6f24a71a49784091574` | Setter vtable pointers and trig exceptional-input calls. |
| `ev_86ab8cb9737372f55da5d42795319762321080eb1ca1c4e423c7bce05fd65d5b` | Both setters and sinf literal. |
| `ev_f8318b8d6705cc72ba000f60f55ac8eede1cabb256df16682538836da2a56055` | Cosf literal, length bounds/assertions, XY mask and sqrt helper. |
