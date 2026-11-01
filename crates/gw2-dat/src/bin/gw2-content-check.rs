//! Inspect verified content-index structure without printing record payloads.

use std::{collections::BTreeMap, process::ExitCode};

use gw2_dat::{archive::Archive, content::ContentIndex, packfile::Packfile};

const USAGE: &str = "usage: gw2-content-check <Gw2.dat> <file_id>...|--base-209026 [--type N]";

fn run() -> Result<(), (u8, String)> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .filter(|p| !p.starts_with("--"))
        .ok_or_else(|| (2, USAGE.to_owned()))?;
    let mut ids = Vec::new();
    let mut base = false;
    let mut wanted_type = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--base-209026" if !base && ids.is_empty() => base = true,
            "--type" if wanted_type.is_none() => {
                wanted_type = Some(
                    args.next()
                        .and_then(|n| n.parse::<u32>().ok())
                        .filter(|&n| n < 483)
                        .ok_or_else(|| {
                            (
                                2,
                                "--type requires an integer from 0 through 482".to_owned(),
                            )
                        })?,
                );
            }
            _ if !base && !arg.starts_with('-') && ids.len() < 32 => {
                let id = arg
                    .parse::<u32>()
                    .ok()
                    .filter(|&n| n != 0)
                    .ok_or_else(|| (2, USAGE.to_owned()))?;
                if ids.contains(&id) {
                    return Err((2, "duplicate file id".to_owned()));
                }
                ids.push(id);
            }
            _ => return Err((2, USAGE.to_owned())),
        }
    }
    if base {
        ids.extend(1282830..=1282861);
    }
    if ids.is_empty() {
        return Err((2, USAGE.to_owned()));
    }
    let archive = Archive::open(&path).map_err(|e| (1, format!("archive open failed: {e}")))?;
    let mut total_records = 0usize;
    let mut totals = BTreeMap::<u32, usize>::new();
    for id in &ids {
        let pf = archive
            .read(*id)
            .and_then(Packfile::new)
            .map_err(|e| (1, format!("file {id}: {e}")))?;
        let index = ContentIndex::parse(&pf).map_err(|e| (1, format!("file {id}: {e}")))?;
        let mut counts = BTreeMap::<u32, usize>::new();
        for entry in index.entries() {
            total_records += 1;
            if wanted_type.is_none_or(|wanted| entry.content_type == wanted) {
                *counts.entry(entry.content_type).or_default() += 1;
                *totals.entry(entry.content_type).or_default() += 1;
            }
        }
        println!(
            "file {id}: index ok; records={} type_metadata={} content_bytes={} flags={:#x}",
            index.entries().len(),
            index.type_count(),
            index.content_bytes().len(),
            index.flags()
        );
        if let Some(wanted) = wanted_type {
            println!(
                "  type {wanted}: {} records",
                counts.get(&wanted).copied().unwrap_or(0)
            );
        } else {
            for (ty, count) in counts {
                println!("  type {ty}: {count} records");
            }
        }
    }
    println!(
        "checked {} containers, {total_records} index records; record payload validity untested",
        ids.len()
    );
    if let Some(wanted) = wanted_type {
        println!(
            "total type {wanted}: {} records",
            totals.get(&wanted).copied().unwrap_or(0)
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("{message}");
            ExitCode::from(code)
        }
    }
}
