//! Read-only build-209026 palette/row/entry/skill-reference check.

use gw2_dat::{
    archive::Archive,
    content::{ContentIndex, resolve_reference},
    packfile::Packfile,
    skill_palette_content::{
        SkillDefinition209026, SkillPaletteDefinition209026, SkillPaletteEntry209026,
        SkillPaletteRow209026,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    process::ExitCode,
};

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn offset(start: usize, i: u32, stride: usize) -> io::Result<u32> {
    let relative = (i as usize)
        .checked_mul(stride)
        .ok_or_else(|| invalid("palette array offset overflow"))?;
    start
        .checked_add(relative)
        .and_then(|at| u32::try_from(at).ok())
        .ok_or_else(|| invalid("palette offset exceeds index width"))
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
    let mut keys = BTreeSet::new();
    let mut uids = BTreeSet::new();
    let mut rows = 0usize;
    let mut entries = 0usize;
    let mut guardian_rows = 0usize;
    let mut candidates = BTreeMap::<u32, Vec<(u32, u32, u32, u32)>>::new();
    for (container, index) in containers.iter().enumerate() {
        for indexed in index.entries().filter(|e| e.content_type == 65) {
            let palette = SkillPaletteDefinition209026::read(index, indexed.offset)?;
            if !keys.insert(palette.key) || !uids.insert(palette.uid) {
                return Err(invalid("duplicate palette key or UID"));
            }
            let Some(row_data) = palette.rows.resolve(&containers, container, 100_000)? else {
                continue;
            };
            for i in 0..palette.rows.count {
                let row = SkillPaletteRow209026::read(
                    &containers[row_data.container_index],
                    offset(row_data.offset, i, palette.rows.stride())?,
                )?;
                rows += 1;
                guardian_rows += usize::from(row.profession == 1);
                let Some(entry_data) =
                    row.entries
                        .resolve(&containers, row_data.container_index, 100_000)?
                else {
                    continue;
                };
                for j in 0..row.entries.count {
                    let entry = SkillPaletteEntry209026::read(
                        &containers[entry_data.container_index],
                        offset(entry_data.offset, j, row.entries.stride())?,
                    )?;
                    let target = resolve_reference(
                        &containers,
                        entry_data.container_index,
                        entry.skill.ok_or_else(|| invalid("null palette skill"))?,
                        0x3c,
                    )?;
                    let skill = SkillDefinition209026::read(
                        &containers[target.container_index],
                        u32::try_from(target.offset)
                            .map_err(|_| invalid("skill offset exceeds index width"))?,
                    )?;
                    entries += 1;
                    if row.profession == 1 && [1, 2, 4, 14, 21, 127].contains(&palette.key) {
                        candidates.entry(palette.key).or_default().push((
                            skill.key,
                            entry.minimum_level,
                            entry.context_mode,
                            skill.flags,
                        ));
                    }
                }
            }
        }
    }
    println!(
        "checked 32 base containers: palettes={} rows={rows} entries={entries} resolved_type64_skills={entries} guardian_rows={guardian_rows}",
        keys.len()
    );
    println!(
        "Guardian candidates: palette key -> (skill key, minimum level, raw mode, skill flags): {candidates:?}"
    );
    println!("record/reference bounds checked; gameplay eligibility and runtime state untested");
    Ok(())
}
fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 || args[0].starts_with('-') {
        eprintln!("usage: gw2-skill-palette-check <build-209026 Gw2.dat>");
        return ExitCode::from(2);
    }
    match run(&args[0]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("palette check failed: {error}");
            ExitCode::FAILURE
        }
    }
}
