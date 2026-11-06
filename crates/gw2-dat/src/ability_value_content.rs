//! Build-209026 named ability-value lookup, without recursive evaluation.
//!
//! Source: 0x141481d70..0x141481f8d. Only the first ability description and
//! first eligible override group participate. The caller supplies both mode
//! booleans; this reader assigns no gameplay meaning or default to them.

use crate::content::{ContentIndex, ContentReference, ResolvedContent, resolve_reference};
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityValueMode209026 {
    pub first: bool,
    pub second: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityValueLimits209026 {
    /// Bounds the first description's override-group count.
    pub max_groups: usize,
    /// Combined declared rows of the selected group and, only if needed, base.
    /// Other groups' variable arrays are not read or charged to this budget.
    pub max_combined_rows: usize,
    /// Maximum UTF-16 units per name, excluding its terminating zero.
    pub max_name_units: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityValueProvenance209026 {
    /// An input whose tag is not 448 is returned without consulting the skill.
    Original,
    Override {
        group_index: usize,
        row_index: usize,
    },
    Base {
        row_index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedAbilityValue209026<'a> {
    /// Exactly 0x30 bytes; the caller interprets the returned tag separately.
    pub value: ResolvedContent<'a>,
    pub provenance: AbilityValueProvenance209026,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn dword(bytes: &[u8], at: usize) -> u32 {
    // Every caller first checks the containing fixed-size range.
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn range<'a>(
    containers: &[ContentIndex<'a>],
    container: usize,
    offset: usize,
    length: usize,
) -> Result<ResolvedContent<'a>> {
    resolve_reference(
        containers,
        container,
        ContentReference::Local { offset },
        length,
    )
}

fn reference(
    containers: &[ContentIndex<'_>],
    owner: ResolvedContent<'_>,
    field: usize,
) -> Result<Option<ContentReference>> {
    let slot = owner
        .offset
        .checked_add(field)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| invalid("ability value reference slot overflow"))?;
    containers[owner.container_index].reference(slot)
}

fn required<'a>(
    containers: &[ContentIndex<'a>],
    owner: ResolvedContent<'a>,
    field: usize,
    length: usize,
) -> Result<ResolvedContent<'a>> {
    let target = reference(containers, owner, field)?
        .ok_or_else(|| invalid("required ability value reference is null"))?;
    resolve_reference(containers, owner.container_index, target, length)
}

fn array<'a>(
    containers: &[ContentIndex<'a>],
    owner: ResolvedContent<'a>,
    field: usize,
    count: usize,
    stride: usize,
) -> Result<Option<ResolvedContent<'a>>> {
    let length = count
        .checked_mul(stride)
        .ok_or_else(|| invalid("ability value array extent overflow"))?;
    match reference(containers, owner, field)? {
        Some(target) => {
            resolve_reference(containers, owner.container_index, target, length).map(Some)
        }
        None if count == 0 => Ok(None),
        None => Err(invalid("nonempty ability value array is null")),
    }
}

fn name(
    containers: &[ContentIndex<'_>],
    owner: ResolvedContent<'_>,
    field: usize,
    max_units: usize,
) -> Result<String> {
    match reference(containers, owner, field)? {
        Some(ContentReference::String { index }) => {
            // String indexes belong to the container of the pointer slot.
            // Classified index zero is valid and is not a null name.
            containers[owner.container_index].string(index, max_units)
        }
        _ => Err(invalid("ability variable name is not a classified string")),
    }
}

fn eligible(flags: u32, mode: AbilityValueMode209026) -> bool {
    flags & 1 != 0
        || flags & 4 != 0 && mode.first
        || flags & 8 != 0 && mode.second
        || flags & 2 != 0 && !mode.first && !mode.second
}

fn find_value<'a>(
    containers: &[ContentIndex<'a>],
    variables: ResolvedContent<'a>,
    desired: &str,
    hash: u32,
    remaining_rows: &mut usize,
    max_name_units: usize,
) -> Result<Option<(ResolvedContent<'a>, usize)>> {
    let count = usize::try_from(dword(variables.bytes, 0x10))
        .map_err(|_| invalid("ability variable count overflow"))?;
    *remaining_rows = remaining_rows
        .checked_sub(count)
        .ok_or_else(|| invalid("ability variables exceed combined row limit"))?;
    let Some(rows) = array(containers, variables, 8, count, 0x40)? else {
        return Ok(None);
    };
    for (index, bytes) in rows.bytes.as_chunks::<0x40>().0.iter().enumerate() {
        // A different raw hash never reaches the native string comparison.
        if dword(bytes, 8) != hash {
            continue;
        }
        let row = ResolvedContent {
            container_index: rows.container_index,
            offset: rows.offset + index * 0x40,
            bytes,
        };
        if name(containers, row, 0, max_name_units)?.eq_ignore_ascii_case(desired) {
            return Ok(Some((
                ResolvedContent {
                    container_index: row.container_index,
                    offset: row.offset + 0x10,
                    bytes: &row.bytes[0x10..0x40],
                },
                index,
            )));
        }
    }
    Ok(None)
}

/// Resolve one checked 0x30-byte value. Non-448 tags pass through before the
/// skill, mode or lookup limits are consulted. Named values require type 64,
/// ability kind 0, a nonempty description array and its first description only.
///
/// Exactly the first eligible group is searched. A miss falls back to base;
/// later groups are never searched. Base references are read only on fallback,
/// preserving the native override-success short circuit. Group-table and
/// visited-variable-array extents are checked in full; unrelated description
/// rows, later group arrays, and names with different raw hashes are not read.
///
/// Local/external references retain the owning container's classification.
/// ASCII-only case-insensitive equality AND the raw hash must match. A returned
/// named value is not recursively resolved. Missing names, malformed consulted
/// data, absent containers and unsupported skill kinds are errors; no files load.
pub fn resolve_ability_value209026<'a>(
    containers: &[ContentIndex<'a>],
    value_container: usize,
    value_offset: u32,
    skill_container: usize,
    skill_offset: u32,
    mode: AbilityValueMode209026,
    limits: AbilityValueLimits209026,
) -> Result<ResolvedAbilityValue209026<'a>> {
    let value = range(containers, value_container, value_offset as usize, 0x30)?;
    if dword(value.bytes, 0) != 448 {
        return Ok(ResolvedAbilityValue209026 {
            value,
            provenance: AbilityValueProvenance209026::Original,
        });
    }
    let desired = name(containers, value, 8, limits.max_name_units)?;
    let hash = dword(value.bytes, 0x10);
    let skill = range(containers, skill_container, skill_offset as usize, 0x68)?;
    if dword(skill.bytes, 0x10) != 64 {
        return Err(invalid("ability value skill is not content type64"));
    }
    if dword(skill.bytes, 0x58) != 0 {
        return Err(Error::new(
            ErrorKind::Unsupported,
            "ability value skill kind is not zero",
        ));
    }
    let body = required(containers, skill, 0x60, 0xb4)?;
    if dword(body.bytes, 0xb0) == 0 {
        return Err(invalid("ability has no first description"));
    }
    // The count can exceed one; later description rows are deliberately ignored.
    let description = required(containers, body, 0xa8, 0x2c)?;
    let group_count = usize::try_from(dword(description.bytes, 0x28))
        .map_err(|_| invalid("ability override group count overflow"))?;
    if group_count > limits.max_groups {
        return Err(invalid("ability override groups exceed limit"));
    }
    let mut remaining_rows = limits.max_combined_rows;
    if let Some(groups) = array(containers, description, 0x20, group_count, 0x18)? {
        for (group_index, bytes) in groups.bytes.as_chunks::<0x18>().0.iter().enumerate() {
            if !eligible(dword(bytes, 0), mode) {
                continue;
            }
            let group = ResolvedContent {
                container_index: groups.container_index,
                offset: groups.offset + group_index * 0x18,
                bytes,
            };
            if let Some((value, row_index)) = find_value(
                containers,
                group,
                &desired,
                hash,
                &mut remaining_rows,
                limits.max_name_units,
            )? {
                return Ok(ResolvedAbilityValue209026 {
                    value,
                    provenance: AbilityValueProvenance209026::Override {
                        group_index,
                        row_index,
                    },
                });
            }
            break;
        }
    }
    let base = required(containers, description, 8, 0x14)?;
    let (value, row_index) = find_value(
        containers,
        base,
        &desired,
        hash,
        &mut remaining_rows,
        limits.max_name_units,
    )?
    .ok_or_else(|| invalid("named ability variable was not found"))?;
    Ok(ResolvedAbilityValue209026 {
        value,
        provenance: AbilityValueProvenance209026::Base { row_index },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        packfile::{FLAG_64BIT_POINTERS, Packfile},
        synthetic::build_packfile,
    };

    const SKILL: usize = 0x40;
    const BODY: usize = 0xb0;
    const DESC: usize = 0x170;
    const BASE: usize = 0x1b0;
    const GROUPS: usize = 0x1d0;
    const BASE_ROWS: usize = 0x230;
    const FIRST_ROWS: usize = 0x2c0;
    const SECOND_ROWS: usize = 0x350;
    const HASH: u32 = 0xa1b2c3d4;
    const MODE: AbilityValueMode209026 = AbilityValueMode209026 {
        first: false,
        second: false,
    };
    const LIMITS: AbilityValueLimits209026 = AbilityValueLimits209026 {
        max_groups: 2,
        max_combined_rows: 4,
        max_name_units: 20,
    };

    fn put(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn pointer(bytes: &mut [u8], at: usize, value: usize) {
        bytes[at..at + 8].copy_from_slice(&(value as u64).to_le_bytes());
    }

    struct Fixture {
        bytes: Vec<u8>,
        local: Vec<u32>,
        external: Vec<(u32, u32)>,
        string_slots: Vec<u32>,
        names: Vec<Vec<u16>>,
    }

    impl Fixture {
        fn new(length: usize, names: &[&str]) -> Self {
            Self {
                bytes: vec![0; length],
                local: vec![],
                external: vec![],
                string_slots: vec![],
                names: names.iter().map(|s| s.encode_utf16().collect()).collect(),
            }
        }
        fn local(&mut self, slot: usize, offset: usize) {
            self.local.push(slot as u32);
            pointer(&mut self.bytes, slot, offset);
        }
        fn external(&mut self, slot: usize, container: u32, offset: usize) {
            self.external.push((slot as u32, container));
            pointer(&mut self.bytes, slot, offset);
        }
        fn name(&mut self, slot: usize, index: usize, hash: u32) {
            self.string_slots.push(slot as u32);
            pointer(&mut self.bytes, slot, index);
            put(&mut self.bytes, slot + 8, hash);
        }
        fn variable(&mut self, at: usize, name: usize, hash: u32, tag: u32, payload: u32) {
            self.name(at, name, hash);
            put(&mut self.bytes, at + 0x10, tag);
            put(&mut self.bytes, at + 0x18, payload);
        }
        fn pack(&self) -> Packfile {
            let mut main = vec![0; 0x88];
            fn append(main: &mut Vec<u8>, field: usize, bytes: &[u8], stride: usize) -> usize {
                let at = main.len();
                put(main, field, (bytes.len() / stride) as u32);
                main[field + 4..field + 12]
                    .copy_from_slice(&(at as i64 - (field + 4) as i64).to_le_bytes());
                main.extend_from_slice(bytes);
                at
            }
            append(&mut main, 0x7c, &self.bytes, 1);
            append(
                &mut main,
                0x34,
                &self
                    .local
                    .iter()
                    .flat_map(|n| n.to_le_bytes())
                    .collect::<Vec<_>>(),
                4,
            );
            append(
                &mut main,
                0x40,
                &self
                    .external
                    .iter()
                    .flat_map(|(a, b)| a.to_le_bytes().into_iter().chain(b.to_le_bytes()))
                    .collect::<Vec<_>>(),
                8,
            );
            append(
                &mut main,
                0x58,
                &self
                    .string_slots
                    .iter()
                    .flat_map(|n| n.to_le_bytes())
                    .collect::<Vec<_>>(),
                4,
            );
            let table = append(&mut main, 0x70, &vec![0; self.names.len() * 8], 8);
            for (index, units) in self.names.iter().enumerate() {
                let slot = table + index * 8;
                let target = main.len();
                main[slot..slot + 8].copy_from_slice(&(target as i64 - slot as i64).to_le_bytes());
                for unit in units.iter().copied().chain([0]) {
                    main.extend_from_slice(&unit.to_le_bytes());
                }
            }
            Packfile::new(build_packfile(
                FLAG_64BIT_POINTERS,
                *b"cntc",
                &[(*b"Main", 0, main, vec![])],
            ))
            .unwrap()
        }
    }

    fn fixture() -> Fixture {
        // Hand-authored shape; no frozen asset bytes.
        let mut f = Fixture::new(0x3a0, &["Power", "pOwEr", "missing", "POWER"]);
        put(&mut f.bytes, 0, 448);
        f.name(8, 0, HASH);
        put(&mut f.bytes, SKILL + 0x10, 64);
        f.local(SKILL + 0x60, BODY);
        f.local(BODY + 0xa8, DESC);
        put(&mut f.bytes, BODY + 0xb0, 1);
        f.local(DESC + 8, BASE);
        f.local(DESC + 0x20, GROUPS);
        put(&mut f.bytes, DESC + 0x28, 2);
        f.local(BASE + 8, BASE_ROWS);
        put(&mut f.bytes, BASE + 0x10, 1);
        f.variable(BASE_ROWS, 1, HASH, 437, 11);
        for (group, rows, name, result) in [
            (GROUPS, FIRST_ROWS, 2, 22),
            (GROUPS + 24, SECOND_ROWS, 3, 33),
        ] {
            put(&mut f.bytes, group, 1);
            f.local(group + 8, rows);
            put(&mut f.bytes, group + 0x10, 1);
            f.variable(rows, name, HASH, 437, result);
        }
        f
    }

    fn resolved(
        f: &Fixture,
        mode: AbilityValueMode209026,
        limits: AbilityValueLimits209026,
    ) -> Result<(AbilityValueProvenance209026, usize, u32, u32)> {
        let pf = f.pack();
        let indexes = [ContentIndex::parse(&pf)?];
        let result = resolve_ability_value209026(&indexes, 0, 0, 0, SKILL as u32, mode, limits)?;
        assert_eq!(result.value.bytes.len(), 0x30);
        Ok((
            result.provenance,
            result.value.offset,
            dword(result.value.bytes, 0),
            dword(result.value.bytes, 8),
        ))
    }

    #[test]
    fn ability_value_literal_passthrough_does_not_read_invalid_skill_or_names() {
        let mut f = Fixture::new(0x30, &[]);
        put(&mut f.bytes, 0, u32::MAX);
        pointer(&mut f.bytes, 8, usize::MAX); // Deliberately unclassified.
        let pf = f.pack();
        let indexes = [ContentIndex::parse(&pf).unwrap()];
        let value = resolve_ability_value209026(
            &indexes,
            0,
            0,
            usize::MAX,
            u32::MAX,
            MODE,
            AbilityValueLimits209026 {
                max_groups: 0,
                max_combined_rows: 0,
                max_name_units: 0,
            },
        )
        .unwrap();
        assert_eq!(value.provenance, AbilityValueProvenance209026::Original);
        assert_eq!(value.value.bytes, f.bytes);
        assert!(resolve_ability_value209026(&indexes, 0, 1, 0, 0, MODE, LIMITS).is_err());
    }

    #[test]
    fn ability_value_first_eligible_miss_uses_base_not_later_group_or_description() {
        let mut f = fixture();
        // Later descriptions would be outside the buffer. They must not be read.
        put(&mut f.bytes, BODY + 0xb0, u32::MAX);
        // The second group's array is malformed but never selected.
        pointer(&mut f.bytes, GROUPS + 24 + 8, usize::MAX);
        assert_eq!(
            resolved(&f, MODE, LIMITS).unwrap(),
            (
                AbilityValueProvenance209026::Base { row_index: 0 },
                BASE_ROWS + 0x10,
                437,
                11
            )
        );
        // Finding the first override must not even classify an invalid base.
        pointer(&mut f.bytes, FIRST_ROWS, 1);
        f.local.retain(|&slot| slot != (DESC + 8) as u32);
        pointer(&mut f.bytes, DESC + 8, usize::MAX);
        assert_eq!(
            resolved(&f, MODE, LIMITS).unwrap().0,
            AbilityValueProvenance209026::Override {
                group_index: 0,
                row_index: 0
            }
        );
    }

    #[test]
    fn ability_value_mode_flags_choose_first_group_with_both_false_semantics() {
        for (first, second, flags, selected) in [
            (false, false, 2, true),
            (true, false, 2, false),
            (false, true, 2, false),
            (true, true, 2, false),
            (false, false, 4, false),
            (true, false, 4, true),
            (false, true, 4, false),
            (true, true, 4, true),
            (false, false, 8, false),
            (true, false, 8, false),
            (false, true, 8, true),
            (true, true, 8, true),
            (false, false, 1, true),
            (true, true, 1, true),
            (false, false, 0x10, false),
        ] {
            let mut f = fixture();
            put(&mut f.bytes, GROUPS, flags);
            pointer(&mut f.bytes, FIRST_ROWS, 1);
            let result = resolved(&f, AbilityValueMode209026 { first, second }, LIMITS).unwrap();
            assert_eq!(
                result.0,
                AbilityValueProvenance209026::Override {
                    group_index: if selected { 0 } else { 1 },
                    row_index: 0,
                },
                "mode {first}/{second}, flags {flags}"
            );
        }
    }

    #[test]
    fn ability_value_matching_requires_hash_and_ascii_only_case_equality() {
        let mut f = fixture();
        // A same-hash name collision is skipped, then a same-name/wrong-hash row.
        put(&mut f.bytes, BASE + 0x10, 2);
        f.variable(BASE_ROWS + 64, 1, HASH.wrapping_add(1), 437, 99);
        pointer(&mut f.bytes, BASE_ROWS, 2);
        assert!(resolved(&f, MODE, LIMITS).is_err());
        put(&mut f.bytes, BASE_ROWS + 64 + 8, HASH);
        assert_eq!(
            resolved(&f, MODE, LIMITS).unwrap().0,
            AbilityValueProvenance209026::Base { row_index: 1 }
        );
        f.names[0] = "Ä".encode_utf16().collect();
        f.names[1] = "ä".encode_utf16().collect();
        assert!(resolved(&f, MODE, LIMITS).is_err());
        f.names[1] = f.names[0].clone();
        assert!(resolved(&f, MODE, LIMITS).is_ok());
    }

    #[test]
    fn ability_value_external_refs_and_string_zero_use_each_owning_container() {
        let mut a = Fixture::new(0xa8, &["Power"]);
        put(&mut a.bytes, 0, 448);
        a.name(8, 0, HASH);
        put(&mut a.bytes, SKILL + 0x10, 64);
        a.external(SKILL + 0x60, 1, 0);
        let mut b = Fixture::new(0xb4, &["wrong body string"]);
        b.external(0xa8, 2, 0);
        put(&mut b.bytes, 0xb0, 1);
        let mut c = Fixture::new(0x30, &["wrong description string"]);
        c.external(8, 3, 0);
        c.external(0x20, 4, 0);
        put(&mut c.bytes, 0x28, 1);
        let mut base = Fixture::new(0x60, &["POWER"]);
        base.local(8, 0x20);
        put(&mut base.bytes, 0x10, 1);
        base.variable(0x20, 0, HASH, 437, 77);
        let mut groups = Fixture::new(0x18, &["wrong group string"]);
        put(&mut groups.bytes, 0, 1);
        groups.external(8, 5, 0);
        put(&mut groups.bytes, 0x10, 1);
        let mut rows = Fixture::new(0x40, &["power"]);
        rows.variable(0, 0, HASH, 437, 88);
        for hit in [true, false] {
            if !hit {
                rows.names[0] = "miss".encode_utf16().collect();
            }
            let packs = [&a, &b, &c, &base, &groups, &rows].map(Fixture::pack);
            let indexes: Vec<_> = packs
                .iter()
                .map(|p| ContentIndex::parse(p).unwrap())
                .collect();
            let result =
                resolve_ability_value209026(&indexes, 0, 0, 0, SKILL as u32, MODE, LIMITS).unwrap();
            assert_eq!(result.value.container_index, if hit { 5 } else { 3 });
            assert_eq!(dword(result.value.bytes, 8), if hit { 88 } else { 77 });
            assert!(
                resolve_ability_value209026(&indexes[..5], 0, 0, 0, SKILL as u32, MODE, LIMITS)
                    .is_err()
            );
        }
    }

    #[test]
    fn ability_value_limits_charge_selected_and_fallback_rows_only() {
        let mut f = fixture();
        for limits in [
            AbilityValueLimits209026 {
                max_groups: 1,
                ..LIMITS
            },
            AbilityValueLimits209026 {
                max_combined_rows: 1,
                ..LIMITS
            },
            AbilityValueLimits209026 {
                max_name_units: 4,
                ..LIMITS
            },
        ] {
            assert!(resolved(&f, MODE, limits).is_err());
        }
        assert!(
            resolved(
                &f,
                MODE,
                AbilityValueLimits209026 {
                    max_combined_rows: 2,
                    ..LIMITS
                }
            )
            .is_ok()
        );
        pointer(&mut f.bytes, FIRST_ROWS, 1);
        assert!(
            resolved(
                &f,
                MODE,
                AbilityValueLimits209026 {
                    max_combined_rows: 1,
                    ..LIMITS
                }
            )
            .is_ok()
        );
        put(&mut f.bytes, GROUPS + 0x10, u32::MAX);
        assert!(resolved(&f, MODE, LIMITS).is_err());
    }

    #[test]
    fn ability_value_empty_selected_group_still_falls_back_without_later_groups() {
        let mut f = fixture();
        f.local.retain(|&slot| slot != (GROUPS + 8) as u32);
        pointer(&mut f.bytes, GROUPS + 8, 0);
        put(&mut f.bytes, GROUPS + 0x10, 0);
        let one_row = AbilityValueLimits209026 {
            max_combined_rows: 1,
            ..LIMITS
        };
        assert_eq!(
            resolved(&f, MODE, one_row).unwrap().0,
            AbilityValueProvenance209026::Base { row_index: 0 }
        );
        // No override groups is also valid, with a null array and zero budget.
        f.local.retain(|&slot| slot != (DESC + 0x20) as u32);
        pointer(&mut f.bytes, DESC + 0x20, 0);
        put(&mut f.bytes, DESC + 0x28, 0);
        assert!(
            resolved(
                &f,
                MODE,
                AbilityValueLimits209026 {
                    max_groups: 0,
                    ..one_row
                }
            )
            .is_ok()
        );
        f.local.retain(|&slot| slot != (BASE + 8) as u32);
        pointer(&mut f.bytes, BASE + 8, 0);
        put(&mut f.bytes, BASE + 0x10, 0);
        assert!(
            resolved(&f, MODE, one_row).is_err(),
            "empty base cannot supply the name"
        );
    }

    #[test]
    fn ability_value_returned_named_value_is_not_recursively_resolved() {
        let mut f = fixture();
        put(&mut f.bytes, BASE_ROWS + 0x10, 448);
        // Its name pointer is malformed, which would fail a recursive lookup.
        pointer(&mut f.bytes, BASE_ROWS + 0x18, usize::MAX);
        let result = resolved(&f, MODE, LIMITS).unwrap();
        assert_eq!(
            result.0,
            AbilityValueProvenance209026::Base { row_index: 0 }
        );
        assert_eq!(result.2, 448);
    }

    #[test]
    fn ability_value_malformed_consulted_content_fails_closed() {
        for (slot, value) in [
            (SKILL + 0x10, 63),
            (SKILL + 0x58, 1),
            (BODY + 0xb0, 0),
            (DESC + 0x28, u32::MAX),
            (BASE + 0x10, u32::MAX),
        ] {
            let mut f = fixture();
            put(&mut f.bytes, slot, value);
            assert!(resolved(&f, MODE, LIMITS).is_err(), "slot {slot:x}");
        }
        for slot in [
            SKILL + 0x60,
            BODY + 0xa8,
            DESC + 8,
            DESC + 0x20,
            BASE + 8,
            GROUPS + 8,
        ] {
            let mut f = fixture();
            let last = f.bytes.len() - 1;
            pointer(&mut f.bytes, slot, last);
            assert!(resolved(&f, MODE, LIMITS).is_err(), "short target {slot:x}");
            let mut f = fixture();
            f.local.retain(|&s| s != slot as u32);
            assert!(resolved(&f, MODE, LIMITS).is_err(), "unclassified {slot:x}");
            pointer(&mut f.bytes, slot, 0);
            assert!(resolved(&f, MODE, LIMITS).is_err(), "null {slot:x}");
        }
        let mut f = fixture();
        f.string_slots.retain(|&s| s != 8);
        assert!(
            resolved(&f, MODE, LIMITS).is_err(),
            "zero index without classification is null"
        );
        let mut f = fixture();
        f.local.push(8);
        assert!(
            resolved(&f, MODE, LIMITS).is_err(),
            "conflicting name classification"
        );
        let mut f = fixture();
        f.names[0] = vec![0xd800];
        assert!(resolved(&f, MODE, LIMITS).is_err(), "invalid UTF16");
        let mut f = fixture();
        pointer(&mut f.bytes, 8, usize::MAX);
        assert!(resolved(&f, MODE, LIMITS).is_err(), "invalid string index");
    }
}
