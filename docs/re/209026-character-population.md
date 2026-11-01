# Build 209026: player records and character finalization

Static REA findings, 2026-11-01, on the frozen executable identified in
[mission transfer](209026-mission-transfer.md). No live population replay
was attempted; the User Agreement decision remains pending.

## Player creation: server 428

Protocol 0, group 0 server **428** has descriptor `0x1425c7d70`, handler
`0x141254ec0`, and fields **Dword, String(32), Guid, Dword**. Registration
is in the same table as [server 444](209026-character-control.md).

The first field is explicitly `message.playerId` in the assertion at
`0x142171900`. The handler requires that `context->PlayerFind(playerId)`
returns null, then calls `0x1411b7280` with all four fields. This allocates
and constructs a player object through `0x1411b9cc0`, grows the player
pointer array at context +0x80 if needed, clears newly added slots, stores
the player at that index, and also inserts a GUID lookup entry. It then
notifies the context's event collection.

This establishes how the player table used by server 444 can be populated.
The string and final Dword meanings are not yet established. The GUID is
used as a lookup key; it has not been equated with any auth/account GUID.

## Character factory

Character-context construction registers an engine-agent factory under
agent type **0**: `0x1411b2455` passes context +0x2d8 and type 0 to
`0x141023f80`. That helper's assertions name `factoryTable` and `agentType`
in `Engine/Agent/AgApi.cpp`. The callback is `0x1411b66e0`.

The callback allocates a 0xb08-byte character, constructs it through
`0x1411d2810`, obtains its index through virtual +0x190, grows the character
pointer array at context +0x60, and stores the new character at that index.
It returns the character's +0x48 interface. The engine message that invokes
this factory and the initial component data remain unresolved.

## Finalization: server 170

Protocol 0, group 0 server **170** is registered at pair `0x14216eff0`, with
descriptor `0x1425d89f0` and handler `0x14124ccf0`. Its only field is a
**Dword**, passed to the raw character-index lookup `0x1411b4500`.
The handler requires a nonnull character and invokes `0x1411da260`.

Before setting the finalized flag, that function checks:

- `!(m_flags & FLAG_CREATE_FINALIZED)` — finalization has not already run.
- `m_attitudeTowardControlled < Content::AFFINITY_ATTITUDES` — the inspected
  signed comparison uses an upper bound of 4.
- `IsPlayer() || IsMonster()`.
- `!IsPlayer() || GetPlayer()` — a player character requires its player entry.

It then sets character +0x178 mask 0x10, invokes component callbacks,
performs player-specific updates, and calls character-context
`0x1411b61f0` before further character updates. These are real initialization
dependencies; the packet is not merely a permission to flip one flag.

Together with server 444's setter/getter checks, this supports a static
dependency: an existing, initialized character and its player association
must be finalized before controlled-character readiness. It does not yet
provide the full sequence of messages or content values needed to enter a map.

## Offline validation

The hash-guarded extraction found exactly server 428, 170 and 444 with the
expected descriptors and handlers. It wrote a private 0600 schema at
`data/protocol/209026/character-population.xml` (675 bytes, SHA-256
`d5a7734b8484d2fd4238afb22707cb3d5b818bd24835669165a917b48853a5eb`).

The existing census binary parsed a hand-authored concatenation completely:
ID 428 at offset 0 (24 bytes), ID 170 at offset 24 (4 bytes), and ID 444 at
offset 28 (6 bytes), total 34 bytes. No messages were sent. This validates
descriptor translation and framing only; it omits required character
creation and initialization and is not a valid bootstrap sequence.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-population-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_2c304c1edcd41414586b2ea9cf81d2d16622dcf64edffb2a021db742f4ccc8e1` | Server 428 layout, handler, allocation and player-array insertion. |
| `ev_40981d4fc56b71b43d77abac48501688bca653bdc313768f7a1f2b4607e27ac6` | Character factory allocation and character-array insertion. |
| `ev_2456454d93476462f13b4e8c943a36d65901f93e71ebe6305655d3a130749a3d` | Type-0 factory registration and finalized flag write. |
| `ev_6d8ab3bf150e74ed6db71a56327d58ca69a6dce9be3d5420cfaaddc889dc4e53` | Factory-table registration helper and finalization callbacks. |
| `ev_279304f8620dc2fadffe76058c7dbf137f3e7a47c90ef3f6279f0c1a96baa937` | Finalizer entry and named prerequisites. |
| `ev_64c5608920f831930790b7a5b34449d6a7f40aad3646b349856e178e2ad0d685` | Finalization message handler and table pair; player association assertion. |
| `ev_e7f3dccffcc190ab1fd206a99c442b72d69ca9f60a0304821c4aa0a919b83e9e` | Server 170 descriptor and explicit playerId diagnostic for 428. |
| `ev_c8e41b939931a672055d0ac259bfdb8069af7a11091da5b31a93d9ddeb13ad69` | Hash-guarded extraction of the three definitions. |
