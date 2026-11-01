//! Minimal TLS 1.2 server for the portal's `/Auth/StartTls` upgrade.
//!
//! Build 209026's ClientHello (docs/re/209026-login.md) offers only static-RSA suites
//! (`0x002f 0x0035 0x003c 0x003d`) with no extensions. rustls doesn't do static RSA key
//! exchange, so this implements exactly one suite, the one offered first:
//! **`TLS_RSA_WITH_AES_128_CBC_SHA` (0x002f)**, per RFC 5246:
//! - handshake: ServerHello (no extensions), Certificate, ServerHelloDone, then the
//!   client's ClientKeyExchange (RSA PKCS#1 v1.5 premaster), ChangeCipherSpec, Finished;
//! - keys: SHA-256 PRF; key block = client MAC, server MAC (20 each), client key,
//!   server key (16 each). TLS 1.2 CBC has explicit per-record IVs, so the block has no IVs;
//! - records: MAC-then-encrypt, HMAC-SHA1 over `seq ‖ type ‖ version ‖ length ‖ data`,
//!   AES-128-CBC with a random explicit IV and TLS padding.
//!
//! Scope: one connection, no renegotiation, no session resumption. A bad premaster
//! falls back to a random one (RFC 5246 §7.4.7.1) so failures surface as a Finished
//! mismatch, not an oracle.

use std::fmt;
use std::io::{Read, Write};

use aes::Aes128;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::NoPadding};
use hmac::{Hmac, Mac};
use rand::RngCore;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use sha1::Sha1;
use sha2::{Digest, Sha256};

pub const TLS12: [u8; 2] = [3, 3];
pub const SUITE_RSA_AES128_CBC_SHA: u16 = 0x002f;

const CT_CCS: u8 = 20;
const CT_ALERT: u8 = 21;
const CT_HANDSHAKE: u8 = 22;
const CT_APP: u8 = 23;
const MAX_RECORD: usize = 16384 + 2048;

#[derive(Debug)]
pub enum TlsError {
    Io(std::io::Error),
    Protocol(String),
    /// The peer sent an alert `(level, description)`.
    Alert(u8, u8),
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "tls io: {e}"),
            Self::Protocol(m) => write!(f, "tls protocol: {m}"),
            Self::Alert(l, d) => write!(f, "tls alert level {l} description {d}"),
        }
    }
}
impl std::error::Error for TlsError {}
impl From<std::io::Error> for TlsError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

fn proto<T>(m: impl Into<String>) -> Result<T, TlsError> {
    Err(TlsError::Protocol(m.into()))
}

/// TLS 1.2 PRF with HMAC-SHA256 (RFC 5246 §5).
pub fn prf(secret: &[u8], label: &[u8], seed: &[u8], out_len: usize) -> Vec<u8> {
    let mac = |data: &[&[u8]]| {
        let mut m = <Hmac<Sha256> as Mac>::new_from_slice(secret).expect("hmac takes any key");
        for d in data {
            m.update(d);
        }
        m.finalize().into_bytes().to_vec()
    };
    let mut a = mac(&[label, seed]);
    let mut out = Vec::with_capacity(out_len + 32);
    while out.len() < out_len {
        out.extend(mac(&[&a, label, seed]));
        a = mac(&[&a]);
    }
    out.truncate(out_len);
    out
}

/// Server identity: private key plus the certificate chain (DER, leaf first).
pub struct ServerConfig {
    pub key: RsaPrivateKey,
    pub cert_chain: Vec<Vec<u8>>,
}

struct Dir {
    mac: Vec<u8>,
    key: [u8; 16],
    seq: u64,
}

impl Dir {
    fn mac(&self, ct: u8, data: &[u8]) -> Vec<u8> {
        let mut m = <Hmac<Sha1> as Mac>::new_from_slice(&self.mac).expect("hmac key");
        m.update(&self.seq.to_be_bytes());
        m.update(&[ct, TLS12[0], TLS12[1]]);
        m.update(&(data.len() as u16).to_be_bytes());
        m.update(data);
        m.finalize().into_bytes().to_vec()
    }

