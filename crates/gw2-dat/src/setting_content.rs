//! Build-209026 type-150 settings with a type-434 integer variant.
//! See `docs/re/209026-skill-slot-levels.md`; no fallback policy is applied.

use crate::content::ContentIndex;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegerSettingDefinition209026 {
    pub guid: [u8; 16],
    /// Signed content value. A consumer may impose a narrower domain.
    pub value: i32,
}

impl IntegerSettingDefinition209026 {
    pub fn read(index: &ContentIndex<'_>, offset: u32) -> Result<Self> {
        let start = offset as usize;
        let invalid = |message| Error::new(ErrorKind::InvalidData, message);
        let bytes = start
            .checked_add(0x3c)
            .and_then(|end| index.content_bytes().get(start..end))
            .ok_or_else(|| invalid("short integer setting record"))?;
        let dword = |at| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        if dword(0x10) != 150 || dword(0x28) != 434 || dword(0x30) != 434 {
            return Err(invalid(
                "expected type-150 setting with declared and variant integer type 434",
            ));
        }
        Ok(Self {
            guid: bytes[..16].try_into().unwrap(),
            value: i32::from_le_bytes(bytes[0x38..0x3c].try_into().unwrap()),
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
    fn index_bytes(content: &[u8]) -> Packfile {
        let mut main = vec![0; 0x88];
        put(&mut main, 0x7c, content.len() as u32);
        main[0x80..0x88].copy_from_slice(&8i64.to_le_bytes());
        main.extend(content);
        Packfile::new(build_packfile(
            FLAG_64BIT_POINTERS,
            *b"cntc",
            &[(*b"Main", 0, main, vec![])],
        ))
        .unwrap()
    }
    fn record() -> Vec<u8> {
        let mut bytes = vec![0; 0x5c];
        for i in 0..16 {
            bytes[0x20 + i] = i as u8 + 1;
        }
        put(&mut bytes, 0x30, 150);
        put(&mut bytes, 0x48, 434);
        put(&mut bytes, 0x50, 434);
        bytes
    }
    #[test]
    fn setting_content_preserves_guid_and_signed_values() {
        let mut bytes = record();
        for value in [i32::MIN, -1, 0, 11, 31, i32::MAX] {
            put(&mut bytes, 0x58, value as u32);
            let pf = index_bytes(&bytes);
            let view =
                IntegerSettingDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20)
                    .unwrap();
            assert_eq!(
                view.guid,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
            );
            assert_eq!(view.value, value);
        }
    }
    #[test]
    fn setting_content_rejects_truncated_or_mistyped_records() {
        let bytes = record();
        for cut in 0..bytes.len() {
            let pf = index_bytes(&bytes[..cut]);
            assert!(
                IntegerSettingDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20)
                    .is_err()
            );
        }
        let pf = index_bytes(&bytes);
        assert!(
            IntegerSettingDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), u32::MAX)
                .is_err()
        );
        for field in [0x30, 0x48, 0x50] {
            let mut bytes = record();
            put(&mut bytes, field, 435);
            let pf = index_bytes(&bytes);
            assert!(
                IntegerSettingDefinition209026::read(&ContentIndex::parse(&pf).unwrap(), 0x20)
                    .is_err()
            );
        }
    }
}
