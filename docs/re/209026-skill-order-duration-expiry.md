# Build 209026: ordinary skill-command duration expiry

An admitted and executed ordinary kind-2 command has a **client-local end
path**. Its duration can lead to the bound character's order-completion
callback without a separate server84 packet. This establishes dispatch of a
completion callback, not successful typed0x78 release, healing, recharge, or
the retail10176 response sequence.

This is a read-only continuation of
[the echo preconditions](209026-skill-order-echo-preconditions.md). No runtime,
archive, helper, server, or deployment changes were made for this slice.

## Exact duration-to-end chain

1. Kind-2 start reaches **0x141032c00**. As established in the preconditions
   note, it requires nonzero command **+0x70**, named `m_durationMs`. It adds
   that value to world elapsed milliseconds **+0x1d4** and calls
   **0x140d4ba90**. The complete setter is only a Dword store to command
   **+0x48**, followed by return.
2. World update **0x14102f760** advances world **+0x1d4** and dispatches
   agent virtual **+0x140** through either its active-agent list or its
   general list. Which list it uses depends on world **+0x1cc** bit7.
   The update itself requires world **+0x1cc** bit0.
3. The previously established ordinary bound-agent table **0x14211aee0**
   maps **+0x140 → 0x1410258c0**. That update requires its own Dword
   **+0x30** bit2 clear and bit3 set. It calls **0x141039490** for each
   nonnull active-command slot at bound-agent **+0xd8** and **+0xe0**.
   These are object-relative offsets; they are not the character's order
   component slots.
4. Complete leaf **0x141039490..0x1410394d0** reads command **+0x48**. Zero
   returns immediately. Otherwise it obtains the world from command
   **+0x58**, compares world **+0x1d4** with the deadline, and returns while
   the subtraction's sign flag is set (`cmp; js`). On expiry it clears
   command **+0x48** before dispatching the end path. This records the exact
   comparison without assuming an unbounded signed or unsigned time domain.
5. With command **+0x20** bound-agent nonnull, the reader passes the command
   in RDX, zeroes R8D, and tail-dispatches bound-agent virtual **+0x178**.
   Its ordinary table slot at **0x14211b058** is **0x1410261b0**. This thunk
   subtracts **0x18** from RCX, sets R9D to1, and jumps to common end
   **0x1410261c0**. The null-bound-agent fallback instead dispatches command
   virtual+0x20; it is not the ordinary bound-character chain claimed here.
6. The already retained complete common end **0x1410261c0..0x141026382**
   clears the corresponding active command slot and invokes the command's
   own end callback. If the bound-character interface exists, command flag
   **bit17 is clear**, and **bit12 is set**, it then passes the same command
   behavior container (**command+0x28**) to the character lifecycle callback:
   **bit8 set → virtual+0x80 cancellation; bit8 clear → virtual+0x90
   completion**. The latter maps through **0x1411dc080** to order-component
   completion **0x14127fe10**.

The final callback still uses the established slot and behavior/kind matching
rules in [the predicted-order lifecycle note](209026-predicted-skill-order-lifecycle.md).
It is not an unconditional completion of whichever order currently occupies
the slot. Replacement, cancellation, owner removal, changed flags, or an
unserviced update path can change the outcome. For an unchanged server34
kind-2 command, the constructor's `0x801000` flags provide bit12 and leave
bits17/8 clear; this does not prove those values remain unchanged at runtime.

## What the positive-duration experiment can establish

The source supports expecting an ordinary command-end callback after its
selected diagnostic duration, **if admission, execution, continued update,
and correlation all succeed**. Server84 is not a prerequisite in this
specific duration-to-end chain. The existing passive observer's actual
completion-return hook can distinguish callback execution and its release
request from the separate start observation.

This slice does not re-expand the typed0x78 completion state machine. It does
not establish that its callback requests release, that a release finishes,
that server84 has no other role, or that any health/recharge effect occurs.
The correlated server84 updates remain a separate source contract. The
selected1000ms remains an explicit research value, not10176 retail timing.

## Retained evidence and bounds

Frozen executable SHA256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

Private complete bundle:
`/home/blizz/re/gw2/notes/209026-skill-order-duration-expiry-20261106.json`
SHA256 **`e1208b0a6b68cf8e35ffa451c9f43029ad420c96174f8660059fffb11445dc1e`**.
All12 captured queries exited0, retained untruncated output, and verified
owned-process cleanup. Missing leaf `.pdata` bounds and initial partial
function fragments are retained, followed by explicit bounded continuation
reads. Adjacent printed functions are not assigned to this chain.

| Evidence | Supported observation |
| --- | --- |
| `ev_8a8f116bb44155951e4d662ae418ba474c52fa466d1881fd3cfb376ce1f11a57` | Complete two-instruction deadline setter. |
| `ev_d8d307bf2a50b9afc76b1969014faa5e54a9cf9e4c74569fc403a6fe49cbd2f6` | World elapsed-time increment and agent+0x140 dispatch; normal return, not every trailing assertion block. |
| `ev_6576d1f726db3247bf0121b08dbeb56db3cd3f58786404f797f2dfd86bda58c9`, `ev_89bab32eb1ebb11d8c3d7411cc53217df2b22c86e9f027cd349beed925151982` | Concrete bound+0x140 target and complete ordinary update through return. |
| `ev_607eed62afa2b73ab6b0c5e5ea60eb49db84e432c7c694697d36e1dd6cdf4113` | Complete deadline reader and bound+0x178 end dispatch. |
| `ev_d12f19766771c54275428a5d3b98da94fdeacbf0ea32ab0a6a1126d5982640c5` | Exact aligned pointer from ordinary bound+0x178 to the end thunk; bounded supported-reference encodings only. |
| `ev_54000555b438c2da6bade948530ff48c153e19c7fb5d9fff23e63669f2e76dac` | Previously retained complete end thunk/common end and character cancellation/completion selection. |

The last Evidence record is reused from the private bundle linked by the
predicted-order lifecycle note; it is not relabeled as a new observation.
There was no broad scheduler census and no claim that source analysis
observed the staged experiment execute.
