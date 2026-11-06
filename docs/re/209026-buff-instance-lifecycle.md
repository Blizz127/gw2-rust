# Build 209026: explicit buff-instance lifecycle

Static REA binds the buff collection's actual insertion and removal methods to
server messages 734–739. These operations supply a concrete delivery contract
for future buff effects. A subsequent typed counter trace names one condition
subset, as bounded below. Durations, stacking policy, the boon class and which
effect a skill should produce or remove remain unresolved.

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

## Named condition-count subset

The complete insertion counter `1412c2c90..1412c2da9` and removal counter
`1412c2db0..1412c2f5c` receive the already-resolved tag-1 buff body. Their
direct callers are `1412c3c58` during creation and `1412c4372` during removal.
They use body Dword +0x14 as a status key in the context's +0x80 collection.
Its 16-byte rows contain the key at +0, total count at +4, condition count
at +8 and occupancy/hash at +0x0c.

Insertion increments the total count and increments the second count only
when body Dword +0x18 equals 2 (`1412c2ce8..1412c2cee`). A new row initializes
those counts to 1 and that same equality result. Removal decrements the total;
if it remains nonzero, the same +0x18 comparison selects the second decrement.
The assertion at `1412c2f39` names that field `buffStatusKey->conditionCount`,
in `CmbtCliBuff.cpp` line 272. Thus value 2 is the source-named condition-count
subset in this bookkeeping, rather than an inference from the icon style.

Both methods bypass status key 94 (`0x5e`). When the total reaches zero,
removal deletes the whole status row. These counters therefore do not prove
a total condition-instance count covering every status key. The trace does
not name the other +0x18 values as boons, inspect actual authored buff records,
or establish cleanse eligibility, priority, count or timing.

## Retained Mesmer sword buff record

A bounded cached-record follow-up resolves the type2 description facts of
both10170 and10171 to the same buff SkillDef738, UID9153. The fact payload's
typed64 external reference selects container27/file1282857, offset4625960.
The record validates type64 and kind1; its classified local+0x60 reference
resolves body offset4626176 in the same container. The checked body minimum
is0x1c bytes: status key+0x14 is78 and class+0x18 is2. Status78 is not the
excluded94, so this record belongs to the named condition-count subset above.
No complete buff-structure size is established. The subsequent bounded lookup
below resolves the localized name and the description's time input.

`gw2_dat::buff_content::BuffContent209026` now validates the0x68 SkillDef,
requires kind1 and resolves the classified body reference with a0x1c minimum.
It returns raw status/class values, the definition and both source/body
locations, preserving local/external reference ownership. It does not classify
unknown values or turn the status94 counter exception into an effect rule.
Description facts do not supply authoritative application, duration, stacking,
targeting, damage modifiers, timing or cleanse policy.

Private findings `209026-mesmer-sword-authored-buff-reader-findings-20261106.json`,
SHA256 `d936621c728d7bfc5b781c940dfa65b08e27a848598237553f50cccc64fbc619`.
Exact record evidence SHA256
`dcb95be297dc5688e62933deb6fdad40a3e0fd0d7337c3d8c453a9d734605cdf`,
REA `ev_9e65ace5129ff98da025e44249f18168504734cfe4b2caf107888c39462c7349`.
The preceding fact-reference capture is
`ev_0acb97209defd9a295ef202d07f627620d26a0f6a9db97d8a2f6eb3a06337654`.
Both exit0 without truncation and verify owned cleanup. Root checks16 distinct
referenced pins and the bounded helpers. Only retained caches are read;
unverified unrelated processes remain untouched. No archive or runtime access
occurs in this follow-up.

## Vulnerability identity and description time input

Buff738's SkillDef+0x34 is textID334824, which resolves to **Vulnerability** in
the retained localization cache. Both10170/10171 type2 fact payloads contain a
signed literal1, the typed buff738 reference and named value `Vuln Duration`
(string9477, hash2619650194). The named resolver selects time tag444 with
signed64 value5000 in all four explicit mode combinations; the override group
has no matching variable, so the base row supplies it.

Native type2 duration code resolves payload+0x60 and reads tag444 through
`1412e3ec0`. It converts the signed64 input to float, applies the selected
multiplier and divides by1000 for display: multiplier1 yields5 seconds.
The class2 branch can consult attribute9, level and modifier35, with a combined
upper cap of2. Those current context inputs are not observed here, so this is
not a claim that the effective runtime duration always equals5 seconds.
The signed1 is used conditionally as a display multiplier/count for raw
stacking type4. It does not prove authoritative stack count, stacking rules
or the meaning of message735's raw value/byte5.

This resolves description inputs, not on-hit admission, target choice, wire
duration or damage/stacking/removal policy. A bounded type2 fact reader can
now reuse the existing named-value resolver and typed buff reader; the live
increment above does not yet implement that fact reader or a gameplay effect.

Private findings `209026-mesmer-sword-buff-name-duration-findings-20261106.json`,
SHA256 `3f615913606f540cc46d7f6a695facd373ab03c307558d8721ab243d43cffc1b`;
complete bundle `209026-mesmer-sword-buff-name-duration-evidence-20261106.json`,
SHA256 `6dff206c14478e3d99db33f190bb90e0d289b073d15dd07f20d9beeb06a2b238`.
The actual cache read is
`ev_fa6662bbafffd659f415e1fa0ff9a1418ecc540321261f59ce5b49271042cf7f`.
The complete signed-time accessor and corrected duration tail are
`ev_f1dae5a1789dd4a17bb88c29f1a49ae843300e3f5335dd19045fe8be54af6733`.
Root checks the reader and referenced pins and inspects the duration fields,
modifier branch and accessor instructions. Four retained source captures have
aggregate truncation flags from rendered terminal output; their raw bytes are
fully retained and normalized frames have zero omissions. Selected code and
constant windows remain available; no blanket untruncated-capture claim is made.
Exact bounds and truncation details are retained in
`209026-mesmer-sword-buff-name-duration-coverage-addendum-20261106.json`,
SHA256 `111830d57ef42fcbdefe018480db13a1ecbe5eb0df59fc3a2dca26cc0a6ec9f3`.
The rejected first schema call and failed float-only resolver attempt remain
preserved; the successful explicit time resolver supplies the result above.