    fn seal(&mut self, ct: u8, data: &[u8]) -> Vec<u8> {
        let mut body = data.to_vec();
        body.extend(self.mac(ct, data));
        let pad = 15 - (body.len() % 16);
        body.extend(std::iter::repeat_n(pad as u8, pad + 1));
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);
        let n = body.len();
        cbc::Encryptor::<Aes128>::new(&self.key.into(), &iv.into())
            .encrypt_padded_mut::<NoPadding>(&mut body, n)
            .expect("block aligned");
        self.seq += 1;
        let mut out = iv.to_vec();
        out.extend(body);
        out
    }

    fn open(&mut self, ct: u8, frag: &[u8]) -> Result<Vec<u8>, TlsError> {
        if frag.len() < 16 + 32 || !frag.len().is_multiple_of(16) {
            return proto("bad ciphertext length");
        }
        let (iv, ct_body) = frag.split_at(16);
        let mut body = ct_body.to_vec();
        cbc::Decryptor::<Aes128>::new(&self.key.into(), iv.into())
            .decrypt_padded_mut::<NoPadding>(&mut body)
            .map_err(|_| TlsError::Protocol("cbc".into()))?;
        let pad = *body.last().unwrap_or(&0) as usize;
        let ok_pad = pad + 1 + 20 <= body.len() && body[body.len() - pad - 1..].iter().all(|&b| b as usize == pad);
        let pad = if ok_pad { pad } else { 0 };
        let data_len = body.len().saturating_sub(pad + 1 + 20);
        let (data, rest) = body.split_at(data_len);
        let want = self.mac(ct, data);
        let mac_ok = rest.len() >= 20 && want.iter().zip(&rest[..20]).fold(0u8, |a, (x, y)| a | (x ^ y)) == 0;
        if !(ok_pad && mac_ok) {
            return proto("bad record mac");
        }
        self.seq += 1;
        Ok(data.to_vec())
    }
}

/// An established TLS 1.2 session over `S`.
pub struct TlsStream<S> {
    inner: S,
    rx: Dir,
    tx: Dir,
    raw: Vec<u8>,
    plain: Vec<u8>,
    /// ClientHello random, for logging.
    pub client_random: [u8; 32],
    /// Suites the client offered.
    pub offered_suites: Vec<u16>,
}

/// Reads one TLS record (`type, fragment`) from `buf` + `s`.
fn read_record<S: Read>(s: &mut S, buf: &mut Vec<u8>) -> Result<(u8, Vec<u8>), TlsError> {
    loop {
        if buf.len() >= 5 {
            let len = u16::from_be_bytes([buf[3], buf[4]]) as usize;
            if len > MAX_RECORD {
                return proto(format!("record too large ({len})"));
            }
            if buf.len() >= 5 + len {
                let ct = buf[0];
                let frag = buf[5..5 + len].to_vec();
                buf.drain(..5 + len);
                return Ok((ct, frag));
            }
        }
        let mut chunk = [0u8; 8192];
        let n = s.read(&mut chunk)?;
        if n == 0 {
            return Err(TlsError::Io(std::io::ErrorKind::UnexpectedEof.into()));
        }
        buf.extend_from_slice(&chunk[..n]);
    }
}

fn record(ct: u8, frag: &[u8]) -> Vec<u8> {
    let mut out = vec![ct, TLS12[0], TLS12[1]];
    out.extend((frag.len() as u16).to_be_bytes());
    out.extend_from_slice(frag);
    out
}

fn hs(msg_type: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![msg_type];
    out.extend(&(body.len() as u32).to_be_bytes()[1..]);
    out.extend_from_slice(body);
    out
}

/// Pulls whole handshake messages out of plaintext handshake records.
struct HsReader {
    pending: Vec<u8>,
}

impl HsReader {
    fn next<S: Read>(&mut self, s: &mut S, raw: &mut Vec<u8>) -> Result<(u8, Vec<u8>, Vec<u8>), TlsError> {
        loop {
            if self.pending.len() >= 4 {
                let len = u32::from_be_bytes([0, self.pending[1], self.pending[2], self.pending[3]]) as usize;
                if self.pending.len() >= 4 + len {
                    let whole: Vec<u8> = self.pending.drain(..4 + len).collect();
                    return Ok((whole[0], whole[4..].to_vec(), whole));
                }
            }
            match read_record(s, raw)? {
                (CT_HANDSHAKE, frag) => self.pending.extend(frag),
                (CT_ALERT, a) if a.len() >= 2 => return Err(TlsError::Alert(a[0], a[1])),
                (ct, _) => return proto(format!("unexpected record type {ct} in handshake")),
            }
        }
    }
}

/// Parsed ClientHello fields we care about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientHello {
    pub version: [u8; 2],
    pub random: [u8; 32],
    pub suites: Vec<u16>,
}

pub fn parse_client_hello(body: &[u8]) -> Result<ClientHello, TlsError> {
    let get = |r: std::ops::Range<usize>| body.get(r).ok_or_else(|| TlsError::Protocol("short ClientHello".into()));
    let version = [body.first().copied().unwrap_or(0), body.get(1).copied().unwrap_or(0)];
    let random: [u8; 32] = get(2..34)?.try_into().expect("32");
    let sid_len = *get(34..35)?.first().expect("1") as usize;
    let p = 35 + sid_len;
    let cs_len = u16::from_be_bytes(get(p..p + 2)?.try_into().expect("2")) as usize;
    let suites = get(p + 2..p + 2 + cs_len)?.as_chunks::<2>().0.iter().map(|c| u16::from_be_bytes(*c)).collect();
    Ok(ClientHello { version, random, suites })
}

