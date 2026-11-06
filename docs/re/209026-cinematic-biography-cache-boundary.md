# Build 209026: cinematic cache and biography selection boundary

The [cinematic start contract](209026-cinematic-start-contract.md) establishes
type 7 `CinematicDef` lookup and its +0x30 filename consumer. Existing private
decoded caches contain a partial catalog of numbered definitions, but
this slice does **not** identify the Norn intro's key or a race/biography rule
that selects it. No cinematic is selected from a filename or numeric ordering.

## Cache coverage

The reader admitted only existing `content-type-*-file-*.pf` caches, verified
each against the retained build-209026 hash inventory, and disabled archive
fallback before reading any content. It inspected type-7 entries in each
admitted container's content index, rather than searching every record body.

There are **1,366 type-7 indexed records in 28 cached containers**. Containers
1282837, 1282838, 1282840 and 1282849 were absent from this cache naming family
and were not read. This is not a complete 32-container cinematic census.
The retained type metadata places the numeric data key at record +0x28.

Eight sampled records have a classified file-reference fixup at +0x30. Using
the source-established [shared file table](209026-content-file-references.md)
from container 1282830 resolves, for example:

| Container | Content offset | Numeric key | +0x30 file-table index | Encoded file ID |
| --- | --- | --- | --- | --- |
| 1282830 | 6997976 | 1298 | 3395 | 2586695 |
| 1282830 | 6998128 | 1299 | 3388 | 2586686 |
| 1282830 | 6998280 | 1301 | 3396 | 2586696 |

All three use decoder suffix byte zero. These are structural examples only;
none is established as a Norn cinematic. The source/table container SHA-256 is
`e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f`.

## Exact biography definition

The [creation-answer definition](209026-creation-answers.md) is type 260 at
container **1282831:3991224**, source SHA-256
`f9af9a85b3de5bd09c1840d058bb61442e414cdfeb0f99720f8896e2d6ccba56`.
Its +0x28 pointer reaches the 45 question records at 3991280. Within the
56-byte interval preceding that array, its only classified content references
are +0x18, +0x20 and +0x28. The first two share target 4006872; this inspection
does not assign them new semantics.

Across all **45 question headers**, the only classified record references
are the known answer-array pointers at question +0x20. Across their **193
answer records**, the only classified content references are four type-77
item links at answer +0x38. They belong to question index 1 and answer bytes
190, 2, 3 and 4, targeting container 1282860 offsets 4903232, 4902880, 4884080
and 4875080 respectively. No classified file references occur in these
question or answer records, and none of the four item targets is type 7.

This excludes a **direct relocated cinematic/file link in those specific
question and answer records**. It does not exclude a numeric selector in an
unnamed scalar, another content type, a native selection rule, or server
selection. No race-to-cinematic consumer was established. The next useful
input is an exact selected cinematic key or a demonstrated selector that
consumes race/biography; the catalog alone cannot justify a server-720 value.

## Retained evidence

Private catalog `~/re/gw2/notes/cinematic-cache-catalog-20261106.json` is
199350 bytes, SHA-256
`c23dc693a65d57d76a44cef2e12d1267122372230ea134efaa55344d0f24b5f6`.
It retains all 1,366 index rows, all admitted source hashes and the bounded
sample/reference results. Record intervals may include unindexed subordinate
data; their whole interval is not asserted to be the fixed definition size.

Complete REA bundle
`~/re/gw2/notes/209026-cinematic-biography-cache-20261106.json` is 61760 bytes,
SHA-256 `f940dca25f53a43a9068075e1d45ddadf2f6f19ad708f08c1b407117b0761e4f`.

- `ev_8fadeeebdcdb4cfaffc079387f8b28636391943f31b2189675982a6ade61455e`:
  successful cache-only catalog, sample file fixups and all answer links.
- `ev_362cece9e6cddc5ed378c7605f2bacab260a87df905c70a4b97e6a4257c6a1d6`:
  exact type-260 header and all question reference slots.
- `ev_e6e18ce0061da64d9bbdf3575b9971adb8e7e738a59890a75d33134d5450cd97`:
  superseded first attempt stopped at a conservative 1,024-record count bound
  before export; its failure is retained, not treated as an absence result.

Both successful captures retained all raw terminal bytes. This investigation
did not open either archive, launch or attach a client, acquire a runtime
lease, or change a gameplay fixture or publisher state.
