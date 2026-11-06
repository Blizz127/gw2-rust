//! Build-209026 type-13 damage facts and their numeric tooltip calculation.
//!
//! Source: `docs/re/209026-mesmer-sword-damage-facts.md`. This is a tooltip
//! estimate, not server hit damage. The caller chooses the fact, mode, weapon
//! pair, level-table entries and attribute; no combat defaults are supplied.

use crate::{
    ability_value_content::{
        AbilityValueLimits209026, AbilityValueMode209026, resolve_ability_value209026,
    },
    content::{ContentIndex, ResolvedContent, resolve_reference},
    skill_palette_content::SkillDefinition209026,
};
use std::io::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageTooltipFormula209026 {
    /// Skill flags mask 0x08: clamp base * coefficient * multiplier to zero,
    /// then round. No weapon, attribute, level or global setting is consulted.
    Flat,
    /// Ordinary path: weapon/attribute estimate, clamp to one, round, multiply.
    Weapon,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageFact209026 {
    pub base: i32,
    pub coefficient: f32,
    pub multiplier: i32,
    pub raw_weapon_selector: u32,
    pub formula: DamageTooltipFormula209026,
}

/// Explicit inputs selected by the tooltip caller. Both table values must
/// correspond to its selected level; this struct does not choose a level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageTooltipInputs209026 {
    pub weapon_strength: [u32; 2],
    /// The formatter zero-extends the context's attribute-1 Dword.
    pub attribute_one: u32,
    pub monster_defense_at_level: u32,
    pub monster_attribute_at_level: u32,
    pub base_damage_multiplier: f32,
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn unsupported(message: &'static str) -> Error {
    Error::new(ErrorKind::Unsupported, message)
}

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
        .ok_or_else(|| invalid("damage fact source container is absent"))?;
    let slot = entry
        .checked_add(field)
        .ok_or_else(|| invalid("damage fact reference slot overflow"))?;
    let reference = index
        .reference(slot)?
        .ok_or_else(|| unsupported("damage fact requires a direct reference"))?;
    resolve_reference(containers, source, reference, length)
}

fn finite(value: f32) -> Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(unsupported("nonfinite damage tooltip value or evaluation"))
    }
}

fn round_nonnegative(value: f32) -> Result<u32> {
    // Native helper 1409bf050 adds f32 0.5, then CVTTSS2SI to i64. The
    // formatter keeps the low Dword. Reject overflow instead of truncating it.
    let rounded = finite(finite(value)? + 0.5)?;
    if value < 0.0 || f64::from(rounded) >= f64::from(u32::MAX) + 1.0 {
        return Err(unsupported(
            "damage tooltip conversion outside unsigned dword",
        ));
    }
    Ok(rounded as u32)
}

impl DamageFact209026 {
    /// Read a directly referenced type-377/data-type-13 definition and its
    /// payload. Each scalar is resolved once by the named ability-value reader.
    /// This subset accepts resolved integer434 / float437 / integer434 only.
    /// Limits apply separately to each of the three lookups. No archive loads,
    /// definition-key fallback, recursive lookup or implicit mode is performed.
    pub fn read(
        containers: &[ContentIndex<'_>],
        source_container: usize,
        entry_offset: u32,
        skill_container: usize,
        skill_offset: u32,
        mode: AbilityValueMode209026,
        limits: AbilityValueLimits209026,
    ) -> Result<Self> {
        let skill_index = containers
            .get(skill_container)
            .ok_or_else(|| invalid("damage fact skill container is absent"))?;
        let skill = SkillDefinition209026::read(skill_index, skill_offset)?;
        let definition = target(containers, source_container, entry_offset, 0x28, 0x30)?;
        if dword(definition.bytes, 0x10) != 377 {
            return Err(invalid("damage fact definition is not content type377"));
        }
        if dword(definition.bytes, 0x2c) != 13 {
            return Err(unsupported("damage fact definition data type is not13"));
        }
        let payload = target(containers, source_container, entry_offset, 0x40, 0x94)?;
        let mut values = [0; 3];
        for (i, tag) in [434, 437, 434].into_iter().enumerate() {
            let offset = payload
                .offset
                .checked_add(i * 0x30)
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| invalid("damage value offset overflow"))?;
            let value = resolve_ability_value209026(
                containers,
                payload.container_index,
                offset,
                skill_container,
                skill_offset,
                mode,
                limits,
            )?
            .value;
            if dword(value.bytes, 0) != tag {
                return Err(unsupported("unsupported resolved damage value tag"));
            }
            values[i] = dword(value.bytes, 8);
        }
        Ok(Self {
            base: values[0] as i32,
            coefficient: finite(f32::from_bits(values[1]))?,
            multiplier: values[2] as i32,
            raw_weapon_selector: dword(payload.bytes, 0x90),
            formula: if skill.flags & 0x08 != 0 {
                DamageTooltipFormula209026::Flat
            } else {
                DamageTooltipFormula209026::Weapon
            },
        })
    }

