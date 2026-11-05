# Build 209026: texture keys and manifest-remapped file selection

A texture's high-resolution cache key does **not** prove that the original
component filename is the file opened. For a fresh CPU composite source,
the client can open a second numeric file ID supplied by an MFST type-0
property. The payload is an ID, not a boolean quality hint, and the traced
lookup contains no adjacent-ID arithmetic.

This source-only REA slice used frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No archive, runtime, container or frozen probe was changed or accessed.

## Property contract and population

`140d90480(id, 1)` calls `140d8fc20`. That helper supports variant 1 here,
queries **`140d97290(id, type=0)`**, requires a four-byte payload, and returns
its little-endian Dword. Missing or malformed metadata yields zero; a zero
payload also behaves as no remap to the texture policy.

`140d97290` retrieves metadata under key `(uint64(id) << 32) | type` from
the table at `1427fb180`, under the lock at `1427fb150`. Its value indexes
12-byte records at `1427fb1a0`; record length and blob offset locate bytes
under `1427fb1c0`. The mapping comes from data, not `id + 1`.

The complete MultiManifest consumer `140d91cf0` loads **MFST v6**. Its
property-table array at `manifest+0x34`, count `+0x30`, contains
`{baseId, propertyIndex}` rows. The indexed 13-byte property at
`manifest+0x28` supplies a type byte, byte count and data pointer. At
`140d91e62` these are passed to `140d97540(baseId, type, length, data)`, then
`140d97730` inserts the bytes into the exact table above. Manifest record
registration through `140d973e0` is a separate earlier loop.

## Fresh CPU acquisition policy

The component's half-size argument chooses public request flags `0x1032`
or `0x10b2`. The acquisition routine `140a72530` adds CPU-only bit `0x4`
for a blit source, then bit `0x40000` when the converted flags contain
`0x14`. Thus the two inputs become `0x41036` and `0x410b6`.

`140a70670` produces **separate key flags and task flags**. CPU-only bit
`0x10` forces its quality variable to zero. For these exact inputs the
complete branches produce:

| Component half-size | Converted flags | Nonzero remap available | Key flags | Task flags | Numeric file selected later |
| --- | --- | --- | --- | --- | --- |
| 0 | `0x41036` | no | `0x19` | `0x420` | ArtCatalog-resolved original ID |
| 0 | `0x41036` | yes | `0x19` | `0x440` | Remap of the ArtCatalog-resolved ID |
| 1 | `0x410b6` | no | `0x1a` | `0x220` | ArtCatalog-resolved original ID |
| 1 | `0x410b6` | yes | `0x1a` | `0x420` | ArtCatalog-resolved original ID |

The initial availability lookup uses the **original** compact filename ID.
The later remap lookup uses the **ArtCatalog-resolved** ID; an active catalog
override can therefore matter between these steps. The table describes
the selected control flow, not a guarantee that every later lookup succeeds.

The cache lookup `140a70530`, called at `140a726ca`, uses original filename
and key flags. A cache hit returns existing texture data and does not
reconstruct its load history. On a miss, constructor `140a6c120` stores:

| File-task field | Meaning in this path |
| --- | --- |
| `+8` | Copy of original filename |
| `+0x10` | Key flags from R8D |
| `+0x28` | Texture data |
| `+0x30` | Original filename's decoded numeric ID |
| `+0x7c` | Task flags from R9D |

The assertion in `140a70450` names key bit 0 `TEXTURE_KEY_HIGH_RES`.
Acquisition passes that bit to `140a6e3f0`: zero installs the task in
texture `+0x50` (`m_fileLowRes`), nonzero in `+0x58` (`m_fileHighRes`).
Consequently a high holder can retain the **original filename**, while its
task opens a remapped file. Neither holder filename nor task `+0x30` alone
records the actual selected open ID.

## Actual metadata and file-open path

Both state-1 handler `140a73d20` and state-2 handler `140a73b50` test
**task flags bit `0x20`**:

