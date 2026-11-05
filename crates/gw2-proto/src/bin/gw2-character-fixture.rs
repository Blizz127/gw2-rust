//! Build a private synthetic auth character body from explicit appearance bytes.
//! Unknown fields and preview equipment are zero; this is not character creation.

use bytes::BytesMut;
use gw2_proto::{
    Encode, Guid,
    character_summary::{SummaryV11, V11_SIZE},
};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
};

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn build(
    appearance: &[u8],
    profession: u8,
    map: u16,
    name: &str,
    guid: Guid,
) -> io::Result<([u8; V11_SIZE], BytesMut)> {
    if appearance.len() != 45
        || name.is_empty()
        || name.encode_utf16().count() >= 20
        || name.chars().any(char::is_control)
    {
        return Err(invalid(
            "expected 45 appearance bytes and 1..19 non-control UTF-16 name units",
        ));
    }
    if !(1..=9).contains(&profession)
        || appearance[27] >= 5
        || appearance[28] > 15
        || appearance[29] > 1
    {
        return Err(invalid(
            "profession or appearance scalar outside creation bounds",
        ));
    }
    let mut summary = [0; V11_SIZE];
    summary[0] = 11;
    summary[1..3].copy_from_slice(&map.to_le_bytes());
    summary[4] = 1;
    summary[5] = profession;
    summary[6..51].copy_from_slice(appearance);
    SummaryV11::parse(&summary)?;
    let mut body = BytesMut::new();
    guid.encode(&mut body);
    0u32.encode(&mut body); // Unresolved record field, synthetic zero.
    0u64.encode(&mut body);
    0u64.encode(&mut body);
    for unit in name.encode_utf16().chain(std::iter::once(0)) {
        unit.encode(&mut body);
    }
    (V11_SIZE as u16).encode(&mut body);
    body.extend_from_slice(&summary);
    Ok((summary, body))
}

fn guid(text: &str) -> io::Result<Guid> {
    if text.len() != 32 || !text.is_ascii() {
        return Err(invalid("GUID must be 32 hex digits in wire byte order"));
    }
    let mut bytes = [0; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
            .map_err(|_| invalid("invalid GUID hex"))?;
    }
    Ok(Guid(bytes))
}

fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    options.open(path)?.write_all(bytes)
}

fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [
        appearance_path,
        profession,
        map,
        name,
        identity,
        destination,
    ] = args.as_slice()
    else {
        return Err(invalid(
            "usage: gw2-character-fixture APPEARANCE_BIN PROFESSION MAP_KEY NAME GUID_WIRE_HEX NEW_OUTPUT_DIRECTORY",
        ));
    };
    let profession = profession
        .parse()
        .map_err(|_| invalid("invalid profession"))?;
    let map = map.parse().map_err(|_| invalid("invalid map Word"))?;
    let identity = guid(identity)?;
    let mut appearance = Vec::new();
    fs::File::open(appearance_path)?
        .take(46)
        .read_to_end(&mut appearance)?;
    let (summary, body) = build(&appearance, profession, map, name, identity)?;
    let mut manifest = format!(
        "build=209026\nprofession={profession}\nmap_key={map}\nlevel=1\nappearance_input={}\nsummary_bytes={}\nrecord_body_bytes={}\n",
        appearance_path,
        summary.len(),
        body.len()
    );
    manifest.push_str("record_format=Guid,Dword,Qword,Qword,terminated UTF16 name,u16-counted summary\nmessage_id_and_request_id=omitted; observer supplies correlation\nopaque_fields=synthetic zero\npreview_equipment=zero; slot mapping not inferred\nmap_lookup=caller responsibility\nappearance_source=see extraction provenance\nruntime=unverified\n");
    let destination = Path::new(destination);
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(destination)?;
    write_private(&destination.join("summary.bin"), &summary)?;
    write_private(&destination.join("record-body.bin"), &body)?;
    write_private(&destination.join("manifest.txt"), manifest.as_bytes())?;
    println!(
        "build=209026 summary_bytes={} record_body_bytes={} preview_equipment=empty runtime=unverified",
        summary.len(),
        body.len()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("character fixture: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gw2_proto::Decode;

    #[test]
    fn fixture_body_preserves_explicit_identity_appearance_and_utf16_boundaries() {
        let mut appearance = [0; 45];
        appearance[27..30].copy_from_slice(&[3, 9, 0]);
        let identity = Guid([0x5a; 16]);
        let (summary, body) = build(&appearance, 7, 379, "Mesmer 🦀", identity).unwrap();
        let parsed = SummaryV11::parse(&summary).unwrap();
        assert_eq!(
            (
                parsed.map_definition_key(),
                parsed.profession(),
                parsed.race(),
                parsed.sex()
            ),
            (379, 7, 3, 0)
        );
        assert_eq!(parsed.appearance(), appearance);
        assert!(parsed.preview_equipment().iter().all(Option::is_none));
        let mut cursor = &body[..];
        assert_eq!(Guid::decode(&mut cursor).unwrap(), identity);
        assert_eq!(u32::decode(&mut cursor).unwrap(), 0);
        assert_eq!(u64::decode(&mut cursor).unwrap(), 0);
        assert_eq!(u64::decode(&mut cursor).unwrap(), 0);
        for unit in "Mesmer 🦀".encode_utf16().chain(std::iter::once(0)) {
            assert_eq!(u16::decode(&mut cursor).unwrap(), unit);
        }
        assert_eq!(u16::decode(&mut cursor).unwrap(), 159);
        assert_eq!(cursor, summary);
    }

    #[test]
    fn fixture_rejects_unbounded_names_bad_selectors_and_appearance_extents() {
        let valid = [0; 45];
        for name in ["", "x\0y", "x\ny", &"x".repeat(20), &"🦀".repeat(10)] {
            assert!(build(&valid, 7, 379, name, Guid([0; 16])).is_err());
        }
        assert!(build(&valid, 7, 379, &"x".repeat(19), Guid([0; 16])).is_ok());
        for profession in [0, 10, 255] {
            assert!(build(&valid, profession, 379, "Fixture", Guid([0; 16])).is_err());
        }
        for size in [0, 44, 46] {
            assert!(build(&vec![0; size], 7, 379, "Fixture", Guid([0; 16])).is_err());
        }
        for (index, value) in [(27, 5), (28, 16), (29, 2)] {
            let mut bytes = valid;
            bytes[index] = value;
            assert!(build(&bytes, 7, 379, "Fixture", Guid([0; 16])).is_err());
        }
        assert_eq!(
            guid("000102030405060708090a0b0c0d0e0f").unwrap(),
            Guid(std::array::from_fn(|i| i as u8))
        );
        for text in [
            "",
            "a",
            "gg000000000000000000000000000000",
            "🦀0000000000000000000000000000",
        ] {
            assert!(guid(text).is_err());
        }
    }
}
