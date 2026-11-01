# Build 209026: packed health transitions through server 84

Static REA of the frozen executable identified in [client-build](../client-build.md).
This complements [health component messages](209026-character-health-endurance.md):
256's state byte applies only on allocation, while later transitions use the
character's packed behavior-command dispatcher. No command was replayed.

## Transport and dispatch

The agent registration table contains server **84**, descriptor
`0x1425b8380`, handler `0x141034fb0`. Its fields after the frame ID are:
**Word agent index, Dword remote tick, Byte command flags, Dword behavior
value, BufferVarSmall behavior data (maximum 250)**. The last descriptor's
raw parameter is 0x100fa, using the already traced low-16-bit length bound.
The Dwords in this outer frame use normal protocol varints.

The handler allocates through `0x1410386c0` and submits via
`0x14102fcf0`. The command constructor uses kind 0x29, combines the packet
flag byte with 0x12000, installs vtable `0x14211cb18`, copies the behavior
value into its +0x28 container and copies exactly the decoded buffer bytes.
The existing [clock/scheduling rules](209026-agent-command-clock.md) apply;
this packet is not established as an immediate creation command.

Command virtual +8 is `0x14103e880`. On its accepted existing-agent path it
calls bound-agent-interface virtual +0x160. This is the base agent's
**+0x18 interface**, vtable `0x14211aee0`, rather than its primary vtable.
Its +0x160 method `0x141025e80` invokes command virtual +0x10. For server 84,
that is `0x14103aa40`, which requires a bound agent and forwards to that
interface's virtual +0x1b0, `0x141032d90`.

The latter passes command flags +0x50 and behavior container +0x28 to game
component virtual +0xc0. For a character's +0x48 interface, vtable
`0x142161670`, +0xc0 is `0x1411dcfa0`. It subtracts 0x48 from its receiver;
**flags mask 0x10 selects `0x1411dd4d0`**, the dispatcher traced below.
The bit comes from the packet flag byte and survives constructor flags.
This establishes a static route, not a complete set of usable packet flags,
queue timing, content values or callback prerequisites.

Server 42 has the same five outer field kinds, but uses command kind 9
and vtable `0x14211c418`. Its method `0x141039e20` calls bound-interface
+0x1a8 (`0x141032d10`), which forwards to component +0xb8. That character
vtable entry is the no-op `0x140163990`. It is not interchangeable with
server 84 for this traced character transition path.

## Packed payloads

`0x1411dd4d0` reads the behavior container's byte count at +4 and chooses
inline bytes at +8 for counts through 8, otherwise the pointer stored at
+8. It reads the first payload byte, subtracts 0x2d, bounds the result by
0x51, and uses selector table `0x1411de5c0` plus RVA table `0x1411de528`.
The verified entries are:

| Opcode byte | Target | Required payload bytes | Resulting stored health state |
| --- | --- | --- | --- |
| 0x33 | `0x1411dd8e3` | 13 | 1, dead. |
| 0x34 | `0x1411dd9c5` | 13 | 2, downed. |
| 0x49 | `0x1411de38c` | 3 | 0, alive. |

Each branch asserts the exact payload length before use. These packed
fields are direct little-endian memory reads, **not nested protocol
Dword varints**:

| Offset | 0x33 / 0x34 (13 bytes) | 0x49 (3 bytes) |
| --- | --- | --- |
| 0 | Opcode byte. | Opcode byte. |
| 1 | Raw u16 detail; dead passes all bits, downed passes bit 0 only to its event. | Raw byte detail passed to event. |
| 3 | Raw byte detail passed to event. | — |
| 4 | Raw u32 content key, resolved through content virtual +0x230. | — |
| 8 | Raw u32 agent identifier, resolved through `0x141024060`. | — |
| 12 | Raw byte normalized to nonzero/zero for event. | — |
| 2 | Part of the u16 above. | Raw byte normalized to nonzero/zero for event. |

No human names are assigned to the detail fields or content lookup result.
The two long payloads share layout, but have different state and event
semantics. The raw nonzero bytes are preserved rather than restricted to 1.

## State effects and limits

All three branches first query the corresponding state predicate and skip
the transition when already dead/downed/alive. On a change they retain the
old state, write character +0x188, dispatch the corresponding event and
call `0x1411e3ab0`, the state-update tail traced under
[finalization](209026-character-finalization-dependencies.md).

