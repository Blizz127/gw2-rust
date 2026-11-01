//! Metadata-only inspection of decrypted build-209026 captures.
//! Wire evidence and limits: docs/re/209026-character-protocol.md.
use std::{
    collections::HashSet,
    fs::File,
    io::{self, Read, Write},
};

use gw2_proto::{
    dynamic::{self, CountPrefix, Rules, StringEncoding},
    schema::{self, Message},
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

fn census(messages: &[Message], bytes: &[u8], output: &mut impl Write) -> io::Result<usize> {
    let mut ids = HashSet::new();
    if messages.iter().any(|message| !ids.insert(message.id)) {
        return Err(invalid("duplicate message ID in selected group"));
    }
    let rules = Rules {
        string: Some(StringEncoding::Utf16Terminated),
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
        if message.fields.iter().any(|field| {
            !field.fields.is_empty()
                || !matches!(
                    field.kind.as_str(),
                    "Byte"
                        | "Word"
                        | "Qword"
                        | "Dword"
                        | "Guid"
                        | "String"
                        | "BufferFixed"
                        | "BufferVarSmall"
                        | "BufferVarLarge"
                )
        }) {
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
