# ENT-04: Connection pooling + circuit breaker para server-mode

## Metadata
- **Plan file:** docs/plans/PROMPT-MAESTRO-FREEZE.md
- **Fuente:** docs/Backlog.md:173
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Tipo:** Rust (core, server feature)
- **Turns estimados:** 20-30
- **Creado:** 2026-08-02T12:00
- **last-synced:** 2026-08-02T12:00
- **Estado:** ✅ COMPLETADO

## Contexto (validado en sesión 2026-08-02)
- La métrica `OOM_TRIPS` existe en `src/metrics/core/registry.rs:42` pero **solo se referencia en tests** — el governor OOM no la incrementa en producción. No hay circuit breaker (state machine) ni ConnectionPool.
- El server HTTP (`src/cli_server.rs`) ya tiene: semáforo de concurrencia (`max_blocking_threads`), keep-alive nativo de hyper, rate limiter Governor, auth middleware, metrics middleware.
- `src/wal_shipping.rs:146` ya usa exponential backoff (patrón a reutilizar).
- Scope decidido por el usuario: **Circuit breaker (state machine closed→open→half-open) + ConnectionPool explícito**.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/cli_server.rs` (run, app, execute_query), `src/config.rs` (VantaConfig), `src/metrics/core/registry.rs` |
| Callees | `src/governor.rs`, `src/memory_governor.rs`, `tokio::sync::Semaphore`, `prometheus::IntCounter` |
| Implicaciones | Contratos públicos NO cambian (sin cambios en API HTTP, CLI o SDK). `VantaConfig` gana campos nuevos con defaults (no breaking). `OOM_TRIPS` deja de ser dead code y refleja trips reales. Tests existentes del server deben seguir pasando. |
| Riesgo | Medio — concurrencia y middleware HTTP; requiere unit tests + e2e para no romper el path de query. |

## Contrato
`cargo nextest run --profile audit --workspace --build-jobs 2` pasa, `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 errores, `cargo fmt --check` limpio, **y** los unit tests del circuit breaker verifican la transición closed→open→half-open→closed, **y** un test e2e verifica 503 + `Retry-After` cuando el breaker está abierto.

## Herramientas
- cargo-mcp (check, clippy, fmt, test), rust-analyzer-mcp (diagnostics), codegraph (blast radius)

## Investigation Notes
- Patrón circuit breaker (Fowler / resilience4j): estados Closed → Open → HalfOpen. Se abre tras N fallos consecutivos; en Open rechaza de inmediato (fast-fail); tras timeout pasa a HalfOpen y permite un probe; éxito → Closed, fallo → Open. Ver nota: https://martinfowler.com/bliki/CircuitBreaker.html (patrón estándar, sin dependencia externa).
- El breaker debe alimentarse de: (1) incremento de `OOM_TRIPS` en `memory_governor`, (2) respuestas 5xx observadas por middleware, (3) semáforo agotado (saturación). No añadir dependencias nuevas: semáforo + `AtomicUsize`/`AtomicU64` bastan.
- `Retry-After` header es el mecanismo HTTP estándar para backoff del cliente.

## Steps

### Step 1: Agregar configuración
- **Archivos:** `src/config.rs`, `docs/operations/CONFIGURATION.md`
- **Acción:** Agregar a `VantaConfig` (con defaults): `circuit_breaker_failure_threshold: u32` (default 5), `circuit_breaker_open_timeout_secs: u64` (default 30), `max_connections: usize` (default = max_blocking_threads * 2), `pool_acquire_timeout_ms: u64` (default 5000). Documentar en CONFIGURATION.md (tabla de env vars).
- **Verify:** `cargo check -p vantadb --features server`

### Step 2: Crear módulo circuit breaker
- **Archivos:** `src/circuit_breaker.rs` (nuevo), `src/lib.rs` (mod decl)
- **Acción:** `CircuitBreaker` con estado atómico (`AtomicU8` 0=Closed 1=Open 2=HalfOpen), contador de fallos `AtomicU32`, `last_open: AtomicU64` (epoch secs). API: `allow_request() -> bool`, `record_success()`, `record_failure()`. Reglas: Closed→Open cuando fallos >= threshold; Open rechaza hasta timeout; HalfOpen permite 1 probe. Unit tests inline para las 3 transiciones.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test circuit_breaker` (o test inline)

### Step 3: Crear módulo connection pool
- **Archivos:** `src/connection_pool.rs` (nuevo), `src/lib.rs`
- **Acción:** `ConnectionPool` = semáforo con límite configurable + contador de conexiones activas (`AtomicUsize`). API: `acquire() -> Result<PoolGuard, PoolError>` con timeout de adquisición; `PoolGuard` libera al dropear (RAII). `pool_saturated()` expone saturación para alimentar el breaker. Unit tests: límite respetado, timeout, RAII release.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test connection_pool`

### Step 4: Integrar breaker + pool en ServerState
- **Archivos:** `src/cli_server.rs` (ServerState, app, run), `src/config.rs`
- **Acción:** Agregar `circuit_breaker: Arc<CircuitBreaker>` y `pool: Arc<ConnectionPool>` a `ServerState`. Crear middleware `circuit_breaker_middleware`: si `!allow_request()` → 503 con header `Retry-After: <open_timeout>`; tras la respuesta, `record_success()` si 2xx/4xx, `record_failure()` si >=500. Montar en `app()` (el más externo, antes de auth).
- **Verify:** `cargo check -p vantadb --features server`

### Step 5: Conectar OOM governor a la métrica
- **Archivos:** `src/memory_governor.rs`, `src/metrics/core/registry.rs`
- **Acción:** Donde `MemoryGovernor` previene una asignación OOM, incrementar `OOM_TRIPS` (si feature prometheus activa). Esto convierte `OOM_TRIPS` en métrica real.
- **Verify:** `cargo check -p vantadb --features prometheus`

### Step 6: Usar el pool en execute_query
- **Archivos:** `src/cli_server.rs` (execute_query)
- **Acción:** Reemplazar el bloqueo de semáforo directo por `pool.acquire()`. Si timeout → `record_failure()` + responder 503 con Retry-After. Mantener el `spawn_blocking` de ejecución.
- **Verify:** `cargo check -p vantadb --features server`

### Step 7: Test e2e del breaker
- **Archivos:** `vantadb-server/tests/server.rs` (o `e2e.rs`)
- **Acción:** Test que: construye un `CircuitBreaker` con threshold bajo y open_timeout corto, lo fuerza a Open (N fallos), hace request a `/api/v2/query` y verifica 503 + header `Retry-After`. Después del timeout, verifica HalfOpen→Closed con un request exitoso.
- **Verify:** `cargo nextest run --profile audit -p vantadb-server --test server`

## Dependencias
- Ninguna (tarea standalone). Feature `server` y `prometheus` ya existen.

## Notas
- No añadir crates nuevos (bb8/deadpool/r2d2) — el pool es un wrapper de semáforo, YAGNI.
- `OOM_TRIPS` es `IntCounter` feature-gated por `prometheus`; el incremento va con `#[cfg(feature = "prometheus")]`.
- No cambiar el path de respuesta existente (QueryResponse JSON) en el path de éxito — solo el fast-fail 503 cuando breaker/pool abiertos.

## Context Save Point
- **Fecha:** 2026-08-02T12:00
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** Scope elegido por usuario = breaker + pool explícito (opción 2 de 4). Sin dependencias nuevas.
- **Problemas conocidos:** ninguno
- **Próxima tarea:** N/A
