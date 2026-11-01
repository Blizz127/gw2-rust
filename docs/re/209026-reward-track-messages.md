# Build 209026: reward-track manager creation

Static REA trace, 2026-11-01, against the frozen executable identified in
[mission transfer](209026-mission-transfer.md). No client replay or User
Agreement interaction. This follows the conditional dependency in
[specialization updates](209026-specialization-messages.md).

The [numeric-index follow-up](209026-content-numeric-ids.md) confirms
that this message path uses data keys at record +0x28 through content
virtual +0x70, distinct from specialization wire UIDs at +0x14.

## Server 565

Protocol 0/group 0 pair `0x142170670` associates descriptor `0x1425d4370`
with handler `0x141259d00`. The wire fields after the little-endian ID are:

| Field | Encoding | Inspected use |
| --- | --- | --- |
| Player index | Dword | Player-table lookup, required nonnull. |
| Four track IDs | Fixed array of 4 Dwords | Type-353 lookup for four reward-track slots. |
| Four slot bytes | Fixed array of 4 Bytes | Copied to four Dword fields at manager +0x98..+0xa4. Gameplay meaning unresolved. |

Both arrays have no length prefix. Nested definitions `0x1425ace00` and
`0x1425acd60` contain Dword and Byte respectively. Decoded pointers at
+6 and +0xe are unpacked-memory offsets, not wire offsets.

The handler calls `0x1411baad0`, which asserts **!m_rewardTrackMgr**
(literal `0x14215ef10`), allocates 0xf8 bytes, constructs through
`0x1412477a0`, and stores the pointer at player +0x9a60. It then asserts
that pointer exists and initializes it through `0x141248460`.
Thus 565 is a creation operation; repeating it is not an observed reset.

The constructor installs vtable `0x1421711e0`, stores the player at +0x30,
clears four active pointers +0x38..+0x50, four 16-byte slot records
+0x58..+0x97, and byte flags +0xf0. It subscribes embedded listeners to
player +0x9760 through player virtual +0x18/object virtual +0x30 and
player +0x90 through player virtual +0x420 (`0x1411bbb40`). Both insert
list entries without immediate listener callbacks; the latter asserts
against duplicates. The manager's player-listener virtual +0xc0 is the
no-op `0x140163990`, so the control-assignment event traced separately
does not directly initialize reward progress through that listener slot.

For each of four slots, `0x141248460`:

1. Resolves the input ID with content-service virtual +0x70, type **353**
   (`0x161`). The CnContext vtable at `0x14220e160` maps that slot to
   `0x1412e1be0`, then `0x1412e68e0`. This lookup uses a combined
   type/low-22-bit ID hash in backend +0xb8, distinct from the +0x58
   lookup used in specialization assignment. No equivalence of those
   two numeric-ID indexes is asserted here.
2. If resolution returns null, logs `NULL REWARD TRACK: %s` through
   `0x1409bb390` with the player's GUID string, then continues. This
   static trace does not establish zero or any missing ID as a valid
   gameplay choice.
3. Clears that slot's bit in manager +0xf0, stores the pointer at
   +0x38 + slot*8, and emits a manager +0xc0 event through `0x14026e8bc`.
4. If the slot record's pointer is null (all four are initially null),
   calls virtual +0xe0, `0x141249110`. It copies the active pointer to
   +0x58 + slot*16, clears +0x60 + slot*16, and stores the result of
   virtual +0x20 at +0x64 + slot*16.
5. Stores the corresponding raw byte as a Dword at +0x98 + slot*4 and
   emits another event through `0x14025552c`.

Virtual +0x20 is `0x141247ef0`. It obtains the embedded player progress
object (+0x9760), returns zero for a null track, otherwise dereferences
track +0x40 and reads that referenced record's numeric key at +0x28.
Progress virtual +0x28 (`0x141231a20`) looks up that key in its sorted
16-byte entries. Missing progress returns zero; an existing progress
object supplies virtual +0x30. The nonnull track's +0x40 reference is
not null-guarded. Content validity therefore remains a prerequisite. The
[base-content follow-up](209026-reward-track-content.md) resolves all 529
track progress references to type 53 and maps Guardian thresholds.

The direct creation/initialization bodies add no owned-character or
coreStats dereference. They do require an existing player and global
content services. Runtime listener effects and useful slot IDs/bytes
remain unresolved.

## Adjacent messages and later initialization

