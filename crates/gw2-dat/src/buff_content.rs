//! Bounded build-209026 buff SkillDef bodies, without effect policy.
//!
//! Native status-counter methods 1412c2c90 / 1412c2db0 read body +0x14 as
//! the status key. For keys other than 94, raw class +0x18 == 2 contributes
//! to the source-named `conditionCount`. Other class values remain unnamed.
//! Evidence: ev_408381750df6698a3facd781a79bd951d305437f7a7306c6d915bdf6cda9c717.
//! The retained sword facts reference SkillDef 738 / UID 9153, status 78,
//! class 2 (ev_9e65ace5129ff98da025e44249f18168504734cfe4b2caf107888c39462c7349).
//! These fields do not establish application, duration, stacking or removal policy.

use crate::content::{ContentIndex, ContentReference, resolve_reference};
use crate::skill_palette_content::SkillDefinition209026;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuffContent209026 {
    pub definition: SkillDefinition209026,
    /// Location of the validated type-64, kind-1 SkillDef.
    pub source_container: usize,
    pub skill_offset: u32,
    /// Original fixup classification, before resolving the body location.
    pub body_reference: ContentReference,
    pub body_container: usize,
    pub body_offset: usize,
    /// Raw body +0x14, including values excluded from native status counters.
    pub status_key: u32,
    /// Raw body +0x18; no complete condition/boon enum is inferred.
    pub class: u32,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

// Callers first validate the required record range.
fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

impl BuffContent209026 {
    /// Read a type-64 SkillDef whose +0x58 kind is 1, then its classified
    /// +0x60 body reference. Only the 0x68-byte SkillDef prefix and 0x1c-byte
    /// body prefix are required; these are not claims about full struct sizes.
    ///
    /// `containers` uses the original external-container ordering. Classified
    /// local/external offset zero is valid; null, string and file references
    /// are rejected. No file is opened, and unknown scalar values are preserved.
    pub fn read(
        containers: &[ContentIndex<'_>],
        source_container: usize,
        skill_offset: u32,
    ) -> Result<Self> {
        let source = containers
            .get(source_container)
            .ok_or_else(|| invalid("buff source container is absent"))?;
        let definition = SkillDefinition209026::read(source, skill_offset)?;
        let offset = skill_offset as usize;
        let bytes = offset
            .checked_add(0x68)
            .and_then(|end| source.content_bytes().get(offset..end))
            .ok_or_else(|| invalid("short buff SkillDef reference"))?;
        if dword(bytes, 0x58) != 1 {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "SkillDef kind is not a buff",
            ));
        }
        let slot = skill_offset
            .checked_add(0x60)
            .ok_or_else(|| invalid("buff body reference slot overflow"))?;
        let body_reference = source
            .reference(slot)?
            .ok_or_else(|| invalid("buff body reference is null"))?;
        let body = resolve_reference(containers, source_container, body_reference, 0x1c)?;
        Ok(Self {
            definition,
            source_container,
            skill_offset,
            body_reference,
            body_container: body.container_index,
            body_offset: body.offset,
            status_key: dword(body.bytes, 0x14),
            class: dword(body.bytes, 0x18),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packfile::{FLAG_64BIT_POINTERS, Packfile};
    use crate::synthetic::build_packfile;

    const SKILL: u32 = 0x40;
    const BODY: usize = 0xc0;
    const SLOT: u32 = SKILL + 0x60;

    fn put(data: &mut [u8], at: usize, value: u32) {
        data[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn pointer(data: &mut [u8], at: usize, value: u64) {
        data[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn fixture(skill: usize, body: usize) -> Vec<u8> {
        // Synthetic field values, not copied proprietary record bytes.
        let mut data = vec![0; (skill + 0x68).max(body + 0x1c)];
        put(&mut data, skill + 0x10, 64);
        put(&mut data, skill + 0x14, 9153);
        put(&mut data, skill + 0x28, 738);
        put(&mut data, skill + 0x38, 0x0400_0000);
        put(&mut data, skill + 0x58, 1);
        pointer(&mut data, skill + 0x60, body as u64);
        put(&mut data, body + 0x14, 78);
        put(&mut data, body + 0x18, 2);
        data
    }

    fn pack(
        content: &[u8],
        local: &[u32],
        external: &[(u32, u32)],
        strings: &[u32],
        files: &[u32],
    ) -> Packfile {
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
                0x4c,
                4,
                files.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
            // A valid string index 0 lets the reader reject the reference kind,
            // rather than failing only because the string table is absent.
            (0x70, 8, vec![0; 8]),
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

    fn read_local(data: &[u8], skill: u32, local: &[u32]) -> Result<BuffContent209026> {
        let pf = pack(data, local, &[], &[], &[]);
        BuffContent209026::read(&[ContentIndex::parse(&pf)?], 0, skill)
    }

    #[test]
    fn buff_content_reads_minimum_local_body_and_preserves_provenance() {
        let value = read_local(&fixture(SKILL as usize, BODY), SKILL, &[SLOT]).unwrap();
        assert_eq!(
            value.definition,
            SkillDefinition209026 {
                uid: 9153,
                key: 738,
                flags: 0x0400_0000
            }
        );
        assert_eq!((value.status_key, value.class), (78, 2));
        assert_eq!((value.source_container, value.skill_offset), (0, SKILL));
        assert_eq!((value.body_container, value.body_offset), (0, BODY));
        assert_eq!(
            value.body_reference,
            ContentReference::Local { offset: BODY }
        );
    }

    #[test]
    fn buff_content_resolves_external_body_from_nonzero_source_owner() {
        let mut source = fixture(SKILL as usize, BODY);
        put(&mut source, BODY + 0x14, 999);
        put(&mut source, BODY + 0x18, 888);
        let target = fixture(SKILL as usize, BODY);
        let a = pack(&source, &[SLOT], &[], &[], &[]);
        let b = pack(&source, &[], &[(SLOT, 2)], &[], &[]);
        let c = pack(&target, &[], &[], &[], &[]);
        let indexes = [
            ContentIndex::parse(&a).unwrap(),
            ContentIndex::parse(&b).unwrap(),
            ContentIndex::parse(&c).unwrap(),
        ];
        let value = BuffContent209026::read(&indexes, 1, SKILL).unwrap();
        assert_eq!((value.status_key, value.class), (78, 2));
        assert_eq!(
            (
                value.source_container,
                value.body_container,
                value.body_offset
            ),
            (1, 2, BODY)
        );
        assert_eq!(
            value.body_reference,
            ContentReference::External {
                container_index: 2,
                offset: BODY as u64
            }
        );
        // The same local classification belongs to container 0, not container 1.
        let local = BuffContent209026::read(&indexes, 0, SKILL).unwrap();
        assert_eq!(
            (local.status_key, local.class, local.body_container),
            (999, 888, 0)
        );
    }

    #[test]
    fn buff_content_accepts_classified_zero_offsets_and_zero_container() {
        let value = read_local(&fixture(0, BODY), 0, &[0x60]).unwrap();
        assert_eq!((value.skill_offset, value.body_offset), (0, BODY));
        let source = fixture(SKILL as usize, 0);
        let value = read_local(&source, SKILL, &[SLOT]).unwrap();
        assert_eq!(
            (value.status_key, value.class, value.body_offset),
            (78, 2, 0)
        );
        let a = pack(&source[..0x1c], &[], &[], &[], &[]);
        let b = pack(&source, &[], &[(SLOT, 0)], &[], &[]);
        let value = BuffContent209026::read(
            &[
                ContentIndex::parse(&a).unwrap(),
                ContentIndex::parse(&b).unwrap(),
            ],
            1,
            SKILL,
        )
        .unwrap();
        assert_eq!(
            (value.body_container, value.body_offset, value.status_key),
            (0, 0, 78)
        );
    }

    #[test]
    fn buff_content_preserves_unknown_and_counter_excluded_values() {
        for (status, class) in [(0, 0), (94, 2), (u32::MAX, u32::MAX), (78, 1)] {
            let mut data = fixture(SKILL as usize, BODY);
            put(&mut data, BODY + 0x14, status);
            put(&mut data, BODY + 0x18, class);
            let value = read_local(&data, SKILL, &[SLOT]).unwrap();
            assert_eq!((value.status_key, value.class), (status, class));
        }
    }

    #[test]
    fn buff_content_rejects_wrong_type_kind_source_and_skill_offset() {
        let data = fixture(SKILL as usize, BODY);
        for (field, value) in [(0x10, 63), (0x58, 0), (0x58, 2), (0x58, u32::MAX)] {
            let mut bad = data.clone();
            put(&mut bad, SKILL as usize + field, value);
            let error = read_local(&bad, SKILL, &[SLOT]).unwrap_err();
            assert_eq!(
                error.kind(),
                if field == 0x10 {
                    ErrorKind::InvalidData
                } else {
                    ErrorKind::Unsupported
                }
            );
        }
        let pf = pack(&data, &[SLOT], &[], &[], &[]);
        let indexes = [ContentIndex::parse(&pf).unwrap()];
        assert!(BuffContent209026::read(&indexes, 1, SKILL).is_err());
        assert!(BuffContent209026::read(&indexes, 0, u32::MAX).is_err());
    }

    #[test]
    fn buff_content_rejects_every_truncated_skill_and_body_prefix() {
        let data = fixture(0, BODY);
        let target = pack(&data[BODY..], &[], &[], &[], &[]);
        let mut skill = data[..0x68].to_vec();
        pointer(&mut skill, 0x60, 0);
        for length in 0..0x68 {
            let source = pack(&skill[..length], &[], &[(0x60, 1)], &[], &[]);
            let indexes = [
                ContentIndex::parse(&source).unwrap(),
                ContentIndex::parse(&target).unwrap(),
            ];
            assert!(
                BuffContent209026::read(&indexes, 0, 0).is_err(),
                "skill length {length}"
            );
        }
        let source = pack(&skill, &[], &[(0x60, 1)], &[], &[]);
        for length in 0..=0x1c {
            let target = pack(&data[BODY..BODY + length], &[], &[], &[], &[]);
            let indexes = [
                ContentIndex::parse(&source).unwrap(),
                ContentIndex::parse(&target).unwrap(),
            ];
            assert_eq!(
                BuffContent209026::read(&indexes, 0, 0).is_ok(),
                length == 0x1c,
                "body length {length}"
            );
        }
    }

    #[test]
    fn buff_content_rejects_null_unclassified_conflicting_string_and_file_refs() {
        let data = fixture(SKILL as usize, BODY);
        assert!(read_local(&data, SKILL, &[]).is_err());
        assert!(read_local(&data, SKILL, &[SLOT, SLOT]).is_err());
        let mut zero = data.clone();
        pointer(&mut zero, SLOT as usize, 0);
        assert!(read_local(&zero, SKILL, &[]).is_err());
        for pf in [
            pack(&data, &[SLOT], &[(SLOT, 0)], &[], &[]),
            pack(&data, &[], &[(SLOT, 0), (SLOT, 0)], &[], &[]),
            pack(&zero, &[], &[], &[SLOT], &[]),
            pack(&zero, &[], &[], &[], &[SLOT]),
        ] {
            assert!(
                BuffContent209026::read(&[ContentIndex::parse(&pf).unwrap()], 0, SKILL).is_err()
            );
        }
    }

    #[test]
    fn buff_content_rejects_absent_external_targets_and_invalid_body_ranges() {
        let original = fixture(SKILL as usize, BODY);
        for external in [false, true] {
            for offset in [original.len() as u64 - 1, original.len() as u64, u64::MAX] {
                let mut data = original.clone();
                pointer(&mut data, SLOT as usize, offset);
                let pf = if external {
                    pack(&data, &[], &[(SLOT, 0)], &[], &[])
                } else {
                    pack(&data, &[SLOT], &[], &[], &[])
                };
                assert!(
                    BuffContent209026::read(&[ContentIndex::parse(&pf).unwrap()], 0, SKILL)
                        .is_err()
                );
            }
        }
        let pf = pack(&original, &[], &[(SLOT, 1)], &[], &[]);
        assert!(BuffContent209026::read(&[ContentIndex::parse(&pf).unwrap()], 0, SKILL).is_err());
    }
}
