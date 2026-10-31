//! Loader for packfile schemas (`packfiles.xml`, ascalon's format; see
//! `reference/README.md`). Independent stable-Rust implementation.

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

#[derive(Debug, Clone, PartialEq)]
pub struct PackfileSchema {
    pub name: String,
    pub chunks: Vec<ChunkSchema>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkSchema {
    pub name: String,
    pub versions: Vec<VersionSchema>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionSchema {
    pub id: u32,
    pub type_name: String,
    pub fields: Vec<PfField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PfField {
    /// Field kind: `Dword`, `ArrayPtr`, `Ptr`, `Struct`, `WcharPtr`, ...
    pub kind: String,
    pub name: String,
    pub size: Option<u32>,
    /// Element/target type: a primitive kind, or a struct name when `fields` is non-empty.
    pub type_name: Option<String>,
    pub fields: Vec<PfField>,
}

impl ChunkSchema {
    pub fn version(&self, id: u32) -> Option<&VersionSchema> {
        self.versions.iter().find(|v| v.id == id)
    }
}

/// All schemas for a packfile type and chunk name. The reference schema has
/// ambiguous duplicates (e.g. `mapc`/`nm15` appears twice with different
/// structs), so callers that decode real data should try each candidate.
pub fn find_all<'s>(
    schemas: &'s [PackfileSchema],
    pf_type: &str,
    chunk: &str,
) -> Vec<&'s ChunkSchema> {
    schemas
        .iter()
        .filter(|p| p.name == pf_type)
        .flat_map(|p| &p.chunks)
        .filter(|c| c.name == chunk)
        .collect()
}

/// Finds the first schema for a packfile type and chunk name (4CCs, NULs trimmed).
/// See [`find_all`] for ambiguous names.
pub fn find<'s>(
    schemas: &'s [PackfileSchema],
    pf_type: &str,
    chunk: &str,
) -> Option<&'s ChunkSchema> {
    schemas
        .iter()
        .filter(|p| p.name == pf_type)
        .flat_map(|p| &p.chunks)
        .find(|c| c.name == chunk)
}

#[derive(Debug)]
pub struct PfSchemaError(pub String);

impl std::fmt::Display for PfSchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "packfile schema: {}", self.0)
    }
}
impl std::error::Error for PfSchemaError {}

type R<T> = Result<T, PfSchemaError>;

fn err(e: impl std::fmt::Display) -> PfSchemaError {
    PfSchemaError(e.to_string())
}

fn attr(e: &BytesStart<'_>, name: &str) -> R<Option<String>> {
    match e.try_get_attribute(name).map_err(err)? {
        Some(a) => Ok(Some(a.unescape_value().map_err(err)?.into_owned())),
        None => Ok(None),
    }
}

