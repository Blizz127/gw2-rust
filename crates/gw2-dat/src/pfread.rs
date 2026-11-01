//! Schema-driven reader for raw (un-fixed) packfile chunk payloads.
//!
//! Layout rules (from ascalon `asset/src/packfile.rs`, reference build 208039,
//! **unverified on our build**): structs are packed. A pointer is a 4- or 8-byte
//! signed offset relative to its own slot within the chunk payload (0 = null).
//! `ArrayPtr` = `u32 count` + pointer. `List` = `ArrayPtr` of pointers.
//! `CharPtr`/`WcharPtr`/`FileName`/`FileRef` point at NUL-terminated strings.
//!
//! This reads the on-disk bytes directly, without ascalon's unsafe pointer
//! fix-up. Every access is bounds-checked, and depth and element budgets stop
//! cycles or garbage counts from running away.

use std::io::{Error, ErrorKind, Result};

use crate::pfschema::{PfField, VersionSchema};

#[derive(Debug, Clone, PartialEq)]
pub enum PfValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    /// Fixed tuples: `Byte3/4`, `Word3`, `Dword2/4`, `Float2/3/4`, `Guid` (as bytes).
    Bytes(Vec<u8>),
    Words(Vec<u16>),
    Dwords(Vec<u32>),
    Floats(Vec<f32>),
    Text(String),
    Null,
    Array(Vec<PfValue>),
    Struct(Vec<(String, PfValue)>),
}

impl PfValue {
    /// Field lookup on a `Struct`.
    pub fn get(&self, name: &str) -> Option<&PfValue> {
        match self {
            PfValue::Struct(f) => f.iter().find(|(n, _)| n == name).map(|(_, v)| v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_depth: usize,
    pub max_elements: usize,
    pub max_string: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { max_depth: 48, max_elements: 4_000_000, max_string: 64 * 1024 }
    }
}

struct Reader<'a> {
    data: &'a [u8],
    ptr_width: usize,
    limits: Limits,
    elements: usize,
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidData, msg.into())
}

/// Decodes a chunk payload with the given version schema.
/// `ptr64` comes from the packfile header (`Packfile::is_64bit`).
pub fn read_chunk(data: &[u8], version: &VersionSchema, ptr64: bool, limits: Limits) -> Result<PfValue> {
    let mut r = Reader { data, ptr_width: if ptr64 { 8 } else { 4 }, limits, elements: 0 };
    let mut pos = 0usize;
    r.read_struct(&version.fields, &mut pos, 0)
}

