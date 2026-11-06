# Build 209026: literal amount facts and Ether Feast tooltip evaluation

Static REA, 2026-11-06, against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This follows the [animation-data boundary](209026-mesmer-heal-animation-data.md).
Only retained frozen content, the frozen executable and the exact missing
container 1282851 from the frozen archive were read. No client runtime,
working archive, container, gameplay or deployment was changed.

**The selected 10176 description has two literal type-zero amount facts.**
Their formatter derives a numeric tooltip amount from explicit level and
selected-attribute inputs. It does not establish authoritative healing,
cast completion, recharge scheduling or a response to predicted command 0x53.
The bounded implementation is
[`skill_fact_content.rs`](../../crates/gw2-dat/src/skill_fact_content.rs).

## Exact content and selection

Skill 10176 is type 64 at content offset 4074696 in container 1282860.
Its +0x58 tag is zero and +0x60 points to body 4074944. Body +0xa8 points to
description rows at 4075496, with count +0xb0 equal to one. That row's +0
points to descriptor 4075544 and +8 to variables 4076728. Descriptor +0x20
points to four 0x68-byte fact entries at 4075640; +0x28 is count four.
Descriptor flags +0x30 are zero and override pointer +0x58 is null.

| Entry offset | Definition target | Payload offset | Definition data type |
| --- | --- | --- | --- |
| 4075640 | Container index 21 / file 1282851 / offset 5810816 | 4076056 | 0 |
| 4075744 | Same definition | 4076344 | 0 |
| 4075848 | Container index 21 / file 1282851 / offset 5810144 | 4076632 | 7 |
| 4075952 | Same type-seven definition | 4076680 | 7 |

Definition pointers are classified external references in entry +0x28;
payload pointers are classified local references in entry +0x40.
A definition's content type is Dword +0x10: both targets are type 377.
Its data type is Dword +0x2c. The complete formatter
`0x140891f50..0x140892a57`, crossing its pdata fragments, dispatches on this
definition data type through `0x140894a50..0x140894ad4`. Type zero at
`0x140892013..0x140892020` forwards entry +0x40 to `0x140892ab0`.
The cached entry +0x48 tag is not that dispatch discriminator.

The definition resolver `0x140894dd0..0x140894e3f` can also look up a null
definition reference by a key. The bounded reader requires a direct classified
definition and does not implement this fallback or global catalog selection.

Definition 5810816's text ID +0x28 is 23939, decoded in the retained text export
as “Healing.” Entry 4075744 overrides its text with ID 38037, “Heal per Clone.”
The type-seven entries have text IDs 277876, “Conditions Removed,” and override
1121465, “Conditions Removed per Clone.” Their payloads each contain literal
float 1, but the new reader intentionally rejects type seven.

Container fingerprints:

- 1282860: `/home/blizz/re/gw2/notes/content-type-147-file-1282860.pf`,
  9,571,548 bytes, SHA-256
  `05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23`.
- 1282851: `/home/blizz/re/gw2/notes/content-type-377-file-1282851.pf`,
  11,525,676 bytes, SHA-256
  `acb0a858f427daf612baebdc999fdcb0299c2ac50ddd8e731dace8caa0c4d547`.
- Selected text export: `209026-mesmer-re-handoff-20261105/plain-texts.json`
  under the private notes directory, SHA-256
  `c6199a19c78e24c2efb656f06857ec2c49ea49b4a6905c4c858ac07a55b8dfaf`.

## Six-value literal layout and numeric contract

The type-zero payload is six consecutive 0x30-byte value objects. Each has a
Dword tag +0 and value +8. Complete amount formatter
`0x140892ab0..0x140892cc2` reads them in this order:

