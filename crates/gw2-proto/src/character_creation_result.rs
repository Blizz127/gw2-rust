//! Build-209026 creation-scene server responses 1048–1051.
//! See `docs/re/209026-character-creation.md` for handlers and source evidence.
//! These codecs establish framing, not creation acceptance or persistence.

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::{Decode, Encode, Guid};

/// String(20) includes the terminating zero; stored names omit that terminator.
pub const MAX_NAME_UNITS209026: usize = 19;
/// Wire descriptor limit. This is not the receiving character-record capacity.
pub const MAX_RESULT_BLOB_BYTES209026: usize = 1024;
/// The traced 1048 consumer's separate character-record blob limit.
/// Sending code must enforce that runtime guard independently of this codec.
pub const RESULT_CONSUMER_BLOB_LIMIT209026: usize = 512;

/// Deliberately without Debug, as with the creation request, to avoid implicitly
/// logging character names and record data. Unknown fields remain raw.
#[derive(Clone, PartialEq, Eq)]
pub struct CreationResult209026 {
    /// First GUID, copied into the client's name-keyed character record.
    pub character_guid: Guid,
    /// The inspected record-update call does not consume this second GUID.
    pub raw_guid: Guid,
    /// Raw UTF-16 units without NUL; text validity is not a framing constraint.
    pub name_units: Vec<u16>,
    pub raw_word: u16,
    pub raw_qwords: [u64; 2],
    /// Opaque record bytes. No summary/content validation is performed here.
    pub blob: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum CreationResponseFrame209026 {
    Result1048(CreationResult209026),
    /// Empty creation-scene readiness message; ordering belongs to the caller.
    Startup1049,
    /// Empty creation-scene startup message; ordering belongs to the caller.
    Startup1050,
    /// Feeds the creation error UI, not a success acknowledgement. Preserve the
    /// full Dwords even though the handler consumes their low Words.
    ErrorUi1051 {
        raw_dwords: [u32; 4],
    },
}

impl CreationResponseFrame209026 {
    pub fn message_id(&self) -> u16 {
        match self {
            Self::Result1048(_) => 1048,
            Self::Startup1049 => 1049,
            Self::Startup1050 => 1050,
            Self::ErrorUi1051 { .. } => 1051,
        }
    }

