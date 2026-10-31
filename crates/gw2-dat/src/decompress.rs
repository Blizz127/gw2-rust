// Ported from valaphee/ascalon (MIT, commit 03fb60b), rewritten for stable Rust.
//
//! The Gw2 Huffman + LZ77 decompressor.
//!
//! Bit layout: the stream is consumed as little-endian `u32` words, and bits
//! are taken MSB-first from each word.

use std::io::{Error, ErrorKind, Result};
use std::sync::LazyLock;

fn invalid() -> Error {
    ErrorKind::InvalidData.into()
}

fn eof() -> Error {
    ErrorKind::UnexpectedEof.into()
}

/// MSB-first bit reader over little-endian `u32` words.
pub struct BitReader<'a> {
    input: &'a [u8],
    word: u64,
    bits: u32,
    position: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            word: 0,
            bits: 0,
            position: 0,
        }
    }

    /// Look at the next `count` (0..=32) bits without consuming them.
    ///
    /// Only whole little-endian `u32` words are consumed from the input (a
    /// trailing partial word is ignored, as in the reference). Bits past the
    /// end read as zero; use [`BitReader::read`] to detect running out of input.
    pub fn peek(&mut self, count: u32) -> u32 {
        let count = count.min(32);
        if count == 0 {
            return 0;
        }

        if self.bits < count
            && let Some((word, rest)) = self.input.split_first_chunk::<4>()
        {
            self.input = rest;
            self.word |= u64::from(u32::from_le_bytes(*word)) << (32 - self.bits);
            self.bits += 32;
        }

        (self.word >> (64 - count)) as u32
    }

    /// Consume and return the next `count` (0..=32) bits.
    pub fn read(&mut self, count: u32) -> Result<u32> {
        if count > 32 || (self.bits as usize) + (self.input.len() / 4) * 32 < count as usize {
            return Err(eof());
        }

        let value = self.peek(count);
        let bits = self.bits.checked_sub(count).ok_or_else(eof)?;

        self.word <<= count;
        self.bits = bits;
        self.position += count as usize;

        Ok(value)
    }

    /// Number of bits consumed so far.
    pub fn position(&self) -> usize {
        self.position
    }
}

/// Decompress `input` into exactly `output.len()` bytes.
pub fn decompress(input: &[u8], output: &mut [u8]) -> Result<()> {
    let mut bits = BitReader::new(input);

    bits.read(4)?;
    let length_bias = bits.read(4)? + 1;

    let mut output_pos = 0;
    while output_pos < output.len() {
        let literals = read_tree(&mut bits)?;
        let offsets = read_tree(&mut bits)?;

        for _ in 0..(bits.read(4)? + 1) << 12 {
            if output_pos == output.len() {
                break;
            }

            let symbol = u32::from(literals.read(&mut bits)?);
            if symbol < 0x100 {
                output[output_pos] = symbol as u8;
                output_pos += 1;
                continue;
            }

            let symbol = symbol - 0x100;
            let group = symbol / 4;
            let mut length = match group {
                0 => symbol,
                1..=6 => (1u32 << (group - 1)) * (4 + symbol % 4),
                _ if symbol == 28 => 0xFF,
                _ => return Err(invalid()),
            };
            if group > 1 && symbol != 28 {
                length |= bits.read(group - 1)?;
            }
            length += length_bias;

            let symbol = u32::from(offsets.read(&mut bits)?);
            let group = symbol / 2;
            let mut offset = match group {
                0 => symbol,
                1..=16 => (1u32 << (group - 1)) * (2 + symbol % 2),
                _ => return Err(invalid()),
            };
            if group > 1 {
                offset |= bits.read(group - 1)?;
            }

            let offset = offset as usize + 1;
            if offset > output_pos {
                return Err(invalid());
            }

            let output_end = output_pos + (length as usize).min(output.len() - output_pos);
            while output_pos < output_end {
                let count = offset.min(output_end - output_pos);
                output.copy_within(output_pos - offset..output_pos - offset + count, output_pos);
                output_pos += count;
            }
        }
    }

    Ok(())
}

const HASH_BITS: usize = 8;
const HASH_SIZE: usize = 1 << HASH_BITS;
const MAX_CODE_BITS: usize = 32;
const MAX_SYMBOLS: usize = 285;

struct HuffmanTreeBuilder {
    head: [u16; MAX_CODE_BITS],
    next: [u16; MAX_SYMBOLS],
}

