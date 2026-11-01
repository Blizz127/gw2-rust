//! Build-209026 servers 509–511: assign controlled-player progress.
//! Content data key +0x28 selects type 53. There is no player-index field.
//! See `docs/re/209026-player-progress.md` for channel and lifetime constraints.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// Server 509: replace the BIT representation's channel mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitProgressFrame209026 {
    pub progress_key: u32,
    pub channel_mask: u16,
}

impl BitProgressFrame209026 {
    pub fn encode_frame(&self, output: &mut BytesMut) {
        509u16.encode(output);
        self.progress_key.encode(output);
        self.channel_mask.encode(output);
    }

    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != 509 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected progress message 509",
            ));
        }
        let progress_key = u32::decode(&mut cursor)?;
        let channel_mask = u16::decode(&mut cursor)?;
        *input = cursor;
        Ok(Self {
            progress_key,
            channel_mask,
        })
    }
}

/// Server 510: raw BIT_MASK blocks; content determines each channel's width.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitMaskProgressFrame209026 {
    pub progress_key: u32,
    pub words: Vec<u32>,
}

impl BitMaskProgressFrame209026 {
    /// Append one frame, leaving output unchanged if the array is too long.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        let count = u8::try_from(self.words.len()).map_err(|_| {
            Error::new(
                ErrorKind::InvalidInput,
                "bit-mask progress exceeds 255 words",
            )
        })?;
        510u16.encode(output);
        self.progress_key.encode(output);
        count.encode(output);
        for word in &self.words {
            word.encode(output);
        }
        Ok(())
    }

    /// Parse raw words without inferring block widths or channel assignments.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != 510 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected progress message 510",
            ));
        }
        let progress_key = u32::decode(&mut cursor)?;
        let count = u8::decode(&mut cursor)?;
        let mut words = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            words.push(u32::decode(&mut cursor)?);
        }
        *input = cursor;
        Ok(Self {
            progress_key,
            words,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterProgressFrame209026 {
    pub progress_key: u32,
    /// Consecutive values for enabled content channels, not explicit channel IDs.
    /// The descriptor permits at most four; content may permit fewer.
    pub values: Vec<u32>,
}

impl CounterProgressFrame209026 {
    /// Append one frame, leaving output unchanged if the array is too long.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if self.values.len() > 4 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "progress exceeds four values",
            ));
        }
        511u16.encode(output);
        self.progress_key.encode(output);
        (self.values.len() as u8).encode(output);
        for value in &self.values {
            value.encode(output);
        }
        Ok(())
    }

    /// Consume one complete frame transactionally without content validation.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        if u16::decode(&mut cursor)? != 511 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected progress message 511",
            ));
        }
        let progress_key = u32::decode(&mut cursor)?;
        let count = u8::decode(&mut cursor)?;
        if count > 4 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "progress exceeds four values",
            ));
        }
        let mut values = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            values.push(u32::decode(&mut cursor)?);
        }
        *input = cursor;
        Ok(Self {
            progress_key,
            values,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> Vec<(Vec<u8>, CounterProgressFrame209026)> {
        [
            ("ff 01 9e 26 01 3c", 4894, vec![60]),
            ("ff 01 ff ff ff ff 0f 00", u32::MAX, vec![]),
            (
                "ff 01 00 04 00 80 01 80 80 01 ff ff ff ff 0f",
                0,
                vec![0, 128, 16_384, u32::MAX],
            ),
        ]
        .into_iter()
        .map(|(hex, progress_key, values)| {
            (
                hex.split_whitespace()
                    .map(|b| u8::from_str_radix(b, 16).unwrap())
                    .collect(),
                CounterProgressFrame209026 {
                    progress_key,
                    values,
                },
            )
        })
        .collect()
    }

    #[test]
    fn progress_golden_frames_and_stream_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(bytes, _)| bytes.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, frame) in fixtures {
            assert_eq!(
                CounterProgressFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
            let mut output = BytesMut::from(&b"prefix"[..]);
            frame.encode_frame(&mut output).unwrap();
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
        }
        assert!(input.is_empty());
    }

    fn rejects(bytes: &[u8], kind: ErrorKind) {
        let mut input = bytes;
        assert_eq!(
            CounterProgressFrame209026::decode_frame(&mut input)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(input, bytes);
    }

    #[test]
    fn progress_truncations_and_invalid_varints_preserve_input() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                rejects(&golden[..cut], ErrorKind::UnexpectedEof);
            }
        }
        for bad in [[0xff, 0xff, 0xff, 0xff, 0x10], [0x80; 5]] {
            let mut bytes = vec![0xff, 0x01];
            bytes.extend(bad);
            bytes.push(0);
            rejects(&bytes, ErrorKind::InvalidData);
            for at in 0..4 {
                let mut bytes = vec![0xff, 0x01, 0, 4];
                bytes.extend(vec![0; at]);
                bytes.extend(bad);
                bytes.extend(vec![0; 3 - at]);
                rejects(&bytes, ErrorKind::InvalidData);
            }
        }
    }

    #[test]
    fn progress_bounds_and_wrong_ids_are_transactional() {
        for count in [5, 255] {
            rejects(&[0xff, 0x01, 0, count], ErrorKind::InvalidData);
        }
        for id in [510u16, 512] {
            let mut bytes = id.to_le_bytes().to_vec();
            bytes.extend([0, 0]);
            rejects(&bytes, ErrorKind::InvalidData);
        }
        for count in [5, 256] {
            let frame = CounterProgressFrame209026 {
                progress_key: 1,
                values: vec![0; count],
            };
            let mut output = BytesMut::from(&b"prefix"[..]);
            assert_eq!(
                frame.encode_frame(&mut output).unwrap_err().kind(),
                ErrorKind::InvalidInput
            );
            assert_eq!(&output[..], b"prefix");
        }
    }
}

