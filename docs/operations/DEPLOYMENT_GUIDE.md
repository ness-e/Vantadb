---
title: VantaDB Deployment Guide
type: operations
status: active
tags: [vantadb, operations, deployment]
last_reviewed: 2026-07-10
aliases: []
---

# VantaDB Deployment Guide

## Overview

VantaDB runs as a single binary with zero external runtime dependencies (no JVM, no Python, no system database). This makes it straightforward to deploy in production, embedded or as a standalone HTTP/MCP server.

This guide covers three deployment models:

- **systemd** — Linux production service (recommended for most on-prem)
- **Docker** — Containerized deployment (portable, CI-friendly)
- **Kubernetes** — Orchestrated deployment (scalable, cloud-native)

---

## 1. Getting the Binary

### Option A: GitHub Releases (recommended)

Download the latest release from [GitHub Releases](https://github.com/vantadb/vantadb/releases):

```bash
# Linux x86_64
curl -LO https://github.com/vantadb/vantadb/releases/latest/download/vantadb-linux-x86_64.tar.gz
tar xzf vantadb-linux-x86_64.tar.gz
sudo install vantadb /usr/local/bin/
```

### Option B: Build from source

```bash
# Clone
git clone https://github.com/vantadb/vantadb.git
cd vantadb

# Build release binary (includes default features: CLI, HTTP server, Fjall backend)
cargo build --release --features "cli,server"

# Install
sudo install target/release/vanta-cli /usr/local/bin/
```

**Features for production:**

| Feature | Required for | When to include |
|---------|-------------|-----------------|
| `default` | CLI, Arrow IPC, Fjall, telemetry | Always |
| `server` | HTTP/MCP server | Always for server mode |
| `tls` | HTTPS support | When not using a reverse proxy |
| `opentelemetry` | OpenTelemetry trace export | When using OTLP collectors |
| `rocksdb` | RocksDB backend | When migrating from RocksDB |
| `advanced-tokenizer` | Multilingual search | When deploying search workloads |
| `remote-inference` | Ollama-based embeddings | When using LLM embedding endpoints |

```bash
# Typical production build
cargo build --release --features "cli,server,tls,opentelemetry,advanced-tokenizer,remote-inference"
```

---

## 2. Systemd Service

### Service Unit

Create `/etc/systemd/system/vantadb.service`:

```ini
[Unit]
Description=VantaDB Server
Documentation=https://vantadb.dev/docs
After=network.target

[Service]
Type=simple
User=vantadb
Group=vantadb
WorkingDirectory=/var/lib/vantadb
ExecStart=/usr/local/bin/vanta-cli server --http --port 8080 -d /var/lib/vantadb/data
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

# Security hardening
NoNewPrivileges=true
ProtectSystem=full
ProtectHome=true
PrivateDevices=true
PrivateTmp=true
CapabilityBoundingSet=~CAP_SYS_PTRACE

# Memory limits (adjust per workload)
MemoryMax=4G
MemoryHigh=3G

# Environment
Environment=VANTADB_RATE_LIMIT_RPM=1000
Environment=VANTADB_LOG_FORMAT=json
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

### Setup

```bash
# Create user and data directory
sudo useradd --system --no-create-home --shell /usr/sbin/nologin vantadb
sudo mkdir -p /var/lib/vantadb/data
sudo chown -R vantadb:vantadb /var/lib/vantadb

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable vantadb
sudo systemctl start vantadb

# Verify
sudo systemctl status vantadb
journalctl -u vantadb -f
```

### Read-Only Mode (query-only replica)

Deploy a replica that serves queries without accepting writes:

```ini
ExecStart=/usr/local/bin/vanta-cli server --http --port 8081 -d /var/lib/vantadb/replica --read-only
```

---

## 3. Docker

### Dockerfile

```dockerfile
FROM alpine:3.21 AS build
RUN apk add --no-cache curl
ARG VANTADB_VERSION=0.6.9
RUN curl -L "https://github.com/vantadb/vantadb/releases/download/v${VANTADB_VERSION}/vantadb-linux-x86_64.tar.gz" \
  | tar xz -C /usr/local/bin/

FROM alpine:3.21
RUN apk add --no-cache ca-certificates tzdata
RUN addgroup -S vantadb && adduser -S vantadb -G vantadb
COPY --from=build /usr/local/bin/vanta-cli /usr/local/bin/
USER vantadb
EXPOSE 8080
VOLUME ["/data"]
ENTRYPOINT ["vanta-cli"]
CMD ["server", "--http", "--port", "8080", "-d", "/data"]
```

### Docker Compose

```yaml
# docker-compose.yml
services:
  vantadb:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - vantadb-data:/data
    environment:
      - VANTADB_RATE_LIMIT_RPM=1000
      - VANTADB_LOG_FORMAT=json
      - VANTA_BACKEND=fjall
      - RUST_LOG=info
      - VANTADB_API_KEY=${VANTADB_API_KEY}
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "vanta-cli", "status"]
      interval: 15s
      timeout: 5s
      retries: 3

