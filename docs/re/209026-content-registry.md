# Build 209026: appearance keys and content containers

Static REA analysis and a read-only archive check, 2026-11-01. Target identity
is recorded in [client-build.md](../client-build.md). No client UI progression
or character replay occurred.

`gw2_dat::content::ContentIndex` exposes a borrowed view of the verified
64-bit `cntc` / `Main` version-0 header, type metadata, 16-byte index entries
and raw content bytes. It accepts the observed uncompressed-content mode
(flags bit 0 clear), checks relative array bounds and record offsets, and
does not claim to relocate content references or resolve GUID/name indexes.
Type metadata may be absent outside the first base container, as the client
uses the first container's shared type table.

Three focused tests and the library build pass. An offline REA smoke reads
all 32 base containers through the Rust view: **982,772 index entries**,
with the expected 1,554 type-45, ten type-137, 78 type-147 and one type-260
records. The first container has 483 type metadata entries; the other 31
have none. Selected record type fields agree with their index entries.
This verifies index framing, not complete record or relocation semantics.
Evidence: `ev_209d8f8c9838c51beeda293358f5a73c13e59625b6a55df5e694178278c859ae`,
private bundle `~/re/gw2/notes/209026-content-index-rust-check-1.json`.

`ContentIndex::reference` follows only the already traced local,
external-container and string-index fixup tables. It reports external
references as a container index plus content offset; the caller must select
and bound the target container. File references and unclassified nonzero
slots remain unsupported. String reads have an explicit UTF-16 unit limit.

Six focused content-reader tests and the library build pass. The reference
smoke reads all 32 frozen base containers through Rust: 78 palette names,
50 matching encoded palette queries, 90 distinct race/sex/profession default
strings accepted by the appearance reader, and 180 external equipment-template
references whose offsets lie inside their selected target containers. It
checks target bounds, not the complete target record semantics. Zero values
in classified reference slots are handled as offsets/indexes; unclassified
zero is null. Duplicate/conflicting fixups, unclassified nonzero references,
file references, bad local/string targets, invalid UTF-16 and string-limit
violations are rejected.

Evidence: `ev_c204d52a0a7b9366e52cf6a4a934520ad3db1f670fcbf76085d0db4159d6e5a5`,
private bundle `~/re/gw2/notes/209026-content-reference-rust-check-1.json`.

## Constant keys resolve through GUIDs

The [appearance keys](209026-appearance-validation.md) 50–59 select entries
at `0x1425e6980 + key * 72`. `0x1412dde50` checks the entry's key and type,
then uses a thread-local cache through `0x1412de200`. Each entry has its key
at +0, type at +4 and GUID at +8. The initializer beginning at `0x14020cdec`
writes all ten appearance entries with type 137; these entries are zero in
the executable's initial data image.

The cache miss calls content-service virtual +0x68 with the GUID. The
`CnContext` vtable at `0x14220e160` routes this through `0x1412e1a70` to
`0x1412e6410`, which searches two backend slots. A successful GUID-index
entry supplies a packed location to `0x1412e6df0`:

- High 8 bits select a container from backend +0x28, bounded by +0x34.
- Low 24 bits select a 16-byte index entry from container +0x2c.
- Index-entry +4 is added to container +0x80, the content base.

The outer lookup accepts the requested type or indirection type 150. The
latter must name the expected type at +0x28. This is not a direct mapping
from small appearance key to archive file ID.

The ten GUIDs were extracted privately with executable SHA-256 and
initializer-entry checks: `~/re/gw2/notes/appearance-registry-209026.json`,
953 bytes, SHA-256
`a4e8198f8734708989ae2f9bd8eac81544188f7163f8ffa54af2a84ffb9bf906`.

## Base content archive references

`0x1412e6e60` obtains the base list through `0x1417ed7b0`: **32** pointers
at `0x142648850`, targeting eight-byte file references at `0x142371b70`.
The loader `0x1412e7ae0` passes each through `0x140dbcba0` before opening
the archive entry and looking up its `Main` chunk. The file-reference decoder
uses `(word1 * 0xff00 + word0 + 0xff00ff01) mod 2^32`; the observed list
therefore names archive IDs **1282830–1282861**.

The loader applies local content relocations, external references and string
indices before indexing records. Consequently, raw archive offsets must not
be treated as already relocated pointers.

One bounded archive read through REA confirmed file **1282830** is a
64-bit `cntc` packfile, `Main` version 0, with a 13,195,848-byte payload.
The existing reader successfully decoded it with the historical 208039
`PackContent` schema: flags 2, 483 type records, 30,313 index entries and
8,828,472 content bytes. This confirms a structural match for this file;
it does not establish the semantics of every field.

## Appearance records and default strings

