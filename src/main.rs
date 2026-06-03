mod core;
mod discovery;
mod security;
mod transport;

use core::engine::Engine;
use core::message::Message;
use transport::server::start_server;

use std::{env, sync::Arc};
use tokio::{
    sync::Mutex,
    time::{Duration, sleep},
};

#[tokio::main]
async fn main() {
    // CLI INPUT
    // cargo run NODE_A 7001

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: cargo run <NODE_ID> <PORT>");
        return;
    }

    let node_id = args[1].clone();
    let port: u16 = args[2].parse().unwrap();

    let addr = format!("127.0.0.1:{}", port);

    println!("🚀 Starting {} on {}", node_id, addr);

    // ENGINE

    let engine = Arc::new(Mutex::new(Engine::new()));

    // DISCOVERY

    {
        let mut eng = engine.lock().await;
        eng.start_discovery(node_id.clone(), addr.clone()).await;
    }

    // TCP SERVER

    let engine_clone = engine.clone();

    tokio::spawn(async move {
        start_server(port, engine_clone).await;
    });

    // EVENT LISTENER

    let mut rx = {
        let eng = engine.lock().await;
        eng.events.subscribe()
    };

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    println!("EVENT => {:?}", event);
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    // WAIT FOR NETWORK STABILIZATION

    sleep(Duration::from_secs(5)).await;

    // TEST MESSAGE (TEMPORARY HARDCODE)

    {
        let msg = Message::new(
            node_id.clone(),
            None,
            format!("Hello mesh from {}", node_id),
        );

        let mut eng = engine.lock().await;

        // inject into mesh (NOT send to target)
        eng.inject_message(msg).await;
    }

    // KEEP ALIVE

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
