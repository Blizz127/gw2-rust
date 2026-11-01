//! Build-209026 skill-bar lifecycle and selection frames, IDs 618–623.
//! See `docs/re/209026-character-skillbar.md` for effects and ordering limits.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillbarOperation209026 {
    /// 618 allocates a skill bar; it carries no skill records.
    Create,
    /// 619 emits a character event for an existing skill bar.
    Notify,
    /// 620 removes an existing skill bar.
    Remove,
    /// 621 clears a slot flag whose gameplay name remains unresolved.
    ClearSlotFlag(u8),
    /// 622 selects one slot; raw 255 is the client's no-selection sentinel.
    AutoAttackSlot(u8),
    /// 623 supplies a mask; zero clears selection.
    AutoAttackMask(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillbarFrame209026 {
    pub character_index: u32,
    pub operation: SkillbarOperation209026,
}

impl SkillbarFrame209026 {
    /// Append a complete frame without choosing or validating gameplay values.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use SkillbarOperation209026::*;
        let id: u16 = match self.operation {
            Create => 618,
            Notify => 619,
            Remove => 620,
            ClearSlotFlag(_) => 621,
            AutoAttackSlot(_) => 622,
            AutoAttackMask(_) => 623,
        };
        id.encode(output);
        self.character_index.encode(output);
        match self.operation {
            Create | Notify | Remove => {}
            ClearSlotFlag(slot) | AutoAttackSlot(slot) => slot.encode(output),
            AutoAttackMask(mask) => mask.encode(output),
        }
    }

    /// Consume one frame transactionally, retaining raw slots and mask bits.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use SkillbarOperation209026::*;
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !(618..=623).contains(&id) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected skill-bar message 618 through 623",
            ));
        }
        let character_index = u32::decode(&mut cursor)?;
        let operation = match id {
            618 => Create,
            619 => Notify,
            620 => Remove,
            621 => ClearSlotFlag(u8::decode(&mut cursor)?),
            622 => AutoAttackSlot(u8::decode(&mut cursor)?),
            623 => AutoAttackMask(u32::decode(&mut cursor)?),
            _ => unreachable!(),
        };
        *input = cursor;
        Ok(Self {
            character_index,
            operation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> [(Vec<u8>, SkillbarOperation209026); 6] {
        use SkillbarOperation209026::*;
        [
            ("6a 02 ac 02", Create),
            ("6b 02 ac 02", Notify),
            ("6c 02 ac 02", Remove),
            ("6d 02 ac 02 fe", ClearSlotFlag(254)),
            ("6e 02 ac 02 ff", AutoAttackSlot(255)),
            ("6f 02 ac 02 ff ff ff ff 0f", AutoAttackMask(u32::MAX)),
        ]
        .map(|(text, operation)| {
            (
                text.split_whitespace()
                    .map(|b| u8::from_str_radix(b, 16).unwrap())
                    .collect(),
                operation,
            )
        })
    }

    #[test]
    fn skillbar_golden_variants_preserve_raw_slots_and_masks() {
        for (golden, operation) in fixtures() {
            let expected = SkillbarFrame209026 {
                character_index: 300,
                operation,
            };
            let mut input = golden.as_slice();
            assert_eq!(
                SkillbarFrame209026::decode_frame(&mut input).unwrap(),
                expected
            );
            assert!(input.is_empty());
            let mut output = BytesMut::from(&b"prefix"[..]);
            expected.encode_frame(&mut output);
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
        }
        // Zero mask, distinct from the byte sentinel used by message 622.
        let mut input = &b"\x6f\x02\x00\x00"[..];
        assert_eq!(
            SkillbarFrame209026::decode_frame(&mut input).unwrap(),
            SkillbarFrame209026 {
                character_index: 0,
                operation: SkillbarOperation209026::AutoAttackMask(0),
            }
        );
        assert!(input.is_empty());
    }

    #[test]
    fn skillbar_consecutive_frames_have_exact_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(bytes, _)| bytes.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, operation) in fixtures {
            assert_eq!(
                SkillbarFrame209026::decode_frame(&mut input)
                    .unwrap()
                    .operation,
                operation
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn skillbar_bad_frames_leave_input_unchanged() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(SkillbarFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        for bytes in [
            vec![0x69, 2, 0],
            vec![0x70, 2, 0],
            vec![0x6a, 2, 0xff, 0xff, 0xff, 0xff, 0x10],
            vec![0x6f, 2, 0, 0xff, 0xff, 0xff, 0xff, 0x10],
            vec![0x6f, 2, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0],
        ] {
            let mut input = bytes.as_slice();
            assert_eq!(
                SkillbarFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
    }
}
