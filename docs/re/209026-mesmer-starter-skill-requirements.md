# Build 209026: Mesmer starter weapon and healing skill requirements

2026-11-05, frozen executable/archive follow-up to the
[initial palette sources](209026-mesmer-initial-palette-sources.md).
This establishes a concrete explicit palette366 experiment and the
ordinary level gates for template210's weapon. It does not establish the
retail server's initial skill selection or observe a successful cast.

## Explicit healing-palette experiment

For an existing player index1 with an initialized, owned, living level1
Mesmer character in locale0, the source-supported message order is:

1. Server605: skill data key10176, player index1.
2. Server612: palette data key366, slot0, locale selector0, player index1.

605 sets an availability bit without rebuilding. 612 stores the palette
and immediately rebuilds the owned character's skill bar. Consequently
this order establishes availability before the selection is evaluated.
605 has no flags field. Using 613 instead would introduce two notification
flags whose downstream meanings remain unresolved; it is unnecessary
for this two-message experiment.

This is an explicit candidate assignment. Mesmer type315 +0x78 and +0x80
both reference palette366, but the initial server selection policy
remains unknown. The client standard-slot route reads the player's
selected-palette arrays, initially zero, without an automatic profession
fallback in the inspected path. Equipping item33355 therefore does not
populate the healing slot.

The [605 handler](209026-skill-availability.md) requires an existing player
and a resolving type64 skill. The [612 setter](209026-player-skill-palettes.md)
also dereferences the owned character and its +0x520 skill-bar component
without null guards. The experiment belongs after the existing
[component/finalization prerequisites](209026-character-finalization-dependencies.md),
including the core stats, inventory, Mesmer profession component, player
ownership and registered skill bar. 605 can be loaded earlier; 612 cannot
be used as a substitute for those components.

The wire fields are numeric data keys at record +0x28, not record UIDs.
612's field order is Dword palette key, Byte slot, Byte locale, Dword
player index. 605's order is Dword skill key, Dword player index. Existing
typed codecs retain their framing; this investigation changes no codec.

## Why palette366 can pass the inspected entry predicate

Palette366 is type65 at file1282837 offset5371432. Its profession7,
category2 row contains one entry at5371528, referencing type64 skill10176
at file1282860 offset4074696. The concrete checks are:

| Check | Frozen input and consequence |
| --- | --- |
| Slot/category | `0x1411c81e0` maps slot0 to category2. |
| Entry minimum level | Entry +0x1c is1; the independent entry-level check requires character level at least1. |
| Mode | Raw mode +0x14 is3, which skips the mode restriction in the inspected predicate. |
| Optional conditions | Entry pointers +0x08, +0x30, +0x40 and +0x50 are null. No referenced prerequisite, chain state, auxiliary condition or specialization is imposed by those fields. |
| Other selectors | +0x24/+0x28 are23, +0x48/+0x4c are24, +0x3c and +0x64 are zero; these select the unrestricted branches of their comparisons. |
| Availability | The ordinary entry path calls adapter virtual +0x128. 605 supplies a successful availability-bit branch for skill10176. |
| Locale | Skill flags +0x38 are `0x14000006`: locale0 uses bit26 and succeeds; locale1 uses bit28 and succeeds; locale2 uses bit29 and fails; locale3 is accepted by this locale predicate. |

The complete entry predicate is `0x141212fe0..0x14121342b`, with
specialization helper `0x141213430`. A selected palette can satisfy the
slot predicate through stored-palette membership, but that does not
remove the outer minimum-level, availability or locale checks.

612 setter `0x1412291e0` stores locales1/3 in skill-object +0x88 and other
locales in +0x60. Initial character agent state1 maps to locale0 in the
[locale trace](209026-character-locale.md), so selector0 targets the array
used by the current bootstrap. Sharing an array does not make every
locale eligible: selector2 still corresponds to a skill locale that
10176 rejects. Sending all locale values is not required by this source
slice.

## Availability is explicit, but 605 is not universally necessary

The ordinary player adapter `0x1411f7b60` resolves the associated player's
skill object and calls `0x1412286a0`. Its inventory checks inspect slot26,
slot33 and gathering-related slots34/35/36/60/61/62, followed by profession
and transformation alternatives. This is not an automatic grant from
the equipped primary item in slot29. Mesmer primary vtable
`0x14217c0f0` virtual +0x118 points to `0x140168aa0`, which returns false.

