# Build 209026: skill 10176 animation lookup and exact records

Static REA, 2026-11-06, against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This follows the action-selector boundary in the
[predicted skill order note](209026-predicted-skill-order-lifecycle.md).
Only the frozen executable, exact frozen archive index entries and one selected
pack were read. No working archive, client runtime, container or gameplay state
was accessed or changed in this slice.

**The two 10176 keys resolve to animation-token records. They do not resolve to
healing or recharge definitions in this path.** Both exact records are present
in frozen pack 184788. Their event and segment quantities must not be substituted
for an authoritative cast duration, healing interval or recharge.

## Context construction proves the lookup domain

The complete getter `0x1411757e0..0x1411757f1` calls `0x1409b1fb0` and returns
game-context +0x78. Registry initializer `0x140208910..0x140208939` registers
constructor `0x141175800` at index 15 through `0x1409b1de0`, and destructor
`0x141175880` through `0x1409b1df0`.

Constructor `0x141175800..0x141175879` allocates 0xb0 bytes, installs vtable
`0x142155cc0` and initializes its embedded manager at +8 through
`0x141179440`. That vtable's +0x30 is `0x141175ab0`: it adds 8 to `this` and
tail-calls `0x141179670..0x1411797b2`.

This lookup searches the root at manager +0x90. The root has Dword count +0 and
pointer +4. It binary-searches 20-byte rows by the first Qword token; each row
has Dword variant count +8 and pointer +0xc. It then searches 56-byte variant
rows by their first Qword. It returns the exact matching variant or null.

The caller `0x14129a4d0..0x14129a662` contains the diagnostic at `0x14217cec0`
beginning `GetSkillAbilityAnimationData could not find animation with token`.
Its other labels include `CharSkill.cpp` and `m_animationData`. Together with
the constructed manager and pack loader below, this proves animation-token and
variant-token lookup. The numeric keys are not file IDs.

The retained skill 10176 record in cached container 1282860 points to body
offset 4074944. Its primary animation token is `0xb309585954f7`, alternate token
`0x215a908ef7`, and both variant tokens are zero. The caller selects the
alternate pair only when its selector is nonzero and the alternate token exists;
otherwise it uses the primary pair. The gameplay meaning of that selector is
not established here. In particular, it is not labeled a game mode.

## Pack selection and frozen provenance

The relevant loader prefix at `0x1411797c0..0x1411798ac` names
`AnimManager.cpp`, `m_packFile` and `!m_packSequences`. It opens a constant
encoded filename, requests chunk tag `seqn` (`0x6e716573`) with version zero,
and stores the returned root at manager +0x90. The rest of this loader also
handles other chunks; only the sequence lookup is interpreted here.

The complete predicate `0x1409b1db0..0x1409b1dd7` tests whether game-context
+0x310 is zero, falling back to its designated global context if the getter
returns null. A true result selects constant `0x142155fd0`, bytes
`d3d4020100000000`; false selects `0x142155fd8`, bytes `d2d4020100000000`.
The previously established [compact filename decoder](209026-composite-source-filename.md)
maps these constants to **184788** and **184787**, respectively. This static
selection is not a runtime observation of the predicate.

An exact two-ID lookup in the frozen build-209026 index found 184787 absent and
184788 at zero-based MFT index 389497, aliases `[3827541, 184788]`, stored size
1,524,360 and decoded size 8,916,756. No substitute was guessed for the absent ID.

- Frozen archive size: 89,165,623,112 bytes; archive stat unchanged.
- MFT SHA-256: `3187214f9c7c6b92a51ec41c5e70bc0315102f1e165533322a2e00f45e89d967`.
- Index SHA-256: `2267413db3948d35da30fe53a558a2f6aa37bc19ce784b326cce71de4887c6a5`.
- Extracted pack: `/home/blizz/re/gw2/notes/animation-pack-184788-209026.pf`.
- Pack SHA-256: `72e5fce4c2098946d1644469881fcc43795349ea7e16c8a357b3c5594c35723e`.

The pack is `PF`, kind `anic`, flags 5. Its `seqn` v0 payload begins at file
offset 28, has 1,764,956 data bytes and 1,853 primary rows. Only the two known
tokens and their exact variant-zero rows were selected. Other chunks were not
used to infer skill execution.

## Exact matched rows

Offsets in this section are relative to the `seqn` payload. Raw stored pointer
fields are signed relative offsets from the pointer field itself; bounds were
checked before reading each row, event array, segment or pointed Dword.

| Field | Primary token `0xb309585954f7` | Alternate token `0x215a908ef7` |
| --- | --- | --- |
| Primary row index / offset | 1187 / 23756 | 419 / 8396 |
| Available variant count | 35 | 19 |
| Matched variant token / index | 0 / 0 | 0 / 0 |
| Matched variant offset | 403600 | 164760 |
| Segment count / event count | 1 / 6 | 1 / 3 |
| Segment offset | 1228160 | 769904 |
| Segment byte +0 / flags byte +5 | 0 / 0 | 0 / 0 |
| Segment +9 pointed-data offset | 1681320 | 1554544 |
| First Dword of pointed segment data | 2067 | 1667 |

