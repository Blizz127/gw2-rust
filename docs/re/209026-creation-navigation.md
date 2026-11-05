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

- Skip: child ID setup at `0x14029da04`, lookup call at `0x14029da10`,
  layout call at `0x14029da24`, mode 9.
- Next: child ID setup at `0x14029da69`, lookup call at `0x14029da99`,
  layout call at `0x14029daac`, mode 5.
- Back: child ID setup at `0x14029dab6`, lookup call at `0x14029dae1`,
  layout call at `0x14029daf4`, mode 9.

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
   `0x141073a60` operates on mask `0x200` (**set means locally hidden**);
   enable setter `0x141071230` operates on mask `0x10` (**set means locally
   disabled**). These are local state bits, not a guarantee of effective
   visibility through ancestors or clipping. Creation flags
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

### State polarity and layout allocation semantics

The setters first compare the requested Boolean with the inverse of the
corresponding state bit. If they differ, `0x14108ba10` toggles that bit.
The bit getter `0x14108bb30` is exactly `return *flags & mask`; the mutation
helper calculates `((*flags & ~clearMask) | setMask) ^ toggleMask`.
Both setters pass zero clear/set masks and their respective toggle mask.
Thus a true visibility/enabled request clears the local hidden/disabled bit.
The creation scene's transition code independently binds true visibility to
showing the new scene and false visibility to hiding the previous one.

Layout helper `0x141072820` forwards its arguments to `0x141074a50` with
the apply Boolean set. The four floats at R8 are
`[minX, minY, maxX, maxY]`, an allocation rectangle called `margins` by the
assertion `MathRectIsValid(*margins)` at `0x142129260`. The implementation
checks max ≥ min on both axes and derives available width/height by
subtraction. With the observed zero/absent optional constraints:

- Mode 5 (Next) positions the measured widget at `(minX, minY)`.
- Mode 9 (Back/Skip) positions it at `(maxX - measuredWidth, minY)`.

Neither mode stretches the widget to fill the allocation rectangle. The
measured dimensions come through `0x141088560`, and the resulting position
and size are applied through `0x141088d20` at `0x14107514c`. These allocation
rectangles are not established as the final renderer clipping rectangles.
Consequently a zero cached measured size, local hidden state, and an off-screen
allocation are distinct diagnostics in the runtime log.

### Run121: creation and layout succeed

Read-only REA audit `ev_28de53b0460d0717165f5ba4ca4054ed77e87681b42bd33577aa8cd1e00f12b5`
captured 41 valid navigation log records at 2026-11-05 17:42:10 UTC from
`/home/blizz/gw2-sandbox/runs/run121-norn-creator-ui-probe/client.out`.
The observed file was 185535 bytes, SHA-256
`201d934035df0a92baa4dce69813c858c8d2e66c4918de24ea4add5309d3e54b`.
This is a timestamped file observation, not a claim about later appends.

Across captured scenes 0–3, including appearance, navigation IDs 13/14/15
exist as handles 231/233/235. After measurement their sizes are respectively
78×32, 79×32, and 125×32. Their state settles to `0xa104`: both local hidden
and disabled bits are clear. The root and two captured ancestors likewise
have those bits clear. All six captured layout passes receive 1280×900.
Their allocation rectangles remain:

| Control | Allocation rectangle |
| --- | --- |
| Next | `[674, 0, 1280, 900]` |
| Back | `[0, 0, 606, 900]` |
| Skip | `[0, 0, 1280, 900]` |

The measured controls therefore are neither missing, locally hidden/disabled,
nor zero-size in those observations. The probe does not observe their final
screen rectangles, clipping, alpha, or paint submissions. Root cached measured
size remains zero, but that cache is not its actual layout size: the observed
layout event explicitly supplies 1280×900. It must not be interpreted as a
zero-size root.

The next bounded check is navigation paint and applied geometry. Widget jump
tables at `0x140419f3c/0x140419f74` bind **event 8** to `0x14041998d`, which
resolves the widget and calls paint `0x140419fc0`. At paint entry, RCX is the
widget and widget `+0x18` is its handle; RDX is the paint argument block.
The painter reads its horizontal extent from RDX `+0x10`. A handle-filtered
paint observation can distinguish missing paint dispatch from later texture,
clipping, or draw-submission failure. The separate layout application call is
`0x14107514c → 0x141088d20`, with computed position and size pointers in
RDX/R8. Screen coordinate orientation and final clipping are still unproven.
There is no evidence here for an additional startup response.

Complete REA captures are in
`/home/blizz/re/gw2/notes/209026-run121-creator-navigation-20261105.json`;
exact filtered raw lines and parsed snapshots are in
`/home/blizz/re/gw2/notes/run121-creator-nav-snapshot2-20261105.json`.
The paint-event mapping is evidence
`ev_dc84721e79e75279cb43858da8b71bad62579a1ebf9b4129b1ef5448280ab5fc`.

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

Probe-preparation follow-up retained in
`/home/blizz/re/gw2/notes/209026-creation-navigation-probe-20261105.json`:
`ev_728e170a6fd17b8e06c58d901c9db55036550fc9af63541d5e633fe219ec80b1`
binds layout-entry arguments and the layout helper's unchanged forwarding;
`ev_cc573b61ff5d88651b9ddba3cf6567cd382cff66adbbd2548e294d5935732444`
records exact guarded bytes at the update, layout, and navigation call sites.
The addresses above distinguish child lookup calls from the later layout calls.

State/layout follow-up retained in
`/home/blizz/re/gw2/notes/209026-creation-navigation-polarity-20261105.json`:
`ev_c3f3fef102b76ff5abbb50c2f190e5f83b395c819363afea9e4e36c3f85ee1ff`
contains the exact flag getter/mutation helper and complete allocation helper;
`ev_f41d0b45427091ba91189f1f79d770a675181b16ce74079b5615c8c2e85b2812`
binds its assertion/position/size strings. The original setter captures above
establish the desired-Boolean comparison and event dispatch.

Compiler exception ranges split several functions into fragments; the
continuations cited above were followed explicitly. This is bounded static
evidence, not observation of the failed run's UI memory or renderer.
