# Build 209026 collision query filters

Static REA analysis of frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
These predicates refine the [initial layer matrix](209026-prop-fallback.md).
They do not yet identify the movement/ground query inputs. No runtime replay.

## Shared extended predicate

`0x1410bd190` receives two 32-bit filter values A and B. Its result can be
expressed with numeric fields, without assigning unsupported semantic names:

```
common = A & B & 0xff00
match_a = (common == (A & 0xff00)) if A & 0x20000 else (common != 0)
match_b = (common == (B & 0xff00)) if B & 0x20000 else (common != 0)
matched = match_a and match_b
inverted = bool((A ^ B) & 0x40000)
reject = bool((A | B) & 0x10000) and (matched == inverted)
```

The low five layer bits do not affect this predicate. If neither value has
bit 16 set, it accepts regardless of the other fields. This is applied before
the base filter in both custom body-pair and world-ray paths.

## Custom body-pair path

`0x1410bd3d0` reads filter values at each body/collidable `+0x2c`, applies
that predicate, then **masks both values to `0x1f`** before calling
`0x1416d08f0`. Therefore this path always uses the layer matrix. The base
function's high-group branch exists but cannot be reached from this particular
custom body-pair call. This narrows the earlier general description.

## World-ray path

`0x1410bd310` reads a 64-bit query field at `+0x28` and the candidate's
32-bit filter at `+0x2c`:

- Zero query field skips the preliminary mask and tagged-pointer checks.
- A nonzero field other than all-ones requires its low-32-bit mask to contain
  `1 << (candidate_filter & 31)`.
- Every nonzero field, including all-ones, also requires candidate byte
  `+0x28 == 1`. A signed byte at candidate `+0x20` locates another object
  relative to the candidate. Its qword at `+0x18` must have bit 1 clear and
  a nonzero value after clearing its low two bits. The precise object/tag
  ownership meaning is not established here.

It then applies the shared extended predicate to query dword `+0x24` and
candidate filter, followed by `0x1416d08c0`. That wrapper passes **full**
filter values to `0x1416d08f0`, unlike the custom body-pair path.

The base function uses the layer matrix unless the high 16 bits are equal
and nonzero. For that special case it rejects if either of these five-bit
comparisons matches, otherwise accepts without consulting the layer matrix:

```
((A >> 5) & 31) == ((B >> 10) & 31)
((A >> 10) & 31) == ((B >> 5) & 31)
```

## Shape-collection ray path

`0x1410bd290` reads query qword `+0x38`. Zero and all-ones skip its mask
check; other values require a bit matching the low five bits of the child
filter returned through collection vtable `+0x28` for the supplied shape key.
It then delegates to `0x1416d0860`, which compares query dword `+0x20` and
the full child filter through the base function. This override does not call
the shared extended predicate or the world-ray tagged-pointer check.

Scene initialization installs these overrides in the filter's subobjects:
body-pair `+0x10` (vtable `0x14212e570`), shape ray `+0x20`
(`0x14212e5a8`), and world ray `+0x28` (`0x14212e5c0`).

## Scope and evidence

Private bundle: `~/re/gw2/notes/209026-ray-filters-1.json`.

The diagnostic surface probe deliberately does not synthesize these query
fields or claim client-filter equivalence. The [scene wrapper and collector trace](209026-ray-query-construction.md)
now establishes generic query construction. Actual movement-query caller masks
and later matrix updates remain open.

- Custom predicates: `ev_62dc09fa6df6181f46998bc09425be1dd5e2025427df86b7c73c23727bf65e8e`.
- Base wrappers and group comparisons: `ev_6605399078e6e0ed26b1ed5a1723acb4da36c4837b3884830e3a9f46a0163a50`.
- Vtable references: `ev_2be4a64a949078ccc6bf748ad98178339f7eae0557731adabded9d21bb7ecc94`.
- Constructor installation/call references: `ev_4f891420b4811505a5a89fa70db461a6e4469a9fd3ef0236946d467d0f0e2209`.
