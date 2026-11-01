# Build 209026: server 86 position update

Static REA follow-up on the frozen executable identified in
[client-build.md](../client-build.md). No client replay occurred. The User
Agreement hold remains unchanged.

Server **86**, descriptor `0x1425b88b0`, handler `0x1410350f0`, contains
**Word agent index, Dword remote tick, Byte command flags, Point3**.
Point3 has three raw float words followed by one varint Dword. The constructor
`0x1410388c0` allocates 0xc0 bytes, sets kind **0x2b**, ORs the incoming
flag byte with **0x08032000**, installs vtable `0x14211cb88`, and clears
command +0xbc. It copies the decoded Point3 to +0xa8 and tick to +0xb8.
It also calls `0x141030750(world, &point)`: this forwards to world +8
interface virtual +0x2d8. A false result adds command flag **0x40000**.
That query's implementation and the resulting scheduling effect remain open.

The handler submits through `0x14102fcf0`. Command virtual +0x10,
`0x14103aac0`, requires a bound agent and calls bound-interface virtual
+0x238. As established in [server 84](209026-health-transitions.md), this
interface is whole base agent +0x18. Its vtable `0x14211aee0` maps +0x238
to **`0x14102b370`**. This method has no server-64-style active-command
+0xf0 prerequisite, but it does have the following gates and alternate paths:

- If `0x14102f700(world)` is true and bound-interface +0xd0 has mask
  0x4000, it returns without the completion call.
- Command flag 0x80000 selects `0x141030440` with command +0x70.
- Otherwise collision virtual +0x290 can skip the update. On the full
  collision vtable this is `0x14109ed50`, testing collision flags +0x58 bit 27.
- Nonzero command +0xbc selects collision virtual +0x448 with command
  +0x70 and tick +0xb8. The server-86 constructor sets +0xbc to zero.
- The zero branch calls collision virtual **+0x4a8** with command +0xa8
  and third argument **1**, then completes through `0x14103ea90`.

## Full collision position write

The full collision constructor installs vtable **`0x14212c908`**. Its
+0x4a8 entry is **`0x1410a3240`**, traced through its return at
`0x1410a35f7`. This establishes a position write on this implementation;
the simple collision implementation is described below. Common command
admission paths are not exhaustively traced in this follow-up.

1. Collision flags +0x58 bit 7 triggers an object +0x68 virtual +0x68
   query. A nonnull result returns early.
2. The point is converted to `[X/32, Y/32, -Z/32]`, retaining its tag,
   then resolved by `0x1410b7710`, as in
   [agent creation](209026-agent-position.md).
3. Bounds helper `0x1410b6a60` is called. If it returns nonzero, the method
   replaces the destination with the existing tagged point +0x120/+0x130
   and resolves that old point. **It does not use the clamped requested
   destination**, unlike the creation constructor.
4. Unless flags +0x58 bit 30 bypasses it, the method computes the distance
   from current position +0x140 and calls virtual +0x540. Zero would skip
   the write, but this full vtable's entry is **`0x1402406f0`**, which
   unconditionally returns **1**.
5. A flag at byte +0x5f bit 0 can trigger tag/frame conversion through
   object +0x80 when the selected tag differs from +0x450. A separate
   virtual +0x300 branch updates bookkeeping at +0x3a0/+0x3a4. Their
   gameplay meanings are not assigned here.
6. At `0x1410a3520`, the selected tagged point is written to both
   +0x30/+0x40 and +0x120/+0x130. The resolved vector is written to
   **+0x140** at `0x1410a3556`. Flags bit 31 also forwards it to
   `0x1417b9450` through object +0xf0.
7. The nonzero third argument copies global vector `0x14289e770` into
   +0x150 and conditionally forwards it to `0x1417b9490`. Its runtime
   contents were not captured; this note does not name that vector.
   It calls `0x1410a4c30(this, 1, third_arg)`, invokes secondary-interface
   +0x48, clears flags bit 30, and stores virtual +0x238's result at +0x3b4.

Command virtual +0x30 (`0x14103e0f0`) independently serializes the zero
+0xbc form as message 86 or 25 according to its direction argument, with
the same field copies. The nonzero form selects 87 or 26. This corroborates
the short position frame; it does not establish client-message semantics.

