# Build 209026: server 87 collision position state

Static REA analysis on the same frozen executable as
[server 86](209026-agent-position-update.md). No client replay occurred.

Server **87**, descriptor **`0x1425b8b60`**, handler **`0x141035190`**,
has this field order:

| Field | Kind | Decoded offset | Command destination |
| --- | --- | --- | --- |
| Agent index | Word | +2 | Bound by submission |
| Remote tick | Dword | +4 | +0xb8 |
| Command flags | Byte | +8 | +0x50, OR 0x08032000 |
| Position vector | Float4 | +9 | +0x70 |
| Transform tag | Dword | +0x19 | +0x80 |
| Second vector | Float4 | +0x1d | +0x84 |
| Third vector | Float4 | +0x2d | +0x94 |
| First state byte | Byte | +0x3d | +0xa4 |
| Second state byte | Byte | +0x3e | +0xa5 |

Kind 9 is **Float4, 16 raw bytes**, verified in the
[primitive-width follow-up](209026-primitive-fields.md). Each Dword remains
a varint on the wire. The first vector plus tag is not the 12-byte XYZ plus
tag encoding of Point3.

The handler allocates 0xc0 bytes and calls **`0x141038960`**. Like 86,
this constructs command kind **0x2b**, vtable **`0x14211cb88`**, but sets
command **+0xbc = 1**. It checks the first vector through `0x141030760`
and the second through `0x141030770`; either false result adds flag 0x40000.
The first forwards to scene virtual +0x2e0, **`0x1410bc5c0`**: sum of
all four squared float components must be strictly less than scene +0x4b8.
The second forwards to +0x2f8, **`0x1410bc780`**, using scene +0x4bc.
Neither predicate applies server 86's map-unit scaling or resolves the tag.

Both message forms use the same bound-agent method `0x14102b370`. After
its common gates, the nonzero +0xbc branch calls collision virtual
**+0x448** with command +0x70 and remote tick +0xb8, then completes.
The shared serializer `0x14103e0f0` selects server 87 (or opposite-direction
26) for this command form. It only serializes the tag when its bit 30 is
set, otherwise emitting zero; the incoming decoder retains the raw Dword.

## Full collision implementation

Full collision vtable `0x14212c908` maps +0x448 to **`0x1410a2710`**.
The complete method through `0x1410a29fc` requires flags +0x58 **bit 1
set and bit 7 clear**. Its input begins with a 16-byte vector and a tag
at +0x10. It passes that pair directly to the scene transform resolver,
without 1/32 scaling or Z-negation. Thus the position vector is already
in collision coordinates on this path, unlike server 86.

It can convert frame/tag according to existing associated-object state,
then writes the tagged point to collision +0x30/+0x40 and resolves it.
An expanded scene-bounds check (+0x50..+0x60) controls whether the tagged
point is also copied to +0x120/+0x130. On success, input bytes +0x34/+0x35
are stored to object +0xf8 offsets +0x6a/+0x69. On failure, the existing
+0x120/+0x130 pair is restored to +0x30/+0x40 and the local second vector
is zeroed. The already resolved stack vector is not recomputed in that
branch; this note does not describe it as a complete rollback.

The method saves old resolved position +0x140, then writes the stack
resolved vector to +0x140 and the second input vector to **+0x150**.
Flags bit 31 additionally forwards these to `0x1417b9450`/`0x1417b9490`.
It calls `0x1410a4c30(this,1,0)`. Unless bit 30 bypasses it, a distance
hook at virtual +0x538 receives distance and scene tick +0x214 minus
remote tick. That hook is the always-true `0x1402406f0` in this full
vtable, so its rollback branch is not selected by this implementation.
Finally it clears bit 30. The complete method does not read input's third
Float4 (+0x24); its broader purpose remains open. The second vector's
gameplay name and the two state-byte meanings also remain unassigned.

## Simple collision implementation

