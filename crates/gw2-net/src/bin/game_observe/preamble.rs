//! Fixed build-209026 connection prefixes; opaque fields are never logged.
use std::io::{self, Read};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Preamble {
    #[default]
    Auth,
    Mission,
}

impl Preamble {
    pub fn read(self, reader: &mut impl Read) -> io::Result<Vec<u8>> {
        let mut header = [0u8; 16];
        reader.read_exact(&mut header)?;
        if header[4..8] != 209026u32.to_le_bytes() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected frozen build 209026 preamble",
            ));
        }
        if self == Self::Mission {
            // GcTransGameJoin::OnStart: docs/re/209026-mission-join.md.
            if header[..4] != 0x000c0500u32.to_le_bytes()
                || header[8..12] != 101u32.to_le_bytes()
                || header[12..16] != 0x48u32.to_le_bytes()
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid build-209026 mission preamble",
                ));
            }
            let mut bytes = header.to_vec();
            bytes.resize(84, 0);
            reader.read_exact(&mut bytes[16..])?;
            return Ok(bytes);
        }
        Ok(header.to_vec())
    }
}

#[cfg(test)]
pub(super) fn fixture(mode: Preamble) -> Vec<u8> {
    let mut bytes = vec![0xa5; if mode == Preamble::Mission { 84 } else { 16 }];
    bytes[4..8].copy_from_slice(&209026u32.to_le_bytes());
    if mode == Preamble::Mission {
        bytes[..4].copy_from_slice(&0x000c0500u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&101u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&0x48u32.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preambles_are_bounded_and_leave_key_record_untouched() {
        for mode in [Preamble::Auth, Preamble::Mission] {
            let mut input = fixture(mode);
            for cut in 0..input.len() {
                assert_eq!(
                    mode.read(&mut &input[..cut]).unwrap_err().kind(),
                    io::ErrorKind::UnexpectedEof
                );
            }
            input.extend_from_slice(&[0, 6, 0x44, 0x33, 0x22, 0x11]);
            let mut cursor = &input[..];
            assert_eq!(mode.read(&mut cursor).unwrap(), fixture(mode));
            assert_eq!(cursor, &[0, 6, 0x44, 0x33, 0x22, 0x11]);
            input[4] ^= 1;
            assert_eq!(
                mode.read(&mut &input[..]).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
        for offset in [0, 8, 12] {
            let mut input = fixture(Preamble::Mission);
            input[offset] ^= 1;
            assert_eq!(
                Preamble::Mission.read(&mut &input[..]).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
    }
}
