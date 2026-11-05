//! Build-209026 Mesmer profession frames, IDs 481–483.
//! Creation layout and profession identity: `docs/re/209026-character-profession.md`.
//! All three field layouts: `data/protocol/209026/server-tables.xml`.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

/// Wire values only; no profession selection, state mutation or default values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MesmerOperation209026 {
    /// 481 installs a new Mesmer profession component.
    Create {
        /// Gameplay names unresolved; both bytes are retained without masking.
        values: [u8; 2],
    },
    /// 482 removes an existing profession component.
    Remove,
    /// 483 updates the two unnamed values on an existing Mesmer component.
    Values([u8; 2]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MesmerFrame209026 {
    pub character_index: u32,
    pub operation: MesmerOperation209026,
}

impl MesmerFrame209026 {
    /// Append one complete frame, preserving both raw bytes.
    pub fn encode_frame(&self, output: &mut BytesMut) {
        use MesmerOperation209026::*;
        let id: u16 = match self.operation {
            Create { .. } => 481,
            Remove => 482,
            Values(_) => 483,
        };
        id.encode(output);
        self.character_index.encode(output);
        match self.operation {
            Create { values } | Values(values) => values.encode(output),
            Remove => {}
        }
    }

    /// Consume one frame; leave input unchanged on every error.
    /// Component lifetime and matching profession are the caller's responsibility.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        use MesmerOperation209026::*;
        let mut cursor = *input;
        let id = u16::decode(&mut cursor)?;
        if !(481..=483).contains(&id) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "expected Mesmer message 481 through 483",
            ));
        }
        let character_index = u32::decode(&mut cursor)?;
        let operation = match id {
            481 => Create {
                values: <[u8; 2]>::decode(&mut cursor)?,
            },
            482 => Remove,
            483 => Values(<[u8; 2]>::decode(&mut cursor)?),
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

    fn fixtures() -> [(Vec<u8>, MesmerFrame209026); 3] {
        use MesmerOperation209026::*;
        [
            // Index 300 and unmasked byte values 128/255.
            (
                vec![0xe1, 0x01, 0xac, 0x02, 0x80, 0xff],
                MesmerFrame209026 {
                    character_index: 300,
                    operation: Create { values: [128, 255] },
                },
            ),
            (
                vec![0xe2, 0x01, 0x00],
                MesmerFrame209026 {
                    character_index: 0,
                    operation: Remove,
                },
            ),
            // The full Dword domain is preserved; no gameplay index bound here.
            (
                vec![0xe3, 0x01, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x00, 0xff],
                MesmerFrame209026 {
                    character_index: u32::MAX,
                    operation: Values([0, 255]),
                },
            ),
        ]
    }

    #[test]
    fn mesmer_golden_fields_and_raw_bytes() {
        for (golden, expected) in fixtures() {
            let mut input = golden.as_slice();
            assert_eq!(
                MesmerFrame209026::decode_frame(&mut input).unwrap(),
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
    fn mesmer_frames_preserve_next_frame_boundary() {
        let fixtures = fixtures();
        let stream: Vec<u8> = fixtures
            .iter()
            .flat_map(|(bytes, _)| bytes.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (golden, expected) in fixtures {
            assert_eq!(
                MesmerFrame209026::decode_frame(&mut input).unwrap(),
                expected
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn mesmer_malformed_frames_leave_input_unchanged() {
        for (golden, _) in fixtures() {
            for cut in 0..golden.len() {
                let mut input = &golden[..cut];
                assert!(MesmerFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, &golden[..cut]);
            }
        }
        for bytes in [
            vec![0xe0, 1, 0],
            vec![0xe4, 1, 0],
            vec![0xe2, 1, 0xff, 0xff, 0xff, 0xff, 0x10],
            vec![0xe2, 1, 0x80, 0x80, 0x80, 0x80, 0x80, 0],
        ] {
            let mut input = bytes.as_slice();
            assert_eq!(
                MesmerFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, bytes);
        }
    }
}
