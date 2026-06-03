
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingMessage {
    pub id: String,
    pub destination: String,
    pub payload: Vec<u8>,
    pub retries: u8,
}