volumes:
  vantadb-data:
```

### Quick Start

```bash
# Build and run
docker compose up -d

# Verify
curl http://localhost:8080/health

# With API key
curl -H "Authorization: Bearer $(cat .apikey)" http://localhost:8080/health
```

---

## 4. Kubernetes

### Namespace

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: vantadb
```

### StatefulSet

```yaml
# statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: vantadb
  namespace: vantadb
  labels:
    app: vantadb
spec:
  serviceName: vantadb
  replicas: 1
  selector:
    matchLabels:
      app: vantadb
  template:
    metadata:
      labels:
        app: vantadb
    spec:
      terminationGracePeriodSeconds: 30
      securityContext:
        fsGroup: 1000
        runAsUser: 1000
        runAsNonRoot: true
      containers:
        - name: vantadb
          image: vantadb/server:latest
          imagePullPolicy: IfNotPresent
          args:
            - server
            - --http
            - --port
            - "8080"
            - -d
            - /data
          ports:
            - containerPort: 8080
              name: http
          env:
            - name: VANTADB_RATE_LIMIT_RPM
              value: "1000"
            - name: VANTADB_LOG_FORMAT
              value: json
            - name: VANTA_BACKEND
              value: fjall
            - name: RUST_LOG
              value: info
            - name: VANTADB_API_KEY
              valueFrom:
                secretKeyRef:
                  name: vantadb-api-key
                  key: api-key
          livenessProbe:
            httpGet:
              path: /health
              port: http
            initialDelaySeconds: 5
            periodSeconds: 15
          readinessProbe:
            httpGet:
              path: /health
              port: http
            initialDelaySeconds: 3
            periodSeconds: 10
          resources:
            requests:
              memory: "512Mi"
              cpu: "500m"
            limits:
              memory: "4Gi"
              cpu: "2"
          volumeMounts:
            - name: data
              mountPath: /data
      volumes:
        - name: data
          persistentVolumeClaim:
            claimName: vantadb-data
```

### PersistentVolumeClaim

```yaml
# pvc.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: vantadb-data
  namespace: vantadb
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 50Gi
  storageClassName: standard
```

### Service

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: vantadb
  namespace: vantadb
  labels:
    app: vantadb
spec:
  selector:
    app: vantadb
  ports:
    - port: 8080
      targetPort: http
      name: http
  type: ClusterIP
```

### API Key Secret

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: vantadb-api-key
  namespace: vantadb
type: Opaque
stringData:
  api-key: "change-me-to-a-strong-random-token"
```

### Ingress (TLS)

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: vantadb
  namespace: vantadb
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - vantadb.example.com
      secretName: vantadb-tls
  rules:
    - host: vantadb.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: vantadb
                port:
                  name: http
```

### Deploy

```bash
kubectl apply -f namespace.yaml
kubectl apply -f secret.yaml
kubectl apply -f pvc.yaml
kubectl apply -f service.yaml
kubectl apply -f statefulset.yaml
kubectl apply -f ingress.yaml