The final player-skill predicate `0x1412284b0..0x14122869f` ORs the explicit
availability bit with additional mode/player-state and transformation
branches. The bit set by 605 is sufficient for that predicate, while
the other branches prevent claiming that every client state strictly
requires 605. The function still evaluates additional component getters;
setting a bit does not eliminate lifetime prerequisites. 612 does not
set the availability bit.

This distinction also affects weapon testing: `0x1411d5670` can accept
an available weapon skill before reaching its ordinary slot-level
fallback. Sending605 for every skill in a weapon palette can therefore
override the level behavior being investigated. Palette membership,
availability state and a retail unlock policy must remain separate.

## Starter weapon route and ordinary level gates

Template210 references item33355 in equipment slot29. Its discriminator24
item subrecord +0x58 points to palette90; skin3873 supplies a skin
reference, not this skill-palette selection. The
[equipment getter mapping](209026-item-equipment-delivery.md) establishes
active weapon set4's primary slot29 and secondary slot30.

The normal skill-bar route takes slots5/6/7 from the primary inventory
palette getter `0x1411f7530` and slots8/9 from secondary getter
`0x1411f7680`. Inventory virtual +0x18/+0x20 implementations
`0x14128f4a0`/`0x14128f4d0` return the respective equipment entries, without
falling back from an empty secondary entry to the primary. With only
the template's weapon in slot29, the secondary slot30 is empty.

For an ordinary, non-downed character in locale0, `0x1411c85d0` supplies:

| Skill-bar slot | Palette category | Palette source | Level fallback | Setting / file1282832 offset |
| ---: | ---: | --- | ---: | --- |
| 5 | 3 | Primary | 0 | Literal zero; no setting lookup |
| 6 | 4 | Primary | 2 | 0x273 / 319152 |
| 7 | 5 | Primary | 4 | 0x275 / 205576 |
| 8 | 6 | Secondary | 6 | 0x277 / 267552 |
| 9 | 7 | Secondary | 8 | 0x279 / 357920 |

All four retrieved settings are type150 records with declared and
embedded variant type434; their signed values are at +0x38. The helper
also resolves locale1's five corresponding settings to8. These values
extend the existing [standard-slot settings](209026-skill-slot-levels.md).
They describe the final level fallback, after earlier successful slot
branches, and are not unconditional restrictions on every skill state.
Palette90's inspected profession7 entries independently require level1.

The source therefore supports one ordinary primary weapon slot at level1
through this fallback, while the later primary slots need higher levels
and the secondary slots lack an equipped item in the current fixture.
This is consistent with the reported single weapon icon; the screenshot
alone does not identify its skill or prove every eligibility branch.

Palette90's category3 entry order is45230,10172,10171,10170. Skill45230
has a prerequisite reference and specialization59 condition;10172 and
10171 have chain references to10171 and10170 respectively. Skill10170
has null optional references and is the static base candidate. Chain
state, the active selected entry and actual cast behavior were not
observed in this source investigation. Category4 contains10334;
category5 retains its ordered conditional entries ending in10173.
No automatic availability messages are inferred from this content list.

## Exact base-sword eligibility boundary

For the archived palette90/profession7/category3 entry10170, normal slot5
selection with the resolver's bypass flag clear reduces to four Boolean
requirements. The complete retained predicate and exact content entry
eliminate its other optional restrictions:

| Requirement | Native result boundary | Reject condition |
| --- | --- | --- |
| Slot predicate, adapter virtual+0x38 | `0x14121310f` | Zero result |
| Entry minimum level, adapter virtual+0x50 | `0x141213120` | Unsigned result below1 |
| Availability, adapter virtual+0x128 | `0x141213185` | Zero result |
| Skill locale predicate `0x1411be3c0` | `0x1412131a2` | Zero result;10170 flags permit locale0/3, reject1/2 |

This is a conditional source reduction, not an observed runtime result.
A palette90 pointer alone does not prove the resolver selected profession7,
category3 or visited10170. Row selection can return null; otherwise the
first passing candidate in45230/10172/10171/10170 order wins. A null overall
result therefore cannot yet be attributed specifically to availability.
Unconditional component getters also retain their lifetime prerequisites.

The retained map is
`~/re/gw2/notes/209026-slot5-10170-eligibility-map-20261106.json`, SHA-256
`3244ed6e190ddbcf99f0b96ac135152e819c40f7bd1f89ec9f7d3850ddbc0d14`.
Its exact entry evidence is
`ev_a06197aa4d3398ac0c2c71fa017e2c3f577b9ce615a8fc41d4d814db35fd4df6`;
the complete predicate is
`ev_bba6ac6e72f39d5b4dea317d73ffa82c94e75928fa789237a298e86d1acfbb56`.
It also records the row loop, adapter and locale Evidence. No new runtime,
grant, palette replacement or gameplay behavior follows from this reduction.

