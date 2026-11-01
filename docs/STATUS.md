# Status

Lead: Codex, session `claude-gw2-rust` (owner handoff 9acee8ca7cde4358bbdf0fb61fdc2310). Updated 2026-11-01.

## Blocker

- **Client frozen** 2026-10-31 19:03Z as `/home/blizz/gw2-client/build-209026` (see
  [client-build.md](client-build.md)). The REA target is `Gw2-64.new.exe` (sha256 `99342078…`).
  REA confirms that target identity; current certificate-check analysis is described below.
- `Gw2.dat` reader is verified on real data (see client-build.md). Queensdale = map file 192711.
  The schema reader now honors explicit element limits and rejects overflowing
  64-bit pointer offsets. Five focused tests and the inspector build pass.
  Queensdale's 20 chunks decode with explicit `--max-elements 16000000`;
  its occlusion chunk exceeds the unchanged four-million default. See
  [re/209026-packfile-reader-bounds.md](re/209026-packfile-reader-bounds.md).
  A borrowed `ContentIndex` now reads the verified base-container layout.
  Three focused tests/build and all 32 frozen containers pass, enumerating
  982,772 records with the independently established map/appearance/palette/
  answer counts. Reference methods now interpret local, external and string
  fixups without mutating archive bytes. Six focused tests/build and a
  32-container smoke pass: 78 palette names, 50 matching queries, 90 parsed
  defaults and bounds for 180 external template references. See
  [re/209026-content-registry.md](re/209026-content-registry.md).
  `gw2-content-check` exposes metadata-only index/type counts; its build,
  single-container and full-base smokes, and twelve CLI rejection cases pass.
