# Build 209026: player skill palettes

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This follows the [skill-bar palette
routes](209026-character-skillbar.md). No client replay occurred.

## Server 612 selects a palette

Registration pair `0x142170950` connects descriptor `0x1425d7ab0` to
handler `0x14125b850`. Protocol 0/group 0 server **612** has exactly:

| Wire field | Type | Observed use |
| --- | --- | --- |
| Palette content key | Dword | Zero supplies null; otherwise content-context virtual +0x238 resolves type 65. |
| Slot | Byte | Zero extended and passed to the palette setter. |
| Context selector | Byte | Zero extended; selects one of two palette arrays. |
| Player index | Dword | Character-context virtual +0x118 looks up a player; handler requires nonnull. |

Dwords use the existing unsigned variable-length encoding. This message
does **not** start with a character index. Handler decoded offsets are
+2/+6/+7/+8; they are not wire offsets. Content virtual +0x238 is
`0x1412e1eb0`, which forwards type 0x41 and the key to generic lookup.
Player lookup is `0x1411b77f0`, indexed through context +0x80 and bounded
by count +0x8c, as in [controlled-character assignment](209026-character-control.md).

## Stored arrays and rebuild dependency

The player owns an embedded object at +0x9be0, constructed by
`0x141226ad0` with vtable `0x14216b548`. Source diagnostic
`0x14216b630` identifies `Game/Char/Cli/ChCliSkill.cpp`. Its palette getter
at virtual +0x58 is `0x141227c90`; the setter is `0x1412291e0`.
For standard slots 0–4, both use these arrays relative to that object:

```text
selector in {1, 3}: palettes[slot] at +0x88 + slot * 8
other selectors:   palettes[slot] at +0x60 + slot * 8
```

Predicate `0x14122aea0` jumps to `0x1411c5640`, whose exact check is
`((selector - 1) & 0xfffffffd) == 0`. Each array's 0x28-byte region is
zeroed by the constructor. Assertion `0x14216b6e0` names
`CHAR_SKILLBAR_SLOT_STANDARD_FIRST/LAST`; both getter and setter also
allow numeric slot 21 past that assertion. This trace does not establish
safe storage semantics for that exceptional slot or valid arbitrary bytes.

The slot lookup during normal rebuild obtains the selector from
character +0xd8 through primary virtual +0x1b0 (`0x1411a1dd0`). Character
construction derives it from an agent state through `0x14122acc0`, whose
switch diagnostic explicitly names **agentState**. The meanings of the
resulting selector values remain unresolved; do not label these arrays
land/water solely from the two-array shape.

After storing a palette, the setter notifies listeners, then reads the
**owned character** at player +0x20 and its skill bar at character +0x520.
It tail-calls `0x1411f9500`, which immediately invokes skill-bar virtual
+0x1f8. Neither dereference has a null guard. Thus a 612 update requires
an existing player, owned-character association and skill bar, including
when clearing a palette with key zero. The resulting normal rebuild also
has the [profession and registration dependencies](209026-character-skillbar.md).
Setting palette pointers is not an independent pre-allocation step.

## Palette rows

Row selector `0x141212bc0` scans palette +0x30/+0x38 as an array of
0x18-byte records and returns the first row meeting both constraints:

- Row +0 is zero or equals the context's profession scalar. Adapter
  virtual +0xc0 forwards through `0x1411f7790` to the coreStats +0x12c
  getter `0x141027030`.
- Row +4 equals the mapped slot category, unless that category is 29.
  Palette +0x40 value 10 forces category 29; value 11 uses slot mapper
  `0x1411c8340`; other values use `0x1411c81e0`.

The chosen row is then scanned for a passing entry by `0x141212fe0`.
That predicate checks multiple content references, scalar comparisons,
slot eligibility and conditional context flags. Its full gameplay field
mapping remains unfinished. A matching profession/row alone does not
establish a usable skill or a valid initial palette key.

## Typed frame and focused checks

`gw2_proto::skill_palette::SkillPaletteFrame209026` encodes and
transactionally decodes server 612 with the exact field order above.
It preserves raw bytes and all u32 values without choosing a palette,
validating gameplay slots or emitting messages automatically.

Three focused tests and `cargo build -p gw2-proto --lib` pass. Independent
hex fixtures cover distinct key/index varint widths, null selection,
u32 maxima and raw byte retention. Tests also check append prefixes,
concatenated frame boundaries, every truncated fixture and invalid IDs,
overflowing or overlong varints in either Dword position. No client
messages were sent; these results establish framing only.

## Evidence and limits

Private bundle: `~/re/gw2/notes/209026-player-skill-palettes-1.json`
(1,042 records, 23,214,755 bytes). The User Agreement hold remains;
these are static dependencies, not a successful player bootstrap.

| REA evidence | Observation |
| --- | --- |
| `ev_61a7015693334d8695b7b71bca7761158053a926dcc9cf13ac38708a8167999d` | Palette getter vtable target and complete row selector. |
| `ev_d22a391e63a1f3fb301ece687fd8793b8ea532907f3c0d0d2164f2a0306aa8da` | Two-array getter, slot mapping helpers and scalar source target. |
| `ev_946b5d78c8f06c43233f6bda8ed71832345321e6ef105be0c362775975d991f7` | Character +0xd8 getter, source/assertion literals and entry predicate prefix. |
| `ev_7312d87285ff84d05d8d2152ccd8e857d0f6f633f98f50c14a62f74864b53ff4` | Selector predicate, constructor array initialization and setter search. |
| `ev_711045674805e930b666f3b000630ba722f5530eda7fae3e7b0b60f595759945` | Complete setter and profession getter. |
| `ev_0eb6665f1fdea2bf168056dc4a8421960a0a4e56e50332fb362c42c946206b04` | Direct setter caller and character selector initialization. |
| `ev_e12fa52e051af68036dc1903fe96bd6b7d093f5266db4822552858d0e71fc0cd` | Handler function boundary, state conversion and content lookup target. |
| `ev_749a8dda9fa19ed4a6dc72bb3da1b23e90593103749fd8a4973d6e4abacb7f12` | Registration pair, full handler, type-65 lookup and entry predicate tail. |
| `ev_132a21536aa4e403f504e43a1282bf2bfce25181bb3d1dbae7da73589e3ca6c8` | Hash-guarded message descriptor, agentState literal and immediate rebuild wrapper. |
| `ev_9ce5c851c4573d5616a4388b4c9e901eb9eeb6ef5689cc19765b17bb602b7d97` | Indexed player lookup and constructor zeroing wrapper. |
