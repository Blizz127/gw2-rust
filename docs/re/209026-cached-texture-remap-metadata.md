# Cached texture remaps: build 208784 metadata, build 209026 asset comparison

The original download prefix contains explicit MFST type-0 mappings for all
17 requested texture IDs. Its cached root and linked manifest both declare
**208784**, so this is **not authenticated build-209026 manifest metadata**.
All 34 original/remapped record sizes and file-ID aliases agree with the
frozen 209026 archive. That supports a private compatibility experiment;
it does not prove that build 209026 would select this older manifest.

The mapping values below were read from four-byte property payloads. They
were not inferred from neighboring IDs. The client policy is documented in
[the remapped-file selection trace](209026-texture-remapped-file-selection.md).

## Cache provenance and exact root link

The read-only candidate was
`/home/blizz/gw2-client/wineprefix-dl/drive_c/users/devbox/AppData/Roaming/Guild Wars 2/Local.dat`.
It is a 40,375,192-byte AN97 archive with 323 MFT entries. Its recognized
40-byte archive header has SHA-256
`25a7d02bc51cf334b9eabbbb3cf75c461b5e2840f2163c4591bf4388441329ce`.
No whole-file hash or credential/preferences content was exported.

| Artifact | Exact identity |
| --- | --- |
| Cached root | MFT index 16; indexed IDs 4101 and 3831448; PF/ARMF with ARMF v1 chunk; build 208784; 20,508 decoded bytes; 305 manifest entries |
| Root SHA-256 | `d30dd8e2ee4f0a3de3c8e9cf31fee6e64c659c9b194e20fa0f0a32bdf4364413` |
| Selected root row | Base 970570 → file 3831142; size 2,919,828; raw flags 129 |
| Linked manifest | MFT index 75; indexed IDs 970570 and 3831142; PF/ARMF with MFST v6 chunk; build 208784; 2,919,828 decoded bytes |
| Manifest SHA-256 | `3a2592698686b6811df05cd703a3f3d3b12bafcc9ed392a45c3d57d1ca912a17` |
| Manifest counts | 114,978 records; 26,322 properties; 26,322 property-table rows |

This is an exact root-to-manifest link, not a conclusion from the prefix
directory name or file modification time. The recognized cached root is
also older than 209026; it does not supply the missing current-build link.
An exact lookup of source-defined root base 4101 in frozen `Gw2.dat` was
absent. No other frozen-archive payloads were scanned for manifests.

## Selected properties and record sizes

Every listed property is type **0**, length **4**, and appears once among
the selected target rows in this manifest. Sizes are the manifest records'
decoded byte counts. The private extraction retains each property's index
and both full `{baseId,fileId,size,flags}` records, including cases where
the installed file ID differs from its base ID.

| Original base ID | Remapped base ID | Original bytes | Remapped bytes |
| --- | --- | --- | --- |
| 139375 | 139376 | 4,548 | 9,208 |
| 139377 | 139378 | 3,884 | 8,284 |
| 139379 | 139380 | 3,724 | 8,060 |
| 139381 | 139382 | 55,032 | 218,108 |
| 139383 | 139384 | 262,164 | 1,048,596 |
| 140115 | 140116 | 5,448 | 11,924 |
| 140117 | 140118 | 8,024 | 18,140 |
| 140119 | 140120 | 988 | 2,156 |
| 140121 | 140122 | 252 | 932 |
| 140123 | 140124 | 57,272 | 231,176 |
| 140125 | 140126 | 131,092 | 524,308 |
| 140515 | 140516 | 1,836 | 4,436 |
| 140517 | 140518 | 2,360 | 5,244 |
| 140519 | 140520 | 2,908 | 6,912 |
| 140521 | 140522 | 47,312 | 184,308 |
| 140523 | 140524 | 262,164 | 1,048,596 |
| 142103 | 142104 | 32,240 | 129,308 |

