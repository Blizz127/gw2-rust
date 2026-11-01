# Build 209026: bit and bit-mask progress updates

Static REA follow-up to [player progress](209026-player-progress.md),
2026-11-01. The frozen client remained stopped; no progress was replayed.

## Server 509

Handler `0x141257720` resolves the type-53 data key, obtains the controlled
player and zero-extends the decoded Word into setter `0x141232120`.
The setter gets/creates the progress object, asserts data type 0 (BIT),
and replaces its representation's Dword +8 with this mask. It dispatches
only the final progress notification (listener virtual +8), including
when the new mask equals the previous mask. It does not apply the
content channel mask to the incoming value.

Wire fields after the little-endian message ID are one unsigned Dword
varint and one fixed little-endian Word. There is no player index.

## Server 510

Handler `0x1412577c0` passes the decoded byte count and Dword-array pointer
to setter `0x1412321a0`. The descriptor permits 0–255 Dwords, each encoded
as an unsigned varint. The setter requires data type 1 (BIT_MASK).

The block width comes from Dword +0x10 in the parameters referenced by
definition +0x38. It converts that unsigned value to float, multiplies by
`0.03125` (`0x3d000000` at `0x14192df28`) and calls the ceiling helper
`0x1409bee70`. Thus ordinary representable content sizes use
`ceil(parameter / 32)` Dwords per channel. This is a float calculation,
not evidence for integer arithmetic at extreme parameter values. The
parameter pointer is dereferenced without a null guard.

For nonzero block width, the setter asserts that the supplied count is
an exact multiple of the width. It scans enabled bits of definition
+0x40 in increasing channel order, stopping when the remaining count is
zero or channel 15 has been visited. Each channel gets the next block,
replacing its previous Dword vector; trailing zero Dwords are trimmed
from the stored vector. A short whole-block update leaves later channels
unchanged. The inspected function has no final assertion that excess
whole blocks were consumed. Zero width takes a distinct path that does
not consume input; no useful fixture is inferred for it.

The setter dispatches only the final progress notification (listener
virtual +8), including for an empty array. Unlike server 511, it does
not dispatch per-channel notifications. Completion for this data type
uses a separate parameter access in `0x141231f80`, mapped below.

## Completion counts set bits, independently of block width

Progress virtual +0x10 (`0x141231bf0`) returns the BIT_MASK representation,
whose vtable is `0x14216c768`. The constructor allocates 0x208 bytes and
initializes sixteen 0x20-byte vectors beginning at +8. Representation
virtual +0 (`0x141231850`) visits the selected vector and counts its set
bits. It isolates each least-significant set bit, obtains its index through
`0x1409bf190` (BSR on that isolated bit), and advances to the next set bit.
It counts all stored words; it does not mask off bits beyond the declared
width in this getter.

The type-1 branch in progress virtual +0x38 (`0x141231f80`) compares that
count against **Dword +0 in the parameters**, unsigned. The storage width
at parameter +0x10 is a separate field. All-channel completion still
requires each enabled channel to pass. `ProgressDefinition209026::bit_mask_parameters` exposes both raw
values without assuming they are equal or selecting an initial mask.
The bounded reader returns no bit-mask parameters for other data types.
Four progress-reader tests pass, including unequal count/width, zero width,
external targets, null parameters and truncated parameter extents.

## Complete frozen base census

Four bounded REA batches cover all 32 base containers. The helper checks
content types, classified references and parameter extents, retaining
source hashes and every record's UID/key/location. Across **23,939**
type-53 definitions, keys and UIDs are each unique and every parameters
reference is nonnull:

| Data type | Count |
| --- | ---: |
| BIT (0) | 11,337 |
| BIT_MASK (1) | 2,986 |
| COUNTER (2) | 9,547 |
| MAXIMUM (3) | 69 |