impl HuffmanTreeBuilder {
    const NONE: u16 = u16::MAX;

    fn new() -> Self {
        Self {
            head: [Self::NONE; MAX_CODE_BITS],
            next: [Self::NONE; MAX_SYMBOLS],
        }
    }

    fn add(&mut self, symbol: u16, bits: u8) -> Result<()> {
        let symbol = symbol as usize;
        let bits = bits as usize;
        if symbol >= MAX_SYMBOLS || bits >= MAX_CODE_BITS {
            return Err(invalid());
        }

        self.next[symbol] = self.head[bits];
        self.head[bits] = symbol as u16;

        Ok(())
    }

    fn build(self) -> Result<Option<HuffmanTree>> {
        let mut hash_bits = [0; HASH_SIZE];
        let mut hash_symbol = [0; HASH_SIZE];
        let mut long = [LongEntry::default(); MAX_CODE_BITS - HASH_BITS];
        let mut long_len = 0;
        let mut symbols = [0; MAX_SYMBOLS];

        let mut code = 0u32;
        let mut any = false;
        let mut symbol_len = 0;

        for width in 0..MAX_CODE_BITS {
            let mut symbol = self.head[width];
            if symbol != Self::NONE {
                any = true;
            }

            if width <= HASH_BITS {
                while symbol != Self::NONE {
                    if width == 0 {
                        return Err(invalid());
                    }

                    let shift = HASH_BITS - width;
                    let start = (code as usize) << shift;
                    let end = (code.wrapping_add(1) as usize) << shift;
                    if end > HASH_SIZE {
                        return Err(invalid());
                    }

                    // A wrapped (over-subscribed) code yields start >= end,
                    // i.e. an empty range, exactly as in the reference.
                    for entry in start..end {
                        hash_bits[entry] = width as u8;
                        hash_symbol[entry] = symbol;
                    }

                    code = code.wrapping_sub(1);
                    symbol = self.next[symbol as usize];
                }
            } else {
                let start = symbol_len;

                while symbol != Self::NONE {
                    if symbol_len >= MAX_SYMBOLS {
                        return Err(invalid());
                    }

                    symbols[symbol_len] = symbol;
                    symbol_len += 1;

                    code = code.wrapping_sub(1);
                    symbol = self.next[symbol as usize];
                }

                if symbol_len != start {
                    if long_len >= long.len() {
                        return Err(invalid());
                    }

                    long[long_len] = LongEntry {
                        comparison: code.wrapping_add(1).wrapping_shl(32 - width as u32),
                        bits: width as u8,
                        end: (symbol_len - 1) as u16,
                    };
                    long_len += 1;
                }
            }

            code = code.wrapping_shl(1).wrapping_add(1);
        }

        Ok(any.then(|| {
            HuffmanTree::Multi(Box::new(MultiTree {
                hash_bits,
                hash_symbol,
                long,
                long_len,
                symbols,
            }))
        }))
    }
}

/// Code lengths of the fixed "dictionary" tree used to encode the code
/// lengths of every other tree.
fn build_dictionary() -> Result<HuffmanTree> {
    let mut lengths = [16u8; 256];

    macro_rules! lengths {
        ($($bits:expr => $($symbol:expr),+ $(,)?);+ $(;)?) => {
            $(
                $(lengths[$symbol] = $bits;)*
            )*
        };
    }

    lengths! {
        3 => 0x08, 0x09, 0x0A;
        4 => 0x00, 0x07, 0x0B, 0x0C;
        5 => 0x06, 0x29, 0x2A, 0xE0;
        6 => 0x04, 0x05, 0x20, 0x28, 0x2B, 0x2C, 0x40, 0x4A;
        7 => 0x03, 0x0D, 0x25, 0x26, 0x27, 0x48, 0x49;
        8 => 0x24, 0x47, 0x4B, 0x4C, 0x69, 0x6A;
        9 => 0x23, 0x46, 0x60, 0x63, 0x67, 0x68, 0x88, 0x89, 0xA0, 0xE8;
        10 => 0x01, 0x02, 0x2D, 0x43, 0x44, 0x45, 0x65, 0x66, 0x80, 0x87, 0x8A, 0xA8, 0xA9, 0xC0, 0xC9, 0xE9;
        11 => 0x0E, 0x4D, 0x64, 0x6B, 0x6C, 0x84, 0x85, 0x8B, 0xA4, 0xA5, 0xAA, 0xC8, 0xE5;
        12 => 0x83, 0x86, 0xA6, 0xA7, 0xC7, 0xCA, 0xE7;
        13 => 0x22, 0x2E, 0x8C, 0xC4, 0xE4, 0xE6;
        14 => 0x4E, 0x6D, 0xC6, 0xEC;
        15 => 0x0F, 0x10, 0x11, 0x8D, 0xAB, 0xAC, 0xCC, 0xEA;
    }

    let mut builder = HuffmanTreeBuilder::new();
    for symbol in (0..256u16).rev() {
        builder.add(symbol, lengths[symbol as usize])?;
    }

    builder.build()?.ok_or_else(invalid)
}

