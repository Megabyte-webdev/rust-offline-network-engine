use std::{ collections::{ HashMap, HashSet }, sync::Arc, time::Instant };

use tokio::sync::Mutex;

use crate::discovery::lan_discovery::{ DiscoveryPacket, LanDiscovery };
use crate::security::SecurityLayer;

use crate::core::{
    peer_graph::PeerGraph,
    router::Router,
    message::Message,
    event_bus::EventBus,
    events::EngineEvent,
};

use crate::transport::tcp_transport::TcpTransport;

fn current_timestamp() -> u64 {
    use std::time::{ SystemTime, UNIX_EPOCH };

    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[derive(Clone, Debug)]
pub struct RouteEntry {
    pub next_hop: String,
    pub cost: u32,
    pub latency_ms: u32,
    pub success_rate: u8,
    pub success_count: u32,
    pub fail_count: u32,
    pub last_updated: u64,
}

impl RouteEntry {
    pub fn score(&self) -> f32 {
        let cost_weight = self.cost as f32;
        let latency_weight = (self.latency_ms as f32) * 1.5;
        let reliability_penalty = ((100 - self.success_rate) as f32) * 3.0;

        cost_weight + latency_weight + reliability_penalty
    }
}

pub struct Engine {
    pub graph: PeerGraph,
    pub router: Router,
    pub events: EventBus,
    pub transport: TcpTransport,
    pub peers: Arc<Mutex<HashMap<String, String>>>,
    pub seen_messages: Arc<Mutex<HashSet<String>>>,
    pub routing_table: Arc<Mutex<HashMap<String, Vec<RouteEntry>>>>,
    // RTT tracking
    pub pending_acks: Arc<Mutex<HashMap<String, Instant>>>,
    pub security: SecurityLayer,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            graph: PeerGraph::new(),
            router: Router::new(),
            events: EventBus::new(),
            transport: TcpTransport::new(),

            peers: Arc::new(Mutex::new(HashMap::new())),
            seen_messages: Arc::new(Mutex::new(HashSet::new())),
            routing_table: Arc::new(Mutex::new(HashMap::new())),

            pending_acks: Arc::new(Mutex::new(HashMap::new())),
            security: SecurityLayer::new("defcomm-secret"),
        }
    }

    pub fn connect(&mut self, a: &str, b: &str) {
        self.graph.add_peer(a, b);

        self.events.emit(EngineEvent::PeerConnected {
            a: a.to_string(),
            b: b.to_string(),
        });
    }

    // BASIC SEND
    pub async fn send(&mut self, msg: Message, addr: &str) {
        let forwarded = self.router.route(msg.clone(), &self.graph);

        self.events.emit(EngineEvent::MessageReceived {
            msg: msg.clone(),
        });

        for (_node, msg) in forwarded {
            let json = serde_json::to_vec(&msg).unwrap();
            let encrypted = self.security.encrypt(&json);
            let _ = self.transport.send(addr, encrypted).await;

            self.events.emit(EngineEvent::MessageForwarded {
                to: addr.to_string(),
                msg,
            });
        }
    }

    // DISCOVERY
    pub async fn start_discovery(&mut self, id: String, addr: String) {
        let peers = self.peers.clone();

        tokio::spawn(async move {
            LanDiscovery::listen(move |packet: DiscoveryPacket| {
                let peers = peers.clone();

                tokio::spawn(async move {
                    let mut map = peers.lock().await;
                    map.insert(packet.id.clone(), packet.addr.clone());

                    println!("🔍 Discovered peer => {} @ {}", packet.id, packet.addr);
                });
            }).await;
        });

        tokio::spawn(async move {
            LanDiscovery::announce(&id, &addr).await;
        });
    }

    // RTT + RELAY CORE
    pub async fn relay_message(&mut self, packet: Vec<u8>) {
        let decrypted = match self.security.decrypt(&packet) {
            Ok(v) => v,
            Err(e) => {
                println!("decrypt error {:?}", e);
                return;
            }
        };

        let msg: Message = match serde_json::from_slice(&decrypted) {
            Ok(m) => m,
            Err(e) => {
                println!("json error {:?}", e);
                return;
            }
        };

        println!("received {}", msg.payload);

        // 2. ACK HANDLING
        if msg.payload == "ACK" {
            if let Some(start) = self.pending_acks.lock().await.remove(&msg.id) {
                let rtt = start.elapsed().as_millis() as u32;
                println!("⏱ RTT => {} ms from {}", rtt, msg.from);

                // ... (Your existing RTT routing table update logic) ...
            }
            return;
        }

        // 3. DEDUP
        {
            let mut seen = self.seen_messages.lock().await;
            if seen.contains(&msg.id) {
                return;
            }
            seen.insert(msg.id.clone());
        }

        // 4. TTL
        if msg.ttl <= 0 {
            return;
        }
        let mut forwarded = msg.clone();
        forwarded.ttl -= 1;

        // 5. SMART ROUTE (with safe encryption)
        let table = self.routing_table.lock().await;
        let mut routed = false;

        if let Some(routes) = table.get(&msg.from) {
            if
                let Some(best) = routes
                    .iter()
                    .min_by(|a, b| a.score().partial_cmp(&b.score()).unwrap())
            {
                if best.success_rate >= 30 {
                    if let Some(addr) = self.peers.lock().await.get(&best.next_hop) {
                        self.pending_acks.lock().await.insert(msg.id.clone(), Instant::now());

                        let json = serde_json::to_vec(&forwarded).unwrap();
                        if let Ok(encrypted) = self.security.encrypt(&json) {
                            // PASS THE RESULT DIRECTLY
                            self.transport.send(addr, Ok(encrypted)).await;
                            routed = true;
                        }
                    }
                }
            }
        }
        drop(table);

        // 6. FALLBACK FLOOD
        if !routed {
            let peers = self.peers.lock().await;
            for (_id, addr) in peers.iter() {
                let json = serde_json::to_vec(&msg).unwrap();
                if let Ok(encrypted) = self.security.encrypt(&json) {
                    // PASS THE RESULT DIRECTLY
                    self.transport.send(addr, Ok(encrypted)).await;
                    println!("📡 fallback => {}", addr);
                }
            }
        }
    }

    // ROUTE LEARNING
    pub async fn learn_route(&mut self, origin: String, from_peer: String, latency_ms: u32) {
        let mut table = self.routing_table.lock().await;

        let entry = RouteEntry {
            next_hop: from_peer,
            cost: 1,
            latency_ms,
            success_rate: 100,
            success_count: 0,
            fail_count: 0,
            last_updated: current_timestamp(),
        };

        table.entry(origin).or_insert_with(Vec::new).push(entry);
    }

    // FAILURE HANDLING
    pub async fn update_failure(&mut self, peer: String) {
        let mut table = self.routing_table.lock().await;

        for routes in table.values_mut() {
            for r in routes.iter_mut() {
                if r.next_hop == peer {
                    r.fail_count += 1;

                    let total = r.success_count + r.fail_count;
                    r.success_rate = if total == 0 {
                        0
                    } else {
                        (((r.success_count as f32) / (total as f32)) * 100.0) as u8
                    };

                    r.last_updated = current_timestamp();
                }
            }
        }
    }

    // ROUTE DECAY (SELF HEALING)
    pub async fn decay_routes(&mut self) {
        let mut table = self.routing_table.lock().await;
        let now = current_timestamp();

        for (_dest, routes) in table.iter_mut() {
            routes.retain(|r| now - r.last_updated < 30);
        }
    }

    pub async fn decay_link_quality(&mut self) {
        let mut table = self.routing_table.lock().await;

        for routes in table.values_mut() {
            for r in routes.iter_mut() {
                let age = current_timestamp() - r.last_updated;

                if age > 10 {
                    r.success_rate = r.success_rate.saturating_sub(1);
                }
            }
        }
    }

    // MESSAGE ENTRY POINT
    pub async fn inject_message(&mut self, msg: Message) {
        let peers = self.peers.lock().await;

        for (_, addr) in peers.iter() {
            let json = serde_json::to_vec(&msg).unwrap();

            let encrypted = self.security.encrypt(&json);

            self.transport.send(addr, encrypted).await;
        }
    }
}
