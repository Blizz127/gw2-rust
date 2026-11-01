# Build 209026: standard skill-slot level settings

Static REA executable/archive follow-up, 2026-11-01, to
[Guardian palette eligibility](209026-guardian-palette-eligibility.md).
No settings were modified and no client replay occurred.

## Frozen archive values

The five standard-slot settings selected by `0x1411c85d0` resolve by GUID
to type-150 records in archive file **1282832**. Each has declared value
type +0x28 and variant type +0x30 equal to **434 (0x1b2)**. Its signed
integer value is at +0x38.

| Slot | Setting ID | Record offset | Frozen value | Executable initializer fallback |
| --- | --- | ---: | ---: | ---: |
| 0 | 0x271 / 625 | 232584 | 0 | 0 |
| 1 | 0x27c / 636 | 347736 | 11 | 5 |
| 2 | 0x27d / 637 | 364344 | 15 | 10 |
| 3 | 0x27e / 638 | 285464 | 19 | 20 |
| 4 | 0x270 / 624 | 288464 | 31 | 30 |

The helper scanned base containers 0–7, visiting 686 type-150 records and
finding all five GUIDs. Hash-checked source files and record locations are
retained privately. This bounded scan is not a complete census of all
setting definitions or overlay behavior.

These numbers apply to the slot predicate's final level fallback. Its
stored-palette membership and other earlier successful branches can
return before that comparison. The outer palette-entry predicate still
has its own +0x1c level threshold. For example, palette 127's entry requires
level at least 1 even though slot 0's setting is zero. A setting value
alone does not establish skill availability or a valid player bootstrap.

## Descriptor initialization and resolution

The settings descriptor table at `0x1425e6980` has 0x48-byte entries.
Its selected records are zero in the executable image; static initializer
stores near `0x140217936..0x140217db6` populate their ID, type, GUID and
fallback variant. `0x1412e3590` initializes a type-434 variant and stores
the supplied integer at variant +8. This establishes the fallback column
above without assuming the table's zero image bytes are runtime settings.

Existence helper `0x1412ddbb0` verifies the indexed descriptor ID, then
calls `0x1412de200`. The latter uses a per-setting pointer cache and content
virtual +0x68 to resolve the descriptor's GUID at +8.

The slot-level caller first checks existence. **If no content record
resolves, it returns zero without asking for the descriptor fallback.**
When present, it calls `0x1412ddc30` with expected type 434. That helper
checks record type 150 and its value-type field; on success it returns
record +0x30. Its missing/wrong-type diagnostic branches instead return
the descriptor variant at +0x18. Variant getter `0x1412e3b70` returns
Dword +8 only for type 434, otherwise zero. The caller asserts a
nonnegative result before using it as a level threshold.

Thus the five archived values, initializer fallbacks and missing-record
zero behavior are three different cases. A future implementation should
preserve that distinction rather than hard-code the initializer defaults.

## Evidence

Private bundle: `~/re/gw2/notes/209026-skill-slot-levels-1.json`.
Private helper/results: `inspect-slot-level-settings.py` and
`slot-level-settings-209026-0-8.json` (mode 0600).

| REA evidence | Observation |
| --- | --- |
| `ev_7127001dfebd5801804c40b897f3da387530f78b7257b8ff462bc86db8662460` | Slot-level caller, setting existence/type lookup and nonnegative assertion. |
| `ev_30cb878d0ecb7cb7e8583aa9f079af35f5f62e53795356bc5ca9b56ee1bc5309` | Exact standard-slot setting table. |
| `ev_99f61838db99ec2146d5169f3d15f554e5556535270e2ddeb2c71f9be57df558` | Descriptor indexing, existence helper and typed variant integer getter. |
| `ev_88bfc5ca38c753360531a1ef0fca0b067b857244e34d3efffa7286f6a8eda433` | Settings lookup/type checks and descriptor fallback; initial hex dump addresses were malformed and are not used. |
| `ev_1e63b2a7329b94eed19eaf4fbb1079f04d8d232f95595d55a5168d61e65bac95` | Correct selected table addresses contain zero image bytes. |
| `ev_602a68d4a0fb60851227d3aed94691e864164f00351498107a353828785f5f63` | Candidate RIP-relative initializer stores, subsequently disassembled. |
| `ev_da10f6d4ff4e82bb255f79cd7ac3cdcb0097bc3cde95c492f9c441e58332464d` | Descriptor ID/type/GUID stores and supplied fallback integers. |
| `ev_2691b4468b0bec08e80130a6641f289e9b858d7c7c9784afcb89ad7399ff73bf` | All five matched archive records and integer-variant constructor. |
