use serde::{ Deserialize, Serialize };
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MockV1 {
    id: Uuid,
    timestamp: i64,
    sequence_number: u64,
    correlation_id: Uuid
}

impl MockV1 {
    pub fn new(id: Uuid, timestamp: i64, sequence_number: u64, correlation_id: Uuid) -> Self {
        Self {
            id,
            timestamp,
            sequence_number,
            correlation_id
        }
    }

    // Getters
    pub fn get_type(&self) -> u64 {
        1
    }

    pub fn get_version(&self) -> u64 {
        1
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn get_correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
