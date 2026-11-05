# Build 209026 creation navigation controls

Static REA inspection, 2026-11-05, of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The original static pass launched, attached to, or modified no client. The
later run121–run125 sections audit retained logs from separately coordinated,
isolated client runs. Run125 also includes the runtime owner's recorded X-window
resize intervention; the observer itself did not change target memory.

**Run125 resolved the missing navigation as X-parent window clipping.** The
game child and internal UI were 1280×900, but the parent was only 1272×866.
Next occupied screen rows 868–900, entirely below that parent. Resizing only
the parent to 1280×900 immediately exposed Back, Next, and Skip. The following
measured click reached Next's owner handler. This result is attributable to
the recorded parent resize, not the passive probes or a server change.

The inspected creation and layout paths do not require a server reply or a
ready character model to create and position Next. Next is created
unconditionally, and all three labels are plain localized strings. The static
and earlier runtime investigations below are retained as the evidence sequence;
their unresolved hypotheses preceded the run125 intervention. This result does
not establish that the separate missing character preview is resolved.

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
Widget paint `0x140419fc0..0x14041a1d1` draws the realized material or
requests a fallback material through the UI resource factory; it has no model
or network gate. Widget `+0x88 == 0` alone does not prove a resource failure,
because that fallback path is normal.

## Original passive diagnostic

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

### Run122: applied geometry and initial paint dispatch succeed

Final read-only audit
`ev_9f39aeb29d27f87fcd96686231ad4b7c108766715e01c9aaf0238f430543148d`
captured 31 valid probe records after an `inferior_exited` summary, at
2026-11-05 17:56:01 UTC. The 143078-byte `client.out` snapshot has SHA-256
`3a08d7b056b7f0459fafb76d73ae68509c36afb7e4ab5ea35fcb13800ba45c52`.

All three navigation widgets reach `0x140419fc0` at initial scene 0, roughly
0.314–0.318 seconds after selection. Their paint widths agree with their
measured sizes, and icon dimensions are 32×32. Applied frame rectangles are
valid and remain unchanged through the captured appearance sample at +2.114s:

| Control / ID | Handle | Applied rectangle (`+0x30`) | Derived top-origin viewport rectangle |
| --- | --- | --- | --- |
| Back / 13 | 235 | `[528, 0, 606, 32]` | `[528, 868, 606, 900]` |
| Next / 14 | 237 | `[674, 0, 753, 32]` | `[674, 868, 753, 900]` |
| Skip / 15 | 239 | `[1155, 0, 1280, 32]` | `[1155, 868, 1280, 900]` |

Their propagated clipping rectangles (`+0x20`) and those of the root/ancestors
are the full `[0, 0, 1280, 900]`. Local hidden/disabled bits and the empty
parent-clip-intersection bit `0x2` are clear. This establishes correctly sized
bottom-edge geometry and initial paint dispatch. **No failed boundary is
observed before paint entry.** The first unobserved boundary is inside/after
paint: resource lookup, child-text paint, draw submission, or cached UI
composition. The probe does not establish which of those causes the missing
pixels, nor does a paint callback establish a successful rendered frame.

The geometry field meanings come from the bounded update trace at
`0x141089456..0x14108979b`. It writes the applied rectangle to `+0x30`, sets
state bit `0x2` for an empty intersection with the parent clip/viewport, and
stores the propagated clip at `+0x20`. With state bit `0x1`, that clip is
intersected with the current frame. Getter `0x141088710` binds parent-relative
top-origin position to `(own.minX-parent.minX, parent.maxY-own.maxY)`.
The table's viewport conversion uses the observed height 900 and preserves
the raw client rectangles alongside it.

All three realized-material fields `widget+0x88` were null at paint entry.
This is not a failure: the source takes its normal resource-factory fallback
path. Back/Next had `+0x90` set and `+0x98` null, selecting the built-in
filenames at `0x14041a0fb`; Skip had both set, selecting the supplied filenames.
The next narrow observation is factory return RAX at `0x14041a131`, followed
by the submission call `0x14041a19c → 0x14106f6f0`, filtered using the widget
in RSI and its handle at `+0x18`. Child text is a separate UI child 0 and its
rendering was not measured here.

