//! `gw2-dat-ls <Gw2.dat> [N] [--sample M]`
//!
//! Prints the entry count and the first N (default 20) file ids, sorted ascending,
//! with stored size and compression flag.
//!
//! `--sample M` also reads M compressed and M uncompressed entries through
//! `Archive::read` (block-trailer stripping + decompressor). For each one it prints
//! ok/error, the output length and the first 4 bytes, so real format magics (`PF`,
//! `ATEX`, ...) show whether the layout guesses hold on real data.

use std::process::ExitCode;

use gw2_dat::archive::Archive;

fn main() -> ExitCode {
    let mut limit = 20usize;
    let mut sample = 0usize;
    let mut path = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let parsed = match a.as_str() {
            "--sample" => args.next().map(|n| n.parse().map(|n| sample = n)),
            _ if path.is_none() => {
                path = Some(a);
                continue;
            }
            _ => Some(a.parse().map(|n| limit = n)),
        };
        if !matches!(parsed, Some(Ok(()))) {
            eprintln!("usage: gw2-dat-ls <Gw2.dat> [N] [--sample M]");
            return ExitCode::from(2);
        }
    }
    let Some(path) = path else {
        eprintln!("usage: gw2-dat-ls <Gw2.dat> [N] [--sample M]");
        return ExitCode::from(2);
    };

    let archive = match Archive::open(&path) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("failed to open {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("entries: {}", archive.len());
    let mut ids: Vec<u32> = archive.file_ids().collect();
    ids.sort_unstable();
    for &id in ids.iter().take(limit) {
        if let Some(entry) = archive.entry(id) {
            println!("{id:>10}  stored={:>10}  compressed={}", entry.size, entry.is_compressed());
        }
    }

    if sample == 0 {
        return ExitCode::SUCCESS;
    }
    let mut failures = 0usize;
    for want_compressed in [true, false] {
        println!("-- sample: compressed={want_compressed}");
        let picks = ids
            .iter()
            .copied()
            .filter(|id| archive.entry(*id).is_some_and(|e| e.is_compressed() == want_compressed))
            .take(sample);
        for id in picks {
            let stored = archive.entry(id).map_or(0, |e| e.size);
            match archive.read(id) {
                Ok(bytes) => {
                    let head: Vec<String> =
                        bytes.iter().take(4).map(|b| format!("{b:02x}")).collect();
                    let ascii: String = bytes
                        .iter()
                        .take(4)
                        .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
                        .collect();
                    println!(
                        "{id:>10}  stored={stored:>10}  out={:>10}  head={} {ascii:?}",
                        bytes.len(),
                        head.join(" ")
                    );
                }
                Err(e) => {
                    failures += 1;
                    println!("{id:>10}  stored={stored:>10}  ERROR {e}");
                }
            }
        }
    }
    println!("sample failures: {failures}");
    if failures == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
