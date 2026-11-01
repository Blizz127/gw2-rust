//! Explicit synthetic authentication probe, not an account service.
//! Message layouts: docs/re/209026-auth-messages.md and
//! docs/re/209026-character-protocol.md.
use bytes::{Bytes, BytesMut};
use gw2_proto::{
    Encode,
    character_summary::SummaryV11,
    dynamic::{self, CountPrefix, Rules, StringEncoding, Value},
    schema::{Field, Message},
};
use std::io::{self, ErrorKind};

pub const CHARACTER_BODY_LIMIT: usize = 238;

/// Explicit local fixtures. No Debug implementation: bodies can contain names.
#[derive(Clone)]
pub struct ReplyFixture {
    body: Bytes,
    character: Option<Bytes>,
}

impl ReplyFixture {
    pub fn new(body: Bytes, character: Option<Bytes>) -> io::Result<Self> {
        if body.is_empty() || body.len() > 65000 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "auth fixture body exceeds bound",
            ));
        }
        if let Some(record) = &character {
            validate_character(record)?;
        }
        Ok(Self { body, character })
    }
}

fn validate_character(bytes: &[u8]) -> io::Result<()> {
    let invalid = || io::Error::new(ErrorKind::InvalidData, "invalid character record fixture");
    if bytes.len() > CHARACTER_BODY_LIMIT {
        return Err(invalid());
    }
    let message = Message {
        id: 17,
        name: String::new(),
        addr: None,
        fields: vec![
            field("Guid", None),
            field("Dword", None),
            field("Qword", None),
            field("Qword", None),
            field("String", Some(20)),
            field("BufferVarLarge", Some(512)),
        ],
    };
    let rules = Rules {
        string: Some(StringEncoding::Utf16Terminated),
        buffer_var_large: Some(CountPrefix::U16),
        ..Rules::default()
    };
    let mut remaining = bytes;
    let Value::Struct(fields) =
        dynamic::decode_message(&message, &rules, &mut remaining).map_err(|_| invalid())?
    else {
        return Err(invalid());
    };
    let Value::Text(name) = &fields[4].1 else {
        return Err(invalid());
    };
    let Value::Bytes(summary) = &fields[5].1 else {
        return Err(invalid());
    };
    if !remaining.is_empty()
        || name.is_empty()
        || !SummaryV11::parse(summary)
            .map_err(|_| invalid())?
            .passes_preview_sex_check()
    {
        return Err(invalid());
    }
    Ok(())
}

fn field(kind: &str, size: Option<u32>) -> Field {
    Field {
        kind: kind.into(),
        name: String::new(),
        size,
        type_name: None,
        fields: vec![],
    }
}

fn messages() -> Vec<Message> {
    [
        (1, vec![field("Dword", None)]),
        (
            2,
            vec![field("String", Some(32)), field("String", Some(32))],
        ),
        (
            3,
            vec![field("Dword", None), field("BufferFixed", Some(16))],
        ),
        (
            10,
            vec![
                field("Dword", None),
                field("Guid", None),
                field("Guid", None),
                field("String", Some(64)),
                field("String", Some(28)),
            ],
        ),
    ]
    .into_iter()
    .map(|(id, fields)| Message {
        id,
        name: id.to_string(),
        addr: None,
        fields,
    })
    .collect()
}

pub struct AuthProbe {
    pending: BytesMut,
    messages: Vec<Message>,
    fixture: ReplyFixture,
    replied: bool,
}

impl AuthProbe {
    pub fn new(fixture: ReplyFixture) -> Self {
        Self {
            pending: BytesMut::new(),
            messages: messages(),
            fixture,
            replied: false,
        }
    }