Coverage limit: the probe stopped at its 512-callback budget before its
requested late appearance sample. Its aggregate `apply:395` was incremented
before the Python ownership filter; only 13 emitted `apply_input` records
belong to the navigation handles. It must not be read as 395 navigation
layouts. The three emitted paint records are positive initial-dispatch
evidence, not proof of missing later painting or failed cached rendering.
Any continuation should apply the ownership filter before consuming the stop
budget and retain an independent sparse heartbeat for late geometry.

Complete raw filtered rows and parsed data are preserved at
`/home/blizz/re/gw2/notes/run122-creator-nav-paint-final-20261105.json`;
full REA captures are at
`/home/blizz/re/gw2/notes/209026-run122-creator-nav-paint-audit-20261105.json`.
New geometry-source captures are retained in
`/home/blizz/re/gw2/notes/209026-creation-nav-paint-probe-source-20261105.json`,
including `ev_1697c044d958430a9940be553d90e7fe068df89f40a073e9bce7b02a1de7328d`,
`ev_86aa5779de608dcd3824f91ccedc2851cdfb1454745eaef39555d06498716380`,
and `ev_b8f6d4b1bd379f89d4bf37c1bbf5e3864ee8a28161e96383bff33821f4895661`.

### Paint output, inherited alpha, and text: bounded source continuation

The resource is named **material** by the assertion in `0x141075450`.
The nav painter has no silent early return: it selects `widget+0x88` or the
factory fallback, builds a descriptor, and calls `0x14106f6f0`. Factory
return RAX at `0x14041a131` and the fallback submission at `0x14041a19c`
are distinct boundaries. A nonnull material proves object acquisition, not
that its image is loaded or visible.

Descriptor constructor `0x14106ea60` initializes packed color `+0x18` to
`0xffffffff`; its high byte is the submitted alpha. The nav path supplies
material, 32×32 dimensions, model rectangle, and layer 0. Submission
`0x14106f6f0` asserts the required fields, optionally rounds the rectangle
using the UI scale at `0x14289deb0`, and silently returns only if the
resulting rectangle has zero/negative extent. Its queue boundary is
`0x14106fc80 → 0x141079ba0`, with frame RDI, descriptor RBX, drawable R8,
and rounded rectangle at RSP `+0x40`.

Queue helper `0x141079c30` groups drawables by layer. Unless frame state
`+0x29c & 0x400` bypasses inherited alpha, it multiplies the descriptor
alpha by `0x14107a3f0(frame+0x108)`. That helper multiplies the frame's
float `+0x158` and each ancestor's corresponding value, stopping after a
frame with creation flags `+0x2a0 & 0x44`, then clamps the result to [0,1].
This supplies a source-backed transparency test shared by child render
components; the field's runtime value was not captured in run122.

The label is an independent immediate child 0 of each nav frame. Its event 8
dispatch reaches `0x14104a240`. This painter returns before drawing if
text widget Dword `+0x78 == 0xffffffff`. Otherwise its normal path calls
`0x141048960` at `0x14104a406`, passing callback `0x141046740`. That callback
builds text parameters and takes one of two draw paths,
`0x141074570` or `0x141070280`; it does not share the icon material lookup.
An absent icon material alone therefore does not establish the cause of an
absent label. The new probe records only the start-offset sentinel and
whether the text pointer is present, never text or character names.

The initializer's order calls are also distinct from a final GPU draw order.
`0x141073290 → 0x14108b670` stores the requested order at frame `+0xb0`
and reinserts its hierarchy node through `0x14108b3b0`, which compares
sibling order values. Background child 12 receives 1, preview child 17
receives 2, and Back/Next/Skip receive 3/4/5. This establishes ordered sibling
insertion, but not the renderer's final traversal direction, layer
composition, or whether an opaque footer covers the controls. **Occlusion
remains unresolved; initial paint dispatch or queue insertion alone cannot
rule it out.**

