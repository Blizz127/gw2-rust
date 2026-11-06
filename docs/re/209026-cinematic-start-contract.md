# Build 209026: cinematic start and stop contract

Static REA, 2026-11-06. The cinematic subsystem has a typed server start
path that resolves a **type 7 CinematicDef** and uses its filename and
playback fields. This establishes a concrete missing input for the Norn
opening: the selected content key and authoritative trigger/sequence. It
does not identify that opening or make character creation play it.

The [bounded cache/biography follow-up](209026-cinematic-biography-cache-boundary.md)
retains 1,366 type-7 index entries and verifies sample filename references.
The inspected biography question/answer records contain no direct relocated
cinematic reference. This does not exclude scalar or server selection and
does not choose a Norn key.

## Registered messages

Registration table **0x1421815b8** and the complete terminated descriptors
agree with `data/protocol/209026/server-tables.xml`:

| ID | Descriptor | Handler | Body fields |
| --- | --- | --- | --- |
| 720 | 0x1425e47b0 | 0x1412b1f50 | Dword, Dword |
| 722 | 0x1425e4850 | 0x1412b1ff0 | Dword, Dword, Byte |
| 721 | 0x1425e4920 | 0x1412b2050 | Dword |

Native decoded offsets below include the two-byte header. They are not
wire byte offsets. The descriptor's Dword tokens retain the existing
varint interpretation; no leading player index is present.

For **720**, complete handler `0x1412b1f50..0x1412b1fe5` resolves the second
Dword at decoded+6 through content virtual+0x58 with numeric type **7**.
Its assertion explicitly names `cinematicDef`. It obtains the cinematic
context from TLS+0xa8 and calls context virtual+0x38 with the definition,
float zero, the first Dword at decoded+2, and trailing arguments 1 and 0.
The first Dword's gameplay meaning remains unassigned; it must not be
renamed as the definition's playback flags.

## Concrete receiver and playback arguments

Factory `0x1412afe70` creates the context through `0x1412afba0`, which
installs primary table **0x142181398**. Its +0x38 entry is
**0x1412b0340**. That complete method reads:

| Definition member | Use in this path |
| --- | --- |
| +0x30 | Filename passed to `0x14130c150` and the common start routine |
| +0x38 | Dword playback flags passed to the common start routine |
| +0x68 | Float passed separately from the handler's float-zero argument |

Common start **0x1412b0440..0x1412b06d2**, labelled `CinCliContext.cpp`,
logs `CinCli: Starting cinematic`. It handles an existing scene, searches
the cached scene array by filename, and otherwise constructs an 0x88-byte
scene through `0x1412b0b70`. It registers its listener and calls
**0x1412b1b00**. The complete body through `0x1412b1c8e` logs
`CinCli: Playing cinematic with flags %d`, stores the parameters, and
acquires/starts the scene resource along its conditional playback path.
The literal extension-specific alternative and all downstream resource
implementations have not been assigned semantics here.

The definition's flags and float are therefore client data inputs, while
the server supplies a separate Dword. No default flags, value, duration,
opening key or sequence follows from the general contract.

**722** is a second start route, not the preload method. Its complete
handler resolves decoded+6 using content virtual+0x190 and calls
`0x1412b0280` with that record, decoded+2 and the final byte. The callee
reads record+0x30, calls the same common start with flags 4 and a null
type-7 definition, and retains the record/final byte in context+0x58/+0x60.
This slice does not bind virtual+0x190 to a numeric content type. The
separately inspected context+0x40 method `0x1412b06e0` does resolve type 7
or type 41 according to bit 2 and populate the cached scene array; its
presence does not turn message 722 into a preload request.

## Matching stop path and response boundary

**721** forwards its Dword to `0x1412b08c0`. When context+0x28 holds a
scene, it clears context+0x58/+0x60, then compares the supplied value with
the +0x14 member of each applicable nonnull definition. The first
definition is scene+0x18; a second comes from the complete leaf
`0x1412b0ef0`. A mismatch returns without the stop dispatch. Otherwise,
scene virtual+0x10 resolves through constructor table **0x1421818c0** to
`0x1412b1f20`, which calls the resource's +0xa0 method and tails to
`0x1412b18f0` with argument 1. This is the observed matching stop/end path;
it does not identify which Norn content key should be stopped.

Complete `0x1412b18f0..0x1412b1a52` clears active state, performs view and
listener cleanup, and conditionally invokes `0x1412b2080` once when scene
flag 0x20 is set and 0x2000 is clear. That helper sends **client 263** with
one byte equal to whether its input is nonzero. Another inspected early
playback branch passes zero. These boolean writes do not establish a
named success/skip/error enum or an authoritative story transition.

## Evidence and limits

All helper reads assert frozen PE SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Only Python/objdump helpers ran; no client or live server was launched,
and no archive was accessed.
No runtime lease was obtained, no message was emitted, and no server
behavior or publisher state changed.

- Main ten-capture bundle:
  `~/re/gw2/notes/209026-cinematic-start-contract-20261106.json`, SHA-256
  `4af50720fb9e5f4db245c20a90468ce1243bb6a990456770de3945eb1fef82d6`.
- Complete end/leaf follow-up:
  `~/re/gw2/notes/209026-cinematic-end-contract-20261106.json`, SHA-256
  `600837c072e24d96d7cd42c0b4a1c16fe8b097ec1f8bbf9da193bfe26d5e6982`.
- Independent descriptor check:
  `~/re/gw2/notes/209026-cinematic-descriptor-check-20261106.json`, SHA-256
  `c1e605dd7fb147f73e8742c90a0be094fab9ad06db3d9c91e525a740573ca8ba`;
  REA `ev_ffb52692bf514f2513ef0273de97cba8059bf77ff208f70dbbd60579b4d67bcc`.

Principal REA records are `ev_9c5d19c05c1809bf5f17b7f4e425475e5b68feb3f2c1fff9ffc1989df2a46ba8`
(720/typed lookup), `ev_25b08969624705df8283dbcbd5cf9e7c84cce07ee96af81543ea0481b8a10f00`
(constructor and 721/722), `ev_bd73b257c3384f2fdbe197a5d70da4a23ba9b5bf556edd82bfdad29c68ef1b1d`
(concrete table/forwarders), `ev_c101c8fc6a3536f3bab75b34381a7ba2a69f7944ad715460e18babc6e48119bd`
(complete play body), and `ev_e1c6f0886c0d5e60c1ca1ce3025edc11b2f04d2dd9458b696dd3f04bc89d1b1f`
(complete end body). The main bundle retains initial bounded seed/reference
queries and the explicit continuation windows rather than treating a short
`.pdata` fragment as a complete logical function.

All thirteen captures exited 0 with complete raw terminal output and
verified owned-process cleanup. The initial string-seed record flags one
omitted rendered frame; its raw output is complete. Some records could
not verify ownership of other process environments; they preserve this
limitation and do not claim a host-wide cleanup audit. The leaf getter
lacked a `.pdata` entry and was subsequently captured to its return.
