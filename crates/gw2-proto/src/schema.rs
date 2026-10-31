//! Loader for `protocols.xml` message schemas (ascalon's format).
//!
//! The file format follows ascalon's `protocol-schema` crate (MIT, commit
//! `03fb60b`). This is an independent stable-Rust implementation.

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub build_id: Option<u32>,
    pub protocols: Vec<Protocol>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Protocol {
    pub name: String,
    pub groups: Vec<MessageGroup>,
}

/// One `<Messages>` block. Client and server messages are registered together.
#[derive(Debug, Clone, PartialEq)]
pub struct MessageGroup {
    pub name: String,
    /// Client → server.
    pub client: Vec<Message>,
    /// Server → client.
    pub server: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: u16,
    pub name: String,
    /// Address of the handler/sender seen at capture time, if recorded.
    pub addr: Option<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Schema type tag, e.g. `Dword`, `ArrayVarSmall`, `Optional`.
    pub kind: String,
    pub name: String,
    pub size: Option<u32>,
    /// Element type for single-primitive containers (`TypeName` attribute).
    pub type_name: Option<String>,
    pub fields: Vec<Field>,
}

impl Schema {
    pub fn messages(&self) -> impl Iterator<Item = &Message> {
        self.protocols
            .iter()
            .flat_map(|p| p.groups.iter())
            .flat_map(|g| g.client.iter().chain(g.server.iter()))
    }
}

#[derive(Debug)]
pub struct SchemaError(pub String);

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "schema: {}", self.0)
    }
}
impl std::error::Error for SchemaError {}

type R<T> = Result<T, SchemaError>;

fn err(e: impl std::fmt::Display) -> SchemaError {
    SchemaError(e.to_string())
}

fn attr(e: &BytesStart<'_>, name: &str) -> R<Option<String>> {
    match e.try_get_attribute(name).map_err(err)? {
        Some(a) => Ok(Some(a.unescape_value().map_err(err)?.into_owned())),
        None => Ok(None),
    }
}

fn tag(e: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(e.name().as_ref()).into_owned()
}

pub fn parse(xml: &str) -> R<Schema> {
    let mut r = Reader::from_str(xml);
    r.config_mut().trim_text(true);
    let mut schema = Schema { build_id: None, protocols: Vec::new() };
    loop {
        match r.read_event().map_err(err)? {
            Event::Start(e) if e.name().as_ref() == b"Protocols" => {
                schema.build_id = attr(&e, "BuildId")?.and_then(|v| v.parse().ok());
            }
            Event::Start(e) if e.name().as_ref() == b"Protocol" => {
                let name = attr(&e, "Name")?.unwrap_or_default();
                schema.protocols.push(Protocol { name, groups: parse_groups(&mut r)? });
            }
            Event::Eof => return Ok(schema),
            _ => {}
        }
    }
}

fn parse_groups(r: &mut Reader<&[u8]>) -> R<Vec<MessageGroup>> {
    let mut groups = Vec::new();
    loop {
        match r.read_event().map_err(err)? {
            Event::Start(e) if e.name().as_ref() == b"Messages" => {
                let mut g = MessageGroup {
                    name: attr(&e, "Name")?.unwrap_or_default(),
                    client: Vec::new(),
                    server: Vec::new(),
                };
                loop {
                    match r.read_event().map_err(err)? {
                        Event::Start(e) if e.name().as_ref() == b"Client" => {
                            g.client = parse_messages(r, b"Client")?
                        }
                        Event::Start(e) if e.name().as_ref() == b"Server" => {
                            g.server = parse_messages(r, b"Server")?
                        }
                        Event::End(e) if e.name().as_ref() == b"Messages" => break,
                        Event::Eof => return Err(err("unexpected EOF in <Messages>")),
                        _ => {}
                    }
                }
                groups.push(g);
            }
            Event::End(e) if e.name().as_ref() == b"Protocol" => return Ok(groups),
            Event::Eof => return Err(err("unexpected EOF in <Protocol>")),
            _ => {}
        }
    }
}

