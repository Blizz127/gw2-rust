//! Build-209026 type-2 buff description facts, before display modifiers.
//!
//! The formatter at 140893000 consumes three values: signed integer tag 434,
//! typed buff tag 64, and signed time tag 444. Named values use 141481d70.
//! Evidence: `209026-mesmer-sword-buff-name-duration-findings-20261106.json`
//! and its separate coverage addendum (raw selected source windows retained).
//! Values are authored UI inputs, not wire stack counts, effect duration or
//! application policy. No defaults, unit conversion or modifier is applied.

use crate::ability_value_content::{
    AbilityValueLimits209026, AbilityValueMode209026, ResolvedAbilityValue209026,
    resolve_ability_value209026,
};
use crate::buff_content::BuffContent209026;
use crate::content::{ContentIndex, ContentReference, ResolvedContent, resolve_reference};
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillBuffFactContent209026<'a> {
    pub entry: ResolvedContent<'a>,
    pub definition: ResolvedContent<'a>,
    pub payload: ResolvedContent<'a>,
    /// The three selected value locations and literal/base/override provenance.
    pub values: [ResolvedAbilityValue209026<'a>; 3],
    /// Signed tag-434 Dword at value +8; not an authoritative stack count.
    pub raw_integer: i32,
    pub buff: BuffContent209026,
    /// Signed tag-444 Qword at value +8, without display or server conversion.
    pub raw_time: i64,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn unsupported(message: &'static str) -> Error {
    Error::new(ErrorKind::Unsupported, message)
}

fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn target<'a>(
    containers: &[ContentIndex<'a>],
    owner: ResolvedContent<'a>,
    field: usize,
    length: usize,
) -> Result<ResolvedContent<'a>> {
    let slot = owner
        .offset
        .checked_add(field)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| invalid("buff fact reference slot overflow"))?;
    let reference = containers[owner.container_index]
        .reference(slot)?
        .ok_or_else(|| invalid("buff fact reference is null"))?;
    resolve_reference(containers, owner.container_index, reference, length)
}

