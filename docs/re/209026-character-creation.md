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

The exact UI event leading to this context call, the mission bootstrap needed
before it can send, the server reply, and successful creation remain unverified.
No runtime fixture or server handler was changed on this evidence alone.

## Evidence

Private REA bundle: `~/re/gw2/notes/209026-character-static-6.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_6b3c558f0b94bfe8166fd339428bc4001646f3a7febe9c966dd11f4ef88be529` | Creation-context arguments and alternate stub path. |
| `ev_238f010d582ba3cdbad82fa10e17ccabcc9efa032c53a1e207ab116b23356d97` | Event bridge and return-zero stub. |
| `ev_228de915174f9a0fab1358fd0915c8b365efb705c6038f51e5ead2e8d6f19e8b` | Message 415 sender, mission getter, named assertions. |
| `ev_dab94c2ec2c7d0d2cd218526e65ac2a71a19049e4b4ef9ffebf4c7d4efeef7c8` | Mission connection constructed with protocol 0. |
| `ev_fff247464838a7d71fb2b5f3362139c24c97f8b1e12ca74e59dd55ea3de25a33` | Nested descriptors and protocol-group registration callers. |
| `ev_5bdd8aa9c9059fc9537b233b025f5ae86cd3527d7b04a5c610dfb5c3749bd499` | Group-39 registration and client table links message 415's descriptor. |
| `ev_991eae87e448c157ed0bb3046416c95f88043f4279ff42cac608b68ffe47f0f9` | Complete descriptor, primitive element descriptors and sender structure. |
