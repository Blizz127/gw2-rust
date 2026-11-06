# Mesmer starting experiences: completion ledger

Owner goal, 2026-11-05: finish the Mesmer starting area to 100%, retail
accurate, using the shipped client as the base source of truth. Latest
ordering: **Norn first**, then Human. This ledger preserves the full goal;
passing a codec or entering an empty map is not completion.

The pinned executable is build 209026 (`Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`).
Source-derived behaviors must retain REA evidence and distinguish client
observations, implementation choices and missing server behavior. Cached
API/wiki names may locate content but do not replace client verification.

## Required end state and evidence

| Requirement | Current state | Evidence needed for completion |
| --- | --- | --- |
| Norn Mesmer creation/selection and correct appearance | Private creation/save/world/waypoint flow passes in run144; restart relog passes in run127. Saved mode enabled live03:29:33Z and independently verified03:43:51Z. | Creation/rendering and activation evidence available; preserve biography/appearance/equipment through the remaining full gameplay experience. |
| Correct Mesmer components and startup skills/equipment | Partial: set4/item33355/palette90 are observed. Run216's605 grant passes availability and produces a real targeted10170 request with one30-point response and no release hit. Correction is live in4c46cad-sk and independently verified. Final-cache instrumentation is partial; full unlock/stat/effect policy remains open. | Client-backed defaults/selection rules, real skill bar and equipped items, correct unlocks and stats. |
| Norn opening tutorial (Great Hunt, key 379) | Reconstructed story actions/NPCs and a stand-in boss are live; Thora/trophy-kill progression is client-smoked. Publisher226 starts at stage4 and exercises fixed item/coin claim and map28 transfer. The full saved-character sequence, authentic conditions and retail XP/binding policy remain open. | Source-backed spawn, intro/dialogue, NPC encounters/objectives/boss, rewards and tutorial exit in client. |
| Wayfarer Foothills (key 28) terrain, movement and travel | Transfer/arrival is live in76af245. Prepared217 transfers379→28, renders the character and opens the map UI. Post-transfer movement/combat and combined saved-character runtime admission remain unobserved. | Correct entry/spawn, collision, movement, map transitions, waypoint use/death recovery in client. |
| Mesmer combat and profession mechanics | Diagnostic target damage, downed/defeat and waypoint rejoin work. Explicit no-clone Ether Feast is live; run233 observes cooldown/retry/recast and run240 observes positive native health recovery under ordinary enemy damage. Sword-chain increment 7cfe9c4 is included in current403ac37; run241 observes all three native requests, matching hits and the slot icon cycle. Retail effects and profession mechanics remain incomplete. | Relevant weapons, attacks, healing, clones/phantasms/shatters, cooldowns, conditions/boons, target/death/downed behavior verified in encounters. |
| Starting-area NPCs, enemies, vendors and interactions | NPC/dialogue/combat/loot foundations and ambient ravens are available; authored Great Hunt population and placements remain open. | Client-backed population/identity/placement plus exercised dialogue, shops, combat, loot and respawns. |
| All starting-map hearts, events and discovery objectives | All15 map28 heart areas and14 substitute NPCs are in76af245. Generic inside-area kill progress and placeholder rewards remain fixture rules. Authored tasks/events and full discovery are incomplete. | Source-backed inventory of objectives, implemented outcomes/rewards, completion counts and client playback for every objective. |
| XP, leveling, skills and equipment progression within area | Explicit fixture XP/level, loot, armor, biography mask and dyes are exercised; retail reward/unlock rules remain incomplete. | Correct rewards/stat progression/unlocks and inventory changes across the starting experience. |
| Relevant starting story, scenes and audio | Playback missing; client cinematic start/stop contract recovered, Norn selection unresolved | Correct sequence, dialogue/choices/cinematics/audio and branch completion observed in client. |
| Persistence and interruption recovery | Initial character records are durable and restart relog passes; world XP/loot/heart/rejoin state and the new guarded Ether Feast health/cooldown carry are process-local only. The existing store has no progression fields or update API; restart progression requires a separately authorized storage-format change. | Save/relog and disconnect/restart preserve character, items, position and progress without duplicate rewards; preserve original creation data and rollback compatibility. |
| Human Mesmer opening and Queensdale, after Norn | Incomplete | Equivalent creation/tutorial (138)/area (15) coverage, including source-backed identification of the remembered circus/town sequence. |
| Final completion audit | Not reached | Every required item above proven with current source and runtime evidence; no placeholder counted as retail accurate. |

