# Build 209026: creation movement parameters

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This extends the eight-float array
in [creation dimensions](209026-agent-dimensions.md). No client replay or
new bootstrap defaults were introduced.

## Object offsets

The base constructor `0x141024aa0` installs vtable `0x14211a7a0` on the
whole object and `0x14211aee0` on its +0x18 interface. The update method
`0x1410258c0` is interface virtual +0x140 (slot `0x14211b020`). Its
`this+0x140` therefore means **whole object +0x158**, not array element 2.
All offsets below are whole-object offsets unless marked otherwise.

| Array index | Storage | Confirmed role |
| --- | --- | --- |
| 0 | +0x138 | Base movement speed; also initializes current/previous speed at +0x15c/+0x160. |
| 1 | +0x13c | Scale multiplying a turn command's scalar to produce yaw rate. |
| 2 | +0x140 | Separately stored scalar, exposed by getter `0x141027390`; semantic name still unresolved. |

## Speed

`0x14102dc90` updates current speed at +0x15c, retains the old value at
+0x160, sets a five-update counter at +0x164, and forwards the new speed
to collision virtual +0x4b8. Full implementation `0x1410a3660` multiplies
it by 1/32 and stores collision +0x2b8. Simple implementation
`0x1410a7d20` does the same at +0x18c and asserts **`speed >= 0`**
(literal `0x14212d888`). Their getters `0x14109e430` and `0x1410a6540`
multiply back by 32. Thus the agent-side speed uses the map distance
basis, while collision stores its corresponding distance basis.

This is a speed parameter, not a guarantee of actual displacement per
update. Full collision movement consumes +0x2b8; the simple path's
`0x1410a5b10` also compares it with a queued speed and uses the smaller
value when advancing toward waypoints. Movement state, elapsed-time
handling and constraints must still be respected.

The array-update command executor `0x14102ae00` treats index 0 specially:
it stores the base speed and usually calls `0x14102dc90`. It skips that
call when the active command's type byte +0x54 is 0x11 and its +0xd4
field is nonzero. Therefore base speed and current speed can differ.

## Turn scale and yaw rate

Executor `0x14102a480` computes:

```text
agent.current_yaw_rate (+0x158) = array[1] * command.scalar (+0x7c)
```

It can also install the command's facing vector. In update
`0x1410258c0`, the collision-facing getter is converted to an angle by
`0x141026690`; the next angle adds:

```text
(float(update_elapsed) / 1000.0f) * current_yaw_rate
```

The method constructs `[cos(angle), sin(angle), 0]` and sends it to
collision virtual +0x498. This establishes a radians-per-second yaw
rate when elapsed time is supplied in milliseconds. Array 1 is the
scale used to form that rate; the command scalar's range is not yet
established here.

Changing array index 1 through `0x14102ae00` rescales the existing rate
as `(old_rate / old_array1) * new_array1`, then stores the new parameter.
There is no zero-divisor guard in this observed branch. This is evidence
of client behavior, not validation that zero is a safe bootstrap value.
Another executor, `0x14102a520`, derives a turn rate from the angle
between facing vectors and a duration, so +0x158 is not always the
product above.

## Third scalar: bounded findings

Getter `0x141027390` returns array 2 unchanged. Its whole-object virtual
slot is +0x618. A call at `0x141431cdf` in the code bearing source path
`MvAdvancedMovementProxy.cpp` compares the result with 1. For a different
value, it selects one of two parameters and computes
`max(0, 1 + selected_parameter * (value - 1))`, multiplying three other
movement values by that result. The selected parameters and adjusted
values have not yet been named. This supports a scaling role but does
not establish a specific speed, size or acceleration meaning.

A direct call at `0x14155b086`, in code bearing `DecalGrid.cpp`, multiplies
the value by -1000, truncates it to an integer and passes
`-100000 - truncated_value` to `0x140a82670`. The purpose of that integer
is not established here. Index 2 takes the ordinary storage-only branch
in `0x14102ae00`; it does not directly update collision speed or yaw rate.

Setters `0x14102cc70`, `0x14102cbc0` and `0x14102cb10` queue index 0, 1
and 2 commands respectively through constructor `0x141037fb0`. Their
existence does not establish server message IDs for those updates.
Content-specific initial values and valid movement fixtures remain open.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-movement-parameters-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_2b37a47dcbc251a046092e2428dd47f3142514b330db60300b5e4ca2b2d4ef4d` | Turn executors, update arithmetic and indexed update. |
| `ev_06b3f3c87d69d0ce84bb94ebdc64d93bdbe332735a8739fa1bfbd522082f413b` | Update prologue, indexed setters, speed forwarding and getter references. |
| `ev_7b9bca3eea7bf5b34238c2db18a337101c6a1cf67b97b227ed2f46acfde21580` | Update vtable slot, collision setter slots and scalar getters. |
| `ev_a514e1c47822a44bdae5765a7d351f8dbbaecc797621b8d432f227df206cc225` | Constructor's +0x18 interface installation and both collision speed setters. |
| `ev_4ed7973ea1be136558acd0030609729990e759679de3514eaf99d3052a10bc1b` | Speed assertion, /32 constant, collision consumers and third-scalar call sites. |
| `ev_5a2134ddaf97ef0f7b5b0e15fde1f61fc01df139adbd30ea3d6a1114ff11114e` | Third-scalar arithmetic in movement and decal code. |
| `ev_a3d05c22e89b58e06bcd46bfecb5a0651bbeecf4f1e31d9facb3238a4e17b87b` | Decal source path, movement receiver setup and bounded agent-offset search. |
| `ev_adc91889bdc8ae655ebf9340d4a53f7e8863aa659e2433fb4931bd7ff448f796` | Movement source path, waypoint speed use and complete indexed-update/speed-forwarding methods. |
