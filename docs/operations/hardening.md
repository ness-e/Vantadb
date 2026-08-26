# VantaDB Server — Security Hardening Guide

> **Audience**: Operators deploying VantaDB Server in production or semi-trusted environments.
> **Scope**: VantaDB Server (`vantadb-server` crate, `vanta-cli server` binary) — local-first, embedded engine with optional HTTP API.
> **Last updated**: 2026-08-26 (aligns with `vantadb` v0.5.0)

---

## Executive Summary

VantaDB is a **local-first** embedded database: by default it runs as a library inside your process with no network exposure. The optional HTTP server (`vanta-cli server`) adds a network API for multi-client access, MCP integration, and the Vanta Studio web console.

**Threat model**: We assume the server runs on a host you control. We do **not** assume the network is trusted. The hardening guidance below follows a "secure by default" posture — the server refuses to start in insecure configurations unless explicitly overridden.

---

## Quick Comparison: VantaDB vs. Qdrant / Weaviate / Milvus / Marqo

| Capability | VantaDB | Qdrant | Weaviate | Milvus | Marqo |
|---|---|---|---|---|---|
| **Default auth** | ✅ Refuse-to-start on non-loopback without API key (FIND-07) | ❌ Open by default | ❌ Open by default | ❌ Open by default | ❌ Open by default |
| **Rate limiting** | ✅ Fail-closed (AUD-021), per-IP, burst control | ✅ Configurable | ✅ Configurable | ✅ Configurable | ✅ Configurable |
| **API key rotation** | ✅ Zero-downtime via `alt_api_key` (SRV-04, Qdrant v1.17 pattern) | ✅ `alt_api_key` v1.17+ | ❌ | ❌ | ❌ |
| **Namespace-scoped RBAC** | ✅ Per-namespace read/write (SRV-05, Qdrant v1.9 pattern) | ✅ Per-collection JWT RBAC v1.9+ | ✅ RBAC (Enterprise) | ✅ RBAC (Enterprise) | ❌ |
| **Audit logging** | ✅ JSONL rotation + tracing IDs (SRV-01, SRV-02, Qdrant v1.17+ pattern) | ✅ v1.17+ | ✅ Enterprise | ✅ Enterprise | ❌ |
| **TLS** | ✅ rustls (feature `tls`) | ✅ | ✅ | ✅ | ✅ |
| **Unprivileged Docker** | ✅ Multi-stage, `--target unprivileged`, read-only rootfs | ✅ `-unprivileged` image | ❌ | ❌ | ❌ |
| **Network bind guard** | ✅ Loopback-only unless API key set (FIND-07) | ⚠️ Manual config | ⚠️ Manual config | ⚠️ Manual config | ⚠️ Manual config |
| **Dependencies** | Minimal (Rust stdlib, no JVM/Go runtime) | C++/Rust | Go + Java modules | Go + C++ + etcd | Python + OpenSearch |

**Key differentiators (already implemented):**
- **Fail-closed rate limiting** (AUD-021): if governor config fails to build, server refuses to start rather than serving unthrottled.
- **Refuse-to-start guard** (FIND-07, `HTTP_API.md:591-618`): binding `0.0.0.0` without `VANTADB_API_KEY` is a hard error unless `--allow-insecure` is passed.
- **Audit log rotation** (SRV-01): size-based rotation with capped file count, no unbounded growth.
- **Request tracing** (SRV-02): `x-request-id` / `x-tracing-id` / `traceparent` captured in audit events and spans.
- **Multi-key rotation** (SRV-04): `VANTADB_API_KEY` + `VANTADB_ALT_API_KEY` accepted simultaneously for zero-downtime rollover.
- **Namespace RBAC** (SRV-05): `token_role_map` tokens can hold `NamespaceRead("ns")` / `NamespaceWrite("ns")` permissions.

---

## 1. Authentication & Authorization

### 1.1 Enable API Key (Required for Non-Loopback)