These descriptors/handlers are mapped, but their setter semantics have
not yet been fully traced. All begin with a player-index Dword and require
the existing manager:

| ID | Descriptor | Handler | Remaining wire fields | Callee |
| --- | --- | --- | --- | --- |
| 566 | `0x1425d4840` | `0x141259e40` | Dword, Byte | `0x1412493a0` |
| 567 | `0x1425d4a30` | `0x141259f70` | Dword, Byte, Dword | `0x141249500` |
| 568 | `0x1425d44c0` | `0x141259da0` | Dword, Byte | `0x1412491f0` |
| 569 | `0x1425d4590` | `0x141259ed0` | Byte, Byte | `0x1412494c0` |

Server **431**, pair `0x14216fe30`, descriptor `0x1425c81a0`, has no
payload. Handler `0x141255010` calls context function `0x1411b7700`.
That function emits a context event, dereferences the controlled
character's skill bar (+0x520), calls virtual +0x310 on character +0xb00,
requires the context +0xa0 player's reward-track manager, and calls
`0x141248d10` to refresh all four slot records through virtual +0xe0.
It then calls a helper on embedded player +0x18c8, sets context +0x208
bit 0, and tail-calls `0x1411b9680` with zero. The +0xb00 interface,
other callbacks, packet's full meaning and placement in the startup
sequence remain open. This establishes a further consumer requiring
565's manager, not a complete instruction to send 431.

The [wardrobe follow-up](209026-wardrobe-startup.md) identifies +0xb00
as m_wardrobeMgr, maps lazy allocation through 247/248, and resolves
431's direct wardrobe/coreStats and skill-bar owner checks. Its remaining
listener and progress callbacks still do not establish full readiness.

## Evidence

The typed `RewardTrackCreationFrame209026` in
`crates/gw2-proto/src/reward_track.rs` preserves 565's player index,
four raw IDs and four slot bytes. It decodes transactionally and does
not validate game state. Focused `cargo test -p gw2-proto --lib reward_track`
passed **3/3**, and `cargo build -p gw2-proto --lib` passed. Hand-authored
wire fixtures cover distinct variable-length IDs, zero/u32::MAX, raw
bytes through 255, fixed-array ordering without count prefixes, append
and concatenation boundaries, every truncated prefix, wrong IDs and
invalid varints at all five Dword positions. These were offline codec
checks; no frame was sent to the client.

Private cumulative bundle:
`~/re/gw2/notes/209026-reward-track-creation-1.json`.
Search hits and rel32 candidates were followed by bounded instruction
inspection; candidate bytes alone were not treated as call sites.

| REA evidence | Observation |
| --- | --- |
| `ev_b1167842237ce1aa03ddfb86cffd3545130a667048c0f60271989e635192923c` | Specialization's manager and progress consumer. |
| `ev_43589ea92b6b9092c4751126784780e33e812e3ab492c2025bafbc88b2cc86a4` | Candidate player +0x9a60 displacement references. |
| `ev_253cb3357a70e8433a189c6d5df43d7105bd1acef2277856829b7bb2f6dd8875` | Manager allocation and creation/update handlers. |
| `ev_66e4801e6c20748f4ad692c5c43eb5bf002e5e6aeae426860753523c4a211021` | Registration pairs, constructor and named creation assertion. |
| `ev_81834f16d02c936d56dc834967e5b150b604b39c42bfe412f354546a643f46d1` | 565–569 descriptors, constructor tail and full initializer. |
| `ev_d04aa0ae67bf4e64264b44866ac70c0e70eb7010d58fae14d56fac1d85fd6baa` | Nested Byte descriptor, manager slots, null-track log, later initialization. |
| `ev_949d3de178ba71ec57c5a61ff0bbeab29ac2b3a8f7773735903f3faca1d8e832` | Slot refresh, player subscription and progress getter. |
| `ev_f153441cc13363b1e0758d95a3b49d287bd187146187b6ba4bdfd87831fe135f` | Later handler, no-op listener slot and content lookup wrapper. |
| `ev_cbbd1a78ab3be27772483609222e085c4831e41fb429fd2134bb7e2f3257e561` | 431 registration, content vtable, subscription tail and progress index lookup. |
| `ev_52e41e6b37be1d64bdf6ce06bf712043b7d3867e1d71c2df718839f1cfdb4312` | Empty 431 descriptor, backend lookup and four-slot refresh. |
