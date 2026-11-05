//! Build-209026 container-dialog and bag-slot frames: server 286 and 300.
//! See `docs/re/209026-npc-talk-loop.md` ("Loot").
//!
//! **286 is a container dialog, not bag storage.** Its handler `0x141250710`
//! resolves the key as a type-35 item (`containerDef`) for the controlled
//! character, and the dialog asserts `containerDef->GetItemType() ==
//! ITEM_TYPE_CONTAINER` (DlgContainerUi.cpp:2413; sandbox run 145 with the
//! Starter Backpack 8932, a storage bag, tripped it). It is the "open a loot
//! container" UI. 300's handler `0x141251100` calls bag setter `0x141291a10`,
//! which asserts `bagSlot < GetBagSlotCount()`: storage bags must exist first,
//! by a route not yet established (likely an equipped bag item).

use std::io::{Error, ErrorKind, Result};

use bytes::BytesMut;

use crate::Encode;

/// Type-35 item key of the Starter Backpack, a storage bag (item type field 3,
/// 20 at definition +0x4c). Not valid for 286, which needs a container item.
pub const STARTER_BACKPACK_ITEM: u32 = 8932;

/// Server 286: open the container dialog for a container item. `byte_1` is
/// unnamed; `instances` (at most 10) are presumably the offered item instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerDialogFrame209026 {
    pub container_item_key: u32,
    pub byte_1: u8,
    pub instances: Vec<u32>,
}

impl ContainerDialogFrame209026 {
    pub const MESSAGE_ID: u16 = 286;

    /// Append a frame; output is unchanged if `instances` exceeds 10.
    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if self.instances.len() > 10 {
            return Err(Error::new(ErrorKind::InvalidInput, "at most 10 bag instances"));
        }
        Self::MESSAGE_ID.encode(output);
        self.container_item_key.encode(output);
        self.byte_1.encode(output);
        (self.instances.len() as u8).encode(output);
        self.instances.iter().for_each(|i| i.encode(output));
        Ok(())
    }
}

/// Server 300: place `item_instance` in bag slot `bag_slot` of the character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BagSlotFrame209026 {
    pub character_index: u32,
    pub item_instance: u32,
    pub bag_slot: u32,
}

impl BagSlotFrame209026 {
    pub const MESSAGE_ID: u16 = 300;

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.character_index.encode(output);
        self.item_instance.encode(output);
        self.bag_slot.encode(output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{self, CountPrefix, Rules};
    use crate::schema;

    #[test]
    fn frames_match_the_client_schema() {
        let server = schema::parse(include_str!("../../../data/protocol/209026/server-tables.xml")).unwrap();
        let messages: Vec<_> = server
            .protocols
            .into_iter()
            .filter(|p| p.name == "0")
            .flat_map(|p| p.groups)
            .filter(|g| g.name == "table-0x14216e940")
            .flat_map(|g| g.server)
            .collect();
        let rules = Rules { array_var_small: Some(CountPrefix::U8), ..Rules::default() };
        let mut out = BytesMut::new();
        ContainerDialogFrame209026 { container_item_key: STARTER_BACKPACK_ITEM, byte_1: 0, instances: vec![] }
            .encode_frame(&mut out)
            .unwrap();
        BagSlotFrame209026 { character_index: 1, item_instance: 0x7001, bag_slot: 0 }.encode_frame(&mut out);
        let mut input = &out[..];
        assert_eq!(dynamic::decode_frame(&messages, &rules, &mut input).unwrap().0.id, 286);
        assert_eq!(dynamic::decode_frame(&messages, &rules, &mut input).unwrap().0.id, 300);
        assert!(input.is_empty());
        let mut keep = BytesMut::from(&b"x"[..]);
        assert!(ContainerDialogFrame209026 { container_item_key: 1, byte_1: 0, instances: vec![0; 11] }
            .encode_frame(&mut keep)
            .is_err());
        assert_eq!(&keep[..], b"x");
    }
}
