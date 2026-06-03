# Offline Network Engine - Deployment Guide

## Table of Contents
1. [System Requirements](#system-requirements)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Running the Engine](#running-the-engine)
5. [Monitoring](#monitoring)
6. [Troubleshooting](#troubleshooting)
7. [Security Checklist](#security-checklist)

## System Requirements

### Minimum Specifications
- **OS:** Linux (Ubuntu 20.04+), macOS (11.0+), Windows 10+
- **CPU:** 2 cores @ 2.0 GHz minimum
- **RAM:** 512 MB minimum (2GB recommended)
- **Disk:** 100MB free space for application + logs
- **Network:** 1 Mbps connection minimum

### Recommended Specifications
- **CPU:** 4+ cores @ 2.5 GHz
- **RAM:** 4GB+
- **Disk:** 10GB SSD for database
- **Network:** 10 Mbps+
- **Firewall:** UDP ports 5353-5356, TCP ports 8000-8100 (configurable)

### Rust Version
- **Minimum:** Rust 1.70.0
- **Recommended:** Latest stable (1.75+)

## Installation

### From Source
```bash
git clone https://github.com/defcomm/offline-network-engine.git
cd offline-network-engine
cargo build --release
```

### Binary Release
```bash
# Download from releases page
chmod +x defcomm-network-engine
./defcomm-network-engine --version
```

### As a Library (Cargo.toml)
```toml
[dependencies]
defcomm-network-engine = "0.1.0"
```

## Configuration

### Environment Variables
```bash
# Core settings
NETWORK_INSTANCE_ID=<uuid>              # Unique node ID (auto-generated if not set)
NETWORK_PORT=8000                       # TCP listening port
NETWORK_DISCOVERY_PORT=5353             # mDNS discovery port
NETWORK_LOG_LEVEL=info                  # debug, info, warn, error

# Security
NETWORK_ENCRYPTION_KEY=<32-byte-hex>   # 256-bit AES key (optional, auto-generated)
NETWORK_AUTH_TOKEN=<token>             # Authentication token
NETWORK_VERIFY_PEERS=true              # Enable peer verification

# Performance
NETWORK_MAX_PEERS=100                  # Maximum simultaneous connections
NETWORK_BUFFER_SIZE=16384              # Message buffer size
NETWORK_HEARTBEAT_INTERVAL=5000        # Heartbeat interval (ms)
NETWORK_RECONNECT_ATTEMPTS=5           # Reconnection retry count

# Storage
NETWORK_DATA_DIR=/var/lib/defcomm      # Data storage directory
NETWORK_DB_PATH=/var/lib/defcomm/db    # Database path

# Advanced
NETWORK_NAT_TRAVERSAL=true             # Enable NAT traversal
NETWORK_BLOOM_FILTER_SIZE=1000000      # Bloom filter size
NETWORK_TTL=64                         # Message TTL
```

### Configuration File (config.toml)
```toml
[network]
port = 8000
discovery_port = 5353
log_level = "info"
instance_id = "node-1"

[security]
encryption_enabled = true
verify_peers = true
token_expiry_seconds = 3600

[performance]
max_peers = 100
buffer_size = 16384
heartbeat_interval_ms = 5000

[storage]
data_dir = "/var/lib/defcomm"
db_path = "/var/lib/defcomm/db"

[advanced]
nat_traversal = true
bloom_filter_size = 1000000
ttl = 64
```

### Configuration Precedence
1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration file
4. Default values (lowest priority)

## Running the Engine

### Standalone Mode
```bash
# Start with defaults
./defcomm-network-engine

# Start with custom port
./defcomm-network-engine --port 9000

# Start with config file
./defcomm-network-engine --config config.toml

# Start with specific log level
./defcomm-network-engine --log-level debug
```

### Systemd Service (Linux)
```ini
# /etc/systemd/system/defcomm-network.service
[Unit]
Description=Offline Network Engine
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/defcomm-network-engine --config /etc/defcomm/config.toml
Restart=on-failure
RestartSec=10
User=defcomm
Group=defcomm

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable defcomm-network
sudo systemctl start defcomm-network
sudo systemctl status defcomm-network
```

### Docker Deployment
```dockerfile
FROM rust:latest

WORKDIR /app
COPY . .

RUN cargo build --release

EXPOSE 8000 5353

ENV NETWORK_LOG_LEVEL=info
ENV NETWORK_PORT=8000

CMD ["/app/target/release/defcomm-network-engine"]
```

Build and run:
```bash
docker build -t defcomm-network-engine .
docker run -d \
  -p 8000:8000 \
  -p 5353:5353 \
  -v /var/lib/defcomm:/var/lib/defcomm \
  --env NETWORK_LOG_LEVEL=info \
  --name defcomm-network \
  defcomm-network-engine
```

### Docker Compose (Multi-node Cluster)
```yaml
version: '3.8'

services:
  node1:
    build: .
    ports:
      - "8001:8000"
      - "5353:5353"
    environment:
      NETWORK_INSTANCE_ID: "node-1"
      NETWORK_PORT: 8000
      NETWORK_LOG_LEVEL: info
    volumes:
      - node1_data:/var/lib/defcomm

  node2:
    build: .
    ports:
      - "8002:8000"
      - "5353:5353"
    environment:
      NETWORK_INSTANCE_ID: "node-2"
      NETWORK_PORT: 8000
      NETWORK_LOG_LEVEL: info
    volumes:
      - node2_data:/var/lib/defcomm

volumes:
  node1_data:
  node2_data:
```

## Monitoring

### Health Checks
```bash
# Check node status
curl http://localhost:8000/health

# Get peer count
curl http://localhost:8000/peers

# Get metrics
curl http://localhost:8000/metrics
```

### Log Monitoring
```bash
# View logs in real-time
tail -f /var/log/defcomm/network.log

# Filter for errors
grep "ERROR" /var/log/defcomm/network.log

# Monitor throughput
grep "THROUGHPUT" /var/log/defcomm/network.log | tail -100
```

### Metrics to Monitor
- **Node Count:** Peers connected to this node
- **Message Throughput:** Messages/second
- **CPU Usage:** Should stay <15% under normal load
- **Memory Usage:** Baseline ~50MB, grows with peer count
- **Latency:** p50 <50ms, p99 <200ms
- **Packet Loss:** Should be <0.1%
- **Uptime:** Track for SLA compliance

### Prometheus Integration (Optional)
```toml
[monitoring]
prometheus_enabled = true
prometheus_port = 9090
```

Access metrics at: `http://localhost:9090/metrics`

## Security Checklist

Before production deployment:

- [ ] Generate unique encryption key
  ```bash
  openssl rand -hex 32 > /etc/defcomm/encryption.key
  ```

- [ ] Set restrictive file permissions
  ```bash
  chmod 600 /etc/defcomm/config.toml
  chmod 700 /var/lib/defcomm
  ```

- [ ] Configure firewall
  ```bash
  sudo ufw allow 8000/tcp
  sudo ufw allow 5353/udp
  ```

- [ ] Enable peer verification
  ```bash
  NETWORK_VERIFY_PEERS=true
  ```

- [ ] Set up authentication tokens
  ```bash
  # Generate token
  openssl rand -base64 32 > /etc/defcomm/auth.token
  ```

- [ ] Configure rate limiting
  ```toml
  [security]
  rate_limit_enabled = true
  rate_limit_requests_per_second = 1000
  ```

- [ ] Enable audit logging
  ```toml
  [logging]
  audit_enabled = true
  audit_file = "/var/log/defcomm/audit.log"
  ```

- [ ] Test encryption and authentication
  ```bash
  ./test-security.sh
  ```

## Troubleshooting

### Port Already in Use
```bash
# Check what's using the port
lsof -i :8000

# Use different port
./defcomm-network-engine --port 9000
```

### Discovery Not Working
```bash
# Check mDNS
avahi-resolve-address 224.0.0.251

# Verify firewall
sudo ufw allow 5353/udp
```

### High CPU Usage
- Reduce `max_peers` setting
- Check for CPU-bound operations in logs
- Consider horizontal scaling

### Memory Leaks
- Monitor with: `valgrind --leak-check=full`
- Check for message queue buildup
- Restart service if needed

### Network Issues
- Check latency: `ping <peer-ip>`
- Check routing: `traceroute <peer-ip>`
- Verify firewall rules

For more help, see TROUBLESHOOTING.md

## Next Steps

1. Review SECURITY.md for security best practices
2. Check PERFORMANCE.md for tuning recommendations
3. See EXAMPLES.md for integration patterns
4. Join community forums for support

---

*For updates and support: https://github.com/defcomm/offline-network-engine*
