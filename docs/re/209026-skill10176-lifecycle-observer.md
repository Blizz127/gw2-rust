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

## Fresh run220 preparation

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
At this preparation checkpoint no client, archive access or shared container
operation had occurred for220. The subsequent run is recorded below.

## Run220: request and diagnostic reply observed; lifecycle incomplete

The publisher completed the separate working-copy operation and granted
window30 at06:39:42Z. Root verified the exact active grant, copy marker,
native publisher identity and staged pins at06:40:36Z, then launched once.
The seven mapped-asset checks passed before launch. UI and outer run exited0;
the container was removed, frozen/working PE hashes matched and console
privacy checks passed. Root returned window30 complete at
**2026-11-06T06:46:22Z**. Run145 remains unchanged and unlaunched.

One native10176 slot0 request reached the observer server, which emitted one
explicit1000ms diagnostic echo. This confirms request/carrier behavior only.
SLO recorded armed and terminal events, with one callback rejected,
`window_open=false` and partial coverage. No typed-start/completion sample
was accepted. RCO sampled the native control listener: its guarded owner had
second level1, effective attribute6=0 and a null recharge context at that
instant. It recorded no query/630 events and no terminal. That isolated null
sample does not establish later context state or an accepted healing cast.

No retail duration, animation completion, healing or recharge success is
established. The rejected producer sample requires a bounded guard audit;
guards must not be relaxed to turn incomplete instrumentation into a pass.

Private REA receipts:

- `209026-run220-window30-verification-20261106.json`,
  `ev_7f2e020c5807e0c84fdef8392fd5e1ff2edb40f69f4ce5160ffb28f5eaa657ab`.
- `209026-run220-lifecycle-launch-20261106.json`,
  `ev_f2dc465f0262c5091ed347de6a15ac3a2b21097caadd21fc09cd94567b8a83a1`.
- `209026-run220-lifecycle-readback-20261106.json`,
  `ev_6c151d910951cc08114de60592800ece3d292fad789b42b90e647aebeaa611bb`.
- `209026-run220-window30-return-20261106.json`,
  `ev_35c445f13de4943093d1fa78d89985c49e4b2c7740f7f6bc2ada5724014d3276`.

The bounded rejection audit excludes missing/stale marker, source/RIP/inferior,
deadline and memory-read failures. Only the producer hook was enabled; one
deliberate ownership/SkillDef/slot guard rejected the callback. The existing
aggregate counter cannot identify which guard. The+58/80000 request gate and
null target are reported fields, not rejection causes in this observer.
The retained producer ABI agrees with its register handling. Known cached
definition alignment does not prove runtime pointer alignment.

The marker was touched immediately before key6 at06:45:32.986234Z. Because
no qualifying request opened the observation window, RCO queries remained
disabled. Its earlier context-null sample is not a cast-time observation.
This is an instrumentation coverage failure, not evidence of failed native
skill execution. The minimal next diagnostic is a bounded first-failed-guard
enum at the same producer hook, preserving every guard and access order.

Private review: `run220-slo-rejection-review-20261106.json`, SHA-256
`3aca4b50907bebbae9cda9706d1614782f3ee510185884a2d43a4db3ef25fbab`.
REA `ev_23bb78c21d8fdef3d01e577f39d0be7f631487f6b96775df38d192dfecea2913`;
complete bundle `209026-run220-slo-rejection-audit-20261106.json`, SHA-256
`8eb07a0b6720613e4bc2e76bbdd49c5bf6046c0a3ce600a75a0712efa19fa558`.
No observer change or new runtime occurred during this audit.

## Run222: exact rejection is the SkillDef alignment check

The separate SLO145R revision adds at most four distinct first-failed-guard
records at the existing producer site, preserving every guard and access
order.37 observer,5 standalone-reader and11 compact-reader tests passed,
including differential register/memory traces. Fresh222 uses pristine145
inputs, the same diagnostic1000ms server and validated220 UI/timing.
Original runs/helpers remain unchanged. Its1838-row manifest SHA-256 is
`087b907f7bd5adc222ae61fb715c8b1cbc9bfe9bf71249b6157b72bd8d8115c4`.
Staging receipt: `run222-skill10176-rejection-staging-receipt-20261106.json`,
SHA-256 `4f8b2f7a1533b41b19f8e3498183f62f0ec76f8bf2da4ec50f06ae8972eac7c9`.

Window31 was granted06:57:40Z and verified06:58:43Z. The single run exited0,
with one10176/slot0 request and diagnostic reply. At263784ms the new event
reports `skill_definition_alignment` with a fresh marker. Earlier ownership
and control guards passed; the non-null value read from R9's reference failed
the observer's8-byte check before key/flags sampling. No pointer or low-bit
pattern was emitted. This does not prove an encoded reference or authorize
masking bits; compare the native dereference contract next.

The lifecycle window never opened. RCO sampled only control-time context
nullness and fixture level/attribute inputs; no queries/630 events occurred.
Native typed execution, healing/recharge and retail timing remain unconfirmed.
UI/cleanup/privacy and PE hashes passed. Window31 returned complete at
**07:05:08Z**; no root runtime lease remains.

