# Build 209026: passive 10176 order-lifecycle observation

2026-11-06. Private preparation for a future run145, following the corrected
[prediction lifecycle](209026-predicted-skill-order-lifecycle.md) and
[skill request boundary](209026-mesmer-skill-command-path.md). No client was
launched, archive opened, packet changed or runtime result claimed here.
The initial proposed fixture sent server34/packed0x78 with Word0. Before any
run145 launch, the [execution-precondition trace](209026-skill-order-echo-preconditions.md)
replaced that input with an explicit nonzero diagnostic duration and the
controlled-order admission flag. The observer is unchanged and does not
establish a correct retail duration.

## Seven bounded source sites

| Site | Live inputs and observation |
| --- | --- |
| `0x141283f90` | Producer entry: RCX component, RDX optional target, R8D skillbar slot, R9 pointer to SkillDef pointer. Reads key +0x28 and flags +0x38. Records request entry, not successful transmission. |
| `0x14127ef80` | Receiver entry: RCX component, EDX outer flags, R8 behavior container. Records only exact 0x53/three-byte or 0x78/thirteen-byte forms. |
| `0x14127f03f` | Actual matching-helper result: RDI component, RBX existing order, R14 incoming container, R12D outer flags, EAX Boolean result. Rechecks current slot→order relation. |
| `0x14127f249` | The corrected gate's pending conditional branch: RDI component, R14 container, R12D outer flags, RAX resolved SkillDef. Verifies +0x58 bit0, exact10176 and native ZF consistent with the preceding 0x80000 test. `branch_rejects` reports the branch decision, not a subsequently observed return. |
| `0x14127f426` | Typed start dispatch: RDI/RCX current order, RAX concrete order table. Requires current ownership, active slot membership, flag0x100 and exact virtual+8 target. It observes the pending call, not its successful return or visible animation. |
| `0x14127eb33` | Replacement-abort callback has returned. RDI order; requires `[RSP+0x38] == 0x14127f050`, saved caller component at `[RSP+0x30]`, RBP slot0/1 and that slot already null. This selects the same-slot receiver replacement path, not every cancellation. |
| `0x14127fed4` | Completion callback has returned: RSI component, RDI order, RBP slot, R14 incoming container, EAX callback result. Rechecks current slot, typed skill and matching container. A nonzero return requests the normal release path; this is **not** a post-release observation. |

All sites have exact 16-byte guards. The helper requires x86-64/all-stop GDB,
the selected owned inferior, frozen-image PE fingerprints and all seven code
guards before installing positive-number plain breakpoints with command lists.
It overrides no Python `Breakpoint.stop()` method. Existing breakpoints at
the same exact locations cause installation to fail; partial installations
delete only their own breakpoints.

## Live ownership and field boundaries

At every observation the component must have table **0x142179b90**,
controlled Dword **+0x40 != 0**, whole character **C-0x4b0** with primary
table **0x142160be0**, and its bound interface at **+0x48** with table
**0x142161670**. The primary-character guard is independently documented in
[tooltip character context](209026-tooltip-character-context.md).
Component **+0x58** remains a separate raw flags field; its bit0 is not
renamed as control.

The predicted order's table is **0x142179a28**, kind0x53, skill pointer
**+0xa0**, slot **+0xa8**. The received order's table is **0x142179988**,
kind0x78, skill pointer **+0x78**, slot **+0x80**. Both have owner character
at **+8**, flags **+0x14**, and behavior value **+0x20**. The new bounded
factory entry proves it subtracts0x4b0 from its component before supplying
the owner to the0x78 constructor; the predicted constructor receives the
same whole-character relation directly.

An incoming0x78 must carry key10176 at packed+2. An incoming0x53 contains
no skill key: it is classified only when the selected current order is a
typed predicted0x53 whose skill definition has key10176. Kind alone never
supplies that identity. No character/order/behavior-container pointer is
stored between stops. Behavior values are compared locally; output contains
only the comparison Boolean, not either numeric value.

