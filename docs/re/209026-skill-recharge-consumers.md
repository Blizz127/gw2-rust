# Build 209026 per-skill recharge consumers

This frozen-source slice narrows the raw server630 fields documented in
[the command-path note](209026-mesmer-skill-command-path.md#per-skill-recharge-is-a-separate-message-family).
Category **0** is the ordinary skill recharge queried by paths asserting
`hasSkillRecharge`. Category **1** maintains a replenishing available count:
the wire Word becomes the current count, expiry increments it toward a
skill-content limit, and a zero count blocks the inspected usability gate.
These descriptions follow the consumers; they are not recovered enum names.
The category-0 Word remains unresolved. No packet was sent and no gameplay
default was selected by this investigation.

The frozen PE SHA256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All native observations below use REA captures of hash-checked, bounded
objdump windows and exact E8/E9 target queries. No live client, archive,
container, configuration, or application code was changed.

## Record and content relationships

The object at skillbar +`0x150` has constructor `0x1412122d0` and vtable
`0x142169cf8`. Its first two virtual entries are deleting destructor
`0x141212500` and scheduled callback `0x141213b30`; the following table is a
different object and must not be included in this interface. The object
holds a callback/context pointer at +`0x28`, initialized null, and the
recharge-rate scale at +`0x30`, initialized `1.0f`.

Server630 reaches `0x141214030`. Its per-skill list entry has:

| Offset | Observed use |
| --- | --- |
| +`0x10` | Remaining timer value. |
| +`0x18` | Skill definition pointer. |
| +`0x20` | Total timer value. |
| +`0x24` | Category, widened from the wire Byte. |
| +`0x28` | Value widened from the wire Word. |
| +`0x2c` | Pause state; new entries start at zero. |

Lookup identity is the pair `(skill definition pointer, category)`, so
categories0 and1 can have independent records for the same skill. The
setter requires skill definition +`0x58` equal0 and body +`0x60` nonnull.
The named assertion in another consumer calls this body
`skillDef->GetAbility()`.

For category1, the maximum count is selected from the ability body:

* A context key comes from callback/context virtual +`0x18`, or fallback
  `0x14140f1d0` when that pointer is null.
* The array at body +`0x190`, count +`0x198`, has eight-byte rows containing
  `(context key, count)`. A matching row supplies its second Dword.
* Otherwise body +`0xf0` supplies the count. Helper `0x1412125a0` performs
  this lookup. This slice does not assign a name to the context key.

A zero selected limit makes category1's setter return before applying
either insertion/update or zero-remaining removal. Other categories do not
pass through this limit check.

## Update, expiry, and pause

The setter clamps incoming remaining to `min(remaining, total)`, advances
the existing timers, and updates or removes the matching record. Positive
remaining stores total, the incoming Word, and remaining. Existing pause
state is preserved; new entries have pause0. Zero clamped remaining removes
a matching record. This is not a generic replacement of all recharge state.

The setter also compares an old and new value for a callback at context
virtual +0. The arguments are skill reference, category, old value, and
new value. For a new category1 entry the old value is the content limit;
for category1 removal the new value is that limit. For a new non-category1
entry the old value starts at0. Existing records supply their stored Word
as the old value. Consequently a nonzero or changing category-0 Word can
reach a callback, even though it does not govern that category's expiry.
It is not established as ignored or padding.

Scheduled callback `0x141213b30` contains the named assertion
`m_rechargeList.Head() || m_slotRechargeList.Head()` in `CharSkillbar.cpp`.
Its expired per-skill branch at `0x141213d90` does the following:

| Category | Expired-record behavior |
| --- | --- |
| 0 | Removes the record. If context exists, invokes virtual +8 with the skill reference. The stored Word is not used by this expiry branch. |
| 1 | Reads the current Word/count and current content limit. If the limit is nonzero and count is less than limit minus1, increments count, resets remaining to total, and reinserts the same record. Otherwise removes the record and uses the limit as the resulting count. If old/new differ, invokes context virtual +0. |
| Other | Removes the record in this branch, without the category-0 completion callback or category-1 increment. This does not establish the entire raw category domain. |

The category1 comparison is unsigned. The setter itself does not clamp the
incoming count to its content limit. These client mechanics are not a
recommendation to send an out-of-range count.

Server631 reaches `0x141214470` and finds the same `(skill, category)` pair.
It compares the incoming selector with the stored pause value, treats
nonzero as paused, and stores1 when pausing. Resumption stores0. The branch
also adjusts/reschedules timer state. It does not reinterpret the stored
Word or switch one category into another. See the existing
[rate-scale note](209026-character-skillbar.md#recharge-rate-scale) for
advancement and scheduling; no new time-unit/default inference is needed
for the category distinction.

## Consumers confirm available-count versus ordinary recharge

`0x141214b60` finds an entry by the pair and delegates to `0x1412149b0`.
The latter returns a query record containing adjusted remaining at +0,
skill pointer at +8, remaining at +`0x10`, total at +`0x14`, category at
+`0x18`, and the stored Word/count at +`0x1c`. It accounts for rate, pause,
and the existing `0xffffffff` remaining sentinel without changing the
stored record.

The exact direct callers establish two different uses:

* `0x1411f83b0` queries category0 and asserts `hasSkillRecharge` if absent.
  `0x1411f81f0` also queries category0, can combine a separate slot recharge,
  and asserts `hasSkillRecharge || hasSlotRecharge` if neither exists.
  The named source is `ChCliSkillbar.cpp`.
* The usability predicate `0x1411f6c20` first considers category0. A present
  category0 normally rejects this gate; an explicit ability-flag/elapsed
  exception exists (body +`0x74` bit`0x40`, elapsed at least125), so presence
  is not an unconditional global prohibition.
* If the content maximum count is positive, that predicate queries
  category1. A present record whose returned +`0x1c` count is zero rejects
  the gate. A positive count, or no category1 record, continues through
  other eligibility checks. Along with replenishment on expiry, this
  establishes the Word as an **available count for category1**, rather
  than a remaining-ticks value or an arbitrary second duration.

## Implementable boundary

The source supports distinguishing ordinary per-skill recharge (category0)
from count replenishment (category1) in diagnostics and higher-level logic.
Category1 requires an applicable nonzero content maximum, caller-selected
current count, and the timer values. It does not replace ordinary recharge;
both records can coexist. Keep the raw codecs lossless, as they already are.

This slice does **not** justify a default Word for category0, a recovered
name for the complete category enum, a default packet for skill10176, or
the complete cast/acknowledgement/heal chain. The context virtual +0/+8
callback contract is established at its call sites, but its concrete
binding and downstream category-0 Word handling remain unbound. The
inspected skillbar construction/registration paths did not establish that
binding; this is a bounded result, not proof that no binding exists.
An exact callback-pointer assignment/implementation trace or a separately
authorized known-good message observation is the remaining boundary for
choosing the category-0 Word. No gameplay packet recipe is claimed here.

## Retained evidence

Private complete eleven-record bundle:
`/home/blizz/re/gw2/notes/209026-skill-recharge-consumers-20261106.json`,
SHA256 `14fcac342d5551c95efafa3c67dfb75baa3eb9eaf93928ce8bed6f6702f726ee`.
All captures exited0, report nontruncated raw output and verified owned
process-group cleanup. Some report inaccessible unrelated `/proc` entries;
they do not establish ownership or cleanup of those other processes.
The complete captures remain private rather than copying binary output
into this repository.

| Evidence | Relevant scope |
| --- | --- |
| `ev_1fbf5fe8eefc87f4c2f5ab8f74dc07c3b69ea1346af50cb3d52652740f955511` | Complete630/631 setters, timer advancement, recharge virtual table. |
| `ev_b4410f67a5842898a61e5f67cd3d2fae71c4d8b00b5f3b046b28a08807449879` | Complete scheduled expiry; context-limit lookup. |
| `ev_ab5bb7e43ba6622a57d407859da2df9604865f1a0bfcc4217a3a6aed8fb3f6cc` | Query output fields and pair lookup; ability assertion. |
| `ev_a7dae5e7a3e80d46d86cf5efff268da606230e6eb53c0dbe9ec4e6e35dcb7254` | Seven exact E8/E9 query-call candidates, each instruction-validated in its containing `.pdata` chunk. Indirect/conditional references are outside this query's coverage. |
| `ev_a110bb9bd74d0e71f6f7f46f4bb0c3d7098d81e1ed62c3397fb1aa36f30cccef` | Category0 named assertions, category1 zero-count usability gate, query forwarding. |

Some exploratory windows include an unrelated adjacent function or end
mid-instruction. Claims above rely on the complete aligned bodies/calls
identified explicitly, not those cut boundaries or inferred table extent.

## Bounded context-binding follow-up

A separate frozen-source follow-up did **not** establish a concrete
recharge context constructor, context vtable, or virtual +0 target. It
therefore does not tighten the category-0 Word's value domain or justify
a default of0 or1. The following narrows where the binding has been sought:

* The complete shared-object constructor `0x141277790` creates the six
  recharge objects with `0x1412122d0`, stores their pointers in its array
  at +`0x120`, and increments each reference count at +`0x20`. It does not
  install a recharge context after construction. Its own +`0x28` is a
  different field and must not be confused with recharge object +`0x28`.
* Getter `0x141278320` selects one array entry using the owner's +`0x100`
  index and +`0x12c` count. It returns the pointer without changing it.
  The exact E8/E9 target query found only the known skillbar-constructor
  call at `0x1411f4bb6`; this excludes indirect and other branch encodings
  from the query's coverage.
* A bounded listing of +`0x150` references in `0x1411f4720..0x1411fc000`
  returned22 syntactic hits. Some are ability-body fields rather than the
  skillbar field. The validated recharge flows lead to construction,
  reference counting, queries, rate/record/pause setters, and the separate
  slot recharge setter. No context assignment was established there.
  The corresponding shared-owner +`0x120` listing in
  `0x141277790..0x141279600` returned7 hits, covering construction,
  destruction, and the getter, without such an assignment.
* An exact search for the five-byte leaf sequence
  `mov qword [rcx+0x28],rdx; ret` was used only after these typed owner
  paths failed to provide a binding. Five matches were found. Two are
  interior linked-list writes; two belong to increment/decrement methods
  referenced by the separate table at `0x1423b5688/90`. The standalone
  method `0x141018190` has eight exact direct callers. Several construct
  their receiver with `0x141016ee0`, whose named source is `Rad.cpp` and
  assertion is `m_keyDefs`, with a different layout. None of the inspected
  callers or table references demonstrated a recharge object. This exact
  sequence query is not an exhaustive search for all possible writers.

A recharge-named metadata lookup also found
`FLAG_AMMUNITION_RECHARGE_IS_ACTIVE` in `SkillImage.cpp` and
`m_slotRecharges.Count() == 0` in `ClmClaimableGuild.cpp`. Their validated
references did not establish this context interface. The mere presence of
those names must not be used to bind an unrelated interface or enum.

The positive construction fact is **context starts null**. The negative
result is **no binding found in this bounded slice**, not that the client
can never assign it or that all category-0 Word values are interchangeable.
If context is null at the setter call, its notification branch is skipped;
the Word is still stored and exposed by the query record. An actual
non-null context identity or a demonstrated pointer assignment is needed
to resolve the virtual target. No such identity was invented, and no
runtime inspection or packet experiment was performed.

Separate eighteen-record bundle:
`/home/blizz/re/gw2/notes/209026-skill-recharge-context-binding-20261106.json`,
SHA256 `c44197843ab02c1eb5d4e74ca7c7aeec4e2f15c12cb7a9a34d8f890d9c6e422e`.
All captures exited0 with nontruncated raw output and verified owned
process-group cleanup; the same unrelated `/proc` visibility limitation
applies. The previous eleven-record bundle is unchanged.

| Evidence | Scope |
| --- | --- |
| `ev_ca5fb78c6574524a0692141473a02e7450ca08d896d0ffafee7b6c00e90856cf` | Complete shared-object construction. |
| `ev_08a7c90a6a520af32b854910411885697294b7ebe33df6ec9d911418876c9fa1`, `ev_b66a0ed782c068427751d4aa7861a99bdba069d2603988a9351a9b415637dbf9` | Bounded owner-field reference listings. |
| `ev_0175866f2f5a9cee82b447a99551b3ba7cd493f0987bd9974242e5f77c2613e4`, `ev_dc08ed825b00a43686d777f127d2ba059e0b25644a5668468cdaeb5ba97e2ef3` | Complete shared getter and exact direct reference. |
| `ev_09e7b3d41327a73b391fd33a872e2a337997f4aa467659a20ad5e95ecd7de60d`, `ev_88d19293ed3669b462a566c643648d980be7a9aa2e7e576cf8706b1e3390e7ca`, `ev_331d3c0bc378ed506a288899a028029ac1922b0f835996eea252961d31e440ec` | Exact setter candidates, bounded direct callers, distinct named receiver construction. |
