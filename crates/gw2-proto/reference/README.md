# Reference message schema: build 208039 (NOT our build)

`protocols-208039.xml` is copied unchanged from valaphee/ascalon
(`protocol/protocols.xml`, commit `03fb60b`, MIT, Copyright (c) 2026 Valaphee).
It was captured at runtime from client build **208039** with a Frida agent.

Do not treat it as observed for the client build we freeze. Message ids and layouts
drift between builds. Our own schema will be regenerated for our build through REA
and saved next to this file as `protocols-<build>.xml`, with its evidence IDs
recorded in `docs/re/`.

Census (from this file): 2 protocols, 58 message groups, 1770 messages. Nearly all
of them are client→server layouts (`<Client>` 22 groups, `<Server>` 1). Field
types: Dword 2282, Byte 962, Word 422, Float 230, String 218, Guid 197,
ArrayVarSmall 139, Optional 101, Qword 86, Float3 69, ArrayFixed 61, Point3 43,
BufferVarSmall 41, Float4 31, Float2 24, ArrayVarLarge 20, CString 17,
BufferVarLarge 6, BufferFixed 4, Address 4, Unknown 2.
