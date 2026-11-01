# Build 209026: bodies used during world and collision entry

Static REA follow-up to [server 46](209026-agent-world-entry.md) and
[server 49](209026-agent-scene-entry.md), on the same frozen executable.
No runtime launch, messages or new default flags were introduced.

The inspected full and simple collision constructors each establish the
body pointer used by their entry path. The full path's optional extent
operation also allocates a separate **visibility phantom**. These are
client-local allocations, not evidence of another network creation message.
This does not validate the input dimensions or every entry dependency.

## Simple collider's entry body

The simple constructor `0x1410a54f0` initially clears whole collision
+0x160, then calls `0x1410a8820` before returning (at `0x1410a579f`).
That complete helper computes an AABB around resolved position +0x130
using the existing height/radius and movement parameters. It tests for
**intersection** with expanded scene bounds +0x50/+0x60, not full containment:
its comparisons are box minimum <= scene maximum and scene minimum <= box
maximum, across XYZ. If needed it clamps/reprojects the center through
`0x1410b55a0`, `0x1410b70d0`, `0x1410a8a90` and `0x1410a8d30`, then asserts
the resulting AABB intersects those bounds.

When +0x160 is null, it allocates 0x130 bytes, initializes the object with
that AABB through `0x1417215b0`, installs vtable `0x14212d150`, stores it
at **+0x160**, and sets its +0x18 owner to the secondary collider interface
(whole collider +0x10). When the pointer already exists, it calls
`0x1410b97a0(scene, body, &resolved_position)` instead.

Thus a successfully completed simple constructor supplies the pointer
asserted nonnull by simple scene entry `0x1410a7a60`. This is distinct from
the capsule shape stored at +0x170 and from the full class's rigid body.
The later position setter also invokes this helper, updating the same body.

## Full collider's rigid body

The shared base constructor starts secondary +0x50 (whole collider +0x60)
at null. The full constructor then creates a capsule through
`0x1410b6000`, prepares its body description and constructs a 0x2c0-byte
body through `0x141723e20`. At **`0x14109ae1c`** it calls
`0x1410c19d0(whole_collider + 0x10, new_body, 0xff00ff00)`.

The shared setter asserts the incoming body is nonnull, releases an old
secondary +0x50 body if present, writes the new body through a pointer to
that field, takes a reference and assigns its +0x18 owner back to the
secondary collider interface. Further bookkeeping follows; this note does
not reinterpret the complete meaning of the supplied mask.

The field's identity is confirmed by the later `m_rigidBody` assertion
at `0x14212dcf0`. This constructor attachment precedes the factory-flag
branch that creates/skips the auxiliary +0x108 shape. It does not depend
on receiving a separate network body-allocation message. Allocation failure
is not a supported successful-construction case in this trace.

## Server 46's extents create a visibility phantom

Full collision virtual +0x118 resolves to **`0x1410a4e60`**. It divides
all three extent components by 32, without the position Z sign flip,
and calls `0x1410c1ed0` on the secondary interface with the unchanged
mask. Simple collision's corresponding slot is the no-op `0x140163990`.

The shared helper asserts **m_rigidBody != null** and
**m_visibilityPhantom == null**. It allocates 0x140 bytes and calls
`0x1410c8050` with the mask, converted extents and rigid-body +0x1a0 vector.
The resulting object has vtable `0x14212f550` and stores:

| Phantom offset | Value |
| --- | --- |
| +0x110 | Converted extent Float4 |
| +0x120 | Vector copied from rigid body +0x1a0 |
| +0x130 | Unchanged incoming mask |

It is stored at secondary **+0x68** (whole collider +0x78), with owner
secondary-interface pointer OR 1 at phantom +0x18, then passed to
`0x1410b9730`, the same scene helper called by the entry methods.
The `m_visibilityPhantom` name comes directly from assertions at
`0x14212f370` and `0x14212f398`.

For server 46, entry-option bit 0 zeroes the extents and thereby skips
this allocation. That branch still sets AGENT_FLAG_ENTERED_WORLD, and
option bit 1 independently controls collision scene entry. The trace
establishes these allocations and distinctions; it does not establish
which option bits a real initial player should use or the full visibility
behavior of the phantom. Constructor success alone is not proof of a
playable bootstrap.

## Evidence

Private bundle: `~/re/gw2/notes/209026-entry-collision-bodies-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_279ca1fb5cc557ea52446709a1f42505324a923f058486ef159c653fd7b16972` | Complete simple constructor, null initialization and final body-helper call. |
| `ev_54aec112cff3f21dbecca7d76c60a5af01bf1470973706b1109981c7ffc0bcda` | Complete simple body helper and full extent conversion. |
| `ev_c380357bc8eb2294f7e9e573e6c6d0baa4284a714ab6f3c4896e9e24ee92d8ec` | Full/simple +0x118 entries adjacent to scene-entry slots. |
| `ev_a98ba96bb2141f0c493a9d03c137563c3653140a52ea81b42c16ccbf9ac2b836` | Complete visibility-phantom allocation helper, gates and storage. |
| `ev_8969c97427a598b03a5631d5ad5778832c63af66e6777a2135380cad577edf89` | Named visibility-phantom assertions. |
| `ev_0e4c91ddd3ba50d04818a4cc212843ddaf758f780827cb8ff56052d268dae95d` | Named rigid-body assertion. |
| `ev_74eddcf655c7526875e6e67e1eaf73a4fc09f1cc80634b63b86a467688e804c2` | Complete phantom constructor and extent/vector/mask copies. |
| `ev_be73dd9abf8d366f1ed9088f7835e06dffc28fabb659dc4ab716ff0ea1f31aae` | Rigid-body setter through its field assignment/reference/owner writes; later bookkeeping only partially inspected. |
| `ev_c15082ec3b78bd16dd3d2340e865de067ec0d773b34e123cdebf1d7f78c1369b` | Full constructor body allocation and setter invocation before auxiliary-shape branch. |

The initial bounded store search did not find the attachment because the
setter writes through a field-address register. The final conclusion rests
on the actual constructor call and setter instructions, not that negative
search. No exhaustive body-lifecycle or allocation-failure analysis is claimed.
