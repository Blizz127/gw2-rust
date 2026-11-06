# Build209026: explicit level-one starter attributes

The prepared and saved population paths create coreStats with server183.
Its constructor initializes the earned/effective attribute arrays to zero.
The [character-backed tooltip adapter](209026-tooltip-character-context.md)
reads those stored effective values; changing the level does not replace
them with the baseline. The existing fixture therefore lacks primary
attribute inputs even though its health was explicitly set to388.

An explicit unmodified level-one profile can supply **37 for selectors1..4**.
The [default client attribute curve](209026-mesmer-health-speed-baseline.md)
supplies37 at level1 for those selectors. The account-flag alternate curve
would supply24 and is not selected by the constructor default used here.
The [four starter items](209026-mesmer-starter-equipment-health.md) and
[three biography masks](209026-mesmer-biography-equipment.md) contribute
zero attributes with prefix112 and no extra upgrades in this profile.

Using that baseline as earned/effective state, with the separate signed
column zero, is the explicit unmodified server-model choice. It is not an
authenticated retail snapshot or a universal value for other gear, modifiers,
account settings or later levels. The generic equipment parser does not
establish those content conditions, so no value is inferred from profession.

## Existing update contracts

Both updates require an existing raw character and coreStats. They use the
same setters as extended creation182 without choosing its other unknown fields.

| ID | Registration / descriptor / handler | Wire body |
| --- | --- | --- |
|187|`0x14216f110` / `0x1425dd680` / `0x14124e070`|Dword character, Byte selector, Word unsigned value|
|189|`0x14216f130` / `0x1425dd910` / `0x14124e1a0`|Dword character, Byte selector, Word unsigned value, Word signed value|

Complete187 handler `0x14124e070..0x14124e101` calls `0x141208e90`,
storing coreStats+0x7c+4*selector. Complete189 handler
`0x14124e1a0..0x14124e242` first calls `0x141209080`, storing its unsigned
Word at +0xb0+4*selector, then sign-extends the final Word and calls
`0x141208d00`, storing +0x48+4*selector. The effective getter at primary
virtual+0x70 reads +0xb0 directly. Selector assertions require nonzero and
less than12. The signed column remains unnamed in the protocol API.

Helpers `0x1412083c0`, `0x1412084b0`, `0x141208590` and `0x141208630`
maintain cached notification values. In particular +0x204 is copied from
+0xb0; these helpers do not synthesize the effective baseline. This confirms
the earlier tooltip-context note, without exhausting every indirect writer.

## Implemented profile

`--mission-base-attributes VALUE` takes an explicit u16 and requires a prepared
or saved population. `GW2_BASE_ATTRIBUTES` forwards it to the world role and
its supervisor preflight. It is omitted by default.

Configured population adds187 then189 for each selector1..4, after equipment
and the other components, immediately before170 finalization. Unsigned values
are the supplied baseline;189's signed column is zero. There is no update to
selectors5..11, no second coreStats creation, and no guessed182 Dword/floats.
Profiles omitting the setting emit the same bytes as before. Saved joins
carry the setting while preserving selected GUID, appearance and dyes.

Concrete candidate: `~/re/gw2/notes/mission-profile-base-attributes-20261106.conf`,
SHA256 **ce829f7c7c99ee68d4b3e1ece0ee3308433f7468fb5ced60140d92fa4d0f36f1**.
It derives from the locally verifiedcdn900 profile, SHA256
`3e5699820e0fc7c752485f81a04eddd52994eb6f40dc71971ed788022d6d043f`:
add `GW2_BASE_ATTRIBUTES=37`, remove retired `GW2_AUTOATTACK_SKILL=10170`,
preserve every other line. It makes no saved/prepared mode choice. Receipt:
`~/re/gw2/notes/base-attributes-profile-20261106.json`. Original kit unchanged.

## Checks and remaining scope

- Six core-stat codec tests pass, including three new187/189 tests with
  literal bytes, extracted-schema decoding, stream boundaries, all truncations,
  malformed Dwords and raw selector/signed/unsigned limits.
- All38 mission tests pass. New coverage checks exactly eight inserted
  frames with/without equipment, unchanged surrounding bytes, values0/37/65535,
  once-only population and unchanged BeginPlaying.
- All7 saved-population tests pass, including both GUIDs and return selection,
  exact56 added bytes and unchanged equipment/appearance/startup around them.
- CLI rejection/requirement tests and no-listener saved-profile preflight pass,
  including explicit0/37/65535. Three focused supervisor tests pass for unchanged
  default argv and explicit prepared/saved forwarding. Shell syntax passes.

