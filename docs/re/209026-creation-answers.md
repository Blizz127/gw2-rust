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

## Frozen archive content

The [base-container lookup](209026-content-registry.md) makes the actual
question data accessible without entering the client UI. Static tracing of
`0x1412e6360` confirms that service slot +0x48 iterates a content type,
using the caller's cursor; the creation initializer supplies cursor zero
and requests type 260.

A read-only REA scan of all 32 base containers found exactly **one indexed
type-260 record**: archive file **1282831**, index entry **17301**, content
offset **3991224**. The decoded packfile's SHA-256 is
`f9af9a85b3de5bd09c1840d058bb61442e414cdfeb0f99720f8896e2d6ccba56`.
The private extractor verified the mapped type/offset, local-relocation
membership and array bounds before reading question and answer records.

The record has **45 questions and 193 answer entries**, with 190 distinct
answer bytes. Question indices are 1–36 and 181–189; all are unique.
Answer bytes 187, 188 and 189 each occur under both questions 188 and 189.
They therefore cannot be treated as globally unique content identifiers.

Applying the established flags predicate to every race/profession combination:

- Each of the 45 combinations has exactly **five applicable questions**.
- Each applicable question has **3–6 eligible answers** after answer filtering.
- No answer byte collides between eligible questions in any one combination.
- Selecting the first eligible answer per question and sorting the five bytes
  produces five distinct values for all 45 combinations. This is a deterministic
  analysis choice, not the client's random-selection policy.

The question-only initializer can encounter answers excluded by the separate
answer predicate: across all applicable question/combination pairs there are
44 such occurrences. This confirms that the two previously traced selection
paths are not interchangeable for this content. It does not establish which
path is last applied by the full creation UI or what the server accepts.

Extracted records and eligibility sets are private in
`~/re/gw2/notes/creation-answers-209026.json` (53,257 bytes, mode 0600),
SHA-256 `4d7a79e22d8d8954d56c1edb8dc3f686f7565d04c807599aff7d1a37ad453994`.
No answer fixture was sent to the client and no runtime creation was tested.

## Evidence

Private export: `~/re/gw2/notes/209026-creation-answers-static-1.json`.
Archive findings: `~/re/gw2/notes/209026-creation-answer-content-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_fb7d3fddc7c2d784843da0f612d72584ff87212f69bdb3fdaed40f4046f6ea27` | Content retrieval, question iteration, random default selection and bulk byte setter. |
| `ev_775f6d3f94e9e43a010ce25736d0269c5f8172aca8d6b8ec7aeceea34afce368` | Applicability predicate and creation vtable pointer reference. |
| `ev_c6772b65ff604f0981b7f58be89b4026b70f5e7701b4da6c48df848cbdeb9618` | Vtable slots, profession bit jump table and madLib assertion. |
| `ev_fa5be1564bf34f33087c4a02828d60652294b54716ef8f7400eceede32b1ff4e` | Answer replacement, question bitset, sorted insertion search, filtered selection and race reset. |
| `ev_e9f9a114ae887363e88bd14667eb9fe0cd5143f2c1cf7bd2c3bde5f7956d65b0` | Byte insertion implementation, request bridge and question assertion. |
| `ev_7efc6ab871c47a6eee891ec860213ea729b69aa5fb7ba4c020a7b9c676bc7bd0` | Type iterator and cursor-zero creation lookup. |
| `ev_fea1a9d83933cecdd63cbc0800e8c111a4877918b79812a330d2399c2fc717dc` | All 32 base containers; one indexed type-260 record. |
| `ev_417ae2d8a22e463ccdbc6081afbd1eb4e2da36cfd4a9f2b25951df09aab13ed6` | Bounded question/answer extraction and eligibility analysis. |
| `ev_b31a55a9c7e671548f0a03aa9dfdad6669dc9e2a6e8fa7b46d8e850bb2ebc38c` | Reused bytes, no co-applicable collisions and 45 deterministic selection checks. |
