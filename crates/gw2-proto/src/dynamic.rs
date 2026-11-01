//! Schema-driven ("dynamic") message decoding and encoding.
//!
//! Given a [`schema::Message`], this decodes the message body into a [`Value`]
//! tree and encodes one back. It powers capture tools and early server work
//! before typed message structs are generated.
//!
//! Field kinds whose wire encoding is known from the reference implementation
//! are always supported:
//! `Byte Word Dword Qword Float Float2/3/4 Point3 Guid Address Optional
//! ArrayFixed`.
//!
//! Kinds whose encoding is **not established** for our build go through [`Rules`]
//! and are rejected by default: `String CString ArrayVarSmall ArrayVarLarge
//! BufferVarSmall BufferVarLarge BufferFixed Unknown`. Set a rule only once REA
//! (or a capture cross-checked with REA) shows the encoding.

use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;

use bytes::BytesMut;

use crate::schema::{Field, Message};
use crate::{Decode, Encode, Guid, Point3};

/// How a variable-length count is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountPrefix {
    U8,
    U16,
    Varint,
}

/// How a `String` (wide string) is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    /// Count of UTF-16 code units (per [`CountPrefix`]), then the units, little-endian.
    Utf16Counted(CountPrefix),
    /// Build 209026: UTF-16LE units followed by zero; Size includes the terminator.
    Utf16Terminated,
}

/// Encodings that are not yet confirmed. `None` means "reject this field kind".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rules {
    pub string: Option<StringEncoding>,
    /// `CString`: count-prefixed bytes (no terminator on the wire).
    pub cstring: Option<CountPrefix>,
    pub array_var_small: Option<CountPrefix>,
    pub array_var_large: Option<CountPrefix>,
    pub buffer_var_small: Option<CountPrefix>,
    pub buffer_var_large: Option<CountPrefix>,
    /// `BufferFixed`: exactly `Size` raw bytes.
    pub buffer_fixed_is_size_bytes: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    Floats(Vec<f32>),
    Point3(Point3),
    Guid(Guid),
    Address(SocketAddr),
    Optional(Option<Box<Value>>),
    Array(Vec<Value>),
    Bytes(Vec<u8>),
    Text(String),
    /// Named fields in schema order.
    Struct(Vec<(String, Value)>),
}

fn unsupported(kind: &str) -> Error {
    Error::new(ErrorKind::Unsupported, format!("field kind {kind} has no confirmed encoding"))
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidData, msg.into())
}

fn read_count(p: CountPrefix, buf: &mut &[u8]) -> Result<usize> {
    Ok(match p {
        CountPrefix::U8 => u8::decode(buf)? as usize,
        CountPrefix::U16 => u16::decode(buf)? as usize,
        CountPrefix::Varint => u32::decode(buf)? as usize,
    })
}

fn write_count(p: CountPrefix, n: usize, out: &mut BytesMut) -> Result<()> {
    let too_big = || invalid(format!("count {n} does not fit {p:?}"));
    match p {
        CountPrefix::U8 => u8::try_from(n).map_err(|_| too_big())?.encode(out),
        CountPrefix::U16 => u16::try_from(n).map_err(|_| too_big())?.encode(out),
        CountPrefix::Varint => u32::try_from(n).map_err(|_| too_big())?.encode(out),
    }
    Ok(())
}

fn take_bytes(buf: &mut &[u8], n: usize) -> Result<Vec<u8>> {
    if buf.len() < n {
        return Err(Error::from(ErrorKind::UnexpectedEof));
    }
    let (head, rest) = buf.split_at(n);
    *buf = rest;
    Ok(head.to_vec())
}

/// A field's element description for containers: either a primitive kind
/// (`TypeName`) or a struct of sub-fields.
fn element_of(field: &Field) -> Field {
    match &field.type_name {
        Some(t) => Field { kind: t.clone(), name: String::new(), size: None, type_name: None, fields: Vec::new() },
        None => Field { kind: "Struct".into(), name: String::new(), size: None, type_name: None, fields: field.fields.clone() },
    }
}

/// Upper bound on a decoded element count. This guards against a misparsed
/// count allocating huge vectors.
pub const MAX_ELEMENTS: usize = 1 << 16;

