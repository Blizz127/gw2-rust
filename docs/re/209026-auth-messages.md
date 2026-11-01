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
