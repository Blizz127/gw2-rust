//! Bounded build-209026 creation equipment content views.
//!
//! Layout evidence: `docs/re/209026-creation-equipment.md`. These readers retain
//! every template and equipment slot, including slot zero. They do not apply the
//! creation preview's mode selection, level matching, or later override passes.

use crate::content::{ContentIndex, ContentReference, ResolvedContent, resolve_reference};
use std::io::{Error, ErrorKind, Result};

/// A source location in the caller's explicitly ordered content-container set.
/// The caller maps `container_index` to archive ID and content digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquipmentContentLocation209026 {
    pub container_index: usize,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocatedEquipmentContent209026<T> {
    pub source: EquipmentContentLocation209026,
    pub value: T,
}

/// Only the equipment-template reference list of a type-137 race/sex definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreationEquipmentReferences209026 {
    pub uid: u32,
    pub count: u32,
    pub reference: Option<ContentReference>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreationEquipmentTemplate209026 {
    pub uid: u32,
    pub level: u32,
    pub profession: u32,
    pub entry_count: u32,
    pub entry_reference: Option<ContentReference>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreationEquipmentEntry209026 {
    pub item_reference: Option<ContentReference>,
    /// Raw equipment slot, not quantity; slot zero is preserved.
    pub equipment_slot: u32,
}

/// Identity and kind only; no armor, weapon, skin or gameplay interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreationEquipmentItem209026 {
    pub uid: u32,
    pub key: u32,
    pub kind: u32,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn bytes_at<'a>(index: &ContentIndex<'a>, offset: u32, length: usize) -> Result<&'a [u8]> {
    let start = offset as usize;
    start
        .checked_add(length)
        .and_then(|end| index.content_bytes().get(start..end))
        .ok_or_else(|| invalid("short creation equipment record"))
}

fn record<'a>(index: &ContentIndex<'a>, offset: u32, length: usize, kind: u32) -> Result<&'a [u8]> {
    let bytes = bytes_at(index, offset, length)?;
    if dword(bytes, 0x10) != kind {
        return Err(invalid("wrong creation equipment content type"));
    }
    Ok(bytes)
}

fn reference(
    index: &ContentIndex<'_>,
    offset: u32,
    field: u32,
) -> Result<Option<ContentReference>> {
    let slot = offset
        .checked_add(field)
        .ok_or_else(|| invalid("equipment pointer slot overflow"))?;
    match index.reference(slot)? {
        Some(ContentReference::String { .. }) => {
            Err(invalid("expected equipment content reference"))
        }
        reference => Ok(reference),
    }
}

fn array_reference(
    index: &ContentIndex<'_>,
    offset: u32,
    field: u32,
    count: u32,
    stride: usize,
) -> Result<Option<ContentReference>> {
    let reference = reference(index, offset, field)?;
    let length = u64::from(count) * stride as u64;
    match reference {
        None if count != 0 => return Err(invalid("null nonempty equipment array")),
        Some(ContentReference::Local { offset }) => {
            usize::try_from(length)
                .ok()
                .and_then(|length| offset.checked_add(length))
                .and_then(|end| index.content_bytes().get(offset..end))
                .ok_or_else(|| invalid("equipment array outside container"))?;
        }
        Some(ContentReference::External { offset, .. }) => {
            offset
                .checked_add(length)
                .ok_or_else(|| invalid("external equipment array overflow"))?;
        }
        _ => {}
    }
    Ok(reference)
}

fn resolve_array<'a>(
    containers: &[ContentIndex<'a>],
    source: usize,
    reference: Option<ContentReference>,
    count: u32,
    stride: usize,
    limit: usize,
) -> Result<Option<ResolvedContent<'a>>> {
    if source >= containers.len() {
        return Err(invalid("equipment source container is absent"));
    }
    let count = usize::try_from(count).map_err(|_| invalid("equipment count overflow"))?;
    if count > limit {
        return Err(invalid("equipment array exceeds element limit"));
    }
    let length = count
        .checked_mul(stride)
        .ok_or_else(|| invalid("equipment array length overflow"))?;
    match reference {
        Some(reference) => resolve_reference(containers, source, reference, length).map(Some),
        None if count == 0 => Ok(None),
        None => Err(invalid("null nonempty equipment array")),
    }
}

