# Build 209026: species 3172 composite appearance inputs

Static REA investigation, 2026-11-05, against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Only the executable and previously extracted content were read. No archive,
client runtime, packet fixture or production code was changed.

**Species 3172 has an authored Norn-female composite token pair despite its
null direct model file.** The ordinary NPC model path obtains such a pair
from character `m_compositeData`, which server **377** can install before
finalization **170**. This establishes an actionable appearance input;
it does not establish Great Hunt membership, placement, or successful rendering.

## Exact retained content

The [readable-name audit](209026-species-plain-name-coverage.md) identifies
species **3172**, label `Eir`, at **1282845:7100312**, name ID **425678**
(English file **2441139**, entry **718**, offset **14821**).
This pass independently hash-checked the retained container:
`81028af31ccfaf10571e166eb09d49d5c75ce1e6f0eea7656be1cc9c80ad841e`.
The reader explicitly disables its subprocess/archive fallback.

| Species field | Value |
| --- | --- |
| `+0xf0`, direct model input | Null |
| `+0x120`, additional direct-model input | Null |
| `+0x90`, local array / `+0x98`, count | **1282845:7100928**, **1** |
| `+0x100`, flags | `0x30a80021`; no new flag meanings assigned |

The 32-byte entry at **7100928** contains zero at `+0` and `+8`,
**`0x0000ac0b4a6749ee`** at `+0x10`, and **`0x0000000001249650`** at
`+0x18`. These last two values are scalar tokens, not relocated pointers
or file IDs. Native token encoding uses five bits per alphabetic character
(`0x140e4ca90..0x140e4cb50`); their spellings are **`nornfemale`** and
**`preir`**. Independently, `0x1411c8c10` compares the first entry's `+0x10`
against this exact profile token at `0x1411c8d20` and assigns race **3**,
sex **1**. The character constructor calls that classifier at `0x1411d2ef5`.

## Which branch actually selects it

The [ordinary model builder](209026-npc-character-foundation.md),
`0x141166a80`, obtains the character and species through the view interfaces.
The constructor-established view `+8` table is **`0x1421492f0`**, with
`+0x168 -> 0x141107be0` and `+0x1d8 -> 0x141108840`; the latter reaches
the primary species getter. Whole-character primary table
`0x142160be0` has `+0x1d0 -> 0x141096ac0`, returning character **`+0x110`**.
The named assertion identifies this record as **`compositeData`**.

The selector has distinct paths:

| Condition | Inputs selected by `0x141166a80` |
| --- | --- |
| View override `+0xb50` is present and its species-array count is nonzero | `0x141108780` selects index **0**. Array stride is **32**; entry `+0x10/+0x18` supply profile/preset. Entry `+8` supplies the optional fifth composite-creation argument when the caller's argument is null. |
| No override predicate, character `compositeData` nonnull | Record `+8/+0x10` supply profile/preset. |
| Override predicate from character `+0x258`, with no indexed view override | The character's `+0x258` token replaces the preset; `compositeData +8` supplies the profile and a nonnull composite record is asserted. This token's producer is not traced here. |
| Selected profile token is zero | Direct species `+0xf0/+0x120` model branch. A null filename is replaced by the empty filename constant. |

The override predicate is `0x1411098f0`; character virtual `+0x2f8`
(`0x1411d87a0`) returns the scalar token at `+0x258`. Both character
`+0x110/+0x258` and initial view `+0xb50` are zeroed by their constructors.
Consequently, the ordinary no-override/no-composite state does **not** select
the species array merely because it has one entry. Species 3172 alone is
insufficient for this branch to acquire the authored composite pair.

With a nonzero selected profile, `0x141166d6a` calls **`0x1412d8570`**:
RCX is the profile token, RDX the preset token. Its constructor
**`0x1412d1b20`** stores these at composite **`+0xa40/+0xa48`**.
`0x1412d5bb0` then calls **`0x1412db100(profile, preset)`**, whose lookup
checks both tokens and returns the matched preset pointer at hash-node `+0x40`.
The preset consumer reads a component array at `+0x10`, count `+0xc`,
stride `0x1c`, plus packed color fields. A lookup failure enters a separate
fallback branch at `0x1412d5ebd`; merely reaching composite construction
does not prove that the intended preset was found.

## Server 377 supplies the ordinary composite record

Registration **`0x14216faf0`** binds descriptor **`0x1425c3480`** to handler
**`0x141253350`**. Its fields, in wire order, are:

| Field | Descriptor |
| --- | --- |
| 0 | Dword character index |
| 1 | String[128], passed to the existing name setter |
| 2 | Word flags; independently mapped in the [NPC talk note](209026-npc-talk-loop.md) |
| 3 | Optional **three Qwords**, nested descriptor `0x1425c3080` |
| 4 | Optional **Float, four Dwords, Float**, nested descriptor `0x1425c31c0` |

