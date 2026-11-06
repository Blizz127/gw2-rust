# Build 209026: quest 154 text and localization keys

2026-11-05. The six step definitions and their localized text IDs are now
identified. Their text records require keys: the client decoder returns an
empty string when the key is zero. This provides a source-supported explanation
for the blank story text seen in the [Norn playback](209026-norn-mesmer-playback.md),
but this static trace did not inspect that run's live key dictionary.
The actual keys, decoded wording, NPCs and encounter conditions remain unresolved.

## Quest and step content

The [lifecycle trace](209026-quest-lifecycle.md) established type-57 quest
**154** at container **1282830:4145560**, associated type-68 key **2**, and
its sole goal **1** at **1282830:4146200**. Goal +0x28 directly references
type-45 map **379**. This is the basis for selecting this tutorial quest;
neither its numeric key nor its opaque metadata name supplies English wording.

The diagnostic consumer `0x1411fe650` routes the following fields through
localized-text helper `0x1412e38e0`, then `0x1410dbdd0` / `0x1410dbde0`:

| Definition field | Text ID | Confirming consumer |
| --- | ---: | --- |
| Quest +0x2c | **239267** | `0x1411fe6ad` |
| Active goal +0x40 | **34076** | `0x1411fe919–0x1411fe928` |
| Goal +0x44 | **34077** | `0x1411fe863` |
| Step +0x48 | See below | `0x1411ff23d–0x1411ff248` |

Goal +0x50/count +0x58 holds six type-343 step definitions, stride 0x60,
starting at **1282830:4146312**. Array order is not a recovered server
progression sequence.

| Array index | Step +0x14 ID | Definition +0x10 flags | Step +0x48 text ID | Step +0x38 sector reference |
| ---: | ---: | ---: | ---: | --- |
| 0 | 1 | 0 | **34078** | 201 / 1282846:3289648 |
| 1 | 2 | 2 | **34079** | 200 / 1282846:3233008 |
| 2 | 3 | 2 | **34080** | 201 / 1282846:3289648 |
| 3 | 6 | 2 | **34081** | Null |
| 4 | 4 | 2 | **34081** | 202 / 1282846:3179240 |
| 5 | 5 | 2 | **34082** | 202 / 1282846:3179240 |

The independently traced targets are type-63 **sector definitions**, not NPCs.
Their geography and tutorial-map membership are documented in
[entry geography](209026-norn-entry-geography.md) and
[sector polygons](209026-norn-sector-polygons.md).
All six steps have raw +0x28 value 17; its meaning remains unnamed. Fields
+0x18, +0x20, +0x30, +0x40, +0x50 and +0x58 are zero, and +0x38 is the
only classified relocation in each step record. This bounded inspection
establishes no NPC reference, kill count, trigger condition or boss script.

## Text manifest and exact entries

Language initialization `0x1413d0d80` passes the encoded filename at
`0x142261530` through `0x1410dc870` to loader `0x1410db630`. It identifies
archive file **110865**, a `PF/txtm` manifest. The manifest declares
**1024 strings per file** and six language slots; language slot 1 is empty.
For language slot 0, the relevant mappings are:

| Text ID | Archive file | Entry index | Record offset | Record bytes | Header word +2 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 239267 | 2440957 | 675 | 39237 | 19 | 0x47 |
| 34076 | 2440757 | 284 | 21408 | 45 | 0x48 |
| 34077 | 2440757 | 285 | 21453 | 288 | 0x47 |
| 34078 | 2440757 | 286 | 21741 | 29 | 0x47 |
| 34079 | 2440757 | 287 | 21770 | 76 | 0x48 |
| 34080 | 2440757 | 288 | 21846 | 40 | 0x49 |
| 34081 | 2440757 | 289 | 21886 | 34 | 0x41 |
| 34082 | 2440757 | 290 | 21920 | 34 | 0x42 |

The `strs` files contain size-prefixed records with six-byte headers.
Loader `0x1410dd76d–0x1410dd7e1` validates the size and bit-width field.
Every listed record has byte +4 equal to **7**, byte +5 zero, and nonzero
word +2. Retained payload bytes are encrypted data, not UTF-16 plaintext.
The manifest join and exact payloads are retained privately in
`quest154-text-manifest-20261105.json` and
`quest154-text-entries-20261105.json` under `~/re/gw2/notes/`.

## Exact empty-string path and unresolved key provenance

