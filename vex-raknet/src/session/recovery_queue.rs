use dashmap::DashMap;

use crate::frame::FrameBatch;
use crate::packets::AcknowledgementRecord;

#[derive(Debug)]
pub struct RecoveryQueue {
    frames: DashMap<u32, FrameBatch>,
}

impl RecoveryQueue {
    pub fn new() -> Self {
        Self {
            frames: DashMap::new(),
        }
    }

    pub fn insert(&self, batch: FrameBatch) {
        self.frames.insert(batch.get_batch_number(), batch);
    }

    pub fn confirm(&self, records: &Vec<AcknowledgementRecord>) {
        for record in records {
            match record {
                AcknowledgementRecord::Single(id) => {
                    self.frames.remove(id);
                }
                AcknowledgementRecord::Range(range) => {
                    for id in range.clone() {
                        self.frames.remove(&id);
                    }
                }
            }
        }
    }

    pub fn recover(&self, records: &Vec<AcknowledgementRecord>) -> Vec<FrameBatch> {
        let mut recovered = Vec::new();
        for record in records {
            match record {
                AcknowledgementRecord::Single(id) => {
                    if let Some(frame) = self.frames.get(id) {
                        recovered.push((*frame.value()).clone());
                    }
                }
                AcknowledgementRecord::Range(range) => {
                    recovered.reserve(range.len());
                    for id in range.clone() {
                        if let Some(frame) = self.frames.get(&id) {
                            recovered.push((*frame.value()).clone());
                        }
                    }
                }
            }
        }

        recovered
    }
}