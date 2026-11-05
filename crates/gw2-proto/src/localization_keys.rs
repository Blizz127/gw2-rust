//! Build-209026 global localized-text keys, server messages 747 and 748.
//! See `docs/re/209026-quest154-content.md` for the decoder and key provenance.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode};

pub const MAX_LOCALIZATION_KEYS209026: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalizationKey209026 {
    pub text_id: u32,
    /// The supplied 64-bit key, preserved without deriving or validating it.
    pub key: u64,
}

/// These messages affect the global text dictionary and have no player index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalizationKeysFrame209026 {
    /// 747 clears the client's text-key dictionary.
    Clear,
    /// 748 inserts or updates the supplied entries in wire order.
    /// An empty list is valid; it does not clear the dictionary.
    Set { entries: Vec<LocalizationKey209026> },
}

impl LocalizationKeysFrame209026 {
    /// Append one frame; an oversized list leaves the output unchanged.
    /// Zero IDs/keys and repeated IDs are preserved as explicit wire values.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        match self {
            Self::Clear => 747u16.encode(output),
            Self::Set { entries } => {
                if entries.len() > MAX_LOCALIZATION_KEYS209026 {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        "localization-key message exceeds 64 entries",
                    ));
                }
                748u16.encode(output);
                (entries.len() as u8).encode(output);
                for entry in entries {
                    entry.text_id.encode(output);
                    entry.key.encode(output);
                }
            }
        }
        Ok(())
    }

    /// Consume one complete frame, leaving input unchanged on every error.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        let frame = match u16::decode(&mut cursor)? {
            747 => Self::Clear,
            748 => {
                let count = u8::decode(&mut cursor)? as usize;
                if count > MAX_LOCALIZATION_KEYS209026 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "localization-key message exceeds 64 entries",
                    ));
                }
                let mut entries = Vec::with_capacity(count);
                for _ in 0..count {
                    entries.push(LocalizationKey209026 {
                        text_id: u32::decode(&mut cursor)?,
                        key: u64::decode(&mut cursor)?,
                    });
                }
                Self::Set { entries }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "expected localization-key message 747 or 748",
                ));
            }
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, CountPrefix, Rules, Value};
    use crate::schema;

    fn fixtures() -> [(LocalizationKeysFrame209026, &'static [u8]); 3] {
        // Synthetic wire probes only; none supplies a real content key.
        [
            (LocalizationKeysFrame209026::Clear, &[0xeb, 0x02]),
            (
                LocalizationKeysFrame209026::Set { entries: vec![] },
                &[0xec, 0x02, 0],
            ),
            (
                LocalizationKeysFrame209026::Set {
                    entries: vec![
                        LocalizationKey209026 { text_id: 0, key: 0 },
                        LocalizationKey209026 {
                            text_id: 300,
                            key: 0x0123_4567_89ab_cdef,
                        },
                        LocalizationKey209026 {
                            text_id: u32::MAX,
                            key: u64::MAX,
                        },
                    ],
                },
                &[
                    0xec, 0x02, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xac, 0x02, 0xef, 0xcd, 0xab, 0x89,
                    0x67, 0x45, 0x23, 0x01, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff,
                ],
            ),
        ]
    }

    #[test]
    fn localization_keys_golden_values_and_stream_boundaries() {
        let fixtures = fixtures();
        let stream: Vec<_> = fixtures
            .iter()
            .flat_map(|(_, bytes)| bytes.iter().copied())
            .collect();
        let mut input = stream.as_slice();
        let mut consumed = 0;
        for (frame, golden) in fixtures {
            let mut output = BytesMut::from(&b"prefix"[..]);
            frame.encode_frame(&mut output).unwrap();
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], golden);
            assert_eq!(
                LocalizationKeysFrame209026::decode_frame(&mut input).unwrap(),
                frame
            );
            consumed += golden.len();
            assert_eq!(input, &stream[consumed..]);
        }
        assert!(input.is_empty());
    }

    #[test]
    fn localization_keys_match_extracted_client_schema() {
        let tables = schema::parse(include_str!(
            "../../../data/protocol/209026/server-tables.xml"
        ))
        .unwrap();
        let messages: Vec<_> = tables
            .protocols
            .into_iter()
            .filter(|p| p.name == "0")
            .flat_map(|p| p.groups)
            .filter(|g| g.name == "table-0x14213c4f8")
            .flat_map(|g| g.server)
            .collect();
        assert_eq!(messages.len(), 2);
        let rules = Rules {
            array_var_small: Some(CountPrefix::U8),
            ..Rules::default()
        };
        for (frame, golden) in fixtures() {
            let mut input = golden;
            let (message, value) = dynamic::decode_frame(&messages, &rules, &mut input).unwrap();
            assert!(input.is_empty());
            let (id, fields) = match frame {
                LocalizationKeysFrame209026::Clear => (747, vec![]),
                LocalizationKeysFrame209026::Set { entries } => (
                    748,
                    vec![(
                        "0".into(),
                        Value::Array(
                            entries
                                .iter()
                                .map(|entry| {
                                    Value::Struct(vec![
                                        ("0".into(), Value::U32(entry.text_id)),
                                        ("1".into(), Value::U64(entry.key)),
                                    ])
                                })
                                .collect(),
                        ),
                    )],
                ),
            };
            assert_eq!(messages.iter().filter(|m| m.id == id).count(), 1);
            assert_eq!(message.id, id);
            assert_eq!(value, Value::Struct(fields));
        }
        let set = messages.iter().find(|m| m.id == 748).unwrap();
        assert_eq!(set.fields.len(), 1);
        assert_eq!(set.fields[0].kind, "ArrayVarSmall");
        assert_eq!(set.fields[0].size, Some(MAX_LOCALIZATION_KEYS209026 as u32));
    }

    #[test]
    fn localization_keys_capacity_and_repeated_ids_preserve_wire_order() {
        let entries: Vec<_> = (0..64)
            .map(|key| LocalizationKey209026 { text_id: 300, key })
            .collect();
        let frame = LocalizationKeysFrame209026::Set { entries };
        let mut golden = vec![0xec, 0x02, 64];
        for key in 0..64u8 {
            golden.extend_from_slice(&[0xac, 0x02, key, 0, 0, 0, 0, 0, 0, 0]);
        }
        let mut encoded = BytesMut::new();
        frame.encode_frame(&mut encoded).unwrap();
        assert_eq!(encoded.as_ref(), golden);
        let mut input = golden.as_slice();
        assert_eq!(
            LocalizationKeysFrame209026::decode_frame(&mut input).unwrap(),
            frame
        );
        assert!(input.is_empty());

        for count in [65, 255, 256] {
            let oversized = LocalizationKeysFrame209026::Set {
                entries: vec![LocalizationKey209026 { text_id: 0, key: 0 }; count],
            };
            let mut output = BytesMut::from(&b"prefix"[..]);
            assert_eq!(
                oversized.encode_frame(&mut output).unwrap_err().kind(),
                ErrorKind::InvalidInput
            );
            assert_eq!(output.as_ref(), b"prefix");
        }
    }

    #[test]
    fn localization_keys_invalid_or_incomplete_frames_preserve_input() {
        for (_, golden) in fixtures() {
            for cut in 0..golden.len() {
                let original = &golden[..cut];
                let mut input = original;
                assert_eq!(
                    LocalizationKeysFrame209026::decode_frame(&mut input)
                        .unwrap_err()
                        .kind(),
                    ErrorKind::UnexpectedEof
                );
                assert_eq!(input, original);
            }
        }
        let mut invalid = vec![
            vec![0xea, 0x02],
            vec![0xed, 0x02],
            vec![0xec, 0x02, 65],
            vec![0xec, 0x02, 255],
        ];
        // Test both first and later Dwords, including the second entry after
        // a completely consumed first entry. Following data must not hide errors.
        for valid_before in [0, 1] {
            for malformed in [[0xff, 0xff, 0xff, 0xff, 0x10], [0x80; 5]] {
                let mut bytes = vec![0xec, 0x02, 2];
                bytes.extend(std::iter::repeat_n(0, valid_before * 9));
                bytes.extend_from_slice(&malformed);
                bytes.extend_from_slice(&[0; 18]);
                invalid.push(bytes);
            }
        }
        for original in invalid {
            let mut input = original.as_slice();
            assert_eq!(
                LocalizationKeysFrame209026::decode_frame(&mut input)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
            assert_eq!(input, original);
        }
    }
}
