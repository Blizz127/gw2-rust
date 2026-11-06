//! Selected build-209026 `anic` / `seqn` v0 animation records.
//!
//! Values remain raw content: this reader does not derive cast durations,
//! execute effects, select a locale, or load an archive. Source and limitations:
//! `docs/re/209026-animation-sequence-reader.md`.

use crate::packfile::Packfile;
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationEvent209026 {
    pub kind: u8,
    pub raw_dword_1: u32,
    pub raw_dword_5: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationSegment209026 {
    pub kind: u8,
    pub flags: u8,
    /// First Dword at the optional signed-relative pointer in segment +9.
    pub data_at_9_first: Option<u32>,
    /// First Dword at the optional signed-relative pointer in segment +17.
    pub data_at_17_first: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationSequence209026 {
    pub animation_token: u64,
    pub variant_token: u64,
    /// Original order is retained, including coincident and unknown events.
    pub events: Vec<AnimationEvent209026>,
    pub segments: Vec<AnimationSegment209026>,
}

/// Borrowed index of one `seqn` chunk. Only a selected variant is decoded.
pub struct AnimationSequences209026<'a> {
    payload: &'a [u8],
    rows: &'a [u8],
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn bytes(data: &[u8], offset: usize, length: usize) -> Result<&[u8]> {
    offset
        .checked_add(length)
        .and_then(|end| data.get(offset..end))
        .ok_or_else(|| invalid("animation field extends past payload"))
}

fn dword(data: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes(data, offset, 4)?.try_into().unwrap(),
    ))
}

fn qword(data: &[u8], offset: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(
        bytes(data, offset, 8)?.try_into().unwrap(),
    ))
}

fn pointer(data: &[u8], slot: usize) -> Result<Option<usize>> {
    let relative = i64::from_le_bytes(bytes(data, slot, 8)?.try_into().unwrap());
    if relative == 0 {
        return Ok(None);
    }
    // i128 also admits i64::MIN without negating it or wrapping a usize.
    let target = slot as i128 + i128::from(relative);
    let target = usize::try_from(target).map_err(|_| invalid("animation pointer overflow"))?;
    bytes(data, target, 0)?;
    Ok(Some(target))
}

fn array(data: &[u8], slot: usize, count: usize, stride: usize) -> Result<&[u8]> {
    let extent = count
        .checked_mul(stride)
        .ok_or_else(|| invalid("animation array extent overflow"))?;
    match pointer(data, slot)? {
        Some(offset) => bytes(data, offset, extent),
        None if count == 0 => Ok(&[]),
        None => Err(invalid("nonempty animation array is null")),
    }
}

fn sorted_unique(rows: &[u8], stride: usize) -> Result<()> {
    let mut previous = None;
    for row in rows.chunks_exact(stride) {
        let token = qword(row, 0)?;
        if previous.is_some_and(|value| value >= token) {
            return Err(invalid("animation token table is not sorted and unique"));
        }
        previous = Some(token);
    }
    Ok(())
}

