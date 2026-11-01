# Build 209026: character-view reveal listener

Static REA follow-up to [character callbacks](209026-character-entry-callbacks.md).
No client launch, agreement interaction or packet transmission was performed.

## Listener and component lifetime

The constructor at `0x141105240` is identified by assertions referencing
`Game/AgentView/AvCharBase.cpp`. It stores its supplied character at whole
view +0xb38, initializes whole +0x820 to null, and registers the listener at
whole +0x28 with character primary virtual +0x558 at `0x1411055bf`.
The listener vtable is `0x142141558`.

| Character notification | Listener slot / entry | Implementation |
| --- | --- | --- |
| Conceal | +0x448 / `0x1421419a0` | `0x14110c710` |
| Reveal | +0x450 / `0x1421419a8` | `0x14110c760` |

Both complete implementations assert that the supplied character equals
listener +0xb10, which is whole view +0xb38. Conceal calls virtual +0xc0
on the pointer at listener +0x7f8 (**whole view +0x820**). Reveal calls
that component's virtual +0xc8, then tail-calls `0x141134730` on whole
view +0x2b8. Neither callback guards the +0x820 pointer against null.
Registration itself does not immediately replay reveal, as established
in the earlier registration trace.

The ordinary derived constructor `0x1411589b0` retains this listener
vtable and installs primary vtable `0x142149138`. The factory
`0x14110f150` constructs it at `0x14110f1c3`, then immediately calls
primary virtual +0x1a0 at `0x14110f1de`. That table entry,
`0x1421492d8`, resolves to initialization method `0x141159290`.
This method allocates 0x150 bytes, calls constructor `0x1411683b0`, and
stores the result at whole +0x820 (`0x14115931c`). Thus the ordinary
factory performs the required allocation synchronously after construction;
it is not a separate network component-creation operation on this path.
The factory's event caller and ordering relative to agent entry still
require tracing. This does not prove that reveal cannot arrive too early.

A second initialization method, `0x141167e20`, allocates 0xe8 bytes,
calls `0x14116c7c0` with view +0xbf8, and stores the result at whole +0x820
(`0x141167ea2`). The latter constructor's assertion source identifies
`Game/AgentView/AvCharMountModel.cpp`. This mount-model branch must not
be assumed necessary for ordinary initial spawning.

## Immediate reveal effects and limits

| Component vtable | Conceal +0xc0 | Reveal +0xc8 |
| --- | --- | --- |
| `0x14214e458`, ordinary allocation | `0x14116b4f0` | `0x14116b530`: write float 1.0 at component +0x58, then tail-call `0x14116b9f0` |
| `0x14214e810`, mount-model allocation | `0x14116b4f0` | `0x14116dde0`: write float 1.0 at component +0x58 and return |

The shared conceal implementation queries an object through component
+0xc0's +0x08 interface, virtual +0x168, then returned object virtual
+0x350. If the result is zero, it zeroes component offsets +0x54 through
+0x5f. These float fields' precise semantics are not established here.

The inspected beginning of ordinary reveal's tail function `0x14116b9f0`
has gates on component +0xe0, that object's virtual +0x50, character
primary +0x350, character +0x08 virtual +0x68, and further returned
objects. Its remaining branches are not exhausted. The complete second
view callback `0x141134730` also checks character primary +0x350 and
character +0x08 virtual +0x68 before following further component calls.
Consequently, setting AGENT_FLAG_REVEALED and dispatching this listener
does not establish complete model or rendering readiness.

## Evidence

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Private bundle: `~/re/gw2/notes/209026-character-view-reveal-1.json`
(1,514 records; 29,906,189 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_00a55c96adb53b363be03d756d8cadf74ece7ad5e3457631a6960999812b8714` | Base constructor and listener vtable installation. |
| `ev_9aa4b43fe7a711218acf3e28e89e326e900db477be61c33401544a62afcdcd6e` | Actual character-listener registration context. |
| `ev_eced38633fcc643faad2cff05bb99b99aa8a58cb6703bd5ef92c90cd31b5c02d` | Complete reveal/conceal methods and base-field initialization. |
| `ev_e81edc1c0dd479d6424365e7c7945a95199f8d482313c446e56ef06b50ab1717` | AvCharBase assertion source. |
| `ev_d75587ef3f21c61f8ddb241da2817676d2db4e4519091daf3ee5f8c421765277` | Derived initialization stores to whole +0x820. |
| `ev_e3000ef0a9364a9cfc1a439954e85b01d7d4d93cdae018e0439b804819c17b68` | Ordinary view constructor and component constructor vtables. |
| `ev_39d3d25b91f3fd44fa4c99ff076676b5b56be7c9527d45a62e405d3e9ca4da29` | Both component reveal/conceal table entries. |
| `ev_46e205d181a0c69a94a9963cd651658f7ae2c0caacea212517ac54ca859dfa71` | Initialization and conceal function boundaries. |
| `ev_646d77fd2f2affd17690101992895390b1d2aeb2e1b745f197f97f5361d6ddbb` | Ordinary primary +0x1a0 resolves to initialization. |
| `ev_080a4d6ce4e1278b3d5229da01c1f500260ada9a2e5ee88ca16c62af219fc35c` | Factory constructor/init sequence, bounded ordinary reveal tail, complete secondary view callback. |
| `ev_02a629912c19ead84f70097d17a2bc14f1b4ea340ca38485a1b7b92f094ac08d` | Complete immediate component reveal/conceal implementations. |
| `ev_7a94d282e6eabd47da7c38790088989ede0def521fb735edbab69dd3c5917e24` | Mount-model assertion source. |

