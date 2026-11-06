# Build 209026: damage-tooltip base multiplier setting

Static REA, 2026-11-06. The exact setting requested by the damage-fact
formatter is the literal floating-point value **1.0**. This supplies one
input to the client tooltip estimate; it does not establish server hit damage,
target mitigation, critical-hit policy, or effect timing. No gameplay code,
client runtime, working archive, shared container, or publisher state changed.

## Exact consumer and record

The retained formatter capture binds the lookup to
`0x140893c50..0x140893f4d`. At `0x140893e68..0x140893e80` it constructs
the GUID from four Dwords `cdd051a5`, `47394d0b`, `13fd68b6`, `16107d34`,
then calls the content context's virtual **+0x68** at `0x140893e88`.
The exact 16 bytes in memory are:

```text
a5 51 d0 cd 0b 4d 39 47 b6 68 fd 13 34 7d 10 16
```

The returned pointer is named by the `baseDamageMultiplierConfig` assertion.
At `0x140893eac`, the formatter checks its Dword **+0x28** against
**437 / 0x1b5**; the corresponding assertion names
`Content::CONTENT_TYPE_NUMBER`. At `0x140893ece..0x140893ed2` it passes
the value object at **+0x30** to float accessor **0x1412e3c90**, before
multiplication and the formatter's existing clamp/rounding path. The accessor's
literal-float contract is also recorded in the
[amount-fact source note](209026-mesmer-tooltip-facts.md).

A prior retained source note located the exact record at
**container 1282832, content offset 341728**. The new inspection verifies that
location independently against the container's index and the entire GUID:

| Field | Verified value |
| --- | --- |
| Indexed record type and record +0x10 | **150**, setting |
| Index namespace | **1492** |
| Next indexed record / available extent | **341856 / 128 bytes** |
| Declared value type, Dword +0x28 | **437 / 0x1b5** |
| Value-object tag, Dword +0x30 | **437 / 0x1b5** |
| Literal float, setting +0x38 = value object +8 | bits **0x3f800000**, value **1.0** |

The three tag/value fields are not declared relocations. The type-150 record
is distinct from its declared number type and nested number-value tag; the
literal is at **+0x38**, not directly at +0x30. No missing/wrong-type fallback
is needed for this exact record.

## Frozen provenance and bounds

The prior location came from private
`~/re/gw2/notes/swarm-20261105/damage-loot.md`; it was a lead, not the value
oracle. A bounded names-only cache search found no extracted 1282832 file.
The retained numeric-content source manifest supplied expected SHA-256
**63bd50638f1386c9d92ecdecf9c9260e02500d1d9ac616e33883577033c387de**.

The authorized one-file extraction used the immutable frozen archive and
the same [frozen-versus-working resource distinction](209026-quest154-authored-start.md)
as the preceding story follow-up. It pinned extractor SHA-256
**ecff83ff81bcff68d418698f98363688c665401a4b0a4307658b07ce86dc812b**
and canonical frozen archive device **2065**, inode **293163139**, link count
**1**, size **89165623112**, mode **0555**, modification time
**1793470205585736518 ns**. Those fields and the extractor hash were unchanged
afterward. There was no full archive hash, other content extraction, or working
archive stat/read.

The **9766548-byte** decoded file matched the retained SHA-256 before exclusive
private caching at `~/re/gw2/notes/content-type-150-file-1282832.pf`, mode 0600.
The exact-record reader then disabled archive fallback, checked the indexed
extent, compared all 16 GUID bytes, required type 150 and both 437 tags, and
checked the literal bits/value. No other setting search was necessary.

Private helpers are `extract-frozen1282832-damage-setting-20261106.py` and
`read-damage-multiplier-setting-20261106.py`, under `~/re/gw2/notes`.
Complete new Evidence plus the retained formatter capture is preserved in
`~/re/gw2/notes/209026-damage-tooltip-number-setting-20261106.json`,
**58666 bytes**, SHA-256
**046554d29bc8604f2a53746c6e5d9b9a54cefe6e32998a18c581c1459e6efca6**.
Both new helper captures exited zero with complete raw output and verified
cleanup of their owned process groups. No client execution is claimed.

| REA evidence | Observation |
| --- | --- |
| `ev_af698ab39642ab21134c04f5f95498e027ebead86751a38e84cd2c404c3ba3c7` | Retained formatter: exact GUID, named assertion, +0x28 number-type guard and +0x30 float-value consumer. |
| `ev_cd62e6c1e91fb730746772e1d06e82e3a24de8ae40780b9362c2e0f3da2590f9` | Pinned single frozen extraction and matching decoded container hash. |
| `ev_90038b698c1b1e2a629cb1f25f91c25faf2eb0678d7f48ca59014b885ae4fedd` | Exact indexed setting record, GUID, tags, nonrelocated literal and value 1.0. |
