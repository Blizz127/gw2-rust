# Build 209026: StartSsoLogin continuation

Investigation 2026-11-01. The preceding runtime request is documented in
[209026-gethostname.md](209026-gethostname.md), run 44. Frozen executable
SHA-256: `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

## Static observations through REA

The sender `0x140fff5a0` selects authentication enum `0x16` (StartSsoLogin),
constructs the request and binds callback `0x140ffbcb0`. Its default provider
string at `0x142111578` is `Portal`. The callback is shared with subsequent
SSO authorization waits.

On a successful transport status, the callback obtains the XML node and has
these branches, in order:

- A nonempty `AuthPromptHeader` leads to an auth-prompt UI callback and optional
  `AuthPromptInput` children (`Message` and parsed `InputId`).
- Nonempty `KeyData` with a pending key-exchange object invokes that flow.
- `AuthType`, when present, is converted to an enum. Some values display prompts;
  others require `RequestId` and send enum `0x19` (WaitForSsoAuthz) with that ID.
- Without those branches, **UserId is required** and must parse through
  `0x140de5190` into a 16-byte GUID. Missing or invalid UserId takes the error path.
  Optional `LoginName` and `Provider` replace the saved strings; `ResumeToken`
  is optionally parsed as another GUID.
- `PasswordToken` is decoded if present. With no resulting token, the saved
  password-token-version field is set to 3 and the path skips token upgrade.
  An optional `EmailVerified` value is read before the completion callback.

The completion callback receives the parsed user GUID, saved login name,
provider, email-verification flag and hostname. The client then records the GUID,
continues its connection work, and clears the pending callback pointer. This
describes client behavior, not an implemented account system or successful login.

**Next local experiment:** a synthetic, LF-terminated Reply containing a new
test UserId, the synthetic login name, and Portal provider, without prompt or
token fields. The local fixture grants no access to ArenaNet. Observe the
callback and next request before claiming the continuation works. Never echo
the captured Password field or copy real credential caches.

## Evidence

Raw records: `~/re/gw2/notes/209026-startsso-static.json`. Bounded GNU objdump
and byte reads were invoked through REA Rizin against the pinned target; hashes
remain unchanged.

| Evidence ID | Observation |
| --- | --- |
| `ev_c8941f0f1a98059058579608fb1c1e6501efb0940fb68a3dd6f90d53ed5ba83a` | Enum/name strings; sender and callback binding. |
| `ev_ce067e8879b9d888b46c4a407b771f23aff4d38fab63b7b728fe9b31bd26e41e` | Success-status callback branches and field-name bytes. |
| `ev_1754497148b523b2b62dd5e1ee4d0f047c7e20b9edb68b52ff8d72b989cac3d0` | Default Portal provider and GUID decoder. |

## Run 45: synthetic SSO reply advances to game accounts

The observer now supports explicit `--reply-xml /PATH=FILE` fixtures inside TLS.
Files are limited to the STS maximum body size, UTF-8 and final LF; duplicate
paths and fixtures without a TLS identity are rejected. These opt-in fixtures
do **not** implement credential validation or an account database.

The private StartSsoLogin fixture contains a new synthetic UserId GUID, the test
login name and Portal provider. No prompt, resume or password-token field is sent.
The client GUID parser returns 1 and the completion call at `0x140ffc4ee` is
reached. The next captured request is:

```text
P /Account/ListMyGameAccounts STS/1.0
l:46
s:4;timeout=29000
```

Its Request contains `GameCode` (3 characters). Values of captured login fields,
identifiers and passwords remain private. This verifies the local fixture's
SSO continuation; it does not prove game account selection or a game connection.

REA Evidence
`ev_a7d226ac06bb3a0eeb465ffcab8d8d3826c0bab6197501e5b93be1255935046d`
is exported to `~/re/gw2/notes/209026-startsso-run45.json`. Private run files are
under `~/gw2-sandbox/runs/run45-codex-startsso/`. The wrapper exits 0, removes
its exact container, verifies unchanged executable hashes and reads 1,124,232
archive entries. REA verifies owned-group cleanup with the earlier documented
host process-survey limitation. Build, focused Clippy, diff check and the extended
TLS smoke (GetHostname plus a file-backed XML reply with sequence matching) pass.

Next: map ListMyGameAccounts and continue with a synthetic local game account.
