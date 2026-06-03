use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PeerGraph {
    pub connections: HashMap<String, Vec<String>>,
}

impl PeerGraph {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, a: &str, b: &str) {
        self.connections
            .entry(a.to_string())
            .or_default()
            .push(b.to_string());

        self.connections
            .entry(b.to_string())
            .or_default()
            .push(a.to_string());
    }

    pub fn neighbors(&self, id: &str) -> Vec<String> {
        self.connections.get(id).cloned().unwrap_or_default()
    }
}
