//! Minimal writer for `PF` packfiles (32- or 64-bit relative pointers), the inverse of
//! [`crate::packfile`] + [`crate::pfread`].
//!
//! The layout is copied from real build-209026 data: `STAR` file 186291 has header
//! `"PF" 01 00 00 00 0c 00 <type>` (flags 1 = 32-bit pointers, header size 12).
//! A chunk is `name, u32 size (bytes after this field), u16 version, u16 header size
//! (16), u32 data size`, then the payload, then the fix-up table
//! (`u32 count, u32 slot offsets…`). Pointers are `i32` offsets relative to their own
//! slot (0 = null), and every non-null pointer slot is listed in the fix-up table.
//! 64-bit files (`mapc` 192711) use flags 5 and 8-byte pointer slots.
//!
//! The 64-bit client widens 32-bit packfiles on load and asserts `pointers.Count()`
//! (`Packfile.cpp` 0x27b, REA `FUN_140de8eb0`) when such a file has no fix-ups, so
//! files without pointers should be written as 64-bit.

/// Builds one chunk payload. Fields are appended in order (packed, no padding).
#[derive(Debug, Default, Clone)]
pub struct ChunkBuilder {
    data: Vec<u8>,
    fixups: Vec<u32>,
    ptr64: bool,
}

impl ChunkBuilder {
    /// A chunk with 32-bit pointer slots.
    pub fn new() -> Self {
        Self::default()
    }

    /// A chunk with 64-bit pointer slots (pair with `packfile(.., ptr64 = true, ..)`).
    pub fn new64() -> Self {
        Self { ptr64: true, ..Self::default() }
    }

    pub fn is_64bit(&self) -> bool {
        self.ptr64
    }

    /// Current end of the payload (where the next append lands).
    pub fn pos(&self) -> usize {
        self.data.len()
    }

    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.data.push(v);
        self
    }

    pub fn u16(&mut self, v: u16) -> &mut Self {
        self.data.extend_from_slice(&v.to_le_bytes());
        self
    }

    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.data.extend_from_slice(&v.to_le_bytes());
        self
    }

    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.data.extend_from_slice(&v.to_le_bytes());
        self
    }

    pub fn bytes(&mut self, b: &[u8]) -> &mut Self {
        self.data.extend_from_slice(b);
        self
    }

    /// Pads with zeros to a multiple of `n`.
    pub fn align(&mut self, n: usize) -> &mut Self {
        while !self.data.len().is_multiple_of(n) {
            self.data.push(0);
        }
        self
    }

    /// Reserves a null pointer slot and returns its offset; fill it with [`Self::point`].
    pub fn ptr_slot(&mut self) -> usize {
        let at = self.data.len();
        if self.ptr64 { self.u64(0) } else { self.u32(0) };
        at
    }

    /// Makes the pointer at `slot` point at payload offset `target`.
    pub fn point(&mut self, slot: usize, target: usize) {
        let rel = target as i64 - slot as i64;
        if self.ptr64 {
            self.data[slot..slot + 8].copy_from_slice(&rel.to_le_bytes());
        } else {
            let rel = i32::try_from(rel).expect("pointer offset fits i32");
            self.data[slot..slot + 4].copy_from_slice(&rel.to_le_bytes());
        }
        self.fixups.push(slot as u32);
    }

    /// Appends a NUL-terminated UTF-16LE string and returns its offset.
    pub fn wstr(&mut self, s: &str) -> usize {
        let at = self.data.len();
        for u in s.encode_utf16().chain(std::iter::once(0)) {
            self.u16(u);
        }
        at
    }

    /// Payload and fix-up table (`u32 count` + sorted slot offsets).
    fn finish(&self) -> (Vec<u8>, Vec<u8>) {
        let mut f = self.fixups.clone();
        f.sort_unstable();
        let mut table = (f.len() as u32).to_le_bytes().to_vec();
        for o in f {
            table.extend_from_slice(&o.to_le_bytes());
        }
        (self.data.clone(), table)
    }
}

/// Serializes a packfile of type `kind` with the given chunks `(name, version, payload)`.
/// All chunks must use the same pointer width; it sets the header flags (1 or 5).
pub fn packfile(kind: [u8; 4], chunks: &[([u8; 4], u16, &ChunkBuilder)]) -> Vec<u8> {
    let ptr64 = chunks.first().is_some_and(|c| c.2.ptr64);
    assert!(chunks.iter().all(|c| c.2.ptr64 == ptr64), "mixed pointer widths");
    let mut out = b"PF".to_vec();
    out.extend_from_slice(&(if ptr64 { 5u16 } else { 1u16 }).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&12u16.to_le_bytes()); // header size
    out.extend_from_slice(&kind);
    for (name, version, b) in chunks {
        let (data, table) = b.finish();
        let size = 8 + data.len() + table.len(); // bytes after the size field
        out.extend_from_slice(name);
        out.extend_from_slice(&(size as u32).to_le_bytes());
        out.extend_from_slice(&version.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&data);
        out.extend_from_slice(&table);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packfile::Packfile;

    #[test]
    fn header_matches_real_star_layout_and_fixups_round_trip() {
        let mut b = ChunkBuilder::new();
        b.u32(7);
        let p = b.ptr_slot();
        let s = b.wstr("hi");
        b.point(p, s);
        let blob = packfile(*b"TEST", &[(*b"TEST", 3, &b)]);
        assert_eq!(&blob[..12], b"PF\x01\x00\x00\x00\x0c\x00TEST");
        let pf = Packfile::new(blob).unwrap();
        assert!(!pf.is_64bit());
        let c = pf.chunks().next().unwrap().unwrap();
        assert_eq!((c.name(), c.version()), (*b"TEST", 3));
        assert_eq!(c.bytes().len(), 4 + 4 + 6);
        assert_eq!(c.fixup_offsets().unwrap(), vec![4]);
        assert_eq!(i32::from_le_bytes(c.bytes()[4..8].try_into().unwrap()), 4);
    }

    #[test]
    fn sixty_four_bit_header_and_slots() {
        let mut b = ChunkBuilder::new64();
        let p = b.ptr_slot();
        let s = b.wstr("x");
        b.point(p, s);
        let blob = packfile(*b"TEST", &[(*b"TEST", 0, &b)]);
        assert_eq!(&blob[..8], b"PF\x05\x00\x00\x00\x0c\x00");
        let pf = Packfile::new(blob).unwrap();
        assert!(pf.is_64bit());
        let c = pf.chunks().next().unwrap().unwrap();
        assert_eq!(i64::from_le_bytes(c.bytes()[..8].try_into().unwrap()), 8);
        assert_eq!(c.fixup_offsets().unwrap(), vec![0]);
    }
}