| Payload offset | Accepted literal tag | Meaning in this formatter | Primary | Per clone |
| --- | --- | --- | --- | --- |
| +0x00 | 0x1b2 signed integer; alternatively 0x1b5 float | Base, float truncated toward zero | 120 | 0 |
| +0x30 | 0x1b0 enum | Attribute selector | 6 | 6 |
| +0x60 | 0x1b5 float | Attribute coefficient | 1.0 | 0.1 |
| +0x90 | 0x1b5 float | Level exponent | 2.0 | 2.0 |
| +0xc0 | 0x1b5 float | Level coefficient | 0.85 | 0.1 |
| +0xf0 | 0x1b2 signed integer | Integer multiplier | 1 | 1 |

The stored float bit patterns are 0x3f800000, 0x40000000,
0x3f59999a and 0x3dcccccd, respectively. Literal accessors are
`0x1412e3b70` (signed integer), `0x1412e3940` (enum) and
`0x1412e3c90` (float). The native accessors have fallback values on wrong
tags; the reader rejects those unsupported inputs instead of manufacturing a
literal from a fallback.

For a nonzero selector, the formatter calls context virtual +0x10(selector).
Its Dword result is converted as a **signed** integer. Selector zero directly
uses zero. Context virtual +0x18 returns the level; its Dword is zero-extended
before conversion, so the evaluator accepts an **unsigned** level.

Let `A` be the explicitly supplied signed selected-attribute value, `L` the
explicit unsigned level, and `R` the native contribution conversion:

```text
attribute_part = R(f32(A) * attribute_coefficient)  // A := 0 if selector == 0
level_part     = R(powf(f32(L), level_exponent) * level_coefficient)
amount         = (level_part + attribute_part + truncated_base) * multiplier
```

The addition order above follows the source. The two contributions round
separately before the integer additions and multiplication. The complete helper
`0x1412c0e90..0x1412c0eb1` adds 0.5 for positive input, subtracts 0.5 otherwise,
then performs truncating signed conversion. **The half adjustment itself uses
single precision.** Consequently the immediately preceding f32 below 0.5
(0x3effffff) becomes 1 after the adjustment and conversion; 0x3efffffe becomes
0. A generic language `round()` call would not faithfully express this boundary.

The power helper is `0x140e5d3a0..0x140e5e265`. Its error path
`0x140e805e4..0x140e80733` passes the exact “powf” name at `0x1420c92b0`.
The Rust evaluator uses platform `f32::powf`; arbitrary-exponent last-bit
identity with the frozen client's math library is not claimed. Tests cover the
exact exponent-two facts and explicit rounding boundaries.

Nonfinite values, invalid signed conversions and intermediate integer overflow
return errors. This bounded behavior deliberately does not reproduce x86 invalid
conversion sentinels or integer wraparound. Public float bases retain their
representation and are truncated only during evaluation. No text localization
or signed-magnitude formatting is implemented.

The preceding value resolver `0x141481d70..0x141481f8d` performs named-variable
resolution for tag 0x1c0. All twelve values in these two facts are literals, so
that branch is not taken for them. The formatter obtains a mode through the
character's virtual +0x228, or a fallback if no character is supplied; the new
reader does not infer that mode or generalize these literals to every description.
It rejects named variables and all other unsupported tags.

## Independent data check and scope of the observed 121

The source formula and the real frozen records give:

| Explicit level | Explicit selector-6 value | Primary amount | Per-clone amount |
| --- | --- | --- | --- |
| 1 | 0 | 121 | 0 |
| 80 | 0 | 5560 | 640 |
| 1 | 5 | 126 | 1 |
| 1 | 100 | 221 | 10 |
| 80 | 100 | 5660 | 650 |

Root separately observed the level-one fixture's Ether Feast tooltip displaying
121 and 0 in run 144, with health 388. That screenshot was admitted by
`ev_538379a4b2347f3fe1b199467be147c6ba552c2728602320fdbf67f690853515`,
SHA-256 `43b7821b3960bf3e00a01f2d160c4af3d68a6cb87298af52a07b1157fc0083dc`.
The explicit input (1,0) is consistent with the display. **The concrete tooltip
context adapter and its actual selector-6 return were not captured.** Neither
that attribute's name nor a formula deriving it from health is established here.

