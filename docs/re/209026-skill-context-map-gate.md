# Build 209026: map gate for character skill context overrides

Static REA, 2026-11-06, against the frozen executable and exact cached
map 379 definition. No client runtime, container, archive or gameplay state
was accessed or changed.

**When frozen map 379 is the installed current definition, primary character
virtual +0x220 returns fallback sentinel 4, and +0x228 resolves context 1.**
The map's override-enable bit is clear, so the related-character and buff
queries are skipped regardless of their values. This closes the map-379
character-override uncertainty left in the
[tooltip adapter](209026-tooltip-character-context.md) and
[cast-tooltip](209026-mesmer-cast-tooltip.md) notes. It does not establish
the same result before the map is installed or for other maps.

## Outer gate and exact map field

Character primary virtual +0x220 is the complete logical function
`0x1411d7b50..0x1411d7c26`. Its first operation is
`0x141411600..0x14141166f`; a zero result immediately returns 4.

That gate reads the literal **ScriptSkillEnableBuffModeStatuses** at
`0x1422dd5d0` using `0x141414340`, with fallback argument 1.
The complete settings helper checks a global lookup, then the mission's
settings collection, and returns the supplied fallback if neither has a
value. No actual setting value needs to be assumed for the map-379 result.

If the setting evaluates to zero, the gate returns zero. Otherwise it
obtains game-context +0x1c8 and reads that object's +0x1e8 pointer.
This is exactly the current-map-definition chain returned by
`0x141411100..0x14141111c`. If the map pointer is null the gate returns
one; if nonnull it returns **bit 13 of map Dword +0x9c**:

```text
gate = setting != 0 && (currentMap == null || (currentMap.word9c & 0x2000) != 0)
```

The bounded reader selected only the retained indexed map-379 row:

| Field | Frozen value |
| --- | --- |
| Container / index / content offset | 1282833 / 18427 / 3863896 |
| Type / data key | 45 / 379 |
| Dword +0x9c | **0x00000001** |
| Bit 13 | **Clear** |
| Map type +0x2c | 7 |
| Reference +0x100 | Null |

Thus, with that current map, the gate is zero whether the named setting is
zero or nonzero. +0x220 returns 4 without inspecting a related character,
buff context or status entries.

Primary virtual +0x228, `0x1411d7c30..0x1411d7c4f`, invokes
`0x14140f1d0` on sentinel 4. The already established type-7/null-+0x100
path returns 1 for this exact map. For skill 10176, that selects the
[base recharge field 20000](209026-mesmer-tooltip-facts.md), rather than
the context-2/3 overrides 25000, before any separate recharge-display
modifier. This is content selection, not an authoritative server recharge.

## What happens when another map enables the gate

The remaining +0x220 branches are now bounded precisely:

1. Character primary virtual +0x268 returns a related character.
   Its implementation `0x1411d80b0..0x1411d80d5` looks up the index
   stored at whole-character +0x1c8 through character-context helper
   `0x1411b4500`. The relationship's gameplay name is not assigned here.
2. If nonnull, that character's virtual +0x220 is called recursively.
   A result other than 4 is returned immediately.
3. Otherwise the caller obtains its constructor-owned buff context through
   character +0x40 virtual +0x48. Vtable `0x142161518` binds this to
   `0x140400100`, returning whole-character +0xd0.
   The adjacent +0x50 slot has the same getter, consistent with the earlier
   [buff-context note](209026-character-buff-context.md).
4. Buff-context virtual +0x60 is queried with numeric selectors 23, 24 and
   25. Their return values have priority in that order: 23 nonzero gives
   context 1; else 24 nonzero gives 2; else 25 nonzero gives 3; all zero
   gives fallback 4.

The numeric status names and their wire producers remain unassigned.
They are not character construction bytes or unconditional defaults.

## Constructor state of the queried buff table

The buff constructor `0x1412c2560..0x1412c279f` installs vtable
`0x1421841a8`. Its virtual +0x60 is the complete predicate
`0x1412c38d0..0x1412c397a`.

This method first checks Dword +0x84. Zero returns false without a lookup.
For populated state it looks up the numeric selector in the collection
starting at +0x80, checks the selected entry, and conditionally performs
another lookup with selector 14. The exact meanings of the entry's two
compared Dwords are not established here; the predicate is not reduced to
simple membership.

At `0x1412c26a3` the constructor stores Qword value 8 at +0x80,
therefore Dword +0x84 is zero. It also clears pointer +0x88.
The demonstrated constructor state makes all three queries false before
later population. The constructor's owner subscription was already traced
as insertion without immediate dispatch in the buff-context note. This is
a secondary construction fact; the map-379 conclusion does **not** rely
on the table staying empty after later callbacks.

## Evidence and limits

Private bundle:
`/home/blizz/re/gw2/notes/209026-skill-context-map-gate-20261106.json`,
270,211 bytes, SHA-256
`4fdadce876071a339f3830db39b7910a06b259113386ef6d9031687999ec9d14`.
It retains 12 new captures, the prior complete selector/fallback capture
and the cached-map reader. All captures exited zero with complete raw and
normalized terminal frames; omitted duplicate rendered views are recorded.

| REA evidence | Scope |
| --- | --- |
| `ev_4df6c42281206b3bb6219fc6a88b357493f63cb073a9c37c37f334eb9592cc03` | Complete gate and related-character lookup wrapper |
| `ev_9dbab58bdd168aa2772e6e877310e85fde4dfa7009b73cb9664ed56f650dcbf9` | Exact named setting |
| `ev_848557293fb963f6b21b86e70286153bdbe39b360bebe2026f1fe8a1ba6212cb` | Same current-map getter and buff-context getter |
| `ev_0dc240aea6ca90a91a6d827b2f2ad6d643989ebb973d3888fddc6b2ec1eec9c3` | Exact map 379 +0x9c, type and null fallback reference |
| `ev_647f43330b965d96c946596f03328b6799ed314253ca9450f9c34d00a2bac52c` | Complete named-setting lookup and fallback |
| `ev_e299c05d299e4c39a974d2937dc7bbb1fd32e6f89d740c6ae0ed1a7a20d5e589` | Complete buff-status predicate |
| `ev_39c1fdb6682674ea7fe201b30b299c89ec3b13921ce80212607fa9cca89d0625`, `ev_15e39c90095080561f22139bfcd2218dcaae09601070d029ce871c53f19604f1` | Constructor through its return, including table initialization |

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Cached map-container SHA-256:
`5ab847328dd307407d9701711890f08282f9c6f3639d24bb9684ea46006e26ef`.
No fallback/default or recharge duration was added to emulator code.
