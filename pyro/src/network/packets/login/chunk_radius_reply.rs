
use util::{Result};
use util::bytes::{BinaryWriter, MutableBuffer, VarInt};

use util::Serialize;
use crate::ConnectedPacket;

/// Sent in response to [`ChunkRadiusRequest`](crate::ChunkRadiusRequest), to notify the client of the allowed render distance.
#[derive(Debug, Clone)]
pub struct ChunkRadiusReply {
    /// Maximum render distance that the server allows (in chunks).
    pub allowed_radius: i32,
}

impl ConnectedPacket for ChunkRadiusReply {
    const ID: u32 = 0x46;

    fn serialized_size(&self) -> usize {
        self.allowed_radius.var_len()
    }
}

impl Serialize for ChunkRadiusReply {
    fn serialize(&self, buffer: &mut MutableBuffer) -> Result<()> {
        buffer.write_var_i32(self.allowed_radius);
        Ok(())
    }
}