- Set: resolve the original ID through ArtCatalog, then encode that result
  as the compact filename. The nonnumeric filename case uses catalog
  virtual `+0x18`.
- Clear: resolve the original numeric ID through catalog virtual `+0x10`,
  apply `140d90480(resolvedId, 1)`, then encode the returned remapped ID.

The catalog is installed by `1402373d4..1402373dc` from factory
`140fd9e60`, whose concrete table `14210dc60` maps virtual `+0x10` to
`140fda8f0`. That method returns the original ID when catalog `+0xc` is
zero; otherwise it queries a catalog override and falls back to the
original ID if the query returns zero. Its constructor starts `+0xc` at
zero, but this is not a measurement of the active runtime catalog.

State 1 passes the selected compact filename to `140d85ce0` for metadata.
State 2 passes it to **`140d85e60` at `140a73c52`** for loading. The latter
uses the Asset service's registered load function: initialization
`140d90240` registers **`140d90280`**, which forwards the filename to
`140d908a0`. That routine decodes the selected compact filename and writes
the numeric ID to the asset-load request's `+0x28` at `140d908d4`.
This establishes an actual file-selection effect, beyond mip-drop policy.

The [loaded-dimension writeback](209026-texture-loaded-dimensions-writeback.md)
then replaces provisional texture dimensions with decoded-file dimensions.
The runner's separate full-texture branch patch does not upscale an image.

## Run 137 interpretation and smallest remaining discriminator

The root's paired run-137 observation identifies current owner 6, scene 3,
slot 9, descriptor kind 9, half-size 0, primary **140123** and first mask
**140121**. The source rectangles are `[0,0,384,256]`,
`[128,256,384,512]`, and `[384,256,896,512]`. Root's frozen-header evidence
`ev_a02d468da993c6fa0d502f3d514d6a3698c84c9a09236e5a9e71230fa58dc109`
reports both original files as 512-by-256. These facts cannot yet establish
that those original headers describe the files selected by the load task.
The pending 1024-square acquisition fields are also not final dimensions.

The repository's manifest generator emits properties only when explicitly
provided. Root's subsequent audit of the synthetic MFST files served under
run 137's `--latest` root **2130706600** found:

| MFST ID | Bytes | Records | Property count / property-table count | SHA-256 |
| --- | --- | --- | --- | --- |
| 2130706601 | 13,109,648 | 819,347 | 0 / 0 | `12309ed55d0651c35ce2adc8a1347e592664aa9f30e40e404979afc43e3a0289` |
| 2130706602, 2130706603, 2130706604 | 112 each | 1 each | 0 / 0 each | `195226488309fb3558de1dce3e79b797e0b6d5246be5899d24f2c09d64fd23d5` each |

These four files supply no type-0 remaps. The ordinary manifest record
`140123 -> 315194`, size 57,272, and record `140121 -> 140121`, size 252,
are base-ID/file-ID records, **not** high-resolution property mappings.
Root's evidence is
`ev_d2f5f9ee017dd660535ed366ab5a9ee9351c830372a3b4a21cd09e7ca6155454`,
retained in
`/home/blizz/re/gw2/notes/209026-synthetic-texture-manifest-audit-20261105.json`.

On the fresh half-size-0 path, absent remap metadata selects the original
ID despite the high-resolution key. The served-file audit does **not**
establish the client's active metadata table, cache history,
resolved/remapped IDs, successful loaded dimensions or the cause of the
partial character image.

The smallest remaining check is the actual type-0 payload for these exact
base IDs in the served/loaded manifests. If runtime selection still needs
confirmation, decode the compact filename **pointed to by RCX** at the
state-2 call `140a73c52`, paired with its current task/owned source, and
the matched loaded dimensions at `140a70450`. That avoids mistaking the
original task filename for the opened file. No adjacent-ID guess,
coordinate rescale, readiness override or probe change follows here.

## Proposed read-only load/transfer diagnostic contract