- The client runs offline in the sandbox (Wine 10 and GE-Proton 11).
  - **Silent-connection blocker solved (2026-10-31, run 21)**:
    - Cause: the client's StsLib address lookup recurses forever when the host's only
      address is in 127.0.0.0/24, and that killed its socket I/O thread.
    - Fix: resolve the ArenaNet names to 127.1.0.x. The client is untouched. Details in
      [re/209026-connect-path.md](re/209026-connect-path.md).
  - The client then sends, speaking first:
    - asset CDN (`assetcdn.101.ArenaNetworks.com`, TCP 80): `GET /latest64/101`. This is
      the patch check, and the launcher shows "Downloading 0KB" while it waits.
    - cligate (`cligate.101.ncplatform.net`, **TCP 6112**): plaintext
      `P /Sts/Connect STS/1.0` (ConnType 400, ProductName Gw2-64, Program 101,
      Build 1004), then `/Sts/Ping` every 30 s.
  - **Certificate check mapped statically (2026-11-01):** the RSA client requires
    SHA-1/RSA signatures, a matching DNS SAN, valid dates and a chain signature
    verified against an embedded issuer Name/key entry. The local generator now
    emits SHA-1/RSA, DNS SAN and keyEncipherment; its focused tests pass 2/2,
    including OpenSSL trust/hostname verification. Existing generated identities
    were not replaced ([re/209026-certificate-check.md](re/209026-certificate-check.md)).
  - `-authsrv host[:6112]` sets the later AuthSrv hop (docs/re/209026-cmdline-and-hosts.md).
  - **Patch step solved (2026-11-01, run 35,
    [re/209026-patcher.md](re/209026-patcher.md)):**
    - `gw2-assetcdn` answers `/latest64/101` and serves files.
    - `gw2-manifest-gen` builds an `ARMF` root plus `Launcher64` (every local asset) and
      `English` (language) manifests from `Gw2.dat`.
    - The launcher reaches its normal ready screen offline: 100%, PLAYABLE, READY, login
      form, build 209026.
  - **Login (run 36/37, [re/209026-login.md](re/209026-login.md)):**
    - LOG IN sends `P /Auth/StartTls` (`s:1;timeout=29000`).
    - After our `200 OK` the client sends a TLS 1.2 ClientHello: static-RSA AES-CBC
      suites only, no extensions.
  - **TLS passed (2026-11-01, run 39):** the local server completes StartTls and
    TLS 1.2 AES128-SHA with the frozen client in the offline sandbox. The runtime
    hostname is `cligate.101.ncplatform.net.`; the certificate SAN needs that final
    dot. The test configures our own trust modulus in process memory only, guarded
    against the original bytes. Both executable hashes remain unchanged.
    First decrypted request: `/Auth/GetHostname`, with `LoginName` and `Provider`.
    See [re/209026-login-tls.md](re/209026-login-tls.md) for REA evidence and limits.
  - **GetHostname passed (run 44):** the opt-in local reply supplies Hostname and
    Local. The client requires a final LF before parsing XML. After that correction,
    it sends `/Auth/StartSsoLogin` on the same TLS connection. See
    [re/209026-gethostname.md](re/209026-gethostname.md).
  - **Synthetic SSO continuation passed (run 45):** an explicit local reply fixture
    supplies a test UserId, LoginName and Provider. The client reaches its completion
    callback and sends `/Account/ListMyGameAccounts`. This is a protocol fixture,
    not credential validation or an account database. See
    [re/209026-startsso.md](re/209026-startsso.md).
  - **Game-account alias passed (run 46):** a one-entry alias fixture is accepted,
    and the client sends `/Auth/RequestGameToken` with GameCode and AccountAlias.
    See [re/209026-game-accounts.md](re/209026-game-accounts.md).
  - **Synthetic game token passed (run 47):** the GUID reply reaches the completion
    callback. The client opens a second local TCP connection and sends 16 + 66
    binary bytes while the portal connection stays open. See
    [re/209026-game-token.md](re/209026-game-token.md).
  - **Binary cipher passed (runs 48/50):** the guarded in-memory public-key
    substitution produces the same DH shared prefix as the local server. Its
    key reply is accepted, and all 57 decoded bytes exactly match the client's
    pre-encryption buffer. See [re/209026-game-handshake.md](re/209026-game-handshake.md).
  - **First binary reply passed (run 52):** a compressed/encrypted protocol-3
    server message 2 reaches its handler with the intended value. The client
    responds with messages 1 and 10; the latter continuation carries the synthetic
    portal token. All 115 decoded bytes match the client plaintext. See
    [re/209026-auth-messages.md](re/209026-auth-messages.md).
  - **Correlated binary auth fixture passed (run 53):** the client matches ID 8
    account data to its request and ID 7 marks it complete. Client ID 4 follows;
    ID 34 is then observed at serialization. All 230 captured bytes match plaintext.
    Run 54 repeats this result but shows a crash dialog after completion:
    `s_configDataError != NET_PENDING`, `GcApi.cpp(2621)`.
  - **Configuration reply passed (run 55):** correlated server ID 10 with an empty
    buffer, before completion, clears the pending flag. No crash dialog or assertion
    is observed; the launcher remains on its login spinner. The client sends
    `/Auth2f/Check` and `/Auth/GetUserInfo` next.
  - **Post-auth portal replies (run 56):** empty replies are consumed and the
    launcher displays email verification for the synthetic identity. Field probes
    identify AuthTypeName/AuthType and LastPasswordChange. No Send action performed.
    See [re/209026-post-auth-portal.md](re/209026-post-auth-portal.md).
  - **Verified fixture (run 57):** EmailVerified=1 reaches the SSO callback;
    AuthTypeName None is consumed. The launcher displays its optional 2FA signup
    reminder. **Run 58:** Remind me later reaches the news panel and active Play
    button. All 233 captured bytes match client plaintext.
  - **Run 59 owner-decision gate:** Play opens a User Agreement / I ACCEPT panel.
    It remains untouched; explicit owner direction requested through the coordinator.
    Further UI progression waits. Independent character-protocol RE continues.
  - **Static character-list path:** server ID 17 populates records consumed by
    the character-select scene. The summary blob's version-11 size and validation
    checks are mapped. The preview copies 45 appearance bytes from summary +6,
    establishing its race/sex offsets; preview construction rejects sex 2 even
    though normalization admits it. Several other fields remain unresolved. Account reply
    consumers identify home shard, auth base time and a billing category enum.
    Character-slot capacity is computed from the ID-8/ID-13 account array plus
    mode-dependent base counts; the formula is documented, runtime UI untested. The new
    `gw2-capture-census` reports IDs/boundaries without field values; focused
    tests/build/clippy pass and the run-58 capture parses completely. See
    [re/209026-character-protocol.md](re/209026-character-protocol.md).
    Named consumers also establish summary level/profession and its required
    map-definition key. Frozen-content tracing resolves key 15 to a type-45
    data-ID record referencing Queensdale file 192711; runtime registration
    remains untested ([map-definition content](re/209026-map-definition-content.md)).
    The new `gw2-summary-check` checks version-11 structure
    without printing values or claiming content/preview validity; two focused
    tests, checker/census builds and six CLI smoke cases pass.
    The six preview records now map to type-66 skin data IDs and four type-9
    Word color IDs each. `SummaryV11::preview_equipment()` exposes them with
    the observed negative/zero skip rules; three summary tests and checker
    build pass. Named slot mapping and runtime rendering remain unverified.
    A private Human/Male profession-1 candidate now combines shipped appearance,
    template skins/colors and map key 15. Its 159-byte summary and 226-byte
    record pass structural, frame-census and observer startup validation.
    Opaque fields are zero and slot order is an explicit fixture convention;
    no client replay occurred. The complete equipment loop passes resolved
    models/colors rather than the record index to model calls; that does not
    prove permutation equivalence. Details are in the character-protocol note.
    Static refresh mapping identifies client ID 25, pending ID-17 list records,
    and completion event 0x65 returning to char_select on success; runtime
    ordering remains untested.
    The dynamic codec also now enforces variable-buffer `Size` maxima in both
    directions, matching the client; two focused tests and observer/census
    builds pass, and saved capture boundaries remain unchanged.
    REA also confirms raw 4/8/12-byte Float/Float2/Float3 encodings in both
    directions. The census admits them; four focused tests, build and private
    mission-control fixture smoke pass. An explicit terminated CString rule
    now handles raw bytes with inclusive size bounds; its focused codec test,
    five census tests, observer/census builds and real-descriptor fixture pass.
    Text encoding and gameplay meanings remain unknown. See
    [re/209026-primitive-fields.md](re/209026-primitive-fields.md).
  - **Static character-creation path:** the appearance constant registry now maps to
    ten GUIDs and 32 base content containers (archive IDs 1282830–1282861).
    All ten type-137 records are located in file 1282852. Relocation-checked
    extraction yields 90 shipped default strings, one per race/sex/profession;
    the bounded `gw2-default-check` now parses all 90/90, preserves the supplied
    59/60 color rows and applies the client's Sylvari appearance adjustments.
    Three focused tests, build and five CLI rejection cases pass. Runtime
    validity remains unverified.
    See [re/209026-content-registry.md](re/209026-content-registry.md).
    Creation context sends ID 415 through
    mission protocol 0, group 39. Registration and descriptor establish its
    bounded string, fixed/variable arrays and scalar wire layout. Local validation
    identifies race/sex offsets and the separate profession byte, with embedded
    numeric labels; content-dependent appearance validity remains unresolved. The static
    creation-scene exchange maps server 1049/client 410 and server 1050/client
    411, plus the view readiness flags. Server 1051 feeds the creation error
    dialog; server 1048 updates the list and, outside cinema states, starts
    name-based auth request 22. Auth server ID 8 also supplies that pending
    name transaction's character GUID through a separate type-0x15 branch,
    before completion by ID 3. Private partial mission-schema boundary fixtures
    parse correctly. Runtime startup and creation remain unverified. See
    [re/209026-character-creation.md](re/209026-character-creation.md).
    The bounded answer array is now traced to sorted content answer bytes,
    with separate question-bitset bookkeeping and race/profession filters.
    The single base type-260 record in archive 1282831 contains 45 questions
    and 193 answers. Content filtering yields five questions per race/profession
    combination, with no co-applicable answer-byte collisions. The two client
    selection paths differ on some answers; runtime/server acceptance remains
    unverified. See
    [re/209026-creation-answers.md](re/209026-creation-answers.md).
    The final 24 Dwords map to item-color selections for six equipment slots;
    content validity and runtime creation remain unverified. See
    [re/209026-creation-colors.md](re/209026-creation-colors.md).
    Race/sex content also supplies 18 equipment-template references each,
    sharing nine level-1 templates across all races/sexes. Their three armor
    entries for slots 2/3/6 resolve to concrete type-66 skins; later preview
    overrides remain unverified. See
    [re/209026-creation-equipment.md](re/209026-creation-equipment.md).
    New `gw2-creation-check` inspects a complete ID-415 frame with a 250-byte
    bound and no field-value output. Two focused parser tests, checker/census
    builds and six CLI smoke cases pass; this does not validate game content.
  - **Static mission join:** auth request 29 and reply 20 lead through a
    correlated route transaction to the protocol-0 game join. Creation uses
    route mode 1 (`GAME_TYPE_CHARCREATE`); its scene call path and the small
    mission control group are mapped. Address encoding
    copies 28 raw bytes with a big-endian port; the Rust port codec is corrected.
    Exact-byte IPv4/IPv6 test and observer/census builds pass. Mission bootstrap
    remains untested. The observer now accepts an explicit `--mission-preamble`
    mode for the 84-byte prefix; three focused tests (including auth/mission
    loopback cipher exchanges), build and CLI fixture-conflict check pass.
    Validated prefixes are retained in exclusive mode-0600 sidecars; the
    loopback test verifies their exact bytes and no-overwrite behavior.
    The capture limit is now exactly 1 MiB even after a short initial read;
    the focused loopback regression and observer build pass.
    See [re/209026-mission-join.md](re/209026-mission-join.md).
    Static mission ID 1065 supplies an immediate/deferred next destination;
    map-loading IDs 1060/1061/1063 are mapped separately. The synthetic
    redirect fixture parses all 39 bytes, but no transfer sequence was sent.
    Load callbacks identify client 420 (MapLoaded) and 418 (FailedToLoad),
    with matching prepared-map/state prerequisites. Offline load/result
    boundary fixtures parse 44/4 bytes completely; no live load was attempted.
    Route preparation supplies the loader's map definition before joining;
    ordinary readiness sends text/audio languages in client 423. Server 1036
    requires loader READY, while the default-view ID-1061 callback is empty.
    Client 420 is sent before model/asset/agent streaming finishes; the later
    client 412 (AgentsLoaded) follows READY and connection/view guards.
    This provides a static candidate for sequencing server 1036, still untested.
    Agent streaming requires a finalized controlled character. Server 444's
    equal-index branch assigns an existing character; an offline fixture
    parsed two frames completely (16 bytes). Population ordering remains open.
    See [re/209026-character-control.md](re/209026-character-control.md).
    Server 428 populates player entries; server 170 finalizes an existing
    character with player/monster and association checks. A three-message
    framing fixture parsed completely (34 bytes), without client replay.
    Initial character components remain unresolved; see
    [re/209026-character-population.md](re/209026-character-population.md).
    Server 37 reaches the registered character factory through a queued
    engine-agent creation command. Its Point3 encoding is now verified and
    admitted by the census; focused test/build and 386-byte framing smoke
    passed. No live bootstrap is claimed; queue timing and required content
    remain open ([re/209026-agent-creation.md](re/209026-agent-creation.md)).
    The first server-37 creation dispatch is immediate after queue bookkeeping.
    Its identity field maps high-nibble-3 characters to player-table indexes;
    server 443 separately assigns the player's owned-character pointer.
    Server 151 sets attitude only for an existing character. Servers 182/183
    are alternative coreStats creation paths; 185 only updates an existing
    component. Their private framing census consumed 249 bytes completely;
    named icon/trait consumers establish the third scalar byte as profession.
    The two level fields' exact names remain unresolved;
    see [re/209026-character-core-stats.md](re/209026-character-core-stats.md).
    Downstream finalization requires the fresh player's owned-character pointer
    and coreStats, plus inventory for the finalized coreStats. Server 282 creates
    that inventory component; its framing check passed (two frames, seven bytes;
    truncated varint rejected). See
    [re/209026-character-finalization-dependencies.md](re/209026-character-finalization-dependencies.md).
    Appearance validation now has a 45-byte branch map and ten race/sex
    content keys (type 137). All 90 shipped defaults have since been extracted
    and parsed; full content-dependent validation remains unresolved. See
    [re/209026-appearance-validation.md](re/209026-appearance-validation.md).
    An offline generic-property audit passes 1,916 explicit byte bounds
    across those defaults, with 795 zero shortcuts and 495 missing-property
    success paths. It leaves 844 specialized nonzero bytes unchecked and
    makes no full-validation or rendering claim.
    A follow-up locates composite archive 154681 (`comp` version 19), matches
    all ten race/sex profiles and passes 336 more nonzero count/simple-branch
    checks, reducing the unchecked set to 508 packed-key/palette bytes.
    The remaining 159 packed-key and 349 palette checks now pass. All 4,050
    appearance bytes in the 90 adjusted defaults are accounted for by the
    static reconstruction, including permissive missing-property paths;
    no live validator/rendering result is claimed. Encoded palette-name
    conversion is documented in [re/209026-content-names.md](re/209026-content-names.md).
    `gw2_dat::content_name::encode_209026` now provides that bounded conversion;
    two focused tests, library build and all 50 archive-reference checks pass.
    Server 256 creates or updates health; server 217 creates endurance.
    Their framing smoke consumed two frames/44 bytes and rejected a truncated
    float. Scalar meanings and live initialization remain partly unresolved;
    see [re/209026-character-health-endurance.md](re/209026-character-health-endurance.md).
    The local observer now accepts one explicit `--auth-character-record` body
    and correlates server 17 before login completion. Bounded structural checks,
    focused auth tests, the toy-key transport smoke, build and six CLI startup
    checks passed. This has not been replayed with the client; map/appearance
    content validity and the Agreement decision remain pending.
    See [re/209026-map-ready.md](re/209026-map-ready.md).
    See [re/209026-mission-transfer.md](re/209026-mission-transfer.md).
    The game is not yet connectable; character selection remains pending.
  - Scope and ordering for the time-locked progression, plus the core-game completion
    matrix: [SCOPE.md](SCOPE.md).