pub fn decode_message(msg: &Message, rules: &Rules, buf: &mut &[u8]) -> Result<Value> {
    decode_struct(&msg.fields, rules, buf)
}

fn decode_struct(fields: &[Field], rules: &Rules, buf: &mut &[u8]) -> Result<Value> {
    let mut out = Vec::with_capacity(fields.len());
    for f in fields {
        out.push((f.name.clone(), decode_field(f, rules, buf)?));
    }
    Ok(Value::Struct(out))
}

fn decode_elements(field: &Field, n: usize, rules: &Rules, buf: &mut &[u8]) -> Result<Value> {
    if n > MAX_ELEMENTS {
        return Err(invalid(format!("element count {n} over {MAX_ELEMENTS}")));
    }
    let el = element_of(field);
    let mut v = Vec::with_capacity(n.min(1024));
    for _ in 0..n {
        v.push(decode_field(&el, rules, buf)?);
    }
    Ok(Value::Array(v))
}

pub fn decode_field(f: &Field, rules: &Rules, buf: &mut &[u8]) -> Result<Value> {
    let size = f.size.map(|s| s as usize);
    Ok(match f.kind.as_str() {
        "Byte" => Value::U8(u8::decode(buf)?),
        "Word" => Value::U16(u16::decode(buf)?),
        "Dword" => Value::U32(u32::decode(buf)?),
        "Qword" => Value::U64(u64::decode(buf)?),
        "Float" => Value::F32(f32::decode(buf)?),
        "Float2" => Value::Floats(<[f32; 2]>::decode(buf)?.to_vec()),
        "Float3" => Value::Floats(<[f32; 3]>::decode(buf)?.to_vec()),
        "Float4" => Value::Floats(<[f32; 4]>::decode(buf)?.to_vec()),
        "Point3" => Value::Point3(Point3::decode(buf)?),
        "Guid" => Value::Guid(Guid::decode(buf)?),
        "Address" => Value::Address(SocketAddr::decode(buf)?),
        "Struct" => decode_struct(&f.fields, rules, buf)?,
        "Optional" => match u8::decode(buf)? {
            0 => Value::Optional(None),
            1 => Value::Optional(Some(Box::new(decode_field(&element_of(f), rules, buf)?))),
            t => return Err(invalid(format!("bad Optional tag {t}"))),
        },
        "ArrayFixed" => {
            let n = size.ok_or_else(|| invalid("ArrayFixed without Size"))?;
            decode_elements(f, n, rules, buf)?
        }
        "ArrayVarSmall" | "ArrayVarLarge" => {
            let rule = if f.kind == "ArrayVarSmall" { rules.array_var_small } else { rules.array_var_large };
            let p = rule.ok_or_else(|| unsupported(&f.kind))?;
            let n = read_count(p, buf)?;
            if let Some(max) = size
                && n > max
            {
                return Err(invalid(format!("{} count {n} over Size {max}", f.kind)));
            }
            decode_elements(f, n, rules, buf)?
        }
        "BufferVarSmall" | "BufferVarLarge" => {
            let rule = if f.kind == "BufferVarSmall" { rules.buffer_var_small } else { rules.buffer_var_large };
            let p = rule.ok_or_else(|| unsupported(&f.kind))?;
            let n = read_count(p, buf)?;
            if let Some(max) = size
                && n > max
            {
                return Err(invalid(format!("{} count {n} over Size {max}", f.kind)));
            }
            Value::Bytes(take_bytes(buf, n)?)
        }
        "BufferFixed" if rules.buffer_fixed_is_size_bytes => {
            let n = size.ok_or_else(|| invalid("BufferFixed without Size"))?;
            Value::Bytes(take_bytes(buf, n)?)
        }
        "String" => match rules.string {
            Some(StringEncoding::Utf16Terminated) => {
                let max = size.unwrap_or(MAX_ELEMENTS).min(MAX_ELEMENTS);
                let mut units = Vec::new();
                loop {
                    if units.len() >= max {
                        return Err(invalid("String exceeds Size without a terminator"));
                    }
                    let unit = u16::decode(buf)?;
                    if unit == 0 { break; }
                    units.push(unit);
                }
                Value::Text(String::from_utf16(&units).map_err(|_| invalid("bad UTF-16"))?)
            }
            Some(StringEncoding::Utf16Counted(p)) => {
                let n = read_count(p, buf)?;
                if let Some(max) = size
                    && n > max
                {
                    return Err(invalid(format!("String length {n} over Size {max}")));
                }
                let raw = take_bytes(buf, n * 2)?;
                let units: Vec<u16> = raw.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect();
                Value::Text(String::from_utf16(&units).map_err(|_| invalid("bad UTF-16"))?)
            }
            None => return Err(unsupported("String")),
        },
        "CString" => {
            let p = rules.cstring.ok_or_else(|| unsupported("CString"))?;
            let n = read_count(p, buf)?;
            Value::Bytes(take_bytes(buf, n)?)
        }
        other => return Err(unsupported(other)),
    })
}