static DICTIONARY: LazyLock<Result<HuffmanTree>> = LazyLock::new(build_dictionary);

fn dictionary() -> Result<&'static HuffmanTree> {
    DICTIONARY
        .as_ref()
        .map_err(|e| Error::new(e.kind(), "invalid built-in dictionary tree"))
}

fn read_tree(bits: &mut BitReader<'_>) -> Result<HuffmanTree> {
    let dictionary = dictionary()?;

    let count = bits.read(16)? as u16;
    if count > MAX_SYMBOLS as u16 {
        return Err(invalid());
    }

    let mut builder = HuffmanTreeBuilder::new();

    let mut remaining = i32::from(count) - 1;
    let single = remaining as u16;

    while remaining >= 0 {
        let code = dictionary.read(bits)?;

        let width = (code & 0x1F) as u8;
        let mut count = i32::from(code >> 5) + 1;

        if width == 0 {
            remaining -= count;
            continue;
        }

        while count > 0 && remaining >= 0 {
            builder.add(remaining as u16, width)?;
            remaining -= 1;
            count -= 1;
        }
    }

    Ok(builder.build()?.unwrap_or(HuffmanTree::Single(single)))
}

/// Decode one symbol of the built-in dictionary tree (test support).
#[cfg(test)]
pub(crate) fn dictionary_decode(bits: &mut BitReader<'_>) -> Result<u16> {
    dictionary()?.read(bits)
}

#[derive(Clone, Copy, Default)]
struct LongEntry {
    comparison: u32,
    bits: u8,
    end: u16,
}

struct MultiTree {
    hash_bits: [u8; HASH_SIZE],
    hash_symbol: [u16; HASH_SIZE],

    long: [LongEntry; MAX_CODE_BITS - HASH_BITS],
    long_len: usize,

    symbols: [u16; MAX_SYMBOLS],
}

enum HuffmanTree {
    Single(u16),
    Multi(Box<MultiTree>),
}

