# Build 209026: controlled-player progress messages

Static REA analysis, 2026-11-01, for the frozen client identified in
[client-build.md](../client-build.md). This follows the
[reward-track content census](209026-reward-track-content.md).
No messages were replayed and no progress default is selected.

## Dispatch and wire layout

These protocol 0/group 0 handlers first resolve **type 53 by data key**,
then obtain the controlled player through character-context virtual +0x70.
They have **no player-index field**. If the controlled player is absent,
they skip the update after the content lookup. Missing content is asserted
(or reported fatally in 509), not treated as a clearing instruction.

| ID | Pair / descriptor / handler | Payload after ID |
| --- | --- | --- |
| 509 | `0x1421702e0` / `0x1425cdda0` / `0x141257720` | Dword progress key, Word bit-channel mask |
| 510 | `0x1421702f0` / `0x1425cdee0` / `0x1412577c0` | Dword progress key, ArrayVarSmall(255) Dword |
| 511 | `0x142170300` / `0x1425ce000` / `0x141257860` | Dword progress key, ArrayVarSmall(4) Dword |

509/510 use content virtual +0x1e8 (`0x1412e1db0`), forwarding type 53 to
+0x70. 511 calls +0x70 directly. The [numeric-ID audit](209026-content-numeric-ids.md)
establishes this as record +0x28, distinct from UID +0x14.
Nested definition `0x1425ace00` contains one Dword. Dwords use unsigned
varints; kind-17 arrays use a one-byte count, as already established in
[character protocol](209026-character-protocol.md#array-boundaries-for-the-capture-census).
510's bit-mask unpacking is only partially traced and remains outside the
small codec added for 511.

## Progress representation

Player +0x9760 is the embedded manager named by the `ChCliProgress.cpp`
source diagnostic at `0x14216c810`. Its sorted lookup is documented in the
reward-track note. Helper `0x141231c80` returns an existing object or
inserts/allocates one from a content definition. Constructor `0x141231460`
stores that definition at object +0x10 and vtable `0x14216c718`.
Object virtual +0x20 (`0x141231e10`) returns definition +0x2c.
Named assertion strings and branch comparisons establish:

| Value | Progress data type |
| ---: | --- |
| 0 | BIT |
| 1 | BIT_MASK |
| 2 | COUNTER |
| 3 | MAXIMUM |

The counter/maximum representation is a 0x48-byte object with vtable
`0x14216c778` and sixteen cleared Dwords at +8. Its getter
`0x141231970` returns the selected Dword. Definition +0x40 is a channel
mask. Progress virtual +0x30 (`0x141231ed0`), used by reward tracks,
returns the **unsigned maximum** over enabled channels 0–15; it does not
sum them. The one-bit representation stores the raw channel mask at +8;
509's setter `0x141232120` requires type 0 and replaces that mask with
the zero-extended Word before notifying.

## Server 511: counter/maximum assignment

The handler copies its decoded Dword array into a temporary bounded vector
and calls `0x1412324b0`. That helper obtains/creates the object and requires
data type 2 or 3. It walks enabled bits of definition +0x40 in increasing
channel order, assigning consecutive supplied values to those channels.
It stops when all supplied values are consumed; excess values that cannot
fit enabled channels produce an assertion. It does not require supplying
every enabled channel, so a shorter update leaves later channels unchanged.

Each consumed value replaces the stored Dword, with no arithmetic increment
or comparison against its old value. This also holds for type 3 in the
inspected setter; its type name alone does not establish monotonic updates.
Each assignment dispatches manager listeners at virtual +0 through
`0x140255568`, passing the progress object, channel and prior value.
After all assignments (including an empty array), the helper dispatches
virtual +8 through `0x140255560`. Callback dispatch passes through generic
event helpers; downstream behavior is not exhaustively established.

The reward manager's subscribed progress interface is at manager +8,
vtable `0x1421712e8`. Its per-channel callback `0x141248d40` compares the
updated progress definition to each nonnull selected track's +0x40 target
and updates matching track-slot state and notifications. Its final callback
`0x141248f00` first tests progress virtual +0x40; if true, it enumerates
type-353 content and evaluates matching track dependency references.
This expands the content/listener work of a live progress update; no safe
arbitrary ordering is inferred from the small wire layout.

## Guardian specialization consequence

All nine progress definitions identified in the Guardian census have
**data type 2 and channel mask 0x10**. Therefore a 511 update with exactly
one value targets channel **4**. The reward getter returns that value,
and specialization recalculation compares it with the mapped thresholds.
For example, specialization data key 13 / wire UID 15 references track
key 103 and progress key 4894. This maps a possible local fixture field;
it does not select or validate an unlock level. A framing-valid empty or
four-value array is not automatically a valid content update.

## Specialization refresh after progress

Player construction calls specialization constructor `0x141224c70` at
player +0x9cc0. It clears the selected specializations, progress bytes and
trait pointers, then subscribes its secondary interface (+8, vtable
`0x14216b378`) to player +0x9760 through progress virtual +0x30.
Subscription `0x140320a90` only inserts the listener, ignoring duplicates;
it does not immediately invoke it. Secondary virtual +0 is the no-op
`0x140163990`; secondary +8 is `0x141225830`.

Thus 511's final notification reaches the specialization listener once
per message, after all supplied channel values have been assigned. For
each nonnull selected specialization, `0x141225830` compares its
track +0x40 progress definition pointer with the updated object's
virtual +0 definition getter (`0x140273290`). Unmatched slots are skipped.
For a match it clears/recomputes the corresponding byte using the same
controlled-owned-character/reward-manager checks and threshold loop as
assignment. This logic is inlined: looking only for direct calls to
`0x141226390` misses it. A changed byte emits specialization-listener
virtual +0x28 through `0x14026e8bc`. The callback itself neither clears
traits nor directly calls the skill-bar rebuild; further listener effects
remain open.

This establishes automatic progress-byte refresh for an already assigned
specialization with the required owned/control/manager state. It does not
require another assignment message. Repeating 640 would clear that slot's
three selected traits even when its specialization UID is unchanged.
Repeating an unchanged 642 specialization pointer skips assignment-time
progress recalculation. Neither operation should be used as a generic
progress-refresh substitute. A missing owned character remains an
unguarded dereference when the progress callback matches a nonnull slot.

## Typed framing and focused checks

`gw2_proto::progress::CounterProgressFrame209026` covers server 511.
It encodes the data key and one-byte-counted Dword vector, bounds the count
at four, and leaves output unchanged for an oversized vector. Decode is
transactional. It does not validate content/channel compatibility, create
server state or emit messages automatically.

Three focused tests and `cargo build -p gw2-proto --lib` pass. Independent
hex fixtures cover empty, one-value and four-value arrays; zero/MAX keys;
and distinct varint widths. Checks include consecutive frame boundaries,
append prefixes, all truncated fixtures, unsupported adjacent IDs, bad
counts, and overflowing/overlong varints in every Dword position. The
Guardian-shaped fixture is a framing example, not a selected live default.

## Evidence

Private bundles: `~/re/gw2/notes/209026-player-progress-1.json` and
`~/re/gw2/notes/209026-player-progress-refresh-1.json` (listener follow-up).
The private `inspect-guardian-progress.py` follows the earlier recorded
track targets through REA and checks definition type and channel mask.

| REA evidence | Observation |
| --- | --- |
| `ev_47f579111a0bafe47489c1ade039dbc22d1ade5901d042d44e9f25e7faba3651` | Progress lookup adapters, insertion/allocation and source diagnostic. |
| `ev_e6e8ce0f6571f95637381854e8b32662f968f0be09d1b0d0ed7c822017f9dd14` | Constructor type branches and named data-type assertions. |
| `ev_1ebdadfd4b4f9b93c1108689de0365b4b45a4030c7242a4569bc92bc8aa56c0a` | Complete bit and counter setters; progress vtable. |
| `ev_75e8fe9329b22886ffed4a46aaac5a5e0d49531a5105784ca5344605f4af0813` | Setter callers, type getter and unsigned maximum aggregation. |
| `ev_841cabf5bbf718f62f24f12adf6e2bf90d38e3b5f4cd471f29a7de47d4e2876c` | Guardian type/mask values and three message handlers. |
| `ev_1da3c650a7e2a3ac4d3d25096c1481365fb54f1f7ba6be965184dd31afb549fd` | Registration pairs, channel getter and event-slot dispatch. |
| `ev_0ac0a299c0194246e9a66029ffe8af5bcb4c8129b0fcbdd5351fc36e58421cc0` | Hash-guarded descriptors, type-53 key adapter and constructor tail. |
| `ev_1cef4dba324e334655dc3ebc52b1a203770f03e4a17ee68c4691aecc734a6081` | Reward-manager progress callbacks. |
| `ev_e063b038165d1719acb0e0e36eb178076693c718375185c075e6f9570bbe1ebd` | Player constructor and assignment/bulk recalculation branches. |
| `ev_c5e64eb9bd33e781ca4ef506e22236d12a6b048475a78f75a28c757261a97395` | Complete specialization constructor and progress subscription. |
| `ev_602331a0afe151898bc2047646f359a05c1c1ec2d2ff4a9e645b5b3ad294853b` | Specialization secondary listener vtable. |
| `ev_8ae72d407a004fd7b1e3c57a13b5d92c03b3375757077768556ad32372673a4e` | Complete final progress callback, three inlined recalculation branches. |
| `ev_82408acafd8deee21310ba3a2eb5a977d2690877e593f74be8fac1e5672b7517` | Definition getter, specialization event slot and complete subscription helper. |
