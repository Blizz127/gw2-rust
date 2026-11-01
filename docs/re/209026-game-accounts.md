# Build 209026: ListMyGameAccounts

Investigation 2026-11-01, following run 45's encrypted account query. Frozen
executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

## REA static observations

Account enum 4 is ListMyGameAccounts. Constructor `0x140fffdc0` selects it via
`0x1410088f0` and writes the optional GameCode child. Its caller at
`0x140ff9c70` builds the response callback object with interface vtable
`0x142111950`; that table binds success handler `0x140ff7370`.

That handler obtains the reply XML node and iterates **all direct children**
(null name filter). Every child must contain an **Alias child element**; a
missing Alias takes the error path. The Alias text is converted to UTF-16 in
a 64-code-unit buffer. Empty strings are skipped. Nonempty aliases are copied
with individual terminators into a list, followed by an extra NUL. The caller's
callback at vtable offset `0x20` receives that list.

This callback does not read a game-account GUID, UserName or DataCenter. Helpers
with those field names were found in shared/social code, but following the
Account enum sender and bound response callback establishes the relevant path.

**Derived local fixture to test:** an LF-terminated Reply with one child entry
containing Alias `BansheeRealm`. The entry tag `GameAccount` is our readable
fixture choice; this handler's iteration does not check that tag. Observe the
list callback and subsequent request before claiming selection or game access.

## Evidence

Raw bundle: `~/re/gw2/notes/209026-game-accounts-static.json`. Bounded byte reads
and GNU objdump windows through REA Rizin; whole-text reference filters locate
callers, then bounded windows establish behavior. Executable hashes unchanged.

| Evidence ID | Observation |
| --- | --- |
| `ev_5fdb4a2fb2f1cf8ccbc3285b5f3f82d98746532092d948c92307d6ffd8f1713c` | Account enum table pairs ListMyGameAccounts with 4. |
| `ev_163e5161923146ff26040fb4c7b0bbcf34b95d880f5b3f30ac188c7a365c4d06` | Enum-table registration; earlier field consumers belong to social data. |
| `ev_07e7e1260e8b541da8048778a699bded693c976e00e252580f74beb859364d0d` | Account method-name setter references registered table. |
| `ev_76a0285fe3b118a5de3e3846f0e8343d4124b634e17d8bfd80698ad4484c7cd1` | Enum-4 sender calls that setter. |
| `ev_c57d0f205935649a4d20672591f85a33ed40146cc495cadd315baeb4bb1f602c` | Request constructor and submitting caller. |
| `ev_2ad31fe22cdeda5615f8e37d62d8893e0158eb74c5ea60feeb17585244af1955` | Callback object/interface construction. |
| `ev_98fc20219cd04bcf0a85168dcff6c091b1b1ddc1f643d9f6c72ddfa791980595` | Interface vtable and Alias literal. |
| `ev_f47da86e4279e60e6570d08db42950b4087babc4bac98cb229acfdcc4411b3f6` | Complete success handler and alias-list construction. |

## Run 46: alias accepted, game token requested

The private reply fixture is
`<Reply><GameAccount><Alias>BansheeRealm</Alias></GameAccount></Reply>` followed
by LF. The client reaches the nonempty-alias branch and the list completion
callback. Its next request, on the same TLS connection, is:

```text
P /Auth/RequestGameToken STS/1.0
l:88
s:5;timeout=29000
```

The Request contains GameCode (3 characters) and AccountAlias (12 characters).
No token reply has been sent yet; game connection and character selection remain
unverified. No Rust change was needed for this experiment: it uses the already
checked `--reply-xml` fixture path.

REA Evidence
`ev_86aa828a5425bf65454c678ca75f6a0738b5499e67b99793238defc1aa0cf106`
is exported in `~/re/gw2/notes/209026-game-accounts-run46.json`. Raw run files:
`~/gw2-sandbox/runs/run46-codex-game-accounts/`. The wrapper exits 0, removes
its exact container, confirms unchanged executable hashes and 1,124,232 archive
entries. REA verifies owned-group cleanup with the previously documented host
process-survey limitation.
