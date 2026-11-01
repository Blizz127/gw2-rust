//! Build-209026 creation defaults extracted from type-137 content records.
//!
//! This reader accepts the complete decimal section format observed in the
//! frozen archive. It is stricter than the client's permissive string importer;
//! it does not validate cosmetic references or establish runtime validity.

use std::io::{Error, ErrorKind, Result};

pub const DEFAULT_TEXT_LIMIT: usize = 4096;

pub struct AppearanceDefault209026 {
    appearance: [u8; 45],
    colors: Vec<[u32; 4]>,
}

fn invalid() -> Error {
    Error::new(
        ErrorKind::InvalidData,
        "invalid build-209026 appearance default",
    )
}

fn decimals<const N: usize>(text: &str) -> Result<[u32; N]> {
    let mut values = text.split(',');
    let mut result = [0; N];
    for slot in &mut result {
        let token = values.next().ok_or_else(invalid)?.trim_ascii();
        if token.is_empty() || token.len() > 10 || !token.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        *slot = token.parse().map_err(|_| invalid())?;
    }
    if values.next().is_some() {
        return Err(invalid());
    }
    Ok(result)
}

impl AppearanceDefault209026 {
    /// Parse one default string for a separately selected race/sex definition.
    /// The returned appearance includes the client's post-import Sylvari edits.
    pub fn parse(text: &[u8], race: u8, sex: u8) -> Result<Self> {
        if text.len() > DEFAULT_TEXT_LIMIT || race >= 5 || sex >= 2 {
            return Err(invalid());
        }
        let text = std::str::from_utf8(text).map_err(|_| invalid())?;
        let text = text.strip_prefix("appearanceV0='").ok_or_else(invalid)?;
        let (appearance, colors) = text.split_once("' armorColorV0='").ok_or_else(invalid)?;
        let colors = colors.strip_suffix('\'').ok_or_else(invalid)?;
        let mut bytes = [0; 45];
        for (dst, value) in bytes.iter_mut().zip(decimals::<45>(appearance)?) {
            *dst = u8::try_from(value).map_err(|_| invalid())?;
        }
        if bytes[27] != race || bytes[29] != sex {
            return Err(invalid());
        }
        // Shipped defaults contain only 59 or 60 rows despite the client's
        // 68-row loop. Keep only supplied rows, separately from the client's
        // statically traced empty-token zero-fill behavior.
        // At least eight rows are needed to provide request 415's rows 2–7.
        let count = colors.split(',').count();
        if !(32..=272).contains(&count) || count % 4 != 0 {
            return Err(invalid());
        }
        let mut rows = Vec::with_capacity(count / 4);
        let mut tokens = colors.split(',');
        for _ in 0..count / 4 {
            let mut row = [0; 4];
            for slot in &mut row {
                *slot = decimals::<1>(tokens.next().ok_or_else(invalid)?)?[0];
            }
            rows.push(row);
        }
        if race == 4 {
            bytes[0] = 255;
            bytes[37] = 0;
        }
        Ok(Self {
            appearance: bytes,
            colors: rows,
        })
    }

    pub fn appearance(&self) -> &[u8; 45] {
        &self.appearance
    }

    /// Only explicitly provided rows; no values are synthesized for a short tail.
    pub fn colors(&self) -> &[[u32; 4]] {
        &self.colors
    }

    /// Request 415 sends equipment rows 2–7, each with four color selections.
    pub fn creation_colors(&self) -> [u32; 24] {
        std::array::from_fn(|i| self.colors[2 + i / 4][i % 4])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(race: u8, sex: u8) -> String {
        fixture_rows(race, sex, 68)
    }

    fn fixture_rows(race: u8, sex: u8, rows: u32) -> String {
        let mut bytes = [128u32; 45];
        bytes[27] = race.into();
        bytes[29] = sex.into();
        let appearance = bytes.map(|n| n.to_string()).join(",");
        let colors = (0..rows * 4)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!("appearanceV0='{appearance}' armorColorV0='{colors}'")
    }

    #[test]
    fn keeps_short_color_tail_explicit_and_bounds_supplied_rows() {
        for rows in [8, 59, 60, 68] {
            let d =
                AppearanceDefault209026::parse(fixture_rows(0, 0, rows).as_bytes(), 0, 0).unwrap();
            assert_eq!(d.colors().len(), rows as usize);
            assert_eq!(d.colors().last().unwrap()[3], rows * 4 - 1);
            assert_eq!(d.creation_colors(), std::array::from_fn(|i| 8 + i as u32));
        }
        for rows in [0, 7, 69] {
            assert!(
                AppearanceDefault209026::parse(fixture_rows(0, 0, rows).as_bytes(), 0, 0).is_err()
            );
        }
        let wrapped = fixture_rows(0, 0, 60)
            .replace("armorColorV0='0,", "armorColorV0='\r\r\n0,\t")
            .replace(",239'", ",239\r\r\n'");
        let d = AppearanceDefault209026::parse(wrapped.as_bytes(), 0, 0).unwrap();
        assert_eq!(d.colors().len(), 60);
        assert_eq!(d.colors()[59][3], 239);
    }

    #[test]
    fn import_preserves_fields_and_applies_sylvari_adjustments() {
        for race in 0..5 {
            for sex in 0..2 {
                let d = AppearanceDefault209026::parse(fixture(race, sex).as_bytes(), race, sex)
                    .unwrap();
                assert_eq!(d.appearance()[27], race);
                assert_eq!(d.appearance()[29], sex);
                assert_eq!(d.appearance()[0], if race == 4 { 255 } else { 128 });
                assert_eq!(d.appearance()[37], if race == 4 { 0 } else { 128 });
                assert_eq!(d.appearance()[44], 128);
                assert_eq!(d.colors()[67], [268, 269, 270, 271]);
                assert_eq!(d.creation_colors(), std::array::from_fn(|i| 8 + i as u32));
            }
        }
        let max = fixture(0, 0).replace("armorColorV0='0,", "armorColorV0='4294967295,");
        assert_eq!(
            AppearanceDefault209026::parse(max.as_bytes(), 0, 0)
                .unwrap()
                .colors()[0][0],
            u32::MAX
        );
    }

    #[test]
    fn rejects_incomplete_wrong_context_and_out_of_range_defaults() {
        let valid = fixture(2, 1);
        for end in 0..valid.len() {
            assert!(AppearanceDefault209026::parse(&valid.as_bytes()[..end], 2, 1).is_err());
        }
        for bad in [
            format!("{valid}\n"),
            valid.replacen("128,", "256,", 1),
            valid.replacen("128,", "-1,", 1),
            valid.replacen("128,", "+1,", 1),
            valid.replacen("128,", "1.0,", 1),
            valid.replacen("128,", "1 2,", 1),
            valid.replacen("128,", ",", 1),
            valid.replacen("128,", "", 1),
            valid.replacen("128,", "128,128,", 1),
            valid.replace("armorColorV0='0,", "armorColorV0='4294967296,"),
            valid.replace(",271'", ",271,272'"),
            valid.replace(",271'", "'"),
            valid.replace("appearanceV0", "appearanceV1"),
            " ".repeat(DEFAULT_TEXT_LIMIT + 1),
        ] {
            assert!(AppearanceDefault209026::parse(bad.as_bytes(), 2, 1).is_err());
        }
        for (race, sex) in [(0, 1), (2, 0), (5, 1), (2, 2)] {
            assert!(AppearanceDefault209026::parse(valid.as_bytes(), race, sex).is_err());
        }
        assert!(AppearanceDefault209026::parse(&[0xff], 0, 0).is_err());
    }
}