/// Runs the server handshake. `pre` holds bytes already read past the STS layer.
pub fn accept<S: Read + Write>(mut s: S, cfg: &ServerConfig, pre: Vec<u8>) -> Result<TlsStream<S>, TlsError> {
    let mut raw = pre;
    let mut hr = HsReader { pending: Vec::new() };
    let mut transcript = Sha256::new();

    let (t, body, whole) = hr.next(&mut s, &mut raw)?;
    if t != 1 {
        return proto(format!("expected ClientHello, got handshake type {t}"));
    }
    transcript.update(&whole);
    let ch = parse_client_hello(&body)?;
    if ch.version != TLS12 {
        return proto(format!("client version {:?}", ch.version));
    }
    if !ch.suites.contains(&SUITE_RSA_AES128_CBC_SHA) {
        return proto(format!("no supported suite in {:04x?}", ch.suites));
    }

    let mut server_random = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut server_random);
    let mut sh = TLS12.to_vec();
    sh.extend(server_random);
    sh.push(0); // empty session id
    sh.extend(SUITE_RSA_AES128_CBC_SHA.to_be_bytes());
    sh.push(0); // null compression; no extensions (the client sent none)
    let mut certs: Vec<u8> = Vec::new();
    for c in &cfg.cert_chain {
        certs.extend(&(c.len() as u32).to_be_bytes()[1..]);
        certs.extend(c);
    }
    let mut cert_body = (certs.len() as u32).to_be_bytes()[1..].to_vec();
    cert_body.extend(certs);
    let mut flight = Vec::new();
    for m in [hs(2, &sh), hs(11, &cert_body), hs(14, &[])] {
        transcript.update(&m);
        flight.extend(m);
    }
    s.write_all(&record(CT_HANDSHAKE, &flight))?;
    s.flush()?;

    let (t, body, whole) = hr.next(&mut s, &mut raw)?;
    if t != 16 {
        return proto(format!("expected ClientKeyExchange, got handshake type {t}"));
    }
    transcript.update(&whole);
    if !hr.pending.is_empty() {
        return proto("unexpected handshake data after ClientKeyExchange");
    }
    let enc = body.get(2..).filter(|e| body.len() >= 2 && u16::from_be_bytes([body[0], body[1]]) as usize == e.len());
    let mut pms = [0u8; 48];
    rand::thread_rng().fill_bytes(&mut pms);
    pms[..2].copy_from_slice(&TLS12);
    if let Some(Ok(p)) = enc.map(|e| cfg.key.decrypt(Pkcs1v15Encrypt, e))
        && p.len() == 48
        && p[..2] == TLS12
    {
        pms.copy_from_slice(&p);
    }
    let seed = [ch.random.as_slice(), &server_random].concat();
    let master = prf(&pms, b"master secret", &seed, 48);
    let kb = prf(&master, b"key expansion", &[server_random.as_slice(), &ch.random].concat(), 72);
    let mut rx = Dir { mac: kb[0..20].to_vec(), key: kb[40..56].try_into().expect("16"), seq: 0 };
    let mut tx = Dir { mac: kb[20..40].to_vec(), key: kb[56..72].try_into().expect("16"), seq: 0 };

    match read_record(&mut s, &mut raw)? {
        (CT_CCS, b) if b == [1] => {}
        (CT_ALERT, a) if a.len() >= 2 => return Err(TlsError::Alert(a[0], a[1])),
        (ct, _) => return proto(format!("expected ChangeCipherSpec, got record type {ct}")),
    }
    let fin = match read_record(&mut s, &mut raw)? {
        (CT_HANDSHAKE, frag) => rx.open(CT_HANDSHAKE, &frag)?,
        (ct, _) => return proto(format!("expected Finished, got record type {ct}")),
    };
    let want = prf(&master, b"client finished", &transcript.clone().finalize(), 12);
    if fin.len() != 16 || fin[0] != 20 || fin[4..] != want[..] {
        return proto("client Finished mismatch (wrong server key or premaster)");
    }
    transcript.update(&fin);
    let verify = prf(&master, b"server finished", &transcript.finalize(), 12);
    let mut out = record(CT_CCS, &[1]);
    out.extend(record(CT_HANDSHAKE, &tx.seal(CT_HANDSHAKE, &hs(20, &verify))));
    s.write_all(&out)?;
    s.flush()?;
    Ok(TlsStream { inner: s, rx, tx, raw, plain: Vec::new(), client_random: ch.random, offered_suites: ch.suites })
}

