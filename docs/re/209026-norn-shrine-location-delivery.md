# Build 209026: Norn shrine location delivery

Static REA, 2026-11-06. A distinct typed lead establishes **server 1140 as
one source of runtime resurrection-shrine locations**. The handler supplies
the location to the POI constructor; this path does not derive it from the
POI definition or a map-layout record. It does not recover the actual
locations of tutorial POIs 970/971 or select a player entry point.

The [Great Hunt audit](209026-great-hunt-species-link-audit.md) already binds
type 52 keys 970/971 to map 379 and subtype 1, with no matching placement found
in its bounded layout/reference search. This follow-up starts at the exact
subtype assertion's consumer, rather than repeating that search.

## Typed delivery chain

Registration **0x1422e6650** pairs descriptor **0x142614180** with handler
**0x141445fc0**. Frozen descriptor fields are header 1140, Dword, Point3,
Dword, Byte, matching `data/protocol/209026/server-tables.xml`. The body is:

| Field | Established consumer |
| --- | --- |
| Dword POI data key | Content virtual+0x1e0 resolves numeric type 52; result is asserted nonnull. |
| Point3 | Caller-supplied location: three floats and the existing Point3 trailing Dword. |
| Dword | Argument to content virtual+0x58 for type 401; wider meaning remains unassigned. |
| Byte flags | Bit 1 and bit 0 are separately forwarded; no initial flag values selected. |

There is no leading player index. Dwords and the Point3 trailing Dword use
their established varint encoding; the native decoded-memory offsets below
are not wire offsets.

Complete handler **0x141445fc0..0x14144606b**, labelled `PoiCliMsg.cpp`,
copies the 16 bytes at decoded message **+6..+0x15** into a local Point3
value. It passes that value to **0x141443550..0x1414435b2**, which tails
to **0x141443330..0x14144354a**. That complete helper:

- checks the definition and asserts that no registry entry already has its
  data key;
- asserts definition+0x44 equals 1, explicitly named
  `POINT_OF_INTEREST_TYPE_RESURRECTION_SHRINE`;
- resolves the separate type 401 argument and calls constructor
  **0x141445600..0x1414456d2**;
- registers the returned runtime POI under definition+0x28 and notifies
  context listeners.

The constructor installs table **0x1422e69c8** and copies its R8 Point3
input into runtime object **+8..+0x17**, preserving all four Dwords. It
stores the POI definition at **+0x18**. The creation helper and constructor
do not read definition+0x48 or a map-layout position to replace that input.
This establishes provenance of the stored location on this exact path,
without assigning a coordinate conversion or new meaning to the Point3 tag.

## Changed next action and remaining dependency

An authentic 1140 payload for keys 970/971 would now provide a typed,
reviewable runtime-placement source, including its location and auxiliary
state. Alternatively, an independently demonstrated authored placement
consumer could supply those values. The retained definitions, empty tutorial
POI layout lists and unselected Mission interest points provide neither.
The general constructor contract therefore cannot justify emitting guessed
locations, type 401 values or flags.

This is one proven server-driven create path, not proof that every other
client path lacks authored positions. Even an authentic shrine location
would not by itself establish character entry selection or the missing
**actor/action/condition → quest154 step** rule. No fixture or codec was
added; no all-container, species-name, fixed-map-table or literal step-key
search was repeated.

## Evidence

All queries asserted frozen PE SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Private complete bundle:
`~/re/gw2/notes/209026-norn-shrine-location-delivery-20261106.json`,
**166278 bytes**, SHA-256
**5c7015899325361ebbb6500c8c034392f3204c552a031e87f513d462026ecbd2**.

| REA evidence | Scope |
| --- | --- |
| `ev_96dd5ff13fa9f7fed52512565bf6d87e662d107749ffdf1c02d3a25ab67090ce` | Complete typed resurrection-shrine creation and registration helper. |
| `ev_1c90ec61f2ae64216e83e4869c7531cb0622940853311d680442b16d378f5080` | Complete constructor and exact location copy. |
| `ev_9fa2fcc94f66969f4b27d508eb8882cddf83a3b4a92598e463d96dd27fbd8252` | Complete forwarding wrapper. |
| `ev_2b9cf9ad224c0e6d1ac5755619a67e5c8a2316dccd13005f243c517d3f050a2e` | Complete decoded-message handler and argument flow. |
| `ev_14b1a66d5fdc5c0e228286055f6dc78791241205709f4112550121ad7ecf4413`, `ev_8477d383a70fba0a7a849aa7b8368182d0ab2a96457bc055bdcc43e5232d422d` | Exact registration pair and descriptor kinds/header. |

Two additional captures retain the validated exact E8/E9 caller edges;
they are not exhaustive indirect-call censuses. All eight captures exited
zero with complete raw output and verified owned helper-process cleanup.
No archive, client, shared container, resource lease, publisher state or
gameplay behavior changed. No live placement was observed.
