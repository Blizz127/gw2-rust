# Build 209026: appearance validation and content keys

Static REA analysis, 2026-11-01, on the frozen executable identified in
[character creation](209026-character-creation.md). No appearance fixture
was sent. Subsequent work extracted all 90 race/sex/profession defaults;
see [content registry](209026-content-registry.md). The branch map below
still does not establish full content-dependent validity of those defaults.

## Race/sex content selection

The nonzero-byte path of `0x1411ca0d0` checks appearance byte 27 (race < 5)
and byte 29 (sex <= 1), then calls `0x1411beed0(race, sex)`. That helper
indexes the Dword table at `0x14215f248` with **race * 3 + sex** and passes
the selected key and content type **0x89 (137)** to `0x1412dde50`.

| Race | Sex 0: Male | Sex 1: Female |
| --- | --- | --- |
| 0: Asura | 51 | 50 |
| 1: Charr | 53 | 52 |
| 2: Human | 55 | 54 |
| 3: Norn | 57 | 56 |
| 4: Sylvari | 59 | 58 |

The race/sex labels come from the previously mapped embedded getters.
These numbers are **content keys, not archive file IDs**. When the lookup's
out-parameter is nonzero, the helper resolves result +0x30 through
`0x1412e38a0`; it finally requires a nonnull result. The exact content-storage
and indirection formats are being traced in
[content registry](209026-content-registry.md), which identifies the base
archive containers and the intermediate GUID lookup.

The same helper supplies the creation defaults described in
[creation colors](209026-creation-colors.md). This provides a concrete
content lookup to pursue for defaults instead of guessing an appearance.

## Per-byte branches

The validator first returns true for any zero byte. For nonzero values,
after the race/sex checks and content lookup, its byte-index table at
`0x1411ca71c` selects a branch through `0x1411ca6cc`. A hash-guarded decoder
enumerated all 45 entries.

| Appearance offset | Established branch behavior for a nonzero value |
| --- | --- |
| 0 | Uses generic content-property validation for Sylvari; otherwise compares against `0x1412d9fa0`'s count, admitting equality. |
| 1, 5 | Uses content-index lookup (`0x1412da060` / `0x1412da310`) and a subsequent `0x1412d9da0` lookup; packed key construction is not yet named. |
| 2 | Requires value < selected type-137 content's +0x30 count. |
| 4 | Requires value < `0x1412da250`'s result. |
| 12, 13 | Resolve a key through `0x1412da5c0` and require a match in the selected content's +0x38 array, count +0x40. The branches combine bytes 12/13. |
| 14, 30, 32, 44 | Obtain a palette key through `0x1412da6e0`, `0x1412dabd0`, `0x1412dad10`, or `0x1412da410`; look up type 0x93 and require index < palette +0x38 count. With the validator's third argument zero, selected record +0x14 must also be zero. |
| 15 | Similar palette validation using `0x1411c3180`, but a missing palette is accepted by this branch. |
| 16 | Resolves a key through `0x1412da830`, combines byte 17, and requires a match in content +0x48 array, count +0x50. |
| 17 | Immediately accepted after the common checks; byte 16's branch still consumes it. |
| 27 | Race < 5. |
| 28 | Value <= 15. |
| 29 | Sex <= 1. |
| 31 | Requires value <= `0x1412dade0`'s result. |
| 37 | Special nested palette validation for Sylvari; otherwise generic content-property validation. |
| 3, 6–11, 18–26, 33–36, 38–43 | Generic content-property validation through `0x1411cbf30`. |

The generic helper walks nested property records, maps each property enum
through `0x1411bf740`, and looks for the requested appearance offset. A match
must have record kind 2 and a nonnull data pointer; the byte must be <= the
first Dword of that data. **If no property matches, the helper returns true.**
Thus passing this validator alone does not prove that every byte has a
known content meaning or that a character will render correctly.

The existing creation checker deliberately reports the earlier context
predicate separately from content-dependent validity. It has not been
changed to imply these archive-dependent checks passed. The three wire
bytes beyond the 45-byte appearance remain opaque.

## Evidence

### Shipped-default generic-bound audit

An offline reconstruction followed the selected type-137 record's +0x68
array (count +0x70, stride 48), each entry's +0x18 pointer, its +8 array
(count +0x10, stride 48), and each group's +8 property array (count +0x10,
stride 24). Property +0 supplies the enum mapped by `0x1411bf740`; +8 is
the kind and +0x10 points to the bound data for kind 2. The audit retains
the **first** matching property, matching the client's early return.

All 90 shipped defaults were checked after the known Sylvari adjustments.
Of their 4,050 appearance bytes, 795 take the validator's immediate zero
success path, 1,916 nonzero bytes pass an explicit generic-property bound,
and 495 nonzero bytes take the generic helper's missing-property success
path. **844 nonzero bytes use specialized branches and were not checked by
this audit.** No generic bound failed. Missing-property success is not a
content validity check, and these counts do not establish full validation
or rendering. The existing public checkers retain their limited claims.