pub fn encode_message(msg: &Message, rules: &Rules, value: &Value, out: &mut BytesMut) -> Result<()> {
    encode_struct(&msg.fields, rules, value, out)
}

fn encode_struct(fields: &[Field], rules: &Rules, value: &Value, out: &mut BytesMut) -> Result<()> {
    let Value::Struct(items) = value else { return Err(invalid("expected Struct value")) };
    if items.len() != fields.len() {
        return Err(invalid(format!("struct has {} values for {} fields", items.len(), fields.len())));
    }
    for (f, (_, v)) in fields.iter().zip(items) {
        encode_field(f, rules, v, out)?;
    }
    Ok(())
}

fn encode_elements(f: &Field, items: &[Value], rules: &Rules, out: &mut BytesMut) -> Result<()> {
    let el = element_of(f);
    for v in items {
        encode_field(&el, rules, v, out)?;
    }
    Ok(())
}

pub fn encode_field(f: &Field, rules: &Rules, v: &Value, out: &mut BytesMut) -> Result<()> {
    let mismatch = || invalid(format!("value {v:?} does not match field kind {}", f.kind));
    let size = f.size.map(|s| s as usize);
    match (f.kind.as_str(), v) {
        ("Byte", Value::U8(x)) => x.encode(out),
        ("Word", Value::U16(x)) => x.encode(out),
        ("Dword", Value::U32(x)) => x.encode(out),
        ("Qword", Value::U64(x)) => x.encode(out),
        ("Float", Value::F32(x)) => x.encode(out),
        ("Float2", Value::Floats(x)) if x.len() == 2 => x.iter().for_each(|f| f.encode(out)),
        ("Float3", Value::Floats(x)) if x.len() == 3 => x.iter().for_each(|f| f.encode(out)),
        ("Float4", Value::Floats(x)) if x.len() == 4 => x.iter().for_each(|f| f.encode(out)),
        ("Point3", Value::Point3(x)) => x.encode(out),
        ("Guid", Value::Guid(x)) => x.encode(out),
        ("Address", Value::Address(x)) => x.encode(out),
        ("Struct", _) => encode_struct(&f.fields, rules, v, out)?,
        ("Optional", Value::Optional(None)) => 0u8.encode(out),
        ("Optional", Value::Optional(Some(inner))) => {
            1u8.encode(out);
            encode_field(&element_of(f), rules, inner, out)?;
        }
        ("ArrayFixed", Value::Array(items)) => {
            if Some(items.len()) != size {
                return Err(invalid(format!("ArrayFixed needs {size:?} elements, got {}", items.len())));
            }
            encode_elements(f, items, rules, out)?;
        }
        ("ArrayVarSmall" | "ArrayVarLarge", Value::Array(items)) => {
            let rule = if f.kind == "ArrayVarSmall" { rules.array_var_small } else { rules.array_var_large };
            let p = rule.ok_or_else(|| unsupported(&f.kind))?;
            if let Some(max) = size
                && items.len() > max
            {
                return Err(invalid(format!("{} has {} elements, Size {max}", f.kind, items.len())));
            }
            write_count(p, items.len(), out)?;
            encode_elements(f, items, rules, out)?;
        }
        ("BufferVarSmall" | "BufferVarLarge", Value::Bytes(b)) => {
            let rule = if f.kind == "BufferVarSmall" { rules.buffer_var_small } else { rules.buffer_var_large };
            let p = rule.ok_or_else(|| unsupported(&f.kind))?;
            if let Some(max) = size
                && b.len() > max
            {
                return Err(invalid(format!("{} has {} bytes, Size {max}", f.kind, b.len())));
            }
            write_count(p, b.len(), out)?;
            out.extend_from_slice(b);
        }
        ("BufferFixed", Value::Bytes(b)) if rules.buffer_fixed_is_size_bytes => {
            if Some(b.len()) != size {
                return Err(invalid("BufferFixed length differs from Size"));
            }
            out.extend_from_slice(b);
        }
        ("String", Value::Text(s)) => match rules.string {
            Some(StringEncoding::Utf16Terminated) => {
                let units: Vec<u16> = s.encode_utf16().collect();
                if units.contains(&0) {
                    return Err(invalid("terminated String contains a zero unit"));
                }
                let max = size.unwrap_or(MAX_ELEMENTS).min(MAX_ELEMENTS);
                if units.len() >= max {
                    return Err(invalid("String and terminator exceed Size"));
                }
                units.iter().for_each(|u| u.encode(out));
                0u16.encode(out);
            }
            Some(StringEncoding::Utf16Counted(p)) => {
                let units: Vec<u16> = s.encode_utf16().collect();
                if let Some(max) = size
                    && units.len() > max
                {
                    return Err(invalid(format!("String of {} units over Size {max}", units.len())));
                }
                write_count(p, units.len(), out)?;
                units.iter().for_each(|u| u.encode(out));
            }
            None => return Err(unsupported("String")),
        },
        ("CString", Value::Bytes(b)) => {
            write_count(rules.cstring.ok_or_else(|| unsupported("CString"))?, b.len(), out)?;
            out.extend_from_slice(b);
        }
        (k, _) if !KNOWN_KINDS.contains(&k) => return Err(unsupported(k)),
        _ => return Err(mismatch()),
    }
    Ok(())
}

