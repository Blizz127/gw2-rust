# Build 209026: reward-track slots and pin values

Static REA follow-up to [reward-track creation](209026-reward-track-messages.md),
2026-11-01, on the same frozen executable. No runtime replay occurred.

## Pin values: creation 565 and update 569

Getter `0x141248aa0`, reward-manager virtual +0xc0, bounds its slot at four
and returns Dword `manager +0x98 + slot*4`. Its assertion string at
`0x142171520` is **`slot < arrsize(m_pinnedTracks)`**. This identifies the
four values initialized from server 565's final four Bytes as pin state.
They remain raw numeric values here; this does not prove that only 0/1 is
accepted or establish their UI interpretation.

Server 569 (`0x1425d4590`, handler `0x141259ed0`) carries Dword player
index, Byte slot, Byte pin value. Handler and setter `0x1412494c0`
zero-extend/store the pin byte at +0x98 + slot*4 and notify manager listeners
at virtual +0x38 via `0x14025552c`. Unlike the getter, this inspected
setter and its handler have **no slot-bound check**. The four-element
storage establishes the intended range 0–3; raw Byte framing does not
make other slot values valid. Existing creation-codec `slot_bytes` remain
raw for compatibility and are now documented as pin values.

## Active selection: 568

Server 568 (`0x1425d44c0`, handler `0x141259da0`) carries player-index
Dword, type-353 data-key Dword, slot Byte. Setter `0x1412491f0` resolves
content virtual +0x70, logs a missing track and continues, then asserts
`slot < arrsize(m_tracks)` (four). It clears the slot's bit in manager
+0xf0, stores the new active pointer at +0x38 + slot*8, and dispatches
listener virtual +0x28. If the separate slot-record pointer at
+0x58 + slot*16 is null, it initializes that record through virtual +0xe0.
It does not change the pin value and does not grant player progress.
The active pointer can change while an existing slot-record pointer is
retained; the two arrays must not be merged in a reconstruction.

## Map-specific entry selection: 566

Server 566 (`0x1425d4840`, handler `0x141259e40`) carries player-index
Dword, **type-45 map data-key Dword**, and Byte entry index. Setter
`0x1412493a0` resolves type 45 via +0x70 and immediately reads the map
record, without a null guard. Assertion `0x1421714c0` names
`mapDef->GetRewardTrackCount() > trackIndex`.

It selects the map's pointer array at +0x178/count +0x180 and inserts or
replaces that pointer in manager +0x18's keyed table, keyed by the map
input data key. This is distinct from the four active track slots. The
inspected body does not emit a manager listener event. A bounded archive follow-up checked Queensdale's known type-45/data-key-15
record (file 1282830, content offset 3754040): its reward-track count at
+0x180 is **zero**. Thus no Byte entry index can satisfy 566's count check
for this map record. Do not fabricate a map-specific track-selection
message for the Queensdale fixture. This does not exclude tracks obtained
through other game systems or the separate four active slots.

## Per-track keyed state: 567

Server 567 (`0x1425d4a30`, handler `0x141259f70`) carries player-index
Dword, type-353 data-key Dword, Byte state, Dword value. Setter
`0x141249500` stores the last two fields as Dwords in a 16-byte entry of
manager +0xa8's table, keyed by the track data key. It then resolves the
track through +0x70 and notifies virtual +0 for nonzero state, or virtual
+0x10 for zero state. No nonnull assertion guards that notification target.

Getter `0x141248370` looks up the same table by track +0x28 and passes the
stored trailing value multiplied by 1000 to `0x1409bbf40`; absence supplies
zero. This suggests a time-related value but does not yet establish its
unit, epoch or gameplay meaning. No semantic name or default is assigned.

## Evidence and scope

Private bundles: `~/re/gw2/notes/209026-reward-track-slots-1.json` and
`~/re/gw2/notes/209026-queensdale-reward-tracks-1.json`. The latter includes
private extraction `queensdale-reward-tracks-209026.json` and its source hash.
Only documentation and the creation codec's field comment changed; wire
behavior is unchanged. `git diff --check` passed. No new runtime or test
result is claimed for these adjacent messages; they have no typed codecs.

| REA evidence | Observation |
| --- | --- |
| `ev_29c9f0b4cf7d3c1bd0b33ddfec87ac6a6bfdc8c1af18c875d8def676c843e6fc` | Complete 566/567/569 setters. |
| `ev_388a4f5e462718cf8e32f494c0cbef0cc8b59ced5e2604283bea0b4404ddef63` | Complete 568 setter and all four handlers; active/slot-record getters. |
| `ev_b0d1afc191016f0ad12c101b5d49fad6ed79aaacf48099863578f04180f1e62d` | Keyed trailing-value getter, event slot and all four descriptors. |
| `ev_b887b04d54285e95e3ee8dd9f74c1323861fa51dd702b19f08590d207fa87957` | Bounded field-consumer search and map/slot assertions. |
| `ev_1b4d3553f8e62f51fc1f73aa8fd5f7205b8dc5dae1481034885e746c5f872fae` | Complete pin-value getter and m_pinnedTracks assertion. |
| `ev_3203314fab7fb1a15a181644f53982eca73c2ce84c9ea9431df0bd7e9a351c7e` | Queensdale map record has zero map-specific reward-track entries. |