## Stages

| # | Stage | State |
| --- | --- | --- |
| 1 | Freeze client, capture traffic, message list for our build | client frozen; REA observations and offline patch/STS captures recorded; complete message list pending |
| 2 | Gw2.dat reader | **verified on real build-209026 data**: archive, decompressor, packfiles, map chunks, typed collision |
| 3 | Login stub: redirect, secure login, our own key | offline synthetic fixtures reach logged-in launcher with active Play button; credential validation and account persistence remain unimplemented |
| 4 | Character select / creation | static protocol investigation underway; Play UI waits on explicit User Agreement decision; no character-select screen yet |
| 5 | One zone (Queensdale, map 15) load-in and spawn | Primary-map prop translations, local rotations and sequence selection are implemented. Eight focused prop/zone tests/build and all 8,301 Queensdale prop matrices pass offline checks; 536,453 selected vertex instances agree with independent rotations within 0.000072 collision units. 62 props have empty selected collision lists and no blockers; the client substitutes a small layer-15 box, whose initial filter pairs only with layer 0 (see [fallback trace](re/209026-prop-fallback.md)). Token lookup selects 819 non-first animations, including 212 different mesh lists. Resident terrain heights, packed surface selectors and chunk collision exclusions are implemented: seven focused terrain tests/build pass, plus 544,768 exact grid nodes, 1,089,536 interior height queries, all cell surface IDs and 532 chunk flags checked offline. Primary-map terrain triangle extraction also passes an all-1,089,536-triangle consistency check. All 4,381 zone placements have verified fixed-sequence selection and transforms (92,367 triangles); see [zone collision](re/209026-zone-collision.md). The offline `gw2-map-probe` now reports vertical hits by terrain/prop/zone source; two geometry tests, build, argument checks and three real archive probes pass ([probe scope/results](re/209026-surface-probe.md)). The [static ray-filter predicates](re/209026-ray-filters.md) are documented; character ray wrappers now have verified mask `0x7830` and six-point height sampling with water context ([character trace](re/209026-character-ray-mask.md)). Typed [water metadata](re/209026-water-volumes.md) and probe output now expose Queensdale’s enabled global water at Z=0 (no explicit polygons); two focused tests/build and REA smoke pass. Server 37 now has a traced map-unit position and transform tag, including tag-zero conversion and constructor bounds handling ([agent position](re/209026-agent-position.md)); the probe reports equivalent packet coordinates for surface hits. The [dimension trace](re/209026-agent-dimensions.md) confirms final-float radius, array-element-3 height, two height adjustments and the ordinary capsule’s bottom-relative origin. Typed [scene bounds](re/209026-scene-bounds.md) distinguish the full-character clamp from the expanded simple-character bounds; two tests/build and Queensdale probe smoke pass. The [creation-flag trace](re/209026-agent-creation-flags.md) maps the three bytes to path/shape selection and distinguishes identity from the collision player bit. Build-specific [facing conversion](re/209026-agent-facing.md) decodes signed pitch/yaw and encodes finite creation directions with nearest-even quantization; sentinel directions are rejected, and five focused tests plus the library build pass. The [movement-parameter trace](re/209026-agent-movement-parameters.md) identifies array 0 as base speed and array 1 as a yaw-rate scale; array 2 remains unnamed, with bounded movement/decal consumer evidence. The [command-clock trace](re/209026-agent-command-clock.md) establishes 40 ms timestamp ticks, the delay estimator and client-17 delay report; 12,005 isolated native-leaf/model comparisons pass. The [behavior-data trace](re/209026-agent-behavior-data.md) supports an empty payload and zero auxiliary Dword for first type-0 creation only; the factory and completion path do not interpret them. The [dimension-offset trace](re/209026-agent-dimension-offsets.md) follows array 5 into body/character vertical-origin conversion and array 7 into the water level used by movement. The [alternate-shape trace](re/209026-agent-alternate-shape.md) identifies array 6 as the alternate convex shape Y radius, derives its 37 input points and confirms the simple-character radius cap. A typed [server-37 frame codec](re/209026-agent-creation.md) now preserves all explicit fields with transactional decoding and bounded behavior data; all eight focused agent-creation tests and the library build pass. Full walkable ground, content-specific dimensions, valid bootstrap choices and runtime state remain unresolved. See [terrain heights](re/209026-terrain-height.md) [surface/collision flags](re/209026-terrain-flags.md), and [terrain collision basis](re/209026-terrain-collision-basis.md). See [coordinates](re/209026-map-coordinate-spaces.md), [rotation/mesh](re/209026-collision-rotation.md) and [sequence selection](re/209026-collision-animation-selection.md). |
| 6 | Movement, a few NPCs/mobs, basic attacks (web API data) | `gw2-api` importer done. Build 209026 cache: maps 1086, skills 4763, specializations 81, professions 9, races 5, continents 2, Queensdale floor (47 POIs, 17 hearts, 7 hero challenges, 26 sectors) |
| 7 | Combat, events, loot | later |

