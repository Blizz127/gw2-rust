# Build 209026: ordinary NPC character foundation

Static REA analysis, 2026-11-05, of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This extends [agent creation](209026-agent-creation.md), the
[character finalizer](209026-character-population.md), and
[view finalization](209026-character-view-finalization.md).
No NPC identity, placement, stats or encounter behavior was chosen.
No client launch, packet replay or implementation change was performed
for this investigation.

The ordinary nonplayer character path uses **server 37, agent type 0,
identity `0x20000000 + species key`**. The client resolves the key in
numeric content **type 67** and stores the result as `m_speciesDef`.
The constructor also creates its profession component. This is a
different initialization path from a player character. The client's
`IsMonster` classification alone does not establish hostile attitude,
Great Hunt membership, or an authored spawn.

## Identity and content resolution

The existing server-37 trace establishes field 5 as the agent-type byte,
field 6 as the identity Dword, and field 0 as the separate agent-index
Word. Field 6 reaches base-agent +0x40, then character +0xa0 through
base-agent virtual +0x118.

Character primary vtable `0x142160be0`, slot +0x410, resolves to
`0x1411d9540`. It returns:

```text
(character.identity & 0xf0000000) == 0x20000000
```

The named assertion `character->IsMonster()` at `0x1421716d8`
is checked immediately after a call to this slot in `0x14124f900`.
This binds the numeric discriminator to the client predicate, rather
than inferring a name from its bit pattern.

In character constructor `0x1411d2810`, the nonplayer identity branch
at `0x1411d2d88` subtracts `0x20000000` and calls content-context
virtual +0x248. The constructor-established content vtable entry
`0x14220e3a8` resolves to `0x1412e1f00`, which calls the generic
numeric lookup with type `0x43` (67). The returned pointer is stored
at character +0xa8 and checked against the literal `m_speciesDef`.

For an ordinary identity with high nibble 2, the remaining 28 bits
therefore identify the species definition. An arbitrary integer is
insufficient: the species lookup must succeed. The species key is
content identity; the message's separate agent index identifies the
agent instance. The earlier raw character-index lookup relationship
must also be respected by later component messages.

The same constructor's high-nibble-2 branch at `0x1411d2e39`
allocates 0x48 bytes, constructs through `0x141288bd0`, asserts
`!m_profession`, and stores the result at character +0x510.
That constructor calls profession base constructor `0x141240720`
and installs vtable `0x14217a290`. Sending the player's Mesmer
profession creation message would attempt to create a component that
already exists on this path. NPC species identity does not imply
the player's race, appearance or Mesmer profession.

## Ordinary model and view path

The existing finalization trace establishes this chain:

```text
server 170 -> character finalizer 0x1411da260
 -> context finalization listeners
 -> AvContext 0x141110570
 -> eligibility 0x141111ad0
 -> ordinary view factory 0x14110f150
 -> view initialization and model component at view +0x820
```

Eligibility returns true immediately for a nonplayer identity; the
server-428 player-state gate is on the other branch. This does not
prove that every later view callback or asynchronous asset request
succeeds.

View definition getter `0x1411087a0` first considers an override
at view +0xb50. Without that override, it returns no ordinary species
record for the player or the separate clone-predicate branch. On the
ordinary nonplayer branch it calls character primary +0x300
(`0x1411d87b0`), which returns character +0xa8.

The model-loading route `0x141169ff0` calls `0x141166a80`
for its mode-0/mode-3 branch. The latter is anchored by
`Game/AgentView/AvCharModelHelpers.cpp` and asserts `speciesDef`.
It obtains the species record from the view and considers separate
description/appearance overrides. When it selects the direct species
branch at `0x141166db4`, it reads species +0xf0, passes it through
`0x14130c150`, and supplies that result plus species +0x120 to
loader `0x140cf7000`. A null transformed filename is replaced
with an empty filename constant. That fallback is not evidence of
a usable visible model.

This branch provides a concrete path to a species-authored model.
It does not directly require the player's 45-byte coreStats appearance
or wardrobe setup. Other species/override/model modes and downstream
view consumers remain separate dependencies.