The standalone passive helper is
`/home/blizz/re/gw2/notes/creator-nav-output-probe-20261105.py`, entry point
`arm_creator_nav_output(inf, slide)`, SHA-256
`126e74b2c0e1a918c3d77e33cba815cd77ae3c572fa3aef9c269c5b1bb6839a9`.
It dynamically resolves nav and text-child ownership, applies ownership
before sample budgets, and arms the queue breakpoint only across a sampled
owned icon submission. Register/read conditions filter the sites, which are
also disabled after sampling their owned handles and rearmed in sparse
0/2/8/12-second windows. A separate 8192-callback ceiling bounds actual probe
stops. Syntax/mock checks cover alpha inheritance, text sentinel handling,
submit/queue correlation, conditions, disable/rearm, the callback ceiling,
and 1000 unrelated callbacks consuming no sample budget. These checks
validate the probe, not the client.

### Actual Next click and scene progression

Initializer `0x14029d2f0` passes **scene+0x50** as the button owner. Its
vtable `0x141933a88` has handler `0x14029c370`: child IDs 13/14/15 dispatch
respectively to `0x14029dfa0`, `0x14029e270`, and `0x14029e570`.
Next calls scene-advance helper `0x14029ddf0` at `0x14029e30a`.
The latter writes its selected next-scene index at `0x14029deb3`, updates
the local scene, and calls `0x14029edd0` (the client-416 update path).

The normal table at `0x142559680` advances 0→1→2→3→4→5→6→7→8→9
before its sentinel. Two locally selected alternate tables skip different
steps. Their predicates have not been bound to a runtime variant here;
scene 4 or later must not be labeled biography/name solely from this table.
The source trace gives no additional server acknowledgment prerequisite for
the ordinary Next action.

At run122's observed 1280×900 layout, `(710,884)` lies inside Next's actual
rectangle. A coordinated UI click there, paired with the owner-dispatch
and scene-write observations, can distinguish an invisible but functional
control from a later hit-test/advance problem. This source pass performed
no click or other runtime action.

Twenty complete source captures are retained in
`/home/blizz/re/gw2/notes/209026-creation-nav-output-source-20261105.json`.
Key captures are `ev_90c63b6a5765cc72937fb83e59723f68f327499f0fb92958e5c43257fc3eec67`
(descriptor/material),
`ev_6b2b0e887b64c6502a7617838184bf2eefdaac06456b1b9759d4cb4cea2db196`
(submission/queue),
`ev_6ed85f10e79f45f04a0e0745866ee4b0959e708ea108b3f79051a209903302cf`
(alpha),
`ev_fc45bc2bf2760a54fc80d8a4d2531611d19b3a533ea8ffd5ad6edad7c002b328`
and `ev_388166397b11b07fadd1a9f21b4fe518556f705862055e8215f9e8d2a4f39037`
(text paint), and
`ev_d7c56e46384f311e719572af435f832731be3767d4bcbf042e86c1ba17c548ca`
(button owner). The two complete follow-up captures for sibling ordering and
the text callback are retained in
`/home/blizz/re/gw2/notes/209026-creation-nav-order-text-source-20261105.json`:
`ev_c48ac6d3d2c81b29b948413d002b50b5d8b29a8d68b55bd57c0ef14d9c77118c`
and `ev_90dde5775bb0ffa2a02e0e56d83cde26026cd953718993a2d0ce310d5e926f8a`.

### Run123: icon queue and label paint succeed; no Next-owner dispatch observed

Final REA audit
`ev_7d60bf279c8d33efa73c0db97b598146e7a702ce6a8e290a412613b66424a2d3`
read the completed `run123-norn-creator-render-submit/client.out` at
2026-11-05 18:12:21 UTC. The 216707-byte snapshot has SHA-256
`6b792c283214e1f7a69e5e632cc9a992a6c6e0a99925d4d3305242a44a88e2bb`.
All 41 navigation records parsed. The probe exited normally, reporting 381
actual callbacks and 38 relevant samples, with no read error, sample-budget
termination, or pending unmatched submission.