impl Reader<'_> {
    fn bytes(&self, pos: usize, n: usize) -> Result<&[u8]> {
        pos.checked_add(n)
            .and_then(|end| self.data.get(pos..end))
            .ok_or_else(|| Error::new(ErrorKind::UnexpectedEof, format!("read of {n} at {pos} past {}", self.data.len())))
    }

    fn take<const N: usize>(&self, pos: &mut usize) -> Result<[u8; N]> {
        let b: [u8; N] = self.bytes(*pos, N)?.try_into().unwrap();
        *pos += N;
        Ok(b)
    }

    fn u32(&self, pos: &mut usize) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(pos)?))
    }

    /// Reads a pointer slot at `pos` and returns the absolute target, or None.
    fn ptr(&self, pos: &mut usize) -> Result<Option<usize>> {
        let slot = *pos;
        let off: i64 = match self.ptr_width {
            4 => i32::from_le_bytes(self.take(pos)?) as i64,
            _ => i64::from_le_bytes(self.take(pos)?),
        };
        if off == 0 {
            return Ok(None);
        }
        let target = i64::try_from(slot).ok()
            .and_then(|slot| slot.checked_add(off))
            .and_then(|target| usize::try_from(target).ok())
            .filter(|&target| target < self.data.len())
            .ok_or_else(|| invalid(format!("pointer at {slot} with offset {off} outside chunk of {}", self.data.len())))?;
        Ok(Some(target))
    }

    fn count_elements(&mut self, n: usize) -> Result<()> {
        self.elements = self.elements.saturating_add(n);
        if self.elements > self.limits.max_elements {
            return Err(invalid(format!("element budget {} exceeded", self.limits.max_elements)));
        }
        Ok(())
    }

    fn read_struct(&mut self, fields: &[PfField], pos: &mut usize, depth: usize) -> Result<PfValue> {
        if depth > self.limits.max_depth {
            return Err(invalid("schema/pointer depth limit exceeded"));
        }
        let mut out = Vec::with_capacity(fields.len());
        for f in fields {
            out.push((f.name.clone(), self.read_field(f, pos, depth)?));
        }
        Ok(PfValue::Struct(out))
    }

    /// Element of a container: a struct (when `fields` is non-empty) or the
    /// primitive named by `type_name`.
    fn read_element(&mut self, f: &PfField, pos: &mut usize, depth: usize) -> Result<PfValue> {
        if !f.fields.is_empty() {
            return self.read_struct(&f.fields, pos, depth + 1);
        }
        let kind = f.type_name.as_deref().ok_or_else(|| invalid(format!("{} {} has no element type", f.kind, f.name)))?;
        let el = PfField { kind: kind.to_string(), name: String::new(), size: None, type_name: None, fields: Vec::new() };
        self.read_field(&el, pos, depth + 1)
    }

    fn read_string(&self, target: usize, wide: bool) -> Result<String> {
        let max = self.limits.max_string;
        if wide {
            let mut units = Vec::new();
            let mut p = target;
            loop {
                let u = u16::from_le_bytes(self.bytes(p, 2)?.try_into().unwrap());
                if u == 0 {
                    break;
                }
                units.push(u);
                p += 2;
                if units.len() > max {
                    return Err(invalid("unterminated wide string"));
                }
            }
            Ok(String::from_utf16_lossy(&units))
        } else {
            let rest = self.bytes(target, 0).map(|_| &self.data[target..])?;
            let end = rest.iter().take(max + 1).position(|&b| b == 0).ok_or_else(|| invalid("unterminated string"))?;
            Ok(String::from_utf8_lossy(&rest[..end]).into_owned())
        }
    }

    fn read_field(&mut self, f: &PfField, pos: &mut usize, depth: usize) -> Result<PfValue> {
        if depth > self.limits.max_depth {
            return Err(invalid("schema/pointer depth limit exceeded"));
        }
        macro_rules! tuple {
            ($ty:ty, $n:expr, $variant:ident) => {{
                let mut v = Vec::with_capacity($n);
                for _ in 0..$n {
                    v.push(<$ty>::from_le_bytes(self.take(pos)?));
                }
                PfValue::$variant(v)
            }};
        }
        Ok(match f.kind.as_str() {
            "Byte" => PfValue::U8(self.take::<1>(pos)?[0]),
            "Byte3" => PfValue::Bytes(self.take::<3>(pos)?.to_vec()),
            "Byte4" => PfValue::Bytes(self.take::<4>(pos)?.to_vec()),
            "Guid" => PfValue::Bytes(self.take::<16>(pos)?.to_vec()),
            "Word" => PfValue::U16(u16::from_le_bytes(self.take(pos)?)),
            "Word3" => tuple!(u16, 3, Words),
            "Dword" | "Token32" => PfValue::U32(self.u32(pos)?),
            "Dword2" => tuple!(u32, 2, Dwords),
            "Dword4" => tuple!(u32, 4, Dwords),
            "Token" | "Token64" => PfValue::U64(u64::from_le_bytes(self.take(pos)?)),
            "Float" => PfValue::F32(f32::from_le_bytes(self.take(pos)?)),
            "Float2" => tuple!(f32, 2, Floats),
            "Float3" => tuple!(f32, 3, Floats),
            "Float4" => tuple!(f32, 4, Floats),
            "Struct" => self.read_struct(&f.fields, pos, depth + 1)?,
            "CharPtr" | "WcharPtr" | "FileName" | "FileRef" => match self.ptr(pos)? {
                None => PfValue::Null,
                Some(t) => PfValue::Text(self.read_string(t, f.kind != "CharPtr")?),
            },
            "Ptr" => match self.ptr(pos)? {
                None => PfValue::Null,
                Some(mut t) => self.read_element(f, &mut t, depth)?,
            },
            "Array" => {
                let n = f.size.ok_or_else(|| invalid(format!("Array {} without Size", f.name)))? as usize;
                self.count_elements(n)?;
                let mut v = Vec::with_capacity(n.min(4096));
                for _ in 0..n {
                    v.push(self.read_element(f, pos, depth)?);
                }
                PfValue::Array(v)
            }
            "ArrayPtr" | "List" => {
                let n = self.u32(pos)? as usize;
                let target = self.ptr(pos)?;
                self.count_elements(n)?;
                match target {
                    None if n == 0 => PfValue::Array(Vec::new()),
                    None => return Err(invalid(format!("{} {} has count {n} but null pointer", f.kind, f.name))),
                    Some(mut t) => {
                        let mut v = Vec::with_capacity(n.min(4096));
                        for _ in 0..n {
                            if f.kind == "List" {
                                v.push(match self.ptr(&mut t)? {
                                    None => PfValue::Null,
                                    Some(mut e) => self.read_element(f, &mut e, depth)?,
                                });
                            } else {
                                v.push(self.read_element(f, &mut t, depth)?);
                            }
                        }
                        PfValue::Array(v)
                    }
                }
            }
            other => return Err(Error::new(ErrorKind::Unsupported, format!("packfile field kind {other}"))),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pfschema::parse;

    const XML: &str = r#"<Packfiles><Packfile Name="test"><Chunk Name="demo">
      <Version Id="1" TypeName="Demo">
        <Dword Name="id"/>
        <Float3 Name="pos"/>
        <ArrayPtr Name="items" TypeName="Item">
          <Word Name="kind"/>
          <WcharPtr Name="label"/>
        </ArrayPtr>
        <Ptr Name="extra" TypeName="Extra"><Token64 Name="token"/></Ptr>
        <List Name="refs" TypeName="Ref"><Byte Name="b"/></List>
        <ArrayPtr Name="raw" TypeName="Byte"/>
        <CharPtr Name="name"/>
        <Ptr Name="none" TypeName="Dword"/>
      </Version></Chunk></Packfile></Packfiles>"#;

    /// Builds a synthetic chunk payload; pointers are relative to their slot.
    struct Builder {
        buf: Vec<u8>,
        w: usize,
        patches: Vec<(usize, usize)>, // (slot, target)
    }

    impl Builder {
        fn new(w: usize) -> Self {
            Self { buf: Vec::new(), w, patches: Vec::new() }
        }
        fn u8(&mut self, v: u8) {
            self.buf.push(v)
        }
        fn u16(&mut self, v: u16) {
            self.buf.extend_from_slice(&v.to_le_bytes())
        }
        fn u32(&mut self, v: u32) {
            self.buf.extend_from_slice(&v.to_le_bytes())
        }
        fn f32(&mut self, v: f32) {
            self.buf.extend_from_slice(&v.to_le_bytes())
        }
        fn ptr_slot(&mut self) -> usize {
            let s = self.buf.len();
            self.buf.extend(std::iter::repeat_n(0, self.w));
            s
        }
        fn point(&mut self, slot: usize, target: usize) {
            self.patches.push((slot, target));
        }
        fn here(&self) -> usize {
            self.buf.len()
        }
        fn finish(mut self) -> Vec<u8> {
            for (slot, target) in self.patches {
                let off = target as i64 - slot as i64;
                if self.w == 4 {
                    self.buf[slot..slot + 4].copy_from_slice(&(off as i32).to_le_bytes());
                } else {
                    self.buf[slot..slot + 8].copy_from_slice(&off.to_le_bytes());
                }
            }
            self.buf
        }
    }

    fn build(w: usize) -> Vec<u8> {
        let mut b = Builder::new(w);
        b.u32(42);
        b.f32(1.0);
        b.f32(2.0);
        b.f32(3.0);
        b.u32(2);
        let items = b.ptr_slot();
        let extra = b.ptr_slot();
        b.u32(2);
        let refs = b.ptr_slot();
        b.u32(3);
        let raw = b.ptr_slot();
        let name = b.ptr_slot();
        let _none = b.ptr_slot();

        // items: 2 x (Word, WcharPtr)
        let t = b.here();
        b.point(items, t);
        b.u16(1);
        let l1 = b.ptr_slot();
        b.u16(2);
        let l2 = b.ptr_slot();
        // extra -> Token64
        let t = b.here();
        b.point(extra, t);
        b.buf.extend_from_slice(&0x1122_3344_5566_7788u64.to_le_bytes());
        // refs -> 2 pointers -> Byte each (second null)
        let t = b.here();
        b.point(refs, t);
        let r1 = b.ptr_slot();
        let _r2 = b.ptr_slot();
        let t = b.here();
        b.point(r1, t);
        b.u8(9);
        // raw bytes
        let t = b.here();
        b.point(raw, t);
        b.buf.extend_from_slice(&[7, 8, 9]);
        // strings
        let t = b.here();
        b.point(l1, t);
        for u in "Shaemoor".encode_utf16().chain([0]) {
            b.u16(u);
        }
        let t = b.here();
        b.point(l2, t);
        for u in "Claypool".encode_utf16().chain([0]) {
            b.u16(u);
        }
        let t = b.here();
        b.point(name, t);
        b.buf.extend_from_slice(b"queensdale\0");
        b.finish()
    }

    #[test]
    fn reads_synthetic_chunk_in_both_pointer_widths() {
        let s = parse(XML).unwrap();
        let v = s[0].chunks[0].version(1).unwrap();
        for (w, ptr64) in [(4, false), (8, true)] {
            let data = build(w);
            let out = read_chunk(&data, v, ptr64, Limits::default()).unwrap();
            assert_eq!(out.get("id"), Some(&PfValue::U32(42)));
            assert_eq!(out.get("pos"), Some(&PfValue::Floats(vec![1.0, 2.0, 3.0])));
            let PfValue::Array(items) = out.get("items").unwrap() else { panic!() };
            assert_eq!(items[0].get("label"), Some(&PfValue::Text("Shaemoor".into())));
            assert_eq!(items[1].get("kind"), Some(&PfValue::U16(2)));
            assert_eq!(items[1].get("label"), Some(&PfValue::Text("Claypool".into())));
            assert_eq!(out.get("extra").unwrap().get("token"), Some(&PfValue::U64(0x1122_3344_5566_7788)));
            let PfValue::Array(refs) = out.get("refs").unwrap() else { panic!() };
            assert_eq!(refs[0].get("b"), Some(&PfValue::U8(9)));
            assert_eq!(refs[1], PfValue::Null);
            assert_eq!(out.get("raw"), Some(&PfValue::Array(vec![PfValue::U8(7), PfValue::U8(8), PfValue::U8(9)])));
            assert_eq!(out.get("name"), Some(&PfValue::Text("queensdale".into())));
            assert_eq!(out.get("none"), Some(&PfValue::Null));
        }
    }

    #[test]
    fn honors_explicit_element_budget_even_with_padding() {
        let s = parse(XML).unwrap();
        let v = s[0].chunks[0].version(1).unwrap();
        let mut data = build(8);
        data.resize(data.len() + 1024, 0);
        // Two items, two list entries, and three raw bytes consume seven elements.
        let exact = Limits { max_elements: 7, ..Limits::default() };
        assert!(read_chunk(&data, v, true, exact).is_ok());
        for max_elements in [0, 1, 6] {
            let tight = Limits { max_elements, ..Limits::default() };
            assert!(read_chunk(&data, v, true, tight).is_err());
        }
    }

    #[test]
    fn extreme_64_bit_pointer_offsets_return_errors() {
        let s = parse(XML).unwrap();
        let v = s[0].chunks[0].version(1).unwrap();
        for offset in [i64::MAX, i64::MIN] {
            let mut data = build(8);
            data[20..28].copy_from_slice(&offset.to_le_bytes());
            assert!(read_chunk(&data, v, true, Limits::default()).is_err());
        }
    }

    #[test]
    fn hostile_input_fails_cleanly() {
        let s = parse(XML).unwrap();
        let v = s[0].chunks[0].version(1).unwrap();
        let data = build(4);
        for cut in 0..data.len() {
            let _ = read_chunk(&data[..cut], v, false, Limits::default());
        }
        // Huge count is stopped by the element budget.
        let mut bad = data.clone();
        bad[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(read_chunk(&bad, v, false, Limits::default()).is_err());
        // Pointer outside the chunk.
        let mut bad = data.clone();
        bad[20..24].copy_from_slice(&0x7fff_0000i32.to_le_bytes());
        assert!(read_chunk(&bad, v, false, Limits::default()).is_err());
        // Self-referencing Ptr chain hits the depth limit instead of looping.
        let cyc = parse(r#"<Packfiles><Packfile Name="t"><Chunk Name="c"><Version Id="1" TypeName="N">
            <Ptr Name="next" TypeName="N"><Ptr Name="next" TypeName="N"><Ptr Name="next" TypeName="Dword"/></Ptr></Ptr>
            </Version></Chunk></Packfile></Packfiles>"#).unwrap();
        let cv = cyc[0].chunks[0].version(1).unwrap();
        let tight = Limits { max_depth: 1, ..Limits::default() };
        assert!(read_chunk(&[4, 0, 0, 0, 0xfc, 0xff, 0xff, 0xff], cv, false, tight).is_err());
    }

    #[test]
    fn reference_kinds_are_all_supported() {
        // Every field kind used by the 208039 schema is handled by read_field.
        let s = parse(include_str!("../reference/packfiles-208039.xml")).unwrap();
        fn walk(fs: &[PfField], bad: &mut Vec<String>) {
            const OK: &[&str] = &["Byte", "Byte3", "Byte4", "Guid", "Word", "Word3", "Dword", "Token32", "Dword2", "Dword4",
                "Token", "Token64", "Float", "Float2", "Float3", "Float4", "Struct", "CharPtr", "WcharPtr", "FileName",
                "FileRef", "Ptr", "Array", "ArrayPtr", "List"];
            for f in fs {
                if !OK.contains(&f.kind.as_str()) {
                    bad.push(f.kind.clone());
                }
                if f.fields.is_empty()
                    && let Some(t) = &f.type_name
                    && !OK.contains(&t.as_str())
                    && f.kind != "Struct"
                {
                    bad.push(format!("{}<{t}>", f.kind));
                }
                walk(&f.fields, bad);
            }
        }
        let mut bad = Vec::new();
        for c in s.iter().flat_map(|p| &p.chunks) {
            for v in &c.versions {
                walk(&v.fields, &mut bad);
            }
        }
        bad.sort();
        bad.dedup();
        assert!(bad.is_empty(), "unsupported: {bad:?}");
    }
}
