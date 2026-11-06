# Build 209026: Mesmer sword damage facts and named values

The frozen sword facts contain named, context-dependent coefficients. Their
tooltip formatter also uses weapon strength, character attribute 1, two
level-indexed monster tables and a global setting. This establishes tooltip
inputs and calculation order; it does not establish damage against an actual
target, boon removal, hit timing or server chain advancement.

This static REA slice uses frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621` and
hash-pinned extracted content. It launches no client and changes no gameplay.

## Exact sword facts

All three skills are type 64, ability kind 0, flags `0x04000006`, with one
description row. Ability Dword +0x70 is zero, base recharge is zero and the
alternate animation token is zero. The first description's descriptor flags
are also zero. These values are not a recovered repeat or cast schedule.

| Skill | Record in 1282860 | Ability | First description | Primary animation token |
| --- | --- | --- | --- | --- |
| 10170 | 3965608 | 3965984 | 3966560 | `0x1789af184f7` |
| 10171 | 4263688 | 4264064 | 4264616 | `0x2789af184f7` |
| 10172 | 4032088 | 4032432 | 4032984 | `0x3789af184f7` |

The type-13 definition is type-377 record **1282851:5803328**, with text ID
48366. Each payload holds three 0x30-byte values followed by a raw Dword
selector at +0x90. The first and third values have integer tag 434. The middle
value has named-reference tag 448; its +8 is a classified string-table index,
and +0x10 is the name hash. An index such as 1673 is not a skill or stat key.

| Skill / fact entry | Payload | Base / multiplier | Coefficient name | Base float | Override float |
| --- | --- | --- | --- | --- | --- |
| 10170 / 3966704 | 3967016 | 1 / 1 | Damage Multiplier | 1.0 | 0.5 |
| 10171 / 4264760 | 4265072 | 1 / 1 | Damage Multiplier | 1.0 | 0.5 |
| 10172 / 4033128 | 4033544 | 0 / 1 | Damage Multiplier No Boons | 2.0 | 1.3329999446868896 |
| 10172 / 4033232 | 4033696 | 0 / 1 | Damage Multiplier | 1.5 | 0.6660000085830688 |

All four raw selectors are zero. All selected coefficient objects have float
tag 437. The two names have hashes **3616319567** and **1551735903**,
respectively. The last two override bit patterns are **1068146622** and
**1059749626**; the decimal spellings above retain their actual f32 values.
“No Boons” is an observed content name, not proof of a server-side predicate.

## Named-value selection

Complete logical function `0x141481d70..0x141481f8d` returns the original
value pointer immediately when its tag is not 448. For a named value on a
kind-0 ability it uses **only the first description row**, reached through
ability +0xa8 with nonzero count +0xb0. Description +8 points to the base
variable object; that object's +8/+0x10 are its variable pointer/count.
Description +0x20/+0x28 are the override-group pointer/count.

Each 24-byte override group has flags at +0, variable pointer at +8 and count
at +0x10. Complete helper `0x141481c30..0x141481d69` selects the **first
eligible group**, using these disjunctive conditions:

| Flag mask | Eligibility |
| --- | --- |
| 1 | Always |
| 4 | First effective mode boolean |
| 8 | Second effective mode boolean |
| 2 | Both effective mode booleans are false |

It scans that group's 64-byte variables in order. A row has its string/hash
at +0/+8 and its value object at +0x10. Both the raw hash and the name must
match. `0x1409b8610` delegates to `0x1409b75f0`, which folds only ASCII A–Z
while comparing UTF-16 units. Non-ASCII case folding is not performed.

If the first eligible group lacks the name, lookup falls back to the base
array immediately; it does **not** try later eligible groups. The base
pointer is loaded before the override call, but its object is dereferenced
only on fallback. The first matching base row wins. Failure logs
`Param not found for %d`; there is no valid default value. The selected
value is returned without recursive named-value resolution.

The mode booleans are not universally equivalent to raw character context.
When `0x1414100a0` / `0x141411600` succeeds, they are `context == 2` and
`context == 3`. Otherwise they come from `0x14140f5d0` and
`0x14140f9a0 || 0x14140f9e0`. Once exact
[map 379 is installed](209026-skill-context-map-gate.md), its clear enable
bit, null +0x100 reference and map type 7 make both booleans false. Thus the
four **base** coefficients above apply to this content selection regardless
of the supplied raw context on that gate. The sword groups all have flags
12, so either effective boolean being true selects their overrides.

## Damage tooltip calculation

Dispatcher `0x140891f50` uses the actual Dword RVA jump table at
`0x140892a58`. Data type 13 selects `0x140892929`, validates the definition
through `0x140894930`, then calls `0x140893c50..0x140893f4d` with the entry's
+0x40 payload. The formatter resolves all three values through the helper
above before accessing their integer/float representations.

For flags **mask 0x08** set, the formatter takes a separate flat calculation
path. This is bit 3, not bit 8 (`0x100`). The three sword SkillDefs have
mask 0x08 clear and use the ordinary path:

1. Obtain level through tooltip context virtual +0x18, a weapon pair through
   `0x140895550`, and unsigned attribute 1 through context virtual +0x10.
2. Add `s_monsterDefenseLut[level]` and `s_monsterAttributeLut[level]` as
   Dwords, then convert the zero-extended sum to f32. Their accessors are
   `0x1412df560` / `0x1412df520`, with tables `0x1421d6360` /
   `0x1421d61c0` and an asserted level below 101.
3. Add the two weapon Dwords, interpret that sum as signed for conversion,
   multiply by f32 0.5, then add the signed fact base. Multiply this by
   unsigned attribute 1 divided by the table sum.
4. Multiply the global base-damage setting by the fact coefficient, then
   multiply by the previous result, preserving that single-precision order.
   Clamp the result to at least f32 1.0.
5. `0x1409bf050` adds f32 0.5 and converts toward zero to a signed 64-bit
   integer after its range assertion. The caller uses the low Dword and
   multiplies by the integer fact multiplier before formatting text ID
   `0xba90`. This is not a claim that arbitrary invalid/overflowing inputs
   should be emulated by unchecked host casts.

The setting lookup uses GUID bytes `a551d0cd0b4d3947b668fd13347d1016`,
checks declared type 437 at +0x28 and reads the value at +0x30. Its actual
frozen value is [1.0 in the exact setting record](209026-damage-tooltip-number-setting.md).
The constants at `0x141924c58` and `0x141924c5c` are exactly 0.5 and 1.0.

Payload selector 1 or 5 substitutes slot 5; selector 2 substitutes 8;
selector 3 substitutes 12; selector 4 substitutes 1. Other values preserve
the caller's slot. Therefore these four zero-selector sword facts require
the caller's slot. The inspected fact-widget path loads widget +0x78,
passes it as dispatcher argument five, and then as the damage formatter's
third argument. This slice does not identify the widget field's initializer
or assert that every display path supplies slot 5.

The denominator is derived from level tables, without querying a target's
armor. This calculation must therefore remain distinct from actual combat
damage. The weapon pair also requires the concrete item/context branch;
copying an item's authored pair is insufficient when its scaling tag is set.
The [starter weapon audit](209026-starter-weapon-tooltip-inputs.md) derives
105–117, mean 111, for the inspected equipped-item/static-player branch,
and retains the caller-selection and player-state qualifications.

## Implemented resolver and verification

`gw2-dat::ability_value_content::resolve_ability_value209026` implements the
kind-0 lookup above. It accepts separate checked skill/value locations,
explicit mode booleans and limits for group count, combined selected/fallback
rows and UTF-16 name length. It returns the exact 0x30-byte value plus
original/base/override provenance. It does not interpret the returned scalar,
resolve recursively, load files, choose a map mode or execute a skill.

Classified local/external offset zero and string index zero remain valid.
Selected array extents are checked before traversal. Unsupported skill kinds,
missing names, malformed consulted references, invalid UTF-16 and budget
overruns fail closed. Native short circuits are preserved: literal values
do not touch skill data, override success does not touch the base object,
and later groups/descriptions are not substituted. These defensive checks
are stricter than the native pointer/assertion behavior on malformed input.

Nine focused tests passed with
`cargo test -p gw2-dat --lib ability_value_content -- --test-threads=1`.
They cover precedence and empty groups, all mode masks, hash collisions and
ASCII-only equality, external ownership, zero indexes, bounds, malformed
inputs and nonrecursive returns. The final module SHA-256 is
`adde4604e074c6b7c14397838be73bb3c087591b0f15474825bfc08e512039d2`.

A separately compiled private harness then matched the independent cached
parser for **16 named cases**: four sword facts across all four boolean
pairs, including exact value bits, offsets and base/override provenance.
It also checked **eight original integer values** with invalid skill
locations and zero lookup budgets, proving literal passthrough. Both the
module and inputs were hash-checked unchanged around the comparison.
Final REA evidence:
`ev_8af75b912bfff31edc28c71ee04afabdd4251e57ba1b3a920bdad37c91962199`.
The run exited zero with no truncation; no gameplay or archive access occurred.
No server binary or held run145 input was changed by this reader increment.

## Type-13 reader and numeric evaluator

The follow-up `gw2-dat::skill_damage_fact_content::DamageFact209026` now
connects the exact type-377/data-type-13 definition and payload to the named
resolver. It reads the base, coefficient, multiplier, raw weapon selector and
the SkillDef's **0x08 mask** selecting the flat formula. The supported resolved
scalar tags are integer 434 / float 437 / integer 434, matching all four sword
facts. Other tags, missing direct definitions and definition-key fallback are
not silently substituted. Limits apply separately to each scalar lookup.

`weapon_selector(caller_slot)` preserves the caller's slot for raw zero and
unknown values. `evaluate` requires explicit weapon strength, attribute 1,
both level-table values and base-damage multiplier for the ordinary path.
The flat path ignores those inputs. Neither API supplies an implicit weapon,
map mode, character level or target armor. Flat multiplication occurs before
rounding; ordinary multiplication occurs afterward. The implementation uses
the observed f32 add-0.5 conversion, not `f32::round()`.

The evaluator rejects nonfinite values, zero/overflowing table sums, a weapon
sum exceeding signed Dword, and conversion/multiplier overflow. These are
bounded-reader policies; it does not emulate native wraparound or invalid
conversion results. It implements the numerical tooltip, not combat damage.

Seven focused tests pass:
`cargo test -p gw2-dat --lib skill_damage_fact_content -- --test-threads=1`.
They cover external definition offset zero, the actual definition discriminator,
mask 0x08 versus 0x100, malformed ranges/tags, selector preservation, unsigned
attribute conversion, separate multiplier/clamp ordering, an f32 half-boundary
that differs from `round()`, and arithmetic rejection. Final module SHA-256:
`18dd023a2c93fbec133bc18509fdd5b5c389f24ff6b2a16df5b414f863b454bd`.

A private compiled checker read all four actual sword facts in all four mode
pairs: **16 decoded cases**, **48 numerical cases** and **three non-type-13
rejections**. Container slots 30 and 21 contain the exact skill/definition
caches; all other slots are genuinely empty indexes. A separate Python parser
and explicit f32-operation oracle match every field and result. They use the
frozen level-1 table entries **128 and 10**, the verified setting 1.0, the
conditional weapon pair 105/117, and explicitly chosen attribute samples
0/100/1000. The samples are not observed live attributes or tooltip captures.

For example, the base 10170/10171 facts give 1/81/812 for those three attribute
samples. The 10172 “No Boons” fact gives 1/161/1609 and its other damage fact
gives 1/121/1207. These are conditional numerical checks, not authoritative hit
amounts or a recovered boon predicate. Input and module hashes remained
unchanged during the check; no archive or game runtime was accessed.

Exact table evidence:
`ev_b944317be57897884cf8f363cbc564e3b4f7c1d46033ef826a422c1c7ebd246a`.
Final compiled comparison:
`ev_7e0b1b3a60ea234d91f213486ddb0f406331dc2ce816baff8a00b7234c5dbeb3`.
Both captures exited zero without truncation. Private bundle:
`~/re/gw2/notes/209026-damage-fact-reader-20261106.json`, 6807526 bytes,
SHA-256 `1e169033afc0ff32faa52d1e15b76ac49d6bc51a36206b9054bf5ba7023c08a5`.
The helper is `~/re/gw2/notes/check-damage-facts-20261106.py`.

## Mind Slash follow-up: map 28, tooltip 30 and chain boundary

A bounded follow-up confirms the same **base coefficient 1.0** when frozen
Wayfarer Foothills map 28 is installed. Its exact type-45 record is
**1282830:3731704**, with Dword +0x9c = 0, map type +0x2c = 5 and a null,
unrelocated +0x100 reference. The container SHA-256 is
`e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f`.

`0x1414100a0` tail-calls the established map gate `0x141411600`; its
bit-13 check is false for this record. The named-value fallback predicates
are also false: `0x14140f5d0` receives null +0x100, `0x14140f9a0` accepts
types 9–15 except 13, and `0x14140f9e0` tests type 18. Thus both effective
mode booleans are false for this exact installed map, as for map 379.
This proves the named-value selection, not a sampled runtime map or the
result of an unrelated context consumer.

The repeated cached read of skill 10170 confirms **base 1, coefficient 1.0,
integer multiplier 1**, and raw weapon selector 0. Applying the existing
tooltip calculation to the explicit conditional inputs below gives:

| Tooltip input | Value |
| --- | --- |
| Equipped-item static-branch strength | 105–117, mean 111 |
| Effective attribute 1 | 37 |
| Second coreStats level | 1 |
| Monster defense / attribute table entries | 128 / 10 |
| Global multiplier / resolved coefficient | 1.0 / 1.0 |
| f32 result before add-0.5 conversion | 30.02898406982422 |
| Formatted integer | **30** |

This is a derived tooltip sample, not an observed hit or a justification
for fixed 30 damage against every target. The formatter uses the mean
weapon pair and a level-table denominator, without reading actual target
armor. Its attribute and level inputs are **effective attribute 1 and the
second level**, not earned attribute 187 and earned level 188. The earned
attribute curve does not establish effective scaling or authorize swapping
those columns. Target defense, authoritative strike arithmetic, weapon-roll
policy, modifiers, hit outcome and server effect timing remain unresolved.

The same follow-up closes two previously unexpanded events from the
[exact sword action record](209026-mesmer-sword-animation-data.md):

| Event / raw quantity | Character notification / listener slot | Installed ordinary-view path |
| --- | --- | --- |
| 7 / 120 | `0x1411dbc50`, +0x3e8 | `0x14110c050` → `0x141131370` |
| 9 / 400 | `0x1411dbca0`, +0x3f0 | `0x14110c0d0` → `0x141131530` |

Both callbacks belong to the established AvCharBase listener table
`0x142141558`, assert the passed character matches the view's character,
and operate on whole-view +0x2b8, **AvCharEffect**. The helpers assert
`streakIndex < arrsize(m_streaks[0])`. Event 7 uses index 0, clears/replaces
two retained streak-handle slots, and calls `0x14112b7c0` for two effect
classes. That complete helper selects item/skill visual-effect rows, calls
the effect manager and retains returned handles. Event 9 uses index 0 and
calls the manager's virtual +0x30 on those handles with a zero third
argument; it does not itself clear their slot fields. The deeper effect
implementations are not exhausted here.

These concrete paths supply no damage transaction or chain-state write.
The skillbar's constructor-installed +0x20 interface, table `0x142165158`,
has no-op `0x140163990` in both matching slots; its subscription caveat
remains as recorded in the [finish-listener note](209026-typed78-finish-listeners.md).
This is bounded coverage, not a census of every dynamic listener. The
shared event loop scales local event time and can force dispatch, so 120,
400, the type-3 quantity 240 and the type-0 quantity 900 must not become
server hit times merely because they are authored quantities.

The [628 predecessor contract](209026-mesmer-sword-chain.md) remains exact:
10170 with count ≥1 can select 10171; 10171 with count ≥1 can select
10172; count 0 or predecessor 10172 returns these entries to the base
candidate, assuming earlier entry 45230 is ineligible and the remaining
palette predicates pass. The newly traced
events do not establish **when to advance, reset or expire that state**.
628 alone also does not prove follow-up availability. No chain trigger,
additional grant, damage formula or gameplay default was implemented.

Private findings: `209026-mind-slash-damage-chain-boundary-20261106.json`,
SHA-256 `7c1746c9d42f1b98e25116b378820ab656b1a0ccde8a30475d99b564c21c421b`.
Full evidence under `~/re/gw2/notes/`:
`209026-mind-slash-damage-chain-boundary-evidence-20261106.json`,
342439 bytes, SHA-256
`0e8ea7eb119f8d66872a00794a5a7ad48c827bdb03e1e368b7def6d1f6658377`.

| REA evidence | Follow-up scope |
| --- | --- |
| `ev_a276ec5e5a9e5ace5d0fe3e8f9e046e22b97afbb2a9a4e48bb48554186748709` | Hash-checked retained map-28 fields, with archive fallback forbidden. |
| `ev_caa0c6b75cc6272291c68d4bcee28cae8ec77fce8120fa4988c4673308472749`, `ev_bcd5f564818e99ada761c7ad514b2d13c445a9b1b13ac9764d86c9fc5931262a` | Exact fallback predicates and map-gate alias. |
| `ev_863b686d92b2650e482351a31690af1c67207e947b086e5228d5695c6c4fbf60` | Cached 10170 fact and explicit f32 tooltip-30 sample. |
| `ev_c4fa901a0d3550de5218eece3f80dc9be95ef164ac076e6eb46f7a8b9e894e5a`, `ev_cf086bf0fa382537b38719f7119b54ec002c7b905cf492b42a82c595a43b1061`, `ev_26b0050d4e0349158575264bec1438f76fdfec20955690dabca6dbfcdfe67685`, `ev_5b5f15345ba90dec2e282c47a39d3e5533dafd5c7211bc92d59c8809a4d61038` | Event table, notification wrappers, listener slots and argument dispatch. |
| `ev_ad3653ecd13793c797ad39721ab8a0b624e43aa1fb9832442653cbb3c5c49cd0`, `ev_ee96b03dc7052c05f17de75da2e190c9e7bb6f72714a15da9fa822852ada10f7`, `ev_310524348e192289846bfec7146e0584275fab4c7082c843ba313bace6963700`, `ev_befabf2b94d3bdfd426c77a0bda93c08b6f4d146c5a38f82dff54f5b75c039aa` | Installed view callbacks and concrete streak-handle paths. |
| `ev_584b47dd49dc282b16d3e35f0fb56b1eb239b68c321539ecb68bdfec86fb7185` | Exact skillbar slots and named streak assertion. |

All 13 captures exited zero with complete raw/normalized output and
verified owned-process cleanup. One duplicate rendered frame was omitted;
some unrelated process environments were unreadable. No host-wide cleanup,
runtime observation, archive access or release change is claimed.

## Evidence

Complete private source/check export:
`/home/blizz/re/gw2/notes/209026-sword-named-damage-facts-final-20261106.json`,
6671969 bytes, SHA-256
`266cbbb44595f30543c5a2a416b48b22731b43b0a088da72ec0e9e6636435bff`.
It retains earlier session evidence as well as this slice. Initial partial
windows and an earlier code-byte table dump are retained but are not used
as the jump-table proof; the exact RVA table and completed logical bodies
listed below support those claims.

Pinned cached containers:

- 1282860, `content-type-147-file-1282860.pf`: 9571548 bytes, SHA-256
  `05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23`.
- 1282851, `content-type-377-file-1282851.pf`: 11525676 bytes, SHA-256
  `acb0a858f427daf612baebdc999fdcb0299c2ac50ddd8e731dace8caa0c4d547`.

The private cached-only helper `read-sword-fact-kinds-20261106.py` independently
checks the records, relocation classifications, strings, named-variable rows
and all four explicit boolean combinations. It has no archive fallback.

| REA evidence | Scope |
| --- | --- |
| `ev_1139614cc70d841fc2a761ec769eb5e6b3f728ddec017c5ef4e0a23aeae2433e` | Exact cached sword facts, ability flags, named rows and mode combinations; raw frames complete, repeated rendered view truncated. |
| `ev_05858a45ed56b9aeca0cfd3c57797c56015afe7329e493477390b6d80a5f6b77` | Complete named resolver. |
| `ev_2a03da2f1b671f59914658f2aaa7e5f8d6653a1ad88dcf0b85b96666d9ae469e` | Complete first-eligible-group helper. |
| `ev_a93f603f54aa69acb34dcab349b955dcb8462470dc16e70393c7fce58e73d2f6`, `ev_21c4b15915fccd124b59711498a03b67086404418f8644371c793308ea37e3be`, `ev_1d97aa97a8c9f364f9070be45a76c41c12bd59ee08c39e253508633de32c7893` | Name getter, effective mode leaves and complete ASCII-only UTF-16 comparison. |
| `ev_953c37bf53469e5fae78579a67e45c02cedbf35f35254e8c4272e979c1fd95a3`, `ev_81d5e0f048e13d6a69949d53f2ea2d8aa3c273ac76470efe5efbc76682f8d50c` | Actual jump table and type-13 dispatch. |
| `ev_63863cda1f820485a152823c2a4af9e574fcc9b4f90463b7e2627814e7a13994`, `ev_f9631581b3c81b80eb1bb758bbde575e0fab407176d45dad3c9cda140a231a79`, `ev_5a4de395193eca196e8586a67bba9f22002fb28c04677a66efa16e79091c8988`, `ev_af698ab39642ab21134c04f5f95498e027ebead86751a38e84cd2c404c3ba3c7` | Formatter fragments covering the logical body through its return. |
| `ev_4fe68cbd78a93d4f900626a191a7fba3f284be96048b3acca76cbd260e835b75`, `ev_e8a53ef5d589fa717accb57b3f297f7ba1aa567582af12cdad77403c0068d821` | Rounding, table accessors, weapon-pair helper and exact float constants. |
| `ev_c15225bc8d6c2d846173d8dfec4de949d570359ce6a6bebe96598a1b6434fd11`, `ev_9daa3b5aecdebf34985e87d9e3ec9a90305bebe8da6613754b6fa7f821dabf19`, `ev_80481642a1957e7dc2508d969c914f400757615697fd34f86b505cba845de31c` | Bounded direct-call candidate and verified widget-to-dispatcher slot argument; no initializer claim. |
