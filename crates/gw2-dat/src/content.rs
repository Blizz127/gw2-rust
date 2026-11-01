//! Borrowed index view for build-209026 `cntc` / `Main` version 0.
//!
//! Content references remain unrelocated. Type metadata in the first base
//! container is shared by subsequent containers; it is not required locally.

use std::io::{Error, ErrorKind, Result};

use crate::packfile::Packfile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexEntry {
    pub content_type: u32,
    pub offset: u32,
    pub namespace_index: u32,
    pub root_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeInfo {
    pub guid_offset: Option<u32>,
    pub uid_offset: Option<u32>,
    pub data_id_offset: Option<u32>,
    pub name_offset: Option<u32>,
    pub track_references: u8,
}

pub struct ContentIndex<'a> {
    flags: u32,
    types: &'a [u8],
    entries: &'a [u8],
    content: &'a [u8],
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

// Callers provide already bounded records.
fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn array(data: &[u8], at: usize, stride: usize) -> Result<&[u8]> {
    let field = data
        .get(at..at + 12)
        .ok_or_else(|| invalid("short content header"))?;
    let count = dword(field, 0) as usize;
    let relative = i64::from_le_bytes(field[4..12].try_into().unwrap());
    if relative == 0 {
        return if count == 0 {
            Ok(&[])
        } else {
            Err(invalid("null content array"))
        };
    }
    let slot = i64::try_from(at + 4).map_err(|_| invalid("content array offset overflow"))?;
    let start = slot
        .checked_add(relative)
        .and_then(|p| usize::try_from(p).ok())
        .ok_or_else(|| invalid("content array offset overflow"))?;
    let end = count
        .checked_mul(stride)
        .and_then(|n| start.checked_add(n))
        .ok_or_else(|| invalid("content array length overflow"))?;
    data.get(start..end)
        .ok_or_else(|| invalid("content array outside chunk"))
}

impl<'a> ContentIndex<'a> {
    pub fn parse(packfile: &'a Packfile) -> Result<Self> {
        if packfile.r#type() != *b"cntc" || !packfile.is_64bit() {
            return Err(invalid("expected 64-bit cntc packfile"));
        }
        let mut main = None;
        for chunk in packfile.chunks() {
            let chunk = chunk?;
            if chunk.name() == *b"Main" {
                if main.is_some() || chunk.version() != 0 {
                    return Err(invalid("duplicate or unsupported content Main chunk"));
                }
                main = Some(chunk.bytes());
            }
        }
        let data = main.ok_or_else(|| invalid("missing content Main chunk"))?;
        if data.len() < 0x88 {
            return Err(invalid("short content header"));
        }
        let flags = dword(data, 0);
        if flags & 1 != 0 {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "compressed content mode",
            ));
        }
        let result = Self {
            flags,
            types: array(data, 4, 17)?,
            entries: array(data, 0x28, 16)?,
            content: array(data, 0x7c, 1)?,
        };
        for entry in result.entries() {
            if entry.offset as usize >= result.content.len() {
                return Err(invalid("content record offset outside content array"));
            }
        }
        Ok(result)
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Raw bytes, before local/external/string reference fixups.
    pub fn content_bytes(&self) -> &'a [u8] {
        self.content
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = IndexEntry> + '_ {
        self.entries.chunks_exact(16).map(|b| IndexEntry {
            content_type: dword(b, 0),
            offset: dword(b, 4),
            namespace_index: dword(b, 8),
            root_index: dword(b, 12),
        })
    }

    pub fn type_count(&self) -> usize {
        self.types.len() / 17
    }

    pub fn type_info(&self, content_type: u32) -> Option<TypeInfo> {
        let start = (content_type as usize).checked_mul(17)?;
        let b = self.types.get(start..start.checked_add(17)?)?;
        let offset = |at| {
            let value = dword(b, at);
            (value != u32::MAX).then_some(value)
        };
        Some(TypeInfo {
            guid_offset: offset(0),
            uid_offset: offset(4),
            data_id_offset: offset(8),
            name_offset: offset(12),
            track_references: b[16],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{packfile::FLAG_64BIT_POINTERS, synthetic::build_packfile};

    fn payload() -> Vec<u8> {
        let mut b = vec![0; 0x88 + 17 + 16 + 32];
        b[..4].copy_from_slice(&2u32.to_le_bytes());
        for (field, count, target) in [(4, 1u32, 0x88), (0x28, 1, 0x99), (0x7c, 32, 0xa9)] {
            b[field..field + 4].copy_from_slice(&count.to_le_bytes());
            let delta = target as i64 - (field + 4) as i64;
            b[field + 4..field + 12].copy_from_slice(&delta.to_le_bytes());
        }
        for (i, value) in [0u32, 20, u32::MAX, 24].into_iter().enumerate() {
            b[0x88 + 4 * i..0x8c + 4 * i].copy_from_slice(&value.to_le_bytes());
        }
        b[0xa1..0xa5].copy_from_slice(&2u32.to_le_bytes());
        b[0xa5..0xa9].copy_from_slice(&3u32.to_le_bytes());
        b[0xa9..].fill(0xab);
        b
    }

    fn pack(b: Vec<u8>) -> Packfile {
        Packfile::new(build_packfile(
            FLAG_64BIT_POINTERS,
            *b"cntc",
            &[(*b"Main", 0, b, vec![])],
        ))
        .unwrap()
    }

    #[test]
    fn reads_index_and_shared_type_metadata_without_relocation() {
        let pf = pack(payload());
        let index = ContentIndex::parse(&pf).unwrap();
        assert_eq!(index.flags(), 2);
        assert_eq!(
            index.entries().collect::<Vec<_>>(),
            [IndexEntry {
                content_type: 0,
                offset: 0,
                namespace_index: 2,
                root_index: 3,
            }]
        );
        assert_eq!(index.type_count(), 1);
        assert_eq!(
            index.type_info(0),
            Some(TypeInfo {
                guid_offset: Some(0),
                uid_offset: Some(20),
                data_id_offset: None,
                name_offset: Some(24),
                track_references: 0,
            })
        );
        assert_eq!(index.type_info(u32::MAX), None);
        assert_eq!(index.content_bytes(), &[0xab; 32]);
        let mut b = payload();
        b[4..16].fill(0); // Subsequent containers may omit the shared type table.
        let pf = pack(b);
        assert_eq!(ContentIndex::parse(&pf).unwrap().type_count(), 0);
    }

    #[test]
    fn rejects_truncation_counts_and_extreme_relative_offsets() {
        let original = payload();
        for cut in 0..original.len() {
            let pf = pack(original[..cut].to_vec());
            assert!(ContentIndex::parse(&pf).is_err());
        }
        for field in [4, 0x28, 0x7c] {
            for delta in [0, i64::MIN, i64::MAX, -500] {
                let mut b = original.clone();
                b[field + 4..field + 12].copy_from_slice(&delta.to_le_bytes());
                assert!(ContentIndex::parse(&pack(b)).is_err());
            }
            let mut b = original.clone();
            b[field..field + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(ContentIndex::parse(&pack(b)).is_err());
        }
        let mut b = original;
        b[0x9d..0xa1].copy_from_slice(&32u32.to_le_bytes());
        assert!(ContentIndex::parse(&pack(b)).is_err());
    }

    #[test]
    fn rejects_unsupported_container_modes_and_duplicate_main() {
        for (flags, kind, tag, version) in [
            (0, *b"cntc", *b"Main", 0),
            (4, *b"nope", *b"Main", 0),
            (4, *b"cntc", *b"nope", 0),
            (4, *b"cntc", *b"Main", 1),
        ] {
            let pf = Packfile::new(build_packfile(
                flags,
                kind,
                &[(tag, version, payload(), vec![])],
            ))
            .unwrap();
            assert!(ContentIndex::parse(&pf).is_err());
        }
        let pf = Packfile::new(build_packfile(
            4,
            *b"cntc",
            &[
                (*b"Main", 0, payload(), vec![]),
                (*b"Main", 0, payload(), vec![]),
            ],
        ))
        .unwrap();
        assert!(ContentIndex::parse(&pf).is_err());
        let mut b = payload();
        b[0] |= 1;
        assert!(ContentIndex::parse(&pack(b)).is_err());
    }
}
