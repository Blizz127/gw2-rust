# Build 209026 character-creation request

Static REA analysis, 2026-11-01, of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This establishes a creation-context send path and its descriptor. No character
has been created, no request has been captured at runtime, and no response or
persistence behavior is yet established. The User Agreement remains untouched.

## Connection and registration

The creation context at `0x14129cc20` validates its data with `0x1412a0440`,
then calls `0x141410d70`, which emits event `0x10000001` and forwards to
`0x141418710`. That sender constructs message `0x19f` (415), obtains the mission
connection from `0x14023f640`, and calls serializer `0x140ff0110` with a
41-byte decoded structure. The mission connection is constructed with protocol
number 0 at `0x14023f41e`; this is separate from protocol-3 authentication.

Registration `0x14021edc0` passes protocol 0, group 39, 41 client definitions
at `0x1422ddaa0`, and 48 server definitions at `0x1422ddbf0` to `0x140ff3d30`.
Client table index 4 points to `0x14260cfc0`, whose root identifies message 415.
Thus the descriptor is connected to the sender's protocol, rather than merely
being an unreferenced matching message number.

## Wire layout

After the two-byte little-endian message ID, the descriptor encodes:

| Order | Encoding | Source or established meaning |
| --- | --- | --- |
| 1 | String, limit 20 UTF-16 code units including terminator | Name at context data +0x78; validation accepts 1–19 units before NUL. |
| 2 | Fixed array of 48 Bytes; no count prefix | Context data base, containing appearance data; detailed field meanings unresolved. |
| 3 | Byte | Context data +0xa0; validator/sender check value below 11; meaning unresolved. |
| 4 | Byte | Level from context data +0x30; context validates 1–80, sender asserts below 255. |
| 5 | Dword varint | Flags: context supplies 0 or 8 according to a virtual predicate. |
| 6 | One-byte count, then Byte elements; maximum 32 | Context data +0x38 array; sender assertion names `madLibAnswerIdsBytes`. |
| 7 | Fixed array of 24 Dword varints; no count prefix | A 96-byte decoded block copied from context object +0x480; element meanings unresolved. |

Descriptor kind 16 is a fixed array; kind 17 has a one-byte element count.
Both recurse through the nested element descriptor. The Byte element descriptor
is `0x1425acd60`; the Dword element descriptor is `0x1425ace00`. Dwords use the
existing variable-length integer encoder. In particular, the last array's
96-byte decoded representation is **not** a fixed 96-byte wire payload.

This evidence permits the capture census to admit one level of `ArrayFixed`
with an explicit count and verified primitive elements, subject to its existing
65,536-element limit. Nested and implicit TypeName elements remain rejected.

The sender's packed decoded offsets, relative to its message ID, are:
name pointer +2, appearance pointer +0xa, two Bytes +0x12/+0x13, flags
+0x14, answer count +0x18, answer pointer +0x19, final array pointer +0x21.
These offsets describe local serializer input, not on-wire pointer values.

## Limits and next dependency

The nearby context path `0x14129ec50` ends at `0x140163990`, a return-zero
stub, and is not evidence of another active network request. Likewise, an
auth-protocol message containing a name and buffer must not be labeled character
creation solely from that shape. The creation-context path above provides the
connection and registration evidence.

The exact UI event leading to this context call and successful creation remain
unverified. The startup exchange below is mapped statically; no runtime fixture
or server handler was changed on this evidence alone.

## Creation-scene startup exchange

After the [mode-1 route and game join](209026-mission-join.md), mission-context
flags at +0x150 coordinate two readiness inputs:

1. Successful game-join completion (event 0x25) reaches `0x141418077`, sets
   bit 0x40, and calls `0x141418df0`.
2. Empty server ID 1049, descriptor `0x14260eb40`, handler `0x141412890`,
   sets bits 0x81 and calls the same routine.
3. `0x141418df0` requires both bits 0xc0. With bit 1 set, it calls
   `0x141418500`, which sends empty client ID 410 (0x19a).
4. Empty server ID 1050, descriptor `0x14260eb90`, handler `0x1414128c0`,
   sends empty client ID 411 (0x19b), then calls view-interface slot +0x140.

