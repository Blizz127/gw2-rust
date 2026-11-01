# Build 209026: content file-reference fixups

Static REA follow-up to [fallback model records](209026-character-model-records.md).
No client or renderer was run.

## Shared table selection

Fixup helper `0x1412e7fd0` walks the current container's Dword slot list
at header +0x4c/+0x50. Each slot identifies a Qword in content +0x80.
The raw Qword indexes the file-reference pointer array at **header +0x20
of the container selected by argument R8D**. The helper replaces the slot
with that array's pointer; it does not select the current container's table
implicitly.

The base loader passes **R8D = 0** at `0x1412e729d` before calling the
helper at `0x1412e72a6`, for each loaded base container. Therefore all 32
frozen base containers use the first container's file table. A second
caller at `0x1412e7876` uses R14D instead; that separate loading path's
origin is not established here. A reader should accept an explicitly
selected table, rather than assume that every content set uses zero.

On disk, the selected table is the relative pointer array described at
Main +0x1c. The first frozen container, file **1282830**, contains 91,014
entries. The fallback-record container, file 1282855, has no local entries.
This explains the earlier checked local-array decode failure.

## Encoded archive reference

Decoder `0x140dbcba0` accepts two leading little-endian Words at least
0x100. The third Word either terminates the reference with zero, or is
at least 0x100 and followed by a zero fourth Word. In the latter case the
optional output byte receives the low byte of the third Word; zero is
returned there for the shorter form. This byte's broader purpose is not
named here.

The archive ID is computed with 32-bit wrapping arithmetic:
`word1 * 0xff00 + word0 + 0xff00ff01`.
Other forms return zero from this decoder; its callers may use another
loader path. A strict content reader can reject unsupported filename
forms without treating them as encoded archive IDs. Table index zero is
a valid classified reference, not null. Pointer bounds, arithmetic bounds
and duplicate/conflicting fixup classifications still require checks in
the offline reader.

## Frozen fallback model references

A hash-checked read followed the observed table selection and validated
slot membership and index bounds for all nine type-146 records:

| Content offset in file 1282855 | Shared table index | Archive model file ID |
| --- | --- | --- |
| 0x3cd8b8 | 4723 | 533056 |
| 0x3cd918 | 4894 | 533060 |
| 0x3cd978 | 5408 | 533049 |
| 0x3cd9d8 | 4892 | 533061 |
| 0x3cda38 | 4721 | 533057 |
| 0x3cda98 | 5407 | **527140** |
| 0x3cdaf8 | 4893 | 533059 |
| 0x3cdb58 | 5409 | 533050 |
| 0x3cdbb8 | 4722 | 533055 |

Human/Guardian's matching record therefore references file **527140**.
A [subsequent archive check](209026-character-model-records.md#archive-presence-and-fallback-scale)
confirms that all nine are present as MODL containers with bounded chunk
framing. Model validity and rendering success remain unverified.

Source hashes: table file 1282830
`e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f`;
record file 1282855
`9f9226f71e3871066237d560e95b62ef33c847818a7bd78cea586507f492cfa6`.

## Rust reader and focused verification

`ContentIndex::resolve_file_reference(slot, table)` now resolves the
encoded archive-ID form through a caller-selected content file table.
It returns `ContentFileReference { file_id, suffix }`, retaining the
client decoder's optional byte. It does not open files. The existing
`reference` method still rejects file slots as unsupported content
record/string references.

Three focused tests pass: shared-table resolution with classified index
zero and both encoded forms; missing/conflicting fixups, bad slots and
indexes; malformed pointers, truncated encodings and unsupported names.
The `gw2-dat` library build passes. An offline REA check of the two frozen
archive containers matches all nine independently decoded model IDs and
rejects the incorrect local table in all nine cases. No runtime model
load was attempted.

Smoke evidence: `ev_7f867f95d962c9d807e1f0c896300c957213d6a81027003847918cc703453d62`.
Private validation bundle:
`~/re/gw2/notes/209026-content-file-references-rust-1.json`.

## Evidence

Private bundle: `~/re/gw2/notes/209026-content-file-references-2.json`.
Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| REA evidence | Observation |
| --- | --- |
| `ev_77e4ed9355c005b67424e6027c9411aa844c6fddb5fd7ac33f05204d8e0a3aa2` | Complete file fixup helper and selected-container table access. |
| `ev_b47297f6f06290bafce9ba10a400675d8ac8e10a5cc11dc1840e25296e72636d` | Two relative-call candidates. |
| `ev_20d6613eb12a065e5a8b6670480e23c63906acc1de2573a8acfb4bd64f42be1a` | Base caller passes zero; second caller passes R14D. |
| `ev_3653aeebf2b5697e78da10afad77ad397e6eb8daf4776b85221ec4a3f9c28c51` | Hash-checked nine-record resolution through the first container's table. |
| `ev_f46a1f58482baa387da174c13d380316d468d0f0c7cf0b181ffbd8531aa24cf3` | Complete encoded-reference decoder and optional output byte. |
