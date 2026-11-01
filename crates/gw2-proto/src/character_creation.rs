//! Bounded inspection of a complete build-209026 client creation frame.
//!
//! Evidence: `docs/re/209026-character-creation.md` and its answer/color notes.
//! Framing and the known context scalar checks do not validate game content.

use std::io::{Error, ErrorKind, Result};

use crate::Decode;

pub const MESSAGE_ID: u16 = 415;
pub const MAX_FRAME_SIZE: usize = 250;

/// Parsed wire data, deliberately without Debug to avoid accidental disclosure.
/// The fixed arrays bound memory use independently of untrusted counts.
pub struct CreationRequest209026 {
    name: [u16; 19],
    name_len: usize,
    appearance: [u8; 48],
    profession: u8,
    level: u8,
    flags: u32,
    answers: [u8; 32],
    answer_len: usize,
    item_colors: [[u32; 4]; 6],
}

impl CreationRequest209026 {
    /// Parses exactly one frame, including its little-endian message ID.
    pub fn parse(mut bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_FRAME_SIZE {
            return Err(invalid("creation frame exceeds maximum size"));
        }
        if u16::decode(&mut bytes)? != MESSAGE_ID {
            return Err(invalid("unexpected creation message ID"));
        }
        let mut name = [0; 19];
        let mut name_len = 0;
        loop {
            let unit = u16::decode(&mut bytes)?;
            if unit == 0 {
                break;
            }
            if name_len == name.len() {
                return Err(invalid("creation name exceeds terminated bound"));
            }
            name[name_len] = unit;
            name_len += 1;
        }
        let appearance = <[u8; 48]>::decode(&mut bytes)?;
        let profession = u8::decode(&mut bytes)?;
        let level = u8::decode(&mut bytes)?;
        let flags = u32::decode(&mut bytes)?;
        let answer_len = usize::from(u8::decode(&mut bytes)?);
        let mut answers = [0; 32];
        if answer_len > answers.len() {
            return Err(invalid("creation answer count exceeds maximum"));
        }
        for answer in &mut answers[..answer_len] {
            *answer = u8::decode(&mut bytes)?;
        }
        let mut item_colors = [[0; 4]; 6];
        for slot in &mut item_colors {
            *slot = <[u32; 4]>::decode(&mut bytes)?;
        }
        if !bytes.is_empty() {
            return Err(invalid("trailing bytes after creation frame"));
        }
        Ok(Self {
            name,
            name_len,
            appearance,
            profession,
            level,
            flags,
            answers,
            answer_len,
            item_colors,
        })
    }

    /// Raw UTF-16 code units without the terminator; text validity is unchecked.
    pub fn name_units(&self) -> &[u16] {
        &self.name[..self.name_len]
    }

    pub fn appearance(&self) -> &[u8; 48] {
        &self.appearance
    }

