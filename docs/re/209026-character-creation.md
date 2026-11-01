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

## Later result candidates

Server ID 1048 has two GUIDs, String(20), Word, two Qwords, and a u16-counted
Byte array with descriptor maximum 1024. Handler `0x141412840` passes the name,
first GUID, Qwords and byte array to `0x14023c700`, then invokes view slot +0x30.
The callee finds or adds a name-keyed record in the same character list used by
auth ID 17, copies the GUID and blob, and stores the Qwords. The inspected caller
does not pass the second GUID or Word to that callee. Its blob destination is
the existing record's 512-byte area; the descriptor maximum must not be treated
as evidence that a 1024-byte fixture is valid for this consumer.

Server ID 1051 has four Dwords; handler `0x1414128f0` takes their low Words
and emits event `0x10000002`. These are useful candidates for later capture,
but their ordering relative to request 415 and their success/error semantics
have not been established by runtime evidence.

## Evidence

Private REA bundles: `~/re/gw2/notes/209026-character-static-6.json` and
`~/re/gw2/notes/209026-mission-static-4.json`.

| REA evidence | Observation |
| --- | --- |
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