Server 86 is therefore a position-update candidate that does not share
[server 64's active-command requirement](209026-agent-point-adjustment.md).
No teleport, locomotion, flag default or valid replay sequence is inferred.
`gw2_proto::agent_position::AgentPosition209026` preserves explicit values
and float bits, with transactional decoding; it does not simulate collision
or assert acceptance by the client. Three focused tests pass: literal wire
goldens/concatenation, raw float bits/unsigned limits, and every truncated
prefix plus wrong IDs and overflowing tick/tag varints. The library build
also passes. No runtime replay or default fixture is added.

## Simple collision follow-up

Constructor `0x1410a54f0` installs primary vtable **`0x14212d1c8`**.
Its +0x290 predicate is the same flags-bit-27 getter as the full object;
its +0x4a8 update is **`0x1410a7c30`**. This entire method through
`0x1410a7d05` does not read the incoming third argument. It calls
`0x1410a5a30` on its +0x10 interface, then **`0x1410a67f0`**, which
clears flag masks 0x800/0x40000, releases the two backing allocations at
+0xf0 and +0x100 when their capacity markers allow it, clears their
counts, and zeros vectors +0x140/+0x150.

The method converts the incoming point by the same 1/32 and Z-negation
rule, preserving the tag. It calls **`0x1410a8a90`**, which copies that
tagged point to **+0x30/+0x40** at `0x1410a8c68`. When old/new transform
IDs differ, an earlier branch converts existing frame-dependent vectors
through scene helpers. It then resolves the newly stored point. If the
resolved XYZ is outside expanded scene **+0x50..+0x60**, it calls
**`0x1410b55a0`**: componentwise `min(scene_max, point)` followed by
`max(scene_min, result)`. It converts that clamped vector back into the
selected frame through `0x1410b70d0` and replaces +0x30. Thus this update
path clamps finite out-of-bounds destinations; it does not use the full
object's old-point fallback. NaN behavior is not described as a safe clamp.

Next it calls **`0x1410a8d30(this, 1)`**. This resolves the stored point
and calls **`0x1410a5fb0`** for a collision-adjusted destination. The
latter uses half the height at +0x17c for its vertical segment, water
state at +0x168, and query **`0x141099a50` with mask 0x7830**. Flag bit
49 bypasses that query; a miss also falls back to input plus half-height
along collision Z. A hit sets flag bit 50; miss/bypass leaves it clear.
The query's entire collision traversal is not reimplemented here.

The update helper computes differences against prior positions and scales
them by **25.0** (four float words at `0x14212d140`). It stores vectors
at +0x140/+0x150. It can zero those vectors and snap the corresponding
position when the squared magnitude exceeds float bits **0x4480147b**
at `0x14212d8bc` and its second argument is nonzero. Another comparison
uses componentwise absolute differences with **0.01** at `0x14212d100`.
The final resolved-position writes to **+0x130** occur at `0x1410a8f50`
or `0x1410a8f7f`; the latter also requires the +0x10 interface predicate
`0x1410c1140` to be false, count +0xf8 zero and flags bit 27 clear.
Other paths retain the difference vector for subsequent advancement.
This is evidence of conditional snapping/interpolation, not an unconditional
immediate copy of packet XYZ to +0x130.

The outer update finally calls `0x1410a8820` and asserts that +0x130 XYZ
lies within expanded scene bounds. This assertion follows the requested-point
clamp and collision adjustment. The full and simple implementations therefore
need distinct runtime checks before a world server can rely on matching
position outcomes. The transport codec requires no change.

Private follow-up bundle: `~/re/gw2/notes/209026-agent-position-simple-1.json`
(1,344 records, 27,400,823 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_e4c31c3d13237500ccf7ba0bcef6ab232812884bbaae29dd2c77f174f0e200ed` / `ev_304c994879be0cf4bf233c7573e1e2bff3ca4aca2e8b572b50f09890412ab7a9` | Simple constructor and vtable entries. |
| `ev_b90fb68e125a9c5324970ee2b7358fb516ef5b5656bcc9b207ffd22a67a2fbc7` | Complete simple position-update method. |
| `ev_63da78c1417ebcf8f4dfd03da8254dac6ff0e9a88146a06e4407bfc8e252d76a` / `ev_598ac06727ab05567d04ad78a5fcbde1e50e132e3f822fa91d50425f05d54f40` | Tagged-point helper, advancement helper and buffer reset. |
| `ev_5437019609c02fdbc52cb1f5998dd6e005d41e696f296b03deeca6ac1da9436b` | Complete expanded-bounds clamp and ground-query wrapper. |
| `ev_7cca4ebd44768726be4801db31a71cf969110f1430865f0a7b57e281091ca124` | Difference scale, threshold and component tolerance constants. |

Private bundle: `~/re/gw2/notes/209026-agent-position-update-1.json`
(1,337 records, 27,248,915 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_35163a3f78f6196abd9547ebb9b678bc21d728c6ce6cca9e92cf5375ba59aefd` | Registered Point3 descriptors, including 86. |
| `ev_49c600ec5c90ca5e286e3aeb81f4587512c91a0eff900b765f3ff580de3cf59b` / `ev_388452e8f3f3d955df2484bb6b3a629e0b4f383ffd105288dff7f220e66e65cb` | Handler and constructor. |
| `ev_aa5b27e048895c07ccd207396c29c0d86ac5b6e19141a3947413dcb683c22014` / `ev_7c6d56e72f6705e8fcc88b2298df3ce797ae2ae77f275f973851924d80dfe9dd` | Command vtable, executor and point-query wrapper. |
| `ev_cc68163bc5def336d490ffeb05ac9502ef593175a82356510f0268a61c809c69` / `ev_637ddf57db4fec46a92d776eb1f91a00d7e1b0184613ceeed78fa194d00266da` | Bound-agent vtable and complete update method. |
| `ev_0d47e036bce25f930a8be8e78dfa04d883a768e48d275ceab959fffc0b6fd54c` / `ev_31f6f20067d8edf820806968d6f3fe1be69983b842ba55e96d85498a8b662bf1` | Full collision vtable and update/predicate entries. |
| `ev_643d79c46d3411e6456516f49b22fabda0c33ec0ab26c156411f8b0706f2c7b6` / `ev_c1678867a305c5adabc9187d301f05d97e01fec8204e76153c87083fd578931f` / `ev_7451fdf53c7f109c3a83052e9fc3d74ce835c87c5016e7a2d1109749809e183a` | Complete full collision update and predicate. |
| `ev_e4bd10e17dba85bbf1f84712c29d535ccbbe22b7ef736f014081ab7645cda001` / `ev_8684d892cf91c8d0632eea8b8a57f6bf704d633b2e115426e34dd1dd35a92452` | +0x540 entry, always-true body and command serializer. The batch qword reader stopped on the unmapped global-vector address after emitting the vtable entry; no global-vector value was obtained. |