    /// Apply only the fact's selector substitution. Other values, including
    /// zero, preserve the caller's slot; no primary-weapon default is inferred.
    pub fn weapon_selector(&self, caller_slot: u32) -> u32 {
        match self.raw_weapon_selector {
            1 | 5 => 5,
            2 => 8,
            3 => 12,
            4 => 1,
            _ => caller_slot,
        }
    }

    /// Evaluate the numeric tooltip in native single-precision operation order.
    /// Flat facts ignore `inputs`; ordinary facts require them. The ordinary
    /// multiplier is applied after rounding; the flat multiplier before it.
    ///
    /// Bounded domain: nonfinite arithmetic, zero denominator, overflowing
    /// Dword sums, signed weapon-sum overflow and result overflow are errors.
    /// This deliberately rejects native wraparound/invalid-conversion cases.
    /// No target armor, critical chance, hit timing or damage application occurs.
    pub fn evaluate(&self, inputs: Option<DamageTooltipInputs209026>) -> Result<u32> {
        let coefficient = finite(self.coefficient)?;
        if self.formula == DamageTooltipFormula209026::Flat {
            let amount = finite(self.base as f32 * coefficient)?;
            let amount = finite(amount * self.multiplier as f32)?;
            return round_nonnegative(amount.max(0.0));
        }
        let input = inputs.ok_or_else(|| invalid("weapon damage tooltip inputs are required"))?;
        let defense = input
            .monster_defense_at_level
            .checked_add(input.monster_attribute_at_level)
            .filter(|&n| n != 0)
            .ok_or_else(|| unsupported("invalid damage tooltip level-table sum"))?;
        let strength = input.weapon_strength[0]
            .checked_add(input.weapon_strength[1])
            .filter(|&n| n <= i32::MAX as u32)
            .ok_or_else(|| unsupported("damage tooltip weapon sum exceeds signed dword"))?;
        let ratio = input.attribute_one as f32 / defense as f32;
        let mean_plus_base = (strength as i32 as f32 * 0.5) + self.base as f32;
        let scaled_weapon = finite(ratio * mean_plus_base)?;
        let configured_coefficient = finite(finite(input.base_damage_multiplier)? * coefficient)?;
        let estimate = finite(configured_coefficient * scaled_weapon)?.max(1.0);
        let rounded = round_nonnegative(estimate)?;
        let product = i64::from(rounded) * i64::from(self.multiplier);
        u32::try_from(product).map_err(|_| unsupported("damage tooltip multiplier overflow"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        packfile::{FLAG_64BIT_POINTERS, Packfile},
        synthetic::build_packfile,
    };

    fn put(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn pointer(bytes: &mut [u8], at: usize, value: u64) {
        bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    fn pack(content: Vec<u8>, local: &[u32], external: &[(u32, u32)]) -> Packfile {
        let mut main = vec![0; 0x88];
        for (field, stride, bytes) in [
            (0x7c, 1, content),
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
        ] {
            let start = main.len();
            put(&mut main, field, (bytes.len() / stride) as u32);
            main[field + 4..field + 12]
                .copy_from_slice(&(start as i64 - (field + 4) as i64).to_le_bytes());
            main.extend(bytes);
        }
        Packfile::new(build_packfile(
            FLAG_64BIT_POINTERS,
            *b"cntc",
            &[(*b"Main", 0, main, vec![])],
        ))
        .unwrap()
    }
    fn fixture() -> (Vec<u8>, Vec<u8>) {
        let mut source = vec![0; 0xa0];
        put(&mut source, 0x10, 64); // Skill at0; entry at0x40.
        pointer(&mut source, 0x40 + 0x28, 0);
        pointer(&mut source, 0x40 + 0x40, 0x30);
        put(&mut source, 0x40 + 0x48, 999); // Not the authoritative discriminator.
        let mut data = vec![0; 0xc4];
        put(&mut data, 0x10, 377);
        put(&mut data, 0x2c, 13);
        for (at, tag, bits) in [
            (0x30, 434, 1),
            (0x60, 437, 1.5f32.to_bits()),
            (0x90, 434, 2),
        ] {
            put(&mut data, at, tag);
            put(&mut data, at + 8, bits);
        }
        (source, data)
    }
    fn read(source: Vec<u8>, data: Vec<u8>) -> Result<DamageFact209026> {
        let packs = [
            pack(source, &[], &[(0x68, 1), (0x80, 1)]),
            pack(data, &[], &[]),
        ];
        let indexes = packs
            .iter()
            .map(ContentIndex::parse)
            .collect::<Result<Vec<_>>>()?;
        DamageFact209026::read(
            &indexes,
            0,
            0x40,
            0,
            0,
            AbilityValueMode209026 {
                first: false,
                second: false,
            },
            AbilityValueLimits209026 {
                max_groups: 0,
                max_combined_rows: 0,
                max_name_units: 0,
            },
        )
    }
    fn fact() -> DamageFact209026 {
        DamageFact209026 {
            base: 1,
            coefficient: 1.0,
            multiplier: 1,
            raw_weapon_selector: 0,
            formula: DamageTooltipFormula209026::Weapon,
        }
    }
    fn inputs() -> DamageTooltipInputs209026 {
        DamageTooltipInputs209026 {
            weapon_strength: [105, 117],
            attribute_one: 200,
            monster_defense_at_level: 40,
            monster_attribute_at_level: 60,
            base_damage_multiplier: 1.0,
        }
    }

    #[test]
    fn damage_fact_reads_external_definition_at_zero_and_exact_mask() {
        let (mut source, data) = fixture();
        let f = read(source.clone(), data.clone()).unwrap();
        assert_eq!((f.base, f.coefficient, f.multiplier), (1, 1.5, 2));
        assert_eq!(f.formula, DamageTooltipFormula209026::Weapon);
        put(&mut source, 0x38, 0x100); // Bit8 is not mask8.
        assert_eq!(
            read(source.clone(), data.clone()).unwrap().formula,
            DamageTooltipFormula209026::Weapon
        );
        put(&mut source, 0x38, 0x08);
        assert_eq!(
            read(source, data).unwrap().formula,
            DamageTooltipFormula209026::Flat
        );
    }
    #[test]
    fn damage_fact_rejects_wrong_types_tags_short_references_and_nonfinite_coefficient() {
        for (offset, value) in [
            (0x10, 64),
            (0x2c, 0),
            (0x30, 437),
            (0x60, 448),
            (0x90, 437),
            (0x68, f32::NAN.to_bits()),
        ] {
            let (source, mut data) = fixture();
            put(&mut data, offset, value);
            assert!(read(source, data).is_err(), "offset {offset:x}");
        }
        let (source, data) = fixture();
        for end in [0, 0x2f, 0x30, 0xc3] {
            assert!(read(source.clone(), data[..end].to_vec()).is_err());
        }
        let (mut source, data) = fixture();
        pointer(&mut source, 0x80, u64::MAX);
        assert!(read(source, data).is_err());
    }
    #[test]
    fn damage_fact_selector_has_no_implicit_weapon_default() {
        let mut f = fact();
        for (raw, expected) in [
            (0, 19),
            (1, 5),
            (2, 8),
            (3, 12),
            (4, 1),
            (5, 5),
            (6, 19),
            (u32::MAX, 19),
        ] {
            f.raw_weapon_selector = raw;
            assert_eq!(f.weapon_selector(19), expected);
        }
    }
    #[test]
    fn damage_tooltip_ordinary_order_and_unsigned_attribute() {
        let mut f = fact();
        assert_eq!(f.evaluate(Some(inputs())).unwrap(), 224);
        f.base = 0;
        f.coefficient = 1.5;
        assert_eq!(f.evaluate(Some(inputs())).unwrap(), 333);
        f.coefficient = 1.0;
        let input = DamageTooltipInputs209026 {
            weapon_strength: [100, 100],
            attribute_one: 0x80000000,
            monster_defense_at_level: 0x80000000,
            monster_attribute_at_level: 0,
            ..inputs()
        };
        assert_eq!(f.evaluate(Some(input)).unwrap(), 100);
        assert!(f.evaluate(None).is_err());
    }
    #[test]
    fn damage_tooltip_flat_and_weapon_apply_multiplier_on_different_sides_of_rounding() {
        let mut f = DamageFact209026 {
            base: 1,
            coefficient: 0.4,
            multiplier: 2,
            formula: DamageTooltipFormula209026::Flat,
            ..fact()
        };
        assert_eq!(f.evaluate(None).unwrap(), 1); // Round(0.8), not round(0.4)*2.
        let ignored = DamageTooltipInputs209026 {
            base_damage_multiplier: f32::NAN,
            ..inputs()
        };
        assert_eq!(f.evaluate(Some(ignored)).unwrap(), 1);
        f.coefficient = -1.0;
        assert_eq!(f.evaluate(None).unwrap(), 0);
        f.formula = DamageTooltipFormula209026::Weapon;
        assert_eq!(f.evaluate(Some(inputs())).unwrap(), 2); // Clamp1 before *2.
        f.multiplier = 0;
        assert_eq!(f.evaluate(Some(inputs())).unwrap(), 0);
    }
    #[test]
    fn damage_tooltip_rounds_with_single_precision_add_half_not_round_method() {
        let f = DamageFact209026 {
            coefficient: f32::from_bits(0x3effffff),
            formula: DamageTooltipFormula209026::Flat,
            ..fact()
        };
        assert_eq!(f.coefficient.round(), 0.0);
        assert_eq!(f.evaluate(None).unwrap(), 1);
    }
    #[test]
    fn damage_tooltip_rejects_unsupported_arithmetic_instead_of_saturating_or_wrapping() {
        let mut f = fact();
        for input in [
            DamageTooltipInputs209026 {
                base_damage_multiplier: f32::NAN,
                ..inputs()
            },
            DamageTooltipInputs209026 {
                monster_defense_at_level: 0,
                monster_attribute_at_level: 0,
                ..inputs()
            },
            DamageTooltipInputs209026 {
                monster_defense_at_level: u32::MAX,
                ..inputs()
            },
            DamageTooltipInputs209026 {
                weapon_strength: [i32::MAX as u32, 1],
                ..inputs()
            },
            DamageTooltipInputs209026 {
                weapon_strength: [u32::MAX, 1],
                ..inputs()
            },
        ] {
            assert!(f.evaluate(Some(input)).is_err());
        }
        f.multiplier = -1;
        assert!(f.evaluate(Some(inputs())).is_err());
        f.multiplier = i32::MAX;
        assert!(f.evaluate(Some(inputs())).is_err());
        f.formula = DamageTooltipFormula209026::Flat;
        f.coefficient = f32::MAX;
        assert!(f.evaluate(None).is_err());
        f.coefficient = f32::INFINITY;
        assert!(f.evaluate(None).is_err());
    }
}
