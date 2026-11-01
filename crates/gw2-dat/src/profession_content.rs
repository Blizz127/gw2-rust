//! Bounded build-209026 profession, specialization and trait record views.
//! External references are retained, not loaded. See
//! `docs/re/209026-specialization-content.md` for evidence and scope.

use std::io::{Error, ErrorKind, Result};

use crate::content::{ContentIndex, ContentReference};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentPointerArray209026 {
    pub count: u32,
    /// Local byte extent is checked. External destination bounds and the
    /// individual pointed-to records must be checked by the caller.
    pub reference: Option<ContentReference>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfessionDefinition209026 {
    /// Profession enum, not a numeric content key.
    pub profession: u32,
    pub specializations: ContentPointerArray209026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecializationDefinition209026 {
    /// Content UID at +0x14, used by specialization/trait wire messages.
    pub uid: u32,
    /// Data key at +0x28; distinct from the wire UID.
    pub key: u32,
    pub flags: u32,
    pub discriminator: u32,
    pub progress_reference: Option<ContentReference>,
    pub major_traits: ContentPointerArray209026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraitDefinition209026 {
    /// Content UID at +0x14, used by specialization/trait wire messages.
    pub uid: u32,
    /// Data key at +0x28; distinct from the wire UID.
    pub key: u32,
    /// Major-trait consumers compare this with the selected slot plus one.
    pub tier: u32,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn record<'a>(
    index: &ContentIndex<'a>,
    offset: u32,
    length: usize,
    expected_type: u32,
) -> Result<&'a [u8]> {
    let start = offset as usize;
    let bytes = start
        .checked_add(length)
        .and_then(|end| index.content_bytes().get(start..end))
        .ok_or_else(|| invalid("short profession-related content record"))?;
    if word(bytes, 0x10) != expected_type {
        return Err(invalid("wrong profession-related content type"));
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
        .ok_or_else(|| invalid("content pointer slot overflow"))?;
    match index.reference(slot)? {
        Some(ContentReference::String { .. }) => Err(invalid("expected content record reference")),
        value => Ok(value),
    }
}

fn pointer_array(
    index: &ContentIndex<'_>,
    offset: u32,
    field: u32,
    count: u32,
) -> Result<ContentPointerArray209026> {
    let reference = reference(index, offset, field)?;
    let length = u64::from(count) * 8;
    match reference {
        None if count != 0 => return Err(invalid("null nonempty content pointer array")),
        Some(ContentReference::Local { offset }) => {
            let length = usize::try_from(length)
                .map_err(|_| invalid("content pointer array length overflow"))?;
            offset
                .checked_add(length)
                .and_then(|end| index.content_bytes().get(offset..end))
                .ok_or_else(|| invalid("content pointer array outside container"))?;
        }
        Some(ContentReference::External { offset, .. }) => {
            offset
                .checked_add(length)
                .ok_or_else(|| invalid("external content pointer array overflow"))?;
        }
        _ => {}
    }
    Ok(ContentPointerArray209026 { count, reference })
}

impl ProfessionDefinition209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0xc4, 315)?;
        Ok(Self {
            profession: word(bytes, 0xa8),
            specializations: pointer_array(index, offset, 0xb8, word(bytes, 0xc0))?,
        })
    }
}

impl SpecializationDefinition209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0xcc, 386)?;
        Ok(Self {
            uid: word(bytes, 0x14),
            key: word(bytes, 0x28),
            flags: word(bytes, 0x38),
            discriminator: word(bytes, 0xc8),
            progress_reference: reference(index, offset, 0x50)?,
            major_traits: pointer_array(index, offset, 0x58, word(bytes, 0x60))?,
        })
    }
}

