//! Extract a frozen build-209026 creation appearance with provenance.
//! Does not select a runtime loadout or map equipment slots into a summary.

use gw2_dat::{
    appearance::{AppearanceDefault209026, DEFAULT_TEXT_LIMIT},
    archive::Archive,
    content::{ContentIndex, ContentReference, resolve_reference},
    creation_equipment_content::CreationEquipmentReferences209026,
    packfile::Packfile,
};
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::Path,
};

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
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
    let [archive_path, race, sex, profession, destination] = args.as_slice() else {
        return Err(invalid(
            "usage: gw2-appearance-extract GW2_DAT RACE SEX PROFESSION NEW_OUTPUT_DIRECTORY",
        ));
    };
    let number = |s: &str| {
        s.parse::<u8>()
            .map_err(|_| invalid("invalid numeric selector"))
    };
    let (race, sex, profession) = (number(race)?, number(sex)?, number(profession)?);
    if race >= 5 || sex >= 2 || !(1..=9).contains(&profession) {
        return Err(invalid("race must be 0..4, sex 0..1 and profession 1..9"));
    }
    // Build-specific registry/locations: docs/re/209026-content-registry.md.
    // Offsets are admitted only for the exact archived source digest below.
    let key = 51 + u32::from(race) * 2 - u32::from(sex);
    let offsets = [
        129824u32, 136984, 159064, 98128, 120936, 105088, 143984, 113352, 151584, 90976,
    ];
    let offset = offsets[(key - 50) as usize];
    let archive = Archive::open(archive_path)?;
    let mut hashes = Vec::new();
    let packs = (1282830..=1282861)
        .map(|id| {
            let bytes = archive.read(id)?;
            hashes.push(format!("{:x}", Sha256::digest(&bytes)));
            Packfile::new(bytes)
        })
        .collect::<io::Result<Vec<_>>>()?;
    if hashes[22] != "54de38b4b7c4fdf69c18154fb254a3f5f346c1b031b68a8416153c4f751ac5d3" {
        return Err(invalid(
            "appearance source differs from frozen build 209026",
        ));
    }
    let containers = packs
        .iter()
        .map(ContentIndex::parse)
        .collect::<io::Result<Vec<_>>>()?;
    let source = &containers[22];
    if source
        .entries()
        .filter(|entry| entry.content_type == 137 && entry.offset == offset)
        .count()
        != 1
    {
        return Err(invalid("selected appearance record missing or ambiguous"));
    }
    let record = source
        .content_bytes()
        .get(offset as usize..offset as usize + 0x84)
        .ok_or_else(|| invalid("short appearance record"))?;
    let count = u32::from_le_bytes(record[0x80..0x84].try_into().unwrap()) as usize;
    if count == 0 || count > 1000 {
        return Err(invalid("appearance default count outside limit"));
    }
    let reference = source
        .reference(offset + 0x78)?
        .ok_or_else(|| invalid("missing appearance defaults"))?;
    let rows = resolve_reference(&containers, 22, reference, count * 24)?;
    let mut selected = None;
    for row in 0..count {
        let fields = &rows.bytes[row * 24..row * 24 + 24];
        let candidate = u32::from_le_bytes(fields[16..20].try_into().unwrap());
        if candidate != u32::from(profession) {
            continue;
        }
        let slot = u32::try_from(rows.offset + row * 24)
            .map_err(|_| invalid("appearance row offset overflow"))?;
        let Some(ContentReference::String { index }) =
            containers[rows.container_index].reference(slot)?
        else {
            return Err(invalid("appearance default is not a classified string"));
        };
        let text = containers[rows.container_index].string(index, DEFAULT_TEXT_LIMIT)?;
        let parsed = AppearanceDefault209026::parse(text.as_bytes(), race, sex)?;
        if selected.replace((parsed, slot, index)).is_some() {
            return Err(invalid("multiple defaults for selected profession"));
        }
    }
    let (default, string_slot, string_index) =
        selected.ok_or_else(|| invalid("profession default absent"))?;
    let refs = CreationEquipmentReferences209026::read(source, offset)?;
    let templates = refs.templates(&containers, 22, 1000)?;
    let matching: Vec<_> = templates
        .iter()
        .filter(|t| t.value.level == 1 && t.value.profession == u32::from(profession))
        .collect();
    if matching.len() != 1 {
        return Err(invalid("level-1 equipment template absent or ambiguous"));
    }
    let template = matching[0];
    let mut used = BTreeSet::from([22, rows.container_index, template.source.container_index]);
    let mut provenance = format!(
        "build=209026\nrace={race}\nsex={sex}\nprofession={profession}\nappearance_key={key}\nappearance_file=1282852\nappearance_offset={offset}\nstring_file={}\nstring_slot={string_slot}\nstring_index={string_index}\nappearance_sha256={:x}\ncolor_rows={}\ntemplate_file={}\ntemplate_offset={}\ntemplate_uid={}\n",
        1282830 + rows.container_index,
        Sha256::digest(default.appearance()),
        default.colors().len(),
        1282830 + template.source.container_index,
        template.source.offset,
        template.value.uid
    );
    for entry in template
        .value
        .entries(&containers, template.source.container_index, 256)?
    {
        used.insert(entry.source.container_index);
        let item = entry
            .value
            .item(&containers, entry.source.container_index)?
            .ok_or_else(|| invalid("template item absent"))?;
        used.insert(item.source.container_index);
        provenance.push_str(&format!(
            "item={} slot={} kind={} entry_file={} entry_offset={} item_file={} item_offset={}\n",
            item.value.key,
            entry.value.equipment_slot,
            item.value.kind,
            1282830 + entry.source.container_index,
            entry.source.offset,
            1282830 + item.source.container_index,
            item.source.offset
        ));
    }
    for index in used {
        provenance.push_str(&format!(
            "source_file={} sha256={}\n",
            1282830 + index,
            hashes[index]
        ));
    }
    provenance.push_str("appearance=client archived default with traced importer adjustments\nequipment=reference metadata only; no summary slot mapping or runtime loadout\nruntime_selection=unverified\n");
    // All extraction/validation happens before creating an output directory.
    let destination = Path::new(destination);
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(destination)?;
    write_private(&destination.join("appearance.bin"), default.appearance())?;
    let colors: Vec<u8> = default
        .colors()
        .iter()
        .flatten()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    write_private(&destination.join("colors-le-u32.bin"), &colors)?;
    write_private(&destination.join("provenance.txt"), provenance.as_bytes())?;
    println!(
        "build=209026 race={race} sex={sex} profession={profession} appearance_bytes=45 template_reference=checked runtime=unverified"
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("appearance extract: {error}");
        std::process::exit(1);
    }
}