The index builder at `0x1412e8060` confirms the 17-byte type-info stride
and the first Dword's GUID offset. A bounded private scan used that metadata
to compare GUIDs at indexed record locations. It found all ten type-137
records in **file 1282852**; their GUID offset is zero.

| Constant key | Index entry | Content offset |
| --- | --- | --- |
| 50 | 303 | 129824 |
| 51 | 304 | 136984 |
| 52 | 307 | 159064 |
| 53 | 299 | 98128 |
| 54 | 302 | 120936 |
| 55 | 300 | 105088 |
| 56 | 305 | 143984 |
| 57 | 301 | 113352 |
| 58 | 306 | 151584 |
| 59 | 298 | 90976 |

The scan covered files 1282830–1282852. The first batch reached its 60-second
provider deadline after 14 completed files; REA retained that partial output
and the process was confirmed stopped. A smaller second batch completed.
Files 1282853–1282861 were unnecessary for locating these records and remain
unchecked; global duplicate/override behavior is not established.

`0x14129c240` takes an array at type-137 object +0x78, count +0x80,
with 24-byte entries. Entry +0x10 selects profession. The importer reads
the first pointer of the selected entry as a wide string. On disk, the
array pointer is a content-relative offset recorded in `localOffsets`, and
the string slot is an index recorded in `stringIndices`. Resolving that
index through the packfile's wide-string table yields **nine defaults per
race/sex, one for each profession 1–9**: 90 strings in total. Each sampled
string has `appearanceV0='...' armorColorV0='...'` sections. Parsing all
values and runtime validation remain separate work. The importer tail at
`0x14129dfd9` asserts that the parsed race/sex match the selected context.
After the 68-by-4 color loop, **Sylvari (race 4) overrides appearance byte 0
to 255 and byte 37 to zero** (`0x14129e185`). A reader intended to reproduce
imported defaults must retain this adjustment rather than copy the string's
45 bytes unchanged.

A first strict reader requiring 272 explicit color values rejected the
shipped data. An exhaustive shape check established that **all 90 strings
have 45 appearance bytes**, but 80 have **236 color values (59 rows)** and
the ten profession-9 entries have **240 values (60 rows)**. Parsed race/sex
match the corresponding registry keys. This is fewer values than the client's
68-row read loop. A local extraction reader reports the provided rows;
the subsequent static tail trace below separately establishes the importer's
zero-fill behavior for the absent values.
Request 415's rows 2–7 are present in every extracted default.
Nine profession-9 strings also contain CR/CR/LF whitespace adjacent to
three color tokens. A second strict-reader rejection exposed this formatting;
the token audit is retained as
`ev_2f3700e119920918406d72e4bab164af2092b3c2392b34a945d0a6ef742f8e1a`.

### Exhausted color tokens

The token helper `0x1409ba190` receives the delimiter set comma/apostrophe
at `0x14215f3b4`, an output buffer and capacity 16. At the end of a canonical
string it skips the closing apostrophe, leaves the cursor at the terminating
wide NUL, writes an empty terminated token and returns zero. The color loop
does not use that return value; its inspected assertion tests a different,
already nonnull appearance cursor.

Numeric helper `0x1409ba150` forwards to `0x140e66a40`, then `0x140e65234`
with base 10. Empty input consumes no digit, leaves its digit-seen flag clear,
and returns zero through `0x140e65900`–`0x140e65915`. Therefore the remaining
iterations write zero into the matrix. For the canonical extracted strings,
the static path fills rows **59–67** (profession 1–8) or **60–67**
(profession 9) with zero. This is a control-flow conclusion, not a runtime
observation. The extraction reader still preserves the distinction between
supplied and absent rows; request 415's selected rows are unaffected.

Private bundle: `~/re/gw2/notes/209026-default-color-tail-static-1.json`.
Evidence: `ev_dc97364b3576bd8077903246d681b4a2e315bb722be6532e5e4dc7605296959d`
(delimiter bytes and token-helper exhaustion),
`ev_983d3a5f097731a3a1e15d2e558593a37b7e91fda6b15213e6b4d8c171136f67`
(numeric wrapper),
`ev_00ef8c767add772b29d1f80bb1e26bb564468950e2e598555a400f9000e6f755`
(numeric setup), and
`ev_6390bca441802fb8b37f337cc06f4f42fa19bd9d1401bb1fa9a6b66e3089d99b`
(no-digit zero-return branch).

The extraction checked the archive entry's hash against the location scan,
pointer bounds, relocation membership and string termination. Private data:

- `appearance-content-locations-209026-14-23.json`, SHA-256
  `c12065a4a807bd9b0f522f166667ccf79569cddbae63490ea0d76bedb9145c86`.
- `appearance-defaults-209026.json`, 67,801 bytes, SHA-256
  `ec1d5ea6af4b8e214ac6cc75567cd3a71aaf6749fc481e0274945bce08844163`.

