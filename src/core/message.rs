use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: Option<String>,
    pub payload: String,
    pub ttl: u8,
    pub visited: Vec<String>,
}

impl Message {
    pub fn new(from: String, to: Option<String>, payload: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            payload,
            ttl: 8,
            visited: vec![],
        }
    }
}
