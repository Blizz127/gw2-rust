# Build 209026: server-37 position into collision

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). No mission packet was sent and the
User Agreement hold remains unchanged.

## Coordinate result

Server 37 field 10 is a Point3: three raw float words followed by a varint
u32. Its XYZ values reach both collision-character constructors in map
units. Before resolving the fourth value, both constructors compute:

```text
collision XYZ = [map X / 32, map Y / 32, -map Z / 32]
```

The fourth value is a transform tag on this path. Zero selects no extra
transform. Therefore an identity-primary-map collision point `[x,y,z]`
corresponds to server-37 Point3 `([32*x,32*y,-32*z], 0)` before bounds
handling. This connects the offline geometry probe to packet coordinates;
it does not establish a valid spawn or the meaning of every Point3 message.

## Dataflow

| Stage | Observed position handling |
| --- | --- |
| Handler `0x1410335c0` | Constructs a command through `0x141035c50`. |
| Command constructor `0x141035c50` | Copies decoded message +0x1e..+0x2d unchanged into command +0x88..+0x97. XYZ precedes the u32 tag. |
| Execute `0x141039b10` | Passes command +0x88 as argument 8 to base-agent constructor `0x141024aa0`. |
| Base constructor | Reads that pointer from its adjusted stack +0xb8. The full collision factory `0x14109ca00` receives it as argument 9; the simple factory `0x1410a5e20` receives it as argument 6. |
| Full factory | Passes the same pointer as argument 11 to `0x14109a900`, where it is `[rbp+0x1d0]`. |
| Simple factory | Passes the same pointer as argument 9 to `0x1410a54f0`, where it is `[rsp+0xd0]`. |

The full constructor's conversion begins at `0x14109acd2`. Constant
`0x14192df28` is 1/32; `0x141936e00` supplies the Z sign bit. It stores
the converted vector plus unchanged tag at character +0x30/+0x40,
+0x120/+0x130 and +0x1a0/+0x1b0. It resolves the first tagged point through
`0x1410b7710`, then calls the bounds helper below. The resulting vector
is used for physics construction and copied to character +0x140 at
`0x14109b216`.

The simple constructor performs the same conversion at `0x1410a56bc`,
building a vector at stack +0x20 with the tag at +0x30. It queries water
through `0x1410b7650`, resolves the tagged point through `0x1410b7710`,
and stores the result at character +0x130. It asserts that XYZ lies in
scene +0x50..+0x60; it does not call the full constructor's clamp here.

## Transform resolution and bounds

`0x1410b7710` reads the tag after the 16-byte collision vector. If both
bits 31 and 30 are set, it uses the low 24 bits to look up scene +0x4f0.
When an object is found, `0x1410abe00` supplies its transform and
`0x1410be200` applies it. Otherwise the vector is copied unchanged.
Zero thus has a directly observed identity path. The complete tag
namespace and associated-object lifecycle remain unresolved; this is not
evidence that the u32 is a map ID.

The full constructor calls `0x1410b6a60` with the resolved vector. For
finite XYZ already inside inclusive scene +0x10..+0x20, that helper returns
zero without changing the vector. Otherwise it clamps components using
SSE min/max and returns one. In the latter case the constructor replaces
its +0x30 and +0x120 vectors with the clamped result and clears their tags.
An out-of-bounds input therefore need not become the requested initial
position. The follow-up [scene-bounds trace](209026-scene-bounds.md)
establishes their sources: the full clamp combines the map rectangle and
terrain-cell inset with archive Z, while the simple check uses expanded
archive XYZ bounds.

## Practical scope

The offline probe can report the equivalent tag-zero packet XYZ for each
triangle hit. Such a point is a surface intersection, not an agent origin
with validated dimensions, clearance, water behavior, filtering or bootstrap
state. No automatic spawn choice or client replay follows from this trace.
Float conversion also requires finite representable values; multiplying an
arbitrary finite collision coordinate by 32 can overflow.

The updated probe builds and passes a real Queensdale center smoke through
REA. Collision hit `[0,0,-0.23037942]` reports packet surface point
`[0,0,7.3721414]`, tag zero. Triangle counts and the previous hit/normal
are unchanged. This is an output check, not a client spawn test.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-position-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_b2cf1d171f721d9f6e0b11c485c82bd898f693099d9a0b37ff1075eaf5630da0` | Handler and execute method. |
| `ev_30932644a3ad7ebcf1804bf81fe9cb390d818a8418e85544a1e853a4a3734795` | Command copy and base constructor argument flow. |
| `ev_2a8db278af0a5c520e0cb24cc628cbb7f1e90a2b514729202cad6a62128bd648` | Full and simple collision factories. |
| `ev_4bf75002cce11ccee01f10458ae5730503a8809b1d12af18d3d733d68adc3d28` | Constructor prologues, full conversion constants and simple conversion. |
| `ev_85371b5adab45fc49ef94d9c0b3441ca1c54f46d5911ab86fbb8ebb4d73cb9d5` | Full conversion, tag storage, bounds call and final position. |
| `ev_2d314b545a8de8246dc6b8ac7943e28b478ef5e6cf5ef3e4565abb163c329023` | Bounds clamp, tag resolver and scale load. |
| `ev_6c66cc456acf01de30b2ae33702f36f7c269f77a54aeadedf755f7139aa9380b` | Updated offline probe smoke and tag-zero surface point. |

The full constructor has split PE runtime-function entries: the entry
starting at `0x14109a900` ends at `0x14109a92e`, but its body continues to
`0x14109b30e`. The short metadata range alone is not its complete body.