## Implementation and checks

The type-2 fact reader is commit `664e2bdcc2411b089762bce2113cfb9016fb83ad`
on integrated `f7674f1`; main carries the identical reader as `cd871d8`.
`SkillBuffFactContent209026` resolves all three values with explicit mode and
bounds, preserves their literal/base/override provenance and owning containers,
and returns signed integer/time values plus the typed buff. The release includes
the byte-identical existing named-value resolver dependency. Eight new reader
tests and nine resolver tests pass. The retained-cache Rust smoke follows both
actual sword fact paths in all four modes: integer1, time5000, buff738,
status78/class2, with time resolved from base row5. No conversion, modifier,
wire count, duration selection or gameplay policy is added.

The musl game build and both current-profile saved auth/world no-listener
checks pass, preserving all15 story arguments. The artifact is1893424 bytes,
SHA256 `8f80b79229eca44b34f17d7693853ef20a554331809bc0911b90313e7fa7b3ee`.
Private receipt `skill-buff-fact-f7674f1-20261106-checks/receipt.json`, SHA256
`42692ef1e76e0ad86d9c38560faf3f618e7139a48cf6efabd4756140c4bfff1d`.
REA `ev_f367defb41cb2da332da17e5f9f181e9a0e884f6ea096728d8aaa78aa6d3ed60`
exits0 without truncation and verifies owned cleanup; four unrelated EACCES
process environments remain unverified and untouched. Retained evidence
`209026-skill-buff-fact-rust-cache-smoke-20261106.json`, SHA256
`0f3b2edcd907bcb27a8954ddd05120d1d561eb5c6494b0592856e837c8957ef0`.
The exact artifact was handed to the sole publisher; delivery is not a live
release claim. The later [duration/level trace](209026-buff-duration-level.md)
also separates the client duration field, modifier level input and UI row count.

The typed reader is commit `11d2c560025922c3729cd89585fdf53b8508fd12`, based
on integrated release `d0288e1`; main carries the same change as `2c491ed`.
Eight focused tests cover reference ownership, classified zero offsets,
unknown/excluded scalar values, every truncated header/body prefix, incorrect
types and invalid references/ranges. A private Rust checker follows both actual
retained sword-fact paths through their original container indices, reads
buff738 with the new API and rejects the two ability records as buffs.
Unused container slots are empty synthetic sentinels, not substituted assets.

The musl game build and two saved auth/world no-listener checks pass using
the current powder profile, all15 story arguments and27-byte skills. The first
build lacked the isolated worktree's four private protocol XML inputs; copying
their verified existing bytes resolved that setup failure. No schema changed.
The tested artifact is1893528 bytes, SHA256
`0e66f19c08677df860ef52b9704f854b8a5801e7f8e72f506c1f85a3bdea2dfa`.
The sole publisher deployed that exact artifact at16:57:41Z; independent
16:59:09Z hash/health/feed readback passes with rollback and saved mode retained.
See the [release receipt](209026-saved-norn-live-activation.md#verified-buff-content-reader-11d2c56).
No new gameplay effect follows from this reader.

Private check receipt `buff-content-d0288e1-20261106-checks/receipt.json`,
SHA256 `f6cd95ce11146c1bdb17f10d582eecda71aa7dd9804a51a3a4943ef38e66aaa2`.
Actual-cache REA `ev_6d2afef38c84edf4c9838c733a904fd888062236ddd3d796d3f1f4c6b193e80f`,
file `209026-buff-content-rust-cache-smoke-20261106.json`, SHA256
`7bad3dd87401fb7cbadfd5ce20dce079d4aa022404eb3b97b4f39b3bf1649743`.
The capture exits0 without truncation and verifies owned cleanup; one unrelated
EACCES process remains unverified and untouched. Only retained caches are read;
no archive, client runtime or owner character store is accessed.

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
condition removal. The earlier `AsBuff.cpp` icon-style discriminator at body
+0x18 and mapping input at +0x14 are now supplemented by the named counter
above; the UI mask alone still cannot name the remaining classes. Body +0x0c
is explicitly stacking type; iterator bits 0x40/0x80 are owner/state filters.
None of these supplies an Ether Feast removal policy.

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
The subsequent counter findings are
`~/re/gw2/notes/209026-buff-status-class-counter-findings-20261106.json`, SHA256
`4ae0a219818b72890a000f2813147abbc2de4f9fdb78dd7bea6175598bc9ca03`.
Complete raw evidence is
`~/re/gw2/notes/209026-buff-status-class-counter-edge-20261106.json`, SHA256
`9ac171ae85a80c77ef2a87a8f05e8308e8f30cdefe50f9d1cb833848a3fc2fb5`,
REA `ev_408381750df6698a3facd781a79bd951d305437f7a7306c6d915bdf6cda9c717`.
That single target-free capture exited 0, was complete and nontruncated, and
left the frozen executable hash unchanged; string-region warnings are retained.
Implementation logs and fixture pins are under
`~/re/gw2/notes/buff-lifecycle-codec-fccd-20261106-checks/`.
