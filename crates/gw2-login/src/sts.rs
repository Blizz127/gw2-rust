//! STS/1.0 message codec: HTTP-like text framing used by the portal.
//!
//! Shape (public, unverified for our build):
//!
//! ```text
//! P /Sts/Connect STS/1.0\r\n      request: method, path, protocol
//! l:123\r\n                       header "l" = body length in bytes
//! s:1\r\n                         header "s" = sequence number
//! \r\n
//! <body: l bytes, usually XML>
//!
//! STS/1.0 200 OK\r\n              response: protocol, status, reason
//! s:1R\r\n                        reply sequence is "<n>R"
//! l:45\r\n
//! \r\n
//! <body>
//! ```
//!
//! After `/Auth/StartTls` the stream switches to TLS-style records. This codec
//! does not handle that part.

use std::fmt;

/// Upper bound on one header block and one body. This keeps a hostile or
/// misparsed stream from causing unbounded buffering.
pub const MAX_HEAD: usize = 16 * 1024;
pub const MAX_BODY: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartLine {
    Request { method: String, path: String, protocol: String },
    Response { protocol: String, status: u16, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub start: StartLine,
    /// Headers in wire order; keys are kept as sent.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StsError {
    HeadTooLarge,
    BodyTooLarge(usize),
    BadStartLine(String),
    BadHeader(String),
    BadLength(String),
    NotUtf8,
}

impl fmt::Display for StsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeadTooLarge => write!(f, "STS header block over {MAX_HEAD} bytes"),
            Self::BodyTooLarge(n) => write!(f, "STS body of {n} bytes over {MAX_BODY}"),
            Self::BadStartLine(l) => write!(f, "bad STS start line {l:?}"),
            Self::BadHeader(l) => write!(f, "bad STS header {l:?}"),
            Self::BadLength(v) => write!(f, "bad STS length header {v:?}"),
            Self::NotUtf8 => write!(f, "STS header block is not UTF-8"),
        }
    }
}
impl std::error::Error for StsError {}

impl Message {
    pub fn request(method: &str, path: &str, seq: u32, body: impl Into<Vec<u8>>) -> Self {
        Self {
            start: StartLine::Request {
                method: method.into(),
                path: path.into(),
                protocol: "STS/1.0".into(),
            },
            headers: vec![("s".into(), seq.to_string())],
            body: body.into(),
        }
    }

