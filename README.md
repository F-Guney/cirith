# Cirith

Lightweight API Gateway written in Rust.

## Features

- **Reverse Proxy** — Pingora-based high-performance proxy
- **Dynamic Routing** — SQLite-backed, manage via Admin API
- **Authentication** — API key with SHA256 hashing
- **Rate Limiting** — IP-based sliding window
- **SSRF Protection** — Blocks private IPs, restricted hosts
- **Docker Ready** — Multi-service docker-compose

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

### Using Docker Compose

```bash
docker-compose up --build
```

Services:

- Admin API: http://localhost:3000
- Gateway: http://localhost:6191

## Configuration

Create `config.yaml`:

```yaml
server:
  admin_port: 3000
  gateway_port: 6191

database:
  url: "data/cirith.db"

auth:
  enabled: true
  keys:
    - name: "default"
      key_hash: "sha256-hash-here"

rate_limit:
  max_requests: 100
  window_seconds: 60
```

Generate API key hash:

```bash
echo -n "your-secret-key" | sha256sum
```

## API Endpoints

## Admin API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /health | Health check |
| GET | /admin/routes | List routes |
| POST | /admin/routes | Create route |
| DELETE | /admin/routes/:path | Delete route |
| GET | /admin/keys | List API keys |
| POST | /admin/keys | Create API key |
| DELETE | /admin/keys/:id | Delete API key |

## Gateway

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /health | Health check |
| * | /* | Proxy to upstream |

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
- [x] Rate limiting (Gateway)
- [x] Authentication (Gateway)
- [x] HTTPS upstream support
- [ ] Dashboard UI
- [ ] Redis rate limiting
- [ ] JWT authentication
- [ ] Hot-reload config

## License

TBD

---

Built with 🦀 by F-Guney