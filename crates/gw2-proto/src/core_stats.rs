//! Build-209026 core-stat creation frames, IDs 182 and 183.
//! Evidence and semantic limits: `docs/re/209026-character-core-stats.md`.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// One server-182 attribute record, with fields in wire order.
/// Gameplay column names remain unresolved; the third Word is signed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeRecord209026 {
    pub value_0: u16,
    pub value_1: u16,
    pub value_2: i16,
}

/// Additional server-182 fields. No inferred defaults or float validation.
#[derive(Debug, Clone)]
pub struct ExtendedCoreStats209026 {
    pub value: u32,
    /// All twelve wire records, including record 0 skipped by the handler.
    pub attributes: [AttributeRecord209026; 12],
    /// Three trailing floats, in wire order; gameplay names remain unresolved.
    pub values: [f32; 3],
}

/// Core-stat creation, preserving framing without claiming valid game content.
/// These are alternative creation messages, not consecutive initialization steps.
#[derive(Debug, Clone)]
pub struct CoreStatsCreation209026 {
    pub character_index: u32,
    pub appearance: [u8; 45],
    pub earned_level: u8,
    /// Second scalar byte, stored at component +0xe0; name not yet established.
    pub level_2: u8,
    pub profession: u8,
    /// `Some` selects message 182; `None` selects message 183.
    pub extended: Option<ExtendedCoreStats209026>,
}

impl CoreStatsCreation209026 {
    pub const EXTENDED_MESSAGE_ID: u16 = 182;
    pub const COMMON_MESSAGE_ID: u16 = 183;

    /// Append one complete frame. All fields have fixed or bounded encodings.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        let id = if self.extended.is_some() {
            Self::EXTENDED_MESSAGE_ID
        } else {
            Self::COMMON_MESSAGE_ID
        };
        id.encode(output);
        self.character_index.encode(output);
        self.appearance.encode(output);
        self.earned_level.encode(output);
        self.level_2.encode(output);
        self.profession.encode(output);
        if let Some(extra) = &self.extended {
            extra.value.encode(output);
            for row in &extra.attributes {
                row.value_0.encode(output);
                row.value_1.encode(output);
                (row.value_2 as u16).encode(output);
            }
            extra.values.encode(output);
        }
    }

    /// Consume one frame, leaving the input slice unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if ![Self::EXTENDED_MESSAGE_ID, Self::COMMON_MESSAGE_ID].contains(&id) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected core-stat creation message 182 or 183",
            ));
        }
        let character_index = u32::decode(&mut cursor)?;
        let appearance = <[u8; 45]>::decode(&mut cursor)?;
        let earned_level = u8::decode(&mut cursor)?;
        let level_2 = u8::decode(&mut cursor)?;
        let profession = u8::decode(&mut cursor)?;
        let extended = if id == Self::EXTENDED_MESSAGE_ID {
            let value = u32::decode(&mut cursor)?;
            let attributes =
                <[[u16; 3]; 12]>::decode(&mut cursor)?.map(|row| AttributeRecord209026 {
                    value_0: row[0],
                    value_1: row[1],
                    value_2: row[2] as i16,
                });
            Some(ExtendedCoreStats209026 {
                value,
                attributes,
                values: <[f32; 3]>::decode(&mut cursor)?,
            })
        } else {
            None
        };
        let frame = Self {
            character_index,
            appearance,
            earned_level,
            level_2,
            profession,
            extended,
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn common_fixture() -> Vec<u8> {
        let mut wire = vec![0xb7, 0, 0xac, 2]; // 183, character index 300.
        wire.extend(0u8..45);
        wire.extend([80, 17, 9]);
        wire
    }

    fn extended_fixture() -> Vec<u8> {
        let mut wire = common_fixture();
        wire[0] = 0xb6;
        wire.extend([0xff, 0xff, 0xff, 0xff, 0x0f]);
        // Distinct unsigned/signed fields, including row zero and both limits.
        wire.extend([0x34, 0x12, 0xff, 0xff, 0, 0x80]);
        wire.extend([0xff, 0xff, 0, 0x80, 0xff, 0xff]);
        for _ in 2..11 {
            wire.extend([1, 0, 2, 0, 3, 0]);
        }
        wire.extend([0x78, 0x56, 0xbc, 0x9a, 0xff, 0x7f]);
        // -0, NaN payload, +infinity: preserved, not claimed valid for a client.
        wire.extend([0, 0, 0, 0x80, 0x45, 0x23, 0xc1, 0x7f, 0, 0, 0x80, 0x7f]);
        wire
    }

    #[test]
    fn core_stats_common_golden_and_following_frame() {
        let golden = common_fixture();
        assert_eq!(golden.len(), 52);
        let next = extended_fixture();
        let mut wire = golden.clone();
        wire.extend_from_slice(&next);
        let mut input = wire.as_slice();
        let frame = CoreStatsCreation209026::decode_frame(&mut input).unwrap();
        assert_eq!(frame.character_index, 300);
        assert_eq!(frame.appearance, std::array::from_fn(|i| i as u8));
        assert_eq!(
            (frame.earned_level, frame.level_2, frame.profession),
            (80, 17, 9)
        );
        assert!(frame.extended.is_none());
        assert_eq!(input, next);
        let mut encoded = BytesMut::from(&b"prefix"[..]);
        frame.encode_frame(&mut encoded);
        assert_eq!(&encoded[6..], golden);
        CoreStatsCreation209026::decode_frame(&mut input).unwrap();
        assert!(input.is_empty());
    }

    #[test]
    fn core_stats_extended_golden_preserves_signed_column_and_float_bits() {
        let golden = extended_fixture();
        assert_eq!(golden.len(), 141);
        let next = common_fixture();
        let mut wire = golden.clone();
        wire.extend_from_slice(&next);
        let mut input = wire.as_slice();
        let frame = CoreStatsCreation209026::decode_frame(&mut input).unwrap();
        let extra = frame.extended.as_ref().unwrap();
        assert_eq!(extra.value, u32::MAX);
        for (index, expected) in [
            (0, (0x1234, 0xffff, i16::MIN)),
            (1, (0xffff, 0x8000, -1)),
            (5, (1, 2, 3)),
            (11, (0x5678, 0x9abc, i16::MAX)),
        ] {
            let row = extra.attributes[index];
            assert_eq!((row.value_0, row.value_1, row.value_2), expected);
        }
        assert_eq!(
            extra.values.map(f32::to_bits),
            [0x80000000, 0x7fc12345, 0x7f800000]
        );
        assert_eq!(input, next);
        let mut encoded = BytesMut::new();
        frame.encode_frame(&mut encoded);
        assert_eq!(encoded, golden);
    }

    #[test]
    fn core_stats_bad_frames_leave_input_unchanged() {
        for golden in [common_fixture(), extended_fixture()] {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(CoreStatsCreation209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        let mut wrong_id = common_fixture();
        wrong_id[0] = 0xb9; // 185 appearance update is not creation.
        let overflowing_index = vec![0xb7, 0, 0xff, 0xff, 0xff, 0xff, 0x10];
        let mut overflowing_value = extended_fixture();
        overflowing_value[56] = 0x10;
        for malformed in [wrong_id, overflowing_index, overflowing_value] {
            let mut input = malformed.as_slice();
            assert_eq!(
                CoreStatsCreation209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, malformed);
        }
    }
}
