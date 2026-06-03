use crate::core::message::Message;
use crate::core::peer_graph::PeerGraph;
use std::collections::HashSet;

pub struct Router {
    pub seen: HashSet<String>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    pub fn route(&mut self, mut msg: Message, graph: &PeerGraph) -> Vec<(String, Message)> {
        let mut forwarded = vec![];

        // 1. prevent duplicates
        if self.seen.contains(&msg.id) {
            return forwarded;
        }

        self.seen.insert(msg.id.clone());

        // 2. TTL check
        if msg.ttl == 0 {
            return forwarded;
        }

        msg.ttl -= 1;
        msg.visited.push(msg.from.clone());

        // 3. forward logic
        let neighbors = graph.neighbors(&msg.from);

        for n in neighbors {
            if !msg.visited.contains(&n) {
                let mut new_msg = msg.clone();
                new_msg.from = msg.from.clone();

                forwarded.push((n, new_msg));
            }
        }

        forwarded
    }
}
