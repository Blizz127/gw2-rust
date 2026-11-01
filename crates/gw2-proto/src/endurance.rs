//! Build-209026 endurance frames, IDs 217–221.
//! See `docs/re/209026-character-health-endurance.md` for client semantics.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// Explicit wire operations; these do not validate component lifetime or values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnduranceOperation209026 {
    /// 217 requires endurance to be absent.
    Create {
        points: u32,
        /// Denominator for the first fraction getter and percentage adjustment.
        maximum: u32,
        effective_maximum: u32,
        /// Divides effective maximum into segments; gameplay name unresolved.
        segment_divisor: u32,
    },
    /// 218 removes an existing component.
    Remove,
    /// 219 sets points on existing endurance.
    Points(u32),
    /// 220 sets both maxima on existing endurance.
    Maxima {
        maximum: u32,
        effective_maximum: u32,
    },
    /// 221 sets the segment divisor on existing endurance.
    SegmentDivisor(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnduranceFrame209026 {
    pub character_index: u32,
    pub operation: EnduranceOperation209026,
}

impl EnduranceFrame209026 {
    /// Append a complete frame, preserving all unsigned wire values.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use EnduranceOperation209026::*;
        let id: u16 = match self.operation {
            Create { .. } => 217,
            Remove => 218,
            Points(_) => 219,
            Maxima { .. } => 220,
            SegmentDivisor(_) => 221,
        };
        id.encode(output);
        self.character_index.encode(output);
        match self.operation {
            Create {
                points,
                maximum,
                effective_maximum,
                segment_divisor,
            } => {
                [points, maximum, effective_maximum, segment_divisor].encode(output);
            }
            Remove => {}
            Points(value) | SegmentDivisor(value) => value.encode(output),
            Maxima {
                maximum,
                effective_maximum,
            } => {
                [maximum, effective_maximum].encode(output);
            }
        }
    }

    /// Consume exactly one frame; leave input unchanged on any error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use EnduranceOperation209026::*;
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !(217..=221).contains(&id) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected endurance message 217 through 221",
            ));
        }
        let character_index = u32::decode(&mut cursor)?;
        let operation = match id {
            217 => {
                let [points, maximum, effective_maximum, segment_divisor] =
                    <[u32; 4]>::decode(&mut cursor)?;
                Create {
                    points,
                    maximum,
                    effective_maximum,
                    segment_divisor,
                }
            }
            218 => Remove,
            219 => Points(u32::decode(&mut cursor)?),
            220 => {
                let [maximum, effective_maximum] = <[u32; 2]>::decode(&mut cursor)?;
                Maxima {
                    maximum,
                    effective_maximum,
                }
            }
            221 => SegmentDivisor(u32::decode(&mut cursor)?),
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

    fn fixtures() -> [(Vec<u8>, EnduranceOperation209026); 5] {
        use EnduranceOperation209026::*;
        [
            (
                "d9 00 ac 02 7f 80 01 80 80 01 ff ff ff ff 0f",
                Create {
                    points: 127,
                    maximum: 128,
                    effective_maximum: 16384,
                    segment_divisor: u32::MAX,
                },
            ),
            ("da 00 ac 02", Remove),
            ("db 00 ac 02 00", Points(0)),
            (
                "dc 00 ac 02 ff ff ff ff 0f 00",
                Maxima {
                    maximum: u32::MAX,
                    effective_maximum: 0,
                },
            ),
            ("dd 00 ac 02 80 80 80 80 08", SegmentDivisor(0x8000_0000)),
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
    fn endurance_golden_operations_and_unsigned_boundaries() {
        for (golden, operation) in fixtures() {
            let expected = EnduranceFrame209026 {
                character_index: 300,
                operation,
            };
            let mut input = golden.as_slice();
            assert_eq!(
                EnduranceFrame209026::decode_frame(&mut input).unwrap(),
                expected
            );
            assert!(input.is_empty());
            let mut output = BytesMut::from(&b"prefix"[..]);
            expected.encode_frame(&mut output);
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
        }
    }

    #[test]
    fn endurance_consecutive_frames_have_exact_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(bytes, _)| bytes.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, operation) in fixtures {
            assert_eq!(
                EnduranceFrame209026::decode_frame(&mut input)
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
    fn endurance_rejects_truncation_and_overflow_transactionally() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(EnduranceFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        // Every creation Dword position: both overflow bits and overlong varints.
        for position in 0..5 {
            for invalid in [[0xff, 0xff, 0xff, 0xff, 0x10], [0x80; 5]] {
                let mut bytes = vec![217, 0];
                bytes.extend(std::iter::repeat_n(0, position));
                bytes.extend(invalid);
                bytes.extend([0; 5]);
                let mut input = bytes.as_slice();
                assert_eq!(
                    EnduranceFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::InvalidData
                );
                assert_eq!(input, bytes);
            }
        }
        for bytes in [[216, 0, 0], [222, 0, 0]] {
            let mut input = bytes.as_slice();
            assert_eq!(
                EnduranceFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
    }
}
