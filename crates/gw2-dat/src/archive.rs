// Ported from valaphee/ascalon (MIT, commit 03fb60b), rewritten for stable Rust.
//
//! `Gw2.dat` archive reader: AN header, MFT, file-id index, entry reads.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use crate::decompress::decompress;

pub(crate) const AN_HEADER_SIZE: usize = 40;
pub(crate) const MFT_HEADER_SIZE: usize = 24;
pub(crate) const MFT_ENTRY_SIZE: usize = 24;
const INDEX_ENTRY_SIZE: usize = 8;
/// Entries are stored in blocks of this many bytes, each ending in a 4-byte trailer.
pub(crate) const BLOCK_SIZE: usize = 0x10000;

const AN_VERSION: u8 = 151;
/// Upper bound on a decompressed entry, to avoid absurd allocations from corrupt headers.
pub const MAX_DECOMPRESSED_SIZE: u32 = 1 << 30;

/// `MftEntry::flags` bit: entry is in use.
pub const MFT_FLAG_ENTRY_USED: u8 = 1 << 0;
/// `MftEntry::flags` bit: first stream of a multi-stream entry.
pub const MFT_FLAG_FIRST_STREAM: u8 = 1 << 1;

fn invalid(msg: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, msg)
}

fn le_u16(b: &[u8], off: usize) -> u16 {
    b.get(off..off + 2)
        .and_then(|s| s.try_into().ok())
        .map_or(0, u16::from_le_bytes)
}

fn le_u32(b: &[u8], off: usize) -> u32 {
    b.get(off..off + 4)
        .and_then(|s| s.try_into().ok())
        .map_or(0, u32::from_le_bytes)
}

fn le_u64(b: &[u8], off: usize) -> u64 {
    b.get(off..off + 8)
        .and_then(|s| s.try_into().ok())
        .map_or(0, u64::from_le_bytes)
}

/// One 24-byte MFT entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MftEntry {
    /// Absolute file offset of the stored data.
    pub offset: u64,
    /// Stored size in bytes, including the 4-byte trailer of every 64 KiB block.
    pub size: u32,
    /// 0 = stored raw, 8 = compressed (8-byte header + Huffman/LZ stream).
    pub extra_bytes: u16,
    pub flags: u8,
    pub stream: u8,
    pub next_stream: u32,
    pub crc: u32,
}

impl MftEntry {
    fn parse(b: &[u8]) -> Self {
        Self {
            offset: le_u64(b, 0),
            size: le_u32(b, 8),
            extra_bytes: le_u16(b, 12),
            flags: b.get(14).copied().unwrap_or(0),
            stream: b.get(15).copied().unwrap_or(0),
            next_stream: le_u32(b, 16),
            crc: le_u32(b, 20),
        }
    }

    pub fn is_compressed(&self) -> bool {
        self.extra_bytes == 8
    }
}

/// A `Gw2.dat` archive over any seekable reader (a [`File`] by default).
pub struct Archive<R = File> {
    reader: Mutex<R>,
    len: u64,
    index: HashMap<u32, MftEntry>,
    /// Every MFT entry after the header slot, in MFT order (0-based).
    entries: Vec<MftEntry>,
    /// Raw file index: `(file id, 0-based MFT entry)` in on-disk order, id 0 skipped.
    pairs: Vec<(u32, usize)>,
}

impl Archive<File> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_reader(File::open(path)?)
    }
}

impl<R: Read + Seek> Archive<R> {
    pub fn from_reader(mut reader: R) -> Result<Self> {
        let len = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        let mut an_header = [0u8; AN_HEADER_SIZE];
        reader.read_exact(&mut an_header)?;

        if an_header[0] != AN_VERSION || an_header[1..4] != *b"AN\x1A" {
            return Err(invalid("invalid AN magic"));
        }

        let mft_offset = le_u64(&an_header, 0x18);
        let mft_size = le_u32(&an_header, 0x20) as usize;
        let mft = read_range(&mut reader, len, mft_offset, mft_size, "MFT out of bounds")?;

        if mft.len() < MFT_HEADER_SIZE || mft[..4] != *b"Mft\x1A" {
            return Err(invalid("invalid MFT magic"));
        }

        let entry_bytes = &mft[MFT_HEADER_SIZE..];
        let (entry_chunks, rem) = entry_bytes.as_chunks::<MFT_ENTRY_SIZE>();
        if !rem.is_empty() {
            return Err(invalid("invalid MFT size"));
        }
        let entries: Vec<MftEntry> = entry_chunks.iter().map(|c| MftEntry::parse(c)).collect();

        // `entry_count` includes the header slot, so entries.len() == entry_count - 1.
        if (le_u32(&mft, 12) as usize).checked_sub(1) != Some(entries.len()) {
            return Err(invalid("invalid MFT entry count"));
        }

        let [_, mft_index_entry, _, ..] = entries.as_slice() else {
            return Err(invalid("invalid MFT entry count"));
        };

        let index_bytes = read_range(
            &mut reader,
            len,
            mft_index_entry.offset,
            mft_index_entry.size as usize,
            "file index out of bounds",
        )?;
        let (index_chunks, rem) = index_bytes.as_chunks::<INDEX_ENTRY_SIZE>();
        if !rem.is_empty() {
            return Err(invalid("invalid file index size"));
        }

        let mut index = HashMap::new();
        let mut pairs = Vec::new();
        for raw in index_chunks {
            let file_id = le_u32(raw, 0);
            if file_id == 0 {
                continue;
            }
            // `mft_index` is 1-based into `entries`.
            let entry = (le_u32(raw, 4) as usize)
                .checked_sub(1)
                .and_then(|i| entries.get(i))
                .ok_or_else(|| invalid("file index refers to missing MFT entry"))?;
            index.insert(file_id, *entry);
            pairs.push((file_id, le_u32(raw, 4) as usize - 1));
        }

        Ok(Self {
            reader: Mutex::new(reader),
            len,
            index,
            entries,
            pairs,
        })
    }

