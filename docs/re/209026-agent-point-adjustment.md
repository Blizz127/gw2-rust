# Build 209026: server 64 adjusts an existing command's point

Static REA follow-up to [agent creation](209026-agent-creation.md), on the
same frozen executable. No runtime replay occurred.

Server **64**, descriptor `0x1425b6aa0`, handler `0x1410345c0`, contains
**Word agent index, Dword remote tick, Byte command flags, Point3, Byte**.
The Point3 keeps the existing outer protocol encoding: three raw floats
and one varint Dword. The final byte's gameplay name remains open.

The handler allocates an 0x88-byte command via `0x141037630`, then submits
through `0x14102fcf0`. Construction uses kind 0x18, ORs the incoming flag
byte with **0x110000**, installs vtable `0x14211c728`, stores the full
16-byte decoded Point3 at command +0x70 and zero-extends the last byte
into Dword +0x80. It does not carry behavior data.

Command virtual +0x10 is `0x14103a3f0`. It requires a bound agent and a
zero discriminator at bound-interface +0x2c, then subtracts 0x18 from that
interface pointer and calls `0x14102a690` with the whole base agent.
This interface distinction is the same one established for
[server 84 execution](209026-health-transitions.md).

The complete `0x14102a690` body establishes two gates: base-agent +0xf0
must be nonnull, and that active command must have **flags +0x50 mask
0x200000**. If either fails, the method returns without applying the point.
The base-agent constructor explicitly clears +0xf0 at `0x141024c07`.
The normal command start method `0x141025e80` stores active commands at
base-agent +0xf0/+0xf8 according to command flags, as traced in server 84's
execution evidence. This is not a general initial-position setter.

After those gates, the active command kind at +0x54 selects one of three
plain 16-byte copies:

| Active kind | Copy helper | Destination in active command |
| --- | --- | --- |
| 0x11 | `0x14103eae0` | +0xc0 |
| 0x12 | `0x14103eaf0` | +0xb0 |
| 0x1b | `0x14103eb50` | +0xb8 |

Other kinds skip the copy but still reach the following collision call.
For a nonzero incoming final byte, it calls collision interface virtual
+0x510 with the Point3 pointer and third argument 1. For zero, it calls
virtual +0x430 with the same Point3 and third argument equal to active
command flags bit 9. Both use the collision object at base-agent +0x50.
The collision implementations are traced below; active-kind gameplay names
remain unresolved. A teleport or ordinary locomotion name is not assigned.

This narrows future movement work: server 64 alone does not establish a
position update on a newly created idle agent. No live movement fixture
or additional codec is introduced from this limited trace.

