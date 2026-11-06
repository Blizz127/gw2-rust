# Build 209026: buff duration, modifier level input and UI count

Static client source connects message 735's raw value to the native buff UI's
duration calculation. It also separates the per-instance modifier level input,
active state and displayed instance count. This is a client-consumer contract;
it does not recover the server's application, level-selection or stacking policy.

The frozen executable SHA256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This extends the [explicit instance lifecycle](209026-buff-instance-lifecycle.md)
and [constructor-owned buff context](209026-character-buff-context.md).
All findings below are static. No client execution, packet delivery or gameplay
effect was observed in this slice.

| Input or state | Client use established here | Boundary |
| --- | --- | --- |
| 735 raw Dword, instance `B+0x40` | Duration added to the instance timestamp by the UI | Server-authoritative duration selection is not recovered. |
| Timestamp, `B+0x44` | Creation clock value; activation can backdate it | Default clock is milliseconds; an installed callback remains unobserved. |
| 735 raw byte 5, `B+0x48` | Unsigned modifier-interface scalar whose tooltip counterpart supplies level | This does not select a server level or establish a default. |
| 735 raw byte 6, `B+0x4c` | Per-instance `isActive`, tested for nonzero | The full byte is preserved, not restricted to 0/1 on the wire. |
| UI group `U+0x1c` | Count of per-ID UI rows; rendered for stacking types 0, 2 and 4 when above one | This is distinct from byte 5 and does not establish authoritative stacking rules. |

## Exact instance-to-UI path

Registered creation handler `1412c4f30` calls `1412c3a60`, which constructs the
120-byte instance through `1412c23a0..1412c2558`. The constructor installs
vtable `142184160` and stores the raw value, clock result and two bytes at the
offsets above. The existing instance ID remains at `B+0x18`.

The context's iterator, primary vtable `1421841a8` slot `+0x28`, is
`1412c33e0..1412c349b`. After its occupied-entry and visibility/state checks,
it returns `B+0x10`. The complete AsBuff refresh `1403b7ee0..1403b80f9`
obtains the combatant's buff context through virtual `+0x40`, calls that
iterator, and passes its result to `1403b3f90`. This establishes the argument
identity rather than relying on similar field layouts.

The complete population family `1403b3f90..1403b4352` appends a 12-byte UI
row. At `1403b4270..1403b42a5`, it copies:

| Source in returned `buffData = B+0x10` | UI row field |
| --- | --- |
| `+0x08`, instance `B+0x18` | `+0`: buff ID |
| `+0x34`, instance `B+0x44` | `+4`: start timestamp |
| `+0x30`, instance `B+0x40` | `+8`: duration |

The source locations are retained in
`ev_1a1166aca00404a9d8517f6f237f1669080c6fc64cab940f4440ab39e137242c`
and `ev_4d660cc326f29abecd6e6d4c475ab52ae61e17e009459ce9b7a403d5a57cc2af`.
The iterator and constructor are included in the full evidence bundle below.

## Clock, remaining time and active state

`1409cb4a0..1409cb4b5` first tests callback pointer `142682360`. If null, it
tails directly to `WINMM!timeGetTime`, whose result is milliseconds. If nonnull,
it dispatches to that callback instead. The frozen image contains zero at the
pointer, but its runtime installation and callback units were not observed.
The safe general contract is duration in the same clock units as the timestamp;
the default branch supplies milliseconds.

The complete UI getter `1403b57f0..1403b58c9` uses unsigned 32-bit arithmetic.
For a row with start `s`, duration `d` and current clock `t`, it first forms
`end = (s + d) mod 2^32`, then returns `end - t` if `end > t`, otherwise zero.
For authored stacking types 1 and 3, it uses the first row's remaining time and
adds each later row's full duration, also with 32-bit addition. Other types use
the maximum remaining time across rows. This is the display aggregation rule;
it neither removes an instance nor establishes a server expiry scheduler.

734's complete activation method `1412c39a0..1412c3a5a` asserts that the
instance was inactive. For incoming raw value `r`, it computes
`elapsed = max(B+0x40 - r, 0)` with unsigned comparisons, sets `B+0x4c` to 1,
and stores `max(clock - elapsed, 0)` at `B+0x44`. It leaves `B+0x40` unchanged.
Away from wrap and saturation boundaries, `r <= B+0x40` therefore describes
remaining duration at activation. The source formula, including its boundary
behavior, is stronger evidence than assigning a name from the packet shape.

The UI activation helper `1403b3e40..1403b3f8d` finds the row by buff ID and
copies the new timestamp. For stacking types 1 and 3, it swaps that row to the
front. Population calls this helper when `buffData+0x3c` is nonzero, directly
linking per-instance active state to this UI operation.

