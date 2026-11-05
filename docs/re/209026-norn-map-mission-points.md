# Build 209026: bounded Norn map mission-point decode

Static frozen-archive REA inspection, 2026-11-05. This follows the untouched
`msn` chunk identified in the [Great Hunt species-link audit](209026-great-hunt-species-link-audit.md).
It establishes three authored text-labelled records in the map asset shared by
maps 379 and 28. It does **not** establish a retail player spawn, NPC identity,
placement, or which record any particular gameplay path selects.

## Exact artifact and bounded decode

Archive file **187611** retained its previously established decoded SHA-256:
`2dc9c92929362a10cdb22b5350957eac764ccc1d53c8a97b01c5798223211bbc`.
The PF container has flags 5 and 64-bit relative pointer slots. Its `msn\0`
version-2 chunk starts at decoded file offset **2,979,748** and has a
**200-byte payload**, SHA-256
`65f7be0ef1fb73a3fa3fb79a817e509e8fa013fc51a5a45b91e800e08bb2d095`.

The vendored **208039 historical schema**, `MapMission` version 2, describes an
`interestPoint` array with `position: Float3`, `forward: Float3`, and a wide-string
`description` pointer. The subsequent native 209026 trace below confirms those
field names and the consuming layout. Applying it to these exact bytes yields:

- Count 3 at payload +0; relative pointer slot +4 resolves to +16.
- Three 32-byte records at payload +16, +48 and +80.
- Each record has six floats and a relative pointer at record +24.
- The declared fixup table is exactly `[4, 40, 72, 104]`, matching the array
  pointer and the three string pointers. Strings resolve to +112, +160 and +190,
  terminate within the payload, and decode without replacement characters.

| Index / payload offset | Exact UTF-16 text | First three floats | Following three floats |
| --- | --- | --- | --- |
| 0 / +16 | `Norn Homestead Greenies` | `[-158.4352569580078, -32513.3203125, -1364.9456787109375]` | `[0.3761749267578125, -0.908203125, 0.1878662109375]` |
| 1 / +48 | `Mountain River` | `[8685.2119140625, -30712.8125, -196.51795959472656]` | `[0.6083984375, -0.791015625, 0.060760498046875]` |
| 2 / +80 | `cave` | `[-10453.673828125, -42241.8125, -579.4468383789062]` | `[-0.314453125, -0.9453125, -0.1024169921875]` |

The numeric values above are unchanged decoded floats. No coordinate-system
conversion, facing conversion, surface query, or spawn selection was performed.
In particular, `Norn Homestead Greenies` is not evidence that record 0 is the
Great Hunt player's start or an NPC spawn group.

## Native 209026 loader and indexed reader

The exact `msn\0` byte query in mapped `.text`, `.rdata` and `.data` found eight
occurrences: five instruction immediates in two functions and three format
registration entries. The two complete inspected functions are:

- **`0x140c3be90..0x140c3c0cd`**: obtains the `msn` chunk through the input
  object's virtual +0x58, reads count +0 and pointer +4, and iterates source
  records with stride **0x20**. It copies XYZ from +0/+4/+8 and the next vector
  from +0xc/+0x10/+0x14 without scaling or axis changes. The string pointer is
  +0x18; null selects the literal **`Old Point`**. It calls text-copy helper
  `0x1409b8660` with limit **0x80**, then appends the resulting **0x118-byte**
  record, whose trailing 0x100 bytes hold the copied label. The returned
  object's array pointer is +0x10 and count is +0x1c.
- **`0x140c3c0d0..0x140c3c14d`**: obtains the same chunk, forwards it with
  its schema descriptor to a second object's virtual +0xa0, and releases
  the input chunk. It does not select or consume an individual point.

The executable itself contains the corresponding schema names and references:

| Address | Native descriptor/string |
| --- | --- |
| `0x141e4d1c0` | Element schema; names `position`, `forward`, `description`. |
| `0x141e54850` | `PackMapInterestPoint` |
| `0x141e4d240` | Array schema referring to the element descriptor. |
| `0x141e54868` | `interestPoint` |
| `0x141e54878` | `MapMission` |

Module descriptor **`0x141d62348`** has numeric ID **11**, name **`Mission`**,
and factory pointer **`0x140c3be90`** at +0x20. Getter `0x140c3c3f0`
returns that descriptor; a verified call at `0x140c39d53` passes it to the
module-registration helper. The factory installs vtable **`0x141d62390`**:

| Virtual slot | Function | Established operation |
| --- | --- | --- |
| +0x28 | `0x140314f20` | Returns the count at object +0x1c. |
| +0x30 | `0x140c3c320` | Checks the supplied index against that count, then copies exactly one 0x118-byte record from object +0x10 to the caller's output buffer. |

The reader's complete range is `0x140c3c320..0x140c3c3e2`, including its
out-of-range assertion. These functions establish indexed data access; they
contain no choice of the Great Hunt start, named NPC placement, player
position update, or camera selection. Their callers remain the missing link.
They also do not prove that the data is exclusively debug/camera content.

## Bounded downstream lookup and next dependency

The known scene constructor at `0x141095070` obtains map module 16 through
map virtual **+0x130**. Reading that slot from the established map vtable
`0x141d61b68` gives **`0x1409493d0`**. A bounded instruction-pattern query
then looked for `mov edx,11` followed within the next 96 bytes by either
a disp32 virtual call through +0x130 or a direct call to that exact target.
It found **zero byte candidates** in `.text`.

That negative result is specific to those encodings and proximity. It does
not cover constants passed through another register, other module lookup
interfaces, inlining, cached pointers, indirect transfers, or callers of the
indexed getter whose receiver is already available. No whole-client absence
claim follows from it. Raw byte candidates, if present in a later query, must
be checked against instruction boundaries and receiver identity.

The smallest remaining dependency is now a **typed caller of Mission virtual
+0x30**, with its receiver proven to originate from module 11, and the source
of its record index. That would distinguish a character/map-entry path from
a camera/debug interest-point path or another subsystem. Any spawn claim
additionally needs the caller's coordinate conversion and an explicit link
selecting that record for map **379**: the same archive map file serves **28**,
so asset presence alone does not identify tutorial entry semantics.

This is a smaller next query than another species-name or all-container scan.
The existing [plain-name coverage](209026-species-plain-name-coverage.md) found
no exact readable Thora/Ido/Issormir or trophy-enemy match. The bounded quest/map
relocation audit also supplies no species/placement link. Those limitations
remain unchanged, and no generic combat fixture is retail population evidence.
The [saved Norn profile](209026-saved-norn-creator.md) still relies on an explicit
diagnostic world position; this finding does not replace that setting.

## Evidence and access boundary

REA evidence:
`ev_8e08599a373e1eef7c7f396530d78db338fed58bd14f47798a8509cc933c0eb0`.
The capture exited zero, was not truncated, and verified owned-process cleanup.
An unrelated inaccessible `/proc` entry was left untouched and remains a
recorded process-observation limitation. This was a helper process inspecting
static archive bytes, not game execution.

Before reading, the helper checked that frozen archive
`/home/blizz/gw2-client/build-209026/Gw2.dat` (device 2065, inode 293163139)
differs from the working archive (device 2065, inode 293170927). The working
archive was only statted. Frozen size, inode and modification time were unchanged
afterward. Extraction used the existing `gw2-dat-cat`, SHA-256
`ecff83ff81bcff68d418698f98363688c665401a4b0a4307658b07ce86dc812b`;
the full decoded file hash above was asserted before interpreting the chunk.
No shared client, archive mutation, runtime reservation or deployment was used.

Private retained artifacts under `/home/blizz/re/gw2/notes/`:

- `209026-norn-map-mission-points-20261105-evidence.json`: complete returned REA evidence.
- `norn-map187611-msn-v2-20261105.py`: exact bounded extraction and layout checks.
- `norn-map187611-msn-v2-20261105.json`: raw 200-byte payload, fixups, offsets and decoded records.

Existing task-owned notes were left unchanged. No server defaults or gameplay
implementation changed.

### Native consumer evidence addendum