    /// Number of indexed files.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Indexed file ids in unspecified order.
    pub fn file_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.index.keys().copied()
    }

    pub fn contains(&self, file_id: u32) -> bool {
        self.index.contains_key(&file_id)
    }

    /// MFT entry of `file_id`, if indexed.
    pub fn entry(&self, file_id: u32) -> Option<&MftEntry> {
        self.index.get(&file_id)
    }

    /// Read (and decompress, if needed) the contents of `file_id`.
    pub fn read(&self, file_id: u32) -> Result<Vec<u8>> {
        self.read_limited(file_id, None)
    }

    /// The first `n` decoded bytes of an entry (fewer if the entry is shorter).
    /// For compressed entries only the first stored block is read and only `n`
    /// bytes are decompressed, which makes type scans over the whole archive cheap.
    pub fn read_head(&self, file_id: u32, n: usize) -> Result<Vec<u8>> {
        self.read_limited(file_id, Some(n))
    }

    /// All MFT entries (0-based, header slot excluded), including ones that no file id
    /// points at, such as the archive's own index entries.
    pub fn mft_entries(&self) -> &[MftEntry] {
        &self.entries
    }

    /// The raw file index as `(file id, 0-based MFT entry)` pairs, in on-disk order.
    /// Several ids can name the same entry (an asset's base id and current file id).
    pub fn index_pairs(&self) -> &[(u32, usize)] {
        &self.pairs
    }

    /// Decoded size of an MFT entry (0-based): the size field of the compression
    /// header for compressed entries, else the stored size minus the 4-byte trailer
    /// of every 64 KiB block. Reads at most 8 bytes.
    pub fn decoded_size(&self, mft: usize) -> Result<u32> {
        let entry = *self.entries.get(mft).ok_or(ErrorKind::NotFound)?;
        match entry.extra_bytes {
            0 => {
                let blocks = (entry.size as usize).div_ceil(BLOCK_SIZE) as u32;
                Ok(entry.size.saturating_sub(4 * blocks))
            }
            8 => {
                let end = entry.offset.checked_add(8).ok_or_else(|| invalid("entry out of bounds"))?;
                if entry.size < 8 || end > self.len {
                    return Err(invalid("compressed entry too short"));
                }
                let mut h = [0u8; 8];
                {
                    let mut reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
                    reader.seek(SeekFrom::Start(entry.offset))?;
                    reader.read_exact(&mut h)?;
                }
                match h[..4] {
                    [0x08, 0x00, 0x01, 0x80] => Ok(u32::from_le_bytes([h[4], h[5], h[6], h[7]])),
                    [0x80, 0x01, 0x00, 0x08] => Ok(u32::from_be_bytes([h[4], h[5], h[6], h[7]])),
                    _ => Err(invalid("invalid compression header")),
                }
            }
            _ => Err(invalid("unknown MFT extra_bytes")),
        }
    }

    /// Like [`Archive::read_head`], but for a raw MFT entry number (0-based).
    pub fn read_mft_head(&self, mft: usize, n: usize) -> Result<Vec<u8>> {
        let entry = *self.entries.get(mft).ok_or(ErrorKind::NotFound)?;
        self.read_entry(&entry, Some(n))
    }

    /// Like [`Archive::read`], but for a raw MFT entry number (0-based).
    pub fn read_mft(&self, mft: usize) -> Result<Vec<u8>> {
        let entry = *self.entries.get(mft).ok_or(ErrorKind::NotFound)?;
        self.read_entry(&entry, None)
    }

    fn read_limited(&self, file_id: u32, head: Option<usize>) -> Result<Vec<u8>> {
        let entry = *self.index.get(&file_id).ok_or(ErrorKind::NotFound)?;
        self.read_entry(&entry, head)
    }

    fn read_entry(&self, entry: &MftEntry, head: Option<usize>) -> Result<Vec<u8>> {

        let full = entry.size as usize;
        let end = entry
            .offset
            .checked_add(u64::from(entry.size))
            .ok_or_else(|| invalid("entry out of bounds"))?;
        if end > self.len {
            return Err(invalid("entry out of bounds"));
        }
        // A head read stops after the first stored block.
        let size = if head.is_some() { full.min(BLOCK_SIZE) } else { full };

        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(size)
            .map_err(|_| Error::from(ErrorKind::OutOfMemory))?;

        {
            let mut reader = self.reader.lock().unwrap_or_else(PoisonError::into_inner);
            reader.seek(SeekFrom::Start(entry.offset))?;

            let mut block = vec![0u8; BLOCK_SIZE];
            let mut remaining = size;
            while remaining > 0 {
                let len = remaining.min(BLOCK_SIZE);
                reader.read_exact(&mut block[..len])?;

                // The last 4 bytes of every block are a trailer, not data.
                bytes.extend_from_slice(&block[..len.saturating_sub(4)]);

                remaining -= len;
            }
        }

        match entry.extra_bytes {
            0 => {
                if let Some(n) = head {
                    bytes.truncate(n);
                }
                Ok(bytes)
            }
            8 => {
                let header: [u8; 8] = bytes
                    .get(..8)
                    .and_then(|s| s.try_into().ok())
                    .ok_or_else(|| invalid("compressed entry too short"))?;
                let (magic, size) = header.split_at(4);
                let size: [u8; 4] = size.try_into().map_err(|_| invalid("bad header"))?;
                let output_len = match magic {
                    [0x08, 0x00, 0x01, 0x80] => u32::from_le_bytes(size),
                    [0x80, 0x01, 0x00, 0x08] => u32::from_be_bytes(size),
                    _ => return Err(invalid("invalid compression header")),
                };
                if output_len > MAX_DECOMPRESSED_SIZE {
                    return Err(invalid("decompressed size too large"));
                }
                let want = head.map_or(output_len as usize, |n| n.min(output_len as usize));

                let mut output = Vec::new();
                output
                    .try_reserve_exact(want)
                    .map_err(|_| Error::from(ErrorKind::OutOfMemory))?;
                output.resize(want, 0);
                decompress(&bytes[8..], &mut output)?;
                Ok(output)
            }
            _ => Err(invalid("unknown MFT extra_bytes")),
        }
    }
}

