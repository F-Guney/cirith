# Cirith

Lightweight API Gateway written in Rust.

## Features

- **Reverse Proxy** — Route requests to multiple upstreams (Pingora-based)
- **Rate Limiting** — IP-based sliding window algorithm
- **Authentication** — API key with SHA256 hashing
- **Admin API** — REST API for routes and keys management
- **Metrics** — Request counters endpoint
- **SQLite Storage** — Persistent routes and API keys

## Architecture

```
                    ┌─────────────────────┐
                    │      Clients        │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                ▼                ▼
       ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
       │   Gateway   │  │   Admin     │  │   Admin     │
       │   :6191     │  │   :3000     │  │   :3000     │
       │  (Pingora)  │  │   (Axum)    │  │   (Axum)    │
       └──────┬──────┘  └──────┬──────┘  └─────────────┘
              │                │
              │                ▼
              │         ┌─────────────┐
              │         │   SQLite    │
              │         └─────────────┘
              ▼
       ┌─────────────┐
       │  Upstreams  │
       └─────────────┘
```

## Quick Start

### From Source

```bash
# Clone
git clone https://github.com/F-Guney/cirith.git
cd cirith

# Build all
cargo build --workspace --release

# Run Gateway (port 6191)
cargo run -p cirith-gateway

# Run Admin API (port 3000) - separate terminal
cargo run -p cirith-admin
```

### Using Docker

> Docker setup is being updated for the new workspace structure.

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
    upstream: "http://httpbin.org"
  - path: "/api/v2"
    upstream: "http://jsonplaceholder.typicode.com"
```

Generate API key hash:

```bash
echo -n "your-secret-key" | sha256sum
```

## API Endpoints

### Gateway (port 6191)

Proxy requests based on route config:

```bash
# Route to httpbin.org
curl http://localhost:6191/api/get

# Route to jsonplaceholder
curl http://localhost:6191/api/v2/posts/1
```

### Admin API (port 3000)

#### Health Check

```bash
curl http://localhost:3000/health
```

#### Metrics

```bash
curl http://localhost:3000/metrics
```

#### Routes Management

```bash
# List routes
curl http://localhost:3000/admin/routes

# Add route
curl -X POST http://localhost:3000/admin/routes \
  -H "Content-Type: application/json" \
  -d '{"path": "/test", "upstream": "http://example.com"}'

# Delete route
curl -X DELETE http://localhost:3000/admin/routes/test
```

#### API Keys Management

```bash
# List API keys
curl http://localhost:3000/admin/keys

# Add API key
curl -X POST http://localhost:3000/admin/keys \
  -H "Content-Type: application/json" \
  -d '{"name": "new-app", "key": "secret-key-here"}'

# Delete API key
curl -X DELETE http://localhost:3000/admin/keys/new-app
```

## Project Structure

```
cirith/
├── Cargo.toml          # Workspace root
├── config.yaml         # Configuration
├── shared/             # Common code (config, storage, auth)
├── admin/              # Admin API (Axum)
└── gateway/            # Reverse Proxy (Pingora)
```

## Tech Stack

- **Rust** — Memory safe, high performance
- **Pingora** — High-performance proxy framework (by Cloudflare)
- **Axum** — Web framework for Admin API
- **SQLite** — Persistent storage
- **Tokio** — Async runtime

## Roadmap

- [x] Pingora-based gateway
- [x] Config-driven routing
- [ ] Rate limiting (Gateway)
- [ ] Authentication (Gateway)
- [ ] HTTPS upstream support
- [ ] Dashboard UI
- [ ] Redis rate limiting
- [ ] JWT authentication
- [ ] Hot-reload config

## License

TBD

---

Built with 🦀 by F-Guney