const KNOWN_KINDS: &[&str] = &[
    "Byte", "Word", "Dword", "Qword", "Float", "Float2", "Float3", "Float4", "Point3", "Guid",
    "Address", "Struct", "Optional", "ArrayFixed", "ArrayVarSmall", "ArrayVarLarge",
    "BufferVarSmall", "BufferVarLarge", "BufferFixed", "String", "CString",
];

/// Whether every field in a message (recursively) is decodable under `rules`.
pub fn is_supported(fields: &[Field], rules: &Rules) -> bool {
    fields.iter().all(|f| {
        let own = match f.kind.as_str() {
            "String" => rules.string.is_some(),
            "CString" => rules.cstring.is_some(),
            "ArrayVarSmall" => rules.array_var_small.is_some(),
            "ArrayVarLarge" => rules.array_var_large.is_some(),
            "BufferVarSmall" => rules.buffer_var_small.is_some(),
            "BufferVarLarge" => rules.buffer_var_large.is_some(),
            "BufferFixed" => rules.buffer_fixed_is_size_bytes,
            "Unknown" => false,
            k => KNOWN_KINDS.contains(&k),
        };
        let elem_ok = f.type_name.as_deref().is_none_or(|t| is_supported(&[element_of(f)], rules) && t != "Unknown");
        own && elem_ok && is_supported(&f.fields, rules)
    })
}