This allows either readiness input to arrive first. It does not prove duplicate
messages are idempotent or that these are the only server messages required.
In the ordinary map-data path, ID 1063 sets bit 0x80 without bit 1, causing the
same routine to take a different branch. The creation path has distinct startup
behavior from that map-data path.

View slot +0x140 resolves to `0x14094da70`. It sets local view flag 0x10 or,
if flag 8 is already present, calls view slot +0xc0 with state 0. Its counterpart
at slot +0x150, `0x14094daa0`, sets flag 8 or makes the same transition when
flag 0x10 is present. The character-select path calls this counterpart after
starting the route. Slot +0xc0 is the state setter `0x14094f810`; the enum table
at `0x141b7db70` explicitly names state 0 **char_create**, state 3
**char_create_load**, and state 4 **char_select**. This establishes the scene
transition's target, not that its rendering/assets have worked at runtime.

## Result handlers and next request

Server ID 1048 has two GUIDs, String(20), Word, two Qwords, and a u16-counted
Byte array with descriptor maximum 1024. Handler `0x141412840` passes the name,
first GUID, Qwords and byte array to `0x14023c700`, then invokes view slot +0x30.
The callee finds or adds a name-keyed record in the same character list used by
auth ID 17, copies the GUID and blob, and stores the Qwords. The inspected caller
does not pass the second GUID or Word to that callee. Its blob destination is
the existing record's 512-byte area; the descriptor maximum must not be treated
as evidence that a 1024-byte fixture is valid for this consumer.

The view slot resolves to `0x14094b090`. It returns without further action in
states 1/2 (char_create_cinema and char_create_cinema_load); otherwise it calls
slot +0x48, `0x14094b030`. That routine obtains the creation-context name and
passes it with view +0x7a0 to slot +0x60, `0x14094b180`, then clears +0x7a0.
The latter emits event `0x10000007`, chooses a scalar from the supplied value,
setting 0xbb or `0x14023b9c0`, calls `0x14023d7e0` with it, then starts the
name-based transaction at `0x14023c5f0`. The scalar's meaning is unresolved.

The name transaction has type `0x80000015`, vtable `0x141923798`, and a
20-unit name buffer at +0x40. Its send method `0x14023e460` calls
`0x140241b10`, which sends **auth client ID 22**: request-ID Dword followed
by the name string. Completion `0x14023e070` copies its GUID/name to globals
on zero error and emits event 0x5d with a 96-byte result. This is evidence of
a further auth transaction after the character-list update, not a direct
return to character select or a completed world entry. The reply and following
route are partly mapped below.

Auth server **ID 3**, descriptor `0x1425b0a50`, supplies Dword request ID,
Dword scalar, Address, then four Dwords whose low Words form the result code.
Handler `0x140240750` calls `0x14023add0`, which finds the matching request
at transaction +0x30 and requires state 3 at +0x20. It invokes virtual slot
+0x20 with the address/scalar, sets state 1, and copies the four-Word result
to +0x24. For the name transaction, slot +0x20 is `0x14023e2d0`: it copies
the 28-byte address to +0x78 and scalar to +0x94. Unlike the generic completion
reply, this path populates the destination before completion event 0x5d.
The inspected constructor zeroes the GUID at +0x68, and this callback does not
fill it; the meaning/use of that zero GUID in subsequent code remains unresolved.
No additional reply or completed world join is claimed here.

Completion event 0x5d is dispatched by `0x140950d40`: the byte table at
`0x140950f78` maps index `0x5d - 0x16` to 6, whose target is `0x140950eba`.
On zero error it looks up the character record by the result's name through
`0x14023c3c0`; on error it zeroes the local 0x258-byte record. It then calls
`0x14094db50` with the result, GUID, name and the looked-up summary length/blob.

That routine emits success event `0x10000009`, copies the result GUID to mission
context +0x20 and name to +0x30, and normalizes the summary into +0x58 through
the existing character-blob validator. Failure emits event `0x10000008` instead.
On success, its continuation tests mission-context +0x228 bit 1 (the connected
flag) and view state 0–3. If both hold, `0x141410f30` asserts context mode +0x1b4
is zero and the GUID is nonzero, then sends empty mission client **ID 430**
through `0x1414188d0`. Otherwise `0x14094f4d0` chooses content/map parameters
and calls `0x141410e80`, the general auth route-request path already mapped in
the mission-join notes. The ID-3 destination address is not directly consumed
by these inspected branches. Their exact runtime selection after creation,
including how the GUID becomes nonzero, remains unresolved.

