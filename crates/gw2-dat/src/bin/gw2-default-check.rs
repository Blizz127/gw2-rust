//! Check one privately extracted appearance-default string without printing it.

use std::io::Read;
use std::process::ExitCode;

use gw2_dat::appearance::{AppearanceDefault209026, DEFAULT_TEXT_LIMIT};

fn run() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [path, race, sex] = args.as_slice() else {
        return Err(std::io::Error::other(
            "usage: gw2-default-check FILE RACE SEX",
        ));
    };
    let number = |s: &std::ffi::OsStr| -> std::io::Result<u8> {
        s.to_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| std::io::Error::other("invalid race/sex argument"))
    };
    let (race, sex) = (number(race)?, number(sex)?);
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(DEFAULT_TEXT_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)?;
    let default = AppearanceDefault209026::parse(&bytes, race, sex)?;
    println!(
        "build 209026: parsed default; 45 appearance bytes, {} supplied color rows; runtime validity untested",
        default.colors().len()
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