736 writes the supplied duration and clears the active state of an existing
active instance. 737 writes the supplied duration with its existing conditional
owner notification. The complete UI update body `1403b6890..1403b6978` matches
the buff ID, copies `buffData+0x30` to row duration and refreshes the group's
remaining time. This slice does not retrace every listener-registration link
from those two owner notifications to that callback.

Clock, timer and activation evidence respectively:
`ev_e5da9d7e20e94788c8f112f1ded1ab83e4ff9a7c1ce41c54a369be0148c9b1fa`,
`ev_2436fee1b2b239e80da161d8c21f699b6636ef00e8f3552f600fa660a2826a15`,
`ev_587f831e3a940ec9fbad1e883fb4f05c2cc0734da120fc89ed61af435e18358e`,
and `ev_5b8d2c1209c86a7eb1e562cf0e94f3ccdadb2c634c73e9b911c60df9f679c5dc`.

## Byte 5 and the separate UI count

Instance vtable slot 0 is `1403481d0`, returning Dword `B+0x48`. The named
buff-modifier evaluator `1412c1a40` takes a buff modifier, instance and manager;
several linear or quadratic branches consume that instance getter.

The typed tooltip counterpart makes the semantic link explicit. `140890080`
constructs a substitute instance interface with vtable `141b68af8`, stores its
current description context's virtual `+0x18` result at substitute `+0x14`, and
passes it to the same evaluator. Its slot-0 method `14088e190` returns that
field. For the concrete character description adapter, virtual `+0x18` is
`14088f140`, ultimately reading core-stat second-level Dword `+0xe0` through
`140282470`; see [the character context](209026-tooltip-character-context.md).

Thus byte 5 has a modifier level-input role by typed-interface equivalence.
This is not a recovered C++ field name or a rule selecting caster, target,
earned or effective level for a received instance. Its full unsigned byte
domain remains intact. This investigation did not enumerate all later writes
to that field.

Displayed count uses a different source. The exact switch at
`1403b4a47..1403b4a65` and table `1403b4c78..1403b4c8f` route stacking types
0, 2 and 4 to a branch reading UI group `+0x1c`; values above one are rendered.
That is the number of per-ID rows populated above. The retained buff738 record
has stacking type 4, so it selects this count path when displayed. No byte-5
stack-count interpretation or authoritative stack limit follows.

The complete modifier consumer is
`ev_9924eac92c89e92d7c651ec28461778d5c634a9321a03a3509ab2602aa845a4f`;
the substitute getter and count switch are
`ev_19c0013a5962aa4e2b9231cc45f35e19592514d3770f4bdbf7d12ee0204b3585`.
The retained tooltip constructor/evaluator call is
`ev_ed97265e026534217cfd2e905b353c920b40a319d26ee85e811af050b2509f9a`.

## Evidence coverage and remaining policy

Private findings:
`/home/blizz/re/gw2/notes/209026-buff-duration-level-findings-20261106.json`,
14,701 bytes, SHA256
`d1f5e401d98e36ffff1ef87400c27322b9e0c927edd1e3671601d172461b06cf`.
The complete 26-record bundle is
`/home/blizz/re/gw2/notes/209026-buff-duration-level-source-evidence-20261106.json`,
931,680 bytes, SHA256
`0730c5c5c3390fbeeaebb164ec6d9771dca56e919a742020afad21678f26b006`.
All dependency pins were checked. All 14 new captures exited zero without
output truncation; target-free Rizin captures reported the frozen PE unchanged.

Complete covered bodies include the clock and scalar getters, UI remaining-time
getter, population family, activation helper, typed refresh/caller bodies and
modifier evaluator through `1412c1d2c`. Its bytes at `1412c1d30..1412c1d6f`
are a switch table, not executable instructions. Initial partial population and
modifier captures were superseded by complete chained-family captures. Adjacent
partial clock/classifier bodies and the retained `1403b8852..1403b89db` tooltip
tail are not claimed as complete functions. Older captures retain their original
aggregate truncation flags and exact details: raw observed/retained bytes and
frames match, and normalized omissions are zero. Omitted duplicate rendered
frames are not described as untruncated captures.

