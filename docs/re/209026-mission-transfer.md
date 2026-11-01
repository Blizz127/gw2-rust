# Build 209026 mission transfer and map-loading messages

Static REA findings from frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`,
2026-11-01. These extend the [mission join](209026-mission-join.md) and
[creation](209026-character-creation.md) paths. No mission runtime, world
load or successful creation is claimed. The User Agreement remains untouched.

## Server-supplied destination

Protocol 0, group 39 server **ID 1065**, descriptor `0x14260f510`, handler
`0x141412fa0`, has this layout after its two-byte ID:

| Field | Encoding | Inspected consumer |
| --- | --- | --- |
| 0 | Address, 28 bytes | Next destination; stored at context +0x190 if deferred. |
| 1 | Dword varint | Not read by the inspected handler. |
| 2 | Dword varint | Join parameter, stored at +0x188 if deferred. |
| 3 | Word, little-endian | Map-definition key, widened into +0x18c if deferred. |
| 4 | Dword varint | Join parameter, stored at +0x1ac if deferred. |
| 5 | Dword varint | Not read by the inspected handler. |

If context +0x150 mask 0x20 is clear, the handler calls `0x141417e30`, then
`0x141417b80` with redirect type 1, fields 2/3, the destination, field 4 and
a final zero. That routine stores the join parameters and starts
`0x14023c990`, the already mapped game-join transaction. It does not send an
auth route request on this inspected branch.

If mask 0x20 is set, it saves those values in context +0x184 through +0x1b0
and sets mask 0x10 instead. The event-0x25 join-completion branch at
`0x141417f97` clears masks 0x20/0x2 and calls the view's join-result handler.
At `0x141418026`, a saved mask 0x10 takes precedence over the ordinary
readiness continuation: it clears that mask and passes the saved destination
and parameters to `0x141417b80`. Only the other branch sets readiness mask
0x40 and calls `0x141418df0`.

The common join helper stores its map argument at context +0x15c.
`0x141418360` later uses that value with the same map-definition lookup
service/slot established for character summaries. This identifies ID 1065's
Word field as a map-definition key. It also identifies auth reply 20's field 3:
transaction +0x48 becomes completion result +0x10, which the event-0x23
handler forwards as the map argument at `0x141418123`.

This is a direct mission-to-mission redirect mechanism. It is a candidate
continuation after creation's empty client ID 430, but no observed exchange
establishes that relationship or the correct response sequence. The meanings
of the other join scalars and valid destination/map combinations remain open.

## Nearby messages are distinct paths

Server **ID 1054** has Word + Dword fields. Handler `0x141412a10` maps the
Word through `0x1412e19c0` and `0x1414162d0`, optionally forwards the nonzero
Dword to `0x14023d7e0`, and calls general route setup `0x141417d50`.
Its diagnostic explicitly names **OnErrorRedirect** and a map value.
This is not evidence of the ordinary successful transfer acknowledgement.

The map-loading handlers are separately identified by embedded diagnostics:

| Server ID | Established layout | Static behavior |
| --- | --- | --- |
| 1063 | Four Dwords, two Bytes | Stores map ID from field 1 masked with 0x3fffffff at context +0x1f0; looks up its map definition into +0x1e8; sets readiness mask 0x80. Field-1 high bits also set context flags 0x200/0x400. |
| 1060 | Float3, Float2, Float2, Word, Byte, Qword, Dword | Handler `0x141412da0` logs **OnRecvLoadMap** with the Word and context map ID, then forwards vectors and other parameters to `0x141418ee0`. See the load callback findings below. |
| 1061 | Empty | Handler `0x141412d60` calls `0x141418ec0` and logs **OnRecvLoadAgents** with context map ID. |

These handlers establish message roles and some field flow, not a complete
map bootstrap. ID 1063's readiness branch differs from creation's ID 1049
because it does not set mask 0x1. No map/agent population fixture is inferred
from the empty ID-1061 message alone.

## Load command and result callbacks

The later [map-readiness analysis](209026-map-ready.md) establishes that
MapLoaded precedes READY and identifies the separate client ID-412
AgentsLoaded signal. Do not infer readiness for BeginPlaying from ID 420 alone.

For a nonzero redirect type, the shared join helper `0x141417b80` first
calls view slot +0x188 with the route's map-definition key, before constructing
the binary game-join transaction. That slot resolves to `0x14094e620`, which
switches the view to state 0xd and calls `0x140953670` on its embedded loader.
The latter logs **MapLoader: BeginMap**, resolves the key through content
slot +0x1a8, stores the definition at loader +0x2b8, requires it to be nonnull,
and sets loader state 1. Redirect type zero uses the separate creation slot
+0x148 instead. This establishes where the prepared definition originates.

ID 1060's handler converts its Byte to `2 * value * pi / 255` using float
constants at `0x14193fc84` and `0x14192bb9c`. This establishes a full-turn
angular scale; the world axis and orientation convention remain unknown.
The Float3 is passed separately, while the two Float2 values are copied as
16 consecutive bytes into context +0x1d8. The Word is used by the inspected
diagnostic but not forwarded to the load helper. The final Dword goes to
`0x1410b5900`; its meaning is still unresolved.

`0x141418ee0` obtains the resource pointer at map definition +0x68 (or an
embedded fallback if absent) and calls view slot +0x190. The view interface's
vtable is `0x141b7e818`; this slot resolves to `0x14094e6c0`, which forwards
the resource, map definition, vectors, angle, Qword and callback to
`0x1409538c0` at its embedded loader. The loader stores the Float3 at +0x2a8,
angle at +0x2b4, Qword at +0x318 and callback at +0x210. Physical roles of the
vector and Qword remain unresolved.

Before accepting these parameters, the loader asserts its state is one of
`STATE_LOAD_CONTENT`, `STATE_LOAD_MANIFEST`, or `STATE_SERVER_WAIT`
(numeric states 1, 2 and 3 respectively, mapped in the later readiness notes), and
`m_mapDef == mapDef`. Thus this command expects a matching prepared load;
it is not established as a standalone map-bootstrap message.

The ID-1060 path selects callback `0x141418fb0`. A nonzero callback result
sends empty client **ID 420** through `0x141418840` when a mission connection
exists, then logs **MsgSendMapLoaded**. A zero result forwards to
`0x141419010`, which sends empty client **ID 418** through `0x141418810`
and logs **MsgSendFailedToLoad** after its resource handling. The alternate
callback `0x141419010` does nothing for nonzero results. These are static
completion paths; neither message has been observed from a live map load.

Private boundary fixtures additionally parse ID 1060 + ID 1061 as
**42 + 2 = 44 bytes**, and IDs 420/418 as **2 + 2 = 4 bytes**, with no
remainder. The vectors and scalars are handwritten placeholders. These
mode-0600 fixtures were never sent and do not represent an established
message sequence or valid map content.

Two adjacent messages must not be mistaken for substitutes for content
loading. ID 1061's view callback at slot +0x198 resolves to `0x140163990`,
a bare return, in this default view. Its inspected handler then logs
OnRecvLoadAgents and returns; it does not itself create agents. Empty server
ID 1036 calls view slot +0x170 (`0x14094dfd0`), which explicitly asserts
`m_mapLoader.State() == CMapLoader::STATE_READY` (loader state 0xd), switches
the view to state 0x10 and invokes game-system callbacks. That assertion
means ID 1036 cannot be treated as proof that earlier loading completed.

The ordinary readiness branch also differs from creation: when context
+0x150 has both masks 0x40/0x80 and lacks mask 0x1, `0x141418df0` retrieves
text/audio language preferences, checks `LanguageIsPermitted`, then calls
`0x141418bb0`. It sends client **ID 423**, two Bytes in text-then-audio order,
with named upper-bound assertions `< LANGUAGES` (6). Language enum labels
are not mapped here. The mask-0x1 branch remains creation's empty ID 410.
These dependencies are static; a complete successful runtime order is still
unverified.

Load callback evidence is exported separately in
`~/re/gw2/notes/209026-map-load-static-1.json`:

| REA evidence | Observation |
| --- | --- |
| `ev_8c911ab24364890e572d62d41587e17758b00051ccb052be52da1baf66d3c91b` | Load helper, resource forwarding and success/failure callbacks. |
| `ev_d2d733e42156b2da160d4381d1e8a5ead30e2e90d0219abdda83944ecbcb928b` | ID-1060 fields, result senders and explicit result diagnostics. |
| `ev_bbfd8b08c260f0ad5f45553b90348ffb155eaf7d813dacafbffb74d650ccf25e` | View vtable slot and float constants. |
| `ev_4900a13621ebc86e2a06a4a8e532b40d2dd8b6a8832666c4eee65dc0d4111cf2` | View forwarding to embedded loader. |
| `ev_b161b3bb7781e754b9aad87db2da91bbd9acfb794648b3e863fea218fe88ffee` | Interface +8 construction, loader state/definition guards and parameter storage. |
| `ev_a6c13ee541b2de04a5aaeb178f3b4b9a4f0b33f6aae533b1852323b3a71e777b` | Both fixture censuses and named loader assertions. |

The preparation extension is exported in
`~/re/gw2/notes/209026-map-prepare-static-1.json`:

| REA evidence | Observation |
| --- | --- |
| `ev_3a8901d6ec7732fec73afaed7f5245fa339541ee5f74d66f40157843530e7cb3` | Ordinary readiness branch, LoadAgents view slot and begin-playing guard. |
| `ev_ea8d980df947dcbe42a9a1b63b65cf3e0e8e071fdf771c4b3df3df145601204e` | View preparation, ID-423 sender, STATE_READY assertion and bare-return callback. |
| `ev_4ec6d9e3879a472b685c34931c0ef61f01c3fd828c9c36f785ccad9b14ef50d1` | Join-to-view call, prepared map-definition storage and language assertions. |
| `ev_3651de70b6ea102950ddec91148374a90825f439d59be5aa82d4afe728c377b4` | Complete join argument flow, BeginMap diagnostic and ID-1061 handler. |

## Boundary check and evidence

Private `~/re/gw2/notes/mission-redirect-server.fixture` contains a handwritten
ID-1065 frame with a loopback address and a multi-byte Dword, followed by empty
ID 1036. The capture census parses **37 + 2 = 39 bytes** with no remainder.
This is deliberately a boundary fixture, not a captured or valid transfer
sequence; it was never sent to the client. It is stored mode 0600.

Private REA bundle: `~/re/gw2/notes/209026-mission-transfer-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_629f72a57940f1237914b2a087a1054f86e5e2a65ece0f5d0e7892cc894afb14` | Registered descriptors/handlers and named load-map/load-agents diagnostics. |
| `ev_2d2b5314d5adeaf1cd5b4a7e64a7a75a0f3d8bfcd85c3c69d9edca5903471ebc` | Route-result argument flow, map-loading handlers and ID-1065 branches. |
| `ev_2e3a38778eb3a90d5eb1438e529c4f1ae5cfcc5c589c5c7548a2278cae13134a` | Deferred destination stores and matching reads. |
| `ev_fdb9722c8f4bb7a5157ec246fa89cf4621a89cefb8c5ccee414db8ddbacbe76d` | Join-completion deferred branch and immediate-path helper. |
| `ev_799e554598affb373767291a68bea738fad4a49db6e17bf23ddbbd3d522923db` | Common join helper stores map argument at +0x15c. |
| `ev_03e8273a72bd92a0c5f6b38c3894cbf298105440eda37deff8aac2f64454e842` | Context +0x15c used for map-definition lookup. |
| `ev_902ce4d00a3134c20f4331325fd5895ec4abaa0dc19d09b3041f4a112c5eb2e4` | ID-1054 error-redirect argument flow. |
| `ev_980a7414831eb5746a231c31b9e8c6374ec71a6873b98bec074eec4b361a42a8` | Explicit OnErrorRedirect diagnostic. |
| `ev_ae94e82ac272f15caa291bce4a1bb44df0d52e727f8f60677ea97133f6a6bb8a` | Synthetic redirect fixture's complete census. |
