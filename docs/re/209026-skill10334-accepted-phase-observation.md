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
increment initially had no native acceptance evidence; run250 proved only
grant/selection and request10334. Run260 below later verifies the selected
wrapper, reused steps, completion and slot release. Injected durations cannot
recover a retail server strike/evasion or phase policy.

The retained event3–5 callback also does not establish an evasion marker.
Its Boolean requires unsigned current step at least the wrapper's virtual+0x48
result and a matching event index. For the expected10334 wrapper range[2,4),
that getter returns4, so this exact callback cannot set whole-view+0xb78
mask0x10 during diagnostic steps0/1/2. The known finish helper clears the bit
with other local view/action fields. This conditional source deduction neither
names the bit nor establishes all its writers/readers. The field must not be
used as a step2 evasion predicate. The finite retained audit and17 complete
records are pinned by `209026-view-action-flag-findings-20261106.json`, SHA256
`57a9d929285a2a27f57641938a5b8f436d9ae5cba29dba12f19f362172271032`.

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

## Run256: observer armed, combat sequence did not start

Fresh run256 uses exact ec69643 with both diagnostics enabled only in the
prepared224 near-wolf fixture. Its six-site passive observer composes into the
production runner with an exact byte-for-byte inverse; no earlier SLO145 probe
is installed. Root reviewed the source, all27 metadata pins and1830 manifest
rows. Five changed reader tests, three UI cases and two no-listener startup
checks pass; the unchanged observer retains its29 host tests.

The native observer reports all six source-guarded sites armed. The UI then
exits8 without a target or skill event in the checked post-baseline log. The
observer records zero callbacks and closes on inferior exit,256515ms after
arming. Its request window never opens. This verifies native installation;
it provides no10334 reception, phase, completion, damage or evasion evidence.
The retained UI log identifies `insufficient combo observation time` before
the first Tab. File timestamps place that check around196 UI seconds; its
condition required elapsed+95<270. The earlier world reserve200 and cold
settle125 no longer accommodated the enlarged combo reserve. This is a test
harness budget defect, not a client lifecycle rejection. Run250's earlier75s
reserve had barely fit at about194s. Neither used run/prefix is relaunched.

The approved fresh260 budget uses UI330, world reserve260 and per-target-stage
reserve110, preserving the existing world-admission cutoff below70s. It keeps
cold125, key2 reserve27, quiet21 and observer arm420 unchanged. Wrapper405,
preflight45 and cleanup15 bound execution at465s, with490s foreground capture.
Nine focused budget cases pass, including the observed196s refusal, startup
boundary69/70 and all four target stages. These are operational bounds, not
guaranteed client timings. The complete retained audit and two REA captures
are pinned by `209026-run256-ui-budget-findings-20261106.json`, SHA256
`74ef7c35501358f2051a28a87aa2435a393edf62236d3c9a9d987a1af0aef4c2`.

Window51 returned at14:42:16.887659Z. Strict readback/return verifies all61
immutable non-prefix inputs, the unchanged224-byte prepared record, frozen
and working executable hashes, credential-safe console and absent container.
Launch/UI8 remains a failed observation despite successful cleanup. The live
profile and publisher baton are unchanged; this run is not a deployment.

Private complete Evidence in `~/re/gw2/notes/`:

- `209026-run256-mesmer-sword2-phases-launch-20261106.json`, SHA256
  `aa49d0abb3ff2865afeb8a615eb8061d33c81ec852d7237003bd32f2fc424df6`,
  `ev_76ff5188c2c7c716062dfd2009cb744d37d7e9897b73feb2d4e87eb5e862f985`.
- `209026-run256-mesmer-sword2-phases-readback-20261106.json`, SHA256
  `3ee929a9d2525f71e20640ce5bed96e0e4c48f3d92ac8fc6c2d825d1825f2879`,
  `ev_2c6101ae62fec34fa7601d592e3c6519b470c48a8d953f5e01f67547d29884ed`.
- `209026-run256-mesmer-sword2-phases-return-20261106.json`, SHA256
  `915c305fe730b8ed391947dd06d83dfb0a9e335e7134349b336ce0368e92b7d1`,
  `ev_673f289e39ea64b42d1a49d8a139819c18ef8bf2f6b9074a9a5bb1ab4fd526c0`.

