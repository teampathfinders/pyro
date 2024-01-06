use util::{MutableBuffer, SharedBuffer};
use util::{Deserialize};

use proto::raknet::{Ack, Nak};

use crate::network::RaknetUserLayer;

impl RaknetUserLayer {
    /// Processes an acknowledgement received from the client.
    ///
    /// This function unregisters the specified packet IDs from the recovery queue.
    pub fn handle_ack(&self, packet: SharedBuffer<'_>) -> anyhow::Result<()> {
        let ack = Ack::deserialize(packet)?;
        self.raknet.recovery_queue.confirm(&ack.records);

        Ok(())
    }

    /// Processes a negative acknowledgement received from the client.
    ///
    /// This function makes sure the packet is retrieved from the recovery queue and sent to the
    /// client again.
    pub async fn handle_nack(&self, packet: SharedBuffer<'_>) -> anyhow::Result<()> {
        let nack = Nak::deserialize(packet)?;
        let frame_batches = self.recovery.recover(&nack.records);
        tracing::info!("Recovered raknet: {:?}", nack.records);

        let mut serialized = MutableBuffer::new();
        for frame_batch in frame_batches {
            frame_batch.serialize(&mut serialized)?;

            self
                .socket
                .send_to(
                    serialized.as_ref(), self.address,
                )
                .await?;

            serialized.clear();
        }

        Ok(())
    }
}
