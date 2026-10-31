//! CLI for importing the public GW2 web API into the offline cache.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use gw2_api::{Client, default_root, write_cache, write_single};

const USAGE: &str = "\
gw2-api-import [--out DIR] <command>

Commands:
  build                     print the current game build id
  fetch <endpoint>...       fetch all objects of paginated endpoints
                            (maps, items, skills, professions, races,
                            specializations, continents, ...)
  fetch-map <id>            fetch a single map
  fetch-map-floor <id>      fetch a map's floor data (POIs, waypoints, hearts,
                            hero challenges, sectors) on its default floor

Options:
  --out DIR   cache root (default: <workspace>/data/api)
  -h, --help  show this help

Only the public, unauthenticated API is used; no API key is needed.";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut out: Option<PathBuf> = None;
    let mut rest = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out = Some(args.next().context("--out needs a directory")?.into()),
            s if s.starts_with("--out=") => out = Some(s["--out=".len()..].into()),
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            _ => rest.push(a),
        }
    }
    let root = out.unwrap_or_else(default_root);
    let client = Client::new();

    let Some((cmd, params)) = rest.split_first() else {
        bail!("missing command\n\n{USAGE}");
    };
    match cmd.as_str() {
        "build" => println!("{}", client.fetch_build()?),
        "fetch" => {
            if params.is_empty() {
                bail!("fetch needs at least one endpoint\n\n{USAGE}");
            }
            let build = client.fetch_build()?;
            println!("build {build}");
            for endpoint in params {
                gw2_api::validate_endpoint(endpoint)?;
                eprintln!("fetching {endpoint} ...");
                let objects = client.fetch_all(endpoint)?;
                let path = write_cache(&root, build, endpoint, &objects)?;
                println!(
                    "{endpoint}: {} objects -> {}",
                    objects.len(),
                    path.display()
                );
            }
        }
        "fetch-map" => {
            let [id] = params else {
                bail!("fetch-map needs exactly one map id\n\n{USAGE}");
            };
            let id: u32 = id.parse().with_context(|| format!("bad map id {id:?}"))?;
            let build = client.fetch_build()?;
            let map = client.fetch_map(id)?;
            let path = write_single(&root, build, "maps", u64::from(id), &map)?;
            println!(
                "map {id} ({}) -> {}",
                map["name"].as_str().unwrap_or("?"),
                path.display()
            );
        }
        "fetch-map-floor" => {
            let [id] = params else {
                bail!("fetch-map-floor needs exactly one map id\n\n{USAGE}");
            };
            let id: u32 = id.parse().with_context(|| format!("bad map id {id:?}"))?;
            let build = client.fetch_build()?;
            let map: gw2_api::Map = serde_json::from_value(client.fetch_map(id)?)?;
            let (Some(c), Some(r)) = (map.continent_id, map.region_id) else {
                bail!("map {id} has no continent/region");
            };
            let floor = client.fetch_map_floor(c, map.default_floor, r, id)?;
            let path = write_single(&root, build, "map-floors", u64::from(id), &floor)?;
            let typed: gw2_api::floor::MapFloor = serde_json::from_value(floor)?;
            println!(
                "map {id} ({}) floor {}: {} POIs, {} hearts, {} hero challenges, {} sectors -> {}",
                map.name,
                map.default_floor,
                typed.points_of_interest.len(),
                typed.tasks.len(),
                typed.skill_challenges.len(),
                typed.sectors.len(),
                path.display()
            );
        }
        other => bail!("unknown command {other:?}\n\n{USAGE}"),
    }
    Ok(())
}
