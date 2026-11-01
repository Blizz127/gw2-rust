# Build 209026: GetHostname reply

Investigation 2026-11-01, following the first encrypted request captured in
[209026-login-tls.md](209026-login-tls.md). Frozen executable SHA-256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All client inspection below went through REA; raw evidence remains private.

## Static observations

- The authentication name table begins at `0x1421139e0`. Its entry at
  `0x142113a70` pairs `GetHostname` with enum 9.
- The sender at `0x140ffe2e0` selects enum 9, writes `LoginName`, and installs
  callback `0x140ffbb10` before submitting its request. This links the callback
  to GetHostname rather than relying solely on nearby strings.
- On a successful transport result, that callback obtains the reply's XML node,
  looks up `Local`, and records whether the lookup returned a node. It then
  retrieves the text of `Hostname`. A missing Hostname takes the error path.
  `Local` is tested by node presence, not by parsing its text as a boolean.
- The hostname is saved at object offset `0x8b8` and passed with the Local flag
  to `0x140ffac10`. That helper compares it to the current hostname and returns
  zero without replacing the connection when Local is present and the names
  compare equal. Otherwise it may replace the connection. The callback continues
  via `0x140fff5a0` on zero, or restarts its connection workflow on nonzero.

**Derived next experiment:** answer sequence 2 with `200 OK` and
`<Reply><Hostname>cligate.101.ncplatform.net.</Hostname><Local/></Reply>`.
The Hostname and Local fields follow the parser. The Reply wrapper follows the
existing STS convention; acceptance and the next request still require runtime
confirmation. No authentication success, ticket or character response is invented.

## Evidence

Private bundles: `~/re/gw2/notes/209026-gethostname-static-{1,2}.json`.
Commands report the executable unchanged. Bounded GNU objdump windows were
invoked through REA Rizin; a whole-text objdump reference filter located the
Hostname parser, then bounded windows established its behavior.

| Evidence ID | Observation |
| --- | --- |
| `ev_d7dc54fb0b594724ca67aeaf6c5f8b114f3b32010134d00f4869349a526245ce` | GetHostname string search. |
| `ev_8402037b89cf8bfc2e4e8b39a41fe3927a7fc636d119b252ba67e93eccd83473` | Pointer to the string in the authentication table. |
| `ev_330f4f6cc98b1d0cd09149f63c42534485856e05d2bbdd6ff550cfbe2b93761a` | Table entry with enum 9. |
| `ev_92a5619f978b8ae0b4be2a95fe78564a912f39b743b3beb77e80af18861c74f8` | References to Hostname and table registration. |
| `ev_69f2b9390f6851f45ba4b024a6d594aaa971ba1f9cd9adc0862eb4c23713787e` | Complete reply callback. |
| `ev_ab8cf89cdb6907b4a5cf3bbc41f5426b6f2b99b5ef33b9cc21cb8e445c4751d2` | Local/Hostname strings, callback reference, connection helper. |
| `ev_5b50e87e1719885a70d35472e24539801d071a7a214f6a6dba04bc0e393801df` | Enum-9 sender binds callback; XML lookup helpers. |

One exploratory `psz` command is unsupported by this Rizin version and supplies
no result. A wide `/r` search completed but exceeded the MCP response budget;
its retained Evidence `ev_5d4234fd1017203c4d7ef74fc83a5d12cb09d49bfef6b9a0c796558d7e247452`
was exported and read locally. It showed no code reference to the string itself;
the pointer table and callback linkage above resolve that indirection.
