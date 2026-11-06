# Build 209026 ability recharge content reader

The bounded reader in
[`skill_recharge_content.rs`](../../crates/gw2-dat/src/skill_recharge_content.rs)
reads an ability's base recharge, category-one maximum count, and ordered
context overrides directly from classified content references. It provides
the content inputs needed to replace hard-coded skill values. It does not
apply modifiers, choose server timer fields or establish the category-zero
wire Word.

## Exact source selection

The type64 skill's +0x58 discriminator must be zero. Its +0x60 reference
then supplies the ability body. The source consumers establish:

| Content field | Meaning in the inspected consumer |
| --- | --- |
| Body +0x90 | Base recharge content value, before the recharge-display modifier. |
| Body +0x160 / count +0x168 | Ordered16-byte override rows: context Dword +0 and recharge Dword +8. |
| Body +0xf0 | Base maximum count used by the category-one recharge consumer. |
| Body +0x190 / count +0x198 | Ordered8-byte override rows: context Dword +0 and count Dword +4. |

The complete recharge-widget update `0x14089a0f0..0x14089a363` compares
context keys at `0x14089a1a0` and takes the **first** matching row. Its
match path at `0x14089a234` reads row +8; no match reads body +0x90.
Only after this selection does it apply the separate modifier, numeric
conversion and division by1000 for the displayed seconds. This reader stops
before those later operations.

The complete count helper `0x1412125a0..0x1412125ce` uses the same first-match
rule over8-byte rows, reads row +4 on match and body +0xf0 otherwise. Neither
lookup sorts or deduplicates the array. A matching zero is a real override,
not a reason to fall back to the base. The exact context is a caller-supplied
raw Dword; no general mode-name mapping is introduced.

These details extend the existing
[recharge consumers](209026-skill-recharge-consumers.md) and
[tooltip selection](209026-mesmer-tooltip-facts.md) contracts. The new REA
capture is `ev_766b1737e5bc5ef82979ab9e237bcc26f69f2851a33e505360a07eda7c2fbc34`.
Both claimed bodies are complete in retained raw output. Adjacent partial
functions printed around the count helper are not used as evidence.

## Reader boundaries

`SkillRechargeContent209026::read` takes an explicit ordered container set,
source container, skill offset and combined override-row limit. It rejects
missing/short records, a non-type64 definition, a non-ability discriminator,
null nonempty arrays, string/unclassified references, absent external
containers, out-of-range target extents and excessive combined counts.

After resolving an external ability body, the two array pointer slots are
interpreted in that **body's container**, rather than the original skill's
container. A classified local/external reference to offset zero remains
distinct from a null pointer. Array allocation follows checked resolution
and an explicit count budget. A zero-length array can have a null reference.

All consumed scalar values remain full-width unsigned Dwords, including
zero and the maximum value. The reader preserves override order. Selection
methods return the first matching value or the base without clamping,
inventing an override, loading an archive, selecting a gameplay context, or
emitting a packet. Other skill-body variants are explicitly unsupported.

## Exact Ether Feast record

An independent bounded Python traversal and the compiled Rust reader agree
on the cached frozen type64/key10176 record at offset4074696, body4074944,
in file1282860:

| Value | Observed frozen content |
| --- | --- |
| Base recharge |20000 |
| Recharge overrides, in order | Context2→25000; context3→25000 |
| Base available-count maximum |0 |
| Count overrides | Empty |

The reader returns20000 for explicit contexts0/1/4/u32::MAX and25000 for2/3.
The count maximum is0 in every case. The independent
[map gate](209026-skill-context-map-gate.md) establishes context1 once map379
is installed, selecting20000 before display modifiers.

The count result also narrows the pending implementation: the inspected
category-one setter returns without inserting/updating/removing a record when
the selected content maximum is zero. For this exact frozen ability there
is no count override to change that result. It does not replace the ordinary
category-zero cooldown path or choose its unresolved Word.

The cached file is9,571,548 bytes, SHA-256
`05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23`.
The actual-content check also verifies that a one-row budget rejects the
two-row record and that the source module and cached file remain unchanged.
REA: `ev_65573bab2b7e2b15a68eea18c27c15df2e493cb6d053ef1f98fda371773bdcd3`.
This is an offline reader check, not a native cooldown observation.

## Validation and retained evidence

Four focused tests pass with
`cargo test -p gw2-dat skill_recharge_content --lib`; the library build passes.
They cover duplicate-key first-match behavior, zero overrides and full raw
values, external skill-body/array ownership including classified offset zero,
null/empty distinctions, malformed types/references/extents and combined limits.
Public fixtures are hand-authored schema data; exact frozen bytes stay private.
An initial test-helper call signature error was corrected before these passes.

Private checker project and capture wrapper:

- `/home/blizz/re/gw2/notes/recharge-content-check-20261106/`
- `/home/blizz/re/gw2/notes/check-skill-recharge-content-20261106.py`

Checked module SHA-256:
`51c775ffe41e8765d7a656d4e47a6808c5e4e70477309ed36c6e9a541f01ab98`.
The private evidence bundle
`/home/blizz/re/gw2/notes/209026-skill-recharge-content-reader-20261106.json`
includes both new source/check captures among the retained session evidence;
SHA-256 `65922c91566ed5e8b9c4812cd4b82311f665184889e77d88355945e242966700`.
Both new captures exited zero with complete raw output and verified owned
process-group cleanup. No runtime, shared archive, deployment or saved-mode
activation was performed.

## Display modifier boundary

A bounded follow-up recovered the complete modifier at0x140898050. Query
0x76 returns a skill-filtered sum V and accepted-row count N. The native
single-precision sequence is T=0 when N=0, otherwise
T=V/(float(N)*100)-1, then factor=max(float(N)*T+1,0). When the raw caller
selector is12..18, a separate query0x74 supplies P and divides the factor by
P/100+1, with an assertion P>-100. Algebraic simplification can change
rounding. N counts accepted modifier rows, not clones.

The character adapter's iterator is0x14088e1d0, reached through character
virtual108, interface virtual40 and collection virtual28. Its actual contents
remain unknown; it is not the separately traced virtual50 buff collection.
An empty iterator would yield factor1, but emptiness has not been established
for this character. The widget forwards its incoming raw selector; this
slice does not prove that every caller supplies an equipped-slot index.

Consequently authored20000 cannot yet be advertised as an observed20-second
cooldown. Category-zero's Word and accepted-heal timing remain unresolved.
The earlier map379 context1 proof also does not establish map28's context.
The planned nonzero-duration lifecycle test remains diagnostic, without
healing or recharge replies.

Private findings: `ether-feast-next-execution-boundary-20261106.md`, SHA-256
`c20828e812ebf194394ecf843584a9a3ba0523924643f6b5338d849649047ecb`.
Complete seven-capture bundle:
`209026-ether-feast-recharge-modifier-20261106.json`, SHA-256
`57b51fbe31690733f5d95b91a35639ce0c0bbdcf23f51ba0e96ab725158afab1`.
All captures exited0. One duplicate rendered stream was truncated; raw and
normalized output remain complete. No gameplay or runtime change occurred.
