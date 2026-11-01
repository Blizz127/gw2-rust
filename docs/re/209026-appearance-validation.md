# Build 209026: appearance validation and content keys

Static REA analysis, 2026-11-01, on the frozen executable identified in
[character creation](209026-character-creation.md). No appearance fixture
was sent and no content defaults were extracted from the archive.

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
and indirection formats remain unresolved.

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

Private bundle: `~/re/gw2/notes/209026-appearance-validation-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_55953e9da54e32549a0ff2b5a8dcd7fb23a91e4fe4a908e4fb47285e60157d04` | Complete per-byte validator body and branch targets. |
| `ev_a4f5fcc378d980d2da0fc5b8276105050f7aba0c1997e44df0858245704653bf` | Type-137 lookup, race/sex indexing and property-enum mapper. |
| `ev_8da9ee3fcf5a128962460c90128ab73feb15f07c58b960886d5a884bbeceac5e` | Hash-guarded 45-entry branch decode, content keys and generic property walk. |
| `ev_2c8d1584cf494501809fcd67e063b5ba2db5986af9e395c62a91fc509e9ea119` | Generic upper-bound check and count helper. |
