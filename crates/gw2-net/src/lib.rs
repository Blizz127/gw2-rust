//! GW2 game transport framing.
//!
//! Ported from valaphee/ascalon `network/src/lib.rs` (MIT, commit `03fb60b`),
//! rewritten for stable Rust.
//!
//! Status: reference layout from build 208039. **Not yet confirmed by REA for
//! our client build.** See `docs/STATUS.md`.
//!
//! - Both directions are RC4 with the same 20-byte key ([`rc4_key`]), but with
//!   independent cipher states (one per direction).
//! - Client → server: raw message bytes, RC4 only. No framing at this layer.
//! - Server → client: `u16 compressed_len | u16 raw_len | LZ4 block`, and the
//!   whole frame, header included, is RC4'd.

pub mod dh;

use std::io::{Error, ErrorKind, Result};

use bytes::{Bytes, BytesMut};
use lz4_flex::block::{
    CompressTable, compress_into_with_table, decompress_into, get_maximum_output_size,
};
use rc4::{KeyInit, Rc4, StreamCipher};
use tokio_util::codec::{Decoder, Encoder};

pub use tokio_util::codec::Framed;

type Cipher = Rc4;

fn cipher(key: &[u8; 20]) -> Cipher {
    // rc4 0.2 accepts any key length from 1 to 256 bytes through new_from_slice.
    Cipher::new_from_slice(key).expect("20-byte RC4 key is valid")
}

/// Server side: decodes client bytes, encodes compressed server frames.
pub struct ServerCodec {
    encryptor: Cipher,
    decryptor: Cipher,
    compress_table: CompressTable,
}

impl ServerCodec {
    /// `session_key` is the shared secret from the key exchange (any length ≥ 1).
    pub fn new(session_key: &[u8]) -> Self {
        let key = rc4_key(session_key);
        Self {
            encryptor: cipher(&key),
            decryptor: cipher(&key),
            compress_table: CompressTable::default(),
        }
    }
}

impl Decoder for ServerCodec {
    type Item = Bytes;
    type Error = Error;

    /// Client traffic has no frame boundaries at this layer, so this yields
    /// whatever has arrived, decrypted. Message splitting happens in the
    /// protocol layer.
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Bytes>> {
        if src.is_empty() {
            return Ok(None);
        }
        self.decryptor.apply_keystream(src);
        Ok(Some(src.split().freeze()))
    }
}

impl Encoder<Bytes> for ServerCodec {
    type Error = Error;

    fn encode(&mut self, src: Bytes, dst: &mut BytesMut) -> Result<()> {
        if src.len() > u16::MAX as usize {
            return Err(Error::new(ErrorKind::InvalidInput, "server frame over 65535 bytes"));
        }
        let start = dst.len();
        dst.resize(start + 4 + get_maximum_output_size(src.len()), 0);
        let compressed_len =
            compress_into_with_table(&src, &mut dst[start + 4..], &mut self.compress_table)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        if compressed_len > u16::MAX as usize {
            dst.truncate(start);
            return Err(Error::new(ErrorKind::InvalidInput, "compressed frame over 65535 bytes"));
        }
        dst.truncate(start + 4 + compressed_len);
        dst[start..start + 2].copy_from_slice(&(compressed_len as u16).to_le_bytes());
        dst[start + 2..start + 4].copy_from_slice(&(src.len() as u16).to_le_bytes());
        self.encryptor.apply_keystream(&mut dst[start..]);
        Ok(())
    }
}

/// Client side (for tests, proxies and capture tools).
pub struct ClientCodec {
    encryptor: Cipher,
    decryptor: Cipher,
    /// Bytes at the front of the read buffer that are already decrypted.
    decrypted: usize,
}

impl ClientCodec {
    pub fn new(session_key: &[u8]) -> Self {
        let key = rc4_key(session_key);
        Self { encryptor: cipher(&key), decryptor: cipher(&key), decrypted: 0 }
    }
}

impl Encoder<Bytes> for ClientCodec {
    type Error = Error;

    fn encode(&mut self, src: Bytes, dst: &mut BytesMut) -> Result<()> {
        let start = dst.len();
        dst.extend_from_slice(&src);
        self.encryptor.apply_keystream(&mut dst[start..]);
        Ok(())
    }
}

impl Decoder for ClientCodec {
    type Item = Bytes;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Bytes>> {
        // RC4 is a stream: decrypt each byte exactly once, as it arrives.
        let decrypted = self.decrypted.min(src.len());
        self.decryptor.apply_keystream(&mut src[decrypted..]);
        self.decrypted = src.len();

        if src.len() < 4 {
            return Ok(None);
        }
        let compressed_len = u16::from_le_bytes([src[0], src[1]]) as usize;
        let raw_len = u16::from_le_bytes([src[2], src[3]]) as usize;
        let frame_len = 4 + compressed_len;
        if src.len() < frame_len {
            return Ok(None);
        }

        let frame = src.split_to(frame_len);
        self.decrypted -= frame_len;

        let mut out = vec![0u8; raw_len];
        let written = decompress_into(&frame[4..], &mut out)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        if written != raw_len {
            return Err(Error::new(ErrorKind::InvalidData, "LZ4 length mismatch"));
        }
        Ok(Some(Bytes::from(out)))
    }
}

