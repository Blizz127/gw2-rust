//! Build-209026 default primary-attribute curve from configuration 396.
//!
//! This is the explicit default curve at layout +0x40, not the alternate
//! account-flag curve at +0x38. It chooses no earned/effective-level policy,
//! modifiers, equipment state or wire updates. Source and full-table pin:
//! `docs/re/209026-default-primary-attribute-curve.md`.

/// Source rows indexed by levels 0..=100. Stored Dwords were checked to fit
/// u16; the full source table is retained even though player levels stop at 80.
/// The native primary-attribute helper uses this curve for selectors 1..=4.
pub const DEFAULT_PRIMARY_ATTRIBUTE_CURVE209026: [u16; 101] = [
    30, 37, 44, 51, 58, 65, 72, 79, 86, 93, 100, 100, 110, 110, 120, 120, 130, 130, 140, 140, 150,
    150, 164, 164, 178, 178, 193, 193, 209, 209, 225, 225, 245, 245, 265, 265, 285, 285, 305, 305,
    325, 325, 349, 349, 373, 373, 398, 398, 424, 424, 450, 450, 480, 480, 510, 510, 540, 540, 570,
    570, 600, 600, 634, 634, 668, 668, 703, 703, 739, 739, 775, 775, 819, 819, 863, 863, 908, 908,
    954, 954, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
    1000, 1000, 1000, 1000, 1000, 1000, 1000,
];

/// Look up the explicitly selected default curve for a supported player level.
/// Reject level zero and levels above 80; do not clamp or select a fallback.
pub const fn default_player_primary_attribute209026(level: u8) -> Option<u16> {
    if level >= 1 && level <= 80 {
        Some(DEFAULT_PRIMARY_ATTRIBUTE_CURVE209026[level as usize])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_curve_preserves_source_steps_and_nonplayer_tail() {
        // Independent retained source checkpoints include changes of slope,
        // repeated rows and the distinction between raw row0 and a player.
        for (level, expected) in [
            (0, 30),
            (1, 37),
            (2, 44),
            (9, 93),
            (10, 100),
            (11, 100),
            (12, 110),
            (13, 110),
            (14, 120),
            (20, 150),
            (22, 164),
            (26, 193),
            (48, 424),
            (62, 634),
            (66, 703),
            (76, 908),
            (80, 1000),
        ] {
            assert_eq!(DEFAULT_PRIMARY_ATTRIBUTE_CURVE209026[level], expected);
        }
        assert!(
            DEFAULT_PRIMARY_ATTRIBUTE_CURVE209026
                .windows(2)
                .all(|pair| pair[0] <= pair[1])
        );
        assert_eq!(&DEFAULT_PRIMARY_ATTRIBUTE_CURVE209026[80..], &[1000; 21]);
    }

    #[test]
    fn player_lookup_is_exact_and_rejects_every_unsupported_byte() {
        for level in 1..=80 {
            assert_eq!(
                default_player_primary_attribute209026(level),
                Some(DEFAULT_PRIMARY_ATTRIBUTE_CURVE209026[usize::from(level)])
            );
        }
        assert_eq!(default_player_primary_attribute209026(0), None);
        for level in 81..=u8::MAX {
            assert_eq!(default_player_primary_attribute209026(level), None);
        }
    }
}
