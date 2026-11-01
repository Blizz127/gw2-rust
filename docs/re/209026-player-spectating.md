# Build 209026: player spectating fields

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This refines the
[player creation/state trace](209026-player-creation-fields.md). No client
was launched, no messages were sent and no Agreement action was taken.

## Direct consumer names

Player vtable `0x14215e9d8` has these entries:

| Virtual offset | Function | Return |
| --- | --- | --- |
| +0x100 | `0x140c341a0` | Qword at player +0xe0, the content record. |
| +0x108 | `0x1409a7210` | Qword at player +0xe8, the agent reference. |
| +0x110 | `0x14026f580` | Dword at player +0x100, the state. |

At `0x1402c2980`, a consumer obtains the selected player from context
virtual +0x70 and calls the first two getters. If both pointers are
non-null, it asserts `!(spectateTarget && spectateCam)` using literal
`0x141938d00`. The source literal at `0x141938a90` is
`Game/Ui/Scenes/Gameplay/GpDownedManager.cpp`.

Together with the setter's agent/content lookups, this identifies the
agent reference as the spectate target and the content record as the
spectate camera. The names are supported by a direct consumer assertion;
they do not come from the visual appearance of a UI icon.

## Protocol consequence

Server **459** setter `0x1411bc280` resolves its first Dword through
engine-agent lookup `0x141024060` (zero produces a null reference), stores
the result at +0xe8, then looks up its second Dword as content type
**0x14e / 334** and stores that record at +0xe0. Its third Dword becomes
the state at +0x100. Thus these first two fields identify a spectate
target and a spectate-camera content record respectively. A fixture
should not populate both with successfully resolved objects: the direct
consumer asserts that they are mutually exclusive.

Server **460** resets the state to 3 and clears both pointers. Server
**428**'s final low bit selects initial state 2 instead of 3 and inserts
input-binding selector 4, as previously traced. These operations belong
to the spectating state machinery. The exact names/meaning of each
numeric state, selector 4's binding membership and all legal transitions
are still unresolved. State 3 plus null pointers is the observed reset
state, not independent proof of a valid map bootstrap. There is still no
evidence requiring 459/460 for ordinary map entry.

## Supporting content consumers

Content type 334 is also iterated at `0x14096d44e` and
`0x14145c3b1`. The first path constructs a world-map UI object through
`0x140974540`, storing the record at object +0x80 and registering an
interface with the context returned by `0x14145c130`. The latter accessor
returns the global context's +0x210 member. Its adjacent assertion/source
literals identify `Game/PvpGame/Cli/PvpCliContext.h` and `.cpp`.

The second iteration reads a position through record +0x28, filters by
distance and performs a scene ray query with mask `0x3830`. These paths
support the camera-point interpretation. They do not prove that these
records are vista objectives or that their coordinates are spawn points.
The two UI resources at `0x141b855d8` / `0x141b855e0` are numeric values
`0x01081a55` / `0x01081a54`, not strings naming the content type.

## Evidence and limits

Private bundle: `~/re/gw2/notes/209026-player-spectating-1.json`
(942 records, 21,644,401 bytes). Static observations only. A broader
getter/consumer command timed out; its partial output was retained,
cleanup was checked, and the decisive getters were reread successfully
with a bounded command. No conclusion here requires that partial run.

| REA evidence | Observation |
| --- | --- |
| `ev_634cb4d6b82bff5ba3cd81fd68ff8d2671303226bad6e1032af395881b3e2452` | Complete getter instructions/vtable pointers and state setter stores. |
| `ev_917f7fe18b06504b3a5ef8241b90063c68f54f8ee30685b58635f2b4199fc90e` | Selected-player getter calls, mutual-exclusion assertion and source literal. |
| `ev_5ce37f2086e58fa373581e79206ebdf7a544d27175d7b2031ead50e8c5ef1deb` | Exact spectate assertion text and file offset. |
| `ev_e274a9841b144ce8e996d69eac2f161a2928ca12cd97dee51d46233866d9e153` | Type-334 iteration/lookup sites. |
| `ev_f7fa8854f6f8faa004a076de705e2eae065e06860de76c696a6b24e0e8f01d51` | World-map object construction and position/ray consumer. |
| `ev_94aef96f1c06b8902a48dd8438ac911e35279bce919b4696325fa882d2c2ec5d` | UI record storage and context registration. |
| `ev_352360fe46cdd36c50ef5d849b51891b1f19f6459031334f918d764a384587fa` | PvP context source literals. |
| `ev_44f3249b327b34ca17998bb783cc4de4a2f56cd4a0e6605f0e80871bfcc52629` | UI resource consumer and position getter. |
| `ev_1ec97f9d0a472b8fc7326e301bc3226853d29d015e35e4701a57286be99fcd46` | Numeric resource values. |
