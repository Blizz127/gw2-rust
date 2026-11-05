# Build 209026: starter equipment and the Mesmer health profile

2026-11-05, bounded read-only REA follow-up. The four retained starter
items, with explicit prefix112 and no upgrade items, add **zero equipment
attributes in the inspected client calculation**. They provide no reason
to increase the [level1 Mesmer baseline388](209026-mesmer-health-speed-baseline.md).
The previously found content multiplier is a skill health-threshold
check, not an alternate maximum-health calculation.

## Concrete candidate profile

A source-based, explicitly unmodified bootstrap can use first-meter
current=388 and maximum=388 under these conditions:

- Profession7, earned and second level both1.
- Default attribute curve: configuration selector true, corresponding
  to constructor state or account flag47 clear.
- Items6465/6467/6464/33355 with prefix112, null primary/secondary upgrade
  references and an empty additional-upgrade list.
- No extra health/attribute modifiers, and initially full health, as
  explicit server-model choices.

The arithmetic remains `round_client(f32(20) * f32(0.9)) + 10*37 = 388`.
The account-flag alternate curve yields258 instead and must not be
silently merged into this profile. The no-modifier/full-health choices
make388 a defensible explicit experiment value; they do not prove the
initial retail server maximum. Server256 still supplies the actual
current/maximum independently.

This result does not select first-meter rate, second-meter values, F5,
endurance or movement speed. Those fields require their own evidence or
explicit experiment inputs. The1645 placeholder remains unsupported by
the inspected health and equipment sources.

## Typed item-to-prefix-to-attribute path

The [retained creation sources](209026-item-equipment-delivery.md) and
fresh archive checks agree:

| Item key | Kind | File / content offset | Subtype prefix field |
| ---: | ---: | --- | --- |
| 6465 | 0 | 1282841 / 1497064 | +0x38 |
| 6467 | 0 | 1282841 / 1504936 | +0x38 |
| 6464 | 0 | 1282841 / 1519368 | +0x38 |
| 33355 | 24 | 1282834 / 3014352 | +0x60 |

All four references resolve to type37, data key112, file1282843 content
offset2968184. This is also the explicit prefix supplied by the current
equipment fixture: the creation message replaces the constructor prefix,
so retaining the archive reference alone would not establish instance
state.

Armor's upgrade interface is at item +0x98; weapon's is at +0xa0. Their
initializers `0x1413cbb30` / `0x1413cbe40` store the resolved prefix at
interface +8. Upgrade virtual0 is `0x14016bb50`, which returns that
pointer. A concrete typed consumer at `0x1408b7a78..0x1408b7b85` obtains
item virtual +0x18, calls upgrade virtual0, and passes its result as
argument3 to `0x1413cca50`. The weapon virtual +0x18 implementation
`0x1413cb730` returns item +0xa0. This establishes pointer provenance;
it is not based on matching field offsets alone.

`0x1413cca50` computes an item scaling value through `0x1413cde50`, then
passes the prefix to `0x1413cc990`. The latter selects the prefix's
array at +0x30/count +0x38, or +0x40/count +0x48 when its alternate-mode
argument is nonzero and that alternate count exists. Each entry is8 bytes:
an attribute index followed by a float coefficient. For each entry it
adds `round_client(max(1.0, itemScale*coefficient))` to the indexed
attribute accumulator. In particular, a zero coefficient in an existing
row would not be equivalent to an absent row.

Prefix112 has the stronger, empty-list representation:

| Prefix field | Retained value |
| --- | ---: |
| Primary row pointer +0x30 / count +0x38 | null / 0 |
| Alternate row pointer +0x40 / count +0x48 | null / 0 |
| Effect reference +0x50 | null |

Both row selections therefore execute zero additions, independent of
the item scaling formula. The separate additive-effect helper
`0x1413cc690` returns when prefix +0x50 is null. Its caller is visible
immediately after the ordinary attribute calculation in the retained
item consumer.

`0x1413cca50` also handles primary/secondary upgrade subrecords and an
additional-upgrade list. Null upgrade arguments and an empty list skip
those additions. These are explicit fixture inputs; this trace does not
establish a retail initial-upgrade policy. The bounded source-reading
evaluator checks all four item/prefix references, both empty row headers
and the null effect reference, and reports zero contribution in all12
attribute columns. It does not emulate arbitrary item stats or assert
that the items have no armor, damage or other gameplay properties.

## The multiplier caller checks a skill threshold

`0x1411f6a10` in `ChCliSkillbar.cpp` asserts a nonnull `skillDef` input.
It obtains the character health interface through primary virtual +0x68,
the core-stats profession/second level, and character virtual +0x228's
selector. It passes the skill definition, profession definition, level
and selector to `0x1411c7540`.

