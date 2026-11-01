# Build 209026: additional primitive wire widths

Static REA analysis of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This extends the verified capture-census subset used by the private mission
schema. It does not establish gameplay meanings or valid gameplay values.

## Kinds 6–8

The encoder `0x140ff04e0` indexes table `0x140ff178c` directly by kind.
The receiver uses kind minus one to index table `0x140ff237c`.

| Kind | Schema name | Encoder branch | Receiver branch | Wire bytes |
| --- | --- | --- | --- | --- |
| 6 | Float | `0x140ff076c` | `0x140ff1d67` | 4 |
| 7 | Float2 | `0x140ff082a` | `0x140ff22fe` | 8 |
| 8 | Float3 | `0x140ff08a2` | `0x140ff1d85` | 12 |

Each branch copies the stated number of bytes unchanged, with no count prefix
or variable-integer transformation. The streaming encoder copies one, two or
three 32-bit words; its buffer path copies 4/8/12 bytes. The receiver checks
remaining input through the shared size table before copying and advancing.
On this x86-64 target these match the existing little-endian f32/vector codecs.
Mission control server ID 5's handler `0x14023ee70` loads its kind-6 fields
with `movss` and passes them to `0x14023b110`, which stores single-precision
values in globals. Vector coordinate meanings remain unassigned.

This evidence admits Float, Float2 and Float3 as capture-census primitives,
including in the already supported one-level arrays. Float4 and Point3 remain
outside that tool's whitelist until separately checked. This is boundary
decoding: arbitrary float bit patterns do not establish valid game values.

## Kind 14: terminated raw bytes

Encoder `0x140ff0cf5` calculates byte-string length plus NUL and compares the
total against descriptor +0x10, then copies that many bytes. Receiver
`0x140ff1e97` scans one byte at a time for NUL and enforces the same inclusive
maximum. It is not count-prefixed. The original dynamic CString rule selected
a count prefix and could not represent this terminator convention. Its values
are already raw byte vectors, so an explicit `CStringEncoding::Terminated`
rule can retain that representation without assigning a text encoding. Size
includes the NUL; decoding must stop at the first NUL within that limit, and
encoding must reject embedded NULs or data that leaves no room for the terminator.
The existing counted rule remains explicit for reference layouts. Default rules
continue to reject CString. The metadata-only census can enable the verified
terminated rule and admit kind 14 after those boundary checks.

Implemented with `Value::Bytes` and explicit `CStringEncoding::{Counted,
Terminated}`. The focused codec test covers non-UTF-8 bytes, all truncations,
inclusive bounds, empty strings, zero-size rejection, embedded-NUL rejection,
preservation of following fields, and output unchanged on invalid encoding.
Default rules still reject the field and the explicit counted mode still works.
All five census tests and observer/census builds pass. A private handwritten
fixture for the actual mission descriptors 1041 (CString128, Float) and 1043
(CString128, CString256) parses as 9 and 5 bytes respectively. It was never sent
to the client; field meanings and text encoding remain unknown.

## Evidence

The census whitelist now includes these three float kinds. All four focused
census tests and the build pass. A handwritten private mission-control fixture
parses ID 5 as 18 bytes followed by empty ID 3 (2 bytes); it was not sent to
the client. The saved run-58 auth capture still parses all six messages/233 bytes.

The ignored mission schema was first re-extracted with kinds 6–8 named accordingly:
12,018 bytes, SHA-256
`3615ec142b5896ec15a001a619d7035344aef715c4954fb1e644ed04c1db2e41`.
The previous schema is retained privately as `mission-before-float-kinds.xml`.
The subsequent kind-14 update preserves descriptor sizes and produces 12,034
bytes, SHA-256
`8814edf4ef309fdac0e1b60b165d93a2e03529dff7c878336ff81f9ffec121b3`.
The prior float-only schema remains private as `mission-before-cstring.xml`.
Message counts remain 43 client/54 server across protocol-0 groups 39/40.

Private bundle: `~/re/gw2/notes/209026-primitive-static-3.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_7ddd0cb187281d3a55ed1b53475719f29e906971223ec1b5cb917cc11bb2659b` | Complete CString receiver and descriptors using kind 14. |
| `ev_fba106612650eae93ea495187e26df0d9e54d2c19fd74d17fa0e0c488963c3e6` | Kind-14 schema extraction and real-descriptor fixture census. |
| `ev_2a54bcca7b1a1c73613775ed64f12e649bd678733336c28833513cab62fa9f16` | Updated private schema extraction metadata. |
| `ev_a6ec7b66340428fec1de107a2f7896a93e9552d003eee2c75acf0cb29280c87e` | Mission-control fixture and saved auth-capture census. |
| `ev_5f947d3d5429eaea23bde02dfdd8813c95d0db4cb57f7e021ebeee285d10dfa4` | Encoder kind-indexed dispatch. |
| `ev_90f2f6a07baff741facc029a8b25a0d6df82dd0f3a482bb519606d7bfaa5f85b` | Kinds 6–8 raw-copy encoder branches and kind-14 byte terminator. |
| `ev_221622a66f246cc0eef7f7a6135b31b0aaf77d1236ab84b30395a8f8fe89fe59` | Receiver dispatch, bounds checks, kinds 6/8 and terminated-string branches. |
| `ev_d8c8f752f67bb3eeeaea91a268fbfbc13499bff1fcddb2b2a779578e65cc526e` | Kind-7 eight-byte receiver and mission control float consumer. |
