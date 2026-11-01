# Build 209026: portal requests after binary auth

Investigation 2026-11-01, following run 55. Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

After the ordered binary account/configuration/completion fixtures, the client
sends `/Auth2f/Check` with no body and `/Auth/GetUserInfo` with an empty Request
element. Both are unanswered in run 55; the launcher remains on its login spinner.
These observations concern the synthetic local identity only.

Auth's method table at `0x1421139e0` assigns GetUserInfo enum 11. Constructor
`0x140fffd10` selects it via `0x141006300`. Sender `0x140ff77f0` builds response
interface vtable `0x142111bc0`, whose success entry is `0x140ff6b30`. That entry
requires an XML node and forwards it to a caller-supplied interface, without
itself reading user fields. The Auth2f method table at `0x142112990` assigns
Check enum 1; its response callback remains to locate.

Next experiment: explicit empty Reply XML fixtures, retaining the required final
LF, and read-only probes recording XML field names/call sites. Empty replies are
parser probes, not established valid response bodies. No client instruction or
result will be changed for these fields.

| REA evidence | Observation |
| --- | --- |
| `ev_24428615ebbf0b46b940cbf193a4088799fb0e1adacfa6b0dde2da69bc4189c6` | Auth method table and GetUserInfo enum. |
| `ev_1917fd47863a85cdd1edf2b5a7dde076d6b0afb49b46bc632bfff84fda690fc0` | GetUserInfo constructor. |
| `ev_07564b4a3711b076249aea91ce88a067888e303e2f1bc9c6007b4a430e7ab57a` | Sender and response object. |
| `ev_d34588b1578c006faf2e411851fb36cbc8dcfe042637cd1c68c334d382c5bcb7` | Response vtable and Auth2f method table. |
| `ev_5235ccd7a16cc6cdd9de988fb16fee0017e745c0e5e8d33c84b4858f118d73a5` | GetUserInfo forwarding handler. |

## Empty replies and email-verification screen: run 56

Both empty Reply fixtures are consumed. XML lookup probes record AuthTypeName
then AuthType for Check. GetUserInfo forwards to `0x140247e70`, which requests
LastPasswordChange. That callback returns without further action when the field
is absent. When present, it also tests for DisableForcePasswordChange and
dispatches an event containing a parsed timestamp and that presence flag.

The screenshot now shows "VALIDATE YOUR E-MAIL ADDRESS" for the synthetic login
name, with Edit Account and Log Out controls. No Send action was performed.
This is a launcher transition, not character selection or a real account login.
No assertion probe fired. Isolation, cleanup, executable hashes and archive
count remain verified; multi-connection plaintext equality remains unverified.

`0x141001ec0` parses AuthTypeName through the table at `0x142112a00`; None maps
to zero, with SMS, Smartphone, Vasco, Email, EmailCode, Totp and MultiInput as
1 through 7. Missing/unknown text returns 8; absent text falls back to numeric
AuthType. The next local fixture will explicitly supply AuthTypeName None.

SSO reads EmailVerified at `0x140ffc4be` through `0x140e27530`. This helper reads
child text as a base-10 integer, defaulting to zero when absent. The completion
call receives its nonzero test. Therefore the verified synthetic identity fixture
needs `<EmailVerified>1</EmailVerified>`, not an empty element. This changes only
our local test response. No production identity or verification mechanism is used.

| REA evidence | Observation |
| --- | --- |
| `ev_f7e34d40f06a833a29e18430e0cfc24264eac57330e2eaabe67b91a0bb998ac4` | Run 56, field/callback observations. |
| `ev_5deaac475eb0f871cd22898c24b38338257c7bbc65720f625fdb42d6de2e750d` | GetUserInfo consumer. |
| `ev_9547de43670190494cb2304bd80f53b79d091903aab9dbf3fa15d64e91570c1c` | AuthType parser, table and user-info field names. |
| `ev_7774747d4bfe0480620ea47f42672a9882e04318ee52ca2cef3592bf2b3c8a4f` | SSO email-verification argument. |
| `ev_ef029ab86097a309fb87dab315f7caf4e8ed353ab9f28db258ec55ffde8c96c1` | None/EmailVerified literals and decimal reader. |

Private export: `~/re/gw2/notes/209026-post-auth-portal-run56.json` (run and
initial field analysis; subsequent email-field analysis will accompany run 57).

## Verified test identity: run 57

SSO's completion argument is now observed as 1. AuthTypeName None is consumed,
and the launcher advances to its optional 2FA signup reminder with a "Remind me
later" control. No signup action is taken. The next UI experiment will use that
existing reminder-dismissal control and inspect the launcher before Play.

Evidence: `ev_1fd0886bfb7476add758468646b0581bb714414ce4d8e3e901e897f96bd75dbf`,
private export `~/re/gw2/notes/209026-verified-fixture-run57.json`.
No assertion was observed; isolation, container cleanup, executable hashes and
archive count remain verified. Character selection has not been reached.

## Active Play button: run 58

Selecting the existing Remind me later control dismisses the optional signup
reminder. The launcher displays its news panel, Edit Account / Log Out controls,
100% playable state, and an active orange Play button. This establishes the
launcher login transition for the explicit synthetic local fixtures. It does
not establish credential validation, account persistence or character selection.

The shorter observation window retains one binary connection: all 233 decrypted
bytes exactly match the client's pre-encryption capture. No assertion is observed.
Network isolation, exact container cleanup, both executable hashes and archive
count remain verified. Evidence:
`ev_45df8de04911ffada7751aeb6d9beed2d1ffbfd23f864e4d6b00dad44d11cdb6`,
private export `~/re/gw2/notes/209026-launcher-play-run58.json`.
Next: select Play and capture the engine's next requests/UI in the same isolation.
