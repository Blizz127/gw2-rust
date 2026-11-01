//! Check the verified profession-related fields in the frozen base containers.

use std::{collections::BTreeSet, io, process::ExitCode};

use gw2_dat::{
    archive::Archive,
    content::ContentIndex,
    packfile::Packfile,
    profession_content::{
        ProfessionDefinition209026, SpecializationDefinition209026, TraitDefinition209026,
    },
};

fn run(path: &str) -> io::Result<()> {
    let archive = Archive::open(path)?;
    let mut professions = BTreeSet::new();
    let mut specializations = BTreeSet::new();
    let mut traits = BTreeSet::new();
    let mut specialization_references = 0u64;
    let mut trait_references = 0u64;
    for file_id in 1282830..=1282861 {
        let packfile = Packfile::new(archive.read(file_id)?)?;
        let index = ContentIndex::parse(&packfile)?;
        for entry in index.entries() {
            let unique = match entry.content_type {
                315 => {
                    let record = ProfessionDefinition209026::read(&index, entry.offset)?;
                    specialization_references += u64::from(record.specializations.count);
                    professions.insert(record.profession)
                }
                386 => {
                    let record = SpecializationDefinition209026::read(&index, entry.offset)?;
                    trait_references += u64::from(record.major_traits.count);
                    specializations.insert(record.key)
                }
                77 => {
                    let record = TraitDefinition209026::read(&index, entry.offset)?;
                    traits.insert(record.key)
                }
                _ => continue,
            };
            if !unique {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "duplicate type-{} key/enum in file {file_id}",
                        entry.content_type
                    ),
                ));
            }
        }
    }
    println!(
        "checked 32 base containers: professions={} specializations={} traits={} specialization_refs={specialization_references} major_trait_refs={trait_references}",
        professions.len(),
        specializations.len(),
        traits.len()
    );
    println!(
        "verified record fields and local array extents; external targets and eligibility not validated"
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
