# Build 209026: terminal creation submit and cinematic boundary

The demonstrated creation UI submit branch reaches client 415 without directly
selecting a cinematic filename or changing the main view to creation-cinematic
state 1/2. This establishes the submit caller missing from the earlier
[creation request note](209026-character-creation.md). The Norn cinematic's
selected input remains outside this directly traced branch.

## Typed submit chain

The [navigation owner](209026-creation-navigation.md) receives child ID 14 at
`0x14029c370`, adjusts the scene+0x50 interface to the complete scene, and
tail-calls `0x14029e270`. This calls scene advance `0x14029ddf0` at
`0x14029e30a`.

The normal navigation table at `0x142559680` has next indices
1,2,3,4,5,6,7,8,9 followed by `0xffffffff`. The two locally selected alternate
tables also contain a terminal sentinel at index 9; their selection policy
is not newly named here. The tested state is the creation scene's +0x78,
distinct from the main view's +0x6c state enum.

At the sentinel, `0x14029def9` disables the creation root through
`0x141071230`. When the earlier existing-player query returned null, the
branch at `0x14029df72` obtains the creation context and invokes virtual
**+0xa8** at `0x14029df84`. The previously constructor-bound table
`0x14217dca8` contains **`0x14129cc20`** in that slot. This is the established
client-415 builder. The nonnull-player branch instead invokes +0x130 and
+0xc0; it is not the new-character submit branch and is not expanded here.

The complete builder `0x14129cc20..0x14129cd3c` validates its data, copies six
color rows, and passes name, appearance, profession, level, answers and flags
to `0x141410d70`. That routine emits **event 0x10000001** and tail-calls the
established 415 serializer. It does not directly select a filename or a main
view state. The retained creation and navigation notes already bind the
creation scene's subscription to events 0x10000001/0x10000002 at
`0x14029d7da`: event 1 disables the root, Next and Skip, while event 2 displays
the creation error and reenables Next/Skip. That known subscriber is not a
missing cinematic prerequisite. No additional demonstrated consumer was
identified in this slice; it supplies no seed for a general subscriber scan.

One exact flag binding is now explicit: context virtual +0x98 is
`0x14129ec30..0x14129ec39`, which returns `context+0x8a4 & 1`. The 415 builder
turns that result into raw flag 0 or 8. This field's policy meaning remains
unnamed; it is not established as a cinematic-skip flag.

The local pre-advance helper `0x14029ed20..0x14029edc9` conditionally calls the
known preview builder `0x14029eb30` and scene+0x58 callbacks. It contains no
direct main-view state selection. The complete preview and event-subscriber
graphs were not reopened for this slice.

## Reply boundary and evidence

The separate retained result-gate check confirms server1048 updates the
roster, then invokes view+0x30. Leaf `0x14094b090..0x14094b0a1` returns for
view states1/2 and otherwise tail-calls view+0x48, the name-based auth join
continuation. The [creation-view bridge](209026-creation-cinematic-view-bridge.md)
binds playback and end callbacks to those states and the join continuation.
Neither branch identifies the cinematic. The remaining
selection dependency is the provenance of the Norn-specific input to the
separately traced cinematic playback path, or an authentic server-selected
key. Race3 and this submit chain alone do not justify any server720 value;
the known submit-event subscriber does not close that selection gap.

Frozen PE SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All reads used that pinned executable; no archive or runtime was accessed.

Private complete submit bundle:
`~/re/gw2/notes/209026-creation-submit-cinematic-boundary-20261106.json`,
149024 bytes, SHA-256
`f482099cb3a5130eaa5c35ecfc5f9e0e64b638aacfd8b9af5fb2f079fe55ec2a`.
Key captures: `ev_ebdd7fc7662ee5380d5dca82472d5645b337af38c5dbaccfe7612be758dd6ac6`
(complete scene advance),
`ev_9866d8ef623be19f9a45cb73848531b35c75ba4c5db849f3ec56135ed6deacd9`
(exact context slots and three navigation tables), and
`ev_0a8ae90bfb5f1a79ff981217675e78367413710184979aa736671088b10d2486`
(pre-helper final return and complete flag getter).

The preceding result/send bundle is
`~/re/gw2/notes/209026-creation-cinematic-result-gate-20261106.json`,
104401 bytes, SHA-256
`7925b48b95eafe1b20ed643e5b72a698df7aeeaf864995285f592d1cc08bc4ad`.
It retains complete 415 builder and forwarding wrapper in
`ev_c75d51559fef4bf5861dcb774eac0092348e70f04e89bea6dd021ea6c4350311`
and exact result leaf in
`ev_72d8d84f5fa8e6ba93107630dd6ac3b793a959de5f5c08b5602bad81a4c804ca`.

## Filename-only playback route; Norn selection still unbound

A bounded follow-up establishes that a type-7 key is not required for every
playback route. The installed cinematic-context table `0x142181398` binds
**+0x30 to `0x1412b03f0`**. Its complete body through `0x1412b043b` forwards
the caller's filename in RDX and flags in R8D to common start `0x1412b0440`,
with an explicitly **null CinematicDef**. Two float arguments and three raw
Dwords are preserved. The wrapper does not select a race, sex, biography,
filename or content key; its caller-supplied values are not defaults.

The exact common-start reference query found three validated direct calls:
`0x1412b0319` from the server-722 record wrapper, `0x1412b03bb` from the type-7
wrapper, and `0x1412b0432` from this filename wrapper. The filename wrapper's
own query found only the table pointer at `0x1421813c8`, with no direct code
reference in the checked encodings. Coverage is E8/E9, REX.W RIP-relative
LEA and aligned absolute Qword references in readable non-code sections;
this does not exclude indirect callers or other encodings.

The remaining prerequisite is a **demonstrated creation-owned typed caller
of context +0x30**, including its filename/argument provenance, or an
authentic server-720/722 selection for that creation state. Neither route
is identified as the Norn male intro here. The existing
[partial type-7 catalog](209026-cinematic-biography-cache-boundary.md) cannot
choose this filename input, and another catalog sweep would not establish
the missing selection relationship. No guessed cinematic or playback flags
were installed.

The complete wrapper/table was already retained in
`ev_bd73b257c3384f2fdbe197a5d70da4a23ba9b5bf556edd82bfdad29c68ef1b1d`,
private bundle `~/re/gw2/notes/209026-cinematic-start-contract-20261106.json`,
SHA-256 `4af50720fb9e5f4db245c20a90468ce1243bb6a990456770de3945eb1fef82d6`.
New exact-reference evidence:

- `ev_64fa0543b42051e71fc315ffad7b3a13ca3a17ca590e25ecbea3fb9be34c12c1`:
  common-start callers.
- `ev_9fdcfae2a4aa1bca09014abdc0f1af835166c57e19a001540056c3f269fca90c`:
  filename-wrapper references.

Both complete records are in
`~/re/gw2/notes/209026-creation-cinematic-filename-route-20261106.json`,
43688 bytes, SHA-256
`f1d6ef1c67791f068e87cf975d56ad553d7e9554ca4679a76465948eec38f0c9`.
Consolidated private findings:
`~/re/gw2/notes/creation-cinematic-filename-route-findings-20261106.md`,
SHA-256 `1df3291f5a3123cfc87c5ccd547a63f0971dc4a7432990e1b13ed20c51de040a`.
Both captures exited 0, retained untruncated output and verified owned-helper
cleanup. Frozen PE identity was checked; no archive, runtime, shared target,
gameplay or publisher state changed.
