# Reference message schema: build 208039 (NOT our build)

`protocols-208039.xml` is copied unchanged from valaphee/ascalon
(`protocol/protocols.xml`, commit `03fb60b`, MIT, Copyright (c) 2026 Valaphee).
It was captured at runtime from client build **208039** with a Frida agent.

Do not treat it as observed for the client build we freeze. Message ids and layouts
drift between builds. Our own schema will be regenerated for our build through REA
and saved next to this file as `protocols-<build>.xml`, with its evidence IDs
recorded in `docs/re/`.

Census (`gw2-schema-census`, 2026-10-31; this corrects an earlier grep-based count):

- Protocol 0 has 57 groups, 481 client→server messages and 1240 server→client messages.
- Protocol 3 has 1 group, 28 client→server messages and 21 server→client messages.
- Total: 1770 messages.
- `<Client>` is the client's send list and `<Server>` its receive list (ascalon's Frida
  agent: `sendMsgArray` → Client, `recvMsgArray` → Server).
- With only the confirmed encodings (`gw2_proto::dynamic`, default `Rules`), **1440 of
  1770** messages decode. The rest need `String`, `CString`, `ArrayVar*`, `BufferVar*`,
  `BufferFixed` or `Unknown` to be confirmed by REA.
