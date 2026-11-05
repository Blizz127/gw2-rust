# Build 209026 creation error codes

REA source investigation, 2026-11-05, against frozen client SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The three creation-specific codes previously left unnamed in the
[creation note](209026-character-creation.md) now have exact frozen English
text. No client playback of these errors is claimed.

## Exact creation name errors

Server 1051's handler `0x1414128f0` reads the low Word of each of its four
decoded Dwords and emits event `0x10000002`. The existing creation event
path forwards that four-Word result and the current name to
`0x1402eeac0`, then reenables the form controls.

The formatter branches on the **first Word**, selects these exact text IDs,
and passes the current name to the localized formatter:

| First result Word | Selected text ID | Verified English text excerpt | English pack / index / record offset |
| --- | --- | --- | --- |
| 0x19 (25) | 47973 / 0xbb65 | `"%str1%" already in use by this account.` | 2440770 / 869 / 46056 |
| 0x1d (29) | 46964 / 0xb774 | `"%str1%" already in use.` | 2440769 / 884 / 56009 |
| 0x1f (31) | 278084 / 0x43e44 | `Inappropriate name. "%str1%" contains words or letter combinations that are not permitted.` | 2440995 / 580 / 38181 |

All three continue with an instruction to select a different name and retry.
These are creation-specific overrides: the generic error-text lookup does not
supply these name-specific texts for those codes.

The native instruction chain is:

- `0x1402eeb2a`: code 0x19 selects ECX = 0xbb65.
- `0x1402eeb23`: code 0x1d selects ECX = 0xb774.
- `0x1402eeb1c`: code 0x1f selects ECX = 0x43e44.
- `0x1402eeb2f`–`0x1402eeb3a`: forwards the name in R8, with EDX = 7,
  to `0x1410dbde0`. This is the text-format call; 7 is not an inferred
  decryption key.
- `0x1402ee890` appends the formatted result code through text 49122
  (0xbfe2), whose exact text is `%str1% (Code=%str2%)`. Its native code
  format at `0x14193fcb8` is `%u:%u:%u:%u:%u`: four result Words and
  the product getter's value.

Before selecting text, the formatter calls `0x140fe4c00` for codes other
than 0x1d/0x1f. The inspected helper copies the eight-byte result into a new
allocation; it does not modify the input result.

All five inspected specific/wrapper/default text records have marker Word +2
equal to zero, width byte +4 equal to 16 and flags byte +5 equal to zero.
They follow the previously established
[no-key UTF-16 path](209026-species-plain-name-coverage.md).
The existing archive batch reader and text manifest 110865 were reused.
No encrypted record or guessed key was needed.

## Bounded generic error lookup

The generic helper `0x1402ef480` was inspected, including its three
indexed branch tables for codes 3–0xba, 0x408–0x44e and 0xbc0–0xbee,
plus explicit codes 0x3f6 and 0xbba. The private catalog records all 304
covered code selections: 303 select one of 63 direct text IDs; code 8
enters an address-dependent branch. Its two texts and the two additional
composition wrappers were read separately. All 67 IDs were readable
zero-marker UTF-16 records.

Useful exact additional meanings:

| First result Word | Text ID | Verified English text |
| --- | --- | --- |
| 0x29 (41) | 49086 / 0xbfbe | `New character limit exceeded.` followed by an instruction to wait a few minutes. |
| 0x2b (43) | 48508 / 0xbd7c | `Maximum character limit reached. Please delete an existing character to create a new one.` |
| 0x31 (49) | 47749 / 0xba85 | `Unable to complete the operation (permission denied).` |

None establishes a generic “unsupported creation request” or unsupported
race/profession/appearance error. The default text 48915 is a network-error
message, not evidence that an arbitrary code is appropriate for unsupported
local requests. No such code was assigned in this work. This is a bounded
UI-text mapping, not a recovered complete server error enum or acceptance
policy.

## Local name-conflict behavior

`CreationScene` now maps the store's `AlreadyExists` error to server 1051
with raw Dwords **[0x19, 0, 0, 0]**. The first value is sourced above; the
remaining zero values are the explicit local profile's result details,
not recovered retail server diagnostics.

The failure leaves scene readiness intact and does not set a created request,
so a corrected name can submit another 415. It logs constant metadata only.
Other store errors retain their prior behavior.

The new regression creates one durable record, submits a conflicting name,
decodes the 1051 reply both through its codec and the frozen server table,
then submits a different name without restarting the scene. The latter returns
1048, and reopening the store recovers both intact records with distinct GUIDs.
All **five creation-scene tests pass**, including prior fragmentation,
readiness, persistence and duplicate-message coverage. These are local
synthetic-frame tests, not frozen-client UI playback.

## Retained evidence

Private complete captures:
`~/re/gw2/notes/209026-creation-error-codes-1.json`.

The generic enumeration and all pack hashes are in
`~/re/gw2/notes/creation-error-catalog-20261105.json`, SHA-256
`e8be6110f8a18510f7a15889df389df8e939d1afe46d02b596166b7133527777`.
Its helper is the adjacent `.py` file.

| Evidence | Observation |
| --- | --- |
| `ev_d82b9f112b5212c66daaa3499770eda8fc15a66502a0a3fceacbeae6f154a71f` | Complete creation-specific formatter and result-code wrapper. |
| `ev_19b4b24a7430ed26db1107d5608c379faaee6be3364f89fcb208c8daa083bc17` | Text entry point and generic error lookup with branch tables. |
| `ev_46ed03b5e596cced920bcd5eaf6fe4c6bd3ab291db3a3bc224ae7e96ad262bb1` | Exact three creation texts, result wrapper and default text; pack hashes. |
| `ev_d4e43bebb15961ff4c468fe39e9ae046cdcd7a3b9f91ea2069d28fef013d0720` | Generic indexed-code catalog and 63 direct readable text IDs. |
| `ev_e86e041bdaf3ccfc12361704768feb8c6247307829c20e3d768454f34a5d743b` | Server 1051 low-Word forwarding, result-copy helper and numeric format literal. |
| `ev_854abf78127491394a61aa2ea78faeb62ad858699ade4f46e28ece6f1f7200d6` | Four additional conditional/composition texts. |

All six captures exited zero and verified cleanup of their owned process
groups. No shared Ghidra target switch, release build or runtime launch was
performed in this slice.
