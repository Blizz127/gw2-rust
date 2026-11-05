# Build 209026: Mesmer initial palette sources

2026-11-05, frozen archive and hash-bound executable inspection. This narrows
the [Mesmer candidate palettes](209026-mesmer-palette-candidates.md) without
yet proving the initial server selection. No gameplay messages were sent by
this investigation.

## Direct profession references

The Mesmer type315 record is in file1282838 at content offset2655160;
its profession enum at +0xa8 is7. Its six direct type65 references are:

| Profession field | Palette data key | Row profession/category | Skill data keys in entry order |
| --- | ---: | --- | --- |
| +0x58 | 357 | 7 / 17 | 10200 |
| +0x60 | 358 | 7 / 17 | 10201 |
| +0x68 | 362 | 7 / 17 | 50440, 10203 |
| +0x70 | 410 | 7 / 0 | 10245 |
| +0x78 | 366 | 7 / 2 | 10176 |
| +0x80 | 366 | 7 / 2 | 10176 |

All listed entries have minimum level1. Raw mode is3 except skill50440's
entry, which has mode2. Both +0x78 and +0x80 point to the **same** record,
file1282837 offset5371432. Palette and skill numbers here are data keys at
+0x28, consistent with [612's numeric lookup](209026-content-numeric-ids.md).
They are not record UIDs.

For comparison, the Guardian definition at file1282842 offset3831408
references palettes138/309/310/311/259/259 at the same six offsets. This
supports a shared profession-record layout. The field names and the rule
selecting between +0x78 and +0x80 remain unproven. Palette366 is therefore
a stronger source-backed candidate than an arbitrary eligible palette,
but these references alone do not prove a fresh character's server state.

The Mesmer record also has an array at +0x88 with count92 and another at
+0x98 with count5. Bounded relocation inspection rejects treating either
as a simple Qword palette-pointer array. The first array's first five
relocations are spaced24 bytes apart and reference type64 skills
10192/10287/10190/10191/10275. The second begins with composite data,
including type64 references13725 and13727 spaced72 bytes apart and nested
arrays. Their complete schema and selection role remain open; count5 is
not evidence of five initially selected standard slots.

## Creation template weapon route

The already extracted [level1 Mesmer equipment template](209026-creation-equipment.md)
contains item data key33355 in raw template slot29. Its item record is
file1282834 offset3014352, with discriminator +0x2c equal to24. Following
item +0x30 reaches file1282834 offset3014528; subrecord +0x58 reaches
type65 palette90, file1282848 offset5452584.

This is the exact discriminator24 route implemented by `0x141212ee0`.
Palette90 contains these profession7 rows, retaining entry order:

| Category | Skill data keys |
| ---: | --- |
| 4 | 10334 |
| 5 | 62675, 62568, 62675, 62568, 10337, 10173 |
| 3 | 45230, 10172, 10171, 10170 |
| 6 | 10358, 10280 |
| 7 | 10174 |

All inspected entries have minimum level1. The first four category5
entries have raw mode2; the remaining entries have mode3. Duplicate keys
are retained because entries may carry different conditions.

This proves the archived template-item-to-palette relationship. It does
not prove the template's server selection policy, the inventory equipment
slot mapping, actual equipped runtime state, or which entry passes all
eligibility conditions. It does not justify sending palette90 to an
arbitrary standard slot through612: the [normal rebuild](209026-character-skillbar.md)
obtains weapon palettes through inventory interfaces.

## Runtime assignment boundary

The inspected normal standard-slot route has no automatic profession
fallback. Adapter `0x1411f79e0` calls `0x1411f7970`; after its character
predicate and associated-player lookup, that helper calls the player's
embedded skill object's virtual +0x58 or returns null. Implementation
`0x141227c90` returns the selected pointer directly from +0x88 for locale1/3
or +0x60 otherwise. Its accepted standard slots are0..4, with a separate
slot21 branch. No type315 +0x78/+0x80 fallback appears in these functions.

The [player palette trace](209026-player-skill-palettes.md) establishes
that those arrays start zero and server612 sets them. Thus content
membership and client eligibility do not supply the server's initial
assignment. A playable bootstrap still needs explicit server-chosen
palette state, plus availability and component prerequisites.

The type315 getter `0x1411b5270` returns character-context
`+0x310 + profession*8`, guarded by profession<11. Its pointer is at the
primary context vtable's first entry, `0x14215dfc8`. A validated direct
E8/E9 reference census found no direct callers; indirect virtual calls
must be traced through the character context. A bounded field-use census
alone does not identify typed consumers or establish that these fields
are unused elsewhere.

A follow-up traces the concrete `TLS -> character context +0x98 ->
virtual0` pointer flow within `0x1411b0000..0x1412d0000`. Confirmed
consumers include specialization traversal at `0x1411cc740`,
`0x14120d6c0`, `0x141224f00`, `0x141225080` and `0x141225220`, using
+0xb8/+0xc0; skill/trait membership at `0x1411ce4c0` and the continuation
after `0x1411ce67b`, using +0xc8/+0xd0; and the skill-list iterators
`0x1412275a0` and `0x141227970`, using +0x88/+0x90 with stride24.
The latter establishes the first composite array's stride in executable
code, rather than inferring it only from relocation spacing.

The skill-bar flow at `0x1411f6a10` passes the returned profession pointer
to `0x1411c7540`, which forwards it to `0x1411c46e0`; the inspected path
uses profession +0xa8 and +0xac/+0xb0/+0xb4 scalar fields. None of these
confirmed flows supplies a +0x78/+0x80 palette to the player arrays.
This bounded trace therefore leaves the **initial selection policy
unresolved**. It is not a whole-program proof that the two palette fields
have no client consumers.

## Evidence and limits

Executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Archive helpers check retained container hashes before following relocated
references. Relevant source hashes include:

- 1282838: `1976719c73b108ce13d775e4fed797931deacedf754213a1c11d229afd647781`
- 1282837: `efabc1cd94ec8cbb72da0f2c35c5e569b29d9414c93974de9adbc710f48ae6fd`
- 1282834: `fc03eee83e799747ebed6f13c5445c353e5bfd1198105b1bc0048c8359013232`
- 1282848: `176ae80621ebfe33adc0bd58bbce865b3d582e402890b1f624a80d0af5793576`

The private bundle retains all visited container hashes and command
captures: `~/re/gw2/notes/209026-mesmer-initial-palette-sources-20261105.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_beafc4c250b8038ef862a761936d7284d1613dc794b68e29ca2e7f54c41ee81c` | Profession direct references, template item palette90 and its profession7 rows. |
| `ev_f8f635bfae89129e99073123620fa8a642f7a8b6ea47aab562bd7ee269a4d3aa` | All six direct Mesmer palettes and exact row entries. |
| `ev_f57300746a163930fc651dbc767682d0c3d5e0e925de05421bd015821a98467b` | Bounded composite-array relocation shapes; no inferred initial slot list. |
| `ev_d216fd6bbc4837ff6ef67655830138c0c7a1ac8f000c8d92b42afeafa2374fee` | Exact profession getter, player palette getter, standard-slot adapter and item-palette helper. |
| `ev_68157faf38c89bffb948d061a8e200c6a8e27125e12438eeb97e5c24341851e6` | Direct-call census and profession getter's vtable reference. |
| `ev_40e12eb6e0f108bbc8a0dce3ffb5d7dba7c6a2a694e59d5ab38482e24f9792e7` | Bounded context-virtual0 pointer flows; manually distinguished unrelated virtual calls. |
| `ev_2a59a295558486be66526893de23cab55ec00cc8ba355d35d09d5ce54f39fa43` | Follow-up helper and continuations for the scalar and composite-list consumers. |

These were read-only reader/disassembly captures, with exit0 and owned
process-group cleanup verified. No cold decompiler analysis was started.
The note does not claim a retail initial skill choice or working combat.
