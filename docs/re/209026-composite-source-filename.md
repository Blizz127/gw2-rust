# Build 209026: composite source filename encoding

The empty-composite diagnostic can identify a source file when its filename
uses the client's compact archive encoding. Texture-data `+0x58` is a pointer
to **`m_fileHighRes`**, and `+0x50` is **`m_fileLowRes`**; neither field is
itself a numeric file ID. This is a frozen-source finding. No runtime file
ID, archive member, or cause of the observed 512-by-128 source is established.

## Exact diagnostic path

At `0x140a74ced`, the empty-only branch reads `[R14+0x58]`, falling back to
`[R14+0x50]` only when the first pointer is null. It then reads the selected
object's `+8` filename pointer. Helper `0x1409b8660` copies null-terminated
16-bit units to stack buffer `RSP+0xe0`, with capacity `0x104` units. At
`0x140a74d20`, `0x140dbcba0` decodes that buffer. A nonzero result is formatted
as `%#08x`; zero selects the ordinary string-conversion path. The resulting
diagnostic says `ProcessComposite(): Source rect entirely outside bounds of file %S`.

The field names come from setter `0x140a6e3f0`: its zero selector stores the
file object at data `+0x50`, guarded by `!m_fileLowRes`; the other branch
stores it at `+0x58`, guarded by `!m_fileHighRes`. The file/load-task
constructor `0x140a6c120` copies its filename through `0x140e4b9f0` and stores
the copied string pointer at object `+8`. The texture-data constructor
`0x140a6be30` initializes both file pointers to null.

## Compact filename format

Decoder `0x140dbcba0` has a `FileArchive.cpp` assertion and reads little-endian
16-bit units `w0`, `w1`, `w2`, and conditionally `w3`:

- `w0 >= 0x100` and `w1 >= 0x100` are required.
- `w2 == 0` terminates the name; the decoder does not inspect `w3` in this case.
- Otherwise, `w2 >= 0x100` and `w3 == 0` are required.
- Accepted input yields the unsigned 32-bit value
  `(w0 - 0x100) + (w1 - 0x100) * 0xff00 + 1`.
- An optional output byte receives `w2 & 0xff`, or zero when `w2` is zero.
  Its meaning is not established here; the composite diagnostic passes null
  for that output pointer. Rejected input returns zero.

The adjacent encoder `0x140dbcb10` constructs the inverse representation
from an ID minus one using quotient/remainder by `0xff00`, adds `0x100` to
both units, and appends the optional byte plus `0x100` when nonzero. Its
diagnostic explicitly names a bad file ID. These units are an encoded
filename, not ordinary text to interpret as a human asset name.

## Bounded observation available next

After the existing same-call source/operation guards, an observer can read
each nonnull high/low object's filename pointer and only six bytes, extending
to eight bytes when `w2` is nonzero. Record presence, read/format status,
recognized numeric ID and optional byte; do not emit ordinary pathname text
or call target functions. Null, unreadable, or unrecognized values must
remain unknown rather than becoming an invented ID.

Record both high and low references alongside the already established source
dimensions. The diagnostic's high-first preference alone does not prove
which file supplied the current pixels. Archive/base-file translation,
current runtime IDs, and the relationship between loaded source size and
authored rectangle extent remain unresolved.

## Evidence

Complete private source captures are retained in
`/home/blizz/re/gw2/notes/209026-composite-source-filename-encoding-20261105.json`,
SHA-256 `f90da10e92438005076967a95be7d6967bac97cbc45e9cadf0f932104ee46f01`.
The executable SHA-256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| Evidence | Scope |
| --- | --- |
| `ev_fb2919f8792fb4cc485f5c5014871fe804971171a756cbee5d1fde0e3ca52f16` | Exact empty diagnostic argument flow |
| `ev_c1003644958d4cebff299e90ed2e7060a5bb292a968b8910023eb689f66ef115` | Complete decoder and source diagnostic |
| `ev_87111125c39ce7bd8514ae9dec8764517f8970d9aa2eee803ed91e22dadd61f7` | 16-bit copy, file-task constructor, encoder and decoder |
| `ev_96bc5ad769483ee341282f9573da0a01910b178b3bb88e5a06edcb246f337290` | Named high/low setter and filename copy |
| `ev_49209488252166db32d0a7a5264aacc535996a6529e2fb11fb4e5f49c036488a` | Exact format and file-ID diagnostic literals |

No client was launched or attached, no archive was opened, and no probe or
target memory was changed for this trace.

## Mask-size failure: current-holder type and lifetime contract

The later size-failure investigation confirms the same access pattern for
**both** operands. In the failure-only branch, `0x140a74a32` reads the
primary R14's high holder at `+0x58`, then low `+0x50` if high is null.
At `0x140a74ac9` it does the same for the current mask RBP. Each chosen
holder's `+8` filename is copied immediately before decoding. These are
current native operand references, not a scene-owner or asset-ID lookup.

For a conservative optional observer, after the
[same-call operation and texture guards](209026-composite-mask-size-contract.md),
read each current high/low holder independently and require:

1. Holder vtable `0x141bfc808` and interface table at `+0x18` equal
   `0x141bfc810`, both adjusted for image slide.
2. Holder `+0x28` points back to the exact current texture-data operand.
3. Holder key flags `+0x10` bit 0 agree with its high/low slot.
4. Holder `+8` is a non-null recognized compact filename, decoded using at
   most six bytes, or eight when the third word requires it.
5. The decoded numeric ID equals the holder's original-ID Dword `+0x30`.

Constructor `0x140a6c120` stores the copied filename at `0x140a6c15a`,
installs the concrete tables at `0x140a6c165/0x140a6c16f`, stores the
texture back-reference at `0x140a6c19d`, and decodes/stores the original ID
at `0x140a6c2c3/0x140a6c2d2`. The named high/low setter is unchanged.

The table guard alone is not a lifetime proof. Deleting destructor
`0x140a6d4e0` initially retains those concrete tables while running cleanup,
then changes to the base table and releases the filename at `0x140a6d5d8`;
it may free the task allocation at `0x140a6d603` or `0x140a6d61e`. Thus the
proposed read follows only the current field that the native diagnostic
itself is about to consume, under all-stop inspection, and discards every
pointer before resuming. It must never reuse an earlier holder address.
Unsupported variants, absent holders, inconsistent relations, malformed
encoding or failed reads remain local unknowns; they do not invalidate an
otherwise qualified scalar operand snapshot.

The resulting number is the **original filename ID**, not the actually
opened remapped ID. In particular, a high holder can retain original 142103
while its load task opens 142104 under the
[type-0 remap contract](209026-texture-remapped-file-selection.md). Both
holder slots should remain separately labelled. Neither high-first native
diagnostic preference nor a recognized original proves which backing file
produced the current pixels.

Fresh frozen-source evidence:
`ev_0ca3060b9f57e9b8cf77e41c8de493335876e85597afb166569932419a551b80`
(complete failing consumer) and
`ev_990de8ab60f761b2015836948f321e13c29af3bb786042071020955629ec1371`
(setter, constructor, destructor and transfer entry). No runtime attribution
is added by these source captures.