## Arming, output and limits

Private helper:
`~/re/gw2/notes/skill10176-lifecycle-observer-20261106.py`, SHA-256
`6ecefce718d02dc3920560fab27db0cd136c4c9946389bec05430c5650b01f8b`.
Entry **`arm_skill10176_lifecycle(inf, slide)`**, output prefix **`SLO145 `**.

Only the producer-entry site is enabled initially. The UI must create/touch
`/sandbox/logs/skill-cast-requested` after arming. A fresh marker plus the
fully guarded10176 request opens the remaining sites and calls the optional
Python-only **`rco_begin_cast_window()`** once. This coordinates the disjoint
recharge observer; it neither evaluates nor calls a target function. Repeated
requests do not renew either observation window.

Limits are128 records including the terminal record,24 per event,2048
callbacks,420 seconds from arming, and60 seconds from the first qualifying
request. Time limits are checked **at callbacks**, not by an asynchronous
target interrupt. Explicit **`stop_skill10176_lifecycle()`**, own-inferior
exit, a limit or a hard read/source failure emits one terminal record and
disables all owned sites. Absence of a terminal record remains incomplete
coverage. Read errors omit exception text and stop conservatively.

Output is a closed scalar schema: event metadata, flags, kind, skill10176,
slots and Booleans. It excludes raw pointers, character names, GUIDs,
behavior IDs, target IDs and raw packed buffers. No inferior memory/register
write or target function call is used. The helper's PE/code checks are
runtime guards, not a replacement for the launcher's full frozen-file hash pin.

The reader `~/re/gw2/notes/read_skill10176_lifecycle_log_20261106.py`
(SHA-256 `26c2913f35a501fd48240edf27c70dcb89a370943f38be002a7da201d44a6292`)
checks the exact schema and record sequence, caps input/line/record sizes,
and distinguishes a terminal record from an unfinished log. It never prints
ignored log text or malformed fields.

## Verification and remaining interpretation boundary

**28 focused host mock tests passed** with
`python3 /home/blizz/re/gw2/notes/test_skill10176_lifecycle_observer_20261106.py`.
Tests cover both typed orders, explicit10176 payload recognition, the corrected
gate, actual comparison results, exact replacement-call stack, completion
versus release, unavailable/invalid memory, ownership drift, marker freshness,
one-time recharge notification (including false-return reporting), partial-install cleanup, code/inferior changes,
record/event/callback/time limits, terminal idempotence and strict log parsing.
These are synthetic checks, not native-client playback. The already proven
plain-breakpoint command-list idiom is reused; no new native fixture was run.

The observer does not directly decode server34 or inspect the command's Word;
the unchanged server fixture/log supplies that separate carrier fact. Missing
receiver events cannot identify which upstream executor gate prevented entry.
Likewise, observing a typed start/completion does not establish retail cast
duration, healing, recharge or successful animation. Those remain separate
observations and source contracts.

Preparation receipt:
`~/re/gw2/notes/skill10176-lifecycle-observer-20261106-receipt.json`.
Source bundle:
`~/re/gw2/notes/209026-skill10176-lifecycle-observer-source-20261106.json`,
SHA-256 `6c0130f19599871feebd34391103f8dee1d03d505df7e1ee68f8cfb9ffd7d0cd`.

| REA Evidence | New source contribution |
| --- | --- |
| `ev_cf38a96e0b8adcfd1bfb55a7abde9e2621dff27c3f163d5c9785ae4ccb2b1550` | Base owner storage, abort callback/stack and normal release body. |
| `ev_5a5ae7168474dbb4e2075f4a37de1a78593267e58d15803aece5f8d3af3b600f` | Concrete predicted/received constructors and their typed fields. |
| `ev_a1dadc329537f26ace4f1804b6590a0c3817b67267add6e6f8cb31934bf22d2c` | Exact site bytes, PE fingerprints, concrete lifecycle virtual slots and bound thunks. |
| `ev_d3540c1e1f8a5cde8c03e59502df61814077ffd4fb99299e94e75ee68e94a1e6` | Exact bounded factory prologue proving component-to-whole-character adjustment. |

