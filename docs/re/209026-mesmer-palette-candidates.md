# Build 209026: Mesmer starting-skill candidates

2026-11-05, read-only frozen-content checks. These records narrow the startup
skill work; they do not establish default selection or working combat.

`gw2-skill-palette-check` now accepts an explicit profession and palette keys:

```text
gw2-skill-palette-check <build-209026 Gw2.dat> 7 271 365 366
```

Its original one-argument Guardian audit remains available. Invalid profession,
key widths or missing key arguments fail before archive reading; a requested
palette with no entry for that profession fails after the census.

The optimized Rust checker visited all 32 base containers and matched the
previous totals of 7,769 palettes, 20,990 rows and 27,320 resolved type-64
references. There are **164 profession-7 rows**. Explicit candidate records:

| Palette key | Skill key | Entry minimum level | Raw mode | Skill flags |
| ---: | ---: | ---: | ---: | ---: |
| 271 | 10177 | 1 | 3 | 335544326 |
| 365 | 10214 | 1 | 3 | 369623046 |
| 365 | 10213 | 1 | 3 | 1442840582 |
| 366 | 10176 | 1 | 3 | 335544326 |

A separate hash-checked Python traversal confirms category2 for these rows,
the same level/mode/flags, and null optional entry references +0x30/+0x40/+0x50.
Palette365's first entry additionally references type64 key10215 at +8; do
not flatten it into a single-skill palette or discard this distinction.
Palettes271/366 have null +8. All inspected entries have +0x24/+0x28=23,
+0x48/+0x4c=24, +0x3c/+0x64=0 and +0x60=8.

These fields can be compared with the already traced
[entry predicate](209026-guardian-palette-eligibility.md), but no default
Mesmer healing palette has been selected. Palette membership, skill
availability, character locale, level and runtime component lifetime remain
separate conditions. Correct rendering also does not supply server-side
skill execution. The standard-slot setting levels remain 0/11/15/19/31.

The Python content reader verifies source hashes before reading:

- 1282837: `efabc1cd94ec8cbb72da0f2c35c5e569b29d9414c93974de9adbc710f48ae6fd`
- 1282860: `05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23`

The broader attempted selection also visited 1282852 while looking for
palette521; it yielded no profession7 row. The final Rust request only
selects the three verified Mesmer candidates above.

## Verification and limits

Release checker build and five invalid-argument cases pass. REA captures:

| Evidence | Result |
| --- | --- |
| `ev_d1c090083c06c93e6e087dde66b58421723037644dde4a9b78ca45282bb21b51` | Hash-checked Python entries and referenced skill records; exit0. |
| `ev_a736d8678475c914f325df05bc8e52a38583674c7e2a927ad2a6582ecfefc121` | Optimized Rust all-container census and Mesmer candidates; exit0. |
| `ev_9a16b0d6fe3b9461f76f630bd428f10090a042dbef522b6b9832cefce2108587` | Earlier debug checker exceeded30s and was terminated; no successful data result. |

Private bundle: `~/re/gw2/notes/209026-mesmer-palettes-20261105.json`.
All captures verified cleanup of their owned process groups; unrelated
unreadable processes were left untouched. These execute readers, not the
game. No runtime skills, palette assignment or deployment was performed.
