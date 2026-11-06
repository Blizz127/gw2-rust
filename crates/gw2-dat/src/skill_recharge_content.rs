//! Build-209026 ability recharge/count content and ordered context overrides.
//!
//! These are the content inputs to client consumers, before modifiers. They do
//! not select server timer fields, the category-zero Word, or a cast schedule.
//! See `docs/re/209026-skill-recharge-content.md` for source and scope.

use crate::content::{ContentIndex, ResolvedContent, resolve_reference};
use crate::skill_palette_content::SkillDefinition209026;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillContextValue209026 {
    pub context: u32,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillRechargeContent209026 {
    pub skill_key: u32,
    /// Ability +0x90, before display modifiers or any server timing policy.
    pub base_recharge: u32,
    /// Ability +0xf0, the category-one content maximum; zero remains valid.
    pub base_available_count: u32,
    /// Original array order is significant: the client uses the first match.
    pub recharge_overrides: Vec<SkillContextValue209026>,
    pub count_overrides: Vec<SkillContextValue209026>,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn dword(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn direct<'a>(
    containers: &[ContentIndex<'a>],
    source: usize,
    offset: usize,
    length: usize,
) -> Result<ResolvedContent<'a>> {
    let container = containers
        .get(source)
        .ok_or_else(|| invalid("skill content source is absent"))?;
    let slot =
        u32::try_from(offset).map_err(|_| invalid("skill content reference offset overflow"))?;
    let reference = container
        .reference(slot)?
        .ok_or_else(|| invalid("skill content reference is null"))?;
    resolve_reference(containers, source, reference, length)
}

fn rows(
    containers: &[ContentIndex<'_>],
    body: ResolvedContent<'_>,
    field: usize,
    count: u32,
    stride: usize,
    value_offset: usize,
) -> Result<Vec<SkillContextValue209026>> {
    let slot = body
        .offset
        .checked_add(field)
        .ok_or_else(|| invalid("skill override slot overflow"))?;
    let slot = u32::try_from(slot).map_err(|_| invalid("skill override slot overflow"))?;
    let reference = containers[body.container_index].reference(slot)?;
    let count = usize::try_from(count).map_err(|_| invalid("skill override count overflow"))?;
    let length = count
        .checked_mul(stride)
        .ok_or_else(|| invalid("skill override extent overflow"))?;
    let Some(reference) = reference else {
        return if count == 0 {
            Ok(Vec::new())
        } else {
            Err(invalid("nonempty skill override array is null"))
        };
    };
    let data = resolve_reference(containers, body.container_index, reference, length)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| invalid("skill override allocation failed"))?;
    for row in data.bytes.chunks_exact(stride) {
        result.push(SkillContextValue209026 {
            context: dword(row, 0),
            value: dword(row, value_offset),
        });
    }
    Ok(result)
}

impl SkillRechargeContent209026 {
    /// Read a type64 skill whose +0x58 tag is zero (ability body at +0x60).
    /// Resolve only classified references in the supplied container ordering.
    /// `max_overrides` caps the combined number of recharge/count override rows.
    /// A classified reference to offset zero is distinct from a null pointer.
    pub fn read(
        containers: &[ContentIndex<'_>],
        source_container: usize,
        skill_offset: u32,
        max_overrides: usize,
    ) -> Result<Self> {
        let source = containers
            .get(source_container)
            .ok_or_else(|| invalid("skill content source is absent"))?;
        let definition = SkillDefinition209026::read(source, skill_offset)?;
        let offset = skill_offset as usize;
        let bytes = offset
            .checked_add(0x68)
            .and_then(|end| source.content_bytes().get(offset..end))
            .ok_or_else(|| invalid("short skill ability reference"))?;
        if dword(bytes, 0x58) != 0 {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "skill content is not an ability",
            ));
        }
        let slot = offset
            .checked_add(0x60)
            .ok_or_else(|| invalid("skill ability slot overflow"))?;
        let body = direct(containers, source_container, slot, 0x19c)?;
        let recharge_count = dword(body.bytes, 0x168);
        let count_count = dword(body.bytes, 0x198);
        let combined = u64::from(recharge_count) + u64::from(count_count);
        if combined > max_overrides as u64 {
            return Err(invalid("skill override arrays exceed element limit"));
        }
        Ok(Self {
            skill_key: definition.key,
            base_recharge: dword(body.bytes, 0x90),
            base_available_count: dword(body.bytes, 0xf0),
            recharge_overrides: rows(containers, body, 0x160, recharge_count, 0x10, 8)?,
            count_overrides: rows(containers, body, 0x190, count_count, 8, 4)?,
        })
    }

    /// First matching context row, otherwise the base. No modifier is applied.
    pub fn recharge_for_context(&self, context: u32) -> u32 {
        self.recharge_overrides
            .iter()
            .find(|row| row.context == context)
            .map_or(self.base_recharge, |row| row.value)
    }

    /// First matching context row, otherwise the base; zero is not replaced.
    pub fn available_count_for_context(&self, context: u32) -> u32 {
        self.count_overrides
            .iter()
            .find(|row| row.context == context)
            .map_or(self.base_available_count, |row| row.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packfile::{FLAG_64BIT_POINTERS, Packfile};
    use crate::synthetic::build_packfile;

    const SKILL: u32 = 0x40;
    const BODY: usize = 0xc0;
    const RECHARGE: usize = 0x280;
    const COUNTS: usize = 0x2c0;

    fn put(data: &mut [u8], at: usize, value: u32) {
        data[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn pointer(data: &mut [u8], at: usize, value: u64) {
        data[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    fn fixture() -> Vec<u8> {
        // Hand-authored schema fixture, not copied frozen asset bytes.
        let mut data = vec![0; 0x300];
        put(&mut data, SKILL as usize + 0x10, 64);
        put(&mut data, SKILL as usize + 0x28, 10176);
        pointer(&mut data, SKILL as usize + 0x60, BODY as u64);
        put(&mut data, BODY + 0x90, 20000);
        put(&mut data, BODY + 0xf0, 3);
        pointer(&mut data, BODY + 0x160, RECHARGE as u64);
        put(&mut data, BODY + 0x168, 3);
        pointer(&mut data, BODY + 0x190, COUNTS as u64);
        put(&mut data, BODY + 0x198, 3);
        for (i, (context, value)) in [(2, 25000), (2, 99999), (3, 0)].into_iter().enumerate() {
            put(&mut data, RECHARGE + i * 16, context);
            put(&mut data, RECHARGE + i * 16 + 8, value);
        }
        for (i, (context, value)) in [(2, 5), (2, 6), (3, 0)].into_iter().enumerate() {
            put(&mut data, COUNTS + i * 8, context);
            put(&mut data, COUNTS + i * 8 + 4, value);
        }
        data
    }
    fn pack(content: &[u8], local: &[u32], external: &[(u32, u32)], strings: &[u32]) -> Packfile {
        let mut main = vec![0; 0x88];
        for (field, stride, data) in [
            (0x7c, 1, content.to_vec()),
            (
                0x34,
                4,
                local.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
            (
                0x40,
                8,
                external
                    .iter()
                    .flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat())
                    .collect(),
            ),
            (
                0x58,
                4,
                strings.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
        ] {
            let start = main.len();
            put(&mut main, field, (data.len() / stride) as u32);
            main[field + 4..field + 12]
                .copy_from_slice(&(start as i64 - (field + 4) as i64).to_le_bytes());
            main.extend(data);
        }
        Packfile::new(build_packfile(
            FLAG_64BIT_POINTERS,
            *b"cntc",
            &[(*b"Main", 0, main, vec![])],
        ))
        .unwrap()
    }
    fn local() -> [u32; 3] {
        [SKILL + 0x60, BODY as u32 + 0x160, BODY as u32 + 0x190]
    }
    fn parse(data: &[u8], local: &[u32], max: usize) -> Result<SkillRechargeContent209026> {
        let pf = pack(data, local, &[], &[]);
        SkillRechargeContent209026::read(&[ContentIndex::parse(&pf)?], 0, SKILL, max)
    }

    #[test]
    fn skill_recharge_content_preserves_order_and_uses_first_match_or_base() {
        let data = parse(&fixture(), &local(), 6).unwrap();
        assert_eq!(data.skill_key, 10176);
        assert_eq!(
            data.recharge_overrides
                .iter()
                .map(|r| r.value)
                .collect::<Vec<_>>(),
            [25000, 99999, 0]
        );
        for (context, recharge, count) in [
            (1, 20000, 3),
            (2, 25000, 5),
            (3, 0, 0),
            (u32::MAX, 20000, 3),
        ] {
            assert_eq!(data.recharge_for_context(context), recharge);
            assert_eq!(data.available_count_for_context(context), count);
        }
    }

    #[test]
    fn skill_recharge_content_resolves_external_body_and_arrays_in_body_container() {
        let original = fixture();
        let mut skill = original[..BODY].to_vec();
        pointer(&mut skill, SKILL as usize + 0x60, 0);
        let mut body = original[BODY..].to_vec();
        pointer(&mut body, 0x160, 0);
        pointer(&mut body, 0x190, (COUNTS - BODY) as u64);
        let a = pack(&skill, &[], &[(SKILL + 0x60, 1)], &[]);
        let b = pack(&body, &[0x190], &[(0x160, 2)], &[]);
        let c = pack(&original[RECHARGE..RECHARGE + 48], &[], &[], &[]);
        let indexes = [
            ContentIndex::parse(&a).unwrap(),
            ContentIndex::parse(&b).unwrap(),
            ContentIndex::parse(&c).unwrap(),
        ];
        let data = SkillRechargeContent209026::read(&indexes, 0, SKILL, 6).unwrap();
        assert_eq!(data.recharge_for_context(2), 25000);
        assert_eq!(data.available_count_for_context(2), 5);
        assert!(SkillRechargeContent209026::read(&indexes[..2], 0, SKILL, 6).is_err());
    }

    #[test]
    fn skill_recharge_content_null_and_classified_zero_are_distinct() {
        let mut data = fixture();
        put(&mut data, BODY + 0x168, 1);
        put(&mut data, BODY + 0x198, 0);
        pointer(&mut data, BODY + 0x160, 0);
        pointer(&mut data, BODY + 0x190, 0);
        put(&mut data, 0, 9);
        put(&mut data, 8, u32::MAX);
        let value = parse(&data, &[SKILL + 0x60, BODY as u32 + 0x160], 1).unwrap();
        assert_eq!(value.recharge_for_context(9), u32::MAX);
        assert!(value.count_overrides.is_empty());
        assert!(parse(&data, &[SKILL + 0x60], 1).is_err());
        put(&mut data, BODY + 0x168, 0);
        assert!(parse(&data, &[SKILL + 0x60], 0).is_ok());
        pointer(&mut data, SKILL as usize + 0x60, 0);
        assert!(parse(&data, &[], 0).is_err());
    }

    #[test]
    fn skill_recharge_content_rejects_bad_types_references_extents_and_limits() {
        assert!(parse(&fixture(), &local(), 5).is_err());
        for (at, value) in [
            (SKILL as usize + 0x10, 63),
            (SKILL as usize + 0x58, 1),
            (BODY + 0x168, u32::MAX),
        ] {
            let mut data = fixture();
            put(&mut data, at, value);
            assert!(parse(&data, &local(), 6).is_err());
        }
        let mut data = fixture();
        pointer(&mut data, BODY + 0x160, 0x2ff);
        assert!(parse(&data, &local(), 6).is_err());
        assert!(parse(&fixture()[..BODY + 0x190], &local(), 6).is_err());
        let pf = pack(
            &fixture(),
            &[BODY as u32 + 0x160, BODY as u32 + 0x190],
            &[],
            &[SKILL + 0x60],
        );
        assert!(
            ContentIndex::parse(&pf)
                .and_then(|index| SkillRechargeContent209026::read(&[index], 0, SKILL, 6))
                .is_err()
        );
        let pf = pack(&fixture(), &local(), &[], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        assert!(SkillRechargeContent209026::read(&[index], 1, SKILL, 6).is_err());
        assert!(SkillRechargeContent209026::read(&[], 0, u32::MAX, 6).is_err());
    }
}