fn find_row(rows: &[u8], stride: usize, token: u64) -> Option<usize> {
    let mut low = 0;
    let mut high = rows.len() / stride;
    while low < high {
        let middle = low + (high - low) / 2;
        let key = qword(rows, middle * stride).ok()?;
        if key < token {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    (low < rows.len() / stride && qword(rows, low * stride).ok()? == token).then_some(low * stride)
}

impl<'a> AnimationSequences209026<'a> {
    /// Require a 64-bit `anic` pack and exactly one `seqn` v0 chunk.
    /// The explicit row limit bounds index validation work; no rows are copied.
    /// Unsorted/duplicate keys are rejected as a reader policy, without trying
    /// to reproduce an ambiguous lookup on malformed or unsupported input.
    pub fn read(packfile: &'a Packfile, max_sequences: usize) -> Result<Self> {
        if packfile.r#type() != *b"anic" || !packfile.is_64bit() {
            return Err(invalid("expected 64-bit anic pack"));
        }
        let mut payload = None;
        for chunk in packfile.chunks() {
            let chunk = chunk?;
            if chunk.name() == *b"seqn" {
                if payload.is_some() || chunk.version() != 0 {
                    return Err(invalid("expected one seqn version zero chunk"));
                }
                payload = Some(chunk.bytes());
            }
        }
        let payload = payload.ok_or_else(|| invalid("animation seqn chunk is absent"))?;
        let count = dword(payload, 0)? as usize;
        if count > max_sequences {
            return Err(invalid("animation sequence count exceeds limit"));
        }
        let rows = array(payload, 4, count, 20)?;
        sorted_unique(rows, 20)?;
        Ok(Self { payload, rows })
    }

    /// Exact token pair only: no alternate, variant-zero or nearby-row fallback.
    /// `max_variants` bounds the selected token's variant table; `max_items`
    /// bounds the selected row's combined event/segment count before allocation.
    /// Unknown kinds/flags and all event Dwords remain unchanged.
    pub fn find(
        &self,
        animation_token: u64,
        variant_token: u64,
        max_variants: usize,
        max_items: usize,
    ) -> Result<Option<AnimationSequence209026>> {
        let Some(row) = find_row(self.rows, 20, animation_token) else {
            return Ok(None);
        };
        let count = dword(self.rows, row + 8)? as usize;
        if count > max_variants {
            return Err(invalid("animation variant count exceeds limit"));
        }
        let root = pointer(self.payload, 4)?.ok_or_else(|| invalid("animation root is null"))?;
        let variants = array(self.payload, root + row + 12, count, 56)?;
        sorted_unique(variants, 56)?;
        let Some(variant) = find_row(variants, 56, variant_token) else {
            return Ok(None);
        };
        let start = pointer(self.payload, root + row + 12)?
            .ok_or_else(|| invalid("animation variants are null"))?
            + variant;
        let segment_count = dword(self.payload, start + 32)? as usize;
        let event_count = dword(self.payload, start + 44)? as usize;
        if segment_count
            .checked_add(event_count)
            .is_none_or(|count| count > max_items)
        {
            return Err(invalid("animation event/segment count exceeds limit"));
        }
        let segment_rows = array(self.payload, start + 36, segment_count, 25)?;
        let event_rows = array(self.payload, start + 48, event_count, 9)?;
        let mut events = Vec::new();
        events
            .try_reserve_exact(event_count)
            .map_err(|_| invalid("animation event allocation failed"))?;
        for event in event_rows.chunks_exact(9) {
            events.push(AnimationEvent209026 {
                kind: event[0],
                raw_dword_1: dword(event, 1)?,
                raw_dword_5: dword(event, 5)?,
            });
        }
        let mut segments = Vec::new();
        segments
            .try_reserve_exact(segment_count)
            .map_err(|_| invalid("animation segment allocation failed"))?;
        if segment_count != 0 {
            let segment_start = pointer(self.payload, start + 36)?
                .ok_or_else(|| invalid("animation segments are null"))?;
            for (index, segment) in segment_rows.chunks_exact(25).enumerate() {
                let first = |field| -> Result<Option<u32>> {
                    pointer(self.payload, segment_start + index * 25 + field)?
                        .map(|offset| dword(self.payload, offset))
                        .transpose()
                };
                segments.push(AnimationSegment209026 {
                    kind: segment[0],
                    flags: segment[5],
                    data_at_9_first: first(9)?,
                    data_at_17_first: first(17)?,
                });
            }
        }
        Ok(Some(AnimationSequence209026 {
            animation_token,
            variant_token,
            events,
            segments,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{packfile::FLAG_64BIT_POINTERS, synthetic::build_packfile};

    fn put(data: &mut [u8], at: usize, value: u32) {
        data[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn token(data: &mut [u8], at: usize, value: u64) {
        data[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    fn ptr(data: &mut [u8], at: usize, target: usize) {
        data[at..at + 8].copy_from_slice(&(target as i64 - at as i64).to_le_bytes());
    }
    fn fixture() -> Vec<u8> {
        // Synthetic layout, with a backward variant pointer and backward data
        // pointers. Two keys and two variants exercise exact pair selection.
        let mut data = vec![0; 400];
        put(&mut data, 0, 2);
        ptr(&mut data, 4, 128);
        token(&mut data, 128, 20);
        put(&mut data, 136, 2);
        ptr(&mut data, 140, 16);
        token(&mut data, 148, u64::MAX);
        // Second key has a valid empty variant table.
        token(&mut data, 16, 0);
        token(&mut data, 72, 99);
        put(&mut data, 16 + 32, 2);
        ptr(&mut data, 16 + 36, 180);
        put(&mut data, 16 + 44, 3);
        ptr(&mut data, 16 + 48, 240);
        data[180] = 0;
        data[185] = 0xf0;
        ptr(&mut data, 189, 300);
        ptr(&mut data, 197, 0); // Nonzero relative pointer to payload offset zero.
        data[205] = 0xff;
        data[210] = 0x01;
        ptr(&mut data, 222, 304);
        put(&mut data, 300, 2067);
        put(&mut data, 304, 0);
        for (i, (kind, a, b)) in [(3, 1067, 0), (0xff, 1067, u32::MAX), (0, 1350, 7)]
            .into_iter()
            .enumerate()
        {
            data[240 + i * 9] = kind;
            put(&mut data, 241 + i * 9, a);
            put(&mut data, 245 + i * 9, b);
        }
        data
    }
    fn pack(data: Vec<u8>) -> Packfile {
        Packfile::new(build_packfile(
            FLAG_64BIT_POINTERS,
            *b"anic",
            &[(*b"seqn", 0, data, vec![])],
        ))
        .unwrap()
    }
    fn read(data: Vec<u8>) -> Result<Option<AnimationSequence209026>> {
        AnimationSequences209026::read(&pack(data), 2)?.find(20, 0, 2, 5)
    }

    #[test]
    fn animation_sequence_exact_pairs_preserve_raw_order_and_signed_pointers() {
        let pack = pack(fixture());
        let index = AnimationSequences209026::read(&pack, 2).unwrap();
        let selected = index.find(20, 0, 2, 5).unwrap().unwrap();
        assert_eq!(
            selected.events,
            [
                AnimationEvent209026 {
                    kind: 3,
                    raw_dword_1: 1067,
                    raw_dword_5: 0
                },
                AnimationEvent209026 {
                    kind: 255,
                    raw_dword_1: 1067,
                    raw_dword_5: u32::MAX
                },
                AnimationEvent209026 {
                    kind: 0,
                    raw_dword_1: 1350,
                    raw_dword_5: 7
                },
            ]
        );
        assert_eq!(
            selected.segments,
            [
                AnimationSegment209026 {
                    kind: 0,
                    flags: 240,
                    data_at_9_first: Some(2067),
                    data_at_17_first: Some(2)
                },
                AnimationSegment209026 {
                    kind: 255,
                    flags: 1,
                    data_at_9_first: None,
                    data_at_17_first: Some(0)
                },
            ]
        );
        let empty = index.find(20, 99, 2, 0).unwrap().unwrap();
        assert!(empty.events.is_empty() && empty.segments.is_empty());
        for (key, variant) in [(0, 0), (19, 0), (21, 0), (20, 1), (20, 100), (u64::MAX, 0)] {
            assert!(index.find(key, variant, 2, 5).unwrap().is_none());
        }
    }

    #[test]
    fn animation_sequence_rejects_limits_bad_extents_and_null_nonempty_arrays() {
        let pack = pack(fixture());
        assert!(AnimationSequences209026::read(&pack, 1).is_err());
        let index = AnimationSequences209026::read(&pack, 2).unwrap();
        assert!(index.find(20, 0, 1, 5).is_err());
        assert!(index.find(20, 0, 2, 4).is_err());
        // Every used pointer slot: root, variants, segments, events, both
        // segment data fields. Extreme signed values must never wrap or panic.
        for slot in [4, 140, 52, 64, 189, 197, 222] {
            for relative in [i64::MIN, i64::MAX, 10000] {
                let mut data = fixture();
                data[slot..slot + 8].copy_from_slice(&relative.to_le_bytes());
                assert!(read(data).is_err(), "slot {slot}, relative {relative}");
            }
        }
        for slot in [4, 140, 52, 64] {
            let mut data = fixture();
            data[slot..slot + 8].fill(0);
            assert!(read(data).is_err());
        }
        for field in [0, 136, 48, 60] {
            let mut data = fixture();
            put(&mut data, field, u32::MAX);
            assert!(read(data).is_err());
        }
        for length in 0..308 {
            assert!(
                read(fixture()[..length].to_vec()).is_err(),
                "length {length}"
            );
        }
        assert!(read(fixture()[..308].to_vec()).is_ok());
    }

    #[test]
    fn animation_sequence_rejects_ambiguous_keys_and_wrong_containers() {
        for (at, value) in [(148, 20), (148, 19), (72, 0)] {
            let mut data = fixture();
            token(&mut data, at, value);
            assert!(read(data).is_err());
        }
        let mut data = fixture();
        token(&mut data, 16, 100);
        assert!(read(data).is_err());
        for (flags, kind, chunks) in [
            (0, *b"anic", vec![(*b"seqn", 0, fixture(), vec![])]),
            (4, *b"cntc", vec![(*b"seqn", 0, fixture(), vec![])]),
            (4, *b"anic", vec![]),
            (4, *b"anic", vec![(*b"seqn", 1, fixture(), vec![])]),
            (
                4,
                *b"anic",
                vec![
                    (*b"seqn", 0, fixture(), vec![]),
                    (*b"seqn", 0, fixture(), vec![]),
                ],
            ),
        ] {
            let pack = Packfile::new(build_packfile(flags, kind, &chunks)).unwrap();
            assert!(AnimationSequences209026::read(&pack, 2).is_err());
        }
        let mut data = vec![0; 12];
        let empty = pack(data.clone());
        assert!(
            AnimationSequences209026::read(&empty, 0)
                .unwrap()
                .find(0, 0, 0, 0)
                .unwrap()
                .is_none()
        );
        // Even an empty nonnull table must point inside or to the end of data.
        ptr(&mut data, 4, 13);
        assert!(AnimationSequences209026::read(&pack(data), 0).is_err());
    }
}