fn location(
    container_index: usize,
    base: usize,
    row: u32,
    stride: usize,
) -> Result<EquipmentContentLocation209026> {
    let offset = (row as usize)
        .checked_mul(stride)
        .and_then(|relative| base.checked_add(relative))
        .and_then(|offset| u32::try_from(offset).ok())
        .ok_or_else(|| invalid("equipment location exceeds content index width"))?;
    Ok(EquipmentContentLocation209026 {
        container_index,
        offset,
    })
}

impl CreationEquipmentReferences209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0x64, 137)?;
        let count = dword(bytes, 0x60);
        Ok(Self {
            uid: dword(bytes, 0x14),
            count,
            reference: array_reference(index, offset, 0x58, count, 8)?,
        })
    }

    /// Preserve reference order and duplicates. Null template entries are errors.
    pub fn templates(
        &self,
        containers: &[ContentIndex<'_>],
        source: usize,
        limit: usize,
    ) -> Result<Vec<LocatedEquipmentContent209026<CreationEquipmentTemplate209026>>> {
        let Some(array) = resolve_array(containers, source, self.reference, self.count, 8, limit)?
        else {
            return Ok(Vec::new());
        };
        (0..self.count)
            .map(|row| {
                let slot = location(array.container_index, array.offset, row, 8)?;
                let target = reference(&containers[slot.container_index], slot.offset, 0)?
                    .ok_or_else(|| invalid("null creation equipment template"))?;
                let target = resolve_reference(containers, slot.container_index, target, 0x44)?;
                let source = location(target.container_index, target.offset, 0, 1)?;
                Ok(LocatedEquipmentContent209026 {
                    source,
                    value: CreationEquipmentTemplate209026::read(
                        &containers[source.container_index],
                        source.offset,
                    )?,
                })
            })
            .collect()
    }
}

impl CreationEquipmentTemplate209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0x44, 139)?;
        let entry_count = dword(bytes, 0x40);
        Ok(Self {
            uid: dword(bytes, 0x14),
            level: dword(bytes, 0x28),
            profession: dword(bytes, 0x2c),
            entry_count,
            entry_reference: array_reference(index, offset, 0x38, entry_count, 16)?,
        })
    }

    /// Preserve all entries, including empty item references and equipment slot 0.
    pub fn entries(
        &self,
        containers: &[ContentIndex<'_>],
        source: usize,
        limit: usize,
    ) -> Result<Vec<LocatedEquipmentContent209026<CreationEquipmentEntry209026>>> {
        let Some(array) = resolve_array(
            containers,
            source,
            self.entry_reference,
            self.entry_count,
            16,
            limit,
        )?
        else {
            return Ok(Vec::new());
        };
        (0..self.entry_count)
            .map(|row| {
                let source = location(array.container_index, array.offset, row, 16)?;
                Ok(LocatedEquipmentContent209026 {
                    source,
                    value: CreationEquipmentEntry209026::read(
                        &containers[source.container_index],
                        source.offset,
                    )?,
                })
            })
            .collect()
    }
}

impl CreationEquipmentEntry209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = bytes_at(index, offset, 16)?;
        Ok(Self {
            item_reference: reference(index, offset, 0)?,
            equipment_slot: dword(bytes, 8),
        })
    }

    pub fn item(
        &self,
        containers: &[ContentIndex<'_>],
        source: usize,
    ) -> Result<Option<LocatedEquipmentContent209026<CreationEquipmentItem209026>>> {
        if source >= containers.len() {
            return Err(invalid("equipment source container is absent"));
        }
        let Some(reference) = self.item_reference else {
            return Ok(None);
        };
        let target = resolve_reference(containers, source, reference, 0x30)?;
        let source = location(target.container_index, target.offset, 0, 1)?;
        Ok(Some(LocatedEquipmentContent209026 {
            source,
            value: CreationEquipmentItem209026::read(
                &containers[source.container_index],
                source.offset,
            )?,
        }))
    }
}

