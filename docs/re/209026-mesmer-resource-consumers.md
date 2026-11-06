# Build209026 Mesmer resource consumers

2026-11-06. The two values in server481/483 now have a concrete display
interpretation: **first = displayed capacity; second = current resource
count**. The source supports these roles through the Mesmer status widget's
segment layout and texture selection. This bounded executable-only slice
does **not** establish that the second value counts live clone agents,
that the first must be3, or how a server chooses initial/update values.
The raw codec and observer defaults were not changed.

The frozen executable SHA256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
REA captured hash-checked source windows and exact target references; no
client, archive, launcher, server, or publisher action was performed.

## Exact component-to-widget relationship

The [profession-state note](209026-mesmer-profession.md) establishes wire
order, component storage, primary table `0x14217c0f0`, and secondary table
`0x14217c270` at component +`0x48`. The secondary getters are now traced:

| Wire value | Component Dword | Secondary virtual | Native getter |
| --- | --- | --- | --- |
| First | +`0x54` | +`0x18` | `0x1403b57e0`: returns receiver +`0x0c`. |
| Second | +`0x50` | +`0x10` | `0x140168d10`: returns receiver +8. |

Primary profession virtual +`0x80` uses `0x141288c40`, which returns the
component's +`0x48` secondary interface. Widget helper `0x1414f8e20`
gets its character's profession through virtual +`0xa8`, then uses that
profession virtual +`0x80`. Its assertions explicitly reference
`AsProfessionMesmer.cpp`, `profession`, and `mesmer`.

Registration at `0x1414fc030..0x1414fc06d` stores that secondary interface
in widget +`0x70` and registers the widget's embedded listener at +`0x40`
through secondary virtual0. The paired removal path `0x1414fbd00` asserts
`m_registeredMesmerNotifier == character->GetProfession()->GetProfessionMesmer()`.
This ties the widget to the exact Mesmer interface rather than merely
matching a generic getter offset.

## Capacity and current-count consumers

The function beginning at `0x1414fb1a0` reads secondary virtual +`0x10`
into R12D and +`0x18` into R15D. In its ordinary segment-rendering branch:

* First/R15D divides the available bar width at `0x1414fb266` and bounds
  the segment loop at `0x1414fb39d`.
* For each segment index, `0x1414fb372..0x1414fb384` compares it unsigned
  with second/R12D. Indices less than second select one texture; the
  remaining segments select the other.
* Those selections are submitted through the widget rendering path at
  `0x1414fb388..0x1414fb396`.

Therefore first controls the number of resource positions and second
controls how many receive the alternate/active texture. The individual
texture assets were not decoded in this executable-only slice. The
existing profession predicate also accepts a requested quantity only when
it is no greater than second; the existing ratio is `(first-second)/first`.
Together these support capacity/current-resource-count roles without
requiring an asset name or a live-entity census.

This rendering branch is skipped when the existing specialization helper
`0x14122aa20` returns discriminator `0x23`. No name is assigned to that
discriminator here, and the observed branch is not generalized to every
Mesmer specialization.

The UI formatting branch at `0x1414f9f46..0x1414f9f64` separately passes
second to `0x1410dbde0`, after selecting key `0x32221`, `0xf7709`, or
`0x11b44b` according to the specialization result. Those referenced text
resources were not resolved, so their numeric keys do not prove a clone
label. A separate layout branch uses first at `0x1414f9dd4..0x1414f9e62`.

## Remaining boundary

Bounded ASCII/UTF16 named-text checks found the Mesmer component/widget
source anchors but no field-linked clone/illusion label. That result is
limited to those checks; it does not establish that no such resource or
consumer exists. Proving **clone count** would require a field-linked
localized label or a demonstrated clone-agent/state producer. This trace
does not establish a server-side clone creation/destruction policy,
live clone census, specialization-specific maximum, or retail login pair.
It leaves the existing arithmetic guard and explicit caller-supplied state
unchanged; neither constructor `[1,0]` nor research `[3,0]` becomes a retail
default through these display consumers.

## Evidence

Private eleven-record bundle:
`/home/blizz/re/gw2/notes/209026-mesmer-resource-consumers-20261106.json`,
SHA256 `aa51695ccf04243cf1390943ba47d5f7c602342050c63e71ec1162f88e9a0db3`.
All captures exited0 with nontruncated output and verified owned-process
cleanup. Unrelated inaccessible `/proc` entries remain outside that claim.

| Evidence | Scope |
| --- | --- |
| `ev_c534316767284a2b07132646711b610eddecfc67150f91ccc7c4ae2b32c85a8e` | Secondary getters, constructor offsets, notification trampoline. |
| `ev_185034fdef39b8430e9c8d469ffca003b3eb3f16a5455135fd4937dd16640f89`, `ev_da665e8cf0cf210b47db62a9fcb365c9214e0a07cab024bf20f9b61f83dca792` | Named UI anchors and instruction-validated exact references. |
| `ev_928df30898507a0c7a1a1cea4a2e42bcfc16dd16cb9fa2abfd5be91ed1673e43`, `ev_6dbdc137927ea8cf35256c414d2bb817903147e084d8c4dfc906dc56477450c0` | Profession/interface relationship, listener registration and removal. |
| `ev_dfd3306dc78e39379247123b820daf53365effa9ac8f5d896907100cf15daa7c`, `ev_4f1a9c81683b9455c618fe0cc0661af68a772d9efefa423b2fd5eb2d3b4700a2` | Exact helper callers, segment loop, value formatting, and layout. |
| `ev_0875ad7fdb8982b61ea806db74cf3925540fe2a8de0721bed4e80018a040742a` | Aligned render prologue and bounded UTF16 named-text check. |

Initial `.pdata` queries could not bound several short leaf getters;
subsequent exact aligned windows supply those instructions. Other windows
include unrelated adjacent code, cut boundaries, or jump-table data. The
claims above rely only on the identified aligned instructions and calls,
not disassembly of those surrounding bytes or a claimed whole-function
absence of other behavior.