    /// Parse only through the first complete ID 10. Incomplete reads retain all
    /// bytes; future messages remain capture-only after the explicit reply.
    pub fn push(&mut self, bytes: &[u8]) -> io::Result<Option<Bytes>> {
        if self.replied {
            return Ok(None);
        }
        if self.pending.len() + bytes.len() > 65536 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "auth probe input exceeds bound",
            ));
        }
        self.pending.extend_from_slice(bytes);
        let rules = Rules {
            string: Some(StringEncoding::Utf16Terminated),
            buffer_fixed_is_size_bytes: true,
            ..Rules::default()
        };
        while !self.pending.is_empty() {
            let mut cursor = &self.pending[..];
            let (message, value) = match dynamic::decode_frame(&self.messages, &rules, &mut cursor)
            {
                Ok(result) => result,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
                Err(e) => return Err(e),
            };
            let used = self.pending.len() - cursor.len();
            let id = message.id;
            let _ = self.pending.split_to(used);
            eprintln!("Auth probe parsed client message {id} ({used} bytes)");
            if id == 10 {
                let Value::Struct(fields) = value else {
                    unreachable!()
                };
                let Value::U32(request) = fields[0].1 else {
                    unreachable!()
                };
                let mut reply = BytesMut::new();
                8u16.encode(&mut reply);
                request.encode(&mut reply);
                reply.extend_from_slice(&self.fixture.body);
                // Configuration must arrive while this request is still pending.
                // Explicit empty fixture; no recovered account configuration yet.
                10u16.encode(&mut reply);
                request.encode(&mut reply);
                reply.extend_from_slice(&0u16.to_le_bytes());
                if let Some(record) = &self.fixture.character {
                    17u16.encode(&mut reply);
                    request.encode(&mut reply);
                    reply.extend_from_slice(record);
                }
                7u16.encode(&mut reply);
                request.encode(&mut reply);
                reply.extend_from_slice(&[0; 6]); // Two result fields, four status fields.
                self.replied = true;
                self.pending.clear();
                return Ok(Some(reply.freeze()));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gw2_proto::{Decode, Guid};

    fn character_record(name: &str, sex: u8) -> Bytes {
        let mut record = BytesMut::new();
        Guid([0x55; 16]).encode(&mut record);
        300u32.encode(&mut record);
        0x1234_5678_9abc_def0u64.encode(&mut record);
        0xfedc_ba98_7654_3210u64.encode(&mut record);
        for unit in name.encode_utf16().chain(std::iter::once(0)) {
            unit.encode(&mut record);
        }
        // A structural fixture only; no content-valid map/appearance claim.
        let mut summary = [0u8; 159];
        summary[0] = 11;
        summary[4] = 1;
        summary[5] = 1;
        summary[0x23] = sex;
        159u16.encode(&mut record);
        record.extend_from_slice(&summary);
        record.freeze()
    }

    #[test]
    fn character_fixture_rejects_truncation_trailing_data_and_bad_summary() {
        let record = character_record("🦀🦀🦀🦀🦀🦀🦀🦀abc", 1); // 19 UTF-16 units.
        validate_character(&record).unwrap();
        let mut maximum = record.to_vec();
        maximum.splice(16..18, [0xff, 0xff, 0xff, 0xff, 0x0f]);
        assert_eq!(maximum.len(), CHARACTER_BODY_LIMIT);
        validate_character(&maximum).unwrap();
        for cut in 0..record.len() {
            assert!(validate_character(&record[..cut]).is_err(), "cut {cut}");
        }
        let mut trailing = record.to_vec();
        trailing.push(0);
        assert!(validate_character(&trailing).is_err());
        assert!(validate_character(&character_record("", 0)).is_err());
        assert!(validate_character(&character_record("🦀🦀🦀🦀🦀🦀🦀🦀abcd", 0)).is_err());
        assert!(validate_character(&character_record("Fixture", 2)).is_err());
        let mut bad_version = record.to_vec();
        let start = bad_version.len() - 159;
        bad_version[start] = 10;
        assert!(validate_character(&bad_version).is_err());
        assert!(validate_character(&vec![0; CHARACTER_BODY_LIMIT + 1]).is_err());
    }

    #[test]
    fn character_record_is_correlated_and_precedes_completion_for_every_split() {
        let record = character_record("Fixture", 0);
        let body = Bytes::from_static(&[0x44; 2]);
        let mut input = BytesMut::new();
        10u16.encode(&mut input);
        123456u32.encode(&mut input);
        Guid([0x11; 16]).encode(&mut input);
        Guid([0x33; 16]).encode(&mut input);
        input.extend_from_slice(&[0; 4]);
        let mut expected = BytesMut::new();
        8u16.encode(&mut expected);
        123456u32.encode(&mut expected);
        expected.extend_from_slice(&body);
        10u16.encode(&mut expected);
        123456u32.encode(&mut expected);
        expected.extend_from_slice(&[0; 2]);
        17u16.encode(&mut expected);
        123456u32.encode(&mut expected);
        expected.extend_from_slice(&record);
        7u16.encode(&mut expected);
        123456u32.encode(&mut expected);
        expected.extend_from_slice(&[0; 6]);
        for cut in 0..input.len() {
            let fixture = ReplyFixture::new(body.clone(), Some(record.clone())).unwrap();
            let mut probe = AuthProbe::new(fixture);
            assert!(probe.push(&input[..cut]).unwrap().is_none());
            assert_eq!(probe.push(&input[cut..]).unwrap().unwrap(), expected);
            assert!(probe.push(&input).unwrap().is_none());
        }
    }

    #[test]
    fn fragmented_auth_request_replies_once_with_matching_request_id() {
        let mut input = BytesMut::new();
        2u16.encode(&mut input);
        input.extend_from_slice(&[65, 0, 0, 0, 66, 0, 0, 0]);
        3u16.encode(&mut input);
        300u32.encode(&mut input);
        input.extend_from_slice(&[0x22; 16]);
        1u16.encode(&mut input);
        42u32.encode(&mut input);
        10u16.encode(&mut input);
        123456u32.encode(&mut input);
        Guid([0x11; 16]).encode(&mut input);
        Guid([0x33; 16]).encode(&mut input);
        input.extend_from_slice(&[0, 0, 0, 0]);
        for cut in 0..input.len() {
            let mut probe =
                AuthProbe::new(ReplyFixture::new(Bytes::from_static(&[0x44; 2]), None).unwrap());
            assert!(probe.push(&input[..cut]).unwrap().is_none());
            let reply = probe.push(&input[cut..]).unwrap().unwrap();
            let mut cursor = &reply[..];
            assert_eq!(u16::decode(&mut cursor).unwrap(), 8);
            assert_eq!(u32::decode(&mut cursor).unwrap(), 123456);
            assert_eq!(&cursor[..2], &[0x44; 2]);
            cursor = &cursor[2..];
            assert_eq!(u16::decode(&mut cursor).unwrap(), 10);
            assert_eq!(u32::decode(&mut cursor).unwrap(), 123456);
            assert_eq!(&cursor[..2], &[0, 0]);
            cursor = &cursor[2..];
            assert_eq!(u16::decode(&mut cursor).unwrap(), 7);
            assert_eq!(u32::decode(&mut cursor).unwrap(), 123456);
            assert_eq!(cursor, &[0; 6]);
            assert!(probe.push(&input).unwrap().is_none());
        }
    }
}
