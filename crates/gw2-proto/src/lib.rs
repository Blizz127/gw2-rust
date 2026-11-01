//! GW2 message primitives and the message-schema loader.
//!
//! The primitive encodings originated in valaphee/ascalon `protocol/src/lib.rs`
//! (MIT, commit `03fb60b`) and were rewritten for stable Rust. Build-209026
//! evidence and the verified subset are recorded in `docs/re/`; support in this
//! library alone does not establish an encoding for a particular client build.
//!
//! A message on the wire starts with a little-endian `u16` id, followed by its
//! fields in schema order. That comes from the 208039 dispatch hook in ascalon's
//! Frida agent, which reads `rawData.readU16()`.
//!
//! Variable-field encodings require explicit rules in [`dynamic`]. The capture
//! census enables only the subset verified for build 209026.

pub mod agent_creation;
pub mod character_creation;
pub mod character_summary;
pub mod core_stats;
pub mod dynamic;
pub mod endurance;
pub mod guardian;
pub mod health;
pub mod player_creation;
pub mod schema;
pub mod skill_palette;
pub mod skillbar;

use std::io::{Error, ErrorKind, Result};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use bytes::{BufMut, BytesMut};

pub trait Encode {
    fn encode(&self, buf: &mut BytesMut);
}

pub trait Decode: Sized {
    /// Decodes from the front of `buf` and advances it.
    fn decode(buf: &mut &[u8]) -> Result<Self>;
}

fn take<const N: usize>(buf: &mut &[u8]) -> Result<[u8; N]> {
    if buf.len() < N {
        return Err(Error::from(ErrorKind::UnexpectedEof));
    }
    let (head, rest) = buf.split_at(N);
    *buf = rest;
    Ok(head.try_into().unwrap())
}

/// Schema `Byte`.
impl Encode for u8 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(*self);
    }
}
impl Decode for u8 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(take::<1>(buf)?[0])
    }
}

/// Schema `Word`: u16 little-endian.
impl Encode for u16 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16_le(*self);
    }
}
impl Decode for u16 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(u16::from_le_bytes(take(buf)?))
    }
}

/// Schema `Dword`: u32 as an unsigned LEB128 varint (7-bit groups, at most 5 bytes).
impl Encode for u32 {
    fn encode(&self, buf: &mut BytesMut) {
        let mut v = *self;
        while v >= 0x80 {
            buf.put_u8((v as u8 & 0x7F) | 0x80);
            v >>= 7;
        }
        buf.put_u8(v as u8);
    }
}
impl Decode for u32 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        let mut value = 0u32;
        for shift in (0..35).step_by(7) {
            let byte = u8::decode(buf)?;
            if shift == 28 && byte & 0x70 != 0 {
                return Err(Error::new(ErrorKind::InvalidData, "varint overflows u32"));
            }
            value |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(Error::new(ErrorKind::InvalidData, "varint longer than 5 bytes"))
    }
}

/// Schema `Qword`: u64 little-endian.
impl Encode for u64 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u64_le(*self);
    }
}
impl Decode for u64 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(u64::from_le_bytes(take(buf)?))
    }
}

/// Schema `Float`: f32 little-endian. `Float2/3/4` are `[f32; N]`.
impl Encode for f32 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_f32_le(*self);
    }
}
impl Decode for f32 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(f32::from_le_bytes(take(buf)?))
    }
}

/// Schema `ArrayFixed` and `Float2/3/4`: N elements back to back, no count.
impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode(&self, buf: &mut BytesMut) {
        for v in self {
            v.encode(buf);
        }
    }
}
impl<T: Decode + Default + Copy, const N: usize> Decode for [T; N] {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        let mut out = [T::default(); N];
        for v in &mut out {
            *v = T::decode(buf)?;
        }
        Ok(out)
    }
}

/// Schema `Optional`: a u8 presence tag (0 or 1), then the value if present.
impl<T: Encode> Encode for Option<T> {
    fn encode(&self, buf: &mut BytesMut) {
        match self {
            None => buf.put_u8(0),
            Some(v) => {
                buf.put_u8(1);
                v.encode(buf);
            }
        }
    }
}
impl<T: Decode> Decode for Option<T> {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        match u8::decode(buf)? {
            0 => Ok(None),
            1 => Ok(Some(T::decode(buf)?)),
            _ => Err(Error::new(ErrorKind::InvalidData, "bad Optional tag")),
        }
    }
}

/// Schema `Point3`: three f32s, then a varint `u32`.
/// Build 209026 server 37 uses map-unit XYZ and a transform tag (zero is
/// untransformed); see `docs/re/209026-agent-position.md`. Other message
/// semantics are not established by this wire codec.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point3(pub [f32; 3], pub u32);

impl Encode for Point3 {
    fn encode(&self, buf: &mut BytesMut) {
        self.0.encode(buf);
        self.1.encode(buf);
    }
}
impl Decode for Point3 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self(Decode::decode(buf)?, Decode::decode(buf)?))
    }
}

/// Schema `Guid`: 16 bytes in Windows GUID layout
/// (`u32 LE, u16 LE, u16 LE, [u8; 8]`). Kept in that wire order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Guid(pub [u8; 16]);