## Explicit observer input

`gw2-game-observe --mission-skills FILE` accepts a bounded binary batch
of605 grants followed by612 selections. It requires an explicit population
fixture and sends the bytes once after1036/431/83, before an optional quest.
Thus the owned character and skill bar exist before612's immediate rebuild.
Duplicate readiness messages do not resend the selection.

The parser preserves the supplied bytes and checks matching player indices,
unique nonzero skill keys, nonzero palette keys, standard slots0..4 and
locales0..3. It rejects duplicate destinations even when locales alias the
same array (0/2 or1/3), grants after selections, unsupported operations,
truncation and empty batches. At least one palette selection is required
to rebuild after availability loading. Zero grants are allowed when the
caller has established availability through another path.

The4096-byte,64-grant and skill-key1..1,000,000 limits are observer resource
policies, not retail rules. The key cap bounds the client's growing bitset.
The parser cannot verify archived record existence, profession membership,
eligibility or initial selection policy. No skill is selected by default.

All49 enabled observer tests pass, with two pre-existing capture-dependent
tests ignored. Coverage includes aliased locales, grant ordering and bounds,
foreign players, truncation, startup prerequisites and once-only delivery
alongside equipment and quest initialization. Three CLI rejection checks
cover missing population, duplicate option and missing filename. The
optimized musl observer builds.

The run116 input is12 bytes:605(10176,player1), then
612(366,slot0,locale0,player1). SHA-256:
`bf7e83481d73b5acb4836a742072e5118f7001c8a065766273b64c16c5b9e55e`.
It grants no additional weapon skills. Runtime results are recorded in the
[Norn playback note](209026-norn-mesmer-playback.md).

## Evidence and limits

Executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The private, hash-checked container readers retain all visited identities.
Relevant container hashes are:

- 1282832: `63bd50638f1386c9d92ecdecf9c9260e02500d1d9ac616e33883577033c387de`
- 1282837: `efabc1cd94ec8cbb72da0f2c35c5e569b29d9414c93974de9adbc710f48ae6fd`
- 1282848: `176ae80621ebfe33adc0bd58bbce865b3d582e402890b1f624a80d0af5793576`
- 1282860: `05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23`

Private bundle:
`~/re/gw2/notes/209026-mesmer-starter-skill-requirements-20261105.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_c3dadf8759e4ee4acb3a4a6cb1d8abbdb82487f14d53fe297e900ce30a2c1fc0` | Complete slot/category and level-selection functions; availability precedes the slot-level fallback. |
| `ev_5cb990e5f3c78730e3f6c5a4909adceb2bf2d8f00902c7af24903c03dbedc30a` | Weapon-slot setting descriptor GUIDs and jump-table destinations. |
| `ev_88ad60b063a1f891e0ce6d731a2face515a283e20bde011eff9a69b8b2fbb8c5` | All16 targeted setting GUIDs resolve in file1282832; this is not a global overlay census. |
| `ev_a06197aa4d3398ac0c2c71fa017e2c3f577b9ce615a8fc41d4d814db35fd4df6` | Exact palette366 and palette90 entries, skill flags and optional references. |
| `ev_bba6ac6e72f39d5b4dea317d73ffa82c94e75928fa789237a298e86d1acfbb56` | Complete entry eligibility predicate and specialization helper. |
| `ev_8ece6f4196501560d28b6b25be8bfa480afd154572a3f50dcc1b3aba29e156dc` | Stack-adapter vtable targets for chain, availability and specialization checks. |
| `ev_f58a1246f532c007bceb91e3502d89e81ce015986d9079cf05d0c1e0f674584c` | Player availability inventory/profession route and secondary weapon getter. |
| `ev_7a151bcb90341f59b4151328486e291a2fd81aed0c83dc4842c3c95d9c54fcd6` | Complete605/612 setters and ordinary-player availability adapter. |
| `ev_0d6ef503147fdfd28e35be69cf04c73ddd98bc31e1a98c185a332e67f135e58c` | Full final availability predicate, including alternatives to the explicit bit. |
| `ev_ed8de8d1f4627b8eb4da8023836e6dfd43960f87060a52a402a6919dd5ddc68f` | Mesmer availability virtual returns false; primary/secondary equipment getters do not substitute entries. |

