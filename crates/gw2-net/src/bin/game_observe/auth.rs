//! Explicit synthetic authentication probe, not an account service.
//! Message layouts: docs/re/209026-auth-messages.md.
use bytes::{Bytes, BytesMut};
use gw2_proto::{
    Encode,
    dynamic::{self, Rules, StringEncoding, Value},
    schema::{Field, Message},
};
use std::io::{self, ErrorKind};

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
    body: Bytes,
    replied: bool,
}

impl AuthProbe {
    pub fn new(body: Bytes) -> Self {
        Self {
            pending: BytesMut::new(),
            messages: messages(),
            body,
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
                reply.extend_from_slice(&self.body);
                // Configuration must arrive while this request is still pending.
                // Explicit empty fixture; no recovered account configuration yet.
                10u16.encode(&mut reply);
                request.encode(&mut reply);
                reply.extend_from_slice(&0u16.to_le_bytes());
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
            let mut probe = AuthProbe::new(Bytes::from_static(&[0x44; 2]));
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
