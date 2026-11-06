# Ether Feast must not trigger the diagnostic attack path

The live combat fixture treated every targeted `SkillBegin` as an attack.
With a hostile selected, skill10176 (Ether Feast) therefore lowered its
health and could award kill XP and loot. The client identifies this skill
as healing/condition removal: see the admitted run144 tooltip observation in
[the skill source note](209026-mesmer-heal-animation-data.md). Keeping an
enemy selected does not make that heal an offensive skill.

The regression reproduced the defect through an actual client23 wire layout:
target3, skill10176, healing slot0. With a lethal fixture damage amount, the
old handler reduced NPC health118 to0, awarded2500XP/level2 and a Broken Claw.
This is an owned server test, not a new client-runtime observation.

The correction excludes10176 from the generic damage fixture before its
implicit attack-target update. It emits no fabricated cast reply or health
increase, starts no retaliation, and awards no loot, XP or heart progress.
The standalone opt-in order-echo diagnostic remains available when the damage
fixture is disabled. Weapon-damage regression requests now use10170 instead
of incorrectly using the heal key. This does not establish generic offensive
skill classification or implement Ether Feast's actual effects/recharge.

The focused regression covers two selected hostile targets, unchanged health,
respawn state, XP/level, loot count, heart/reward state, attack target and
retaliation timer. The first correction exposed a second mutation before the
damage branch: it still changed the remembered attack target. Moving the guard
before that mutation addresses both effects.

Validation: all32 mission tests passed, including the new regression and
existing attack, kill/loot, counterattack, downed/defeat and heart/rejoin tests.
The ordinary observer binary built, and its no-listener `--check-config`
startup preflight passed with the prepared Norn profile and combat/heart/
respawn options. No new in-game check was run for this guard. Deployment is
through the existing sole publisher; this source note is not a live receipt.
