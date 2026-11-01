//! `gw2-dat-ids <Gw2.dat> [--sample N]`
//!
//! Groups the raw file index by MFT entry and prints how many entries have 1, 2, 3…
//! ids, then N sample groups (`mft#: ids…`). This is the input for generating
//! patcher manifests (base id / file id pairs) from a local install.

use std::collections::BTreeMap;
use std::process::ExitCode;

use gw2_dat::archive::Archive;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!("usage: gw2-dat-ids <Gw2.dat> [--sample N]");
        return ExitCode::from(2);
    };
    let sample = args.iter().position(|a| a == "--sample").and_then(|i| args.get(i + 1)).and_then(|n| n.parse().ok()).unwrap_or(10usize);
    let a = match Archive::open(path) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut groups: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
    for &(id, mft) in a.index_pairs() {
        groups.entry(mft).or_default().push(id);
    }
    let mut sizes: BTreeMap<usize, usize> = BTreeMap::new();
    for ids in groups.values() {
        *sizes.entry(ids.len()).or_default() += 1;
    }
    println!("index pairs {}, mft entries referenced {}", a.index_pairs().len(), groups.len());
    for (k, n) in &sizes {
        println!("{n:>9} entries with {k} id(s)");
    }
    for (mft, ids) in groups.iter().step_by((groups.len() / sample.max(1)).max(1)).take(sample) {
        let e = a.mft_entries()[*mft];
        println!("mft#{mft}: ids {ids:?} size {} compressed {}", e.size, e.is_compressed());
    }
    ExitCode::SUCCESS
}