Nav handles were Back 224, Next 226, Skip 228; their immediate text children
were 225/227/229. The bounded 236-slot ownership scan was not truncated;
44 unrelated slots were unreadable, while all six required owners resolved
and passed subsequent pointer/parent validation.

At initial paint, all three material factory returns were nonnull. Each icon
submitted a nonnull material, packed color `0xffffffff` (alpha 255), layer 0,
32×32 dimensions, and pixel scale 1.0. Each reached the queue boundary with
the same descriptor and a nonnull drawable. Pixel rounding preserved these
nonzero model rectangles:

| Control | Icon model rectangle at submission and queue |
| --- | --- |
| Back | `[0, 0, 32, 32]` |
| Next | `[47, 0, 79, 32]` |
| Skip | `[93, 0, 125, 32]` |

All three text children reached both `0x14104a240` and the normal draw call
at `0x14104a406`. Each had a text pointer and raw start offset 3, so the
empty-text sentinel did not suppress these calls. All six frames had complete
ancestor-alpha chains with effective alpha 1.0. Once laid out they retained
state `0xa104`, full 1280×900 clipping, and the same bottom-edge rectangles
as run122, including the appearance samples at +8.073 and +12.070 seconds.

Thus **no failed icon boundary is observed through queue insertion**, and
the label path is observed through invocation of its text-layout draw helper.
Material object acquisition still does not prove texture readiness; the
label callback's resulting drawable, subsequent renderer traversal, clipping
in later render passes, final composition, and occlusion were not observed.
The icon and text paths must not be described as producing visible pixels.

The probe recorded only three scene-write events: 0→1, 1→2, and 2→3.
It recorded **no** `0x14029c370` navigation-owner click and no later scene
advance before normal exit. Correlation with the coordinated UI action log
is required to bind this absence to the attempted `(710,884)` Next click;
the navigation log alone does not establish that an input event was delivered
to the client. If delivery is established, the next input boundary is the
coordinate conversion/hit test/owner dispatch, before Next's scene advance.
There remains no source evidence for a missing server response to ordinary
Next.

Exact filtered lines and parsed records are preserved privately in
`/home/blizz/re/gw2/notes/run123-creator-nav-output-final-20261105.json`.
All three REA audits, including the two earlier incomplete snapshots, are in
`/home/blizz/re/gw2/notes/209026-run123-creator-nav-output-audit-20261105.json`.

### Input conversion and actual hit selection for the next diagnostic

The bounded source continuation identifies a concrete chain to observe before
changing any UI or protocol state. Windows-message translator `0x140c2aa60`
extracts signed x/y from the message's packed fields at `+0x18/+0x1a`.
Its table maps messages `0x201/0x202` to internal events `0x14/0x17`, with
button field 0. At translator return `0x140c2aae0`, R14 still points to the
input message, RBX to the translated record, and EAX reports success.

Dispatcher `0x14106b1a0` converts ordinary down/up coordinates to
`(x/inputWidth, 1-(y+1)/inputHeight)` using integer globals
`0x14289c860/0x14289c864`, then queues events 29/32 through `0x14106dbf0`.
Mouse movement normally queues event 31. Capture-mode branches and button
state checks exist, so the probe preserves the actual raw and normalized
values rather than assuming the ordinary branch ran.

Frame mouse selector `0x141084790` multiplies the normalized coordinates by
the float UI viewport at `0x14289dea8/0x14289deac`. It obtains an ordered
frame array through `0x141079170` and visits that array from the end toward
the beginning. A candidate must supply the requested `frame+0x1dc` mask,
pass state predicate `0x14108b9e0`, avoid the excluded frame/descendant relation,
and match geometry through `0x141088990`. With a request mask other than
`0x100`, it also requires the ancestor-enabled predicate `0x14108b990`.
If a candidate fails the match, its creation flag `0x2` can still terminate
the search. This is an actual route for an intervening frame to block target
selection. It was a hypothesis before run125, whose successful post-resize
selection did not encounter such a blocking frame.