fn message_head(e: &BytesStart<'_>) -> R<Message> {
    let id = attr(e, "Id")?
        .ok_or_else(|| err("Message without Id"))?
        .parse()
        .map_err(err)?;
    Ok(Message {
        id,
        name: attr(e, "Name")?.unwrap_or_default(),
        addr: attr(e, "Addr")?,
        fields: Vec::new(),
    })
}

fn parse_messages(r: &mut Reader<&[u8]>, end: &[u8]) -> R<Vec<Message>> {
    let mut out = Vec::new();
    loop {
        match r.read_event().map_err(err)? {
            Event::Start(e) if e.name().as_ref() == b"Message" => {
                let mut m = message_head(&e)?;
                m.fields = parse_fields(r, b"Message")?;
                out.push(m);
            }
            Event::Empty(e) if e.name().as_ref() == b"Message" => out.push(message_head(&e)?),
            Event::End(e) if e.name().as_ref() == end => return Ok(out),
            Event::Eof => return Err(err("unexpected EOF in message list")),
            _ => {}
        }
    }
}

fn field_head(e: &BytesStart<'_>) -> R<Field> {
    Ok(Field {
        kind: tag(e),
        name: attr(e, "Name")?.unwrap_or_default(),
        size: attr(e, "Size")?.map(|s| s.parse()).transpose().map_err(err)?,
        type_name: attr(e, "TypeName")?,
        fields: Vec::new(),
    })
}

fn parse_fields(r: &mut Reader<&[u8]>, end: &[u8]) -> R<Vec<Field>> {
    let mut out = Vec::new();
    loop {
        match r.read_event().map_err(err)? {
            Event::Empty(e) => out.push(field_head(&e)?),
            Event::Start(e) => {
                let mut f = field_head(&e)?;
                let end_tag = e.name().as_ref().to_vec();
                f.fields = parse_fields(r, &end_tag)?;
                out.push(f);
            }
            Event::End(e) if e.name().as_ref() == end => return Ok(out),
            Event::Eof => return Err(err("unexpected EOF in field list")),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reference_208039_schema() {
        let xml = include_str!("../reference/protocols-208039.xml");
        let s = parse(xml).unwrap();
        assert_eq!(s.build_id, Some(208039));
        assert_eq!(s.protocols.len(), 2);
        assert_eq!(s.messages().count(), 1770);
        let groups: usize = s.protocols.iter().map(|p| p.groups.len()).sum();
        assert_eq!(groups, 58);
        // First message in the file: protocol 0, group 0, client id 376 with two Guids.
        let m = &s.protocols[0].groups[0].client[0];
        assert_eq!(m.id, 376);
        assert_eq!(m.fields.iter().map(|f| f.kind.as_str()).collect::<Vec<_>>(), ["Guid", "Guid"]);
    }

    #[test]
    fn nested_fields_and_attributes() {
        let xml = r#"<Protocols BuildId="1"><Protocol Name="9"><Messages Name="0">
            <Client><Message Id="5" Name="x" Addr="0x140001000">
              <String Name="0" Size="255"/>
              <ArrayVarSmall Name="1" Size="4"><Dword Name="0"/><Byte Name="1"/></ArrayVarSmall>
              <Optional Name="2" TypeName="Float3"/>
            </Message><Message Id="6" Name="y"/></Client>
            <Server/></Messages></Protocol></Protocols>"#;
        let s = parse(xml).unwrap();
        let g = &s.protocols[0].groups[0];
        assert_eq!(g.client.len(), 2);
        let m = &g.client[0];
        assert_eq!(m.addr.as_deref(), Some("0x140001000"));
        assert_eq!(m.fields[0].size, Some(255));
        assert_eq!(m.fields[1].fields.len(), 2);
        assert_eq!(m.fields[2].type_name.as_deref(), Some("Float3"));
        assert!(g.client[1].fields.is_empty());
    }

    #[test]
    fn rejects_truncated_xml() {
        assert!(parse(r#"<Protocols><Protocol Name="0"><Messages Name="0"><Client>"#).is_err());
    }
}
