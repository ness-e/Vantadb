# Plan de Ejecución: FIND-89 — consolidar lecturas directas de env en `Config` (2026-09-14)

> **Fuente:** fila FIND-89 en `docs/dev/Backlog.md` (registrada por F3C Q2=B) + datos de
> ADR-043 (lista de 12 ficheros, a corregir abajo) + reconocimiento en código 2026-09-14
> (ver §Reconocimiento) + guía §1 KISS&DRY (una sola representación) y CCP.
> **Alcance:** UNA sola fuente de verdad para la configuración: todo `env::var` de
> negocio se lee vía `Config` (ya partida por dominios en F3C). Sin reorganización
> (excluida por decisión explícita del owner 2026-09-14).
> **Estado:** ✅ COMPLETED (1/1, 2026-09-14) · **Rama:** `develop`
> **Norma:** guía completa (Apéndice V manda) + BOUNDARIES.md + §10 Adaptador
> (sin campaign MCP para IDs no registrados; glob-antes-de-Read; ≥10 skills vía tool
> `skill`; verify en bash; no commitea worker; Gate D GO).

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 1 (FIND-89 consolidación) |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

## Gate P — triage

1. Bug/feature real (no cosmético): C7 dejó el comportamiento partido — `Config` publica
   `VANTADB_*` pero los lectores directos siguen usando `VANTA_*` legacy. El próximo rename
   se fractura igual. → DO.
2. Esfuerzo acotado (7 ficheros con `env::var` real + restos en ayuda/comentarios) vs impacto
   (una sola fuente, C7 completado de verdad). → DO, no DEFER.
3. Sin dependencias bloqueantes (F3C mergeada: `Config` por dominios + fachada existen).

## Reconocimiento en código (2026-09-14, corrige la lista de ADR-043)

Solo **7 ficheros** usan `env::var` real fuera de `src/config.rs` (los otros 5 de la lista
ADR-043 — `cli.rs`, `cli_handlers/server.rs`, `server/bootstrap.rs`, `error.rs`,
`server/errors.rs` — solo mencionan `VANTA_DB`/`VANTA_BACKEND` en textos de ayuda o
comentarios, más `env!("CARGO_PKG_VERSION")` en tiempo de compilación, que está bien).

| # | Fichero:línea | Var que lee hoy | Destino (hipótesis a validar en DISCOVERY) |
|---|---|---|---|
| 1 | `src/llm.rs:56,90` / `:63,72,102` / `:478,480,669,736` / `:561,563,859` | `VANTA_EMBEDDING_PROVIDER`, `VANTA_LOCAL_MODEL`, `VANTA_LLM_URL`, `VANTA_LLM_MODEL`, `VANTA_LLM_SUMMARIZE_MODEL`, `VANTA_OPENAI_API_KEY`, `VANTA_OPENAI_MODEL` | Campos `LlmCfg` (los 3 del medio ya son `VANTADB_*` en Config: usar el campo); `OPENAI_*`/`EMBEDDING_PROVIDER`/`LOCAL_MODEL` → espejos `VANTADB_*` nuevos (breaking documentado, misma doctrina C7) |
| 2 | `src/physical_plan/vector.rs:63,157` | `VANTA_LOCAL_MODEL` | Mismo campo `LlmCfg` que fila 1 |
| 3 | `src/index/graph/prefetch.rs:55,58` | `VANTA_PREFETCH`, `VANTA_DISABLE_PREFETCH` | Campo `prefetch_mode` (ya existe en Config): usar el campo |
| 4 | `src/server/telemetry.rs:35,41,97,116` | `VANTADB_LOG_JSON`, `VANTADB_LOG_FORMAT`, `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME` | `VANTADB_*` → campos Config si existen, si no añadirlos al dominio server; `OTEL_*` **SE QUEDAN** (estándar third-party OpenTelemetry, excepción documentada) |
| 5 | `src/storage/engine/maintenance.rs:550` | `VANTA_BACKUP_DIR` | Añadir al dominio storage (`StorageCfg`) como `VANTADB_BACKUP_DIR` o mapear a campo existente |
| 6 | `src/crypto.rs:179` | `VANTADB_ENCRYPTION_KEY` (prefijo ya nuevo) | Campo Config en dominio que corresponda (solo centralizar, sin rename) |
| 7 | `src/metadata.rs:27` | `ENV_REPORTED_VERSION` (const interna) | Verificar en DISCOVERY: si es mecanismo interno de versión reportada, se queda con justificación de una línea |
| 8 | `src/cli.rs:14-15`, `src/cli_handlers/server.rs:271,273`, `src/backend.rs:126` | Textos `VANTA_DB`/`VANTA_BACKEND` (ayuda/comentarios) | Actualizar textos al nombre vigente (mecánico, mismo PR) |

## Tasks

