# Build 209026: engine-agent creation and Point3

Static REA findings, 2026-11-01. Target identity is recorded in
[mission transfer](209026-mission-transfer.md). The client remains offline;
no population messages were sent and the User Agreement hold is unchanged.

## Server 37 creation path

`0x140205230` adds 23 client definitions at `0x14211bdb0` and 54 server
definition/handler pairs at `0x14211be70` to **protocol 0, group 0**.
Its final registration argument is 1; this is not the group ID. The
character messages described elsewhere also contribute to group 0.

Server **37**, descriptor `0x1425b3a00`, handler `0x1410335c0`, has:

| Field | Type | Confirmed use |
| --- | --- | --- |
| 0 | Word | Passed to the agent command queue and creation object's index field. |
| 1 | Dword | Passed as a separate queue argument; timing/sequence meaning unresolved. |
| 2 | Byte | Combined with fixed command flags. |
| 3 | Dword | Copied into the command's auxiliary-data object. |
| 4 | BufferVarSmall, max 250 | Copied into the auxiliary-data object. Raw descriptor parameter is 0x100fa; the established buffer length bound uses its low 16 bits. |
| 5 | Byte | Agent type, used to select the factory. Character factory is type 0. |
| 6 | Dword | Forwarded into base-agent initialization; meaning unresolved. |
| 7–9 | Byte each | Forwarded into creation parameters; meanings unresolved. |
| 10 | Point3 | Three raw float words plus a variable-length Dword; details below. |
| 11–12 | Word each | Interpreted as signed values for angular conversion, with 0x7fff as a special value. Full semantics unresolved. |
| 13 | ArrayFixed(8), Float | Eight raw float words copied into the creation command. |
| 14 | Float | Forwarded to the agent constructor. |

The handler allocates a command through `0x141035c50`, then submits it via
`0x14102fcf0`. The command has vtable `0x14211c338`; its +0x10 method is
`0x141039b10`, which allocates and constructs a base agent through
`0x141024aa0`. The constructor stores field 5 as agent +8, then uses that
value to find the registered factory. Its virtual call at `0x141024ef6`
reaches the type-0 character factory mapped in
[character population](209026-character-population.md).

This is a queued creation path. Queue timing, required auxiliary data,
content parameters and subsequent component initialization remain open.
The path alone does not establish a working map-entry sequence.

## Point3 wire representation

Descriptor kind **10** is three raw 32-bit float words followed by one
unsigned variable-length Dword. It is not just a Float3 or four fixed
floats. The last Dword's game meaning is not assigned here.

- Decoder switch `0x140ff237c` is indexed by kind minus one. Kind 10
  enters `0x140ff1dc1`, requires and copies 12 bytes, then falls through
  to the existing Dword decoder `0x140ff23f0`. Decoded storage is 16 bytes.
- Encoder switch `0x140ff178c` is indexed directly by kind. Kind 10 enters
  `0x140ff0a2a`, writes three raw 32-bit words, advances the source by 12,
  then calls the Dword encoder `0x140ff1800` for one value.

The existing Rust `Point3([f32; 3], u32)` codec matches that layout.
The census now admits Point3 on this build-specific evidence. Its focused
test passes with a hand-authored u32::MAX varint, signed zero/infinity/NaN
float bits, every frame truncation, overflowing fifth varint byte and an
immediately following frame. Encoding preserves the original point bytes.

## Offline validation

The focused census test passed (1/1), and the census binary built. The
hash-guarded extractor emitted only server 37 into the ignored private
`data/protocol/209026/agent-creation.xml` (mode 0600, 799 bytes, SHA-256
`75d3eba40cd30a119d71f554dd23a0860d5e6a851b93e50d3dba8b1f742c8a7f`).

Two hand-authored server-37 frames, with empty and 250-byte buffers, parsed
completely: 68 and 318 bytes, total 386 bytes. The CLI rejected a truncated
frame and a 251-byte buffer. The content/float choices are framing fixtures,
not valid world state; none was sent to the client.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-creation-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_f35c9c550a17cf57cebe4a3845ce9bc995f14dcaedb2d316b437bcc4f7720796` | Agent protocol/group registration and table. |
| `ev_f00e34108c04b73cc46118d0cb55baa9a31f6bccce1c81fdf965822065bbb0ff` | Hash-guarded extraction of 54 server definitions. |
| `ev_9e84e6177276fb8a5d1e101a00af386ab1022ef1d93e1b95936cdad0ab3e4bac` | Server 37 handler, command construction and base constructor. |
| `ev_9c5809ea841b810068a80d1137510ee40e562f89c91e10ee75e17c8886b1a8fb` | Command calls the base constructor. |
| `ev_b7fc10b0e149fb4fe91c8ea5ec8508691cce3ac76c6c910160058b0c072894ab` | Creation command's vtable and method pointer. |
| `ev_b3403cf9714b79d128bb21123982d856b94484bc9487cb29b81b0e26eca55d3d` | Factory lookup and invocation. |
| `ev_e8c9af498cd45dee40ceec5da59ae2ef8b8f8191b5d7928fc6f7c8d1f2d834ef` | Eight-float array's nested descriptor. |
| `ev_d686f518b2f7aa555c4825745d1825ddfd30a0a399918ac6e86a22aa58861484` | Point3 decoder and both switch tables. |
| `ev_246e148bdbe9aa4285713e5b3bf002e2da20ed05e3c8de373289893590580540` | Point3 encoder and decoded-size metadata. |
| `ev_128274762e5f6fecd3404179b210ca7d3bd916106f08d93a41f263a92e642bef` | Hash-guarded server-37 schema extraction. |
