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
