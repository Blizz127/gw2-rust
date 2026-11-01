# Build 209026: creation bytes and collision branches

Static REA findings, 2026-11-01, extending [agent creation](209026-agent-creation.md)
and [dimensions](209026-agent-dimensions.md). The frozen executable identity
is recorded in [client-build.md](../client-build.md). No client replay or
new bootstrap defaults were introduced.

## Three distinct bytes

The three bytes immediately before Point3 have separate consumers. Field
numbers below match server 37's zero-based schema field list.

| Field | Command offset | Base-agent destination/use |
| --- | --- | --- |
| 7 | +0x7c | Passed to `0x141031290`; whole base agent +0x48 (subobject +0x30). |
| 8 | +0x80 | Whole base agent +0xe8; collision path and factory flags. |
| 9 | +0x84 | Whole base agent +0x108; additional factory flags and later options. |

Field 7's initialization asserts that masks 0x08 and 0x20 are clear.
The assertion strings name them `AGENT_FLAG_ENTERED_WORLD` and
`AGENT_FLAG_REVEALED`. This does not establish names for its remaining bits.
Field 2 is a separate command-queue flag byte and is not one of these.

## Factory flags

Let `A` be field 8, `B` field 9, `r` the final radius float and `h4`
array element 4. For finite valid dimension inputs, the instruction flow
at `0x141024ca8`..`0x141024d33` is:

```text
F = 1 if (A & 0x09) != 0 else 0
if A & 0x10: F = 0x10
if B & 0x01: F |= 0x02
if B & 0x04: F |= 0x20

eligible = (A & 0x08) != 0 or ((A & 0x01) != 0 and (B & 0x02) != 0)
if not (B & 0x01) and eligible and h4 / r < tanf(angle):
    F |= 0x0c
elif B & 0x01:
    F |= 0x04
```

`angle` is the exact f32 at `0x14192bb84`, bits `0x3f5f66f3`,
approximately 50 degrees in radians. The math helper `0x140e5c850` is
identified as `tanf` by its exceptional-input helper `0x140e7f598`, which
passes the literal `tanf` at `0x1420c86bc` to the math error path. This
is a dimension-ratio branch, not a triangle-normal walkability test.

The full factory `0x14109ca00` asserts `F & 1`, with the named string
`COLLIDER_CHAR_CREATE_FLAG_PLAYER`. Setting A's 0x10 overrides the initial
player bit rather than simply adding another bit. A classification value
with high nibble 3 (player identity) does not itself supply this factory bit.

## Constructor selection

Before constructing collision, the base constructor may augment A with
internal bit 0x800. It does so only when A has bit 0x02 and
`0x1410244b0` returns true. That helper obtains the agent context and
calls `0x14102f700`, whose exact predicate is:

```text
context[+0x10] != null and context[+0x10][+0x08] == 0
```

The selected paths are then:

| Condition | Call |
| --- | --- |
| Internal 0x800 set | `0x140168aa0`, a zero-returning stub in this executable. |
| 0x800 clear and A & 0x02 | Simple factory `0x1410a5e20`. |
| Otherwise | Full factory `0x14109ca00`, including its player-flag assertion. |

The command constructor clears bit 0x800, but the base constructor can
reintroduce it through this context predicate. The stub path's intended
context and lifecycle are unresolved; it must not be treated as a working
character-construction path. The constructor later uses the returned
collision pointer.

## Shape effects in the full constructor

The shared collision base initializes its flag word to zero. The full
constructor subsequently sets its baseline flags, then maps:

| Factory flag | Full collision flag/effect |
| --- | --- |
| F & 0x02 | Sets bit 35 (`0x800000000`). |
| F & 0x08 | Sets bit 47 (`0x800000000000`). |
| F & 0x10 | Sets bit 52 (`0x10000000000000`). Later meaning unresolved here. |
| F & 0x04 | Skips construction of the auxiliary shape at collision +0x108. |

With bits 35 and 47 both clear, the constructor applies the second height
minimum and builds the ordinary capsule described in the dimension note.
If bit 47 is set it selects the additional-parameter shape helper
`0x1410b6260`; otherwise bit 35 selects the cylinder helper `0x1410b60f0`.
The branch mask is `0x800800000000`.

In the simple constructor, F & 1 sets collision flag 0x10000 and F & 0x20
sets 0x1000000. This does not establish equivalence with the full path.
After collision creation, B & 0x08 separately invokes collision virtual
+0x4d0 with argument 1; that option's full behavior is not mapped here.

For example, A=1 and B=0 selects the full factory, supplies its player
bit, and leaves the two alternate-shape bits clear regardless of h4/r.
This is a branch example, **not** evidence that those are the correct
bootstrap bytes for a real player. The remaining base-agent effects,
content dimensions, timing and component initialization still apply.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-creation-flags-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_30932644a3ad7ebcf1804bf81fe9cb390d818a8418e85544a1e853a4a3734795` | Byte copies, base destinations, flag arithmetic and constructor selection. |
| `ev_b2cf1d171f721d9f6e0b11c485c82bd898f693099d9a0b37ff1075eaf5630da0` | Execute method forwards creation fields. |
| `ev_a82f523ac3afea69e2f099ef79d509f14353bfe36be4c6e96c022c7952fedf36` | Field-7 assertions and player-flag assertion. |
| `ev_1d9ff61bc07ec6427e753769c4cd20306fc5fd65b6c8e92248abbfc95bb30822` | Context-query wrapper, angle bits and zero-returning stub. |
| `ev_826f876732fcc2ab907a704b8b72f3abe62644b44598939e001590ab70ac7c81` | Context predicate, initial collision flags and named field-7 assertions. |
| `ev_7a0f17e46bff0c0000f4724f9c4d94af22d2e0855d1bdf1e3ccf8d22346c9964` | Math helper exceptional-input call. |
| `ev_a6fc0a6cb6555d4c8ca1becfa60e3b93320a002da13bcf622354f2afa61af4cd` | Math error-name reference and full-constructor flag mapping. |
| `ev_4c622267a8699f55c62f759e591f4c4f8d3de3f6cec4cab274ed1bd15165ed83` | `tanf` literal and its error-handler use. |
| `ev_85371b5adab45fc49ef94d9c0b3441ca1c54f46d5911ab86fbb8ebb4d73cb9d5` | Primary and auxiliary shape branches. |
