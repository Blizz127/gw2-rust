// Ported from valaphee/ascalon (MIT, commit 03fb60b), rewritten for stable Rust.
//
//! `PF` chunk container parsing.
//!
//! Only the container is parsed here: header and raw chunks by 4CC. The
//! schema-driven typed views and the pointer fix-up pass of the reference are
//! intentionally not ported; [`PackfileChunk::fixup_offsets`] exposes the raw
//! fix-up table for callers that want to relocate pointers themselves.

use std::io::{Error, ErrorKind, Result};

const PF_HEADER_SIZE: usize = 12;
const CHUNK_HEADER_SIZE: usize = 16;

/// Header flag: pointers inside chunks are 64-bit (otherwise 32-bit).
pub const FLAG_64BIT_POINTERS: u16 = 4;

fn invalid(msg: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, msg)
}

fn le_u16(b: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(off..off + 2)?.try_into().ok()?))
}

fn le_u32(b: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(off..off + 4)?.try_into().ok()?))
}

/// A parsed `PF` container (owns its bytes).
pub struct Packfile {
    bytes: Vec<u8>,
    flags: u16,
    header_size: usize,
    kind: [u8; 4],
}

impl Packfile {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() < PF_HEADER_SIZE {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }
        if bytes[..2] != *b"PF" {
            return Err(invalid("invalid magic"));
        }

        let flags = le_u16(&bytes, 2).ok_or_else(|| invalid("short header"))?;
        let header_size = usize::from(le_u16(&bytes, 6).ok_or_else(|| invalid("short header"))?);
        if header_size < PF_HEADER_SIZE || header_size > bytes.len() {
            return Err(invalid("invalid header size"));
        }
        let kind: [u8; 4] = bytes[8..12]
            .try_into()
            .map_err(|_| invalid("short header"))?;

        Ok(Self {
            bytes,
            flags,
            header_size,
            kind,
        })
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }

    /// Whether pointers in chunk data are 8 bytes wide.
    pub fn is_64bit(&self) -> bool {
        self.flags & FLAG_64BIT_POINTERS != 0
    }

    /// The 4-byte file type tag (e.g. `b"STRS"`).
    pub fn r#type(&self) -> [u8; 4] {
        self.kind
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Iterate the chunks. After the first error the iterator ends.
    pub fn chunks(&self) -> PackfileChunks<'_> {
        PackfileChunks {
            rest: &self.bytes[self.header_size..],
        }
    }
}

pub struct PackfileChunks<'a> {
    rest: &'a [u8],
}

impl<'a> Iterator for PackfileChunks<'a> {
    type Item = Result<PackfileChunk<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }

        let parsed = PackfileChunk::parse(self.rest);
        match &parsed {
            Ok(chunk) => self.rest = &self.rest[chunk.raw.len()..],
            Err(_) => self.rest = &[],
        }
        Some(parsed)
    }
}

/// One raw chunk: 4CC name, version, payload and trailing "extra" bytes.
#[derive(Clone, Copy)]
pub struct PackfileChunk<'a> {
    raw: &'a [u8],
    name: [u8; 4],
    version: u16,
    header_size: usize,
    data_size: usize,
}

impl<'a> PackfileChunk<'a> {
    fn parse(input: &'a [u8]) -> Result<Self> {
        let size = le_u32(input, 4).ok_or_else(|| invalid("truncated chunk header"))?;
        let total = (size as usize)
            .checked_add(8)
            .ok_or_else(|| invalid("chunk size overflow"))?;
        let raw = input
            .get(..total)
            .ok_or_else(|| invalid("chunk extends past end of file"))?;

        let header_size =
            usize::from(le_u16(raw, 10).ok_or_else(|| invalid("short chunk header"))?);
        if header_size < CHUNK_HEADER_SIZE || header_size > raw.len() {
            return Err(invalid("invalid chunk header size"));
        }
        let data_size = le_u32(raw, 12).ok_or_else(|| invalid("short chunk header"))? as usize;
        if data_size > raw.len() - header_size {
            return Err(invalid("chunk data size exceeds chunk"));
        }

        Ok(Self {
            raw,
            name: raw[..4]
                .try_into()
                .map_err(|_| invalid("short chunk header"))?,
            version: le_u16(raw, 8).ok_or_else(|| invalid("short chunk header"))?,
            header_size,
            data_size,
        })
    }

    pub fn name(&self) -> [u8; 4] {
        self.name
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    /// Chunk payload: `data_size` bytes, or everything after the header when
    /// `data_size` is 0.
    pub fn bytes(&self) -> &'a [u8] {
        let body = &self.raw[self.header_size..];
        match self.data_size {
            0 => body,
            n => &body[..n],
        }
    }