The followup queried only these **34 exact IDs** in frozen
`/home/blizz/gw2-client/build-209026/Gw2.dat`. Every decoded size matched,
and every cached record's `fileId` was an alias of the same frozen MFT entry
as its `baseId`. For example, original 140123 aliases file 315194, while
remapped 140124 aliases file 1099086. This check used index/MFT metadata and
decoded-size headers, not a full archive sweep or per-file pixel comparison.
Both archive files retained unchanged size and modification time.

Root separately extracted three exact mapped assets from frozen `Gw2.dat`.
The native header contract gives **140124: DXT5, 1024×512**, **140122:
DXTA, 1024×512**, and **140522: DXT5, 1024×1024**. These exceed the
corresponding originals' 512×256, 512×256 and 512×512 dimensions. The
run-137 relative rectangles fit the first two mapped headers. This is
asset evidence, not a runtime file-selection or repaired-image claim.
Evidence `ev_78ee5dd9c4f5b932624629b48cccd3bf142b904d091036203dddb7f7a96a8e4d`
retains the bounded extraction and unchanged frozen archive identity;
per-asset hashes are in
`~/re/gw2/notes/remapped-texture-assets-20261105/receipt.json`.

## Reader boundaries and superseded results

The first narrow reader admitted only **PF container type MFST** and found
none. That result did not cover the authentic **PF/ARMF container with an
MFST chunk**, recognized in a subsequent bounded header pass. It must not
be cited as absence of cached manifests. The corrected initial three-target
search stopped after finding all three mappings: 59 recognized MFST bodies,
12,620,196 decoded bytes. The 17-target expansion then read only the pinned
MFT-75 manifest, plus recognized root ARMF bodies for the provenance link.

An intermediate reader also used incorrect packed property/table count
offsets and stopped at its bounds check. The corrected MFST v6 count offsets
are **36/48 for 64-bit pointers**, **28/36 for 32-bit pointers**. Focused
synthetic checks passed for both layouts and truncated-array rejection.
The failed capture and corrected results are preserved together. No payload
from the failed interpretation was used as a mapping.

Nonmanifest entries were inspected only through bounded decoded headers;
their bodies, names, preferences and credentials were not emitted or parsed.
There was no working-archive access, client/container launch, endpoint call,
source mutation, production manifest change or runtime-selection claim.

## Evidence and private artifacts

- Root link and exact 17 properties/34 records:
  `ev_ee27b2e963922eeb579a31fb8d2527768410cacb76cf0c741f429923220afa90`.
- Exact frozen-index comparison:
  `ev_ff1b9eb180d5e549939d4f8881dac37646c7bc9c2b29b6182cd2950caa7e5a6f`.
- Corrected initial three-target discovery:
  `ev_f1a84a46b76a160b92db83380e6bdc933efbdfc48e5d5006e49091ccf9cd8477`.

The authoritative root/17-target capture is untruncated. The comparison and
three-target discovery retain every raw terminal record; their truncation
flags concern repeated rendered frames. The earlier magic inventory lost
raw rows to its output limit, so it is not used as an exhaustive inventory.

Private artifacts under `/home/blizz/re/gw2/notes/`:

| File | SHA-256 |
| --- | --- |
| `208784-cached-texture-remaps-exact-20261105.json` | `a982acb84a1ac5d0761fb46f8de1021f97d0e70dfe3806c199f21780327410c0` |
| `208784-remaps-vs209026-index-20261105.json` | `24988164df9122224c466e54e30daf3f91c300652f84c31b8e1f7b3ba64b9acb` |
| `208784-cached-texture-manifest-evidence-20261105.json` | `828bb63e917030e7a1ab05e225608f25a3d46627f39b135d0c2168e5e6f0ff91` |

The focused reader is `read-original-exact-remaps-20261105.rs`, SHA-256
`1c047926a415873d169ddd0230f6619d1cffc51666980062b28e04fc39765edc`.
