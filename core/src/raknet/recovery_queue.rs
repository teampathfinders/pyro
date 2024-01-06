use dashmap::DashMap;
use proto::raknet::AckRecord;

use crate::raknet::FrameBatch;

/// Holds previously sent raknet to be able to recover them when packet loss occurs.
///
/// This data structures keeps track of all raknet that have been sent by the server.
/// When the client sends an ACK, the specified raknet are remove from the queue.
/// If a NAK is received, the specified raknet can be recovered from the queue.
#[derive(Default, Debug)]
pub struct Recovery {
    frames: DashMap<u32, FrameBatch>,
}

impl Recovery {
    /// Creates a new recovery queue.
    pub fn new() -> Self {
        Default::default()
    }

    /// Inserts a frame batch into the queue.
    ///
    /// The frame batch will stay in the queue until it is acknowledged.
    #[inline]
    pub fn insert(&self, batch: FrameBatch) {
        self.frames.insert(batch.sequence_number, batch);
    }

    /// Removes the specified raknet from the recovery queue.
    ///
    /// This method should be called when an ACK is received.
    pub fn confirm(&self, records: &[AckRecord]) {
        for record in records {
            match record {
                AckRecord::Single(id) => {
                    self.frames.remove(id);
                }
                AckRecord::Range(range) => {
                    for id in range.clone() {
                        self.frames.remove(&id);
                    }
                }
            }
        }
    }

    /// Recovers the specified raknet from the recovery queue.
    ///
    /// This method should be called when a NAK is received.
    pub fn recover(&self, records: &[AckRecord]) -> Vec<FrameBatch> {
        let mut recovered = Vec::new();
        for record in records {
            match record {
                AckRecord::Single(id) => {
                    if let Some(frame) = self.frames.remove(id) {
                        recovered.push(frame.1);
                    }
                }
                AckRecord::Range(range) => {
                    recovered.reserve(range.len());
                    for id in range.clone() {
                        if let Some(frame) = self.frames.remove(&id) {
                            recovered.push(frame.1);
                        }
                    }
                }
            }
        }

        recovered
    }
}
