# Build 209026: initial binary authentication messages

Investigation 2026-11-01, frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Client observations below use REA; raw inputs remain private.

## Registration and first outgoing messages

Four registration functions are at `0x140ff3c70`, `0x140ff3d30`,
`0x140ff3df0`, and `0x140ff3e80`. Their spacing and arguments match the older
reference's registration mechanism. Static caller `0x1401a7100` registers
protocol 3, group 40: 28 client entries at `0x141923f00`, 21 server entries
at `0x141923fe0`. The latter entries pair a definition pointer with a handler.

Message definition records have 40-byte entries: kind at +0, message ID or
field parameter at +16, nested definition pointer at +24. The root kind is 1,
fields start at +40, and kind 0 terminates the fields. The complete build schema
has not yet been extracted; type semantics still need confirmation individually.

Run 51 probes outgoing serialization at `0x140ff0219`, after definition lookup.
It observes two messages in the 57-byte plaintext:

| Protocol | ID | Definition | Caller return |
| --- | --- | --- | --- |
| 3 | 2 | `0x1425af2e0` | `0x140241011` |
| 3 | 3 | `0x1425af380` | `0x140241041` |

ID 2 has two kind-13 fields, each with parameter 32 (the older reference calls
these String). Caller `0x140240f50` fills two local buffers through
`0x1409cbbf0` and `0x1409cbab0`, then sends their pointers. Their contents and
exact text encoding are not printed or yet assigned semantics. ID 3 is sent
immediately afterward using a value from `0x1409afcc0` and a pointer from
`0x1402421a0`. Neither message is yet identified as a credential/token message.
The private synthetic Token and UserId are absent in standard UUID encodings.

## Candidate first server reply

Protocol-3 server ID 2 has definition `0x1425b09d0`: one kind-4 field, with
handler `0x140240700`. The handler stores the decoded u32 at context +0x28,
sets global active context `0x14266c408` if empty, then calls `0x1402423a0`.
That helper sends client ID 1 with one u32 value derived from a clock-like
helper, and installs a recurring callback. The exact semantic name of the
server field is not established; a local fixture value 1 will test this branch.
Server ID 1's handler `0x1402406f0` simply returns success.

The outgoing kind-4 switch selects `0x140ff068c`, which calls `0x140ff1800`.
That routine emits low seven-bit groups with bit 7 indicating continuation.
A small value 1 is therefore one wire byte. The message-ID kind uses a direct
little-endian u16. Candidate fixture bytes are ID 2 followed by varint 1.
Incoming kind-4 behavior will be checked by the handler's decoded value.

## Server framing

Receive path `0x140feeef0` reads a four-byte header after RC4: two little-endian
u16 lengths. If the first is zero, it copies the second length of raw bytes.
Otherwise it consumes the first length of compressed bytes and calls
`0x1415757e0` with the second length as output size. The existing server codec
uses LZ4 with this header; compatibility remains a runtime test, not a completed
claim. A single explicit encrypted reply fixture will test the frame, decoded
message ID, field value and resulting outgoing messages.

## Evidence and validation

Private export: `~/re/gw2/notes/209026-auth-messages-static.json`.

