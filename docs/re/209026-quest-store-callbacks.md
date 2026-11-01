# Build 209026: quest store response callbacks

Static REA follow-up, 2026-11-01, to the
[conditional store request](209026-quest-store-request.md). The observed
callback paths produce internal process events. They do not establish a
required zone-load wait, successful operation or observed wire exchange.
The client stayed stopped; no store request or agreement acceptance occurred.

## Search response and errors

The second request's interface vtable `0x142111810` has success entry
+0x20 = **`0x140ff72f0`** and error entry +0x28 = **`0x140ff6a50`**.
The interface lies at whole-object +0x10, so its callback at interface -8
is whole-object +8.

The success entry calls the response object's virtual +0x58 for XML.
A null result constructs an error with numeric argument **0x3f** through
`0x140e339d0`, then invokes the request's error entry. A non-null XML
result and non-null callback invoke callback virtual +0x20, passing the
response, XML, auxiliary array at whole-object +0x58, and count at +0x64.

The error entry forwards a non-null callback to virtual +0x18 without
filtering an error code. For the request-ID callback vtable
`0x1419248c0`, this is **`0x140247420`**: it converts the error through
`0x140e345d0` to an eight-byte temporary and calls callback virtual +0x10,
**`0x140246f20`**. That method supplies a twelve-byte payload to
`0x141069d20` with internal event ID **0x50**:

| Payload offset | Value |
| --- | --- |
| +0, four bytes | Request ID from callback +8 |
| +4, eight bytes | Converted error value |

This is the error branch of the actual search request. A neighboring
request error function filters particular codes, but it is not the
+0x28 entry in this vtable and is not used to infer this branch's behavior.

## Item and package events

Callback virtual +0x20 is **`0x140247460`**. Its complete body through
`0x140247664` looks up these literal XML names:

| Collection / child | Child parser | Internal event on non-null parsed result |
| --- | --- | --- |
| `Items` / `Item` | `0x14024e8d0` | **0x42** |
| `Packages` / `Package` | `0x14024ea50` | **0x43** |

It iterates each collection's children through XML virtual +0x160, passes
the auxiliary array/count to each child parser, and supplies the returned
buffer and parser-reported length to `0x141069d20`. Temporary allocations
are released after each iteration. The parsers' complete payload schemas
are not traced here.

If neither collection resolves, the callback reaches its normal return
without invoking either child parser or emitting either item/package
event. The callback body does not read its request-ID object or emit an
additional explicit completion event. This is a bounded statement about
this method, not proof that other request-lifecycle code cannot signal
completion. Callback virtual +8, `0x140246e90`, is a null-guarded call to
virtual +0 with EDX=1; it is not treated as a success notification.

## Process delivery and remaining unknowns

All three event emissions supply EDX=0 and use the pointer at
**`0x14266c740`**. Initializer `0x140245a90` stores the result of
`0x141069a10` there. The getter reads **thread service context +0x30**.

`0x141069d20` returns false for a null process pointer or process +0x34
mask 0x20. With mask 4 set and EDX=0, it returns true without calling the
queue helper. Otherwise it passes the process's +0x680 object, event ID,
and one buffer/length pair to **`0x14106dc50`** and converts that helper's
result to Boolean. The callback bodies do not check this return value.
The [dispatch follow-up](209026-quest-store-dispatch.md) establishes copied
queue buffers, outer event 0x23 and the receiving store handler.
These IDs are internal dispatch values, **not game wire message IDs**.

The initial selected service-method reads and a bounded immediate-value search in
`0x140250000..0x140280000` did not identify the receiving handlers. This
is not an exhaustive receiver census: handlers may be registered through
tables or reside elsewhere. Primary service methods +0/+8 are listener
list insertion/removal, not the sought dispatcher. In particular, the
combined trace does not yet show how the earlier request tracker clears or whether
its pending state affects readiness. No error or empty-result fixture
should be described as validated from this static evidence alone.

## Evidence

Private bundle: `~/re/gw2/notes/209026-quest-store-callbacks-1.json`
(1,796 records). Executable SHA-256 remains
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Helper-driven reads bind the executable, not the external helper source.
Raw-reference and immediate searches provide candidates only. Unaligned
range edges and neighboring function bodies are not used as evidence for
the named methods. Rizin's large string-region warnings occurred; literal
names were checked with a direct bounded byte read.

| REA evidence | Observation |
| --- | --- |
| `ev_deb82017a879e2cfbe619ff17f2208c393a1016ce2d8b53ab3d297b285a2fe0e` | Callback and second-request interface entries. |
| `ev_7eb088527a501fdd5d7af7bd95c4c70b27cd744e898e642abf586ca6daa3f5c3` | Error adapter and request error forwarding. |
| `ev_2de1cb2f79c85c3b7edf887a3634fb87e3fe0e9bc01a8e04d6964a4740547aa3` / `ev_8948af23963312033fad33c5f69c7465836ade122ff06bfc80330a8a655b83e6` | Twelve-byte error payload, complete search response entry, callback release entry and queue gates. |
| `ev_36e2deafea5de47f1e3b5affaed923004a6b0dd7ae3d4343bb2dab5f422e6fee` / `ev_761342aed1b0f49cd62032f8a8b40c1cfb91664913f42d735486ed8615dbcaec` | Item/package loops and complete callback return. |
| `ev_b67a348ab6ce8cb818968d7076ceb4180ed5f85fc6e7d1db0410b552c60da89a` | XML name bytes. |
| `ev_59fca8c34b8c9dc571d305dd11de8db5e72a5daff6ebc4e302695a171d1c8915` / `ev_234ae0cbfb1e47946a16afac602183af98ef6be589eeaeb3ae210bb375cdab03` | Unwind records and selected constructor tail; not a full registration audit. |
| `ev_e864cf2937e83ecbcc1298f4a9f5e8eee4aafcf067646a8ddb4eb2812f787aad` / `ev_c8075e23caf302988511a02dd5bbd6057c54d5b86ba227342c6b259b319f0463` | Process-pointer reference candidates and validated setter. |
| `ev_4b53ffbf49c20bab99a5f2a94a37513ed9942ef66404883170ad378ce5d64212` / `ev_d11cfce7772d482e2533505f18b7740cbb5f02c8aba12e7918ed8a99f4607576` | Initializer caller candidate and disassembled call. |
| `ev_5eb92ebc99e799a72fcde201923132222e6eeae8c83e2e8b50dfa3a1f8ceae10` / `ev_c09bd3672e0a2ae39e86c8e362091a395d33ee7a52bc8e06756d0c41a7b9ff07` | Selected service tables and listener methods. |
| `ev_b5be908c0c0cc48f4d0915a0ccc729e9cf7e67315d646e3993aaa198f308c852` / `ev_91e91e5b73884c1d19e05a9a5b7cfdfb0b65cc7cf5ac5d08a885a8d1473c747f` | Initializer, selected service methods and thread-context process getter. |
| `ev_d4d9184af66e4908d61c2ce60c75630cb7194eb18a09f5632bd3a13c84a7ba47` | Bounded immediate candidates; did not locate the receiver. |
