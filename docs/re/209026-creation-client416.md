# Build 209026 creation-scene progress message 416

Static REA inspection, 2026-11-05, of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No client was launched or modified for this investigation.

Client **416 reports the current creation scene together with the current
creation data**. The inspected UI path changes scenes and updates its controls
locally before sending 416. This path does not install a completion callback or
set a wait flag, and it supplies no evidence for a required 416 acknowledgment.
Missing creation-screen controls therefore do not justify inventing a reply.

## Exact field and caller binding

Protocol-0 group-39 client table `0x1422ddaa0`, index 5, points to descriptor
`0x14260d130`. Its root identifies message 416. The extracted
`data/protocol/209026/mission.xml` records a Byte followed by the same fields as
415: String(20), fixed 48 Bytes, two Bytes, Dword, counted Byte array up to 32,
and fixed 24 Dwords. These are wire encodings, not fixed-size host structures.

The send chain is:

1. `0x14029edd0` updates creation-scene controls and reaches `0x14029f02e`.
2. Getter `0x14129c3d0` returns the creation context. At `0x14029f033`, the UI
   passes the low byte of its field `+0x78` to context virtual slot `+0xd0`.
3. Context constructor `0x14129ac90` installs vtable `0x14217dca8`. Slot `+0xd0`
   at `0x14217dd78` is `0x14129c740`.
4. `0x14129c740` copies all six four-color rows, supplies its context data
   `+0x458` (appearance, name `+0x78`, profession `+0xa0`, level `+0x30`,
   answers `+0x38`), and forwards the unchanged leading byte through
   `0x141410cc0` to sender `0x141418590`.
5. The sender writes ID `0x1a0`, serializes the leading byte and creation data,
   obtains the mission connection, and calls `0x140ff0110` with host size
   `0x2a`. It returns without a response-dependent branch.

The diagnostic at `0x14029f063` names the switch field **`m_currentScene`**;
its source string names `Game/Ui/Scenes/ScnCharCreate.cpp`. The switch covers
0–9. This binds the leading byte to the scene, not a request sequence number.
Scene-transition paths write that field at `0x14029deb3` and `0x14029e843`,
then invoke `0x14029edd0` after updating local controls. There is no network
response check between those writes and the message send.

## Nearby server 1052 is not an established reply

Server 1052 has Byte + three Dwords. Handler `0x141412960` converts its Byte
to the predicate `byte == 1`; when true it adds the first Dword to the local
time helper and stores a deadline at mission context `+0x14c`. It dispatches
that predicate and the three Dwords through the mission listener registry
`+0x460`, using trampoline `0x14025553c` (listener virtual slot `+0x80`).

The creation-scene constructor at `0x14029b5c0` installs listener vtable
`0x141933b00` at its `+0x68` subobject and registers it through
`0x141411670`, which selects mission context `+0x460`. This listener's slot
`+0x80` is `0x140163990`, a bare `ret 0` stub. Thus this scene does not consume
that notification to unlock its controls. This does not establish the purpose
of 1052 for other listeners, nor all server-side uses of 416.

## Evidence and scope

Complete captures are retained privately in
`/home/blizz/re/gw2/notes/209026-creation-client416-20261105.json`.

| REA Evidence | Observation |
| --- | --- |
| `ev_0dc802ac6122ab48140b6b830d2054d47a0d9c0cea040b4f18f73c561cc16ac5` | Exact 416 sender. |
| `ev_9b01a24b3bb3d952bee5c7b9b79db542da4f38381892fabf4b3bd72a5deeed5c` | 1052 handler and sender's direct trampoline reference. |
| `ev_00d288befeaf2a6e860072a8f99d1139523010fea453a78861c9e1287d1c16b0` | Listener dispatch trampoline selects virtual slot `+0x80`. |
| `ev_ffc7906c741f13dd653ab6a1f9465bcec39ccd80410a14d2a54518d3f4111d46` | Complete creation-context 416 method and field flow. |
| `ev_db5c58628bf20e9e403705a3a71c3bbf21b53c01a929b28ee9bcc329bfc7e66c` | Context virtual table and 416 descriptor bytes. |
| `ev_63ce30cf6f66000173144f18a4425ad6015a931898458d4a08095176b1497dea` | UI update function, context vtable installation and UI listener setup. |
| `ev_24c37d3da4416b042e0587f241561673aa75ff0f902987115f6aa582a29c1f26` | `m_currentScene` diagnostic, listener slot and local call sites. |
| `ev_717b2b76ef2e1860bc39db0fc58d268b9dedb9a30dd72a0698ca1cdd103fea98` | Registry selection, local scene transitions and bare-return stub. |

This is bounded static evidence. It does not explain the missing controls in
run119 or prove a viewport cause; the viewport experiment remains independent.