| Evidence ID | Observation |
| --- | --- |
| `ev_bfc2fc663d5ebc2f769aac26be648c6c563a8e1a9a8cd57abc20d1a1f1bbbe89` | Bounded registration signature search. |
| `ev_63a9e59f1acea6762983f9fcf9dedc9d5fe5d9c218e021df7d9195249fe91b97` | Registration functions. |
| `ev_5c4ba957b92621d1743b7967b005328d3807271970ff1e24b7fe0a3209b970bb` | Direct static registration callers. |
| `ev_afef1fe7035a80c662716c8d3034b5ddb1e053dd67adb0ac6f4bad8f7f225bb0` | Protocol-3 and protocol-0 ID-2 definitions differ. |
| `ev_5fdeada934ee8fcc6844bd4897363526a25159182df25529a16a1d66e87bae33` | Outgoing serializer lookup and probe placement. |
| `ev_15fed024679a23377e33ba37781059c1d51b9ceb31054ea4bca083f564a9f481` | Run 51 protocol/ID/definition/caller observations; exact plaintext match again. |
| `ev_be5015bc2ab936008b767e8637d340a892c813478ab4079362347f8eab91b753` | First two message constructions. |
| `ev_5e7c47f4b7286c3e7bd213fc7d7515fd0e49f70b709050d3d325f439b205e330` | Server registration pointers. |
| `ev_e29a621ac3cbca0f315653e262e655bf1400d9d14f7c4ff08985078ae38bf341` | Initial server handlers. |
| `ev_e661c4dee2d22fa4ad3caefb040016089bbcf4cd032b24902154c86afbd8b388` | ID-2 continuation sends client ID 1. |
| `ev_4b9f16a05e9a594e4e83c696dbd27919adc0f55d11c24f11169bcd9054569fe5` | Kind-4 switch target and reply definition. |
| `ev_e48be10e2df55c3b285af632a3be943dee055392a85061f08cbf8d07319f2459` | Seven-bit varint encoding. |
| `ev_fd3b490085138f71b270450384a03ff7fbf721a04de236bee60e2fbef16ba822` | Server frame lengths and raw/compressed branches. |

Run 51 used network-none Docker, own in-memory public identity, synthetic portal
fixtures, and mode-0600 captures. Cleanup, unchanged executable hashes and archive
count 1,124,232 were verified. No character-selection screen is reached yet.

## Offline reply verification: run 52

The explicit fixture contains server message ID 2 and varint value 1. The observer
compresses and encrypts it using `ServerCodec`. The client reaches `0x140240700`,
and the decoded u32 at +2 equals 1. It then sends protocol-3 client ID 1 and
**ID 10**, definition `0x1425af5c0`, caller return `0x140241775`.

The two captured client writes total 115 bytes (57 + 58), exactly equal to the
client's pre-encryption buffers. The synthetic portal Token now appears in this
plaintext. This verifies one LZ4-compressed server frame, incoming u16/varint
handling for this reply, and continued RC4 state across both client writes.
It does not establish arbitrary message semantics or character selection.

ID 10's static fields are kind 4, kind 11, kind 11, kind 13 (parameter 64),
and kind 13 (parameter 28). The next step is to map its construction and the
corresponding successful server response before adding another fixture.

The observer's new `--reply-frame FILE` option sends one explicit bounded
plaintext message block through its server codec after key exchange. It is off
by default. Focused build, clippy, and the split-read server-frame test passed.
Network isolation, cleanup, unchanged executables and archive count were verified.

Evidence: `ev_5c3222b1673679e682200585b4f83250fb4694fcb9c1704dd7b4951c64d242fd`;
private export `~/re/gw2/notes/209026-auth-ready-run52.json`.
Definition-byte capture: `ev_6bd28470bb62ebb9dfddbfac03905e7a805a17332198210f84154ca3e334a209`.

## ID 10 and reply correlation

Sender `0x1402416f0` takes the request u32, two 16-byte values, and two string
pointers. Server ID 8's handler `0x140240850` forwards the first u32 to
`0x14023a770`, which searches outstanding requests by the field at +0x30 and
requires request state 3. For request type 7 it stores returned identity and
account fields. This handler also sends client ID 4 once, then sets a context
flag. ID 8 alone does not mark the outstanding request complete.

Server ID 7's handler `0x1402407a0` calls `0x14023a9e0`. That helper searches by
the same request u32, requires state 3, records two result fields for request
type 7, then sets state 1 and copies four low-u16 status fields. A minimal
synthetic experiment can therefore send ID 8 with the actual request ID, then
ID 7 with that ID and zero result/status fields. The meanings and acceptable
values of the other account fields remain to be tested; zero-filled fields
are a fixture hypothesis, not a recovered production account format.

ID 8 fields: Dword, Guid, Guid, seven Dwords, Byte, BufferFixed(32),
ArrayVarSmall(max 50), BufferFixed(32). The nested array entry consists of
Word, Word, Dword, Dword, Dword. ID 7 has seven Dwords, including request ID.