## Run260: native diagnostic phases and slot clear observed

Fresh260 uses the reviewed330-second UI budget and465-second total envelope,
the same exact ec69643 binary, prepared224 record, actor/profile inputs and
six-site observer as256, and a pristine145 prefix. Root verifies34 metadata
pins and1830 manifest rows. The nine changed budget cases and two startup
checks pass; unchanged reader, input-sequence and observer tests are reused
through exact pins/source inverses.

The four ordinary requests10170/10171/10172/10170 match health118→88→58→28→0,
then logged2500XP, earned2, the605/612 grant, target4 respawn and fresh target4
selection precede one10334/slot6 request. Serials are2147483669/3674/3679/3684/
3691, forward deltas5/5/5/7. No extra skill request or10334 damage occurs.

The independent native observer records a qualified request, then correlated
13-byte typed78 reception, active owned order/start dispatch, and phase0.
Four-byte existing-order receptions subsequently select steps1 and2; each is
followed by start and phase dispatch with the same scoped correlation. Every
available wrapper snapshot matches base391, extra789, range[2,4), six steps
and21 events. No optional phase-field read is unavailable.

| Observed step | Selected event interval | Offset | Cursor at completion | Completed-step counter |
| --- | --- | ---: | ---: | ---: |
| 0 | [0,1) | 0 | 1 | 0 |
| 1 | [1,3) | 40 | 3 | 0 |
| 2 | [3,18) | 280 | 18 | 1 |

All three completion callbacks return0: none requests release. A later matching45
direct path clears slot0 while the tracked order remains identifiable immediately
before its release dispatch. The observer stops at `tracked_slot_cleared`,
record17, with20 callbacks and two rejected out-of-scope observations. Their
stages are receiver/packet and start/order; the retained rejection scalars do
not identify the rejected behavior kinds. Raw receiver register arguments are
preserved and are not treated as decoded outbound wire flags. Slot clearing
does not prove deallocation or successful effects.

Root reviews the hash-admitted before-key2 and7-second images: the second
weapon icon is colored; displayed health changes275→388, and the latter view
shows Wolf100%. The server independently logs ordinary regeneration from
236→274.8→313.59998→352.39996→388. This is not10334 healing. No tooltip/name,
exact animation/cancellation, recharge, strike count or evasion is established.
The injectedD1000 schedule and debugger observations cannot recover retail
server timing. Event cursors reaching their interval ends do not identify
individual event effects or authoritative combat outcomes.

Launch/UI0 and cleanup pass. Window53 returns at15:05:12.167973Z with61 immutable
non-prefix inputs, unchanged prepared224, both executable hashes, console
privacy and absent container verified. Full raw/normalized Evidence is retained
without truncation. Both diagnostics remain off live; no gameplay/default,
record, schema or publisher-baton change follows from this observation.

Private evidence in `~/re/gw2/notes/`:

- Launch `209026-run260-mesmer-sword2-phases-launch-20261106.json`, SHA256
  `b7057a7ec53716336b2086f80d23f44294a0baf9c49ce56bfff1ba9a7aa50836`,
  REA `ev_027f929fec287281f6e0ce2117fe57f5437f3d2bef3c7db6cc45b743ae2dd9dd`.
- Readback `209026-run260-mesmer-sword2-phases-readback-20261106.json`, SHA256
  `0d35ebe013819c29e2aab1317dbaf6f1971d535f374fd864e35ec417345dcc7c`,
  REA `ev_b7a35486b18494b2f463d98bc7ce5c5fb8cb11bbc3104f211d312ed3449507f9`.
- Return `209026-run260-mesmer-sword2-phases-return-20261106.json`, SHA256
  `c9d24be8f81f6170aa6c778ac577f5593b47a94c0aebcd7e377fceafc4efb186`,
  REA `ev_cd3d88ce1238cb1ee920f923eebc1118d56b1f8e2905c07e91539cf9aa4a56fe`.
- Root outcome `run260-mesmer-sword2-phases-outcome-20261106.json`, SHA256
  `33cbe48ae53669cad596ef8fddbfcab54bd2122ea4852592921471bbe5c526d9`.
