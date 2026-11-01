# Build 209026: creation item colors

Static REA analysis, 2026-11-01, on the frozen executable identified in
[character creation](209026-character-creation.md). No runtime continuation
or creation request was sent.

## Request 415's final array

The final `ArrayFixed Size=24` of Dwords comes from six consecutive 16-byte
copies in `0x14129cc20`: context +0x480, +0x490, +0x4a0, +0x4b0, +0x4c0,
and +0x4d0. These are **item-color selections for equipment slots 2–7,
four color slots per equipment slot**.

The source is a larger matrix beginning at context +0x460. Setter
`0x14129f680(context, value, equipSlot, itemColorSlot)` writes to:

```
context + 0x460 + 16 * equipSlot + 4 * itemColorSlot
```

Its assertions explicitly name `equipSlot < CHAR_EQUIP_SLOTS` and
`itemColorSlot < ConstItem::COLOR_SLOTS`; the upper bounds are 0x44 and 4.
These inspected comparisons are signed upper-bound assertions, not evidence
of a complete defensive range check. The sender selects matrix rows 2–7
in ascending order and retains color slots 0–3 within each row. The numbers
are decoded Dword values; their wire representation remains the existing
variable-length Dword encoding, not a fixed 96-byte wire block.

## Content and preview consumers

After writing the value, the setter calls `0x14129d5f0` to find the matching
equipment record among eight context entries. That helper maps a record's
armor category through `0x1411bf460` to the equipment-slot enum. The setter's
subsequent assertions name `itemArmor` and `skinArmor`, establishing the
armor-preview context without assigning localized names to the numeric slots.

For a nonzero color selection, the setter looks up content type 9 by that
value using the content service's slot +0x70. For zero, it starts with a
color pointer from the skin's corresponding slot (+8/+0x10/+0x18/+0x20),
with possible fallback to another matrix selection, and stores the resulting
content object's +0x28 Dword back into the selected matrix cell. The later
loop also chooses content type 9 for nonzero matrix entries and uses the
result's color-parameter records for the preview. This supports describing
the values as content color selections, not packed RGB. Actual color names,
the complete fallback policy and server-side validity remain unverified.

## Defaults

`0x14129c240` selects a default appearance entry from race/sex content returned
by `0x1411beed0`, filtering its 24-byte records by profession at +0x10. The
selected record is retained at context +0x8b0, or null if none match.

Importer `0x14129deb0` receives the matrix pointer in R8 (the initializer
passes context +0x460). After its 45 appearance-byte reads, it parses four
decimal integer values per row for 0x44 rows and writes all those matrix
cells. If +0x8b0 is null, it returns before those color writes. The complete
constructor's default matrix state has not been traced, so that branch does
not establish zero defaults. The request still copies only the six rows
identified above.

The earlier handwritten request-415 fixture, whose first final-array Dword
is 300 and others zero, remains a serialization boundary fixture. It does
not establish a valid color choice or a usable character.

The [content-registry trace](209026-content-registry.md) now locates the
ten race/sex definitions in archive file 1282852 and extracts their 90
profession-specific default strings privately. This provides shipped input
for parsing appearance and color selections; runtime use remains untested.

## Evidence

Private bundle: `~/re/gw2/notes/209026-creation-colors-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_e9f9a114ae887363e88bd14667eb9fe0cd5143f2c1cf7bd2c3bde5f7956d65b0` | Six 16-byte copies and forwarding through the request bridge. |
| `ev_c6772b65ff604f0981b7f58be89b4026b70f5e7701b4da6c48df848cbdeb9618` | Color setter, matrix arithmetic, content type-9 lookup and preview reads. |
| `ev_7fa82f6b2b743ce2c2da3b0896844d30c92a2ce81e668775687e93e62a97ba25` | Default selection, equipment-record lookup and explicit equipment/color/armor assertions. |
| `ev_ab108b8ce1c76082097da8ba2a1ba0be86283a463fb379787b0724204210d94a` | Armor-category to equipment-slot mapping function. |
| `ev_add088fcbd7cee290233336a6936a571f2893e38750f3d67f6d3d68395aefb22` | Appearance importer followed by 68 rows of four color values. |
