# Build 209026: active quest initialization and lifecycle

2026-11-05. Bounded static trace supporting the Norn-first Mesmer starting
experience. The follow-up identifies **quest 154, goal 1** through its direct
map-379 reference and traces goal initialization. It does **not** establish
the retail startup flags, NPCs, boss scripts or server objective progression.

## Quest definition identity

The already identified `CnContext` vtable `0x14220e160` has virtual +0x220
pointing to `0x1412e1e60`. That method forwards its unsigned argument to
generic numeric lookup virtual +0x70 with **content type 57 (0x39)**.
The callers explicitly name the result `successorQuestDef`. The active-quest
removal assertion names definition +0x28 as `GetDataId()`.

Do not substitute map key379 or a type 68 key for this quest-definition key.
Type68 is separately referenced by each inspected quest definition at +0x38;
its complete gameplay role remains unresolved here.

## Server messages

All listed messages start with Dword **player index**, resolved through
character-context virtual +0x118, followed by the fields below. Dwords are
unsigned protocol varints. Existing extracted descriptor tables provide the
wire layouts; the following handler/helper traces supply bounded roles.

| ID | Handler / helper | Fields after player | Observed effect |
| --- | --- | --- | --- |
| 526 | `0x1412585a0` / `0x141200d30` | Dword quest data key | Resolves type 57, inserts the definition into manager collection +0xb8/count+0xc4 if absent, notifies; when matching the active quest, asserts `IsComplete()` and notifies it. Full collection semantics remain unassigned. |
| 527 | `0x141258610` / `0x141200e70` | Dword quest data key, Byte | Replaces the active quest using a resolved type 57 definition; stores the raw byte at new quest +0x88. |
| 528 | `0x141258680` / `0x1412010c0` | None | Requires an active quest, dispatches the manager listener list, and conditionally refreshes navigation. |
| 529 | `0x1412586e0` / `0x1412011b0` | Point3, Point3 | Copies the two complete 16-byte decoded point records to manager +0x118 and +0x100, then refreshes several consumers. These are not proven starting coordinates. |
| 530 | `0x141258780` / `0x1412011f0` | Dword quest data key | Requires a matching active quest data ID, clears manager +0x58/+0x80, dispatches listeners, destroys the old quest. |
| 531 | `0x1412588c0` / `0x141201370` | Byte, Dword successor quest data key | Resolves nonzero successor key as type 57 (zero means null), requires an active quest, stores guest-completion availability/successor and notifies. Names are from the explicit assertion. |
| 532 | `0x1412587f0` / `0x1412012a0` | Dword goal ID | Requires an active quest and forwards the goal ID to `0x1412008d0`; selects a goal definition and constructs its steps. See prerequisites below. |
| 533 | `0x141258860` / `0x1412012f0` | None | Requires active quest, conditionally dispatches a quest listener and refreshes manager consumers. |
| 534 | `0x141258980` / `0x141201420` | Dword | Zero becomes null; nonzero resolves type 68 through virtual+0x58. Stores manager+0x88 and clears +0xc4. Full role untraced. |
| 535 | `0x141258a10` / `0x141201440` | ArrayVarSmall of Word, capacity 64 | Resolves each key through the type 57 getter, compacts nonnull results, forwards adjusted count. Collection semantics untraced. |
| 536 | `0x141258b20` / `0x1412014e0` | ArrayVarSmall of Word, capacity 64 | Resolves keys through type 57 getter and compacts nonnull pointers, but forwards original wire count. Do not assume it handles missing keys like 535. |

Server 538 remains the separate quest-manager reported marker documented in
[content direction](209026-content-direction-startup.md). It does not create
an active quest by itself.

## 527 construction details and constraints

`0x141200e70` removes an existing active quest before looking up the new
type 57 key and asserting `questDef`. It allocates 0x90 bytes, installs vtable
`0x142165f58`, and stores the definition at quest+0x58 and owning manager at
quest+0x60. The raw message byte is stored as a Dword at quest+0x88.
The method then installs this quest at manager+0x58. The diagnostic text
`Quest Create: %S` names this construction path.

Definition+0x38 becomes manager+0x80; a previous +0x80 value can be retained
in +0x88. The constructor leaves active goal +0x8 null and step count +0x7c
zero. The follow-up below establishes completion-bit and goal-selection
constraints, but does not establish a complete retail raw-byte startup value
or observe the resulting objective display.

## Base-content census

A hash-checked scan of all 32 base containers found **767 type 57 records**
in 199 namespace IDs. Each record's +0x38 resolves to type 68. The private
catalog retains each quest data key, container, record offset, namespace,
index root field and referenced type 68 location/key, plus every input hash:

`~/re/gw2/notes/norn-quest-catalog-20261105.json`

This is the base catalog, not a reconstruction of the runtime-visible
registry. A namespace match or raw numeric coincidence must not be used to
declare a Great Hunt quest identity.

