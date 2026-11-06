# Build209026: skill action activation, deactivation and explicit cast request

The ordinary skill-input path can send packed **0x2a before the separate
SkillBegin0x23 request**. Input-action deactivation reaches the path that can
send **0x29**. Generating a new cast or hit from0x29 therefore turns an end
notification into another attack. Moving that synthetic cast to0x2a would
also be incorrect:0x2a precedes client admission checks, and an admitted
ordinary cast has its own0x23 producer.

This closes the input-caller gap in
[the producer note](209026-skill-input-29-2a.md). The evidence is static,
from frozen build209026 executable SHA256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No client was launched or attached, no working archive was read, and this
lane changed no server behavior. An ordinary repeat timer, a completion
callback that requests the next sword attack, and server acceptance policy
remain separate questions.

## Concrete action registration and active-state direction

The binding is established through constructors and receivers, rather than
matching unrelated virtual offsets:

| Object or edge | Source binding |
| --- | --- |
| Input-binding context | Static initializer `0x14021d340` constructs global `0x1428b5980` through `0x1413aa4e0`; its primary table is `0x142256688` |
| Context getter | `0x1413ac960` returns that global |
| Skill-input listener | Constructor `0x1402c9d70` installs table `0x14193a0e8` and registers itself through context virtual +`0xd8` |
| Registration | Context +`0xd8` is `0x140285f30`, inserting the supplied listener into the collection at context +`0x20`; +`0xe0` unregisters it |
| Activation target | Listener table +0 is `0x1402ca180` |
| Deactivation target | Listener table +8 is `0x1402ca420` |

Complete action-state setter `0x1413b2130..0x1413b2208` compares incoming
EDX with action +`0xb0` bit0. Equal state returns without notification.
Otherwise it changes bit0 and obtains the action's numeric identifier
through its virtual +0. The preceding helper `0x140168aa0` is exactly a
return-zero stub in this frozen image, selecting this numeric-ID route.
The notification selects:

- Nonzero state: thunk `0x140255568`, tail-calling listener virtual +0.
- Zero state: thunk `0x140255560`, tail-calling listener virtual +8.

Thus activation/deactivation direction is proven by the state write and
the installed listener table. It is not inferred from adjacent opcodes.
Context initialization `0x1413ab9e0` registers engine events0x1d and0x20
with `0x1413acf30` and `0x1413ad080`. These respectively insert/remove an
input-set entry and compare the resulting action sets through
`0x1413aea20`: old-minus-new actions receive state0, new-minus-old state1.
These engine event numbers are not assigned OS key or mouse-button names
here. Configurable physical bindings remain upstream.

The exact switch tables map numeric actions to slots as follows:

| Numeric action | Slot | Activation / deactivation |
| --- | --- | --- |
| 21..25 | 5..9 | Both |
| 26..30 | 0..4 | Both |
| 31 | 21 | Both |
| 32..36 | 12..16 | Both |
| 169 | 20 | Both |
| 115 | 11 | Activation only; deactivation selects its default branch |

The jump-table bounds, actual branch destinations and immediate slot
arguments were checked together. An unchanged active state does not repeat
these notifications; that fact does not exclude a different repeating
mechanism elsewhere.

Evidence: constructor/context captures `ev_47a188c3f9d7fa9796b14afb54cdafaced79a1e4db5c5dcf60dcf65ffe128195`,
`ev_57b729ee833d41770066570079f6b1c3acd301f4bc96a0a9c01d329e69130f0a`,
`ev_e4828dfc87f4f2058dec275d1d2d42e2dfcc420a226465d3fd55db28ecb453e1`;
state setter `ev_78e9c65a7ddf76c7ab411465bcccb137dadbe65039ba95289062b954ec55ff9e`;
notification thunks/difference helper `ev_b7d52f7e6686283f04fbf48b2ba00d176c565120e6258dd16d6bf58ec009c69e`;
exact tables `ev_4753584023fe557f9c5ac932434a9650499e034db0ecb3dffd1a2e1ee641387a`;
return-zero stub `ev_6d47c3c19a294188f91d31ec5d944fa01f889227e6059361c0738093045e9893`.

