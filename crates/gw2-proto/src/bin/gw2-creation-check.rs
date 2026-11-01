//! Metadata-only inspection of one complete build-209026 creation frame.
use std::{
    fs::File,
    io::{self, Read},
};

use gw2_proto::character_creation::{CreationRequest209026, MAX_FRAME_SIZE};

fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: gw2-creation-check RAW_FRAME",
        ));
    }
    let mut bytes = Vec::with_capacity(MAX_FRAME_SIZE + 1);
    File::open(&args[0])?
        .take((MAX_FRAME_SIZE + 1) as u64)
        .read_to_end(&mut bytes)?;
    let request = CreationRequest209026::parse(&bytes)?;
    println!(
        "build=209026 id=415 bytes={} structural_checks=passed",
        bytes.len()
    );
    println!(
        "context_scalar_checks={}",
        if request.passes_context_scalar_checks() {
            "passed"
        } else {
            "failed"
        }
    );
    println!(
        "name_policy=unverified appearance_content=unverified answers=unverified colors=unverified creation=unverified"
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("creation check: {error}");
        std::process::exit(1);
    }
}