Private REA receipts:

- `209026-run222-window31-verification-20261106.json`,
  `ev_18a8abf9f6bacb411a2c1044e5d216601ed5e6ee09c18c5101954fd1f6381e1d`.
- `209026-run222-rejection-launch-20261106.json`,
  `ev_3c5a00f8da3c78f47f4baa8b343cdd58793299ddc93b766357d8af9eeec44eed`.
- `209026-run222-rejection-readback-20261106.json`,
  `ev_bd45b31afe5d6bdff4199e36e12da809e6451be207b87e0da35673c698d59e8b`.
- `209026-run222-window31-return-20261106.json`,
  `ev_e044e59426ab7f100604b18a2c725e3d104f6cb5bd443121174c1b681a44204d`.

### Native contract corrects the observer's alignment assumption

The complete producer loads `RAX=[R9]` at141283fa4, checks nullness, then
reads flags at`[RAX+38]` directly. Its ordinary packet path reloads the same
reference and reads key`[RAX+28]` at141284212. Complete14129a4d0 directly
reads discriminator+58/body+60; complete14127de10 copies the raw reference
into the predicted order and reads flags+38. None of these paths masks,
shifts, decodes or tests alignment. The8-byte requirement was introduced by
the observer and contradicts the observed accepted native-reference form.
No four-byte assumption or pointer-bit masking replaces it.

The new separate SLO145N revision removes only that SkillDef alignment test,
preserving nullness, bounded reads, exact skill identity and all ownership,
source/RIP/inferior/marker/lifetime guards.42 observer tests pass, including
all seven nonzero mod8 residues with exact read-address checks and negative
controls. Five standalone and eleven compact-reader checks also pass.
Fresh run223 is staged from pristine145 with the same diagnostic1000ms
fixture. No source change to the game or new runtime is implied by staging.

Contract note: `skill10176-native-reference-contract-20261106.md`, SHA-256
`93012ab8c9a934215e4a11f5bb0d926064b56fdc6ebaac5bf845716ed9afc28f`.
New source bundle: `209026-skill10176-native-reference-source-20261106.json`,
SHA-256 `bcefe4a8d49f709ebe0a9a43296c065461eb8fc06dd8e6bbfc97ec2ec199aafd`.
Run223 staging receipt:
`run223-skill10176-native-reference-staging-receipt-20261106.json`, SHA-256
`5c6f55a4c84202addbaa609f888255cbce75d27dbd245ea62546b1a2d8b0af09`.
Manifest SHA-256:
`93a41ae9a5527c125c8242b809dcc74dadd76b51bb398d021de1b9570eacfb53`.
A fresh window request was submitted to the sole publisher. Previous inputs
remain unchanged and no grant is inferred from that submission.

## Run223: typed start and completion callback observed

Window32 was granted07:12:16Z and independently verified07:13:12Z. Root
launched the pinned native-reference observer once. The run and UI exited0;
one10176/slot0 request received one explicit1000ms diagnostic reply. The
observer accepted all four callbacks with zero rejections:

| Event | Bounded native observation |
| --- | --- |
| Request | Controlled owner,10176/slot0; request gate does not block; recharge observer notified. |
| Receiver | Kind0x78, empty order slot0, packed flags0; outer flags41948179. |
| Start dispatch | Current owned order active in slot0, flags18688/0x4900; pending typed callback dispatch. |
| Completion return | Same active slot and matching behavior; flags20736/0x5100; callback returns false, so it does not request the normal release path. |

The lifecycle terminal reports inferior exit and partial coverage. No
post-completion flag clear, order release or visible animation completion
was sampled. A false completion return is distinct from a failed callback.
RCO opened its window after the guarded request, but recorded no query-return
or630 events and no terminal. Its earlier control-time sample still has null
context, second level1 and effective attribute6=0. It does not establish
cast-time context. Healing, recharge and retail timing remain unknown.

The strict readback, console privacy and frozen/working executable hashes
passed. Root verified shared-container absence and returned only
`window_32.cleanup` at **2026-11-06T07:22:15Z**. Resource cleanup succeeded;
incomplete observer coverage is retained separately. This diagnostic changes
no live server/profile and is not a retail Ether Feast implementation.

Private REA receipts:

- `209026-run223-window32-verification-20261106.json`,
  `ev_57c5a490f555fa7649c959e05360f42b6181291aefff1b8a5c810222286cead1`.
- `209026-run223-native-reference-launch-20261106.json`,
  `ev_7b44e2903b497a7970a68149ded67190aa412cd5ba2f5e7ad6e0d0239a44b23e`.
- `209026-run223-native-reference-readback-20261106.json`,
  `ev_c3cee6ee1623a8712065204e674f5a2e098732b30df48fc3b54ac2aabcf57427`.
- `209026-run223-window32-return-20261106.json`,
  `ev_22a983fdee8eef0963a8ec345ce40b2db774d043acd50fe1e3678571df482165`.
