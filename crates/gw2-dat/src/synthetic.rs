//! SYNTHETIC test fixtures: archives, bit streams and packfiles built in memory.
//!
//! Nothing here is real Gw2.dat data; the layouts mirror what the readers
//! expect (see the ascalon port) so the readers can be exercised without the
//! proprietary archive.

use crate::archive::{AN_HEADER_SIZE, BLOCK_SIZE, MFT_ENTRY_SIZE, MFT_HEADER_SIZE};

/// MSB-first bit writer producing little-endian `u32` words (inverse of `BitReader`).
#[derive(Default)]
pub struct BitWriter {
    bits: Vec<bool>,
}

impl BitWriter {
    pub fn push(&mut self, value: u32, count: u32) {
        for i in (0..count).rev() {
            self.bits.push((value >> i) & 1 == 1);
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        while !self.bits.len().is_multiple_of(32) || self.bits.is_empty() {
            self.bits.push(false);
        }
        let mut out = Vec::new();
        for word in self.bits.chunks(32) {
            let w = word.iter().fold(0u32, |a, &b| (a << 1) | u32::from(b));
            out.extend_from_slice(&w.to_le_bytes());
        }
        out
    }
}

/// Dictionary symbol meaning "width `w`, applies to `n` (1..=8) consecutive symbols".
fn dict_run(width: u32, n: u32) -> u32 {
    ((n - 1) << 5) | width
}

/// Encode a built-in dictionary symbol by brute-forcing its code with the decoder.
fn push_dictionary_symbol(w: &mut BitWriter, symbol: u32) {
    for len in 1..=16u32 {
        for value in 0..(1u32 << len) {
            let mut probe = BitWriter::default();
            probe.push(value, len);
            let bytes = probe.finish();
            let mut r = crate::decompress::BitReader::new(&bytes);
            if let Ok(s) = crate::decompress::dictionary_decode(&mut r)
                && u32::from(s) == symbol
                && r.position() == len as usize
            {
                w.push(value, len);
                return;
            }
        }
    }
    panic!("no dictionary code for {symbol:#x}");
}

/// A tiny hand-assembled compressed stream and its expected output for an
/// output length of 5: `[2, 1, 1, 1, 1]`.
///
/// Literal tree (count 257): symbol 1 width 1 (`1`), symbol 2 width 2 (`01`),
/// symbol 0x100 (match, length code 0) width 2 (`00`). Offset tree: a single
/// symbol 0 (distance 1). Length bias is 3.
pub fn rle_stream() -> (Vec<u8>, Vec<u8>) {
    let mut w = BitWriter::default();
    w.push(0, 4);
    w.push(2, 4); // length_bias = 3

    // Literal tree, walking symbols 256 down to 0.
    w.push(257, 16);
    push_dictionary_symbol(&mut w, dict_run(2, 1)); // 256: width 2
    for _ in 0..31 {
        push_dictionary_symbol(&mut w, dict_run(0, 8)); // 255..8 (x31 = 248 symbols)
    }
    push_dictionary_symbol(&mut w, dict_run(0, 5)); // 7..3 -> total 253 skipped
    push_dictionary_symbol(&mut w, dict_run(2, 1)); // 2: width 2
    push_dictionary_symbol(&mut w, dict_run(1, 1)); // 1: width 1
    push_dictionary_symbol(&mut w, dict_run(0, 1)); // 0: unused

    // Offset tree: count 1, skip -> Single(0).
    w.push(1, 16);
    push_dictionary_symbol(&mut w, dict_run(0, 1));

    w.push(0, 4); // 4096 symbols in this block

    w.push(0b01, 2); // literal 2
    w.push(0b1, 1); // literal 1
    w.push(0b00, 2); // match: length 3, distance 1

    (w.finish(), vec![2, 1, 1, 1, 1])
}

/// One file to place in a synthetic archive.
pub struct SynthFile {
    pub file_id: u32,
    pub payload: Vec<u8>,
    /// `Some(output_len)` → mark compressed (`extra_bytes = 8`) and prepend the
    /// 8-byte compression header; `payload` is then the raw compressed stream.
    pub compressed_len: Option<u32>,
}

impl SynthFile {
    pub fn plain(file_id: u32, payload: Vec<u8>) -> Self {
        Self {
            file_id,
            payload,
            compressed_len: None,
        }
    }
}

/// Split `data` into stored form: each 64 KiB block carries a 4-byte trailer
/// (stand-in for the per-block CRC), so a block holds `BLOCK_SIZE - 4` data bytes.
pub fn with_block_trailers(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for chunk in data.chunks(BLOCK_SIZE - 4) {
        out.extend_from_slice(chunk);
        out.extend_from_slice(&[0xCC; 4]);
    }
    out
}

/// Build a synthetic AN v151 archive: header, entry data, index, MFT.
///
/// MFT slots (1-based after the header): 1 = MFT placeholder, 2 = file-id
/// index, 3 = placeholder, 4.. = files in order.
pub fn build_archive(files: &[SynthFile]) -> Vec<u8> {
    let mut out = vec![0u8; AN_HEADER_SIZE];
    let mut entries: Vec<(u64, u32, u16)> = vec![(0, 0, 0), (0, 0, 0), (0, 0, 0)];

    for f in files {
        let mut data = Vec::new();
        let extra = if let Some(len) = f.compressed_len {
            data.extend_from_slice(&[0x08, 0x00, 0x01, 0x80]);
            data.extend_from_slice(&len.to_le_bytes());
            8
        } else {
            0
        };
        data.extend_from_slice(&f.payload);
        let stored = with_block_trailers(&data);
        entries.push((out.len() as u64, stored.len() as u32, extra));
        out.extend_from_slice(&stored);
    }

    // Index: padding entry (file_id 0), then one per file.
    let mut index = vec![0u8; 8];
    for (i, f) in files.iter().enumerate() {
        index.extend_from_slice(&f.file_id.to_le_bytes());
        index.extend_from_slice(&((i as u32) + 4).to_le_bytes());
    }
    entries[1] = (out.len() as u64, index.len() as u32, 0);
    out.extend_from_slice(&index);

    let mft_offset = out.len() as u64;
    let mut mft = Vec::new();
    mft.extend_from_slice(b"Mft\x1A");
    mft.extend_from_slice(&[0; 8]);
    mft.extend_from_slice(&((entries.len() as u32) + 1).to_le_bytes());
    mft.extend_from_slice(&[0; 8]);
    assert_eq!(mft.len(), MFT_HEADER_SIZE);
    for &(offset, size, extra) in &entries {
        let start = mft.len();
        mft.extend_from_slice(&offset.to_le_bytes());
        mft.extend_from_slice(&size.to_le_bytes());
        mft.extend_from_slice(&extra.to_le_bytes());
        mft.extend_from_slice(&[1, 0]); // flags: used, stream 0
        mft.extend_from_slice(&[0; 8]); // next_stream, crc
        assert_eq!(mft.len() - start, MFT_ENTRY_SIZE);
    }
    out.extend_from_slice(&mft);

    out[0] = 151;
    out[1..4].copy_from_slice(b"AN\x1A");
    out[0x18..0x20].copy_from_slice(&mft_offset.to_le_bytes());
    out[0x20..0x24].copy_from_slice(&(mft.len() as u32).to_le_bytes());
    out
}

/// `(name, version, data, extra)` of a synthetic PF chunk.
pub type SynthChunk = ([u8; 4], u16, Vec<u8>, Vec<u8>);

/// Build a synthetic PF blob with the given `(name, version, data, extra)` chunks.
pub fn build_packfile(flags: u16, kind: [u8; 4], chunks: &[SynthChunk]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"PF");
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&12u16.to_le_bytes());
    out.extend_from_slice(&kind);
    for (name, version, data, extra) in chunks {
        out.extend_from_slice(name);
        let size = 8 + data.len() + extra.len();
        out.extend_from_slice(&(size as u32).to_le_bytes());
        out.extend_from_slice(&version.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        let data_size = if extra.is_empty() { 0 } else { data.len() };
        out.extend_from_slice(&(data_size as u32).to_le_bytes());
        out.extend_from_slice(data);
        out.extend_from_slice(extra);
    }
    out
}
