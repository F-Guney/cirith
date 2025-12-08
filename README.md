# Cirith

Lightweight API Gateway written in Rust.

## Features

- **Reverse Proxy** — Route requests to multiple upstreams
- **Rate Limiting** — IP-based sliding window algorithm
- **Authentication** — API key with SHA256 hashing
- **Admin API** — REST API for routes and keys management
- **Metrics** — Request counters endpoint
- **SQLite Storage** — Persistent routes and API keys

## Quick Start

### Using Docker
```bash
docker run -d \
  --name cirith \
  -p 3000:3000 \
  -v $(pwd)/config.yaml:/app/config.yaml \
  -v $(pwd)/data:/app/data \
  cirith:latest
```

### From Source
```bash
# Clone
git clone https://github.com/F-Guney/cirith.git
cd cirith

# Build
cargo build --release

# Run
./target/release/cirith
```

## Configuration

Create `config.yaml`:
```yaml
server:
  port: 3000
  timeout_seconds: 30

database:
  url: "sqlite:data/cirith.db?mode=rwc"

rate_limit:
  max_requests: 100
  window_secs: 60

auth:
  enabled: true
  api_keys:
    - name: "my-app"
      key_hash: "YOUR_SHA256_HASH"

routes:
  - path: "/api"
    upstream: "https://httpbin.org"
```

Generate API key hash:
```bash
echo -n "your-secret-key" | sha256sum
```

## API Endpoints

### Proxy

All requests (except admin) are proxied based on route config.
```bash
curl -H "x-api-key: your-secret-key" http://localhost:3000/api/get
```

### Health Check
```bash
curl http://localhost:3000/health
```

### Metrics
```bash
curl http://localhost:3000/metrics
```

### Admin API
```bash
# List routes
curl http://localhost:3000/admin/routes

# Add route
curl -X POST http://localhost:3000/admin/routes \
  -H "Content-Type: application/json" \
  -d '{"path": "/test", "upstream": "https://example.com"}'

# Delete route
curl -X DELETE http://localhost:3000/admin/routes/test

# List API keys
curl http://localhost:3000/admin/keys

# Add API key
curl -X POST http://localhost:3000/admin/keys \
  -H "Content-Type: application/json" \
  -d '{"name": "new-app", "key": "secret-key-here"}'

# Delete API key
curl -X DELETE http://localhost:3000/admin/keys/new-app
```

## Architecture
```
Client Request
      │
      ▼
┌─────────────┐
│   Cirith    │
│  Gateway    │
├─────────────┤
│ Auth Check  │
│ Rate Limit  │
│ Route Match │
└──────┬──────┘
       │
       ▼
   Upstream
```

## Tech Stack

- **Rust** — Memory safe, high performance
- **Axum** — Web framework
- **SQLite** — Persistent storage
- **Tokio** — Async runtime

## Roadmap

- [ ] Pingora migration (high-performance proxy)
- [ ] Dashboard UI
- [ ] Redis rate limiting
- [ ] JWT authentication
- [ ] Hot-reload config

## License

TBD

---

Built with 🦀 by F-Guney