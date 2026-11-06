# Build 209026: positive health changes and combat-text delivery

Static REA, 2026-11-06, against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No client runtime, container or archive was accessed. The existing
[health initialization](209026-character-health-endurance.md) and
[sourced-damage observations](209026-npc-talk-loop.md) provide the starting
anchors; this pass follows their concrete consumers.

**Server 255 supports a positive health delta and a sourced health-adjustment
UI notification.** For ordinary finite values and valid meter state, it
adds the delta and clamps the resulting current value to `[0, maximum]`.
The listener and UI event retain the original delta, rather than the amount
actually added after clamping. With a positive first delta and a zero second
delta, the traced UI branch selects floater type 13, subject to its normal
observer/entity filters. No runtime rendering or authoritative heal schedule
is established by this source pass.

## Handler, meter, and listener contract

The complete handler `0x14124f990..0x14124fa8b` requires an existing character
and health component. A nonzero skill key resolves through content type 0x40;
zero supplies a null skill reference. A nonzero source-agent key resolves
through `0x141024060` and asserts if unknown; zero supplies null. It forwards
both floats, the byte discriminator, the skill reference, the source agent,
and the zero-extended Word to `0x141207350`.

`0x141207350..0x14120745c` saves both incoming floats and calls
`0x141275ee0..0x141275f13` on meters at health +8 and +0x24. The latter adds
the float to meter current +4, then bounds it against zero and meter maximum
+8. These are health component +0xc/+0x10 and +0x28/+0x2c respectively.
It does not advance the meter clock or change its rate.

The byte conversion is explicit, not an identity mapping:

| Wire byte | Listener kind |
| ---: | ---: |
| 0 | 0 |
| 1 | 2 |
| 2 | 3 |
| 3 | 1 |

Other values enter the invalid-kind diagnostic. The listener dispatch at
`0x141206fc0..0x1412071d4`, through trampoline `0x140255568`, receives:

| Callback argument | Established value |
| --- | --- |
| RCX | Registered listener interface |
| RDX | Owner character |
| XMM2 / XMM3 | Original first / second delta |
| Stack argument 5 | Mapped listener kind |
| Stack argument 6 | Skill-definition reference |
| Stack argument 7 | Source-agent pointer |
| Stack argument 8 | Zero-extended raw Word |

There is no effective-gain comparison between the meter update and this
notification. For example, an incoming `+121` while already at maximum
leaves current unchanged but still supplies `+121` to the listeners. This
is a client delivery property, not proof that the retail server sends
uncapped overhealing amounts.

The absolute setter `0x141275fd0..0x141276023` also clamps current.
The maximum setter `0x141276030..0x14127611c` preserves the old
current/maximum fraction, using zero when the old maximum is zero. It
asserts a nonnegative maximum and a fraction in `[0,1]`. Maximum updates,
absolute current synchronization, rate accumulation, and sourced health
deltas therefore have distinct effects. This trace makes no NaN/infinity
contract and does not change the separate alive/downed/dead state.

## CombatTracker filtering and raw Word

Constructor `0x1402de0e0` installs the health listener vtable
`0x14193d0a8` at tracker +0x30; its first slot is
`0x1402e3be0..0x1402e3d33`. The callback verifies that the passed owner
character's agent equals the tracker's agent. Its negative-change filter is:

```text
if (first_delta < 0 || second_delta < 0)
   && (skill == null || skill.kind_at_58 != 1)
   && mapped_listener_kind == 0:
    return without a CombatTracker event
```

The pointer read at `0x1402e3c53`, then tested at +0x58, is the **skill
definition**, not the source agent. The complete dispatch above establishes
the stack argument identity. Positive first delta with second delta zero
bypasses this filter for every supported wire kind.

The callback also resolves configuration ID 0x66 with expected type 0x40.
If the resulting skill pointer equals the received skill and the mapped
kind is 3, it substitutes Word value 1 before forwarding. The configured
skill's identity was not resolved in this slice; this is not a named
Ether Feast rule.

`0x1402e2d20..0x1402e3153` first calls eligibility predicate
`0x1402e46a0`. One established passing case compares its selected service
object's virtual +0x80 result with either the source or receiving agent;
other mode/configuration/relationship branches exist. Those states were
not sampled. Failed eligibility suppresses the UI event, after health has
already changed.

The allocated event uses vtable `0x14193c7d0` and type 8, named
`COMBAT_TRACKER_EVENT_TYPE_HEALTH_ADJUSTMENT` by its getter assertion.
Fields +0x40/+0x48/+0x50 hold source agent, skill, and forwarded raw Word;
+0x58 holds receiving agent; +0x60/+0x64 hold original deltas.
Flag +0x28 bit 5 is set when the mapped kind is 3 (wire byte 2).

The raw Word has a demonstrated conditional numerical use, but no complete
retail semantic name. Event virtual +0xa0 (`0x1402e16b0`) requires a
nonnull skill of kind 1, its body +0x18 equal to 2, and a negative first
delta. If that predicate is true, virtual +0xa8 is false, and Word is
nonzero, the builder divides both deltas by Word and recursively emits
Word subdivisions with their Word cleared. Further aggregation is also
conditional. These are special negative-change paths, not a positive-heal
amount or duration field.