#[cfg(test)]
mod bit_tests {
    use super::*;

    #[test]
    fn bit_progress_fixed_word_and_stream_boundaries() {
        let golden = [0xfd, 0x01, 0x80, 0x01, 0x34, 0x80];
        let frame = BitProgressFrame209026 {
            progress_key: 128,
            channel_mask: 0x8034,
        };
        let mut output = BytesMut::from(&b"prefix"[..]);
        frame.encode_frame(&mut output);
        assert_eq!(&output[..6], b"prefix");
        assert_eq!(&output[6..], golden);
        let mut stream = golden.to_vec();
        stream.extend([0xfd, 1, 0, 0xff, 0xff]);
        let mut input = stream.as_slice();
        assert_eq!(
            BitProgressFrame209026::decode_frame(&mut input).unwrap(),
            frame
        );
        assert_eq!(input, &stream[golden.len()..]);
        assert_eq!(
            BitProgressFrame209026::decode_frame(&mut input).unwrap(),
            BitProgressFrame209026 {
                progress_key: 0,
                channel_mask: u16::MAX
            }
        );
        assert!(input.is_empty());
        for cut in 0..golden.len() {
            let original = &golden[..cut];
            let mut input = original;
            assert_eq!(
                BitProgressFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::UnexpectedEof
            );
            assert_eq!(input, original);
        }
        for bad in [
            vec![0xfe, 1, 0, 0, 0],
            vec![0xfd, 1, 0xff, 0xff, 0xff, 0xff, 0x10, 0, 0],
        ] {
            let mut input = bad.as_slice();
            assert_eq!(
                BitProgressFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bad);
        }
    }

    #[test]
    fn bit_mask_progress_golden_and_maximum_array() {
        let golden = [
            0xfe, 1, 0x80, 1, 4, 0, 0x80, 1, 0x80, 0x80, 1, 0xff, 0xff, 0xff, 0xff, 0x0f,
        ];
        let frame = BitMaskProgressFrame209026 {
            progress_key: 128,
            words: vec![0, 128, 16384, u32::MAX],
        };
        let mut output = BytesMut::from(&b"prefix"[..]);
        frame.encode_frame(&mut output).unwrap();
        assert_eq!(&output[..6], b"prefix");
        assert_eq!(&output[6..], golden);
        let mut stream = golden.to_vec();
        stream.extend([0xfe, 1, 0, 0]);
        let mut input = stream.as_slice();
        assert_eq!(
            BitMaskProgressFrame209026::decode_frame(&mut input).unwrap(),
            frame
        );
        assert_eq!(input, &stream[golden.len()..]);
        assert_eq!(
            BitMaskProgressFrame209026::decode_frame(&mut input)
                .unwrap()
                .words,
            Vec::<u32>::new()
        );
        assert!(input.is_empty());
        for cut in 0..golden.len() {
            let original = &golden[..cut];
            let mut input = original;
            assert_eq!(
                BitMaskProgressFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::UnexpectedEof
            );
            assert_eq!(input, original);
        }
        // All 255 distinct one/two-byte values, with an independent encoding.
        let frame = BitMaskProgressFrame209026 {
            progress_key: 0,
            words: (0..255).collect(),
        };
        let mut golden = vec![0xfe, 1, 0, 255];
        golden.extend(0..128);
        for low in 0..127 {
            golden.extend([low | 0x80, 1]);
        }
        let mut output = BytesMut::new();
        frame.encode_frame(&mut output).unwrap();
        assert_eq!(&output[..], golden);
        let mut input = golden.as_slice();
        assert_eq!(
            BitMaskProgressFrame209026::decode_frame(&mut input).unwrap(),
            frame
        );
        assert!(input.is_empty());
    }

    #[test]
    fn bit_mask_progress_invalid_frames_are_transactional() {
        let frame = BitMaskProgressFrame209026 {
            progress_key: 0,
            words: vec![0; 256],
        };
        let mut output = BytesMut::from(&b"prefix"[..]);
        assert_eq!(
            frame.encode_frame(&mut output).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(&output[..], b"prefix");
        let mut invalid = vec![vec![0xfd, 1, 0, 0], vec![0xff, 1, 0, 0]];
        for bad in [[0xff, 0xff, 0xff, 0xff, 0x10], [0x80; 5]] {
            let mut key = vec![0xfe, 1];
            key.extend(bad);
            key.push(0);
            invalid.push(key);
            for at in [0, 127, 254] {
                let mut words = vec![0xfe, 1, 0, 255];
                words.extend(vec![0; at]);
                words.extend(bad);
                words.extend(vec![0; 254 - at]);
                invalid.push(words);
            }
        }
        for bad in invalid {
            let mut input = bad.as_slice();
            assert_eq!(
                BitMaskProgressFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bad);
        }
    }
}