## Crates (last full workspace check: 75 pass, clippy clean, commit `31772d8`)

Latest runtime check: run 59 reaches the untouched User Agreement panel after Play;
run 58 has 233 bytes matching client plaintext. Container
cleanup, executable hashes and archive index count verified. Focused checks cover
OpenSSL interop and the observer's pipelined StartTls, split STS, GetHostname and explicit XML replies.
The full workspace suite was not rerun for this change.

| Crate | State |
| --- | --- |
| `gw2-net` | Build-209026 DH little-endian parameters, 20-byte shared prefix/XOR and client-to-server RC4 verified. `gw2-game-observe` captures private decrypted traffic with an explicit local identity. One compressed server frame is accepted in run 52; broader frame/message behavior remains to verify. |
| `gw2-proto` | Primitive encodings (varint Dword, Guid, Point3, Address, Optional…), the `protocols.xml` loader, and `dynamic`: a schema-driven decoder/encoder. Unconfirmed kinds (String, ArrayVar*, Buffer*, CString) are rejected until `Rules` are set from REA. `gw2-schema-census <xml>`: 1440 of 1770 messages in the 208039 reference decode with confirmed kinds only. |
| `gw2-dat` | AN/MFT archive, Huffman/LZ decompressor, PF chunks. Also `pfschema`, a loader for the 208039 reference `packfiles.xml` (6 packfile types, 52 chunks, 372 versions, vendored and labeled), and `pfread`, a safe schema-driven chunk decoder (packed structs, slot-relative 4/8-byte pointers, bounded depth and elements). Arrival tools: `gw2-dat-ls --sample`, then `gw2-dat-pf <Gw2.dat> <ids>` to decode real packfiles. `gw2-dat-pf` tries each schema candidate for names the reference lists twice (`mapc`/`nm15`). `mapc::MapCollide` is a typed view of `havk` collision: triangle meshes, geometry→animation→mesh links, prop/zone placements, validated cross-references. It is the basis for ground height and mob pathing. Primary-map prop translation, local rotation and animation-token selection are verified; `terrain::TerrainHeights209026` samples resident terrain in map-local coordinates. Full collision/walkability integration remains open. The navmesh (`nm15`) payload is opaque Havok bytes. |
| `gw2-login` | STS/1.0 codec and `gw2-login-observe`: captures plaintext or, with explicit local identity, upgrades StartTls and captures decrypted STS into private files. `tls`: minimal TLS 1.2 server, `TLS_RSA_WITH_AES_128_CBC_SHA` only, no extensions. OpenSSL interop and offline build-209026 handshake passed. Explicit XML fixtures reach the binary game connection; account storage and credential validation remain to implement. |
| `gw2-api` | `gw2-api-import build / fetch <ep>... / fetch-map <id>`. Cache in gitignored `data/api/<build>/`. |

