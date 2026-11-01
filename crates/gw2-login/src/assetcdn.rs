//! Minimal asset-CDN (`assetcdn.101.ArenaNetworks.com`, HTTP/1.1 on port 80) for the
//! offline sandbox.
//!
//! Observed for build 209026 (docs/re/209026-connect-path.md): the launcher's first
//! request is the patch check
//!
//! ```text
//! GET /latest64/101 HTTP/1.1
//! Cookie: authCookie=access=/latest/*!/manifest/program/*!/program/*~md5=…
//! Host: assetcdn.101.ArenaNetworks.com
//! Connection: keep-alive
//! ```
//!
//! REA (`FUN_140d8c630`, ev_2e70a2e9…): the client copies up to 127 bytes of the body and
//! reads **five whitespace-separated integers** with `strtoul(…, 0)`. A missing or
//! unparsable field becomes 0. `FUN_140fe17c0` uses field 0 as the build, and fields 3
//! and 4 as the file id and size of the `ARMF` root manifest it downloads next. The
//! meaning of fields 1 and 2 is inferred from their order (exe file id and size).
//! `AssetHttpRequest.cpp` builds the path as `/%s/%u` with `latest64` (64-bit client) or
//! `latest` (32-bit), and builds manifests as `/manifest/program/%u/%u`.

/// The five numbers of a `/latest64/<program>` reply, in wire order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Latest {
    pub build: u32,
    pub exe_file_id: u32,
    pub exe_size: u32,
    pub manifest_file_id: u32,
    pub manifest_size: u32,
}

impl Latest {
    /// Reply body: the five numbers separated by spaces, with a trailing newline.
    pub fn body(&self) -> String {
        format!(
            "{} {} {} {} {}\n",
            self.build, self.exe_file_id, self.exe_size, self.manifest_file_id, self.manifest_size
        )
    }

    /// Parses `"b e es m ms"` (any whitespace). Exactly five fields are required.
    pub fn parse(s: &str) -> Option<Self> {
        let v: Vec<u32> = s.split_whitespace().map(|t| t.parse().ok()).collect::<Option<_>>()?;
        let [build, exe_file_id, exe_size, manifest_file_id, manifest_size] = v[..] else {
            return None;
        };
        Some(Self { build, exe_file_id, exe_size, manifest_file_id, manifest_size })
    }
}

/// One parsed HTTP request head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str())
    }

    fn content_length(&self) -> usize {
        self.header("Content-Length").and_then(|v| v.trim().parse().ok()).unwrap_or(0)
    }
}

/// Upper bound for one request head, so a stuck or hostile stream can't buffer forever.
pub const MAX_HEAD: usize = 16 * 1024;

/// Decodes one request from the front of `buf`. Returns the request and the bytes
/// consumed (head plus any `Content-Length` body), `Ok(None)` if more bytes are needed,
/// or an error message.
pub fn decode_request(buf: &[u8]) -> Result<Option<(Request, usize)>, String> {
    let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") else {
        return if buf.len() > MAX_HEAD { Err("request head too large".into()) } else { Ok(None) };
    };
    let head = std::str::from_utf8(&buf[..end]).map_err(|_| "request head is not UTF-8")?;
    let mut lines = head.split("\r\n");
    let mut start = lines.next().unwrap_or_default().split(' ');
    let (Some(method), Some(path), Some(proto)) = (start.next(), start.next(), start.next()) else {
        return Err(format!("bad request line {head:?}"));
    };
    if !proto.starts_with("HTTP/") {
        return Err(format!("not HTTP: {proto:?}"));
    }
    let mut headers = Vec::new();
    for l in lines {
        let (k, v) = l.split_once(':').ok_or_else(|| format!("bad header {l:?}"))?;
        headers.push((k.trim().to_string(), v.trim().to_string()));
    }
    let req = Request { method: method.into(), path: path.into(), headers };
    let total = end + 4 + req.content_length();
    if buf.len() < total {
        return Ok(None);
    }
    Ok(Some((req, total)))
}

/// What the CDN answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// `/latest64/<program>` or `/latest/<program>`.
    Latest { program: u32, bits64: bool },
    Other,
}

pub fn route(req: &Request) -> Route {
    let mut parts = req.path.trim_start_matches('/').split('/');
    match (parts.next(), parts.next().and_then(|p| p.parse().ok()), parts.next()) {
        (Some("latest64"), Some(program), None) => Route::Latest { program, bits64: true },
        (Some("latest"), Some(program), None) => Route::Latest { program, bits64: false },
        _ => Route::Other,
    }
}

/// A complete HTTP/1.1 response that keeps the connection open.
pub fn response(status: u16, reason: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n",
        body.len()
    )
    .into_bytes();
    out.extend_from_slice(body);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact request captured from build 209026 (run 21), authCookie hash elided.
    const CAPTURED: &[u8] = b"GET /latest64/101 HTTP/1.1\r\nCookie: authCookie=access=/latest/*!/manifest/program/*!/program/*~md5=00\r\nCache-Control: no-cache\r\nPragma: no-cache\r\nHost: assetcdn.101.ArenaNetworks.com\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n";

    #[test]
    fn decodes_the_captured_patch_check() {
        for cut in 0..CAPTURED.len() {
            assert_eq!(decode_request(&CAPTURED[..cut]).unwrap(), None, "cut {cut}");
        }
        let (req, used) = decode_request(CAPTURED).unwrap().unwrap();
        assert_eq!(used, CAPTURED.len());
        assert_eq!(req.method, "GET");
        assert_eq!(req.header("host"), Some("assetcdn.101.ArenaNetworks.com"));
        assert_eq!(route(&req), Route::Latest { program: 101, bits64: true });
    }

    #[test]
    fn pipelined_requests_and_routes() {
        let wire = [CAPTURED, b"GET /manifest/program/101/12345 HTTP/1.1\r\n\r\n"].concat();
        let (_, n) = decode_request(&wire).unwrap().unwrap();
        let (r2, n2) = decode_request(&wire[n..]).unwrap().unwrap();
        assert_eq!(n + n2, wire.len());
        assert_eq!(route(&r2), Route::Other);
        let (r3, _) = decode_request(b"GET /latest/101 HTTP/1.1\r\n\r\n").unwrap().unwrap();
        assert_eq!(route(&r3), Route::Latest { program: 101, bits64: false });
        assert!(decode_request(b"P /Sts/Connect STS/1.0\r\n\r\n").is_err());
        assert!(decode_request(&vec![b'a'; MAX_HEAD + 1]).is_err());
    }

    #[test]
    fn latest_body_round_trips_and_matches_client_parse_shape() {
        let l = Latest { build: 209026, exe_file_id: 1, exe_size: 2, manifest_file_id: 3, manifest_size: 4 };
        assert_eq!(l.body(), "209026 1 2 3 4\n");
        assert_eq!(Latest::parse(&l.body()), Some(l));
        assert!(l.body().len() < 128, "client copies at most 127 bytes");
        assert_eq!(Latest::parse("1 2 3 4"), None);
        assert_eq!(Latest::parse("1 2 3 4 x"), None);
    }

    #[test]
    fn response_has_length_and_keep_alive() {
        let r = response(200, "OK", "text/plain", b"abc");
        let s = String::from_utf8(r).unwrap();
        assert!(s.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(s.contains("Content-Length: 3\r\n"));
        assert!(s.ends_with("\r\n\r\nabc"));
    }
}
