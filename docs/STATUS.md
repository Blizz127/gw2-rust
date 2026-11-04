# Status

Lead: Claude, session `claude-gw2-rust`, since the 2026-11-01 ~20:19Z host reboot. The previous Codex lead was thread 01a19513, and the owner handoff was 9acee8ca7cde4358bbdf0fb61fdc2310. Updated 2026-11-01.

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
  - **Candidate bootstrap sequence:** [re/209026-bootstrap-sequence.md](re/209026-bootstrap-sequence.md)
    consolidates the static notes into one ordered candidate path from route to
    BeginPlaying. Each edge is marked stated, inferred or open. No new REA was done
    and nothing was replayed. The [Play route](re/209026-play-route.md) is now traced statically. Play runs the
    name transaction (client 22 → server 8 → server 3), whose completion then sends auth 29 with a game type derived from
    the summary map's type (18-pair table), the summary map key, a hash and a 16-byte context value (wire
    layout mapped). Queensdale (key 15) is map type 5 → game type 3, from archive file 198302's `mMet` table.
    Typed Play-path frames (`gw2_proto::play_route`: client 22/29, server 3/20) pass
    three focused tests cross-checked against `auth.xml`; nothing was exchanged with the client.
    `gw2-game-observe --auth-play-route` adds an opt-in offline Play phase (22 → 8+3, 29 → 20);
    8 observer tests pass, including every-split replies. The run-58/59 captures replay without
    disabling the phase (client 4 and 34 parsed). The Play phase's read timeout is 600 s, because
    run 59 showed the 30 s timeout forcing a re-login. Client replay waits on the Run59 hold.
  - **Mission entry (2026-11-02):** [re/209026-mission-entry.md](re/209026-mission-entry.md).
    The client completes the game join by itself on the type-1 key reply (constant
    success result → PLAYER_FLAG_CONNECTED), so no server acknowledgement exists; 1267
    is unnecessary (it would match no state-3 transaction). Server 1063 is MapData (field roles mapped); it must precede
    1060 and its map ID must equal route result 20 field 3 (inferred). Client 423
    follows once both the join and 1063 landed. Typed frames (`gw2_proto::map_load`:
    1063/1060/1061/1036, client 423/420/418/412) pass 13 focused tests against
    `mission.xml` group 39; nothing was exchanged with the client.
    `gw2-game-observe --mission-preamble --mission-map-key KEY` adds the opt-in offline
    mission phase (after the key reply: 1063 for KEY + placeholder 1060; client frames
    logged by ID; never 1267/1036). 13 observer tests pass, including a loopback
    exchange. The loader will stop at AGENT_STREAM until population exists.
  - **Population sequence (2026-11-02):** `gw2_proto::population` composes the candidate
    order 428 → 37 → 443 → 183 → 282 → 477 → 151 → 247/248 → 565 → 538 → 605 → 170 → 618
    → 444 (A == B) → 46 from existing encoders. Every value either cites a docs/re constraint or is marked
    `placeholder:`. The caller supplies identities, spawn position/facing and 45 appearance
    bytes. 8 tests (order, decoder round-trips, census decode, index consistency, plan
    validation, determinism). Open: whether agent index = character index, 183 vs 182,
    and where 46 falls. Not exchanged with the client.
    The observer's opt-in `--mission-population RECORD --mission-spawn X,Y,Z` sends it after
    client 420 and sends 1036 + 431 only after a later client 412 (431's direct body checks no loader/view state) (17 observer tests, incl. loopback).
  - **Resumed 2026-11-02 ~14:10Z (pool hold lifted).** Bubble bug fixed: `gw2-dns-observe
    --map NAME=IPV4` gives per-name answers (sandbox `DNS_MAP`). Run 60g, the bubble layout
    resolved by DNS (cligate .2, auth .3, sink .9), passes login TLS, auth and Play
    and stops at the untouched User Agreement panel. A hosts file with no DNS responder
    still fails (60e); the client-side cause is untraced. Run 61 (`tools/gw2-runner`
    gdb script with a fresh `make-identity.py` identity, same DNS layout) applies both
    guarded patches and also reaches the agreement panel. After editing `sandbox/entry.sh`, run
    `sandbox/run.sh build`. Otherwise the stale image ignores `DNS_MAP`, `dns.out` says
    "0 mapped names", and every name goes to the sink (run 60f). B3 answered: the owner accepts
    the agreement himself on his own device; automation never clicks it. Next: run 62,
    the launcher's own bwrap bubble shape (uid 0 mapped, its DNS responder, runner
    under setpriv with no caps) on the devbox, then DIT shas and the deploy.
    **Deployed** (same day): `gw2-server.service` on Banshee, kit `gw2-rust-33ce1bf`, all
    four listeners on 100.74.44.12:6112-6115. The probes pass under stackd's `sg docker`
    view, and a killed child restarts the set. DIT has the probe argv and sha. Run 61b
    (deployed argv and data, runner shape) reaches the agreement panel. Open items:
    - the owner's first test past the agreement, which nothing has exercised
    Closed:
    - device reachability: the launcher lead's headless check reached all four ports
    - run 62: the launcher's bridge.py with the real client reached the agreement panel
    - run 63b: the shipped identity + bridge against the LIVE Banshee server reached it too
    Owner-gated before his first test: the 83 GiB client upload to the device and the
    GW2 signing prefix (the coordinator holds both).
  - **Owner device runs, 2026-11-03.** Nothing has reached Banshee yet.
    - b103 (01:24Z): bubblewrap 0.12 refused to bind over the symlinked `/etc/resolv.conf`.
      The launcher fixed it in b104.
    - b104 (02:00Z): the sandbox starts, then the runner's gdb ends within ~20 s.
      - Cause: GE-Proton 11's `bin/wine` runs a 32-bit `start.exe /exec` that double-forks
        the 64-bit client, and the old script detached from that fork.
      - Its connect handler also raised on the i386 frame.
      - Wine 11 also maps the client at a random base, not 0x140000000.
      - Runs 61–64 had used Debian's `wine64` under gdb (the sandbox gdb mode hard-coded
        it), so this was never tested.
    - `gw2-runner.gdb` now:
      - checks the frame's architecture, and its handlers never raise;
      - finds the client's base from `/proc/<pid>/maps` (by inode or path) and relocates
        every address;
      - arms only when the header page and trust bytes match the exe;
      - prints a line when the client image was never seen.
      Following forks (`detach-on-fork off`) armed the patches under GE-Proton 11, but it
      was fragile: a short-lived fork's exit left gdb's all-stop state with running
      threads, and run 66 never armed under Debian Wine. It was reverted. The client must
      run in the traced process, as it does under Wine 10-class loaders.
    - Run 66 (Debian Wine 10, local servers, the revised script) passes:
      - armed at the preferred base;
      - trust and DH patches applied;
      - login TLS, auth and Play;
      - six post-auth screenshots.
    - `sandbox/run.sh` now honours an absolute `WINE` in gdb mode (plus `GDB_WINE`,
      `GDB_EXE`, `WINEDLLOVERRIDES`, `LANG_OVERRIDE`).
    - Second blocker, separate from the runner: under GE-Proton 11's own Wine (Wine 11.0)
      the client's worker thread overflows its stack right after the login click
      (`virtual_setup_exception stack overflow`).
      - It reaches cligate and never sends `/Auth/StartTls`.
      - This reproduces with no gdb and local servers (run 65j). The overflowing stack is
        full of DirectWrite and "Tahoma" data.
      - `dwrite=d` is not a test: libcef.dll then fails to load ("Failed to load CEF
        DLL. Error code: 0x7e").
      - The device bubble passes no LANG (C locale), the same as the sandbox.
    - Narrowing so far:
      - +seh shows no exception storm on the overflowing thread. Its only exception is the
        Chromium `network_change_notifier_win` log line, so this is a genuine recursion.
      - Not the trigger:
        - the C.UTF-8 locale (65p);
        - Proton-style font links (65m);
        - hiding Proton's bundled fonts (65r).
        Wine also rewrites the Tahoma SystemLink at every start, and Debian's prefix has the
        same chain.
      - GE-Proton10-34 (Wine 10 base, run 67) fails the same way, so it is Proton, not
        Wine 11.
      - Debian Wine 10.0 passes (run 66).
      - Kron4ek's vanilla Wine 10.0 tarball hangs in wineboot inside the sandbox image, so it
        is not yet usable.
    - Owner decision (via coordinator): "yes its own wine10 is fine". The launcher ships a
      self-contained Wine 10.x for GW2. It must pass run 66 first and have a recorded source
      and checksum.
    - Device-side alternatives tried:
      - wine-ge-8-26 (Lutris, already on his device), run 70: GE's DXVK 3.10 needs Vulkan
        1.3 features this Wine 8 lacks.
      - wine-ge-8-26 with DXVK 1.10.3, run 70b: graphics work and it reaches login TLS, but
        the runner never armed. A preferred-base fallback was added for anonymous image
        maps; it is untested.
      - Kron4ek's classic multiarch Wine 10.0 hangs in its 32-bit wineboot.
    - **Run 71 passes the gate** with:
      - Kron4ek Wine 10.0 WoW64, read-only: `wine-10.0-amd64-wow64.tar.xz`, sha256
        aeebbbf239e548f0136f1cd72a2e109dd9a572a8f703da3c09fa40d74d5c255f, from
        github.com/Kron4ek/Wine-Builds release 10.0;
      - GE-Proton 11's DXVK v3.10, a fresh prefix and the shipping wrapper with the device
        argv.
      Result:
      - armed at the preferred base, trust and DH patches applied;
      - full login STS, auth and the game key exchange;
      - no stack overflow, 6 post-auth shots, ending at the untouched agreement panel.
    - After the User Agreement (static, 2026-11-03): the client's STS method tables have
      no agreement method.
      - Account: AddProductKey, LinkGameAccount, ListGameAccounts, ListMyGameAccounts,
        RemoveGameAccount.
      - Auth: login, password and SSO calls plus GetHostname, StartTls, LoginFinish, Resume.
      So I ACCEPT is most likely recorded locally by the launcher UI. The next server
      traffic should be the auth play phase (client 22/29 → server 8/3/20) and the game
      server on route 20, both already handled.
      For the owner's b107 log: read Banshee's `server.log` from the recorded baseline. A
      stall right after ACCEPT with no new auth or game lines points client-side.
    - Auth clients 1, 4 and 34 need no reply while the client sits at the agreement panel.
      Banshee conn 2 (2026-11-02, play route enabled) sent 4 and 34 unanswered, then kept the
      auth connection open through about eight `/Sts/Ping`s plus another client-1
      heartbeat, until that run ended.
      The re-logins in local-server runs 66 and 71 come from the observer's shorter
      no-play-route read timeout closing the session, not from the client giving up.
    - **Banshee kit cf490a8, deployed 2026-11-03 07:16Z** with DIT's ack while nobody was
      on.
      - Only `gw2-game-observe` changed. The mission phase now frames the group-0 agent
        client messages ([agent client messages](re/209026-agent-client-messages.md)).
        Before, client 17 between 420 and 412 disabled the phase, so 1036 was never
        sent.
      - Guarded stop, link switch while stopped, start: ready after **3.3 s**.
      - `server.log` baseline for the owner's next run: line 103.
      - stackd r57 has gw2 as wake_only, probes pinned to 5825dfb (`humans` across
        login/auth/game).
      - A guarded stop left the unit `failed` (exit 143 on SIGTERM). The installed unit now
        has `SuccessExitStatus=143` (sha256 02d7ca0f…, daemon-reload only; the previous unit
        is kept as `gw2-server.service.prev-9716fb8e`). The next kit's `deploy/` carries it.
    - **Banshee kit 56945a9, deployed 07:45Z** with DIT's ack.
      - Descriptor kind 15 is verified as Optional, so agent clients 4/7/9/10/11 frame.
      - The kit's unit has `SuccessExitStatus=143`; the guarded stop left the unit
        `inactive`.
      - start→ready 3.3 s. **`server.log` baseline for the owner's next run: line 107.**
      - Rollback: link to cf490a8.
    - **Reading the owner's b107 run.** Banshee kit 56945a9, baseline line 107.
      Tools: `deploy/tail-stamped.sh` (running since 09:01:54Z) and
      `tools/owner-run-report.py`. What run 71 does not cover: his GPU under DXVK 3.10,
      Bazzite's host libraries under Kron4ek Wine, and everything after I ACCEPT.
      Where the furthest milestone points, in order:
      1. Nothing reached Banshee: the device side.
         - gw2-runner.log: "never seen", stack overflow, `gw2 bubble:` or `bwrap:` lines,
           provisioning.
         - The journal: missing host libraries, runtime file modes or symlinks after the
           in-process extract.
         These are launcher fixes.
      2. Login stops before RequestGameToken: a TLS or trust problem.
         - Check the "trust patch applied" line.
         - The package identity matches Banshee (run 63b).
      3. Stops at auth login 10 or the agreement: normal until he clicks I ACCEPT, then
         PLAY.
      4. 22 but no 29: our server 8/3 name-select reply.
      5. 29 but no 6112 mission opening: route 20 (127.1.0.4:6112) through the launcher
         relay to Banshee 6112.
      6. Mission opening but no 420 within ~120 s: client-side map load (Queensdale from
         Gw2.dat), or our 1063/1060 placeholder values.
      7. 420 but no 412: the population sequence.
      8. 1036 sent and agent traffic seen: in the world. The next work is what the client
         waits for there.
    - After BeginPlaying (static, 2026-11-03): no server keepalive is needed on the game
      connection.
      - `GcGameCmd.cpp`'s disconnect reasons are exactly `DISCONNECT_NONE`,
        `_DISPATCH_FAILED`, `_BAD_COUNT` and `_MISSING_HANDLER`, next to "MsgConnDispatch
        failed on message %u". There is no idle or timeout reason (strings near
        file offset 0x1922258).
      - What disconnects is a bad server message: an unregistered id, a bad count, or a
        failing handler.
      - So in-world server messages are added only from the owner's observed traffic and
        verified handlers, never stubbed speculatively.
    - **Owner's first b107/b108 device run (14:22:52Z).**
      - Worked: the Kron4ek Wine 10 prefix provisioned, the bubble preflight passed, the
        runner armed (preferred base, no gdb error, no stack overflow), and the client
        reached Banshee cligate at 14:22:58Z.
      - Then the client showed "Unable To Open Archive File … Gw2.dat". The launcher's
        bubble mounts the client folder `--ro-bind`, and GW2 opens Gw2.dat for write and
        writes Gw2-64.tmp and debug.log there.
      - The sandbox runs mounted it read-write and left Gw2.dat unchanged (mtime and size).
      - Fix (launcher): `--bind` the client folder. No server change.
    - **b109/b110 device runs (14:41–14:59Z).** These runs got through:
      - the writable client folder and both patches;
      - login TLS, SSO, accounts and the token;
      - the auth key exchange.
      Then the client failed with "58:11:5:535". That is error:product:module:line, from
      the client's format string `Network error: %u:%u:%u:%u (error:product:module:line)`.
      - b110's relay trace: up 82 B, down only 2 B of the 22-byte key reply, client EOF
        14 ms later, the rest one RTT (154 ms) later.
      - Banshee wrote `[1,22]` and the nonce as two writes, and Nagle split them. The
        client rejects a split key reply.
      - Run 72 (the same stack over a ~0 RTT ssh tunnel) passed, so the split never
        showed in the sandbox.
      - Fixed in a9624d8: TCP_NODELAY plus one write for the key reply and its first
        frames, with a one-read regression test.
      - Kit a9624d8 deployed 15:05Z (DIT ack), ready after 3.5 s.
      - Kit 8f3af5b deployed 16:07Z in an idle window (DIT ack), ready after 3.3 s.
        - It sends one write per read for all replies.
        - Each auth and game connection logs a sizes-only byte timeline (`net +Nms: up/down
          N B`, client EOF); `tools/owner-run-report.py` prints it.
        - Rollback: link to a9624d8.
    - **First full device login (16:22Z, kit 8f3af5b).**
      - Auth worked: the 30 B key reply in one read, then login messages and the play
        phase.
      - The client then reached character select with "no home world". Our auth server 8
        account record has 7 zero Dwords and an EMPTY world array.
      - It also sent unanswered STS `/Mail/SetOptIn`, `/Presence/SetWatch` and
        `/Game.com.Wallet/SetWatch`, plus auth clients 15, 39 and 11. It never sent 22 or 29.
      - Our login server's 30 s idle read timeout dropped cligate (16:23:08Z).
    - Kit 7941b0f (empty replies for those STS paths, 600 s idle) crash-looped: the
      `--reply-xml` validator rejected the dotted path. Rolled back by hand to 8f3af5b,
      down ~6.5 min with 0 humans.
    - Fixed in b9b83c3 (validator plus test), smoke-ran the real `gw2-server.sh` locally
      first, and `watched-cycle.sh` now rolls back by itself.
    - Deployed 16:36Z (DIT ack), ready after 3.4 s.
    - Home world (static, 16:45Z): auth server 13 is NOT a world list.
      - Handler 0x1402409c0 → 0x14023a8f0 stores 16-byte entries (Word key, Word value,
        3 Dwords) in a global table (`[0x14266c008]`, count `0x14266c014`) and posts event
        0x1d.
      - Lookup 0x14023ceb0 returns an entry's Word1 for Word0 == key. It is called with
        small constant keys (1, 0x4b, 0x51, 0x58) whose results are tested as flags, so this
        is a key→value account flag table.
      - The home-world field is still unidentified (server 8 → 0x14023a770). I asked the
        owner whether PLAY works despite "no home world".
    - **Home world (16:58Z screenshot).** The "Initial Home World Select" list was empty and
      PLAY greyed.
      - REA ([home world](re/209026-home-world.md)): the list is server 23 (clear), 25
        {world, population 0–3, 0, 0} and 24 (publish), filled once when the dialog
        opens. Selecting sends client 14, which server 7 confirms.
      - Client 15 is the User Agreement accept.
      - Fixed in d3b07c9: world 1001 in the login reply, and client 14 answered with
        server 7.
      - Kit d3b07c9 deployed 17:06Z (DIT ack, smoke-run first), ready after 3.3 s.
      - Not verified yet: which check enables PLAY after the selection.
      - **Confirmed live (17:07:03Z).** The owner logged in, selected Anvil Rock (logged twice) and pressed
        PLAY, so PLAY is enabled by the 14→7 exchange.
    - **First map load (17:07Z): stalled on the Queensdale loading screen.**
      - The game server sent 1063 MapData and 1060 LoadMap.
      - The client then sent message 278. No loaded group framed it, so the log showed
        "Mission phase disabled: no schema for message id 278" and the population after 420
        was never sent. The owner closed the client at about 17:09Z.
      - **Fix in 7b051e4.** `data/protocol/209026/client-tables.xml` has all 34 client
        registration tables (525 messages, extracted with
        `~/re/gw2/notes/extract-client-tables.py`; sha256 b0f026b4…, REA ev_3ada7c78…).
        - 278 is in table 0x14223bf20 (ids 278–286).
        - 7b051e4 added 421 ids and left out 8 that tables define differently (30, 34,
          36–40 and 89).
        - **Fixed in 0edbe1f:** those clashes were the auth connection's table
          0x141923f00 against the game's main table 0x14216e260.
          - The auth table is identical to auth.xml (28 of 28 layouts), so it is now
            skipped on 6112.
          - The game table ends exactly at its server table 0x14216e940 (0x6e0 B = 219
            descriptors plus a null) and now wins clashes.
          - Result: 428 ids, none ambiguous. Only auth 29 (route request) is unframed on 6112.
          - Kit 0edbe1f deployed 17:24:19Z (DIT ack, smoke-run first), ready after 3.3 s.
    - **Retries on 0edbe1f (18:47Z, then b114 at 21:17Z): the loading screen still hangs.** Login, world select, PLAY and 278 are fine.
      - The client answers 1063/1060 with 278, 424 and 423, then stays silent.
      - The asset CDN gets no request, and the only unrouted DNS name is loading-live.ncplatform.net (loading-screen web content).
      - **b114's runner stage trace** (runner a8fdb4e): the loader goes LOAD_CONTENT → LOAD_MANIFEST → SERVER_WAIT (all at 21:18:29), and 1060 then moves it to **MAP_DOWNLOAD at 21:18:30**. It never leaves that stage.
        - MAP_DOWNLOAD (`0x1409520b0`) waits, with no timeout, on loader +0x218, the MapPortalLoader request created in BeginMap. Its +0x48 is the sum of its child map-file loaders' +0x38 ([map ready](re/209026-map-ready.md)).
        - **Corrected after b115 (22:38Z):** with a fresh Local.dat the client did fetch the manifests (13 MB), but MAP_DOWNLOAD still entered and exited in the same second. That is an error exit to state 6, not a wait.
        - **Cause:** our root manifest had no `MapValleyHill` / `RegionValley` (Queensdale's map/region manifest names), so the map request is null. Fixed in the CDN data with root 2130706520 ([patcher](re/209026-patcher.md)); run 79 OK.
        - The earlier Local.dat theory was only part of it.
        - **Device result on cd93d58 (2026-11-04 01:25Z, b115): the map loads.** Client 420 MapLoaded arrived at 01:26:27Z, 0.5 s after LoadMap. The server then sent the candidate population sequence, and **the client crashed at 01:26:33Z**: both connections hit EOF, and the owner said "i loaded in and it crashed". The 00:15Z session never got past one idle connection.
          - Next: the device's ArenaNet.log (assertion text) and the b115 stage lines before 01:26:33Z. Don't change population values before the crash is named.
          - **01:51Z retry (cd93d58): stuck at character select**, world "Unknown", PLAY greyed. There was no home-world dialog and no client 14; an unanswered client 12 (841 bytes) followed a click. Cause: no account state, so the login carried no world. **Fixed in 102bad3** (login server 8 Dword6/Dword7 = 1001; [home world](re/209026-home-world.md)): smoke-run, DIT ack, live 02:05:59Z, ready after 3.3 s, rollback cd93d58. Not yet seen on a real client.
          - **02:08Z run (102bad3): PLAY works**, Queensdale loads (420 at 02:08:52Z), then the same crash 4 s later.
          - **Crash named (device ArenaNet.log, pulled by the coordinator; both runs): `Assertion: m_health`, ChCliCharacter.h(575).** Trace: ViewAdvanceDevice `0x14094d22c` → DvContext.cpp `0x14131a777` → `0x14131ab10` → health getter `0x1411d7cb0`. Once a character is controlled, the per-frame device context registers its POI, core stats, health, skill and task managers. The population sequence never created health. **Fixed in 8d3a414:** health 256 (alive, 1645/1645) and endurance 217 (100/100/100, divisor 2) are sent after 151 and before 444; the values are placeholders.
          - **03:18Z / 03:23Z (8d3a414).** At 03:18Z there was no dialog and PLAY was greyed; the only traffic was client 12 (841 B), now identified as the account-preferences upload (PrContext.cpp `0x14027da93` → `0x14023bcc0`). At 03:23Z the dialog came back (14→7), then PLAY, LoadMap and 420, and **the m_health fix held**. The client then crashed with `c0000005` (null write) at `0x140b48c9d`: ImgMip.cpp mip generation in a GrTex2d.cpp texture task (`0x140a6f490`, the second path, flags +0x7c & 0x380 and +0x10 & 0x10), on worker thread 0x178.
            - The device runner (7c3883f, shipped) showed MAP_STREAM → MODELS_STREAM at 03:23:36Z, then the crash. There was no CDN traffic during the load, and the device Gw2.dat is untouched (frozen size, mtime 11-02).
            - **Diagnostic kit 3eb003c** (live 03:38:26Z): population held 15 s after 420 (`--mission-population-delay-ms`), client 12 answered with server 7, CDN requests in server.log. A crash at 420 means map models; a crash at +15 s means our population.
            - Greyed PLAY with Dword6/7 preset is still open. `0x14023d944` zeroes the home world when `0x1410f4be0` (context +0x68 check `0x1410f8ac0`) and the flag table `0x14289ebe0[0x60]` (`0x1410e9aa0`) are both set; not traced further. Workaround: pick the world in the dialog.
          - **03:58Z diagnostic (3eb003c): the crash follows the population.** The population was held from 03:57:45Z (420) and sent at 03:58:00Z; the client crashed at the same rip (`0x140b48c9d`) in that second, and nothing happened during the hold. **f89ea50** (live 04:03:33Z) drops 247: a placeholder type-23 key 1 (UID 87, container 1282847 +0x172e00). Type 23 has the outfit catalog's shape, and 248's four Words fit dye channels. 248 alone still creates the wardrobe. This is the prime suspect, not proven.
            - If the same rip recurs, the next test is launcher-side: dxvk.conf `dxgi.hideNvidiaGpu = False`. The device reports VendorId 0x1002 / DeviceId 0x73df (DXVK's NVIDIA hiding) and VidMem 0 MB.
            - Runner follow-up: the crash report printed `from ?`. `stack_calls` reads 2 KB at the faulting rsp; it should read in 64-byte chunks and keep partial results.
          - Runner crash report (runs 80/81): read-only breakpoints on the ExeError sink `0x1409dc570` (all 8 error paths, including the assertion handler `0x1409db7a0`) and the unhandled-exception filter `0x1409dc720`. They print "GW2_RUNNER: assert … file:line "expr"" and "GW2_RUNNER: crash code … at <offset>" with stack offsets. Armed with no false hits on login; untested on a real crash.
      - The a8fdb4e trace hooked the profiler's per-frame marker (about 1,600 hits/s). It is being moved to the loader's 22 call sites.
      - "Banshee Realm": the world-name patch applied on his device.
        - Regression test: `client_278_during_the_map_load_keeps_the_phase_and_the_population_follows`.
      - Kit 7b051e4 deployed 17:15:31Z (DIT ack, smoke-run first), ready after 3.5 s. The
        owner was told he can retry.
    - Still open: the Proton recursion's root cause (no stable return addresses in the stack
      dump). It doesn't block the Wine 10 path.
  - **PAUSED 2026-11-02 ~06:45Z (owner pool hold).** PLAY-button work for task dbf27f7e17a0.
    Done, local commits only, nothing deployed:
    - no-capture servers and bind guard (71ffe34, c76ce60)
    - bubble runner and identity generator (80a3433)
    - deploy bundle with stackd readiness_only probes (38c1f83); ports 6112–6115 on
      100.74.44.12 agreed with DIT
    - TLS errors printed to the console, plus the sandbox SERVER_BIN override and static
      musl builds (4a2a2dd)
    Open bug: with the launcher's bubble layout (hosts file; cligate 127.1.0.2; auth
    127.1.0.3) the real client aborts TLS with alert 10 before certificate handling
    (runs 60, 60b, 60c). Run 59's layout with the same new static no-capture binaries
    passes through Play (run 60d). Run 60e (hosts file with run 59's IPs) also fails, so the cause is
    **hosts-file name resolution**, not cligate's address. The bubble needs a DNS stub
    that answers like run 59's sink (gw2-dns-observe), or the reason the client's TLS
    depends on how the name was resolved must be found.
    Next steps, in order:
    1. Read 60e and fix the layout with banshee-realm-client.
    2. Run 61, the runner end to end (prepared in ~/gw2-sandbox/runs/run61-runner-e2e).
    3. Send the shas to DIT and deploy via the coordinator.
    Open owner question: B3 (agreement click-through).
  - **Build dependency:** `gw2-game-observe` and the play_route/map_load/population tests
    `include_str!` the client-extracted schemas in gitignored `data/protocol/209026/`.
    A clean checkout without that private directory will not build them.
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
    The branch follow-up identifies B as the player index in both branches
    and character flag bit 3 as IsControlled. Equal IDs install local control;
    unequal IDs update the player's character association without setting
    that flag. Local-player subscriptions use embedded objects and do not
    immediately call listeners; subsequent notification dependencies remain
    open. Owned-character control also directly requires coreStats. No replay
    or User Agreement interaction was performed for this static follow-up.
    Server 565 now has a traced reward-track creation path and typed codec:
    player index, four type-353 IDs and four unnamed bytes. Three focused
    tests and the library build pass. Payload-free server 431 is a further
    manager consumer with unresolved startup callbacks; valid track IDs,
    slot bytes and its exact sequencing remain open. See
    [reward-track messages](re/209026-reward-track-messages.md).
    The 431 follow-up names its extra character object m_wardrobeMgr
    and maps lazy creation through 247/248. Typed small wardrobe frames
    pass three focused tests and library build; their Word meanings remain
    open. Fresh zero 248 allocates the manager without changing its cleared
    four-Word state. 431 additionally asserts skill-bar owner association
    and dereferences wardrobe coreStats; later progress/content callbacks
    still need work ([wardrobe startup](re/209026-wardrobe-startup.md)).
    The [player-tail follow-up](re/209026-player-startup-tail.md) resolves
    several later getters to embedded account/achievement/progress objects
    and confirms a guard around optional player +0x97b8. Content/category
    evaluation and runtime callbacks remain unresolved.
    The [numeric-ID audit](re/209026-content-numeric-ids.md) corrects
    specialization wire values to UIDs (+0x14), distinct from data keys
    (+0x28). The raw codec fields now say UID; catalog views expose both.
    Six focused tests, library/binary builds and the 32-container Rust
    smoke pass, including the independently checked Guardian mapping.
    Palette, availability, reward-track and wardrobe paths use data keys.
    The [reward-track census](re/209026-reward-track-content.md) resolves
    all 529 tracks to type-53 progress definitions and maps Guardian
    thresholds. Missing progress yields zero passed Guardian thresholds;
    active-track selection alone does not grant progress. No replay occurred.
    [Progress messages](re/209026-player-progress.md) now identify server
    511 as a controlled-player update with a type-53 data key and up to
    four values. Guardian definitions use counter channel 4. The bounded
    codec passes three focused tests and the library build; downstream
    callbacks and initial progress choices remain separate work. The
    specialization progress listener is now traced: final progress
    notification refreshes matching slot bytes without clearing traits;
    another 640 assignment would clear them. No replay was performed.
    [Completion predicates](re/209026-progress-completion.md) distinguish
    all-channel completion from numeric maximum and threshold counts. The
    bounded type-53 reader passes three focused tests and builds; a REA
    archive smoke resolves all 81 specialization completion limits. An
    independent pointer join corrected four Guardian rows in the earlier
    reward-track table (key 13 / UID 15 uses track 24, progress 4858).
    [Bit-progress setters](re/209026-bit-progress.md) now establish 509
    mask replacement and 510 channel-block splitting with final-only
    notifications. Both bounded raw codecs pass three focused tests and
    the library build, including the 255-word array boundary. A complete
    progress census now covers 23,939 definitions. BIT_MASK completion
    counts set bits against a threshold distinct from storage width; the
    typed parameter reader passes four focused tests. The optimized Rust
    checker resolves 9,616 counter/maximum limits and 2,986 bit-mask
    parameter records, matching the independent census. Debug archive
    checks timed out; the optimized check passed in about 15 seconds.
    [Character locale](re/209026-character-locale.md) now maps all 15
    agent states to skill-palette locale, including two history-dependent
    states and a callback correction/rebuild. Constructor state 1 supports
    locale 0; later callbacks remain unverified. The [Guardian entry
    follow-up](re/209026-guardian-palette-eligibility.md) reduces six actual
    palette rows and identifies nonrecursive standard-slot membership.
    Palette 127 / skill 9083 is a conditional static slot-0 candidate,
    requiring level, locale, availability and component prerequisites;
    no rendered skill bar or usable action is claimed. Bounded Rust
    palette/row/entry/skill views pass three focused tests and the checker
    build. Its REA archive smoke matches all 7,769 palettes, 20,990 rows,
    27,320 skill references and six independently inspected candidates.
    [Slot-level settings](re/209026-skill-slot-levels.md) resolve standard
    slots 0–4 to frozen values 0/11/15/19/31. Executable initializer defaults
    differ; missing-record behavior and earlier membership success remain
    separate from these final fallback thresholds. The bounded integer-
    setting reader passes two focused tests and the optimized checker
    build; a REA smoke confirms one matching record per standard slot
    across all 32 base containers.
    [Slot follow-up](re/209026-reward-track-slots.md) identifies 565/569
    pin values, 568 active-track selection and 566 map-specific selection.
    Pin value domains and 567 keyed-state meaning remain unresolved.
    Queensdale map key 15 has zero map-specific reward-track entries,
    so no 566 entry-selection message belongs in that map fixture.
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
    The first level is earned level; the second level's name remains unresolved.
    Initialization's earned-level callback already dereferences the associated
    player's owned coreStats, before finalization. For a fresh owned character,
    the required order is 428, 37, 443, then 182 or 183;
    see [re/209026-character-core-stats.md](re/209026-character-core-stats.md).
    Downstream finalization requires the fresh player's owned-character pointer
    and coreStats, plus inventory for the finalized coreStats. Server 282 creates
    that inventory component; its framing check passed (two frames, seven bytes;
    truncated varint rejected). See
    [re/209026-character-finalization-dependencies.md](re/209026-character-finalization-dependencies.md).
    Specialization/major-trait messages 640–643 now have typed framing
    codecs: single-slot assignments, fixed 3+9-key replacement and slot swap.
    Three focused tests and the library build pass. Non-null specialization
    recalculation needs the owned character and conditionally its reward-track
    manager; initial content selections remain unresolved. See
    [re/209026-specialization-messages.md](re/209026-specialization-messages.md).
    The base-content census maps all 81 specialization keys, 2,425 trait keys
    and 729 major-trait membership references. Nine profession definitions
    partition all 81 specializations exactly once, confirming the Guardian
    list; level/unlock/runtime eligibility remains separate. See
    [re/209026-specialization-content.md](re/209026-specialization-content.md).
    Bounded `gw2-dat` profession/specialization/trait views now expose these
    fields and explicit references. Three focused tests and library/checker
    builds pass; the 32-container Rust smoke matches the independent census.
    The bounded cross-container resolver passes two focused tests/build;
    the checker now resolves all 81 profession memberships, 729 major-trait
    and 81 progress references to indexed records of the expected types.
    Gameplay eligibility remains unverified.
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
| 5 | One zone (Queensdale, map 15) load-in and spawn | Primary-map prop translations, local rotations and sequence selection are implemented. Eight focused prop/zone tests/build and all 8,301 Queensdale prop matrices pass offline checks; 536,453 selected vertex instances agree with independent rotations within 0.000072 collision units. 62 props have empty selected collision lists and no blockers; the client substitutes a small layer-15 box, whose initial filter pairs only with layer 0 (see [fallback trace](re/209026-prop-fallback.md)). Token lookup selects 819 non-first animations, including 212 different mesh lists. Resident terrain heights, packed surface selectors and chunk collision exclusions are implemented: seven focused terrain tests/build pass, plus 544,768 exact grid nodes, 1,089,536 interior height queries, all cell surface IDs and 532 chunk flags checked offline. Primary-map terrain triangle extraction also passes an all-1,089,536-triangle consistency check. All 4,381 zone placements have verified fixed-sequence selection and transforms (92,367 triangles); see [zone collision](re/209026-zone-collision.md). The offline `gw2-map-probe` now reports vertical hits by terrain/prop/zone source; two geometry tests, build, argument checks and three real archive probes pass ([probe scope/results](re/209026-surface-probe.md)). The [static ray-filter predicates](re/209026-ray-filters.md) are documented; character ray wrappers now have verified mask `0x7830` and six-point height sampling with water context ([character trace](re/209026-character-ray-mask.md)). Typed [water metadata](re/209026-water-volumes.md) and probe output now expose Queensdale’s enabled global water at Z=0 (no explicit polygons); two focused tests/build and REA smoke pass. Server 37 now has a traced map-unit position and transform tag, including tag-zero conversion and constructor bounds handling ([agent position](re/209026-agent-position.md)); the probe reports equivalent packet coordinates for surface hits. The [dimension trace](re/209026-agent-dimensions.md) confirms final-float radius, array-element-3 height, two height adjustments and the ordinary capsule’s bottom-relative origin. Typed [scene bounds](re/209026-scene-bounds.md) distinguish the full-character clamp from the expanded simple-character bounds; two tests/build and Queensdale probe smoke pass. The [creation-flag trace](re/209026-agent-creation-flags.md) maps the three bytes to path/shape selection and distinguishes identity from the collision player bit. Build-specific [facing conversion](re/209026-agent-facing.md) decodes signed pitch/yaw and encodes finite creation directions with nearest-even quantization; sentinel directions are rejected, and five focused tests plus the library build pass. The [movement-parameter trace](re/209026-agent-movement-parameters.md) identifies array 0 as base speed and array 1 as a yaw-rate scale; array 2 remains unnamed, with bounded movement/decal consumer evidence. The [command-clock trace](re/209026-agent-command-clock.md) establishes 40 ms timestamp ticks, the delay estimator and client-17 delay report; 12,005 isolated native-leaf/model comparisons pass. The [behavior-data trace](re/209026-agent-behavior-data.md) supports an empty payload and zero auxiliary Dword for first type-0 creation only; the factory and completion path do not interpret them. The [dimension-offset trace](re/209026-agent-dimension-offsets.md) follows array 5 into body/character vertical-origin conversion and array 7 into the water level used by movement. The [alternate-shape trace](re/209026-agent-alternate-shape.md) identifies array 6 as the alternate convex shape Y radius, derives its 37 input points and confirms the simple-character radius cap. A typed [server-37 frame codec](re/209026-agent-creation.md) now preserves all explicit fields with transactional decoding and bounded behavior data; all eight focused agent-creation tests and the library build pass. The [player-field trace](re/209026-player-creation-fields.md) identifies server 428's literal name, portal-user GUID and initial-state bit, including its input-binding side effect and related 459/460 transitions. The [spectating trace](re/209026-player-spectating.md) identifies 459's agent target and type-334 camera record through a direct client assertion; individual state values remain unnamed. Its player-listener follow-up shows that 459 removes the controlled view, while 460 can restore it; both require a finalized controlled character when AvContext is subscribed. A typed server-428 frame codec now preserves raw bounded UTF-16, GUID bytes and flags transactionally; three focused tests and the library build pass. The [core-stat follow-up](re/209026-character-core-stats.md) maps appearance scale and corrects server 182: the third attribute Word is signed, while the first two are unsigned. A reward consumer identifies the first level byte as earned level, with explicit getter override limits. Typed alternative 182/183 frames preserve their common fields, signed attribute column and float bits; three focused tests and library build pass. The [constructor trace](re/209026-character-finalization-dependencies.md) resolves the transformation listener to an embedded object and confirms zero initialization of the extra fields omitted by 183. The [health follow-up](re/209026-character-health-endurance.md) identifies creation states 0/1/2 as alive/dead/downed and the first meter rate in units per second. Health messages 257/259/260 update existing component fields; 258 removes that component without changing the health state. Typed health frames cover all five operations, with three focused tests and library build passing. A static setter follow-up distinguishes raw input callback values from clamped meter storage: F5 triggers the maximum-health listener with the first maximum, while its gameplay name remains unresolved. Later [health-state transitions](re/209026-health-transitions.md) use server 84 with packed opcodes 0x33/0x34/0x49; an explicit transport/payload codec passes three focused tests and the library build. State transitions remain untested in the client. The [server-64 point trace](re/209026-agent-point-adjustment.md) establishes an active-command prerequisite, so it is not a standalone initial-position update. Its final byte selects a later destination (full target/simple queue) or the existing position setter; both retain the active-command prerequisite. The [server-86 update trace](re/209026-agent-position-update.md) reaches the full collision position write without that prerequisite, retaining other gates and falling back to the old point on bounds failure. The simple collision path instead clamps the requested point, performs a ground query and conditionally snaps/interpolates the resulting position. Its separate [constructor predicate](re/209026-agent-point-admission.md) compares raw coordinate magnitude against a configurable scene threshold; the failure flag's later effect remains unresolved. Its explicit frame codec passes three focused tests and the library build; client acceptance remains untested. The paired [server-87 state form](re/209026-agent-position-state.md) uses three Float4 fields and collision-coordinate position, with different full/simple consumers. Kind 9 is now verified as 16 raw bytes and admitted to the capture census; seven focused tests, binary build and a two-frame offline schema smoke pass. A typed 87 frame now preserves all vectors, raw tag and state bytes; three focused tests and the library build pass, without runtime transmission. The shared bit-1 movement gate is now named COLLIDER_FLAG_ENTERED_SCENE; [server 49](re/209026-agent-scene-entry.md) reaches the scene-entry setter, with repeated-entry assertions and bootstrap ordering still to resolve. Its explicit transport codec passes two focused tests and the library build. [Server 46](re/209026-agent-world-entry.md) separately supplies agent world entry and optional collision scene entry after position/facing updates; its option bit 1 suppresses only the latter. The collision constructor-tail callback is a no-op; the [corrected character callback](re/209026-character-entry-callbacks.md) returns early for identity high nibble 0x3. Duplicate entry must be avoided. Explicit 46 framing passes three focused tests and the library build. The [entry-body trace](re/209026-entry-collision-bodies.md) confirms constructor-owned body allocation and separates the full world-entry visibility phantom from collision scene entry. The [reveal callback trace](re/209026-agent-reveal.md) connects collision entry to AGENT_FLAG_REVEALED through immediate or deferred callbacks, with matching conceal dispatch on exit; rendering readiness remains unverified. The [character-view listener](re/209026-character-view-reveal.md) forwards reveal into an ordinary component allocated synchronously by the view factory; its [finalization subscription](re/209026-character-view-finalization.md) now connects server 170 to view creation and requires player state 3 (server 428 low flag bit clear) on this path. [Server 585](re/209026-character-view-suppression.md) separately deletes/recreates the ordinary view after finalization; its raw-byte codec passes two focused tests and the library build. The [ordinary model-input trace](re/209026-character-model-inputs.md) connects coreStats appearance and inventory to view construction; its player helper also asserts wardrobe, moving that candidate prerequisite before model creation during finalization. Constructor priority thresholds and the conditional fallback loader are traced; neither view allocation nor fallback entry guarantees a loaded model. The [fallback content census](re/209026-character-model-records.md) finds a unique type-146 match for all 90 ordinary race/sex/profession combinations, including Human/Guardian. The [shared file-reference resolver](re/209026-content-file-references.md) identifies Human/Guardian fallback file 527140; three focused Rust tests, library build and nine-record REA archive smoke pass. All nine fallback model files are present with valid packfile chunk framing; Human/Guardian’s race scale factor is 1. Live policy values and runtime readiness remain unverified. The endurance trace identifies current points, two maximum values and a segment divisor; messages 218–221 remove or update existing endurance. Typed 217–221 frames preserve unsigned values, with three focused tests and library build passing; the divisor's gameplay name and useful initial values remain open. The [deficit tracker](re/209026-endurance-deficit.md) has separate balance/shortfall fields, a strict half-maximum predicate and a frozen type-64 exemption record (key 783); it does not directly change current points. The exemption queries a [constructor-owned buff context](re/209026-character-buff-context.md), whose initial empty collection returns false; no separate network creation is required for this context. The [chatter-registration trace](re/209026-character-profession.md) adds a separate profession-component prerequisite before player finalization and maps nine alternative creation handlers; the profession scalar in coreStats is distinct. Guardian creation 477 uses maximum/rate/current plus two unnamed bytes; update 479 has five wire floats although its handler reads only three. Typed Guardian 477–480 frames preserve all fields, with three focused tests and library build passing. The [skill-bar lifecycle trace](re/209026-character-skillbar.md) maps 618/619/620 to creation/notification/removal and distinguishes registration timing from the profession requirement during normal rebuild. The [ordinary control-view listener](re/209026-character-control-view.md) requires that skill bar before equal-ID 444 when the view is subscribed, earlier than the direct 431 startup use. Its effect-preparation follow-up independently consumes the skill bar and the built-in levelUpSkillDef content; the referenced frozen type-64 key 1080 is present. The slot trace identifies 622/623 as single/multiple auto-attack selection; typed 618–623 lifecycle/selection frames pass three focused tests and the library build. Palette lookup now separates player-backed slots 0–4 from inventory-backed slots 5–9 and 19; eligibility predicates and valid initial records remain open. Message 627 is identified as a positive recharge-rate scale, with old-rate advancement before rescheduling. The [player-palette trace](re/209026-player-skill-palettes.md) identifies server 612 as type-65 palette selection by slot/context/player index; its immediate rebuild requires the owned character and skill bar. A typed 612 codec preserves the exact field order and raw values; three focused tests and the library build pass. The [skill-palette census](re/209026-skill-palette-content.md) checks all 7,769 base palettes and 27,320 type-64 references; candidate Guardian rows still require level, locale and separate skill-availability checks. The [availability trace](re/209026-skill-availability.md) maps 605/606 to initial set/reset and 609/613 to mutations that rebuild an existing owned skill bar. Typed availability frames pass three focused tests and the library build. The empty-inventory follow-up confirms 68 cleared equipment slots and initial weapon set 11 mapping to no-slot sentinels in the inspected palette getters; broader empty-inventory readiness remains untested. The remaining direct finalizer calls include three no-op stubs; context listeners remain unexhausted. Typed [setup frames](re/209026-character-setup-frames.md) for 151/170/282/443/444 pass three focused tests and the library build, retaining 444’s explicit A/B fields. The [wardrobe startup follow-up](re/209026-wardrobe-startup.md) traces all refresh loops: an unchanged empty membership bitset skips per-record eligibility, but coreStats profession must map to a valid armor weight class. The [achievement startup follow-up](re/209026-player-startup-tail.md) checks all 121 threshold records: zero points selects threshold 100 with a null optional sublist, skipping that nested progress loop. Both fixed category queries retain caller-initialized zero outputs because their content lists are empty; none of 486 categories matches the third parent, so its capped child loop is also empty. Six optional runtime progress lookups have verified required channel-8 definitions, and the mandatory value-conversion threshold record is present; the change mask is also stored in the packed `plrs` state, with an explicit null-storage reset path. A [subscriber follow-up](re/209026-startup-subscribers.md) resolves three embedded-player startup callbacks to no-ops and traces a conditional quest callback. The [navigation guard](re/209026-quest-navigation-guard.md) is a direct map-layout link-count read; the empty-quest predicate is false independently of that count, and the map-layout startup callback itself is a no-op. Its two refresh helpers skip deeper quest and marker work for unchanged constructor state, while the final service query resolves built-in descriptor 398 to present type-68 key 156. Its null progress prerequisite skips that lookup after category checks pass, but its mask-0x20 reference maps to category 4/value ID 78/service key 59, leaving a separate subscription branch when the stored or fallback value is false. That registration does not immediately invoke its listener; the next query passes the type-68 GUID to context service +0x1f8, while its final consumer skips input zero. The [store-request follow-up](re/209026-quest-store-request.md) resolves that result to a nonzero request ID and traces GetLimits followed by a conditional GemStoreSearch builder. The [callback follow-up](re/209026-quest-store-callbacks.md) traces internal item/package events 0x42/0x43 and a twelve-byte request-ID/error event 0x50. Missing collections skip those item/package emissions. The [dispatch follow-up](re/209026-quest-store-dispatch.md) identifies copied queue buffers inside outer event 0x23, item/package listener notifications and a no-op error slot in the quest-facing service; a [separate tracker listener](re/209026-quest-store-tracker.md) removes the matching null-valued request entry on ordinary error delivery. A [matching zero-TotalPrice item result](re/209026-quest-store-item-followup.md) can build a separate Purchase request (quantity 1, AsContent=1), track its ID and unsubscribe the quest listener; this is static behavior only and no operation was executed. The inspected search success callback and request/callback destructors emit no additional completion event; successful empty-response cleanup, other error listeners and zone-readiness impact stay open. A [reader sweep](re/209026-quest-store-tracker-readers.md) finds that the tracker's item and package slots are no-ops. None of its 23 getter call sites or six methods reads pending entries; only an exit destructor does. So this tracker supplies no readiness wait for the startup search. Other gates remain unchecked. The [Purchase result callback](re/209026-quest-store-purchase-callback.md) discards success: its +0x20 slot is a no-op, and only an error posts event 0x50. So the guarded Purchase continuation needs no server answer through these callbacks. The [subscription-flag sweep](re/209026-quest-store-subscription-flag.md) finds that quest +0xf0 bit 0x10 is read only by destructor cleanup. With the tracker and Purchase findings, no static wait on a store answer was identified. A never-answered request probably becomes a timeout error through the 0x50 branch, whose retry path is untraced. Dword-load readers outside the quest code region and runtime ordering also remain unverified. The [content-direction startup trace](re/209026-content-direction-startup.md) adds server 538 (one player-index Dword) before 431 for its reported-quest assertion. The typed quest-report frame passes two focused framing tests and the library build; its built-in configuration has a null filter, while five base-content overrides require finalized owned-character/coreStats access and an earned-level getter result at most 9. Alert, chat-context, character, buff and the unnamed state-dependent listener startup slots also resolve to no-ops, completing classification of the original bounded search; this remains an incomplete census of runtime listeners. The payload-free [server-431 startup codec](re/209026-player-startup-frame.md) passes two focused framing tests and the library build, with no observer integration or client execution. Runtime configuration selection, other listeners and runtime behavior remain unverified. Full walkable ground, content-specific dimensions, valid bootstrap choices and runtime state remain unresolved. See [terrain heights](re/209026-terrain-height.md) [surface/collision flags](re/209026-terrain-flags.md), and [terrain collision basis](re/209026-terrain-collision-basis.md). See [coordinates](re/209026-map-coordinate-spaces.md), [rotation/mesh](re/209026-collision-rotation.md) and [sequence selection](re/209026-collision-animation-selection.md). |
| 6 | Movement, a few NPCs/mobs, basic attacks (web API data) | `gw2-api` importer done. Build 209026 cache: maps 1086, skills 4763, specializations 81, professions 9, races 5, continents 2, Queensdale floor (47 POIs, 17 hearts, 7 hero challenges, 26 sectors) |
| 7 | Combat, events, loot | later |

## Crates (last full workspace check 2026-11-01: 221 pass, 1 ignored; `cargo clippy --workspace --all-targets -D warnings` clean on rustc 1.98.1)

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
