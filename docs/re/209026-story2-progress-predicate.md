# Build 209026: story 2 progress predicate

Static REA, 2026-11-06. A distinct authored edge from the
[Great Hunt reference audit](209026-great-hunt-species-link-audit.md) now has
a typed consumer: **story 2 +0x68 → progress 3863 → progress completion
predicate**. The record is BIT progress with channel mask 0x10. This does
not establish the event that sets the bit, the general gameplay meaning of
the story field, or an actor/action-to-step rule. No initial progress value
or new Great Hunt transition follows from this result.

## Exact cached record

The reader admitted only already extracted containers 1282830 and 1282833,
disabled archive fallback, verified their frozen hashes and checked indexed
record bounds and declared pointer classification.

| Source | Field | Target |
| --- | --- | --- |
| Type 68/story 2, 1282830:4127192, indexed extent 304 | +0x68, external relocation | Type 53/progress 3863, 1282833:4183760, indexed extent 160 |

The target has data type **0** at +0x2c and channel mask **0x10** at
+0x40. The already established [progress schema](209026-player-progress.md)
identifies these as BIT and enabled channel 4. The raw Dword at +0x30 is
4; it remains unnamed here and is not treated as a second channel selector.
The parameters reference at +0x38 is local to 1282833:4183872, but the
BIT predicate does not consume counter/bit-mask parameter fields. The
reader's small raw parameter window is not assigned a new schema.

Source hashes:

- 1282830: `e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f`.
- 1282833: `5ab847328dd307407d9701711890f08282f9c6f3639d24bb9684ea46006e26ef`.

The prior [starting-quest proof](209026-quest154-authored-start.md) separately
binds story 2 +0x58 to quest 154 and records 47 quest definitions in this story
group. Neither group membership nor this progress bit equates completion
of quest 154 with completion of the whole story group.

## Exact quest-manager consumer

Quest-manager primary table **0x142166030**, virtual **+0x78**, points to
the complete function **0x1411ffdf0..0x1411ffe70**. Its RDX argument is
named by the assertion **`storylineDef`** at 0x1421666d0; the source label
is `ChCliQuest.cpp`. The function:

1. Requires the story definition and reads its **+0x68** reference at
   **0x1411ffe1e**.
2. Recovers the containing player by subtracting 0x9820 from the manager
   receiver, then calls player virtual **+0x10**. Player table
   **0x14215e9d8** binds this slot to **0x1411bb240..0x1411bb247**, a
   leaf returning **player+0x9760**, the established progress manager.
3. Passes the referenced definition's numeric **Dword+0x28** to progress
   manager virtual **+0x28** at **0x1411ffe3c**. The existing
   [progress lookup proof](209026-reward-track-content.md) binds that
   method to 0x141231a20 and its key-to-existing-object lookup.
4. Returns false when no progress object exists. Otherwise it calls that
   object's virtual **+0x40** and returns whether the result is nonzero.

The progress table **0x14216c718** binds +0x40 to
**0x141232080..0x1412320ee**, which checks every enabled definition
channel through virtual+0x38 and returns false on the first failed channel.
The retained [completion trace](209026-progress-completion.md) identifies
the BIT branch as testing the corresponding stored bit. For this exact
definition, therefore, the predicate requires a present progress object
whose channel 4 bit passes. This is a read of existing progress, not a
write, notification, network request or quest-step completion operation.

## Encounter boundary and bounded coverage

This consumer names the argument and establishes the authored progress
reference, but its callers' full gameplay meaning remains unresolved. The
bounded QuestDataManager region 0x1414c2a80..0x1414c5c00 did not provide
a receiver-proven call to this manager predicate: its inspected +0x78 calls
use the separate UI manager's own index methods. Slot equality is not a
binding to the quest manager. This is not an exhaustive indirect-call search.

Server 509 can transport a known BIT state under the previously established
protocol. Nothing inspected here identifies **which actor, interaction,
kill, location or quest transition changes progress 3863**, or makes that
state an input to one of quest 154's six step IDs. No retail initial mask or
automatic story/quest completion rule is selected. This lead adds a precise
story-level progress query while leaving the
[action-to-step dependency](209026-quest154-objective-ui-boundary.md) open.
The prior fixed-map table and literal PE step-key searches were not repeated.

## Evidence

Every executable query asserted frozen SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Full private bundle:
`~/re/gw2/notes/209026-story2-progress-predicate-20261106.json`,
**193537 bytes**, SHA-256
**0e77008f2a6be0978261978c2e5184f951c4131f5116f12d4d685db84408fd1f**.

| REA evidence | Scope |
| --- | --- |
| `ev_51c119139e4c4dab2ea7e6a2fb03e255e435b059c1af677130c59542c4442c21` | Hash-verified exact story/progress pointer and definition fields. |
| `ev_51a724d8b03670d772e37d038beae06cbeaa4d46e89917d25903f2ffd70a1dee` | Complete named storyline consumer. |
| `ev_7995fe8ac82df0cdf79c8748df271b3030eb8c1c327f5db17aa55b1114582284` | Exact manager/player/progress table slots, player getter leaf and complete all-channel predicate. |
| `ev_a0d57d87b2e4d878171e4d97916a3a181cc1710b5fa7ae5afd78bc15f9f07f1b`, `ev_d2e700f4f809dfee5640b6c0a19198afcf0c63f3658b5b5b789feed11f9bd1b2` | Bounded quest-class and UI caller candidates; not global absence proofs. |

All seven captures retain complete raw output. One initial table query
stopped because the player getter has no `.pdata` entry; the subsequent
explicit 48-byte window establishes the leaf only through its first return.
Neighboring instructions are uninterpreted. Some cumulative rendered frames
were omitted. REA verified owned helper-process cleanup. No archive, game
runtime, container, resource lease, publisher state or gameplay code changed.
