# Build 209026: checked animation-sequence reader

2026-11-06. `gw2_dat::animation_sequence` now reads selected animation
records from the frozen client's `anic` / `seqn` v0 format. It preserves
the original event order and raw values, including coincident events,
unknown types/flags and full-width Dwords. It does not choose a skill's
animation, compute a cast schedule, send packets or apply effects.

## Source contract and reader policy

The [animation lookup and exact-record trace](209026-mesmer-heal-animation-data.md)
establishes the 12-byte root, 20-byte primary rows, 56-byte variant rows,
nine-byte events, 25-byte segments and signed 64-bit relative pointers.
The new reader uses those fields directly:

- `AnimationSequences209026::read(pack, max_sequences)` requires a 64-bit
  `anic` pack with exactly one `seqn` version-zero chunk. It validates the
  bounded primary table without allocating a copy.
- `find(animation_token, variant_token, max_variants, max_items)` selects
  the exact pair. A missing token or variant returns `None`; no alternate
  token, variant-zero or nearby-record fallback is invented.
- `max_items` caps the selected row's combined event and segment count
  before either output vector is allocated. Each extent is checked against
  the selected chunk payload, not the whole pack or trailing fix-up bytes.
- Zero relative pointers remain null. Negative relative pointers and
  nonzero pointers targeting payload offset zero are supported. Extreme
  signed values cannot wrap into the payload.
- Event kind/+1/+5 and segment kind/+5 are retained. Both optional segment
  pointers at +9/+17 expose only the pointed first Dword; the remainder
  of either data object is outside this reader's scope.

Primary and selected variant keys must be sorted and unique. Rejecting
ambiguous or unordered tables, duplicate `seqn` chunks, nonzero versions,
invalid unused segment pointers and excessive counts is this reader's
validation policy, not a claim that the retail client rejects them in the
same way. Unselected variants' event and segment arrays are not traversed.
Charge stages, other variant fields, fix-up execution and other chunks'
schemas are not interpreted.

## Focused verification

`cargo test -p gw2-dat animation_sequence --lib` passes all three tests.
The synthetic fixtures cover backward pointers, payload-offset-zero versus
null, exact/missing token pairs, empty variants, raw unknown kinds/flags,
coincident-event ordering, combined limits, truncated payloads at every
required byte, extreme pointers, null nonempty arrays, ambiguous keys and
wrong/missing/duplicate/versioned chunks. These fixtures contain no copied
client assets.

A separate compiled Rust checker and the previously retained independent
Python extractor agree on both exact skill10176 records in cached pack
184788. The reader validates 1853 primary keys, then the selected token's
35 or 19 variant keys, and decodes only variant zero:

| Token | Ordered `(kind, +1, +5)` events | Segment kind/flags | Pointed first Dword |
| --- | --- | --- | --- |
| `0xb309585954f7` | `(4,0,0)`, `(5,867,0)`, `(1,1067,0)`, `(3,1067,0)`, `(2,1067,0)`, `(0,1350,0)` | 0/0 | +9:2067; +17:null |
| `0x215a908ef7` | `(1,400,0)`, `(3,467,0)`, `(0,800,0)` | 0/0 | +9:1667; +17:null |

Exact lower-than-required primary, variant and combined-element limits
also reject these real records. Both implementations read the cached
8,916,756-byte pack, SHA-256
`72e5fce4c2098946d1644469881fcc43795349ea7e16c8a357b3c5594c35723e`.
The wrapper verifies the module, checker and prior extractor identities,
then verifies the module and cached asset remain unchanged. No archive,
game runtime, shared container, saved roster or publisher state was accessed.

REA evidence `ev_107f22e43b290aec88608fefa94da91c10883a7f3dc4145ab0f091f4b9e43284`
exited zero with all 793 raw output bytes retained, no truncation and owned
process cleanup verified. Unrelated inaccessible `/proc` entries remain
outside that cleanup claim. Private evidence bundle:
`~/re/gw2/notes/209026-animation-sequence-reader-20261106.json`, 5,791,269 bytes,
SHA-256 `4d5b9f305d28b209fe5021d2e01d8c703fc3ad7adc87baff57ec3b244a6fa925`.
The bundle includes prior retained evidence; the ID above identifies this
new check. Private checker:
`~/re/gw2/notes/check-animation-sequence-20261106.py`, SHA-256
`c854737cab4a65ceafdbb999f353643379665fee0891feaec6d8a97c1e328271`.
Reader module SHA-256:
`db63c62ca575a245231e7e4eb68cc4f9aa4bcb17460dd754c266a013a8a95161`.

The [event-consumer trace](209026-skill10176-animation-event-consumers.md)
establishes visual notifications and local view state for the selected
ordinary consumers. The [tooltip trace](209026-mesmer-cast-tooltip.md)
establishes display rounding. Neither makes these raw values an
authoritative server healing, interrupt, success or recharge policy.