The geometry helper checks parent clipping or the viewport, then the frame
rectangle, and invokes a widget-specific geometry event. Consequently the
probe observes the client's selected result rather than treating a Python
rectangle test as the client's final hit decision. The result is RAX at
`0x1410848fd`; the sentinel early return is `0x1410847c4`. Candidate entry
`0x14108488b` exposes RBX as the visited frame and ESI as its excluded-relation
result, with the scaled UI point at RSP `+0x78`.

Frame event dispatcher `0x1410870b0` receives frame+`0x220` in RCX and event
ID in EDX. The nav widget's own table binds events 30 and 34 to its owner
calls at `0x140419c9a/0x140419ccb`. These boundaries distinguish raw input
arrival, normalization, selected frame, frame delivery, and creator-owner
delivery. The expected ordinary `(710,884)` input at input size1280×900
becomes UI point `(710,15)` at viewport1280×900, inside the observed Next
rectangle. Run125 below subsequently measured those dimensions and coordinates.

Prepared helper:
`/home/blizz/re/gw2/notes/creator-input-hit-probe-20261105.py`, entry
`arm_creator_input_hit(inf, slide)`, SHA-256
`b1e69e06e4a42b392c5be32862917352628e52aac9a7630616f1104bf8e23567`.
Before `/sandbox/logs/creator-input-probe-active` exists and appearance scene3
is selected, only a sparse heartbeat with ignore count10 runs; input/hit sites
are disabled. After enabling those sites, it creates the empty observer-side
marker `/sandbox/logs/creator-input-probe-ready`; the UI harness waits for this
marker before its action. This is filesystem coordination, not an inferior
memory write.
The active capture is limited to 35 seconds/4096 callbacks. Candidate/result
sites are enabled only for a sampled query and matched to its stack pointer.
The helper captures numeric coordinates and frame ancestry without text or
keyboard contents. Syntax/mock checks covered the marker gate, raw/normalized
input, candidate/result ownership, frame delivery, and callback limit.

The separate read-only X metadata recipe is
`/home/blizz/re/gw2/notes/creator-input-xgeometry-recipe-20261105.md`.
It records display/window geometry, focus, pointer-under-window, and frame
extents before/after the existing mouse move and click, without window titles.
X window IDs and Windows HWNDs remain separate identities.

Fourteen complete REA captures are retained in
`/home/blizz/re/gw2/notes/209026-creation-input-hit-source-20261105.json`.
Key evidence:
`ev_065be328f70de2bc88c1bcd4c5be91639f00d1426f70b9527d74824b94fb5fb4`
(input normalization),
`ev_03d283dfd4427799825cf601fa558dc04cb032cac3ae88d7146a409c634eb053`
(actual candidate selection),
`ev_8df562f794a6e92b271438fb8bc3cf2ec414a39ac6a55e93398da850fcb5b12e`
(geometry/state predicates and frame dispatch), and
`ev_0a7106b38044b1d9c35defce3b60add019b14a1bdaba6f3ce21a0c498e138b92`
(exact guarded probe bytes).

### Run124: matching dimensions, but input capture has no usable routing coverage

Final read-only REA audit
`ev_47024f63100b113451ddad3f9cc1c37ff7315bcd3a4156e711463a36735e3080`
captured `run124-norn-creator-quiet-input/client.out` at
2026-11-05 18:28:14 UTC: 75076 bytes, SHA-256
`63669a567f98f3fe0bb7fd1e4dc56fe81180b1cc908a52a88822944252b56295`.
The input helper armed, observed its marker in appearance scene3, enabled
its sites, and created its ready marker. At activation it read input dimensions
1280×900, UI viewport1280×900, and pixel scale1.0. The creator root was handle89,
with full viewport geometry/clip and state `0xa104`.

**The probe then exhausted its 4096-callback budget without capturing any
translated input, normalized input, hit query, or owner event.** Its summary
reports 4097 actual callbacks, zero sampled queries, and no pending query.
The broad translator/queue/frame-dispatch sites still invoked Python for
unrelated work despite their textual GDB conditions. This run therefore
provides no usable evidence for an absent input event, selected hit frame,
or occluder. The next probe must place its raw/normalized breakpoints on the
dedicated mouse branches and avoid a continuously enabled generic frame
dispatcher. Increasing the callback budget alone would preserve this problem.

