//! Metadata-only structural check of a raw build-209026 version-11 summary.
use std::{
    fs::File,
    io::{self, Read},
};

use gw2_proto::character_summary::{SummaryV11, V11_SIZE};

fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: gw2-summary-check RAW_SUMMARY",
        ));
    }
    let mut bytes = Vec::with_capacity(V11_SIZE + 1);
    File::open(&args[0])?
        .take((V11_SIZE + 1) as u64)
        .read_to_end(&mut bytes)?;
    let summary = SummaryV11::parse(&bytes)?;
    println!("build=209026 version=11 bytes={V11_SIZE} structural_checks=passed");
    println!(
        "preview_sex_check={}",
        if summary.passes_preview_sex_check() {
            "passed"
        } else {
            "failed"
        }
    );
    println!("map_lookup=unverified appearance_content=unverified preview=unverified");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("summary check: {error}");
        std::process::exit(1);
    }
}
