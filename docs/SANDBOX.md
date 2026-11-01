# Offline client sandbox (Wine on the devbox)

Owner decision 2026-10-31: the GW2 client runs on the devbox under Wine, firewalled,
and never reaches ArenaNet.

## Isolation runner: Docker `--network none`

Unprivileged user namespaces are blocked on this host
(`kernel.apparmor_restrict_unprivileged_userns = 1`):

- `bwrap --unshare-net` fails with `bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`.
- `unshare -rn` fails with `write failed /proc/self/uid_map: Operation not permitted`.

The wotlk-forever lane hit the same wall. Docker (29.1.3) is available to the devbox
user. A `--network none` container has only `lo`.

Preflight, 2026-10-31, using `debian:trixie-slim` and bash `/dev/tcp`:

| Run | Result |
| --- | --- |
| `--network none` → 1.1.1.1:443 | `connect: Network is unreachable` |
| default network (control) → 1.1.1.1:443 | reachable |

## Plan

- **One container, `--network none`.** It runs our server and the client together.
  The client connects to `127.0.0.1`, via `-authsrv`/memory patches or hosts
  overrides, and there is no route anywhere else.
- **Image:** Debian trixie + `wine` + Mesa lavapipe (`lvp_icd`, software Vulkan)
  + DXVK (DX11 → Vulkan) + Xvfb.
- **Mounts:**
  - The client install is mounted read-only from `/home/blizz/gw2-client/<build>/`.
    It stays frozen.
  - The Wine prefix and logs live under `/home/blizz/gw2-sandbox/` (outside the repo).
- **Patch applies only to the running process.** The client's server key / auth host
  is patched in memory, never by editing `Gw2-64.exe` or `Gw2.dat`. The client checks
  and repairs `Gw2.dat`, and the on-disk copy stays pristine.
- **Teardown:** confirm the container exited and that no `wineserver` is left running.
- The image is built and preflighted ahead of time (below). Only the client run waits for the install.

## Implemented (2026-10-31): `sandbox/`

The image (`gw2-rust-sandbox:latest`) is Debian trixie-slim with Wine 10.0, Mesa
lavapipe, DXVK 3.1.1 and Xvfb. `sandbox/run.sh` always starts it with `--network none`.

| Command | What it does |
| --- | --- |
| `sandbox/run.sh build` | Build the image. This needs network; no client is involved. |
| `sandbox/run.sh preflight` | Run the checks below. No client needed. |
| `sandbox/run.sh selftest` | Build `gw2-login-observe`, run it inside the isolated container, send a scripted STS Connect over loopback, and assert the capture. Passed 2026-10-31 (`selftest: ok`). The extra 0-byte connection in the log is the readiness probe. |
| `sandbox/run.sh prepare <build>` | Check the writable working copy `/home/blizz/gw2-sandbox/client-work-<build>` against the frozen baseline: `Gw2.dat` size, and `Gw2-64.new.exe` byte-identical. Then do the launcher's own swap: `Gw2-64.new.exe` becomes `Gw2-64.exe`, and the old `Gw2-64.exe`, the installer bootstrap, is dropped since it is identical to `Gw2Setup-64.exe`. |
| `SERVER="gw2-login-observe --log /sandbox/logs/capture" sandbox/run.sh client <build> [args]` | Run the prepared working copy (writable, so the client can unpack `bin64/`), as the host user, on cores 0-15 by default (`CPUS=`). It optionally starts a workspace binary first. The frozen baseline is never mounted. |

**Preflight checks:**
- isolation: only `lo` exists, and outbound TCP fails
- Wine starts
- Xvfb starts
- Vulkan reports `llvmpipe`
- DXVK DLLs are present

**Preflight result 2026-10-31:**
- `isolation: ok (lo only, outbound unreachable)`
- `wine-10.0 (Debian 10.0~repack-6)`
- Vulkan device `llvmpipe (LLVM 19.1.7, 256 bits)`
- `preflight: ok`
- No container left behind.