Four definitions have zero channel masks; eight have multiple enabled
channels. No definition has mask bits above channel 15. These are catalog
observations, not universal parser restrictions.

For BIT_MASK, the declared widths range from 0 to 249 bits. Twelve have
zero width; key 4766 is the sole multi-channel BIT_MASK record and also
has zero width. The completion-count field differs from width in **861**
records. For example, key 7310 has completion count 120 but width zero.
The maximum `ceil(width / 32) * enabled_channel_count` is eight Dwords
in this catalog, below the wire format's 255-Dword bound. No useful
runtime update or default is inferred for the zero-width records.

## Rust catalog verification

`gw2-progress-check` reads all 32 frozen base containers, checks UID/key
uniqueness and resolves every counter/maximum or bit-mask parameter record
through the bounded reader. The optimized archive smoke matches the
independent census exactly: 23,939 definitions, 9,616 counter/maximum
limits, 2,986 bit-mask parameter records, twelve zero widths and 861
unequal completion-count/width pairs. Four CLI rejection cases also pass.

Both debug and release checker builds pass. The debug archive runs hit
REA's 60-second and 180-second deadlines without a result; neither is
counted as a pass. Process cleanup was checked after each timeout. The
optimized run completed in about 15 seconds. Pointer classification
currently scans fixup tables per query, making this full catalog check
expensive in a debug build. For this check, use:

```sh
cargo build --release -p gw2-dat --bin gw2-progress-check
# Run the resulting checker against the frozen archive through REA.
```

## Codec scope

`gw2_proto::progress::BitProgressFrame209026` and
`BitMaskProgressFrame209026` cover 509 and 510 without imposing content
semantics. Decode leaves input untouched on truncated or invalid frames.
The 510 encoder rejects more than 255 Dwords before appending anything.
Three focused tests and the library build pass: independent golden bytes,
fixed Word encoding, maximum 255-word arrays, invalid varints, truncation,
stream boundaries and transactional failure behavior. Frame validity does
not establish a suitable content key, block count, channel selection, or
safe initialization order.

## Evidence

Private bundle: `~/re/gw2/notes/209026-bit-progress-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_210646ff7632fc36e81f276ebcdd52104d7350be70ebb32099b80b0d6925a6c7` | Complete 509/510 setters, block splitting, trimming and final notification. |
| `ev_900294e13ca3dc901fc5b531603977e0d699514614ffc567e093dae9a34f2f3d` | Float multiplier, ceiling helper, named BIT_MASK assertion and completion branch. |
| `ev_d9fa6fa483b870c0d42dc88f576e5a48813bcecdf2d0ab91101add7bc8a34639` | Complete 509/510 handlers, progress vtable and ceiling constants. |
| `ev_b65cec85dad0f20e60074247dc07a8bf8689e7c79139067e7707c9e52ce51f08` | Containers 0–7: 6,365 progress definitions. |
| `ev_651b7a8e577fd499a4ad631400d73c2196e0760d5c2646a61069fe955302b885` | Containers 8–15: 4,795 definitions. |
| `ev_c7cfc792d16c77bfe3b902b7bb5f0035f6c3e840764b76ecb5390d7cc20c630c` | Containers 16–23: 4,804 definitions. |
| `ev_ea270637ab22e18aafddd1639a3a97824ea58d89aef1e5b5837930218da5d134` | Containers 24–31: 7,975 definitions; BIT_MASK constructor and interface getter. |
| `ev_4c3fb302b0e423a9ec35650ff7e6f28774e389237f71ba18c7c60c2062923068` | Complete per-channel set-bit count getter. |
| `ev_d1af07f71e38727cb3e137629304f5d0e7abd52401d7b3ba48f2df82da963790` | Aggregate uniqueness/counts, zero-width records and BSR leaf. |
| `ev_c8cc4b2086197cc35e809d54efb691f3b560bfa16c42550153772e045da26b67` | Optimized Rust smoke matches the independent complete census. |
