use tokio::net::UdpSocket;
use std::time::Duration;

pub struct DiscoveryPacket {
    pub id: String,
    pub addr: String,
}

pub struct LanDiscovery;

impl LanDiscovery {
    pub async fn announce(id: &str, addr: &str) {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        socket.set_broadcast(true).unwrap();

        let msg = format!("DEFCOMM:{}:{}", id, addr);

        loop {
            let _ = socket.send_to(msg.as_bytes(), "255.255.255.255:7000").await;

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    pub async fn listen<F>(mut on_peer: F) where F: FnMut(DiscoveryPacket) + Send + 'static {
        let socket = UdpSocket::bind("0.0.0.0:7000").await.unwrap();
        let mut buf = vec![0u8; 1024];

        loop {
            let (len, _) = socket.recv_from(&mut buf).await.unwrap();
            let msg = String::from_utf8_lossy(&buf[..len]);

            if let Some(data) = msg.strip_prefix("DEFCOMM:") {
                let parts: Vec<&str> = data.split(':').collect();

                if parts.len() == 2 {
                    on_peer(DiscoveryPacket {
                        id: parts[0].to_string(),
                        addr: parts[1].to_string(),
                    });
                }
            }
        }
    }
}
