# Build 209026: character-protocol investigation

Static investigation 2026-11-01. Runtime UI progression currently waits on the
untouched User Agreement decision recorded in `209026-post-auth-portal.md`.
No character screen, character creation or zone entry has been observed.

## Candidate record messages

Protocol 3 server IDs 17 and 18 share this definition: Dword, Guid, Dword,
Qword, Qword, String(20), BufferVarLarge(512). Handlers `0x140240a00` and
`0x140240a50` forward to `0x14023aa50` and `0x14023ab70`, respectively.
Both find an outstanding request by its ID and require state 3.

ID 17 additionally requires request type 7 (login) or 24. It finds or creates
a name-keyed 0x258-byte record via `0x14023a4e0`. ID 18 appends a record to a
separate vector at `0x14266bee8`. Both store GUID at +8, name at +0x18,
blob at +0x40, blob length at +4, a Dword at +0x240 and two Qwords at
+0x248/+0x250. The latter fields' meanings and Qword wire encoding are unverified.

`0x14023c220` copies an indexed record from the second vector. An item-tooltip
consumer at `0x1408b4a89` searches these records by GUID and formats the name.
This is consistent with character-bound item display, but character semantics
remain an inference until connected to a named operation or runtime evidence.
Do not yet build character fixtures from these inferred field meanings.

Client ID 11's sender `0x140241790` is called from teardown/reset paths, including
`0x14023bc18`; its one Dword must not be assumed to request the character list.

## Capture inspection work

A metadata-only census tool will parse saved decrypted streams using the private
REA-extracted schema and only the already verified field kinds: Byte, Dword,
Guid, terminated UTF-16 String, and raw fixed/u8-counted/u16-counted buffers.
It will report message IDs, byte offsets and lengths, without field values.
Other field kinds will remain rejected rather than using historical encodings
to infer boundaries. This work can proceed without advancing the held UI.

| REA evidence | Observation |
| --- | --- |
| `ev_2bbda5ddb2cd8881247144b53c56168d08a6832ac3977d0992725ac704cf0777` | IDs 17/18 handlers. |
| `ev_aabebd41c0d1fede6902d9eec44c96d905106ab22f6d4087fc0e0c0e787e2485` | Request correlation and record storage. |
| `ev_62a544a39bd6df8e7786399d13acfd5e4661d1e6f8476839990843f19f71541a` | Name-keyed cache. |
| `ev_f224a9e4ea69c8cc6d7574013191185c17f7c2b0b080fd2d7da5ce4e9b45bd92` | Record consumers. |
| `ev_bfa88c3fd20526bfdbe30d3de834857d0666a922fdb3c1e2abda4a7daa328371` | GUID/name tooltip use. |
| `ev_875aa1fa577cfdcc67ae18dc76c409eb4474ead5ce47e1f19f0a4148ae729daa` | Source path identifies item tooltip. |
| `ev_fe0b1cabf1cc8f9c78435ee4786f825c9711e51ff411808292d36e55d00ca7fe` | Client ID 11 sender. |
| `ev_ef308eb2b0d1a927df627604082e35d24b8d24759c04c03a4c7fbe8a10dbb185` | ID 11 reset/teardown callers. |

## Metadata census verification

`gw2-capture-census SCHEMA PROTOCOL GROUP client|server CAPTURE` is implemented
with bounded file reads and no field-value output. It requires BuildId 209026,
an unambiguous group and unique message IDs, and rejects unverified kinds even
when the historical dynamic codec supports them. A focused test checks complete
boundaries, every truncation of a string-bearing message, rejection of an
unverified Float field, and absence of the test string from output. Build and
clippy pass.

Applied through REA to run 58's saved decrypted capture:

| Offset | Client ID | Bytes |
| ---: | ---: | ---: |
| 0 | 2 | 36 |
| 36 | 3 | 21 |
| 57 | 1 | 7 |
| 64 | 10 | 51 |
| 115 | 4 | 115 |
| 230 | 34 | 3 |

All 233 bytes are consumed in six messages. This verifies stream boundaries,
not the semantic meanings of their fields. Evidence:
`ev_a5c47782ac3dec6276661ad3601ab52dfbe9930a5a3435178fd97383117437a5`.

## Character-select consumer established

