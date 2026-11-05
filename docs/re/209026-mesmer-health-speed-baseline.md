# Build 209026: Mesmer baseline health and movement-speed limits

2026-11-05, bounded static REA follow-up on the frozen executable and
archive. **The client calculates a level1 Mesmer baseline of388 using its
default attribute curve.** An account configuration flag selects an
alternate curve yielding258. This establishes a client baseline calculation;
the final maximum delivered through server256 remains server-supplied and
may include additional state or modifiers. No source-backed movement-speed
replacement was established. No bootstrap code or runtime state was changed.

## Health formula

`CharConst.cpp` function `0x1411c46e0` takes a profession-definition pointer,
a level, and an unsigned attribute input. It requires level>0 and, when
the profession pointer exists, profession enum<11. For a nonnull profession
whose enum is not10, its calculation is:

```text
health = round_client(f32(playerHealthLut[level]) * professionCoefficient)
         + 10 * attributeInput
```

The table accessor `0x1412df660` checks level<101 against the explicit
assertion `level < arrsize(s_playerHealthLut)`, then reads a Dword from
`0x1421d6d20 + 4*level`. Its level1 entry is20. Null profession or enum10
instead selects `s_monsterHealthLut` through `0x1412df5a0`; that branch is
not used for Mesmer enum7.

The coefficient comes from profession +0xb0 if `0x14140f5d0` is true,
otherwise +0xb4 if either `0x14140f9a0` or `0x14140f9e0` is true, otherwise
+0xac. These mode predicates need not be named to evaluate the inspected
Mesmer record: **all three coefficients have identical f32 bits0x3f666666**
(approximately0.9). The record is type315, file1282838 offset2655160,
profession enum7 at +0xa8. The corresponding Guardian coefficients are
all0.25; those values are profession metadata, not movement parameters.

`round_client` is `0x1409bedf0`: it checks the float lies between-0.5 and
4194303, adds f32 12582912 (bits0x4b400000), and returns the resulting
float's bit pattern masked with0x003fffff. It uses the executing client's
floating-point rounding environment; it is not an integer truncation.
For the inspected level1 input, the multiply is exactly f32 18.0 and the
returned integer is18, so a tie-breaking assumption is unnecessary.

Two validated direct callers of this formula were found. Consumer
`0x14088f1bc..0x14088f243` obtains coreStats virtual +0x10, computes the
level-table attribute value through `0x1411c3980`, obtains the profession
definition through character-context virtual0, and tail-calls the formula.
CoreStats virtual +0x10 reads the **second level field**, not earned-level
virtual0; both levels are1 in the current candidate bootstrap. Another
caller, `0x1411c7540`, supplies the same level-table value, then applies
additional content multipliers. The latter is not evidence that the raw
baseline is the final health maximum in every context.

## Attribute curve and configuration selector

`0x1411c3980(level)` resolves configuration ID396 (0x18c) with expected
type252 (0xfc). The descriptor at `0x1425ed8e0` is populated by initializer
stores `0x1402135a9..0x1402135e7`. Its GUID, in wire byte order, is:

```text
7d af 96 c6 bf 4a 71 4d b4 d0 ea 51 c9 8f 4d 4e
```

The existing configuration lookup `0x1412dde50` resolves that GUID and
accepts a direct type252 record or an appropriate type150 indirection.
The frozen base-container scan found exactly one direct matching record:
**file1282841 offset175152, type252**. It has these two references:

| Layout field | Type391 table offset | Array header offset | Count | Entry1 |
| --- | ---: | ---: | ---: | ---: |
| +0x38 | 176280 | 176336 | 101 | 24 |
| +0x40 | 173616 | 173672 | 101 | 37 |

All these records are in file1282841. `0x1411c3980` requires table kind1
at +0x28, follows table +0x30 to the array header, bounds the supplied
level against header +8, then reads the level-indexed Dword. The adjacent
leaf `0x1411bf730` uses this same curve for attribute indexes below5 and
returns zero for indexes5 and above. The specific attribute enum names
are not established by this trace.

The selector is `0x1412de110`: with a configuration context at TLS +0xd8,
it returns bit1 of context +0x1910. True chooses layout +0x40; false
chooses +0x38. Constructor `0x1412de160` clears the context and explicitly
sets mask0x2, so a newly constructed context selects +0x40.

The bounded direct-call census found one caller of selector setter
`0x1412de080`: **game-server17**, handler `0x1410f63b0`, registered in
table `0x14213fe70`. This is separate from auth-server17's character
record. Its descriptor is ArrayVarLarge(max10) Dwords. After replacing
the account configuration words, the handler sets the selector to the
inverse of account configuration flag47, then notifies character context.
Flag getter `0x1410f7e20` forwards flag47 to `0x1410f7550`; the mapping
entry at `0x14213f850 + 47*8` is word index1, mask0x00020000. An absent
word returns false. The flag's product name is not assigned here.

Thus the source-derived level1 calculations are:

| Configuration case | Attribute contribution | Mesmer baseline |
| --- | ---: | ---: |
| Constructor default, or account flag47 clear | 10×37 | 18+370 = **388** |
| Account flag47 set | 10×24 | 18+240 = **258** |

This is a meaningful baseline input for an explicitly selected bootstrap
model. It does not establish a live retail account's flag words, equipment
modifiers, adjusted maximum, second health meter, or server256's extra
float. In particular, the [health constructor and updates](209026-character-health-endurance.md)
accept current/maximum from the server; they do not automatically replace
those values with the calculation above. The existing1645 placeholder has
no support from this baseline trace.

