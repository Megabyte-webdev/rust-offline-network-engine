use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

pub struct TcpTransport;

impl TcpTransport {
    pub fn new() -> Self {
        Self
    }

    pub async fn send(&self, addr: &str, data: Result<Vec<u8>, aes_gcm::Error>) {
        // Only attempt to send if encryption succeeded
        if let Ok(data) = data {
            if let Ok(mut stream) = TcpStream::connect(addr).await {
                let _ = stream.write_all(&data).await;
            }
        } else {
            eprintln!("Failed to encrypt data, skipping send.");
        }
    }
}