## On arrival checklist (client copy)

1. `tools/freeze-client.sh build-209026 <folder>`. This moves the folder Devbox names, makes it read-only, and writes
   a SHA256 manifest. Copy the key hashes into `docs/client-build.md`.
2. `gw2-dat-ls Gw2.dat 20 --sample 25`, then `gw2-dat-pf Gw2.dat <PF ids from the sample>`: opens the archive and reads 25 compressed and
   25 uncompressed entries, printing each one's length and magic. That tests the
   block-trailer guess (probably a per-block CRC) and the decompressor on real data.
3. REA probes below.
4. Sandbox image is already built and preflighted ([SANDBOX.md](SANDBOX.md)); run `sandbox/run.sh client <build>`.
5. Run `gw2-login-observe` inside the sandbox. Launch the client with `-authsrv 127.0.0.1`
   if REA shows the flag still exists. Commit the capture summary under `docs/re/` and
   keep the raw logs in `~/re/gw2/notes/`.

## Known facts and their sources

Keep observed, inferred and unknown separate. Build-209026 REA observations and
offline runtime findings are recorded in `docs/re/`; reference-source guesses below
remain provisional unless a build-specific finding confirms them.

**Observed (outside the client):**
- Live build 209026 on 2026-10-31 (`GET https://api.guildwars2.com/v2/build`).
- ascalon at commit `03fb60b` (cloned to `~/re/gw2/refs/ascalon`, MIT): message
  layouts are for build **208039**, not ours.

