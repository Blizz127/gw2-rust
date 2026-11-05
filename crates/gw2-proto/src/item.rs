//! Build-209026 item creation (armor 960, weapon 971) and equipment (288/299).
//! See `docs/re/209026-item-equipment-delivery.md` for source consumers and limits.
//! These codecs preserve caller-supplied state; they do not choose a loadout.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode, Guid};

/// Shared creation fields. Instance IDs and content data keys are distinct namespaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemCommon209026 {
    /// Runtime item-service registry ID, subsequently referenced by message 288.
    pub instance_id: u32,
    /// Type-35 item definition data key, not an instance ID or archive file ID.
    pub item_data_key: u32,
    /// Stored at item +0x50. Flag meanings and retail initial values are unresolved.
    pub raw_flags: u8,
    /// The optional Guid occurs last on the wire, after subtype fields and upgrades.
    pub guid: Option<Guid>,
}

impl ItemCommon209026 {
    fn encode_prefix(&self, output: &mut BytesMut) {
        self.instance_id.encode(output);
        self.item_data_key.encode(output);
        self.raw_flags.encode(output);
    }

    fn decode_prefix(input: &mut &[u8]) -> Result<Self> {
        Ok(Self {
            instance_id: u32::decode(input)?,
            item_data_key: u32::decode(input)?,
            raw_flags: u8::decode(input)?,
            guid: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemUpgrades209026 {
    /// Type-37 prefix definition data key. Zero does not preserve a constructor prefix.
    pub prefix_data_key: u32,
    /// Type-35 upgrade item definition keys, not runtime instance IDs.
    pub primary_upgrade_data_key: u32,
    pub secondary_upgrade_data_key: u32,
    /// Descriptor-bounded ArrayVarSmall: a byte count followed by at most four Dwords.
    pub additional_upgrade_data_keys: Vec<u32>,
}

impl ItemUpgrades209026 {
    fn validate(&self) -> Result<()> {
        if self.additional_upgrade_data_keys.len() > 4 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "item creation permits at most four additional upgrade keys",
            ));
        }
        Ok(())
    }

    fn encode_fixed(&self, output: &mut BytesMut) {
        self.prefix_data_key.encode(output);
        self.primary_upgrade_data_key.encode(output);
        self.secondary_upgrade_data_key.encode(output);
    }

    fn encode_array(&self, output: &mut BytesMut) {
        (self.additional_upgrade_data_keys.len() as u8).encode(output);
        for key in &self.additional_upgrade_data_keys {
            key.encode(output);
        }
    }

    fn decode_fixed(input: &mut &[u8]) -> Result<Self> {
        Ok(Self {
            prefix_data_key: u32::decode(input)?,
            primary_upgrade_data_key: u32::decode(input)?,
            secondary_upgrade_data_key: u32::decode(input)?,
            additional_upgrade_data_keys: Vec::new(),
        })
    }

    fn decode_array(&mut self, input: &mut &[u8]) -> Result<()> {
        let count = u8::decode(input)?;
        if count > 4 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "item creation permits at most four additional upgrade keys",
            ));
        }
        for _ in 0..count {
            self.additional_upgrade_data_keys.push(u32::decode(input)?);
        }
        Ok(())
    }
}

/// Server 960, requiring a type-35 item definition of the armor subtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmorItemCreationFrame209026 {
    pub common: ItemCommon209026,
    /// Type-66 skin definition data key.
    pub skin_data_key: u32,
    /// Four Words in descriptor order, copied to armor +0xe0/+0xe2/+0xe4/+0xe6.
    pub colors: [u16; 4],
    pub upgrades: ItemUpgrades209026,
}

impl ArmorItemCreationFrame209026 {
    /// Append a complete frame; leave output unchanged if the array exceeds its bound.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        self.upgrades.validate()?;
        960u16.encode(output);
        self.common.encode_prefix(output);
        self.skin_data_key.encode(output);
        self.colors.encode(output);
        self.upgrades.encode_fixed(output);
        self.upgrades.encode_array(output);
        self.common.guid.encode(output);
        Ok(())
    }

    /// Consume one complete frame, leaving input unchanged on every error.
    /// Definition kind, instance uniqueness, and other client state belong to the caller.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != 960 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected armor message 960",
            ));
        }
        let mut common = ItemCommon209026::decode_prefix(&mut cursor)?;
        let skin_data_key = u32::decode(&mut cursor)?;
        let colors = <[u16; 4]>::decode(&mut cursor)?;
        let mut upgrades = ItemUpgrades209026::decode_fixed(&mut cursor)?;
        upgrades.decode_array(&mut cursor)?;
        common.guid = Option::<Guid>::decode(&mut cursor)?;
        *input = cursor;
        Ok(Self {
            common,
            skin_data_key,
            colors,
            upgrades,
        })
    }
}

