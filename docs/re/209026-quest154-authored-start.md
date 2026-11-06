# Build 209026: authored starting quest for story 2

Static REA, 2026-11-06, against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`
and an existing hash-verified extracted content container. No client runtime,
shared runtime container, archive, publisher, or gameplay state was accessed.

**Type-68 story definition 2 identifies quest 154 as its starting quest.**
The native assertion names the field `storyDef->GetStartingQuest()` and
the actual consumer reads story +0x58. The cached record points exactly
to type-57 quest 154, which points back to story 2 at quest +0x38.
Combined with the earlier exact goal-to-map-379 link, this strengthens the
authored tutorial identity. It does not identify a spawn, NPC, action trigger,
or the timing/order of the six tutorial objectives.

## Selected gap and concrete consumer

The [species-link audit](209026-great-hunt-species-link-audit.md) left an
uninterpreted reference from associated type-68 key 2, field +0x80, to
type-68 key 9. That was selected for a bounded follow-up rather than
repeating the exhausted all-container species/name scans or mission-point
caller searches. The role of +0x80 remains unresolved; this pass instead
establishes the adjacent starting-quest relationship through a named consumer.

An exact `storyDef` string query found the assertion at **0x1422f28a8**,
`storyDef->GetStartingQuest()`. Its sole matched disassembly reference is
**0x1414c444c**, inside complete function
**0x1414c43e0..0x1414c451e**. The source label at **0x1422f2800** is
`Game/Ui/Services/QuestManager/QuestDataManager.cpp`.

The function follows its virtual +0 result, then that object's virtual
+0x10 result. It reads the latter record's **+0x38** into RBX; the assertion
below identifies RBX as the story definition. The named story branch requires that pointer,
its +0x38 reference, and predicate **0x1411c9100** to be nonzero. It then:

1. Tests Qword **[RBX+0x58]** at **0x1414c4438**.
2. Emits the named starting-quest assertion if that pointer is null.
3. Passes **[RBX+0x58]** to its own virtual **+0xb0** at
   **0x1414c445e–0x1414c4462**.

The complete predicate `0x1411c9100..0x1411c9148` tests mask **0x7f**
in the story's referenced +0x38 record, Dword +0x28. This pass does not
assert that this UI branch is selected for the current tutorial runtime.
The assertion and the exact tested/read field establish the authored field's
meaning independently of that runtime predicate.

Constructor prefix **0x1414c2a80** installs primary table **0x1422f2558**.
Its +8 entry is the consumer above; +0xb0 is
**0x1414c5140..0x1414c5262**. That complete method looks up the story
group, finds the quest pointer within that group's quest array, and returns
`group_index * 1000 + 1 + quest_index`, or `0xffffffff` when not found.
It computes a UI data identifier. It does not start a quest, spawn actors,
advance a step, or send a server request in the inspected body.

## Exact cached content join

Source container **1282830** was read from its existing extracted cache,
SHA-256 **e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f**.
The record types, keys, bounds, and declared local relocation membership
were checked before following the two fields:

| Record | Content offset | Field | Exact target |
| --- | ---: | --- | --- |
| Story, type 68/key **2** | **4127192** | +0x58, starting quest | 1282830:**4145560**, type 57/key **154** |
| Quest, type 57/key **154** | **4145560** | +0x38, story | 1282830:**4127192**, type 68/key **2** |

Story 2's next indexed record is 304 bytes later; the examined fixed span
through +0xcf fits inside that bound. The earlier
[quest content trace](209026-quest154-content.md) independently binds its
sole goal 1 to map 379 and records the six step definitions. No text was
decrypted or inferred from the numeric key in this pass.

The retained all-base quest catalog, read without another container census,
contains **47 quest definitions** whose +0x38 targets story 2, and **53**
whose +0x38 targets story 9. These counts distinguish a story grouping
from an individual tutorial objective; their catalog order is not a recovered
progression order. Catalog artifact:
`~/re/gw2/notes/norn-quest-catalog-20261105.json`, SHA-256
`e958830f6c23178c299f86f5b5632cd8c75f38c6d16b474b448e1b076dfd9ba9`.

## Unresolved +0x80 link and encounter boundary

Story 2's declared external relocation **+0x80** still targets
**1282835:4241824**, previously classified as type-68 key **9**.
A bounded inspection of the identified QuestDataManager method region,
`0x1414c2a80..0x1414c5c00`, did not establish a receiver-proven read of
that story field. The observed +0x80 operands include manager fields,
stack slots, virtual-table slots and other record fields; offset equality
alone does not bind them to this story reference. No global offset scan
or full subscriber/caller census was attempted. The field is **not named
“next story,” “next quest,” or “prerequisite”** on this evidence.

The existing cached reader had no extracted container matching its
`content-type-*-file-1282835.pf` path. Its archive fallback was explicitly
disabled and raised before any archive access. A subsequent names-only
`rg --files` under `~/re/gw2/notes` found no path containing `1282835`.
This is a cache-location limitation, not proof that no retained record
projection exists under another name. The failed attempt is preserved.

The concrete gained link is therefore **story 2 → starting quest 154 →
goal 1 → map 379**. It does not close the
[action/actor/condition-to-step boundary](209026-quest154-objective-ui-boundary.md).
Neither the unnamed story-9 link, the story-group counts, sector references,
nor generic fixture combat supplies a retail Great Hunt encounter trigger.
No new actor, placement, quest state, or gameplay default follows.

## Evidence and limits

Private bundle:
`~/re/gw2/notes/209026-quest154-authored-start-20261106.json`, 379087 bytes,
SHA-256 **27c8b1e53718a61b35a55ba3cd3bc100ca2374aa2970c93f2d8571656924dc3f**.
It retains all 17 captures. Capture 3 intentionally exits one at the
forbidden archive fallback. Capture 12's contextual candidate output was
truncated; capture 14 supersedes it with the complete 37-line bounded
candidate list. No absence conclusion depends on the truncated output.
Other raw captures are complete; some duplicate rendered terminal frames
were omitted by their display budgets. No runtime validation is claimed.

| REA evidence | Observation |
| --- | --- |
| `ev_05b77c5653c81b833f05a3fe0c6921b588f6a885bd926cdac1076729bc18e6a8`, `ev_0606b9c7359e09e776b3e6cd60b5590db4b0e251537428b2cdf00de8854cab96` | Exact assertion seed and instruction reference. |
| `ev_a9a00ad9b8d18a27b4ac662fc9aae68f6af9d5b80ae533f40a5fa8cbc7ed82e6` | Complete named starting-quest consumer. |
| `ev_418e54834749c387c5edab34bb2b1f631d6114cc12371960731d36e5105e6550`, `ev_785381da49d8345d1465e960f7cc2ceb7f51096cf617423c5047bb8857a227f9` | Source label, consumer table reference and complete story predicate. |
| `ev_dfa9375b8f3bedb1f1bcaa355ab2253e878a08438ebbc3b4016476c996b04205`, `ev_59dc2b330790cb2b3deee284282c4a4b6d939c26b16807a1912ffd878de2ad9f`, `ev_b8ef6b617446721f1717385e7f4741af19330ce991ccb8aec72edba677f71f00` | Constructor table installation and concrete +0xb0 UI identifier method. |
| `ev_3d7781ca515c875fe17c53759ce0737ab6ef7afd3903a030195000ad2957c7c3`, `ev_b0f8d7bb3cbd1f5fe4c8e3c68cd1df11b9bca406101428b477bc889ec3cd2c1e` | Cached story fields, exact bidirectional story/quest link, source hash and retained catalog counts. |
| `ev_540d1aa21c74168bc8f871ea0b0d8c39314824b007dca6ba804a30ae8b9a86fe`, `ev_76251c43f061258c87559b658402fe4863bd479b35cbeb8b7a2c4ea835c3aa5f` | Bounded +0x80 candidates and concrete examples with distinct receivers. |