## Explicit lifecycle codec

`gw2_proto::quest::QuestLifecycleFrame209026` now encodes and decodes
527/528/530/532. Its operation enum exposes `Create { quest_data_id, raw_byte }`,
`Notify`, `Remove { quest_data_id }` and `SelectGoal { goal_id }`, with a
common `player_index`.
The codec preserves the complete unsigned Dword and Byte domains. It has no
default quest, state byte or gameplay ordering policy. Existing
`QuestManagerReportFrame209026` for 538 is unchanged.

Three new tests cover literal wire fixtures, prefix/stream boundaries,
truncation at every byte, neighboring message rejection and malformed
varints in either Dword position. Independent schema decoding against the
extracted character table checks the exact IDs, field types, order and
values. After adding 532, `cargo test -p gw2-proto --lib quest::tests` passes
all five focused tests, including both existing report tests.

These frames remain explicit. The caller must enforce existing-player,
active-quest, goal-selection and matching-removal prerequisites; wire
decoding alone cannot check them. This codec change adds no bootstrap quest.

## Direct tutorial quest links

The client traverses definition +0xa8/count +0xb0 with **0x70-byte** elements
at `0x1411fdde0`, `0x1411fe650`, `0x1411ffa40` and `0x1412008d0`. The latter
matches the incoming goal ID against element +0x10. Element +0x28 is a
classified relocation to a type-45 map record. The bounded batch over 229
quest definitions in containers 1282830–1282837 produced these exact links:

| Quest data ID | Definition container:offset | Associated type 68 | Goal ID / array index | Goal offset | Map key / target container:offset |
| ---: | --- | ---: | --- | ---: | --- |
| **154** | 1282830:4145560 | 2 | **1 / 0** | 4146200 | **379** / 1282833:3863896 |
| 11 | 1282833:4222528 | 6 | 1 / 0 | 4224120 | 138 / 1282833:3784456 |

Quest 154 has exactly one goal. Its goal +0x50/count +0x58 points to six
0x60-byte step definitions beginning at container 1282830 offset 4146312.
The content index labels the goal as type 341 and its steps as type 343;
these embedded types do not use the ordinary type-57 header layout.
In array order, step +0x14 IDs are **1, 2, 3, 6, 4, 5**, and step +0x10
flags are **0, 2, 2, 2, 2, 2**. The individual IDs and flags do not establish
the server's encounter sequence. Goal +0x18 is null with count +0x20 zero.

These are direct content references, not guessed numeric matches. Global
uniqueness was not tested beyond this batch. Decoding 116 type-57/type-68
metadata names in the first four containers produced opaque content names,
so those names were not used to identify the tutorial. The retained join is
`~/re/gw2/notes/norn-quest-goal-maps-20261105-0-8.json`.

## Completion flags and 532 prerequisites

Quest vtable +0x28 resolves to `0x1411ffa20`, which returns
`(quest+0x88) & 1`. The callers' assertions name this method `IsComplete()`.
Vtable +0x30 resolves to `0x1411ffff0`, which returns bit 1; its gameplay
meaning remains unnamed. A byte-pattern scan found no direct E8 calls to
either leaf in the executable's text section. This does not enumerate their
indirect virtual callers or establish meanings for higher flag bits.

`0x1412008d0` first asserts `!IsComplete()`. If a goal is already active,
it passes that goal's ID to `0x1412006a0` before asserting `!m_activeGoal`
and `!m_stepArray.Count()`. It then searches the definition's goal array for
the exact incoming ID, asserts a match, and installs that goal at quest+0x8.
For each step definition it allocates a 0x18-byte object, stores the definition
at +0x8, zeroes the Qword at +0x10, then writes runtime flags at +0x10 as
**0xa if definition flags bit 1 is set, otherwise 0**. Thus quest 154's
initial runtime step flag values are **0, 0xa, 0xa, 0xa, 0xa, 0xa**; each
object's remaining Dword at +0x14 is zero. Its gameplay meaning is not named
by this trace.

For an explicit research sequence, a fresh 527 with completion bit 0 clear
meets the inspected state prerequisites for 532 selecting goal 1. A later
528 can notify the manager after goal construction. This is a static
precondition argument, not proof of retail startup flags or runtime success.
**532 is not idempotent**: the existing-goal path completes/removes that goal;
finishing the last goal sets completion bit 0 at `0x14120085c`. Resending 532
for quest 154's only goal can therefore mark the quest complete. Do not use
it as a harmless repeated refresh.

## Remaining source work

Trace the bit-1 getter's indirect consumers before naming or defaulting the
full creation byte. Establish localized step meanings, NPC identifiers,
encounter placement, transitions, rewards and server scheduling separately.
This audit neither proves these are absent from the client nor claims client
assets reproduce the server scripts.