/// Read `size` bytes at `offset`, checking against the total stream length first.
fn read_range<R: Read + Seek>(
    reader: &mut R,
    total: u64,
    offset: u64,
    size: usize,
    msg: &'static str,
) -> Result<Vec<u8>> {
    let end = offset
        .checked_add(size as u64)
        .ok_or_else(|| invalid(msg))?;
    if end > total {
        return Err(invalid(msg));
    }
    let mut buf = vec![0u8; size];
    reader.seek(SeekFrom::Start(offset))?;
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::synthetic::{SynthFile, build_archive, rle_stream};

    fn payload(n: usize, seed: u8) -> Vec<u8> {
        (0..n)
            .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
            .collect()
    }

    #[test]
    fn reads_uncompressed_entries_across_blocks() {
        let small = payload(100, 1);
        let exact_block = payload(BLOCK_SIZE - 4, 2); // exactly one full stored block
        let multi = payload(150_000, 3); // spans three stored blocks
        let empty = Vec::new();
        let img = build_archive(&[
            SynthFile::plain(0x10, small.clone()),
            SynthFile::plain(0x20, exact_block.clone()),
            SynthFile::plain(0x30, multi.clone()),
            SynthFile::plain(0x40, empty.clone()),
        ]);

        let a = Archive::from_reader(Cursor::new(img)).unwrap();
        assert_eq!(a.len(), 4);
        assert!(!a.is_empty());
        let mut ids: Vec<u32> = a.file_ids().collect();
        ids.sort_unstable();
        assert_eq!(ids, [0x10, 0x20, 0x30, 0x40]);

        assert_eq!(a.read(0x10).unwrap(), small);
        assert_eq!(a.read(0x20).unwrap(), exact_block);
        assert_eq!(a.read(0x30).unwrap(), multi);
        assert_eq!(a.read(0x40).unwrap(), empty);
        assert!(a.contains(0x10) && !a.contains(0x99));
        assert_eq!(a.entry(0x10).unwrap().size, 104);
        assert_eq!(a.read(0x99).unwrap_err().kind(), ErrorKind::NotFound);
        // decoded_size agrees with a full read for every indexed entry
        for &(id, mft) in a.index_pairs() {
            assert_eq!(a.decoded_size(mft).unwrap() as usize, a.read(id).unwrap().len(), "id {id:#x}");
        }
    }

    #[test]
    fn reads_compressed_entry() {
        let (stream, expected) = rle_stream();
        let img = build_archive(&[SynthFile {
            file_id: 7,
            payload: stream,
            compressed_len: Some(expected.len() as u32),
        }]);
        let a = Archive::from_reader(Cursor::new(img)).unwrap();
        assert!(a.entry(7).unwrap().is_compressed());
        assert_eq!(a.read(7).unwrap(), expected);
        let (_, mft) = a.index_pairs()[0];
        assert_eq!(a.decoded_size(mft).unwrap() as usize, expected.len());
    }

    #[test]
    fn rejects_bad_magic_and_version() {
        let good = build_archive(&[SynthFile::plain(1, vec![1, 2, 3])]);

        let mut bad = good.clone();
        bad[1] = b'X';
        assert_eq!(
            Archive::from_reader(Cursor::new(bad)).err().unwrap().kind(),
            ErrorKind::InvalidData
        );

        let mut bad = good.clone();
        bad[0] = 150;
        assert!(Archive::from_reader(Cursor::new(bad)).is_err());

        // MFT magic.
        let mut bad = good.clone();
        let mft_offset = le_u64(&good, 0x18) as usize;
        bad[mft_offset] = b'X';
        assert_eq!(
            Archive::from_reader(Cursor::new(bad)).err().unwrap().kind(),
            ErrorKind::InvalidData
        );
    }

    #[test]
    fn rejects_truncated_and_inconsistent_archives_without_panicking() {
        let good = build_archive(&[
            SynthFile::plain(1, payload(1000, 9)),
            SynthFile::plain(2, payload(70_000, 4)),
        ]);
        assert!(Archive::from_reader(Cursor::new(Vec::new())).is_err());
        for end in (0..good.len()).step_by(7) {
            // Must not panic; opening or reading may legitimately fail.
            if let Ok(a) = Archive::from_reader(Cursor::new(good[..end].to_vec())) {
                let _ = a.read(1);
                let _ = a.read(2);
            }
        }

        // Bad MFT entry count.
        let mut bad = good.clone();
        let mft_offset = le_u64(&good, 0x18) as usize;
        bad[mft_offset + 12..mft_offset + 16].copy_from_slice(&0u32.to_le_bytes());
        assert!(Archive::from_reader(Cursor::new(bad)).is_err());

        // MFT size claims more than the file holds.
        let mut bad = good.clone();
        bad[0x20..0x24].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Archive::from_reader(Cursor::new(bad)).is_err());

        // Index entry pointing at a nonexistent MFT slot.
        let mut bad = good.clone();
        let entries_start = mft_offset + MFT_HEADER_SIZE;
        let index_offset = le_u64(&good, entries_start + MFT_ENTRY_SIZE) as usize;
        bad[index_offset + 12..index_offset + 16].copy_from_slice(&999u32.to_le_bytes());
        assert!(Archive::from_reader(Cursor::new(bad.clone())).is_err());
        bad[index_offset + 12..index_offset + 16].copy_from_slice(&0u32.to_le_bytes());
        assert!(Archive::from_reader(Cursor::new(bad)).is_err());
    }

    #[test]
    fn malformed_compressed_entries_error_cleanly() {
        // Truncated stream: header claims 5 bytes but only the first word is present.
        let (stream, _) = rle_stream();
        let img = build_archive(&[SynthFile {
            file_id: 7,
            payload: stream[..4].to_vec(),
            compressed_len: Some(5),
        }]);
        let a = Archive::from_reader(Cursor::new(img)).unwrap();
        assert!(a.read(7).is_err());

        // Absurd decompressed size.
        let img = build_archive(&[SynthFile {
            file_id: 7,
            payload: stream.clone(),
            compressed_len: Some(u32::MAX),
        }]);
        let a = Archive::from_reader(Cursor::new(img)).unwrap();
        assert_eq!(a.read(7).unwrap_err().kind(), ErrorKind::InvalidData);

        // extra_bytes = 8 but entry shorter than the 8-byte header.
        let mut img = build_archive(&[SynthFile::plain(7, vec![1, 2])]);
        let mft_offset = le_u64(&img, 0x18) as usize;
        let file_entry = mft_offset + MFT_HEADER_SIZE + 3 * MFT_ENTRY_SIZE;
        img[file_entry + 12..file_entry + 14].copy_from_slice(&8u16.to_le_bytes());
        let a = Archive::from_reader(Cursor::new(img)).unwrap();
        assert!(a.read(7).is_err());
    }
}