impl<'a> SkillBuffFactContent209026<'a> {
    /// Read the entry's classified definition (+0x28, type 377/data type 2)
    /// and payload (+0x40, exactly three 0x30-byte values). Resolve each value
    /// with the explicit ability context, mode and limits supplied by the caller.
    /// Limits apply separately to each of the three bounded lookups. Literal
    /// values do not consult the ability context; named results are not recursive.
    /// Local/external references retain the container owning each pointer slot.
    /// The entry's cached +0x48 value is not used as the type discriminator.
    pub fn read(
        containers: &[ContentIndex<'a>],
        entry_container: usize,
        entry_offset: u32,
        ability_container: usize,
        ability_offset: u32,
        mode: AbilityValueMode209026,
        limits: AbilityValueLimits209026,
    ) -> Result<Self> {
        let entry = resolve_reference(
            containers,
            entry_container,
            ContentReference::Local {
                offset: entry_offset as usize,
            },
            0x48,
        )?;
        let definition = target(containers, entry, 0x28, 0x30)?;
        if dword(definition.bytes, 0x10) != 377 {
            return Err(invalid("buff fact definition is not type 377"));
        }
        if dword(definition.bytes, 0x2c) != 2 {
            return Err(unsupported("buff fact data type is not 2"));
        }
        let payload = target(containers, entry, 0x40, 0x90)?;
        let value = |field: usize| {
            let offset = payload
                .offset
                .checked_add(field)
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| invalid("buff fact value offset overflow"))?;
            resolve_ability_value209026(
                containers,
                payload.container_index,
                offset,
                ability_container,
                ability_offset,
                mode,
                limits,
            )
        };
        let values = [value(0)?, value(0x30)?, value(0x60)?];
        for (value, tag) in values.iter().zip([434, 64, 444]) {
            if dword(value.value.bytes, 0) != tag {
                return Err(unsupported("buff fact value has unsupported resolved tag"));
            }
        }
        let buff_definition = target(containers, values[1].value, 8, 0x68)?;
        let buff_offset = u32::try_from(buff_definition.offset)
            .map_err(|_| invalid("buff SkillDef offset overflow"))?;
        let buff =
            BuffContent209026::read(containers, buff_definition.container_index, buff_offset)?;
        Ok(Self {
            entry,
            definition,
            payload,
            values,
            raw_integer: dword(values[0].value.bytes, 8) as i32,
            buff,
            raw_time: i64::from_le_bytes(values[2].value.bytes[8..16].try_into().unwrap()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ability_value_content::AbilityValueProvenance209026 as Provenance;
    use crate::packfile::{FLAG_64BIT_POINTERS, Packfile};
    use crate::synthetic::build_packfile;

    const ENTRY: usize = 0;
    const DEF: usize = 0x50;
    const PAYLOAD: usize = 0x90;
    const BUFF: usize = 0x120;
    const BUFF_BODY: usize = 0x190;
    const ABILITY: usize = 0x1b0;
    const BODY: usize = 0x220;
    const DESC: usize = 0x2e0;
    const BASE: usize = 0x310;
    const ROW: usize = 0x330;
    const GROUPS: usize = 0x380;
    const FIRST: usize = 0x3b0;
    const SECOND: usize = 0x3f0;
    const HASH: u32 = 0x1234_abcd;
    const MODE: AbilityValueMode209026 = AbilityValueMode209026 {
        first: false,
        second: false,
    };
    const LIMITS: AbilityValueLimits209026 = AbilityValueLimits209026 {
        max_groups: 2,
        max_combined_rows: 2,
        max_name_units: 20,
    };

    fn put(b: &mut [u8], at: usize, value: u32) {
        b[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn wide(b: &mut [u8], at: usize, value: u64) {
        b[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }

    struct Fixture {
        bytes: Vec<u8>,
        local: Vec<u32>,
        external: Vec<(u32, u32)>,
        strings: Vec<u32>,
        names: Vec<&'static str>,
    }
    impl Fixture {
        fn new(len: usize) -> Self {
            Self {
                bytes: vec![0; len],
                local: vec![],
                external: vec![],
                strings: vec![],
                names: vec!["Time", "time", "Other"],
            }
        }
        fn local(&mut self, slot: usize, at: usize) {
            self.local.push(slot as u32);
            wide(&mut self.bytes, slot, at as u64);
        }
        fn external(&mut self, slot: usize, container: u32, at: usize) {
            self.external.push((slot as u32, container));
            wide(&mut self.bytes, slot, at as u64);
        }
        fn name(&mut self, slot: usize, index: usize) {
            self.strings.push(slot as u32);
            wide(&mut self.bytes, slot, index as u64);
            put(&mut self.bytes, slot + 8, HASH);
        }
        fn variable(&mut self, at: usize, name: usize, tag: u32, value: u64) {
            self.name(at, name);
            put(&mut self.bytes, at + 0x10, tag);
            wide(&mut self.bytes, at + 0x18, value);
        }
        fn pack(&self) -> Packfile {
            fn append(main: &mut Vec<u8>, field: usize, data: &[u8], stride: usize) -> usize {
                let at = main.len();
                put(main, field, (data.len() / stride) as u32);
                main[field + 4..field + 12]
                    .copy_from_slice(&(at as i64 - (field + 4) as i64).to_le_bytes());
                main.extend_from_slice(data);
                at
            }
            let mut main = vec![0; 0x88];
            append(&mut main, 0x7c, &self.bytes, 1);
            append(
                &mut main,
                0x34,
                &self
                    .local
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<_>>(),
                4,
            );
            append(
                &mut main,
                0x40,
                &self
                    .external
                    .iter()
                    .flat_map(|(a, b)| a.to_le_bytes().into_iter().chain(b.to_le_bytes()))
                    .collect::<Vec<_>>(),
                8,
            );
            append(
                &mut main,
                0x58,
                &self
                    .strings
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<_>>(),
                4,
            );
            let table = append(&mut main, 0x70, &vec![0; self.names.len() * 8], 8);
            for (i, name) in self.names.iter().enumerate() {
                let slot = table + i * 8;
                let dest = main.len();
                main[slot..slot + 8].copy_from_slice(&(dest as i64 - slot as i64).to_le_bytes());
                for unit in name.encode_utf16().chain([0]) {
                    main.extend_from_slice(&unit.to_le_bytes());
                }
            }
            Packfile::new(build_packfile(
                FLAG_64BIT_POINTERS,
                *b"cntc",
                &[(*b"Main", 0, main, vec![])],
            ))
            .unwrap()
        }
    }
    fn fixture(named_time: bool) -> Fixture {
        let mut f = Fixture::new(0x430);
        f.local(ENTRY + 0x28, DEF);
        f.local(ENTRY + 0x40, PAYLOAD);
        put(&mut f.bytes, DEF + 0x10, 377);
        put(&mut f.bytes, DEF + 0x2c, 2);
        put(&mut f.bytes, PAYLOAD, 434);
        put(&mut f.bytes, PAYLOAD + 8, 1);
        put(&mut f.bytes, PAYLOAD + 0x30, 64);
        f.local(PAYLOAD + 0x38, BUFF);
        put(&mut f.bytes, BUFF + 0x10, 64);
        put(&mut f.bytes, BUFF + 0x14, 9153);
        put(&mut f.bytes, BUFF + 0x28, 738);
        put(&mut f.bytes, BUFF + 0x58, 1);
        f.local(BUFF + 0x60, BUFF_BODY);
        put(&mut f.bytes, BUFF_BODY + 0x14, 78);
        put(&mut f.bytes, BUFF_BODY + 0x18, 2);
        put(
            &mut f.bytes,
            PAYLOAD + 0x60,
            if named_time { 448 } else { 444 },
        );
        if named_time {
            f.name(PAYLOAD + 0x68, 0);
        } else {
            wide(&mut f.bytes, PAYLOAD + 0x68, 5000);
        }
        put(&mut f.bytes, ABILITY + 0x10, 64);
        f.local(ABILITY + 0x60, BODY);
        f.local(BODY + 0xa8, DESC);
        put(&mut f.bytes, BODY + 0xb0, 1);
        f.local(DESC + 8, BASE);
        f.local(BASE + 8, ROW);
        put(&mut f.bytes, BASE + 0x10, 1);
        f.variable(ROW, 1, 444, 5000);
        f
    }
    fn read<'a>(
        indexes: &[ContentIndex<'a>],
        mode: AbilityValueMode209026,
        limits: AbilityValueLimits209026,
    ) -> Result<SkillBuffFactContent209026<'a>> {
        SkillBuffFactContent209026::read(indexes, 0, ENTRY as u32, 0, ABILITY as u32, mode, limits)
    }

    #[test]
    fn skill_buff_fact_literal_values_and_locations_need_no_ability_context() {
        let p = fixture(false).pack();
        let indexes = [ContentIndex::parse(&p).unwrap()];
        let r =
            SkillBuffFactContent209026::read(&indexes, 0, 0, usize::MAX, u32::MAX, MODE, LIMITS)
                .unwrap();
        assert_eq!(
            (r.raw_integer, r.raw_time, r.buff.definition.key),
            (1, 5000, 738)
        );
        assert_eq!(
            (r.entry.offset, r.definition.offset, r.payload.offset),
            (ENTRY, DEF, PAYLOAD)
        );
        for (i, v) in r.values.iter().enumerate() {
            assert_eq!(
                (v.value.container_index, v.value.offset, v.provenance),
                (0, PAYLOAD + i * 0x30, Provenance::Original)
            );
        }
    }
    #[test]
    fn skill_buff_fact_preserves_signed_integer_and_time_full_width() {
        for (integer, time) in [(i32::MIN, i64::MIN), (-1, -1), (i32::MAX, i64::MAX), (0, 0)] {
            let mut f = fixture(false);
            wide(
                &mut f.bytes,
                PAYLOAD + 8,
                0xfedc_ba98_0000_0000 | u64::from(integer as u32),
            );
            wide(&mut f.bytes, PAYLOAD + 0x68, time as u64);
            let p = f.pack();
            let r = read(&[ContentIndex::parse(&p).unwrap()], MODE, LIMITS).unwrap();
            assert_eq!((r.raw_integer, r.raw_time), (integer, time));
        }
    }
    #[test]
    fn skill_buff_fact_named_time_obeys_modes_first_group_and_fallback() {
        let mut f = fixture(true);
        f.local(DESC + 0x20, GROUPS);
        put(&mut f.bytes, DESC + 0x28, 2);
        for (at, rows, flag, time) in [(GROUPS, FIRST, 4, 6000), (GROUPS + 24, SECOND, 8, 7000)] {
            put(&mut f.bytes, at, flag);
            f.local(at + 8, rows);
            put(&mut f.bytes, at + 0x10, 1);
            f.variable(rows, 1, 444, time);
        }
        for (first, second, time, provenance) in [
            (false, false, 5000, Provenance::Base { row_index: 0 }),
            (
                true,
                false,
                6000,
                Provenance::Override {
                    group_index: 0,
                    row_index: 0,
                },
            ),
            (
                false,
                true,
                7000,
                Provenance::Override {
                    group_index: 1,
                    row_index: 0,
                },
            ),
            (
                true,
                true,
                6000,
                Provenance::Override {
                    group_index: 0,
                    row_index: 0,
                },
            ),
        ] {
            let p = f.pack();
            let r = read(
                &[ContentIndex::parse(&p).unwrap()],
                AbilityValueMode209026 { first, second },
                LIMITS,
            )
            .unwrap();
            assert_eq!((r.raw_time, r.values[2].provenance), (time, provenance));
        }
        wide(&mut f.bytes, FIRST, 2); // First eligible group misses: base, not later group.
        let p = f.pack();
        let r = read(
            &[ContentIndex::parse(&p).unwrap()],
            AbilityValueMode209026 {
                first: true,
                second: true,
            },
            LIMITS,
        )
        .unwrap();
        assert_eq!(
            (r.raw_time, r.values[2].provenance),
            (5000, Provenance::Base { row_index: 0 })
        );
    }
    #[test]
    fn skill_buff_fact_resolves_named_integer_without_recursion() {
        let mut f = fixture(false);
        put(&mut f.bytes, PAYLOAD, 448);
        f.name(PAYLOAD + 8, 0);
        f.variable(ROW, 1, 434, i32::MIN as u32 as u64);
        // variable() adds a duplicate name fixup; keep the pre-existing one.
        f.strings.pop();
        let p = f.pack();
        let r = read(&[ContentIndex::parse(&p).unwrap()], MODE, LIMITS).unwrap();
        assert_eq!(r.raw_integer, i32::MIN);
        assert_eq!(r.values[0].provenance, Provenance::Base { row_index: 0 });
        put(&mut f.bytes, ROW + 0x10, 448);
        let p = f.pack();
        assert!(read(&[ContentIndex::parse(&p).unwrap()], MODE, LIMITS).is_err());
    }
    #[test]
    fn skill_buff_fact_external_named_buff_uses_each_pointer_owner() {
        let mut entry = Fixture::new(0x48);
        entry.external(0x28, 1, 0);
        entry.external(0x40, 2, 0);
        let mut def = Fixture::new(0x30);
        put(&mut def.bytes, 0x10, 377);
        put(&mut def.bytes, 0x2c, 2);
        let mut payload = Fixture::new(0x90);
        put(&mut payload.bytes, 0, 434);
        put(&mut payload.bytes, 8, 1);
        put(&mut payload.bytes, 0x30, 448);
        payload.name(0x38, 0);
        put(&mut payload.bytes, 0x60, 444);
        wide(&mut payload.bytes, 0x68, 5000);
        let mut ability = Fixture::new(0x190);
        put(&mut ability.bytes, 0x10, 64);
        ability.local(0x60, 0x70);
        ability.local(0x118, 0x130);
        put(&mut ability.bytes, 0x120, 1);
        ability.local(0x138, 0x170);
        ability.external(0x178, 4, 0);
        put(&mut ability.bytes, 0x180, 1);
        let mut row = Fixture::new(0x40);
        row.variable(0, 1, 64, 0);
        row.external(0x18, 5, 0);
        let mut buff = Fixture::new(0x84);
        put(&mut buff.bytes, 0x10, 64);
        put(&mut buff.bytes, 0x28, 738);
        put(&mut buff.bytes, 0x58, 1);
        buff.local(0x60, 0x68);
        put(&mut buff.bytes, 0x7c, 78);
        put(&mut buff.bytes, 0x80, 2);
        let packs = [
            entry.pack(),
            def.pack(),
            payload.pack(),
            ability.pack(),
            row.pack(),
            buff.pack(),
        ];
        let indexes = packs
            .iter()
            .map(|p| ContentIndex::parse(p).unwrap())
            .collect::<Vec<_>>();
        let r = SkillBuffFactContent209026::read(&indexes, 0, 0, 3, 0, MODE, LIMITS).unwrap();
        assert_eq!(
            (
                r.definition.container_index,
                r.payload.container_index,
                r.values[1].value.container_index,
                r.values[1].value.offset
            ),
            (1, 2, 4, 0x10)
        );
        assert_eq!(
            (
                r.buff.source_container,
                r.buff.skill_offset,
                r.buff.body_container,
                r.buff.status_key
            ),
            (5, 0, 5, 78)
        );
        assert_eq!(r.values[1].provenance, Provenance::Base { row_index: 0 });
    }
    #[test]
    fn skill_buff_fact_rejects_wrong_definitions_value_tags_and_nonbuff_skill() {
        for (at, value) in [
            (DEF + 0x10, 64),
            (DEF + 0x2c, 0),
            (PAYLOAD, 437),
            (PAYLOAD + 0x30, 434),
            (PAYLOAD + 0x60, 434),
            (BUFF + 0x10, 65),
            (BUFF + 0x58, 0),
        ] {
            let mut f = fixture(false);
            put(&mut f.bytes, at, value);
            let p = f.pack();
            assert!(
                read(&[ContentIndex::parse(&p).unwrap()], MODE, LIMITS).is_err(),
                "field {at:x}"
            );
        }
    }
    #[test]
    fn skill_buff_fact_rejects_malformed_refs_and_short_fixed_ranges() {
        for slot in [0x28, 0x40, PAYLOAD + 0x38] {
            for raw in [0, u64::MAX] {
                let mut f = fixture(false);
                f.local.retain(|x| *x != slot as u32);
                wide(&mut f.bytes, slot, raw);
                let p = f.pack();
                assert!(read(&[ContentIndex::parse(&p).unwrap()], MODE, LIMITS).is_err());
            }
            let mut f = fixture(false);
            f.local.push(slot as u32);
            let p = f.pack();
            assert!(read(&[ContentIndex::parse(&p).unwrap()], MODE, LIMITS).is_err());
            let mut f = fixture(false);
            f.local.retain(|x| *x != slot as u32);
            f.name(slot, 0);
            let p = f.pack();
            assert!(read(&[ContentIndex::parse(&p).unwrap()], MODE, LIMITS).is_err());
        }
        for (slot, len) in [(0x28, 0x30), (0x40, 0x90), (PAYLOAD + 0x38, 0x68)] {
            for short in 0..len {
                let mut f = fixture(false);
                let at = f.bytes.len() - short;
                wide(&mut f.bytes, slot, at as u64);
                let p = f.pack();
                assert!(read(&[ContentIndex::parse(&p).unwrap()], MODE, LIMITS).is_err());
            }
        }
        let p = fixture(false).pack();
        let indexes = [ContentIndex::parse(&p).unwrap()];
        assert!(SkillBuffFactContent209026::read(&indexes, 1, 0, 0, 0, MODE, LIMITS).is_err());
        assert!(
            SkillBuffFactContent209026::read(&indexes, 0, u32::MAX, 0, 0, MODE, LIMITS).is_err()
        );
    }
    #[test]
    fn skill_buff_fact_named_time_enforces_limits_and_missing_context() {
        let f = fixture(true);
        let p = f.pack();
        let indexes = [ContentIndex::parse(&p).unwrap()];
        for limits in [
            AbilityValueLimits209026 {
                max_combined_rows: 0,
                ..LIMITS
            },
            AbilityValueLimits209026 {
                max_name_units: 3,
                ..LIMITS
            },
        ] {
            assert!(read(&indexes, MODE, limits).is_err());
        }
        assert!(
            SkillBuffFactContent209026::read(&indexes, 0, 0, usize::MAX, u32::MAX, MODE, LIMITS)
                .is_err()
        );
        let mut f = fixture(true);
        f.local(DESC + 0x20, GROUPS);
        put(&mut f.bytes, DESC + 0x28, 1);
        let p = f.pack();
        assert!(
            read(
                &[ContentIndex::parse(&p).unwrap()],
                MODE,
                AbilityValueLimits209026 {
                    max_groups: 0,
                    ..LIMITS
                }
            )
            .is_err()
        );
        let mut f = fixture(true);
        wide(&mut f.bytes, ROW, 2);
        let p = f.pack();
        assert!(read(&[ContentIndex::parse(&p).unwrap()], MODE, LIMITS).is_err());
    }
}
