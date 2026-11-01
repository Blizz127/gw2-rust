//! Bounded build-209026 type-53 progress definitions.
//! See `docs/re/209026-progress-completion.md` for the parameter interpretation.

use std::io::{Error, ErrorKind, Result};

use crate::content::{ContentIndex, ContentReference, resolve_reference};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressDefinition209026 {
    pub uid: u32,
    /// Data key used by server 511, distinct from UID.
    pub key: u32,
    /// Raw content value: 0 bit, 1 bit mask, 2 counter, 3 maximum.
    pub data_type: u32,
    /// Consumers inspect the low sixteen bits; preserve the complete field.
    pub channel_mask: u32,
    pub parameters: Option<ContentReference>,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

impl ProgressDefinition209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let start = offset as usize;
        let bytes = start
            .checked_add(0x44)
            .and_then(|end| index.content_bytes().get(start..end))
            .ok_or_else(|| invalid("short progress definition"))?;
        if dword(bytes, 0x10) != 53 {
            return Err(invalid("expected content type 53"));
        }
        let slot = offset
            .checked_add(0x38)
            .ok_or_else(|| invalid("progress parameter slot overflow"))?;
        let parameters = index.reference(slot)?;
        if matches!(parameters, Some(ContentReference::String { .. })) {
            return Err(invalid("progress parameters cannot be a string reference"));
        }
        Ok(Self {
            uid: dword(bytes, 0x14),
            key: dword(bytes, 0x28),
            data_type: dword(bytes, 0x2c),
            channel_mask: dword(bytes, 0x40),
            parameters,
        })
    }

    /// Resolve the completion limit for counter/maximum types only.
    /// `source_container` identifies the definition's container. Other data
    /// types return None without interpreting their parameter representation.
    pub fn counter_limit(
        &self,
        containers: &[ContentIndex<'_>],
        source_container: usize,
    ) -> Result<Option<u32>> {
        if !matches!(self.data_type, 2 | 3) {
            return Ok(None);
        }
        let reference = self
            .parameters
            .ok_or_else(|| invalid("counter progress has null parameters"))?;
        let target = resolve_reference(containers, source_container, reference, 0x14)?;
        Ok(Some(dword(target.bytes, 0x10)))
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

    fn definition() -> Vec<u8> {
        let mut bytes = vec![0; 0x64];
        put(&mut bytes, 0x30, 53);
        put(&mut bytes, 0x34, 15);
        put(&mut bytes, 0x48, 4894);
        put(&mut bytes, 0x4c, 2);
        put(&mut bytes, 0x60, 0x8000_0010);
        bytes
    }

    #[test]
    fn progress_content_resolves_local_and_external_parameters() {
        let mut bytes = definition();
        put(&mut bytes, 0x10, 60);
        let pf = pack(&bytes, &[0x58], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        let view = ProgressDefinition209026::read(&index, 0x20).unwrap();
        assert_eq!(
            (view.uid, view.key, view.data_type, view.channel_mask),
            (15, 4894, 2, 0x8000_0010)
        );
        assert_eq!(view.parameters, Some(ContentReference::Local { offset: 0 }));
        assert_eq!(view.counter_limit(&[index], 0).unwrap(), Some(60));

        put(&mut bytes, 0x4c, 3);
        let source = pack(&bytes, &[], &[(0x58, 1)]);
        let mut target = vec![0; 0x14];
        put(&mut target, 0x10, u32::MAX);
        let destination = pack(&target, &[], &[]);
        let indexes = [
            ContentIndex::parse(&source).unwrap(),
            ContentIndex::parse(&destination).unwrap(),
        ];
        let view = ProgressDefinition209026::read(&indexes[0], 0x20).unwrap();
        assert_eq!(view.counter_limit(&indexes, 0).unwrap(), Some(u32::MAX));
        assert!(view.counter_limit(&indexes[..1], 0).is_err());
        assert!(view.counter_limit(&indexes, 2).is_err());
        let short = pack(&target[..0x13], &[], &[]);
        assert!(
            view.counter_limit(
                &[
                    ContentIndex::parse(&source).unwrap(),
                    ContentIndex::parse(&short).unwrap()
                ],
                0
            )
            .is_err()
        );
    }

    #[test]
    fn progress_content_bounds_and_reference_classification() {
        let mut bytes = definition();
        for cut in 0..bytes.len() {
            let pf = pack(&bytes[..cut], &[], &[]);
            assert!(
                ProgressDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20).is_err()
            );
        }
        let pf = pack(&bytes, &[], &[]);
        let index = ContentIndex::parse(&pf).unwrap();
        let view = ProgressDefinition209026::read(&index, 0x20).unwrap();
        assert!(view.counter_limit(&[index], 0).is_err());
        let index = ContentIndex::parse(&pf).unwrap();
        assert!(ProgressDefinition209026::read(&index, u32::MAX).is_err());
        // A nonzero pointer without a fixup and conflicting fixups are rejected.
        bytes[0x58] = 1;
        let pf = pack(&bytes, &[], &[]);
        assert!(ProgressDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20).is_err());
        let pf = pack(&bytes, &[0x58], &[(0x58, 1)]);
        assert!(ProgressDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20).is_err());
        put(&mut bytes, 0x30, 54);
        let pf = pack(&bytes, &[], &[]);
        assert!(ProgressDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20).is_err());
    }

    #[test]
    fn progress_content_does_not_reinterpret_other_parameter_types() {
        for data_type in [0, 1, 4, u32::MAX] {
            let mut bytes = definition();
            put(&mut bytes, 0x4c, data_type);
            let pf = pack(&bytes, &[], &[]);
            let index = ContentIndex::parse(&pf).unwrap();
            let view = ProgressDefinition209026::read(&index, 0x20).unwrap();
            assert_eq!(view.data_type, data_type);
            assert_eq!(view.counter_limit(&[index], 0).unwrap(), None);
        }
    }
}
