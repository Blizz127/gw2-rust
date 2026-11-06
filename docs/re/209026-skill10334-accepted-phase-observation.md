# Build209026: bounded10334 accepted-phase observation

The initial typed0x78 constructor and an existing-order update have different
payloads. The initial form is13 bytes. Typed0x78 vtable142179988 +0x40 is
141287390, which asserts length4 and forwards bytes2/3 to141287410. The
receiver14127ef80 selects this callback when packed flag0x04 is present,
then marks the matching existing order active and calls its start callback.
Thus `[78,04,step,rawFlags]` under server34 updates and restarts a phase.
Server84/0x47 only updates fields and returns; it does not restart.

Same character, outer slot, behavior value and kind0x78 are essential. A
different behavior value can remove the current order before flag0x04
suppresses construction; matching behavior with a different kind asserts.
The raw codec supplies no admission or lifetime guarantees.

The selected10334 primary variant is0x3ef495582 under token0x4d195684f7.
The retained initializer and21 events derive these half-open event ranges:

| Step | Authored quantity | Cumulative offset | Event interval |
| --- | --- | --- | --- |
| 0 | 40 | 0 | [0,1) |
| 1 | 240 | 40 | [1,3) |
| 2 | 720 | 280 | [3,18) |
| 3 | 69 | 1000 | [18,18) |
| 4 | 30 | 1069 | [18,20) |
| 5 | 903 | 1099 | [20,21) |

The wrapper's initial scalar is1180−(720+69)=391. First start initializes
order+0xf0 to clock+391. Updating to step2 adds789; completion at steps2/3
increments+0xec. Each start selects only its current interval and applies
the character's virtual+0x2b0 rate to the offset. No automatic advancement
is established by these paths. Ordinary event7/9 consumers manipulate visual
streaks. Display multiplier8 is not an authoritative strike count or timing.
The authored recharge base10000 has no overrides; its start/cancellation
policy and effective gameplay duration are separate inputs.

The isolated increment adds `--mission-observe-mesmer-sword2-action-ms D`,
default off, with explicitly supplied1..65535ms. It requires the existing
earned-level observation configuration and validates actual earned level2,
grant10334, controlled caster, slot6, primary33355/set4, a living player and
living hostile target. It does not grant at startup. A pending Ether Feast
prevents this diagnostic from starting.

Its fixed diagnostic schedule is initial step0 at0, same-order restart step1
at2D, restart step2 at4D, and matching84/45 at6D. Every34 uses WordD,
retains the original request header/correlation, and adds outer admission
mask0x10. Updates use raw step flags2 (start gate0x4000 set, completion-flush
0x8000 clear). These choices are explicitly diagnostic, not retail timing
or flags. If an entire next phase deadline was missed, the scheduler finishes
instead of emitting a compressed update burst. It participates in due_at.

There are no10334 health, damage, aggro, reward, evade, chain-count or recharge
effects. Admitted normal attack/heal replacement finishes the old diagnostic
before the new response; rejected inputs and raw00/29/2a leave it untouched.
Same-owner death or target loss may send matching45. Character/GUID/map/session
loss, disable, transfer and disconnect discard pending work without old-agent
frames. No pending action survives a connection or enters carry/disk data.
Matching45 may defer while the client order is active; this is not a proven
immediate cancellation command.

Focused tests cover literal update framing, raw fields, truncation, encoded
requests, fixed phase/release order, no effects, replay, rejected and admitted
replacement, target/life/owner loss, delayed deadlines and configuration.
The existing chain/heal and saved-profile controls remain separate. This
increment has no native acceptance evidence; run250 proved only grant/selection
and request10334. A later bounded native observation can verify the selected
wrapper, reused steps, completion and slot release. Injected durations cannot
recover a retail server strike/evasion or phase policy.

Private complete Evidence:
`209026-skill10334-step47-contract-evidence-20261106.json`, SHA256
`10fd6788b7ca61c7c96c08ac680406b6c672ef26e903e6ec65fa70c5221a1230`;
`209026-skill10334-reused-order-update-evidence-20261106.json`, SHA256
`11e0d2ca905c3da20c4393b2d603259cca583a2d4aee8f7473b9c0bbfd838009`.
The latter's decisive complete callback capture is
`ev_69ebc0ec41a7dd36de26fb89df109c642378d628a53cc4809f11aa37f76f355d`.
The source/derived contract pins all seven reused bundles in
`209026-skill10334-accepted-phase-contract-findings-20261106.json`, SHA256
`0c6b5dd4d1c15e0e69e8e469a69d3b0e1faa73bc6784e8abfdc3611a2c812e21`.
Frozen PE SHA256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