Eight focused tests cover the hand-authored literal layout, local/external
references including classified zero offsets, malformed reference classes,
missing/short targets, type/tag rejection, finite conversion limits, separate
rounding, zero selector, float-base truncation and integer overflow. Public test
fixtures contain semantic tag/value rows; exact frozen payload bytes stay private.

An additional private checker called the new reader on the actual frozen entries,
with the exact definition container at slot 21 and source container at slot 30.
Unused slots were aliases, not a claim to reconstruct the complete container
catalog. It verified both classified references and all six fields, evaluated the
five explicit input pairs above, and rejected the two actual type-seven entries.
The independent REA check exited zero:
`ev_2f944e3e08c7e849f1f037bc1dd1e231d495e03d6d11adf0cef5cd244462fb66`.
The checked module SHA-256 was
`ed5f7bcfd674fb972be3448cace79da202581240b506c6db7308f2af64a8d123`.

## Recharge display is a separate contract

The exact 10176 body has Dword +0x90 = 20000. Its +0x160 points to two
16-byte context-override rows, count +0x168 = 2:

| Row offset | Raw context selector +0 | Raw value +8 |
| --- | --- | --- |
| 4077008 | 2 | 25000 |
| 4077024 | 3 | 25000 |

The complete logical recharge-widget update `0x14089a0f0..0x14089a363`
selects a matching row for the character context, otherwise body +0x90.
It applies the separate modifier helper `0x140898050..0x140898226`,
converts the product through `0x1409bee70..0x1409bef06`, divides by
float 1000 at `0x141924c6c`, and passes the result to child widget 6.
Description flags +0x30 bit 8 can hide the display; that bit is clear here.

With an explicitly assumed modifier of one, the base yields 20 and selectors
2/3 yield 25. **Actual modifier inputs and selector meanings remain unresolved.**
The observed 20 does not alone identify the active context. This display path
does not select server 630/631 fields or prove a recharge execution schedule;
none are wired by the fact reader.

## Evidence retention and limitations

The complete 36 source/data captures and bounded readers are retained in
`/home/blizz/re/gw2/notes/209026-mesmer-tooltip-facts-20261106.json`
(1,290,084 bytes), SHA-256
`d98abb2f14b8093ac8869d493f2ffe027f88d3605946834a9fbf87d03b2f7e1b`.
This includes the initial cached-only failure for missing container 1282851,
then the exact frozen-ID extraction; the failed attempt is not treated as evidence
of an absent shipped definition. A two-byte-offset string-reference attempt was
also superseded by the corrected exact `SdCost.cpp` reference.

Key complete captures:

- Fact dispatch and exact raw records:
  `ev_321043154ed1aea16f02c95403c195f94be3aa884f5f75a70173781da8e6bc3c`,
  `ev_b0c37e412471198fcb4ebb1bcff537106e2fc6df7101dda516eb82942e9c79f2`.
- Amount helper, rounding and definition accessors:
  `ev_cda0ae2dbf7fa94a45ba1dca9ac47fed1d0f0919599771819869b17c10b5b99c`,
  `ev_f98f51713e60781557c900e178a2a65486b0bc500178441447ec2e5d17eafbe2`,
  `ev_7055a0fbfc31412d37fb69b2ef1103d2da1354febc8a006b70a98e649166c446`.
- Recharge widget and actual overrides:
  `ev_ed5f1c9c05f7e4c518f3fc52b58d1941d4f42f1261ca34f98c6ac1bdc080e6dd`,
  `ev_26268bbdab6c95ab56c558a21757fcef971bcdad3e0805c14b77d8c0d22929e1`.

The separate real-reader check, source and executable pins are retained in
`/home/blizz/re/gw2/notes/209026-mesmer-tooltip-facts-reader-check-20261106.json`
(28,559 bytes), SHA-256
`7063408c1e704a618b524a46e84314410315bc11c8d9726e9e17c405e09744b3`.
Raw PTY evidence is complete; some duplicate rendered terminal frames were omitted.
These static and offline-reader observations do not assert client runtime
execution of the new code, or identify authoritative server skill effects.