Both are under `~/re/gw2/notes/`, mode 0600. These are shipped defaults,
not evidence of successful character creation or rendering.

## Bounded local reader

`gw2_dat::appearance::AppearanceDefault209026` reads these two explicit
decimal sections, bounded by 4096 bytes. It checks exactly 45 appearance
values, byte range and expected race/sex, then applies the Sylvari edits.
It retains only supplied color rows: 8–68 complete rows of four `u32`
values, allowing ASCII whitespace around tokens. `creation_colors()` returns
rows 2–7 in wire order. It rejects incomplete rows, numeric overflow, extra
sections and trailing data. This is a strict extraction reader, not a complete
emulation of the client's permissive importer.

`gw2-default-check FILE RACE SEX` reads at most 4097 bytes and prints only
structure/counts, with no appearance/color values. Three focused unit tests
cover malformed input, truncation, bounds, race/sex, Sylvari adjustments,
short tails, wrapped tokens and selected-row order. Tests and checker build
pass. A private REA check passes all **90/90** extracted defaults and five
CLI rejection cases. Runtime creation/rendering remains untested.

## Evidence

Private bundle: `~/re/gw2/notes/209026-content-registry-static-1.json`.
Subsequent index/default evidence is retained in
`~/re/gw2/notes/209026-content-defaults-static-1.json`.
The importer-tail evidence is in `209026-content-defaults-static-2.json`.
Corpus shape/token checks and final reader validation are retained in
`209026-content-defaults-static-3.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_47fb25da5feca749879bd7db0a91216115e6b9082b43bc9530602891439ed3c0` | Constant-key lookup and type checks. |
| `ev_fda31e8f1e2b95a8aad5c54f10f5fe33b157552e5afc333478d7d1c668196f13` | Cache and GUID dispatch. |
| `ev_628b1d4bfe284ee02149189851e789823abf29ee9347f251131c3c1633791005` | Context vtable and appearance initializers. |
| `ev_4f1460f2925547ad74ab8276720d794b589f1ab49efcec3531f8ff2645660701` | Guarded GUID extraction and backend lookup. |
| `ev_10cb1e529b96e4c5f23463aff86f81a7afd6fcb0f319915bc16b955ac63e53e9` | Packed container/index resolution and base-loader entry. |
| `ev_3fb9bcc46b00cedab99acbb61d118e7120473329423344b38a18f08dea21b62c` | Relocation pass and 32-entry base list. |
| `ev_1e5746037cca13609173b3f5775adcb5f701e6ae34ebcbaf9424f00849397b0a` | File-reference pointers and archive loader. |
| `ev_cc1d027c7c5fbc032b0df7fa3a223362c390961213afc9fb97092790cc5e90f8` | Encoded references and file-ID decoder. |
| `ev_72d876d41d7dc8abbb2874462d3c1b1439608a1c8a4775f49cc4924f4caa9638` | First content-container archive decode. |
| `ev_43a0ed38297a3f3da8cfd176dc9b30734f549b6411b2612a34e19a196c029dea` | Type-info stride and indexed GUID construction. |
| `ev_53e290ebfe771805152be339d2db97add06b2cfe6e2a257eedfaafaa9a42736f` | Partial first scan, 14 completed files before deadline. |
| `ev_6e0d2f90c013f2831ac0e1b3a7b1a5ccd958489bc92fc6bd16c41d4e22ea92af` | Completed second batch; all ten GUID matches. |
| `ev_825763585720c7fd2c3804e3c0304fab29d3e69884f85cc536d2e4a28ff5cbf0` | Record locations and type-137 metadata. |
| `ev_2378d7531511f446989794f1a072207dcdc90957c45f9ea4e2948937d52eddee` | Default array offsets and importer string access. |
| `ev_e3de06eb630a74a5216b2338d5640f02c2b2ef91873a059ea100088c9df9e771` | Relocation-checked extraction of 90 defaults. |
| `ev_f4086219017949db5c5a51294252e034bf5d79bc613ecb1b23800b1a5f9082e2` | One extracted default's section structure. |
| `ev_56f9032195eeff1db875bc739fdd69808a474cb9c3fb91ca1a6dec45e5022fda` | Race/sex assertions, 272 color reads and Sylvari overrides. |
| `ev_4d90f2db1f2af032d88831d25dbeabf1a0e8eb8b2e2340aa501e4db5f2a6444d` | Initial strict reader rejection; not a successful corpus check. |
| `ev_61d1a79d57344ffb2c32f794c29f3f7cfb3ba6211e2b49feb04e20d90f244f14` | All 90 strings: appearance/color arities and context values. |
| `ev_01346f3afd4df4c9d95736f15d3c5bc041fc90dc7dc65b1a9217d17d86252e8f` | Final reader: 90 corpus passes and five CLI rejection cases. |
