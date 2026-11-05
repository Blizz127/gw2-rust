# Build 209026 creator model archive presence

2026-11-05. Bounded follow-up to the owned creator model probe: inspect archive
IDs 29213, 29193 and 30126, then their three directly encoded SKEL references.
No client was launched or attached, and no working-archive bytes were read.

## Archive separation and method

The selected frozen archive was `/home/blizz/gw2-client/build-209026/Gw2.dat`:
device 2065, inode 293163139, link count 1, size 89,165,623,112. The active working
path `/home/blizz/gw2-sandbox/client-work-209026/Gw2.dat` had device 2065,
inode 293170927 and link count 1. The files are distinct; the working path was
only statted to check this boundary.

Target-free REA process captures ran the existing `gw2-dat-cat` reader against
the frozen path. Reader SHA256:
`ecff83ff81bcff68d418698f98363688c665401a4b0a4307658b07ce86dc812b`.
This records our format-aware reader's decompression and bounded PF parsing,
not execution of the client's loader. Frozen inode, size and modification time
were unchanged after both checks; the entire 89 GB archive was not rehashed.

## Requested resources

All three reads returned exit 0 with empty stderr. Each decoded file is a PF
container, flags 5, header size 12, file-type tag `MODL`. Each complete chunk walk
ends exactly at the file boundary and contains MODL v70, SKEL v1, PRPS v0,
ANIM v25, GEOM v1 and ROOT v1.

| Archive ID | Decoded bytes | Decoded SHA256 |
| ---: | ---: | --- |
| 29213 | 54,996 | `7c8a6a1ed52e10f20be71e462f659941d7ebf89067bd218ed86a821794933979` |
| 29193 | 57,428 | `433af427f6344f78130e25ef0725ddb078a5ee8a4cfa5ae2d2961d6f675923ef` |
| 30126 | 55,780 | `b584b3798ed008a49f64a5b29b192b0efed419fb9fe4fddeaed61049dd9eee52` |

These requested IDs are therefore present and reader-decodable in the frozen
archive. The race/sex labels supplied by the model probe are not independently
derived from these packfile headers.

## Direct SKEL references

Each SKEL payload is 32 bytes, with fixup slots `[0,8,16]`. Its Qword at
payload+8 is relative 16, pointing to the encoded reference at payload+24.
The other two pointer slots are zero. The historical build 208039 reference
schema's SKEL v1 calls this field `fileReference`; that field name is a schema
interpretation, not a newly traced build 209026 consumer. The bytes and fixup
classification below are observed from the build 209026 resources.

Using the separately established build 209026 filename decoder
(`word1 * 0xff00 + word0 + 0xff00ff01`, wrapping at 32 bits) gives:

| Parent ID | SKEL chunk file offset | Encoded Words at payload+24 | Referenced archive ID | Decoded bytes |
| ---: | ---: | --- | ---: | ---: |
| 29213 | 260 | 18709,256,0 | 18454 | 56,884 |
| 29193 | 516 | 15467,256,0 | 15212 | 54,840 |
| 30126 | 596 | 29638,256,0 | 29383 | 55,320 |

All three direct reads also returned exit 0 and complete PF/MODL containers:
flags 1, MODL v65, SKEL v1, PRPS v0, ANIM v25, GEOM v1 and ROOT v1.
Their decoded SHA256 values are:

- 18454: `62ea4f7d645d5a4083caaa56871a026ef439f58d58fc2abae5b80d48897fa8f2`
- 15212: `671bbfdb9ba035d4a55c224cc3608e2adc85fc4d23de89087fb763d7c7c8f4cd`
- 29383: `e92376cb7de1b49fbc621c45ae6c1f82f2a53662d5681fe4b23c061f60a13354`

No recursive dependencies, animation references or model semantics were scanned.
Presence does not establish renderability, working-archive equality, async file
state transitions or delivery of the model completion callback. Use the separate
runtime evidence to distinguish readiness from a stalled completion path.

## Evidence

- `ev_f76e4468c036e412d9135c5650f412d5132aba6c1ca9b9977c25dc122e8b0fe0`:
  inode boundary, requested three reads, hashes and complete chunk framing.
- `ev_96c6541d3f8646b77f6e60cba8f7cabeb38faff8fccd7471e147bbe801032066`:
  SKEL slots, encoded references and three direct target reads.
- `ev_f46a1f58482baa387da174c13d380316d468d0f0c7cf0b181ffbd8531aa24cf3`:
  earlier native encoded-reference decoder, documented in
  [content file references](209026-content-file-references.md).

Both new captures exited0 without truncation and verified cleanup of their owned
process groups. REA could not inspect some unrelated host process metadata and
left those processes untouched. It did not observe filesystem syscalls or network
activity; archive immutability above is the selected metadata check, not a full
filesystem audit.

Complete new Evidence records remain private in
`/home/blizz/re/gw2/notes/209026-creator-model-archive-check-20261105.json`
(81,447 bytes, SHA256
`5a59270f823b147fd2e1b76bfcf3e00086e633c188280d3336290336b097893a`).
The six mode 0600 decoded artifacts are under
`/home/blizz/re/gw2/notes/creator-model-files-20261105/`.