Captures were read-only helper/disassembly commands, with exit0 and
owned-process cleanup verified and no capture truncation. Some captures
retain process-observation unknowns where `/proc` environment reads were
denied; these do not claim a complete host-process census. No client
protocol messages were sent by this investigation. The initial retail
selection policy, broader
unlock policy, all alternative availability modes and successful combat
remain outside this bounded result.

## Earned level 2: slot 6 admission and skill 10334

The November 6 follow-up separates an existing level notification from a
missing skill implementation. The final slot predicate reads **earned** level
through `0x141207f20`, not the second/effective level. Palette90's profession7,
category4 row has one candidate, skill10334, with entry minimum level1 and
ordinary slot6 fallback level2. The retained [peer run143 observation](209026-npc-talk-loop.md)
reports the second weapon icon after191/188 awarded level2; it does not identify
the resolved skill or prove execution. Current XP progression already sends188.

In the reviewed server commit `7cfe9c404968c330407eec0dfb6121be185b71dd`,
`SkillFixture::available` remains the initial605 set. Additionally, when the
sword-chain feature is enabled, `MissionPhase::respond` applies its gate to
every skill request: only10176/slot0 or the expected chain key/slot5 passes.
Thus adding10334 to availability alone would still reject slot6. A dedicated
slot6 action must be distinguished from chain sequencing, with availability
used consistently for admission and matching-character rejoin. Granting the
whole palette at startup is not a level-unlock correction: explicit availability
can bypass the final level fallback. The authored server grant policy remains
unrecovered.

One bounded REA read of existing caches establishes the distinct action data:

| Field | Exact cached value |
| --- | --- |
| Skill definition | Type64, UID3386, key10334; file1282860 offset3881776 |
| Ability | Offset3882216; flags +0x70 =8 |
| Recharge input | Base10000; no context overrides; available-count base0 |
| Action selection | Primary token `0x4d195684f7`, variant `0x3ef495582`; alternate0 |
| Selected animation | Cached pack184788, row217904;21 events,6 segments |
| Distinct event quantities | Type3:1080; type0:1180 |

The [retained selector and initializer](209026-mesmer-sword-animation-data.md)
support that exact nonzero variant and the type0 contribution to action+0x10.
Do not substitute the sword-chain variant0. Segment flags include0x10/0x20/0x80,
so the simple one-segment heal interpretation also does not transfer. Two
type13 display facts contain integer multiplier8; one coefficient is about0.45,
the other named `Damage Multiplier` resolves to about0.128. These facts do not
establish eight authoritative strikes, their cadence, target defense, evade
behavior, or a recharge-start moment. Type2/type7 facts remain uninterpreted in
this slice. Neither an admission relaxation nor generic flat damage implements
this separate action.

Exact cached-read evidence is
`ev_4a63cf671ba4f1772c1e42371bdd4ca3aa92e51c18bc7652c6325390377d19c8`:
exit0, untruncated, owned-process cleanup verified, input hashes rechecked.
Complete private evidence
`209026-mesmer-slot6-skill10334-evidence-20261106.json` has SHA-256
`3748e49a5d77cb3b3e28e6b6d565d696e4dff375322f5b5c461ce8f15164e2b3`.
The compact source/code findings and proposed guards are in
`209026-mesmer-slot6-admission-and-effect-findings-20261106.json`, SHA-256
`d18426c39814e8533c79955084350c2b6e12c5ce5da901569cdece90931ed733`.
No archive, game runtime, gameplay code or skill policy was changed.

The retained full initializer further resolves segment flag0x10 as the start
of an accumulated range, closed by0x20: for10334, wrapper+0x28 is720+69=789
and the segment bounds at+0x2c/+0x30 are **[2,4)**. Its final wrapper+0x10
is **1180−789=391**, not the raw type0 quantity1180. The already-bound
correlated-step helper `0x141287410..0x141287492` adds789 to order+0xf0 when
the incoming step equals2. Completion `0x141285ca0..0x141285d9b` increments
order+0xec for steps2/3; its independent0x2000 release test remains unchanged.
This closes local timing/counter bookkeeping, not an evade, strike, recovery,
or authoritative timer rule. No new query was needed. Complete seven-record
retained evidence: `209026-skill10334-flag10-consumer-evidence-20261106.json`,
SHA-256 `4efda22c874af209e6a64a82cf6d251af68940c2885791dee1f95bf17da05991`.
Arithmetic and branch receipt: `209026-skill10334-flag10-consumer-findings-20261106.json`,
SHA-256 `c21bce87bd28eb060479bf288f3ad6c4d4a8916503b1f083ed841385f91442b5`.

