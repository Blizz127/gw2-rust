//! Build-209026 Guardian profession frames, IDs 477–480.
//! See `docs/re/209026-character-profession.md` for lifecycle and field evidence.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

#[derive(Debug, Clone, Copy)]
pub struct GuardianMeter209026 {
    pub maximum: f32,
    pub rate: f32,
    pub current: f32,
}

/// Wire values only; no profession selection, state mutation or default values.
#[derive(Debug, Clone, Copy)]
pub enum GuardianOperation209026 {
    /// 477 installs a new Guardian profession component.
    Create {
        meter: GuardianMeter209026,
        /// Gameplay names unresolved; both bytes are retained without masking.
        values: [u8; 2],
    },
    /// 478 removes an existing profession component.
    Remove,
    /// 479 updates an existing Guardian meter.
    Meter {
        meter: GuardianMeter209026,
        /// Present on the wire, although the inspected handler ignores them.
        trailing: [f32; 2],
    },
    /// 480 updates the two unnamed values on an existing Guardian component.
    Values([u8; 2]),
}

#[derive(Debug, Clone, Copy)]
pub struct GuardianFrame209026 {
    pub character_index: u32,
    pub operation: GuardianOperation209026,
}

impl GuardianFrame209026 {
    /// Append one complete frame, preserving raw floats and bytes.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use GuardianOperation209026::*;
        let id: u16 = match self.operation {
            Create { .. } => 477,
            Remove => 478,
            Meter { .. } => 479,
            Values(_) => 480,
        };
        id.encode(output);
        self.character_index.encode(output);
        match self.operation {
            Create { meter, values } => {
                [meter.maximum, meter.rate, meter.current].encode(output);
                values.encode(output);
            }
            Remove => {}
            Meter { meter, trailing } => {
                [meter.maximum, meter.rate, meter.current].encode(output);
                trailing.encode(output);
            }
            Values(values) => values.encode(output),
        }
    }

    /// Consume one frame; leave input unchanged on every error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use GuardianOperation209026::*;
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !(477..=480).contains(&id) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected Guardian message 477 through 480",
            ));
        }
        let character_index = u32::decode(&mut cursor)?;
        let operation = match id {
            477 | 479 => {
                let [maximum, rate, current] = <[f32; 3]>::decode(&mut cursor)?;
                let meter = GuardianMeter209026 {
                    maximum,
                    rate,
                    current,
                };
                if id == 477 {
                    Create {
                        meter,
                        values: <[u8; 2]>::decode(&mut cursor)?,
                    }
                } else {
                    Meter {
                        meter,
                        trailing: <[f32; 2]>::decode(&mut cursor)?,
                    }
                }
            }
            478 => Remove,
            480 => Values(<[u8; 2]>::decode(&mut cursor)?),
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

    fn fixtures() -> [Vec<u8>; 4] {
        [
            // Index 300, maximum 10, rate -0.5, current 3.25, raw bytes 128/255.
            "dd 01 ac 02 00 00 20 41 00 00 00 bf 00 00 50 40 80 ff",
            "de 01 ac 02",
            // Five floats; trailing negative zero and a NaN with payload bits.
            "df 01 ac 02 00 00 20 41 00 00 00 bf 00 00 50 40 00 00 00 80 45 23 c1 7f",
            "e0 01 ac 02 00 ff",
        ]
        .map(|text| {
            text.split_whitespace()
                .map(|b| u8::from_str_radix(b, 16).unwrap())
                .collect()
        })
    }

    fn assert_meter(meter: GuardianMeter209026) {
        assert_eq!(
            [meter.maximum, meter.rate, meter.current].map(f32::to_bits),
            [0x4120_0000, 0xbf00_0000, 0x4050_0000]
        );
    }

    #[test]
    fn guardian_golden_fields_and_trailing_float_bits() {
        for (variant, golden) in fixtures().iter().enumerate() {
            let mut input = golden.as_slice();
            let frame = GuardianFrame209026::decode_frame(&mut input).unwrap();
            assert_eq!(frame.character_index, 300);
            assert!(input.is_empty());
            match frame.operation {
                GuardianOperation209026::Create { meter, values } => {
                    assert_eq!(variant, 0);
                    assert_meter(meter);
                    assert_eq!(values, [128, 255]);
                }
                GuardianOperation209026::Remove => assert_eq!(variant, 1),
                GuardianOperation209026::Meter { meter, trailing } => {
                    assert_eq!(variant, 2);
                    assert_meter(meter);
                    assert_eq!(trailing.map(f32::to_bits), [0x8000_0000, 0x7fc1_2345]);
                }
                GuardianOperation209026::Values(values) => {
                    assert_eq!(variant, 3);
                    assert_eq!(values, [0, 255]);
                }
            }
            let mut output = BytesMut::from(&b"prefix"[..]);
            frame.encode_frame(&mut output);
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
        }
    }

    #[test]
    fn guardian_five_float_update_preserves_next_frame_boundary() {
        let fixtures = fixtures();
        let stream = fixtures.concat();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for golden in fixtures {
            let frame = GuardianFrame209026::decode_frame(&mut input).unwrap();
            let mut output = BytesMut::new();
            frame.encode_frame(&mut output);
            assert_eq!(output, golden.as_slice());
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn guardian_malformed_frames_leave_input_unchanged() {
        for golden in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(GuardianFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        for bytes in [
            vec![0xdc, 1, 0],
            vec![0xe1, 1, 0],
            vec![0xde, 1, 0xff, 0xff, 0xff, 0xff, 0x10],
            vec![0xde, 1, 0x80, 0x80, 0x80, 0x80, 0x80, 0],
        ] {
            let mut input = bytes.as_slice();
            assert_eq!(
                GuardianFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
    }
}
