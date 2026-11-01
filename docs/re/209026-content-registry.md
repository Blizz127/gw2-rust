# Build 209026: appearance keys and content containers

Static REA analysis and a read-only archive check, 2026-11-01. Target identity
is recorded in [client-build.md](../client-build.md). No client UI progression
or character replay occurred.

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
values and runtime validation remain separate work.

The extraction checked the archive entry's hash against the location scan,
pointer bounds, relocation membership and string termination. Private data:

- `appearance-content-locations-209026-14-23.json`, SHA-256
  `c12065a4a807bd9b0f522f166667ccf79569cddbae63490ea0d76bedb9145c86`.
- `appearance-defaults-209026.json`, 67,801 bytes, SHA-256
  `ec1d5ea6af4b8e214ac6cc75567cd3a71aaf6749fc481e0274945bce08844163`.

Both are under `~/re/gw2/notes/`, mode 0600. These are shipped defaults,
not evidence of successful character creation or rendering.

## Evidence

Private bundle: `~/re/gw2/notes/209026-content-registry-static-1.json`.
Subsequent index/default evidence is retained in
`~/re/gw2/notes/209026-content-defaults-static-1.json`.

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
