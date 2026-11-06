//! Build-209026 literal type-zero amount facts, as evaluated for tooltip text.
//!
//! Evidence: `docs/re/209026-mesmer-tooltip-facts.md`. This is a bounded subset
//! of the client formatter, not skill execution. The caller selects the fact,
//! supplies the content-container ordering, and supplies both context inputs.
//! Named variables, definition-key fallback and other fact types are unsupported.

use crate::content::{ContentIndex, ResolvedContent, resolve_reference};
use std::io::{Error, ErrorKind, Result};

/// The first literal keeps its original representation. Float bases truncate
/// toward zero during evaluation; they do not use the contribution rounding rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LiteralAmountBase209026 {
    Integer(i32),
    Float(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiteralAmountFact209026 {
    pub base: LiteralAmountBase209026,
    /// Raw selector passed to the context's attribute getter. Zero selects zero
    /// without calling that getter; other selectors have no inferred name here.
    pub attribute_selector: u32,
    pub attribute_coefficient: f32,
    pub level_exponent: f32,
    pub level_coefficient: f32,
    pub multiplier: i32,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn unsupported(message: &'static str) -> Error {
    Error::new(ErrorKind::Unsupported, message)
}

// All scalar reads follow a checked range of the complete value payload.
fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn target<'a>(
    containers: &[ContentIndex<'a>],
    source: usize,
    entry: u32,
    field: u32,
    length: usize,
) -> Result<ResolvedContent<'a>> {
    let index = containers
        .get(source)
        .ok_or_else(|| invalid("amount fact source container is absent"))?;
    let slot = entry
        .checked_add(field)
        .ok_or_else(|| invalid("amount fact pointer slot overflow"))?;
    let reference = index
        .reference(slot)?
        .ok_or_else(|| unsupported("amount fact requires a direct content reference"))?;
    resolve_reference(containers, source, reference, length)
}

fn literal(bytes: &[u8], at: usize, tag: u32) -> Result<u32> {
    if dword(bytes, at) != tag {
        return Err(unsupported("unsupported amount fact value tag"));
    }
    Ok(dword(bytes, at + 8))
}

fn finite(value: f32) -> Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(unsupported("nonfinite amount fact value or evaluation"))
    }
}

fn truncate(value: f32) -> Result<i32> {
    // Avoid Rust's saturating float cast and the client's invalid-conversion
    // sentinel. The upper bound is exclusive, since i32::MAX rounds up in f32.
    let value = finite(value)?;
    if f64::from(value) < f64::from(i32::MIN) || f64::from(value) >= f64::from(i32::MAX) + 1.0 {
        return Err(unsupported("amount fact conversion outside signed dword"));
    }
    Ok(value as i32)
}

fn round_contribution(value: f32) -> Result<i32> {
    let value = finite(value)?;
    // Source 1412c0e90 adds/subtracts 0.5 in single precision before CVTTSS2SI.
    // This deliberately is not f32::round(), including near-half boundaries.
    truncate(if value > 0.0 {
        value + 0.5
    } else {
        value - 0.5
    })
}

