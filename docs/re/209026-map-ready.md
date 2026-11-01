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
| 2 | LOAD_MANIFEST | Advances to server wait after its inspected checks. |
| 3 | SERVER_WAIT | Requires the command-received flag at +0x2c4 before advancing to state 5; ID 1060's load helper sets this flag. |
| 5 | MAP_DOWNLOAD | Resource acquisition; error paths lead to state 6. |
| 7 | MAP_STREAM | Success advances to 9 and invokes MapLoaded callback; failure advances to 8. |
| 9 | MODELS_STREAM | Further model streaming before state 10. |
| 10 | MAP_ASSET_STREAM | Further asset processing before state 11. |
| 11 | AGENT_STREAM | `0x140951ce0` waits for a nonnull service result and streaming readiness/timing conditions. |
| 12 | READY_WAIT | `0x140952f00` waits for its countdown, finishes service callbacks, then sets 13. |
| 13 | READY | The view may emit AgentsLoaded once the connection and view conditions above also hold. |

This table describes the inspected normal path and selected branches, not
every state or every failure condition. In particular, the exact meanings
of the service's nonnull object and all model/agent streaming prerequisites
remain open. Empty server ID 1061 reaches a no-op default-view callback;
it cannot stand in for whatever population data those services require.

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