The actual104-byte fact-row filter is now bound: list enumeration
`0x140890740` calls `0x140897c50` before forwarding the selected row through
fact-widget virtual+0x30 to the formatter. Let A be `0x14140f5d0()` or
character virtual+0x220 returning2, and B be `0x14140f9a0()` or
`0x14140f9e0()` or that character result3. Both10334 rows have+0x4c=4,
so their+0x60 masks remain2 and12. Mask0 or bit0x1 accepts all; otherwise
each active A/B requires bit0x4/0x8 respectively; when neither is active,
bit0x2 is required.
This is the fact filter itself, not borrowed named-variable override semantics.
For exact installed maps379 and28, map+0x9c bit13 is clear: the established
map gate forces character+0x220 to4 before buff/related-character overrides,
and all three map predicates are false. Consequently the normal branch selects
**base0, coefficient0.449999988, display multiplier8** and excludes the
coefficient0.128000006 row; mask12 selects that alternate row when A or B
is active. Neither row has a description-variable override group. This is
static composition for those installed maps, not a runtime sample of10334 or
the fixture's character-context return, and it establishes no authoritative
eight-strike count, actual damage, evasion or hit timing.

Complete nine-capture evidence is
`209026-skill10334-fact-context-filter-evidence-20261106.json`, SHA-256
`dcb9bc0124d43095beced3364c967cf4f89e48b431214867ef79154a7a8aea0e`.
The final findings, exact branch addresses and separately pinned map28/379
evidence are in `209026-skill10334-fact-context-filter-final-findings-20261106.json`,
SHA-256 `284f89eba78a46de2f94826d2a99fa45a8b0c6cc74f1ccd23fdf3133592eafba`.
The decisive filter capture is
`ev_26a55a7c4185df6bd239cdb61e9b35cbd726e0fb5c10fd6b8fb2f617eafa4986`;
all raw/normalized frames are retained, with duplicate rendered-frame limits
recorded. No gameplay admission or effect policy changed.

### Earned-level observation increment f926ce0 (verified live; disabled)

Isolated commit `f926ce06bd165ab912c02947e24e0bd85c4f03a1`, based on exact
`403ac377dc4606e5188bfda6a94d0c120c13efa0`, adds default-off CLI option
`--mission-observe-mesmer-sword2`. It requires validated Mesmer sword-chain mode,
positive XP per kill, primary palette366/slot0/locale0 with10176, no startup10334
and no legacy autoattack/echo/NPC cast. After a real earned188 reaches level2,
it emits60510334 followed by the existing612 rebuild once per connection.
Initial skill bytes remain unchanged; only in-memory availability is extended.
The unchanged chain gate rejects10334 before target, health, aggro or pending-heal
mutation. No accepted action, damage, recharge, strike count or timing is added.

Matching existing process-local XP carry can regrant after its replayed188,
only while the option remains enabled. There is no new persisted grant bit,
storage format or restart progression. This is fixture observation tooling,
not retail unlock policy or a playable second skill. The current profile and
supervisor are unchanged, so normal live behavior remains unchanged on release.

Three new encoded-wire/state tests cover grant order, no level1 grant or
duplicate, effect-free rejection and replay/option-off compatibility. Nine
chain/config controls, the static musl build and two no-listener auth/world
startup checks also pass. Root reviewed the full diff and verified22 receipt
pins, then handed the increment directly to the existing native publisher.
It was deployed13:00:46Z to `/home/blizz/gw2-rust-f926ce0`, rollback403ac37.
Root's independent13:03:57Z readback verifies all hashes, four healthy owned
listeners, unchanged saved mode/record/backup and the diagnostic flag's absence.
No native execution of10334 is claimed.
[Release evidence](209026-saved-norn-live-activation.md#verified-sword2-observation-release-f926ce0).

Artifact:
`/home/blizz/scratch/gw2-observe-mesmer-sword2-403ac377-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`,
1846128 bytes, SHA-256
`bffb89e2d20b68cc234520017395006940e79ee96d0bf4f73091b309e4a16d3f`.
Private complete checks:
`~/re/gw2/notes/observe-mesmer-sword2-403ac377-20261106-checks/receipt.json`,
SHA-256 `4c1527b6e01d4d6d1b07faf117a4baf62bd09dbc26263ce5b64fa6673d76c1b9`.