impl<S: Read + Write> TlsStream<S> {
    /// Next chunk of application data (empty on close_notify).
    pub fn read_app(&mut self) -> Result<Vec<u8>, TlsError> {
        if !self.plain.is_empty() {
            return Ok(std::mem::take(&mut self.plain));
        }
        loop {
            let (ct, frag) = read_record(&mut self.inner, &mut self.raw)?;
            let data = self.rx.open(ct, &frag)?;
            match ct {
                CT_APP if !data.is_empty() => return Ok(data),
                CT_APP => continue,
                CT_ALERT if data.len() >= 2 && data[1] == 0 => return Ok(Vec::new()),
                CT_ALERT if data.len() >= 2 => return Err(TlsError::Alert(data[0], data[1])),
                _ => return proto(format!("unexpected record type {ct} after handshake")),
            }
        }
    }

    pub fn write_app(&mut self, data: &[u8]) -> Result<(), TlsError> {
        for chunk in data.chunks(16384) {
            let sealed = self.tx.seal(CT_APP, chunk);
            self.inner.write_all(&record(CT_APP, &sealed))?;
        }
        self.inner.flush()?;
        Ok(())
    }

    /// Sends a `close_notify` alert. The caller then drops or shuts down the stream.
    pub fn close_notify(&mut self) -> Result<(), TlsError> {
        let sealed = self.tx.seal(CT_ALERT, &[1, 0]);
        self.inner.write_all(&record(CT_ALERT, &sealed))?;
        self.inner.flush()?;
        Ok(())
    }

    pub fn get_ref(&self) -> &S {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unhex(s: &str) -> Vec<u8> {
        let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    #[test]
    fn prf_matches_openssl_tls1_prf_sha256() {
        // Expected value (100 bytes, so the A(i) chaining runs four times) from:
        //   openssl kdf -keylen 100 -kdfopt digest:SHA256
        //   -kdfopt hexsecret:000102030405060708090a0b0c0d0e0f
        //   -kdfopt seed:"test labelseed" TLS1-PRF
        let secret: Vec<u8> = (0u8..16).collect();
        let want = unhex(include_str!("../testdata/prf-sha256.hex"));
        assert_eq!(want.len(), 100);
        assert_eq!(prf(&secret, b"test label", b"seed", 100), want);
        // A shorter output is a prefix of the longer one.
        assert_eq!(prf(&secret, b"test label", b"seed", 72), want[..72]);
    }

    #[test]
    fn reads_captured_client_hello_record_from_build_209026() {
        // Run 37 capture (~/re/gw2/notes/run37-starttls-clienthello.log): the whole
        // 56-byte record the client sent right after our StartTls `200 OK`.
        let rec = unhex(
            "16 03 03 00 33 01 00 00 2f 03 03 20 f7 1d 6a 9e
             db 43 f4 8c a3 ce e8 8e 70 7f bf 4a 02 14 fd 82
             52 f2 bd 95 57 04 b6 46 01 f9 57 00 00 08 00 2f
             00 35 00 3c 00 3d 01 00",
        );
        assert_eq!(rec.len(), 56);
        // Split across the `pre` buffer and the stream to exercise record reassembly.
        let mut raw = rec[..7].to_vec();
        let mut s = std::io::Cursor::new(rec[7..].to_vec());
        let mut hr = HsReader { pending: Vec::new() };
        let (t, body, whole) = hr.next(&mut s, &mut raw).unwrap();
        assert_eq!(t, 1);
        assert_eq!(whole, rec[5..]);
        assert!(hr.pending.is_empty() && raw.is_empty());
        let ch = parse_client_hello(&body).unwrap();
        assert_eq!(ch.version, TLS12);
        assert_eq!(ch.suites, vec![0x002f, 0x0035, 0x003c, 0x003d]);
        assert_eq!(ch.random[..4], [0x20, 0xf7, 0x1d, 0x6a]);
        assert_eq!(ch.random[31], 0x57);
    }

    #[test]
    fn rejects_truncated_client_hello() {
        assert!(parse_client_hello(&[3, 3, 0]).is_err());
    }

    #[test]
    fn record_seal_open_round_trip_and_tamper() {
        let mk = || Dir { mac: vec![7; 20], key: [9; 16], seq: 0 };
        let (mut a, mut b) = (mk(), mk());
        for msg in [b"".as_slice(), b"hello", &[0xab; 100]] {
            let sealed = a.seal(CT_APP, msg);
            assert_eq!(b.open(CT_APP, &sealed).unwrap(), msg);
        }
        let mut sealed = a.seal(CT_APP, b"x");
        let n = sealed.len();
        sealed[n - 17] ^= 1;
        assert!(b.open(CT_APP, &sealed).is_err());
    }
}