impl CreationEquipmentItem209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0x30, 35)?;
        Ok(Self {
            uid: dword(bytes, 0x14),
            key: dword(bytes, 0x28),
            kind: dword(bytes, 0x2c),
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

    fn pointer(bytes: &mut [u8], at: usize, value: u64) {
        bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn pack(content: &[u8], local: &[u32], external: &[(u32, u32)], strings: &[u32]) -> Packfile {
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
            (
                0x58,
                4,
                strings.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
            (
                0x70,
                8,
                if strings.is_empty() {
                    vec![]
                } else {
                    vec![0; 8]
                },
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
    fn creation_equipment_cross_container_provenance_and_slot_zero() {
        let mut a = vec![0; 0x100];
        put(&mut a, 0x30, 137);
        put(&mut a, 0x34, 13);
        pointer(&mut a, 0x78, 0x90);
        put(&mut a, 0x80, 2);
        pointer(&mut a, 0x90, 0x20);
        pointer(&mut a, 0x98, 0x20); // Duplicate reference must remain visible.
        put(&mut a, 0xd0, 35);
        put(&mut a, 0xd4, 55);
        put(&mut a, 0xe8, 555);
        put(&mut a, 0xec, 24);
        let mut b = vec![0; 0x110];
        put(&mut b, 0x30, 139);
        put(&mut b, 0x34, 17);
        put(&mut b, 0x48, 1);
        put(&mut b, 0x4c, 7);
        pointer(&mut b, 0x58, 0x80);
        put(&mut b, 0x60, 2);
        pointer(&mut b, 0x80, 0xc0);
        put(&mut b, 0x88, 0);
        pointer(&mut b, 0x90, 0xd0);
        put(&mut b, 0x98, 29);
        put(&mut b, 0xe0, 35);
        put(&mut b, 0xe4, 66);
        put(&mut b, 0xf8, 666);
        put(&mut b, 0xfc, 0);
        let packs = [
            pack(&a, &[0x78], &[(0x90, 1), (0x98, 1)], &[]),
            pack(&b, &[0x58, 0x90], &[(0x80, 0)], &[]),
        ];
        let containers = packs
            .iter()
            .map(ContentIndex::parse)
            .collect::<Result<Vec<_>>>()
            .unwrap();
        let refs = CreationEquipmentReferences209026::read(&containers[0], 0x20).unwrap();
        assert_eq!((refs.uid, refs.count), (13, 2));
        assert!(refs.templates(&containers, 0, 1).is_err());
        let templates = refs.templates(&containers, 0, 2).unwrap();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0], templates[1]);
        assert_eq!(
            templates[0].source,
            EquipmentContentLocation209026 {
                container_index: 1,
                offset: 0x20
            }
        );
        let template = templates[0].value;
        assert_eq!(
            (template.uid, template.level, template.profession),
            (17, 1, 7)
        );
        assert!(template.entries(&containers, 1, 1).is_err());
        let entries = template.entries(&containers, 1, 2).unwrap();
        assert_eq!(
            entries
                .iter()
                .map(|e| e.value.equipment_slot)
                .collect::<Vec<_>>(),
            [0, 29]
        );
        assert_eq!(
            entries[0].source,
            EquipmentContentLocation209026 {
                container_index: 1,
                offset: 0x80
            }
        );
        let item = entries[0].value.item(&containers, 1).unwrap().unwrap();
        assert_eq!(
            item.source,
            EquipmentContentLocation209026 {
                container_index: 0,
                offset: 0xc0
            }
        );
        assert_eq!(
            item.value,
            CreationEquipmentItem209026 {
                uid: 55,
                key: 555,
                kind: 24
            }
        );
        let item = entries[1].value.item(&containers, 1).unwrap().unwrap();
        assert_eq!(
            item.source,
            EquipmentContentLocation209026 {
                container_index: 1,
                offset: 0xd0
            }
        );
        assert_eq!(
            item.value,
            CreationEquipmentItem209026 {
                uid: 66,
                key: 666,
                kind: 0
            }
        );
    }

    #[test]
    fn creation_equipment_rejects_short_records_types_and_extreme_offsets() {
        for (ty, length) in [(137, 0x64), (139, 0x44), (35, 0x30)] {
            let mut bytes = vec![0; length];
            put(&mut bytes, 0x10, ty);
            for cut in 0..length {
                let pf = pack(&bytes[..cut], &[], &[], &[]);
                let index = ContentIndex::parse(&pf).unwrap();
                assert!(record(&index, 0, length, ty).is_err());
            }
            let pf = pack(&bytes, &[], &[], &[]);
            let index = ContentIndex::parse(&pf).unwrap();
            assert!(record(&index, 0, length, ty + 1).is_err());
            assert!(record(&index, u32::MAX, length, ty).is_err());
        }
        let pf = pack(&[0; 15], &[], &[], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        assert!(CreationEquipmentEntry209026::read(&index, 0).is_err());
        assert!(CreationEquipmentEntry209026::read(&index, u32::MAX).is_err());
    }

    #[test]
    fn creation_equipment_empty_missing_and_bad_references_are_distinct() {
        let mut bytes = vec![0; 0xa0];
        put(&mut bytes, 0x10, 139);
        let pf = pack(&bytes, &[], &[], &[]);
        let containers = [ContentIndex::parse(&pf).unwrap()];
        let template = CreationEquipmentTemplate209026::read(&containers[0], 0).unwrap();
        assert!(template.entries(&containers, 0, 0).unwrap().is_empty());
        assert!(template.entries(&containers, 1, 0).is_err());
        let empty = CreationEquipmentEntry209026::read(&containers[0], 0x80).unwrap();
        assert_eq!(empty.item(&containers, 0).unwrap(), None);
        assert!(empty.item(&containers, 1).is_err());

        put(&mut bytes, 0x40, 1);
        let pf = pack(&bytes, &[], &[], &[]);
        assert!(
            CreationEquipmentTemplate209026::read(&ContentIndex::parse(&pf).unwrap(), 0).is_err()
        );
        pointer(&mut bytes, 0x38, 0x98); // Only eight bytes remain, needs sixteen.
        let pf = pack(&bytes, &[0x38], &[], &[]);
        assert!(
            CreationEquipmentTemplate209026::read(&ContentIndex::parse(&pf).unwrap(), 0).is_err()
        );
        pointer(&mut bytes, 0x38, u64::MAX - 7);
        let pf = pack(&bytes, &[], &[(0x38, 0)], &[]);
        assert!(
            CreationEquipmentTemplate209026::read(&ContentIndex::parse(&pf).unwrap(), 0).is_err()
        );
        pointer(&mut bytes, 0x38, 0);
        let pf = pack(&bytes, &[], &[], &[0x38]);
        assert!(
            CreationEquipmentTemplate209026::read(&ContentIndex::parse(&pf).unwrap(), 0).is_err()
        );
        let pf = pack(&bytes, &[], &[(0x38, 1)], &[]);
        let containers = [ContentIndex::parse(&pf).unwrap()];
        let template = CreationEquipmentTemplate209026::read(&containers[0], 0).unwrap();
        assert!(template.entries(&containers, 0, 1).is_err());
        // Null entry in a nonempty race/sex template list is never silently skipped.
        put(&mut bytes, 0x10, 137);
        put(&mut bytes, 0x60, 1);
        pointer(&mut bytes, 0x58, 0x80);
        let pf = pack(&bytes, &[0x58], &[], &[]);
        let containers = [ContentIndex::parse(&pf).unwrap()];
        let refs = CreationEquipmentReferences209026::read(&containers[0], 0).unwrap();
        assert!(refs.templates(&containers, 0, 1).is_err());
        // A classified local zero offset is a real reference, not null. It
        // resolves here to type 137, which is neither a template nor an item.
        let pf = pack(&bytes, &[0x58, 0x80], &[], &[]);
        let containers = [ContentIndex::parse(&pf).unwrap()];
        let refs = CreationEquipmentReferences209026::read(&containers[0], 0).unwrap();
        assert!(refs.templates(&containers, 0, 1).is_err());
        let entry = CreationEquipmentEntry209026::read(&containers[0], 0x80).unwrap();
        assert!(entry.item(&containers, 0).is_err());
    }
}