The earlier inference for ID 17 is now supported by a direct static chain.
`ScnCharSelect.cpp` references identify the scene; its routine `0x1402a3550`
iterates `0x14023c2f0`. That accessor copies records from the same vector at
`0x14266bf98` populated by ID 17. For each record the scene passes its blob
length (+4) and blob bytes (+0x40) to `0x141419e80`. This is a thunk to
`0x1417f4d90` with callback object `0x142610190`. The blob format is described below.
The character-select item widget also copies a 0x258-byte record.

ID 17 therefore supplies records used by character selection. ID 18 uses the
same record layout in a different list; its precise role remains unverified.
Runtime list population is still untested and remains behind the UI gate.

| REA evidence | Observation |
| --- | --- |
| `ev_7f3868eb38ac54988eb45c2846cd9e4d3d1eb5c7b65411a307c11c82a71f9c72` | Character-scene and widget source references. |
| `ev_535f127e57becec5b1f026a85858fd53eb4444d0bf52755a6580a28eb0ade789` | Scene iterates record accessor and invokes blob decoder. |
| `ev_176240d6026f8a2e4ac45bfcb0ea4356e40e3ad258cbbfd6791e26b820b7d630` | Accessor reads ID-17 vector. |
| `ev_ae068f97653fc0143eb6ff829dabccc178df12df177cc137eb7b3b15afb6aee6` | Character item copies the record. |
| `ev_007a08c64d1a850ed19c20215c1c28d6fca0c8479ba865441081868b4e1eae72` | Blob-decoder thunk and descriptor. |

## Summary blob version and checks

`0x1417f4ae0` clears a 159-byte output structure, marks its version byte 255,
and rejects inputs shorter than five bytes. Version 11 requires exactly 159
bytes and is copied directly. Older versions 7, 8 and 9 require 153 bytes;
version 10 requires 157. Those paths convert into version 11, with additional
field conversions for versions 7/8. Other historical versions below 11 take a
fatal-error path; malformed size/version combinations fail. These are static
observations, not tested fixture recommendations.

`0x1417f4d90` then checks these offsets in the normalized structure:

| Offset | Constraint and established meaning |
| --- | --- |
| +4 | Level Byte <= 80 |
| +5 | Profession Byte in 1..=9 |
| +0x21 | Race Byte < 5 |
| +0x22 | Byte <= 15 |
| +0x23 | Sex Byte < 3; preview has the stricter check below |
| +1 | u16 map-definition lookup key accepted by callback `0x141419e40` |

The callback requires a non-null lookup result from a content interface.
Specifically, it uses the service at `0x1409b1fb0()` +0xe0 and virtual slot
+0x1a8. The game-loading scene at `0x1402a7d5a` uses that same service/slot,
stores the returned object, and asserts `m_mapDef` from `GameLoad.cpp`.
The summary's +1 Word is therefore a map-definition lookup key. The character
detail consumer at `0x140433685` performs the same lookup from the normalized
summary and passes the result to its display path. No particular numeric key's
presence in the frozen client's loaded content has yet been demonstrated.
The preview consumer below establishes the appearance substructure and its
race/sex fields. Other field names require separate consumer evidence; matching
a numeric range alone is insufficient.
The scene additionally reads a GUID at normalized blob +0x85 and skips its
resource lookup when all 16 bytes are zero. A blob passing the basic checks is
therefore insufficient to guarantee a rendered character.

Evidence: `ev_0205d1011aed3e65f16c7dacf89651e6aeb4d826a3386a6668e9fb7566cef53d`
(version conversion),
`ev_d105b602296b43cd635bc3e99ebe6b591df78d0f54d804f818b878bd456b1188`
(range checks),
`ev_b668f5948f5bda3a3c9b567c907cbc6b263bd14ad9848509d625670e547e2b2e`
(content lookup),
`ev_81eb316c831dccd39758b438c1e0cbc6b09818772e381963177a0c3cc408c2b7`
(scene GUID use).

## Summary-to-preview appearance conversion