The [Vulnerability name and fact-time result](209026-buff-instance-lifecycle.md#vulnerability-identity-and-description-time-input)
separately establishes authored description input 5000 and a literal display
input 1. Neither value becomes an authoritative application policy merely
because the client duration and count consumers are now understood. Application
trigger, target admission, level selection, multiplicity, stacking/merge/cap
rules and expiry/removal scheduling remain server-side reconstruction questions.
The same applies to combat damage changes and cleanse policy. No gameplay
implementation, observed buff display or retail-equivalence claim is made here.

## Exact buff738 modifier values

A bounded follow-up reads only cached `1282857:4626176` and its direct modifier
collection. One group at offset 4626320 contains two 72-byte rows at offsets
4626504 and 4626576, with raw kinds 41 and 43. Both have formula type 6, base
float 1.0, coefficients 0, condition 0, flags 0 and null optional references. The
exact evaluator switch maps formula 6 to `1412c1be2`, which returns the base field
directly without an instance getter. These two authored modifier values
therefore do not consume byte 5's level input. Two modifier rows do not mean two
stacks, strikes or applications. The later selector trace below binds the UI
labels and percent suffix without establishing authoritative effect policy.

The complete presentation collector `140894e40..1408952bd` groups matching
kind/reference rows and applies context and prerequisite filters before passing
selected rows to the evaluator. Current mode/filter results remain unobserved.
Authoritative effect, application and stacking policy remain unresolved.
Constant formulas do not justify an arbitrary wire-level
byte 5 default.

Collection source:
`ev_0c7e223389b1349d1e30c0302a3962aeb8c0c0016a624eceed0ae129a80d7a9c`;
typed cache read:
`ev_aa574240cf36fa7a232967957e8ccdd5f5fa73c8e7d8782630aa620d4dad20db`;
exact formula table:
`ev_94c2c2142ab470abd5ed9a51ddc496a47ffa7cd9b75bc2450b4c1f9786946417`.
Private findings `209026-buff738-modifier-findings-20261106.json`, 13,043 bytes,
SHA256 `34f9278e462a7d3c635e73d206640219b682fd9807b4614bf548c1711f4a7b48`;
full seven-record bundle `209026-buff738-modifier-source-evidence-20261106.json`,
418,847 bytes, SHA256
`81f5a803ea39ce49c08b075c955968c9078d6919fe7720fa09ce9f26156d3e93`.
The collection capture has aggregate truncation from rendered output: all
11,805 raw bytes and 35 raw frames are retained, with zero normalized omissions;
12 of 35 rendered frames are retained and 23 have unknown size. Its full selected
function is present. The data and table captures are untruncated. All three
new captures exited zero; process cleanup and source/cache pins passed. No
runtime, archive access or gameplay effect occurred.

## Modifier labels and UI units

The exact kind41 selector cell maps to `14088d901`, returning text187494,
**Incoming Damage**. Kind43 maps to `14088d90b`, returning text590644,
**Incoming Condition Damage**. Both category cells select return1 from
`14088d391`; positive values use text82509 (`+%str2%%% %str1%`) and negative
values use71511. These templates explicitly supply a percent suffix.

Both authored formula6 values remain1.0. Neither kind takes the special42/44
subtraction branch. Both pass the absolute numeric value to general formatter
`1403124e0`; its body was outside the first bounded trace. The follow-up below
recovers its numeric result for the exact1.0 input. The label remains Incoming Damage,
without renaming it to strike or physical damage. The UI suffix does not prove
an authoritative damage formula, additive or multiplicative stacking, a cap,
application timing, target eligibility, expiry or current filter selection.

Private findings `209026-buff738-modifier-kind-findings-20261106.json`,17,520
bytes, SHA256
`007c0ddd8e03249264ae963be25b181da615f1418be8bc58ff205166d13bb1b5`;
full11-record bundle `209026-buff738-modifier-kind-source-evidence-20261106.json`,
644,496 bytes, SHA256
`b94094a47b00cbd9b05a13e3d41ddfb392950caf2c8a2e3497200031708d4fe6`.
Formatter evidence `ev_7d6f1d865657d7242a51ceb1681fd9cfdc3de2e72f6c3cd69fe5dbd09c25bbb7`;
selectors `ev_80fba4a9abf7e00750adb04e6f4b1221677fe6d3eaba50e8c8654037c6563bb7`;
exact cells `ev_821835c46e4a11e8a86be4877a5106686ba5837b29789ba2eab1fb9e7e68a975`;
selected text `ev_ebe1078446cd3f184207a5da6c928d29e0345d3dc2469bd9e9289b85aba226a1`.
All four captures exited0. The selector capture retains all19,457 raw bytes,
28raw frames and zero normalized omissions; its aggregate truncation concerns
rendered output only (10of28 frames retained). The exact-cell capture has two
unrelated process-environment EACCES observations; owned cleanup is verified.
No runtime, archive or gameplay change was made.

## Two modifier eligibility predicates

Both authored buff738 rows pass the condition predicate `1412c09b0` on normal
return. The caller at `140895031` passes row+0x14 as ECX; both rows contain0.
The exact switch cell selects `1412c09f8`, returning EAX1 at `1412c0a07`
independently of EDX. Incoming EDX4 still calls `14140f1d0` before dispatch and
replaces EDX with its result. That external call's side effects, exception or
nonreturn behavior are outside this slice; its returned value cannot change
condition0's result. Current selector mode is therefore not an unresolved
input for this predicate's Boolean result.

The later call at `1408951b6` passes the actual row in R8 to `1412c0ef0`.
Its first test is byte[row+0x28]&0x1e. Both authored flags are0, selecting the
direct EAX1/return path `1412c1016..1412c1039`. This path calls no external
function and does not dereference the character/context arguments or read the
extra stack flags. Both recorded rows pass this second predicate as well.

These results close two specific collection filters. Kind/reference matching,
visited-row filtering and other collection context still matter; no current
rendered tooltip or authoritative application, duration, stacking, cap or damage
policy is inferred. No gameplay change follows from these presentation checks.

Each target-free REA capture reads a fixed window of at most512 bytes from the
pinned frozen PE, stopping at the exception-directory boundary. The condition
body/table is256 bytes; the later body is342 bytes. Root reviewed the complete
relevant instruction paths, table cell, caller registers and authored fields.
Private condition finding `209026-buff738-condition-zero-findings-20261106.json`,
SHA256`0462f6ff847a3b073c39c6f8b48b2e23af356ca0c00336ef489e529222caa06a`.
Later predicate finding `209026-buff738-later-predicate-findings-20261106.json`,
SHA256`faac408979080252c1307e5c302cfdffaa484704cfc05b7a1f63e685fdcc23eb`.
Full captures:

- `209026-buff738-condition-zero-window-20261106.json`, SHA256
  `13c34df94546d9a56427587589b611d07dacf35c159bbab378dc4513ca9deff2`,
  Evidence`ev_6faf71c7af508efd220fa47f87dfa057794869339ba016f2088be2fd4e617db3`.
- `209026-buff738-later-predicate-window-20261106.json`, SHA256
  `d63f2eca7eb70214bfdae9febecf7b95c27a241b1e12610156ece9d40c78c416`,
  Evidence`ev_bfffa666b4d3354ae3fb944911f1f067259b74c562ec405f1f0b8ef420cc41aa`.

Both new captures retain all raw, normalized and rendered output, exit0 and
verify owned cleanup and unchanged frozen-PE/helper hashes. The previously
retained caller's rendered-output omissions remain as documented above.

## Numeric formatting for the exact 1.0 values

For one selected row on the retained literal-count1 description path, the
source-derived English strings are **+1% Incoming Damage** and
**+1% Incoming Condition Damage**. These are static formatting results; neither
a live tooltip nor an authoritative server damage rule was observed.

The complete268-byte contiguous unwind family of `1403124e0` scales the absolute
input by the exact float4.0 at `14192de40`. Converter `1409bedf0` accepts4.0
within its bounds[-0.5,4194303], adds12582912.0 and masks the resulting binary32
bits with0x3fffff. The exact sum12582916.0 has bits0x4b400004, yielding4.
Both arithmetic operations are exactly representable under all four standard
SSE rounding modes for this input. Dividing4 into whole/remainder quarters
gives1/0. Text49780 is `%num1%`; its numeric argument is1, and the fraction
append is skipped. The previously pinned positive template supplies the percent
suffix and corresponding label. Other inputs and instance counts are unclaimed.

Private final finding `209026-buff738-numeric-display-final-20261106.json`,
SHA256`37b5bb2c1b260643ff5dddb347d8d8933f1412056650564d99807211887e04f7`.
Root verified its eight pins and reviewed the converter, constants, text rows
and outer arithmetic/formatting path. The initial8-byte entry fragment remains
retained as partial evidence; the later full268-byte family supersedes its
coverage. New converter/input and three-literal captures are untruncated with
verified owned cleanup and unchanged frozen-PE/helper/cache identities:

- `209026-buff738-numeric-display-inputs-20261106.json`, SHA256
  `29efdc07af79769b5e97e04c822734b5934c753f4f358b33ccfae4423a324450`,
  Evidence`ev_0e782cecc8c64487c550ff640fa0a58052c3fc4f3050c9c3449ad6ed8218faf6`.
- `209026-buff738-conversion-literals-20261106.json`, SHA256
  `c759f8a8e69dcaae51c02caa606de3b96867525a511ed6d79d8493a343eaa7f0`,
  Evidence`ev_97634f207ada0d1af49cd0e203e6ab0ac060efd4b704d0c825582209a718febd`.
- Complete formatter `209026-buff738-numeric-formatter-family-window-20261106.json`,
  SHA256`b94ff8a9bab59948eb870dadb83d8a145cdb424eda9e6e27ba9239acdad8a902`,
  Evidence`ev_c0eb09530680faad968b7b22e0a15964502aa4797b055457fef3c6cac0d13a92`.
