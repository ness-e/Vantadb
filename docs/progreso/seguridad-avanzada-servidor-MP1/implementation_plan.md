# Plan de Implementación: MP1 — Seguridad Avanzada del Servidor

## Contexto

**Pista Paralela MP1** del Plan Maestro. El servidor HTTP (`vantadb-server`) actualmente expone un endpoint sin ninguna capa de autenticación ni limitación de tasa. Esto es un riesgo real para cualquier despliegue en redes no locales (Fly.io, Docker en LAN).

El Plan dice "TLS forzado con `rustls` + rate limiting 100 req/min". Tras análisis de impacto:

### Decisión de Diseño: TLS como Feature Flag, no forzado

> [!IMPORTANT]
> TLS forzado en modo embebido local (`127.0.0.1`) no añade seguridad real (same-host) pero **sí añade fricción**: requiere generar/distribuir certificados en el SDK Python y en las integraciones LangChain/LlamaIndex. Por tanto, TLS se implementará como **feature flag `tls`** — activo por defecto en modo producción (Fly.io), desactivado por defecto en desarrollo.
>
> Esta decisión es arquitectónicamente correcta: SQLite no cifra el transporte local, y VantaDB sigue el mismo principio de embedded-first.

### Alcance de esta implementación

1. **Rate Limiting** — `tower-governor` con `100 req/min por IP` (configurable via env var `VANTADB_RATE_LIMIT_RPM`)
2. **Bearer Token Auth** — Middleware Axum que valida `Authorization: Bearer <token>` contra `VANTADB_API_KEY`. Si la var no está definida, el servidor arranca sin auth (modo desarrollo, retro-compatible).
3. **TLS opcional** — Feature flag `tls` que usa `axum-server` con `tokio-rustls`. Carga cert/key desde rutas configurables (`VANTADB_TLS_CERT`, `VANTADB_TLS_KEY`).

## Análisis de Impacto en Cascada (FMEA preventivo)

| Riesgo | Probabilidad | Mitigación |
|---|---|---|
| Rate limiter bloquea tests de integración | Alta | El rate limit solo aplica si `VANTADB_RATE_LIMIT_RPM > 0`. Tests usan `0` (desactivado) |
| Bearer token rompe `health_check` | Media | El endpoint `/health` está exento de auth (liveness probe de Docker/Fly.io) |
| TLS feature rompe compilación sin certs | Alta | Feature `tls` desactivado por defecto. Sin certs configurados → servidor arranca sin TLS y loguea warning |
| `tower-governor` incompatible con Axum 0.8 | Baja | `tower-governor` 0.4+ soporta `axum` 0.8 via `tower::Service`. Verificado en la API del crate |

## Proposed Changes

---

### Capa de Configuración

#### [MODIFY] [config.rs](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/src/config.rs)
- Agregar 3 campos nuevos a `VantaConfig`:
  - `api_key: Option<String>` — leído de `VANTADB_API_KEY`
  - `rate_limit_rpm: u32` — leído de `VANTADB_RATE_LIMIT_RPM`, default `100`
  - `tls_cert_path: Option<String>` / `tls_key_path: Option<String>` — leídos de `VANTADB_TLS_CERT`/`VANTADB_TLS_KEY`

---

### Servidor HTTP

#### [MODIFY] [Cargo.toml (vantadb-server)](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/Cargo.toml)
- Agregar a `[dependencies]`:
  - `tower-governor = "0.4"` — rate limiting por IP
  - `tower = { version = "0.4", features = ["util"] }` — mover de dev-deps a deps
  - `axum-server = { version = "0.7", optional = true }` — solo para feature `tls`
  - `tokio-rustls = { version = "0.26", optional = true }` — solo para feature `tls`
  - `rustls-pemfile = { version = "2", optional = true }` — parseo de PEM certs
- Agregar feature `tls = ["dep:axum-server", "dep:tokio-rustls", "dep:rustls-pemfile"]`

#### [NEW] [middleware.rs](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/src/middleware.rs)
Nuevo módulo con dos middlewares Axum:
- `auth_layer()` — extractor que valida Bearer token. Retorna `401 Unauthorized` con body JSON en caso de fallo.
- Constante de exención: `/health` no requiere auth.

#### [MODIFY] [server.rs](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/src/server.rs)
- Agregar `api_key: Option<String>` a `ServerState`
- En `app()`: aplicar `tower_governor::GovernorLayer` + `auth_layer` a rutas protegidas
- El router queda: `/health` sin auth, `/api/v2/query` con auth + rate limit

#### [MODIFY] [main.rs](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/src/main.rs)
- Pasar `api_key` desde `config` a `ServerState`
- Bajo feature `tls`: si los paths de cert/key están configurados, usar `axum_server::tls_rustls::RustlsConfig` en lugar de `TcpListener` plain
- Loguear en inicio qué modo de seguridad está activo (none / auth-only / auth+tls)

---

### Limpieza

#### [DELETE] `vantadb-server/vanta_certification.json`
- Archivo de certificación generado que no debe estar en el crate del servidor (existe también en la raíz del workspace).

---

### Tests

#### [MODIFY] `vantadb-server/tests/` — tests de integración existentes
- Asegurar que los tests existentes pasan con `VANTADB_RATE_LIMIT_RPM=0` y sin `VANTADB_API_KEY` (modo retro-compatible).

---

## Plan de Verificación

### Compilación
```powershell
cargo build -p vantadb-server
cargo build -p vantadb-server --features tls
```

### Tests
```powershell
cargo test -p vantadb-server
```

### Verificación Manual (a cargo del usuario)
1. Arrancar el servidor sin `VANTADB_API_KEY` → `/api/v2/query` accesible (retro-compatible).
2. Arrancar con `VANTADB_API_KEY=secret` → `/api/v2/query` retorna `401` sin header. Con `Authorization: Bearer secret` → respuesta normal.
3. Verificar `/health` siempre accesible sin auth.
