//! Bounded build-209026 skill palette records, without gameplay eligibility.
//! See `docs/re/209026-guardian-palette-eligibility.md`.

use crate::content::{ContentIndex, ContentReference, ResolvedContent, resolve_reference};
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteRecordArray209026 {
    pub count: u32,
    pub reference: Option<ContentReference>,
    stride: usize,
}

impl PaletteRecordArray209026 {
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Resolve only within the supplied ordered container set and element limit.
    pub fn resolve<'a>(
        &self,
        containers: &[ContentIndex<'a>],
        source_container: usize,
        max_elements: usize,
    ) -> Result<Option<ResolvedContent<'a>>> {
        if source_container >= containers.len() {
            return Err(invalid("source palette container is absent"));
        }
        let count = usize::try_from(self.count).map_err(|_| invalid("palette count overflow"))?;
        if count > max_elements {
            return Err(invalid("palette array exceeds element limit"));
        }
        let length = count
            .checked_mul(self.stride)
            .ok_or_else(|| invalid("palette array extent overflow"))?;
        match self.reference {
            Some(reference) => {
                resolve_reference(containers, source_container, reference, length).map(Some)
            }
            None if count == 0 => Ok(None),
            None => Err(invalid("null nonempty palette array")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillPaletteDefinition209026 {
    pub uid: u32,
    pub key: u32,
    pub flags: u32,
    pub kind: u32,
    pub rows: PaletteRecordArray209026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillPaletteRow209026 {
    pub profession: u32,
    pub category: u32,
    pub entries: PaletteRecordArray209026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillPaletteEntry209026 {
    pub skill: Option<ContentReference>,
    pub minimum_level: u32,
    /// Raw selector at +0x14; not a complete mode/eligibility decision.
    pub context_mode: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillDefinition209026 {
    pub uid: u32,
    pub key: u32,
    pub flags: u32,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}
fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn record<'a>(index: &ContentIndex<'a>, offset: u32, length: usize) -> Result<&'a [u8]> {
    let start = offset as usize;
    start
        .checked_add(length)
        .and_then(|end| index.content_bytes().get(start..end))
        .ok_or_else(|| invalid("short skill palette record"))
}
fn reference(
    index: &ContentIndex<'_>,
    offset: u32,
    field: u32,
) -> Result<Option<ContentReference>> {
    let slot = offset
        .checked_add(field)
        .ok_or_else(|| invalid("palette pointer slot overflow"))?;
    match index.reference(slot)? {
        Some(ContentReference::String { .. }) => {
            Err(invalid("palette pointer cannot reference a string"))
        }
        value => Ok(value),
    }
}
fn array(
    index: &ContentIndex<'_>,
    offset: u32,
    field: u32,
    count: u32,
    stride: usize,
) -> Result<PaletteRecordArray209026> {
    let reference = reference(index, offset, field)?;
    if count != 0 && reference.is_none() {
        return Err(invalid("null nonempty palette array"));
    }
    Ok(PaletteRecordArray209026 {
        count,
        reference,
        stride,
    })
}
impl SkillPaletteDefinition209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0x44)?;
        if dword(bytes, 0x10) != 65 {
            return Err(invalid("expected skill palette type 65"));
        }
        Ok(Self {
            uid: dword(bytes, 0x14),
            key: dword(bytes, 0x28),
            flags: dword(bytes, 0x2c),
            kind: dword(bytes, 0x40),
            rows: array(index, offset, 0x30, dword(bytes, 0x38), 0x18)?,
        })
    }
}
impl SkillPaletteRow209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0x18)?;
        Ok(Self {
            profession: dword(bytes, 0),
            category: dword(bytes, 4),
            entries: array(index, offset, 8, dword(bytes, 0x10), 0x70)?,
        })
    }
}
impl SkillPaletteEntry209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0x70)?;
        Ok(Self {
            skill: reference(index, offset, 0)?,
            minimum_level: dword(bytes, 0x1c),
            context_mode: dword(bytes, 0x14),
        })
    }
}
impl SkillDefinition209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0x3c)?;
        if dword(bytes, 0x10) != 64 {
            return Err(invalid("expected skill type 64"));
        }
        Ok(Self {
            uid: dword(bytes, 0x14),
            key: dword(bytes, 0x28),
            flags: dword(bytes, 0x38),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        packfile::{FLAG_64BIT_POINTERS, Packfile},
        synthetic::build_packfile,
    };

    fn put(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn pack(content: &[u8], local: &[u32], external: &[(u32, u32)]) -> Packfile {
        let mut main = vec![0; 0x88];
        for (field, stride, bytes) in [
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
        ] {
            let start = main.len();
            put(&mut main, field, (bytes.len() / stride) as u32);
            main[field + 4..field + 12]
                .copy_from_slice(&(start as i64 - (field + 4) as i64).to_le_bytes());
            main.extend(bytes);
        }
        Packfile::new(build_packfile(
            FLAG_64BIT_POINTERS,
            *b"cntc",
            &[(*b"Main", 0, main, vec![])],
        ))
        .unwrap()
    }

    #[test]
    fn skill_palette_content_resolves_cross_container_chain() {
        // Definition at 0x20 -> external row at zero -> local entry -> external skill.
        let mut source = vec![0; 0xac];
        put(&mut source, 0x30, 65);
        put(&mut source, 0x34, 9);
        put(&mut source, 0x48, 127);
        put(&mut source, 0x4c, 0x8000_0001);
        put(&mut source, 0x58, 1);
        put(&mut source, 0x60, 4);
        put(&mut source, 0x80, 64);
        put(&mut source, 0x84, 17);
        put(&mut source, 0x98, 9083);
        put(&mut source, 0xa8, 1 << 26);
        let mut rows = vec![0; 0x90];
        put(&mut rows, 0, 1);
        put(&mut rows, 4, 2);
        put(&mut rows, 8, 0x20);
        put(&mut rows, 0x10, 1);
        put(&mut rows, 0x20, 0x70);
        put(&mut rows, 0x34, 3);
        put(&mut rows, 0x3c, 1);
        let pfs = [
            pack(&source, &[], &[(0x50, 1)]),
            pack(&rows, &[8], &[(0x20, 0)]),
        ];
        let indexes: Vec<_> = pfs
            .iter()
            .map(ContentIndex::parse)
            .collect::<Result<_>>()
            .unwrap();
        let palette = SkillPaletteDefinition209026::read(&indexes[0], 0x20).unwrap();
        assert_eq!(
            (palette.uid, palette.key, palette.flags, palette.kind),
            (9, 127, 0x8000_0001, 4)
        );
        assert!(palette.rows.resolve(&indexes, 0, 0).is_err());
        assert!(palette.rows.resolve(&indexes[..1], 0, 1).is_err());
        let resolved = palette.rows.resolve(&indexes, 0, 1).unwrap().unwrap();
        assert_eq!(
            (
                resolved.container_index,
                resolved.offset,
                resolved.bytes.len()
            ),
            (1, 0, 0x18)
        );
        let row = SkillPaletteRow209026::read(&indexes[1], 0).unwrap();
        assert_eq!(
            (row.profession, row.category, row.entries.stride()),
            (1, 2, 0x70)
        );
        let resolved = row.entries.resolve(&indexes, 1, 1).unwrap().unwrap();
        assert_eq!(
            (
                resolved.container_index,
                resolved.offset,
                resolved.bytes.len()
            ),
            (1, 0x20, 0x70)
        );
        let entry = SkillPaletteEntry209026::read(&indexes[1], 0x20).unwrap();
        assert_eq!((entry.minimum_level, entry.context_mode), (1, 3));
        let skill = resolve_reference(&indexes, 1, entry.skill.unwrap(), 0x3c).unwrap();
        let skill =
            SkillDefinition209026::read(&indexes[skill.container_index], skill.offset as u32)
                .unwrap();
        assert_eq!((skill.uid, skill.key, skill.flags), (17, 9083, 1 << 26));
    }

    #[test]
    fn skill_palette_content_record_bounds_and_types() {
        for (kind, length) in [(0, 0x44), (1, 0x18), (2, 0x70), (3, 0x3c)] {
            let mut content = vec![0; length];
            if kind == 0 {
                put(&mut content, 0x10, 65);
            }
            if kind == 3 {
                put(&mut content, 0x10, 64);
            }
            for cut in 0..=length {
                let pf = pack(&content[..cut], &[], &[]);
                let index = ContentIndex::parse(&pf).unwrap();
                let read = |offset| match kind {
                    0 => SkillPaletteDefinition209026::read(&index, offset).map(|_| ()),
                    1 => SkillPaletteRow209026::read(&index, offset).map(|_| ()),
                    2 => SkillPaletteEntry209026::read(&index, offset).map(|_| ()),
                    _ => SkillDefinition209026::read(&index, offset).map(|_| ()),
                };
                assert_eq!(read(0).is_ok(), cut == length);
                assert!(read(u32::MAX).is_err());
            }
        }
        let mut content = vec![0; 0x44];
        for wrong in [0, 53, 64, 66] {
            put(&mut content, 0x10, wrong);
            let pf = pack(&content, &[], &[]);
            assert!(
                SkillPaletteDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0).is_err()
            );
        }
        put(&mut content, 0x10, 65);
        let pf = pack(&content, &[], &[]);
        assert!(SkillDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0).is_err());
    }

    #[test]
    fn skill_palette_content_array_bounds_and_classified_zero() {
        let mut content = vec![0; 0x64];
        put(&mut content, 0x30, 65);
        put(&mut content, 0x58, 1);
        // Classified zero points to a row at offset zero in this same container.
        let pf = pack(&content, &[0x50], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        let palette = SkillPaletteDefinition209026::read(&index, 0x20).unwrap();
        let indexes = [index];
        assert_eq!(
            palette
                .rows
                .resolve(&indexes, 0, 1)
                .unwrap()
                .unwrap()
                .offset,
            0
        );
        assert!(palette.rows.resolve(&indexes, 1, 1).is_err());
        let pf = pack(&content, &[], &[]);
        assert!(
            SkillPaletteDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20).is_err()
        );
        let pf = pack(&content, &[0x50], &[(0x50, 0)]);
        assert!(
            SkillPaletteDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20).is_err()
        );
        put(&mut content, 0x50, 0x63);
        let pf = pack(&content, &[0x50], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        let palette = SkillPaletteDefinition209026::read(&index, 0x20).unwrap();
        assert!(palette.rows.resolve(&[index], 0, 1).is_err());
        let pf = pack(&content, &[], &[]);
        assert!(
            SkillPaletteDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20).is_err()
        );
        put(&mut content, 0x50, 0);
        put(&mut content, 0x58, u32::MAX);
        let pf = pack(&content, &[0x50], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        let palette = SkillPaletteDefinition209026::read(&index, 0x20).unwrap();
        let indexes = [index];
        assert!(palette.rows.resolve(&indexes, 0, 1).is_err());
        assert!(palette.rows.resolve(&indexes, 0, usize::MAX).is_err());
        put(&mut content, 0x58, 0);
        let pf = pack(&content, &[], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        let palette = SkillPaletteDefinition209026::read(&index, 0x20).unwrap();
        assert!(palette.rows.resolve(&[index], 0, 0).unwrap().is_none());
    }
}