    /// Append one complete frame. Every validation failure leaves output intact.
    /// The schema's 1024-byte blob bound is separate from the 512-byte consumer.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if let Self::Result1048(result) = self {
            if result.name_units.len() > MAX_NAME_UNITS209026 || result.name_units.contains(&0) {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "invalid creation result name framing",
                ));
            }
            if result.blob.len() > MAX_RESULT_BLOB_BYTES209026 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "creation result blob exceeds 1024 bytes",
                ));
            }
        }
        self.message_id().encode(output);
        match self {
            Self::Result1048(result) => {
                result.character_guid.encode(output);
                result.raw_guid.encode(output);
                for unit in &result.name_units {
                    unit.encode(output);
                }
                0u16.encode(output);
                result.raw_word.encode(output);
                result.raw_qwords.encode(output);
                (result.blob.len() as u16).encode(output);
                output.extend_from_slice(&result.blob);
            }
            Self::Startup1049 | Self::Startup1050 => {}
            Self::ErrorUi1051 { raw_dwords } => raw_dwords.encode(output),
        }
        Ok(())
    }

    /// Consume one frame and preserve following frames. On any error, leave the
    /// input unchanged. Allocation is bounded by the descriptor's field limits.
    pub fn decode_frame(input: &mut &[u8]) -> Result<Self> {
        let mut cursor = *input;
        let frame = match u16::decode(&mut cursor)? {
            1048 => {
                let character_guid = Guid::decode(&mut cursor)?;
                let raw_guid = Guid::decode(&mut cursor)?;
                let mut name_units = Vec::with_capacity(MAX_NAME_UNITS209026);
                loop {
                    let unit = u16::decode(&mut cursor)?;
                    if unit == 0 {
                        break;
                    }
                    if name_units.len() == MAX_NAME_UNITS209026 {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "creation result name exceeds String(20)",
                        ));
                    }
                    name_units.push(unit);
                }
                let raw_word = u16::decode(&mut cursor)?;
                let raw_qwords = <[u64; 2]>::decode(&mut cursor)?;
                let length = usize::from(u16::decode(&mut cursor)?);
                if length > MAX_RESULT_BLOB_BYTES209026 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "creation result blob exceeds 1024 bytes",
                    ));
                }
                if cursor.len() < length {
                    return Err(ErrorKind::UnexpectedEof.into());
                }
                let blob = cursor[..length].to_vec();
                cursor = &cursor[length..];
                Self::Result1048(CreationResult209026 {
                    character_guid,
                    raw_guid,
                    name_units,
                    raw_word,
                    raw_qwords,
                    blob,
                })
            }
            1049 => Self::Startup1049,
            1050 => Self::Startup1050,
            1051 => Self::ErrorUi1051 {
                raw_dwords: <[u32; 4]>::decode(&mut cursor)?,
            },
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "expected creation response 1048–1051",
                ))
            }
        };
        *input = cursor;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, CountPrefix, Rules, StringEncoding, Value};
    use crate::schema;

    fn result() -> CreationResult209026 {
        CreationResult209026 {
            character_guid: Guid([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
            raw_guid: Guid([
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ]),
            name_units: vec![0x41, 0x3a9, 0xd83d, 0xde00],
            raw_word: 0x8134,
            raw_qwords: [0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210],
            blob: vec![0, 0xfe, 0xff],
        }
    }

    fn fixtures() -> Vec<(CreationResponseFrame209026, Vec<u8>)> {
        vec![
            (
                CreationResponseFrame209026::Result1048(result()),
                vec![
                    0x18, 4, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0xf0, 0xf1,
                    0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe,
                    0xff, 0x41, 0, 0xa9, 3, 0x3d, 0xd8, 0, 0xde, 0, 0, 0x34, 0x81, 0xef, 0xcd,
                    0xab, 0x89, 0x67, 0x45, 0x23, 1, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc,
                    0xfe, 3, 0, 0, 0xfe, 0xff,
                ],
            ),
            (CreationResponseFrame209026::Startup1049, vec![0x19, 4]),
            (CreationResponseFrame209026::Startup1050, vec![0x1a, 4]),
            (
                CreationResponseFrame209026::ErrorUi1051 {
                    raw_dwords: [0, 127, 128, u32::MAX],
                },
                vec![0x1b, 4, 0, 0x7f, 0x80, 1, 0xff, 0xff, 0xff, 0xff, 0x0f],
            ),
        ]
    }

    #[test]
    fn creation_result_golden_streams_and_all_truncations() {
        let fixtures = fixtures();
        let mut stream: Vec<_> = fixtures
            .iter()
            .flat_map(|(_, wire)| wire.iter().copied())
            .collect();
        stream.push(0xa5);
        let mut input = stream.as_slice();
        for (frame, wire) in fixtures {
            let mut output = BytesMut::from(&b"prefix"[..]);
            frame.encode_frame(&mut output).unwrap();
            assert_eq!(&output[..6], b"prefix");
            assert_eq!(&output[6..], wire);
            assert!(CreationResponseFrame209026::decode_frame(&mut input).unwrap() == frame);
            for length in 0..wire.len() {
                let original = &wire[..length];
                let mut input = original;
                assert!(CreationResponseFrame209026::decode_frame(&mut input).is_err());
                assert_eq!(input, original);
            }
        }
        assert_eq!(input, [0xa5]);
    }

    #[test]
    fn creation_result_matches_extracted_schemas() {
        let rules = Rules {
            string: Some(StringEncoding::Utf16Terminated),
            array_var_large: Some(CountPrefix::U16),
            ..Rules::default()
        };
        for (xml, group) in [
            (
                include_str!("../../../data/protocol/209026/mission.xml"),
                "39",
            ),
            (
                include_str!("../../../data/protocol/209026/server-tables.xml"),
                "table-0x1422ddbf0",
            ),
        ] {
            let messages: Vec<_> = schema::parse(xml)
                .unwrap()
                .protocols
                .into_iter()
                .filter(|p| p.name == "0")
                .flat_map(|p| p.groups)
                .filter(|g| g.name == group)
                .flat_map(|g| g.server)
                .collect();
            for (frame, wire) in fixtures() {
                let expected = match &frame {
                    CreationResponseFrame209026::Result1048(result) => vec![
                        ("0".into(), Value::Guid(result.character_guid)),
                        ("1".into(), Value::Guid(result.raw_guid)),
                        ("2".into(), Value::Text("AΩ😀".into())),
                        ("3".into(), Value::U16(result.raw_word)),
                        ("4".into(), Value::U64(result.raw_qwords[0])),
                        ("5".into(), Value::U64(result.raw_qwords[1])),
                        (
                            "6".into(),
                            Value::Array(
                                result
                                    .blob
                                    .iter()
                                    .map(|&byte| Value::Struct(vec![("0".into(), Value::U8(byte))]))
                                    .collect(),
                            ),
                        ),
                    ],
                    CreationResponseFrame209026::Startup1049
                    | CreationResponseFrame209026::Startup1050 => vec![],
                    CreationResponseFrame209026::ErrorUi1051 { raw_dwords } => raw_dwords
                        .iter()
                        .enumerate()
                        .map(|(i, &value)| (i.to_string(), Value::U32(value)))
                        .collect(),
                };
                assert_eq!(
                    messages
                        .iter()
                        .filter(|m| m.id == frame.message_id())
                        .count(),
                    1
                );
                let mut input = wire.as_slice();
                let (message, value) =
                    dynamic::decode_frame(&messages, &rules, &mut input).unwrap();
                assert_eq!(message.id, frame.message_id());
                assert!(input.is_empty());
                assert_eq!(value, Value::Struct(expected));
                let mut output = BytesMut::new();
                message.id.encode(&mut output);
                dynamic::encode_message(message, &rules, &value, &mut output).unwrap();
                assert_eq!(output.as_ref(), wire);
                if message.id == 1048 {
                    assert_eq!(message.fields[2].size, Some(20));
                    assert_eq!(message.fields[6].kind, "ArrayVarLarge");
                    assert_eq!(message.fields[6].size, Some(1024));
                }
            }
        }
    }

    #[test]
    fn creation_result_wire_bounds_are_separate_from_consumer_and_text_validation() {
        for name in [vec![], vec![0xd800; 19]] {
            for length in [0, 512, 513, 1024] {
                let mut data = result();
                data.name_units = name.clone();
                data.blob = vec![0xa5; length];
                let frame = CreationResponseFrame209026::Result1048(data);
                let mut wire = BytesMut::new();
                frame.encode_frame(&mut wire).unwrap();
                let count_offset = 2 + 32 + (name.len() + 1) * 2 + 2 + 16;
                assert_eq!(
                    &wire[count_offset..count_offset + 2],
                    &(length as u16).to_le_bytes()
                );
                assert_eq!(wire.len(), count_offset + 2 + length);
                let mut input = wire.as_ref();
                assert!(CreationResponseFrame209026::decode_frame(&mut input).unwrap() == frame);
                assert!(input.is_empty());
            }
        }
        assert_eq!(RESULT_CONSUMER_BLOB_LIMIT209026, 512);
    }

    #[test]
    fn creation_result_invalid_encode_is_atomic() {
        let mut too_long_name = result();
        too_long_name.name_units = vec![1; 20];
        let mut embedded_nul = result();
        embedded_nul.name_units = vec![1, 0, 2];
        let mut too_long_blob = result();
        too_long_blob.blob = vec![0; 1025];
        for data in [too_long_name, embedded_nul, too_long_blob] {
            let mut output = BytesMut::from(&b"keep"[..]);
            assert_eq!(
                CreationResponseFrame209026::Result1048(data)
                    .encode_frame(&mut output)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidInput
            );
            assert_eq!(output.as_ref(), b"keep");
        }
    }

    #[test]
    fn creation_result_invalid_decode_is_transactional() {
        let mut invalid = vec![vec![0x17, 4]];
        let mut unterminated = fixtures()[0].1[..34].to_vec();
        for _ in 0..20 {
            unterminated.extend_from_slice(&[1, 0]);
        }
        invalid.push(unterminated);
        let mut oversized = fixtures()[0].1.clone();
        let count_offset = oversized.len() - 5;
        oversized[count_offset..count_offset + 2].copy_from_slice(&1025u16.to_le_bytes());
        invalid.push(oversized);
        for field in 0..4 {
            let mut overflow = vec![0x1b, 4];
            overflow.extend(std::iter::repeat_n(0, field));
            overflow.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x1f]);
            invalid.push(overflow);
        }
        for original in invalid {
            let mut input = original.as_slice();
            let error = CreationResponseFrame209026::decode_frame(&mut input)
                .err()
                .unwrap();
            assert_eq!(error.kind(), ErrorKind::InvalidData);
            assert_eq!(input, original);
        }
    }
}
