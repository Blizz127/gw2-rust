//! Read-only check of build-209026 progress definitions and typed parameters.

use gw2_dat::{
    archive::Archive, content::ContentIndex, packfile::Packfile,
    progress_content::ProgressDefinition209026,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    process::ExitCode,
};

fn run(path: &str) -> io::Result<()> {
    let archive = Archive::open(path)?;
    let packfiles: Vec<_> = (1282830..=1282861)
        .map(|id| archive.read(id).and_then(Packfile::new))
        .collect::<io::Result<_>>()?;
    let containers: Vec<_> = packfiles
        .iter()
        .map(ContentIndex::parse)
        .collect::<io::Result<_>>()?;
    let mut keys = BTreeSet::new();
    let mut uids = BTreeSet::new();
    let mut types = BTreeMap::<u32, usize>::new();
    let mut counter_limits = 0;
    let mut bit_parameters = 0;
    let mut zero_width = 0;
    let mut different_fields = 0;
    let mut max_full_update_words = 0u64;
    let mut zero_channels = 0;
    let mut multiple_channels = 0;
    for (container, index) in containers.iter().enumerate() {
        for entry in index.entries().filter(|e| e.content_type == 53) {
            let view = ProgressDefinition209026::read(index, entry.offset)?;
            if !keys.insert(view.key) || !uids.insert(view.uid) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "duplicate progress key or UID",
                ));
            }
            *types.entry(view.data_type).or_default() += 1;
            let channels = (view.channel_mask & 0xffff).count_ones();
            zero_channels += usize::from(channels == 0);
            multiple_channels += usize::from(channels > 1);
            counter_limits += usize::from(view.counter_limit(&containers, container)?.is_some());
            if let Some(params) = view.bit_mask_parameters(&containers, container)? {
                bit_parameters += 1;
                zero_width += usize::from(params.bit_count == 0);
                different_fields += usize::from(params.bit_count != params.completion_count);
                // Catalog summary only: the client uses float ceiling at runtime.
                let words = u64::from(params.bit_count).div_ceil(32) * u64::from(channels);
                max_full_update_words = max_full_update_words.max(words);
            }
        }
    }
    println!(
        "checked 32 base containers: progress={} types={types:?} counter_limits={counter_limits} bit_mask_parameters={bit_parameters}",
        keys.len()
    );
    println!(
        "zero_channels={zero_channels} multiple_channels={multiple_channels} bit_mask_zero_width={zero_width} differing_count_width={different_fields} max_full_update_words={max_full_update_words}"
    );
    println!("record/parameter bounds checked; runtime update eligibility untested");
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 || args[0].starts_with('-') {
        eprintln!("usage: gw2-progress-check <build-209026 Gw2.dat>");
        return ExitCode::from(2);
    }
    match run(&args[0]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("progress check failed: {error}");
            ExitCode::FAILURE
        }
    }
}
