# Build 209026: Ether Feast activation-time tooltip

Static REA, 2026-11-06, against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`,
the retained skill container and the exact two animation records already
identified in [the animation note](209026-mesmer-heal-animation-data.md).
No runtime, container or working archive was accessed or changed.

**The primary 10176 animation supplies an execute-event quantity of 1067,
converted to approximately 1.067 seconds and formatted in quarter-second
increments. Under nearest rounding, the displayed value is 1.** This is a
tooltip derivation, not proof of an authoritative 1000 ms cast or a server
schedule. The alternate record supplies 467, displayed as ½ under the same
rounding model. Neither result comes from the animation's type-0 values
1350/800 or the [local predicted command's 1000](209026-predicted-skill-order-lifecycle.md).

## Concrete tooltip binding and selection

The cost-widget update beginning at `0x140899750` obtains child ID 1.
For this branch it requires a nonnull skill, skill kind +0x58 equal to zero,
and skill flags +0x38 bit 0x80000 clear. It initializes a temporary timing
object through `0x14089b9c0`, calls `0x14088f4b0` at `0x140899887`,
then calls `0x14089b7c0` with selector 0x17. It passes that returned
float directly to the child's virtual +0x20 at `0x1408998a7`; visibility
depends on the value being positive. There is no intervening recharge
modifier call in this activation-number branch.

For 10176, the exact cached type-64 record is in file 1282860 at offset
4074696. Kind is zero, flags are `0x14000006`, and +0x60 points to its
body at 4074944. The relevant body fields are:

| Body field | Value | Use in this path |
| --- | --- | --- |
| +0 | `0xb309585954f7` | Primary animation token |
| +8 | `0x215a908ef7` | Alternate animation token |
| +0x10 | 0 | Alternate variant token |
| +0x18 | 0 | Primary variant token |
| +0x80 / +0x84 | 0 / 0 | Primary segment-loop limits, unused by these unflagged rows |
| +0x88 / +0x8c | 0 / 0 | Alternate segment-loop limits, unused by these unflagged rows |

The complete selector `0x14088f4b0..0x14088f530` checks skill flag bits
28 and 26. Both are set for this record, so its selection is the boolean
returned by the tooltip adapter's virtual +0x88. The
[character adapter](209026-tooltip-character-context.md) binds that slot
to `0x14088f610`: null character returns zero; otherwise it calls
character primary virtual +0x4b0.

Primary vtable `0x142160be0` binds +0x4b0 to `0x1411d9d50`, which
reads character Dword +0xd8 and tail-calls `0x14122aea0` ->
`0x1411c5640`. The complete leaf tests
`((value - 1) & 0xfffffffd) == 0`: true for 1 or 3. This is the same
**charLocale** field independently established by the
[skill-palette](209026-player-skill-palettes.md) and
[state-conversion](209026-character-locale.md) traces.

Thus, with this character adapter and exact skill:

- Locales 1 or 3 select the alternate token/variant.
- Other locales select the primary token/variant.
- If the selected animation lookup returns null, the timing initializer
  attempts the other pair.

The constructor's established locale 0 supports the primary path as a
construction model. Later state changes and the actual run-144 tooltip
adapter/locale were not sampled here. Numeric locale names remain unresolved;
this selection is distinct from the character's +0x228 recharge-context
selector.

## Animation event to tooltip seconds

Initializer `0x14089c100` forwards the selected skill and boolean to the
complete logical timing function `0x14089bb00..0x14089c0c4`.
Its source diagnostic names `SkpSkillTiming.cpp`. It uses animation-context
virtual +0x30, whose actual token/variant lookup was established in the
[animation note](209026-mesmer-heal-animation-data.md).

The matched 56-byte rows in frozen pack 184788 have no charge stages, one
25-byte segment with flags byte +5 equal to zero, and one event of type 3.
The 14-entry event switch table at `0x14089c0c8` routes only type 3 to
`0x14089bf8e`; the other recognized types are ignored by this extrema
calculation. The function converts event Dword +1 to float and divides it
by the literal f32 **1000.0** at `0x141924c6c`. It collects the minimum
and maximum type-3 values. The related predicate names `HasExecute1()`.

For these exact rows, no segment has range flags 0x10/0x20, so the additional
flagged-range adjustment is absent. The one execute event precedes the
segment total; the remaining adjustment comparisons leave both extrema
unchanged:

| Selected row | Variant offset in seqn payload | Segment total raw | Execute raw | Timing +0x24 and +0x28 |
| --- | ---: | ---: | ---: | --- |
| Primary | 403600 | 2067 | 1067 | f32 1.0670000314712524, bits `0x3f889375` |
| Alternate | 164760 | 1667 | 467 | f32 0.46700000762939453, bits `0x3eef1aa0` |

The data's raw time quantities are therefore thousandths of the tooltip's
seconds unit. The raw 1350/800 type-0 events affect the separate action
initializer described earlier; this tooltip extrema switch ignores them.

`0x14089ad40..0x14089ae4f` classifies the timing for display. Selector
0x17 is not its special 0x13 case, 10176 does not match its excluded GUID,
there is no charge-stage branch, the execute extrema are equal, and the
minimum is at least 0.25. It returns display kind 3.
`0x14089b7c0..0x14089b951` then returns timing +0x24 through
`0x14089ba30`. No healing attribute, recharge-context override or quickness
multiplier is applied by this demonstrated path. That does not exclude
modifiers in actual skill execution or other client consumers.

## Quarter-second formatting explains the displayed 1

The child builder installs vtable `0x141b691c8`; its virtual +0x20 is
`0x140899640..0x140899680`. That setter clears the float sign bit and
calls the complete logical formatter `0x1403124e0..0x1403125eb`.
For a positive value, the formatter:

1. Multiplies by f32 4.0.
2. Calls `0x1409bedf0..0x1409bee70`, the
   [previously identified client rounding helper](209026-mesmer-health-speed-baseline.md).
3. Clamps the resulting unsigned quarter count to at least one.
4. Formats the quotient by four as an integer and appends the remainder's
   UTF-16 glyph: none, U+00BC (¼), U+00BD (½), or U+00BE (¾).

The rounding helper bounds its input to [-0.5, 4194303], adds f32 12582912
(bits `0x4b400000`), then returns the result's bits masked by
`0x003fffff`. It depends on the executing floating-point rounding
environment. The private oracle explicitly models nearest-even single
precision; it does not sample or force target MXCSR.

| Row | Seconds × 4, f32 | Quarter count under nearest rounding | Formatted value |
| --- | ---: | ---: | --- |
| Primary | 4.26800012588501 | 4 | 1 |
| Alternate | 1.8680000305175781 | 2 | ½ |

This provides a source-and-data explanation consistent with root's run-144
observation of the cast icon followed by **1**, retained in the
[animation note](209026-mesmer-heal-animation-data.md). The observation
alone does not identify every intermediate runtime input. In particular,
rounding the tooltip cannot justify substituting 1000 for the underlying
1067 event quantity or selecting either as the server's cast duration.

## Exact map 379 fallback, separate from cast locale

The earlier [tooltip context note](209026-tooltip-character-context.md)
left the selected map's fallback-reference chain unresolved. A bounded
read of the already indexed map 379 record now establishes:

- Type 45, key 379, file 1282833, offset 3863896, index 18427.
- Map type Dword +0x2c is 7.
- The reference at +0x100 is null; there is no second +0x30 target to follow.

For this record, the complete global fallback
`0x14140f1d0..0x14140f22d` returns **1**. The special map-type branch
does not match 7, and the null reference takes the absent-data return.

This remains conditional at the character boundary: primary virtual +0x228
calls +0x220 first and invokes this map fallback only when that returns
sentinel 4. Those character-specific queries remain unresolved. If the
fallback is used for map 379, context 1 selects 10176's base recharge field
20000 instead of its context-2/3 overrides 25000, before any separate
recharge-display modifier. This does not assert the actual character
override state, final displayed recharge or server recharge policy.

## Evidence and scope

Private bundle:
`/home/blizz/re/gw2/notes/209026-mesmer-cast-tooltip-20261106.json`,
530,885 bytes, SHA-256
`e3df839241e24347d0a24c2fed34bf0d99cf10cf08d630c1f882511f0fc946c9`.

The bundle preserves all 18 new captures, one retained context capture,
and the two bounded reader sources. Every capture exited zero. Capture 6's
output limit omitted part of a separately requested accessor; the entire
timing initializer through its return was retained, and capture 8 recaptured
the accessor completely. Other truncation flags concern duplicate rendered
terminal views. Jump-table bytes and instructions beyond the stated logical
function returns were not interpreted as function code.

Exact-data reader:
`/home/blizz/re/gw2/notes/read-10176-cast-tooltip-20261106.py`,
SHA-256 `01434c88f4dbf05a932d7b42af08d4ef2251c78ffd4ebfec18cc338d6077a04e`.
It pins the cached skill and animation pack hashes, bounds every selected
row/reference, verifies flags and excluded-GUID status, and evaluates only
the demonstrated unflagged rows. It does not implement a general animation
timing engine or establish server behavior.

| REA evidence | Scope |
| --- | --- |
| `ev_73d76d94a7fd94722ba3b6f202e3a6cfbc4acecf968ff54e2933ed0193d39875` | Cost-widget binding, selector and display-value function |
| `ev_43969ad69a8cbcba200f7eb0d5d1b0698748f36138d912b78e3bf4c925cfea44` | Complete timing initializer |
| `ev_27c6e85abc088f2887be39caf634d96cb2575a9fbc4e8b3e7b9eec8f40b189d7` | Exact event switch table |
| `ev_6a5d7509130f92c859c142cf394cdb62f9d4b834436e7644719fee7bef12c162` | Numeric-widget and character virtual bindings |
| `ev_32bedd6196eeae8d0202b8f2b9fdf07c0165c8d0874817367937b2a18b0aed0d` | Complete quarter formatter |
| `ev_89e770efbee31144a5258d202e611dceb12f6745eb36b10a7d95a9c9f9b2c661` | Rounding helper and locale predicate |
| `ev_8c65751718c564da4bf78f0b0cdb39a2c3b19c2dd1c06bb97d894f933d8f17c1` | Exact skill/animation rows and explicitly scoped arithmetic oracle |
| `ev_7dfbeda5e3aeac440c382c034369f3d157c233e595b5e1bce843ba604052c5b1` | Exact map 379 fallback fields |

Cached skill-container SHA-256:
`05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23`.
Cached map-container SHA-256:
`5ab847328dd307407d9701711890f08282f9c6f3639d24bb9684ea46006e26ef`.
Animation pack SHA-256:
`72e5fce4c2098946d1644469881fcc43795349ea7e16c8a357b3c5594c35723e`.