Server ID 1051 has four Dwords; handler `0x1414128f0` takes their low Words
and emits event `0x10000002`. The creation scene subscribes to that event and
`0x10000001` at `0x14029d7da`. Event 1 disables form controls; event 2 reaches
`0x14029bae5`, passes the four-Word result and current name to `0x1402eeac0`,
then reenables controls 0xe/0xf. The callee chooses localized error text
(special handling for first-Word codes 0x19/0x1d/0x1f), formats the result
through `0x1402ee890`, and constructs a dialog via `0x1410708c0`.
Thus ID 1051 feeds the **creation error UI**; it is not a demonstrated success
acknowledgement. Exact localized meanings, runtime ordering relative to request
415, and successful creation remain unverified.

These paths are static findings only. The User Agreement decision remains
pending; no UI acceptance, creation fixture or runtime continuation was added.

## Private schema preparation

REA extraction from the frozen artifact generated ignored
`data/protocol/209026/mission.xml`: protocol 0 groups 39 and 40, 43 client
and 54 server definitions, 12,104 bytes, SHA-256
`aa140ba13e1107d9411c227f5320fdd9ee4cd305beca3906eb0b0ba7ff6fb44d`.
The extractor checks the executable hash and writes exclusively with mode 0600.
That initial extraction retained explicit `UnverifiedKind` tags for kinds 6,
7, 8 and 14. The [subsequent primitive check](209026-primitive-fields.md)
establishes kinds 6–8 and terminated-byte kind 14 and records the updated schema
hashes. This is a partial extraction, not the full mission message registry.

The schema parser accepts all 97 definitions. With the capture census, local
handwritten fixtures parse server IDs 1049/1050 as two 2-byte frames, client
IDs 410/411 likewise, and client ID 415 as one 86-byte frame including a
multi-byte Dword in its final fixed array. The raw fixtures remain private in
`~/re/gw2/notes/mission-*.fixture`. These are boundary checks; the creation
fixture's zero appearance block is not validated character data and was never
sent to the client.

Evidence: `ev_327f5b54412d9d00b8209f927420c279eaeab8de5fc4c054b8933f5a5d8e4473`
(extraction metadata),
`ev_f30bc66b6f7e85ed0e25ed328d270a190255da1be74db09b6b1fc10921136db0`
(schema/fixture census), exported privately in `209026-mission-static-5.json`.

## Evidence

