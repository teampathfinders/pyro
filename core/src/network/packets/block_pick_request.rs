use util::{Deserialize, Result, Vector};
use util::bytes::{BinaryRead, SharedBuffer};

use crate::network::ConnectedPacket;

/// Sent by the client when the user requests a block using the block pick key.
#[derive(Debug)]
pub struct BlockPickRequest {
    /// Position of the block to pick.
    pub position: Vector<i32, 3>,
    /// Whether to include the block's NBT tags.
    pub with_nbt: bool,
    /// Hot bar slot to put the item into.
    pub hotbar_slot: u8,
}

impl ConnectedPacket for BlockPickRequest {
    const ID: u32 = 0x22;
}

impl<'a> Deserialize<'a> for BlockPickRequest {
    fn deserialize<R>(reader: R) -> anyhow::Result<Self>
    where
        R: BinaryRead<'a> + 'a
    {
        let position = reader.read_veci()?;
        let with_nbt = reader.read_bool()?;
        let hotbar_slot = reader.read_u8()?;

        Ok(Self {
            position,
            with_nbt,
            hotbar_slot,
        })
    }
}