The UI script exited0 and logged its measured Next-click action and normal
completion. The ready marker followed the active marker by about67ms. The
numeric metadata file before movement contains only pointer `(710,490)`,
screen0, window0; it contains no focused-window geometry. The at-click file
records pointer `(710,884)`, screen0, window543. These are X observations,
not client input observations. The identity and geometry of window543 and
the game X window are not established by these files. In particular, a
possible desktop/root-window hit must be verified before being asserted.

Filtered input records, exact metadata, file hashes, and marker mtimes are
preserved in
`/home/blizz/re/gw2/notes/run124-creator-input-final-20261105.json`.
Full REA captures are in
`/home/blizz/re/gw2/notes/209026-run124-creator-input-audit-20261105.json`.

### Replacement observer: dedicated mouse sites and native GDB filtering

The separate helper
`/home/blizz/re/gw2/notes/creator-input-hit-probe-mouse-sites-20261105.py`
uses entry point `arm_creator_input_hit_narrow(inf, slide)` and SHA-256
`1b24645e7e8e64e8025defeb1262d0f2a038d03ea21f5e0c862a8a33847dd5f3`.
The run124 helper is preserved unchanged. The replacement uses raw-button
branch `0x140c2ae93`, absolute-movement branch `0x140c2acaa`, and the dedicated
normalized down/up/move calls at `0x14106b847/0x14106b9ba/0x14106bac3`.
It removes the shared translator-return, all-event queue, and generic frame
dispatcher breakpoints. Raw records at these earlier branch sites do not
claim translated output has already been initialized.

Each site is a public base `gdb.Breakpoint` with a native command list:
`silent`, `python cni_dispatch(name)`, `continue`. It has no Python `stop()`
override. An independent owned native fixture under GDB16.3 demonstrated
why that distinction matters: a subclass callback ran1000 times despite a
false condition or ignore count100. The command-list form produced zero
callbacks for a false condition, 900 for ignore count100, and ten samples
when its callback reset the ignore count99. Disable/rearm and installation
from another `Arm.stop()` also passed. This verifies observer behavior in
that fixture; it does not establish input delivery in the game.

The real-GDB result is REA evidence
`ev_9f168c2562ac00b661da4151c5d903e303bf748c03deabd07a3b04b220517b32`.
The complete fixture, driver, results and Evidence are retained under
`/home/blizz/re/gw2/notes/gdb-breakpoint-semantics-oiuknhd1/` (see `README.md`).
Native breakpoint traps still have overhead; this result concerns filtering
before the Python command callback. Separate helpers must also avoid sharing
an address when each command list ends with `continue`, which can prevent
another list at that address from running.
The helper's separate syntax/mock checks cover public command-list wiring,
the active/ready marker gate, raw input without reading unfinished output,
all three normalized mouse branches, stack-scoped candidate/result capture,
owner delivery, and the callback limit. Mouse-site byte guards are retained
in `/home/blizz/re/gw2/notes/209026-creation-input-mouse-sites-source-20261105.json`,
evidence `ev_086b81042c0c68336c913506559d8b5108a0849f30734a734ad34f0992352fc5`.

### Run125: X-parent clipping resolved, then Next delivery observed

The runtime owner's initial numeric X-window capture, REA
`ev_bf0c81da9d0163089434e232125c958e84b6841022121cd13a0b5576586a265d`,
identified this hierarchy at appearance:

| X window | Role in captured hierarchy | Size | Position |
| --- | --- | --- | --- |
| 543 | Root | 1280×900 | 0,0 |
| 12582920 | Parent of game child | 1272×866 | 0,0 |
| 8393941 | Game child | 1280×900 | 0,0 |

