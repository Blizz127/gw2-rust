//! `gw2-manifest-gen --out DIR [--build N] [--root-id N] [--manifest NAME]... [--from-dat Gw2.dat] [--record-flags N] [--manifest-flags N] [--limit N]`
//!
//! Writes patcher manifests for the offline asset CDN into `DIR/<fileId>`:
//! - an `ARMF` root manifest at `--root-id`;
//! - for each `--manifest NAME[:FLAGS]`, an `MFST` manifest listed in the root under that
//!   name, at the following ids (`root-id + 1`, `+ 2`, …).
//!
//! With `--from-dat`, the first manifest lists every asset of that archive
//! (`manifest::records_from_archive`); the others stay empty.
//!
//! It prints the matching `/latest64` reply line (`build 0 0 root-id root-size`).
//! The client caches downloaded manifests in its `Gw2.dat` by file id, so use a fresh
//! `--root-id` whenever the content changes. The manifests list no records yet; they are
//! probes for what the client checks next.

use std::process::ExitCode;

use gw2_dat::archive::Archive;
use gw2_dat::manifest::{ManifestFile, Record, manifest, records_from_archive, root_manifest};

fn usage() -> ExitCode {
    eprintln!("usage: gw2-manifest-gen --out DIR [--build N] [--root-id N] [--manifest NAME]... [--from-dat Gw2.dat]");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let (mut out, mut build, mut root_id) = (None, 209026u32, 0x7f00_0001u32);
    let mut names = Vec::new();
    let mut dat = None;
    let (mut record_flags, mut manifest_flags) = (0u32, 0u32);
    let mut limit: Option<usize> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let Some(v) = args.next() else { return usage() };
        match a.as_str() {
            "--out" => out = Some(v),
            "--build" => build = v.parse().unwrap_or(build),
            "--root-id" => root_id = v.parse().unwrap_or(root_id),
            "--manifest" => names.push(v),
            "--from-dat" => dat = Some(v),
            // Experiment knobs: flags written into every record / every root manifest entry.
            "--record-flags" => record_flags = v.parse().unwrap_or(0),
            "--manifest-flags" => manifest_flags = v.parse().unwrap_or(0),
            // Keep only the first N records of the --from-dat list.
            "--limit" => limit = v.parse().ok(),
            _ => return usage(),
        }
    }
    let Some(out) = out else { return usage() };
    let write = |id: u32, bytes: &[u8]| std::fs::write(format!("{out}/{id}"), bytes);
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("create {out}: {e}");
        return ExitCode::FAILURE;
    }
    let all: Vec<Record> = match &dat {
        None => Vec::new(),
        Some(p) => match Archive::open(p) {
            Ok(a) => records_from_archive(&a)
                .into_iter()
                .take(limit.unwrap_or(usize::MAX))
                .map(|r| Record { flags: record_flags, ..r })
                .collect(),
            Err(e) => {
                eprintln!("{p}: {e}");
                return ExitCode::FAILURE;
            }
        },
    };
    let mut files = Vec::new();
    for (i, spec) in names.iter().enumerate() {
        // NAME or NAME:FLAGS (flags bit 8 marks a language manifest, REA FUN_140d91ad0).
        let (name, flags) = match spec.split_once(':') {
            Some((n, f)) => (n.to_string(), f.parse().unwrap_or(manifest_flags)),
            None => (spec.clone(), manifest_flags),
        };
        let id = root_id + 1 + i as u32;
        // The first manifest lists every asset; the others repeat its first record, since
        // a manifest without records asserts `manifest->records`.
        let m = manifest(build, if i == 0 { &all } else { &all[..all.len().min(1)] });
        if let Err(e) = write(id, &m) {
            eprintln!("write {out}/{id}: {e}");
            return ExitCode::FAILURE;
        }
        files.push(ManifestFile { base_id: id, file_id: id, size: m.len() as u32, flags, name: name.clone() });
        eprintln!("manifest {name:?}: id {id}, {} bytes", m.len());
    }
    let root = root_manifest(build, &files, &[]);
    if let Err(e) = write(root_id, &root) {
        eprintln!("write {out}/{root_id}: {e}");
        return ExitCode::FAILURE;
    }
    println!("{build} 0 0 {root_id} {}", root.len());
    ExitCode::SUCCESS
}
