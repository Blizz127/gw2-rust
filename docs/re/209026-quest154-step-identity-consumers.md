# Build 209026: authored step identity and the remaining encounter rule

Static REA, 2026-11-06. **No retail actor, placement or action-to-step rule
was established.** This bounded slice found a concrete additional identity
path: the quest manager indexes the first 16 bytes of authored step definitions
and exposes an exact-key membership query. That path supplies a precise next
consumer seed; it does not itself explain what action completes a step.
No gameplay code, archive, client runtime, working container, publisher state
or resource hold changed.

The later [typed-caller follow-up](#typed-caller-follow-up-fixed-map-navigation-rows)
binds this membership query to fixed map-navigation rows. That exact table
contains neither map379 rows nor any of quest154's six step keys, so it does
not provide the missing Great Hunt actor/action rule.

## Selected gap and bounded result

The [step content note](209026-quest154-content.md) leaves definition +0x28
unnamed, with raw value **17** on all six Great Hunt steps. This was selected
instead of repeating the all-container species/name/relocation searches,
mission-point caller searches, or unnamed story +0x80 work. The inspected
diagnostic, active-step and navigation consumers did not establish a typed
read of that field. **17 remains unbound**: it is not identified as an action
kind, objective kind, species key, count, location type or trigger.

The complete active-step branch at **0x1412029c6..0x141202a65**, within
refresh helper **0x141202810..0x141202bab**, obtains each runtime step through
quest virtual+0x20 and its definition through step virtual+0. After testing
completion, runtime bit1 and `IsNoUi()`, it checks definition flags +0x10
mask **4**. A matching step takes the helper's false-return path before its
later marker creation. Quest154's six definition flags, **0,2,2,2,2,2**, do
not select that particular mask4 branch. This is a display/navigation guard,
not a recovered action rule or authority to supply marker coordinates.

## Exact authored-key collection

**0x1411fdde0..0x1411fdf73** receives a quest definition and iterates its
goal array at +0xa8/count+0xb0, stride0x70. For each goal it iterates steps
at +0x50/count+0x58, stride0x60. The step-definition pointer is passed to
hash helper **0x1402f95a0** at **0x1411fdeae**. The containing collection
is the manager's **+0x140**.

At **0x1411fdf1e..0x1411fdf21**, insertion copies exactly the step's first
16 bytes into an entry. Entries have stride **20**, with the four-Dword key
at +0 and hash/occupancy Dword at +0x10. The complete hash/lookup body
**0x1402f95a0..0x1402f96be** hashes 16 bytes and, after a matching hash,
compares all four Dwords at **0x1402f9677..0x1402f969b**. It does not use
the numeric step ID at +0x14 or raw +0x28 value for key equality.

The manager constructor at **0x1411fd630** installs primary table
**0x142166030** at **0x1411fd64c..0x1411fd653**. The exact pointer at table
entry **0x142166098**, primary virtual **+0x68**, is **0x1411ffd10**.
Its complete body **0x1411ffd10..0x1411ffd52**:

1. Returns false when the collection storage at manager +0x148 is null.
2. Otherwise passes the caller's key pointer to `0x1402f95a0`.
3. Returns whether the selected entry's +0x10 is nonzero.

The established 542 path also passes an exact step-definition pointer and
manager+0x140 to helper **0x1402fab80** at **0x1412019aa..0x1412019b4**.
The established 546 path passes the same kinds of arguments to
**0x1411fd550** at **0x141201db9..0x141201dc3**. This slice records those
concrete set-helper edges without assigning additional lifecycle meaning to
their untraced bodies or naming the whole collection.

These are three distinct identities:

| Identity | Established consumer |
| --- | --- |
| Definition first16 bytes | Manager+0x140 exact-key set and virtual+0x68 membership query |
| Definition Dword +0x14 | Server step-state operations select the active step by numeric ID |
| Runtime step Dword +0x14 | Separate runtime objective registry/HUD binding |

Neither membership nor equality of an unrelated numeric value establishes
an NPC association. The membership query contains no spawn, condition
evaluation, step-completion call or network send in the inspected body.

## Exact quest154 keys and next source boundary

The cache-only reader verified container1282830 SHA-256
**e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f**,
quest154's sole goal1, its six-step array, and the known numeric IDs/flags.
It disabled archive fallback. The authored identity bytes, in memory order,
are:

| Step ID | Content offset | First16 bytes, hexadecimal |
| ---: | ---: | --- |
| 1 | 4146312 | `b4dbe79a4ac3ce4ebd4b2c61b21c6fa3` |
| 2 | 4146408 | `b61f9e509c458a4bbed5186ca91a8e90` |
| 3 | 4146504 | `99ed0b80c4b2a14c9791c3b5da9c3806` |
| 6 | 4146600 | `f01e15388856224aae6cbc90be2dbef3` |
| 4 | 4146696 | `a77d63c80df79b4aa66187d3c979ac1d` |
| 5 | 4146792 | `ccedf9d41ca33b449b7474c496b37e54` |

This order remains the authored array order, not a recovered action sequence.
The next precise consumer seed is **quest-manager primary virtual+0x68**,
with a receiver proven to use table **0x142166030**, and its incoming
16-byte key argument. A caller that supplies a particular marker/actor
definition's field could establish another concrete join. No such caller
receiver/argument binding was recovered in this slice; a generic +0x68 call
elsewhere is not that proof. The exact step1 key above is also a specific
future reference seed, without requiring another speculative name scan.

An implementable retail transition still needs the authored or authentic
**action/actor/condition → step ID** rule, and any required runtime objective
state. The investigated paths instead consume known step state, authored
identity, or navigation positions. This is not proof that every shipped
asset lacks an encounter rule, nor proof that the rule must be server-only.
No Great Hunt actor, trigger, timer, count or coordinate should be selected
from this result alone.

## Evidence and coverage

All executable reads asserted frozen SHA-256
**99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621**.
The field-candidate windows were limited to the known ChCliQuest methods;
they are candidate discovery, not receiver proof for every matching operand.
Exact typed-name string queries and content-type metadata checks did not
identify +0x28 semantics. No global offset scan, all-container content scan,
archive extraction or game execution was performed.

The initial `.pdata` queries returned fragments for several functions.
Explicit continuation captures below supply the interpreted complete builder,
membership and hash/lookup bodies. Other neighboring partial functions and
misaligned leading/trailing bytes in bounded windows are uninterpreted.
Complete raw output from all **15** successful captures is retained; several
captures omitted cumulative rendered-terminal frames within display budgets.
REA verified owned helper-process cleanup. These limitations do not establish
runtime state or exhaustive indirect-caller coverage.

Private bundle:
`~/re/gw2/notes/209026-quest154-step-identity-consumers-20261106.json`,
**565731 bytes**, SHA-256
**f97f87f9303a5819626a8fa0a7a8b41c4c1af6b71a4ce548a708063a3e0c1258**.

| REA evidence | Established scope |
| --- | --- |
| `ev_bb62db6698754721a0e9928a0f3b78a3605e21a3344c74434846186f89e82cd8` | Complete goal/step iteration and first16-byte set insertion. |
| `ev_c21b896d62b382f9493bd4d2b7dff4bdf301748bd0b83684c46429b69b5e7a04` | Complete leading refresh helper, typed active-step branch and definition-mask4 guard. |
| `ev_983d34fb1c4752983e3118991521477d11bd5ffce8f89f3cc047ed2822fa7f5f` | Class-local set-reference candidates, including 542/546 helper arguments. |
| `ev_191713e0274c1b820ad5bdac077ead4e7c5833c060d1b2eaaadf1b6b0217b47b` | Constructor table installation, exact table-entry pointer and complete membership query. |
| `ev_5c5067155b5acf17231ec684470b3750ae341e0b28e8167e6645ee96fa7208b5` | Complete 16-byte hashing and four-Dword equality path. |
| `ev_255f6a7c651218e3bc396212ef1ccc8cfef14b21778ba735d40d0b6075b1592c` | Hash-verified cached quest154/goal1 step identities, numeric IDs, flags and raw17 values. |

## Typed-caller follow-up: fixed map-navigation rows

Static REA, 2026-11-06. The membership consumer is now bound through a
second player getter. Player primary table **0x14215e9d8**, virtual
**+0x350**, points to **0x1411bb350..0x1411bb38b**. Like +0x358's
`0x1411bb310`, it checks `IsQuestMgrReported()` through player virtual+0x168
and returns **player+0x9820**. Consequently its returned primary manager
uses the already-established table **0x142166030**; this is not a generic
same-offset virtual-call inference.

In `ChCliContentDirection.cpp`, the loop at
**0x141269515..0x141269603** consumes 48-byte rows, with RBX equal to
row+0x20. It tests **row byte+5 mask2**. When that bit is set, it obtains
the player quest manager through +0x350 and calls its exact membership
virtual+0x68 at **0x141269557**, passing **row+8** as the 16-byte key.
A false result skips the row. With the bit clear, the loop bypasses this
membership test. Passing rows undergo a numeric content lookup from their
Word+2; when the corresponding local map-info entry exists, the loop
converts signed Dwords at **row+0x18/+0x1c/+0x20** to floats and appends
them to that entry's coordinate collection. No spawn or action-completion
semantics were recovered from this loop.

The complete predicate **0x14126efb0..0x14126f017** implements the same
flag/key decision. Its receiver's +8 supplies the player; its RDX argument
is the row. For mask2 it tail-calls manager+0x68 with row+8, otherwise it
returns 1. The exact pointer at **0x142172a28** is this predicate. The
containing content-direction method builds a local callback object with
that table and the player at **0x1412694ce..0x1412694d9**, then passes its
address as the fifth argument to **0x141411bb0** at
**0x14126974f..0x14126975b**. The routing helper's internals were not
expanded in this slice.

### Exact row source and Norn result

The enclosing method starts at **0x141268ef0**, established through the
chained unwind entry for `0x1412694e5`. At
**0x141268f8e..0x141268fc0** it obtains the current map definition from
**0x141411100** (global context+0x1c8, then mission+0x1e8), asserts
`localMap`, and retains its numeric **Dword+0x28**. At
**0x1412694dd..0x1412694e5** that map key is passed to
**0x1412e0d00**, with an output row-count pointer.

The complete selector **0x1412e0d00..0x1412e0dee** searches the fixed
executable array **[0x1421f2310, 0x14220c110)**. It contains exactly
**2,208 rows × 0x30 bytes**. Selection compares the input map key with
each row's **Word+0**, returns the first matching row plus the matching
count, or null/count0. This source is executable-resident `ConstMap`
data, not a relocated content-container array.

The exact bounded table audit found:

| Check | Result |
| --- | --- |
| Word+0 equals tutorial map379 | **0 rows** |
| Word+0 equals open map28 | **29 rows**, indices626–654 |
| Key16 in those 29 rows | All zero; byte+5 mask2 clear |
| Any of quest154's six exact step keys anywhere in this table | **0 matches** |

This is a concrete navigation eligibility consumer of the authored-key
collection, but **not a recovered Great Hunt actor, placement or action-to-step
rule**. In particular the map28 coordinates are not tutorial379 spawn
evidence. The negative result covers this exact fixed table, not all
indirect manager callers or all client content. The separate definition
**+0x28 value17 remains unnamed**. Authentic actor/condition-to-step rules
and any required runtime objective state are still missing; this result
does not prove they are exclusively server-side.

### Follow-up evidence and limits

The frozen PE hash above was asserted on every capture. Initial caller
discovery around player +0x358 produced unrelated or active-quest +0x68
calls; those were rejected by receiver tracking. An `InsCliEntrance.cpp`
cache at object+0x88 was also established, but the inspected class-local
uses only registered/unregistered its manager listener and obtained the
active quest. No membership or actor binding is claimed for those candidates.

Private full Evidence bundle:
`~/re/gw2/notes/209026-quest154-step-membership-callers-20261106.json`,
**745456 bytes**, SHA-256
**b85b04d21b8f26ba618328fc3b83c7e8adbfd1e3df2ef20d58c60c5362a2fd8c**.
It preserves **19 captures**, including one deliberately failed output-bound
assertion. An early MOV/getter candidate capture had incomplete raw output;
it is not used for an exhaustive caller claim. All final typed getter,
predicate, row-source and exact-table results have complete raw output;
some cumulative rendered-terminal frames were omitted. REA verified owned
helper-process cleanup. Neighboring disassembly outside the specified
interpreted windows remains uninterpreted. No archive, runtime or publisher
operation was performed, and no hold changed.

| REA evidence | Established scope |
| --- | --- |
| `ev_84beb173689c3c350763c7c864c9ae3f29a9762910e759c91d948be71195552d` | Exact player +0x350/+0x358 and manager+0x68 table pointers. |
| `ev_0440a2323e77d4917bfc708b84ffb5d7a956356f102675384433652d424d03cf` | Complete +0x350 getter and typed row membership/coordinate loop. |
| `ev_694d7dba1ef758a13736fd09c9b6908b14810b8ae9d16daca0ab3cfd0c3fc086` | Chained unwind to enclosing method; localMap key source. |
| `ev_29ffe682ed7b9db8482bdebc0ba29389b8ca5a1a673ba9dcd0baaa1265bde6c6` | Exact row-selector call, callback-object construction and downstream argument. |
| `ev_e1cb1cac11910c90f62ff31d0a954b5075c182ec3d3578ce5776508584b00bdc` | Complete fixed-table selector and source bounds. |
| `ev_db3854055bc79bdd62e78e923b44a304572f4e42ff2701917839cf0553007ab3` | Exact map28/map379 rows and all six quest154 key comparisons within the fixed table. |
| `ev_5c3a764fffec9a3aef5aeefc91e417391016acfa800ce89951c667e6f9a1fc0c` | Exact callback-table pointer, complete predicate and localMap/mapInfo assertions. |

A separate exact-six-key query covered the raw spans of every mapped frozen
PE section and found no literal16-byte occurrence of any quest154 step key.
This closes that specific embedded-key lead; transformed keys, runtime state,
other files and other content references are outside its coverage. Evidence
`ev_8b8d06eee735418dfba416c0d71c0fa27194a0ac793021ceba9250c7e4f19ef5`
is retained in the private starter-base-attributes bundle linked from
[that note](209026-starter-base-attributes.md). The capture exited0 with
complete raw output and verified owned cleanup; no actor rule follows.