All three were viewable, with border width 0. The child/UI's navigation row
was at screen Y 868–900, below the parent's bottom at 866. The runtime owner
then executed only `xdotool windowsize 12582920 1280 900` inside the verified
owned container. REA
`ev_3afdd740d1c0453b757f6db23dc5018978e325fb444523f9ae63209e4ab2aec4`
records that intervention and exit 0. The owner observed Back, Next, and Skip
immediately in `shots/creator-parent-resized.png`. The retained numeric
before-click capture then shows root, parent, and child all 1280×900 at 0,0.
Internal input dimensions and UI viewport remained 1280×900, pixel scale 1.
This binds the visibility correction to the parent resize; neither a different
UI-resolution constant nor another server response was required.
The runtime owner's complete evidence bundle is retained at
`/home/blizz/re/gw2/notes/209026-norn-creator-run125-20261105.json`
(including the initial geometry and resize captures above and
final runtime capture
`ev_bf4614290cee173087c24c54f7b06007f19d5bfbbd4891cf04e0c97df4b3921d`).

The final read-only input audit, REA
`ev_2e2fc3c520f7966f867b70fb53c018606a9158e7266f451963efa290eab4f8fc`,
captured the completed run at 2026-11-05 18:45:13 UTC. The client log was 92010
bytes, SHA-256
`b21a40d9a7830b07bd0fb6b48f8bc6e0db4a795ad757a7b74fa8dc1c79ebc338`.
The UI script exited 0 and the inferior exited normally. The narrow helper
recorded only 34 callbacks: four raw mouse events, three normalized events,
three complete hit queries with nine total candidates, and one Next-owner
event. No callback-limit termination, helper error, or pending query occurred.

The post-resize click followed this actual path:

1. Raw move/down/up reached Windows HWND `0x3007a` at client coordinates
   `(710,884)`; this HWND is not an X window ID.
2. Queued move/down/up events 31/29/32 carried normalized coordinates
   `(0.5546875, 0.016666650772094727)`. The selected UI point was
   `(710, 14.999985694885254)` in the client's bottom-up coordinates.
3. Each hit query visited handle 239 (an unrelated zero-size child 13), Skip
   handle 231, then Next handle 229. Next was immediate creator child 14, order 4,
   mask `0x11f`, flags `0x210`, rectangle `[674,0,753,32]`, and handler
   `0x1404198a0`. Its state was `0xa104` initially and `0xa144` after mouse
   focus. Every query returned hit value 1 and selected handle 229. Its ancestry
   was creator root 89, UI root 2, then frame 1. None of the visited candidates
   had the blocking creation flag `0x2` or an excluded relation.
4. At capture elapsed 0.8303s, creator owner `0x14029c370` received child 14
   while the scene index was 3. This directly establishes Next delivery. The
   narrow helper did not sample the subsequent scene-index transition.

The ready marker preceded the click. At-click X metadata reported `WINDOW=0`,
which must not be interpreted as lack of game delivery: the actual Windows
message, hit result, and owner callback above establish delivery independently.
No renderer or UI-handler modification was needed for this observation.

Filtered raw lines, parsed records, numeric X metadata, hashes, and marker
times are retained in
`/home/blizz/re/gw2/notes/run125-creator-input-final-20261105.json`
(SHA-256 `d47878178cac842b2376d88a952723fd2f2909046523bee42482643667ffb52a`).
The appearance X metadata SHA-256 is
`b9ed765fb06b18a290ebc7b1f603df1501f7e2c1930c5adaa55b060b6246e404`;
the post-resize before-click metadata SHA-256 is
`40f262436dc744bfca2ac527bd33baaca3b4ceab682b19bf2289b76dec283c2b`.
Three complete REA audit objects are retained in
`/home/blizz/re/gw2/notes/209026-run125-creator-input-audit-20261105.json`
(SHA-256 `b85f72c9e157ba09debaf46b283a7b994bc848569846d396af285af536dc1a08`).
The final capture retained all 23973 raw/normalized output bytes and all seven
raw frames; its aggregate truncation flag refers to two omitted duplicate
rendered terminal frames. These read-only audits did not attach to or control
the running client.

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
continuations cited above were followed explicitly. Those source captures are
bounded static evidence; the separately labeled run sections supply the later
runtime observations and their limits.