/// Server 971, requiring a type-35 item definition of the weapon subtype.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponItemCreationFrame209026 {
    pub common: ItemCommon209026,
    /// Type-66 skin definition data key.
    pub skin_data_key: u32,
    pub upgrades: ItemUpgrades209026,
    /// Unnamed subtype fields, identified by their client object offsets.
    pub raw_d8: u8,
    pub raw_f0: u32,
    pub raw_f4: u8,
    pub raw_e8: u64,
}

impl WeaponItemCreationFrame209026 {
    /// Append a complete frame; leave output unchanged if the array exceeds its bound.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        self.upgrades.validate()?;
        971u16.encode(output);
        self.common.encode_prefix(output);
        self.skin_data_key.encode(output);
        self.upgrades.encode_fixed(output);
        self.raw_d8.encode(output);
        self.raw_f0.encode(output);
        self.raw_f4.encode(output);
        self.raw_e8.encode(output);
        self.upgrades.encode_array(output);
        self.common.guid.encode(output);
        Ok(())
    }

    /// Consume one complete frame, leaving input unchanged on every error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != 971 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected weapon message 971",
            ));
        }
        let mut common = ItemCommon209026::decode_prefix(&mut cursor)?;
        let skin_data_key = u32::decode(&mut cursor)?;
        let mut upgrades = ItemUpgrades209026::decode_fixed(&mut cursor)?;
        let raw_d8 = u8::decode(&mut cursor)?;
        let raw_f0 = u32::decode(&mut cursor)?;
        let raw_f4 = u8::decode(&mut cursor)?;
        let raw_e8 = u64::decode(&mut cursor)?;
        upgrades.decode_array(&mut cursor)?;
        common.guid = Option::<Guid>::decode(&mut cursor)?;
        *input = cursor;
        Ok(Self {
            common,
            skin_data_key,
            upgrades,
            raw_d8,
            raw_f0,
            raw_f4,
            raw_e8,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentOperation209026 {
    /// Server 288. The client requires an existing instance and an empty slot below 68.
    Equip {
        item_instance_id: u32,
        equipment_slot: u8,
    },
    /// Server 299. The client setter asserts against 6 (toy) and 11 (sentinel).
    /// The codec preserves the byte; choosing a valid gameplay set is a caller policy.
    SelectWeaponSet { weapon_set: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquipmentFrame209026 {
    pub character_index: u32,
    pub operation: EquipmentOperation209026,
}

impl EquipmentFrame209026 {
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use EquipmentOperation209026::*;
        let id: u16 = match self.operation {
            Equip { .. } => 288,
            SelectWeaponSet { .. } => 299,
        };
        id.encode(output);
        self.character_index.encode(output);
        match self.operation {
            Equip {
                item_instance_id,
                equipment_slot,
            } => {
                item_instance_id.encode(output);
                equipment_slot.encode(output);
            }
            SelectWeaponSet { weapon_set } => weapon_set.encode(output),
        }
    }

    /// Consume one complete frame, leaving input unchanged on every error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !matches!(id, 288 | 299) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected equipment message 288 or 299",
            ));
        }
        let character_index = u32::decode(&mut cursor)?;
        let operation = match id {
            288 => EquipmentOperation209026::Equip {
                item_instance_id: u32::decode(&mut cursor)?,
                equipment_slot: u8::decode(&mut cursor)?,
            },
            299 => EquipmentOperation209026::SelectWeaponSet {
                weapon_set: u8::decode(&mut cursor)?,
            },
            _ => unreachable!(),
        };
        *input = cursor;
        Ok(Self {
            character_index,
            operation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, CountPrefix, Rules, Value};
    use crate::schema;

    fn armor_fixture() -> (ArmorItemCreationFrame209026, Vec<u8>) {
        (
            ArmorItemCreationFrame209026 {
                common: ItemCommon209026 {
                    instance_id: 300,
                    item_data_key: 16384,
                    raw_flags: 255,
                    guid: Some(Guid([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])),
                },
                skin_data_key: 127,
                colors: [0x1234, 0xffff, 0, 1],
                upgrades: ItemUpgrades209026 {
                    prefix_data_key: 128,
                    primary_upgrade_data_key: 0,
                    secondary_upgrade_data_key: u32::MAX,
                    additional_upgrade_data_keys: vec![1, 127, 128, 300],
                },
            },
            // Literal field-order fixture, not a proposed gameplay loadout.
            vec![
                0xc0, 0x03, // 960
                0xac, 0x02, // instance
                0x80, 0x80, 0x01, // item definition
                0xff, 0x7f, // flags, skin
                0x34, 0x12, 0xff, 0xff, 0, 0, 1, 0, // four Words, no count
                0x80, 0x01, 0, 0xff, 0xff, 0xff, 0xff, 0x0f, // prefix, two upgrades
                4, 1, 0x7f, 0x80, 1, 0xac, 2, // bounded byte-counted array
                1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, // Guid
            ],
        )
    }

    fn weapon_fixture() -> (WeaponItemCreationFrame209026, Vec<u8>) {
        (
            WeaponItemCreationFrame209026 {
                common: ItemCommon209026 {
                    instance_id: u32::MAX,
                    item_data_key: 1,
                    raw_flags: 128,
                    guid: None,
                },
                skin_data_key: 300,
                upgrades: ItemUpgrades209026 {
                    prefix_data_key: 112,
                    primary_upgrade_data_key: 128,
                    secondary_upgrade_data_key: 16384,
                    additional_upgrade_data_keys: vec![],
                },
                raw_d8: 255,
                raw_f0: u32::MAX,
                raw_f4: 254,
                raw_e8: 0x0123456789abcdef,
            },
            vec![
                0xcb, 0x03, // 971
                0xff, 0xff, 0xff, 0xff, 0x0f, // instance
                1, 0x80, 0xac, 2, // item, flags, skin
                0x70, 0x80, 1, 0x80, 0x80, 1, // prefix, two upgrades
                0xff, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xfe, // raw d8, f0, f4
                0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, // raw e8
                0, 0, // empty array, absent Guid
            ],
        )
    }

    fn equipment_fixtures() -> [(EquipmentFrame209026, Vec<u8>); 2] {
        [
            (
                EquipmentFrame209026 {
                    character_index: 300,
                    operation: EquipmentOperation209026::Equip {
                        item_instance_id: 128,
                        equipment_slot: 255,
                    },
                },
                vec![0x20, 1, 0xac, 2, 0x80, 1, 0xff],
            ),
            (
                EquipmentFrame209026 {
                    character_index: u32::MAX,
                    operation: EquipmentOperation209026::SelectWeaponSet { weapon_set: 11 },
                },
                vec![0x2b, 1, 0xff, 0xff, 0xff, 0xff, 0x0f, 11],
            ),
        ]
    }

    fn fields(values: Vec<Value>) -> Value {
        Value::Struct(
            values
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), v))
                .collect(),
        )
    }

    fn array(values: impl IntoIterator<Item = Value>) -> Value {
        Value::Array(values.into_iter().map(|v| fields(vec![v])).collect())
    }

    fn guid(value: Option<Guid>) -> Value {
        Value::Optional(value.map(|v| Box::new(fields(vec![Value::Guid(v)]))))
    }

    #[test]
    fn item_golden_fields_and_next_frame_boundaries() {
        let (armor, armor_bytes) = armor_fixture();
        let (weapon, weapon_bytes) = weapon_fixture();
        let equipment = equipment_fixtures();
        let mut encoded = BytesMut::from(&b"prefix"[..]);
        armor.encode_frame(&mut encoded).unwrap();
        weapon.encode_frame(&mut encoded).unwrap();
        for (frame, _) in &equipment {
            frame.encode_frame(&mut encoded);
        }
        let stream: Vec<u8> = [
            armor_bytes.as_slice(),
            weapon_bytes.as_slice(),
            &equipment[0].1,
            &equipment[1].1,
        ]
        .concat();
        assert_eq!(&encoded[..6], b"prefix");
        assert_eq!(&encoded[6..], stream);
        let mut input = stream.as_slice();
        assert_eq!(
            ArmorItemCreationFrame209026::decode_frame(&mut input).unwrap(),
            armor
        );
        let mut consumed = armor_bytes.len();
        assert_eq!(input, &stream[consumed..]);
        assert_eq!(
            WeaponItemCreationFrame209026::decode_frame(&mut input).unwrap(),
            weapon
        );
        consumed += weapon_bytes.len();
        assert_eq!(input, &stream[consumed..]);
        for (frame, bytes) in equipment {
            assert_eq!(
                EquipmentFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += bytes.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn item_golden_fields_match_extracted_client_schema() {
        let tables = schema::parse(include_str!(
            "../../../data/protocol/209026/server-tables.xml"
        ))
        .unwrap();
        let messages: Vec<_> = tables
            .protocols
            .into_iter()
            .filter(|p| p.name == "0")
            .flat_map(|p| p.groups)
            .filter(|g| matches!(g.name.as_str(), "table-0x142260af0" | "table-0x14216e940"))
            .flat_map(|g| g.server)
            .collect();
        let rules = Rules {
            array_var_small: Some(CountPrefix::U8),
            ..Rules::default()
        };
        let (a, armor_bytes) = armor_fixture();
        let (w, weapon_bytes) = weapon_fixture();
        let mut fixtures = vec![
            (
                960,
                armor_bytes,
                fields(vec![
                    Value::U32(a.common.instance_id),
                    Value::U32(a.common.item_data_key),
                    Value::U8(a.common.raw_flags),
                    Value::U32(a.skin_data_key),
                    array(a.colors.into_iter().map(Value::U16)),
                    Value::U32(a.upgrades.prefix_data_key),
                    Value::U32(a.upgrades.primary_upgrade_data_key),
                    Value::U32(a.upgrades.secondary_upgrade_data_key),
                    array(
                        a.upgrades
                            .additional_upgrade_data_keys
                            .into_iter()
                            .map(Value::U32),
                    ),
                    guid(a.common.guid),
                ]),
            ),
            (
                971,
                weapon_bytes,
                fields(vec![
                    Value::U32(w.common.instance_id),
                    Value::U32(w.common.item_data_key),
                    Value::U8(w.common.raw_flags),
                    Value::U32(w.skin_data_key),
                    Value::U32(w.upgrades.prefix_data_key),
                    Value::U32(w.upgrades.primary_upgrade_data_key),
                    Value::U32(w.upgrades.secondary_upgrade_data_key),
                    Value::U8(w.raw_d8),
                    Value::U32(w.raw_f0),
                    Value::U8(w.raw_f4),
                    Value::U64(w.raw_e8),
                    array(
                        w.upgrades
                            .additional_upgrade_data_keys
                            .into_iter()
                            .map(Value::U32),
                    ),
                    guid(w.common.guid),
                ]),
            ),
        ];
        for (frame, bytes) in equipment_fixtures() {
            let (id, mut values) = match frame.operation {
                EquipmentOperation209026::Equip {
                    item_instance_id,
                    equipment_slot,
                } => (
                    288,
                    vec![Value::U32(item_instance_id), Value::U8(equipment_slot)],
                ),
                EquipmentOperation209026::SelectWeaponSet { weapon_set } => {
                    (299, vec![Value::U8(weapon_set)])
                }
            };
            values.insert(0, Value::U32(frame.character_index));
            fixtures.push((id, bytes, fields(values)));
        }
        for (id, bytes, expected) in fixtures {
            assert_eq!(messages.iter().filter(|m| m.id == id).count(), 1);
            let mut input = bytes.as_slice();
            let (message, actual) = dynamic::decode_frame(&messages, &rules, &mut input).unwrap();
            assert_eq!(message.id, id);
            assert_eq!(actual, expected);
            assert!(input.is_empty());
        }
    }

    #[test]
    fn item_all_array_lengths_and_optional_guids_preserve_values() {
        for count in 0..=4 {
            for guid in [None, Some(Guid([0xff; 16]))] {
                let (mut armor, _) = armor_fixture();
                let (mut weapon, _) = weapon_fixture();
                let keys = vec![0, u32::MAX, 128, 16384][..count].to_vec();
                armor.common.guid = guid;
                weapon.common.guid = guid;
                armor.upgrades.additional_upgrade_data_keys = keys.clone();
                weapon.upgrades.additional_upgrade_data_keys = keys;
                let mut bytes = BytesMut::new();
                armor.encode_frame(&mut bytes).unwrap();
                weapon.encode_frame(&mut bytes).unwrap();
                let mut input = &bytes[..];
                assert_eq!(
                    ArmorItemCreationFrame209026::decode_frame(&mut input).unwrap(),
                    armor
                );
                assert_eq!(
                    WeaponItemCreationFrame209026::decode_frame(&mut input).unwrap(),
                    weapon
                );
                assert!(input.is_empty());
            }
        }
    }

    type Decoder = fn(&mut &[u8]) -> Result<()>;

    fn decoders_and_fixtures() -> Vec<(Decoder, Vec<u8>)> {
        vec![
            (
                |b| ArmorItemCreationFrame209026::decode_frame(b).map(|_| ()),
                armor_fixture().1,
            ),
            (
                |b| WeaponItemCreationFrame209026::decode_frame(b).map(|_| ()),
                weapon_fixture().1,
            ),
            (
                |b| EquipmentFrame209026::decode_frame(b).map(|_| ()),
                equipment_fixtures()[0].1.clone(),
            ),
            (
                |b| EquipmentFrame209026::decode_frame(b).map(|_| ()),
                equipment_fixtures()[1].1.clone(),
            ),
        ]
    }

    fn assert_atomic_error(decode: Decoder, bytes: &[u8], kind: ErrorKind) {
        let mut input = bytes;
        assert_eq!(decode(&mut input).unwrap_err().kind(), kind);
        assert_eq!(input, bytes);
    }

    #[test]
    fn item_truncation_wrong_id_and_malformed_dwords_are_atomic() {
        for (decode, bytes) in decoders_and_fixtures() {
            for cut in 0..bytes.len() {
                assert_atomic_error(decode, &bytes[..cut], ErrorKind::UnexpectedEof);
            }
            let mut wrong = bytes.clone();
            wrong[..2].copy_from_slice(&959u16.to_le_bytes());
            assert_atomic_error(decode, &wrong, ErrorKind::InvalidData);
            for malformed in [[0xff, 0xff, 0xff, 0xff, 0x10], [0x80; 5]] {
                let mut bad = bytes[..2].to_vec();
                bad.extend_from_slice(&malformed);
                bad.extend_from_slice(&bytes[2..]);
                assert_atomic_error(decode, &bad, ErrorKind::InvalidData);
            }
        }
    }

    #[test]
    fn item_invalid_array_count_optional_tag_and_upgrade_dword_are_atomic() {
        let fixtures = decoders_and_fixtures();
        for (decode, bytes, count_offset, guid_offset) in [
            (fixtures[0].0, fixtures[0].1.as_slice(), 25, 32),
            (fixtures[1].0, fixtures[1].1.as_slice(), 32, 33),
        ] {
            assert!(matches!(bytes[count_offset], 0 | 4));
            assert!(matches!(bytes[guid_offset], 0 | 1));
            for invalid_count in [5, 255] {
                let mut bad = bytes.to_vec();
                bad[count_offset] = invalid_count;
                assert_atomic_error(decode, &bad, ErrorKind::InvalidData);
            }
            for invalid_tag in [2, 255] {
                let mut bad = bytes.to_vec();
                bad[guid_offset] = invalid_tag;
                assert_atomic_error(decode, &bad, ErrorKind::InvalidData);
            }
            let mut bad = bytes[..count_offset].to_vec();
            bad.extend_from_slice(&[1, 0xff, 0xff, 0xff, 0xff, 0x10, 0]);
            assert_atomic_error(decode, &bad, ErrorKind::InvalidData);
        }
    }

    #[test]
    fn item_oversized_array_encode_is_atomic() {
        let (mut armor, _) = armor_fixture();
        let (mut weapon, _) = weapon_fixture();
        armor.upgrades.additional_upgrade_data_keys = vec![1; 5];
        weapon.upgrades.additional_upgrade_data_keys = vec![1; 5];
        let mut bytes = BytesMut::from(&b"prefix"[..]);
        assert_eq!(
            armor.encode_frame(&mut bytes).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(bytes.as_ref(), b"prefix");
        assert_eq!(
            weapon.encode_frame(&mut bytes).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(bytes.as_ref(), b"prefix");
    }
}