This is a source-derived contract, not an implemented or executed probe.
The minimum lifetime-safe design has two observations and one destruction
site. It can admit only the eleven original slot-9 IDs from runs 136/137:
`140521, 140515, 140517, 140519, 140523` and
`140123, 140121, 140115, 140117, 140119, 140125`. Such admission establishes
an allowlisted texture task, **not** current-scene ownership. A stronger
owner claim requires a separately verified current component relation.

| Site | Native relations and observations | Exact 16-byte guard |
| --- | --- | --- |
| `140a73c52`, immediately before Asset load | RDI = R8 = task; RBX = task+0x20; R9 = task+0x18; RDX = `140a70210`; RCX points to selected compact filename | `e809223100488b6c24784c393b0f8590` |
| `140a70450`, transfer entry before prologue | RCX = task; EDX = format; R8 packs width in low 32 bits and height in high 32; R9D = levels | `48895c2410574883ec608b4178488bd9` |
| `140a6d4e0`, deleting-destructor entry | RCX = task; retire any watched generation before cleanup or free | `48895c2408574883ec20488d0517f318` |

Apply the executable slide consistently to source addresses and concrete
table pointers. At load/transfer, require task vtable **`141bfc808`** and
task `+0x18` interface table **`141bfc810`** before interpreting fields.
Constructor `140a6c120` installs both. Record original ID `+0x30`, key
flags `+0x10`, task flags `+0x7c`, and state `+0x34`. Task `+0x28` supplies
the current texture-data relation; only interpret its fields if its table
is the known GrTex2d **`141bfc700`**. Unsupported types or read failures
remain local unknowns; never print raw addresses or arbitrary strings.

For these nonzero allowlisted original IDs, the load site's numeric path
calls encoder `140dbcb10` at `140a73c19` into **current RSP+0x40**, then
passes that address as RCX. Require `RCX == RSP+0x40`. Read six bytes first
as three little-endian 16-bit words. Require `w0,w1 >= 0x100`; if `w2 == 0`,
accept without another read. Otherwise require `w2 >= 0x100`, then read
only the next two bytes and require `w3 == 0`. The decoded numeric ID is
`(w0-0x100) + (w1-0x100)*0xff00 + 1`; report the optional encoding byte
separately if present. Malformed or unreadable encodings yield no selected
ID. **RCX itself is a pointer, never the scalar file ID.**

At transfer entry, the established state-6 caller at `140a6de7a` has set
task `+0x34` to 7. Its return address at entry `[RSP]` is **`140a6de7f`**;
EDX/R8/R9D were loaded from task `+0x58/+0x68/+0x64`. Require that caller
and compare those current fields if restricting observation to this
known completion path. These values are **incoming decoded transfer
metadata**. Entry does not prove successful transfer, readiness, texel
content or later rendering; `140a70450` can reject the transfer.

Allocate a diagnostic generation at the first accepted load event and
arm its destructor identity filter **before resuming**. Match a transfer
only to a still-watched task with unchanged original ID, key/task flags
and texture-data relation, revalidated at the native site. Do not require
the same thread across the asynchronous load/transfer stages. Duplicate
load attempts in a live generation must be labeled as attempts; pointer
equality without uninterrupted destruction coverage is insufficient.
An unmatched transfer can be labeled unpaired but cannot inherit an
earlier selected ID.

Table `141bfc808` points directly to `140a6d4e0`. This routine performs
cleanup at `140a6d504`, changes to the base table, releases the filename,
and can free its 0xc0-byte allocation at `140a6d603` or `140a6d61e`.
Retire at **entry**, including nonfreeing destruction, before any cleanup
can invalidate fields. The destructor filter needs only RCX equality
against active diagnostic generations; it need not dereference a cached
address. Never follow cached task/texture pointers from a timer or later
scene snapshot.

Suggested observer bounds are two admitted generations per original ID,
22 total, one accepted load and one transfer record per generation plus
its retirement, and a finite interval beginning with the first admitted
task. If a cap, guard failure or instrumentation gap ends coverage,
discard active correlations and label the reason. Native public
breakpoint command lists can filter irrelevant calls without global
per-hit Python logging. Cache hits or loads preceding arming can produce
no matched pair; that is a coverage limit, not evidence of no load.

