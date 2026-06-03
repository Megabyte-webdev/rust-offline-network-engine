use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

pub struct TcpTransport;

impl TcpTransport {
    pub fn new() -> Self {
        Self
    }

    pub async fn send(&self, addr: &str, data: Vec<u8>) {
        if let Ok(mut stream) = TcpStream::connect(addr).await {
            let _ = stream.write_all(&data).await;
        } else {
            eprintln!("Failed to connect to {addr}");
        }
    }
}
