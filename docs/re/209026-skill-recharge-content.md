# Build 209026 ability recharge content reader

The bounded reader in
[`skill_recharge_content.rs`](../../crates/gw2-dat/src/skill_recharge_content.rs)
reads an ability's base recharge, category-one maximum count, and ordered
context overrides directly from classified content references. It provides
the content inputs needed to replace hard-coded skill values. It does not
apply modifiers, choose server timer fields or establish the category-zero
wire Word.

## Exact source selection

The type64 skill's +0x58 discriminator must be zero. Its +0x60 reference
then supplies the ability body. The source consumers establish:

| Content field | Meaning in the inspected consumer |
| --- | --- |
| Body +0x90 | Base recharge content value, before the recharge-display modifier. |
| Body +0x160 / count +0x168 | Ordered16-byte override rows: context Dword +0 and recharge Dword +8. |
| Body +0xf0 | Base maximum count used by the category-one recharge consumer. |
| Body +0x190 / count +0x198 | Ordered8-byte override rows: context Dword +0 and count Dword +4. |

The complete recharge-widget update `0x14089a0f0..0x14089a363` compares
context keys at `0x14089a1a0` and takes the **first** matching row. Its
match path at `0x14089a234` reads row +8; no match reads body +0x90.
Only after this selection does it apply the separate modifier, numeric
conversion and division by1000 for the displayed seconds. This reader stops
before those later operations.

The complete count helper `0x1412125a0..0x1412125ce` uses the same first-match
rule over8-byte rows, reads row +4 on match and body +0xf0 otherwise. Neither
lookup sorts or deduplicates the array. A matching zero is a real override,
not a reason to fall back to the base. The exact context is a caller-supplied
raw Dword; no general mode-name mapping is introduced.

These details extend the existing
[recharge consumers](209026-skill-recharge-consumers.md) and
[tooltip selection](209026-mesmer-tooltip-facts.md) contracts. The new REA
capture is `ev_766b1737e5bc5ef82979ab9e237bcc26f69f2851a33e505360a07eda7c2fbc34`.
Both claimed bodies are complete in retained raw output. Adjacent partial
functions printed around the count helper are not used as evidence.

## Reader boundaries

`SkillRechargeContent209026::read` takes an explicit ordered container set,
source container, skill offset and combined override-row limit. It rejects
missing/short records, a non-type64 definition, a non-ability discriminator,
null nonempty arrays, string/unclassified references, absent external
containers, out-of-range target extents and excessive combined counts.

After resolving an external ability body, the two array pointer slots are
interpreted in that **body's container**, rather than the original skill's
container. A classified local/external reference to offset zero remains
distinct from a null pointer. Array allocation follows checked resolution
and an explicit count budget. A zero-length array can have a null reference.

All consumed scalar values remain full-width unsigned Dwords, including
zero and the maximum value. The reader preserves override order. Selection
methods return the first matching value or the base without clamping,
inventing an override, loading an archive, selecting a gameplay context, or
emitting a packet. Other skill-body variants are explicitly unsupported.

## Exact Ether Feast record

An independent bounded Python traversal and the compiled Rust reader agree
on the cached frozen type64/key10176 record at offset4074696, body4074944,
in file1282860:

| Value | Observed frozen content |
| --- | --- |
| Base recharge |20000 |
| Recharge overrides, in order | Context2→25000; context3→25000 |
| Base available-count maximum |0 |
| Count overrides | Empty |

The reader returns20000 for explicit contexts0/1/4/u32::MAX and25000 for2/3.
The count maximum is0 in every case. The independent
[map gate](209026-skill-context-map-gate.md) establishes context1 once map379
is installed, selecting20000 before display modifiers. The bounded map28
follow-up below establishes the same selection for that installed definition.

The count result also narrows the pending implementation: the inspected
category-one setter returns without inserting/updating/removing a record when
the selected content maximum is zero. For this exact frozen ability there
is no count override to change that result. It does not replace the ordinary
category-zero cooldown path or choose its unresolved Word.

The cached file is9,571,548 bytes, SHA-256
`05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23`.
The actual-content check also verifies that a one-row budget rejects the
two-row record and that the source module and cached file remain unchanged.
REA: `ev_65573bab2b7e2b15a68eea18c27c15df2e493cb6d053ef1f98fda371773bdcd3`.
This is an offline reader check, not a native cooldown observation.

## Validation and retained evidence

Four focused tests pass with
`cargo test -p gw2-dat skill_recharge_content --lib`; the library build passes.
They cover duplicate-key first-match behavior, zero overrides and full raw
values, external skill-body/array ownership including classified offset zero,
null/empty distinctions, malformed types/references/extents and combined limits.
Public fixtures are hand-authored schema data; exact frozen bytes stay private.
An initial test-helper call signature error was corrected before these passes.

Private checker project and capture wrapper:

- `/home/blizz/re/gw2/notes/recharge-content-check-20261106/`
- `/home/blizz/re/gw2/notes/check-skill-recharge-content-20261106.py`

Checked module SHA-256:
`51c775ffe41e8765d7a656d4e47a6808c5e4e70477309ed36c6e9a541f01ab98`.
The private evidence bundle
`/home/blizz/re/gw2/notes/209026-skill-recharge-content-reader-20261106.json`
includes both new source/check captures among the retained session evidence;
SHA-256 `65922c91566ed5e8b9c4812cd4b82311f665184889e77d88355945e242966700`.
Both new captures exited zero with complete raw output and verified owned
process-group cleanup. No runtime, shared archive, deployment or saved-mode
activation was performed.

## Display modifier boundary

