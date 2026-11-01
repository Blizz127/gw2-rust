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
The two collision method implementations and active-kind gameplay names
remain untraced here; a teleport or ordinary locomotion name is not assigned.

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
