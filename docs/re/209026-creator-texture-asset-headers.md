# Build 209026: exact Norn creator texture asset headers

Run 136 bound the final appearance owner 7, scene 3, component slot 9 to
descriptor kind 9 and rectangle set 3. Its primary filename ID was 140521;
mask channels 2/3/4 were 140515/140517/140519 and channel 6 was 140523.
Channels 1/5 were null. All five wrappers were still pending at dispatch,
with requested dimensions 1024-square, one level and no backing. Those
runtime fields are not final loaded image dimensions.

The exact current-dispatch tuples are retained in
`ev_210eb8882b3a9d08f0756147cd6a5b50a4037833d33f31adcb33eed85eeb0a46`
and full record `ev_596316b93628f4fd29ad7b3a6ca7d6e2e557f28a9667b2fdbd26fe9124666c51`.
The probe sampled first dispatch per component slot/current owner; it did
not retain wrappers for later dereference or claim allocation continuity.

## Immutable archive extraction

The corresponding five assets and one slot-0 control were extracted from
the frozen archive, not the working runtime archive. Metadata verified the
two archives are distinct, non-symlink files with one link each:

- Frozen: device 2065, inode 293163139, size 89,165,623,112 bytes.
- Working: device 2065, inode 293170927, same size.

The extraction used the existing `gw2-dat-cat` reader and exactly six numeric
IDs. It created a new mode-0700 directory with private files and verified the
frozen archive's identity, size and modification time stayed unchanged.
Only header metadata and hashes were printed. No pixel image or runtime
archive was decoded or changed by this extraction.

The [native header contract](209026-native-texture-header.md) validates the
signature/FourCC and interprets the little-endian width and height at offsets
8 and 10. All six assets have ordinary `ATEX` signature `0x58455441`.

| Component/channel | File ID | FourCC / native format | Header dimensions | Decoded asset bytes |
| --- | --- | --- | --- | --- |
| Slot 9 primary, channel 0 | 140521 | DXT5 / 26 | 512 × 512 | 47,312 |
| Slot 9 mask, channel 2 | 140515 | DXTA / 27 | 512 × 512 | 1,836 |
| Slot 9 mask, channel 3 | 140517 | DXTA / 27 | 512 × 512 | 2,360 |
| Slot 9 mask, channel 4 | 140519 | DXTA / 27 | 512 × 512 | 2,908 |
| Slot 9 channel 6 | 140523 | 3DCX / 30 | 512 × 512 | 262,164 |
| Slot 0 primary control | 142103 | DXT5 / 26 | 256 × 128 | 32,240 |

These dimensions precede the load-task reduction stage. They do not prove
the working runtime archive's entry bytes match the frozen asset, the final
loaded dimensions, or the coordinates/extent of any selected composition
rectangle. Requested atlas dimensions can differ from a component image's
dimensions without being an error. The next relationship to observe is the
actual selected rectangle and its matched ready source in the demonstrated
descriptor-kind-9 path.

The earlier run-132 **512-by-128** source belonged to an initial scene-0
owner and had no recovered file ID. It is **not** established as any of these
Norn appearance assets. Its particular rectangle, route or dimensions must
not be carried over to slot 9 here.

## Evidence

Metadata: `ev_84af87733345d979c1966464d2979c7049538793fd71e2741b73da413d540ff9`.
Six-asset extraction at **2026-11-05 21:58:51 UTC**:
`ev_9581d7c770e57533c5f05ee1f0e31c930d78c8c0bdb4e1acc19bd8ca78f270b8`.
The capture exited zero with complete output and owned-process cleanup.
Private files and full hashes:
`/home/blizz/re/gw2/notes/run136-texture-assets-20261105/receipt.json`.

The primary asset SHA-256 is
`6fd60473c49dd33c3a93647f45890777d4d9c5738bf62424134b76d7843f477a`;
the three mask hashes are
`a3a6330f2aeb485f1311cb49ab4748ab63cf0c49bac1654a7716cdc80cedfe19`,
`81805643e0370b96529f881db2557cbfbeeeefb005e6851d4c67e7d0c0026c32`, and
`814810907cc32eb3c78716308e38d44dcfd90742169fa9a0a239408feacc7c2c`.
This is source-asset evidence, not a preview fix or a newly published build.
