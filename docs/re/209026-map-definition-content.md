# Build 209026: map-definition key and Queensdale content

Static REA analysis and read-only archive parsing, 2026-11-01. Frozen target
identity: [client-build.md](../client-build.md). No client launch or mission
join occurred.

The summary callback `0x141419e40` calls content-service virtual +0x1a8.
The `CnContext` vtable entry at `0x14220e308` resolves to `0x1412e1d30`,
which forwards the supplied key to `0x1412e68e0` with **content type 45**.
That helper queries the backend +0xb8 index using
`(type << 22) | (key & 0x3fffff)` and resolves the stored container/index
location through `0x1412e6df0`.

The index builder at `0x1412e82b2` obtains this key from the third Dword
of the 17-byte type metadata: the historical schema's `dataIdOffset`.
Zero data IDs are skipped. The frozen type-45 metadata is
`[guidOffset=0, uidOffset=20, dataIdOffset=40, nameOffset=24, track=0]`.
Thus **map definition +0x28 supplies the summary lookup key**; this lookup
does not use the separate UID at +0x14.

A scan of all 32 [base containers](209026-content-registry.md) found
**1,554 indexed type-45 records**. The subsequent data-ID extraction found
1,554 distinct nonzero IDs, with no duplicates in those containers.

## Key 15 and map file 192711

Key **15** resolves to one base-content record:

| Property | Value |
| --- | --- |
| Content-container archive ID | 1282830 |
| Index entry | 15908 |
| Content-relative record offset | 3754040 |
| Content type | 45 |
| Data ID at record +0x28 | 15 |
| File-reference relocation at record +0x68 | Archive ID 192711 |

The file reference was resolved through `fileIndices` and the container's
encoded file-reference table. Its relocation offset is 3754144; the nearest
preceding indexed record is the type-45 record above. It names the Queensdale
map file already identified in [client-build.md](../client-build.md). This
provides a frozen-content link between summary map key 15 and Queensdale.
The exact +0x68 field consumer has not yet been traced; this is archive
reference evidence, not a successful map-loading run.

The summary's `u16` can represent key 15, and the base content contains the
record required by its callback. Actual runtime content registration and
character preview/mission behavior remain untested. Other unresolved summary
fields are unaffected by this finding.

Private extraction: `~/re/gw2/notes/map-definitions-209026.json`,
181,985 bytes, mode 0600, SHA-256
`8d47da7f833ce2c536d2143ce051a7557baa609983c098f506d8398a36ad2cd5`.
It records source-container hashes, record locations and the map-file link.

## Map-specific reward-track selection

The [reward-track slot follow-up](209026-reward-track-slots.md) checked
this same key-15 record's array at +0x178/count +0x180: count is zero.
Server 566 requires an entry index below that count, so no such message
is valid for this base Queensdale record. This does not constrain the
separate active reward-track slots or other progress systems.

## Evidence

Private bundle: `~/re/gw2/notes/209026-map-definition-content-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_840e4e38a47467fba461fbf6202a433d67b36ac20700576c9bd26d031e009b29` | Callback and vtable entry. |
| `ev_61e4407e38a264830489a8157880bbc5a87a76b5236595ae48afbd6600df6cfc` | Type-45 dispatch. |
| `ev_5768f2e5847855c4ad238675610e6a07f00bf174439d06e59bde43996050fa08` | Backend data-ID index query. |
| `ev_9fb0abb0182429bb53964a68f9bf719dd327b26a99ca7867f988b2a84804a1e8` | Index construction from type-info +8. |
| `ev_e4b0ee1e5afe4781e07a17c2b4040e96d2c594f871dee4f8fd59d43291dbd332` | All 32 base containers, 1,554 type-45 entries. |
| `ev_30f5646262f59fb96f1af376f47df67d7c4c14831bf7937e415209b43dfeead0` | Metadata, unique data IDs, key 15 and file-reference resolution. |
| `ev_84924034c6a1846f6a344f5ce83aef1861b29bc86a02f6692d2a9a331b7822bd` | GameLoad consumer repeats the same map-definition lookup. |
