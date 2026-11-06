# Build209026: opt-in Mesmer sword progression

Commit **7cfe9c4 first shipped** at `/home/blizz/gw2-rust-7cfe9c4`, deployed by the
sole publisher on2026-11-06 at11:38:42Z, then rollback e624c8b. It is included in
current403ac37;7cfe9c4 is now its rollback. Root independently
verified the exact tested artifact, profile, supervisor, skills and service
health at11:43:05Z. See the
[release record](209026-saved-norn-live-activation.md#verified-sword-chain-release-7cfe9c4).
The implementation checks below preceded deployment; the separate run241 native
combo observation is recorded after them.

`--mission-mesmer-sword-chain` / `GW2_MESMER_SWORD_CHAIN=1` adds an
explicit server reconstruction to the existing immediate fixture-hit path.
It does not recover retail strike timing, hit/miss rules, damage arithmetic,
cooldowns, chain expiry, physical cancellation or automatic repetition.
The option defaults off and adds no storage or carry fields.

The [native selection contract](209026-mesmer-sword-chain.md) establishes
server605 availability and server628's predecessor/count pair. With ordinary
palette90/profession7/category3 predicates satisfied,10170/1 selects10171,
10171/1 selects10172, and10172/1 returns to10170. Count0 disables both
follow-ups while preserving the last predecessor. These are selection facts;
advancing on an applied fixture hit is the explicit reconstruction policy.

The supported profile requires profession7, item33355 equipped in slot29 with
active set4, explicit grants10170/10171/10172, and a positive finite configured
hit amount. The competing45230 grant and research autoattack, echo and NPC-cast
modes are rejected. No612 standard-slot override is invented for the weapon.
Saved characters keep their original records, appearance and biography items.
Profile validation runs before listeners; each selected world phase validates
again. Creation-scene connections do not receive this gameplay mode.

An encoded ordinary SkillBegin must name the owned character, the currently
expected sword skill, slot5, normal track, and a living configured hostile.
The player must be alive and the skill explicitly available. These checks run
before target tracking, pending-heal cancellation, damage or retaliation.
The ordered connection's wrapping serial guard rejects equality and backward
half-range deltas; this is a reconstruction replay policy, not a recovered
protocol acknowledgement. Rejected requests do not consume the serial.

Only a hit that actually reduces hostile health commits progression and appends
628. Target changes do not reset progression. Input-end29, input-start2a and
uncorrelated00 are not attacks or progression triggers. No repeat timer is added.

A separately admitted Ether Feast cast resets the chain; a rejected or cooling
heal does not. Loss of alive state resets through the existing timer output.
After the first hit, reset emits10170/count0; after the second,10171/count0.
It preserves the actual predecessor and the connection's replay guard. A fresh
join starts at the base skill, matching the newly constructed skill bar's
null-predecessor/count0 state. Chain progression is deliberately not carried
between connections or persisted across restart. Existing health, recharge,
XP, reward, inventory, GUID/map carry and map routing remain separate.

The candidate skills file is27 bytes: it preserves the existing17-byte fixture
and inserts only605 grants10171 and10172 before612. The separate candidate
profile changes only its skills filename and enables this option. It preserves
damage30, Ether Feast, the explicit empty legacy autoattack setting, saved mode,
world28, heart/NPC/POI/vendor inputs and all authored-source limitations.

Validation includes an executed fail-before: encoded10170 reduced hostile
health118→88, but no628 followed. Focused regressions cover three complete
chains, target changes, actual reset wire values, replay/wrap, invalid requests
leaving combat and a pending heal untouched, cooldown rejection, death, a fresh
saved-character join, option-off replies, equipment and CLI prerequisites.
The raw628 codec is copied byte-for-byte from main's existing source module;
its independent schema/golden/truncation tests are retained. Private logs and
artifact/profile pins reside in
`~/re/gw2/notes/mesmer-sword-chain-e624-20261106-checks/`.

No client/runtime, archive or owner store access was part of these implementation
checks. The subsequent deployment/readback is recorded above. Native combo
behavior remains unobserved by this patch's focused offline checks.

## Run241: native three-tap cycle

The prepared near-wolf diagnostic runs the exact published7cfe9c4 binary and
27-byte skills input, with pristine145 inputs and production GDB. Three separate
key1 taps produce these actual native requests and matching fixture hits:

| Request | Behavior serial | Logged enemy health | Post-hit native target display |
| --- | ---: | --- | ---: |
| 10170,slot5,target4 | 2147483669 | 118→88 | 75% |
| 10171,slot5,target4 | 2147483674 | 88→58 | 50% |
| 10172,slot5,target4 | 2147483679 | 58→28 | 24% |

All requests name controlled agent1, normal flags3. Serials advance by5 each;
no increment-by-one assumption is needed. Ordinary NPC19 retaliation continues.
The strict reader finds no extra request, target change, healing/reset, kill,
downed state or regeneration. The UI stops after three inputs; no fourth input
or automatic repetition is tested.

Root viewed hash-admitted `combo-before-1` and `combo-after-1/2/3` images.
The weapon slot changes from its base face/magenta icon to a burst image, then
a blade image, and returns to the original base image after the third hit.
This observes the native icon cycle separately from the typed request keys.
No tooltip names or exact attack-animation/strike timing are established by
these post-hit screenshots. No outbound628 decode was introduced. The requested
three-second observation margin is a diagnostic choice, not a retail expiry.

Launch/UI exit0. All60 immutable nonprefix inputs, the unchanged224-byte prepared
record, privacy and frozen/working PE hashes pass. The shared container is absent;
window43 returned at2026-11-06T11:51:19.087651Z. This observes one prepared cycle,
not saved-mode relog, native heal/death resets or the full tutorial.

Private evidence under `~/re/gw2/notes/`:

- Launch `ev_c425d9740ea0447d6034f6b175ca6d7a7df7148304f6b3bc86b1dc906b612330`,
  `209026-run241-mesmer-sword-chain-launch-20261106.json`, SHA-256
  `2be569d7efb35391c6cd59c97524bd4af7036c50fcb0a008d45cf3b651f3389c`.
- Strict readback `ev_dde05f8706276ef542b4d7c5f0065bccde52353afc76989aa34d59862a93041e`,
  `209026-run241-mesmer-sword-chain-readback-20261106.json`, SHA-256
  `ba0538282ca6937810f2cac2bd817d711fe7b3c94c7817815494dceb75a94250`.
- Return `ev_23682ce5303e4ce0fbd8b8562bac220ddfbf9f2927d80031f28b2e64035eff7a`,
  `209026-run241-mesmer-sword-chain-return-20261106.json`, SHA-256
  `f2e5fc629dd54203c8db3135a3e5771c63591b8b19fdac1e118b3141d60f1722`.
- Root outcome and all four image pins:
  `run241-mesmer-sword-chain-outcome-20261106.json`, SHA-256
  `af7e0faa7b45ea490d2c4608345b04bfd6f81a14d524f0378d589c834ec9227e`.

All captures retain complete output and verified owned cleanup. Runtime evidence
supports the bounded sequence; retail damage, target defense, hit timing,
expiry, repeat behavior and profession mechanics remain unresolved. The
[range consumer](209026-mesmer-sword-range-activation.md) supplies a conditional
client deferred-activation path, not a mandatory server request rejection.