The [reveal trace](209026-character-view-reveal.md) establishes that
collision reveal callbacks use view +0x820 without a null guard.
Consequently **create and finalize the character before entering its
ordinary collision/reveal path** remains the supported ordering.
Server 46's world entry and server 49's scene entry have distinct
roles; see [agent reveal](209026-agent-reveal.md). Allocation of the
model component and successful loading of its model assets are
different conditions.

## Component prerequisites: conditional, not a copied player fixture

| Operation or component | Established requirement for this path |
| --- | --- |
| Server 37 character creation | Type 0; high-nibble-2 identity whose type-67 lookup succeeds; explicit valid command, flags, position, facing, dimensions and radius. No NPC values were derived here. |
| Server 151 attitude | Finalizer requires the stored attitude to pass its signed comparison against 4. Attitude meanings and the appropriate tutorial value must be sourced independently. |
| Server 170 finalization | Character exists, has not been finalized, and passes attitude and player-or-monster checks. It supplies the ordinary view-creation event. |
| Profession | Constructor supplies the NPC component at +0x510; do not duplicate player profession initialization. |
| CoreStats 182 or 183 | Direct finalizer calls the coreStats callback only when character +0x388 is nonnull. This is not proof that every target UI or gameplay consumer tolerates absent coreStats. These two messages are alternative creators. |
| Inventory 282 | If coreStats is present, its finalization callback requires inventory at character +0x3f0. An empty inventory has a known constructor state; sufficiency for all consumers is unproven. |
| Skill bar | Direct finalizer invokes its callback only when character +0x520 is nonnull. NPC skill/action content and later consumers remain open. |
| Health 256 | Known health creator/updater; stores the supplied state separately from the component. Alive/dead/downed state participates in selection. Valid tutorial health values and all listener ordering requirements remain open. |
| Player 428 / owned 443 / controlled 444 | The NPC identity/finalizer branch does not require these player association operations for the NPC. The surrounding local-player context still exists and is used by selection. |

The new finalizer trace covers `0x1411da260..0x1411da38d`
and its nonplayer tail at `0x1411da476..0x1411da498`.
It checks coreStats and skill-bar pointers before their callbacks,
always calls embedded chatter finalization at character +0x330,
and skips the player-only callbacks before dispatching the context
finalization event. It then tails to `0x1411e3ab0`.

The inventory requirement is established in
[finalization dependencies](209026-character-finalization-dependencies.md).
The [health trace](209026-character-health-endurance.md) separates
component presence, state storage and listener callbacks; a null check
in one finalizer is not a whole-client permission to omit a component.
The [coreStats trace](209026-character-core-stats.md) establishes
nonplayer level bounds but does not choose valid NPC profession/stat
or appearance inputs.

## Target selection is an additional gate

The selection-context constructor `0x1410f8dc0` installs vtable
`0x14213fed0`. Slot +0x10 resolves to `0x1410fb9b0`;
its inspected caller checks the result against the assertion
`!agent || AgentCanBeSelection(agent)` in `AsContext.cpp`.
Context mode 0 tails to `0x1410fba30`; mode 1 rejects selection.

The complete helper `0x1410fba30..0x1410fbe4f` includes a
distinct type-0 character branch at `0x1410fbd1f`. Alongside
context/range/relation checks, it:

- Requires an existing character from the agent.
- Rejects when character primary +0x338 (`0x1411d8f50`)
  returns true. This delegates to a context service; its gameplay
  name and complete service semantics remain unresolved.
- Rejects when primary +0x480 (`0x1411d9a30`) returns true
  for the supplied local context. This consults transformation/species
  state, a mask-0x08 test, and context-dependent membership checks.
  The mask is not yet assigned a public flag name.
- Rejects when view lookup `0x141107840(character)` is null.
- Applies further relation-dependent checks involving primary +0x490,
  the dead/downed predicates, and separate character/context state.

The +0x480 implementation uses an override definition's +0x100
when that branch is selected; its fallback uses character +0x264.
It must not be reduced to a single universal packet targetability bit.
Primary +0x490 (`0x1411d9ba0`) returns false for players and
otherwise tests bit 27 from another character flag getter.
The [health predicates](209026-character-health-endurance.md)
identify dead as state 1 and downed as state 2; the admission policy
also depends on relation/context, so neither state alone describes
all selection outcomes.

