# Build 209026: map loaded versus agents loaded

Static REA analysis, 2026-11-01, on the frozen executable identified in
[mission transfer](209026-mission-transfer.md). No live load or player entry
was attempted; the User Agreement hold remains unchanged.

## Two different completion messages

Client **420 / MapLoaded** is sent when the loader's map-stream stage
completes successfully. In `0x140952720`, the success branch at
`0x140952a1c` performs `0x140953450`, changes loader state from 7 to 9, then
calls the stored callback at +0x210 with a nonzero result. The callback
mapped in the transfer notes sends ID 420. State 9 is **MODELS_STREAM**,
not READY. The failure branch changes state to 8 and invokes the callback
with zero, leading to ID 418 / FailedToLoad.

Client **412 / AgentsLoaded** is a separate empty message. The default view
tick at `0x14094cef7` advances the embedded loader with `0x140951770`, then
checks all of the following:

- Loader state is 0xd / READY.
- `0x1414113e0` returns nonzero: mission context +0x228 has mask 0x2 set.
- The view's once-per-transition flag (whole-object +0x60 mask 0x8) is clear.
- Current view state is 0xd. The alternate state 0xf uses a different callback.

This branch calls `0x1414115d0`, which sends ID 412 through
`0x141418560` and logs **MsgSendAgentsLoaded** at `0x1422dd5f8`. The view
then sets its mask 0x8. The interface-pointer-relative equivalent of the
flag is +0x58, which the view-state transition clears at `0x14094f990`.

Server 1036 / BeginPlaying requires READY in the receiving view handler.
**Implementation inference:** for this default-view path, ID 412 is the
appropriate observed readiness candidate before sending 1036; ID 420 alone
is too early. This is a static dependency, not a captured successful
client/server exchange or proof that no other messages are needed.

## Loader stages

The update dispatcher `0x140951770` switches on loader +0x2c0. Embedded
assertions and diagnostics identify these states:

| Value | Name | Relevant dependency or transition |
| --- | --- | --- |
| 1 | LOAD_CONTENT | Prepared map definition must exist; map-content lookup is checked when its +0xd8 pointer is present. |
| 2 | LOAD_MANIFEST | Waits, without a timeout, for MapPortalLoader (`MpLoader.cpp`) to finish its async archive read (`0x1417e9740` starts it, `0x1417e9cd0` polls it). It also arms the 300 s SERVER_WAIT timeout (`+0x2e4 = 0x493e0`). |
| 3 | SERVER_WAIT | Requires the command-received flag at +0x2c4 before advancing to state 5; ID 1060's load helper `0x1409538c0` sets it unconditionally. After 300 s it logs "Completed STATE_SERVER_WAIT with error" and sets state 4, which has no case in the dispatcher. |
| 5 | MAP_DOWNLOAD | Waits, without a timeout, for the map resource at loader +0x218 (done +0x48, error +0x50), calling `0x140d8f620` every 10 s. An encrypted map without a 1060 key goes to state 6. |
| 7 | MAP_STREAM | Waits, without a timeout, for every streamer (+0x230) to finish (see below). Success advances to 9 and invokes MapLoaded callback; failure advances to 8. |
| 9 | MODELS_STREAM | Further model streaming before state 10. |
| 10 | MAP_ASSET_STREAM | Further asset processing before state 11. |
| 11 | AGENT_STREAM | `0x140951ce0` waits for a nonnull controlled character and streaming readiness/timing conditions. |
| 12 | READY_WAIT | `0x140952f00` waits for its countdown, finishes service callbacks, then sets 13. |
| 13 | READY | The view may emit AgentsLoaded once the connection and view conditions above also hold. |

This table describes the inspected normal path and selected branches, not
every state or every failure condition. The full model/agent streaming
prerequisites remain open. Empty server ID 1061 reaches a no-op default-view callback;
it cannot stand in for whatever population data those services require.

## MAP_STREAM wait (`0x140952720`, 2026-11-03)
MAP_STREAM is the stage before client 420. It loops over the loader's streamer list (`+0x230`, count `+0x23c`) on every tick and calls each streamer's status (vtable `+0x30`, at `0x1409527d9`):
- **1–3:** in progress. Timing marks are set in the view's mask (0x80/0x40/0x20), and the streamer is pumped with `+0x20(100, 0)`.
- **4:** done. The result is taken through `+0x40` into `+0x250`, a quality setting (setting 0xb5, giving 100 or 40) is applied through `+0xb0`, then the streamer is destroyed and its slot cleared.
- **5:** failed. The slot is cleared and the stage is marked as an error.

When every slot is done, the loader calls `0x140953450` and logs `MapLoader: Completed STATE_MAP_STREAM`. It then moves to MODELS_STREAM (9) and calls the stored callback, which sends client **420**. If any streamer failed, it logs "…with error", moves to state 8 and the callback sends **418**. An empty list completes at once.

There is **no timeout and no server input** here. A stop in MAP_STREAM means a streamer stuck in status 1–3 (map file data on the client), which the server cannot answer. Verified by disassembly of `0x140952720` (rizin, frozen exe).

## AGENT_STREAM conditions (`0x140951ce0`)

Traced 2026-11-02. Each tick, the stage function does the following, in order:

1. **Hard wait: controlled character.** It calls the charContext getter
   (thread-local +0x98, virtual +0x68) and returns at once while the result is
   null (`0x140951d59`). There is **no timeout** on this wait. Population that
   produces a finalized controlled character is the only way through.
