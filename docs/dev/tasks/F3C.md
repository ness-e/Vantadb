# F3C — S-split-config B+B: diseño (anidado + fachada + unificación env breaking)

> **Plan:** `docs/dev/plans/2026-09-13-cleanCA-fase3.md` · **Wave:** 2 (tras F3X ✅) · **Ruta:** vanta-arch (diseño + ADR-datos) → vanta-worker (F3C-impl por slices)
> **Appetite:** 2–3d (diseño aquí; implementación = F3C-impl tras aprobación) · **Esfuerzo:** 🔴 · **Riesgo:** 🟠 (breaking env documentado)
> **Estado:** ⬜ PENDING (diseño C0 completo en este archivo; BLOQUEADO hasta respuesta humana a §QUESTION) · **Rama:** `develop`
> **Prohibición:** cero cambios en `src/` en esta tarea. Solo este task file. La implementación es F3C-impl.
> **Decisión vigente:** B+B (Q4 Fase 3 + D0): sub-structs por dominio + fachada `Config` plana + unificación `VANTA_*→VANTADB_*` breaking EN EL MISMO cambio.
> **Constraint inviolable:** `apply_to` :203 (8 campos) + watcher 100% en `src/config.rs`.
> **Dependencias:** F3X ✅ COMPLETO (`13f0f729`, hoja neutral `src/index/port_impl.rs`) — colisión en `storage/engine/init.rs` ya resuelta; no bloquea F3B (disjunta: workflows/benches vs config). **nextTask:** F3B / cierre.
> **Research base:** `docs/dev/tasks/C2D0.md` (E1–E3 + Duda) — este archivo actualiza SOLO lo que cambió desde D0.

## SDP (phase=PLAN, keywords: config/ADR/decision)

- `documentation-and-adrs` — ADR-datos con Contexto/Decisión/Consecuencias vacíos para el humano (Regla 5).
- `doubt-driven-development` — revisión adversarial del diseño B+B (§Duda-F3C).
- `api-and-interface-design` — Contract First / Hyrum's Law / One-Version Rule aplicados a fachada + env breaking.
- `database-design` — versionado de formato: sin config persistida en disco → sin migración de datos (re-verificado).
- `spec-driven-development` — spec-cero: §QUESTION estructurada con opciones antes de F3C-impl.
- `idea-refine` — fachada A vs B divergidas antes de converger (recomendación con árbitro = compilador).
- `interview-me` — §QUESTION en formato intención-con-restate, no prosa abierta.
- `planning-and-task-breakdown` — slices C1–C7 reversibles con gate `cargo check` por slice.

## Steps (esta tarea: solo C0; C1–C7 = F3C-impl tras aprobación)

- [x] **C0a — re-medición D0 en árbol post-F3X** (conteos + anclas + churn) → §Evidencia + H1/H2
- [x] **C0b — tabla dominio→struct** (criterio + campos ancla verificados) → §1
- [x] **C0c — contrato de fachada** (Default/with_*/plano/HotReload/watcher) → §2
- [x] **C0d — secuencia de slices + migración + env + changelog** → §3
- [x] **C0e — ADR-datos** (solo datos; decisión = humano, Regla 5) → §4
- [x] **C0f — UNA ronda question con opciones + (Recomendado)** → §5
- [ ] **C1–C7 — F3C-impl** (vanta-worker, tras respuesta humana; NO esta tarea)

## Evidencia re-medida (2026-09-14, post-F3X `13f0f729`) — delta vs D0

