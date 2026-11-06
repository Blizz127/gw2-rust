# Build 209026: exact Mesmer sword animation records

2026-11-06. Frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This reads the three already established sword skills and their exact animation
tokens from retained, hash-checked content and animation caches. No archive,
game runtime, shared container, roster or publisher was accessed. No gameplay
schedule or packet sequence was changed.

**Variant zero is selected by these three ability records, not assumed from
the character's race.** All 68 available variant rows for the three tokens
were checked independently with the existing Rust reader. This closes the
action-data lookup; it does not establish a server hit moment or a Norn male
visual-motion variant.

## Exact selector and the Norn male boundary

The complete `0x14129a4d0..0x14129a662` receives a SkillDef reference in RCX
and a selector in EDX. It requires SkillDef kind +0x58 to be zero and obtains
the ability body through +0x60. It calls the animation context's virtual +0x30
with the following pair:

- When the selector is nonzero and body +8 is nonzero: token from body +8,
  variant from body +0x10.
- Otherwise: token from body +0, variant from body +0x18. The actual loads
  include `0x14129a577` and `0x14129a588`.

The function has no race or sex argument or read. A missing exact pair returns
null; it does not retry variant zero. Its diagnostic names
`GetSkillAbilityAnimationData`, `CharSkill.cpp` and the missing token/variant.
The [previous context trace](209026-mesmer-heal-animation-data.md) binds this
virtual call to the exact two-key binary search over `anic` / `seqn` v0.

The three definitions in retained container 1282860 are:

| Skill | Definition offset | Body offset | Primary token | Alternate token | Primary / alternate variant | Body Dword +0x70 |
| --- | ---: | ---: | --- | ---: | --- | ---: |
| 10170 | 3965608 | 3965984 | `0x1789af184f7` | 0 | 0 / 0 | 0 |
| 10171 | 4263688 | 4264064 | `0x2789af184f7` | 0 | 0 / 0 | 0 |
| 10172 | 4032088 | 4032432 | `0x3789af184f7` | 0 | 0 / 0 | 0 |

Consequently, either selector value uses the primary pair for these unchanged
records. For a Norn male using these same definitions, this particular action
lookup does not introduce a race-specific variant. That is a source-derived
conditional statement, not a runtime selection observation. The other opaque
variant tokens have not been labeled as races, sexes or game modes. Selection
of the model's actual motion/skeleton resources is outside this function.

The source context condition chooses pack 184788 when game-context +0x310 is
zero, otherwise 184787. Only 184788 was present in the previously checked
frozen index. This task reused that cached pack; it did not observe the
predicate in a running Norn character or replace the missing alternative.

## Selected rows and preserved raw events

Offsets below are relative to the `seqn` payload, which starts at file offset
28. It has 1853 primary rows. The selected tokens have 27, 20 and 21 variants;
all exact variant keys and decoded events/segments are retained privately.

| Skill | Primary row index / offset | Available variants | Variant-zero offset | Event / segment counts |
| --- | --- | ---: | ---: | --- |
| 10170 | 785 / 15716 | 27 | 306160 | 10 / 2 |
| 10171 | 809 / 16196 | 20 | 311592 | 8 / 4 |
| 10172 | 822 / 16456 | 21 | 315848 | 6 / 2 |

Nine-byte event tuples retain **`(kind, raw Dword +1, raw Dword +5)`** and
their authored order:

- 10170: `(7,120,0)`, `(4,125,0)`, `(1,125,0)`, `(3,240,2)`,
  `(9,400,0)`, `(5,475,0)`, `(5,575,0)`, `(5,675,0)`, `(5,775,0)`,
  `(0,900,0)`.
- 10171: `(1,225,0)`, `(7,275,0)`, `(3,400,2)`, `(9,475,0)`,
  `(5,650,0)`, `(5,775,0)`, `(0,800,0)`, `(5,875,0)`.
- 10172: `(7,400,0)`, `(1,400,0)`, `(3,480,0)`, `(9,600,0)`,
  `(2,600,0)`, `(0,1200,0)`.

Every selected segment has kind byte +0 = 0 and flags byte +5 = 0. In order,
the first Dwords through segment pointer +9 are `[125,1305]`,
`[225,300,125,630]` and `[350,1450]`. All selected segment pointers +0x11 are
null. These pointed first Dwords are not an interpretation of the remainder
of the segment data.