## Evidence

The staged run-138 implementation adds the post-`140a75cb0` call site
`140a704aa`, where RBX is the current task and transfer-entry RSP equals
current RSP plus `0x68`. It records incoming dimensions separately from
current texture fields after the call. Raw EAX and the `transferRejected`
out parameter at current RSP plus `0x70` remain separate: the CPU path can
return one without establishing readiness. Its four sites use current
task validation and destructor retirement, with no cached-pointer reads
outside those native sites.

Private helper `creator-texture-loads-probe-20261105.py`, SHA-256
`55b4f1898ed0355eda5480389a4562489a312cc901d3a57d3c99b5804cc2490e`,
admits the seventeen explicitly observed IDs documented in
[the cached metadata note](209026-cached-texture-remap-metadata.md), at most
two generations each, 34 total, 160 records and 210 seconds from the first
matched load. Nineteen focused host tests and all four frozen instruction
guards passed (`ev_a739496ee0e51b9f2748dc17caea82bdfc88735077fd00d828b31ba1cb29b21b`,
`ev_2209688a9411a3af6c563a180ae6f1779055361dfe4c6d9c3768755002bed250`).
Those checks establish preparation, not a client runtime result.

Private complete bundle:
`/home/blizz/re/gw2/notes/209026-texture-remapped-file-selection-20261105.json`.
The subsequent diagnostic-contract source captures are retained separately
in `/home/blizz/re/gw2/notes/209026-texture-load-transfer-contract-20261105.json`.

- `ev_8b467020936fb680e0a43748e15afa5125a7c101f68a5777047e2b5cd2d53199`:
  exact four-byte remap helper and complete state-1 selection.
- `ev_a0a24ea2e1322d0e986c0483ed76b631e8eb86bb5a3e554f869d60ae51bca11c`:
  metadata getter, catalog accessor and complete acquisition/cache path.
- `ev_ed1dbc4b05bf648913f7b46c37aad7602fb3907c03a3b2de06ceca2862846478`:
  complete key/task policy helper.
- `ev_d0739741a4869963e1a7924ea22d75c5fa8b03cf725080968f0b3a917e768ce2`:
  complete state-2 selected-file load path.
- `ev_cfa6b9436f232057b69f3c490ae24ec4e11224205d90ac8a8fa75ddbf5c16fb4`:
  complete MFST v6 property consumer and Asset registration.
- `ev_ed55b65a3cf7ffe7357b9c5ee65ff312ab1e4546b8941098d05c279c48725d03`:
  catalog construction, concrete load adapter and property insertion bridge.
- `ev_4289adc596891fd13943c772b3c51dbcdb52c6c1d74349f625d9c5e83f310354`:
  exact catalog vtable entries read as data.
- `ev_8e0a6a444ac8e53aac2b4b60b32283dc9835bd24b083ae5694e22ffeca4b2d31`:
  complete numeric catalog resolver, selected load-ID store, metadata insert.
- `ev_e355e1a9f983dcf9ae8227f132feca9cdfa7fec952504060986677295fb770e9`:
  complete task constructor and transfer routine.
- `ev_6a622beab23dd3871c2876c85b09411481c292ca4de06ad6fdaa9f9a38e90239`:
  exact open-call registers, stack filename, and state-6 transfer caller.
- `ev_ae299267e725fb5abd3d2d99aa0ec54bb12f802b7a2aa648fd0d3071c622fba8`:
  complete deleting destructor and its cleanup routine.
- `ev_6e850c8e7da2fb41f62b27861bae90bcbcdb92737a375918baba067a9567f46e`:
  exact diagnostic-site guards and task table entries read as data.

Accepted source captures exited zero with nontruncated raw output and
verified owned reader-process cleanup. One bounded two-target xref script
stopped after its first result due to a local variable collision; the
second query was rerun independently. No negative xref claim depends on
that failure. Unrelated unreadable process environments remain separate
capture limitations.
