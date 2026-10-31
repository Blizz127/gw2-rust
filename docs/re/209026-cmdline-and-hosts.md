# Build 209026: command-line switches and server hosts (REA)

Target: `Gw2-64.new.exe`, sha256 `99342078…8d621`.
- REA Ghidra import: `ev_9fdc55572b52f948…` (73,285 functions, 53,317 defined strings, 2,518 s).
- Raw outputs (private): `~/re/gw2/notes/cmdline-table-209026.txt` and the `gw2-64-99342078ef57.*` files.

## Observed

**Hosts and auth strings** (`search_strings`, `ev_d635d76ec9e4afce8b0a79ca95a28ef18d309e0ed77b5b06a452ed16740e2755`):

| Address | String |
| --- | --- |
| `0x14230b490` | `cligate.101.ncplatform.net` |
| `0x14230b2b0` | `cligate.102.ncplatform.net` |
| `0x14230b3a0` | `cligate.103.ncplatform.net` |
| `0x14230b7a0` | `cligate.112.ncplatform.net` |
| `0x1420a6bf0` | `assetcdn.%u.arenanetworks.com` |
| `0x141b81680` | `.ncplatform.net/` |
| `0x141b94898` | `bug.arenanetworks.com:6112` |
| `0x142121828` | `Diag.ArenaNetworks.com` |
| `0x1421218f0` | `ICMP.ArenaNetworks.com` |
| `0x1422eaba8` | `bugreports.ncplatform.net` |
| `0x1419243e0` | `AuthSrv (%S)` |
| `0x1419243f0` | `AuthSrv invalid (%S)` |
| `0x141924178` | `Gc::AuthSrvEncryptCallback` |
| `0x141923ec0` | `Gc::RecvAuthSrv_ForceLogout` |
| `0x14213fdf8` | `sizeof(context->authSrvConfigFlags) >= configData.bytes` |

**Command-line switch table:** UTF-16 strings in `.rdata` at `0x14213dcxx`–`0x14213e2ff`, read
with REA `read_bytes`:
- `ev_4cf3c4335016c05681ed609d75898134d592d1586ad805a43f2608c0dc449ca5` (0x14213dc00, 1024 bytes)
- `ev_923208879a6a74dd345418234723605d42510b65a266e46862c2bda76231c813` (0x14213e000, 768 bytes)

Ghidra had not defined most of these as strings. That is why `authsrv` did not show up in the
string search.

```
allowinstall analyze analyzeFull assetsrv audio authsrv autologin defaultcharname dvdInstall
dx9single forwardrenderer yotd chunkedpatching cinema clientPort copydat cuda diag exit
framework fps image isrelaunch localdat mapviewdist mce maploadinfo modelsnapshotfix multi
mumble nodelta nomusic nopatchui nopatch nosound novoice noui patchconnections portal
portalalias prefreset shareArchive repair token tokenpassword uispanallmonitors umbra
uninstall uninstallSilent usecoherent useOldFov userid verify webdisablecache windowed dx9
dx11 partner steamPath ignorecoherentgpucrash AUTH_LOGIN AUTH_PASSWORD AUTH_TYPE epicapp
EpicPortal epicenv epicusername epicuserid epicloca…
```

**Runtime (non-REA), docs/re/209026-runtime-net.md:** with no arguments, the client resolves
`assetcdn.101.ArenaNetworks.com` and `cligate.101.ncplatform.net`, and connects to cligate
on TCP 80.

## Inferred (needs xrefs and decompilation of the parser to confirm)

- **`-authsrv <host>` still exists** as a switch name. Together with the `AuthSrv (%S)`
  and `AuthSrv invalid (%S)` messages, it most likely overrides the auth/portal host, i.e.
  the cligate default. `-assetsrv <host>` probably overrides the asset CDN.
  `-nopatch`/`-nopatchui` probably suppress patching.
- `-portal`/`-portalalias`, `-token`/`-tokenpassword`, `-userid` and `-autologin`, plus
  the `AUTH_LOGIN`/`AUTH_PASSWORD`/`AUTH_TYPE` names (probably environment variables or
  launcher-handoff keys), are the candidate credential handoff for the Banshee ticket.
  A token plus token password is the closest match to a one-use ticket.
- `-clientPort` probably selects the client→server port (the default 80 was observed at runtime).
- The `101/102/103/112` suffix is a portal/region index, shared with `assetcdn.%u`.

## Unknown

- The exact argument syntax (`-authsrv host[:port]`?), and whether these switches are
  honoured in a retail build or gated behind a development flag.
- How `token`/`tokenpassword`/`AUTH_*` enter the login exchange.
- Next REA step: find the code that references the switch table, i.e. the parser, then
  its consumers for `authsrv`, `clientPort`, `token` and `tokenpassword`.