## String and buffer wire observations

The outgoing kind-13 branch at `0x140ff0c0a` obtains a wide-string length, adds
one for the terminator, checks that count against the definition's maximum,
and copies those little-endian u16 units directly. **String is UTF-16LE with
a terminating zero unit and no count prefix** on this path; the schema maximum
includes the terminator. This differs from the previously configurable counted
string option. Empty strings therefore occupy two bytes.

Kind 17 at `0x140ff0f26` writes a u8 element count before recursively encoding
its elements. Kind 18 uses u16. Kind 19 at `0x140ff1198` copies exactly the
schema size in bytes from the field pointer. These observations permit bounded
parsing of the initial client messages and construction of an empty-array
account fixture. Incoming handling of the richer fixture still needs runtime
verification.

Private export: `~/re/gw2/notes/209026-auth-success-static.json`.

| Evidence ID | Observation |
| --- | --- |
| `ev_26999b58d55375a43db6f45939acd137347b8faa49699e3b36107389cb33016a` | ID-10 sender. |
| `ev_4ce1e89b64bb7defdf56d06562dbd8eb14c4e2f1b17cf34bef97383a2758ca37` | ID-8 handler and client ID-4 follow-up. |
| `ev_a083b4b967bbfa0d5c46fe9754b6efcb9efb51b387c73c334b3ca1a161382def` | Correlated request data storage. |
| `ev_9e99eb92f2c8ef4d437f4fe73ffd9c38b178a89976c9219420da509a2a5123d6` | ID-7 request completion. |
| `ev_39cd597707855c49e80601e490ee32ca3cb899d4230796fbc03ad4492fdc7d91` | Nested array definitions and String switch. |
| `ev_49983c94e72ecb49d3502d9c5fe3ebf79c5b32e3abfc0aef5b444d8fdcd0c8d3` | Terminated UTF-16 encoder. |
| `ev_3406d2c89365f45a4d78facf74d34091b8a0ae9308b9ca112cbfbea9442cbd44` | Array counts and fixed byte buffers. |

## Correlated synthetic account result: run 53

The observer parses client messages 2 (36 bytes), 3 (21), 1 (7) and 10 (51)
using terminated UTF-16, raw GUIDs/fixed buffers and varints. It takes the actual
request ID from message 10 and sends ID 8's explicit private body followed by
ID 7 with zero result/status fields. The body contains synthetic identity GUIDs,
seven zero Dwords, a zero Byte, two fresh private 32-byte blocks and an empty
array. These field values are a local experiment, not a complete account model.

The client reaches ID 8's handler, matches the request ID, and enters the
request-type-7 data-storage path (`0x14023a7f0`). It sends client ID 4 and reaches
`0x14023aa48` after marking the outstanding request complete. The next observed
serialization is client ID 34, definition `0x1425af950`, caller return
`0x1402418a1`. The wire capture totals 230 bytes and exactly matches the client
RC4 input; the third write is 115 bytes for the ID-4 follow-up. The ID-34 probe
establishes a serialization attempt, not yet a separately captured wire message.

The last periodic screenshot preceded this result and still shows the login
form. No post-result screenshot or character-selection screen is established.
The next step is to trace ID 34 and capture the resulting UI state.

Implementation adds opt-in `--auth-reply-body FILE`; it preserves split client
reads and replies once to a complete ID 10. It performs no credential validation.
`StringEncoding::Utf16Terminated` adds bounded, terminated wide strings without
changing the existing counted option. Focused tests verify terminator bounds,
surrogate pairs, truncated input, split/coalesced auth messages and correlation;
observer build and clippy pass. An initial clippy style warning was corrected.

Evidence: `ev_498df340b729b9de04a714b433e05c1f58af7a941e05d6c843d5a24b03e1ad92`,
private export `~/re/gw2/notes/209026-auth-success-run53.json`. Isolation, container
cleanup, executable hashes and archive count remain verified.

## Post-authentication crash: run 54

