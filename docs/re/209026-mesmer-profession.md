# Build 209026: Mesmer profession state

2026-11-05. Static inspection of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This supports the Norn-first Mesmer starting-area work. It is not a retail
initial-state or combat implementation claim.

## Messages and lifetime

The client-derived registration table and inspected handlers agree:

| Server ID | Handler | Fields after Dword character index | Operation |
| --- | --- | --- | --- |
| 481 | `0x141256770` | Byte, Byte | Allocate/install Mesmer component, then set both values. |
| 482 | `0x141256840` | None | Remove profession through `0x1411d6a00`. |
| 483 | `0x1412568a0` | Byte, Byte | Fetch the existing profession and set both values. |

The Dword uses the protocol varint encoding. These are alternatives to the
Guardian operations, not additional components to install on a Guardian.
483 does not create or type-check a missing/different profession in the
inspected handler. The shared profession installation and finalization
prerequisites are in [character profession](209026-character-profession.md).

481 allocates 0x88 bytes, calls constructor `0x141294fa0`, installs the
component through `0x1411e21a0`, and zero-extends both message bytes into
setter `0x1412952a0`. The constructor installs primary vtable
`0x14217c0f0` and secondary interface `0x14217c270` at component +0x48.
The direct source literal identifies `ChCliProfessionMesmer.cpp`.

## Observed arithmetic, without inferred gameplay names

| Wire value | Stored Dword | Constructor value |
| --- | --- | --- |
| First byte | +0x54 | 1 |
| Second byte | +0x50 | 0 |

The setter retains both unsigned values without clamping. If either changes,
it dispatches the listener collection at +0x58 through `0x14026dce0` and
trampoline `0x140255568`. The constructor clears the collection pointers;
the creation handler registers no intervening listener.

Primary virtual +0x08 (`0x141295040`) compares an unsigned argument against
the second stored value. Primary virtual +0xf8 (`0x141295050`) returns
`float(u32(first - second)) / float(first)` without a zero-divisor or
underflow guard. Thus a synthetic population plan rejects first=0 and
second>first; this is a candidate-fixture arithmetic constraint, not a
claim that the raw protocol rejects those values. The raw codec preserves
all byte pairs.

Neither constructor values [1,0] nor synthetic test values [3,0] establish
retail login defaults, clone limits, unlock state or resource semantics.
Those require the content/state-selection and downstream consumer traces.

The later [resource-consumer trace](209026-mesmer-resource-consumers.md)
establishes **first = displayed capacity**, **second = current resource
count** in the ordinary Mesmer status widget. It does not establish a live
clone census, maximum three, retail login pair or server update policy.
The raw codec preserves both byte values and keeps the explicit pair API.

## Implementation and verification

`gw2_proto::mesmer` encodes/decodes 481–483 transactionally. Golden-frame,
concatenated-frame and malformed/truncated-input tests pass. The population
builder now takes an explicit `PopulationProfession209026` and four explicit
reward-track keys. Mesmer coreStats profession 7 accompanies creator 481;
Guardian coreStats profession 1 accompanies creator 477. Full stream checks
against the extracted server schemas and typed frame boundaries pass for both.

The observer's research fixture requires an explicit pair of options:
`--mission-mesmer-values FIRST,SECOND --mission-reward-tracks A,B,C,D`,
together with its existing population record, spawn and map options. It
rejects mismatched summary professions. There is deliberately no default
Mesmer track selection or state. The existing Guardian fixture retains its
previous values. Common health, movement, equipment and skill placeholders
remain; a Mesmer-shaped bootstrap does not establish a playable Mesmer.

Focused observer tests cover explicit state/summary matching, readiness
sequencing across tutorial/open-map keys, and argument width/range handling.
Encrypted TCP loopback tests also pass at every delivery split for both the
existing Guardian fixture and a synthetic Norn Mesmer tutorial379 fixture.
The observer build passes. No Mesmer client playback or deployment has been
performed for this change.

## Evidence

Private bundle: `~/re/gw2/notes/209026-mesmer-profession-20261105.json`.
REA ran hash-checked Python/objdump readers, not the game. Both exited 0 with
owned-process cleanup verified. Unrelated unreadable `/proc` entries were
left untouched; process sampling and inherited-environment limits apply.

| REA Evidence | Observation |
| --- | --- |
| `ev_ec948c6cd2da934312d68852b4cb21a8f04304b0de79ed03926bd828e6d03a7a` | 481–483 handlers, constructor, setters, ratio and predicate. |
| `ev_1a0b38b349243e682081f955833139538ad524ddd05da73759de84ccd22acb69` | Mesmer source literal and primary/secondary vtable pointers. |

Earlier registration/enum evidence is retained in
[character profession](209026-character-profession.md). Raw assembly and
client artifacts remain outside the repository.

A noncached Ghidra pseudocode request reached its 330-second startup deadline
without completing import/analysis. No pseudocode result is used here. REA
session inspection confirmed no pending cleanup and the owned Java process
was gone; the native session was then closed. The bounded hash-checked
REA/objdump observations above supply the recorded instruction evidence.
