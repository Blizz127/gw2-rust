# Build 209026: client 23 and server 84 behavior framing

Command serializer **`0x14103df90`** selects message **23** when its context
type argument `edx` equals 1 and message **84** otherwise. It emits the same
fields in both directions:

| Field | Wire representation |
| --- | --- |
| Agent index | Word, fixed little-endian |
| World tick | Dword varint |
| Command flags | Byte |
| Behavior value | Dword varint |
| Behavior data | BufferVarSmall, byte count followed by at most 250 raw bytes |

The client-23 descriptor is **`0x1425b8260`**, with BufferVarSmall raw
parameter `0x100fa`. It appears in both `agent-client.xml` and the
`ctable-0x14211bdb0` group of `client-tables.xml`. Server 84 is in
`server-tables.xml`, group `table-0x14211be70`.

The serializer gets the index through the bound agent, the tick from
world +0x1d0, flags from command +0x50, behavior value from +0x28, and
payload count from +0x2c, with the payload pointer obtained through
`0x141024700`. Command vtable `0x14211cb18` +0x30 points to this serializer.
These observations establish framing, not that an arbitrary client-23
payload is a skill cast or a health transition.

REA evidence supplied by the parallel source trace:

- `ev_7653a404eed112bfa3c2a1bfaa44525cb566734be199dd2ccb3abdcefc34c13e`:
  complete serializer and context-dependent message selection.
- `ev_91bd66a608a4c4e462a54e0a3a3c2d3af6af8889d95dc3438af7da8525d92e4a`:
  command vtable +0x30 binding.

`AgentBehaviorFrame209026` retains its existing `MESSAGE_ID=84`,
`encode_frame` and `decode_frame` APIs and server bytes. Explicit
`CLIENT_MESSAGE_ID=23`, `encode_client_frame` and `decode_client_frame`
add the other direction. The existing `remote_tick` field name is retained;
it carries the sender's world tick. Both directions preserve raw payloads
and reject the opposite message ID. Encoding oversized payloads and all
decoding failures leave the caller's buffer unchanged.

Validation: `cargo test -p gw2-proto agent_behavior --lib` passes all six
tests. Client golden bytes, concatenated frames, every truncation of the
golden frame, zero/250-byte payloads, 251-byte rejection, varint overflow,
wrong direction and schema agreement are covered. The existing server-84
and packed health-transition tests also pass. No observer integration or
release build is part of this change.