**Server in the container:** the host-built `gw2-login-observe` (glibc 2.43 host) ran inside
the container and captured an STS message over loopback.

**Incident 2026-10-31 19:25Z:** while I was testing a guard, `run.sh client 209026` started a
container. It used the half-copied working copy, whose `Gw2-64.exe` at that point was the
installer bootstrap. Wine refused the root-owned run (`'/sandbox/prefix' is not owned by
you`) and the container exited before any game executable was launched. It ran with
`--network none`, and no container was left behind. Fixes: client mode now runs as the host
user and requires `prepare`, which only swaps in the real client after verifying it.

**First offline client runs (2026-10-31 19:28–19:40Z, build 209026, `--network none`):**
- **Run 1:** the container user couldn't execute `entry.sh`, because the file mode in the
  image was owner-only. Fixed with mode 755 and a world-writable `/sandbox`.
- **Run 2:** the prepared working copy `Gw2-64.exe` (sha256 `99342078…`) starts under
  Wine 10.0 + DXVK 3.1.1 on lavapipe (`Found device: llvmpipe`). The DXVK log shows the
  launcher UI uses D3D9.
  - It shows the GW2 launcher window, "Connection error(s) detected. Retrying…"
    (screenshot kept privately at `~/re/gw2/notes/run2-offline-launcher.png`).
  - **Nothing reached the observer** on 127.0.0.1 ports 80, 443, 6112 and 6600. The name
    lookups fail first, since there is no DNS offline.
  - `bin64/` stayed as 0-byte placeholders. The client zeroed its `Gw2-64.tmp`
    (self-update staging) and created a fresh `Local.dat` in the sandbox's own Wine prefix.
    It holds no credentials.
  - Next: map the client's hostnames, from REA string evidence, to 127.0.0.1 with
    `--add-host`, so its first connection lands on the observer.
- Client runs now take a screenshot of the virtual display every 20 s into
  `/sandbox/logs/shots/` (`SHOT_EVERY`).

**Where state lives:** the Wine prefix and logs persist under `/home/blizz/gw2-sandbox/`.
DXVK goes into the prefix on first client run (`.dxvk-installed` marker).

**Sandbox network notes (2026-10-31, runs 11–21):**
- **Map ArenaNet names to 127.1.0.x, never 127.0.0.x.** The client's StsLib
  address lookup recurses until the stack overflows when the target is in 127.0.0.0/24 and
  the host has no non-loopback address (docs/re/209026-connect-path.md). Use `DNS_SINK=0`
  with
  `ADD_HOSTS="cligate.101.ncplatform.net=127.1.0.1 cligate.101.ncplatform.net.=127.1.0.1 assetcdn.101.arenanetworks.com=127.1.0.2"`.
  The trailing-dot form is needed: the client looks up `cligate.101.ncplatform.net.`.
- The container gets a fixed hostname (`gw2-sandbox`) that resolves to 127.0.0.1.
- **The client trims its archive.** `MultiArchiveTrim` drops every asset that no
  manifest it loaded references. A root manifest that lists nothing (run 27) emptied the
  working copy's `Gw2.dat` index: entry 1 became 8.99 MB of zeros. Every later run then
  saw all assets as missing. `sandbox/run.sh restore <build>` copies the frozen baseline
  back (89 GB, a few minutes on ext4). After an experimental run, check the index with
  `gw2-dat-ls <working Gw2.dat> 1` (entries must stay 1,124,232). Serve manifests that
  reference every local asset.
- `GDB_SCRIPT=/sandbox/logs/x.gdb` runs the client under a custom gdb script. Arm address
  breakpoints from a one-shot `tbreak connect if *(unsigned short *)$rsi == 2` (the first
  AF_INET connect), since the PE image isn't mapped at start.
- `GDB_STACKDUMP=1` runs the client under gdb (in the image now). It dumps the PE stack
  (`STACK_LO`..`STACK_HI`, default `0x7c0000`..`0x8c0000`) into
  `logs/stack-dump.bin` when Wine reports a fatal stack overflow.