```bash
# Minimum viable production config
export VANTADB_API_KEY="sk-$(openssl rand -hex 32)"
export VANTADB_HOST="0.0.0.0"
export VANTADB_PORT=8080
vanta-cli server --http --db ./data
```

**What happens without it:**
- Binding `127.0.0.1` → dev mode allowed (warns in logs)
- Binding `0.0.0.0` / non-loopback → **hard error**, server exits (FIND-07)

**Override (dev only):**
```bash
vanta-cli server --http --host 0.0.0.0 --allow-insecure  # LOGS PROMINENT WARNING
```

### 1.2 Enforce Auth at Startup

```bash
export VANTADB_REQUIRE_AUTH=true
# Server refuses to start if VANTADB_API_KEY is unset, regardless of host.
```

### 1.3 Zero-Downtime Key Rotation (SRV-04)

Pattern mirrors [Qdrant `alt_api_key` v1.17](https://qdrant.tech/documentation/security/#rotate-an-admin-api-key):

```bash
# 1. Deploy new key as alt_api_key (rolling restart, one replica at a time)
export VANTADB_API_KEY="sk-old-key"
export VANTADB_ALT_API_KEY="sk-new-key"
# Both keys now accepted simultaneously

# 2. Switch clients to new key

# 3. Promote new key, drop alt (another rolling restart)
export VANTADB_API_KEY="sk-new-key"
unset VANTADB_ALT_API_KEY
```

### 1.4 Namespace-Scoped RBAC (SRV-05)

Configure per-namespace read/write in `token_role_map` (via config file or env):

```toml
# VantaConfig.rbac_config.token_role_map
[rbac_config.token_role_map]
"sk-reader-team-a" = "team_a_reader"
"sk-writer-team-a" = "team_a_writer"
"sk-admin" = "admin"

# Role definitions (programmatic, via Rbac::add_role)
# team_a_reader  -> NamespaceRead("team-a")
# team_a_writer  -> NamespaceRead("team-a") + NamespaceWrite("team-a")
# admin          -> Admin (all namespaces)
```

**Effect**: A token with `team_a_writer` can write to namespace `team-a` but is denied on `team-b`. Global `Read`/`Write` permissions still work for non-namespaced endpoints.

### 1.5 TLS (Feature-Gated)

```bash
# Requires: cargo build --features tls
export VANTADB_TLS_CERT=/path/to/cert.pem
export VANTADB_TLS_KEY=/path/to/key.pem
vanta-cli server --http --host 0.0.0.0 --port 443
```

Uses `rustls` (no OpenSSL dependency). TLS 1.2 + 1.3. Client cert validation optional.

---

## 2. Network Hardening

### 2.1 Bind Address

| Host | Auth Required | Use Case |
|---|---|---|
| `127.0.0.1` / `localhost` / `::1` | No (dev mode) | Local development, sidecar |
| `0.0.0.0` / specific LAN IP | **Yes** (FIND-07) | Production, multi-client |

### 2.2 Reverse Proxy (Recommended)

Terminate TLS at nginx/Traefik/Caddy; forward to VantaDB on loopback:

```nginx
# nginx snippet
server {
    listen 443 ssl;
    server_name vantadb.example.com;

    ssl_certificate /etc/ssl/certs/vantadb.pem;
    ssl_certificate_key /etc/ssl/private/vantadb.key;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

```bash
# VantaDB trusts proxy for client IP (rate limiting, audit)
export VANTADB_TRUSTED_PROXIES="127.0.0.1,::1,10.0.0.5"
export VANTADB_HOST=127.0.0.1  # Bind loopback only
export VANTADB_API_KEY="sk-..."
```

### 2.3 CORS (Web Console)

```bash
export VANTADB_ALLOWED_ORIGINS="https://studio.example.com,https://admin.example.com"
# Empty = no CORS headers sent (secure default)
```

---

## 3. Rate Limiting & DoS Protection

| Setting | Default | Description |
|---|---|---|
| `VANTADB_RATE_LIMIT_RPM` | 600 | Requests/minute per IP. `0` = disabled. |
| Burst (no auth) | `rpm` | Full burst for local web console (REST-01). |
| Burst (with auth) | `rpm/10` | Conservative (AUD-021 fail-closed). |

**Fail-closed guarantee** (AUD-021): If `GovernorConfig` fails to build (e.g., `rpm=0` edge case), the server **refuses to start** rather than serving without a limiter.

---

## 4. Audit Logging (SRV-01, SRV-02)

```bash
export VANTADB_AUDIT_LOG_PATH=/var/log/vantadb/audit.jsonl
export VANTADB_AUDIT_MAX_BYTES=10MB      # Rotate at size
export VANTADB_AUDIT_MAX_FILES=5         # Keep .1..N, delete older
```

**Rotation**: Atomic rename `audit.jsonl` → `audit.jsonl.1` → `.2` ... capped at `MAX_FILES`.

**Tracing correlation** (SRV-02): Every audit entry includes `request_id` from the first matching header:
1. `x-request-id`
2. `x-tracing-id`
3. `traceparent` (W3C)

Truncated to 256 chars. Enables end-to-end tracing from client → server → audit log.

---

## 5. Docker Hardening (SRV-07)

### 5.1 Standard Image (Multi-Stage)

```dockerfile
# Build: docker build -t vantadb-server -f vantadb-server/Dockerfile .
# Run:   docker run -d -p 8080:8080 -v vantadb-data:/var/lib/vantadb/data vantadb-server
```

- **Base**: `debian:bookworm-slim` (minimal attack surface)
- **User**: `vantadb` UID 1000 (non-root)
- **Binary**: Statically linked where possible, stripped
- **Healthcheck**: `/health` endpoint

### 5.2 Unprivileged Variant (Qdrant Pattern)

```bash
docker build --target unprivileged -t vantadb-server:unprivileged -f vantadb-server/Dockerfile .
docker run -d \
  --read-only \
  --cap-drop=ALL \
  --security-opt=no-new-privileges:true \
  -p 8080:8080 \
  -v vantadb-data:/var/lib/vantadb/data \
  vantadb-server:unprivileged
```

**Hardening applied:**
- Read-only root filesystem (`--read-only`)
- All capabilities dropped (`--cap-drop=ALL`)
- No privilege escalation (`no-new-privileges`)
- Only `/var/lib/vantadb/data` (volume) + `/tmp` + `/run` (tmpfs) writable

### 5.3 Release Binary Variant (No Local Build)

```bash
docker build --target release-binary \
  --build-arg VERSION=v0.5.0 \
  -t vantadb-server:v0.5.0 \
  -f vantadb-server/Dockerfile .
```

Downloads pre-built `vanta-cli` from GitHub Releases. Verifies checksum via HTTPS + TLS.

### 5.4 Docker Compose Profiles

```bash
# Development (local build)
docker compose -f vantadb-server/docker-compose.yml up -d

# Unprivileged (hardened)
docker compose -f vantadb-server/docker-compose.yml --profile unprivileged up -d

# Production (release binary, requires VERSION)
VERSION=v0.5.0 docker compose -f vantadb-server/docker-compose.yml -f vantadb-server/docker-compose.prod.yml up -d
```

---

## 6. Configuration Checklist (Production)

| Item | Env Var | Required | Notes |
|---|---|---|---|
| API Key | `VANTADB_API_KEY` | ✅ Yes (non-loopback) | 32+ byte random |
| Alt Key (rotation) | `VANTADB_ALT_API_KEY` | Optional | SRV-04 |
| Require Auth | `VANTADB_REQUIRE_AUTH` | Recommended | `true` |
| Bind Host | `VANTADB_HOST` | ✅ Yes | `0.0.0.0` or LAN IP |
| TLS Cert/Key | `VANTADB_TLS_CERT`/`_KEY` | ✅ If direct exposure | Or terminate at proxy |
| Rate Limit | `VANTADB_RATE_LIMIT_RPM` | Recommended | `600` default |
| Trusted Proxies | `VANTADB_TRUSTED_PROXIES` | If behind proxy | Comma-separated IPs |
| CORS Origins | `VANTADB_ALLOWED_ORIGINS` | If web console | Comma-separated origins |
| Audit Path | `VANTADB_AUDIT_LOG_PATH` | Recommended | JSONL rotation |
| Audit Max Bytes | `VANTADB_AUDIT_MAX_BYTES` | Optional | `10MB` default |
| Audit Max Files | `VANTADB_AUDIT_MAX_FILES` | Optional | `5` default |
| Dashboard Dir | `VANTADB_DASHBOARD_DIR` | Optional | Serves `/dashboard` |
| Allow Insecure | `VANTADB_ALLOW_INSECURE` | **Never in prod** | Dev override only |

---

## 7. Operational Procedures

### 7.1 Key Rotation Runbook

1. Generate new key: `openssl rand -hex 32`
2. Set `VANTADB_ALT_API_KEY` on all replicas (rolling restart)
3. Verify both keys work: `curl -H "Authorization: Bearer sk-old" /health` && `curl -H "Authorization: Bearer sk-new" /health`
4. Update clients to new key
5. Promote: set `VANTADB_API_KEY=new`, unset `VANTADB_ALT_API_KEY` (rolling restart)
6. Verify old key rejected: `curl -H "Authorization: Bearer sk-old" /health` → 401

### 7.2 Audit Log Review

```bash
# Recent denied auth attempts
jq 'select(.op=="auth" and .outcome=="err")' /var/log/vantadb/audit.jsonl | tail -20

# By tracing ID (client correlation)
jq 'select(.request_id=="abc-123")' /var/log/vantadb/audit.jsonl

# By namespace
jq 'select(.namespace=="team-a")' /var/log/vantadb/audit.jsonl
```

### 7.3 Security Monitoring Alerts

| Signal | Query | Threshold |
|---|---|---|
| Auth failures | `rate(vantadb_auth_failures_total[5m])` | > 10/min |
| Rate limit hits | `rate(vantadb_rate_limited_total[5m])` | > 5/min |
| Circuit breaker open | `vantadb_circuit_breaker_open` | == 1 |
| Audit rotation lag | `vantadb_audit_file_size_bytes / vantadb_audit_max_bytes` | > 0.9 |

---

## 8. References

- **Qdrant Security Docs**: <https://qdrant.tech/documentation/security/> (API key types, `alt_api_key` rotation, JWT RBAC, audit logging, hardening)
- **Qdrant Hardening**: <https://qdrant.tech/documentation/security/#hardening> (unprivileged image, read-only rootfs, capabilities)
- **VantaDB HTTP API Spec**: `docs/api/HTTP_API.md` (FIND-07 guard, route table, auth flows)
- **VantaDB Config Reference**: `docs/operations/CONFIGURATION.md` (all `VANTADB_*` env vars)
- **Rustls TLS**: <https://github.com/rustls/rustls> (memory-safe TLS implementation)
- **OWASP Container Security**: <https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html>

---

## 9. Version History

| Version | Date | Changes |
|---|---|---|
| 0.5.0 | 2026-08-26 | SRV-01/02/04/05: audit rotation, tracing IDs, multi-key rotation, namespace RBAC, Docker hardening, this guide |
| 0.4.x | — | FIND-07 refuse-to-start, AUD-021 fail-closed rate limit, basic Bearer auth |

---

**Next Steps (Post-v0.5.0):**
- SRV-06: OIDC/JWT integration (delegated)
- SRV-09: mTLS for inter-node (distributed mode)
- SRV-10: Encryption at rest (feature `encryption`)