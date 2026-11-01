//! `gw2-manifest-gen --out DIR [--build N] [--root-id N]`
//!
//! Writes an `ARMF` root manifest for the offline asset CDN to `DIR/<root-id>` and
//! prints the matching `/latest64` reply line (`build 0 0 root-id size`). For now the
//! root manifest lists no manifests (probe for the client's next requirement).

use std::process::ExitCode;

use gw2_dat::manifest::root_manifest;

fn main() -> ExitCode {
    let (mut out, mut build, mut root_id) = (None, 209026u32, 0x7f00_0001u32);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let v = args.next();
        match (a.as_str(), v) {
            ("--out", Some(v)) => out = Some(v),
            ("--build", Some(v)) => build = v.parse().unwrap_or(build),
            ("--root-id", Some(v)) => root_id = v.parse().unwrap_or(root_id),
            _ => {
                eprintln!("usage: gw2-manifest-gen --out DIR [--build N] [--root-id N]");
                return ExitCode::from(2);
            }
        }
    }
    let Some(out) = out else {
        eprintln!("usage: gw2-manifest-gen --out DIR [--build N] [--root-id N]");
        return ExitCode::from(2);
    };
    let root = root_manifest(build, &[], &[]);
    if let Err(e) = std::fs::create_dir_all(&out).and_then(|_| std::fs::write(format!("{out}/{root_id}"), &root)) {
        eprintln!("write {out}/{root_id}: {e}");
        return ExitCode::FAILURE;
    }
    println!("{build} 0 0 {root_id} {}", root.len());
    ExitCode::SUCCESS
}