/// Derives the 20-byte RC4 key from a session secret.
///
/// The secret is folded into 20 bytes (repeated if shorter, XOR-folded if
/// longer), then run through one SHA-1 compression round's first five steps
/// plus the feed-forward. This is not a full SHA-1.
pub fn rc4_key(input: &[u8]) -> [u8; 20] {
    assert!(!input.is_empty(), "session key must not be empty");

    let mut bytes = [0u8; 20];
    if input.len() >= 20 {
        bytes.copy_from_slice(&input[..20]);
    } else {
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = input[i % input.len()];
        }
    }
    for (i, b) in input.iter().enumerate().skip(20) {
        bytes[i % 20] ^= b;
    }

    let mut w = [0u32; 5];
    for (i, word) in w.iter_mut().enumerate() {
        *word = u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
    }

    let (mut a, mut b, mut c, mut d, mut e) =
        (0x67452301u32, 0xEFCDAB89u32, 0x98BADCFEu32, 0x10325476u32, 0xC3D2E1F0u32);
    const K: u32 = 0x5A827999;

    e = e.wrapping_add(w[0]).wrapping_add(d ^ (b & (c ^ d))).wrapping_add(a.rotate_left(5)).wrapping_add(K);
    b = b.rotate_left(30);
    d = d.wrapping_add(w[1]).wrapping_add(c ^ (a & (b ^ c))).wrapping_add(e.rotate_left(5)).wrapping_add(K);
    a = a.rotate_left(30);
    c = c.wrapping_add(w[2]).wrapping_add(b ^ (e & (a ^ b))).wrapping_add(d.rotate_left(5)).wrapping_add(K);
    e = e.rotate_left(30);
    b = b.wrapping_add(w[3]).wrapping_add(a ^ (d & (e ^ a))).wrapping_add(c.rotate_left(5)).wrapping_add(K);
    d = d.rotate_left(30);
    a = a.wrapping_add(w[4]).wrapping_add(e ^ (c & (d ^ e))).wrapping_add(b.rotate_left(5)).wrapping_add(K);
    c = c.rotate_left(30);

    let add = [a, b, c, d, e];
    let mut out = [0u8; 20];
    for i in 0..5 {
        out[i * 4..i * 4 + 4].copy_from_slice(&w[i].wrapping_add(add[i]).to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &[u8] = b"synthetic-session-key-for-tests-only";

    #[test]
    fn rc4_matches_rfc6229_vector() {
        // RFC 6229, 40-bit key 0x0102030405, keystream offset 0.
        let mut c = Rc4::new_from_slice(&[1, 2, 3, 4, 5]).unwrap();
        let mut buf = [0u8; 8];
        c.apply_keystream(&mut buf);
        assert_eq!(buf, [0xb2, 0x39, 0x63, 0x05, 0xf0, 0x3d, 0xc0, 0x27]);
    }

    #[test]
    fn rc4_key_folds_short_and_long_inputs() {
        // Short inputs repeat; a 1-byte key equals the same byte repeated 20 times.
        assert_eq!(rc4_key(&[7]), rc4_key(&[7; 20]));
        // Bytes beyond 20 are XOR-folded, so appending 20 zero bytes changes nothing.
        let mut long = [0u8; 40];
        long[..20].copy_from_slice(&[9; 20]);
        assert_eq!(rc4_key(&long), rc4_key(&[9; 20]));
        assert_ne!(rc4_key(b"a"), rc4_key(b"b"));
    }

    #[test]
    fn server_frames_round_trip_through_client_with_split_reads() {
        let mut server = ServerCodec::new(KEY);
        let mut client = ClientCodec::new(KEY);

        let msgs: Vec<Bytes> = vec![
            Bytes::from_static(b"hello"),
            Bytes::from(vec![0xAB; 3000]),
            Bytes::from((0..=255u8).cycle().take(9000).collect::<Vec<_>>()),
            Bytes::new(),
        ];
        let mut wire = BytesMut::new();
        for m in &msgs {
            server.encode(m.clone(), &mut wire).unwrap();
        }

        // Feed the client in awkward chunk sizes to exercise partial decryption.
        let mut inbox = BytesMut::new();
        let mut got = Vec::new();
        for chunk in wire.chunks(7) {
            inbox.extend_from_slice(chunk);
            while let Some(frame) = client.decode(&mut inbox).unwrap() {
                got.push(frame);
            }
        }
        assert_eq!(got, msgs);
        assert!(inbox.is_empty());
    }

    #[test]
    fn client_bytes_round_trip_through_server() {
        let mut server = ServerCodec::new(KEY);
        let mut client = ClientCodec::new(KEY);
        let mut wire = BytesMut::new();
        client.encode(Bytes::from_static(b"\x01\x00abc"), &mut wire).unwrap();
        client.encode(Bytes::from_static(b"\x02\x00def"), &mut wire).unwrap();
        assert_ne!(&wire[..], b"\x01\x00abc\x02\x00def");
        let mut first = wire.split_to(3);
        let a = server.decode(&mut first).unwrap().unwrap();
        let b = server.decode(&mut wire).unwrap().unwrap();
        assert_eq!([a, b].concat(), b"\x01\x00abc\x02\x00def");
    }

    #[test]
    fn wrong_key_does_not_decode() {
        let mut server = ServerCodec::new(KEY);
        let mut client = ClientCodec::new(b"some-other-key");
        let mut wire = BytesMut::new();
        server.encode(Bytes::from(vec![1u8; 500]), &mut wire).unwrap();
        // Either an error, an incomplete frame, or garbage: never the original payload.
        if let Ok(Some(b)) = client.decode(&mut wire) {
            assert_ne!(&b[..], &[1u8; 500][..]);
        }
    }
}