Target geometry helper `0x1410fcb60` separately resolves the
character view, asserts `avCharacter`, and queries its anchors.
A valid character record alone therefore does not supply the view
needed by selection and aiming. This bounded trace does not establish
combat legality, target UI component completeness, damage processing,
AI, death credit or quest progress.

## Next concrete source work

A first encounter fixture needs a **source-selected type-67 record**
associated with the Great Hunt, followed through its model references
and character/species flags. Record existence or an English display
name alone cannot establish tutorial membership. The selected record
must then be paired with sourced creation dimensions, attitude,
stats/health and authored placement, or with an explicitly identified
diagnostic placement.

Before treating a candidate as a targetable enemy, trace the remaining
selection-service gate and target UI component getters, then verify
the component ordering with that specific species. Keep the NPC
construction path separate from the equipped-player fixture.
This investigation establishes the structural foundation, not a
ready-to-send encounter sequence.

## Evidence

Private bundle:
`~/re/gw2/notes/209026-npc-character-foundation-1.json`
(19 complete REA Evidence records; static, target-free captures).
The bundle retains command manifests, output, limitations and cleanup.
All included process captures exited successfully. Explicit disassembly
windows and PE unwind fragments have bounded coverage; only the
identified branches above are attributed to their functions.

| REA evidence | Observation |
| --- | --- |
| `ev_084a70dd1894bd88b046e7a8508a2fdb6ea4553d912a0d119d59d2f1b27016fd` | Named monster/clone assertions. |
| `ev_12f8f4e9c45ca6d4394aaf7873a2b175a0f64ed5e36e4ed8fd70f072d7cefc64` | Named IsMonster consumer calls character +0x410. |
| `ev_a5691bdab50e232b8a329fb63b54d2ec6787323f1b272ae5399afdb403dc3c15` | Identity discriminator and species getter. |
| `ev_ac4d9fea5e05b1982829f162df797dfa299cc1320f4a7de9a3cf752eb161b6fa` | View species selection and constructor initial state. |
| `ev_84f856629ed471bc3d01e9519edcec76dbd1ef7543d8ad8e3a4812e45b7e6d24` | Species-key lookup and automatic profession branches. |
| `ev_43b9f09dab414c510956edb8d3f7f0d8209b2c6cdf48e7941091b3385f24990c` | Type-67 numeric lookup binding and named component assertions. |
| `ev_5ace9274f45bfce60bc33c732ed0c2ef255773a2c1ec9c605b7d3f1aa7279dbc` | Profession constructor and conditional finalizer component callbacks. |
| `ev_3ff7e82d37ab9a3437f354875f9055e11d7256e5bfb061d7fefa0c612b67f200` | Model loader route and nonplayer finalizer tail. |
| `ev_edbf2a18ed7008556d493a6583a3a33c3a6cb32b32bfd5c65db995e5640155bf` | Species model builder and direct species +0xf0/+0x120 branch. |
| `ev_2330ee651d6b676c12d885f19ea1208367bcafbc7b277e77032ec00cd498cb3b` | Model helper source, species assertion and getter table entries. |
| `ev_f8edf1ae793ce3c41f9293da4261918a2ba4f20ba5cd1d67f398c24db8e922d8` | Selection assertion caller and character-view geometry consumer. |
| `ev_64d93bfd7afef8dbc45e25a83638da46612d861f48e2c7dd897e55cec47b45a7` | Selection constructor installs the traced primary vtable. |
| `ev_412288d213ca362926d24be974071d7539176ed98e0195e1b6a4fd0bc2da04db` | Selection mode wrapper. |
| `ev_d2656858403776d8cec6dd51d7fafb7f2e19bebf96475f42359a042b10d15973` | Complete admission helper, including missing-view rejection. |
| `ev_4b724977f5380e36bdeb8bb972341f2cfec8a793c0aef4333caefa265a95cafa` | Character selection gate table bindings and implementations. |
| `ev_3c90e560c0899291da05d675b43a93d48d077f61eea6a68ba1b3b797701e244f` | Complete +0x480 continuation and character +0x110 getter. |
