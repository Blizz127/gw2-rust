//! Importer and offline cache for the public, unauthenticated Guild Wars 2
//! web API (`https://api.guildwars2.com/v2`).
//!
//! No API key is used or required: only public endpoints are touched.
//!
//! Raw JSON is cached under `<root>/<build>/<endpoint>.json` (an array of all
//! objects) with a `<root>/<build>/manifest.json` describing the import. The
//! typed structs ([`Map`], [`Item`], [`Skill`]) are tolerant views used when
//! loading from that cache.

pub mod floor;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Base URL of the public API.
pub const DEFAULT_BASE_URL: &str = "https://api.guildwars2.com/v2";
/// Maximum page size the API accepts.
pub const MAX_PAGE_SIZE: u32 = 200;
/// Delay between consecutive requests.
pub const REQUEST_DELAY: Duration = Duration::from_millis(150);
/// Maximum retries for 429/5xx/transport failures.
pub const MAX_RETRIES: u32 = 5;

const MAX_BODY_BYTES: u64 = 256 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Typed structs (tolerant; raw JSON is what is cached)
// ---------------------------------------------------------------------------

/// `[[x0, y0], [x1, y1]]` rectangle as used by `map_rect` / `continent_rect`.
pub type Rect = [[f64; 2]; 2];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Map {
    pub id: u32,
    pub name: String,
    pub min_level: u32,
    pub max_level: u32,
    pub default_floor: i32,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub floors: Vec<i32>,
    pub region_id: Option<u32>,
    pub region_name: Option<String>,
    pub continent_id: Option<u32>,
    pub continent_name: Option<String>,
    pub map_rect: Option<Rect>,
    pub continent_rect: Option<Rect>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Item {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub level: Option<u32>,
    pub rarity: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Skill {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub level: Option<u32>,
    pub rarity: Option<String>,
    pub icon: Option<String>,
    pub professions: Vec<String>,
}

/// Contents of `manifest.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Manifest {
    pub build: u32,
    pub fetched_at_unix: u64,
    pub fetched_at: String,
    /// endpoint name -> number of objects cached
    pub endpoints: BTreeMap<String, usize>,
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Parse an `X-Page-Total` header value.
pub fn parse_page_total(value: Option<&str>) -> Option<u32> {
    value?.trim().parse().ok()
}

/// Endpoint names become file names, so only a conservative charset is allowed.
pub fn validate_endpoint(endpoint: &str) -> Result<()> {
    let ok = !endpoint.is_empty()
        && endpoint
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    if !ok {
        bail!("invalid endpoint name {endpoint:?} (allowed: a-z 0-9 _ -)");
    }
    Ok(())
}

/// Format a unix timestamp as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn format_utc(unix: u64) -> String {
    let days = (unix / 86_400) as i64;
    let rem = unix % 86_400;
    // Civil-from-days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// One page of objects plus the total page count reported by the server.
#[derive(Debug, Clone)]
pub struct Page {
    pub items: Vec<serde_json::Value>,
    /// `X-Page-Total`, if the server sent it.
    pub page_total: Option<u32>,
}

/// Polite, sequential client for the public API.
pub struct Client {
    agent: ureq::Agent,
    base_url: String,
    delay: Duration,
    retry_base: Duration,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

struct RawResponse {
    body: String,
    page_total: Option<u32>,
}

impl Client {
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(120)))
            .user_agent("gw2-rust-api-import (+https://github.com/Blizz127/gw2-rust)")
            .build();
        Self {
            agent: ureq::Agent::new_with_config(config),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            delay: REQUEST_DELAY,
            retry_base: Duration::from_millis(500),
        }
    }

    /// GET `path` (relative to the base URL, may include a query), retrying
    /// on 429/5xx/transport errors with exponential backoff.
    fn get(&self, path: &str) -> Result<RawResponse> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        let mut attempt = 0u32;
        loop {
            sleep(self.delay);
            let failure = match self.agent.get(&url).call() {
                Ok(mut resp) => {
                    let status = resp.status().as_u16();
                    if (200..300).contains(&status) {
                        let page_total = parse_page_total(
                            resp.headers()
                                .get("x-page-total")
                                .and_then(|v| v.to_str().ok()),
                        );
                        let body = resp
                            .body_mut()
                            .with_config()
                            .limit(MAX_BODY_BYTES)
                            .read_to_string()
                            .with_context(|| format!("reading body of {url}"))?;
                        return Ok(RawResponse { body, page_total });
                    }
                    let retry_after = resp
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.trim().parse::<u64>().ok());
                    let body = resp.body_mut().read_to_string().unwrap_or_default();
                    if status != 429 && status < 500 {
                        bail!("GET {url} failed: HTTP {status}: {}", body.trim());
                    }
                    (format!("HTTP {status}"), retry_after)
                }
                Err(e) => (format!("transport error: {e}"), None),
            };
            if attempt >= MAX_RETRIES {
                bail!(
                    "GET {url} failed after {MAX_RETRIES} retries: {}",
                    failure.0
                );
            }
            let backoff = self.retry_base * 2u32.pow(attempt);
            let wait = failure
                .1
                .map_or(backoff, |s| backoff.max(Duration::from_secs(s.min(120))));
            eprintln!(
                "  {} on {url}; retry {}/{MAX_RETRIES} in {:?}",
                failure.0,
                attempt + 1,
                wait
            );
            sleep(wait);
            attempt += 1;
        }
    }

    /// `GET /v2/build` -> `{"id": N}`.
    pub fn fetch_build(&self) -> Result<u32> {
        #[derive(Deserialize)]
        struct B {
            id: u32,
        }
        let raw = self.get("build")?;
        let b: B = serde_json::from_str(&raw.body).context("parsing /v2/build")?;
        Ok(b.id)
    }

    /// `GET /v2/<endpoint>` -> list of ids (for endpoints that expose them).
    pub fn fetch_ids(&self, endpoint: &str) -> Result<Vec<u64>> {
        validate_endpoint(endpoint)?;
        let raw = self.get(endpoint)?;
        serde_json::from_str(&raw.body).with_context(|| format!("parsing id list of {endpoint}"))
    }

    /// `GET /v2/<endpoint>?page=N&page_size=S` (page is 0-based).
    pub fn fetch_page(&self, endpoint: &str, page: u32, page_size: u32) -> Result<Page> {
        validate_endpoint(endpoint)?;
        if page_size == 0 || page_size > MAX_PAGE_SIZE {
            bail!("page_size must be 1..={MAX_PAGE_SIZE}, got {page_size}");
        }
        let raw = self.get(&format!("{endpoint}?page={page}&page_size={page_size}"))?;
        let items: Vec<serde_json::Value> = serde_json::from_str(&raw.body)
            .with_context(|| format!("parsing page {page} of {endpoint}"))?;
        Ok(Page {
            items,
            page_total: raw.page_total,
        })
    }

    /// Fetch every object of a paginated endpoint, sequentially.
    pub fn fetch_all(&self, endpoint: &str) -> Result<Vec<serde_json::Value>> {
        let mut all = Vec::new();
        let mut page = 0u32;
        loop {
            let p = self.fetch_page(endpoint, page, MAX_PAGE_SIZE)?;
            let total = p.page_total;
            let short = p.items.len() < MAX_PAGE_SIZE as usize;
            all.extend(p.items);
            page += 1;
            eprintln!(
                "  {endpoint}: page {page}/{} ({} objects)",
                total.map_or("?".into(), |t| t.to_string()),
                all.len()
            );
            match total {
                Some(t) if page >= t => break,
                None if short => break,
                _ => {}
            }
        }
        Ok(all)
    }

    /// `GET /v2/continents/<c>/floors/<f>/regions/<r>/maps/<m>` as raw JSON.
    pub fn fetch_map_floor(
        &self,
        continent: u32,
        floor: i32,
        region: u32,
        map: u32,
    ) -> Result<serde_json::Value> {
        let path = format!("continents/{continent}/floors/{floor}/regions/{region}/maps/{map}");
        let raw = self.get(&path)?;
        serde_json::from_str(&raw.body).with_context(|| format!("parsing {path}"))
    }

    /// `GET /v2/maps/<id>` as raw JSON.
    pub fn fetch_map(&self, id: u32) -> Result<serde_json::Value> {
        let raw = self.get(&format!("maps/{id}"))?;
        serde_json::from_str(&raw.body).with_context(|| format!("parsing map {id}"))
    }
}

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