Exact Ether Feast 10176 has skill kind 0 and flags `0x14000006`, as retained
in [the cast-time data trace](209026-mesmer-cast-tooltip.md). It does not
enter that kind-1 negative subdivision predicate. This does not establish
which raw Word or discriminator the retail server chooses for its heal.

## Positive UI branch

The previously bound AsCombatEventManager listener `0x1403a0b70` requires
either source or receiver to pass `0x14039f980`. That helper includes the
same direct selected-service-object/agent comparison, followed by a
relationship path. The type switch at `0x1403a1a34` routes event type 8
to `0x1403a0e27`.

For the ordinary positive branch, it adds the two original deltas. When
the sum is positive, it chooses **13 + (second_delta > 0)**. Thus positive
first delta and zero second delta select type 13; a positive second delta
selects 14. These numeric floater types are not wire kind values.

The branch converts each delta's absolute magnitude separately through
`0x1409bf050..0x1409bf09c`: add literal f32 0.5, then truncate to a signed
64-bit integer, with the caller retaining the low Dword. For small finite
nonnegative magnitudes this is half-up rounding. The function's assertion
checks input at least -0.5; this caller has already cleared the sign bit.
No larger-range or nonfinite UI-number contract is claimed.

Skill flag +0x38 mask 0x100000 enables a separate floater aggregation path;
10176's retained flags have that bit clear. Its branch therefore calls
`0x14039fa80` at `0x1403a1190`. The complete helper
`0x14039fa80..0x14039fc5e` requires a nonnull receiver and type below 0x36,
creates a child with callback `0x1403a99a0`, and supplies UI event 0x51
with the skill, 0x52 with the floater type, and 0x53 with the integer
magnitude pair. It also supplies the receiver-derived position through
event 0x4f. This establishes a health-gain floater **dispatch**, not its
rendered color, visibility, or pixels under current UI state.

## Supported next increment and remaining policy

The existing `SourcedHealthFrame209026` can represent an explicit positive
health change, with character/agent identity, skill 10176, both deltas,
and raw discriminator/Word supplied by the caller. The client will apply
the bounded meter update and attempt the sourced UI notification above.
An absolute-current frame alone does not provide this event path.

This source does **not** choose the retail heal amount, server-side cap,
clone contribution, conditions removed, success/cancel point, or recharge.
It also does not select a retail discriminator/Word pair. Tooltip amount
evaluation and command completion are separate source contracts; deriving
a displayed value does not authorize applying it at a guessed time.
No gameplay implementation or packet defaults were changed in this pass.

## Evidence and coverage

Private complete bundle:
`~/re/gw2/notes/209026-health-gain-delivery-20261106.json`, 507656 bytes,
SHA-256 `c7275f8a4c4dce6698dd84f47deee8237a358eed2e69abcbd89fc9194caac0ed`.
All 20 bounded process readers exited zero. Raw PTY captures are complete;
some duplicate rendered terminal frames were omitted by output budgets.
The large UI event switch was followed through its health branch and
shared return, without claiming all unrelated event cases were traced.

| REA evidence | Contract |
| --- | --- |
| `ev_69e6c480a9a1187b7214271927b0919779cf7a54c21f5c83677519e4616a4fdf` | Complete 255 handler, delta/absolute clamps, kind remap and original-delta notification. |
| `ev_d485b3dec9872a6cbcdb9081047b1caf60079c8e883207558e50600c7dff15b6` | Complete dispatcher and health listener; exact skill/source arguments; maximum-setter continuation. |
| `ev_aa253bd6b21f27250e2d5140da4f7914ba9ad72c8cf135e3e3ba09fc5c9b1595` | Event fields, subdivision, aggregation and listener delivery. |
| `ev_1b478ac168433f381d69573b74f13b5294f639eeb82201e020038db855b98aba` | CombatTracker eligibility and event constructor. |
| `ev_f664471d0754f578c2b412405b581a964faf2c4dc2296d7b5151439ca7e8373d` | Exact event predicates +0xa0/+0xa8/+0xc0 and delta getters. |
| `ev_a3ed5266a63f09e52f406b6b94fa460e6ab6181b2435510c2daf6f9a2992a0e8` | Listener constructor binding, UI consumer prefix, complete floater creation. |
| `ev_c3537feb3cc865e46253eb23bc9426b80e5285ed70886f17938eb95a8ec78e98` | Type-8 switch destination, getter bindings and health-adjustment assertion. |
| `ev_607480bfb040b34ea23f0231477825c27f271e1965d8904f167381bdcb98e617` | Positive type 13/14 branch and magnitude calls. |
| `ev_dd09e254fc320a6770829370b3ae1cbd3eacce72a0763b90b72554e746792073` | Continuation to concrete floater call and return. |
| `ev_47d25617dd875973cd943db43d32d871d07d9e447c8270aa532103fb4aa4c86a` | Complete UI entity-eligibility helper and event getters. |
| `ev_5d4e8bc75cc1cf273200e9e2cae5a1639ba1f95d6ff6aed898ff1ef9f7c4140b`, `ev_3d57617ce79e6dc8e851498644a03eef2ac75824affa91ed187c97f3163b3dfb` | Complete magnitude-conversion leaf and exact -0.5/+0.5 constants. |