These are server output and configuration checks, not a new client observation.
The subsequent [explicit curve option](209026-earned-attribute-progression.md)
adds earned-attribute maintenance on level-up and rejoin. Complete effective-level
and attribute progression remains unresolved. The startup correction does not resolve that larger requirement,
skill execution, localization keys or the authored Great Hunt encounter.
The sole publisher's unanswered roster prompt and runtime hold are preserved.

## Checked cumulative artifact, not deployed

The newer [9a8eb71 curve build](209026-earned-attribute-progression.md#checked-cumulative-artifact-not-deployed)
includes this tested candidate. Its files and fixed37 profile remain intact.

Source **d9d0bfe** includes the earlier slot-release and explicit skill-order
composition fixes. Its offline `x86_64-unknown-linux-musl` release build passed
in the isolated source tree below, using a separate target directory.

- Binary: `/home/blizz/scratch/gw2-base-attributes-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`.
- Size: **1582544 bytes**, static PIE.
- SHA256: **76a40bae68b3448c2d446358accfd71ed05fb010e4f950090a1cc620e72d6984**.
- Required supervisor: `/home/blizz/scratch/gw2-base-attributes-20261106-src/deploy/gw2-server.sh`.
- Supervisor SHA256: **c006b89524e011039a9b5bdb4df913d2505e82a9f44f7e69a31cd02b8735b6ad**.

At **2026-11-06T02:49:19.544962Z**, this exact binary passed prepared auth,
prepared Norn mission and saved Norn mission `--no-capture --check-config`.
All exited successfully without starting a listener. The saved check used a
private empty temporary store; the owner store and roster selection were not
accessed. Mission checks supplied the explicit baseline37, plus the previous
synthetic hostile/damage and1000ms diagnostic echo. Those diagnostic values
remain test inputs, not new deployment defaults.

The retired autoattack CLI rejection, two legacy-setting compatibility checks
and supervisor shell syntax also passed. Eight pinned fixture inputs and the
run145 manifest remained unchanged. All four ignored schema inputs match the
build-input receipt. Reproducible private records:

- `~/re/gw2/notes/base-attributes-build-inputs-20261106.json`.
- `~/re/gw2/notes/base-attributes-build-checks-20261106.json`.
- `~/re/gw2/notes/check-base-attributes-build-20261106.py`.

Pair this binary and supervisor with the candidate profile described above.
If the publisher's profile has changed since the recorded baseline, apply only
the two specified setting changes to that profile and preserve its other work.
This handoff does not select saved or prepared mode. Prior checked artifacts
remain intact. The last verified live kit remains **82d8d04-cdn900**; no new
deployment or client runtime observation is claimed.

## Source evidence

Frozen PE SHA256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This follow-up opened only that pinned executable through REA helpers.
Earlier curve/equipment/mask evidence remains in the linked notes.

| REA evidence | Scope |
| --- | --- |
|`ev_f310f5a052ea9935c3997511271dd0600d3b388757376ffa327c0d55c235a7aa`|Exact187/189 registrations, field kinds and handlers; no schema written.|
|`ev_6de9bce998841dbce30f1a39941b8a0362b8f0dfcb629866f04955ea4344d7fb`|Complete187/189 handlers and unsigned/signed setter arguments.|
|`ev_68a06e875d948a0facfb1cddce4390e4f50d32906ff0bbb6dde8cd52be3d1c5b`|Setters and notification loops.|
|`ev_34ec0fe7c0a688b53715a53780097c2f11e7481fa913cbfea02ff3a284c21799`|Constructor zero arrays and signed-column notification helper.|
|`ev_fc3952c5aee3a532f0170fd3edbf5195bc9b4496f04eee40c11c105b9c4d87fa`|Complete630 helper continuation and primary vtable.|
|`ev_e3ac9247dc9c3e117b16797c9e3906d2c566f44d248998b16d74420074c34516`|Earned/effective getters and direct second-level getter.|

Private bundle `~/re/gw2/notes/209026-starter-base-attributes-20261106.json`,
7093206 bytes, SHA256
**649bf573e7f983f53f518ed56ea4f1fc0e30247f478f9cfba09ebb01d734b8d3**.
It includes prior evidence; only the IDs above support this slice. All cited
captures exited0 with complete raw/normalized output and verified owned cleanup.
Some cumulative rendered frames were omitted; unrelated unreadable process
metadata remains an observation limitation. No archive or runtime was accessed.