The complete prior receiver/request/completion source remains in the linked
prediction note's pinned bundle. This new bundle also preserves one helper
syntax failure and an oversized partial factory capture, superseded by the
successful exact-entry query. No claim relies on omitted factory bytes, the
isolated executor prologue, jump-table bytes decoded as instructions, or
unobserved runtime behavior.

## Prepared run145 inputs, no runtime result

The unlaunched private run is
`/home/blizz/gw2-sandbox/runs/run145-mesmer-skill-order-observation`.
It uses the exact offered runner0e6ac30 plus these seven passive sites and the
three disjoint recharge/context sites, deployed metadata900, prepared synthetic
inputs and no saved-character store. Removing only the helper insertions and
their arm call restores the production GDB script exactly. The combined
script hash is `429f56c0d1ad356e2060820d633bd11e4348de1f1468d9d08a30dfe9a46172c2`.

The corrected research server is commitcac0e9a's static binary, SHA-256
`52cc0108825effb3d00a0a4561be8ab10e1f7b98742dd82802325117073f0ea2`.
Duration1000ms is explicitly selected for this experiment; it is not inferred
from the rounded tooltip. The fixture has no hostile/damage/loot/heart/respawn
options and introduces no healing or630 reply. Its UI presses key6 once.

The final1831-entry input manifest is
`probe-inputs.json`, SHA-256
`3bf20d201df00ee1e3b000706b4327d9e4fb94150f2c7aa2383bc9de3aef47ec`.
The earlier72770c6 manifest is retained privately. Shell/Python syntax and both
auth/mission no-listener preflights pass. Private copied archive readers are
included in the manifest. The launch wrapper rechecks the exact lease before
archive preflight and immediately before launch. **NOT-LAUNCH-READY remains**;
no fresh shared-resource window has been granted and no client has launched.

Initial observer composition: REA
`ev_f652257e9b531c6ac6b43c62fb07d910a0cca70b279dc509b59bf1ebf8542f65`.
Corrected input preparation: REA
`ev_b2c0ab515a7c9124189408eda3d057bfa9bda3e40c77826fb2646c538e82ec98`.
Both captures exited zero with complete raw output and no runtime/archive
access. Strict scalar readers retain incomplete coverage when terminal records
are missing; preparation is not a successful native cast observation.

## Fresh run220 staged; not yet launched

`/home/blizz/gw2-sandbox/runs/run220-skill10176-lifecycle` copies the pinned
never-launched145 inputs without modifying them. Corrected cac0e9a server,
passive SLO/RCO helpers and composed GDB script retain the hashes above.
The UI has no Escape key, allows125 seconds for cold models and presses6
once with a fresh marker. It introduces no healing/recharge/damage response.
The1000ms duration remains an explicit diagnostic value.

Two no-listener preflights, five shell syntax checks, exact production-script
inverse/composition and executable-mode checks pass. Ten focused reader
checks pass, including rejecting a previously armed observer that has already
terminated. The unchanged observers' earlier mock results are reused rather
than reported as rerun. The1836-row input manifest includes file modes,
SHA-256 `4550b49c7843e8afeb4e0c2962c327cf3c4713d55b78c2fc53ac1a86fa61ae47`.

Staging receipt: `run220-skill10176-lifecycle-staging-receipt-20261106.json`,
SHA-256 `508c4282de475aa49518f1388046b66bfa02efb496a59b6056dce8a9ca180d92`.
Root reviewed the harness and sent one native window request to the existing
publisher. No grant is implied by submission. The launcher requires a fresh
exact active lease, completed publisher working-copy operation and removal
of the preparation sentinel. It checks the copy gate before and after
archive preflight. Preflight45s, run345s and kill-grace15s bound execution.
No client, archive access or shared container operation has occurred for220.