For skill discriminator +0x58 equal to0, `0x1411c7540` computes the same
level-table baseline through `0x1411c3980` and `0x1411c46e0`. It follows
skill +0x60, searches that subrecord's +0x110 pointer / +0x118 count for
an8-byte selector/value row, and uses the matching float or default float
at +0x54. The result is:

```text
threshold = round_client(f32(baseline) * selectedFloat * f32_bits(0x3c23d70a))
```

The final constant at `0x141934804` is the f32 representation of0.01.
Other skill discriminators return zero through the inspected branch.
The caller then reads health virtual0, the current first-meter value,
and returns whether current health is strictly greater than this computed
threshold. There is no maximum-health store in either function. This
clarifies the earlier multiplier finding without naming every skill
subrecord field or inferring an initial server modifier from it.

## Explicit population health input

`PopulationPlan.initial_health` now supplies the living character's full
first meter in server256. The builder requires a finite positive value
and chooses no profession-specific health default. The observer exposes
`--mission-health VALUE`, which requires an explicit population fixture;
without the option it preserves its legacy1645 research value. The Norn
source-profile experiment selects388 explicitly. Rate, second meter and
F5 retain their separately labeled research zeros.

Twelve population tests pass, including an independent frame decode that
checks current=max388, unchanged auxiliary health fields and byte-identical
unrelated frames. All49 enabled observer tests pass, with two pre-existing
capture-dependent tests ignored. Eight CLI rejection checks cover nonfinite,
zero/negative, missing, duplicate and population-free arguments. The musl
observer builds. Runtime results belong in the
[playback note](209026-norn-mesmer-playback.md).

## Evidence and limits

Executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The source-reading checks verify retained container hashes:

- 1282841: `62ef0c04651578ca4cf6a4322fe4cb792bd865b614069d639858986b13594e39`
- 1282843: `ac91841287684a0a9262ec53b2e095498448276da6f0208d53421675024b75ea`
- 1282834: `fc03eee83e799747ebed6f13c5445c353e5bfd1198105b1bc0048c8359013232`

Private bundle:
`~/re/gw2/notes/209026-mesmer-starter-equipment-health-20261105.json`.
Private evaluator: `mesmer-starter-equipment-attributes-20261105.py`.

| REA evidence | Observation |
| --- | --- |
| `ev_c6b2905d6988b457e979ae8139b9d651a5e3da394e4b54fa553734d7a041e3cc` | Complete skill threshold multiplier `0x1411c7540`. |
| `ev_f5d29eec00f3458edceaf72a84ad8adbf233e920809041d50fc9a9af5aa0f061` | `skillDef` assertion and boolean comparison with current health. |
| `ev_f3e5e2edcbbfb475795f8f2f6c3eb7d8191f0c182345fc5f78c17dde9c37651d` | Exact0.01 multiplier bits. |
| `ev_7ee4af0b628abfc5068e7a6ff502e93688c929e74d141a2a9ddedd0582ec02e2` | Prefix setter and upgrade storage. |
| `ev_a0de766599e13e68ad4b8a4465e1d928fb5e2d886c3444bfd7b10197d89666eb` | Upgrade virtual0 returns the prefix; weapon virtual +0x18 target. |
| `ev_68603352ae570ad6fa0cd76fea77fc4eaaa243f014e6753bb1750cc294bae9ac` | Item scaling helper and named rarity multiplier assertion. |
| `ev_ddbf88635b6c59a79ddb32e777cdca8af3f4bcf2f951ee8fac51f333534d96f1` | Verified direct callers linking item scaling to the prefix accumulator. |
| `ev_fdde5870d358169e57332cdec76d6e553ef65f1c9c0fb1b3d236c1fe095f6fa2` | Verified accumulator callers and typed runtime-prefix flow. |
| `ev_3f976ef839c03bb8f78ab499ac246554329fa10d06cadd51b9672164205b8e5a` | Complete row accumulator, upgrade iteration, effect helper and focused typed caller slice. |
| `ev_c2bf43086cd44ad9898f31a422fcb9025052c9e8f469907ae570dfee9af0287f` | All four source references and zero prefix contributions, with hashes. |

These captures exited0 and retained complete process output. Function
continuations split by PE unwind records were followed with bounded
instruction slices; trailing jump-table bytes and adjacent partial
functions were not interpreted as code evidence. Owned process-group
cleanup was verified; process-observation permission limitations remain
in the bundle where reported. No runtime values or server modifiers
were sampled by the source investigation; the explicit observer input is
documented separately above.
