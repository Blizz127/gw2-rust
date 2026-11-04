//! Asset-patcher manifests (`ARMF` root manifest, `MFST` manifests) for the offline
//! asset CDN.
//!
//! Layouts follow the 208039 reference schema (`PackAssetRootManifest` = `ARMF` v1,
//! `PackAssetManifest` = `MFST` v6). The type names match strings in the build-209026
//! client (REA, docs/re/209026-patcher.md), but the field layouts are **not yet
//! confirmed** for our build. The client downloads the root manifest by the file id
//! and size given in the `/latest64/101` reply, checks the packfile signature is
//! `ARMF` (`FUN_140fdd6e0`), and parses it.

use crate::pfwrite::{ChunkBuilder, packfile};

/// One `PackAssetManifestFile` entry of the root manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestFile {
    pub base_id: u32,
    pub file_id: u32,
    pub size: u32,
    pub flags: u32,
    pub name: String,
}

/// One `PackAssetExtraFile` entry of the root manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtraFile {
    pub base_id: u32,
    pub file_id: u32,
    pub size: u32,
    pub file_type: u32,
}

/// `ARMF` v1: `buildId, manifests[], extraFiles[]`.
/// Written with 64-bit pointers, so the client needs no 32→64 widening (which asserts
/// on files without fix-ups).
pub fn root_manifest(build: u32, manifests: &[ManifestFile], extra: &[ExtraFile]) -> Vec<u8> {
    let mut b = ChunkBuilder::new64();
    b.u32(build);
    b.u32(manifests.len() as u32);
    let m_ptr = b.ptr_slot();
    b.u32(extra.len() as u32);
    let e_ptr = b.ptr_slot();
    if !manifests.is_empty() {
        let start = b.pos();
        b.point(m_ptr, start);
        let mut name_slots = Vec::new();
        for m in manifests {
            b.u32(m.base_id).u32(m.file_id).u32(m.size).u32(m.flags);
            name_slots.push(b.ptr_slot());
        }
        for (slot, m) in name_slots.into_iter().zip(manifests) {
            let s = b.wstr(&m.name);
            b.point(slot, s);
        }
        b.align(4);
    }
    if !extra.is_empty() {
        let start = b.pos();
        b.point(e_ptr, start);
        for e in extra {
            b.u32(e.base_id).u32(e.file_id).u32(e.size).u32(e.file_type);
        }
    }
    packfile(*b"ARMF", &[(*b"ARMF", 1, &b)])
}

/// One `PackAssetManifestRecord`: an asset's base id, current file id, size, flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    pub base_id: u32,
    pub file_id: u32,
    pub size: u32,
    pub flags: u32,
}

/// Records for every asset in a local archive: entries with two ids become
/// `base = smaller id, file = larger id`, single ids become `base = file = id`.
/// `size` is the decoded size (run 30: with the stored size the client re-downloaded
/// every asset). Sorted by base id. The pairing rule is an
/// inference from the index layout, not confirmed in REA.
pub fn records_from_archive<R: std::io::Read + std::io::Seek>(a: &crate::archive::Archive<R>) -> Vec<Record> {
    let mut by_mft: std::collections::BTreeMap<usize, Vec<u32>> = std::collections::BTreeMap::new();
    for &(id, mft) in a.index_pairs() {
        by_mft.entry(mft).or_default().push(id);
    }
    let mut out: Vec<Record> = by_mft
        .into_iter()
        .filter_map(|(mft, ids)| {
            let size = a.decoded_size(mft).ok()?;
            let (lo, hi) = (*ids.iter().min()?, *ids.iter().max()?);
            Some(Record { base_id: lo, file_id: hi, size, flags: 0 })
        })
        .collect();
    out.sort_by_key(|r| r.base_id);
    out
}

/// One `MFST` v6 property: `type` and its data bytes, attached to the record whose
/// `baseId` is `base_id`. The client's MultiArchive keeps them per file
/// (MultiManifest.cpp `0x140d91cf0` -> `0x140d973e0`) and reads them back by
/// (file, type) through `0x140d97290`. Type 0 with 4 bytes is what the texture loader
/// asks for (`0x140d8fc20` via `0x140d90480`): nonzero selects the 0x400 mip-drop base,
/// so a texture keeps all its levels at the highest texture quality.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub base_id: u32,
    pub kind: u8,
    pub data: Vec<u8>,
}

/// `MFST` v6 (`PackAssetManifest`): `buildId, totalRecordSize (Token, 8 bytes),
/// records[], streams[], properties[], propertyTable[]`. Streams are written empty.
/// 64-bit pointers, like [`root_manifest`].
pub fn manifest(build: u32, records: &[Record]) -> Vec<u8> {
    manifest_with_properties(build, records, &[])
}

