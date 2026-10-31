//! Minimal DNS responder for the offline sandbox: it logs every query and answers
//! A queries with a fixed IPv4 address (127.0.0.1). Other query types get an
//! empty NOERROR answer. Inside the `--network none` container this catches every
//! hostname the client tries, and points the next connection at our own
//! listeners.

use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub id: u16,
    pub name: String,
    pub qtype: u16,
    /// Offset just past the question section (QNAME + QTYPE + QCLASS).
    pub question_end: usize,
}

/// Parses the first question of a DNS query. Returns None for anything else
/// (responses, compressed or malformed names, no question).
pub fn parse_query(pkt: &[u8]) -> Option<Query> {
    if pkt.len() < 12 {
        return None;
    }
    let id = u16::from_be_bytes([pkt[0], pkt[1]]);
    let flags = u16::from_be_bytes([pkt[2], pkt[3]]);
    if flags & 0x8000 != 0 {
        return None; // a response, not a query
    }
    let qdcount = u16::from_be_bytes([pkt[4], pkt[5]]);
    if qdcount == 0 {
        return None;
    }
    let mut pos = 12;
    let mut labels = Vec::new();
    loop {
        let len = *pkt.get(pos)? as usize;
        pos += 1;
        if len == 0 {
            break;
        }
        if len & 0xC0 != 0 || labels.len() > 127 {
            return None; // compression pointers don't appear in queries we care about
        }
        let label = pkt.get(pos..pos + len)?;
        labels.push(String::from_utf8_lossy(label).to_ascii_lowercase());
        pos += len;
    }
    let qtype = u16::from_be_bytes([*pkt.get(pos)?, *pkt.get(pos + 1)?]);
    pkt.get(pos + 2..pos + 4)?; // QCLASS present
    Some(Query { id, name: labels.join("."), qtype, question_end: pos + 4 })
}

/// Builds the response: an A record (TTL 60) for A queries, no answers otherwise.
pub fn build_response(pkt: &[u8], q: &Query, addr: Ipv4Addr) -> Vec<u8> {
    let answer = q.qtype == 1;
    let mut out = Vec::with_capacity(q.question_end + 16);
    out.extend_from_slice(&q.id.to_be_bytes());
    // QR=1, opcode copied, AA=1, RD copied, RA=1, RCODE=0.
    let rd = pkt[2] & 0x01;
    let opcode = pkt[2] & 0x78;
    out.push(0x80 | opcode | 0x04 | rd);
    out.push(0x80);
    out.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT
    out.extend_from_slice(&(answer as u16).to_be_bytes()); // ANCOUNT
    out.extend_from_slice(&[0, 0, 0, 0]); // NSCOUNT, ARCOUNT
    out.extend_from_slice(&pkt[12..q.question_end]);
    if answer {
        out.extend_from_slice(&[0xC0, 0x0C]); // name: pointer to the question
        out.extend_from_slice(&1u16.to_be_bytes()); // TYPE A
        out.extend_from_slice(&1u16.to_be_bytes()); // CLASS IN
        out.extend_from_slice(&60u32.to_be_bytes()); // TTL
        out.extend_from_slice(&4u16.to_be_bytes());
        out.extend_from_slice(&addr.octets());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(name: &str, qtype: u16) -> Vec<u8> {
        let mut p = vec![0x12, 0x34, 0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0];
        for l in name.split('.') {
            p.push(l.len() as u8);
            p.extend_from_slice(l.as_bytes());
        }
        p.push(0);
        p.extend_from_slice(&qtype.to_be_bytes());
        p.extend_from_slice(&1u16.to_be_bytes());
        p
    }

    #[test]
    fn answers_a_queries_with_loopback() {
        let p = query("Auth1.Example.COM", 1);
        let q = parse_query(&p).unwrap();
        assert_eq!(q.name, "auth1.example.com");
        assert_eq!(q.qtype, 1);
        let r = build_response(&p, &q, Ipv4Addr::LOCALHOST);
        assert_eq!(&r[..2], &[0x12, 0x34]);
        assert_eq!(r[2] & 0x80, 0x80);
        assert_eq!(u16::from_be_bytes([r[6], r[7]]), 1);
        assert_eq!(&r[r.len() - 4..], &[127, 0, 0, 1]);
        assert_eq!(&r[12..q.question_end], &p[12..]);
    }

    #[test]
    fn other_types_get_empty_answers_and_junk_is_ignored() {
        let p = query("example.com", 28);
        let q = parse_query(&p).unwrap();
        let r = build_response(&p, &q, Ipv4Addr::LOCALHOST);
        assert_eq!(u16::from_be_bytes([r[6], r[7]]), 0);
        assert_eq!(r.len(), q.question_end);
        assert!(parse_query(&[0u8; 5]).is_none());
        let mut resp = query("x.y", 1);
        resp[2] |= 0x80;
        assert!(parse_query(&resp).is_none());
        let mut trunc = query("abc.def", 1);
        trunc.truncate(15);
        assert!(parse_query(&trunc).is_none());
    }
}