/// Decodes a full message frame: `u16` id, then the body of the matching
/// message from `candidates`. Returns the message and its value.
pub fn decode_frame<'m>(
    candidates: &'m [Message],
    rules: &Rules,
    buf: &mut &[u8],
) -> Result<(&'m Message, Value)> {
    let id = u16::decode(buf)?;
    let msg = candidates
        .iter()
        .find(|m| m.id == id)
        .ok_or_else(|| invalid(format!("no schema for message id {id}")))?;
    Ok((msg, decode_message(msg, rules, buf)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::parse;

    const XML: &str = r#"<Protocols BuildId="1"><Protocol Name="0"><Messages Name="0"><Client>
        <Message Id="10" Name="known">
          <Byte Name="0"/><Word Name="1"/><Dword Name="2"/><Qword Name="3"/><Float3 Name="4"/>
          <Point3 Name="5"/><Guid Name="6"/><Address Name="7"/>
          <Optional Name="8" TypeName="Dword"/>
          <ArrayFixed Name="9" Size="2"><Byte Name="0"/><Float Name="1"/></ArrayFixed>
        </Message>
        <Message Id="11" Name="vars">
          <String Name="0" Size="8"/><ArrayVarSmall Name="1" Size="4" TypeName="Word"/>
          <BufferVarLarge Name="2"/>
        </Message>
      </Client></Messages></Protocol></Protocols>"#;

    fn known_value() -> Value {
        Value::Struct(vec![
            ("0".into(), Value::U8(7)),
            ("1".into(), Value::U16(0xBEEF)),
            ("2".into(), Value::U32(300)),
            ("3".into(), Value::U64(1 << 40)),
            ("4".into(), Value::Floats(vec![1.0, 2.0, 3.0])),
            ("5".into(), Value::Point3(Point3([4.0, 5.0, 6.0], 15))),
            ("6".into(), Value::Guid(Guid([9; 16]))),
            ("7".into(), Value::Address("127.0.0.1:6112".parse().unwrap())),
            ("8".into(), Value::Optional(Some(Box::new(Value::U32(5))))),
            (
                "9".into(),
                Value::Array(vec![
                    Value::Struct(vec![("0".into(), Value::U8(1)), ("1".into(), Value::F32(0.5))]),
                    Value::Struct(vec![("0".into(), Value::U8(2)), ("1".into(), Value::F32(1.5))]),
                ]),
            ),
        ])
    }

    #[test]
    fn known_kinds_round_trip_as_a_frame() {
        let s = parse(XML).unwrap();
        let msgs = &s.protocols[0].groups[0].client;
        let rules = Rules::default();
        let mut out = BytesMut::new();
        10u16.encode(&mut out);
        encode_message(&msgs[0], &rules, &known_value(), &mut out).unwrap();
        let mut slice = &out[..];
        let (m, v) = decode_frame(msgs, &rules, &mut slice).unwrap();
        assert_eq!(m.id, 10);
        assert_eq!(v, known_value());
        assert!(slice.is_empty());
        assert!(is_supported(&msgs[0].fields, &rules));
    }

    #[test]
    fn unconfirmed_kinds_are_rejected_by_default() {
        let s = parse(XML).unwrap();
        let m = &s.protocols[0].groups[0].client[1];
        let rules = Rules::default();
        assert!(!is_supported(&m.fields, &rules));
        let err = decode_message(m, &rules, &mut &[1u8, 0, 65, 0][..]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Unsupported);
    }

    #[test]
    fn rules_enable_variable_kinds_with_bounds() {
        let s = parse(XML).unwrap();
        let m = &s.protocols[0].groups[0].client[1];
        let rules = Rules {
            string: Some(StringEncoding::Utf16Counted(CountPrefix::U16)),
            array_var_small: Some(CountPrefix::U8),
            buffer_var_large: Some(CountPrefix::Varint),
            ..Rules::default()
        };
        assert!(is_supported(&m.fields, &rules));
        let v = Value::Struct(vec![
            ("0".into(), Value::Text("Ascalon".into())),
            ("1".into(), Value::Array(vec![Value::U16(1), Value::U16(2)])),
            ("2".into(), Value::Bytes(vec![0xAA; 200])),
        ]);
        let mut out = BytesMut::new();
        encode_message(m, &rules, &v, &mut out).unwrap();
        let mut slice = &out[..];
        assert_eq!(decode_message(m, &rules, &mut slice).unwrap(), v);
        assert!(slice.is_empty());

        // Size bounds are enforced in both directions.
        let too_long = Value::Struct(vec![
            ("0".into(), Value::Text("Divinity's Reach".into())),
            ("1".into(), Value::Array(vec![])),
            ("2".into(), Value::Bytes(vec![])),
        ]);
        assert!(encode_message(m, &rules, &too_long, &mut BytesMut::new()).is_err());
        assert!(decode_message(m, &rules, &mut &[9u8, 0][..]).is_err());
    }

    #[test]
    fn truncated_and_mismatched_input_fail_cleanly() {
        let s = parse(XML).unwrap();
        let msgs = &s.protocols[0].groups[0].client;
        let rules = Rules::default();
        let mut full = BytesMut::new();
        encode_message(&msgs[0], &rules, &known_value(), &mut full).unwrap();
        for cut in 0..full.len() {
            assert!(decode_message(&msgs[0], &rules, &mut &full[..cut]).is_err(), "cut {cut}");
        }
        assert!(decode_frame(msgs, &rules, &mut &[99u8, 0][..]).is_err());
        assert!(encode_message(&msgs[0], &rules, &Value::U8(1), &mut BytesMut::new()).is_err());
    }

    #[test]
    fn variable_buffers_enforce_size_before_copying_or_writing() {
        let rules = Rules {
            buffer_var_small: Some(CountPrefix::U8),
            buffer_var_large: Some(CountPrefix::U16),
            ..Rules::default()
        };
        for (kind, prefix) in [("BufferVarSmall", vec![3]), ("BufferVarLarge", vec![3, 0])] {
            let mut field = Field { kind: kind.into(), name: String::new(), size: Some(3), type_name: None, fields: vec![] };
            let mut wire = prefix.clone();
            wire.extend_from_slice(&[0x81, 0, 0xfe]);
            let mut cursor = wire.as_slice();
            let value = Value::Bytes(vec![0x81, 0, 0xfe]);
            assert_eq!(decode_field(&field, &rules, &mut cursor).unwrap(), value);
            assert!(cursor.is_empty());
            let mut output = BytesMut::new();
            encode_field(&field, &rules, &value, &mut output).unwrap();
            assert_eq!(output.as_ref(), wire);

            for end in 0..wire.len() {
                assert_eq!(decode_field(&field, &rules, &mut &wire[..end]).unwrap_err().kind(), ErrorKind::UnexpectedEof);
            }
            // Reject the count before trying to read a missing oversized body.
            let mut over = prefix.clone();
            over[0] = 4;
            assert_eq!(decode_field(&field, &rules, &mut over.as_slice()).unwrap_err().kind(), ErrorKind::InvalidData);
            let mut output = BytesMut::from(&[0x55][..]);
            assert!(encode_field(&field, &rules, &Value::Bytes(vec![0; 4]), &mut output).is_err());
            assert_eq!(output.as_ref(), [0x55]);

            field.size = Some(0);
            let empty = vec![0; prefix.len()];
            assert_eq!(decode_field(&field, &rules, &mut empty.as_slice()).unwrap(), Value::Bytes(vec![]));
            assert!(encode_field(&field, &rules, &Value::Bytes(vec![1]), &mut BytesMut::new()).is_err());
        }
    }

    #[test]
    fn terminated_utf16_preserves_following_fields_and_bounds() {
        let field = Field { kind: "String".into(), name: "text".into(), size: Some(3), type_name: None, fields: vec![] };
        let rules = Rules { string: Some(StringEncoding::Utf16Terminated), ..Rules::default() };
        // A surrogate pair uses two units; the terminator occupies the third.
        let wire = [0x3d, 0xd8, 0x00, 0xde, 0, 0, 0x78, 0x56];
        let mut cursor = &wire[..];
        assert_eq!(decode_field(&field, &rules, &mut cursor).unwrap(), Value::Text("😀".into()));
        assert_eq!(cursor, &[0x78, 0x56]);
        let mut out = BytesMut::new();
        encode_field(&field, &rules, &Value::Text("😀".into()), &mut out).unwrap();
        assert_eq!(&out[..], &wire[..6]);
        for cut in 0..6 {
            assert_eq!(decode_field(&field, &rules, &mut &wire[..cut]).unwrap_err().kind(), ErrorKind::UnexpectedEof);
        }
        assert_eq!(decode_field(&field, &rules, &mut &[65, 0, 66, 0, 67, 0][..]).unwrap_err().kind(), ErrorKind::InvalidData);
        assert!(encode_field(&field, &rules, &Value::Text("abc".into()), &mut BytesMut::new()).is_err());
        assert!(encode_field(&field, &rules, &Value::Text("a\0".into()), &mut BytesMut::new()).is_err());
        assert_eq!(decode_field(&field, &rules, &mut &[0, 0][..]).unwrap(), Value::Text(String::new()));
    }

    #[test]
    fn reference_schema_census() {
        // 208039 reference: how many messages decode with only the confirmed kinds.
        let s = parse(include_str!("../reference/protocols-208039.xml")).unwrap();
        let rules = Rules::default();
        let total = s.messages().count();
        let ok = s.messages().filter(|m| is_supported(&m.fields, &rules)).count();
        assert_eq!(total, 1770);
        assert!(ok > 0 && ok < total, "supported {ok} of {total}");
    }
}
