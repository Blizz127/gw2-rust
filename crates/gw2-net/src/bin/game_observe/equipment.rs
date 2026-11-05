//! Bounded, explicit initial equipment batch for the offline observer.
//! Base content keys and raw fields come from the caller. The optional Mesmer
//! biography addition uses frozen source references and explicit research fields.
//! This validates protocol/state prerequisites, not a retail loadout.

use bytes::Bytes;
use gw2_proto::item::{
    ArmorItemCreationFrame209026, EquipmentFrame209026, EquipmentOperation209026,
    ItemCommon209026, ItemUpgrades209026, WeaponItemCreationFrame209026,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

pub const EQUIPMENT_BYTES_LIMIT: usize = 4096;
/// Observer resource policy: the client grows its sparse instance registry.
/// This is not a claimed client maximum or retail item-ID allocation policy.
const MAX_FIXTURE_INSTANCE_ID: u32 = 4096;

#[derive(Clone)]
pub struct EquipmentFixture(Bytes);

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

/// Resolve exactly one submitted Mesmer profession answer to its item/skin keys.
/// The frozen Norn level-1 lookup is documented in
/// docs/re/209026-mesmer-biography-equipment.md. Missing, repeated, or competing
/// choices do not inherit the creation preview's first-choice fallback.
pub fn mesmer_mask_choice(answers: &[u8]) -> io::Result<(u32, u32)> {
    let mut choices = answers
        .iter()
        .copied()
        .filter(|answer| (187..=189).contains(answer));
    let choice = choices
        .next()
        .ok_or_else(|| invalid("Mesmer biography requires one mask answer"))?;
    if choices.next().is_some() {
        return Err(invalid("Mesmer biography has multiple mask answers"));
    }
    Ok(match choice {
        187 => (6551, 2058),
        188 => (6552, 2059),
        189 => (6553, 2060),
        _ => unreachable!(),
    })
}

impl EquipmentFixture {
    /// Initial armor/weapon creations, then equipment references, then optional
    /// weapon-set selection. Every created instance must be equipped exactly
    /// once. A weapon batch must select a set containing a supplied weapon.
    pub fn parse(bytes: &[u8], character_index: u32) -> io::Result<Self> {
        if bytes.is_empty() || bytes.len() > EQUIPMENT_BYTES_LIMIT {
            return Err(invalid("equipment fixture must contain 1..4096 bytes"));
        }
        let mut cursor = bytes;
        let mut items = BTreeMap::new();
        let mut equipped = BTreeSet::new();
        let mut slots = BTreeMap::new();
        let mut phase = 0;
        let mut selected = false;
        while !cursor.is_empty() {
            let id = cursor
                .get(..2)
                .ok_or_else(|| invalid("truncated equipment frame ID"))?;
            let message_id = u16::from_le_bytes([id[0], id[1]]);
            match message_id {
                960 | 971 => {
                    if phase != 0 || items.len() >= 68 {
                        return Err(invalid(
                            "at most 68 item creations must precede all equipment frames",
                        ));
                    }
                    let (common, weapon) = if message_id == 960 {
                        (
                            ArmorItemCreationFrame209026::decode_frame(&mut cursor)?.common,
                            false,
                        )
                    } else {
                        (
                            WeaponItemCreationFrame209026::decode_frame(&mut cursor)?.common,
                            true,
                        )
                    };
                    if common.instance_id == 0
                        || common.instance_id > MAX_FIXTURE_INSTANCE_ID
                        || common.item_data_key == 0
                    {
                        return Err(invalid(
                            "equipment requires instance ID 1..4096 and a nonzero item data key",
                        ));
                    }
                    if items.insert(common.instance_id, weapon).is_some() {
                        return Err(invalid("duplicate equipment item instance ID"));
                    }
                }
                288 | 299 => {
                    if phase == 2 {
                        return Err(invalid(
                            "weapon-set selection must be the last equipment frame",
                        ));
                    }
                    let frame = EquipmentFrame209026::decode_frame(&mut cursor)?;
                    if frame.character_index != character_index {
                        return Err(invalid(
                            "equipment character index differs from population fixture",
                        ));
                    }
                    match frame.operation {
                        EquipmentOperation209026::Equip {
                            item_instance_id,
                            equipment_slot,
                        } => {
                            phase = 1;
                            let weapon = *items.get(&item_instance_id).ok_or_else(|| {
                                invalid(
                                    "equipment references an instance not created in this fixture",
                                )
                            })?;
                            if equipment_slot >= 68
                                || slots.insert(equipment_slot, weapon).is_some()
                            {
                                return Err(invalid("equipment slots must be unique and below 68"));
                            }
                            if !equipped.insert(item_instance_id) {
                                return Err(invalid("an item instance can be equipped only once"));
                            }
                        }
                        EquipmentOperation209026::SelectWeaponSet { weapon_set } => {
                            // Exact switch tables in 14128fdc0/14128fe70.
                            // Sets 6..10 enter diagnostics; 11 is a sentinel.
                            let pair = [[24, 24], [25, 25], [26, 26], [27, 28], [29, 30], [31, 32]]
                                .get(usize::from(weapon_set))
                                .copied()
                                .ok_or_else(|| invalid("initial weapon set must be 0..5"))?;
                            if !pair.iter().any(|slot| slots.get(slot) == Some(&true)) {
                                return Err(invalid(
                                    "selected weapon set contains no supplied weapon",
                                ));
                            }
                            phase = 2;
                            selected = true;
                        }
                    }
                }
                _ => {
                    return Err(invalid(
                        "equipment fixture allows only frames 960, 971, 288 and 299",
                    ));
                }
            }
        }
        if items.is_empty() || items.len() != equipped.len() {
            return Err(invalid(
                "every created equipment instance must be equipped once",
            ));
        }
        if items.values().any(|weapon| *weapon) && !selected {
            return Err(invalid(
                "a weapon fixture requires explicit weapon-set selection",
            ));
        }
        Ok(Self(Bytes::copy_from_slice(bytes)))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// Add the selected Norn level-1 Mesmer biography item in equipment slot5.
    /// Instance allocation, flags0, absent Guid/upgrades and initial zero colors
    /// are observer research choices. Apply with_creation_colors afterwards to
    /// use actual request415 row5. Existing frame bytes are copied unchanged.
    pub fn with_mesmer_biography(&self, answers: &[u8]) -> io::Result<Self> {
        let (item_data_key, skin_data_key) = mesmer_mask_choice(answers)?;
        let mut cursor = self.bytes();
        let mut item_count = 0;
        let mut max_instance_id = 0;
        let mut creation_end = None;
        let mut equip_insert = self.bytes().len();
        let mut character_index = None;
        let mut slot_occupied = false;
        while !cursor.is_empty() {
            let offset = self.bytes().len() - cursor.len();
            let id = u16::from_le_bytes(cursor[..2].try_into().unwrap());
            match id {
                960 | 971 => {
                    let common = if id == 960 {
                        ArmorItemCreationFrame209026::decode_frame(&mut cursor)?.common
                    } else {
                        WeaponItemCreationFrame209026::decode_frame(&mut cursor)?.common
                    };
                    item_count += 1;
                    max_instance_id = max_instance_id.max(common.instance_id);
                }
                _ => {
                    creation_end.get_or_insert(offset);
                    let frame = EquipmentFrame209026::decode_frame(&mut cursor)?;
                    character_index = Some(frame.character_index);
                    match frame.operation {
                        EquipmentOperation209026::Equip { equipment_slot, .. } => {
                            slot_occupied |= equipment_slot == 5;
                        }
                        EquipmentOperation209026::SelectWeaponSet { .. } => equip_insert = offset,
                    }
                }
            }
        }
        if item_count >= 68 {
            return Err(invalid("biography item would exceed 68 equipment items"));
        }
        if slot_occupied {
            return Err(invalid(
                "Mesmer biography equipment slot5 is already occupied",
            ));
        }
        let instance_id = max_instance_id
            .checked_add(1)
            .filter(|value| *value <= MAX_FIXTURE_INSTANCE_ID)
            .ok_or_else(|| invalid("biography item instance would exceed observer limit4096"))?;
        let character_index =
            character_index.ok_or_else(|| invalid("equipment fixture has no character"))?;
        let creation_end =
            creation_end.ok_or_else(|| invalid("equipment fixture has no equips"))?;
        let mut out = bytes::BytesMut::new();
        out.extend_from_slice(&self.bytes()[..creation_end]);
        ArmorItemCreationFrame209026 {
            common: ItemCommon209026 {
                instance_id,
                item_data_key,
                raw_flags: 0,
                guid: None,
            },
            skin_data_key,
            colors: [0; 4],
            upgrades: ItemUpgrades209026 {
                prefix_data_key: 112,
                primary_upgrade_data_key: 0,
                secondary_upgrade_data_key: 0,
                additional_upgrade_data_keys: vec![],
            },
        }
        .encode_frame(&mut out)?;
        out.extend_from_slice(&self.bytes()[creation_end..equip_insert]);
        EquipmentFrame209026 {
            character_index,
            operation: EquipmentOperation209026::Equip {
                item_instance_id: instance_id,
                equipment_slot: 5,
            },
        }
        .encode_frame(&mut out);
        out.extend_from_slice(&self.bytes()[equip_insert..]);
        Self::parse(&out, character_index)
    }

    /// Apply the actual creation request's dye rows for equipment slots 2..7.
    /// Creation uses Dwords; armor960 uses Words. Reject an unrepresentable
    /// used color instead of truncating it. Other fields/frames are preserved.
    pub fn with_creation_colors(&self, colors: &[[u32; 4]; 6]) -> io::Result<Self> {
        let mut slots = BTreeMap::new();
        let mut cursor = self.bytes();
        while !cursor.is_empty() {
            match u16::from_le_bytes(cursor[..2].try_into().unwrap()) {
                960 => { ArmorItemCreationFrame209026::decode_frame(&mut cursor)?; }
                971 => { WeaponItemCreationFrame209026::decode_frame(&mut cursor)?; }
                _ => {
                    if let EquipmentOperation209026::Equip { item_instance_id, equipment_slot } =
                        EquipmentFrame209026::decode_frame(&mut cursor)?.operation
                    { slots.insert(item_instance_id, equipment_slot); }
                }
            }
        }
        let mut out = bytes::BytesMut::new();
        cursor = self.bytes();
        while !cursor.is_empty() {
            let before = cursor;
            match u16::from_le_bytes(cursor[..2].try_into().unwrap()) {
                960 => {
                    let mut frame = ArmorItemCreationFrame209026::decode_frame(&mut cursor)?;
                    let slot = slots[&frame.common.instance_id];
                    if (2..=7).contains(&slot) {
                        for (output, color) in frame.colors.iter_mut().zip(colors[usize::from(slot - 2)]) {
                            *output = u16::try_from(color).map_err(|_| invalid("creation color does not fit armor Word"))?;
                        }
                    }
                    frame.encode_frame(&mut out)?;
                }
                id => {
                    if id == 971 { WeaponItemCreationFrame209026::decode_frame(&mut cursor)?; }
                    else { EquipmentFrame209026::decode_frame(&mut cursor)?; }
                    out.extend_from_slice(&before[..before.len() - cursor.len()]);
                }
            }
        }
        Ok(Self(out.freeze()))
    }
}

#[cfg(test)]
pub(super) fn fixture_for_test() -> Vec<u8> {
    tests::valid_fixture()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use gw2_proto::item::{ItemCommon209026, ItemUpgrades209026};

    fn item(instance_id: u32) -> Vec<u8> {
        item_key(instance_id, 33355)
    }

    fn item_key(instance_id: u32, item_data_key: u32) -> Vec<u8> {
        let mut out = BytesMut::new();
        WeaponItemCreationFrame209026 {
            common: ItemCommon209026 {
                instance_id,
                item_data_key,
                raw_flags: 0,
                guid: None,
            },
            skin_data_key: 3873,
            upgrades: ItemUpgrades209026 {
                prefix_data_key: 112,
                primary_upgrade_data_key: 0,
                secondary_upgrade_data_key: 0,
                additional_upgrade_data_keys: vec![],
            },
            raw_d8: 0,
            raw_f0: 0,
            raw_f4: 0,
            raw_e8: 0,
        }
        .encode_frame(&mut out)
        .unwrap();
        out.to_vec()
    }

    fn equipment(character_index: u32, operation: EquipmentOperation209026) -> Vec<u8> {
        let mut out = BytesMut::new();
        EquipmentFrame209026 {
            character_index,
            operation,
        }
        .encode_frame(&mut out);
        out.to_vec()
    }

    fn equip(character: u32, instance: u32, slot: u8) -> Vec<u8> {
        equipment(
            character,
            EquipmentOperation209026::Equip {
                item_instance_id: instance,
                equipment_slot: slot,
            },
        )
    }

    fn select(set: u8) -> Vec<u8> {
        equipment(
            1,
            EquipmentOperation209026::SelectWeaponSet { weapon_set: set },
        )
    }

    fn armor() -> Vec<u8> {
        let mut out = BytesMut::new();
        ArmorItemCreationFrame209026 {
            common: ItemCommon209026 {
                instance_id: 101,
                item_data_key: 6465,
                raw_flags: 0,
                guid: None,
            },
            skin_data_key: 13,
            colors: [117, 615, 480, 0],
            upgrades: ItemUpgrades209026 {
                prefix_data_key: 112,
                primary_upgrade_data_key: 0,
                secondary_upgrade_data_key: 0,
                additional_upgrade_data_keys: vec![],
            },
        }
        .encode_frame(&mut out)
        .unwrap();
        out.to_vec()
    }

    #[test]
    fn armor_needs_no_weapon_selection_and_mixed_batch_preserves_both_types() {
        let only_armor = [armor(), equip(1, 101, 2)].concat();
        assert_eq!(
            EquipmentFixture::parse(&only_armor, 1).unwrap().bytes(),
            only_armor
        );
        let mixed = [
            armor(),
            item(100),
            equip(1, 101, 2),
            equip(1, 100, 29),
            select(4),
        ]
        .concat();
        assert_eq!(EquipmentFixture::parse(&mixed, 1).unwrap().bytes(), mixed);
        let wrong_type = [armor(), equip(1, 101, 29), select(4)].concat();
        assert!(
            EquipmentFixture::parse(&wrong_type, 1).is_err(),
            "armor cannot satisfy selected weapon-set prerequisite"
        );
    }

    #[test]
    fn creation_dyes_follow_equipped_slot_and_preserve_other_frames() {
        let bytes = [armor(), item(100), equip(1, 101, 2), equip(1, 100, 29), select(4)].concat();
        let original = EquipmentFixture::parse(&bytes, 1).unwrap();
        let mut colors = [[0; 4]; 6];
        colors[0] = [1, 2, 3, 65535];
        let changed = original.with_creation_colors(&colors).unwrap();
        let mut cursor = changed.bytes();
        let armor = ArmorItemCreationFrame209026::decode_frame(&mut cursor).unwrap();
        assert_eq!(armor.colors, [1, 2, 3, 65535]);
        let mut old = original.bytes();
        let old_armor = ArmorItemCreationFrame209026::decode_frame(&mut old).unwrap();
        assert_eq!(cursor, old);
        assert_eq!(armor.common, old_armor.common);
        assert_eq!(armor.upgrades, old_armor.upgrades);
        assert_eq!(armor.skin_data_key, old_armor.skin_data_key);
        colors[0][3] = 65536;
        assert!(original.with_creation_colors(&colors).is_err());
        assert_eq!(original.bytes(), bytes, "invalid dye cannot alter the stored fixture");
    }

    #[test]
    fn biography_requires_one_mask_answer_and_preserves_unrelated_answers() {
        for (answer, expected) in [
            (187, (6551, 2058)),
            (188, (6552, 2059)),
            (189, (6553, 2060)),
        ] {
            let answers = [53, answer, 117, 121, 124];
            assert_eq!(mesmer_mask_choice(&answers).unwrap(), expected);
            assert_eq!(answers, [53, answer, 117, 121, 124]);
        }
        let bytes = valid_fixture();
        let fixture = EquipmentFixture::parse(&bytes, 1).unwrap();
        for answers in [
            &[][..],
            &[53, 117, 121, 124],
            &[187, 187],
            &[188, 189],
            &[187, 188, 189],
        ] {
            assert!(mesmer_mask_choice(answers).is_err());
            assert!(fixture.with_mesmer_biography(answers).is_err());
            assert_eq!(fixture.bytes(), bytes);
        }
    }

    #[test]
    fn biography_adds_each_sourced_item_before_equips_and_preserves_existing_bytes() {
        let old_creations = [armor(), item(100)].concat();
        let old_equips = [equip(1, 101, 2), equip(1, 100, 29)].concat();
        let old_selection = select(4);
        let bytes = [
            old_creations.clone(),
            old_equips.clone(),
            old_selection.clone(),
        ]
        .concat();
        let original = EquipmentFixture::parse(&bytes, 1).unwrap();
        for (answer, item_key, skin_key) in
            [(187, 6551, 2058), (188, 6552, 2059), (189, 6553, 2060)]
        {
            let changed = original.with_mesmer_biography(&[53, 117, answer]).unwrap();
            assert!(changed.bytes().starts_with(&old_creations));
            let mut cursor = &changed.bytes()[old_creations.len()..];
            let mask = ArmorItemCreationFrame209026::decode_frame(&mut cursor).unwrap();
            assert_eq!(
                mask.common,
                ItemCommon209026 {
                    instance_id: 102,
                    item_data_key: item_key,
                    raw_flags: 0,
                    guid: None,
                }
            );
            assert_eq!(mask.skin_data_key, skin_key);
            assert_eq!(mask.colors, [0; 4]);
            assert_eq!(
                mask.upgrades,
                ItemUpgrades209026 {
                    prefix_data_key: 112,
                    primary_upgrade_data_key: 0,
                    secondary_upgrade_data_key: 0,
                    additional_upgrade_data_keys: vec![],
                }
            );
            assert!(cursor.starts_with(&old_equips));
            cursor = &cursor[old_equips.len()..];
            assert_eq!(
                EquipmentFrame209026::decode_frame(&mut cursor).unwrap(),
                EquipmentFrame209026 {
                    character_index: 1,
                    operation: EquipmentOperation209026::Equip {
                        item_instance_id: 102,
                        equipment_slot: 5
                    },
                }
            );
            assert_eq!(cursor, old_selection);
            assert_eq!(original.bytes(), bytes);
            assert!(changed.with_mesmer_biography(&[answer]).is_err());
        }
    }

    #[test]
    fn biography_armor_only_uses_same_character_and_actual_row5_colors() {
        let bytes = [armor(), equip(42, 101, 2)].concat();
        let original = EquipmentFixture::parse(&bytes, 42).unwrap();
        let augmented = original.with_mesmer_biography(&[188]).unwrap();
        let mut colors = [[0; 4]; 6];
        colors[0] = [1, 2, 3, 4];
        colors[3] = [117, 615, 117, 65535];
        let changed = augmented.with_creation_colors(&colors).unwrap();
        let mut cursor = changed.bytes();
        assert_eq!(
            ArmorItemCreationFrame209026::decode_frame(&mut cursor)
                .unwrap()
                .colors,
            [1, 2, 3, 4]
        );
        let mask = ArmorItemCreationFrame209026::decode_frame(&mut cursor).unwrap();
        assert_eq!(mask.common.instance_id, 102);
        assert_eq!(mask.common.item_data_key, 6552);
        assert_eq!(mask.colors, [117, 615, 117, 65535]);
        assert_eq!(
            EquipmentFrame209026::decode_frame(&mut cursor)
                .unwrap()
                .character_index,
            42
        );
        assert_eq!(
            EquipmentFrame209026::decode_frame(&mut cursor).unwrap(),
            EquipmentFrame209026 {
                character_index: 42,
                operation: EquipmentOperation209026::Equip {
                    item_instance_id: 102,
                    equipment_slot: 5
                },
            }
        );
        assert!(cursor.is_empty());
        colors[3][3] = 65536;
        assert!(augmented.with_creation_colors(&colors).is_err());
        assert_eq!(original.bytes(), bytes);
    }

    #[test]
    fn biography_rejects_occupied_slot_and_respects_instance_limit() {
        let occupied = [armor(), equip(1, 101, 5)].concat();
        let occupied = EquipmentFixture::parse(&occupied, 1).unwrap();
        assert!(occupied.with_mesmer_biography(&[187]).is_err());
        for (max, allowed) in [(4095, true), (4096, false)] {
            let bytes = [item(max), equip(1, max, 29), select(4)].concat();
            let fixture = EquipmentFixture::parse(&bytes, 1).unwrap();
            let changed = fixture.with_mesmer_biography(&[189]);
            assert_eq!(changed.is_ok(), allowed);
            if let Ok(changed) = changed {
                let mut cursor = changed.bytes();
                WeaponItemCreationFrame209026::decode_frame(&mut cursor).unwrap();
                assert_eq!(
                    ArmorItemCreationFrame209026::decode_frame(&mut cursor)
                        .unwrap()
                        .common
                        .instance_id,
                    4096
                );
            }
            assert_eq!(fixture.bytes(), bytes);
        }
    }

    #[test]
    fn biography_accepts_68th_item_and_rejects_full_fixture() {
        let mut cursor = &armor()[..];
        let mut frame = ArmorItemCreationFrame209026::decode_frame(&mut cursor).unwrap();
        for include_slot5 in [false, true] {
            let mut creations = BytesMut::new();
            let mut equips = Vec::new();
            for slot in 0..68 {
                if slot == 5 && !include_slot5 {
                    continue;
                }
                let instance = u32::from(slot) + 1;
                frame.common.instance_id = instance;
                frame.encode_frame(&mut creations).unwrap();
                equips.extend(equip(1, instance, slot));
            }
            creations.extend_from_slice(&equips);
            let fixture = EquipmentFixture::parse(&creations, 1).unwrap();
            let changed = fixture.with_mesmer_biography(&[187]);
            if include_slot5 {
                assert!(
                    changed
                        .err()
                        .unwrap()
                        .to_string()
                        .contains("68 equipment items")
                );
            } else {
                let changed = changed.unwrap();
                let mut cursor = changed.bytes();
                let mut instances = BTreeSet::new();
                for _ in 0..68 {
                    instances.insert(
                        ArmorItemCreationFrame209026::decode_frame(&mut cursor)
                            .unwrap()
                            .common
                            .instance_id,
                    );
                }
                assert_eq!(instances.len(), 68);
                assert!(
                    instances.contains(&69),
                    "allocate after max instance68, not after count67"
                );
                assert_eq!(u16::from_le_bytes(cursor[..2].try_into().unwrap()), 288);
            }
        }
    }

    pub(super) fn valid_fixture() -> Vec<u8> {
        [item(100), equip(1, 100, 29), select(4)].concat()
    }

    #[test]
    fn explicit_batch_preserves_bytes_and_rejects_broken_instance_slot_order() {
        let good = valid_fixture();
        assert_eq!(EquipmentFixture::parse(&good, 1).unwrap().bytes(), good);
        let bad = [
            [item(0), equip(1, 0, 29), select(4)].concat(),
            [
                item(MAX_FIXTURE_INSTANCE_ID + 1),
                equip(1, MAX_FIXTURE_INSTANCE_ID + 1, 29),
                select(4),
            ]
            .concat(),
            [item(u32::MAX), equip(1, u32::MAX, 29), select(4)].concat(),
            [item_key(100, 0), equip(1, 100, 29), select(4)].concat(),
            [item(100), item(100), equip(1, 100, 29), select(4)].concat(),
            [equip(1, 100, 29), item(100), select(4)].concat(),
            [
                item(100),
                equip(1, 100, 29),
                item(101),
                equip(1, 101, 30),
                select(4),
            ]
            .concat(),
            [item(100), equip(1, 101, 29), select(4)].concat(),
            [item(100), equip(2, 100, 29), select(4)].concat(),
            [item(100), equip(1, 100, 68), select(4)].concat(),
            [item(100), equip(1, 100, 29), equip(1, 100, 30), select(4)].concat(),
            [
                item(100),
                item(101),
                equip(1, 100, 29),
                equip(1, 101, 29),
                select(4),
            ]
            .concat(),
            [item(100), item(101), equip(1, 100, 29), select(4)].concat(),
            [item(100), equip(1, 100, 29)].concat(),
            [item(100), equip(1, 100, 29), select(5)].concat(),
            [item(100), equip(1, 100, 29), select(6)].concat(),
            [item(100), equip(1, 100, 29), select(11)].concat(),
            [item(100), equip(1, 100, 29), select(4), select(4)].concat(),
            [item(100), equip(1, 100, 29), select(4), vec![0x10, 0x02, 1]].concat(),
        ];
        for bytes in bad {
            assert!(EquipmentFixture::parse(&bytes, 1).is_err());
        }
        for cut in 0..good.len() {
            assert!(EquipmentFixture::parse(&good[..cut], 1).is_err());
        }
        assert!(EquipmentFixture::parse(&vec![0; EQUIPMENT_BYTES_LIMIT + 1], 1).is_err());
    }
}
