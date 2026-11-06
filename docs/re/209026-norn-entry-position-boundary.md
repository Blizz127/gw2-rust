# Build 209026: Norn initial-position boundary

Retained REA evidence audit, 2026-11-06. The configured spawn
`[-3200,-25600,-3742]` remains a reconstruction choice. No authenticated
Norn entry selector or retail spawn was recovered in these inspected paths.
This audit reuses 16 complete earlier REA records; it makes no new executable,
archive or runtime observation and changes no placement.

## Initial character position and streaming centre are separate inputs

**Source-observed:** server 37 field 10 is Point3. Handler `0x1410335c0`
constructs the command through `0x141035c50`, copying decoded message
`+0x1e..+0x2d` into command `+0x88..+0x97`. Executor `0x141039b10`
passes that point to base-agent constructor `0x141024aa0`, then the full
or simple collision factory. This is the transmitted initial-position
input, not a recovered Norn marker selection.
[Creation layout](209026-agent-creation.md),
[position dataflow](209026-agent-position.md).

Both constructors convert map XYZ to collision `[X/32,Y/32,-Z/32]`,
retaining the transform tag; tag 0 has the identity-resolution path. The full
constructor can clamp to scene bounds, while the simple constructor checks
expanded bounds. These rules establish units and handling, not an authored
spawn, facing, clearance or model-foot position.

**Source-observed:** server 1060's separate Float3 reaches view
`0x14094e6c0`, loader `0x1409538c0`, and loader `+0x2a8`.
AGENT_STREAM `0x140951ce0` later passes it with 1536.0 to service virtual
`+0xf8`, after a controlled character exists. This supports its use as a
streaming centre; the inspected path does not derive server 37's character
point from it. [Load path](209026-mission-transfer.md),
[streaming consumer](209026-map-ready.md#agent_stream-conditions-0x140951ce0).

The [creation continuation](209026-creation-mission-binding.md) sends empty 430.
The complete 1065 handler `0x141412fa0` and common join `0x141417b80`
carry the destination/map and join scalars, not Point3/facing or a Mission
interest-point index. This statement is limited to those bodies, not every
possible character-metadata field.

## Authored points exist; the entry selector remains unbound

Shared map archive 187611, used by 379 and 28, contains three 32-byte `msn0`
v2 interest points with position, forward vector and label. Module 11 loads
them; getter `0x140c3c320`, at Mission vtable slot `0x141d623c0`, returns
a caller-selected row. The retained exact-reference follow-up still does not
bind a caller/index to fresh Norn entry. Neither archive presence nor a label
selects a tutorial spawn. The points are also not proved to be exclusively
camera/debug data. [Complete inventory and bounded negative result](209026-norn-map-mission-points.md).

**Inference:** agreement between locally configured 1060 and 37 positions, or
a successful local entry, cannot establish retail authorship. Quest sectors,
spatial audio and terrain intersections likewise do not supply that binding.
No source-derived replacement coordinate is ready from this evidence.

The minimum new evidence is either:

- A concrete typed caller whose receiver originates from module 11, its index
  source, and a fresh-character map 379 entry binding with position/facing
  conversion; or
- An authoritative original initial map 379 controlled-character position and
  facing tuple: server 37 plus any 46/86 updates before play, distinguished from
  a waypoint, portal or relog. That can establish the server-selected entry
  independently of whether a client-shipped marker supplied it.

Another terrain export, global numeric search or repetition of the already
bounded getter-reference/ID 11 query does not close that dependency. There is
no claim that all client data lacks spawn metadata.

## Retained evidence

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Decoded archive 187611 SHA-256:
`2dc9c92929362a10cdb22b5350957eac764ccc1d53c8a97b01c5798223211bbc`.
These are the identities bound by the earlier captures, not fresh reads.

Private files under `/home/blizz/re/gw2/notes/`:

- `209026-norn-initial-spawn-boundary-evidence-20261106.json`: 624850 bytes,
  16 complete retained REA records and source-bundle pins; SHA-256
  `92bb57e9b68e4326db3e3e28ffe2e3788a3e7542a5e103f3783b84b23a4a3d8e`.
- `209026-norn-initial-spawn-boundary-findings-20261106.json`: 8387 bytes,
  claim-to-evidence mapping and source-note pins; SHA-256
  `6e25850bb82f4a051ef06c5d066dea5d3cc40acbf0bff02e76589c039a43615e`.

Key records are `ev_30932644a3ad7ebcf1804bf81fe9cb390d818a8418e85544a1e853a4a3734795`
(message-to-command point copy),
`ev_d0556c8a75844d68557a107d01cad69724fdac3af5616cba522ddc9887698865`
(streaming-centre consumer), and
`ev_61f431788f127ee875a89c86b7f02de46ff00aa5b802335055e90bc4eec22871`
(creation/redirect). Remaining records cover constructors, bounds, authored
Mission points and the bounded missing-caller result. Aggregate-truncation
flags on two earlier captures retain all raw bytes and normalized frames;
complete Evidence and its limits are preserved. No raw disassembly is copied
into this note, and no runtime/resource or owner-store access occurred.