Decoder **`0x1410e0840`** receives a record pointer in R9 and a 64-bit key
as its fifth argument. At `0x1410e086d–0x1410e0875`, nonzero record word +2
combined with key **0** selects the short return path. It copies one UTF-16
word from **`0x14213bb88`**, whose inspected value is **0**. Thus these
records decode to an **empty string** without a nonzero key.

With a nonzero key, `0x140db0d60` expands/mixes its eight bytes into 20
bytes. This is a specialized derivation, not SHA-1. `0x140db0c10` creates
an RC4 object, identified by its `CptRc4.cpp` assertion, and the decoder
invokes it on the payload at `0x1410e0942`. It then unpacks the specified
bit width: zero terminates, codes 1–31 use the table at `0x14213baa0`, and
larger codes add the header's base minus 32. Without the correct keys,
this trace does not recover the quest's title or objective wording.

Caller **`0x1410e0b50`** obtains a candidate key from formatted source data
via `0x1410e50f0` at `0x1410e0bf1`; that extractor was not traced further.
If the key is zero, it looks up the text ID in the text-context dictionary
at **+0x200** (count +0x204, storage +0x208). Slots are 24 bytes, with
text ID at +0, key at +8 and occupancy/hash state at +0x10. The selected
key is passed to the decoder at `0x1410e0c6c`.

Two observed writer paths exist:

- Server **748** inserts or updates this exact dictionary, as detailed below.
- Local setter **`0x1410dc7a0(text_id, key)`** also inserts or updates it,
  ignoring text ID zero. A complete direct-E8 call scan of the executable's
  text section found five callers at `0x1413117b1`, `0x1413117f2`,
  `0x141311829`, `0x141311865` and `0x1413118a1`. Their bounded enclosing
  consumer reads optional 64-bit values from an input byte buffer according
  to flag bits 28 through 24 and passes nonzero values to this setter.
  This finds no embedded default keys for quest 154. It does not exhaust
  indirect calls or establish the outer input buffer's complete provenance.

The next source requirement is a valid provenance for keys corresponding
to the eight exact text IDs above. No default, guessed or generated keys
are supplied. Nor does this bounded result prove the keys are absent from
all shipped data or that they can only originate at a remote server.

## Server 747/748 wire contracts

The extracted descriptor group `table-0x14213c4f8` in
`data/protocol/209026/server-tables.xml` declares:

| Message | Wire body after u16 little-endian header | Handler and effect |
| ---: | --- | --- |
| **747** | Empty | `0x1410e7730` clears the dictionary's count, clears/frees its slot storage, resets storage pointer and capacity. |
| **748** | Byte count, at most **64** repetitions of **Dword text ID, Qword key** | `0x1410e7600` inserts/updates entries in the same dictionary. |

Dword uses the established unsigned 32-bit varint; Qword is eight bytes
little-endian. These are global text messages: **there is no player index**.
The 748 handler reads a decoded byte count at message +2 and an array
pointer at +3, advances 12 bytes per decoded record, and copies the
record's ID at +0 and key at +4 into dictionary slots. Those decoded
memory offsets are not wire offsets for variable-width Dwords.
An empty 748 list does not clear existing keys.

`gw2_proto::localization_keys::LocalizationKeysFrame209026` exposes
`Clear` and `Set { entries }`, whose entries are `LocalizationKey209026`
with explicit `text_id: u32` and `key: u64`. Both directions preserve zero
and full-width values, repeated IDs and wire order. Oversized encoding
fails before modifying the output; every decode error leaves input unchanged.
There are no default keys or bootstrap integration.

The focused tests cover literal frames, stream boundaries, every truncated
prefix, malformed Dwords at first/later entries, count 64, oversized lists,
duplicate IDs and independent decoding through the extracted client schema.
`cargo test -p gw2-proto --lib localization_keys::tests` passes all four
tests; the new module passes rustfmt and `git diff --check` is clean.

## Evidence and limits

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| Archive file | SHA-256 |
| ---: | --- |
| 110865 | `2e5350b743f20e981f0436f350c6422c69df07ba774391e64144fab8dc8b5243` |
| 2440757 | `87091fc25a797c13d1290468fce16b2a0b0cb9783014279b0f33717bfcd44d56` |
| 2440957 | `ef785b07c1a489063e0fd62e9f14d9ce36aa408ed220474ca19d6b66b00b1882` |

REA captured hash-bound read-only Python/objdump helpers, not the game.
Private bundle: `~/re/gw2/notes/209026-quest154-content-20261105.json`.
All cited final captures exited zero with verified owned-process cleanup.
Two earlier manifest probes exited one because their initial format
assumptions rejected a filename/empty language; the successful parse below
supersedes those attempts. No game client or live service was launched.
The shared ledger includes other agents' evidence; scope claims below
refer to these specific records.

