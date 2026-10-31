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
- The image is built only once the client has arrived. There is nothing to run before that.
