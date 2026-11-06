# Build 209026: authored step identity and the remaining encounter rule

Static REA, 2026-11-06. **No retail actor, placement or action-to-step rule
was established.** This bounded slice found a concrete additional identity
path: the quest manager indexes the first 16 bytes of authored step definitions
and exposes an exact-key membership query. That path supplies a precise next
consumer seed; it does not itself explain what action completes a step.
No gameplay code, archive, client runtime, working container, publisher state
or resource hold changed.

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