**Inferred from ascalon source (reference only, not yet confirmed for our build):**
- Transport is RC4 both ways. The 20-byte RC4 key is a single SHA-1 compression round
  over the folded session key (`rc4_hash`). Server→client frames are
  `u16 compressed_len, u16 raw_len, LZ4 block` and the whole frame is RC4'd.
  Client→server is RC4 only.
- The DH parameters sit in `.rdata` as a 136-byte blob starting `u32 1, u32 4`
  (`research/dh_params.py`). This is the "server key baked into the exe".
- The message schema is registered at runtime through four `register messages`
  functions (build 208039 RVAs 0xFF32C0, +0xC0, +0x180, +0x210) and dispatched at
  0xFEF760. Those RVAs will differ in our build.
- Gw2.dat: `AN\x1A` v151 header → `Mft\x1A` table → entry 2 is the file-id index;
  compressed entries carry an 8-byte header (`08 00 01 80` + u32 size) and use a
  Huffman/LZ scheme. There is a 4-byte trailer per 64 KiB block.
- Network ints: `u32` varint (7-bit groups), `u16`/`u64`/`f32` little-endian.

**Unknown:**
- How the later AuthSrv connection completes (`-authsrv` handling itself is documented
  in [re/209026-cmdline-and-hosts.md](re/209026-cmdline-and-hosts.md)).
