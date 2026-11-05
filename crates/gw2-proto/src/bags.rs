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
/// Wire order is character, **bag slot, then item instance** (handler reads
/// +0x6 as the slot and +0xa as the instance; instance 0 clears the slot).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BagSlotFrame209026 {
    pub character_index: u32,
    pub bag_slot: u32,
    pub item_instance: u32,
}

impl BagSlotFrame209026 {
    pub const MESSAGE_ID: u16 = 300;

    pub fn encode_frame(&self, output: &mut BytesMut) {
        Self::MESSAGE_ID.encode(output);
        self.character_index.encode(output);
        self.bag_slot.encode(output);
        self.item_instance.encode(output);
    }
}

/// Server 305 `{character, slot count}`: handler `0x141251410` calls
/// `0x14128d680`, which grows the inventory's `m_inventorySlots` array
/// (+0xc0, count +0xd4) to `slot_count` and never shrinks it. Equipping a bag
/// with 300 does not size this array; without 305, opening the inventory with
/// a bag asserts ChCliInventory.cpp:653 (sandbox runs 146/147/149/150).
pub struct InventorySlotCountFrame209026 {
    pub character_index: u32,
    pub slot_count: u32,
}

impl InventorySlotCountFrame209026 {
    pub const MESSAGE_ID: u16 = 305;
    /// CHAR_INVENTORY_SLOTS_MAX (assert at `0x1411eca90`).
    pub const MAX_SLOTS: u32 = 0x200;

    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if self.slot_count > Self::MAX_SLOTS {
            return Err(Error::new(ErrorKind::InvalidInput, "inventory slot count exceeds 512"));
        }
        Self::MESSAGE_ID.encode(output);
        self.character_index.encode(output);
        self.slot_count.encode(output);
        Ok(())
    }
}

/// Server 276 `{character, item instance, Byte reason, slot}`: handler
/// `0x1412500e0` calls `0x1411eca90`, which asserts `slot < 0x200`, requires
/// `slot < m_inventorySlots.Count()` (sized by 305) and an empty slot, then
/// stores the item there with location 3 (inventory). Reasons 0, 7 and 8 also
/// call item vtable +0x1c8(1, 1) first (meaning unresolved, inferred to be the
/// new-item mark).
pub struct InventoryItemFrame209026 {
    pub character_index: u32,
    pub item_instance: u32,
    pub reason: u8,
    pub slot: u32,
}

impl InventoryItemFrame209026 {
    pub const MESSAGE_ID: u16 = 276;

    pub fn encode_frame(&self, output: &mut BytesMut) -> Result<()> {
        if self.slot >= InventorySlotCountFrame209026::MAX_SLOTS {
            return Err(Error::new(ErrorKind::InvalidInput, "inventory slot exceeds 511"));
        }
        Self::MESSAGE_ID.encode(output);
        self.character_index.encode(output);
        self.item_instance.encode(output);
        self.reason.encode(output);
        self.slot.encode(output);
        Ok(())
    }
}

/// Server 968 `{Dword instance, Dword item key, Byte flags, Dword count,
/// Optional Guid}`: the creation message for stackable items (consumables,
/// containers, crafting materials, gizmos, mini pets, trophies, upgrade
/// components). Its initializer `0x1413cb930` asserts `count <= 250` and stores
/// the stack count. The Guid is absent.
pub fn encode_stack_item_creation(instance: u32, item_key: u32, count: u32, output: &mut BytesMut) -> Result<()> {
    if !(1..=250).contains(&count) {
        return Err(Error::new(ErrorKind::InvalidInput, "stack count must be 1..=250"));
    }
    968u16.encode(output);
    instance.encode(output);
    item_key.encode(output);
    0u8.encode(output);
    count.encode(output);
    0u8.encode(output);
    Ok(())
}

/// Minimal item creation `{instance, item key, flags, Optional Guid absent}`
/// for messages 959/962 (970 adds one Byte before the Guid). Each message runs
/// the shared factory `0x1413c7720(instance, key)` then its own initializer
/// (959: item vtable +0x290, 962: +0x2a8, 970: `0x1413c7580`), so the message
/// must match the item's class. Research codec; field roles beyond the prefix
/// are unnamed.
pub fn encode_minimal_item_creation(message_id: u16, instance: u32, item_key: u32, output: &mut BytesMut) -> Result<()> {
    if ![959u16, 962, 970].contains(&message_id) {
        return Err(Error::new(ErrorKind::InvalidInput, "minimal item creation supports 959, 962 and 970"));
    }
    message_id.encode(output);
    instance.encode(output);
    item_key.encode(output);
    0u8.encode(output);
    if message_id == 970 {
        0u8.encode(output);
    }
    0u8.encode(output); // Optional Guid: absent
    Ok(())
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
        BagSlotFrame209026 { character_index: 1, bag_slot: 0, item_instance: 0x7001 }.encode_frame(&mut out);
        InventorySlotCountFrame209026 { character_index: 1, slot_count: 20 }.encode_frame(&mut out).unwrap();
        assert!(InventorySlotCountFrame209026 { character_index: 1, slot_count: 513 }
            .encode_frame(&mut BytesMut::new())
            .is_err());
        InventoryItemFrame209026 { character_index: 1, item_instance: 0x7002, reason: 0, slot: 3 }
            .encode_frame(&mut out)
            .unwrap();
        assert!(InventoryItemFrame209026 { character_index: 1, item_instance: 1, reason: 0, slot: 512 }
            .encode_frame(&mut BytesMut::new())
            .is_err());
        let mut input = &out[..];
        assert_eq!(dynamic::decode_frame(&messages, &rules, &mut input).unwrap().0.id, 286);
        assert_eq!(dynamic::decode_frame(&messages, &rules, &mut input).unwrap().0.id, 300);
        assert_eq!(dynamic::decode_frame(&messages, &rules, &mut input).unwrap().0.id, 305);
        assert_eq!(dynamic::decode_frame(&messages, &rules, &mut input).unwrap().0.id, 276);
        assert!(input.is_empty());
        let mut stack = BytesMut::new();
        encode_stack_item_creation(0x7100, 19526, 1, &mut stack).unwrap();
        assert!(encode_stack_item_creation(1, 19526, 251, &mut BytesMut::new()).is_err());
        assert!(encode_stack_item_creation(1, 19526, 0, &mut BytesMut::new()).is_err());
        for (id, frame) in [959u16, 962, 970]
            .into_iter()
            .map(|id| {
                let mut frame = BytesMut::new();
                encode_minimal_item_creation(id, 0x7000, STARTER_BACKPACK_ITEM, &mut frame).unwrap();
                (id, frame)
            })
            .chain([(968, stack)])
        {
            let mut input = &frame[..];
            let all: Vec<_> = schema::parse(include_str!("../../../data/protocol/209026/server-tables.xml"))
                .unwrap()
                .protocols
                .into_iter()
                .filter(|p| p.name == "0")
                .flat_map(|p| p.groups)
                .filter(|g| g.name == "table-0x142260af0")
                .flat_map(|g| g.server)
                .collect();
            assert_eq!(dynamic::decode_frame(&all, &rules, &mut input).unwrap().0.id, id);
            assert!(input.is_empty());
        }
        let mut keep = BytesMut::from(&b"x"[..]);
        assert!(ContainerDialogFrame209026 { container_item_key: 1, byte_1: 0, instances: vec![0; 11] }
            .encode_frame(&mut keep)
            .is_err());
        assert_eq!(&keep[..], b"x");
    }
}
