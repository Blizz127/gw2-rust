# Build 209026 readable species-name coverage

REA source investigation, 2026-11-05, against frozen client SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`
and its retained archive. This closes the bounded name lookup proposed in the
[species schema note](209026-species-name-model-schema.md). No NPC or encounter
was selected, placed or sent to the client.

## Native decoding boundary

The name consumer uses the Dword at species +0x194. The native text decoder
`0x1410e0840` rejects a nonzero record marker at +2 when its key argument is
zero. With marker zero, the branch at `0x1410e0957` copies the payload at +6
without decryption. At `0x1410e0990`–`0x1410e09cf`, marker zero, width byte
+4 equal to 16 and an even payload size select direct UTF-16 copying.
Every readable record found below uses precisely that branch. No encrypted
record was decoded or assigned a guessed key.

## Coverage and exact matches

All 32 base content containers contain **23,653 type-67 species**. Of these,
1,049 have zero name ID. The remaining records reference **19,255 distinct
nonzero IDs** across 1,082 English string packs. The frozen text manifest is
file 110865, SHA-256
`2e5350b743f20e981f0436f350c6422c69df07ba774391e64144fab8dc8b5243`.

**803 IDs (4.17%) have marker zero**, all with width 16 and flags byte zero.
All decode to nonempty text, covering **953 species (4.03% of all species)**.
The rest were retained as record-header metadata without decoding.

Searching that readable subset for Thora, Ido, Eir, Issormir, ice wurm, boar,
griffon and minotaur returned 20 species, of which 17 names start with `Mini `.
The three other exact results are:

| Species key | Content file:offset | Name ID | English file / entry / record offset | Native text | Direct model file at +0xf0 |
| --- | --- | --- | --- | --- | --- |
| 3172 | 1282845:7100312 | 425678 | 2441139 / 718 / 14821 | `Eir` | null |
| 1544 | 1282845:7268648 | 43867 | 2440766 / 859 / 35322 | `Branded Minotaur[s]` | 19830 |
| 8013 | 1282848:5858640 | 167139 | 2440887 / 227 / 10908 | `Juvenile Boar[s]` | 26012 |

The bracketed suffixes are retained source text, before display formatting.
The name `Eir` establishes the label of species 3172; it does not establish
Great Hunt membership, a position, or the model path for that species. A null
direct model field also does not exclude alternate appearance paths.

There are no readable matches for Thora, Ido, Issormir, Ice Wurm, Vicious Dire
Boar, Owl Griffon Sire or Alpine Bull Minotaur. This absence applies only to
the 803 decoded IDs, not to the encrypted records or server encounter data.
The [Great Hunt link audit](209026-great-hunt-species-link-audit.md) keeps the
separate map/quest-link coverage boundary.

## Reproduction and retained evidence

Private helpers reuse the existing `content_209026.py` relocation reader and
repository `gw2_dat::archive::Archive`; they do not introduce another archive
parser. A small private Rust batch reader opens the archive once for the pack
set. All 32 content hashes, all 1,082 pack hashes, header observations, decoded
plain text and species/model joins are retained in:

- `~/re/gw2/notes/species-name-catalog-20261105-{0-8,8-16,16-24,24-32}.json`
- `~/re/gw2/notes/species-english-plain-names-20261105.json`
- `~/re/gw2/notes/species-english-plain-name-matches-20261105.json`

The final match artifact distinguishes `species_file`/`species_offset` from
`text_file`/`text_offset`; use it for the exact addresses above. Its SHA-256 is
`91486ec4598aa74881799054b0f09177dfe1f728cd8e638d1946f71efef5b29e`.
The initial coverage artifact's match projection uses ambiguous `file` and
`offset` keys for text records; its raw species catalog remains separate.

Complete successful REA records are retained privately in
`~/re/gw2/notes/209026-species-plain-name-coverage-1.json`:

| Evidence | Observation |
| --- | --- |
| `ev_2f801adf3cf265a34baecf392a09cb38eb165ee8791546368b4a947c8e78cf9a` | Species catalog containers 0–7. |
| `ev_4f891114446b54734605b3746220a8520cc26fe893b50fe02dc980bdddbd9505` | Species catalog containers 8–15. |
| `ev_e8fa8b69d097ddd85ed1fddb88abe1e78ff6f6e74e8febbff992dd3c5ac99a0d` | Species catalog containers 16–23. |
| `ev_b5bb78eaf73885024fce5b88b9e2a84cb7708a3240610469d6103a659bc0c5f0` | Species catalog containers 24–31. |
| `ev_45eb9f4aea5eb42d469bcc361de9b3eea936db506182f605f34041fb0f65b17e` | Private batch-reader compilation and hashes. |
| `ev_65d0e086f3f0c836ce9e17b15d300fc70a0209273132f9265fca9c39eb44c5ee` | Native zero-marker/no-key decoding branch. |
| `ev_6091b5739bfadd748ce9330f8b9080d1f192048628c1e02c4d777c6d86e5fe03` | Complete English pack coverage and plain decoding. |
| `ev_f69ddb00c12e95228d3885280195eb1a4e57c1ad296b8cadf11d00538ab82a0d` | Exact corrected species/text joins and fractions. |

All captures exited successfully. No gameplay code or client playback changed.
