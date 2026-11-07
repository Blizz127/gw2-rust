# Ether Feast admission on configured Norn story maps

## Verified cumulative resume release

The current release is **10803b4-efresume** at
`/home/blizz/gw2-rust-10803b4-efresume`, deployed by the sole publisher at
the reported time2026-11-07T00:19:11Z. Root independently verified it at
00:19:43 UTC. The reviewed2002824-byte game binary is SHA256
`ca15f7b729a403c42438f87d31220f0847cb0c9b13e13c1f61ca0c7d00926af8`;
the publisher's independent rebuild is identical. All current and rollback
hashes pass, four owned listeners are healthy, humans/bots are zero, saved
mode remains on, and126 story arguments and CDN950 are unchanged.
Rollback is `/home/blizz/gw2-rust-168e7e4-efmaps`. Only BUILD and the game
binary differ; the h379 tutorial-heart correction is preserved.

Private receipt `10803b4-efresume-release-verification-20261107.json`, SHA256
`d8663521a5f5849dca5c75a733462e4250efac5b8d120b3e1944e38c8bd2ff7e`,
binds the untruncated REA readback
`ev_060e1bac26ba8f54df874747f3070cccfba494007021732983b74bd5fd0e9ae0`.
Full capture SHA256 is
`ba5e0e13146264412273e16365125e1ce3654675f9be49c7f762cc53a12e27ba`.
Owner record contents were not read; the activation backup is unchanged.

## Verified admission release

The sole publisher reports **168e7e4-efmaps** deployed at
2026-11-07T00:15:26Z to `/home/blizz/gw2-rust-168e7e4-efmaps`.
Root independently verified it at00:17:10 UTC: exact reviewed game binary
SHA256 `c6754868e8045b3c76841f5b2f6ec9bf9c30686d73ab89b04f6b8027f54af19a`,
all current/rollback hashes, four owned healthy listeners, zero humans/bots,
saved mode on,126 story arguments and matching CDN950 readback. The h379
tutorial-heart profile is unchanged. Only BUILD and the game binary differ
from rollback `/home/blizz/gw2-rust-cb8622a-q171-h379`.

Private receipt `168e7e4-efmaps-release-verification-20261107.json`, SHA256
`85411cfc19c329a496db968a9835af8217115af69602445694480f600169980d`,
binds full untruncated REA
`ev_85f9dc562f9fe707ac52caaf388de1fe2166bbcf904c357dc120049a047f2b49`.
This earlier admission release did not include the resume correction;
the subsequent10803b4 release above includes both changes.

The cb8622a-q171 profile enables Ether Feast and configures maps393,401,396,
405,400 and407. Both prepared and saved-character joins rebuild their mission
phase for the requested story map, then apply `with_ether_feast`. Its older
379/28/392/792 restriction rejected all six destinations before the key reply.
Checking only the initial379 configuration did not exercise that branch.

The correction shares one explicit supported-map predicate between phase and
CLI validation and adds those six maps. It preserves profession, available
primary10176, rejoin-route and incompatible-mode checks. It changes neither
healing amounts/timing nor character ownership, carry or routing. A supported
skill map is not permission to enter an unconfigured destination.

## Client evidence

A bounded cache-only REA capture on 2026-11-07 inspected the six retained
type45 map records. Each has signed type4 at+0x2c, flags0x100 at+0x9c
(bit13 clear), and no classified+0x100 reference. The previously retained
`0x141411600` gate therefore leaves the character+0x220 sentinel4;
`0x14140f1d0` maps that sentinel to recharge context1. This selects the same
10176 base20000 value used for392/792. This is a client-data and code
derivation, not a newly observed retail-server cooldown.

Private complete evidence:

- `209026-ether-feast-six-story-map-contexts-20261107.json`, SHA256
  `42f30ea6e0ca8f25e40801b8a6cd5d2532f4e0ac974fe07e4f417dbdb113e4a8`,
  REA `ev_b5e27ce3531c39cac672e82d505a2b89f724642d17e50758411ef300937534ca`.
- `ether-feast-story-destination-join-findings-20261107.json`, SHA256
  `fd6de45ed631f2ef7fa88995e042987b923523f39a776205b44d122ce5eb5694`,
  pins the exact source/profile, map rows and retained gate/fallback evidence.

No archive fallback, client launch or live-store access was used for this
source check. Original skill policy and limits remain documented in
[the reconstruction record](209026-ether-feast-reconstruction.md).

## Focused verification and limits

The new phase regression failed on all six destinations before the change.
Focused healing tests then passed except for the new connection test's
initial expectation: it omitted the populated-map flag. Correcting that test
fixture produced twelve successful prepared/saved destination key exchanges
and matching map openings. The actual `observe` path is exercised using
loopback sockets and a private synthetic temporary character store; no client
gameplay is implied. Existing unsupported-map, missing-route, research-echo,
healing and cooldown/ownership coverage is retained.

Logs and build/startup receipts are private under
`~/re/gw2/notes/ether-feast-story-maps-20261107-checks/`.

The separate resume correction in source commit
`10803b40d7fd6268f6178e23e748250be4bb358a` now registers the exact configured
world/story destinations and rejoin route in common connection setup.
Original-roster map379 reconnect can select that registered route for its
matching pending story carry. `next_map` remains progression state, and the
skill map list never becomes routing authority. The temporary379 join sends
readiness/control and1065 without consuming or replaying the pending carry;
only the matching destination consumes it. Existing GUID, maximum-health,
enabled-heal and active-session guards remain intact.

The synthetic saved-character/loopback393 regression first failed at
MapLoaded with the pending-carry compatibility error, then passed after the
correction: abort the temporary379 join, reconnect, redirect, and restore
health321, XP5000 and remaining cooldown at393. A separate407 regression
checks unconfigured destinations (even when `next_map` names them), wrong
map/health/profile, other GUID, duplicate active session, abort, matching
restore, early recast rejection and retired-source protection. The existing
map28 reconnect test also passes. Saved request/auth bytes remain unchanged.

The three passing focused tests and actual failure-before log are retained in
`ether-feast-story-resume-20261107-checks/receipt.json`, SHA256
`0293502081da9ac4c3abfa85f4bcd78da7dc1db4f16aed4dc716018b2d113875`.
The source commit also built as a static PIE musl game observer; its separate
`musl-release-receipt.json` pins the artifact and build log. These are offline
implementation checks, not a deployment receipt or native reconnect proof.
The separate publication/readback above establishes the live version.
Normal story placements, authored encounter rules and full retail fidelity
remain open. Process-local progress still does not survive a server restart.

The final cumulative startup receipt is
`ether-feast-story-resume-10803b4-preflight-20261107/checks.json`, SHA256
`93951681b13ef3df8a3916b98de35f9673c84d05e78897fef06a0679655f4c69`.
Both full saved-mode auth/world checks and six supported prepared-map checks
pass without listeners; map15 is rejected by the expected skill guard.
The explicit blank tutorial-heart profile and all126 story arguments are
preserved. Tests use only synthetic owned temporary records/stores.
