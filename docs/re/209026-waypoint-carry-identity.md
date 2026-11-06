# Waypoint progress belongs to the selected character and map

The observer kept one pending waypoint handoff for the entire process.
After character A requested a waypoint redirect, any subsequent world join
could consume A's spawn, XP/earned level, loot count and heart state. Leaving
the redirect and selecting character B could therefore transfer A's progress
to B and remove it from A's pending rejoin.

The population fixture now retains the character GUID already present in its
validated auth record. Saved-character selection obtains that record using
the selected mission GUID, as documented in the
[existing binding trace](209026-creation-mission-binding.md). This fix uses
that established identity; it introduces no packet or schema interpretation.

A pending handoff stores the character GUID and destination map key alongside
its progress. MapLoaded applies and consumes it only when both match the
joining population. A different character, different map, or connection
without population leaves it pending. Matching rejoin still consumes it
once. The GUID is omitted from the handoff's Debug output, so the existing
progress log does not acquire a character identifier.

This remains a single pending handoff in a research observer process. A newer
waypoint request replaces that pending handoff; this change does not provide
a multi-player transfer store or durable progression. Process restart still
loses XP, loot and heart state. Character creation records, roster mode,
launcher channels and the native publisher prompt are unchanged.

The change is an ordinary server-source correction. No new shipped-client
execution, archive extraction or REA observation is claimed.

## Focused verification

The new isolated lifecycle test creates two saved characters in a temporary
store and resolves each through its selected mission GUID. It verifies that
B's join, A's join on another map, and a connection without population do
not consume A's pending handoff. It then checks A's exact population/spawn
and XP/loot/heart restoration, duplicate MapLoaded, a subsequent fresh join,
and the identifier-free Debug representation.

All four waypoint-filter tests, 42 mission-filter tests and seven
saved-population tests passed in a separate Cargo target directory. The
existing heart/earned-attribute lifecycle tests retain same-character
rejoin and reward-once coverage. No failed-before run is claimed.
Private receipt: `~/re/gw2/notes/waypoint-carry-identity-tests-20261106.json`,
SHA-256 `9a516d9a2ae58d8d69c18bf075860f92bd1e09770327356b54a238e3bf0d30d1`.
