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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packfile::Packfile;
    use crate::pfread::{Limits, PfValue, read_chunk};
    use crate::pfschema::{find, parse};

    const REFERENCE: &str = include_str!("../reference/packfiles-208039.xml");

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
    fn empty_root_manifest_is_twenty_eight_bytes_of_payload() {
        let pf = Packfile::new(root_manifest(209026, &[], &[])).unwrap();
        let c = pf.chunks().next().unwrap().unwrap();
        assert_eq!(c.bytes().len(), 4 + (4 + 8) * 2);
        assert!(c.fixup_offsets().unwrap().is_empty());
    }
}