pub fn parse(xml: &str) -> R<Vec<PackfileSchema>> {
    let mut r = Reader::from_str(xml);
    r.config_mut().trim_text(true);
    let mut out = Vec::new();
    let mut cur_pf: Option<PackfileSchema> = None;
    let mut cur_chunk: Option<ChunkSchema> = None;
    loop {
        match r.read_event().map_err(err)? {
            Event::Start(e) => match e.name().as_ref() {
                b"Packfiles" => {}
                b"Packfile" => {
                    cur_pf = Some(PackfileSchema { name: attr(&e, "Name")?.unwrap_or_default(), chunks: Vec::new() })
                }
                b"Chunk" => {
                    cur_chunk = Some(ChunkSchema { name: attr(&e, "Name")?.unwrap_or_default(), versions: Vec::new() })
                }
                b"Version" => {
                    let id = attr(&e, "Id")?.ok_or_else(|| err("Version without Id"))?.parse().map_err(err)?;
                    let type_name = attr(&e, "TypeName")?.unwrap_or_default();
                    let fields = parse_fields(&mut r, b"Version")?;
                    cur_chunk
                        .as_mut()
                        .ok_or_else(|| err("Version outside Chunk"))?
                        .versions
                        .push(VersionSchema { id, type_name, fields });
                }
                other => return Err(err(format!("unexpected <{}>", String::from_utf8_lossy(other)))),
            },
            Event::Empty(e) if e.name().as_ref() == b"Version" => {
                let id = attr(&e, "Id")?.ok_or_else(|| err("Version without Id"))?.parse().map_err(err)?;
                let type_name = attr(&e, "TypeName")?.unwrap_or_default();
                cur_chunk
                    .as_mut()
                    .ok_or_else(|| err("Version outside Chunk"))?
                    .versions
                    .push(VersionSchema { id, type_name, fields: Vec::new() });
            }
            Event::End(e) => match e.name().as_ref() {
                b"Chunk" => {
                    let c = cur_chunk.take().ok_or_else(|| err("stray </Chunk>"))?;
                    cur_pf.as_mut().ok_or_else(|| err("Chunk outside Packfile"))?.chunks.push(c);
                }
                b"Packfile" => out.push(cur_pf.take().ok_or_else(|| err("stray </Packfile>"))?),
                _ => {}
            },
            Event::Eof => {
                if cur_pf.is_some() || cur_chunk.is_some() {
                    return Err(err("unexpected EOF"));
                }
                return Ok(out);
            }
            _ => {}
        }
    }
}

fn field_head(e: &BytesStart<'_>) -> R<PfField> {
    Ok(PfField {
        kind: String::from_utf8_lossy(e.name().as_ref()).into_owned(),
        name: attr(e, "Name")?.unwrap_or_default(),
        size: attr(e, "Size")?.map(|s| s.parse()).transpose().map_err(err)?,
        type_name: attr(e, "TypeName")?,
        fields: Vec::new(),
    })
}

fn parse_fields(r: &mut Reader<&[u8]>, end: &[u8]) -> R<Vec<PfField>> {
    let mut out = Vec::new();
    loop {
        match r.read_event().map_err(err)? {
            Event::Empty(e) => out.push(field_head(&e)?),
            Event::Start(e) => {
                let mut f = field_head(&e)?;
                let tag = e.name().as_ref().to_vec();
                f.fields = parse_fields(r, &tag)?;
                out.push(f);
            }
            Event::End(e) if e.name().as_ref() == end => return Ok(out),
            Event::Eof => return Err(err("unexpected EOF in fields")),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reference_208039_schema() {
        let s = parse(include_str!("../reference/packfiles-208039.xml")).unwrap();
        let names: Vec<_> = s.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["mapc", "MODL", "txtm", "MFST", "ARMF", "cntc"]);
        assert_eq!(s.iter().map(|p| p.chunks.len()).sum::<usize>(), 52);
        assert_eq!(s.iter().flat_map(|p| &p.chunks).map(|c| c.versions.len()).sum::<usize>(), 372);
        let havk = find(&s, "mapc", "havk").unwrap();
        let v6 = havk.version(6).unwrap();
        assert_eq!(v6.type_name, "PackMapCollideV6");
        assert_eq!(v6.fields[0].kind, "Float3");
        assert_eq!(v6.fields[0].name, "boundsMin");
        let mopp = &v6.fields[2];
        assert_eq!((mopp.kind.as_str(), mopp.type_name.as_deref()), ("ArrayPtr", Some("PackMapCollideMoppCodeV6")));
        assert_eq!(mopp.fields[0].type_name.as_deref(), Some("Byte"));

        let nm = find_all(&s, "mapc", "nm15");
        let types: Vec<_> = nm.iter().map(|c| c.versions.last().unwrap().type_name.as_str()).collect();
        assert_eq!(types, ["PackMapNavMeshChunkV3", "PackMapNavMeshV3"]);
    }

    #[test]
    fn rejects_truncated() {
        assert!(parse("<Packfiles><Packfile Name=\"x\"><Chunk Name=\"y\">").is_err());
    }
}
