//! `gw2-dat-cat <Gw2.dat> <file_id>`
//!
//! Writes the decoded (decompressed) contents of one archive entry to stdout.

use std::io::Write;
use std::process::ExitCode;

use gw2_dat::archive::Archive;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [path, id] = &args[..] else {
        eprintln!("usage: gw2-dat-cat <Gw2.dat> <file_id>");
        return ExitCode::from(2);
    };
    let Ok(id) = id.parse::<u32>() else {
        eprintln!("bad file id {id}");
        return ExitCode::from(2);
    };
    let bytes = match Archive::open(path).and_then(|a| a.read(id)) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{path} #{id}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if std::io::stdout().write_all(&bytes).is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
