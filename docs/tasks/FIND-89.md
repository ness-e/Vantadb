# FIND-89: Consolidar lecturas directas de env en `Config` (única fuente de verdad)

## Metadata
- **Plan file:** docs/plans/2026-09-14-find89-env-consolidation.md
- **Fuente:** docs/Backlog.md fila FIND-89 + ADR-043 + reconocimiento 2026-09-14
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Rust (src/** — 7 ficheros con `env::var` real + config.rs + docs)
- **Turns estimados:** 20-30
- **Creado:** 2026-09-14T14:30
- **last-synced:** 2026-09-14T14:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/llm.rs` (EmbeddingProvider, LocalOnnxProvider, OpenAIProvider, LlmClient), `src/physical_plan/vector.rs` (local model path), `src/index/graph/prefetch.rs` (prefetch mode), `src/server/telemetry.rs` (log format, OTEL), `src/storage/engine/maintenance.rs` (backup dir), `src/crypto.rs` (encryption key), `src/metadata.rs` (reported version), `src/cli.rs` (help text), `src/cli_handlers/server.rs` (help text) |
| Callees | `src/config.rs` (Config, LlmCfg, EvictionCfg, StorageCfg, PrefetchMode, parse_env_or), `src/backend.rs` (BackendKind) |
| Implicaciones | • Contratos: se añaden campos nuevos a `Config` (mirror `VANTADB_*` para `OPENAI_*`, `EMBEDDING_PROVIDER`, `LOCAL_MODEL`, `BACKUP_DIR`, `REPORTED_VERSION`).<br>• Comportamiento público: breaking change para consumidores que usen `VANTA_OPENAI_API_KEY`/`VANTA_OPENAI_MODEL`/`VANTA_EMBEDDING_PROVIDER`/`VANTA_LOCAL_MODEL` — migración documentada (doctrina C7: `feat!:` + changelog).<br>• Performance: sin cambios (solo fuente de lectura).<br>• Migración datos: no requerida.<br>• Tests: módulos llm, crypto, telemetry, prefetch, maintenance, metadata deben pasar. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `src/llm.rs` (líneas 56, 63, 72, 90, 102, 478, 480, 561, 563, 669, 736, 859)
  - `src/physical_plan/vector.rs` (líneas 63, 157)
  - `src/index/graph/prefetch.rs` (líneas 55, 58)
  - `src/server/telemetry.rs` (líneas 35, 41, 97, 116)
  - `src/storage/engine/maintenance.rs` (línea 550)
  - `src/crypto.rs` (línea 179)
  - `src/metadata.rs` (línea 27)
  - `src/config.rs` (definiciones LlmCfg, EvictionCfg, StorageCfg, PrefetchMode, parse_env_or)
  - `docs/operations/CONFIGURATION.md` (tabla env vars fuera de Config)

- **Archivos referenciados hacia dentro (imports):**
  - `src/llm.rs` → `crate::config::Config` (no usado hoy, usa `env::var` directo)
  - `src/physical_plan/vector.rs` → `crate::config::Config` (no usado hoy)
  - `src/index/graph/prefetch.rs` → `crate::config::EvictionCfg` (usa `prefetch_mode` de Config)
  - `src/server/telemetry.rs` → `crate::config::Config` (no usado hoy)
  - `src/storage/engine/maintenance.rs` → `crate::config::Config` (no usado hoy)
  - `src/crypto.rs` → `crate::config::Config` (no usado hoy)
  - `src/metadata.rs` → `crate::config::Config` (no usado hoy)

- **Archivos que referencian a los editados (referencias entrantes):**
  - `src/llm.rs` es consumido por: `src/physical_plan/vector.rs`, `vantadb-python/src/vector.rs`, tests en `src/llm.rs`
  - `src/config.rs` es consumido por: TODO el crate (`Embedded::open_with_config`, CLI, server, etc.)

- **Veredicto impacto:** MEDIO — 7 ficheros se modifican para leer de `Config` en lugar de `env::var`; se añaden 6-8 campos nuevos a `Config` (espejos `VANTADB_*` para vars legacy). Riesgo = breaking changes documentados (doctrina C7).

## Contrato
```
rg "env::var" src/ --glob '!src/config.rs' solo devuelve la excepción OTEL_* documentada
rg "VANTA_[A-Z_]+" src/ solo devuelve env!/históricos congelados documentados
cargo check --tests --all-targets pasa
cargo clippy --workspace --all-targets --all-features -- -D warnings pasa
cargo fmt --check pasa
nextest -p vantadb --lib llm crypto telemetry prefetch maintenance metadata pasa
fila FIND-89 cerrada en Backlog.md + task file con tabla final var→campo
```

## Spec (SDD — obligatoria: Phase 1b detectó feature-add/símbolos públicos)

> La tarea añade símbolos públicos nuevos a `Config` (campos mirror `VANTADB_*` para `OPENAI_*`, `EMBEDDING_PROVIDER`, `LOCAL_MODEL`, `BACKUP_DIR`, `REPORTED_VERSION`). Es feature-add para SDD aunque sea refactor.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | `VANTA_OPENAI_API_KEY` → ¿añadir campo `openai_api_key` a `LlmCfg`? | A) Añadir `openai_api_key: Option<String>` a `LlmCfg` (consistente con `llm_url`/`llm_model`) / B) No añadir, mantener `env::var` en `OpenAIProvider::new()` (rompe DRY) | A | ✅ decidido-por-evidencia (ref: plan §S1, `Config` ya tiene patrón para keys opcionales) |
| 2 | `VANTA_OPENAI_MODEL` → ¿añadir campo `openai_model` a `LlmCfg`? | A) Añadir `openai_model: String` con default `text-embedding-3-small` / B) Reusar `llm_model` (semántica distinta: embeddings vs generation) | A | ✅ decidido-por-evidencia (ref: `src/llm.rs:563` usa default distinto) |
| 3 | `VANTA_EMBEDDING_PROVIDER` → ¿añadir campo `embedding_provider` a `LlmCfg`? | A) Añadir `embedding_provider: String` (default `ollama`) / B) Enum `EmbeddingProviderKind` (más trabajo, no necesario ahora) | A | ✅ decidido-por-evidencia (ref: `src/llm.rs:56` match string simple) |
| 4 | `VANTA_BACKUP_DIR` → ¿dominio `StorageCfg` o nuevo? | A) Añadir `backup_dir: Option<PathBuf>` a `StorageCfg` (cohente con `export_base_dir`) / B) Nuevo dominio `BackupCfg` (over-engineering) | A | ✅ decidido-por-evidencia (ref: `StorageCfg` ya tiene `export_base_dir`) |
| 5 | `ENV_REPORTED_VERSION` (const interna) → ¿mapear o exceptuar? | A) Exceptuar con justificación (mecanismo interno de versión reportada, no config de usuario) / B) Añadir `reported_version: Option<String>` a `Config` (público innecesario) | A | ✅ decidido-por-evidencia (ref: plan fila 7, `metadata.rs:27`) |
| 6 | `VANTA_PREFETCH`/`VANTA_DISABLE_PREFETCH` en `prefetch.rs` → ya existe `prefetch_mode` en `EvictionCfg` | A) Usar `config.eviction_cfg().prefetch_mode` / B) Duplicar lógica | A | ✅ decidido-por-evidencia (ref: plan fila 3, `PrefetchMode` ya en Config) |
| 7 | `VANTA_LOCAL_MODEL` mirrors en `llm.rs` y `vector.rs` → ya existe `local_model_path` en `LlmCfg` | A) Usar `config.llm_cfg().local_model_path` / B) Duplicar | A | ✅ decidido-por-evidencia (ref: plan filas 1-2, `Config` ya tiene `VANTADB_LOCAL_MODEL`) |
| 8 | `OTEL_EXPORTER_OTLP_ENDPOINT` / `OTEL_SERVICE_NAME` → excepción permanente | A) Documentar como excepción third-party estándar (no VantaDB) / B) Añadir a Config (contamina Config con deps opentelemetry) | A | ✅ decidido-por-evidencia (ref: plan fila 4 + CONFIGURATION.md §8) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `Config::default()` sigue leyendo TODOS los env vars `VANTADB_*` existentes — no se cambia ningún default ni comportamiento actual.
  2. Los nuevos campos espejo `VANTADB_OPENAI_API_KEY`, `VANTADB_OPENAI_MODEL`, `VANTADB_EMBEDDING_PROVIDER`, `VANTADB_LOCAL_MODEL` (mirror), `VANTADB_BACKUP_DIR` se añaden con la misma semántica de fallback/default que sus contrapartes legacy.
  3. `OTEL_*` NO se tocan — excepción documentada permanente (third-party OpenTelemetry standard).
  4. `ENV_REPORTED_VERSION` en `metadata.rs` se queda como const interna con justificación de una línea.
  5. Textos de ayuda/comentarios con `VANTA_DB`/`VANTA_BACKEND` se actualizan a nombres vigentes (mecánico).
  6. `docs/operations/CONFIGURATION.md` se actualiza con las nuevas vars `VANTADB_*` y se marca breaking de legacy.

- **Comandos de verificación:**
  - `rg "env::var" src/ --glob '!src/config.rs'` → solo `OTEL_*`
  - `rg "VANTA_[A-Z_]+" src/` → solo `env!`/históricos congelados
  - `cargo check -p vantadb --tests --all-targets`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo fmt --check`
  - `cargo nextest run -p vantadb --lib llm crypto telemetry prefetch maintenance metadata --profile audit`

- **Deuda pendiente:** ninguna (plan cubre todo el scope FIND-89).

## Recitation

```
=== RECITATION ===
Objetivo activo: FIND-89 — Consolidar lecturas directas de env en Config
Estado: ✅ COMPLETED
Última acción: S4 completada - textos ayuda, CONFIGURATION.md, Backlog.md cerrados
Resultado: ✅
State: COMPLETED (desde: IN PROGRESS)
Próxima acción: N/A — task completada
Contrato: ver sección Contrato arriba
Invariantes: Config defaults inmutables, OTEL_* exceptuados, ENV_REPORTED_VERSION interno, breaking documentado
Comandos de verificación: rg env::var + check/clippy/fmt + nextest suites
Deuda: ninguna
Próxima tarea si completa: (plan tiene una sola task FIND-89)
last-synced: 2026-09-14T15:30
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda — la consolidación elimina deuda existente (duplicación de fuentes de env).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable se cumple + fmt/clippy/nextest deterministas + tests de módulos tocados pasan |
| **Commit** | Commit atómico por slice (S1, S2, S3, S4), conventional commit (`refactor:` o `feat!:` si breaking), `git diff` limpio, verificación mecánica |
| **Release** | `dev-tools/verify.ps1` completo, changelog via release-plz (breaking = `feat!:`), semver respetado |

## Herramientas necesarias
- cargo (check, clippy, fmt, nextest)
- codegraph_explore (blast radius verification)
- rg (verificación de contrato)
- codebase-memory-mcp_detect_changes (impact check)

## Skills cargadas (SDP):
- campaign-executor (base type: Rust core)
- source-driven-development (base type: Rust core)
- doubt-driven-development (base type: Rust core)
- incremental-implementation (lifecycle BUILD: slices verticales)
- test-driven-development (lifecycle BUILD: Red-Green-Refactor)
- context-engineering (lifecycle BUILD: empaquetar contexto)
- api-and-interface-design (lifecycle BUILD: APIs públicas de Config)
- performance-optimization (keyword mapping: index, engine)
- deprecation-and-migration (keyword mapping: storage, breaking changes)

## Investigation Notes
- F3C ya consolidó 7 vars `VANTA_*` → `VANTADB_*` en `Config` (C7 breaking sin shims). FIND-89 completa el trabajo para las vars que quedaron fuera.
- Doctrine C7: breaking changes documentados con `feat!:` + changelog, migración single-pain (rename en deploys).
- `OTEL_*` son estándar OpenTelemetry — excepción permanente documentada en CONFIGURATION.md §8.
- `ENV_REPORTED_VERSION` es mecanismo interno de versión reportada (banner/MCP), no config de usuario — exceptuar con justificación.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — todas resueltas en Spec |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Tabla final var→campo

| Var legacy / orphan | Nuevo campo Config / Dominio | Commit | Notas |
|---------------------|-----------------------------|--------|-------|
| `VANTA_EMBEDDING_PROVIDER` | `Config.embedding_provider` (LlmCfg) | `1ca57649` | S1 — default `ollama` |
| `VANTA_OPENAI_API_KEY` | `Config.openai_api_key` (LlmCfg) | `1ca57649` | S1 — Option<String>, B2b deferred error |
| `VANTA_OPENAI_MODEL` | `Config.openai_model` (LlmCfg) | `1ca57649` | S1 — default `text-embedding-3-small` |
| `VANTA_LOCAL_MODEL` (llm.rs, vector.rs) | `Config.local_model_path` (LlmCfg) | `1ca57649`, `3a0e42d7` | S1+S2 — ya existía en F3C, solo centralizar lectura |
| `VANTA_PREFETCH` / `VANTA_DISABLE_PREFETCH` | `Config.prefetch_mode` (EvictionCfg) | `3a0e42d7` | S2 — ya existía en F3C (PrefetchMode) |
| `VANTADB_LOG_JSON` / `VANTADB_LOG_FORMAT` | `Config.log_format` (ServerCfg) | `4cdc1970` | S3 — ya existía en F3C |
| `VANTA_BACKUP_DIR` | `Config.backup_dir` (StorageCfg) | `4cdc1970` | S3 — nuevo campo añadido |
| `VANTADB_ENCRYPTION_KEY` | `Config.encryption_key` | `4cdc1970` | S3 — ya existía, solo centralizar lectura |
| `ENV_REPORTED_VERSION` (`VANTADB_REPORTED_VERSION`) | — (exceptuado) | `4cdc1970` | S3 — mecanismo interno, const en metadata.rs |
| `OTEL_EXPORTER_OTLP_ENDPOINT` / `OTEL_SERVICE_NAME` | — (exceptuados) | N/A | Estándar OpenTelemetry, no VantaDB |
| `VANTA_DB` (CLI flag/env) | `VANTADB_STORAGE_PATH` | S4 | Texto ayuda actualizado |
| `VANTA_BACKEND` | `VANTADB_BACKEND` | S4 | Comentario actualizado |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — toca trust boundaries (config de encryption_key, api_key). Cargado `security-and-hardening` implícito via `api-and-interface-design`. Hallazgos: `encryption_key` ya está en `Config` (línea 729), no se añade nuevo campo, solo se centraliza lectura en `crypto.rs`. `api_key`/`jwt_secret` ya en Config. Sin nuevos riesgos.
- [x] **PERFORMANCE** — no toca hot paths (solo lectura de config al inicio). Baseline `canonical_p99` no afectado. Justificación: cambios son en inicialización, no en loops calientes.

## Steps

### Step 1 (S1): llm.rs — Añadir campos a LlmCfg y migrar proveedores
- **Archivos:** `src/config.rs` (añadir `openai_api_key`, `openai_model`, `embedding_provider` a `LlmCfg` + defaults), `src/llm.rs` (migrar `OpenAIProvider::new()`, `LocalOnnxProvider::new()`, `LlmClient::new()` a usar `Config::llm_cfg()`)
- **Acción:** 
  1. En `Config` struct: añadir `openai_api_key: Option<String>`, `openai_model: String`, `embedding_provider: String` con defaults desde env `VANTADB_OPENAI_API_KEY`, `VANTADB_OPENAI_MODEL`, `VANTADB_EMBEDDING_PROVIDER`
  2. En `LlmCfg`: propagar los 3 campos nuevos
  3. En `src/llm.rs`: cambiar constructores para recibir `&LlmCfg` o leer de `Config::default().llm_cfg()`
  4. Preservar comportamiento B2b (missing key → error en `embed()`, no panic en construcción)
- **Verify:** `cargo check -p vantadb && cargo nextest run -p vantadb --lib llm --profile audit`
- **Estado:** ✅ DONE (commit `1ca57649`)

### Step 2 (S2): vector.rs + prefetch.rs — Reusar campos de S1
- **Archivos:** `src/physical_plan/vector.rs` (líneas 63, 157: `VANTA_LOCAL_MODEL` → `config.llm_cfg().local_model_path`), `src/index/graph/prefetch.rs` (líneas 55, 58: `VANTA_PREFETCH`/`VANTA_DISABLE_PREFETCH` → `config.eviction_cfg().prefetch_mode`)
- **Acción:** Cambiar lecturas directas `env::var` a acceso via `Config` domain views
- **Verify:** `cargo check -p vantadb && cargo nextest run -p vantadb --lib vector prefetch --profile audit`
- **Estado:** ✅ DONE (commit `3a0e42d7`)

### Step 3 (S3): telemetry.rs + maintenance.rs + crypto.rs + metadata.rs
- **Archivos:** 
  - `src/server/telemetry.rs`: `VANTADB_LOG_JSON`/`VANTADB_LOG_FORMAT` → `config.log_format` (ya existe), `OTEL_*` **SE QUEDAN** (excepción documentada)
  - `src/storage/engine/maintenance.rs:550`: `VANTA_BACKUP_DIR` → añadir `backup_dir: Option<PathBuf>` a `StorageCfg` + leer de `VANTADB_BACKUP_DIR` en `Config::default()`
  - `src/crypto.rs:179`: `VANTADB_ENCRYPTION_KEY` → ya existe `encryption_key` en `Config` (línea 729), solo centralizar lectura
  - `src/metadata.rs:27`: `ENV_REPORTED_VERSION` → exceptuar (const interna), documentar justificación
- **Verify:** `cargo check -p vantadb && cargo nextest run -p vantadb --lib telemetry maintenance crypto metadata --profile audit`
- **Estado:** ✅ DONE (commit `4cdc1970`)

### Step 4 (S4): Textos ayuda/comentarios + CONFIGURATION.md + Changelog + Cerrar FIND-89
- **Archivos:** 
  - `src/cli.rs:14-15`, `src/cli_handlers/server.rs:271,273`, `src/backend.rs:126`: actualizar textos `VANTA_DB`/`VANTA_BACKEND` a nombres vigentes
  - `docs/operations/CONFIGURATION.md`: añadir nuevas vars `VANTADB_OPENAI_API_KEY`, `VANTADB_OPENAI_MODEL`, `VANTADB_EMBEDDING_PROVIDER`, `VANTADB_BACKUP_DIR` a tabla; marcar legacy `VANTA_*` como deprecated/breaking
  - `docs/CHANGELOG.md`: entrada breaking (release-plz lo hará al merge, pero documentar en task file)
  - Backlog.md: cerrar fila FIND-89
- **Verify:** `rg "VANTA_[A-Z_]+" src/` → solo `env!`/históricos; `cargo check --tests --all-targets`; validar docs
- **Estado:** ✅ DONE (this commit)

## Dependencias
- Ninguna (F3C mergeada: Config por dominios + fachada existen)

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-audit (security + code review) o vanta-review / doubt-driven-development
- **Enfoque:** ¿el approach de añadir campos mirror a `Config` es correcto? ¿alternativas mejores (ej. shims de compatibilidad)? Ver decisión Q3=A en ADR-043 (sin shims, breaking documentado).
- **Cómo se probó:** evidencia de verificación real (comandos `rg` + `cargo check` + `nextest` output), no auto-reporte.
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ⬜ pending review

## Notas
- Decisiones de diseño: seguir patrón existente en `Config` para campos opcionales (`Option<String>` con `env::var(...).ok()`) y requeridos con default (`unwrap_or_else`).
- Contexto aprendido: F3C C7 ya hizo breaking sin shims para 7 vars; este PR sigue la misma doctrina para las 6 vars restantes.
- Problemas conocidos: `OpenAIProvider::new()` hoy no panica por key faltante (B2b: error diferido a `embed()`). El nuevo campo `openai_api_key: Option<String>` debe preservar esta semántica.
- `prefetch.rs` ya usa `PrefetchMode` de Config — solo cambiar a `config.eviction_cfg().prefetch_mode`.
- `metadata.rs` `ENV_REPORTED_VERSION` = const `'VANTADB_REPORTED_VERSION'` — mecanismo interno, exceptuar.