Simple vtable `0x14212d1c8` maps +0x448 to **`0x1410a7a10`**. It requires
flags bit 1, copies only the first vector/tag to a temporary, calls
`0x1410a5a30` on its +0x10 interface, then applies that tagged point through
`0x1410a8a90` and `0x1410a8d30(this,1)`. Those are the clamping and
ground/interpolation helpers documented for server 86. It performs no
map-unit conversion and does not consume the second/third vectors, state
bytes or tick in its complete body. These different consumer paths are
another reason to preserve explicit wire fields without inventing defaults.

## Typed framing scope

A typed server-87 frame can expose agent index, tick, command flags,
collision-position Float4, raw transform tag, two further Float4s and two
state bytes. It must retain all float bits and the incoming tag verbatim,
even where the client's outbound serializer canonicalizes that tag. Decoding
must consume exactly one frame and leave input unchanged on any error.
This API supplies no valid state, component defaults or collision simulation.

Three focused typed-frame tests and the library build pass. Literal 59-byte
and 65-byte goldens cover distinct vectors, raw NaN payloads, negative zero,
infinity, subnormal bits and unsigned maxima; a 57-byte zero frame checks the
minimum length. Prefix/concatenation checks preserve adjacent input. Every
truncated golden, wrong message ID and overflowing tick/tag varint fails
without advancing the caller's slice. These are transport checks only; no
frame was sent to the client.

## Evidence

Private bundle: `~/re/gw2/notes/209026-float4-position-state-1.json`.
(1,383 records, 27,923,579 bytes).

The capture census now supports the verified Float4 primitive. Seven focused
tests and its binary build pass. Private extracted schema
`data/protocol/209026/agent-position-state.xml` is 344 bytes, SHA-256
`a43ef0abe7b7d2eedb2b387267c328e192c20ddfaefcffd49338a799010be70f`.
The CLI smoke parses two handwritten framing fixtures as 59 bytes each,
118 bytes total. No fixture is a validated game state or was sent to the client.
Extraction evidence: `ev_e496df81dc2189305e936f9e63c88f33a7005fe4f21de9132d4bf36a2977a07f`;
CLI evidence: `ev_f280447d66cf9476e44511503070238db35eac7a37af259cca8356854a5ac227`,
in private bundle `~/re/gw2/notes/209026-float4-position-state-2.json`.
An earlier Rizin external-command form returned empty output; only the later
`!env` invocation supplies the successful CLI result.

| REA evidence | Observation |
| --- | --- |
| `ev_33a8520c36809f93486a07e0c69e4af64219f139600cba35f3c360aeb87336f0` | Registered 86/87 descriptors. |
| `ev_d33003cfe4a02abf0eaf3d948d359b0709476264602257e169916be0338acdc4` / `ev_def3059c98f3fcfcadb807a77a10b6a2a7400f6bc264691f60888c1b444287ca` | Handler and complete constructor. |
| `ev_2202c1ad4b0234b70a33a237860036d1074918356ff12ff7f7636910a73cc899` | Full/simple +0x448 and scene predicate entries. |
| `ev_3f9ed3b7d7bd2e0b5df80e0bf2a58496686d5704b948577875e11905a529d6f8` / `ev_2c4c5293fe6fc72993391ec765806112178d7928476feb151813c3b68df2e501` | Complete full/simple state application bodies. |
| `ev_a32bce38e6a5043da6da47e4e166e3f69637589e941ecadaa2eaeb6a7651c9b8` / `ev_8d489f6a83c78d41ba68be93c960268d2a1803377cc792e5187c191faa2c35f9` | Four-component scene magnitude predicates. |
| `ev_3ab8ce13bf9de58046ced2f5efc9e24dca2be59f5fba575f7eece3d6b65fd94a` | Full distance hook maps to always-true stub. |
| `ev_8684d892cf91c8d0632eea8b8a57f6bf704d633b2e115426e34dd1dd35a92452` | Shared serializer and tag-bit rule. |