Private bundle: `~/re/gw2/notes/209026-agent-point-adjustment-1.json`
(1,323 records, 27,095,271 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_4ad25987ef6aa3b937ed3260588fe23cfbc2925a9f3bdc8e48e173f5aa26a973` | Five-field registered descriptor for 64. |
| `ev_f772ef005e9924097df908d22ed9152f43a5a0889754a0b10243d839b351f0b3` | Full handler and submission arguments. |
| `ev_6bf5d7179a3a515df989e8a243cb8ec357776bbca15edf291a5610fb872ba063` | Constructor, command flags and field copies. |
| `ev_25256de90fed40fdfb76270a390c5d38e90f534821ca45c595152dc1dfade45a` | Command vtable. |
| `ev_918e5d89876f53b15283f7a8a5837dccf4acb2da5b318ca964615b78fa9651c2` / `ev_94aaabd843776528fa6848e968e10a08b7ca104032753f6873eee6dad470db78` | Execution checks and adjustment to whole base agent. |
| `ev_484ab7528795fad69969cf0c1924dbffee7b12492760036b794a3c9207f56c65` | Complete point-adjustment method. |
| `ev_52081fdeee75489466d79d686a5477445ff0fadc3f480fb3031818035e453b42` | Three active-command point-copy helpers. |
| `ev_13cf6fd255b5de1cb4ebb69e08a1576ae7cf3f87de6a92a6cf4279d21fccf1c1` | Constructor clears active-command pointers. |

## Collision-consumer follow-up

The two collision classes choose different implementations. The active-command
prerequisites above still apply before any of these calls.

| Class / vtable | Virtual +0x430 (last byte zero) | Virtual +0x510 (last byte nonzero) |
| --- | --- | --- |
| Full, `0x14212c908` | `0x14109f6c0` | `0x1410a44f0` |
| Simple, `0x14212d1c8` | `0x1410a70a0` | `0x1410a8810` |

### Zero final byte: store a later movement destination

The full implementation `0x14109f6c0` returns unless collision flags +0x58
have bit 1 set and bit 27 clear. Its third argument is a mask: bit 0
controls collision bit 11, bit 1 controls bits 12 and 58, and bit 2
controls bit 18. Server 64 passes only active-command flag bit 9 as a
zero/one value, so this route clears bits 12, 18 and 58 and sets/clears
bit 11. It also calls `0x1410a2660`, which synchronizes collision flag
bit 37 with a boolean update to the object at +0xf8 using other flags
and the squared XYZ magnitude at +0x220; that boolean's gameplay meaning
is not established here.

The method sets collision bit 15 and converts the supplied Point3 to
`[X/32, Y/32, -Z/32, 0]`, retaining its tag. It stores this tagged target
at **+0x250/+0x260**, resolves it through `0x1410b7710`, subtracts the
current resolved position +0x140, and writes a normalized direction at
**+0x170**. If the resulting distance is below the float constant 0.01
(`0x3c23d70a` at `0x141934804`), it substitutes the runtime global vector
at `0x14289e750`; that global's live contents were not captured. The method
then notifies the secondary interface +0x10 through virtual +0x48. This
complete body does **not directly write the current resolved position**;
helper/notification side effects are not exhaustively traced.

The simple implementation `0x1410a70a0` requires collision flag bit 1,
calls `0x1410a5a30(this + 0x10)`, and sets/clears bits 18 and 11 from
third-argument bits 2 and 0. It makes the same coordinate conversion and
**appends** the tagged target to the array at +0xf0 (32-byte stride,
count +0xf8, masked capacity +0xfc). It then resolves the current and
new tagged points into stack temporaries. The complete body does not
replace the current tagged/resolved position or invoke the simple snap
helper. Consuming this queue during subsequent updates remains untraced.

### Nonzero final byte: invoke the position setter

The full wrapper `0x1410a44f0` ORs **0x40100000** into collision flags
(bits 20 and 30), calls its virtual +0x4a8 with the supplied Point3 and
third argument 1, then notifies the secondary +0x10 interface via +0x48.
Thus it reaches the full `0x1410a3240` position setter described in
[server 86](209026-agent-position-update.md). Setting bit 30 bypasses
that setter's distance-hook branch; its other gates and bounds fallback
still apply. An early return can precede the setter's normal bit-30 clear.

The simple wrapper `0x1410a8810` is a direct jump to `0x1410a7c30`, the
same simple setter used by server 86. It therefore clears the queued
points and follows the documented bounds clamp, ground adjustment and
conditional snap/interpolation path. It does not set the full wrapper's
bits 20/30. This selector changes the collision operation; it does not
remove server 64's need for an eligible active command.

Private follow-up bundle: `~/re/gw2/notes/209026-agent-point-adjustment-2.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_dde1e167d11e61bfb7a1cf0723d1d63c05474c97e8e7b14b4182d4b82db541a9` | Complete full +0x430 destination body and +0x510 wrapper. |
| `ev_d99c0d737973b1e3bdc946b1cdafd25be5df169913b2b0c935d3deb78cefa27a` | Both vtable pairs, 1/32 conversion factor and 0.01 threshold bits. |
| `ev_f916a2e1399ee69a03dcdc908b82ca7e9a057319df8ba7c7c01f2905e03eec6a` | Complete simple destination body, direct simple setter jump and full flag helper. |

This remains a static trace. No client replay, movement fixture or inferred
initial movement flags were added.