Optional kind **15** uses the existing one-byte presence representation;
see [native optional codec](209026-agent-client-messages.md). Parsed-message
`+0x10/+0x18` are pointers to decoded optionals, **not fixed wire offsets**.

When optional field 3 is present, `0x141253440..0x141253457` passes its
three Qwords to **`0x1411e0950(character, q0, q1, q2)`**. That function
asserts **`!IsFinalized()`** and **`!m_compositeData`**, allocates 24 bytes,
stores its address at character **`+0x110`**, and copies the Qwords to
record **`+0/+8/+0x10`**, respectively. There is no per-token validation
in this setter.

Thus the established appearance dependency is: create the species-3172
character, install one 377 composite record with **q1 =
`0x0000ac0b4a6749ee`**, **q2 = `0x0000000001249650`**, then finalize
with 170. The authored array's auxiliary input is zero, but that input has
not been linked to q0 and does not establish a safe q0 default. The ordinary builder
does not read compositeData `+0` in the traced selection path; its complete
role remains unresolved. Name, flag Word and optional field 4 must remain
explicit choices, since 377 also invokes their setters. This is not a
recommendation to send an otherwise invented all-zero metadata message.

Current `TalkNpcPlan209026::spawn_frames` creates direct-model fixtures
without 377. Adding only species 3172 to that plan would omit the dependency
identified above. This source investigation makes no implementation change.

## Limits and evidence

No model file, individual preset component assets, NPC placement or story
role was guessed. The [preset registration and asset follow-up](209026-eir-preset-assets.md)
resolves the exact `nornfemale/preir` registration and eight component references;
neither pass claims a runtime preset lookup or rendered Eir. Great Hunt
association remains separate from the established label and appearance pair.

Complete private bundle:
`~/re/gw2/notes/209026-eir-alternate-appearance-20261105.json`,
SHA-256 `78932a249f6805a5cc33ee621f6a03dc9ae50d5cca9e398bb6ddf881dc583830`.
It retains 28 process Evidence records, including bounded unsuccessful and
superseded probes. All completed with exit 0 and owned-process cleanup.
An early guessed table address was superseded by the constructor-proven
secondary table above; its values are not used as interface bindings.

| Evidence | Supported claim |
| --- | --- |
| `ev_88ebc0e12c2c53d65862bf377b1738d5a2070c47177fab1a88f770f89713c47f` | Retained species entry, exact scalar tokens, flags and assertion literals |
| `ev_b0acc5890044ca256520e8812d8e622af5d60a100f2c79a057bf0c207a802375` | Profile-token classifier produces race 3 / sex 1 |
| `ev_1003ccc203942f1419ebbec4c5eb116f0f07c80a21431f30a1c43107be99a5fd` | View constructor and actual secondary vtable |
| `ev_3cd40942eea9c7c619285c1f2b640f69099546b15b9411bb842a82e364fa158f` / `ev_bdf204a4cc0fe91ef3996b22976076b23cfb6dc15bb047fdec854899ccdf75f4` | Character getters and species thunk |
| `ev_5e9f9c242cb537df928bb9be91a3efd93a28147e951ab3e6b467c973e05a5ada` / `ev_3bc8eee895155d1db0e675ae74c2d7d4d353c9bd877cac74417898964de31106` | Override predicate, index selector and ordinary caller |
| `ev_edbf2a18ed7008556d493a6583a3a33c3a6cb32b32bfd5c65db995e5640155bf` | Complete model selector, retained in the earlier NPC foundation bundle |
| `ev_c2fe9c362f60aec918a0a46d157a73a20e89e6de1f7b94a5a67a5e2d16c15b2d` / `ev_d57a7827c06874f51c92e8b8952177d171738ed7d80ffae40957fe71ce8de6e8` / `ev_c60df009de9b6189fed6e4f1d95e164eb77b5f663bf46392923ef40a72b41186` | Composite token storage, exact-pair lookup and bounded preset consumer |
| `ev_977e2c3aad00561b1c9e3e1c7588df005095269735d695dd252da1a41e94df44` / `ev_4494fa6e6e7171c51aa25a31e0c9bd451c42299256e7c39a4dc5630abc242dfb` | Composite-record setter, ordering assertions and exact direct caller |
| `ev_81094d2aebcca1be24c75fad24f5b1ae3d7dbaa6cb734b296244745862af771d` / `ev_00758ac7a7a35ae4f90f4d061136a0a4eda5b184093656fd9b839bca090016ee` / `ev_c3210dcc596aecad167973efc28b6d09b101272a64739423b13aaeec96a1a9aa` | 377 registration, complete schema and handler; native token encoding |
