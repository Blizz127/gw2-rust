# Build 209026: complete default primary-attribute curve

2026-11-06. The complete 101-row default curve is recovered from the
already decoded, hash-pinned client container 1282841. The pure
[`primary_attribute_curve` module](../../crates/gw2-proto/src/primary_attribute_curve.rs)
retains every source row and exposes
`default_player_primary_attribute209026(level: u8) -> Option<u16>` for
player levels 1 through 80. Level zero and every value above 80 return
`None`; there is no interpolation, clamping, or fallback.

## Source selection and exact layout

The [baseline source trace](209026-mesmer-health-speed-baseline.md)
establishes that `0x1411c3980` resolves configuration 396, type252, selects
layout +0x40 for the default configuration, requires type391 table kind1,
and reads a level-indexed Dword. The common helper `0x1411bf730` uses that
curve for attribute selectors below 5, including the primary selectors
1 through 4. Constructor state selects the default curve; game-server17
can change the selection using account flag47. The alternate curve at
layout +0x38 is a different input and is not exported here.

The bounded extraction follows these classified local references in
container 1282841:

| Record or field | Exact value |
| --- | --- |
| Type252 configuration layout | offset 175152 |
| Configuration GUID bytes | `7daf96c6bf4a714db4d0ea51c98f4d4e` |
| Layout +0x40 | type391 at offset 173616 |
| Type391 +0x28 | kind 1 |
| Type391 +0x30 | array header at offset 173672 |
| Array header +8 | count 101 |
| Array header +0 | values at offset 173688 |
| Stored element width | unsigned little-endian Dword |

The reader checks the container hash, PF/cntc layout, relocation classes,
record types, table kind, count, and every byte range before reading.
Each of the 101 source Dwords is separately checked to fit `u16`; their
range is 30 through 1000. The final source hash is checked again.

## All source rows

Each row below gives consecutive values for the inclusive level range.
Raw levels 0 and 81 through 100 remain in the exported source array;
the player lookup deliberately rejects them.

| Levels | Values in increasing level order |
| --- | --- |
| 0–9 | 30, 37, 44, 51, 58, 65, 72, 79, 86, 93 |
| 10–19 | 100, 100, 110, 110, 120, 120, 130, 130, 140, 140 |
| 20–29 | 150, 150, 164, 164, 178, 178, 193, 193, 209, 209 |
| 30–39 | 225, 225, 245, 245, 265, 265, 285, 285, 305, 305 |
| 40–49 | 325, 325, 349, 349, 373, 373, 398, 398, 424, 424 |
| 50–59 | 450, 450, 480, 480, 510, 510, 540, 540, 570, 570 |
| 60–69 | 600, 600, 634, 634, 668, 668, 703, 703, 739, 739 |
| 70–79 | 775, 775, 819, 819, 863, 863, 908, 908, 954, 954 |
| 80–89 | 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000 |
| 90–99 | 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000 |
| 100 | 1000 |

The SHA-256 of all 101 values serialized in their original little-endian
Dword representation, exactly 404 bytes, is
`554c6ace367f10b29f8ff14688e2ca5fe63ce8ee0acbe33b83f7865a8bb6d079`.
The source extraction and an independent comparison with the Rust array
check all 101 values, rather than only the displayed checkpoints used by
the unit tests.

## Meaning and limits

This is an explicitly named default configuration curve. Its availability
does not establish which effective level, account configuration, equipment
modifiers, or other adjustments an arbitrary retail character uses. The
pure module chooses no earned/effective-level update policy and sends no
protocol messages. In particular, an earned-level update is not evidence
that the second/effective level should change. The
[starter attribute update contract](209026-starter-base-attributes.md)
keeps the server-supplied columns distinct.

This extraction used only the retained decoded container. It performed
no archive reads, client launch, runtime sampling, or live configuration
change. CLI and mission integration are separate from this source table.

## Evidence, artifacts, and checks

Container:
`/home/blizz/re/gw2/notes/content-type-147-file-1282841.pf`, SHA-256
`62ef0c04651578ca4cf6a4322fe4cb792bd865b614069d639858986b13594e39`.
The existing native source trace uses executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| REA evidence | Observation |
| --- | --- |
| `ev_5f2ecbb3220c78fb69ecb26b85a7f28a540838b5ff8a9fe848e9ffff3c28dae3` | Existing configuration396/type252 resolution, kind check, and level-indexed read. |
| `ev_559a90aeaed76f8beb13b1c20e94d4f96af49bac792ff96cfe9c988bc0432468` | Existing descriptor GUID and common primary-attribute helper. |
| `ev_52f47362cd135e0bdf1d5a82544cab0afd2d862833583a839a75898e07da0524` | Complete new extraction: pointer chain, bounds, source hash, and all 101 rows. |
| `ev_7768f2264d0e7489fa75b9a18a0f174d692f64f7aa1e60e7ec128ad9bd95e469` | All 101 Rust rows equal the extracted source; full Dword-table digest and frozen module pin checked. |

Both new process captures exited 0, were untruncated, and verified cleanup
of their owned process groups. Complete new evidence is retained privately
at `/home/blizz/re/gw2/notes/209026-default-primary-attribute-curve-20261106.json`,
SHA-256 `c9ec70103cf67393a13bff4ae5a9ab0784e7c7015f724e303c5d36108d292ca5`.
The exact comparison receipt, including every source row, is
`/home/blizz/re/gw2/notes/209026-default-primary-attribute-curve-exact-20261106.json`,
SHA-256 `e5b1de4ce42cfc9d9a42ac87c8981c58c1f02e56bd39952dd871a1cc034bf59c`.

The cache-only extraction helper is
`/home/blizz/re/gw2/notes/read-default-primary-attribute-curve-20261106.py`,
SHA-256 `ed38189a9380dc2e28a843251c995a53cf7717467d138d1ec8d405945a4d4cd5`.
It reuses the hash-checked parser prefix from
`read-sword-fact-kinds-20261106.py`
(`fb9d9c582267281fed91fea2c99977fbb262487a8f5ca53cc9ce46be7ab38f79`)
with an allowlist containing only this decoded container and no archive
fallback. Raw extracted data and tool artifacts remain private.

The frozen Rust module SHA-256 is
`948a8b13411f00e79b32e29f8fdc665b028ad65583b9479a864c826162ea76b6`.
Focused command:

```text
cargo test -p gw2-proto --lib primary_attribute_curve -- --test-threads=1
```

Result: 2 passed, 0 failed, 181 filtered. Tests cover source steps and
plateaus, the retained nonplayer tail, exact lookup over all supported
player levels, and rejection of every unsupported byte value. Formatting
and scoped `git diff --check` passed. No broader suite was required for
the pure table lookup.