Each event occupies nine bytes: byte type, Dword +1, Dword +5. The exact
ordered `(type, raw Dword +1, raw Dword +5)` tuples are:

- Primary: `(4,0,0)`, `(5,867,0)`, `(1,1067,0)`, `(3,1067,0)`,
  `(2,1067,0)`, `(0,1350,0)`.
- Alternate: `(1,400,0)`, `(3,467,0)`, `(0,800,0)`.

No event name or time unit is assigned by this extraction. Neither numeric
proximity to a displayed tooltip nor a familiar animation length proves those
semantics.

## What the typed action initializer actually derives

The complete logical initializer `0x14129a1a0..0x14129a4b3` crosses a pdata
fragment boundary at `0x14129a2c6`; both fragments are retained. It stores the
selected animation row at action +8 and constructs per-segment event ranges.

Its reverse scan records an event index for the last encountered type 3..5 at
action +0x18, and stops on type 3 after recording that index at +0x20. Its
forward scan records the first type-0 event's Dword +1. It then iterates the
25-byte segments: **the type is byte +0**, not +4; type 0 uses pointer +9 and
type 1 uses pointer +0x11. The selected pointed data's first Dword contributes
to a cumulative quantity compared with event Dword +1. Segment flags +5 bits
0x10/0x20 delimit an additional range; both are clear in these exact records.

For these one-segment records, the resulting values follow directly from that
code and data:

| Action field | Primary | Alternate | Established interpretation |
| --- | --- | --- | --- |
| +0x10 | 1350 | 800 | Computed quantity overridden by the type-0 event's Dword +1. |
| +0x14 / +0x1c | 0 / 0 | 0 / 0 | Segment indices containing the recorded event indices. |
| +0x18 / +0x20 | 3 / 3 | 1 / 1 | Recorded event indices, not durations. |
| +0x28 / +0x2c / +0x30 | 0 / 0 / 0 | 0 / 0 / 0 | No flagged range in either segment. |

Accessors bound through action vtable `0x14217cce8` corroborate the field
distinctions: +0x18 returns action +0x14, +0x20 returns +0x18, +0x28 returns
+0x1c, and +0x30 returns +0x20. Event accessors `0x14129a100` and `0x14129a120`
return a bounded nine-byte event and its count; `0x141299f90` returns the first
Dword from the selected segment's pointed data. These are animation/action
fields. This slice does not demonstrate their use as the authoritative healing
or recharge schedule, or establish a valid server response to predicted `0x53`.

## Independent tooltip observation and remaining boundary

Root separately viewed run 144's `saved-first-world.png`, admitted by REA
`ev_538379a4b2347f3fe1b199467be147c6ba552c2728602320fdbf67f690853515`,
PNG SHA-256 `43b7821b3960bf3e00a01f2d160c4af3d68a6cb87298af52a07b1157fc0083dc`.
That level-1, health-388 fixture displayed Ether Feast with cast icon 1,
recharge icon 20, Healing 121, Heal per Clone 0, Conditions Removed 1 and
Conditions Removed per Clone 1, plus a game-mode qualification. This is
displayed fixture-state evidence, not a general formula or execution proof.
No additional runtime read was made for this note.

The concrete next boundary is the skill tooltip/fact evaluator for those
displayed values and its exact level, attributes and mode inputs. The animation
lookup cannot supply those missing inputs. Authoritative healing, recharge
state and the matching server-driven command transition remain unresolved;
no values from this note were wired into gameplay.

## Retained evidence and superseded probes

Complete 23-capture bundle:
`/home/blizz/re/gw2/notes/209026-mesmer-heal-animation-data-20261106.json`,
752,925 bytes, SHA-256
`97cfd5a01431486fa0ecfe386ea0056556a6dd9abc5a13b33ef9e20628f17e7c`.
The focused reader is
`/home/blizz/re/gw2/notes/read-10176-animation-rows-20261106.py`, SHA-256
`3bb6951b52b39b4f74601882b8cddc899645a9ced0dd42b9c6e6a6075e0a3e92`.
All raw terminal frames and normalized output are complete; some repeated
rendered terminal frames hit their display budget.

Key captures: binding `ev_ed5e26934a76c10420d60fd94597a25413b6ef28ab3f95aaf21218b1a635928f`,
lookup `ev_c2bd7d24cb64872bde5d9cb7be195a6d3a2475a079b4b56cd1fa998be7707c4d`,
pack extraction `ev_06bebca57abe86d88b9b0aebc1cf737671d96c481ee0238425c9351d5bea8d79`,
corrected exact rows `ev_4988dd3db45039e0c8267502855216ab6589471056b8988b28f2167ac594134e`,
initializer fragments `ev_dcc5fa56f959377d8b0198afef27e743417c997e400b70641effef2f997fdef5`
and `ev_99375734b2d43b6a9e74b32ee3c4f236fba20d3a448ac4a4b6e24045378b73ff`.

Capture 11 is a failed metadata-reader assumption that both conditional IDs
must exist; capture 12 explicitly reports the absent ID. Capture 20 mislabeled
the segment type at +4; capture 22 supersedes it with source-proven byte +0.
The raw segment bytes were unchanged. Both superseded captures remain retained.