- The game messages needed after the confirmed DH/RC4 exchange. The portal flow through synthetic token acceptance is documented above.
- Where runtime-key-encrypted text keys come from.

## First REA probes when Gw2-64.exe arrives

1. `tools/rea-import.sh <frozen>/Gw2-64.exe` (run in the background). It refuses paths
   under `incoming/` or `arenanet-download/` and writable files. It runs a cold Ghidra
   import with a 1 h startup budget and writes
   `~/re/gw2/notes/gw2-64-<sha12>.{snapshot,analyze.json,import.log}`. Then open it in MCP
   with `open_binary(path, provider_id: "ghidra", snapshot_path)`.
   - `inspect_pe_imports` defaults to 64 MiB, so raise `max_file_bytes` if the exe is bigger.
   - Pipeline dry run 2026-10-31 on a read-only copy of Wine's `winver.exe`:
     - the script imported it in 34 s and wrote the snapshot
     - MCP `open_binary` with that snapshot answered `search_strings` (`ev_200eb91d…`) without re-importing
     - outputs were moved to `~/re/gw2/notes/dryrun/`
2. `search_bytes` for the `(1, 4)` DH header in `.rdata` and confirm the 136-byte layout
   by xrefs into the handshake code.
3. Message schema. The register-messages prologue (`48 89 6c 24 08 48 89 74 24 10 48 89 7c 24 18`)
   is a generic MSVC shadow-space save that matches thousands of functions, so filter
   the hits by **adjacency**: four small register functions in a row (spaced +0,
   +0xC0, +0x180, +0x210 in 208039; expect the spacing to drift).
   - Cross-check with the more specific dispatch prologue (`40 53 55 56 48 81 ec b0 00 00 00`)
     and its xrefs from the receive path (the RC4/LZ4 decode loop).
   - Fallback: scan the data sections for arrays of pointers to defs whose u16 at +0x10
     runs through sequential message ids. That is the def layout ascalon's agent reads.
   - Then decompile, read the static def arrays, and regenerate `protocols.xml`
     **statically**. A runtime Frida agent under Wine is the labeled fallback.
4. `search_strings` for `authsrv`, `portal`, `nopatchui`, `assetsrv` and auth hostnames.
   Then trace the login flow.
5. Steam provider. This is a Steam install, so the launcher normally passes `-provider Steam`
   and logs in with a Steam ticket via `steam_api64.dll`.
   - `search_strings` for `provider`, `Steam`, `Portal` and `authsrv`.
   - Inspect PE imports and `bin64/` for steam_api.
   - Find which login path (portal SRP vs Steam ticket) needs the fewest in-memory patches
     to reach our server with no Steam client running.
6. Packfile schema: locate the schema tables that `research/packfiles.py` walks
   (type ids 1..37) for our build.

## Log

- 2026-10-31: lane accepted, CLIENT-NEEDED.md written, ascalon studied, workspace started.
- 2026-10-31: Wine chosen by owner; docker `--network none` sandbox preflight passed;
  gw2-net/proto/dat/api landed; public 2012–13 protocol notes in `docs/re/`.