# Verify
kubectl -n vantadb get pods
kubectl -n vantadb logs -l app=vantadb -f
```

---

## 5. Configuration Reference

All configuration is via environment variables. See [CONFIGURATION.md](CONFIGURATION.md) for the full reference.

### Essential Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `VANTADB_STORAGE_PATH` | `vantadb_data` | Data directory |
| `VANTADB_HOST` / `HOST` | `127.0.0.1` | HTTP bind address |
| `VANTADB_PORT` / `PORT` | `8080` | HTTP port |
| `VANTADB_API_KEY` | `""` | Bearer token authentication (empty = disabled) |
| `VANTADB_RATE_LIMIT_RPM` | `100` | Rate limit (requests/minute) |
| `VANTADB_LOG_FORMAT` | `compact` | `compact`, `json`, `full` |
| `VANTADB_TLS_CERT` | `""` | TLS certificate PEM path |
| `VANTADB_TLS_KEY` | `""` | TLS private key PEM path |
| `VANTA_BACKEND` | `fjall` | `fjall`, `rocksdb`, `memory` |
| `VANTADB_MAX_BLOCKING_THREADS` | `16` | Thread pool size |
| `RUST_LOG` | `info` | Log severity filter |

### Production Settings

```bash
# Production environment recommendation
export VANTADB_LOG_FORMAT=json
export VANTADB_RATE_LIMIT_RPM=1000
export VANTADB_MAX_BLOCKING_THREADS=$(nproc)
export RUST_LOG=info
```

---

## 6. Security

### Authentication

Enable API key authentication to protect the HTTP endpoint:

```bash
export VANTADB_API_KEY="$(openssl rand -hex 32)"
```

All requests must include the key:

```bash
curl -H "Authorization: Bearer $VANTADB_API_KEY" http://localhost:8080/health
```

### TLS

#### Option A: Reverse Proxy (recommended)

Place VantaDB behind a TLS-terminating reverse proxy (nginx, Caddy, Traefik, Cloudflare Tunnel). VantaDB binds to `127.0.0.1` only.

```nginx
# /etc/nginx/sites-available/vantadb
server {
    listen 443 ssl;
    server_name vantadb.example.com;

    ssl_certificate /etc/letsencrypt/live/vantadb.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/vantadb.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

#### Option B: Built-in TLS

Build with `--features tls` and configure:

```bash
export VANTADB_TLS_CERT=/etc/vantadb/cert.pem
export VANTADB_TLS_KEY=/etc/vantadb/key.pem
```

### Network

- Bind to `127.0.0.1` (not `0.0.0.0`) if fronted by a reverse proxy
- For direct access, place VantaDB in a private subnet or use a firewall
- API key is the minimum recommended auth for any exposure beyond localhost

---

## 7. Backup & Restore

See [BACKUP_POLICY.md](BACKUP_POLICY.md) for the full backup operational policy.

```bash
# Online backup (Fjall backend)
vanta-cli backup --out /backups/vantadb-$(date +%F)

# Restore
vanta-cli restore --from /backups/vantadb-2026-07-10

# Restore with index rebuild
vanta-cli restore --from /backups/vantadb-2026-07-10 --rebuild
```

---

## 8. Monitoring

### Health Check

```bash
# Returns 200 OK if the server is operational
curl http://localhost:8080/health
```

### Metrics (Prometheus)

```bash
# Prometheus metrics endpoint (requires --features prometheus at build time)
curl http://localhost:8080/metrics
```

### Logging

In production, use JSON log format for structured log aggregation:

```bash
export VANTADB_LOG_FORMAT=json
```

### Grafana

A pre-built Grafana dashboard is available at `docs/operations/grafana-dashboard.json`.

### OpenTelemetry

Build with `--features opentelemetry` and configure:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
export OTEL_SERVICE_NAME=vantadb-production
```

---

## 9. Performance Tuning

### OS Tuning

```bash
# Increase file descriptor limit
echo "fs.file-max = 100000" >> /etc/sysctl.conf

# Disable swap for consistent latency (if using mmap)
echo "vm.swappiness = 10" >> /etc/sysctl.conf

# Apply
sysctl -p
```

### Thread Pool

Match the thread pool to available CPU cores:

```bash
export VANTADB_MAX_BLOCKING_THREADS=$(nproc)
```

### Memory

Limit VantaDB memory via systemd `MemoryMax=` or container resource limits (`--memory`). The engine uses memory-mapped files for the vector index, so the OS page cache handles hot data. See [MEMORY_TELEMETRY.md](MEMORY_TELEMETRY.md) for details.

---

## 10. Troubleshooting

| Problem | Likely Cause | Solution |
|---------|-------------|----------|
| `Address already in use` | Port conflict | Change `--port` or check existing process |
| `Permission denied` on data dir | Wrong user/group | `chown -R vantadb:vantadb /var/lib/vantadb` |
| High memory usage | OS page cache (mmap) | Set container/process memory limits |
| Slow queries at startup | Indexes being rebuilt | Pre-warm with `vanta-cli rebuild-index` before serving traffic |
| `file lock` errors | Multiple processes writing to same data dir | Use separate data directories per instance |
| RocksDB build fails | Missing C++ compiler on target | Use `--features default,server` (excludes rocksdb) or install gcc/clang |
