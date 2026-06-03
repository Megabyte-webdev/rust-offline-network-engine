use std::collections::{ HashMap, HashSet };
use crate::{
    core::message::Message,
    transport::tcp_transport::TcpTransport,
    security::security::SecurityLayer,
};

pub struct RoutingEngine {
    pub peers: HashMap<String, String>,
    pub seen: HashSet<String>,
}

impl RoutingEngine {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            seen: HashSet::new(),
        }
    }

    pub fn add_peer(&mut self, id: String, addr: String) {
        self.peers.insert(id, addr);
    }

    pub fn route(&mut self, msg: &Message) -> Vec<String> {
        self.peers.values().cloned().collect()
    }

    pub async fn handle_message(
        &mut self,
        msg: Message,
        transport: &TcpTransport,
        security: &SecurityLayer
    ) {
        if self.seen.contains(&msg.id) {
            return;
        }

        self.seen.insert(msg.id.clone());

        let decrypted = security.decrypt(&serde_json::to_vec(&msg).unwrap());
        let msg: Message = serde_json::from_slice(&decrypted).unwrap();

        for addr in self.peers.values() {
            let data = serde_json::to_vec(&msg).unwrap();
            let encrypted = security.encrypt(&data);

            let _ = transport.send(addr, encrypted).await;
        }
    }
}
