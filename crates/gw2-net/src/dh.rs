//! Game-connection Diffie–Hellman.
//!
//! What we have (inferred, not yet REA-confirmed for our build; see
//! `docs/re/public-protocol-notes.md`): a 512-bit prime, g = 4, and the server's
//! public value baked into the client as a 136-byte `.rdata` blob:
//! `u32 1, u32 g, [u8; 64] p, [u8; 64] server_public`.
//!
//! **Unknown:** the bignum byte order in the blob. [`ByteOrder`] stays explicit until
//! REA settles it. Also unknown: exactly how the session secret combines with the
//! server's 20-byte block. [`session_secret`] implements the public description
//! (shared secret XOR block) and is marked provisional.
//!
//! Our server keeps its own private exponent. The client process gets our public
//! value (and, if we choose, our prime) written over the blob in memory.

use num_bigint::BigUint;

pub const BLOB_LEN: usize = 136;
pub const KEY_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    Little,
    Big,
}

fn from_bytes(b: &[u8], o: ByteOrder) -> BigUint {
    match o {
        ByteOrder::Little => BigUint::from_bytes_le(b),
        ByteOrder::Big => BigUint::from_bytes_be(b),
    }
}

/// Fixed-width encoding; returns None if the value does not fit.
fn to_bytes(v: &BigUint, width: usize, o: ByteOrder) -> Option<Vec<u8>> {
    let mut le = v.to_bytes_le();
    if le.len() > width {
        return None;
    }
    le.resize(width, 0);
    if o == ByteOrder::Big {
        le.reverse();
    }
    Some(le)
}

/// The client-side DH parameter blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamBlob {
    pub version: u32,
    pub g: BigUint,
    pub p: BigUint,
    pub server_public: BigUint,
}

impl ParamBlob {
    pub fn parse(b: &[u8], o: ByteOrder) -> Option<Self> {
        if b.len() != BLOB_LEN {
            return None;
        }
        let version = u32::from_le_bytes(b[0..4].try_into().ok()?);
        let g = u32::from_le_bytes(b[4..8].try_into().ok()?);
        Some(Self {
            version,
            g: BigUint::from(g),
            p: from_bytes(&b[8..8 + KEY_BYTES], o),
            server_public: from_bytes(&b[8 + KEY_BYTES..], o),
        })
    }

    pub fn to_bytes(&self, o: ByteOrder) -> Option<[u8; BLOB_LEN]> {
        let g: u32 = self.g.clone().try_into().ok()?;
        let mut out = [0u8; BLOB_LEN];
        out[0..4].copy_from_slice(&self.version.to_le_bytes());
        out[4..8].copy_from_slice(&g.to_le_bytes());
        out[8..8 + KEY_BYTES].copy_from_slice(&to_bytes(&self.p, KEY_BYTES, o)?);
        out[8 + KEY_BYTES..].copy_from_slice(&to_bytes(&self.server_public, KEY_BYTES, o)?);
        Some(out)
    }
}

/// Our server's DH key pair for one (p, g).
#[derive(Clone)]
pub struct ServerKey {
    pub p: BigUint,
    pub g: BigUint,
    private: BigUint,
}

impl std::fmt::Debug for ServerKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerKey").field("p", &self.p).field("g", &self.g).finish_non_exhaustive()
    }
}

impl ServerKey {
    /// Fresh random private exponent in [2, p-2].
    pub fn generate(p: BigUint, g: BigUint) -> std::io::Result<Self> {
        let mut raw = [0u8; KEY_BYTES];
        getrandom::fill(&mut raw).map_err(std::io::Error::other)?;
        let two = BigUint::from(2u32);
        let range = &p - BigUint::from(3u32);
        let private = BigUint::from_bytes_le(&raw) % range + two;
        Ok(Self { p, g, private })
    }

    pub fn from_private(p: BigUint, g: BigUint, private: BigUint) -> Self {
        Self { p, g, private }
    }

    pub fn private(&self) -> &BigUint {
        &self.private
    }

    pub fn public(&self) -> BigUint {
        self.g.modpow(&self.private, &self.p)
    }

    /// The blob to write over the client's baked parameters (in memory only).
    pub fn param_blob(&self, version: u32) -> ParamBlob {
        ParamBlob { version, g: self.g.clone(), p: self.p.clone(), server_public: self.public() }
    }

    /// Shared secret from the client's public value. Returns None for
    /// degenerate client values (0, 1, p-1, ≥ p).
    pub fn shared(&self, client_public: &BigUint) -> Option<BigUint> {
        let one = BigUint::from(1u32);
        if *client_public <= one || *client_public >= &self.p - &one {
            return None;
        }
        Some(client_public.modpow(&self.private, &self.p))
    }
}

/// PROVISIONAL: session secret = shared secret bytes XOR the server's 20-byte
/// block (repeating the block), per the 2013 public notes. Feed the result to
/// [`crate::rc4_key`]. Replace this once REA shows the real combination.
pub fn session_secret(shared: &BigUint, block: &[u8; 20], o: ByteOrder) -> Vec<u8> {
    let mut s = to_bytes(shared, KEY_BYTES, o).expect("shared < p fits 64 bytes");
    for (i, b) in s.iter_mut().enumerate() {
        *b ^= block[i % 20];
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only 512-bit prime: 2^512 - 569 (a known prime). It is NOT the client's prime.
    fn test_prime() -> BigUint {
        (BigUint::from(1u32) << 512usize) - BigUint::from(569u32)
    }

    #[test]
    fn both_sides_agree() {
        let p = test_prime();
        let g = BigUint::from(4u32);
        let server = ServerKey::generate(p.clone(), g.clone()).unwrap();
        let client = ServerKey::generate(p.clone(), g.clone()).unwrap();
        let s1 = server.shared(&client.public()).unwrap();
        let s2 = client.shared(&server.public()).unwrap();
        assert_eq!(s1, s2);
        let block = [0x5Au8; 20];
        let k1 = crate::rc4_key(&session_secret(&s1, &block, ByteOrder::Little));
        let k2 = crate::rc4_key(&session_secret(&s2, &block, ByteOrder::Little));
        assert_eq!(k1, k2);
    }

    #[test]
    fn rejects_degenerate_client_values() {
        let server = ServerKey::generate(test_prime(), BigUint::from(4u32)).unwrap();
        assert!(server.shared(&BigUint::from(0u32)).is_none());
        assert!(server.shared(&BigUint::from(1u32)).is_none());
        assert!(server.shared(&(test_prime() - BigUint::from(1u32))).is_none());
        assert!(server.shared(&test_prime()).is_none());
    }

    #[test]
    fn blob_round_trips_in_both_orders() {
        let server = ServerKey::generate(test_prime(), BigUint::from(4u32)).unwrap();
        let blob = server.param_blob(1);
        for o in [ByteOrder::Little, ByteOrder::Big] {
            let bytes = blob.to_bytes(o).unwrap();
            assert_eq!(&bytes[0..8], &[1, 0, 0, 0, 4, 0, 0, 0]);
            assert_eq!(ParamBlob::parse(&bytes, o).unwrap(), blob);
        }
        assert!(ParamBlob::parse(&[0u8; 10], ByteOrder::Little).is_none());
    }
}
