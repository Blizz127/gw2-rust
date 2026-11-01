# Build 209026: creation answer bytes

Static REA analysis, 2026-11-01. Target identity and request-415 layout are
recorded in [character creation](209026-character-creation.md). No client run,
Agreement interaction or character creation was performed for these findings.

## Source and selection

Creation initializer `0x14129f130` retrieves content type `0x104` through the
content service's vtable slot +0x48. Its null assertion names `madLib` at
`0x141b2c908`. The returned object's +0x28 pointer/+0x30 count describe
0x48-byte question records. Each question has an answer pointer at +0x20,
answer count at +0x28, applicability flags at +0x30, and a question index at
+0x34. Answers occupy 0x40 bytes each.

The initializer clears the byte-vector count at creation data +0x4c and frees
the separate question-bitset storage at +0x60/+0x68. For each question,
`0x1411c5710(flags, race, profession)` returns zero when applicable. The
initializer then selects `random % answer_count` and calls context vtable
slot +0x120, resolved to `0x14129fbd0`. This particular path does not inspect
each answer's applicability flags before choosing it; the inspected code also
has no zero-answer-count guard before division. These are observations of
content assumptions, not assertions that malformed content exists.

A separate helper, `0x14129fe40` (slot +0x128), asserts `question`, checks the
question's applicability, then filters individual answers using their +0x10
flags and the same predicate. It selects from that filtered list, calls the
same setter, and returns the selected answer pointer, or null if none qualify.

## Applicability flags

The predicate requires race 0–4 and profession 1–9. It returns nonzero for
inapplicable content. The selected race bit must be set. For profession,
either its selected bit or bit 14 must be set:

| Selector | Flag bit |
| --- | --- |
| Race 0, 1, 2, 3, 4 | 0, 1, 2, 3, 4 respectively |
| Profession 1 Guardian | 7 |
| Profession 2 Warrior | 12 |
| Profession 3 Engineer | 5 |
| Profession 4 Ranger | 10 |
| Profession 5 Thief | 11 |
| Profession 6 Elementalist | 6 |
| Profession 7 Mesmer | 8 |
| Profession 8 Necromancer | 9 |
| Profession 9 Revenant | 13 |
| Profession-independent alternative | 14 |

The numeric names come from the previously mapped enum getters. Bits outside
this table are not tested by this predicate; their meanings remain unknown.

## Stored and transmitted representation

Setter `0x14129fbd0(context, question, answer)` uses the question's +0x34 index
to address a bit in the Dword vector at creation data +0x58 (pointer +0x60,
count +0x6c). If that bit was already set, it searches the sorted byte vector
for each answer belonging to this question and removes the first matching
answer byte. It then ensures bitset capacity and sets the question bit.

The value inserted is **the byte at answer +0x14**, not the question index or
the answer's position within its question. A lower-bound search supplies the
insertion position to `0x14129ea60`, which grows the byte vector, shifts later
bytes, and inserts that value. This path therefore maintains ascending byte
order. Global uniqueness of answer values is not established by this code.

The vector lives at creation data +0x38 (pointer +0x40, count +0x4c).
`0x14129cc20` passes that vector to `0x141410d70`, the previously mapped
request-415 bridge. Thus request 415's `ArrayVarSmall Size=32` carries these
content answer bytes. The separate question bitset is not passed in that
argument. A bulk setter at `0x14129f3f0` can copy a byte vector directly;
sorting is a property of the per-answer setter, not a proven universal
invariant for all callers or a server-side acceptance rule.

Changing race in `0x1412a0050` calls the initializer again before rebuilding
appearance. Actual content values, localized question meanings, mandatory
answer combinations and server validation remain unverified. The earlier
synthetic request-415 fixture is still only a wire-boundary check.

## Evidence

Private export: `~/re/gw2/notes/209026-creation-answers-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_fb7d3fddc7c2d784843da0f612d72584ff87212f69bdb3fdaed40f4046f6ea27` | Content retrieval, question iteration, random default selection and bulk byte setter. |
| `ev_775f6d3f94e9e43a010ce25736d0269c5f8172aca8d6b8ec7aeceea34afce368` | Applicability predicate and creation vtable pointer reference. |
| `ev_c6772b65ff604f0981b7f58be89b4026b70f5e7701b4da6c48df848cbdeb9618` | Vtable slots, profession bit jump table and madLib assertion. |
| `ev_fa5be1564bf34f33087c4a02828d60652294b54716ef8f7400eceede32b1ff4e` | Answer replacement, question bitset, sorted insertion search, filtered selection and race reset. |
| `ev_e9f9a114ae887363e88bd14667eb9fe0cd5143f2c1cf7bd2c3bde5f7956d65b0` | Byte insertion implementation, request bridge and question assertion. |