impl HuffmanTree {
    fn read(&self, bits: &mut BitReader<'_>) -> Result<u16> {
        match self {
            Self::Single(symbol) => Ok(*symbol),
            Self::Multi(tree) => {
                let MultiTree {
                    hash_bits,
                    hash_symbol,
                    long,
                    long_len,
                    symbols,
                } = &**tree;
                let prefix = bits.peek(HASH_BITS as u32) as usize;
                let width = hash_bits[prefix];
                if width != 0 {
                    bits.read(u32::from(width))?;
                    return Ok(hash_symbol[prefix]);
                }

                let value = bits.peek(32);
                for entry in &long[..*long_len] {
                    if value < entry.comparison {
                        continue;
                    }

                    let delta =
                        ((value - entry.comparison) >> (32 - u32::from(entry.bits))) as usize;
                    let end = entry.end as usize;
                    if delta > end {
                        return Err(invalid());
                    }

                    bits.read(u32::from(entry.bits))?;

                    return Ok(symbols[end - delta]);
                }

                Err(invalid())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthetic::{BitWriter, rle_stream};

    #[test]
    fn bit_reader_reads_msb_first_from_le_words() {
        // Word 0x8000_0001 (LE bytes 01 00 00 80): first bit 1, last bit 1.
        let data = [0x01, 0x00, 0x00, 0x80, 0xFF, 0x00, 0x00, 0x00];
        let mut r = BitReader::new(&data);
        assert_eq!(r.read(1).unwrap(), 1);
        assert_eq!(r.read(30).unwrap(), 0);
        assert_eq!(r.read(1).unwrap(), 1);
        assert_eq!(r.position(), 32);
        // Second word is 0x0000_00FF: 24 zero bits then 8 ones.
        assert_eq!(r.peek(8), 0);
        assert_eq!(r.read(24).unwrap(), 0);
        assert_eq!(r.read(8).unwrap(), 0xFF);
        assert!(r.read(1).is_err());
    }

    #[test]
    fn bit_reader_straddles_words_and_reads_32() {
        let data = [0xEF, 0xBE, 0xAD, 0xDE, 0x78, 0x56, 0x34, 0x12];
        let mut r = BitReader::new(&data);
        assert_eq!(r.read(4).unwrap(), 0xD);
        assert_eq!(r.read(32).unwrap(), 0xEADB_EEF1);
        assert_eq!(r.read(28).unwrap(), 0x234_5678);
        assert!(r.read(1).is_err());
    }

    #[test]
    fn bit_reader_ignores_partial_trailing_word() {
        // 5 bytes: one whole word plus a 1-byte tail that is never readable.
        let data = [0, 0, 0, 0x80, 0xFF];
        let mut r = BitReader::new(&data);
        assert_eq!(r.read(1).unwrap(), 1);
        assert_eq!(r.read(31).unwrap(), 0);
        assert!(r.read(1).is_err());
        assert_eq!(r.peek(8), 0);

        let mut r = BitReader::new(&data[..3]);
        assert!(r.read(1).is_err());
        let mut r = BitReader::new(&data[..4]);
        assert!(r.read(33).is_err());
        assert_eq!(r.read(0).unwrap(), 0);
        assert_eq!(r.read(32).unwrap(), 0x8000_0000);
    }

    #[test]
    fn dictionary_codes_match_hand_computed_canonical_assignment() {
        // Codes count down from all-ones: 3-bit {08,09,0A} = 111,110,101;
        // 4-bit {00,07,0B,0C} = 1001,1000,0111,0110.
        let cases: [(u32, u32, u16); 7] = [
            (0b111, 3, 0x08),
            (0b110, 3, 0x09),
            (0b101, 3, 0x0A),
            (0b1001, 4, 0x00),
            (0b1000, 4, 0x07),
            (0b0111, 4, 0x0B),
            (0b0110, 4, 0x0C),
        ];
        for &(code, len, symbol) in &cases {
            let mut w = BitWriter::default();
            w.push(code, len);
            let bytes = w.finish();
            let mut r = BitReader::new(&bytes);
            assert_eq!(dictionary_decode(&mut r).unwrap(), symbol, "code {code:b}");
            assert_eq!(r.position(), len as usize);
        }
    }

    #[test]
    fn rle_stream_decompresses() {
        let (stream, expected) = rle_stream();
        let mut out = vec![0u8; expected.len()];
        decompress(&stream, &mut out).unwrap();
        assert_eq!(out, expected);
        assert_eq!(out, [2, 1, 1, 1, 1]);

        // Shorter output clips the final match.
        let mut out = vec![0u8; 3];
        decompress(&stream, &mut out).unwrap();
        assert_eq!(out, [2, 1, 1]);
    }

    #[test]
    fn empty_output_needs_no_valid_input_beyond_header() {
        let mut out = [];
        assert!(decompress(&[], &mut out).is_err());
        assert!(decompress(&[0, 0, 0, 0], &mut out).is_ok());
    }

    #[test]
    fn truncated_input_errors_cleanly() {
        let (stream, expected) = rle_stream();
        let mut out = vec![0u8; expected.len()];
        assert!(decompress(&[], &mut out).is_err());
        assert!(decompress(&stream[..4], &mut out).is_err());
        // Every strict prefix must return (Ok or Err) without panicking.
        let mut errors = 0;
        for end in 0..stream.len() {
            if decompress(&stream[..end], &mut out).is_err() {
                errors += 1;
            }
        }
        assert!(errors > 0);
    }

    #[test]
    fn corrupted_and_random_input_never_panics() {
        let (stream, _) = rle_stream();
        let mut out = vec![0u8; 64];
        for i in 0..stream.len() * 8 {
            let mut bad = stream.clone();
            bad[i / 8] ^= 1 << (i % 8);
            let _ = decompress(&bad, &mut out);
        }

        let mut state = 0x1234_5678_9ABC_DEF0u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for _ in 0..2000 {
            let len = (next() % 96) as usize;
            let data: Vec<u8> = (0..len).map(|_| next() as u8).collect();
            let _ = decompress(&data, &mut out);
        }
    }

    #[test]
    fn builtin_dictionary_builds() {
        assert!(dictionary().is_ok());
    }
}
