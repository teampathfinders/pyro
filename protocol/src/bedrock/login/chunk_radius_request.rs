use util::{BinaryRead, SharedBuffer};
use util::Deserialize;
use util::Result;

use crate::bedrock::ConnectedPacket;

/// Sent by the client to request the maximum render distance.
#[derive(Debug)]
pub struct ChunkRadiusRequest {
    /// Requested render distance (in chunks).
    pub radius: i32,
}

impl ConnectedPacket for ChunkRadiusRequest {
    const ID: u32 = 0x45;
}

impl Deserialize<'_> for ChunkRadiusRequest {
    fn deserialize(mut buffer: SharedBuffer) -> anyhow::Result<Self> {
        let radius = buffer.read_var_i32()?;
        let max_radius = buffer.read_u8()?;

        Ok(Self { radius })
    }
}