| Evidence | Observation |
| --- | --- |
| `ev_38781654b454545ebff182fb280ca8f64e3d035f3abc51bc5a1f78d2367232c0` | Complete quest, goal and six step definition fields; also retained in the lifecycle bundle. |
| `ev_770c9603d0614ca6cd8c5ae8226a3b8d572c80091947d38557c02501f82e9a0c` | Source consumers identify the four localized-text fields. |
| `ev_e4982ab889375098fd7cb0729906869ba14793eb5a049001cc9e52cbe7f10356` | Language initialization's fixed manifest filename. |
| `ev_87e2b255f34fc168802aa61da787c5267ec2c467c6b168b8a71ea83a83acb8fd` | Successful manifest parse, text-ID mappings and hashes. |
| `ev_afe95fbf00eec4a812b13ea40430cc9e7999d34dd07986cf4822db68034e0b09` | Exact eight STRS records and file hashes. |
| `ev_3ff261b86a65552a97b875c4775216f059c3e2d89be48433e94783fa52200c03` | Decoder key gate, RC4 call and bit unpacking. |
| `ev_e389b53af368b45c16d86335d3550e6dd9fac2e9e8daae00472c86f4ee8bf818` | Key fallback dictionary and bounded derivation tail. |
| `ev_ce9bacb25b0595ae1171d255507db4c8dea03a04b73d3a8f594007acf1a90fe1` | Complete 748 handler and local setter. |
| `ev_0a6424e2d6b5d96b9dd49e366ae1b448a68e8a661be30b9c909b9d65fad1b7dc` | Complete 747 clear handler through return at `0x1410e77ac`. |
| `ev_bf6289c771c5c84774a0c8f84871e4f8b0f15230eced3bb8995cc904b168a339` | Five direct callers of the local setter. |
| `ev_1f2db8d13f1de9ee9f007d4da6069115cfd771dcda42b1057c2eb4918f1e4703` | NUL placeholder bytes and caller inputs; no recovered quest key. |

## Completion reward and fixed claim request

The exact quest154 record has nine40-byte reward rows at+68/count+70.
Reward.cpp obtains the character's coreStats profession through virtual28
and filters row+10 against it. The Mesmer7 row selects one fixed item,
type35 key32138 at1282847:2575512. Existing retained localization names it
Wooden Rod; this follow-up reverified its reference/type/key, without a new
text decode. Row kind0 populates fixed-item child4. All nine rows are kind0;
none creates choice child5. Optional row filters are null. This applies to
the inspected normal panel path; separate player/quest-manager mode gates
remain explicit.

Quest+5c=72 reaches the native coin display unchanged, including its
100/10000 denomination split and retained Coins label. This supports72
copper displayed alongside the item. It does not credit a wallet. The XP
path uses quest+54=2 and+58=0 with current experience, level-table difference,
mode-specific multiplier and a rounding helper; no fixed XP award is derived.

For a fresh fixed-only panel, ReRewards140854ab0 obtains no choice child and
forwards selected item key0 through its listener to Reward callback1408313f0.
That callback calls quest virtual68,1411fe150, which asserts quest completion
and forwards to sender141263be0. The sender constructs **client205**, one
Dword optional selected-item key, descriptor1425ce300. Its zero form encodes
as `cd0000` under the established framing. Neither quest154 nor fixed
item32138 should be substituted into that field. Nonzero input is not fully
validated by this client path; server validation is still required.

The local submitted bit suppresses repeated UI clicks, not server replays.
Current reviewed0570b950 displays completion but has no claim205 handler or
item/wallet credit. Inventory capacity, successful-credit acknowledgement,
duplicate claims, transfer state and XP policy remain implementation work.
No gameplay, store or runtime was changed by this source investigation.

Private findings: `209026-quest154-reward-claim-20261106.json`, SHA-256
`fd9b92886ad279c65f3739d4a420f50012c534bc46ae7ee6a62ac36b70df7dea`.
Complete REA bundle: `209026-quest154-reward-claim-evidence-20261106.json`,
SHA-256 `c75053c36d78cec2aa5cd9ac573960d74bc2df21721552fe78cc33f68fec830e`.
One exploratory widget-prefix capture omitted unrelated dispatch bytes and
is not complete-body proof; the selected typed paths are retained separately.