Dead/downed both call `0x141285f10` on embedded character +0x4b0. Death also
notifies the character context through `0x1411b61c0`. Downed calls
`0x1411f9a00` if a skill bar exists, then `0x141023cc0` with character +0x50
and argument 4. Alive clears character flags +0x178 mask 2 and rebuilds
an existing skill bar via `0x1411f9500` before its event. These calls are
not all exhausted here; existing rebuild prerequisites remain applicable.

The inspected branches do not change the health component's numerical
current/maximum/rate. State and meter updates remain separate operations.
The pointer lookups and subsequent event listeners can require meaningful
content/agent state; zeros are not asserted to be usable initial values.
No sequence here is a validated death/revive gameplay fixture.

## Evidence

Private bundle: `~/re/gw2/notes/209026-health-transition-1.json`
(1,315 records, 27,028,477 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_5ff1aa33ca58bc5d0c9fa0c77b237b6e9d7300e685e9465d6b4ad366e2fd13ea` | Bounded candidate state writes in the character code region. |
| `ev_a0e04544f85894507117439412bd3034c58ed78fe0c13978384129cd26f71b8a` | Dead/downed/alive branch reads, state stores and callbacks. |
| `ev_7bc547b5e50cb2968aaef3694984f08bcb1b290f0e3159dd8894f8936ac56d4c` | Flag-0x10 wrapper and remaining downed event body. |
| `ev_8ef32c80c7244f28d6393129393d6ee23dabc7e392e49c34526305f9b4daaa32` | Dispatcher entry, inline storage rule and selector indexing. |
| `ev_576bd5b57700fb9e17d1f6ea79fb7bb015df3224873c961844760bef1898a6c2` | Hash-bound resolution of all three opcode selectors. |
| `ev_4ad25987ef6aa3b937ed3260588fe23cfbc2925a9f3bdc8e48e173f5aa26a973` | Five-field agent descriptors, including 42 and 84. |
| `ev_fe85fb7c290b5e32c911b07fc3742284c7f678ada3b50d22acdf73c2253a11eb` | Packet constructors and submission argument forwarding. |
| `ev_d5c0a4e68842260992055cf3f59f7671c2e205da7c3eab2498da27efae31d26f` | Both command constructors, flags, vtables and behavior copies. |
| `ev_25fbce5d4c87d1739eccd0a0e894d3ab5949715465d993f6fb5c9ff9ce09a76f` / `ev_9ceab74264e6f7089d9e6099a0cbcbed368219b4130b20e3ca9c2c89be7b282d` | Command execution vtable entries. |
| `ev_0b32649b6a283e386b6ac48998053c38991a8811d49a34563e1776302029bd64` | Command-specific methods and common existing-agent execution. |
| `ev_0b62f9dfcc3b1ece06f3d76ed1134fa5c3989538e78f1a5634f8f312ac73635c` / `ev_430329b04ea99b1537022a09f6a46a86d2dd611a6ae4c71ae65668b7cfebd629` | Base-agent +0x18 interface and virtual targets. |
| `ev_4166254870142d9a788dee57933bbae1c6b67953c9dabac643d4e790bdbc523b` | Command-method invocation and server-42 component +0xb8 route. |
| `ev_fc2da0d3531020a4e9f88177228ea47ffd9e222b824f1df9eff15002cd49a40c` | Complete callback setup through component +0xc0. |
| `ev_8e9a9fead29a602c019bb6a24af3a482789a0fb014a2fdcf6a61662e5ee3f88f` | Character +0x48 interface's +0xc0 target and adjacent +0xb8 no-op. |

## Typed framing checkpoint

`gw2_proto::agent_behavior::AgentBehaviorFrame209026` encodes/decodes
server 84 with explicit header fields and a bounded raw payload. Decode is
transactional, and an oversized payload is rejected before encode writes
anything. The separate `HealthTransitionPayload209026` parser admits only
the three exact packed layouts above; it preserves the full detail Word
and raw trailing bytes even where an event consumes only a bit/boolean.
The transport codec does not require a health opcode or infer flag values.

Three focused tests pass, along with `cargo build -p gw2-proto --lib`.
Independent byte fixtures cover varint outer versus fixed-width inner
fields, noncanonical boolean bytes, two concatenated frames, preserved
output prefixes, every fixture truncation, unknown IDs/opcodes, extra
payload bytes, invalid varints and 0/250/251-byte bounds. These are local
framing checks only; neither health transitions nor a bootstrap were sent
to the client. The existing runtime hold remains unchanged.
