//! Resolve a saved character by the selected GUID in its mission preamble.
//! The remaining world inputs are explicit local-profile settings.

use bytes::Bytes;
use gw2_proto::{Guid, character_creation::CreationRequest209026, population::PopulationProfession209026};
use std::io;
use super::{character_store::CharacterStore, equipment::EquipmentFixture, mission::PopulationFixture, skills::SkillFixture};

#[derive(Clone)]
pub struct SavedPopulation {
    pub store: CharacterStore,
    pub spawn: [f32; 3],
    pub profession: PopulationProfession209026,
    pub reward_tracks: [u32; 4],
    pub quest: Option<(u32, u8, u32)>,
    pub equipment: Option<Bytes>,
    pub skills: Option<Bytes>,
    pub initial_health: Option<f32>,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

impl SavedPopulation {
    pub fn validate(&self) -> io::Result<()> {
        let PopulationProfession209026::Mesmer { values: [first, second] } = self.profession else {
            return Err(invalid("saved Norn profile requires explicit Mesmer state"));
        };
        if first == 0 || second > first || !self.spawn.iter().all(|value| value.is_finite()) {
            return Err(invalid("invalid saved population state or spawn"));
        }
        if self.initial_health.is_some_and(|value| !value.is_finite() || value <= 0.0) {
            return Err(invalid("initial full health must be finite and positive"));
        }
        if self.quest.is_some_and(|(_, flags, _)| flags & 1 != 0) {
            return Err(invalid("initial quest completion flag must be clear"));
        }
        if let Some(bytes) = &self.equipment {
            let base = EquipmentFixture::parse(bytes, 1)?;
            // Every supported creation choice must fit this profile before it
            // starts accepting connections, including the additional slot5 item.
            for answer in [187, 188, 189] {
                let equipment = base
                    .with_mesmer_biography(&[answer])?
                    .with_creation_colors(&[[u32::from(u16::MAX); 4]; 6])?;
                EquipmentFixture::parse(equipment.bytes(), 1)?;
            }
        }
        if let Some(bytes) = &self.skills { SkillFixture::parse(bytes, 1)?; }
        Ok(())
    }