/// Directory for a build: `<root>/<build>`.
pub fn build_dir(root: &Path, build: u32) -> PathBuf {
    root.join(build.to_string())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes).with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("renaming to {}", path.display()))?;
    Ok(())
}

/// Write `<root>/<build>/<endpoint>.json` and update the manifest.
/// Returns the path of the endpoint file.
pub fn write_cache(
    root: &Path,
    build: u32,
    endpoint: &str,
    objects: &[serde_json::Value],
) -> Result<PathBuf> {
    validate_endpoint(endpoint)?;
    let dir = build_dir(root, build);
    let path = dir.join(format!("{endpoint}.json"));
    write_atomic(&path, &serde_json::to_vec(objects)?)?;

    let mut manifest = read_manifest(root, build)?.unwrap_or_default();
    manifest.build = build;
    manifest.fetched_at_unix = now_unix();
    manifest.fetched_at = format_utc(manifest.fetched_at_unix);
    manifest
        .endpoints
        .insert(endpoint.to_string(), objects.len());
    write_atomic(
        &dir.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(path)
}

/// Write a single raw object to `<root>/<build>/<endpoint>/<id>.json`.
pub fn write_single(
    root: &Path,
    build: u32,
    endpoint: &str,
    id: u64,
    object: &serde_json::Value,
) -> Result<PathBuf> {
    validate_endpoint(endpoint)?;
    let path = build_dir(root, build)
        .join(endpoint)
        .join(format!("{id}.json"));
    write_atomic(&path, &serde_json::to_vec_pretty(object)?)?;
    Ok(path)
}

/// Read `<root>/<build>/manifest.json`, if present.
pub fn read_manifest(root: &Path, build: u32) -> Result<Option<Manifest>> {
    let path = build_dir(root, build).join("manifest.json");
    match fs::read(&path) {
        Ok(bytes) => Ok(Some(
            serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing {}", path.display()))?,
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

/// Load a cached endpoint as typed objects.
pub fn load_cached<T: DeserializeOwned>(root: &Path, build: u32, endpoint: &str) -> Result<Vec<T>> {
    validate_endpoint(endpoint)?;
    let path = build_dir(root, build).join(format!("{endpoint}.json"));
    let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

/// Highest numeric build directory under `root`, if any.
pub fn latest_cached_build(root: &Path) -> Option<u32> {
    fs::read_dir(root)
        .ok()?
        .filter_map(|e| e.ok()?.file_name().to_str()?.parse::<u32>().ok())
        .max()
}

/// Default cache root: `<workspace>/data/api`.
pub fn default_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    // crates/gw2-api -> workspace root
    manifest
        .ancestors()
        .nth(2)
        .unwrap_or(manifest)
        .join("data/api")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const QUEENSDALE: &str = r#"{"id":15,"name":"Queensdale","min_level":1,"max_level":15,"default_floor":1,"type":"Public","floors":[1,65,3,2,0],"region_id":4,"region_name":"Kryta","continent_id":1,"continent_name":"Tyria","map_rect":[[-43008,-27648],[43008,30720]],"continent_rect":[[42624,28032],[46208,30464]]}"#;

    fn temp_root(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("gw2-api-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn parses_queensdale() {
        let m: Map = serde_json::from_str(QUEENSDALE).unwrap();
        assert_eq!(m.id, 15);
        assert_eq!(m.name, "Queensdale");
        assert_eq!((m.min_level, m.max_level, m.default_floor), (1, 15, 1));
        assert_eq!(m.kind.as_deref(), Some("Public"));
        assert_eq!(m.floors, vec![1, 65, 3, 2, 0]);
        assert_eq!(m.region_id, Some(4));
        assert_eq!(m.region_name.as_deref(), Some("Kryta"));
        assert_eq!(m.continent_id, Some(1));
        assert_eq!(m.continent_name.as_deref(), Some("Tyria"));
        assert_eq!(m.map_rect, Some([[-43008.0, -27648.0], [43008.0, 30720.0]]));
        assert_eq!(
            m.continent_rect,
            Some([[42624.0, 28032.0], [46208.0, 30464.0]])
        );
    }

    #[test]
    fn tolerates_missing_and_unknown_fields() {
        // Some maps lack regions/rects; unknown fields must be ignored.
        let m: Map =
            serde_json::from_str(r#"{"id":1,"name":"X","brand_new_field":[1,2]}"#).unwrap();
        assert_eq!(m.id, 1);
        assert!(m.map_rect.is_none() && m.floors.is_empty());
        let i: Item = serde_json::from_str(
            r#"{"id":24,"name":"Sword","type":"Weapon","rarity":"Fine","level":3,"x":1}"#,
        )
        .unwrap();
        assert_eq!(i.kind.as_deref(), Some("Weapon"));
        assert_eq!(i.level, Some(3));
        let s: Skill =
            serde_json::from_str(r#"{"id":5,"name":"Fireball","professions":["Elementalist"]}"#)
                .unwrap();
        assert_eq!(s.professions, vec!["Elementalist"]);
        assert!(s.icon.is_none());
    }

    #[test]
    fn cache_round_trip() {
        let root = temp_root("rt");
        let q: serde_json::Value = serde_json::from_str(QUEENSDALE).unwrap();
        let objs = vec![q, json!({"id":16,"name":"Other"})];
        let path = write_cache(&root, 209026, "maps", &objs).unwrap();
        assert_eq!(path, root.join("209026").join("maps.json"));

        write_cache(&root, 209026, "races", &[json!({"id":"Asura"})]).unwrap();
        let manifest = read_manifest(&root, 209026).unwrap().unwrap();
        assert_eq!(manifest.build, 209026);
        assert_eq!(manifest.endpoints["maps"], 2);
        assert_eq!(manifest.endpoints["races"], 1);
        assert!(manifest.fetched_at.ends_with('Z'));

        let maps: Vec<Map> = load_cached(&root, 209026, "maps").unwrap();
        assert_eq!(maps.len(), 2);
        assert_eq!(maps[0].name, "Queensdale");
        assert_eq!(latest_cached_build(&root), Some(209026));

        let single = write_single(&root, 209026, "maps", 15, &objs[0]).unwrap();
        assert!(single.ends_with("209026/maps/15.json"));
        assert!(load_cached::<Map>(&root, 1, "maps").is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn page_total_header() {
        assert_eq!(parse_page_total(Some("42")), Some(42));
        assert_eq!(parse_page_total(Some(" 7 ")), Some(7));
        assert_eq!(parse_page_total(Some("abc")), None);
        assert_eq!(parse_page_total(None), None);
    }

    #[test]
    fn endpoint_validation_and_time_format() {
        assert!(validate_endpoint("maps").is_ok());
        assert!(validate_endpoint("../etc").is_err());
        assert!(validate_endpoint("").is_err());
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(1_709_210_096), "2024-02-29T12:34:56Z");
    }
}
