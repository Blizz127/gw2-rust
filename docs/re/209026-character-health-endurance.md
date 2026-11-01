# Build 209026: health and endurance initialization

Static REA findings, 2026-11-01, for the frozen executable identified in
[mission transfer](209026-mission-transfer.md). Both messages are registered
in protocol 0, group 0's character table. No runtime replay occurred.

## Server 256: health component creation or update

Descriptor `0x1425e22b0`, pair `0x14216f490`, handler `0x14124fa90`:
**Dword character index, Byte, six Floats**. The handler requires a nonnull
raw character, then calls `0x1411d6310` with the six floats and the byte.

The helper uses whole-character +0x3e8. Existing-component getters and the
removal path explicitly assert **m_health**. If null, it allocates 0x88
bytes and calls constructor `0x1412071e0`, whose vtable/source diagnostics
identify `ChCliHealth.cpp`. The constructor stores the owner at +0x78 and
initializes two `CharMeter` objects at component +8 and +0x24.

If the component already exists, the helper invokes six setters rather
than allocating again. This differs from coreStats and endurance creation,
whose helpers assert that no component exists.

Numbering floats F0 through F5 in wire order:

| Float | Existing-component setter | Storage/operation established |
| --- | --- | --- |
| F0 | `0x1412076b0` | First meter's current value at component +0xc, through `0x141275fd0`. |
| F1 | `0x1412076f0` | First meter's maximum at +0x10, through `0x141276030`. |
| F2 | `0x141207770` | First meter +0xc, component +0x14, through `0x141276120`. Semantic name unresolved. |
| F3 | `0x1412077b0` | Second meter's current value at component +0x28. |
| F4 | `0x1412077f0` | Second meter's maximum at +0x2c. |
| F5 | `0x141207730` | Component +0x40, also exposed by getter `0x141207490`. Meaning unresolved. |

The current-value setter clamps ordinary finite inputs to [0, maximum].
The maximum setter asserts `valueMax >= 0.0f` and preserves the previous
current/maximum fraction while changing the maximum, with a zero-maximum
special case. The update helper sets each maximum before its corresponding
current value. The constructor follows the same ordering. No assertion here
establishes that arbitrary NaN/infinity inputs are usable.

The Byte is stored at character +0x188 **only on the allocation branch**.
That branch also emits a character-component event and can set character
flags +0x178 mask 2 after checking player/state predicates. Neither the byte's
meaning nor those predicates' complete role is assigned here. Sending the
same layout to an existing component therefore does not repeat every
creation-side effect.

## Server 217: endurance component creation

Descriptor `0x1425e00d0`, pair `0x14216f300`, handler `0x14124ec90`:
**five Dwords**. The first is the raw character index. The handler passes
the remaining four values to `0x1411d6240`, which checks **!m_endurance**
at whole-character +0x3d0 and allocates 0x68 bytes through `0x141222b40`.

That constructor stores the four values at component +0x10/+0x14/+0x18/+0x1c
in wire order, copies the first to +0x20, and retains the owner at +0x28.
It also consults the named `CharEnableEnduranceDeficitTracking` setting.
Units and individual scalar meanings remain unresolved; they are Dword
varints on the wire, not four raw little-endian floats or a fixed 16-byte block.

These traces establish message roles and component lifetime behavior. They
do not establish all required initialization messages, valid gameplay
values, or a completed controlled-character/map startup.

## Offline framing check

The hash-guarded extractor wrote private mode-0600
`data/protocol/209026/character-health-endurance.xml` (733 bytes, SHA-256
`d37b82b7e323ece6dbdc9bdd1d8e76b9a51659504f57aa9fe22ed6c73e99e915`).
The existing census consumed two hand-authored frames completely: server
256, 29 bytes; server 217, 15 bytes; total 44. A truncated final health
float was rejected. The endurance values exercise varint boundaries and
are not asserted to be meaningful gameplay values. No traffic was sent.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-health-endurance-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_73100f8ad1f48465e0985faf7b88e68f8054e1d27c3c5645b54a6574098c01cb` | Named health/endurance assertions and component offsets. |
| `ev_55c063c877a889b5c21b5b66479b290a87e336eb46f89b54bd6ece6470230431` | Allocation/update split and endurance allocation. |
| `ev_6c642f060b982734db34d161b3d218aca3946bfcff0e436038af5045f0c0c724` | Allocator call sites in message handlers. |
| `ev_a5ca3498e88fd5243f22244c7a2c57043198bb448274616b699ae3cee084329a` | Full handler arguments and endurance registration pair. |
| `ev_c29007260277eb4417522d41052326a42c4d0e371633709d3f9c89c374c5a89f` | Health registration, endurance descriptor and component constructors. |
| `ev_3aedb5b10a834e84b42a900032b957825320972c009eb314540f5e9f98516d77` | Health descriptor, float setters and meter operations. |
| `ev_38375ed008e6945ca241fcefe66736e0c1c56484c2aa35478741bc9481c7f97d` | Health vtable/getters and named meter maximum assertions. |
| `ev_d8c53bac43bb3003a4d2be082894a8c2c2aa1eb46fd4436e68ee2bbaed299c1a` | Hash-guarded two-message schema extraction. |
