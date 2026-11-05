# Historical MFST75 remaps: frozen-209026 compatibility audit

Source-only preparation, 2026-11-05. **All 26,322 property associations in
the single pinned historical MFST are type 0 with four-byte payloads, and
all pass the requested frozen-index alias and decoded-size checks.** This
supports retaining the complete mapping set for a future scoped experiment.
It does not authenticate these mappings as build-209026 metadata or establish
that expanding the mapping set fixes the observed mask-size failure.

## Exact source and eligibility

The cached root is build **208784**, MFT **16**, SHA-256
`d30dd8e2ee4f0a3de3c8e9cf31fee6e64c659c9b194e20fa0f0a32bdf4364413`.
Its explicit link points to base **970570**, file **3831142**, size
**2,919,828**. Cached MFT **75** resolves those IDs and has SHA-256
`3a2592698686b6811df05cd703a3f3d3b12bafcc9ed392a45c3d57d1ca912a17`.
The reader verifies both decoded hashes and exact index associations before
interpreting MFST v6's **114,978** records and **26,322** properties/table rows.
It reads no other cached artifact body.

Each eligible mapping requires a unique type-0/four-byte source property,
an exact source cached base record, and one target cached record identified
by its base/file alias. For both records, the cached base and file IDs must
resolve unambiguously to the same frozen MFT entry, and that entry's decoded
size must match the cached record. No adjacent-ID arithmetic is used.

| Result | Count |
| --- | ---: |
| Compatible mappings | **26,322** |
| Missing cached or frozen records/aliases | **0** |
| Alias or decoded-size mismatches | **0** |
| Other property kinds or lengths | **0** |
| Duplicate-source exclusions | **0** |
| Header errors or unchecked entries | **0** |
| Unique original/target IDs checked | **52,644** |

The original 20 experiment mappings are present unchanged in the compatible
set. There are no exclusion examples; the canonical report preserves an
empty exclusion array rather than suppressing failures.

## Bounded reads and limits

One archive-reading attempt completed in **16.111 seconds**, within its
160-second internal / 180-second capture bound. It builds index maps once,
sorts only referenced MFT entries by physical offset, and calls
`Archive::decoded_size` once per selected entry. Each such call reads at
most eight stored bytes; uncompressed sizes use MFT metadata. No frozen
asset payloads were decompressed and no 819,347-record manifest regeneration
was performed. The cached and frozen archive sizes/mtimes were unchanged;
the working archive was only statted to reject an inode alias.

These checks establish record compatibility, not equality of historical and
current asset bytes. No full frozen-archive hash was computed. The active
ArtCatalog, native metadata table, optional variant mask, loaded dimensions,
and pixels were not observed. See the
[native remap-selection contract](209026-texture-remapped-file-selection.md).
No full overlay, runtime change or deployment was produced by this task.

## Private retained results

- Canonical compatible mappings and eligibility fields:
  `~/re/gw2/notes/208784-mfst75-compatible-remaps-vs209026-20261105.json`,
  **11,068,672 bytes**, SHA-256
  `e3ab9f25b8ae1d1fd18d23de4be37dc0839995a80a6df591188ce2e030b7daa0`.
- Flushed bounded-reader journal:
  `~/re/gw2/notes/cached-mfst75-remap-audit-20261105.tsv`, **4,839,343 bytes**,
  SHA-256 `ef6a7a95399890a00c18bc58ee8d2f57a1fb42230799514fe6de5a5fe9f73043`.
- Reader source / executable:
  `~/re/gw2/notes/audit-cached-mfst75-remaps-20261105.rs` /
  `~/re/gw2/notes/audit-cached-mfst75-remaps-20261105`.
- Classification reads only that hash-pinned journal. Its focused synthetic
  assertions cover matching, missing and divergent aliases, changed size,
  unchecked/error states and missing cached records.

Complete evidence bundle:
`~/re/gw2/notes/208784-mfst75-remap-compatibility-evidence-20261105.json`.
Reader **`ev_1994ceb8079a985c43ee83bbdd490b6231cc84b1bf5511022802fbeb26ea5246`**
and classification **`ev_d7057d1a4d2c8bdd98c4c70f64a86248f0d889988a97b90305cb8a3cfda537c2`**
both exited zero with nontruncated output and verified owned-process cleanup.