## Movement-speed result

The [movement parameter trace](209026-agent-movement-parameters.md) remains
the authoritative consumer evidence: server37's first movement-array float
initializes agent +0x138 and current/previous speed +0x15c/+0x160. Collision
setters divide that value by32, and their getters multiply by32. This names
the parameter and its distance basis without selecting its initial value.

The follow-up rechecked setter `0x14102cc70`: it compares the supplied
float with +0x138 and, when different, queues an index0 parameter command
through `0x141037fb0`. It neither obtains a profession coefficient nor
calculates a level-dependent base speed. A validated direct E8/E9 census
found no direct callers; the function is installed at vtable pointer
`0x14211ace0`, so this result does not exclude indirect callers or other
content-selected movement rules.

Bounded named-string and consumer searches did not identify an initial
player movement-speed setting or a formula yielding294. The health-related
profession coefficients above do not establish one. **294 remains an
explicit research placeholder**, and successful movement alone does not
validate its retail accuracy. This is not a whole-client proof that no
baseline speed definition exists; further work would need a typed
indirect caller or content-setting consumer that chooses the first
server37 parameter.

## Evidence and validation

Executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Relevant archive container SHA-256:

- Mesmer profession, file1282838:
  `1976719c73b108ce13d775e4fed797931deacedf754213a1c11d229afd647781`.
- Attribute layout/tables, file1282841:
  `62ef0c04651578ca4cf6a4322fe4cb792bd865b614069d639858986b13594e39`.

Private bundle:
`~/re/gw2/notes/209026-mesmer-health-speed-baseline-20261105.json`.
Read-only helpers verify executable/container hashes, follow retained
relocation tables, and preserve evidence privately. The final evaluator
reads the source values and reproduces the f32 multiply/add/bit-mask path;
it reports388 and258. No runtime value was sampled or modified.

| REA evidence | Observation |
| --- | --- |
| `ev_50ae91d3ce43012384247fb5b11461580d994494c5e19a39161cd406025341b2` | Named health-table assertions and all nine profession scalar triplets. |
| `ev_f8b22c1036bee83fa393d9bddda1878d2c37dd07e006a5ba9507611eddf1e679` | Exact player/monster health LUT accessors and bounds. |
| `ev_aa86872100a289d2ec725571b23b2f777376a40e785795229fb00174e0e820c3` | Complete health formula, coefficient branches, and ten-times-attribute addition. |
| `ev_9bf116df06e1ec472cec359cc0b13dccc00f42251ac4d0b4c91a76bf88242a66` | Level1 health-table value20. |
| `ev_8a2d343217fed4ea705fc895388958b2cdc0791ea5ed65a4df9fdfbedc5b244d`, `ev_8d4701670108e5a215d6cb9d6d81d9626a035d3efe68d47df4d65ea8a7b9c5f3` | Rounding instructions and exact constants. |
| `ev_ad3c52afd79b043112c2ba8a71b0f18810fc3f4c25fceec0d29f3b4ed82554f9` | Two validated formula callers and level/attribute argument provenance. |
| `ev_5f2ecbb3220c78fb69ecb26b85a7f28a540838b5ff8a9fe848e9ffff3c28dae3` | Configuration396/type252 resolution, table selection, and indexed value read. |
| `ev_559a90aeaed76f8beb13b1c20e94d4f96af49bac792ff96cfe9c988bc0432468` | Configuration descriptor GUID stores and common attribute-curve leaf. |
| `ev_6dab71c0f6e5478c730f6663389d09571fe8c9211b629c42d9e8efa73e8fce32` | Exact layout and both table references; level1 values24/37. |
| `ev_b0728aefbaf92e8c3419b7a4cdf05313941cbc0386bffa83f3ba5a0c618e9aee`, `ev_da4577eeb604dbfa1c0da2500bdabe01b7b91b7b231480b08eada8126c771f76`, `ev_2a10e83d1af9cf409f14e42abe47fbe92ad8e6c565332c195ea8ad571a6936cc` | Remaining base-container batches find no second matching type150/252 GUID. |
| `ev_2e6589d53fac1b00474c0ac4d769d4e53f2f323e6ca3560898e1ff2a202d4f5f` | Configuration context constructor and selector bit stores/reads. |
| `ev_95a8de2385242e56c2a5142f345f506ed86a5391fa609a9ae13f389bd8ce7247`, `ev_020763d8dc60c078d4321fc47100eb500a7a220e0274c551ba95f57e170f568f` | Server17 selector update and account configuration flag lookup. |
| `ev_063584613a349f0c14b4f85a0e57ec7a06de08d1a0431e4414b21a9b55a6de13`, `ev_d904112c6f4d2b67798f0ff36b8d29b2357fb0481f86640a1fd3f008700c4267` | Speed setter's virtual installation and complete supplied-value queueing path; account flag47 leaf. |
| `ev_5ed4856c62fd2baf41301d658b47691f93ae57d50acf8825d416362741d709fb` | Source-reading f32 evaluation:388/258, and account flag47 word/mask mapping. |

All cited completed process captures exited0 with owned process-group
cleanup verified. Truncated instructions beyond selected leaf boundaries
were not used as evidence. No cold decompiler analysis was started.
