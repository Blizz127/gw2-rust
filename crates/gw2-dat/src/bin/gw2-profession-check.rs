//! Check the verified profession-related fields in the frozen base containers.

use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    process::ExitCode,
};

use gw2_dat::{
    archive::Archive,
    content::{ContentIndex, ContentReference, resolve_reference},
    packfile::Packfile,
    profession_content::{
        ContentPointerArray209026, ProfessionDefinition209026, SpecializationDefinition209026,
        TraitDefinition209026,
    },
};

type Location = (usize, u32);

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn target(
    containers: &[ContentIndex<'_>],
    source: usize,
    reference: ContentReference,
    expected_type: u32,
) -> io::Result<Location> {
    let record = resolve_reference(containers, source, reference, 0x14)?;
    let actual = u32::from_le_bytes(record.bytes[0x10..0x14].try_into().unwrap());
    if actual != expected_type {
        return Err(invalid("catalog reference has wrong target content type"));
    }
    Ok((
        record.container_index,
        u32::try_from(record.offset)
            .map_err(|_| invalid("catalog target offset exceeds index width"))?,
    ))
}

fn members(
    containers: &[ContentIndex<'_>],
    source: usize,
    array: ContentPointerArray209026,
    expected_type: u32,
) -> io::Result<Vec<Location>> {
    // Both catalog arrays have exactly nine entries in frozen build 209026.
    if array.count != 9 {
        return Err(invalid("expected nine frozen catalog choices"));
    }
    let array = resolve_reference(
        containers,
        source,
        array
            .reference
            .ok_or_else(|| invalid("missing catalog array"))?,
        9 * 8,
    )?;
    let mut result = Vec::with_capacity(9);
    for i in 0..9 {
        let slot = u32::try_from(array.offset + i * 8)
            .map_err(|_| invalid("catalog pointer slot exceeds index width"))?;
        let reference = containers[array.container_index]
            .reference(slot)?
            .ok_or_else(|| invalid("null catalog member"))?;
        result.push(target(
            containers,
            array.container_index,
            reference,
            expected_type,
        )?);
    }
    Ok(result)
}

fn run(path: &str) -> io::Result<()> {
    let archive = Archive::open(path)?;
    let packfiles: Vec<_> = (1282830..=1282861)
        .map(|id| archive.read(id).and_then(Packfile::new))
        .collect::<io::Result<_>>()?;
    let containers: Vec<_> = packfiles
        .iter()
        .map(ContentIndex::parse)
        .collect::<io::Result<_>>()?;
    let mut professions = BTreeSet::new();
    let mut specializations = BTreeSet::new();
    let mut traits = BTreeSet::new();
    let mut specialization_uids = BTreeSet::new();
    let mut trait_uids = BTreeSet::new();
    let mut specialization_references = 0u64;
    let mut trait_references = 0u64;
    let mut profession_records = Vec::new();
    let mut specialization_records = BTreeMap::new();
    let mut trait_records = BTreeMap::new();
    let mut progress_records = BTreeSet::new();
    for (container, index) in containers.iter().enumerate() {
        let file_id = 1282830 + container;
        for entry in index.entries() {
            let location = (container, entry.offset);
            let unique = match entry.content_type {
                315 => {
                    let record = ProfessionDefinition209026::read(index, entry.offset)?;
                    specialization_references += u64::from(record.specializations.count);
                    profession_records.push((container, record));
                    professions.insert(record.profession)
                }
                386 => {
                    let record = SpecializationDefinition209026::read(index, entry.offset)?;
                    trait_references += u64::from(record.major_traits.count);
                    specialization_records.insert(location, record);
                    record.uid != 0
                        && specialization_uids.insert(record.uid)
                        && specializations.insert(record.key)
                }
                77 => {
                    let record = TraitDefinition209026::read(index, entry.offset)?;
                    trait_records.insert(location, record);
                    record.uid != 0 && trait_uids.insert(record.uid) && traits.insert(record.key)
                }
                353 => {
                    progress_records.insert(location);
                    continue;
                }
                _ => continue,
            };
            if !unique {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "duplicate type-{} key/enum/UID or zero UID in file {file_id}",
                        entry.content_type
                    ),
                ));
            }
        }
    }
    let mut owned_specializations = BTreeSet::new();
    let mut guardian = Vec::new();
    for (container, profession) in profession_records {
        for location in members(&containers, container, profession.specializations, 386)? {
            let specialization = specialization_records
                .get(&location)
                .ok_or_else(|| invalid("specialization reference is not an indexed record"))?;
            if !owned_specializations.insert(specialization.key) {
                return Err(invalid(
                    "specialization belongs to multiple profession entries",
                ));
            }
            if profession.profession == 1 {
                guardian.push((specialization.key, specialization.uid));
            }
        }
    }
    if owned_specializations != specializations {
        return Err(invalid(
            "profession membership does not cover specialization catalog",
        ));
    }
    let mut progress_references = 0;
    for ((container, _), specialization) in specialization_records {
        let mut tiers = [0; 3];
        let mut keys = BTreeSet::new();
        for location in members(&containers, container, specialization.major_traits, 77)? {
            let record = trait_records
                .get(&location)
                .ok_or_else(|| invalid("trait reference is not an indexed record"))?;
            if !(1..=3).contains(&record.tier) || !keys.insert(record.key) {
                return Err(invalid("invalid or duplicate major-trait choice"));
            }
            tiers[(record.tier - 1) as usize] += 1;
        }
        if tiers != [3, 3, 3] {
            return Err(invalid("major-trait choices do not have three per tier"));
        }
        if let Some(reference) = specialization.progress_reference {
            let location = target(&containers, container, reference, 353)?;
            if !progress_records.contains(&location) {
                return Err(invalid("progress reference is not an indexed record"));
            }
            progress_references += 1;
        }
    }
    println!(
        "checked 32 base containers: professions={} specializations={} traits={} specialization_refs={specialization_references} major_trait_refs={trait_references}",
        professions.len(),
        specializations.len(),
        traits.len()
    );
    println!(
        "resolved all membership/trait targets; exact profession coverage; three choices per tier; progress_refs={progress_references}"
    );
    println!(
        "Guardian specialization (data key, wire UID): {guardian:?}; gameplay eligibility not validated"
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 || args[0].starts_with('-') {
        eprintln!("usage: gw2-profession-check <frozen-209026-Gw2.dat>");
        return ExitCode::from(2);
    }
    match run(&args[0]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("profession content check failed: {error}");
            ExitCode::FAILURE
        }
    }
}