    pub fn profession(&self) -> u8 {
        self.profession
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Content answer bytes, retained in wire order without sorting.
    pub fn answers(&self) -> &[u8] {
        &self.answers[..self.answer_len]
    }

    /// Equipment slots 2–7 in order, each containing color slots 0–3.
    pub fn item_colors(&self) -> &[[u32; 4]; 6] {
        &self.item_colors
    }

    /// Only the mapped scalar assertions from the creation context.
    /// In particular, profession 0/10 pass this bound but are not established
    /// playable choices. No content, name policy or server validation occurs.
    pub fn passes_context_scalar_checks(&self) -> bool {
        self.name_len != 0
            && (1..=80).contains(&self.level)
            && self.profession < 11
            && self.appearance[27] < 5
            && self.appearance[29] < 2
            && [2, 4, 12, 14, 16, 28, 30, 31, 32]
                .iter()
                .all(|&offset| self.appearance[offset] != 0xff)
    }
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boundary_fixture() -> Vec<u8> {
        // Handwritten framing fixture, not valid game content. The unpaired
        // UTF-16 surrogate and unsorted answers must remain lossless.
        let mut bytes = vec![0x9f, 1, 0x00, 0xd8, 0, 0];
        let mut appearance = [0x80; 48];
        appearance[27] = 4;
        appearance[29] = 1;
        appearance[45..].copy_from_slice(&[0xad, 0xbe, 0xef]);
        bytes.extend(appearance);
        bytes.extend([9, 80, 0xff, 0xff, 0xff, 0xff, 0x0f, 3, 0x80, 0, 0xff]);
        // First row: 0, 127, 128, 300. Second row begins u32::MAX.
        bytes.extend([0, 127, 0x80, 1, 0xac, 2, 0xff, 0xff, 0xff, 0xff, 0x0f]);
        bytes.extend([0; 19]);
        bytes
    }

    #[test]
    fn framing_preserves_values_and_rejects_malformed_boundaries() {
        let bytes = boundary_fixture();
        let parsed = CreationRequest209026::parse(&bytes).unwrap();
        assert_eq!(parsed.name_units(), &[0xd800]);
        assert_eq!(&parsed.appearance()[45..], &[0xad, 0xbe, 0xef]);
        assert_eq!((parsed.profession(), parsed.level()), (9, 80));
        assert_eq!(parsed.flags(), u32::MAX);
        assert_eq!(parsed.answers(), &[0x80, 0, 0xff]);
        assert_eq!(parsed.item_colors()[0], [0, 127, 128, 300]);
        assert_eq!(parsed.item_colors()[1], [u32::MAX, 0, 0, 0]);
        assert!(parsed.passes_context_scalar_checks());
        for cut in 0..bytes.len() {
            assert!(CreationRequest209026::parse(&bytes[..cut]).is_err());
        }
        for (offset, bad) in [(0, 0), (61, 33), (60, 0x10), (60, 0x80), (75, 0x10)] {
            let mut altered = bytes.clone();
            altered[offset] = bad;
            assert!(CreationRequest209026::parse(&altered).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(CreationRequest209026::parse(&trailing).is_err());
        let mut unterminated = vec![0x9f, 1];
        for _ in 0..20 {
            unterminated.extend([1, 0]);
        }
        assert!(CreationRequest209026::parse(&unterminated).is_err());

        let mut maximum = vec![0x9f, 1];
        for _ in 0..19 {
            maximum.extend([65, 0]);
        }
        maximum.extend([0; 50]); // Name terminator plus all 48 appearance bytes.
        maximum.extend([1, 1, 0xff, 0xff, 0xff, 0xff, 0x0f, 32]);
        maximum.extend([0; 32]);
        for _ in 0..24 {
            maximum.extend([0xff, 0xff, 0xff, 0xff, 0x0f]);
        }
        assert_eq!(maximum.len(), MAX_FRAME_SIZE);
        let parsed = CreationRequest209026::parse(&maximum).unwrap();
        assert_eq!(parsed.name_units().len(), 19);
        assert_eq!(parsed.answers().len(), 32);
        assert_eq!(parsed.item_colors(), &[[u32::MAX; 4]; 6]);
        maximum.push(0);
        assert!(CreationRequest209026::parse(&maximum).is_err());
    }

    #[test]
    fn scalar_checks_are_separate_from_framing_and_content_validity() {
        let bytes = boundary_fixture();
        for (offset, bad) in [(54, 11), (55, 0), (55, 81), (6 + 27, 5), (6 + 29, 2)]
            .into_iter()
            .chain([2, 4, 12, 14, 16, 28, 30, 31, 32].map(|i| (6 + i, 0xff)))
        {
            let mut altered = bytes.clone();
            altered[offset] = bad;
            let parsed = CreationRequest209026::parse(&altered).unwrap();
            assert!(!parsed.passes_context_scalar_checks());
        }
        for profession in [0, 10] {
            let mut altered = bytes.clone();
            altered[54] = profession;
            assert!(
                CreationRequest209026::parse(&altered)
                    .unwrap()
                    .passes_context_scalar_checks()
            );
        }
        let mut empty_name = bytes;
        empty_name.drain(2..4);
        let parsed = CreationRequest209026::parse(&empty_name).unwrap();
        assert!(parsed.name_units().is_empty());
        assert!(!parsed.passes_context_scalar_checks());
    }
}
