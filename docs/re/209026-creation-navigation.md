# Build 209026 creation navigation controls

Static REA inspection, 2026-11-05, of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No client was launched, attached, or modified for this investigation.

**The inspected creation and layout paths do not require a server reply or a
ready character model to create and position Next.** Next is created
unconditionally. Its label, and the Back and Skip To End labels, are plain
localized strings. These findings do not explain the runtime absence of the
controls; they establish a concrete UI-state/layout diagnostic for the next run.

## Creation, layout, and state

Scene initializer `0x14029d2f0` creates these children of the scene root:

| Child ID | Label | Localized text ID | Widget order |
| --- | --- | --- | --- |
| 13 | Back | 48768 | 3 |
| 14 | Next | 48538 | 4 |
| 15 | Skip To End | 46398 | 5 |

The factory is `0x1410708c0`, with widget handler `0x1404198a0` and
creation flags `0x210`. The local predicate `0x1410e9aa0(0xa)` affects
initial Back/Skip visibility; its broader meaning is not established here.
No preview/model/network predicate surrounds creation of Next.

The current scene pointer is the Qword at **`0x1425595d8`**, assigned by
`0x14029af80` at `0x14029b346` and cleared in the destructor.
The scene stores its root UI handle at `+0x10`, current scene at `+0x78`,
and preview manager/model at `+0x80/+0x88`.

Root dispatcher `0x14029b5c0` routes event **55 / 0x37** through
`0x14029ba10` to layout function **`0x14029d950`**. The complete layout
path through `0x14029dd28` returns early if either input width or height
(the first two floats at RDX) is zero. It otherwise looks up and positions the
navigation children without testing model readiness or a server-response flag:

- Skip: lookup at `0x14029da01`, layout mode 9.
- Next: lookup at `0x14029da69`, layout call at `0x14029da99`, mode 5.
- Back: lookup at `0x14029dab6`, layout call at `0x14029dae1`, mode 9.

The common lookup and layout helpers are `0x141071650` and
`0x141072820`. Later layout branches use the local predicate above for
additional controls; they do not guard these Next/Back positioning calls.

The root's submission event `0x10000001` disables the root, Next, and Skip
through `0x141071230`. Its error event `0x10000002` reenables Next/Skip.
[The client-416 trace](209026-creation-client416.md) does not emit that submit
event or establish an acknowledgment wait. The observed 1049 → client 410 →
1050 → client 411 startup therefore supplies no evidence for another packet
needed to unlock these navigation controls.

## Labels and drawing

The frozen language-0 manifest (file 110865, 1024 strings per file) selects:

- File 2440771: Next at entry 410, offset 23870, record size 14; Back at
  entry 640, offset 37609, size 14.
- File 2440769: Skip To End at entry 318, offset 24008, size 28.

All three records have header word `+2 == 0` and 16-bit text, and decode
directly to the labels above. They do not need the encrypted quest-string keys.

Manifest SHA-256:
`2e5350b743f20e981f0436f350c6422c69df07ba774391e64144fab8dc8b5243`.
File 2440771 SHA-256:
`ecd028a5d83210f18857259d007b46f49c0d3cc1b64a282e6552b2ce9c5f8413`.
File 2440769 SHA-256:
`0560d144b4f8c8430267eac79bc2689b448fc5c347d54df439ed594caf4daca6`.

The nav widget vtable is `0x141aeb630`. Label setter `0x14041a2f0`
resolves text and forwards it through `0x14041a290` to text child 0.
Widget paint `0x140419fc0..0x14041a1d1` draws the realized texture or
requests a fallback texture through the UI resource factory; it has no model
or network gate. Widget `+0x88 == 0` alone does not prove a resource failure,
because that fallback path is normal.

## Exact passive diagnostic on resume

Inspect a quiescent creator screen without calling target functions:

1. Read current scene from `0x1425595d8` and its root handle at `+0x10`.
2. Resolve the root through the bounded UI table: Qword
   `0x14289dd88` is the pointer array, Dword `0x14289dd94` its handle
   bound. `0x141087270` validates the handle and helper
   `0x141086490..0x141086537` returns `table[handle]` directly.
3. Find immediate children 13, 14, and 15. The lookup
   `0x141071650 → 0x14108ad20` keys on `(parentFrame+0x60, childID)`;
   each matching child has that key at `childFrame+0x60/+0x68` and its
   own handle at `+0x270`. A bounded scan of valid non-null table entries
   can recover the same records without executing the lookup.
4. Record frame state Dword `+0x29c`: visibility setter
   `0x141073a60` operates on mask `0x200`; enable setter
   `0x141071230` operates on mask `0x10`. Preserve raw bits and compare
   with those setter branches before assigning polarity. Creation flags
   checked by `0x141074520` live separately at **`+0x2a0`**; the factory's
   `0x210` must not be interpreted as these state bits.
5. Observe event 55's actual root width/height and the three layout calls.
   Frame `+0x18/+0x1c` is a cached measured size, with measurement
   constraints at `+0x10/+0x14`, established by `0x141088560`; it is
   **not established as the final screen rectangle**. Actual clipping,
   placement, and draw submission remain runtime questions.

This separates absent children, visibility/enable state, zero layout input,
off-screen placement/clipping, and resource/drawing failure. It does not
select one as the cause without those runtime observations. The missing 3D
preview is a separate readiness investigation; this trace does not make it a
navigation prerequisite.

## Evidence and retained scope

Thirteen complete REA Evidence records are retained privately in
`/home/blizz/re/gw2/notes/209026-creation-navigation-20261105.json`
(mode 0600). The bundle names each capture and preserves output/coverage.

| Evidence | Main observation |
| --- | --- |
| `ev_fef61043af4a878f626c79c5d2c531304973561ca557cb977c94e47678888a81` | Navigation creation and visibility setter. |
| `ev_daab420c11d53938d34ec5d70a9e90c7a5b3a28560d7392f734304641c7d2446` | Current-scene global and widget handler. |
| `ev_7180bf78c0b98359944fa593470560202e8c49fda946df9525d8dd11f10192a5` | Manifest-bound plain labels. |
| `ev_5503ab5337d477a7dbc53abcd0fa77bf19f88a5810c0c4a83117659ad37b7c67` | Root submit/error branches, button paint, text setters. |
| `ev_93ee74ce2cfab460cb9c04adbf72c9eacf343106d44b474117fd9f3c3475e495` | Root event mapping and zero-size layout guard. |
| `ev_dfb316b65bee0ce097b7e76a1d0a9f7ea0cd43e4df258731ca14bd1b537a1e11` | Complete navigation positioning continuation. |
| `ev_90ef669513c00f4b3328263551786614a9e467394da4c807c6b13b02374bf77e` | UI lookup, handle validation and enable setter. |
| `ev_98920edd10ed50e4dbb5763299187185fb8386f21194475d71f46829fd346ed3` | Child lookup keys and separate creation flags. |
| `ev_3d6723c3fb9a8ae6523136ee1f9c2e1e1bfd18de8bd73f1afc3813518f338838` | Cached measurement semantics. |
| `ev_cc67f6ca3311a9c068c2ca4910a7f4b0a4f9a595638910a6c02bf8cb2bc90a70` | UI table returns direct frame pointers. |

Compiler exception ranges split several functions into fragments; the
continuations cited above were followed explicitly. This is bounded static
evidence, not observation of the failed run's UI memory or renderer.
