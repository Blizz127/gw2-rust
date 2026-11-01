//! Build-209026 content-name conversion for containers with flag bit 1 set.
//!
//! This converts lookup text, without resolving content or guaranteeing that
//! the shortened names are unique. See `docs/re/209026-content-names.md`.

use std::io::{Error, ErrorKind, Result};

use sha2::{Digest, Sha256};

/// Convert a content name using the frozen client's last-dot split and hash.
///
/// Supports at most 511 UTF-16 code units and rejects embedded NUL. Longer
/// input is rejected rather than emulating the client's bounded string copy.
pub fn encode_209026(name: &str) -> Result<String> {
    if name.encode_utf16().take(512).count() > 511 || name.contains('\0') {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "content name exceeds the supported bound or contains NUL",
        ));
    }
    let mut result = String::with_capacity(11);
    if let Some((prefix, leaf)) = name.rsplit_once('.') {
        component(prefix, &mut result);
        result.push('.');
        component(leaf, &mut result);
    } else {
        component(name, &mut result);
    }
    Ok(result)
}

fn component(name: &str, output: &mut String) {
    let mut digest = Sha256::new();
    for unit in name.encode_utf16() {
        digest.update(unit.to_le_bytes());
    }
    let digest = digest.finalize();
    let mut mixed = 0xcbf2_9ce4_8422_2325u64;
    for bytes in digest.as_chunks::<4>().0.iter() {
        let mut word = u32::from_le_bytes(*bytes);
        for _ in 0..4 {
            // The client mixes the entire remaining Dword, not just its low byte.
            mixed = (mixed ^ u64::from(word)).wrapping_mul(0x100_0000_01b3);
            word >>= 8;
        }
    }
    let b = mixed.to_le_bytes();
    let indices = [
        b[0] >> 2,
        ((b[0] & 3) << 4) | (b[1] >> 4),
        ((b[1] & 15) << 2) | (b[2] >> 6),
        b[2] & 63,
        b[3] >> 2,
    ];
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for index in indices {
        output.push(char::from(ALPHABET[usize::from(index)]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_independent_vectors_with_last_dot_and_utf16() {
        // Independent Python SHA-256 / struct / Base64 calculation.
        for (input, expected) in [
            ("", "wqv8U"),
            ("Example", "pesLQ"),
            ("Example.Owner.Palette", "dBOqC.k1g2z"),
            ("A..B", "02vJb.Er+xu"),
            (".B", "wqv8U.Er+xu"),
            ("A.", "yAP5Y.wqv8U"),
            ("café.雪😀", "ZcmSu.1FEUE"),
        ] {
            assert_eq!(encode_209026(input).unwrap(), expected);
        }
    }

    #[test]
    fn bounds_utf16_units_and_rejects_embedded_terminators() {
        assert!(encode_209026(&"a".repeat(511)).is_ok());
        assert!(encode_209026(&"a".repeat(512)).is_err());
        assert!(encode_209026(&format!("{}a", "😀".repeat(255))).is_ok());
        assert!(encode_209026(&"😀".repeat(256)).is_err());
        assert!(encode_209026("before\0after").is_err());
    }
}