## Activation:0x2a, validation, then explicit0x23

Activation dispatcher `0x1402ca180` resolves the controlled character,
maps the action to a slot, and calls `0x140316230(slot, 0, 0)`.
That helper obtains the character's primary skill-bar interface through
character virtual +`0xc8`. After player, slot and mode gates, it resolves
the target and calls skill-bar virtual +`0x1b8` at `0x14031665d`.

The constructor-established primary skill-bar table `0x142164cd8` binds
+`0x1b8` to `0x1411f51e0`. This wrapper reads the current slot SkillDef
through +`0xe8`, then invokes +`0x1b0` = `0x1411f5250`, passing the slot,
that SkillDef reference, the target and the original extra input argument.
The complete body through its return at `0x1411f5f3d` proves the following
ordering:

1. A nonnull ability with body +`0x70` low-byte mask0x80 clear and SkillDef
   +`0x38` mask0x1000 clear calls skill-bar +`0x220` at `0x1411f52cd`.
   This is `0x1411fb360`, which stores the slot at bar +`0x180` and invokes
   owning-character order-component +`0x218`: packed `[0x2a, slot]`, fresh
   outer flags3. The next +`0x228` call can separately update a changed
   target; it is not another0x2a method.
2. The body performs additional validation and local-state handling.
   Numerous branches can return after that0x2a. Examples include the
   bar's stored clock comparison, ability bit13, target/mode checks and
   skill-bar virtual admission methods +`0x30`, +`0x20`, +`0x18`, +8,
   +`0x10` and +`0x38`. These are not reconstructed as server policy here.
3. If a prior slot at bar +`0x74` is not sentinel23, `0x1411f5b78` calls
   +`0x1c8` to end it before storing the new slot and SkillDef. This can
   send0x29 during a new activation, without a physical release event.
4. Helper `0x1411f6570` can select an alternate target path through
   `0x1411faee0`. Its complete body has settings, character, skill-content,
   target and range-dependent gates. This slice does not assign it a
   universal result or assert that every activation reaches ordinary0x23.
5. The ordinary admitted branch reaches `0x1411f5e19..0x1411f5e42`.
   It obtains `[bar+0xf0]+0x4b0`, the owning character's order component,
   and invokes that component's +`0x1d0` with the retained target, slot
   and exact retained SkillDef reference. The established component table
   `0x142179b90` binds +`0x1d0` to `0x141283f90`, the explicit packed
   **SkillBegin0x23 producer**.

At the final call, RDX=retained target, R8D=retained slot and R9 points to a
local copy of the same SkillDef pointer supplied at entry. This is a typed
receiver edge, not an arbitrary call using offset0x1d0. Its explicit
SkillBegin producer and payload are retained in
[the skill command path](209026-mesmer-skill-command-path.md).

Cached sword rows10170/10171/10172 have SkillDef flags `0x04000006` and
ability +`0x70` Dword0. Consequently they satisfy the early0x2a branch's
two bit tests. On the ordinary admitted path, body bits17 and9 are also
clear, so the final target-dependent checks at `0x1411f5c30` fall through
to the explicit0x23 call. This statement is conditional on the preceding
live admission gates; it is not a runtime claim that any particular
fixture passed them. The exact cached body values are retained in
`ev_1139614cc70d841fc2a761ec769eb5e6b3f728ddec017c5ef4e0a23aeae2433e`.

The decisive complete body is
`ev_e5c189324f4c387639e51b49dfc671280c944f1d90e41ae42f45df51649a594e`.
The wrapper/activation route is covered by
`ev_4c225fb4860b1f5b90dfb0b8e9d4fbae2e853046386ff3bb39211997d7cdddc3`,
`ev_6244de6b9e4707155e255c4c7ec174eb268f7a1552d8ad152b9aae8e8b2b7d2e`
and `ev_b093d3e3cf807323e0a5800f5d0352388eecc6eed290ce83113ee9c71f7e815e`.
The alternate helper is complete across
`ev_fd868a18b58fcdbb153b2e12b7953e8ba63e0c1629696bd7cff8b544c1063e87`
and `ev_96a97cc805caa06f75d042ab9be60f944503bedff3eefa51a555a8971c9fbc1c`.

