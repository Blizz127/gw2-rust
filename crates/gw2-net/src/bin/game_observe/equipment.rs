//! Bounded, explicit initial equipment batch for the offline observer.
//! All content keys and raw item fields come from the caller. This validates
//! protocol/state prerequisites, not a retail loadout or item definitions.

use bytes::Bytes;
use gw2_proto::item::{
    ArmorItemCreationFrame209026, EquipmentFrame209026, EquipmentOperation209026,
    WeaponItemCreationFrame209026,
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
