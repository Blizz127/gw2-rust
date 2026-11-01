//! Structural inspection of build-209026 version-11 character summaries.
//!
//! Evidence: `docs/re/209026-character-protocol.md`. This is the blob carried
//! inside character records, not a complete protocol frame. Parsing does not
//! perform the client's required map-definition lookup or content-dependent
//! appearance checks and does not establish that a preview will render.

use std::io::{Error, ErrorKind, Result};

pub const V11_SIZE: usize = 159;

/// A borrowed, structurally checked summary. Unknown bytes remain intact.
/// Historical versions are deliberately rejected instead of converted.
pub struct SummaryV11<'a> {
    bytes: &'a [u8; V11_SIZE],
}

impl<'a> SummaryV11<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self> {
        let bytes: &[u8; V11_SIZE] = bytes.try_into().map_err(|_| {
            Error::new(
                ErrorKind::InvalidData,
                "version-11 summary must be 159 bytes",
            )
        })?;
        if bytes[0] != 11 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "unsupported summary version",
            ));
        }
        // Mirror only the scalar checks in 0x1417f4d90. Creation has stricter
        // level/sex constraints, and the preview rejects sex 2 separately.
        if bytes[4] > 80
            || !(1..=9).contains(&bytes[5])
            || bytes[0x21] >= 5
            || bytes[0x22] > 15
            || bytes[0x23] >= 3
        {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "summary scalar outside client bounds",
            ));
        }
        Ok(Self { bytes })
    }

    pub fn as_bytes(&self) -> &'a [u8; V11_SIZE] {
        self.bytes
    }

    /// Must resolve in the client's map-definition content service.
    pub fn map_definition_key(&self) -> u16 {
        u16::from_le_bytes([self.bytes[1], self.bytes[2]])
    }

    pub fn level(&self) -> u8 {
        self.bytes[4]
    }

    /// Build-209026 profession enum, 1 through 9.
    pub fn profession(&self) -> u8 {
        self.bytes[5]
    }

    /// The 45-byte block copied by the client's preview converter.
    pub fn appearance(&self) -> &'a [u8] {
        &self.bytes[6..51]
    }

    pub fn race(&self) -> u8 {
        self.bytes[0x21]
    }

    /// 0 male, 1 female, 2 none. Parsing accepts 2; preview construction does not.
    pub fn sex(&self) -> u8 {
        self.bytes[0x23]
    }

    /// Only the preview's sex-enum check, not full preview validation.
    pub fn passes_preview_sex_check(&self) -> bool {
        self.sex() != 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boundary_fixture() -> [u8; V11_SIZE] {
        // Deliberately not a valid in-game character: this tests blob structure.
        let mut bytes = [0xa5; V11_SIZE];
        bytes[0] = 11;
        bytes[1..3].copy_from_slice(&0x1234u16.to_le_bytes());
        bytes[4] = 80;
        bytes[5] = 9;
        bytes[0x21] = 4;
        bytes[0x22] = 15;
        bytes[0x23] = 1;
        bytes
    }

    #[test]
    fn summary_v11_preserves_opaque_bytes_and_exact_boundaries() {
        let bytes = boundary_fixture();
        let summary = SummaryV11::parse(&bytes).unwrap();
        assert_eq!(summary.as_bytes(), &bytes);
        assert_eq!(summary.map_definition_key(), 0x1234);
        assert_eq!(summary.level(), 80);
        assert_eq!(summary.profession(), 9);
        assert_eq!(summary.appearance(), &bytes[6..51]);
        assert_eq!((summary.race(), summary.sex()), (4, 1));
        assert!(summary.passes_preview_sex_check());
        for cut in 0..V11_SIZE {
            assert!(SummaryV11::parse(&bytes[..cut]).is_err());
        }
        let mut oversized = bytes.to_vec();
        oversized.push(0);
        assert!(SummaryV11::parse(&oversized).is_err());
        for version in [0, 7, 8, 9, 10, 12, 255] {
            let mut altered = bytes;
            altered[0] = version;
            assert!(SummaryV11::parse(&altered).is_err());
        }
    }

    #[test]
    fn summary_v11_matches_scalar_bounds_without_claiming_content_validity() {
        let bytes = boundary_fixture();
        for (offset, bad) in [(4, 81), (5, 0), (5, 10), (0x21, 5), (0x22, 16), (0x23, 3)] {
            let mut altered = bytes;
            altered[offset] = bad;
            assert!(SummaryV11::parse(&altered).is_err());
        }
        let mut altered = bytes;
        altered[1..3].fill(0); // No map lookup is performed by this parser.
        altered[4] = 0; // Unlike creation, the summary admits level zero.
        altered[5] = 1;
        altered[0x21] = 0;
        altered[0x22] = 0;
        altered[0x23] = 2; // Admitted by normalization, rejected by preview.
        let summary = SummaryV11::parse(&altered).unwrap();
        assert_eq!(summary.map_definition_key(), 0);
        assert_eq!(summary.level(), 0);
        assert!(!summary.passes_preview_sex_check());
        altered[0x23] = 0;
        assert!(
            SummaryV11::parse(&altered)
                .unwrap()
                .passes_preview_sex_check()
        );
    }
}