The follow-up read only frozen executable `Gw2-64.new.exe`, asserting SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`
before each query. It reused the private bounded PE/pointer/disassembly helpers;
no Ghidra import, client process, archive read or working-client access occurred.
Complete returned evidence is retained in
`~/re/gw2/notes/209026-norn-map-mission-consumer-20261105-evidence.json`
(368,817 bytes), SHA-256
`8399b589e7380328b8c0d4d9988c97016c239b15997210b0af39198456db110d`.

| Evidence | Scope |
| --- | --- |
| `ev_a1f1bcbac36ad24375181839231d546fe350802bf7bbf54b1d74eaa3918ee167` | Exact chunk-tag/name seeds and mapped addresses. |
| `ev_f722432c838a696432b1eef9649be495306c627590b256122b9c96962f51b49d` | Complete chunk loader/forwarder; native schema pointers. |
| `ev_ea6d434fe850a7cf6bcfa757e9bd3386c4779df51c3a16a03ef8d8f56003b6bf` | Factory descriptor, Mission vtable, field descriptors and `Old Point`. |
| `ev_096b563ba45a62f13694cab275ff907f6ff22547c785235b23b19d51c6b64c7e` | Complete indexed-copy reader, count accessor and descriptor getter. |
| `ev_4912a20dda8242ee564041e5b4764d6a96351262681a05668237e86fe61d3d81` | Descriptor ID11 and registration call candidate. |
| `ev_69f2e068b810b9378c9695c2a7ca6aa33077e82eeda652029e432e7b4d2ff17b` | Registration call instruction confirmation; bounded registration fragment, not its full function. |
| `ev_615c4b50599c9c59142edcae9ab51e8018be93de1ae7fff23bd6d1320694edfe` | Existing scene's map-module lookup through virtual +0x130. |
| `ev_e48adbe063e62242a84308443e145499551c17b43bfbbbbdd8d1bb9f138d37ec` | Exact field-name strings, map lookup slot target and bounded ID11 candidate absence. |

All eight captures exited zero with verified owned-process cleanup. Raw and
normalized terminal frames are complete. Four captures flag aggregate
truncation because repeated rendered terminal frames exceeded their output
budget; no raw terminal bytes or normalized frames were omitted. Process
sampling and unrelated inaccessible `/proc` entries remain explicit limitations
in the private evidence. These are static observations, not runtime use of
the Mission records.

### Exact-reference follow-up: selected-index consumer remains unknown

A further bounded frozen-executable trace did not identify a typed caller of
Mission virtual `+0x30`. Exact disassembly references to `0x140c3c320`,
`0x140c3c3f0`, `0x141d62348`, and `0x1409493d0` returned only the known
descriptor getter and its registration call. The indexed reader has one
literal pointer reference, **`0x141d623c0`**, its Mission vtable slot. In
contrast, count accessor **`0x140314f20`** has **35** literal pointer
references in tables: its receiver cannot be called Mission merely because
that accessor is used. These queries do not cover dynamically constructed
or indirect references.

The complete map lookup `0x1409493d0..0x14094941a` checks EDX against map
`+0x8c`, then returns `map->array_at_0x80[EDX]`. Registration helper
`0x140c39b70..0x140c39d12` explicitly asserts
`moduleInfo.id < MODULE_COUNT_MAX` using descriptor Dword +0. This confirms
**11 is the Mission module ID**. It copies 0x38-byte descriptors into
flag-selected arrays; the enclosing initialization subsequently sorts one
array by dependencies. Neither operation selects an interest-point index or
uses its coordinates. This registration trace is not a per-map instantiation
or gameplay-use trace.

The additional case-insensitive ASCII queries for `InterestPoint` and
`interest point` returned only the two known schema names. They supplied no
named selector lead. No broad virtual-offset scan, runtime, or archive read
was performed. The required next evidence remains an actual module-11
receiver passed to virtual `+0x30`, including the index's origin and the
returned position/forward's consumer. No record is selected for map 379 by
this result, and neither player/NPC placement nor exclusive camera/debug use
is established.

Evidence: exact references
`ev_bbd16211ff46c1321c29d229eaf5ed85cd910e7a718621951d2f94621ca9c527`;
literal pointers
`ev_592b44af050b444ff434a07363ae5f32db7e62b8fb88284daab2d06f44c086d4`;
map lookup
`ev_e692aea9f59acb86014912b16c78c0ed05c59a6ae6d82c338c60cecaef16f6db`;
registration
`ev_5985bdaa49ab1873b8fcfe5df0595042cbcc347a1e264d77c9e6d8de1e39f74f`;
name seeds
`ev_cd82b48e5f1ba2be418b8b88574f4c0dff941f8c14e1da8ff1ff76f211f5d8e6`.
All nine reader captures exited zero; their raw and normalized terminal
records are complete. Full private bundle:
`~/re/gw2/notes/209026-norn-mission-typed-consumer-followup-20261105.json`,
SHA-256 `c611eb6cce695202b553a6fcd813d943358e74bb0c96f227287e8d6e882153c0`.
