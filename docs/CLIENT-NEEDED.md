# Client needed from Blizz

Devbox: please relay this to Blizz. It is the only thing blocking stages 1, 3 and later.
Stage 2 (the Gw2.dat reader) and the web-API import can start without it, but they
need the real `Gw2.dat` to be verified.

## Which build

The **current live Windows 64-bit build, whatever it is on the day you copy it.**
ArenaNet does not serve old builds, so we freeze whatever we get and stay on it.

- On 2026-10-31 the live build was **209026** (`https://api.guildwars2.com/v2/build`).
  The public reference work (valaphee/ascalon) covers 208039, which is no longer
  obtainable. We will regenerate message layouts for our build ourselves, using REA.
- Please launch the game normally once so it finishes patching, **then close it
  completely** before copying. If you can, note the build number. It is shown on the
  login screen and in the launcher. We also record it ourselves when the copy arrives.

## Which files: the install directory only

Copy the Guild Wars 2 install folder (usually `C:\Program Files\Guild Wars 2\`, or the
Steam/Epic library folder if you installed it there):

| File | Needed | Notes |
| --- | --- | --- |
| `Gw2-64.exe` | yes | The client we reverse engineer. We pin it by SHA256. |
| `Gw2.dat` | yes | Game data archive. It is very large, roughly 70–80 GB. |
| `bin64\` (whole folder) | yes | Runtime DLLs (e.g. CoherentUI / browser, d3d helpers). |
| anything else in that folder | optional | Harmless if included. |

### Do NOT copy (this keeps your ArenaNet account off the devbox)

- `%AppData%\Guild Wars 2\`. In particular **`Local.dat`**, which holds the saved
  login/remembered-account data, and `GFXSettings.*.xml`.
- `%LocalAppData%\ArenaNet\` and any Steam/Epic login or credential caches.
- Any password, authenticator or account email. We never need them. Our server issues
  its own keys and accounts.

## How to get it to the devbox

The devbox has about 4.2 TB free on `/home/blizz` (checked 2026-10-31). Any of these works:

1. **rsync/scp** to `sbdevbox:/home/blizz/gw2-client/<build>/`. Best for an 80 GB file
   because it resumes. Example from Windows (WSL or Git Bash):
   `rsync -a --partial --progress "/mnt/c/Program Files/Guild Wars 2/" sbdevbox:/home/blizz/gw2-client/incoming/`
2. A USB drive or NAS share holding the same folder. Tell Devbox the path and we copy it.

## What we do when it arrives (no action needed from you)

- Record the build id and the SHA256 of `Gw2-64.exe` and `Gw2.dat` in
  `docs/client-build.md`, then `chmod -R a-w` the copy so it is frozen.
- Never let it reach ArenaNet. The launcher self-patches and repairs `Gw2.dat`, so it
  only ever runs with no network route out (firewalled), with `-authsrv`/memory
  patches pointing at our own server. It never logs in to your real account.
- Static reverse engineering (REA + Ghidra) needs no execution at all. That covers the
  key location, message layouts and the login flow.

## One question for Blizz

**Where may the client run for stages 3 and later (live login against our server)?**

- (a) On the devbox under Wine, firewalled (`wine-10.0` is installed). This box has no
  GPU and only virtual displays. GW2 needs DirectX 11, so this means software Vulkan
  (DXVK on lavapipe). It will be slow but may be enough to log in and load a map.
- (b) On a Windows PC or VM you provide, with its network limited to the devbox.
  You play from there.
- (c) Both: devbox for automated testing, your Windows machine for playtests.

**Answered 2026-10-31 (Blizz, relayed by Devbox): "wine is fine".** The client runs on
the devbox under Wine, firewalled. This covers the platform only. The install copy is
still needed.
