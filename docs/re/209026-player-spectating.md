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

## View removal and restoration on state events

A static follow-up resolves the AvContext player listener. Its character-
context listener +0x48 slot is `0x141110920`: when the selected player
changes, it unsubscribes AvContext +0x20 from the supplied old player's
virtual +0x428 and subscribes it to the current context +0x70 player
through virtual +0x420. That slot resolves to `0x1411bbb40`, inserting
into player +0x90. AvContext +0x20 uses vtable `0x142142658`.
The [control trace](209026-character-control.md) establishes the context
selected-player store and +0x48 notification order.

| Server operation | Player event | AvContext effect |
| --- | --- | --- |
| 459, `0x1411bc280` | After storing the supplied state, dispatch player +0x90 virtual +0x110 through `0x14098abfc` | `0x141111640` removes an existing controlled-character view through `0x14110f380`; it then selects the spectate target's view, or null. |
| 460, `0x1411bc350` | After restoring state 3 and clearing target/camera, dispatch player +0x90 virtual +0x118 through `0x140255578` | `0x141111730` applies the ordinary eligibility predicate and creates the controlled-character view if eligible and absent, then restores the selected-view pointer. |

Both callbacks assert that the supplied player matches context virtual
+0x70 and obtain its controlled character through player virtual +0x78.
That getter returns null unless the controlled character exists and is
finalized. The callbacks then pass it into helpers that dereference the
character without a null guard. These events therefore have a concrete
controlled/finalized-character requirement when this listener is installed.

The removal callback does not test the new numeric state before removing
the view. Supplying state 3 in server 459 is consequently **not equivalent
to server 460**, whose separate event restores the view. The reset's
eligibility predicate sees state 3, but still checks associated-player
and existing-view conditions described in the
[view-finalization trace](209026-character-view-finalization.md).
Neither event proves complete rendering readiness. Other listeners on
these notifications remain unexhausted, and no runtime test was performed.

Private follow-up bundle: `~/re/gw2/notes/209026-player-spectating-view-1.json`
(1,551 records; 30,324,287 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_b72d647c30b9c327cc63f6eddc505728ea6f5087a0e0df1f816423ca2ed392eb` | Complete state-set/reset bodies, state stores and event order. |
| `ev_0192b3ee7d59d8b5d48f23862c78e5555aa7145f6598bd6ad1e787674613591e` | Exact event thunks and complete removal callback. |
| `ev_88e1740f85b17a1e370ede2392b4c9588a9df3d356f37c939df4c603c2d24dd3` | Player listener +0x110/+0x118 and context subscription callback entries. |
| `ev_6965ea811f6d0e615b9eeffb4b38ab75bf75dcd4765f9ddda56e3c64cfc4096b` | Complete restoration callback and eligibility predicate. |
| `ev_14b50a962a0cf6e913b9b3fce08efd0f4d467f4cc7ac6cc23f8be97c07dca7e6` | Complete selected-player subscription switch. |
| `ev_f87b7f73c7198c2f4a55db780ce370e039d6ec0ea8de8f8275110b6206dc3c59` | Player +0x420 registration slot resolves to collection insertion. |
| `ev_d45fd2261dcee4a85581da00ff840f9d0cce2b40c360604a5cd8d5ed86d61c4b` | Player collection insertion body and character-dereferencing view lookup. |
| `ev_a3c848e8cb4700c92ddfc527c52c2cbb431b3552435b0a99c2d91ec867b61669` | Controlled-character getter enforces finalization. |

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