impl TraitDefinition209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let bytes = record(index, offset, 0x74, 77)?;
        Ok(Self {
            uid: word(bytes, 0x14),
            key: word(bytes, 0x28),
            tier: word(bytes, 0x70),
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

    #[test]
    fn profession_content_reads_nonzero_offsets_and_preserves_references() {
        let mut data = vec![0; 0x110];
        put(&mut data, 0x30, 315);
        put(&mut data, 0xc8, 9);
        put(&mut data, 0xe0, 2);
        data[0xd8..0xe0].copy_from_slice(&0x100u64.to_le_bytes());
        let pf = pack(&data, &[0xd8], &[], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        assert_eq!(
            ProfessionDefinition209026::read(&index, 0x20).unwrap(),
            ProfessionDefinition209026 {
                profession: 9,
                specializations: ContentPointerArray209026 {
                    count: 2,
                    reference: Some(ContentReference::Local { offset: 0x100 })
                },
            }
        );

        data.fill(0);
        put(&mut data, 0x30, 386);
        put(&mut data, 0x34, 15);
        put(&mut data, 0x48, u32::MAX);
        put(&mut data, 0x58, 0x80000001);
        put(&mut data, 0xe8, 5);
        put(&mut data, 0x80, 2);
        data[0x78..0x80].copy_from_slice(&0x100u64.to_le_bytes());
        let pf = pack(&data, &[0x78], &[(0x70, 7)], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        let spec = SpecializationDefinition209026::read(&index, 0x20).unwrap();
        assert_eq!(
            (spec.uid, spec.key, spec.flags, spec.discriminator),
            (15, u32::MAX, 0x80000001, 5)
        );
        assert_eq!(
            spec.progress_reference,
            Some(ContentReference::External {
                container_index: 7,
                offset: 0
            })
        );
        assert_eq!(
            spec.major_traits.reference,
            Some(ContentReference::Local { offset: 0x100 })
        );

        put(&mut data, 0x30, 77);
        put(&mut data, 0x34, 2373);
        put(&mut data, 0x90, u32::MAX);
        let pf = pack(&data, &[], &[], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        assert_eq!(
            TraitDefinition209026::read(&index, 0x20).unwrap(),
            TraitDefinition209026 {
                uid: 2373,
                key: u32::MAX,
                tier: u32::MAX
            }
        );
    }

    #[test]
    fn profession_content_rejects_short_wrong_type_and_extreme_offsets() {
        for (ty, length) in [(315, 0xc4), (386, 0xcc), (77, 0x74)] {
            let mut data = vec![0; length];
            put(&mut data, 0x10, ty);
            for cut in 0..length {
                let pf = pack(&data[..cut], &[], &[], &[]);
                let index = ContentIndex::parse(&pf).unwrap();
                assert!(record(&index, 0, length, ty).is_err());
            }
            let pf = pack(&data, &[], &[], &[]);
            let index = ContentIndex::parse(&pf).unwrap();
            assert!(record(&index, 0, length, ty + 1).is_err());
            assert!(record(&index, u32::MAX, length, ty).is_err());
        }
    }

    #[test]
    fn profession_content_checks_array_extents_and_reference_classes() {
        let mut data = vec![0; 0xe0];
        put(&mut data, 0x10, 386);
        put(&mut data, 0x60, 2);
        // Unclassified null is invalid for a nonempty array.
        let pf = pack(&data, &[], &[], &[]);
        assert!(
            SpecializationDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0).is_err()
        );
        // A classified external zero offset remains a reference.
        let pf = pack(&data, &[], &[(0x58, u32::MAX)], &[]);
        let spec =
            SpecializationDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0).unwrap();
        assert_eq!(
            spec.major_traits.reference,
            Some(ContentReference::External {
                container_index: u32::MAX,
                offset: 0
            })
        );
        for count in [2, u32::MAX] {
            put(&mut data, 0x60, count);
            data[0x58..0x60].copy_from_slice(&0xd8u64.to_le_bytes());
            let pf = pack(&data, &[0x58], &[], &[]);
            assert!(
                SpecializationDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0)
                    .is_err()
            );
        }
        put(&mut data, 0x60, 2);
        data[0x58..0x60].copy_from_slice(&(u64::MAX - 7).to_le_bytes());
        let pf = pack(&data, &[], &[(0x58, 0)], &[]);
        assert!(
            SpecializationDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0).is_err()
        );
        data[0x58..0x60].fill(0);
        for slot in [0x50, 0x58] {
            let pf = pack(&data, &[], &[(0x58, 0)], &[slot]);
            assert!(
                SpecializationDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0)
                    .is_err()
            );
        }
        put(&mut data, 0x60, 0);
        let pf = pack(&data, &[], &[], &[]);
        assert_eq!(
            SpecializationDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0)
                .unwrap()
                .major_traits,
            ContentPointerArray209026 {
                count: 0,
                reference: None
            }
        );
    }
}