impl LiteralAmountFact209026 {
    /// Read an entry's classified definition (+0x28) and payload (+0x40).
    /// A type-377 definition with data type zero selects six 0x30-byte values.
    /// Classified local/external offset zero remains valid; no file is loaded.
    /// The entry's cached raw +0x48 tag is not the formatter's discriminator.
    pub fn read(
        containers: &[ContentIndex<'_>],
        source_container: usize,
        entry_offset: u32,
    ) -> Result<Self> {
        let definition = target(containers, source_container, entry_offset, 0x28, 0x30)?;
        if dword(definition.bytes, 0x10) != 377 {
            return Err(invalid("amount fact definition is not content type 377"));
        }
        if dword(definition.bytes, 0x2c) != 0 {
            return Err(unsupported("amount fact definition data type is not zero"));
        }
        let payload = target(containers, source_container, entry_offset, 0x40, 0x120)?;
        let bytes = payload.bytes;
        let base = match dword(bytes, 0) {
            0x1b2 => LiteralAmountBase209026::Integer(dword(bytes, 8) as i32),
            0x1b5 => LiteralAmountBase209026::Float(finite(f32::from_bits(dword(bytes, 8)))?),
            _ => return Err(unsupported("unsupported amount fact base tag")),
        };
        Ok(Self {
            base,
            attribute_selector: literal(bytes, 0x30, 0x1b0)?,
            attribute_coefficient: finite(f32::from_bits(literal(bytes, 0x60, 0x1b5)?))?,
            level_exponent: finite(f32::from_bits(literal(bytes, 0x90, 0x1b5)?))?,
            level_coefficient: finite(f32::from_bits(literal(bytes, 0xc0, 0x1b5)?))?,
            multiplier: literal(bytes, 0xf0, 0x1b2)? as i32,
        })
    }

    /// Evaluate the numeric tooltip amount with explicit context inputs.
    /// `selected_attribute_value` is the signed result for `attribute_selector`;
    /// `level` is the unsigned result of the separate level getter.
    /// Contributions round separately before integer addition/multiplication.
    /// Nonfinite values, conversion overflow and integer overflow are errors.
    ///
    /// Uses platform `f32::powf`, not a reimplementation of the client's math
    /// library; last-bit equality for arbitrary exponents is not guaranteed.
    /// No mode, attribute formula, healing execution or recharge is inferred.
    pub fn evaluate(&self, level: u32, selected_attribute_value: i32) -> Result<i32> {
        let base = match self.base {
            LiteralAmountBase209026::Integer(value) => value,
            LiteralAmountBase209026::Float(value) => truncate(value)?,
        };
        let attribute = if self.attribute_selector == 0 {
            0
        } else {
            selected_attribute_value
        };
        let attribute = round_contribution(attribute as f32 * finite(self.attribute_coefficient)?)?;
        let level = finite((level as f32).powf(finite(self.level_exponent)?))?;
        let level = round_contribution(level * finite(self.level_coefficient)?)?;
        level
            .checked_add(attribute)
            .and_then(|value| value.checked_add(base))
            .and_then(|value| value.checked_mul(self.multiplier))
            .ok_or_else(|| unsupported("amount fact integer arithmetic overflow"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packfile::{FLAG_64BIT_POINTERS, Packfile};
    use crate::synthetic::build_packfile;

    // Hand-authored tag/value fixtures from the documented literal schema.
    // Exact original asset bytes and their independent reader check stay in the
    // private REA bundle; these arrays are not copied frozen payloads.
    const PRIMARY: [(u32, u32); 6] = [
        (0x1b2, 120),
        (0x1b0, 6),
        (0x1b5, 1.0f32.to_bits()),
        (0x1b5, 2.0f32.to_bits()),
        (0x1b5, 0.85f32.to_bits()),
        (0x1b2, 1),
    ];
    const CLONE: [(u32, u32); 6] = [
        (0x1b2, 0),
        (0x1b0, 6),
        (0x1b5, 0.1f32.to_bits()),
        (0x1b5, 2.0f32.to_bits()),
        (0x1b5, 0.1f32.to_bits()),
        (0x1b2, 1),
    ];

    fn payload(rows: [(u32, u32); 6]) -> Vec<u8> {
        let mut bytes = vec![0; 0x120];
        for (row, (tag, value)) in rows.into_iter().enumerate() {
            put(&mut bytes, row * 0x30, tag);
            put(&mut bytes, row * 0x30 + 8, value);
        }
        bytes
    }

    fn put(data: &mut [u8], at: usize, value: u32) {
        data[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn pointer(data: &mut [u8], at: usize, value: u64) {
        data[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn pack(
        content: &[u8],
        local: &[u32],
        external: &[(u32, u32)],
        strings: &[u32],
        files: &[u32],
    ) -> Packfile {
        let mut main = vec![0; 0x88];
        for (field, stride, data) in [
            (0x7c, 1, content.to_vec()),
            (
                0x34,
                4,
                local.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
            (
                0x40,
                8,
                external
                    .iter()
                    .flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat())
                    .collect(),
            ),
            (
                0x58,
                4,
                strings.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
            (
                0x4c,
                4,
                files.iter().flat_map(|v| v.to_le_bytes()).collect(),
            ),
            (
                0x70,
                8,
                if strings.is_empty() {
                    vec![]
                } else {
                    vec![0; 8]
                },
            ),
        ] {
            let start = main.len();
            put(&mut main, field, (data.len() / stride) as u32);
            main[field + 4..field + 12]
                .copy_from_slice(&(start as i64 - (field + 4) as i64).to_le_bytes());
            main.extend(data);
        }
        Packfile::new(build_packfile(
            FLAG_64BIT_POINTERS,
            *b"cntc",
            &[(*b"Main", 0, main, vec![])],
        ))
        .unwrap()
    }

    // Entry at 0x20, definition at 0x100, payload at 0x140; pointers at 0x48/0x60.
    fn content(rows: [(u32, u32); 6]) -> Vec<u8> {
        let mut data = vec![0; 0x260];
        pointer(&mut data, 0x48, 0x100);
        pointer(&mut data, 0x60, 0x140);
        put(&mut data, 0x110, 377);
        data[0x140..].copy_from_slice(&payload(rows));
        data
    }

    fn read(data: &[u8]) -> Result<LiteralAmountFact209026> {
        let pf = pack(data, &[0x48, 0x60], &[], &[], &[]);
        LiteralAmountFact209026::read(&[ContentIndex::parse(&pf)?], 0, 0x20)
    }

    #[test]
    fn literal_layout_and_explicit_level_attribute_oracles() {
        let primary = read(&content(PRIMARY)).unwrap();
        let clone = read(&content(CLONE)).unwrap();
        assert_eq!(
            primary,
            LiteralAmountFact209026 {
                base: LiteralAmountBase209026::Integer(120),
                attribute_selector: 6,
                attribute_coefficient: 1.0,
                level_exponent: 2.0,
                level_coefficient: f32::from_bits(0x3f59999a),
                multiplier: 1,
            }
        );
        assert_eq!(
            clone,
            LiteralAmountFact209026 {
                base: LiteralAmountBase209026::Integer(0),
                attribute_selector: 6,
                attribute_coefficient: f32::from_bits(0x3dcccccd),
                level_exponent: 2.0,
                level_coefficient: f32::from_bits(0x3dcccccd),
                multiplier: 1,
            }
        );
        for (level, attribute, expected_primary, expected_clone) in [
            (1, 0, 121, 0),
            (80, 0, 5560, 640),
            (1, 5, 126, 1),
            (1, 100, 221, 10),
            (80, 100, 5660, 650),
        ] {
            assert_eq!(
                primary.evaluate(level, attribute).unwrap(),
                expected_primary
            );
            assert_eq!(clone.evaluate(level, attribute).unwrap(), expected_clone);
        }
    }

    #[test]
    fn classified_external_and_local_zero_offsets_resolve_in_their_containers() {
        let mut entry = vec![0; 0x68];
        let mut definition = vec![0; 0x30];
        put(&mut definition, 0x10, 377);
        let p0 = pack(&entry, &[], &[(0x28, 1), (0x40, 2)], &[], &[]);
        let p1 = pack(&definition, &[], &[], &[], &[]);
        let p2 = pack(&payload(PRIMARY), &[], &[], &[], &[]);
        let indexes = [
            ContentIndex::parse(&p0).unwrap(),
            ContentIndex::parse(&p1).unwrap(),
            ContentIndex::parse(&p2).unwrap(),
        ];
        assert_eq!(
            LiteralAmountFact209026::read(&indexes, 0, 0).unwrap(),
            read(&content(PRIMARY)).unwrap()
        );
        assert!(LiteralAmountFact209026::read(&indexes[..2], 0, 0).is_err());

        // Payload occupies local offset zero; entry after it still resolves zero.
        entry = payload(PRIMARY);
        entry.resize(0x200, 0);
        pointer(&mut entry, 0x180 + 0x28, 0x140);
        put(&mut entry, 0x150, 377);
        let pf = pack(&entry, &[0x1a8, 0x1c0], &[], &[], &[]);
        assert_eq!(
            LiteralAmountFact209026::read(&[ContentIndex::parse(&pf).unwrap()], 0, 0x180).unwrap(),
            read(&content(PRIMARY)).unwrap()
        );
    }

    #[test]
    fn missing_unclassified_string_file_duplicate_and_outside_references_fail() {
        let good = content(PRIMARY);
        for slot in [0x48, 0x60] {
            let other = if slot == 0x48 { 0x60 } else { 0x48 };
            for value in [0, 0x100, u64::MAX] {
                let mut data = good.clone();
                pointer(&mut data, slot as usize, value);
                let pf = pack(&data, &[other], &[], &[], &[]);
                assert!(
                    LiteralAmountFact209026::read(&[ContentIndex::parse(&pf).unwrap()], 0, 0x20)
                        .is_err()
                );
            }
            let mut zero = good.clone();
            pointer(&mut zero, slot as usize, 0);
            for pf in [
                pack(&zero, &[other], &[], &[slot], &[]),
                pack(&good, &[other], &[], &[], &[slot]),
                pack(&good, &[0x48, 0x60, slot], &[], &[], &[]),
                pack(&good, &[0x48, 0x60], &[(slot, 0)], &[], &[]),
                pack(&good, &[other], &[(slot, u32::MAX)], &[], &[]),
            ] {
                assert!(
                    LiteralAmountFact209026::read(&[ContentIndex::parse(&pf).unwrap()], 0, 0x20)
                        .is_err()
                );
            }
            let mut data = good.clone();
            pointer(&mut data, slot as usize, u64::MAX);
            let pf = pack(&data, &[other], &[(slot, 0)], &[], &[]);
            assert!(
                LiteralAmountFact209026::read(&[ContentIndex::parse(&pf).unwrap()], 0, 0x20)
                    .is_err()
            );
        }
        let pf = pack(&good, &[0x48, 0x60], &[], &[], &[]);
        let indexes = [ContentIndex::parse(&pf).unwrap()];
        assert!(LiteralAmountFact209026::read(&indexes, 1, 0x20).is_err());
        assert!(LiteralAmountFact209026::read(&indexes, 0, u32::MAX).is_err());
    }

    #[test]
    fn definition_kind_and_payload_bounds_are_checked_before_scalar_reads() {
        let mut good = content(PRIMARY);
        for (offset, value) in [(0x110, 376), (0x12c, 7)] {
            let mut data = good.clone();
            put(&mut data, offset, value);
            assert!(read(&data).is_err());
        }
        // The source discriminates through the definition, not this cached tag.
        put(&mut good, 0x68, 7);
        assert!(read(&good).is_ok());
        for cut in 0..good.len() {
            assert!(
                read(&good[..cut]).is_err(),
                "accepted truncated container {cut}"
            );
        }
        let mut definition = vec![0; 0x30];
        put(&mut definition, 0x10, 377);
        let mut entry = vec![0; 0x48];
        pointer(&mut entry, 0x40, 0x140);
        let p0 = pack(&entry, &[], &[(0x28, 1), (0x40, 2)], &[], &[]);
        let p2 = pack(&good, &[], &[], &[], &[]);
        for cut in 0..0x30 {
            let p1 = pack(&definition[..cut], &[], &[], &[], &[]);
            let indexes = [
                ContentIndex::parse(&p0).unwrap(),
                ContentIndex::parse(&p1).unwrap(),
                ContentIndex::parse(&p2).unwrap(),
            ];
            assert!(LiteralAmountFact209026::read(&indexes, 0, 0).is_err());
        }
    }

    #[test]
    fn named_variables_wrong_literal_types_and_nonfinite_values_are_rejected() {
        for row in 0..6 {
            for tag in [0, 0x1c0, u32::MAX] {
                let mut data = content(PRIMARY);
                put(&mut data, 0x140 + row * 0x30, tag);
                assert_eq!(read(&data).unwrap_err().kind(), ErrorKind::Unsupported);
            }
        }
        for row in [0, 2, 3, 4] {
            for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let mut data = content(PRIMARY);
                put(&mut data, 0x140 + row * 0x30, 0x1b5);
                put(&mut data, 0x148 + row * 0x30, value.to_bits());
                assert_eq!(read(&data).unwrap_err().kind(), ErrorKind::Unsupported);
            }
        }
    }

    #[test]
    fn single_precision_half_adjustment_and_separate_rounding_match_source() {
        for (value, expected) in [
            (0.0, 0),
            (-0.0, 0),
            (0.5, 1),
            (-0.5, -1),
            (1.5, 2),
            (-1.5, -2),
            // Source adds 0.5 in f32: the nearest value below 0.5 rounds to 1.0.
            (f32::from_bits(0x3effffff), 1),
            (f32::from_bits(0x3efffffe), 0),
            (-f32::from_bits(0x3effffff), -1),
            (-f32::from_bits(0x3efffffe), 0),
        ] {
            assert_eq!(round_contribution(value).unwrap(), expected);
        }
        let mut fact = read(&content(CLONE)).unwrap();
        fact.attribute_coefficient = 0.4;
        fact.level_coefficient = 0.4;
        fact.multiplier = 3;
        assert_eq!(fact.evaluate(1, 1).unwrap(), 0); // Not round((0.4 + 0.4) * 3).
        fact.attribute_selector = 0;
        assert_eq!(fact.evaluate(1, i32::MAX).unwrap(), 0);
        fact.base = LiteralAmountBase209026::Float(-1.9);
        assert_eq!(fact.evaluate(1, 100).unwrap(), -3);
    }

    #[test]
    fn float_base_is_preserved_and_truncated_only_during_evaluation() {
        let mut data = content(PRIMARY);
        put(&mut data, 0x140, 0x1b5);
        put(&mut data, 0x148, 120.9f32.to_bits());
        let fact = read(&data).unwrap();
        assert_eq!(fact.base, LiteralAmountBase209026::Float(120.9));
        assert_eq!(fact.evaluate(1, 0).unwrap(), 121);
        put(&mut data, 0x148, 2147483648.0f32.to_bits());
        assert!(read(&data).unwrap().evaluate(1, 0).is_err());
    }

    #[test]
    fn nonfinite_conversion_power_and_integer_overflow_do_not_saturate_or_wrap() {
        for value in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            2147483648.0,
            -2147483904.0,
        ] {
            assert!(round_contribution(value).is_err());
            assert!(truncate(value).is_err());
        }
        assert_eq!(truncate(-2147483648.0).unwrap(), i32::MIN);
        assert_eq!(truncate(2147483520.0).unwrap(), 2147483520);
        let good = read(&content(PRIMARY)).unwrap();
        for fact in [
            LiteralAmountFact209026 {
                attribute_coefficient: f32::NAN,
                ..good
            },
            LiteralAmountFact209026 {
                level_coefficient: f32::INFINITY,
                ..good
            },
            LiteralAmountFact209026 {
                level_exponent: f32::NAN,
                ..good
            },
            LiteralAmountFact209026 {
                level_exponent: 100.0,
                ..good
            },
            LiteralAmountFact209026 {
                base: LiteralAmountBase209026::Integer(i32::MAX),
                ..good
            },
            LiteralAmountFact209026 {
                multiplier: i32::MAX,
                ..good
            },
        ] {
            assert_eq!(
                fact.evaluate(80, 1).unwrap_err().kind(),
                ErrorKind::Unsupported
            );
        }
        assert!(
            LiteralAmountFact209026 {
                level_exponent: -1.0,
                ..good
            }
            .evaluate(0, 0)
            .is_err()
        );
        assert!(good.evaluate(u32::MAX, 0).is_err());
        assert!(good.evaluate(1, i32::MAX).is_err());
    }
}