    pub fn for_join(&self, preamble: &[u8]) -> io::Result<PopulationFixture> {
        self.validate()?;
        if preamble.len() != 84 {
            return Err(invalid("saved population requires a complete mission preamble"));
        }
        // Verified selected GUID, not the other account/session fields:
        // docs/re/209026-creation-mission-binding.md.
        let guid = Guid(preamble[48..64].try_into().unwrap());
        let saved = self.store.find_guid(guid)?.ok_or_else(||
            io::Error::new(io::ErrorKind::NotFound, "mission selected an unknown saved character"))?;
        let request = CreationRequest209026::parse(saved.request())?;
        let mut fixture = PopulationFixture::with_profession(
            &saved.auth_body(), self.spawn, self.profession, self.reward_tracks,
        )?;
        if let Some((key, flags, goal)) = self.quest { fixture = fixture.with_quest(key, flags, goal)?; }
        if let Some(bytes) = &self.equipment {
            let equipment = EquipmentFixture::parse(bytes, 1)?
                .with_mesmer_biography(request.answers())?
                .with_creation_colors(request.item_colors())?;
            fixture = fixture.with_equipment(equipment.bytes())?;
        }
        if let Some(bytes) = &self.skills { fixture = fixture.with_skills(bytes)?; }
        if let Some(health) = self.initial_health { fixture = fixture.with_initial_health(health)?; }
        Ok(fixture)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::{
        character_store::SavedCharacter,
        mission::{MissionPhase, Seen},
        preamble::Preamble,
    };
    use bytes::BytesMut;
    use gw2_proto::{
        Encode,
        item::{
            ArmorItemCreationFrame209026, EquipmentFrame209026, EquipmentOperation209026,
            ItemCommon209026, ItemUpgrades209026,
        },
        map_load::ClientMapEvent,
    };
    use std::{fs, path::PathBuf};

    /// Synthetic cosmetic/content values exercise persistence and framing only.
    pub(crate) struct SavedScenario {
        pub profile: SavedPopulation,
        pub characters: [SavedCharacter; 2],
        directory: PathBuf,
    }

    impl SavedScenario {
        pub fn new() -> Self {
            let mut nonce = [0u8; 16];
            getrandom::fill(&mut nonce).unwrap();
            let suffix: String = nonce.iter().map(|byte| format!("{byte:02x}")).collect();
            let directory = std::env::temp_dir().join(format!("gw2-saved-population-{suffix}"));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(&directory).unwrap();
            let store = CharacterStore::open(directory.join("store")).unwrap();
            let characters = ["Amber Ω", "Birch"].map(|name| {
                let index = usize::from(name == "Birch");
                store
                    .create(&request(
                        name,
                        index as u8 + 1,
                        colors(index).map(|row| row.map(u32::from)),
                    ))
                    .unwrap()
            });
            drop(store);
            // Every selection below uses a reopened store, not the creator's handle.
            let profile = SavedPopulation {
                store: CharacterStore::open(directory.join("store")).unwrap(),
                spawn: [-123.5, 456.25, -789.0],
                profession: PopulationProfession209026::Mesmer { values: [3, 1] },
                reward_tracks: [11, 22, 33, 44],
                quest: Some((154, 0, 1)),
                equipment: Some(armor_batch([[9; 4]; 6], None)),
                skills: Some(Bytes::from(crate::skills::fixture_for_test())),
                initial_health: Some(777.0),
            };
            Self {
                profile,
                characters,
                directory,
            }
        }

        pub fn capture_path(&self, label: &str) -> PathBuf {
            self.directory.join(format!("{label}.bin"))
        }

        pub fn preamble(&self, selected: usize) -> Vec<u8> {
            let mut bytes = crate::preamble::fixture(Preamble::Mission);
            // Both are real saved GUIDs; only bytes 48..64 select the character.
            bytes[32..48].copy_from_slice(&self.characters[1 - selected].guid().0);
            bytes[48..64].copy_from_slice(&self.characters[selected].guid().0);
            bytes
        }

        pub fn expected(&self, selected: usize) -> PopulationFixture {
            // Construct directly from the chosen auth record, never via for_join
            // or the biography/recolor helpers. Expected armor carries explicit
            // mask keys and all six dye rows, including the added slot5 mask.
            let mut fixture = PopulationFixture::with_profession(
                &self.characters[selected].auth_body(),
                self.profile.spawn,
                self.profile.profession,
                self.profile.reward_tracks,
            )
            .unwrap();
            if let Some((key, flags, goal)) = self.profile.quest {
                fixture = fixture.with_quest(key, flags, goal).unwrap();
            }
            fixture = fixture
                .with_equipment(&armor_batch(
                    colors(selected),
                    Some((6551 + selected as u32, 2058 + selected as u32)),
                ))
                .unwrap();
            if let Some(skills) = &self.profile.skills {
                fixture = fixture.with_skills(skills).unwrap();
            }
            if let Some(health) = self.profile.initial_health {
                fixture = fixture.with_initial_health(health).unwrap();
            }
            fixture
        }
    }

    impl Drop for SavedScenario {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }

    fn colors(index: usize) -> [[u16; 4]; 6] {
        let mut colors = std::array::from_fn(|row| {
            std::array::from_fn(|column| (1000 * (index + 1) + row * 4 + column) as u16)
        });
        if index == 0 {
            colors[0] = [0, 127, 128, u16::MAX];
        }
        colors
    }

    fn request(name: &str, marker: u8, colors: [[u32; 4]; 6]) -> Vec<u8> {
        let mut bytes = BytesMut::new();
        415u16.encode(&mut bytes);
        for unit in name.encode_utf16().chain(std::iter::once(0)) {
            unit.encode(&mut bytes);
        }
        let mut appearance = [0; 48];
        appearance[0] = marker;
        appearance[10] = marker + 10;
        appearance[27..30].copy_from_slice(&[3, 9, 1]);
        appearance[45..].copy_from_slice(&[marker, 0xbe, 0xef]);
        bytes.extend_from_slice(&appearance);
        7u8.encode(&mut bytes);
        1u8.encode(&mut bytes);
        8u32.encode(&mut bytes);
        bytes.extend_from_slice(&[3, 186 + marker, 1, 7]);
        for row in colors {
            row.encode(&mut bytes);
        }
        bytes.to_vec()
    }

    fn armor_batch(colors: [[u16; 4]; 6], mask: Option<(u32, u32)>) -> Bytes {
        let mut bytes = BytesMut::new();
        for (row, colors) in colors.into_iter().enumerate().filter(|(row, _)| *row != 3) {
            ArmorItemCreationFrame209026 {
                common: ItemCommon209026 {
                    instance_id: 101 + row as u32,
                    item_data_key: 201 + row as u32,
                    raw_flags: 0,
                    guid: None,
                },
                skin_data_key: 301 + row as u32,
                colors,
                upgrades: ItemUpgrades209026 {
                    prefix_data_key: 0,
                    primary_upgrade_data_key: 0,
                    secondary_upgrade_data_key: 0,
                    additional_upgrade_data_keys: Vec::new(),
                },
            }
            .encode_frame(&mut bytes)
            .unwrap();
        }
        if let Some((item_data_key, skin_data_key)) = mask {
            ArmorItemCreationFrame209026 {
                common: ItemCommon209026 {
                    instance_id: 107,
                    item_data_key,
                    raw_flags: 0,
                    guid: None,
                },
                skin_data_key,
                colors: colors[3],
                upgrades: ItemUpgrades209026 {
                    prefix_data_key: 112,
                    primary_upgrade_data_key: 0,
                    secondary_upgrade_data_key: 0,
                    additional_upgrade_data_keys: Vec::new(),
                },
            }
            .encode_frame(&mut bytes)
            .unwrap();
        }
        for row in (0..6).filter(|row| *row != 3) {
            EquipmentFrame209026 {
                character_index: 1,
                operation: EquipmentOperation209026::Equip {
                    item_instance_id: 101 + row,
                    equipment_slot: 2 + row as u8,
                },
            }
            .encode_frame(&mut bytes);
        }
        if mask.is_some() {
            EquipmentFrame209026 {
                character_index: 1,
                operation: EquipmentOperation209026::Equip {
                    item_instance_id: 107,
                    equipment_slot: 5,
                },
            }
            .encode_frame(&mut bytes);
        }
        bytes.freeze()
    }

    // Synthetic structural limits; these content keys are never used in playback.
    fn capacity_batch(slots: &[u8], last_instance: u32, extra_upgrades: Option<usize>) -> Bytes {
        let rich = extra_upgrades.is_some();
        let key = if rich { u32::MAX } else { 1 };
        let mut remaining = extra_upgrades.unwrap_or(0);
        let mut creations = BytesMut::new();
        let mut equips = BytesMut::new();
        for (index, &slot) in slots.iter().enumerate() {
            let instance = if index + 1 == slots.len() { last_instance } else { index as u32 + 1 };
            let count = remaining.min(4);
            remaining -= count;
            ArmorItemCreationFrame209026 {
                common: ItemCommon209026 {
                    instance_id: instance,
                    item_data_key: key,
                    raw_flags: 0,
                    guid: rich.then_some(Guid([0xab; 16])),
                },
                skin_data_key: key,
                colors: [0; 4],
                upgrades: ItemUpgrades209026 {
                    prefix_data_key: key,
                    primary_upgrade_data_key: key,
                    secondary_upgrade_data_key: key,
                    additional_upgrade_data_keys: vec![u32::MAX; count],
                },
            }.encode_frame(&mut creations).unwrap();
            EquipmentFrame209026 {
                character_index: 1,
                operation: EquipmentOperation209026::Equip {
                    item_instance_id: instance,
                    equipment_slot: slot,
                },
            }.encode_frame(&mut equips);
        }
        assert_eq!(remaining, 0);
        creations.extend_from_slice(&equips);
        creations.freeze()
    }

    pub(crate) fn frames(fixture: PopulationFixture) -> [Bytes; 3] {
        let mut phase = MissionPhase::new(379)
            .unwrap()
            .with_population(fixture)
            .unwrap();
        [
            phase.opening(),
            phase
                .respond(&Seen::Map(ClientMapEvent::MapLoaded))
                .unwrap()
                .unwrap(),
            phase
                .respond(&Seen::Map(ClientMapEvent::AgentsLoaded))
                .unwrap()
                .unwrap(),
        ]
    }

    #[test]
    fn reopened_selection_uses_guid_at_48_and_preserves_name_appearance_and_dyes() {
        let scenario = SavedScenario::new();
        assert_ne!(scenario.characters[0].guid(), scenario.characters[1].guid());
        assert_ne!(scenario.characters[0].name(), scenario.characters[1].name());
        assert_ne!(
            &scenario.characters[0].summary()[6..51],
            &scenario.characters[1].summary()[6..51]
        );
        let expected = [frames(scenario.expected(0)), frames(scenario.expected(1))];
        assert_ne!(expected[0][1], expected[1][1]);
        for selected in 0..2 {
            let saved = &scenario.characters[selected];
            let reopened = scenario
                .profile
                .store
                .find_guid(saved.guid())
                .unwrap()
                .unwrap();
            assert_eq!(reopened.request(), saved.request());
            assert_eq!(reopened.summary(), saved.summary());
            let actual = scenario
                .profile
                .for_join(&scenario.preamble(selected))
                .unwrap();
            assert_eq!(frames(actual), expected[selected]);
        }
    }

    #[test]
    fn absent_guid_and_incomplete_preamble_never_select_neighbor() {
        let scenario = SavedScenario::new();
        let mut preamble = scenario.preamble(0);
        preamble[48..64].fill(0);
        assert_eq!(
            scenario.profile.for_join(&preamble).err().unwrap().kind(),
            io::ErrorKind::NotFound
        );
        for length in 0..84 {
            assert_eq!(
                scenario
                    .profile
                    .for_join(&preamble[..length])
                    .err()
                    .unwrap()
                    .kind(),
                io::ErrorKind::InvalidInput
            );
        }
        preamble.push(0);
        assert_eq!(
            scenario.profile.for_join(&preamble).err().unwrap().kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn profile_rejects_invalid_state_health_quest_and_component_batches() {
        let scenario = SavedScenario::new();
        let good = &scenario.profile;
        good.validate().unwrap();
        let mut bad = Vec::new();
        for profession in [
            PopulationProfession209026::Guardian,
            PopulationProfession209026::Mesmer { values: [0, 0] },
            PopulationProfession209026::Mesmer { values: [1, 2] },
        ] {
            let mut profile = good.clone();
            profile.profession = profession;
            bad.push(profile);
        }
        for axis in 0..3 {
            for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let mut profile = good.clone();
                profile.spawn[axis] = value;
                bad.push(profile);
            }
        }
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
            let mut profile = good.clone();
            profile.initial_health = Some(value);
            bad.push(profile);
        }
        for flags in [1, 255] {
            let mut profile = good.clone();
            profile.quest = Some((154, flags, 1));
            bad.push(profile);
        }
        for bytes in [Bytes::new(), Bytes::from_static(&[0xff])] {
            let mut profile = good.clone();
            profile.equipment = Some(bytes.clone());
            bad.push(profile);
            let mut profile = good.clone();
            profile.skills = Some(bytes);
            bad.push(profile);
        }
        for profile in bad {
            assert!(profile.validate().is_err());
            assert!(profile.for_join(&scenario.preamble(0)).is_err());
        }
        for values in [[1, 0], [255, 255]] {
            let mut profile = good.clone();
            profile.profession = PopulationProfession209026::Mesmer { values };
            profile.quest = Some((154, 254, 1));
            profile.validate().unwrap();
        }
        for health in [f32::MIN_POSITIVE, f32::MAX] {
            let mut profile = good.clone();
            profile.initial_health = Some(health);
            profile.validate().unwrap();
        }
    }

    #[test]
    fn creation_color_outside_armor_word_bound_is_rejected_before_selection() {
        let scenario = SavedScenario::new();
        let mut color_rows = colors(0).map(|row| row.map(u32::from));
        color_rows[5][3] = u32::from(u16::MAX) + 1;
        let rejected = scenario
            .profile
            .store
            .create(&request("Wide dye", 3, color_rows));
        assert_eq!(rejected.err().unwrap().kind(), io::ErrorKind::InvalidInput);
        assert!(
            scenario
                .profile
                .store
                .find_name("Wide dye")
                .unwrap()
                .is_none()
        );
        assert_eq!(scenario.profile.store.list().unwrap().len(), 2);
        assert_eq!(
            frames(scenario.profile.for_join(&scenario.preamble(0)).unwrap()),
            frames(scenario.expected(0))
        );
    }

    #[test]
    fn profile_rejects_valid_base_equipment_without_room_for_biography() {
        let scenario = SavedScenario::new();
        let full_slots: Vec<_> = (0..68).collect();
        let free_slots: Vec<_> = (0..68).filter(|&slot| slot != 5).collect();
        let cases = [
            (armor_batch([[0; 4]; 6], Some((6551, 2058))), "slot5"),
            (capacity_batch(&full_slots, 68, None), "68 equipment items"),
            (capacity_batch(&[2], 4096, None), "instance"),
            (capacity_batch(&free_slots, 67, Some(11)), "4096 bytes"),
        ];
        for (bytes, expected_error) in cases {
            EquipmentFixture::parse(&bytes, 1).unwrap();
            let mut profile = scenario.profile.clone();
            profile.equipment = Some(bytes.clone());
            let error = profile.validate().unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert!(error.to_string().contains(expected_error), "{error}");
            assert_eq!(profile.equipment.as_ref().unwrap(), &bytes);
        }

        // The final available instance and item slot remain usable.
        let mut profile = scenario.profile.clone();
        profile.equipment = Some(capacity_batch(&free_slots, 4095, None));
        profile.validate().unwrap();
        profile.for_join(&scenario.preamble(0)).unwrap();
    }

    #[test]
    fn profile_accepts_all_biographies_and_maximum_dyes_at_final_byte_limit() {
        let scenario = SavedScenario::new();
        let free_slots: Vec<_> = (0..68).filter(|&slot| slot != 5).collect();
        let bytes = capacity_batch(&free_slots, 67, Some(10));
        let base = EquipmentFixture::parse(&bytes, 1).unwrap();
        let maximum_colors = [[u32::from(u16::MAX); 4]; 6];
        let mut profile = scenario.profile.clone();
        profile.equipment = Some(bytes.clone());
        profile.validate().unwrap();
        for marker in 1..=3 {
            let complete = base.with_mesmer_biography(&[186 + marker]).unwrap()
                .with_creation_colors(&maximum_colors).unwrap();
            assert_eq!(complete.bytes().len(), super::super::equipment::EQUIPMENT_BYTES_LIMIT);
            EquipmentFixture::parse(complete.bytes(), 1).unwrap();
            let saved = profile.store.create(&request(
                &format!("Maximum {marker}"), marker, maximum_colors,
            )).unwrap();
            let mut preamble = scenario.preamble(0);
            preamble[48..64].copy_from_slice(&saved.guid().0);
            profile.for_join(&preamble).unwrap();
        }
        assert_eq!(profile.equipment.as_ref().unwrap(), &bytes);
    }
}
