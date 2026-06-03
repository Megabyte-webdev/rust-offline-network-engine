use tokio::io::AsyncReadExt;

use crate::core::engine::Engine;
use crate::core::message::Message;

use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn start_server(port: u16, engine: Arc<Mutex<Engine>>) {
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();

    println!("🌐 Listening on {}", port);

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        let engine_clone = engine.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];

            if let Ok(n) = socket.read(&mut buf).await {
                if n == 0 {
                    return;
                }

                if let Ok(msg) = serde_json::from_slice::<Message>(&buf[..n]) {
                    println!("📩 RECEIVED => {}", msg.payload);
                    if msg.payload == "ACK" {
                        return;
                    }

                    let mut eng = engine_clone.lock().await;
                    eng.learn_route(msg.from.clone(), msg.from.clone(), 1).await;

                    // THIS IS THE KEY CHANGE
                    eng.inject_message(msg).await;
                }
            }
        });
    }
}