**Task 1: FIND-89 — una sola fuente de env vía `Config`**
- Appetite 1d · 🟡 · 🟠 · Archivos clave: los 7 de la tabla + `src/config.rs` (dominios F3C) + `docs/user/operations/CONFIGURATION.md` + `docs/dev/Backlog.md` (cerrar la fila al terminar)
- Gate Justificación: DRY + CCP (misma razón de cambio junta); C7 incompleto sin esto; lista verificada hoy con `rg` (no la heredada de ADR-043).
- Contrato: `rg "env::var" src/ --glob '!src/config.rs'` solo devuelve la excepción `OTEL_*` documentada + `rg "VANTA_[A-Z_]+" src/` solo devuelve `env!`/históricos congelados documentados + `cargo check --tests --all-targets` + `clippy -D warnings` + `fmt` + suites de los módulos tocados (llm/crypto/telemetry/prefetch) verdes.
- Task file `docs/dev/tasks/FIND-89.md` · ✅ COMPLETED (2026-09-14, review vanta-audit ✅ approve) · Ruta vanta-worker.
- Slices (vertical por fichero, compilable siempre): S1 `llm.rs` (el grande: campos + espejos `VANTADB_OPENAI_*`/`VANTADB_EMBEDDING_PROVIDER`/`VANTADB_LOCAL_MODEL` con breaking documentado) → S2 `vector.rs` + `prefetch.rs` (reusan campos de S1) → S3 `telemetry.rs` (`OTEL_*` exceptuados por escrito) + `maintenance.rs` + `crypto.rs` + `metadata.rs` → S4 textos ayuda/comentarios + CONFIGURATION.md + changelog (breaking de los espejos nuevos) + cerrar fila FIND-89 + cierre.
- Skills (≥10 vía `skill`): campaign-executor, progreso, systematic-debugging, test-driven-development, code-review-and-quality, doubt-driven-development, source-driven-development, planning-and-task-breakdown, codebase-memory, documentation-and-adrs.
- MCP/tools: `codegraph_explore` (quién consume cada campo nuevo), `detect_changes`; bash: `rg -n "env::var" src/ --glob '!src/config.rs'`, `rg -n "VANTA_[A-Z_]+" src/`, `cargo check -p vantadb --tests --all-targets`, `clippy -D warnings`, `fmt --check`, `nextest -p vantadb --lib llm crypto telemetry` (+ `prefetch` donde viva).
- Dependencias: ninguna (F3C mergeada). Impacto: 7 ficheros + config (solo añadir campos/espejos, sin tocar valores por defecto existentes); riesgo = breaking de espejos nuevos (documentado major/changelog, doctrina C7).
- Regla Gate V (obligatoria): variable sin mapeo claro a campo Config → `question` al humano (opciones: añadir campo nuevo / espejo `VANTADB_*` / exceptuar con justificación). **Nunca inventar semántica en silencio** (p. ej. defaults nuevos o renames no listados).
- Verify: los dos `rg` del contrato + check/clippy/fmt + suites + fila FIND-89 cerrada en Backlog + task file con tabla final var→campo.

## SKIP / DEFER / BLOQUEADO

- DEFER: nada (la reorganización está excluida por decisión, no diferida en este plan).
- BLOQUEADO: nada (Gate V resuelve mapeos dudosos en vuelo).
- SKIP: `env!("CARGO_PKG_VERSION")` (tiempo de compilación, correcto), `OTEL_*` (estándar third-party, excepción permanente documentada), históricos congelados en `docs/` (declarados).

## Grafo / Waves (FAIL_MODE=parallel, MAX 3)

```
Wave 0: FIND-89 (una sola task; slices S1→S2→S3→S4 secuenciales por compilabilidad)
```

## Riesgos globales

| Riesgo | Respuesta |
|--------|-----------|
| Defaults cambiados por accidente | cada slice conserva defaults exactos; el diff debe mostrar solo la fuente (env→campo), nunca el valor |
| Breaking no documentado | todo espejo/rename nuevo va a CONFIGURATION.md + changelog vía `feat!:` (Regla 7) |
| Comportamiento Hyrum (warn+default) | se preserva: la lectura vía campo mantiene la misma política ante inválidos |

## Cierre (ejecutado 2026-09-14)

- **Contrato:** verde — `rg env::var`→solo excepciones (test/ENV_REPORTED_VERSION/OTEL_*); `rg VANTA_`→solo comentarios config.rs; `check -p vantadb --tests` ✅; `clippy --lib -D warnings` ✅; `fmt` ✅; nextest 118/118 ✅. Fallos `--all-targets` pre-existentes F3X fuera de scope (confirmado por revisor).
- **Commits:** `1ca57649` (S1) + `3a0e42d7` (S2) + `4cdc1970` (S3) + `c4ddb217` (S4 feat!:) + `410b9b57` (fix clippy).
- **Retrospectiva Start/Stop/Continue:** Start: reconocimiento con `rg` antes de heredar listas (ADR-043 decía 12, reales 7). Stop: `Default` impls delegando a shims deprecated (rompe `-D warnings`). Continue: slices compilables + tabla var→campo + review con corridas propias.
- **`skill progreso`:** fila FIND-89 removida del Backlog (auditoría paralela misma fecha, nota en `backlog-history.md`); entrada en `docs/dev/avance/activo/core-engine.md`; nota de archivo en `docs/dev/avance/meta.md`.
- **Tras el cierre, C7 queda completado de verdad (0 legacy funcional).** Plan archivado en `docs/dev/plans/archive/`.