## Explicit observer experiment

`gw2-game-observe --mission-quest KEY,FLAGS,GOAL` adds an optional research
sequence to an existing mission population fixture. It requires all three
explicit values, a byte-sized flag value with completion bit 0 clear, and
`--mission-population`. The caller must verify the quest/goal content link.
No quest is selected by default, and the option supplies no encounter logic.

After population and client 412 readiness, the observer sends its existing
1036/431/83 sequence, then exactly one 527 Create, 532 SelectGoal and 528
Notify. The phase's once-only guard covers these frames; repeated MapLoaded
or AgentsLoaded events cannot reselect and accidentally complete the goal.
The delayed-population path also gates this sequence until population is sent.

The phase test checks ordering, explicit IDs/flags, early/duplicate readiness,
delayed population and completion-flag rejection. All 44 enabled observer
tests pass (two pre-existing capture-dependent tests remain ignored), including
existing Guardian/Norn TCP tests. The optimized musl observer builds, and six
quest CLI rejection cases fail before binding. These checks establish transport
behavior; the separate playback note records actual client results.

A bounded follow-up counted **162 distinct type 68 definitions** referenced
by the retained 767-quest catalog and inspected six associated records. It
did not establish a Great Hunt identity or name the additional raw fields.
The 15-second metadata sample reached its time limit while resolving
references; the owned process group was terminated and cleanup verified.
This is partial metadata inspection, not an exhaustive semantic search.

## Evidence

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
REA captured hash-checked read-only Python/objdump helpers, not the game.
The original five and all new follow-up captures exited 0 with owned-process
cleanup verified; the earlier metadata timeout is identified separately. Unrelated
unreadable `/proc` entries were left untouched. The standard sampled-process
and inherited-environment limitations remain.

Private bundle: `~/re/gw2/notes/209026-quest-lifecycle-20261105.json`.
The implementation follow-up evidence is retained separately in
`~/re/gw2/notes/209026-quest-lifecycle-codec-20261105.json`.
The goal-link and initialization follow-up is retained in
`~/re/gw2/notes/209026-quest154-goal-initialization-20261105.json`.

| Evidence | Observation |
| --- | --- |
| `ev_4bd8ea4e3606d26682c42f4fb30e639dbedcd4e192299dd7ab5142bb86cb406c` | Handlers 530–538 and named successorQuestDef assertion. |
| `ev_2a9a53d32e504b4bf7db92bfafe8036f1fcf202fa2cc798b9bdb9f1857890e24` | Type57 getter, removal/data-ID assertion, guest-completion state names. |
| `ev_e701a2dfe93ce9fc5d3829091f95c0fe1af084ab0f6ccd20e6614c6b0980a09b` | Handlers 526–529 and exact forwarding. |
| `ev_6e0996de963bfe7187b3c10040e1a3ba63dd6c14899378f54b1fa2f3dfd6bf74` | Complete bounded 527 construction, collection update, notify/point-copy helpers. |
| `ev_0b5f827c11baff44b1bb41a3e9980eb3ecd9d676462ff8a34dd570d287608f5c` | 767-record all-base quest catalog, +0x38 reference types, input hashes. |
| `ev_4f38c87c41b84c30aca70ed8f72b2f2e68c267a9d36da173e467540cf0ff728c` | Derived associated-definition count and six-record metadata sample; timed out with verified owned cleanup. |
| `ev_945ea6b0b705af0e18ddd5c0822d7da6fdcd01697228086f338a0d68c566e956` | 116 successfully decoded opaque metadata names in four containers. |
| `ev_e679bd5dd437b016ba8a0b91237291251c984c0ba2b201469903907b2e2410b5` | Quest vtable entries and the directly named step-array getter. Individual unwind fragments are partial where control crosses their ends. |
| `ev_391e081c2f44c2dc4086c7537de0314cb7886ffe890cb967128936231426c247` | Goal array/count/stride consumers and goal-ID matching. |
| `ev_8cb8fee476afecda5b6cc524a51d2cd3572aea286759d1472d8055d03cb252cb` | 229-quest goal-map join; quest154→379 and quest11→138. |
| `ev_38781654b454545ebff182fb280ca8f64e3d035f3abc51bc5a1f78d2367232c0` | Exact quest154, goal1 and all six step definition fields. |
| `ev_73a4d39659094be89ef9177d386f9674ca08284b865b28f20d7dd6a60526b56b` | Completion/bit-1 accessors and complete 532 goal-selection helper. |
| `ev_3ca27daae13351c5a6d71234e03028527116929ece7d8bdb23f4f63571502855` | Goal completion, last-goal completion-bit write and bounded flag-reader survey. |

The static captures did not start a game client or change live services.
The explicit observer experiment and its actual client limits are recorded
in [Norn playback](209026-norn-mesmer-playback.md).
