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

## Positive floater magnitude does not require a populated Word

A bounded follow-up traced that exact callback into its numeric display
consumer. `0x1403a99a0` forwards the custom events through `0x1403aaf80`
to `0x1403a96c0`. The exact table at `0x1403a9970` binds event 0x52 to
`0x1403a986a`, storing the floater type at control +0xf0, and event 0x53
to `0x1403a98d4`, storing the magnitude pair at +0xe8/+0xec. Event 0x51
separately stores the skill reference at +0xe0. These setters receive no
wire kind or raw Word.

For type 13, the exact digit-builder table selects `0x1403ac5d7` inside
`0x1403ac4a0`. With second magnitude zero, it skips the secondary digit
loop and builds decimal glyph geometry from the first magnitude at +0xe8,
using division by ten and each remainder. An incoming pair `[121, 0]`
therefore supplies the digits for 121. The selected display-resource
getter returns the nonnull static reference `0x141ae1614`; the animation
selector stores type 1. Neither reference nor animation type is assigned
a rendered color, asset identity or semantic name here. Resource readiness
and actual pixels remain outside this static result.

The known kind-2 distinction does not create a positive-heal Word
requirement. Wire kind 2 sets event +0x28 bit 5, read by virtual +0xa8 at
`0x1402e16e0`. Its demonstrated subdivision and aggregation uses first
require virtual +0xa0's negative-change, kind-1-skill predicate. Exact
Ether Feast is a kind-0 skill and a positive first delta fails that
predicate. The traced positive path reaches the numeric display from the
delta regardless of whether the raw Word is zero. This is not a global
equivalence claim for all health listeners or a recovered retail Word
default; it excludes the damage fixture's magnitude-as-Word hypothesis as
a prerequisite for this positive floater.

Given an explicitly supplied finite +121 first delta and zero second
delta, hypothetical current 200/maximum 388 becomes 321 and supplies 121
to this display path. At current 388/maximum 388 the meter stays full but
the original 121 is still notified. These are source arithmetic examples,
not runtime observations or a choice of server overheal policy. Separate
alive/downed/dead state and the accepted effect moment remain unchanged.

## Supported next increment and remaining policy

The existing `SourcedHealthFrame209026` can represent an explicit positive
health change, with character/agent identity, skill 10176, both deltas,
and raw discriminator/Word supplied by the caller. The client will apply
the bounded meter update and attempt the sourced UI notification above.
An absolute-current frame alone does not provide this event path.

The complete authored per-fact equation is already established in
[literal amount facts](209026-mesmer-tooltip-facts.md), with its second-level
and effective-attribute inputs bound in the
[character context](209026-tooltip-character-context.md). Those source
contracts are independent of run 144's observed tooltip number.

This delivery trace does **not** establish authoritative execution of those
facts, server-side cap, clone contribution, conditions removed, success/cancel
point, or recharge.
It also does not select a retail discriminator/Word pair. Tooltip amount
evaluation and command completion are separate source contracts; deriving
a displayed value does not establish a server execution time.
No gameplay implementation or packet defaults were changed in this pass.

The later story-map compatibility check also resolves maps392 and792 to
recharge context1, with the same authored20000 base. Both exact cached map
definitions have type4, flags0x100 with bit13 clear, and null reference+0x100;
the established selector returns1. Locale0 and second-level/attribute inputs
remain separate character state. This supports retaining the same cooldown
through those story maps without claiming their server gameplay is complete.
Private bundle `209026-ether-feast-story-map-contexts-20261106.json`, SHA-256
`ed01cbd18b2e6068591452141303cf98b57f3166c5d925654e2176c28d5c79b7`,
contains the two bounded captures; no runtime or archive access was added.

## Accepted-order start and step do not supply the missing effect policy

A further bounded source slice followed the two previously unbound start
notifications from the complete typed78 start body `0x141285740..0x1412858dc`.
This is distinct from the already traced execute-event and finish listeners:

| Notification | Exact listener dispatch | Constructor-established skillbar entry | Installed ordinary-view entry |
| --- | --- | --- | --- |
| Initial `0x1411db4b0` | Thunk `0x1411d3f70`, virtual +0x350 | `0x140163990`, `ret 0` | `0x14110bad0` |
| Per-step `0x1411db5f0` | Thunk `0x1411d4000`, virtual +0x3b0 | `0x140163990`, `ret 0` | `0x14110bd80` |