Private REA bundles: `~/re/gw2/notes/209026-character-static-6.json` and
`~/re/gw2/notes/209026-mission-static-9.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_09e232402b8a05219e98e2ad793867b579fe3c93208808b37709215f35b0de3f` | Event-0x5d jump-table target and summary lookup. |
| `ev_d7c2fb6ab5e99ff4468933a5e53124f904539e3c7377502ec21c5fa8fc9a7252` | Success/error view handler and name-keyed record lookup. |
| `ev_f28d1f111fc74ae907d8417864a4f63cd0283754d60ada3c22bf6578bf83eeb1` | Context GUID/name/blob writes and continuation assertions. |
| `ev_68b3ead132e5a61a3981f51709a52d54380cf5af16d9a6795784f58873c39d4e` | Empty mission ID-430 sender; ID-18 separate-list consumer. |
| `ev_f8cba293563c4e1f39e149922c827a1a8287cbcb273f00cc0de0820ae725ceb1` | Connected-flag predicate and fallback to general route request. |
| `ev_35fdff9c75b433cd3a66f5ee398b84fc49287908bc440ac0a999949f14e00f97` | Auth server descriptor/handler registration table. |
| `ev_e29a621ac3cbca0f315653e262e655bf1400d9d14f7c4ff08985078ae38bf341` | ID-3 argument unpacking. |
| `ev_115e86141cb6171624bfeb21b25970739e8efbcbf6bbefd08151a6f110fb95d2` | Request/state guard and address-result completion callback. |
| `ev_b55d2d87ffa3b11bd5d99f9e105e029d0f5aa2d927e6d0336613903bdb4ac697` | ID-3 descriptor and name-transaction address/scalar storage. |
| `ev_9a473d0cbded76ac95369395f9c77b8df6fbec7b1976ad0b569f1d5867b4ef5b` | Creation scene event subscription and control branches. |
| `ev_e0e5e4f17e05a938f9a4289384799face8a139e3407067af8ce16dd2057f98d3` | Error event feeds localized error dialog and restores controls. |
| `ev_4e0a0e9362b2c28403e750448ec283347e7e4a459d9f057533bf26d2931cdd12` | ID-1048 view callback and name handoff; error dialog formatter. |
| `ev_d9c7619b92b694a5e30d1fac6c2093c9d03574c26c68544698b82945df86958d` | View slot +0x60 starts scalar and name transactions. |
| `ev_1bed53de7a1f8755fceb6c700c7ee7700011983c9b36c2872054ee6a68a62bfe` | Name transaction constructor and type/vtable. |
| `ev_4816c87a4f94140aa735cef266456512d790d982f1dfe22444a0712b37f996ba` | Transaction vtable and shared state machine. |
| `ev_c3310a4510db035a33f2fb45050401a1a0862c95a683dfdcb504a405dd59cf78` | Send/completion methods, selected GUID/name globals, event 0x5d. |
| `ev_e0837fc840535bd064a4c28aa1fec7fd589569ebbdcac9ec7c4b77702bc932c4` | Auth ID-22 serialization. |
| `ev_b79b1e42d90496e1944a1f9123b4a3fc27dad888d1cf8968db1ceef1e52744ea` | All group-39 descriptor IDs, fields and handlers. |
| `ev_bf7245d05b6961418c418309211243fac4c67d2c4ce852b208a3c6b2289f7ea9` | Post-join readiness test and server registration table. |
| `ev_08a08823c58ceb53c8dd62c2b9b89cfc8dc8c3bffe6dd90c26967cc6ee6534e3` | Empty client 410/411 senders. |
| `ev_2f03c95ef5fb9c170d3fc763823a3a11e38803c8de49aefdc00438f18951adfa` | Readiness flag writers and shared gate. |
| `ev_78b61a88b78cd138e1642d64a74bd532e603eaf3ee1917a14e23b2ac592e8b56` | IDs 1048–1051 handlers. |
| `ev_e41ef8ac1935dcc5f2379ced7bbc144345d16223706d7990ec8802e6b1c7e402` | ID-1048 record storage and view-slot resolution. |
| `ev_641a36866ccb21638e7ee3f2937abadd97d6168b2693e2fd08303d245a4a8229` | Paired view flags, empty server descriptors and character-list insertion. |
| `ev_dc14ef0e213cdb16aa715924475ab526ea0af37c4be72233024f4ddc13ec872d` | View state setter. |
| `ev_c0eef70cef69438a5cc697fe70f8e578d046b11a03a96f322137b3f3e048b1a3` | View enum initialization. |
| `ev_c177189d6bc09ec157d2163bd97075a7d6b490fe4acfa8921a514cccb0f0cdb9` | Enum string/value table. |
| `ev_d090da527c3aad87aa4d50c70b89ef3afded301471ac0135c7bcd7cbefc464e3` | Explicit scene names and distinct map-data readiness path. |
| `ev_6b3c558f0b94bfe8166fd339428bc4001646f3a7febe9c966dd11f4ef88be529` | Creation-context arguments and alternate stub path. |
| `ev_238f010d582ba3cdbad82fa10e17ccabcc9efa032c53a1e207ab116b23356d97` | Event bridge and return-zero stub. |
| `ev_228de915174f9a0fab1358fd0915c8b365efb705c6038f51e5ead2e8d6f19e8b` | Message 415 sender, mission getter, named assertions. |
| `ev_dab94c2ec2c7d0d2cd218526e65ac2a71a19049e4b4ef9ffebf4c7d4efeef7c8` | Mission connection constructed with protocol 0. |
| `ev_fff247464838a7d71fb2b5f3362139c24c97f8b1e12ca74e59dd55ea3de25a33` | Nested descriptors and protocol-group registration callers. |
| `ev_5bdd8aa9c9059fc9537b233b025f5ae86cd3527d7b04a5c610dfb5c3749bd499` | Group-39 registration and client table links message 415's descriptor. |
| `ev_991eae87e448c157ed0bb3046416c95f88043f4279ff42cac608b68ffe47f0f9` | Complete descriptor, primitive element descriptors and sender structure. |