## Current checkpoint

**403ac37 is verified live**, deployed at11:54:07Z to
`/home/blizz/gw2-rust-403ac37`, rollback7cfe9c4. Root's independent11:56:13Z
readback verifies the exact reviewed binary, all current/rollback hashes, four
owned healthy listeners, saved mode/one private record and unchanged backup.
This adds optional movement-log cadence1..1000 for short route observations;
the live profile, supervisor and data are unchanged and default20 remains active.
Four focused tests,15 binary CLI/startup checks and the musl build pass. It changes
no gameplay or client hooks. The native sword/heal observations below remain
their exact earlier artifacts; no403 native traversal is claimed yet.
[Release readback](re/209026-saved-norn-live-activation.md#verified-movement-diagnostic-release-403ac37).

Run240 confirms positive native health recovery on unchanged e624c8b in the
prepared near-wolf fixture. Root reviewed the hash-admitted images: health274
before healing,255 at the first post-input capture,338 at the follow-up with
cooldown17. Ordinary NPC damage continues; the server log independently implies
one121-point gain after one10176 request. Launch/UI0, cleanup, privacy, PE and
all60 immutable input checks pass; window42 returned11:35:27Z. This is not a
saved-character/full-hunt, exact effect-timing or retail-mechanics test.
[Evidence and limits](re/209026-ether-feast-reconstruction.md#run240-positive-native-health-recovery).

Sword-chain increment **7cfe9c4**, now the rollback, was deployed at11:38:42Z to
`/home/blizz/gw2-rust-7cfe9c4`; its rollback was e624c8b. Root's independent11:43:05Z
readback verifies exact binary/profile/supervisor/27-byte skills, current and
rollback hashes, four healthy owned listeners, saved mode/one private record
and unchanged backup. No newer live work was overwritten; the publisher's
later story branch remains separate. Its opt-in sequence advances
10170→10171→10172 after accepted hits, with actual-predecessor/count0 resets on
an admitted heal or loss of alive state. All29 focused checks, the musl build
and two saved-profile startup checks pass. Hit-driven progression/reset rules
remain explicit reconstruction choices; retail strike timing, damage and expiry
are unverified. Separate run241 now passes one native three-tap cycle:
10170→10171→10172, matching enemy health118→88→58→28, and the slot icon returns
to its original image after the third hit. No extra requests or resets occur.
Launch/UI0 and cleanup/privacy/PE/prepared-record checks pass; window43 returned
11:51:19Z. It is a prepared fixture smoke, not a saved-character/full-hunt test.
[Release readback](re/209026-saved-norn-live-activation.md#verified-sword-chain-release-7cfe9c4);
[implementation and limits](re/209026-mesmer-sword-chain-reconstruction.md).
[Native combo evidence](re/209026-mesmer-sword-chain-reconstruction.md#run241-native-three-tap-cycle).
Private implementation receipt:
`~/re/gw2/notes/mesmer-sword-chain-e624-20261106-checks/receipt.json`, SHA-256
`1c709e3dfb679731737306ca432751808910296c1fc5300e101a441180dfe012`.

Earlier **e624c8b**, now the rollback, was deployed by the sole publisher at10:35:43Z to
`/home/blizz/gw2-rust-e624c8b`. Rollback:7c26f45. Root's10:41:12.080308Z
readback verifies the exact tested artifact, all kit/rollback hashes, four
owned healthy listeners, saved mode/one private record and unchanged backup.
It fixes inconsistent admission of unavailable skills before retarget/damage;
the failure-before regression,17 passing focused tests, static build and two
saved-profile startup checks are retained. It changes no grants, chain policy,
profile, store or schema. The full retail goal remains incomplete.
[Verified release](re/209026-saved-norn-live-activation.md#verified-skill-admission-release-e624c8b);
[admission fix](re/209026-skill-caster-ownership.md#explicit-skill-availability-e624c8b-verified-live).

Earlier **7c26f45**, including5a6337b, was deployed by the sole publisher
at09:24:16Z. Kit: `/home/blizz/gw2-rust-7c26f45`; its rollback was a223a90.
Root's independent 09:27:43.186242Z readback verifies the exact artifact, current
and rollback hashes, effective profile, four owned listeners, saved mode/one
record/private 0700 store and unchanged activation backup. Zero humans/bots were
present at that check. Ether Feast is enabled; `story_branch` remains `none`.
It adds an explicit no-clone self-heal and 20-second recharge, with guarded
same-process health/cooldown carry. The original-roster 379 versus pending-map 28
login rejection was corrected; the actual TCP regression failed before and now
follows 1065 to restored state. Focused tests, musl build and eight off/on startup
checks pass. The separate retained run233 audit observes cooldown19, a rejected
retry with “Skill recharging”/16, and a later fresh19. Regeneration had already
restored350→388 before the first cast, so that earlier run did not observe
positive healing; run240 above closes that separate bounded check.
Manual cancellation, clones/conditions and restart progression remain unsupported;
the full retail goal is incomplete.
[Verified release](re/209026-saved-norn-live-activation.md#verified-ether-feast-release-7c26f45);
[implementation and limits](re/209026-ether-feast-reconstruction.md).

Run237 follows the hall's right exterior to a narrow strip, but still reaches
no hostile. The final camera/foliage obstruction and unchanged sparse movement
sample do not identify a specific collision cause. Only step1 completes;
launch/UI13 records the failed full-hunt predicate. Controller release meets
the target; strict cleanup, privacy, PE and original-record checks pass.
Window40 returned10:37:33Z. A route overview remains missing.
[Result and limits](re/209026-saved-norn-creator.md#run237-exterior-route-reaches-a-narrow-strip-hunt-still-fails).

Run238 now observes native M map opening, one wider zoom and return to the world
on exact e624c8b. Root reviewed the hash-bound images: player marker and broad
terrain are visible, but no readable hall/road connection establishes a route.
The neighboring “Hoelbrak (Lodge Level)” label is not a current-map identity
claim. No gameplay events occur; launch/UI0, privacy, cleanup and unchanged
PE/synthetic-record checks pass. Window41 returned11:07:01Z.
[Map result](re/209026-saved-norn-creator.md#run238-native-map-overview-observed-route-still-unresolved).
The [static hostile surface comparison](re/209026-norn-terrain-v14.md#existing-hunt-placements-measured-surfaces-grounding-unresolved)
does not establish model-foot placement or automatic grounding; no actor
coordinate change is justified by those measurements alone.

Run236 observes E/Q turning on exact7c26f45, but the normal-stage0 route enters
the camp hall. Only Thora/step1 completes; no hostile selection, attack, kill,
reward or transfer occurs. UI/launch exit13 records the failed full-hunt
predicate. Release meets the210-second target; cleanup, privacy, executable
hashes and original synthetic record pass. Window39 returned10:16:44Z.
The next route must go around the hall, with less controller planning overhead;
no conclusion about globally unreachable hostiles follows from this attempt.
[Result and limits](re/209026-saved-norn-creator.md#run236-turning-observed-full-hunt-route-fails).

Run235 reaches native Control Options: W/S forward/back, A/D strafe, Q/E turn;
weapon1 and healing6 are also displayed. Root independently viewed the admitted
image. No settings or gameplay input changed. Controller release was within56
seconds; cleanup, privacy, executable hashes and original synthetic record pass.
Window38 returned09:52:33Z.
[Bindings and scope](re/209026-saved-norn-creator.md#run235-displayed-movement-and-turn-bindings-established).

Earlier run234 reaches General Options and its displayed F11 shortcut, but does not
reach Controls or read movement bindings. The controller stayed within its90-second
hard limit but missed the80-second target. Cleanup, privacy, executable hashes and
the original synthetic record pass; window37 returned09:40:45Z. The next probe
uses the observed menu route with fewer round trips and an earlier navigation cutoff.
[Bounded result](re/209026-saved-norn-creator.md#run234-general-options-reached-controls-still-unobserved).

Run230 completes an isolated input diagnostic on exacta223a90. Paired
before/after D500ms images are consistent with lateral movement; exact yaw,
binding and total displacement remain unmeasured. Tab produces no target227,
so no attack is sent. Story remains at step1. UI/capture0 means the bounded
diagnostic completed, not the full hunt. Cleanup, privacy, unchanged executable
hashes and the original synthetic record pass; window36 returned09:00:04Z.
The next route must establish a supported turn/camera input before moving.
[Evidence](re/209026-saved-norn-creator.md#run230-isolated-input-observation-no-hostile-attack).

Run229 corrects the earlier controller failure and exercises native movement,
but stops with only Thora/step1 completed. The only selected/attacked target
is neutral raven45; Tab selects no new target. Navigation reaches the hillside
and does not acquire a hostile, so this is no full hunt or combat-code pass.
Cleanup passes and window35 was returned08:35:16Z.
[Retained attempts](re/209026-saved-norn-creator.md#run229-native-control-and-targeting-boundary).

Earlier **a223a90**, deployed08:41:57Z through the sole publisher, is now the
verified rollback. It restores carried XP/earned stats before quest-completion replay initializes the
reward panel, fixing a constructor-XP0 sampling defect. A failure-before
regression,21 distinct focused tests, musl build and four startup checks
pass. Independent08:43:03Z readback verifies the exact reviewed artifact,
all current/rollback hashes, four owned listeners, saved mode and backup.
It adds no XP payout policy or schema.
[Fix and source binding](re/209026-quest154-content.md#reward-initialization-timing-and-carried-xp-ordering-fix).

Earlier kit: `/home/blizz/gw2-rust-a223a90`. Its independent readback verified all then-current and
rollback hashes, effective settings, saved mode/one record, unchanged backup
and four owned listeners at08:43:03Z; zero humans/bots. The unchanged profile enables
the tutorial merchant; its appearance and placement remain fixture choices.
It retains the checked
quest154 item32138/72-copper claim, exact payout carry and transition timer
fixes, plus a configured20 hostiles/12 ambient actors in Wayfarer. It retains
the interleaved-character transfer fix, wallet and map28 POI discovery;
run224 observes waypoint371 discovery.
Waypoint travel is offline-tested but not exercised by a client click.
The earlier earned attribute increment remains active.
Its rollback was9326c08-vendor. Prepared publisher226 shows a reward claim, weapon icon
and72 copper, followed by logged map28 restoration. The full saved-character
flow, retail item binding/XP policy, effective-level/health scaling and durable progression remain
incomplete; that earlier artifact had no healing implementation. [Release evidence](re/209026-saved-norn-live-activation.md#verified-carried-xp-order-release-a223a90).

Run226 accepts the owned Ether Feast10176 request, typed0x78 start/completion,
then observes the native slot cleared through direct45 handling. The snapshot
precedes release dispatch; deallocation is not observed. Echo1000ms and
finish2000ms are diagnostic, with no healing/recharge claim. UI, cleanup and
executable checks passed; window33 was returned07:49:15Z.
[Bounded result](re/209026-skill10176-lifecycle-observer.md#run226-direct-finish-clears-the-native-slot).

The independent earned-attribute increment **e78b028** on76af245 passed its
isolated build,15 focused Rust tests,3 supervisor checks and4 no-listener
preflights. Its concrete artifact was submitted to the sole publisher and
subsequently released in0763c47 above.
Only `GW2_BASE_ATTRIBUTES=default-curve` is added to the latest profile;
effective-level/health scaling remains incomplete.
[Evidence](re/209026-earned-attribute-progression.md#independent-port-onto-live-world-handover-source).

Earlier **76af245**, deployed 2026-11-06T06:21:31Z at
`/home/blizz/gw2-rust-76af245`. Root independently verified hashes, effective
settings, saved mode, health and rollback at06:25:42Z. It includes the
Wayfarer transfer, GUID/destination-bound carry and world arrival fixes,
plus 14 heart NPCs. Prepared runs establish transfer and spawn; saved-mode
combined transfer remains unobserved. Heart tasks/rewards and several NPC
details remain fixtures. [Release and scope evidence](re/209026-saved-norn-live-activation.md#world-handover-release-76af245).

Run216 passes the targeted sword smoke on exactcf13a3b: fresh hostile4
selection, one owned10170/slot5 request to4, one server255 response118→88,
and one release with no extra order or hit. The17-byte grant enables native
availability; legacy autoattack and echo are absent. UI/cleanup/hash checks
pass and window29 was returned06:10:56Z. The small skills/profile correction
was submitted immediately and deployed by the sole publisher in4c46cad-sk
at06:16:37Z; root verified live hashes, effective settings and health06:19:28Z.
The nearby wolf is only a diagnostic placement, excluded from that handoff.
Final-cache instrumentation remains partial, and retail lifecycle, damage,
chain, healing and cooldown behavior are not established by this smoke.
Earlier214/215 failures remain retained. See the
[runtime evidence](re/209026-sword-slot-selection-runtime.md#run216-targeted-ordinary-request-and-one-damage-response-pass).

Released independent fix: **9451cf0**, the seven-line caster guard
directly on then-live c2cb6fd. Eight focused tests, offline musl build and four
no-listener checks passed at04:03:38Z; current combat settings and Ether Feast
exclusion are preserved. The corrected artifact handoff was submitted to
the existing sole publisher's native inbox at04:33:23Z under taskf45083e2793a.
The publisher integrated it as e788d58 and deployed it in4b3c4c2 at04:46:24Z;
root independently verified live hashes/health at04:53:03Z. See the
[receipt](re/209026-skill-caster-ownership.md#independent-fix-released-through-the-sole-publisher).

Progression storage remains under the owner's no-schema-change restriction.
The private proposal `~/re/gw2/notes/saved-character-progression-proposal-v1-20261106.json`
keeps original GW2CHR01 records intact and specifies a separate transactional
progression database. Root asked the owner directly through asynchronous
input whether to authorize that exception; the question is pending, and no
approval, schema change or reset is assumed. Activation must preserve
existing progress. Do not repeat the question; consult the root-owned
shared persistence-boundary record for any later answer.

The separate cumulative candidate **45c0731** contains the
[caster ownership guard](re/209026-skill-caster-ownership.md). Nine focused
tests passed, including two regressions that failed before the fix. Its
isolated static release binary passed four no-listener configurations at
**2026-11-06T03:45:50Z**. It includes5516cb4's waypoint identity correction,
earned-attribute curve and earlier skill-input corrections. It adds no
profile setting or schema and remains **withheld, not deployed**. Its
coordinator handoff task237f1f15ce02 was refused because taskf45083e2793a was
already active and was not delivered. The native submission above corrects
the release candidate to9451cf0 through the existing publisher.
The publisher's run203 on the earlier5516cb4 candidate reached the world and
cleaned up, but its03:52 readback contains no sword SkillBegin or combat
response. A combat pass is not established for either cumulative candidate;
the later source of the missing request is narrowed below. See the
[readback](re/209026-saved-norn-live-activation.md#subsequent-combat-smoke-readback).
Ether Feast is excluded from generic attack damage; healing/recharge remain
unimplemented and must not be advertised as part of this candidate.

New static findings bind the [cinematic start/stop contract](re/209026-cinematic-start-contract.md)
and [server-delivered shrine locations](re/209026-norn-shrine-location-delivery.md).
They identify concrete missing inputs: the Norn cinematic selection/trigger
and authentic locations/auxiliary state for tutorial POIs 970/971. The
[result-listener check](re/209026-typed78-result-listeners.md) supplies no
heal/cooldown trigger. The [endurance follow-up](re/209026-endurance-deficit.md#bounded-exact-caller-follow-up-2026-11-06)
also corrects a fixture comment: maximum 100 / divisor 2 produces quotient 50,
not two segments; no values changed. These are source/documentation changes,
not a new playable artifact or a reason to rebuild 5516cb4.

The [creation-cinema view bridge](re/209026-creation-cinematic-view-bridge.md)
now binds playback to the conditional creation loading/playback views and
their name-based join continuation. The
[terminal UI submit](re/209026-creation-submit-cinematic-boundary.md) reaches
client415 and its known submission-control event without directly choosing
a cinematic. This narrows the remaining intro dependency to authentic
Norn playback inputs/sequence; the generic UI/view wiring does not select
them. This source slice emits no Norn cinematic and changes no gameplay.

Earlier independently verified live kit: **4c46cad-sk**, deployed06:16:37Z,
now retained as rollback for76af245 above.
It includes ordered Great Hunt story actions, talkable Thora/Ido/Eir, a stand-in
wurm boss and placeholder out-of-combat regeneration. The publisher exercised
Thora, a trophy kill and regeneration in the client; later story steps have
unit coverage but no completed client smoke. Placement, dialogue and some
appearances are reconstructed. Completion-panel display does not establish
reward delivery or authentic action conditions. The latest increment carries
hunt stage through same-process waypoint rejoins, with focused unit coverage;
restart persistence remains missing. The new skills/profile increment
enables real10170 requests and removes the legacy release-triggered attack;
damage30 remains diagnostic. Ether Feast damage
exclusion and diagnostic attack behavior remain enabled; healing is missing.
Saved-norn mode has been on since03:29:33Z, following the owner's03:29:03
native choice. At06:19:28Z root independently verified
mode/store, one private saved record, activation backup hash, all kit hashes,
metadata900, four owned listeners, zero humans/bots, effective skill settings
and retained rollback. Earlier active-player observations are historical.
[Activation/readback evidence](re/209026-saved-norn-live-activation.md).
The prepared Banshee Mesmer is hidden. New creation records persist; XP,
loot and heart progress still reset at a server restart.

[Run142](re/209026-texture-remapall-private-candidate.md) established the
complete male Norn Mesmer preview; [run144](re/209026-saved-norn-creator.md)
then passed the creator/save/world/defeat/waypoint flow with runner0e6ac30
and metadata900. The later mode change uses that prior acceptance and does
not establish full retail encounter behavior. The03:43:52Z feed readback
now offers b128; that query did not inspect its runner pin. Earlier b126
and unanswered-roster observations are historical, not current blockers.

The same publisher session was alive/status shell at03:41:35Z and reported
preparing5516cb4's combat smoke. Root returned unused window23 at03:42:49Z
for publisher priority run203. Window24 was subsequently granted03:56:36Z
for one bounded creator run and cleanup. Run145 remains unlaunched and its
inputs unchanged. [Run148](re/209026-sword-slot-selection-runtime.md) completed
four controlled slot5 activations, each with a null selected SkillDef at
wrapper/body entry and return, no ordinary request call and no observer
errors. The heal control emitted10176. This is a successful bounded diagnostic,
not successful sword combat. Correct slot/palette startup remains unresolved.
Window24 was returned complete04:25:32Z; no further runtime grant exists.
Root's bounded client observations are recorded above. No competing
deployment, native prompt answer or live service restart occurred.
The full Norn-first goal remains open.

An earlier offline increment adds a bounded
[named ability-value resolver and sword damage-fact evidence](re/209026-mesmer-sword-damage-facts.md).
Nine focused tests pass; the compiled reader matches all 16 named cases and
eight literal cases against independently parsed frozen records. It provides
content lookup, not damage execution. The exact tooltip multiplier is 1.0;
the inspected starter-item branch gives strength 105–117. Incoming display-slot
selection and alternate player state remain explicit qualifications.
The [0x29/0x2a producer audit](re/209026-skill-input-29-2a.md) and
[finish-listener audit](re/209026-typed78-finish-listeners.md) supply no retail
repeat/chain timing. A single frozen-source extraction resolves story9's
starting quest358, while story2→9→10 reference semantics remain unknown.
No server or client deployment, run145 launch, or publisher takeover occurred.

The follow-up [type-13 damage-fact reader/evaluator](re/209026-mesmer-sword-damage-facts.md#type-13-reader-and-numeric-evaluator)
passes seven focused tests and an independent frozen-content comparison of
16 fact/mode cases, 48 numerical cases and three wrong-type rejections.
It connects named coefficients to the exact tooltip calculation while keeping
weapon, attribute and level inputs explicit. Combat damage, hit timing and
the final attack-chain policy remain outside that implemented scope.

The coordinator's00:42 UTC waiting/input-needed observation and root's
03:18 native-prompt capture were superseded by the owner's03:29 choice and
the03:43 independent saved-mode readback. Delivered server fix handoffs
remain distinct from deployment receipts. Preserve the sole publisher and
active-player/backup/rollback checks.

The tested server guard `5169b98` excludes Ether Feast10176 from generic
attack damage and implicit attack-target changes. All32 mission tests, build
and no-listener startup preflight passed; it is handed to the publisher,
without a newer live receipt yet. See the
[damage correction](re/209026-ether-feast-damage-guard.md).
The [literal amount reader](re/209026-mesmer-tooltip-facts.md) passes eight
focused tests and actual frozen-record checks. The
[character-backed context](re/209026-tooltip-character-context.md) grounds
its explicit fixture level/attribute inputs, while
[recharge consumers](re/209026-skill-recharge-consumers.md) distinguish
ordinary cooldowns from count replenishment. These are source foundations;
the cast transition and actual healing/recharge execution remain unfinished.

The [activation-tooltip trace](re/209026-mesmer-cast-tooltip.md) binds the
primary execute event1067 to the displayed1 through quarter-second formatting;
it does not supply an authoritative server cast duration. The
[seven-site lifecycle observer](re/209026-skill10176-lifecycle-observer.md)
and [recharge/context observer](re/209026-skill-recharge-consumers.md#prepared-passive-observation-contract)
pass28 and16 host mock tests respectively. Both are staged in the private
run145 diagnostic using the offered runner0e6ac30 and live900 metadata.
No run145 client has launched: a fresh shared-resource window from the
existing integrator remains required. The
[echo-precondition trace and fix](re/209026-skill-order-echo-preconditions.md)
replace the unlaunched Word0 experiment with commitcac0e9a's explicit
nonzero-duration option and controlled-order admission flag. Eight focused
tests and debug/static builds pass. Run145 now stages that static binary with
analyst-selected1000ms; no positive duration is claimed as10176's retail cast
time. Its1831-entry manifest is pinned, auth/mission no-listener preflights
pass, and both archive diagnostic readers are private pinned copies.
The [map gate](re/209026-skill-context-map-gate.md) independently resolves
context1 once map379 is installed, selecting the base recharge content20000
before display modifiers. Actual recharge execution remains unimplemented.

The [recharge content reader](re/209026-skill-recharge-content.md) now resolves
the actual ability and ordered context overrides, with four focused tests,
a library build and an independent frozen-record check. The exact10176
record has base20000, context2/3 overrides25000, and zero count maximum with
no count overrides. This supplies content selection, not a server cooldown
schedule or a default category-zero Word.

The [animation-sequence reader](re/209026-animation-sequence-reader.md)
passes three focused tests and an independent cached-pack comparison of
both exact10176 token/variant pairs. It retains ordered raw events and
segment values with explicit limits and checked signed pointers. The
[event-consumer trace](re/209026-skill10176-animation-event-consumers.md)
establishes visual effects and view-state changes in the selected ordinary
listeners; it supplies no authoritative healing/success time. The
[Mesmer resource consumers](re/209026-mesmer-resource-consumers.md) establish
displayed capacity/current count, while live clone identity, maximum and
initial values remain unresolved. These are offline implementation/source
increments. No client run, publisher dispatch or deployment was performed.

The [cancellation trace](re/209026-skill-cancel-request-correlation.md)
proves the queued-replacement request uses a fresh behavior value, while
aborting an existing client order requires its original behavior, track
and kind. It does not yet identify the manual cancel-input path or a
server interrupt response/policy. Existing raw client23 framing suffices
to preserve the request; no speculative cancellation codec was added.

The earlier [repeat-attack response artifact](re/209026-repeat-attack-admission.md)
c862325 is superseded: the [complete input trace](re/209026-skill-input-action-lifecycle.md)
proves that0x2a can precede a separate admitted SkillBegin and0x29 comes from
deactivation/end paths. Commit **00599af** removes the extra cast/hit from
slot inputs and retires the incorrect autoattack fixture. Older supervisor
profiles remain bootable with an explicit retirement notice. Three new
regressions and affected mission/shared-configuration checks pass. The
[replacement static binary and paired supervisor](re/209026-skill-slot-release-correction.md#replacement-artifact-not-deployed)
passed no-listener auth/mission preflights and retirement checks at02:21:24 UTC.
It is included in45c0731 under the sole publisher/resource constraints,
not deployed. The native roster prompt is resolved. Run145 remains unchanged
and unlaunched. Actual accepted-order animation/lifecycle and
automatic repetition remain unfinished.

The [sword-chain trace/codec](re/209026-mesmer-sword-chain.md), commitf382933,
identifies server628's predecessor/count pair:10170 with count≥1 permits
10171;10171 with count≥1 permits10172, subject to the other palette checks.
Three raw-codec tests pass. Later7cfe9c4 integrates explicit hit-driven progression
and reset policy, with run241 native evidence above; retail timing remains
unfinished. The [story-start trace](re/209026-quest154-authored-start.md)
proves story2→starting quest154→goal1→map379; the story9 link, encounter
actors/placement and step triggers are still unresolved. A bounded manual
cancel-input follow-up reaches an indirect-call boundary; its documented
next observation needs the same currently unavailable runtime window. The
[sword action-data follow-up](re/209026-mesmer-sword-animation-data.md)
proves the primary variant0 fields and checks68 exact variants; selected
event quantities are not a recovered server schedule. The
[step identity follow-up](re/209026-quest154-step-identity-consumers.md)
provides the six authored16-byte keys and a typed membership query, leaving
the actual actor/action-to-step rule unresolved.

The [duration-expiry trace](re/209026-skill-order-duration-expiry.md) closes
the client-local path from a valid ordinary command's deadline to its
character completion callback. A separate server84 is not required on that
conditional path. Typed-order release, healing and recharge still need
runtime evidence; this adds no server effect or retail timing default.

The [typed-order release trace](re/209026-typed78-completion-release.md)
now distinguishes that callback from releasing0x78: its return depends on
flag0x2000, and correlated0x45 provides the demonstrated deferred/direct
release path. Raw0x44/minimal0x45/0x47 payload codecs pass four focused tests
using the existing server84 carrier; no update is sent or scheduled.
The [positive-health delivery trace](re/209026-health-gain-delivery.md)
establishes sourced255 meter/clamp and UI-notification behavior and corrects
the wire-kind mapping. Heal amounts, cancellation timing, clone/condition
effects and retail packet choices remain separate implementation work.

## Earlier source checkpoints

- Historical baseline: Guardian entered Queensdale and walked (run110,
  then-live kit f36bb98), before the later NPC/combat/progression fixtures.
- [Mesmer state](re/209026-mesmer-profession.md): raw messages and explicit
  candidate population path implemented and checked against client schemas.
  No retail startup values established. [Runs 111/112](re/209026-norn-mesmer-playback.md)
  rendered the Norn Mesmer in map 379 and demonstrated local movement; those
  early fixtures had empty skills and equipment.
- [Starter maps](re/209026-human-norn-starter-maps.md): Norn tutorial379/open28
  share archive187611, with game types9/3. Human tutorial138/open15 share
  archive192711, with game types9/3. Auth already preserves requested types.
- [Equipment reader](re/209026-creation-equipment.md): typed traversal and
  real-archive checks identify the shared Human/Norn level-1 Mesmer template
  across both sexes. Runtime template selection/overrides remain unresolved.
- [Skill candidates](re/209026-mesmer-palette-candidates.md): all-container
  Rust census and independent Python traversal agree on three Mesmer
  category2 palettes; default selection and skill execution remain open.
- [Norn terrain](re/209026-norn-terrain-v14.md): version-14 decoding follows
  the client upgrade callback; real Norn and Queensdale probes pass.
- [Fixture CLIs](re/209026-character-fixture-cli.md): explicit archived
  appearance extraction and bounded synthetic auth framing replace manual bytes.
- [Quest initialization](re/209026-quest-lifecycle.md): direct quest154/goal1
  link to map379, explicit 527/532/528 observer experiment and once-only guard.
  Run113 shows a story marker but blank objective text; no encounter implemented.
- [Text-key trace](re/209026-quest154-content.md): all eight quest text IDs
  require keys; the inspected no-key decoder path returns an empty string.
  Explicit747/748 codecs are ready, but valid keys remain unresolved.
- [Equipment delivery](re/209026-item-equipment-delivery.md): explicit
  armor/weapon creation, equip and set-selection frames pass source-schema
  and fixture checks. Run115 renders the starter armor after moving the batch
  before character finalization; the weapon and first weapon icon are visible.
  Its placement is inside the [first objective sector](re/209026-norn-sector-polygons.md),
  not a claimed retail spawn.
- [Health baseline](re/209026-mesmer-health-speed-baseline.md): the default
  level1 Mesmer calculation yields388; run118 displays it through an explicit
  full-health option. Starter prefix112 contributes no attributes. Other
  modifiers and movement speed remain unverified.
  [NPC foundation](re/209026-npc-character-foundation.md)
  establishes species-based character creation; actual encounter content is open.
- [Initial skills](re/209026-mesmer-starter-skill-requirements.md): explicit
  605/612 observer input adds a healing icon in run116. Selection and
  presentation pass; cast execution, effects and cooldowns remain missing.
These earlier source checkpoints preceded the later private creator tests and
prepared-mode server releases above. Synthetic fixtures remain explicitly
labeled; releases retain active-player protection, rollback and the existing
sole publisher. No owner-device action is implied by a local playback.
