//! `gw2-dat-ls <Gw2.dat> [N]` — print the entry count and the first N (default 20)
//! file ids, sorted ascending, with stored size and compression flag.

use std::process::ExitCode;

use gw2_dat::archive::Archive;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: gw2-dat-ls <Gw2.dat> [N]");
        return ExitCode::from(2);
    };
    let limit = match args.next().map(|n| n.parse::<usize>()) {
        None => 20,
        Some(Ok(n)) => n,
        Some(Err(e)) => {
            eprintln!("invalid N: {e}");
            return ExitCode::from(2);
        }
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
    for id in ids.into_iter().take(limit) {
        if let Some(entry) = archive.entry(id) {
            println!(
                "{id:>10}  stored={:>10}  compressed={}",
                entry.size,
                entry.is_compressed()
            );
        }
    }
    ExitCode::SUCCESS
}