impl Encode for Guid {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_slice(&self.0);
    }
}
impl Decode for Guid {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self(take(buf)?))
    }
}

/// Schema `Address`: a Windows `sockaddr_in` (family 2) or `sockaddr_in6`
/// (family 23), padded to 28 bytes. The family is a u16 LE; the port is BE.
/// Build-209026 REA confirms the raw copy and htons port stores; see
/// `docs/re/209026-mission-join.md`.
impl Encode for SocketAddr {
    fn encode(&self, buf: &mut BytesMut) {
        match self {
            SocketAddr::V4(a) => {
                buf.put_u16_le(2);
                buf.put_u16(a.port());
                buf.put_slice(&a.ip().octets());
                buf.put_bytes(0, 20);
            }
            SocketAddr::V6(a) => {
                buf.put_u16_le(23);
                buf.put_u16(a.port());
                buf.put_u32_le(a.flowinfo());
                buf.put_slice(&a.ip().octets());
                buf.put_u32_le(a.scope_id());
            }
        }
    }
}
impl Decode for SocketAddr {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        match u16::decode(buf)? {
            2 => {
                let port = u16::from_be_bytes(take(buf)?);
                let ip = Ipv4Addr::from(take::<4>(buf)?);
                take::<20>(buf)?;
                Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)))
            }
            23 => {
                let port = u16::from_be_bytes(take(buf)?);
                let flow = u32::from_le_bytes(take(buf)?);
                let ip = Ipv6Addr::from(take::<16>(buf)?);
                let scope = u32::from_le_bytes(take(buf)?);
                Ok(SocketAddr::V6(SocketAddrV6::new(ip, port, flow, scope)))
            }
            _ => Err(Error::new(ErrorKind::InvalidData, "unknown address family")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T: Encode + Decode + PartialEq + std::fmt::Debug>(v: T) -> usize {
        let mut buf = BytesMut::new();
        v.encode(&mut buf);
        let len = buf.len();
        let mut slice = &buf[..];
        assert_eq!(T::decode(&mut slice).unwrap(), v);
        assert!(slice.is_empty());
        len
    }

    #[test]
    fn varint_lengths() {
        assert_eq!(round_trip(0u32), 1);
        assert_eq!(round_trip(0x7Fu32), 1);
        assert_eq!(round_trip(0x80u32), 2);
        assert_eq!(round_trip(0x3FFFu32), 2);
        assert_eq!(round_trip(0x4000u32), 3);
        assert_eq!(round_trip(u32::MAX), 5);
    }

    #[test]
    fn varint_rejects_overflow_and_truncation() {
        assert!(u32::decode(&mut &[0xFF, 0xFF, 0xFF, 0xFF, 0x7F][..]).is_err());
        assert!(u32::decode(&mut &[0x80, 0x80][..]).is_err());
        assert!(u32::decode(&mut &[0x80, 0x80, 0x80, 0x80, 0x80, 0x01][..]).is_err());
    }

    #[test]
    fn fixed_and_composite_types() {
        assert_eq!(round_trip(0xBEEFu16), 2);
        assert_eq!(round_trip(0x0123_4567_89AB_CDEFu64), 8);
        assert_eq!(round_trip([1.0f32, -2.5, 3.25]), 12);
        assert_eq!(round_trip(Point3([1.0, 2.0, 3.0], 15)), 13);
        assert_eq!(round_trip(Guid([0x11; 16])), 16);
        assert_eq!(round_trip(Some(300u32)), 3);
        assert_eq!(round_trip(None::<u32>), 1);
        assert_eq!(round_trip("127.0.0.1:6112".parse::<SocketAddr>().unwrap()), 28);
        assert_eq!(round_trip("[::1]:6112".parse::<SocketAddr>().unwrap()), 28);
    }

    #[test]
    fn optional_rejects_bad_tag() {
        assert!(Option::<u8>::decode(&mut &[2, 0][..]).is_err());
    }

    #[test]
    fn address_matches_build_209026_wire_bytes() {
        // Exact bytes catch a shared encode/decode byte-order mistake that a
        // round trip cannot. Port 6112 is 0x17e0 in network order.
        let fixtures: [(&str, [u8; 28]); 2] = [
            (
                "127.1.0.2:6112",
                [
                    2, 0, 0x17, 0xe0, 127, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0,
                ],
            ),
            (
                "[2001:db8::1]:6112",
                [
                    23, 0, 0x17, 0xe0, 0, 0, 0, 0, 0x20, 1, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 1, 0, 0, 0, 0,
                ],
            ),
        ];
        for (text, wire) in fixtures {
            let address: SocketAddr = text.parse().unwrap();
            let mut encoded = BytesMut::new();
            address.encode(&mut encoded);
            assert_eq!(encoded.as_ref(), wire);

            let mut with_next_field = wire.to_vec();
            with_next_field.push(0xa5);
            let mut input = with_next_field.as_slice();
            assert_eq!(SocketAddr::decode(&mut input).unwrap(), address);
            assert_eq!(input, [0xa5]);
            for length in 0..wire.len() {
                assert!(SocketAddr::decode(&mut &wire[..length]).is_err());
            }
        }
    }
}
