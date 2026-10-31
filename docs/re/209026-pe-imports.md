# Gw2-64.new.exe (build 209026): PE imports

Target: `/home/blizz/gw2-client/build-209026/Gw2-64.new.exe`, sha256 `99342078ef57…8d621`.

REA `inspect_pe_imports` → Evidence **`ev_2b7cf3c4c163f80df3b5ab0990941a17dec65b11b0f21ec6f2930087d6c8edd6`**
(confidence: observed). Raw output: `~/re/gw2/notes/pe-imports-209026.json`.

## Observed

The client has 20 import DLLs and 4 delay-load DLLs.

**Delay-loaded:**
- `bink2w64.dll`: 22 functions.
- `libcef.dll`: 36 functions, including `cef_api_hash`.
- `EOSSDK-Win64-Shipping.dll`: 9 functions.
- `steam_api64.dll`: 7 functions: `SteamInternal_SteamAPI_Init`,
  `SteamInternal_ContextInit`, `SteamInternal_FindOrCreateUserInterface`,
  `SteamAPI_GetHSteamUser`, `SteamAPI_RegisterCallback`, `SteamAPI_UnregisterCallback` and
  `SteamAPI_RunCallbacks`.

**Networking:** `WS2_32.dll` (40 functions, including `WSASocketA/W`, `WSAIoctl`, `getaddrinfo`,
`GetAddrInfoExW`).

**Crypto and security:**
- `ADVAPI32.dll`: `CryptAcquireContextA`, `CryptGenRandom`, `CryptReleaseContext`.
- `CRYPT32.dll`: `CryptProtectData`, `CryptUnprotectData`.
- `Secur32.dll`: 7 functions.
- `WINTRUST.dll`: 1 function.

## Inferred (not yet traced)

- Steam (`steam_api64`) and Epic (`EOSSDK`) are **delay-loaded**, so a launch with no
  provider probably never loads them. That fits the plan to launch with no Steam or Epic
  client and redirect the login to our server.
- `Secur32` (SSPI/SChannel) is a candidate for the TLS layer the portal uses after
  `/Auth/StartTls`. Its function list still needs checking, and the use needs tracing.
- `CryptProtectData`/`CryptUnprotectData` most likely protect `Local.dat` (the saved login).
  That is irrelevant to our server and does not touch any real account data.
- No OpenSSL-style imports. The crypto for the game connection (DH, RC4, SHA-1) is most
  likely compiled in, which matches the public notes and ascalon.

## Unknown

- Whether the "provider" path (Steam, Epic, or ArenaNet portal) is chosen by a command-line
  flag, and which flags exist in this build. That is REA probe 4/5 in STATUS.md.
