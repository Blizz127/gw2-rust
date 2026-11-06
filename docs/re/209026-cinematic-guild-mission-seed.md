# Build209026: “Cannot play the cinematic” is a Guild Mission UI path

The exact diagnostic at **0x141ad3cd8** belongs to the ArenaWeb method
**`showMapCinematic`** of **`WuModuleGuildMission`**. It is not a demonstrated
creation/race/biography intro selector. The method uses a cinematic reference
attached to a selected guild-mission target; this slice recovers no numeric
cinematic ID or Norn-specific selection rule.

Static REA, 2026-11-06, frozen executable SHA256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No runtime, archive, publisher state, shared status file or server behavior
was accessed or changed.

## Exact diagnostic and owning method

The exact RIP-relative REX.W LEA query finds one validated reference:
**0x140351e30**. Its `.pdata` entry covers only an interior fragment; the
complete logical method is **0x140351c50..0x140351e93**. Earlier checks have
the diagnostics `Invalid categoryId`, `Invalid slotID` and `Invalid targetID`.

An exact function-address LEA query binds this method at **0x140352a6d**
inside registration **0x140352820**. The latter builds an ArenaWeb callable
holding the function pointer and current module object, then registers it
under the exact UTF16 name at **0x141ad3a68**, `showMapCinematic`. Its two
neighboring registrations are:

| Registered name | Method |
| --- | --- |
| `initData` |0x140351c00, a tail jump to0x140352bf0 |
| `launchMission` |0x140351c10 |
| `showMapCinematic` |0x140351c50 |

The owner is established through construction, not neighboring names alone.
Complete allocation/constructor **0x140354a90..0x140354ba3** installs primary
table **0x141ad35b0**. Its **+0x28** entry is registration0x140352820. The
same table's +0 refcount method **0x140351ea0..0x140351ee7** names
`Game/Ui/Services/WebUi/Modules/WuModuleGuildMission.h` in its assertion.
Destructor0x140351690 independently reinstalls that primary table.

An exploratory neighboring pointer table at0x141ad3588 instead belongs to
an ArenaWeb callable wrapper. It is not the module's primary table and is
not used as owner identity evidence here.

## Selected-target path and failure condition

At method entry, RCX is the module, EDX is the slot index, R8D is the target
index and R9 points to the response interface. The method:

1. Bounds-checks the module's current category index **+0x68** against
   **+0x5c**, then selects a32-byte category row from **+0x50**.
2. Bounds-checks the slot index against category **+0x14**, then selects
   a96-byte slot row from category **+8**.
3. Bounds-checks the target index against slot **+0x54**. Helper
   **0x1403515a0** indexes the slot's embedded collection at **+0x40**,
   whose entries are16 bytes. It loads the selected entry's first pointer,
   then the reference at that pointed object's **+0x40**. The underlying
   content type and key are not resolved by this slice.
4. Getter **0x1413f6970..0x1413f6984** returns game-context **+0x1b0**.
   If the selected reference is nonnull, the method calls that interface's
   **+0x60** predicate with the reference. On a nonzero result, it reports
   success through the response interface and calls context virtual
   **+0xb8** with that same reference.
5. A null selected reference or zero predicate result produces the exact
   `Cannot play the cinematic` response. No other reason is assigned to
   that predicate without its own concrete implementation trace.

This establishes a guild-mission selection/playback request path, not a
debug-console command or the automatic creator intro. The shared cinematic
context may serve other callers; no absence of creator support elsewhere is
claimed. In particular, this branch does not justify using any guild-mission
reference for the Norn intro. The targeted seed is now classified and no
broader filename, field-offset or content census was performed.

## Evidence

Complete13-record private bundle:
`/home/blizz/re/gw2/notes/209026-cinematic-guild-mission-seed-20261106.json`,
SHA256 `35ad6d83bfd034275e1740e5ffa96595bd4a628f5e8be9c5d719f313d11484ff`.
All captures exited0, were nontruncated and verified cleanup of their owned
process groups. Process environments whose ownership could not be verified
remain a limitation; this is not host-wide cleanup. Initial partial/unrelated
windows remain retained but are not used as complete-body evidence.

| Evidence | Scope |
| --- | --- |
| `ev_ffd71315ba7c38f4d5b0118170d00eed81bf388d2baee3ccf94ade4bc27cf265` | Exact diagnostic bytes and its one validated LEA reference |
| `ev_3e5dfc658f7ca7031ba2c390bf8c06668d83797d1ea56601f8795318d2474304` | Complete selected-target cinematic method |
| `ev_70ace96b6465d52c5159ad336fcc1903bb36224e8550a155a6720c2af2650a05`, `ev_b475b4df986c29aeb2bb891932a9586efe6f1dcd0957495455b0e9bd5b5f75b4` | Method-pointer registration and context getter |
| `ev_5d50430e06d0e6720ff35687f2ed089be0111789bd9cfd11c76232a0b956285c` | Exact registered UTF16 names |
| `ev_2e842fbbe59f9cc18a05b78753a18fb2760bf80d22e30ba6da709d8c483e38ec` | Complete16-byte target-entry accessor |
| `ev_f1e7e9236e867ecccc0273350ac34a92aab65e71a4b984c5f22c508ef942891b`, `ev_aab9bc310e59ba27517b82b725d16da4a4d91b48cb7281327ba0d959ad33f8eb` | Actual owner-table references, complete constructor and named module refcount method |

Unique seed helper `norn-intro-cannot-play-seed-xrefs-20261106.py`, SHA256
`bb2545925b9b48f8d45105ce9f262ada7e42a194f33e46b715c890aa55a48f8e`;
exact-name helper `norn-intro-cinematic-method-names-20261106.py`, SHA256
`6c3e581b17735708345890e5e8973fb6e3cf9f99f0a5ea2ba75fdd2b5885632d`.
The exact-reference searches retain their encoding-specific coverage limits;
none is presented as an exhaustive caller or cinematic census.
