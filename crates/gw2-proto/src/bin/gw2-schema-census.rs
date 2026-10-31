//! `gw2-schema-census <protocols.xml>`: message counts per direction, and how
//! many messages the dynamic codec can decode with only the confirmed encodings.

use std::collections::BTreeMap;

use gw2_proto::dynamic::{Rules, is_supported};
use gw2_proto::schema::{Field, parse};

fn kinds(fields: &[Field], acc: &mut BTreeMap<String, usize>) {
    for f in fields {
        *acc.entry(f.kind.clone()).or_default() += 1;
        if let Some(t) = &f.type_name {
            *acc.entry(t.clone()).or_default() += 1;
        }
        kinds(&f.fields, acc);
    }
}

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: gw2-schema-census <protocols.xml>");
        std::process::exit(2);
    };
    let xml = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(1)
    });
    let s = parse(&xml).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1)
    });
    let rules = Rules::default();
    println!("build: {:?}", s.build_id);
    for p in &s.protocols {
        let client: Vec<_> = p.groups.iter().flat_map(|g| &g.client).collect();
        let server: Vec<_> = p.groups.iter().flat_map(|g| &g.server).collect();
        let ok = |v: &[&gw2_proto::schema::Message]| v.iter().filter(|m| is_supported(&m.fields, &rules)).count();
        println!(
            "protocol {}: {} groups, client {} ({} decodable now), server {} ({} decodable now)",
            p.name, p.groups.len(), client.len(), ok(&client), server.len(), ok(&server)
        );
    }
    let mut acc = BTreeMap::new();
    for m in s.messages() {
        kinds(&m.fields, &mut acc);
    }
    println!("field kinds (incl. container element types): {acc:?}");
}
