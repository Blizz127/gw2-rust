//! `gw2-dat-pf <Gw2.dat> <file_id>... [--schema packfiles.xml] [--depth N]`
//!
//! Reads archive entries and, if they are packfiles, decodes each chunk with the
//! matching schema version (default: the vendored 208039 reference schema). It
//! prints a size-capped tree. This is the first real-data check of `pfread`.

use std::process::ExitCode;

use gw2_dat::archive::Archive;
use gw2_dat::packfile::Packfile;
use gw2_dat::pfread::{Limits, PfValue, read_chunk};
use gw2_dat::pfschema::{find_all, parse};

const REFERENCE: &str = include_str!("../../reference/packfiles-208039.xml");

fn fourcc(b: [u8; 4]) -> String {
    String::from_utf8_lossy(&b).trim_end_matches('\0').to_string()
}

fn show(v: &PfValue, indent: usize, depth: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    match v {
        PfValue::Struct(fields) if depth > 0 => {
            for (n, f) in fields {
                match f {
                    PfValue::Struct(_) | PfValue::Array(_) => {
                        out.push_str(&format!("{pad}{n}:\n"));
                        show(f, indent + 1, depth - 1, out);
                    }
                    _ => out.push_str(&format!("{pad}{n}: {f:?}\n")),
                }
            }
        }
        PfValue::Array(items) if depth > 0 => {
            out.push_str(&format!("{pad}[{} items]\n", items.len()));
            for (i, it) in items.iter().take(3).enumerate() {
                out.push_str(&format!("{pad}- [{i}]\n"));
                show(it, indent + 1, depth - 1, out);
            }
        }
        PfValue::Array(items) => out.push_str(&format!("{pad}[{} items]\n", items.len())),
        PfValue::Struct(f) => out.push_str(&format!("{pad}{{{} fields}}\n", f.len())),
        other => out.push_str(&format!("{pad}{other:?}\n")),
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut path = None;
    let mut ids = Vec::new();
    let mut schema_path = None;
    let mut depth = 3usize;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--schema" => schema_path = args.next(),
            "--depth" => depth = args.next().and_then(|d| d.parse().ok()).unwrap_or(3),
            _ if path.is_none() => path = Some(a),
            _ => match a.parse::<u32>() {
                Ok(id) => ids.push(id),
                Err(_) => {
                    eprintln!("bad file id {a}");
                    return ExitCode::from(2);
                }
            },
        }
    }
    let (Some(path), false) = (path, ids.is_empty()) else {
        eprintln!("usage: gw2-dat-pf <Gw2.dat> <file_id>... [--schema packfiles.xml] [--depth N]");
        return ExitCode::from(2);
    };
    let xml = match &schema_path {
        Some(p) => match std::fs::read_to_string(p) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{p}: {e}");
                return ExitCode::FAILURE;
            }
        },
        None => REFERENCE.to_string(),
    };
    let schemas = match parse(&xml) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let archive = match Archive::open(&path) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("failed to open {path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut failures = 0;
    for id in ids {
        let bytes = match archive.read(id) {
            Ok(b) => b,
            Err(e) => {
                println!("file {id}: read error {e}");
                failures += 1;
                continue;
            }
        };
        let pf = match Packfile::new(bytes) {
            Ok(p) => p,
            Err(e) => {
                println!("file {id}: not a packfile ({e})");
                continue;
            }
        };
        let ty = fourcc(pf.r#type());
        println!("file {id}: PF type {ty:?} ptr64={}", pf.is_64bit());
        for chunk in pf.chunks() {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    println!("  chunk error: {e}");
                    failures += 1;
                    break;
                }
            };
            let name = fourcc(chunk.name());
            let ver = chunk.version() as u32;
            print!("  chunk {name:?} v{ver} ({} bytes): ", chunk.bytes().len());
            let candidates: Vec<_> =
                find_all(&schemas, &ty, &name).into_iter().filter_map(|c| c.version(ver)).collect();
            if candidates.is_empty() {
                println!("no schema");
                continue;
            }
            // Ambiguous names: the first candidate that decodes wins.
            let mut errors = Vec::new();
            let mut decoded = None;
            for vs in &candidates {
                match read_chunk(chunk.bytes(), vs, pf.is_64bit(), Limits::default()) {
                    Ok(v) => {
                        decoded = Some((vs.type_name.clone(), v));
                        break;
                    }
                    Err(e) => errors.push(format!("{}: {e}", vs.type_name)),
                }
            }
            match decoded {
                Some((ty_name, v)) => {
                    let note = if candidates.len() > 1 { format!(" (of {} candidates)", candidates.len()) } else { String::new() };
                    println!("{ty_name} ok{note}");
                    let mut s = String::new();
                    show(&v, 2, depth, &mut s);
                    print!("{s}");
                }
                None => {
                    println!("FAILED: {}", errors.join("; "));
                    failures += 1;
                }
            }
        }
    }
    if failures == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