    /// A reply to request sequence `seq` (`s:<seq>R`).
    pub fn reply(status: u16, reason: &str, seq: u32, body: impl Into<Vec<u8>>) -> Self {
        Self {
            start: StartLine::Response {
                protocol: "STS/1.0".into(),
                status,
                reason: reason.into(),
            },
            headers: vec![("s".into(), format!("{seq}R"))],
            body: body.into(),
        }
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// Request sequence number from `s:`. A trailing `R` and `;key=value` parameters
    /// are accepted (build 209026 sends `s:1;timeout=29000` on `/Auth/StartTls`).
    pub fn seq(&self) -> Option<u32> {
        let v = self.header("s")?;
        v.split(';').next()?.trim_end_matches('R').parse().ok()
    }

    /// A `;key=value` parameter of the `s:` header (e.g. `timeout`).
    pub fn seq_param(&self, key: &str) -> Option<&str> {
        self.header("s")?.split(';').skip(1).find_map(|kv| kv.split_once('=').filter(|(k, _)| *k == key).map(|(_, v)| v))
    }

    pub fn path(&self) -> Option<&str> {
        match &self.start {
            StartLine::Request { path, .. } => Some(path),
            StartLine::Response { .. } => None,
        }
    }

    /// Serializes the message. Writes `l:` from the actual body length, replacing
    /// any stale `l` header.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = String::new();
        match &self.start {
            StartLine::Request { method, path, protocol } => {
                out.push_str(&format!("{method} {path} {protocol}\r\n"))
            }
            StartLine::Response { protocol, status, reason } => {
                out.push_str(&format!("{protocol} {status} {reason}\r\n"))
            }
        }
        for (k, v) in self.headers.iter().filter(|(k, _)| k != "l") {
            out.push_str(&format!("{k}:{v}\r\n"));
        }
        if !self.body.is_empty() {
            out.push_str(&format!("l:{}\r\n", self.body.len()));
        }
        out.push_str("\r\n");
        let mut bytes = out.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

fn find_head_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

/// Tries to decode one message from the front of `buf`.
///
/// Returns `Ok(Some((message, consumed)))`, `Ok(None)` if more bytes are needed,
/// or an error if the stream is not STS (e.g. it already switched to TLS records).
pub fn decode(buf: &[u8]) -> Result<Option<(Message, usize)>, StsError> {
    let Some(head_end) = find_head_end(buf) else {
        if buf.len() > MAX_HEAD {
            return Err(StsError::HeadTooLarge);
        }
        // Reject early if the first bytes cannot start an STS line (e.g. 0x16 TLS record).
        if let Some(&b) = buf.first()
            && !b.is_ascii_alphabetic()
        {
            return Err(StsError::BadStartLine(format!("first byte 0x{b:02x}")));
        }
        return Ok(None);
    };
    if head_end > MAX_HEAD {
        return Err(StsError::HeadTooLarge);
    }
    let head = std::str::from_utf8(&buf[..head_end]).map_err(|_| StsError::NotUtf8)?;
    let mut lines = head.split("\r\n");
    let first = lines.next().unwrap_or_default();
    let start = parse_start(first)?;

    let mut headers = Vec::new();
    for line in lines {
        let (k, v) = line.split_once(':').ok_or_else(|| StsError::BadHeader(line.into()))?;
        headers.push((k.trim().to_string(), v.trim().to_string()));
    }
    let body_len = match headers.iter().find(|(k, _)| k == "l") {
        Some((_, v)) => v.parse::<usize>().map_err(|_| StsError::BadLength(v.clone()))?,
        None => 0,
    };
    if body_len > MAX_BODY {
        return Err(StsError::BodyTooLarge(body_len));
    }
    let total = head_end + 4 + body_len;
    if buf.len() < total {
        return Ok(None);
    }
    let body = buf[head_end + 4..total].to_vec();
    Ok(Some((Message { start, headers, body }, total)))
}

fn parse_start(line: &str) -> Result<StartLine, StsError> {
    let bad = || StsError::BadStartLine(line.into());
    let mut parts = line.splitn(3, ' ');
    let a = parts.next().ok_or_else(bad)?;
    let b = parts.next().ok_or_else(bad)?;
    let c = parts.next().unwrap_or_default();
    if a.starts_with("STS/") {
        let status = b.parse().map_err(|_| bad())?;
        Ok(StartLine::Response { protocol: a.into(), status, reason: c.into() })
    } else if c.starts_with("STS/") && b.starts_with('/') {
        Ok(StartLine::Request { method: a.into(), path: b.into(), protocol: c.into() })
    } else {
        Err(bad())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONNECT_BODY: &str = "<Request>\n<ConnType>400</ConnType>\n<ProductName>gw2</ProductName>\n<AppIndex>1</AppIndex>\n<Program>101</Program>\n<Build>1002</Build>\n</Request>\n";

    #[test]
    fn request_round_trip_with_split_input() {
        let m = Message::request("P", "/Sts/Connect", 1, CONNECT_BODY);
        let wire = m.encode();
        assert!(wire.starts_with(b"P /Sts/Connect STS/1.0\r\ns:1\r\nl:"));
        for cut in 0..wire.len() {
            assert_eq!(decode(&wire[..cut]).unwrap(), None, "cut at {cut}");
        }
        let (got, used) = decode(&wire).unwrap().unwrap();
        assert_eq!(used, wire.len());
        assert_eq!(got.path(), Some("/Sts/Connect"));
        assert_eq!(got.seq(), Some(1));
        assert_eq!(got.body, CONNECT_BODY.as_bytes());
    }

    #[test]
    fn reply_and_pipelined_messages() {
        let a = Message::reply(200, "OK", 3, "<Reply/>\n").encode();
        let b = Message::request("P", "/Auth/StartTls", 4, Vec::new()).encode();
        let wire = [a.clone(), b.clone()].concat();
        let (m1, n1) = decode(&wire).unwrap().unwrap();
        assert_eq!(n1, a.len());
        assert_eq!(m1.start, StartLine::Response { protocol: "STS/1.0".into(), status: 200, reason: "OK".into() });
        assert_eq!(m1.header("s"), Some("3R"));
        assert_eq!(m1.seq(), Some(3));
        let (m2, n2) = decode(&wire[n1..]).unwrap().unwrap();
        assert_eq!(n2, b.len());
        assert_eq!(m2.path(), Some("/Auth/StartTls"));
        assert!(m2.body.is_empty());
    }

    #[test]
    fn captured_starttls_request_from_build_209026() {
        let wire = b"P /Auth/StartTls STS/1.0\r\ns:1;timeout=29000\r\nl:11\r\n\r\n<Request/>\n";
        let (m, used) = decode(wire).unwrap().unwrap();
        assert_eq!(used, wire.len());
        assert_eq!(m.path(), Some("/Auth/StartTls"));
        assert_eq!(m.seq(), Some(1));
        assert_eq!(m.seq_param("timeout"), Some("29000"));
        assert_eq!(m.body, b"<Request/>\n");
    }

    #[test]
    fn rejects_non_sts_streams() {
        // A TLS handshake record must not be buffered forever as "incomplete STS".
        assert!(decode(&[0x16, 0x03, 0x03, 0x00, 0x10]).is_err());
        assert!(decode(b"GARBAGE\r\n\r\n").is_err());
        assert!(decode(b"P /x STS/1.0\r\nnocolon\r\n\r\n").is_err());
        assert!(decode(b"P /x STS/1.0\r\nl:abc\r\n\r\n").is_err());
        assert_eq!(
            decode(format!("P /x STS/1.0\r\nl:{}\r\n\r\n", MAX_BODY + 1).as_bytes()),
            Err(StsError::BodyTooLarge(MAX_BODY + 1))
        );
        assert_eq!(decode(&vec![b'A'; MAX_HEAD + 1]), Err(StsError::HeadTooLarge));
    }
}