Anclas estables (sin movimiento desde D0): `HotReloadConfig:151`, `from_config:187`, `apply_to:203` (8 `update!`: `prefetch_mode, log_format, rate_limit_rpm, batch_size, wal_buffer_size, flush_threshold, insert_lock_timeout_ms, sync_mode`), `pub struct Config:248–467` (cierre verificado línea 467), `parse_env_or:534`, `watch_config:1181`, `apply_hot_reload_from_value:1261` (misma línea que D0 cita como watcher; verificado por Read).
Watcher confinado: `rg -l "watch_config|apply_hot_reload_from_value" src/` → **solo `src/config.rs`** ✅ (constraint INTACTO).
`git log -- src/config.rs` → **63 commits** (D0: 62; el 63.º es `e5dc9022` C2S2, que solo refactorizó el parseo de `VANTA_BACKEND` vía `BackendKind::from_name` — **cero campos añadidos/eliminados**).
Struct `Config`: **55 campos `pub`** (53 incondicionales + 2 bajo `#[cfg]` — `advanced-tokenizer`, `hot-reload`) — D0 decía "≈52 (+2)". Como C2S2 no tocó campos, el delta es **ruido de aproximación de D0, NO HALLAZGO**.
F3X verificado: `13f0f729` creó `src/index/port_impl.rs` (hoja neutral, 424L) + ADR-042; `storage/engine/init.rs` ya migrado — la dependencia "tras F3X" está **SATISFECHA**; F3C-impl relee `init.rs` post-F3X sin colisión activa.

### HALLAZGO H1 — blast radius 102 → 108 ficheros con `Config {` (64 en `src/` + 44 en `tests/`)

Método idéntico a D0 (`rg -l "Config {" src/ tests/`). Causa: campaña Fase 2 + F3X (ficheros nuevos/movidos, p. ej. `port_impl.rs`). **Efecto:** los slices C1–C6 migran **108 sitios, no 102**; el gate por slice sigue siendo `cargo check` (el compilador es el árbitro, no el conteo `rg`).

### HALLAZGO H2 — env vars 42 → 57 literales únicos en `src/`; legacy 7 → 12 `VANTA_*`

