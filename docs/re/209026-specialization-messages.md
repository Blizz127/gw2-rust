# Build 209026: specialization and major-trait messages

Static REA analysis, 2026-11-01, of the frozen executable identified in
[mission transfer](209026-mission-transfer.md). No client replay occurred.
This extends the [Guardian specialization-state trace](209026-character-profession.md#specialization-dependent-byte-check)
and the skill eligibility path through embedded player +0x9cc0.

## Wire layouts

All are protocol 0, group 0 server messages. Every layout starts with a
**Dword player index**, which the handler resolves through context virtual
+0x118. Dwords use the established varint encoding. Fixed arrays have no
wire count prefix; nested descriptor `0x1425ace00` contains one Dword.

| ID | Descriptor | Handler | Fields after player index |
| --- | --- | --- | --- |
| 640 | `0x1425db4d0` | `0x14125c8b0` | Dword specialization UID, Byte specialization slot |
| 641 | `0x1425db670` | `0x14125c920` | Dword trait UID, Byte specialization slot, Byte major-trait slot |
| 642 | `0x1425db830` | `0x14125c990` | ArrayFixed(3) Dword specialization UIDs, ArrayFixed(9) Dword trait UIDs, Byte notification value |
| 643 | `0x1425db9c0` | `0x14125ca00` | Byte first specialization slot, Byte second specialization slot |

The registration pairs are at `0x142170bd0..0x142170c08`. The 642 trait
array is ordered by specialization slot, then major-trait slot: indexes
`3 * specialization_slot + major_trait_slot`. Each dimension has three
entries. Its final byte is widened and passed to change notifications;
its semantic name and valid values remain unresolved.

## Single-slot updates

640 calls `0x141225b80`, asserting specialization slot < 3. It resolves
the UID through content virtual +0x58 with **type 386 (0x182)**, dispatches
a pre-update event, stores the pointer at component +0x58 + slot*8, clears
the slot's pending pointer at +0x40 + slot*8 and its three trait pointers
at +0x78 + slot*24, then calls `0x141226390` to recalculate the slot byte
at +0x70. It dispatches change notifications and, only when both the
owned character and its skill bar exist, invokes skill-bar virtual +0x1f8.
It ends with another event dispatch.

The recalculation helper clears the slot byte first. If the specialization
pointer is non-null, it dereferences **player +0x20 (owned character)**
without a null guard and checks character virtual +0x350. This predicate
(`0x1411d9210`) reads character flags +0x178 bit 3, now identified as
**IsControlled()** by [control-setter assertions](209026-character-control.md).
If set, the helper
requires player +0x9a60, explicitly asserted as **m_rewardTrackMgr**, and
uses the specialization's +0x50 content reference to count passed
progress thresholds. Thus the guarded skill-bar tail does not make a
non-null specialization update safe before owned-character assignment.
The flag is now named IsControlled; [server 565](209026-reward-track-messages.md)
creates the reward-track manager. Valid initial track IDs and slot bytes
remain unresolved.

641 calls `0x141225750`. Named assertions independently require
`specializationSlot < CHAR_SPECIALIZATION_SLOTS` and
`majorTraitSlot < CHAR_SPECIALIZATION_MAJOR_TRAIT_SLOTS`, both bound 3.
It resolves the UID as **type 77 (0x4d)**, stores the result in the selected
trait pointer, and dispatches a change event. Its direct path does not
rebuild the skill bar or perform the progress recalculation above. Listener
behavior is not exhaustively traced. These are content UIDs, not three-way
trait-choice indexes.

## Bulk update and swap

642 calls `0x141225c90`. It resolves and updates all three type-386
specializations and all nine type-77 traits. A specialization pointer
change triggers its change notification and progress recalculation; an
unchanged pointer skips those operations. Each trait pointer change is
also notified. The final byte is passed through to these notifications.
After the final event dispatch, it performs the same null-guarded owned
skill-bar rebuild as 640. Non-null changed specializations have the same
owned-character/progress prerequisites.

643 calls `0x141225f90`, requiring distinct slot bytes, each < 3. It swaps
the two specialization pointers, their +0x70 progress bytes, and each of
their three trait pointers; clears both pending +0x40 pointers; and emits
the corresponding events. It does not perform a new content lookup or
direct progress recalculation/skill-bar rebuild in the inspected body.
Event listeners may have further effects.

Content virtual +0x58 resolves to `0x1412e1b30`, forwarding to generic
lookup `0x1412e6820`. The setters do not assert non-null lookup results.
The [numeric-index follow-up](209026-content-numeric-ids.md) confirms that
this is the UID field at record +0x14, distinct from data key +0x28.
The index builder skips zero. This does not establish a safe clearing
operation, valid combinations, or arbitrary message order.

The [progress-listener follow-up](209026-player-progress.md#specialization-refresh-after-progress)
confirms that a matching progress update automatically recalculates an
assigned slot through an inlined callback. It preserves selected traits.
Resending 640 clears traits even for an unchanged specialization, while
unchanged bulk pointers in 642 skip assignment-time recalculation.

## Neighboring player flags: 644

Descriptor `0x1425dbb80`, handler `0x14125ca70`, has player-index Dword
and one Byte. It calls `0x1411bce70` on the player directly: input bit 0
sets/clears player +0x8c bit 5; input bit 1 sets/clears bit 3. Other input
bits do not affect that setter. The flags' names remain unresolved. This
message is distinct from the four specialization/trait layouts above.

The [complete base-content census](209026-specialization-content.md) now
maps 81 specialization data keys and 2,425 trait data keys, with all 729 major-trait
membership references resolved. Content membership, tier and runtime
eligibility remain separate from this framing layer.

## Typed framing scope

`gw2_proto::specialization::SpecializationFrame209026` covers 640–643 only,
preserving raw slots, UIDs and the 642 notification byte. It uses fixed
arrays for the three specializations and three-by-three trait UIDs, with
transactional one-frame decoding. It does not validate content combinations,
choose defaults, update server state or send messages. Out-of-range slots
are useful framing inputs but are not valid client state transitions.

Three focused tests and the library build pass. Four hand-authored fixtures
cover the layouts with distinct varint widths, zero/MAX UIDs, a raw slot
255 and notification 254. The bulk fixture distinguishes every trait
position and verifies specialization-slot-major ordering without array
length prefixes. Output prefixes and consecutive frame boundaries are
preserved. Every truncation, neighboring unsupported ID, and overflowing
or overlong varint at every Dword position is rejected without consuming
input. No runtime or content-validity claim follows from these checks.

## Evidence

Private bundle: `~/re/gw2/notes/209026-specialization-messages-1.json`
(1,107 records, 24,077,101 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_48d38b4448acd55ec1dad31d4b5da0c5f0325486e6a3ba98f5b75e794de931b9` | Bounded setter search and progress-helper references. |
| `ev_fa1d8dab46714ab2f0a620e1d5a5981192cc01ce5250ff3095f8079acdfe9426` | Runtime function bounds and handler call candidates. |
| `ev_7178621c25d25c9c0ed641736a73f42ee990ba4903c3433b0dffd5a670fbffe9` | Complete 640–643 handlers and their field arguments. |
| `ev_3280041ec6f136f1a5f5b249fd80ea1ba1884f90a233411ce2cf5da7a104bd70` | Descriptor registrations and complete single-specialization setter. |
| `ev_0badb3abf9433c8fa6b8483471e837eea5d2cc54cd9774939e32b69f8942e5bd` | Five descriptors, progress recalculation and single-trait setter. |
| `ev_d7643fb93509f8e777847a951985671b369297729e531e20285a71b965aa9826` | Nested Dword descriptor, swap implementation, named slot bounds and 644 handler. |
| `ev_00f220909eb3cd5336f12b9496fce20e172bba44ac293afcf2703efe4477fcca` | Complete bulk setter and two-bit player flag mapping. |
| `ev_579a8a969619d1b06e671dd1b0385a6f072eda4850429451f8c993ddb00cf73f` | Owned-character predicate, content lookup slot and named reward-track assertion. |
| `ev_1b0d9b9f4dcd91b8fab7695a49862d017b1d92f79eb9de9ff7b05b5d1738a3f7` | Generic content-lookup forwarding. |