## Deactivation and other end-path callers

Deactivation dispatcher `0x1402ca420` maps the same ordinary actions to
`0x140316680(slot)`. After controlled-character/player and slot gates,
that helper calls primary skill-bar +`0x1c8` = `0x1411f5f40`. This wrapper
resolves the current slot SkillDef and invokes +`0x1c0` = `0x1411f5f90`.
The previous producer note retains that complete body: matching slot and
mode branches clear cached target/position state and invoke order-component
+`0x210`, producing `[0x29, slot]`; one later branch stores sentinel23.
Some paths return without a send. Neither this chain nor its payload is
a new SkillBegin request.

A separate source-bound `SbSlot.cpp` widget route corroborates the split.
Widget action handler `0x140877a90` can call `0x140316230` at
`0x140877cfc`. Widget endpoint `0x1408790c0` can instead tail-call the
same skill-bar +`0x1c8` end path. Its dispatcher selects those routes for
specific message/payload values, and also reaches the end path during a
destruction branch. Those UI message numbers are left numeric; they do
not justify naming every0x29 as an OS key-up packet. The prior-slot
replacement in the ordinary activation body independently establishes
that broader lifetime use.

Evidence: `ev_69324c67ebace92ff5dd41dbcf700fe85fd49c0aee88b78bc95e0bc9e03a0c88`,
`ev_9d53bf2417e78c27e398da53ff35d09e34fa8757d6641b28d02dd7912bab0520`,
`ev_66a3eb22624e658803d09782b20dfe54e8dcb6ceb2baf9700b37305df47507b5`
and `ev_96cd1e240741b1215412c0de93ac1f88ec9e4f0629a3a66c16a2e1da0d00ae4f`.

## Implementable boundary

For the current observer, preserve0x29/0x2a as distinct typed slot inputs,
and admit the existing explicit SkillBegin request independently. Do not
create an order, damage, an auto-attack timer, recharge or a chain increment
solely from either slot input. In particular, replacing a cast on0x29 with
a cast on0x2a would still manufacture an accepted cast before validation
and could duplicate the following0x23.

This establishes the correction to the earlier `RepeatStart` fixture
interpretation. It does not establish how ordinary automatic repeating
casts are scheduled after completion. No completion-to-next-cast loop was
demonstrated in this bounded slice; no absence of such a loop elsewhere is
claimed. Server628 chain selection, typed78 order lifecycle and real
damage/cooldown values retain their separate source boundaries.

## Private evidence and capture limits

The 47-record source/exploration bundle is
`/home/blizz/re/gw2/notes/209026-skill-input-action-lifecycle-20261106.json`,
SHA256 `83f2d3de59cea41a95c3655a97344f4a21193d21c3b78719f65d40ba905458ed`.
It retains two failed helper attempts: an address lacking the required
`0x` prefix and an overly small UI-table output cap. Both were corrected;
neither failed result supports a source claim. The other45 captures exited0.
All47 report nontruncated output and verified owned-process-group cleanup.
Some also report unreadable unrelated process environments during cleanup;
these are retained and are not represented as a machine-wide process audit.

The separate complete return-zero stub capture is
`/home/blizz/re/gw2/notes/209026-skill-input-numeric-action-stub-20261106.json`,
SHA256 `ec2b3056ae7f007ce4ce18c8f4391b20d4ee03a9c46afac41ba142919326d8e4`.
It exited0 with nontruncated output and verified owned-process cleanup.
Early bounded disassembly fragments, jump-table data and unresolved exact
xref searches in the bundle are not claimed as complete functions or
whole-client negatives. Source summaries above identify complete bodies
and the specific typed edges actually needed for the correction.
