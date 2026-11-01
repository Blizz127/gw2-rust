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
uses a separate parameter access in `0x141231f80`; the counter limit
reader deliberately does not interpret it.

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
