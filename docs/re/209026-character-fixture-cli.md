# Build 209026: source-backed appearance and diagnostic character fixtures

The two CLI steps keep content extraction in `gw2-dat` and protocol framing
in `gw2-proto`; neither crate gains a dependency on the other. They produce
private files for an isolated client experiment, not a persistent character
creation transaction or a retail loadout.

## Extract the selected archived appearance

```sh
target/release/gw2-appearance-extract /path/to/Gw2.dat 3 0 7 /new/private/appearance-directory
```

Arguments are archive path, race, sex, profession, and a new output directory.
Race/sex selectors use the [client enum and registry keys](209026-appearance-validation.md);
profession is 1–9. This example selects Norn male Mesmer. The extractor
admits the recorded appearance offsets only when file 1282852 matches
its frozen SHA-256. It resolves the type-137 default array and selected
profession's string through classified content references, then uses
`AppearanceDefault209026`, including its traced Sylvari importer adjustments.
All reads and reference traversals have explicit bounds.

Outputs are `appearance.bin` (45 bytes), `colors-le-u32.bin` (only archived
color rows), and `provenance.txt`. The latter records selectors, record and
string locations, source-container hashes, and the appearance digest.
It also resolves the exact level-1/profession equipment-template reference
and records item identities/slots with source locations. That reference
check does not select the client's runtime mode branch or later overrides;
no equipment is injected into a character summary or inventory.

## Frame an explicit diagnostic character

```sh
target/release/gw2-character-fixture /new/private/appearance-directory/appearance.bin 7 379 'Banshee Mesmer' 00112233445566778899aabbccddeeff /new/private/fixture-directory
```

Arguments are the appearance file, profession, map key, name, explicit
synthetic GUID as 32 hex digits in wire-byte order, and a new directory.
The builder emits `summary.bin` (version 11, level 1), `record-body.bin`,
and `manifest.txt`. The body starts at Guid and omits the server-17 message
and request IDs, which the observer supplies for request correlation.
Names are limited to 1–19 non-control UTF-16 units. The complete appearance
extent and scalar ranges are checked using the existing summary parser.

Opaque record/summary fields and all six preview equipment records are
explicitly zero. Map content validity remains the caller's responsibility.
The tool neither infers summary equipment ordering nor writes game state.
Both tools require a new output directory and create directories/files
with modes 0700/0600 on Unix; existing destinations are not overwritten.

## Verification (2026-11-05)

Two focused fixture tests cover exact identity/name/summary boundaries,
UTF-16 length limits, appearance extents, scalar ranges and GUID parsing.
Both optimized CLI builds pass. Six extractor argument-rejection cases
fail before archive access, and retrying an existing fixture destination
leaves its record bytes unchanged.

The real archive extraction selected Norn male Mesmer key **57**, record
**1282852:113352**, string slot **119600**, string index **346**. It produced
59 color rows and appearance race/scale/sex **3/9/0**. All 45 appearance
bytes match the retained default extraction, with SHA-256
`b3b957c919a12d3b030b3ed4cc75c0117b2fb564c5859b18e2d97b98af7f9c4f`.
The linked level-1 Mesmer equipment template is UID 210 at 1282832:250800.

The diagnostic character uses map 379, name `Banshee Mesmer`, and the
explicit synthetic GUID shown above. Its summary is 159 bytes and record
body 224 bytes. A separate census decoded the body wrapped with message
17 and synthetic request ID 1 as exactly one complete 227-byte frame.
These checks do not establish client display, map entry, combat or persistence.

Private outputs:
`~/re/gw2/notes/norn-mesmer-appearance-20261105/` and
`~/re/gw2/notes/norn-mesmer-fixture-20261105/`.
Private evidence bundle:
`~/re/gw2/notes/209026-norn-mesmer-fixture-cli-20261105.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_fadf5e2d245a46579469bea41ac69803d7f91484b26f8ce2d6ef263cc2f83b64` | Archive appearance/template extraction exits successfully. |
| `ev_0260017a6b0779b8c919250005169ee858841cd655a1fd35dcabfefaeb51dd46` | Character fixture builder exits successfully; 159-byte summary, 224-byte body. |
| `ev_b54479d1ae1dbda6fd66b12564c66147e46b3317076b3f1564f69326884e8984` | Extracted auth schema census accepts one complete 227-byte server-17 frame. |