The repeated fixture again produces the same 230 bytes and matching plaintext,
but three screenshots taken after request completion show the client's serious
error dialog over the launcher. This is not a successful UI transition. The
isolated Wine prefix's local `ArenaNet.log` reports
`s_configDataError != NET_PENDING`, `GcApi.cpp(2621)`, build 209026. This suggests
configuration data remains pending when the synthetic account request completes;
the responsible state and protocol exchange still need tracing. No crash report
was submitted. Old crash files beside the executable predate these runs and
describe a different thread-creation assertion; they are not evidence for this
failure.

Client ID 34's sender (`0x140241850`) takes a Dword. Its caller at `0x14023bc80`
uses the first byte of a local buffer filled by `0x140e4a500`. The value's meaning
is still unknown. A private schema extraction now parses all 28 client and 21
server definitions in this protocol group; that does not establish all wire
encodings or field meanings.

Run evidence: `ev_1c567277e77777feafb7eaa3f8a803ef21ddb78de08593216d5eff39a015c8b6`,
private export `~/re/gw2/notes/209026-auth-post-ui-run54.json`. Crash-log evidence:
`ev_45ac7b4923bd0e2c01988a6e0856dfa562d076cb68ae9742c4799d7e37a4d99d`.
Network isolation, container cleanup, both executable hashes and archive count
remain verified. Next: trace configuration-data initialization and completion.

## Configuration reply must precede completion

The assertion at `0x14023b907` tests global `0x142554524` against -1. Reset
`0x14023b7d0` assigns -1. Server ID 10, definition Dword + BufferVarLarge(4096),
handler `0x140240990`, calls `0x14023a630` with the request ID, byte count and
buffer pointer. That function requires the matching request to remain in state
3 and type 7, copies the buffer into the configuration-data global, and sets
the error/state global to zero at `0x14023a74c`. Sending this after ID 7 would
be too late. An empty buffer before ID 7 is the next explicit fixture experiment;
its acceptance by downstream configuration consumers is not yet established.

The variable-buffer serializer branches at `0x140ff11f9` and `0x140ff1298`
write u8 and u16 byte counts, respectively, followed by raw bytes. An empty
large buffer therefore has two zero count bytes.

REA evidence: assertion references
`ev_f0909835dfd259d315e4a45de8be9b95d0827562ee4675d5af11ac9b0b998f39`,
state references `ev_97ba32c2af86be86ce6b07faef76b7f01aafcc44161e03c6e20cf7c6bb4b0b49`,
handler `ev_495037a325a63fd5ca5600ee9fbb5659cbed3f3492519386c8926b54fb1e8991`,
request guard `ev_abb35bed9a53df41210cd8edd3495a8fdf84f32f53168581d8cac28ef017f210`,
buffer encoding `ev_9f39f34c2eec4476967954973dd59f5d7e6674d674bcb4489b98d1b3cb13bcdb`.

## Empty configuration accepted: run 55

The opt-in auth fixture now sends correlated ID 10 with an empty buffer between
IDs 8 and 7. The runtime probe sees configuration state zero and byte count zero
before request completion. Client ID 34 now reaches the wire (three bytes), and
the client sends new TLS requests `/Auth2f/Check` (empty body) and
`/Auth/GetUserInfo` (`Request` with no children). Post-completion screenshots
show the launcher login spinner without the previous crash dialog. No assertion
probe fired during this observation window. Character selection is not reached.

The first binary connection captures 233 bytes; after the observer's idle timeout
a second token request and binary connection capture another 230 bytes. The
plaintext comparison helper explicitly reports unavailable because it only
supports one capture file; do not extend the earlier exact-equality claim to
this run. The first connection's pre-encryption probe records 233 bytes.

Focused auth-parser test, observer build and clippy pass. Network isolation,
container cleanup, executable hashes and archive count are verified. Evidence:
`ev_2c4930ed3d642d86a78301b346d54ecfdc818ef33742a6c538ba1dd63b37033c`,
private export `~/re/gw2/notes/209026-auth-config-run55.json`.
Next: map the two newly observed portal reply formats.