The [complete action initializer](209026-mesmer-heal-animation-data.md#what-the-typed-action-initializer-actually-derives)
uses the first type-0 event quantity to override action +0x10. Applied to
these records, that yields 900, 800 and 1200. The type-3 quantities are 240,
400 and 480. These quantities are distinct; neither should silently become
an authoritative server duration or damage time. The two raw +5 values of 2
remain raw and are not labeled attack counts. Type 7/9 consumer semantics
were not expanded in this slice.

The [ordinary event-consumer trace](209026-skill10176-animation-event-consumers.md)
establishes that the shared timed loop divides event quantity by the current
character timing-scale getter, converts it to the local clock and can bypass
the future-time comparison on forced completion. Its demonstrated type-1,
type-2 and type-3..5 ordinary-view subscribers select/clear visual effects or
change view state. This is not a census of all subscribers and supplies no
source-backed authoritative damage transaction. Actual selected order state,
step transitions, timing modifiers, impact/miss outcomes and server hit timing
remain separate dependencies.

## Bounded reader comparison and retained evidence

The independent Python extractor binary-searches only these three primary
tokens, then reads their bounded variant tables and selected arrays. The
unchanged [`AnimationSequences209026`](../../crates/gw2-dat/src/animation_sequence.rs)
reader independently selects all 68 exact pairs. Every decoded event and
segment field agrees: **416 events and 155 segments**. Another 137 assertions
check too-small primary, per-token variant and per-row combined-item limits.
No production module was modified.

The compiled private checker was built with:

```text
cargo build --manifest-path /home/blizz/re/gw2/notes/sword-animation-sequence-check-20261106/Cargo.toml --target-dir /home/blizz/re/gw2/notes/sword-animation-sequence-check-20261106-target --offline
```

It and the independent extractor ran under REA through the hash-checking
wrapper `~/re/gw2/notes/check-sword-animation-sequence-20261106.py`, SHA-256
`e17fce4705ad2fc633367a18b84beb1205179db88d71efa7be4205828f1b37b4`.
Final comparison evidence
`ev_f78484b3da02f27332e372d8f0c75f541bc844f8f26570858ebc76319b616351`
exited zero, untruncated, with owned process cleanup verified. It verifies
all pinned inputs and outputs before and after the comparison.

| Private artifact | SHA-256 |
| --- | --- |
| `content-type-147-file-1282860.pf` | `05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23` |
| `animation-pack-184788-209026.pf` (8,916,756 bytes) | `72e5fce4c2098946d1644469881fcc43795349ea7e16c8a357b3c5594c35723e` |
| `209026-mesmer-sword-animation-rows-exact-20261106.json` | `152aec110f5047a0e835626ba85643b40a260e679860ae5e0792115fff3bea9a` |
| `read-sword-animation-rows-20261106.py` | `994fd20d4f3a566097553d2f8b09773562e4293be10aa911e4f6bd1bc9f2a281` |
| `sword-animation-sequence-check-20261106/src/main.rs` | `88f3d6973c9c7aeb27437cd4aa82838a201aa2b93a7125eab73127a6a00ea0da` |
| Compiled private checker | `fec88b1f20ade451e6a77630311f421cd7780dc4b70faa5bddad93e02299b839` |
| Unchanged production reader module | `db63c62ca575a245231e7e4eb68cc4f9aa4bcb17460dd754c266a013a8a95161` |

Complete five-capture bundle:
`~/re/gw2/notes/209026-mesmer-sword-animation-data-20261106.json`,
SHA-256 `fd32739c54c1f17fb22d2f0a202e4f24b72d0626a9f1f3e38ac84fccb85efe38`.
Selector evidence is
`ev_4df23fdc30859ca56b293c98e188442c96480660d83782a03ac6bfc152c523f4`;
complete raw extraction is
`ev_fd98ebee64f073c4f59e7518324491c11d57331b99d31516f58f764c7ebf5741`.
Two repeated rendered frames were omitted from that extraction's display,
but all raw and normalized output is retained.

The bundle also retains a failed comparison caused by an intermediate
JavaScript JSON copy rounding three opaque nonzero variant keys above its
exact-integer range. No selected variant-zero row changed. Python-only
integer-preserving regeneration
`ev_996d573470670fc177289630d90f372c4fc7db9943b0aaf0936272f1f2f7fcf6`
supersedes that copy; the exact suffixed JSON and final passing comparison
above are authoritative for these extracted rows.