The character-item path at `0x140436874` normalizes the record into a temporary
159-byte summary, then passes it to `0x14147df50` and the result to
`0x14147da90`. The converter copies exactly **45 bytes from summary +6**
to preview-description +0x48. The builder passes that block to
`0x1411be9b0`, whose named assertions identify byte +0x1b as race and
byte +0x1d as sex. These match the independently mapped
[creation appearance fields](209026-character-creation.md#appearance-fields-and-local-validation).
Consequently summary **+0x21 is race** and **+0x23 is sex**, with the same
embedded enum labels. This establishes a shared 45-byte appearance layout;
it does not establish the meaning of creation request bytes 45–47.

The decoder and renderer have different checks: the summary decoder accepts
sex values below 3, but the preview builder explicitly asserts
`sex != CHAR_SEX_NONE` for value 2. A blob passing normalization is therefore
not necessarily safe input for preview creation. Content-dependent appearance
lookups still need valid data, and no synthetic preview has been tested.

The converter also copies six 12-byte records from summary offsets
0x3d, 0x49, 0x55, 0x61, 0x6d and 0x79 to preview-description offsets
0, 0x0c, 0x18, 0x24, 0x30 and 0x3c. If a record's first signed Dword is
negative it clears that entire output record; otherwise it copies the Dword
and following eight bytes. Their content meaning is not yet established.
It clears 12 bytes at output +0x78. The copied appearance region ends at
+0x74; the inspected converter does not write the intervening +0x75–0x77.

The item also retains a normalized summary at object +0x2d0. It passes
summary +4 to localized format ID 0xc274 and summary +5 to the icon-selection
helper `0x140311d60`, whose predicate `0x1411c6040` admits values 1–9.
Another consumer at `0x1404334a7` passes summary +5 to text selector
`0x1411c5da0` and uses summary +0x23 to choose a gendered format.

A separate named consumer establishes +4/+5 without relying on those localized
strings: `0x14036bb6d` normalizes the name-selected record into stack +0x40.
It reads +5 at `0x14036bb8e`, converts it through `0x140338420` (which calls
the established profession-name getter for values 1–9), and assigns it to
the literal **profession** key at `0x141ad0478`. It then reads +4 at
`0x14036bbbb` and assigns the numeric value to **level** at `0x141ad1a50`.
Both belong to an object published under **character**. Thus summary +4 is
level and +5 is profession; the enum labels match the creation request.
The summary-level check admits zero, unlike creation's 1–80 check.

Private REA bundle: `~/re/gw2/notes/209026-summary-preview-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_98dda9015045a29b72c47324bc6172e03a7bc1eca6a9b8c74a2f3514167e275a` | Same content-interface lookup in game loading and summary consumers. |
| `ev_c7b50f54c6c3b7c7d500d2941168acee814fa44fb2dbcdae74aaccaa793dc938` | GameLoad.cpp / m_mapDef strings and summary +1 display lookup. |
| `ev_9ea072dfbda42d47c47fa2173c27d7392050dad322d172fef426848bdc4d68eb` | Name-selected summary's numeric/text property writes and content lookup callback. |
| `ev_fae013ac7a9d9db5d3dac41c2a54e1a073208029d211696f6403228247d1d033` | Literal character/level/profession keys and profession-name conversion. |
| `ev_075501fef02b9043e9e85f87ceff4893bce8822d81dcee000084bca8de407b0d` | Character-item normalization, preview construction and display consumers. |
| `ev_0135260b715de40d51fac0b11a9648a736ac217d663493e2905271aa38bc5ac9` | Exact appearance copy and six-record conversion. |
| `ev_11510a9ab0c9c8262504d2802049a0a6bf24a3f1a2e5507b92237b838d5777d5` | Preview-description appearance passed to the builder. |
| `ev_03ac4d679b76936cf3427cc6c48b799df11109a516dc326e2c7c0f81f337f34a` | Race/sex reads and preview sex-2 assertion; icon-selection path. |
| `ev_4e9cedcd7dd0401f519be4daf2aa65162fbab841ba984dfeb83dbc27e0e56c30` | Named race/sex assertions, 1–9 predicate and summary text consumers. |
| `ev_c66c815beebbc427f8bd47338c44e59508ffc3ed37d49e558db3fbfbabe681fa` | Localized text selector and combined summary display format. |

## Word and Qword encoding confirmed statically

The encoder's switch table at `0x140ff178c` maps kind 3 (Word) to
`0x140ff05ce` and kind 5 (Qword) to `0x140ff06ae`. They copy two and eight
little-endian bytes respectively, advancing the input and wire counts by those
fixed sizes. Kind 4 (Dword) instead calls the already observed varint writer.
The census can now admit Word/Qword without guessing their boundaries.

Evidence: `ev_c3c2675a9c93befe6907945a0bf79699042f712d6a5c97247c1bff743d89c048`,
`ev_601a5ec89d41392a0dffc67e88e90cd3436486bd703461c791667ba236d77cf7`,
`ev_3d15c6d1bc295e724e111250e88d32644d6534696b01798023d1fa966cfc3413`.

## Account scalar consumers

The ID-8 handler `0x140240850` reorders decoded fields when calling
`0x14023a770`; parameter order is not wire order. That helper stores them on
the matching login transaction. Successful completion at `0x14023d8f0` then
copies them to globals. Field numbers below are the zero-based schema names,
including request field 0 and GUID fields 1/2. Structure offsets are decoded
memory offsets, **not wire byte offsets**: Dwords use varints on the wire.

| Schema field | Decoded offset | Transaction offset | Global | Established use |
| --- | --- | --- | --- | --- |
| 3 | +0x26 | +0x1b4 | `0x14266bf3c` | Account/billing category enum |
| 4 | +0x2a | +0x1e8 | `0x14266bee4` | Low byte retained; meaning unresolved |
| 5 | +0x2e | +0x1ec | `0x14266bf68` | Auth base time |
| 6 | +0x32 | +0x1a8 | `0x14266c154` | Home shard ID |
| 7 | +0x36 | +0x1a4 | `0x14266c150` | Shard-related scalar; precise role unresolved |
| 8 | +0x3a | +0x1ac | `0x14266c158` | Getter `0x14023b9f0`; meaning unresolved |
| 9 | +0x3e | +0x1f8 | `0x14266c0a8` | Meaning unresolved |
| 10 (Byte) | +0x42 | +0x1fc | `0x14266c0ac` | Normalized to a nonzero boolean |

Getter `0x14023b9c0` returns the home-shard global. Its caller at
`0x140868ef4` tests zero and otherwise asserts
`NetGameClientAccountGetHomeShardId() == 0`, in `SsdFirstDialog.cpp`.
Completion can force this global to zero under a mode/option condition, even
when the reply contains a nonzero value.

Getter `0x14023cd30` asserts nonzero `s_authBaseTime` and `s_localBaseTime`,
then returns current local time minus local base plus auth base. This establishes
field 5's role; its epoch and units have not been independently confirmed.

Getter `0x14023cd90` returns field 3. A billing-info consumer at
`0x1404d5c75`, with jump table `0x1404d5ee4`, maps 0..7 to `None`, `ArenaUser`,
`RetailUser`, `PressDemo`, `NcsoftUser`, `AlphaTester`, `PlayNC`, and
`OmegaTester`. These are observed enum labels, not recommended fixture values.

No scalar is identified as character-slot capacity; the array path below supplies
the relevant inputs. These findings do not
change runtime fixtures or advance the held agreement screen.

| REA evidence | Observation |
| --- | --- |
| `ev_a4a12ba015117b96055f66575d4924a1a4f95af448e51a4f4503ab8d24f856f6` | ID-8 handler arguments and named home-shard assertion. |
| `ev_766e87c2f9a9bfa4236f7628c48689d81c9557c937fb724aa1290e1a82f8ca48` | Transaction storage, time getter, billing labels. |
| `ev_d9c526462863fc165b19e39320fd55e47c8bd36e54641d29f0037f75d99fd222` | Successful completion copies globals. |
| `ev_3a74c0253ad157d5973c6731132f187343bb4de6865033ee76bee16f3cbf3ffd` | Scalar getters and alternate writers. |
| `ev_2dea93800e503720dbae7d529dce66284ba223ccc074beb6deff31e214027891` | Home-shard getter consumers. |
| `ev_56c9d0205b0c447dbbd4645b9585881f9513dfaf46814d5dac486b56fa7f0a00` | Time assertion names and billing dispatch. |
| `ev_583635912adeebb12523ef94de3e9a53f4bc4875e9729ba17d7cf9b7379e782c` | Billing jump table and remaining enum labels. |

## Array boundaries for the capture census

Encoder table `0x140ff178c` maps kind 17 to `0x140ff0f26` and kind 18 to
`0x140ff1058`. These write an unsigned one-byte or two-byte little-endian
element count, respectively, reject counts above descriptor +0x10, and invoke
`0x140ff04e0` on the nested descriptor (+0x18) once per element. There is no
additional element byte-length prefix. This supports inspecting the ID-8 and
ID-13 arrays of Word/Word/Dword/Dword/Dword records.

The census extension is limited to one array level with explicit, nonempty
primitive fields and an explicit maximum count. Unknown element encodings,
implicit TypeName elements and nested arrays remain rejected. This keeps the
inspection scope tied to the extracted protocol-3 definitions.

Evidence: `ev_848e50853d8278467aa95ba0a17822e888eb6451990d0957e8eb9019b3bfd6b5`
(count writes and element loops),
`ev_924936cd56ca3cbe3850121dbb36a14ed2e4abb6cf32925ad0e5c0c983015994`
(kind dispatch table). Runtime nonempty-array consumption is not yet tested.

The census extension passes its two focused tests, build and clippy. Handwritten
array frames exercise Word/Dword records with multi-byte varints and a Qword
record with a u16 count; every nonempty truncation is rejected. Oversize counts,
unverified fields, nested arrays, TypeName elements, missing maxima and
zero-byte-only elements are rejected. The saved run-58 client capture still
consumes all six messages/233 bytes. A locally reconstructed outgoing fixture
using the saved account body parses IDs 8, 10 and 7 completely; this verifies
fixture boundaries, not a captured server stream or client consumption of a
nonempty array.

## Variable-buffer maximum lengths

REA evidence `ev_5af038444e903f75301152125a4498c201ddca2c8848b55fc907ea794d83bc06`
shows encoder kind 20 at `0x140ff11f9` reading a one-byte count and rejecting
it above the descriptor's low 16-bit maximum. Kind 21 at `0x140ff1298` does the
same with a two-byte count. Both checks precede copying the payload; upper
descriptor bits serve separate flags. For the extracted schemas' explicit
`Size` maxima, the Rust dynamic codec must enforce this limit on both input
and output. Previously it read/wrote the count but ignored `Size` for variable
buffers, unlike variable arrays. Raw evidence is retained privately in
`~/re/gw2/notes/209026-buffer-bounds.json`.

The correction passes the focused buffer-bound test and the existing
variable-field test. Coverage includes exact maximum, zero maximum, every
truncation, an oversized count without a body, and unchanged output on encode
rejection. Observer and census builds pass; the saved run-58 capture retains
its six messages/233 bytes. No live client was launched for this change.

## Character-slot capacity uses the account array

The character-select scene calls `0x140311c80` before comparing the current
list count with a capacity and enabling control 2. `CharSelectList.cpp`
(`0x141aed2d0`) independently uses the same routine to size its list and
populate empty entries after the character records. This establishes a slot
capacity consumer without assigning a guessed meaning to an ID-8 scalar.

Define `A(k)` as the first matching record's second Word in the account array,
keyed by its first Word; absent keys return zero. Getter `0x14023ceb0` implements
this over the 16-byte decoded records at `0x14266c008`, count `0x14266c014`.
Those are the array supplied by ID 8 (schema field 12) at successful login.
Server ID 13 replaces the same array through `0x1402409c0` -> `0x14023a8f0`
and emits event 0x1d. The remaining three Dwords in each record are not read by
this getter; their meanings remain unresolved.

The capacity routine computes:

- If predicate `0x1410f5f00` is nonzero: `2 + A(1)`.
- Otherwise: `5 + A(1) + [A(0x3d) != 0] + [A(0x45) != 0] +
  [A(0x4b) != 0] + [A(0x51) != 0] + [A(0x58) != 0]`.

Brackets contribute either zero or one. A second capacity routine,
`0x140311d40`, returns zero when predicate `0x1410f4d30` is zero and otherwise
returns `A(0x31)`. The list sums these two capacities. Under two additional
mode predicates it displays at least five entries and at least one more than
that sum, with a distinct empty-entry presentation. These are UI display rules,
not evidence that those extra entries can create characters.

The mode predicates consult account/context state; their names and runtime
outcomes remain unverified. An empty account array therefore does **not** imply
zero base slots. This is static evidence only: the held UI has not reached
character selection, and no nonempty entitlement fixture has been exercised.

| REA evidence | Observation |
| --- | --- |
| `ev_cee327649ca6991cfb4445b5746e3c073d598749ec50b7684eb876ff5a862d03` | Scene compares current count with capacity. |
| `ev_cb76ef7ebaa3eb7a1a615e4de11f5792817e387e259eaaa7ac09c13c36ed63d3` | Both capacity formulas. |
| `ev_fd58d419693c9ccb8b0f152a78df0b767478a3d130d2f9fdf8acc8d6c208059f` | Account-array lookup and capacity callers. |
| `ev_bb7c1febd36bd638450e30017eb9a2df7ea90b60259c1b668c867043c468bf15` | List sizing, ID-13 handler, mode branches. |
| `ev_5ad62f85857a595c860ed78c77874fb78931556b7421f1b11217f895f676a7af` | Named list source, array replacement, empty-entry loop. |