The ten race/sex definitions contain 30–39 property records. Several map
more than one property to the same appearance offset, making first-match
behavior relevant. Relocation checks and source-container hashes are
retained in the private result
`~/re/gw2/notes/appearance-generic-bounds-209026.json` (39,942 bytes, mode
0600, SHA-256
`0b5c521640bd2fe90ddcf8af0614cb405d1ccad38df3808d0ff8c8b95a55632e`).

Evidence bundle: `~/re/gw2/notes/209026-appearance-generic-bounds-1.json`.
`ev_37a55782693ea0d86c7cd2372c1f1e6dd0b8350241603a7a4c5d2e564b87ca55`
records the nested traversal and enum mapper;
`ev_83589b2cb9b16d45ce92caec2957686b0d728b5cc21c61713e56fe5271774a7a`
records first-match kind/pointer checks and the inclusive bound; and
`ev_cdb7cf86c028e567cef9c37ec32f80ac7d56897e17add59803bb613ea5b25d46`
records the hash-guarded archive audit.

### Composite profile counts

The count helpers read a separate **`cmpc` file, `comp` chunk version 19**.
Startup at `0x14094c7c5` passes the encoded file reference at `0x141b7ee54`
through `0x1412d88b0` / `0x1412db250` to `0x1412dba00`. Decoding its two
Words with the established file-reference rule yields archive ID **154681**.
The loader requests chunk `comp` and version 19 (`0x14235ffc0`), retaining
the chunk at TLS +0xd0 context +0xb8. The frozen archive matches both.

The chunk has 29 profile records: count +0x24, pointer +0x28, stride 0xe0.
`0x1411c67c0` selects a UTF-16 name from `0x1425bdec0[race*3+sex]` and
packs it with `0x140e4cb60`. All ten selected names are lowercase letters,
at most twelve characters; each letter contributes its 1–26 value in
successive five-bit positions. `0x1412db990` matches that packed key against
profile +8. All ten race/sex keys have unique matches in the archive.

| Appearance byte | Count source | Nonzero validator comparison |
| --- | --- | --- |
| 0, except Sylvari | Composite profile +0x18 | value <= count |
| 2 | Type-137 appearance record +0x30 | value < count |
| 4 | Composite profile +0x3c | value < count |
| 31 | Composite profile +0x94 | value <= count |

A second offline audit checks these counts plus the already traced simple
branches at offsets 17/27/28/29. It passes **336 additional nonzero bytes**
across the 90 defaults, without recounting zero bytes or Sylvari byte 0.
There are no nonzero byte-17 values in this corpus. Together with the
generic audit, this leaves **508 nonzero bytes** needing packed-key or
palette checks. These remain partial static checks, not a runtime validation.

The private decoded file `~/re/gw2/notes/appearance-comp-154681.pf`
is 14,641,324 bytes, SHA-256
`94796c36a36c506ed4d616d798d8c90ae44dfed49372b21ff66a37f3299ab7f5`.
The count audit checks relative pointers against the chunk's fixup table
and retains its result in `appearance-count-bounds-209026.json`
(2,890 bytes, SHA-256
`b7333d90998cb3d21f1c6d9e432008f3300622d8fd80a0e70c11714774576eff`).
Both files are private, mode 0600.

Evidence bundle: `~/re/gw2/notes/209026-appearance-comp-counts-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_48dbc613737ca5f167c53ab05a45b1671f12597669f809780ac106eee382e6f3` | Composite chunk loader and profile stride. |
| `ev_62268d982d5f13db64f4e9ec083c71cad9dd72bf3fe08bb5a06a0f3992cb76a9` | Startup passes the embedded file reference. |
| `ev_9cf17087b771b1bf329e71528bbc01839ba643ab71f9f96c616e23027902d1de` | Hash-guarded reference decode and frozen archive extraction. |
| `ev_d61d9218defbd60fa1a4ccb1008e6f1073169b92d69c9a8164b2f6d8b31f2abc` | Three profile count getters. |
| `ev_f225678109ad2307c18b1d823989d2a78ae1407141283b81075472eccd4c465e` | Race/sex profile-name lookup. |
| `ev_ca3853117583a16046598f69b4baa5ef83bb9020e1a088e151d1bfc12d587b77` | Name table and packing routine. |
| `ev_f664f16a40bc3864b98d1e6bb0fa6429e57d4f50d0ff8dbce0989bb1e2cf78cf` | Selected names and five-bit packing loop. |
| `ev_ae2bed1b5bb8b29ca3887ceabfac0bab8bc922252a9ee143f5179d863aa1a632` | All ten profile matches and 336 additional nonzero checks. |

### Original branch mapping

Private bundle: `~/re/gw2/notes/209026-appearance-validation-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_55953e9da54e32549a0ff2b5a8dcd7fb23a91e4fe4a908e4fb47285e60157d04` | Complete per-byte validator body and branch targets. |
| `ev_a4f5fcc378d980d2da0fc5b8276105050f7aba0c1997e44df0858245704653bf` | Type-137 lookup, race/sex indexing and property-enum mapper. |
| `ev_8da9ee3fcf5a128962460c90128ab73feb15f07c58b960886d5a884bbeceac5e` | Hash-guarded 45-entry branch decode, content keys and generic property walk. |
| `ev_2c8d1584cf494501809fcd67e063b5ba2db5986af9e395c62a91fc509e9ea119` | Generic upper-bound check and count helper. |