A bounded follow-up recovered the complete modifier at0x140898050. Query
0x76 returns a skill-filtered sum V and accepted-row count N. The native
single-precision sequence is T=0 when N=0, otherwise
T=V/(float(N)*100)-1, then factor=max(float(N)*T+1,0). When the raw caller
selector is12..18, a separate query0x74 supplies P and divides the factor by
P/100+1, with an assertion P>-100. Algebraic simplification can change
rounding. N counts accepted modifier rows, not clones.

The character adapter's iterator is0x14088e1d0, reached through character
virtual108, interface virtual40 and collection virtual28. Its actual contents
remain unknown; it is not the separately traced virtual50 buff collection.
An empty iterator would yield factor1, but emptiness has not been established
for this character. The widget forwards its incoming raw selector; this
slice does not prove that every caller supplies an equipped-slot index.

Consequently authored20000 cannot yet be advertised as an observed20-second
cooldown. Category-zero's Word, effective server duration and accepted-heal
timing remain unresolved. The positive-health delivery contract also leaves
the retail health-kind/Word pair and overheal policy unassigned.

Private findings: `ether-feast-next-execution-boundary-20261106.md`, SHA-256
`c20828e812ebf194394ecf843584a9a3ba0523924643f6b5338d849649047ecb`.
Complete seven-capture bundle:
`209026-ether-feast-recharge-modifier-20261106.json`, SHA-256
`57b51fbe31690733f5d95b91a35639ce0c0bbdcf23f51ba0e96ab725158afab1`.
All captures exited0. One duplicate rendered stream was truncated; raw and
normalized output remain complete. No gameplay or runtime change occurred.

## Category-zero operation with null callback context

The recharge receiver's callback context at +0x28 is initialized null. A
nonnull context is **not required** for the demonstrated category-zero timer
path. For positive remaining time, the complete setter
`0x141214030..0x14121446f` updates or creates the matching `(skill, category)`
record, stores total, the wire Word and clamped remaining time, and updates
scheduling before testing context at `0x141214421`. Zero remaining removes
the matching record before that test.
Null context skips only the optional change notification there. No received
state that must first install a context was established in the traced630
handler or its skillbar wrapper.

The query `0x1412149b0..0x141214a80` reads the timer, clock, rate and pause
state without a context dereference. It still returns the stored Word at
output+0x1c. Category-zero expiry unlinks and frees the expired record at
`0x141213f15..0x141213f45`, then tests context at `0x141213f4a`; null skips
the optional expiry callback. That expiry branch does not use the Word to
extend or count the timer.

This removes a presumed context-initialization prerequisite, not the unknown
Word policy. A nonnull context can receive changed old/new Word values, and
the Word remains queryable with null context. No default0 or equivalence of
all values follows. The bounded negative binding searches in the
[consumer note](209026-skill-recharge-consumers.md) do not prove that every
client receiver remains null for its entire lifetime.

## Installed map28 selection

The exact cached type45/key28 row is container1282830, offset3731704:
Dword+0x9c=0, signed map type+0x2c=5, and reference+0x100 null. The already
established gate `0x141411600` returns false because installed-map bit13 is
clear, regardless of the named setting's value. Character virtual+0x220
therefore returns sentinel4, and virtual+0x228 invokes `0x14140f1d0`.

A bounded reread of that complete fallback body establishes the exact type5
path: `0x14140f1e9` branches to `0x14140f1f0`; type18 does not match, and
null+0x100 branches at `0x14140f1ff` to the return1 at `0x14140f224`.
Thus the installed map28 definition selects context1 and10176's base20000.
This uses the recharge-context consumer itself, not an inference from the
separate named-variable map predicates. It does not establish pre-map state,
display modifiers, server timer fields or an effective cooldown.

## Run226 scope and retained follow-up evidence

The isolated skill diagnostic run226 observed an owned10176 request, typed78
start, completion with the slot still occupied, then correlated0x45 direct
release with the slot cleared. Its1000ms command duration and2000ms finish
delay were explicit diagnostic inputs. It sent no healing or recharge reply.
This closes the demonstrated order-release path, not a binding from authored
execute event1067 to server34's duration Word, a heal-success point, or a
cooldown-start policy. The recharge observer sampled null context at an
earlier control callback but recorded no630/query events and no terminal;
that partial coverage cannot establish context state at a later630.

Private findings:
`/home/blizz/re/gw2/notes/ether-feast-production-boundary-after226-20261106.md`,
SHA-256 `13c2ca070a776b274e9f9c08037730378b6a51dfeba2aee4ab3acb30b2bdcb54`.
The new retained-source review bundle is
`/home/blizz/re/gw2/notes/209026-ether-feast-production-boundary-20261106.json`,
SHA-256 `39e322341818b2ef6857ca01f3a08481d0029a8e41bb89603826b1395dd7f1ac`.

- `ev_655bd23c0c1fb6763eb5e6f44f4b565c128ae12df48a8f271c15056efb44642c`:
  retained complete setter, expiry branch and query reread.
- `ev_520b6788fca3833af54d6fad440ba345560ef269cf7d8b3d89bda5323914fa81`:
  complete retained recharge-context fallback reread.
- `ev_a276ec5e5a9e5ace5d0fe3e8f9e046e22b97afbb2a9a4e48bb48554186748709`:
  exact cached map28 fields, retained in the peer damage/chain evidence bundle.
- `ev_04202c443e966f38f2e411cae82b23bb855cbf16020ef1970d72d94c3fd420ef`:
  prior authorized run226 skill-finish readback, distinct from the publisher's
  separate payout run also numbered226.

The first source reread stopped at its explicit output bound; that failure is
retained and superseded by the narrower successful reread. Both successful
readers exited0 with owned-process cleanup verified. The older fallback
capture omitted duplicate rendered frames only; its raw and normalized
frames are complete. This follow-up performed no new runtime, archive access,
gameplay implementation or deployment.