2. **One-time streaming request.** On the first tick with a character
   (`+0x2d0` == 0), it calls service `0x14110f0a0` slot +0xf8 with the loader's
   +0x2a8 Float3 and the constant 1536.0, then sets +0x2d0. +0x2a8 is the
   Float3 from server 1060 ([mission transfer](209026-mission-transfer.md)).
   *Inferred:* this asks for world streaming around the 1060 position within
   that radius.
3. **Soft wait: world readiness, with a timeout.**
   - **Countdown.** The countdown is loader +0x2fc. It is copied from +0x2f8 on
     entry from MAP_ASSET_STREAM (`0x140951af5`). The loader initializer sets
     +0x2f8 to **120000**, or **5000** when client setting 0x3d is on
     (`0x140953d6b`). Each tick subtracts the stage's elapsed argument;
     milliseconds is inferred.
   - **Before it expires.** The stage continues only when `0x1409539c0`
     (loader +0x268) returns nonzero **and** service slot +0x40 returns 0.
   - **After it expires.** It logs the UTF-16 diagnostic `Map load hang on
     STATE_AGENT_STREAM detected: IsWorldReady: %d, CharLoadingCount %d`, with
     those two values, and **continues anyway**.
4. **Completion.** It runs the per-subsystem completion loops, logs `MapLoader:
   Completed STATE_AGENT_STREAM`, and sets **state 12 (READY_WAIT)**.

**Consequences:**
- Only the controlled character blocks indefinitely.
- A placeholder 1060 position that never becomes "world ready" costs at most
  the countdown, about 2 minutes by default.
- Statically, the loader polls for the character on every tick. Population can
  therefore arrive before or after 1060, as long as it produces a finalized
  controlled character. Whether the population handlers depend on map state
  themselves (for example, scene bounds for 37/46) was not examined here.

Evidence: `ev_d0556c8a75844d68557a107d01cad69724fdac3af5616cba522ddc9887698865`
(Rizin-run `agent-stream-windows.py`, SHA-checked), in the private
`~/re/gw2/notes/209026-agent-stream-1.json`.

## Controlled-character prerequisite

The service at thread-local context +0x98 is named **charContext** by the
assertion near `0x1408c00b3`. Its virtual +0x68 getter is the same getter
used by AGENT_STREAM. At `0x1402cdaa0`, its result is immediately checked
against an assertion named **controlledCharacter** (`0x14193aa40`). Other
call sites repeat this naming. Thus the loader waits for the controlled
character, not merely an arbitrary populated character list.

`0x1411b8550` assigns the controlled-character pointer at character-context
+0x98. A nonnull new value is checked against
`!m_controlledCharacter || m_controlledCharacter->IsFinalized()`; the tested
flag is character +0x178 mask 0x10. It updates character control state and
notifies listeners through the context's +0x230 event collection. Identical
old/new pointers return without those updates.

The direct caller `0x1411bc8e0` resolves its integer argument through
`0x1411b4500`, stores the returned character at its own +0x18, asserts that
it exists, then calls the setter. This establishes an existing-character
lookup before assignment. The incoming message and creation/finalization
sequence are not yet sufficient for bootstrap; no speculative population
packet was sent. Follow-up [character-control analysis](209026-character-control.md)
identifies server 444's equal-index branch as one path into this setter,
and confirms that the loader's getter itself rejects unfinalized characters.

## Evidence

Private bundle: `~/re/gw2/notes/209026-map-ready-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_31a77b2e7d0f7be98ea822ab01a54f31a0ca4a652f16f2212217e9309b31e4f2` | Loader field writes and final READY assignment. |
| `ev_8e76c8dc6f49e2beffab7434e8bb8b8f3155c7fad90f14ba7c72e3cb72ca6486` | State transitions and stored callback invocation. |
| `ev_17bd155d1e12bdd665eeaa47f96dcc68eee7d3bafd37a6d0cf86795ead5d20d2` | Successful map-stream callback precedes READY; state diagnostics. |
| `ev_c2474537f568c4112e9d97d7fd87e5bebe5e1bbcb968224274d3d2fb401861f3` | Agent-stream dependencies and model/agent state names. |
| `ev_14d59c1c6fc8c109709e5dd7bea009472f42ade96499e824ac271747e9389760` | View tick's READY/connection/once checks and empty ID-412 sender. |
| `ev_fca0caa0d42ee5c567c233a2b024d48dcac8397953698ae4839cf1ee332fac16` | Ready-message wrapper, connection predicate and bounded view-tick disassembly. |
| `ev_4dc55aed33a651746678610d0b35ff21020c8ad1e0837bf93a99e0c3ebf3ee74` | Explicit MsgSendAgentsLoaded diagnostic, state-1 handler and flag reset. |

Additional private bundle: `~/re/gw2/notes/209026-controlled-character-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_2c86467144a4bc4eccbd9c31be5d5df964f13a2bf8b214d1601ed163edb52470` | charContext assertion identifies the service. |
| `ev_18d79a638e6e59f0074bb9a8253b45fa0456d50b68fc4b8b07a6c5d8eac4b2c1` | Getter result named controlledCharacter. |
| `ev_0a4bfc57dd1a39f280b5098df61696903e0ee31797d74fd43fb4823a5003d3d5` | Assignment, finalized check and notifications. |
| `ev_bc78bfabb09dd7c54a743844115d4719e1871c8f1f9d04cfcde1f36dacfdc4e0` | Direct caller of the setter. |
| `ev_38092b0a47aee7451d279fe7d3cfb990be98e2ebbbea7c08e8ed2b0d9185f1a0` | Caller resolves a character before setting control. |
