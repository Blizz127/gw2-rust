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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentReference {
    Local {
        offset: usize,
    },
    /// Target bounds must be checked against the selected container by the caller.
    External {
        container_index: u32,
        offset: u64,
    },
    String {
        index: usize,
    },
}

pub struct ContentIndex<'a> {
    main: &'a [u8],
    flags: u32,
    types: &'a [u8],
    entries: &'a [u8],
    content: &'a [u8],
    local_fixups: &'a [u8],
    external_fixups: &'a [u8],
    file_fixups: &'a [u8],
    string_fixups: &'a [u8],
    strings: &'a [u8],
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
            main: data,
            flags,
            types: array(data, 4, 17)?,
            entries: array(data, 0x28, 16)?,
            content: array(data, 0x7c, 1)?,
            local_fixups: array(data, 0x34, 4)?,
            external_fixups: array(data, 0x40, 8)?,
            file_fixups: array(data, 0x4c, 4)?,
            string_fixups: array(data, 0x58, 4)?,
            strings: array(data, 0x70, 8)?,
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

    /// Interpret one content-relative pointer slot using its fixup classification.
    /// Duplicate or conflicting classifications are rejected, even for zero values.
    /// A zero value in a classified slot is an offset/index, not a null pointer.
    pub fn reference(&self, slot: u32) -> Result<Option<ContentReference>> {
        let start = slot as usize;
        let bytes = start
            .checked_add(8)
            .and_then(|end| self.content.get(start..end))
            .ok_or_else(|| invalid("content pointer slot outside content array"))?;
        let value = u64::from_le_bytes(bytes.try_into().unwrap());
        if self
            .file_fixups
            .chunks_exact(4)
            .any(|field| dword(field, 0) == slot)
        {
            return Err(Error::new(ErrorKind::Unsupported, "content file reference"));
        }
        let mut result = None;
        let mut record = |reference| -> Result<()> {
            if result.replace(reference).is_some() {
                return Err(invalid("duplicate content pointer fixup"));
            }
            Ok(())
        };
        for field in self.local_fixups.chunks_exact(4) {
            if dword(field, 0) == slot {
                let offset = usize::try_from(value)
                    .ok()
                    .filter(|&p| p < self.content.len())
                    .ok_or_else(|| invalid("local reference outside content array"))?;
                record(ContentReference::Local { offset })?;
            }
        }
        for field in self.external_fixups.chunks_exact(8) {
            if dword(field, 0) == slot {
                record(ContentReference::External {
                    container_index: dword(field, 4),
                    offset: value,
                })?;
            }
        }
        for field in self.string_fixups.chunks_exact(4) {
            if dword(field, 0) == slot {
                let index = usize::try_from(value)
                    .ok()
                    .filter(|&p| p < self.string_count())
                    .ok_or_else(|| invalid("content string index outside string table"))?;
                record(ContentReference::String { index })?;
            }
        }
        match result {
            Some(reference) => Ok(Some(reference)),
            None if value == 0 => Ok(None),
            None => Err(invalid("unclassified nonzero content reference")),
        }
    }

    pub fn string_count(&self) -> usize {
        self.strings.len() / 8
    }

    /// Read one indexed UTF-16 string, rejecting invalid UTF-16 or a missing
    /// terminator within `max_units`. The returned text is not logged here.
    pub fn string(&self, index: usize, max_units: usize) -> Result<String> {
        let start = index
            .checked_mul(8)
            .ok_or_else(|| invalid("string index overflow"))?;
        let field = start
            .checked_add(8)
            .and_then(|end| self.strings.get(start..end))
            .ok_or_else(|| invalid("string index outside string table"))?;
        // A nonempty `strings` slice was borrowed from this same Main payload.
        let slot = self.strings.as_ptr().addr() - self.main.as_ptr().addr() + start;
        let relative = i64::from_le_bytes(field.try_into().unwrap());
        if relative == 0 {
            return Err(invalid("null indexed content string"));
        }
        let target = i64::try_from(slot)
            .ok()
            .and_then(|p| p.checked_add(relative))
            .and_then(|p| usize::try_from(p).ok())
            .filter(|&p| p < self.main.len())
            .ok_or_else(|| invalid("content string outside chunk"))?;
        let mut units = Vec::with_capacity(max_units.min(256));
        for pair in self.main[target..].chunks_exact(2) {
            let unit = u16::from_le_bytes(pair.try_into().unwrap());
            if unit == 0 {
                return String::from_utf16(&units).map_err(|_| invalid("invalid content UTF-16"));
            }
            if units.len() == max_units {
                return Err(invalid("content string limit exceeded"));
            }
            units.push(unit);
        }
        Err(invalid("unterminated content string"))
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

    fn append_table(b: &mut Vec<u8>, field: usize, bytes: &[u8], stride: usize) -> usize {
        let target = b.len();
        b[field..field + 4].copy_from_slice(&((bytes.len() / stride) as u32).to_le_bytes());
        b[field + 4..field + 12]
            .copy_from_slice(&(target as i64 - (field + 4) as i64).to_le_bytes());
        b.extend_from_slice(bytes);
        target
    }

    fn references_payload() -> (Vec<u8>, usize) {
        let mut b = payload();
        b[0xa9..].fill(0);
        b[0xb1..0xb9].copy_from_slice(&31u64.to_le_bytes());
        append_table(&mut b, 0x34, &0u32.to_le_bytes(), 4);
        append_table(&mut b, 0x40, &[8, 0, 0, 0, 2, 0, 0, 0], 8);
        append_table(&mut b, 0x58, &16u32.to_le_bytes(), 4);
        let string_slot = append_table(&mut b, 0x70, &[0; 8], 8);
        b[string_slot..string_slot + 8].copy_from_slice(&8i64.to_le_bytes());
        for unit in "Hé😀".encode_utf16().chain([0]) {
            b.extend_from_slice(&unit.to_le_bytes());
        }
        (b, string_slot)
    }

    #[test]
    fn resolves_classified_zero_offsets_and_bounded_strings() {
        let (b, _) = references_payload();
        let pf = pack(b);
        let index = ContentIndex::parse(&pf).unwrap();
        assert_eq!(
            index.reference(0).unwrap(),
            Some(ContentReference::Local { offset: 0 })
        );
        assert_eq!(
            index.reference(8).unwrap(),
            Some(ContentReference::External {
                container_index: 2,
                offset: 31
            })
        );
        assert_eq!(
            index.reference(16).unwrap(),
            Some(ContentReference::String { index: 0 })
        );
        assert_eq!(index.reference(24).unwrap(), None);
        assert_eq!(index.string_count(), 1);
        assert_eq!(index.string(0, 4).unwrap(), "Hé😀");
        assert!(index.string(0, 3).is_err());
        assert!(index.string(1, 4).is_err());
        assert!(index.string(usize::MAX, 4).is_err());
        assert!(index.reference(25).is_err());
        assert!(index.reference(u32::MAX).is_err());
    }

    #[test]
    fn rejects_conflicting_unknown_and_out_of_range_references() {
        for (slot, value) in [(0, 32u64), (16, 1), (24, 1)] {
            let (mut b, _) = references_payload();
            b[0xa9 + slot..0xa9 + slot + 8].copy_from_slice(&value.to_le_bytes());
            let pf = pack(b);
            assert!(
                ContentIndex::parse(&pf)
                    .unwrap()
                    .reference(slot as u32)
                    .is_err()
            );
        }
        for field in [0x34, 0x58, 0x4c] {
            let (mut b, _) = references_payload();
            let entries = if field == 0x34 {
                vec![0; 8]
            } else {
                vec![0; 4]
            };
            append_table(&mut b, field, &entries, 4);
            let pf = pack(b);
            assert!(ContentIndex::parse(&pf).unwrap().reference(0).is_err());
        }
    }

    #[test]
    fn rejects_invalid_string_pointers_encoding_and_termination() {
        for delta in [0, i64::MAX, i64::MIN, -500] {
            let (mut b, slot) = references_payload();
            b[slot..slot + 8].copy_from_slice(&delta.to_le_bytes());
            let pf = pack(b);
            assert!(ContentIndex::parse(&pf).unwrap().string(0, 4).is_err());
        }
        let (mut b, _) = references_payload();
        b.truncate(b.len() - 2);
        assert!(ContentIndex::parse(&pack(b)).unwrap().string(0, 4).is_err());
        let (mut b, slot) = references_payload();
        b[slot + 8..slot + 10].copy_from_slice(&0xd800u16.to_le_bytes());
        assert!(ContentIndex::parse(&pack(b)).unwrap().string(0, 4).is_err());
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
