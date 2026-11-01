//! Build-209026 server 511: assign controlled-player counter progress.
//! Content data key +0x28 selects type 53. There is no player-index field.
//! See `docs/re/209026-player-progress.md` for channel and lifetime constraints.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

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