    /// Bytes after the payload (the fix-up table); empty when `data_size` is 0.
    pub fn extra_bytes(&self) -> &'a [u8] {
        match self.data_size {
            0 => &[],
            n => &self.raw[self.header_size + n..],
        }
    }

    /// Parse the fix-up table in [`extra_bytes`](Self::extra_bytes): a `u32`
    /// count followed by that many `u32` payload offsets of pointer slots.
    pub fn fixup_offsets(&self) -> Result<Vec<u32>> {
        let extra = self.extra_bytes();
        if extra.is_empty() {
            return Ok(Vec::new());
        }
        let count = le_u32(extra, 0).ok_or_else(|| Error::from(ErrorKind::UnexpectedEof))? as usize;
        let table = count
            .checked_mul(4)
            .and_then(|n| n.checked_add(4))
            .and_then(|end| extra.get(4..end))
            .ok_or_else(|| invalid("fix-up table extends past chunk"))?;
        Ok(table
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| u32::from_le_bytes(*c))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthetic::build_packfile;

    #[test]
    fn iterates_chunks_by_fourcc() {
        let blob = build_packfile(
            FLAG_64BIT_POINTERS,
            *b"TEST",
            &[
                (*b"AAAA", 1, vec![1, 2, 3, 4], vec![]),
                (*b"BBBB", 7, vec![9; 8], {
                    let mut extra = 2u32.to_le_bytes().to_vec();
                    extra.extend_from_slice(&0u32.to_le_bytes());
                    extra.extend_from_slice(&4u32.to_le_bytes());
                    extra
                }),
                (*b"CCCC", 2, vec![], vec![]),
            ],
        );
        let pf = Packfile::new(blob).unwrap();
        assert_eq!(pf.r#type(), *b"TEST");
        assert!(pf.is_64bit());

        let chunks: Vec<_> = pf.chunks().collect::<Result<_>>().unwrap();
        assert_eq!(chunks.len(), 3);

        assert_eq!(chunks[0].name(), *b"AAAA");
        assert_eq!(chunks[0].version(), 1);
        assert_eq!(chunks[0].bytes(), [1, 2, 3, 4]);
        assert!(chunks[0].extra_bytes().is_empty());
        assert!(chunks[0].fixup_offsets().unwrap().is_empty());

        assert_eq!(chunks[1].name(), *b"BBBB");
        assert_eq!(chunks[1].version(), 7);
        assert_eq!(chunks[1].bytes(), [9; 8]);
        assert_eq!(chunks[1].extra_bytes().len(), 12);
        assert_eq!(chunks[1].fixup_offsets().unwrap(), [0, 4]);

        assert_eq!(chunks[2].name(), *b"CCCC");
        assert!(chunks[2].bytes().is_empty());
    }

    #[test]
    fn rejects_bad_header() {
        assert!(Packfile::new(Vec::new()).is_err());
        assert!(Packfile::new(b"PF".to_vec()).is_err());

        let good = build_packfile(0, *b"TEST", &[]);
        let mut bad = good.clone();
        bad[0] = b'X';
        assert_eq!(
            Packfile::new(bad).err().unwrap().kind(),
            ErrorKind::InvalidData
        );

        let mut bad = good.clone();
        bad[6..8].copy_from_slice(&4u16.to_le_bytes()); // header_size < 12
        assert!(Packfile::new(bad).is_err());
        let mut bad = good;
        bad[6..8].copy_from_slice(&200u16.to_le_bytes()); // header_size > len
        assert!(Packfile::new(bad).is_err());
    }

    #[test]
    fn malformed_chunks_error_then_stop() {
        let good = build_packfile(0, *b"TEST", &[(*b"AAAA", 1, vec![1, 2, 3, 4], vec![])]);

        // Truncated at every point: never panics, errors or yields fewer chunks.
        for end in 12..good.len() {
            let pf = Packfile::new(good[..end].to_vec()).unwrap();
            let results: Vec<_> = pf.chunks().collect();
            assert!(results.iter().all(Result::is_err) && results.len() <= 1);
        }

        // Chunk size overflow.
        let mut bad = good.clone();
        bad[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        let pf = Packfile::new(bad).unwrap();
        let mut it = pf.chunks();
        assert!(it.next().unwrap().is_err());
        assert!(it.next().is_none());

        // data_size larger than the chunk.
        let mut bad = good.clone();
        bad[24..28].copy_from_slice(&1000u32.to_le_bytes());
        let pf = Packfile::new(bad).unwrap();
        assert!(pf.chunks().next().unwrap().is_err());

        // chunk header_size too small.
        let mut bad = good;
        bad[22..24].copy_from_slice(&8u16.to_le_bytes());
        let pf = Packfile::new(bad).unwrap();
        assert!(pf.chunks().next().unwrap().is_err());
    }

    #[test]
    fn oversized_fixup_table_is_an_error() {
        let extra = 1000u32.to_le_bytes().to_vec();
        let blob = build_packfile(0, *b"TEST", &[(*b"AAAA", 1, vec![0; 4], extra)]);
        let pf = Packfile::new(blob).unwrap();
        let chunk = pf.chunks().next().unwrap().unwrap();
        assert!(chunk.fixup_offsets().is_err());
    }
}