Método D0 = keys de parseo en `config.rs`; re-medición = todos los literales `"VANTADB_*"/"VANTA_*"` en `src/` (incluye **lecturas directas `env::var` en 12 ficheros fuera de `config.rs**: `cli.rs`, `crypto.rs`, `cli_handlers/server.rs`, `llm.rs`, `metadata.rs`, `index/graph/prefetch.rs`, `server/bootstrap.rs`, `physical_plan/vector.rs`, `server/telemetry.rs`, `error.rs`, `server/errors.rs`, `storage/engine/maintenance.rs` — más mensajes `warn!` y doc-comments).
Legacy actual (12): `VANTA_BACKEND`, `VANTA_BACKUP_DIR`, `VANTA_DB`, `VANTA_DISABLE_PREFETCH`, `VANTA_EMBEDDING_PROVIDER`, `VANTA_LLM_MODEL`, `VANTA_LLM_SUMMARIZE_MODEL`, `VANTA_LLM_URL`, `VANTA_LOCAL_MODEL`, `VANTA_OPENAI_API_KEY`, `VANTA_OPENAI_MODEL`, `VANTA_PREFETCH`.
**Efecto:** (a) la unificación breaking debe cubrir 12 vars legacy, no 7; (b) los 12 ficheros con lectura directa son **deuda a inventariar en C0 de F3C-impl**: o se consolidan en `Config` en este cambio o nacen como fila FIND (ver Q2 en §5). Sin config persistida en disco (env + memoria + builders `with_*`) → **sin migración de datos** en ningún caso (re-verificado).

## §1 — Tabla dominio→struct (criterio + campos ancla verificados)

Criterio de asignación (árbitro final = compilador en cada slice): cada campo va al dominio de su **único escritor** (E1-D0: cambio-conjunto intra-sección 14/14); los 8 campos hot-reload quedan además referenciados por `apply_to` sin mover su semántica. La enumeración exhaustiva de los 55 campos se fija en C0 de F3C-impl con `rg` + `cargo check` (este diseño fija criterio, structs y anclas — no inventa la lista campo por campo).

| Dominio → struct (nombre a validar Q4) | Campos ancla verificados | Evidencia churn (D0 E1, vigente) |
|---|---|---|
| `ServerCfg` | `host`, `port`, `jwt_secret`, `api_key`, `alt_api_key`, `require_auth`, `allow_insecure`, `rate_limit_rpm`, `dashboard_dir`, `audit_log_path`, `audit_max_bytes`, `audit_max_files` | Polo dominante **9/14** commits (SRV-01…08, FIND-07, TSK-107b) — primer slice |
| `StorageCfg` | `storage_path`, `backend_kind`, `version_history_limit`, `segment_optimizer`, `sync_mode`, `wal_buffer_size`, `flush_threshold` | 2/14 + C2S2 reciente (registry/factory) — segundo slice |
| `LlmCfg` | `llm_url`, `llm_model`, `llm_summarize_model`, `local_model_path` (doc: `VANTA_LOCAL_MODEL`) | 1/14 (EMB-02) + polo legacy env (`VANTA_LLM_*`, `VANTA_OPENAI_*`, `VANTA_EMBEDDING_PROVIDER`) |
| `EvictionCfg` | `prefetch_mode` (hot-reload ⚠️), `memory_limit` | 1–2/14 (PERF-04, PERF-06) |
| `PoolCfg` | timeouts pool, circuit-breaker, `insert_lock_timeout_ms` (hot-reload ⚠️), `batch_size` (hot-reload ⚠️) | 1/14 (ENT-04) |
| `RbacCfg` ¿propio o anidado en `ServerCfg`? → **Q1 §5** | `rbac_config` | **Cero churn propio**: nació dentro del polo server (SRV-04) y nunca cambió solo — candidato a quedar anidado en `ServerCfg` |
| Transversal (NO es struct) | `HotReloadConfig` 8 campos + `apply_to:203` + watcher `:1181/:1261` | Constraint inviolable: no se mueve, no cambia semántica |

## §2 — Contrato de la fachada `Config`

`Config` sigue existiendo como **fachada plana**: `Default`, todos los `with_*` builders, acceso plano a campos (`config.port`, `config.storage_path`, …), `..Default::default()` en los 108 sitios, `HotReloadConfig` + watcher intactos. Los sub-structs son organización interna; ningún constructor externo nombra `StorageCfg{…}` directamente (todo pasa por `Config`).

Opciones de fachada (divergidas en diseño; árbitro = compilador + review):
**A. Fuente única en sub-structs + acceso plano compat** (métodos/campos re-exportados en `Config`; sin duplicación de estado — sin riesgo de divergencia).
**B. Campos planos duplicados + `Deref` a sub-structs** (riesgo de divergencia entre las dos fuentes; solo si A choca con `..Default::default()` en algún sitio).
**Recomendado: A** (One-Version Rule: una sola fuente de verdad; Hyrum: el comportamiento observable — warn+default ante env inválido, nunca fail-fast — se preserva idéntico).
`VantaError` mapping sin cambios; ningún `panic`/`unwrap` nuevo en hot path; validación solo en boundaries (parseo env/CLI), el core confía en tipos (Regla §3a).

## §3 — Secuencia de slices + migración + env + changelog (para F3C-impl)

Orden por churn (polo dominante primero; falla rápido donde más duele): **C1 `ServerCfg`** (+Q1: absorbe `RbacCfg` si el humano lo decide) → **C2 `StorageCfg`** (releer `init.rs` post-F3X) → **C3 `LlmCfg`** → **C4 `EvictionCfg`** → **C5 `PoolCfg`** → **C6 `RbacCfg`** (solo si Q1 = propio; si no, se omite) → **C7 unificación env + cierre**.
Gate por slice (mecánico, reversible): `rg` del dominio + migración + `cargo check -p vantadb --tests --all-targets` + `clippy -D warnings` + `fmt` verdes antes del siguiente slice. Suites config al cierre: `nextest -p vantadb config`.
Migración 108 sitios: mecánica (`rg -l "Config {"`), preservando `..Default::default()`; si un sitio rompe por la fachada → Gate V (`question`: ajustar fachada vs sitio, nunca silenciar).
**C7 — unificación breaking (B+B, mismo cambio):** 12 `VANTA_*` → `VANTADB_*` (lista H2), **sin shims/aliases** (dolor único, decisión Q4). Documentación del breaking **sin tocar `docs/CHANGELOG.md` a mano** (Regla 7: lo genera release-plz): commit `feat!:` + footer `BREAKING CHANGE:` + sección en `docs/user/operations/CONFIGURATION.md` (tabla legacy→nuevo + nota de migración) — el changelog nace del commit, no de edición manual. Semver: major (`feat!:`).
Pre-mortem (plan): 108 sitios mecánicos con compilador como red; breaking env contenido en C7 con changelog generado.

## §Duda-F3C (doubt-driven, 1 ciclo, no-interactivo)

- CLAIM: "B+B (split anidado + fachada + unificación env mismo cambio) es ejecutable en 2–3d sin romper hot-reload".
- Ataque: (1) B+B viola One-Version Rule ("una ruptura por vez" — D0 recomendaba B+A); el humano la asumió documentada en Q4, pero el riesgo de doble ruptura simultánea (108 sitios + 12 vars) concentra el blast radius en C7. (2) La fachada plana que no rompe los 108 sitios deja el god-struct de lectura intacto → beneficio cosmético (Duda-D0 finding 3, vigente). (3) 12 lecturas directas `env::var` fuera de `config.rs` sobreviven al split si no se consolidan → dos fuentes de config.
- Reconciliación: (1) es **trade-off firmado por el humano** (Q4) y queda explícito en C7 + Q3; (2) se acepta como costo documentado (el valor es cohesión de escritura + prefijos consistentes, no el lector); (3) se deriva a **Q2** (consolidar vs FIND). La recomendación se mantiene como análisis, NO decisión (Regla 5).

## §4 — ADR-datos (Regla 5: el HUMANO escribe Contexto/Decisión/Consecuencias)

- **Contexto:** _(lo escribe el humano con sus palabras)_
- **Decisión:** _(la escribe el humano — opciones en §5; B+B ya vigente, Q1–Q3 la precisan)_
- **Consecuencias:** _(las escribe el humano; la IA solo aportó Evidencia+H1/H2+§1–§3+Duda)_
- **Datos aportados por IA (referencia):** anclas `config.rs` (:151/:187/:203/:248–467/:534/:1181/:1261); 55 campos (53+2 cfg); H1 108 sitios (64+44); H2 57 literales / 12 legacy / 12 ficheros con lectura directa; E1–E3 + Duda de C2D0.md (base, sin re-evaluar figment/config-rs); Duda-F3C con 3 findings.
- **Deuda de verificación:** enumeración exhaustiva 55 campos → C0 de F3C-impl; docs.rs no re-chequeado (D0 no se re-evalúa salvo divergencia — no la hay).

## §5 — QUESTION — UNA ronda (al humano; respuesta BLOQUEA F3C-impl, no este diseño)

| # | Pregunta | Opciones | Recomendado |
|---|---|---|---|
| 1 | `RbacCfg` ¿struct propio o anidado en `ServerCfg`? (cero churn propio desde SRV-04) | **A.** Anidado en `ServerCfg` (menos structs, refleja churn real) / **B.** Propio `RbacCfg` (simetría 6 dominios, reserva crecimiento RBAC) | **A** (Recomendado) — el churn manda; B solo si prevés crecimiento RBAC propio |
| 2 | 12 ficheros con `env::var` directo fuera de `config.rs` (H2): ¿consolidar en `Config` en este cambio o dejar? | **A.** Consolidar en este cambio (una sola fuente; +alcance C1–C6) / **B.** Dejar + fila FIND con dueño (alcance acotado; dos fuentes temporalmente) | **B** (Recomendado) — One-Version Rule: el split ya es bastante ruptura; la consolidación es su propio cambio |
| 3 | Fachada + C7: ¿ratificás opción A (fuente única) y unificación SIN shims en el mismo cambio (B+B)? | **A.** Sí: fachada A + C7 sin shims (B+B tal cual Q4) / **B.** Fachada A + C7 con shims temporales (compat deploys; rompe "un solo dolor") / **C.** Reabrir B+A (unificación diferida) | **A** (Recomendado) — B+B vigente Q4; B/C solo si cambió tu apetito de breaking |

> Nota de harness: no hay tool `question` en esta sesión → esta tabla ES la ronda. Respondé p. ej. "A+B+A" y F3C-impl se define con esa respuesta. Sin respuesta, F3C-impl sigue BLOQUEADO.

## Verify contrato (esta tarea: solo lectura + este archivo)

- [x] Anclas `config.rs` re-leídas (Read, no memoria): :151/:187/:203/:248–467/:534/:1181/:1261
- [x] `rg -l "watch_config|apply_hot_reload_from_value" src/` → solo `src/config.rs`
- [x] `rg -l "Config {"` src/ tests/ → 108 (64+44) — H1 registrado
- [x] Literales env en `src/` → 57 únicos / 12 legacy / 12 ficheros directos — H2 registrado
- [x] `git log -- src/config.rs` → 63 (63.º = C2S2, cero campos) — sin HALLAZGO en campos
- [x] F3X completo verificado (`13f0f729` + `port_impl.rs`) — dependencia Wave 2 satisfecha
- [x] Cero ediciones en `src/` — `git status --short` limpio en `src/` (solo este task file nuevo + `.opencode`/completions ajenos al scope)
- [ ] Respuesta humana a §5 (pendiente — BLOQUEA F3C-impl, no esta tarea)

## Recitation (handoff a F3C-impl / F3B)

- **Objetivo:** F3C diseño C0 ✅ en `docs/dev/tasks/F3C.md` — cero código, cero commits.
- **Invariantes:** `apply_to` 8 campos + watcher en `config.rs` intocables; warn+default (nunca fail-fast); `docs/CHANGELOG.md` solo vía release-plz; sin respuesta a §5 no hay F3C-impl.
- **Deuda:** enumeración exhaustiva 55 campos → C0 de F3C-impl; §5 pendiente (BLOQUEO).
- **Próxima:** F3B (disjunta, puede correr en paralelo) o F3C-impl tras respuesta humana.

## F3C-impl (2026-09-14, decisiones humanas B/B/A vinculantes)

- Q1=B `RbacCfg` propio (6to dominio) · Q2=B 12 ficheros no se tocan → FIND-89 ·
  Q3=A fachada A fuente única + C7 sin shims (B+B Q4 vigente).
- [x] **C1 `ServerCfg`** (17 campos) + `server_cfg()` — polo dominante primero
- [x] **C2 `StorageCfg`** (15 campos) + `storage_cfg()` — releído `init.rs` post-F3X, sin colisión
- [x] **C3 `LlmCfg`** (4+1 cfg) + `llm_cfg()` — incluye `advanced_tokenizer_config` cfg-gated
- [x] **C4 `EvictionCfg`** (8 campos) + `eviction_cfg()` — `memory_limit`/`prefetch_mode` aquí
- [x] **C5 `PoolCfg`** (8 campos) + `pool_cfg()` — `batch_size`/`insert_lock_timeout_ms` aquí
- [x] **C6 `RbacCfg`** propio + alias compat `RbacConfig` + `rbac_cfg()` — 108 sitios intactos
- [x] **C7 unificación breaking sin shims** (solo `src/config.rs`): 7 vars
  `VANTA_LLM_URL/MODEL/SUMMARIZE/LOCAL/PREFETCH/DISABLE_PREFETCH/BACKEND` →
  `VANTADB_*`; `rg -n "VANTA_" src/config.rs` vacío; `tests/prefetch_benchmark.rs`
  migrado; nota de migración en `docs/user/operations/CONFIGURATION.md` (changelog vía
  release-plz `feat!:` + `BREAKING CHANGE:`, Regla 7); FIND-89 para los 12 ficheros;
  ADR-043 solo datos (firma = humano, Regla 5).
- Verify: `cargo check -p vantadb --tests --all-targets` ✅ ·
  `cargo clippy -p vantadb --all-targets -- -D warnings` ✅ (1 `clone_on_copy` fixed) ·
  `cargo fmt --check -p vantadb` ✅ · `cargo test -p vantadb --lib config` 57 ✅
  (3 F3C nuevos ✅) · censo `Config {` 108 (64+44) sin migración ·
  `nextest -p vantadb config` full-suite no corrido: rustc `STATUS_STACK_BUFFER_OVERRUN`
  compilando targets no relacionados (`openapi_yaml_parity`, `text_index_recovery`,
  `backend_tests`, `durability_recovery`) — toolchain local, deuda fuera de contrato.
- **Estado:** ⏳ IN PROGRESS (implementado + verificado, SIN commit — commitea el lead
  con `feat!:` + footer `BREAKING CHANGE:`). nextTask: cierre de campaña.

> **Cierre 2026-09-14:** COMPLETED (diseno + impl C1-C7 B/B/A + 57/57 + 0 legacy; commit d75459fe feat!:+BREAKING CHANGE; ADR-043 datos, firma humana pendiente Regla 5).
