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

/// The encoded archive-ID form of a content file reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentFileReference {
    pub file_id: u32,
    /// Optional low byte emitted by the client's file-reference decoder.
    pub suffix: u8,
}

/// A checked byte range in an explicitly supplied content-container set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedContent<'a> {
    pub container_index: usize,
    pub offset: usize,
    pub bytes: &'a [u8],
}

/// Resolve a classified record/array reference without loading any files.
/// `containers` must use the same ordering as external container indexes.
/// The caller selects the needed byte length and validates the record type.
pub fn resolve_reference<'a>(
    containers: &[ContentIndex<'a>],
    source_container: usize,
    reference: ContentReference,
    byte_length: usize,
) -> Result<ResolvedContent<'a>> {
    if source_container >= containers.len() {
        return Err(invalid("source content container is absent"));
    }
    let (container_index, offset) = match reference {
        ContentReference::Local { offset } => (source_container, offset),
        ContentReference::External {
            container_index,
            offset,
        } => (
            usize::try_from(container_index)
                .map_err(|_| invalid("external content container index overflow"))?,
            usize::try_from(offset).map_err(|_| invalid("external content offset overflow"))?,
        ),
        ContentReference::String { .. } => {
            return Err(invalid("string reference is not a content byte range"));
        }
    };
    let container = containers
        .get(container_index)
        .ok_or_else(|| invalid("target content container is absent"))?;
    let bytes = offset
        .checked_add(byte_length)
        .and_then(|end| container.content_bytes().get(offset..end))
        .ok_or_else(|| invalid("resolved content range outside target container"))?;
    Ok(ResolvedContent {
        container_index,
        offset,
        bytes,
    })
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
    files: &'a [u8],
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
            files: array(data, 0x1c, 8)?,
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
        self.entries.as_chunks::<16>().0.iter().map(|b| IndexEntry {
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
            .as_chunks::<4>().0.iter()
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
        for field in self.local_fixups.as_chunks::<4>().0.iter() {
            if dword(field, 0) == slot {
                let offset = usize::try_from(value)
                    .ok()
                    .filter(|&p| p < self.content.len())
                    .ok_or_else(|| invalid("local reference outside content array"))?;
                record(ContentReference::Local { offset })?;
            }
        }
        for field in self.external_fixups.as_chunks::<8>().0.iter() {
            if dword(field, 0) == slot {
                record(ContentReference::External {
                    container_index: dword(field, 4),
                    offset: value,
                })?;
            }
        }
        for field in self.string_fixups.as_chunks::<4>().0.iter() {
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

    /// Resolve a classified file slot through an explicitly selected table.
    /// Build 209026's base loader uses container zero's table for every base
    /// container. This method neither opens the archive file nor accepts names.
    pub fn resolve_file_reference(
        &self,
        slot: u32,
        table: &ContentIndex<'_>,
    ) -> Result<ContentFileReference> {
        let file_count = self
            .file_fixups
            .as_chunks::<4>().0.iter()
            .filter(|field| dword(field.as_slice(), 0) == slot)
            .count();
        let conflicts = self
            .local_fixups
            .as_chunks::<4>().0.iter()
            .chain(self.string_fixups.as_chunks::<4>().0.iter())
            .any(|field| dword(field, 0) == slot)
            || self
                .external_fixups
                .as_chunks::<8>().0.iter()
                .any(|field| dword(field, 0) == slot);
        if file_count != 1 || conflicts {
            return Err(invalid("missing or conflicting content file fixup"));
        }
        let start = slot as usize;
        let bytes = start
            .checked_add(8)
            .and_then(|end| self.content.get(start..end))
            .ok_or_else(|| invalid("content file slot outside content array"))?;
        let index = usize::try_from(u64::from_le_bytes(bytes.try_into().unwrap()))
            .map_err(|_| invalid("content file index overflow"))?;
        let start = index
            .checked_mul(8)
            .ok_or_else(|| invalid("content file index overflow"))?;
        let field = start
            .checked_add(8)
            .and_then(|end| table.files.get(start..end))
            .ok_or_else(|| invalid("content file index outside selected table"))?;
        let pointer_slot = table.files.as_ptr().addr() - table.main.as_ptr().addr() + start;
        let relative = i64::from_le_bytes(field.try_into().unwrap());
        if relative == 0 {
            return Err(invalid("null indexed content file reference"));
        }
        let target = i64::try_from(pointer_slot)
            .ok()
            .and_then(|p| p.checked_add(relative))
            .and_then(|p| usize::try_from(p).ok())
            .and_then(|p| table.main.get(p..))
            .ok_or_else(|| invalid("content file reference outside chunk"))?;
        let head = target
            .get(..6)
            .ok_or_else(|| invalid("short encoded content file reference"))?;
        let word = |at| u16::from_le_bytes([head[at], head[at + 1]]);
        let (low, high, suffix) = (word(0), word(2), word(4));
        if low < 0x100
            || high < 0x100
            || (suffix != 0 && (suffix < 0x100 || target.get(6..8) != Some(&[0, 0])))
        {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "content file reference is not an encoded archive ID",
            ));
        }
        Ok(ContentFileReference {
            file_id: u32::from(high)
                .wrapping_mul(0xff00)
                .wrapping_add(u32::from(low))
                .wrapping_add(0xff00ff01),
            suffix: suffix as u8,
        })
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
        for pair in self.main[target..].as_chunks::<2>().0.iter() {
            let unit = u16::from_le_bytes(*pair);
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

    fn file_source() -> Vec<u8> {
        let mut b = payload();
        b[0xa9..].fill(0);
        append_table(&mut b, 0x4c, &0u32.to_le_bytes(), 4);
        b
    }

    fn file_table(words: &[u16]) -> (Vec<u8>, usize) {
        let mut b = payload();
        let slot = append_table(&mut b, 0x1c, &8i64.to_le_bytes(), 8);
        for word in words {
            b.extend_from_slice(&word.to_le_bytes());
        }
        (b, slot)
    }

    #[test]
    fn content_file_reference_uses_selected_table_and_classified_zero_index() {
        let source_pf = pack(file_source());
        let source = ContentIndex::parse(&source_pf).unwrap();
        for (words, expected) in [
            (
                vec![5155, 264, 0],
                ContentFileReference {
                    file_id: 527140,
                    suffix: 0,
                },
            ),
            (
                vec![5155, 264, 0x123, 0],
                ContentFileReference {
                    file_id: 527140,
                    suffix: 0x23,
                },
            ),
            (
                vec![0x100, 0x100, 0],
                ContentFileReference {
                    file_id: 1,
                    suffix: 0,
                },
            ),
        ] {
            let table_pf = pack(file_table(&words).0);
            let table = ContentIndex::parse(&table_pf).unwrap();
            assert_eq!(source.resolve_file_reference(0, &table).unwrap(), expected);
        }
        assert!(source.resolve_file_reference(0, &source).is_err());
        assert!(source.reference(0).is_err()); // File IDs are not content byte ranges.
    }

    #[test]
    fn content_file_reference_rejects_bad_classification_indexes_and_slots() {
        let table_pf = pack(file_table(&[5155, 264, 0]).0);
        let table = ContentIndex::parse(&table_pf).unwrap();
        for value in [1u64, u64::MAX] {
            let mut b = file_source();
            b[0xa9..0xb1].copy_from_slice(&value.to_le_bytes());
            let pf = pack(b);
            assert!(
                ContentIndex::parse(&pf)
                    .unwrap()
                    .resolve_file_reference(0, &table)
                    .is_err()
            );
        }
        for field in [0x34, 0x40, 0x58, 0x4c] {
            let mut b = file_source();
            let stride = if field == 0x40 { 8 } else { 4 };
            let len = if field == 0x4c { 8 } else { stride };
            append_table(&mut b, field, &vec![0; len], stride);
            let pf = pack(b);
            assert!(
                ContentIndex::parse(&pf)
                    .unwrap()
                    .resolve_file_reference(0, &table)
                    .is_err()
            );
        }
        for slot in [8u32, 25, u32::MAX] {
            let mut b = file_source();
            if slot != 8 {
                append_table(&mut b, 0x4c, &slot.to_le_bytes(), 4);
            }
            let pf = pack(b);
            assert!(
                ContentIndex::parse(&pf)
                    .unwrap()
                    .resolve_file_reference(slot, &table)
                    .is_err()
            );
        }
    }

    #[test]
    fn content_file_reference_rejects_bad_pointers_and_encodings() {
        let source_pf = pack(file_source());
        let source = ContentIndex::parse(&source_pf).unwrap();
        for delta in [0i64, i64::MIN, i64::MAX, -500] {
            let (mut b, slot) = file_table(&[5155, 264, 0]);
            b[slot..slot + 8].copy_from_slice(&delta.to_le_bytes());
            let pf = pack(b);
            assert!(
                source
                    .resolve_file_reference(0, &ContentIndex::parse(&pf).unwrap())
                    .is_err()
            );
        }
        for words in [
            vec![],
            vec![5155],
            vec![5155, 264],
            vec![0xff, 264, 0],
            vec![5155, 0xff, 0],
            vec![5155, 264, 1, 0],
            vec![5155, 264, 0x123],
            vec![5155, 264, 0x123, 1],
        ] {
            let pf = pack(file_table(&words).0);
            assert!(
                source
                    .resolve_file_reference(0, &ContentIndex::parse(&pf).unwrap())
                    .is_err()
            );
        }
        let (mut b, _) = file_table(&[5155, 264, 0]);
        b[0x1c..0x20].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(ContentIndex::parse(&pack(b)).is_err());
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

    #[test]
    fn content_resolution_uses_target_container_and_preserves_zero_offsets() {
        let (source, _) = references_payload();
        let source = pack(source);
        let mut target = payload();
        target[0xa9..].fill(0x5c);
        let target = pack(target);
        let containers = [
            ContentIndex::parse(&source).unwrap(),
            ContentIndex::parse(&target).unwrap(),
        ];
        let local = resolve_reference(
            &containers,
            0,
            containers[0].reference(0).unwrap().unwrap(),
            8,
        )
        .unwrap();
        assert_eq!(
            (local.container_index, local.offset, local.bytes),
            (0, 0, &b"\0\0\0\0\0\0\0\0"[..])
        );
        let external = resolve_reference(
            &containers,
            0,
            ContentReference::External {
                container_index: 1,
                offset: 0,
            },
            32,
        )
        .unwrap();
        assert_eq!((external.container_index, external.offset), (1, 0));
        assert_eq!(external.bytes, &[0x5c; 32]);
        let tail = resolve_reference(
            &containers,
            0,
            ContentReference::External {
                container_index: 1,
                offset: 31,
            },
            1,
        )
        .unwrap();
        assert_eq!(tail.bytes, &[0x5c]);
        assert!(
            resolve_reference(&containers, 1, ContentReference::Local { offset: 32 }, 0)
                .unwrap()
                .bytes
                .is_empty()
        );
    }

    #[test]
    fn content_resolution_rejects_missing_containers_strings_and_bad_ranges() {
        let pf = pack(payload());
        let containers = [ContentIndex::parse(&pf).unwrap()];
        for (source, reference, length) in [
            (1, ContentReference::Local { offset: 0 }, 1),
            (
                usize::MAX,
                ContentReference::External {
                    container_index: 0,
                    offset: 0,
                },
                1,
            ),
            (
                0,
                ContentReference::External {
                    container_index: 1,
                    offset: 0,
                },
                1,
            ),
            (
                0,
                ContentReference::External {
                    container_index: u32::MAX,
                    offset: 0,
                },
                1,
            ),
            (
                0,
                ContentReference::External {
                    container_index: 0,
                    offset: u64::MAX,
                },
                1,
            ),
            (
                0,
                ContentReference::External {
                    container_index: 0,
                    offset: 31,
                },
                2,
            ),
            (0, ContentReference::Local { offset: usize::MAX }, 1),
            (0, ContentReference::Local { offset: 1 }, usize::MAX),
            (0, ContentReference::Local { offset: 33 }, 0),
            (0, ContentReference::String { index: 0 }, 0),
        ] {
            assert!(resolve_reference(&containers, source, reference, length).is_err());
        }
    }
}