/// [`manifest`] with properties (`{Byte type, ArrayPtr<Byte> data}`) and one
/// `propertyTable` row `{baseId, propertyIndex}` per property.
pub fn manifest_with_properties(build: u32, records: &[Record], properties: &[Property]) -> Vec<u8> {
    let mut b = ChunkBuilder::new64();
    b.u32(build);
    b.u64(records.iter().map(|r| u64::from(r.size)).sum());
    b.u32(records.len() as u32);
    let r_ptr = b.ptr_slot();
    b.u32(0);
    b.ptr_slot(); // streams
    b.u32(properties.len() as u32);
    let p_ptr = b.ptr_slot();
    b.u32(properties.len() as u32);
    let t_ptr = b.ptr_slot();
    if !records.is_empty() {
        let start = b.pos();
        b.point(r_ptr, start);
        for r in records {
            b.u32(r.base_id).u32(r.file_id).u32(r.size).u32(r.flags);
        }
    }
    if !properties.is_empty() {
        let start = b.pos();
        b.point(p_ptr, start);
        let mut data_slots = Vec::new();
        for p in properties {
            b.u8(p.kind);
            b.u32(p.data.len() as u32);
            data_slots.push(b.ptr_slot());
        }
        let table = b.pos();
        b.point(t_ptr, table);
        for (index, p) in properties.iter().enumerate() {
            b.u32(p.base_id).u32(index as u32);
        }
        for (slot, p) in data_slots.into_iter().zip(properties) {
            let at = b.pos();
            b.point(slot, at);
            b.bytes(&p.data);
        }
    }
    packfile(*b"MFST", &[(*b"MFST", 6, &b)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packfile::Packfile;
    use crate::pfread::{Limits, PfValue, read_chunk};
    use crate::pfschema::{find, parse};

    const REFERENCE: &str = include_str!("../reference/packfiles-208039.xml");

    #[test]
    fn manifest_properties_decode_with_reference_schema() {
        let schemas = parse(REFERENCE).unwrap();
        let recs = [Record { base_id: 135679, file_id: 135679, size: 26908, flags: 0 }];
        let props = [Property { base_id: 135679, kind: 0, data: 1u32.to_le_bytes().to_vec() }];
        let pf = Packfile::new(manifest_with_properties(209026, &recs, &props)).unwrap();
        let chunk = pf.chunks().next().unwrap().unwrap();
        let v6 = find(&schemas, "MFST", "MFST").unwrap().version(6).unwrap();
        let value = read_chunk(chunk.bytes(), v6, true, Limits::default()).unwrap();
        let text = format!("{value:?}");
        assert!(
            text.contains(r#"("properties", Array([Struct([("type", U8(0)), ("data", Array([U8(1), U8(0), U8(0), U8(0)]))])]))"#),
            "{text}"
        );
        assert!(
            text.contains(r#"("propertyTable", Array([Struct([("baseId", U32(135679)), ("properyIndex", U32(0))])]))"#),
            "{text}"
        );
    }

    #[test]
    fn root_manifest_decodes_with_reference_schema() {
        let blob = root_manifest(
            209026,
            &[ManifestFile { base_id: 10, file_id: 11, size: 12, flags: 1, name: "core".into() }],
            &[ExtraFile { base_id: 20, file_id: 21, size: 22, file_type: 3 }],
        );
        let pf = Packfile::new(blob).unwrap();
        assert_eq!(pf.r#type(), *b"ARMF");
        let chunk = pf.chunks().next().unwrap().unwrap();
        let schemas = parse(REFERENCE).unwrap();
        let v1 = find(&schemas, "ARMF", "ARMF").unwrap().version(1).unwrap();
        assert!(pf.is_64bit());
        let v = read_chunk(chunk.bytes(), v1, true, Limits::default()).unwrap();
        assert_eq!(v.get("buildId"), Some(&PfValue::U32(209026)));
        let PfValue::Array(ms) = v.get("manifests").unwrap() else { panic!() };
        assert_eq!(ms[0].get("fileId"), Some(&PfValue::U32(11)));
        assert_eq!(ms[0].get("name"), Some(&PfValue::Text("core".into())));
        let PfValue::Array(es) = v.get("extraFiles").unwrap() else { panic!() };
        assert_eq!(es[0].get("fileType"), Some(&PfValue::U32(3)));
        // name pointer + both array pointers are fixed up
        assert_eq!(chunk.fixup_offsets().unwrap().len(), 3);
    }

    #[test]
    fn manifest_v6_decodes_with_reference_schema() {
        let recs = [Record { base_id: 1, file_id: 2, size: 3, flags: 0 }, Record { base_id: 4, file_id: 5, size: 6, flags: 1 }];
        let pf = Packfile::new(manifest(209026, &recs)).unwrap();
        assert_eq!(pf.r#type(), *b"MFST");
        let chunk = pf.chunks().next().unwrap().unwrap();
        assert_eq!(chunk.version(), 6);
        let schemas = parse(REFERENCE).unwrap();
        let v6 = find(&schemas, "MFST", "MFST").unwrap().version(6).unwrap();
        let v = read_chunk(chunk.bytes(), v6, true, Limits::default()).unwrap();
        assert_eq!(v.get("buildId"), Some(&PfValue::U32(209026)));
        let PfValue::Array(rs) = v.get("records").unwrap() else { panic!() };
        assert_eq!(rs.len(), 2);
        assert_eq!(rs[1].get("fileId"), Some(&PfValue::U32(5)));
        assert_eq!(v.get("streams"), Some(&PfValue::Array(vec![])));
    }

    #[test]
    fn empty_root_manifest_is_twenty_eight_bytes_of_payload() {
        let pf = Packfile::new(root_manifest(209026, &[], &[])).unwrap();
        let c = pf.chunks().next().unwrap().unwrap();
        assert_eq!(c.bytes().len(), 4 + (4 + 8) * 2);
        assert!(c.fixup_offsets().unwrap().is_empty());
    }
}
