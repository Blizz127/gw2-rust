# Build 209026: quest 154 inline-key follow-up

2026-11-05. The inspected quest, goal and step text calls do **not** carry
an inline key from adjacent content fields. They copy one Dword text ID,
encode that ID into a fresh coded string, and append a zero terminator.
The formatted-source parser supports inline keys, but these generated
strings have none. For this path, a usable key must therefore already be
available through the fallback dictionary or another separately established
source. No actual key was recovered.

This narrows the unresolved requirement in
[quest 154 content](209026-quest154-content.md). It does not prove that
keys are absent from all shipped data or that only a remote server can
provide them. The exact call sites below belong to diagnostic consumer
`0x1411fe650`; this follow-up did not trace every HUD consumer or observe
the run113 dictionary at runtime.

## Exact argument flow

Each inspected call copies only a four-byte field into a stack temporary,
then passes that temporary's address to `0x1412e38e0`:

| Call site | Source field | Text ID | Stack temporary |
| --- | --- | ---: | --- |
| `0x1411fe6b7` | Quest definition +0x2c | 239267 | RBP+0x6f |
| `0x1411fe86e` | Goal definition +0x44 | 34077 | RBP+0x6f |
| `0x1411fe928` | Active goal definition +0x40 | 34076 | RBP+0x6f |
| `0x1411ff248` | Step definition +0x48 | 34078–34082 | RBP+0x77 |

The complete wrapper is two instructions:

```text
0x1412e38e0: mov ecx, DWORD PTR [rcx]
0x1412e38e2: jmp 0x1410dbdd0
```

It neither reads a Qword nor follows the original record pointer. The next
wrapper, `0x1410dbdd0`, executes `xor edx,edx` before jumping to
`0x1410dbde0`. That function obtains a text buffer through `0x1410dff70`
and calls `0x1410e1730` with R8D=text ID and R9D=0. Although its variadic
entry saves the other argument registers, this zero selects the formatter's
no-argument path; those saved values cannot supply a key on this path.

Both reuse and allocation paths of `0x1410dff70` reset the buffer's count
to zero at `0x1410e00a0`. `0x1410e1730` calls ID encoder `0x1410e21c0`,
then branches directly to `0x1410e1a24` because its argument-type value is
zero. Finalizer `0x1410e23d0` appends one zero UTF-16 word at
`0x1410e24b5`. No content-adjacent bytes, existing buffer contents or
variadic arguments enter the resulting coded string.

Derived from the inspected base-0x7f00 encoder, the complete words are:

| Text ID | Coded UTF-16 words, hexadecimal |
| ---: | --- |
| 239267 | `8107 2ea3 0000` |
| 34076 | `8101 071c 0000` |
| 34077 | `8101 071d 0000` |
| 34078 | `8101 071e 0000` |
| 34079 | `8101 071f 0000` |
| 34080 | `8101 0720 0000` |
| 34081 | `8101 0721 0000` |
| 34082 | `8101 0722 0000` |

These are arithmetic derivations from the encoder, not captured runtime
strings or decrypted text. The high bit in the first word continues the
numeric ID; it is consumed before the optional-key test.

## What the formatted-source parser actually extracts

Decoder caller `0x1410e0b50` initializes a local Qword to zero at
`0x1410e0bd0`, then calls `0x1410e50f0(start, term, text_id, &key)`.
The key is an output parameter; the function's RAX return is an advanced
source pointer, not the key itself.

`0x1410e50f0` installs parser diagnostic bounds and calls `0x1410e5200`.
For each coded string, that parser:

1. Parses the numeric text ID through `0x1410e53f0`.
2. At `0x1410e52ba`, tests the **next** UTF-16 word. Only a negative
   signed word (bit 15 set) invokes Qword parser `0x1410e5880`; otherwise
   the candidate key is set to zero at `0x1410e52da`.
3. At `0x1410e52e1–0x1410e52e7`, writes that candidate to the output
   only when the parsed ID matches the requested text ID.
4. Walks subsequent/nested coded strings and their argument tokens.

The Qword parser accumulates base-0x7f00 digits into a 64-bit output.
The generated no-argument strings above have a zero immediately after
their IDs, so their matching candidate is zero. `0x1410e0bf6–0x1410e0c49`
then performs the previously documented dictionary fallback. This confirms
the conditional empty-string explanation without assuming the dictionary
is the only key source supported by the entire text engine.

## Adjacent archive fields checked

Hash-checked reads of container 1282830 confirm:

- Quest at 4145560: +0x28=154, +0x2c=239267, +0x30/+0x34=0;
  +0x38 is the previously resolved associated-definition reference.
- Goal at 4146200: +0x40=34076, +0x44=34077, +0x48/+0x4c=0;
  +0x50 is the six-step array reference.
- All six steps starting at 4146312 with stride 0x60: +0x48 is the text
  ID, while +0x4c, +0x50, +0x54 and +0x58 are zero.

In particular, goal +0x44 is a second independently consumed text ID,
not a key extension to +0x40. More decisively, the copied-Dword argument
flow excludes every neighboring field from these text calls regardless
of its stored value.

## Evidence and scope

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Container 1282830 SHA-256:
`e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f`.

Private REA bundle:
`~/re/gw2/notes/209026-quest154-inline-key-followup-20261105.json`.

| Evidence | Observation |
| --- | --- |
| `ev_0c72cd577edd09a8e8e0ad7cb7bb3dd5d9108aefb4c27085652d9d308ba1e9d1` | Exact callers, complete Dword wrapper, no-argument API and parser entry. |
| `ev_0da18bb5da7d76ac70009448f86d6e7fad433a280974b5fa17db68c9d83c8b9f` | Key output argument, fallback, formatter branch and complete inline-key selector. |
| `ev_24f43c92b31c3193316695f7525ddf8ceddf1092437c00849b3c93b24e2c23fb` | Numeric-ID encoder and no-argument formatter tail. |
| `ev_adccb71fc5b01b0876c5404204d2d6dd89ea70db698e5666c74d209fcfc47794` | Terminator write, Qword parser accumulation and exact adjacent content words. |
| `ev_d63355af12f4560ad1d71e092ce5c60acf332fc6a972a230a93f90f6bd628710` | Fully aligned buffer allocator/reuse path, ID parser entry and derived coded words. |

All five read-only helper captures exited zero with verified owned-process
cleanup. Unrelated unreadable `/proc` entries were left untouched. One
intermediate disassembly range began within an instruction; its leading
bytes are not used as evidence, and the final capture includes the complete
aligned buffer routine. No key generation, brute force, runtime client,
server access or gameplay changes were involved.
