# Build209026: creation and world joins on one listener

2026-11-05. Static conclusions below reuse complete retained REA evidence for
frozen executable SHA256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No new client run or launcher change is claimed.

The explicit combination of `--creation-scene WORLD_ADDRESS`,
`--mission-map-key 379`, `--character-store DIR` and the saved Norn profile
serves creation and world connections through the same mission listener.
Both auth destinations and the creation continuation can therefore use the
launcher's existing `127.1.0.4:6112` game route.

## Exact discriminator and its limits

After validating the existing 84-byte mission prefix, read the little-endian
Dword at bytes24..28. It is the destination map key:

- Map0 selects the creation scene and its1049 opening.
- Map379 selects the configured Norn world. Its selected GUID at bytes48..64
  must resolve in the explicit account's character store before any key reply.
- Other maps fail. A missing/zero/unknown world GUID never selects creation.

This is a bounded local profile, not a claim that map0 universally names a
retail server mode. The creation route constructor clears its96-byte payload
and sets game type1, leaving its map key zero. The local auth reply echoes
that map key; the client's join path carries it to preamble24. The world
profile and creation1065 continuation explicitly use map379.

The selected-GUID global is independent of the creation route payload.
The inspected creation constructor does not establish clearing that global;
requiring a zero creation GUID would add an unsupported restriction. Neither
GUID emptiness nor GUID lookup failure is the phase discriminator.

Offset20 remains opaque and is not validated as a zero or an enum.
Standalone creation and existing standalone world/observer modes retain
their previous dispatch behavior; strict map dispatch applies to the explicit
combined configuration only.

## Offset16 is not the auth game type

The earlier coincidence between tutorial game type9 and a proposed redirect
scalar9 does not establish a relationship. The exact data flow is:

1. Auth server20 consumer `0x14023ae60` stores field1 at route transaction+40,
   field2 at+44 and field3 at+48. The echoed request meanings are game type,
   reserved0 and map key, respectively.
2. Completion `0x14023e100` emits event0x23 with96 bytes beginning at
   transaction+38. Therefore event-result+0xc is transaction+44, not+40.
3. Route consumer `0x141418123` loads result+0x10 into `r9d` (map key),
   and `0x141418127` loads result+0xc into `r8d` (the reserved scalar).
   It calls `0x141417b80`.
4. That common helper stores `r8d` at context+158 and `r9d` at+15c.
   It forwards them as the first two arguments to `0x14023c990`.
5. The join constructor stores those arguments at join transaction+40/+44.
   Join start `0x14023e740` copies them to preamble16/24, respectively.

Thus preamble16 receives auth server20 field2, not field1/game type.
Creation server1065 field2 also reaches preamble16 through the common join
helper. The observer now sends explicit research0 in that field, matching
the ordinary echoed auth route's reserved0. Its former value9 was based on
the wrong game-type interpretation. The field's broader retail semantics
remain unresolved. The1065 map Word remains379.

## Focused verification contract

The observer selects exactly one phase after prefix validation and before
the key exchange response. Its combined CLI still requires no-capture mode,
an explicit store, map379, spawn and Mesmer state/tracks; it excludes auth,
raw reply and static population fixtures.

TCP regression coverage uses one listener for alternating creation and both
saved characters, fragmented/coalesced source-shaped prefixes, the retained
nonzero GUID case on creation, and differing offset16 values. It checks
exact single-phase openings, persisted creation1048 results, zero-scalar1065
redirects to the same address, and selected world populations. Rejected map,
unknown/zero world GUID, bad-header and truncated-prefix cases receive no
key reply and create no capture files. Existing standalone-mode tests remain
applicable. These synthetic peers are not a frozen-client playback claim.

## Retained evidence

Records in `~/re/gw2/notes/209026-creation-redirect-static-1.json`:

| Evidence | Observation |
| --- | --- |
| `ev_9b36716eebd275280db4ea9b8495534231d8c0f2076dfad6c56e9f32b6f66c72` | Creation constructor clears payload and sets game type1. |
| `ev_3b783184ecda927779e9c6a1f911d09ea8e5f6b421d1f45c62f8dc71dce5c890` | Auth route-result stores at transaction+40/+44/+48. |
| `ev_148fb9cd3408f4114e91ea6d71a6a305534b23077af8df567718d614a71cf26c` | Event0x23 emits transaction+38 as the96-byte result. |
| `ev_7561ef1a1fa28264ed87b3218e5d7e578df1cbd317bf44c64485438e2e4d14f8` | Route consumer passes result+0xc/+0x10 to the common join helper. |

Records in `~/re/gw2/notes/209026-creation-mission-binding-1.json`:

| Evidence | Observation |
| --- | --- |
| `ev_61f431788f127ee875a89c86b7f02de46ff00aa5b802335055e90bc4eec22871` | Common join helper and1065 field consumers. |
| `ev_49d5441a6f82ebabe545fdb72c9595d49b4448891353357d37113c81b31b1f90` | Join constructor argument stores. |
| `ev_79aafdfbff7e50a92f41f7be2ea4cf0bf5d5498178843d6e870658c5d18819bc` | Exact84-byte prefix construction and selected-GUID placement. |
