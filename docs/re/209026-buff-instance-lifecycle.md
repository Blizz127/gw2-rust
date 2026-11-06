# Build 209026: explicit buff-instance lifecycle

Static REA binds the buff collection's actual insertion and removal methods to
server messages 734–739. These operations supply a concrete delivery contract
for future buff effects. They do not establish durations, stacking policy,
condition/boon classes, or which effect a skill should produce or remove.

The frozen executable SHA256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The existing [constructor-owned context](209026-character-buff-context.md)
remains the receiver. Message field 0 resolves a combatant through service
`1412c0720`; combatant virtual +0x40 supplies the buff context. The instance
mutations are direct calls from registered `CmbtCliMsg` handlers, rather than
insertion methods on the inspected primary query/iteration vtable.

## Registration and fields

Eight descriptor/handler pairs occupy `1421840e0..142184150`. The next address
starts the instance vtable; this registration array is not claimed to have a
null terminator. Each of the eight descriptors individually terminates with
token 0 and agrees with the extracted XML group `table-0x1421840e0`.

All fields below are unsigned varint Dwords except the final two bytes of 735.
Wire field order is listed after the little-endian u16 message header.

| ID | Fields | Handler → mutation |
| --- | --- | --- |
| 734 | combatant, buff ID, raw value | `1412c4ec0` → `1412c39a0` |
| 735 | combatant, source agent, skill key, buff ID, raw value, raw byte 5, raw byte 6 | `1412c4f30` → `1412c3a60` |
| 736 | combatant, buff ID, raw value | `1412c5040` → `1412c3f40` |
| 737 | combatant, buff ID, raw value | `1412c50b0` → `1412c3fc0` |
| 738 | combatant, optional agent, skill key | `1412c5120` → `1412c4050` |
| 739 | combatant, optional agent, buff ID | `1412c51b0` → `1412c41f0` |

The descriptor addresses respectively are `1425e55f0`, `1425e56c0`,
`1425e5830`, `1425e5900`, `1425e59d0`, and `1425e5aa0`.
Decoded native message offsets are not wire offsets for variable-width fields.

## Instance state and lifetime

735 resolves source agent 0 to null, otherwise looking it up through
`141024060`. It resolves the skill key as content type 64 and requires a tag-1
SkillDef with a nonnull buff body. Insertion asserts that the buff ID is absent,
allocates 120 bytes, constructs through `1412c23a0`, and inserts a 24-byte
key/pointer/occupied row. It increments context +0x24 and adds a secondary index
by skill key. A repeated creation is not established as an idempotent update.

The constructor installs vtable `142184160` and preserves:

| Instance offset | Input/state |
| --- | --- |
| +0x08 | SkillDef's numeric skill key |
| +0x10 | SkillDef pointer |
| +0x18 | Buff ID |
| +0x20 | Owning buff context |
| +0x28 | Source weak reference |
| +0x40 | Raw Dword value |
| +0x44 | Clock-helper result at construction |
| +0x48 | Zero-extended raw byte 5 |
| +0x4c | Zero-extended raw byte 6, named `isActive` by a later assertion |

The full active byte is stored, not normalized to 0/1. No meaning is assigned
to byte 5 or units to the raw value. Conditional owner notifications and other
bookkeeping do not establish authoritative stacking or effect rules.

734 finds an existing instance and asserts that it is inactive, then sets
+0x4c to 1. With unsigned comparisons it computes
`b = max(old_value - raw_value, 0)` and stores `max(clock - b, 0)` at +0x44.
Missing context entries return without activation. The clock helper's unit
and server sampling policy remain outside this trace.

736 ignores a missing or inactive instance. Otherwise it stores the supplied
value at +0x40, clears +0x4c, and notifies the owner through virtual +0x130.
737 updates an existing instance's +0x40, with an owner notification conditional
on the existing visibility/state predicate. Neither operation removes an entry.

738 repeatedly removes instances matching the supplied skill key. 739 removes
the exact buff ID. Both handlers resolve an optional agent, but their complete
direct mutation methods do not consume that pointer. Shared removal
`1412c4250..1412c4810` backshifts the hash rows, clears occupancy and pointer,
decrements the count, updates secondary structures/notifications, releases
the source reference and frees the 120-byte instance.

740 and 741 instead insert/remove records in a separate 12-byte value array.
740 carries two Dwords plus a byte-counted array, maximum 255, of Dword/Float
pairs. 741 carries two Dwords. These are not buff-instance creation/removal
messages and are not included in this codec.

## Implementation and checks

`gw2_proto::buff::{BuffFrame209026, BuffOperation209026}` implements explicit
734–739 encoding and transactional decoding. The six operations preserve the
complete raw domains, distinguish a skill key from an instance ID, and supply
no defaults or gameplay validation. It adds no sender, profile option, skill
effect, owner-store access or persistence change.

Four focused tests pass. Six literal wire examples are independently decoded
through the extracted descriptors; concatenated boundaries and append behavior
are checked. Every truncated example, every Dword overflow position and unknown
ID preserves the input slice on error. All 256 active-byte values and full-width
Dwords are preserved. The isolated musl game build and both unchanged-profile
saved auth/world startup checks pass without opening listeners.

This is a delivery foundation, not evidence of a native buff effect or retail
condition removal. A classifier investigation identifies buff-body +0x18 as an
`AsBuff.cpp` icon-style discriminator and +0x14 as a mapping input. Neither is
yet named as boon/condition. Body +0x0c is explicitly stacking type; iterator
bits 0x40/0x80 are owner/state filters. These must not be substituted for the
missing classifier or for an Ether Feast removal policy.

## Evidence

Private findings `209026-buff-lifecycle-source-findings-20261106.json`, SHA256
`f30bada6103c458a1efda97c2d9a329845985c5090d73f07ed794a98048dc724`;
complete 17-capture bundle `209026-buff-lifecycle-source-evidence-20261106.json`,
SHA256 `38c9fceeecd9348345e8c8b45f6c654491197a58ab0acbd677dc8357aac5bb5d`.
Decisive records include:

- Handlers: `ev_b3cdaa4e67748dc9e73c52c112fa81a5e6cf8e2aa0cff022c74e7e6d08c447e1`.
- Complete insertion/constructor and removal bodies:
  `ev_3754edac70e24c8018d982372dac794ca1fb0ef1448bc9c7e3f84c70ce229980`,
  `ev_6a28ac1361a8434cab88dd041fdbccf2b7ce426163ced0c834da7acf0e1f719c`,
  `ev_5cd175aa9409d2bf54fd81523cf4173316c4b753767ae30d91063899d82a033b`.
- Exact terminated descriptors:
  `ev_221138758166fa8b59f28a33059cf9e121f1175863ef41042f2685695268433f`.

A failed exploratory shell subcommand and incomplete neighboring disassembly
ranges remain in the bundle; claims above use corrected complete bodies.
Unverified unrelated process environments remain outside owned-helper cleanup.
No client, archive, active target import or shared runtime resource was used.

Classifier findings `209026-buff-condition-boon-classifier-findings-20261106.json`,
SHA256 `647aed8ca6a26af69af48e666cf51e848419060357c2a96d7e1056f70d1f7075`;
full bundle SHA256 `4d9b89db9d4329dea6e38e7506c7537b3b6a0ac51a7857890c86e3cbe7507ff0`.
Implementation logs and fixture pins are under
`~/re/gw2/notes/buff-lifecycle-codec-fccd-20261106-checks/`.
