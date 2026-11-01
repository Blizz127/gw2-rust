//! Metadata-only inspection of decrypted build-209026 captures.
//! Wire evidence and limits: docs/re/209026-character-protocol.md,
//! docs/re/209026-character-creation.md, docs/re/209026-mission-join.md.
use std::{
    collections::HashSet,
    fs::File,
    io::{self, Read, Write},
};

use gw2_proto::{
    dynamic::{self, CountPrefix, Rules, StringEncoding},
    schema::{self, Field, Message},
};

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn bounded_file(path: &str, maximum: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take((maximum + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(invalid("input exceeds size limit"));
    }
    Ok(bytes)
}

fn verified_primitive(field: &Field) -> bool {
    field.fields.is_empty()
        && field.type_name.is_none()
        && matches!(
            field.kind.as_str(),
            "Byte"
                | "Word"
                | "Qword"
                | "Dword"
                | "Guid"
                | "Address"
                | "String"
                | "BufferFixed"
                | "BufferVarSmall"
                | "BufferVarLarge"
        )
}

fn verified_field(field: &Field) -> bool {
    verified_primitive(field)
        || (matches!(field.kind.as_str(), "ArrayFixed" | "ArrayVarSmall" | "ArrayVarLarge")
            && field.type_name.is_none()
            && field.size.is_some_and(|n| n <= dynamic::MAX_ELEMENTS as u32)
            && !field.fields.is_empty()
            && field.fields.iter().all(verified_primitive)
            // Every element must consume input. Do not admit empty fixed buffers
            // as the only fields in an array element.
            && field.fields.iter().any(|f| f.kind != "BufferFixed" || f.size.is_some_and(|n| n > 0)))
}

fn census(messages: &[Message], bytes: &[u8], output: &mut impl Write) -> io::Result<usize> {
    let mut ids = HashSet::new();
    if messages.iter().any(|message| !ids.insert(message.id)) {
        return Err(invalid("duplicate message ID in selected group"));
    }
    let rules = Rules {
        string: Some(StringEncoding::Utf16Terminated),
        array_var_small: Some(CountPrefix::U8),
        array_var_large: Some(CountPrefix::U16),
        buffer_var_small: Some(CountPrefix::U8),
        buffer_var_large: Some(CountPrefix::U16),
        buffer_fixed_is_size_bytes: true,
        ..Rules::default()
    };
    let mut remaining = bytes;
    let mut count = 0;
    while !remaining.is_empty() {
        let offset = bytes.len() - remaining.len();
        let header = remaining
            .get(..2)
            .ok_or_else(|| invalid(format!("incomplete ID at offset {offset}")))?;
        let id = u16::from_le_bytes(header.try_into().unwrap());
        let message = messages
            .iter()
            .find(|message| message.id == id)
            .ok_or_else(|| invalid(format!("unknown ID {id} at offset {offset}")))?;
        // Restrict historical primitives too: a supported codec is not proof
        // that its encoding has been observed for this build.
        if !message.fields.iter().all(verified_field) {
            return Err(invalid(format!(
                "unverified field encoding for ID {id} at offset {offset}"
            )));
        }
        let before = remaining.len();
        dynamic::decode_frame(messages, &rules, &mut remaining).map_err(|error| {
            invalid(format!(
                "cannot decode ID {id} at offset {offset}: {:?}",
                error.kind()
            ))
        })?;
        writeln!(
            output,
            "offset={offset} id={id} bytes={}",
            before - remaining.len()
        )?;
        count += 1;
    }
    writeln!(
        output,
        "messages={count} bytes={} complete=true",
        bytes.len()
    )?;
    Ok(count)
}

fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 || !matches!(args[3].as_str(), "client" | "server") {
        return Err(invalid(
            "usage: gw2-capture-census SCHEMA PROTOCOL GROUP client|server CAPTURE",
        ));
    }
    let xml = bounded_file(&args[0], 4 * 1024 * 1024)?;
    let xml = std::str::from_utf8(&xml).map_err(|_| invalid("schema is not UTF-8"))?;
    let schema = schema::parse(xml).map_err(|_| invalid("invalid schema"))?;
    if schema.build_id != Some(209026) {
        return Err(invalid("only build 209026 wire rules are enabled"));
    }
    let groups: Vec<_> = schema
        .protocols
        .iter()
        .filter(|p| p.name == args[1])
        .flat_map(|p| &p.groups)
        .filter(|g| g.name == args[2])
        .collect();
    if groups.len() != 1 {
        return Err(invalid("select exactly one protocol/group"));
    }
    let messages = if args[3] == "client" {
        &groups[0].client
    } else {
        &groups[0].server
    };
    let bytes = bounded_file(&args[4], 1024 * 1024)?;
    census(messages, &bytes, &mut io::stdout().lock())?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("capture census: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_and_route_fields_preserve_boundaries_without_values() {
        let schema = schema::parse(r#"<Protocols BuildId="209026"><Protocol Name="0"><Messages Name="39"><Client><Message Id="415"><String Size="20"/><ArrayFixed Size="48"><Byte/></ArrayFixed><Byte/><Byte/><Dword/><ArrayVarSmall Size="32"><Byte/></ArrayVarSmall><ArrayFixed Size="24"><Dword/></ArrayFixed></Message></Client></Messages></Protocol><Protocol Name="3"><Messages Name="40"><Server><Message Id="20"><Dword/><Dword/><Dword/><Dword/><Address/><Dword/></Message></Server></Messages></Protocol></Protocols>"#).unwrap();
        let creation = &schema.protocols[0].groups[0].client;
        let route = &schema.protocols[1].groups[0].server;
        let mut create_frame = vec![0x9f, 1, b'A', 0, 0, 0];
        create_frame.extend_from_slice(&[0; 48]);
        create_frame.extend_from_slice(&[0, 1, 8, 3, 0, 0x80, 0xff]);
        // The first of 24 Dwords is 300, consuming two wire bytes.
        create_frame.extend_from_slice(&[0xac, 2]);
        create_frame.extend_from_slice(&[0; 23]);
        assert_eq!(create_frame.len(), 86);

        let mut route_frame = vec![20, 0, 1, 0x80, 1, 7, 0];
        route_frame.extend_from_slice(&[2, 0, 0x17, 0xe0, 127, 1, 0, 2]);
        route_frame.extend_from_slice(&[0; 20]);
        route_frame.push(0);
        assert_eq!(route_frame.len(), 36);

        for (messages, frame, id) in [(creation, &create_frame, 415), (route, &route_frame, 20)] {
            for end in 1..frame.len() {
                assert!(census(messages, &frame[..end], &mut Vec::new()).is_err());
            }
            let mut pair = frame.clone();
            pair.extend_from_slice(frame);
            let mut output = Vec::new();
            assert_eq!(census(messages, &pair, &mut output).unwrap(), 2);
            let size = frame.len();
            assert_eq!(
                String::from_utf8(output).unwrap(),
                format!(
                    "offset=0 id={id} bytes={size}\noffset={size} id={id} bytes={size}\nmessages=2 bytes={} complete=true\n",
                    size * 2
                )
            );
        }

        // An invalid family must be rejected without exposing address bytes.
        route_frame[7] = 0xff;
        let error = census(route, &route_frame, &mut Vec::new()).unwrap_err();
        assert_eq!(
            error.to_string(),
            "cannot decode ID 20 at offset 0: InvalidData"
        );
        for size in [None, Some(65_537)] {
            let mut malformed = creation.clone();
            malformed[0].fields[1].size = size;
            assert!(census(&malformed, &create_frame, &mut Vec::new()).is_err());
        }
    }

    #[test]
    fn arrays_keep_record_boundaries_and_reject_bad_counts_or_elements() {
        let xml = r#"<Protocols BuildId="209026"><Protocol Name="3"><Messages Name="40"><Server><Message Id="13"><ArrayVarSmall Size="2"><Word/><Word/><Dword/><Dword/><Dword/></ArrayVarSmall></Message><Message Id="20"><ArrayVarLarge Size="2"><Qword/></ArrayVarLarge></Message></Server></Messages></Protocol></Protocols>"#;
        let schema = schema::parse(xml).unwrap();
        let messages = &schema.protocols[0].groups[0].server;
        // Handwritten wire data: two records, including multi-byte varints.
        let mut small = vec![13, 0, 2, 0x34, 0x12, 0x78, 0x56, 0x80, 1, 0, 1];
        small.extend_from_slice(&[0, 0, 1, 0, 2, 0xff, 1, 3]);
        let large = [20, 0, 1, 0, 1, 2, 3, 4, 5, 6, 7, 8];
        for frame in [small.as_slice(), large.as_slice()] {
            for end in 1..frame.len() {
                assert!(census(messages, &frame[..end], &mut Vec::new()).is_err());
            }
        }
        let mut bytes = small.clone();
        bytes.extend_from_slice(&large);
        let mut output = Vec::new();
        assert_eq!(census(messages, &bytes, &mut output).unwrap(), 2);
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "offset=0 id=13 bytes=19\noffset=19 id=20 bytes=12\nmessages=2 bytes=31 complete=true\n"
        );
        small[2] = 3;
        assert!(census(messages, &small, &mut Vec::new()).is_err());
        for field in [
            r#"<ArrayVarSmall Size="2"><Float/></ArrayVarSmall>"#,
            r#"<ArrayVarSmall Size="2"><ArrayVarSmall Size="2"><Byte/></ArrayVarSmall></ArrayVarSmall>"#,
            r#"<ArrayVarSmall Size="2" TypeName="Word"/>"#,
            r#"<ArrayVarSmall Size="2"><BufferFixed Size="0"/></ArrayVarSmall>"#,
            r#"<ArrayVarSmall><Word/></ArrayVarSmall>"#,
        ] {
            let candidate = schema::parse(&xml.replace(
                r#"<ArrayVarSmall Size="2"><Word/><Word/><Dword/><Dword/><Dword/></ArrayVarSmall>"#,
                field,
            ))
            .unwrap();
            let error = census(
                &candidate.protocols[0].groups[0].server,
                &[13, 0, 0],
                &mut Vec::new(),
            )
            .unwrap_err();
            assert!(error.to_string().contains("unverified field encoding"));
        }
    }

    #[test]
    fn reports_boundaries_without_values_and_stops_on_unverified_encoding() {
        let schema = schema::parse(r#"<Protocols BuildId="209026"><Protocol Name="3"><Messages Name="40"><Client><Message Id="2"><String Size="32"/></Message><Message Id="3"><Float/></Message></Client></Messages></Protocol></Protocols>"#).unwrap();
        let messages = &schema.protocols[0].groups[0].client;
        let mut bytes = vec![2, 0];
        for unit in "private-test-value".encode_utf16().chain([0]) {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let mut output = Vec::new();
        assert_eq!(census(messages, &bytes, &mut output).unwrap(), 1);
        let text = String::from_utf8(output).unwrap();
        assert_eq!(
            text,
            "offset=0 id=2 bytes=40\nmessages=1 bytes=40 complete=true\n"
        );
        assert!(!text.contains("private-test-value"));
        for end in 1..bytes.len() {
            assert!(census(messages, &bytes[..end], &mut Vec::new()).is_err());
        }
        bytes.extend_from_slice(&[3, 0, 0]);
        let mut output = Vec::new();
        let error = census(messages, &bytes, &mut output).unwrap_err();
        assert!(error.to_string().contains("unverified field encoding"));
        assert!(!String::from_utf8(output).unwrap().contains("complete=true"));
    }
}
