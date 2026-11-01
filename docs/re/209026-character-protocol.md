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
unverified Qword field, and absence of the test string from output. Build and
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
`0x1417f4d90` with descriptor `0x142610190`. The blob format remains to map.
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
