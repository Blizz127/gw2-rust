# Build 209026 scene ray-query construction

Static REA analysis on the frozen executable, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This follows the [ray-filter predicates](209026-ray-filters.md) into their
scene wrappers and hit collectors. Actual movement callers remain unidentified.

## Scene interface and configuration

Scene constructor `0x1410b2710` installs vtable `0x14212e240` and initializes
flags at object `+0x6d8` to zero. Virtual `+0x300`, `0x1410bc300`, sets or
clears supplied flag bits. Configuration path `0x141414700` requests:

| Setting | Scene bit | Fallback default |
| --- | --- | --- |
| `CollideEnableRaycastFiltering` | 1 | enabled |
| `CollideEnableGroupedRaycasts` | 2 | enabled |

Each setting can be overridden by the configuration lookup. These defaults
do not establish a particular live runtime state. The scene accessor
`0x141095210` jumps to `0x140cd8310`, which asks the current map's virtual
`+0x130` for module index 2.

## Single-ray wrappers

Scene virtual `+0x180` is `0x1410ba190`; virtual `+0x188` is
`0x1410ba270`. Both take two position-like inputs, convert each XYZ to
`[x/32,y/32,-z/32]`, preserve each input's dword at `+0xc` as a separate
transform tag, and pass the caller's fourth argument as a mask. They reverse
the two argument positions when passing the converted endpoint structures
to their helpers; caller meaning/order should not be guessed from argument
position alone.

The first calls `0x141097e30`, which supplies filter-info zero to
`0x141097be0` (nearest hit). The second calls `0x141097a80` (hit list),
which also initializes query filter-info to zero. Both pass null for the
optional restricted-world argument.

Each helper resolves tagged endpoints through `0x1410b7710`, then checks/
clips against scene bounds through `0x1410bd5b0`. Tag values with both high
bits set can resolve an associated transform through the scene table;
otherwise the endpoint vector is copied. That path is not equivalent to
always assuming an identity transform.

The Havok world-ray input contains vectors at `+0` and `+0x10`, a byte at
`+0x20` initialized to zero, dword filter-info at `+0x24`, and qword mask
at `+0x28`. The mask construction is:

```
if scene_flags & 1:
    query_mask = u64(caller_mask) if caller_mask != 0 else UINT64_MAX
else:
    query_mask = 0
```

The original caller mask is also retained in the collector at `+0x18`.
Thus disabling the preliminary filter does not remove every mask check.
The ordinary scene path calls `0x14171df10` with scene world `+0x268`;
that function dispatches into the Havok ray caster `0x1416c2120`.
For these two public wrappers, filter-info zero makes the base layer lookup
use layer 0, which initially pairs with every layer. Their caller mask and
collector checks are therefore essential to deciding eligible geometry.

## Collectors

The hit-list collector uses vtable `0x14212c598`, callback `0x14109a550`.
The nearest-hit collector uses `0x14212c5a8`, callback `0x14109a6d0`.
Both walk the hit collidable's `+0x18` parent links to the root, then:

1. Reject a negative hit fraction.
2. If original caller mask is nonzero, require its bit for the root filter's
   low-five-bit layer.
3. Require root byte `+0x28 == 1` and the nonnull tagged-pointer condition
   described in the filter note.
4. Form the interpolated hit point and pass the result to `0x141096740`.

The nearest collector additionally requires the fraction to be strictly less
than its current best, initially 1. It updates that best after accepting the
hit. A fraction exactly 1 is excluded by this collector. This differs from
the offline probe's deliberately inclusive interval; the probe remains an
independent geometry diagnostic.

The hit-list collector appends through `0x141097a10`; the nearest path uses
a caller-provided 0x58-byte output. The [clipping and generic result layout](209026-ray-results.md) are now
traced. Movement caller masks and type-specific metadata remain open.

## Evidence

Private bundle: `~/re/gw2/notes/209026-ray-query-construction-1.json`.

- Configuration/defaults/accessor: `ev_ab681d6c5fb38eca62e21fe21cc1d898d973ef27dca7066a600cb5b171da2cbf`.
- Module accessor: `ev_8d5dcaba7ffab0ca2e6b9b3f52056197fab9e37776e78d4516faaec8c0723bab`.
- Scene constructor/vtable: `ev_f58261848ae84149bf92e010308cd96ed546a7b2651d1785eab85a0db1c2d601`, `ev_747b8dea8fc732e92a26849f8d6db2188be81ebeee9890a54dc4f02b59708137`.
- Setter/public wrappers: `ev_18e9495bdd3d680956c2c6a5a0cade6907dd7819263a73f99fdc90db1bf71b72`.
- Query construction: `ev_8783a5be7692751381066f328596a1f5e956d7dca263f6cfcfd17274021e2381`.
- Endpoint transform/world entry: `ev_eb5d4477a2584fe07f272e11c8755655df0f70d7d2abab584c12547ebc858a32`.
- Collector callbacks: `ev_ee55b579b0084afe6eea2abaf79d78da476583b24453132edf1356d65c1ee32d`.
