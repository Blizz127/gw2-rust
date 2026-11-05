//! Read-only build-209026 palette/row/entry/skill-reference check.

use gw2_dat::{
    archive::Archive,
    content::{ContentIndex, resolve_reference},
    packfile::Packfile,
    setting_content::IntegerSettingDefinition209026,
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
fn run(path: &str, profession: u32, palette_keys: &[u32]) -> io::Result<()> {
    let archive = Archive::open(path)?;
    let packfiles: Vec<_> = (1282830..=1282861)
        .map(|id| archive.read(id).and_then(Packfile::new))
        .collect::<io::Result<_>>()?;
    let containers: Vec<_> = packfiles
        .iter()
        .map(ContentIndex::parse)
        .collect::<io::Result<_>>()?;
    // Frozen descriptor GUID words, in standard-slot order; see the RE note.
    let slot_guids = [
        [0x2903742du32, 0x452437c6, 0x84f9cea1, 0x7f2159c7],
        [0xd5ae524d, 0x4ab59f15, 0x0e9107bf, 0x5b0f04a6],
        [0xec8901ab, 0x4527a16e, 0xa7ddd6bd, 0x3641dcd1],
        [0x7a36e8bd, 0x454e62cf, 0x98cfd989, 0x7d6f811f],
        [0x84450e1b, 0x45bfa978, 0xa4c77482, 0x502a4462],
    ]
    .map(|words| {
        let mut guid = [0u8; 16];
        for (part, word) in guid.as_chunks_mut::<4>().0.iter_mut().zip(words) {
            *part = word.to_le_bytes();
        }
        guid
    });
    let mut slot_levels = [None; 5];
    let mut keys = BTreeSet::new();
    let mut uids = BTreeSet::new();
    let mut rows = 0usize;
    let mut entries = 0usize;
    let mut profession_rows = 0usize;
    let mut candidates = BTreeMap::<u32, Vec<(u32, u32, u32, u32)>>::new();
    for (container, index) in containers.iter().enumerate() {
        for indexed in index.entries() {
            if indexed.content_type == 150 {
                let start = indexed.offset as usize;
                let guid = start
                    .checked_add(16)
                    .and_then(|end| index.content_bytes().get(start..end))
                    .ok_or_else(|| invalid("short setting GUID"))?;
                if let Some(slot) = slot_guids.iter().position(|wanted| wanted == guid) {
                    let setting = IntegerSettingDefinition209026::read(index, indexed.offset)?;
                    if setting.value < 0 || slot_levels[slot].replace(setting.value).is_some() {
                        return Err(invalid("negative or duplicate standard-slot setting"));
                    }
                }
                continue;
            }
            if indexed.content_type != 65 {
                continue;
            }
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
                profession_rows += usize::from(row.profession == profession);
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
                    if row.profession == profession && palette_keys.contains(&palette.key) {
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
    if slot_levels.iter().any(Option::is_none) {
        return Err(invalid("missing frozen standard-slot setting"));
    }
    println!("standard-slot setting levels (slots 0–4): {slot_levels:?}");
    println!(
        "checked 32 base containers: palettes={} rows={rows} entries={entries} resolved_type64_skills={entries} profession={profession} profession_rows={profession_rows}",
        keys.len()
    );
    println!(
        "Profession {profession} candidates: palette key -> (skill key, minimum level, raw mode, skill flags): {candidates:?}"
    );
    for key in palette_keys {
        if !candidates.contains_key(key) {
            return Err(invalid("requested palette has no entry for the selected profession"));
        }
    }
    println!("record/reference bounds checked; gameplay eligibility and runtime state untested");
    Ok(())
}
fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() || args.len() == 2 || args[0].starts_with('-') {
        eprintln!("usage: gw2-skill-palette-check <build-209026 Gw2.dat> [profession-1..9 palette-key ...]");
        return ExitCode::from(2);
    }
    let (profession, keys) = if args.len() == 1 {
        (1, vec![1, 2, 4, 14, 21, 127])
    } else {
        let profession = args[1].parse::<u32>().ok().filter(|n| (1..=9).contains(n));
        let keys = args[2..].iter().map(|s| s.parse::<u32>()).collect::<Result<Vec<_>, _>>();
        match (profession, keys) {
            (Some(profession), Ok(keys)) if keys.iter().all(|k| *k != 0) => (profession, keys),
            _ => {
                eprintln!("profession must be 1..=9 and palette keys must be positive Dwords");
                return ExitCode::from(2);
            }
        }
    };
    match run(&args[0], profession, &keys) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("palette check failed: {error}");
            ExitCode::FAILURE
        }
    }
}
