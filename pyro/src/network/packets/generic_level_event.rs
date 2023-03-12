use bytes::{BytesMut, Bytes};
use util::{Deserialize, ReadExtensions, Result};
use crate::network::packets::ConnectedPacket;

#[derive(Debug, Clone)]
pub struct GenericLevelEvent {
    pub event_id: i32,
    pub data: nbt::Tag
}

impl ConnectedPacket for GenericLevelEvent {
    const ID: u32 = 0x7c;
}

impl Deserialize for GenericLevelEvent {
    fn deserialize(mut buffer: Bytes) -> Result<Self> {
        let event_id = buffer.get_var_i32()?;
        let data = nbt::deserialize_le(&mut buffer)?;

        Ok(Self {
            event_id,
            data
        })
    }
}