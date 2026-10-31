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
| `SERVER="gw2-login-observe --log /sandbox/logs/capture" sandbox/run.sh client <build> [args]` | Start a workspace binary from `target/debug` (mounted read-only at `/server`) inside the same isolated network, then run `/client/Gw2-64.exe` from the read-only mount. |

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

**Where state lives:** the Wine prefix and logs persist under `/home/blizz/gw2-sandbox/`.
DXVK goes into the prefix on first client run (`.dxvk-installed` marker).

The client has never been run, because there is no install yet.