Both wrappers dispatch through character +0x1e0. The skillbar entries belong
to table `0x142165158` at skillbar +0x20; as in the
[finish-listener trace](209026-typed78-finish-listeners.md), its subscription
to that character collection is not established. Their no-op bodies remain
no-ops if subscribed. The ordinary-view table `0x142141558` has an independently
established subscription, and both concrete callbacks check their owner.

The initial view callback stores a supplied Dword at listener +0xb40 and
calls `0x141130dc0` on whole-view +0x2b8. That complete helper enters the
known AvCharEffect row selector `0x141134030` with mask **0x180000**. The
per-step callback stores the skill key and supplied step at listener
+0xb44/+0xb48, updates a retained reference, and calls the view helpers
`0x141144020` and, when present, `0x14113dcd0`. The former is explicitly
named `AvCharMachine.cpp` and consumes animation data. These exact callbacks
do not install a recharge record or apply a health delta. Their deeper
virtual calls and other dynamic subscribers are not exhaustively traced.

This closes a concrete possible local recharge-start edge without proving a
global absence of client behavior. It supplies no binding from primary
execute1067, action1350 or segment2067 to an authoritative heal or630 send.
The [timed-event consumers](209026-skill10176-animation-event-consumers.md)
already establish local timing-scale/force behavior; the retained run226
proves only the explicit1000ms command and2000ms finish diagnostic's lifecycle.
Neither yields the retail successful-effect timestamp or recharge-start,
cancel/refund policy. Recovering those policies would require an authoritative
execution specification or correlated reference behavior. This is not a
requirement for another client callback or an approval gate: a narrowly
labelled implementation can use the authored values and explicitly choose
its server scheduling policy. For example, the primary1067ms execute event
can be its effect deadline,1350ms its chosen order envelope, and authored
20000ms its unmodified cooldown. Starting that cooldown on successful effect,
the cancellation/refund rule and raw delivery fields remain implementation
choices; they must not be described as recovered retail behavior.

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

Positive-floater follow-up bundle:
`~/re/gw2/notes/209026-ether-feast-positive-floater-20261106.json`, SHA-256
`d63e03ed455b95cb1320298eb7f0dcc61ec158a4235a5d236a0914dd0836c2e5`.
It retains nine complete process captures: one prior event-body reread and
eight bounded frozen-source reads. All exited zero without truncation and
with owned-process cleanup verified. The logical switch tails were captured
separately where pdata ended early; table bytes after returns are treated
as data. No new runtime, archive or gameplay change occurred.

| REA evidence | Follow-up contract |
| --- | --- |
| `ev_6687d9e7887ad601ff4f935339e93c5f571a242613c36501ec67a53a3088b38e` | Exact custom-event table and type/magnitude setter tail. |
| `ev_1fef712420a135596d5f912d4a028695ef05a178764f52e56770fd9693790e38` | Exact type-13 decimal digit case and common exit. |
| `ev_0c7e0ed2056f05275519ed810cfd9e5257ec5ed3e2ddeef00a39c5f13e4e7e8d` | Exact type-13 display-resource, animation, digit and exit targets. |

Start/step follow-up bundle:
`~/re/gw2/notes/209026-ether-feast-start-step-listeners-20261106.json`, SHA-256
`7e750ab8e26dc2937f83523514de8154f20531cab385fbad5d9ebe8b545d7aa9`.
All seven source readers exited zero with complete raw/normalized output and
verified owned-process cleanup. One capture omitted duplicate rendered frames.
Initial pdata fragments are retained; subsequent logical continuations close
the claimed view-helper bodies. The per-step collection dispatcher captures
its callback and subsequent list bookkeeping but cuts its final epilogue;
no completeness claim depends on that epilogue or adjacent table bytes.

| REA evidence | Start/step contract |
| --- | --- |
| `ev_4f3af42a3a3648364cda4f254477143ccaeffad3d889159bebc1e598d4f30611` | Complete notification wrappers. |
| `ev_fa7b3ca10b21525091f1e557ca6cd2ad8fdd35c24651d358d4d90ebd7f955eaa` | Exact dispatch thunks and both concrete table entries. |
| `ev_c841f09ffd1bf2b62a1bd5b1107b36ddb5d8080b941650a9d6ab72fdf503c5e4` | Complete installed view callbacks and owner checks. |
| `ev_e5e53469dc690ffbdcd27b925d6bc87e68ff231899bd4337079ab32d4eee1cdf`, `ev_995b34b6a08881397d056e958c6a157f74fab60155e9125d96b2d15235c7b164` | Effect-selection bridge, named animation consumer, logical continuations and exact no-op. |
