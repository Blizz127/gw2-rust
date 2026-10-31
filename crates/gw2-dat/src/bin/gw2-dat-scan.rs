//! `gw2-dat-scan <Gw2.dat> [--type TYPE] [--limit N] [--map-rects]`
//!
//! Reads only the head of every entry (`Archive::read_head`) and prints a
//! histogram of file types: the PF packfile type for packfiles, else the first 4
//! bytes. With `--type`, also lists the ids of that type with decoded sizes.
//! `--map-rects` instead prints `file_id<TAB>x0<TAB>y0<TAB>x1<TAB>y1` for every
//! `mapc` packfile, from its `parm` chunk (`MapParam.rect`, the same units as the
//! web API `map_rect`).

use std::collections::BTreeMap;
use std::process::ExitCode;

use gw2_dat::archive::Archive;
use gw2_dat::packfile::Packfile;

fn label(head: &[u8]) -> String {
    let tag = |b: &[u8]| -> String {
        b.iter().map(|&c| if c.is_ascii_graphic() { c as char } else { '.' }).collect()
    };
    if head.len() >= 12 && &head[..2] == b"PF" {
        format!("PF:{}", tag(&head[8..12]).trim_end_matches('.'))
    } else {
        tag(&head[..head.len().min(4)])
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut path = None;
    let mut want = None;
    let mut limit = 50usize;
    let mut map_rects = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--type" => want = args.next(),
            "--limit" => limit = args.next().and_then(|n| n.parse().ok()).unwrap_or(50),
            "--map-rects" => map_rects = true,
            _ => path = Some(a),
        }
    }
    let Some(path) = path else {
        eprintln!("usage: gw2-dat-scan <Gw2.dat> [--type TYPE] [--limit N]");
        return ExitCode::from(2);
    };
    let archive = match Archive::open(&path) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut ids: Vec<u32> = archive.file_ids().collect();
    ids.sort_unstable();
    if map_rects {
        for &id in &ids {
            let Ok(head) = archive.read_head(id, 4096) else { continue };
            if head.len() < 12 || &head[..2] != b"PF" || &head[8..12] != b"mapc" {
                continue;
            }
            let Ok(pf) = Packfile::new(head) else { continue };
            for chunk in pf.chunks() {
                let Ok(chunk) = chunk else { break };
                if &chunk.name()[..4] == b"parm" && chunk.bytes().len() >= 16 {
                    let f: Vec<f32> = chunk.bytes()[..16]
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|b| f32::from_le_bytes(*b))
                        .collect();
                    println!("{id}\t{}\t{}\t{}\t{}", f[0], f[1], f[2], f[3]);
                    break;
                }
            }
        }
        return ExitCode::SUCCESS;
    }
    let mut hist: BTreeMap<String, usize> = BTreeMap::new();
    let mut hits = Vec::new();
    let mut errors = 0usize;
    for &id in &ids {
        match archive.read_head(id, 16) {
            Ok(h) => {
                let l = label(&h);
                if want.as_deref().is_some_and(|w| l == w || l == format!("PF:{w}")) && hits.len() < limit {
                    hits.push((id, archive.entry(id).map_or(0, |e| e.size)));
                }
                *hist.entry(l).or_default() += 1;
            }
            Err(_) => errors += 1,
        }
    }
    let mut by_count: Vec<_> = hist.into_iter().collect();
    by_count.sort_by_key(|e| std::cmp::Reverse(e.1));
    println!("entries {}  head-read errors {errors}", ids.len());
    for (l, n) in by_count.iter().take(60) {
        println!("{n:>9}  {l}");
    }
    if let Some(w) = want {
        println!("-- first {} ids of {w}:", hits.len());
        for (id, stored) in hits {
            println!("{id:>10}  stored={stored}");
        }
    }
    ExitCode::SUCCESS
}
