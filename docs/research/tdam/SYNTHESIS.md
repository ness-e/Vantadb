# TDAM → VantaDB — SYNTHESIS de la investigación (9 reportes)

> Fecha: 2026-08-18 · Fuente: `docs/research/tdam/01..09-*.md` (8 sub-agentes + cobertura deploy/uso, clone completo `feat/server_team` v2.0.0-beta.1)
> Estado: ✅ verificación multi-agente completada en 2 rondas — ronda 1: 05/06/07/08 REVISADOS y corregidos (rutas/símbolos fabricados reemplazados por refs reales), 01 ampliado (StatefulPipelineManager/telemetría/gateway/offload), 02/03/04 parcheados, 09 añadido; ronda 2: PLAN + SYNTHESIS verificados contra clon y reportes (atribuciones §2.3 corregidas, propuestas marcadas, omisiones añadidas — ver final del doc).
> Objetivo: unir la investigación, decidir qué se porta y dónde, y definir los siguientes pasos.

## 1. Lectura cruzada — qué es TDAM de verdad

Los 9 reportes confirman una sola cosa: **TDAM es una orquestación LLM encima de un SQLite**; VantaDB es el motor que TDAM no tiene. TDAM gana en **semántica de sesión y gestión de ventana de contexto**; VantaDB gana en **almacenamiento, grafo y local-first**. No hay ni una pieza de TDAM que requiera copiar su stack de persistencia — todo lo valioso es **modelo de datos, prompts/algoritmos LLM-driven y patrones de orquestación**.

Mapeo de los 9 reportes a los 10 gaps originales:

| # | Gap | Reporte | Esencia portátil | Dónde vive |
|---|---|---|---|---|
| 1 | Context offload | 05 | swap compresión→re-inyección MMD; mild/aggressive/emergency LLM-free + token estimator | `vanta-memory` (engine) + core (cursor por sesión — patrón `lastOffloadedToolCallId` de 05 §3) |
| 2 | Scene extraction | 02 | L2 = agente LLM con tools; contrato META {created,updated,summary,heat} + scene_name | `vanta-memory` + core (nodo escena) |
| 3 | Persona L3 | 02 | triggers por checkpoint + diff incremental + navegación por heat | `vanta-memory` |
| 4 | Skill memory | 03 | snapshots inmutables multi-versión + optimistic lock + TTL + extracción LLM review agent | core (esquema) + `vanta-memory` (extracción) |
| 5 | Metadata plane | 06 | entidades teams/users/agents/tasks/assets + permission-checker allow-only (cadena real: resource→owner→member→visibility→role-default→ACL) | core (`entity_*` — propuesta: hoy el repo no tiene tablas entity_*; checker real 96 líneas) |
| 6 | Consolidación triggers | 01 | timers per-session + locks por grano (L1 session / L2-L3 agent) | server/integrations (orquestación) |
| 7 | Telemetría por capa | 01,06 | latencias por L1/L2/L3/recall + envelope trace | core (metrics snapshot ya existe) |
| 8 | Proxy transparente | 07 | transport verbatim OpenAI/Anthropic + ciclo inject→forward→write-back | `vanta-proxy` (binario opcional) |
| 9 | Billing/quota | 06 | CreditCalculator + NoopQuotaReporter (ojo: TDAM tiene 2 calculadoras inconsistentes — ÷1000 en credit-calculator vs ÷10000 en metric-tracking) | `vantadb-server` (solo server mode) |
| 10 | Wiki ingest + MCP | 08 | state machine pending→ready + chunker 12k/400 + merge serial + 12 tools query-only | `vanta-memory`/MCP (grafo ya existe en core) |
| — | Deploy/uso/plugins | 09 | 3 imágenes (`memory-core/hub/proxy`) + Panel+KS combined + hermes/openclaw plugins + CLI seed; **referencia directa** (09 §8): patrón `setup-claude-code.sh` → MCP (`base_url`), Dockerfile proxy → vanta-server, CLI seed → comando de import, SDK memory-core → estructura sdk/python+node | referencia (09) |

## 2. Decisiones por capa (principio rector: core puro, sin LLM, WASM-compatible)

### 2.1 Core (Rust, LLM-free) — 3 adiciones baratas, sin nuevos deps
1. **Search profile por namespace** (04): `mode: keyword|vector|hybrid`, `rrf_k`, `candidate_k` — parámetros expuestos en el store; RRF ya existe.
2. **Entidades `entity_*` + permission-checker allow-only** (06): modelo de datos teams/users/agents/tasks/assets + checker allow-only de 96 líneas reales (cadena resource→owner→member→visibility→role-default→ACL→deny, 06 §1.4). **Nota de verificación:** el repo VantaDB hoy NO tiene tablas `entity_*` (modelo namespace+key); es una adición propuesta, no "ya existente" — verificar contra 06 §6 antes de implementar. RBAC ya existe (`src/rbac.rs`), esto es el modelo de datos.
3. **Tabla `skills` multi-versión** (03): `(skill_id, version)` UNIQUE + `is_head`, índice único parcial por owner+name, `expected_version` optimistic lock, TTL keep-recent=3, content-hash idempotencia — esquema genérico, sin LLM.
   - Opcional: nodo **escena** (02) con META contract en el grafo + cursor `lastOffloadedToolCallId` (05) por sesión.

### 2.2 vanta-memory (crate nuevo, la pieza que falta — LLM-driven)
- **Context Engine offload** (05): `assemble(msgs, ratio) → {messages, report}` (firma real de 05; el patrón es portar el ensamblado diferido, no la firma literal) — mild cascade por score (guard summary>original revierte), aggressive one-shot con fingerprint, emergency; token estimator: **3 chars/token** (01 §9 metric-tracking) o tiktoken `o200k_base`/`cl100k_base` (05) — decidir al implementar; **MMD Mermaid como memoria de tarea persistente** *(propuesta VantaDB: reusar el contrato META de 02 en el MMD de 05 — no está en los reportes)*. L1/L1.5/L2/L4 LLM opcionales; modo LLM-free = compresión local mild/aggressive/emergency sin LLM (05) *(detalle "summaries del sistema" = propuesta)*.
- **L1 extracción + dedup** (01): contratos `MemoryRecord`/`DedupDecision`, split new(10)+background(5), dedup 2 fases (recall → juicio LLM store/update/merge/skip).
- **L2 scenes + L3 persona** (02): agente LLM con tools read/write/edit sandboxed, heat, soft-delete `[DELETED]`, triggers por checkpoint (5 prioridades), navegación por heat.
- **Skill extracción** (03): transcript con marcadores anti role-capture + pre-carga de skills para dedup + review agent sin delete.
- **Recall patrón** (04): `prependContext` (memorias, dinámico) vs `appendSystemContext` (persona/escenas, cacheable) + `sanitizeText` anti feedback-loop + budget con truncación por code-point.

### 2.3 Extensión de lo existente (sin servicios nuevos)
- **vantadb-mcp**: añadir tools `skill_*` (6 del review agent, 03), `scene_*` (read/list por heat, 02) y las 12 tools query-only del patrón MCP de 08 (`code_*`/`wiki_*`, mounts `/v3/wiki/*`). **Propuesta VantaDB (sin cita a TDAM):** `graph_bfs/dfs/explain/meta` sobre el graphrag existente (08 NO expone `graph_*`) y `memory_tier` en put/search (06 §3.1: campo que TDAM no tiene). El patrón wiki-ingest (08) expone el graphrag existente, no un servicio nuevo.
- **vantadb-server**: rutas `/conversation/add` (01 §10) y `/skill/listing` (03 §4) como data plane de referencia. **Ojo (07 §3):** TDAM NO tiene `/v3/session/init` (el init es state machine local team→agent→task) ni `/v3/knowledge/query` (solo `/v3/knowledge/list` + tools MCP self-discovery) — si VantaDB las expone son endpoints propios, no copia. Quota solo en server mode (06).
- **vanta-proxy (binario opcional)**: interceptación verbatim de 3 protocolos wire (OpenAI Chat Completions / Anthropic Messages / Responses API, 07) + ciclo auth→session→inject→rate-limit→forward→write-back→reporting; `spaceId` del path; patrón mem-command de 07 (`mem:sync|create-skill|help`; VantaDB definiría sus propios comandos — `/remember` `/forget` son propuesta, no TDAM). Sin los 5 agent-adapters específicos de TDAM.

### 2.4 Qué NO copiar (deuda TDAM)
- Split de 4 servicios + Redis/streams/locks multi-nodo (01, 03, 04) — VantaDB local-first.
- SQLite/sqlite-vec/FTS5 + dual-write JSONL + memory-cleaner (01, 04) — VantaDB ES el motor; fuente única en el store.
- Sidecar BM25 Python / jieba-wasm (04) — salvo target zh.
- `ZERO_VEC_BUFFER` hack, `IMemoryStore`/`IStorageBackend` split (04) — YAGNI.
- Cola Redis/DLQ/mutex de skills (03) — pipeline síncrono o cola local.
- Store **dual SQLite/Mongo** `meta_*` (SQLite default con 12 tablas; Mongo solo si `TDAI_METADATA_MONGO_URI`) + router monolítico 55 endpoints (06) — generado desde schema.
- `@colbymchenry/codegraph` (08) — VantaDB ya tiene grafo propio.
- Prompts en chino con reglas Kenty (01) — extraer principios, reescribir.
- Dual cliente/servidor offload (05) — elegir UNA vía (server/MCP).
- Deploy TDAM: 3 imágenes Docker + Panel+KS combined + hermes/openclaw plugins + CLI seed (09) — contexto de empaquetado, no portar el layout.

## 3. Orden de implementación propuesto (por valor/esfuerzo)

| Fase | Qué | Reporte | Esfuerzo | Valor |
|---|---|---|---|---|
| **F1** | Search profile por namespace (mode/rrf_k/candidate_k) en core + exponer en MCP/search | 04 | S | 🔥 barato y visible |
| **F2** | Entidades entity_* + permission-checker allow-only en core | 06 | M | Alto: multi-agente real |
| **F3** | Skill memory: esquema multi-versión en core + tools MCP | 03 | M | Alto: activos > texto |
| **F4** | `vanta-memory` crate: L1 extracción+dedup + scenes + persona (agentes LLM) | 01, 02 | L | 🔥 killer semántico |
| **F5** | Context Engine offload en `vanta-memory` (assemble + MMD + mild/aggressive) | 05 | L | 🔥 killer contexto |
| **F6** | `vanta-proxy` binario (transparent injection) | 07 | M | Alta adopción coding agents |
| **F7** | Wiki ingest state machine + tools query-only MCP (usa graphrag existente) | 08 | M | Medio |

Notas del orden:
- **Consolidación por triggers** (01, tabla fila 6) no es fase propia: es la orquestación server (timers per-session + locks por grano) que habilita F4 — implementar dentro de F4 como capa de orquestación.
- **Billing/quota** (06, fila 9) queda **diferido** a server mode — no en F1–F7. Al portar, decidir UNA calculadora de crédito (TDAM tiene dos inconsistentes: ÷1000 credit-calculator vs ÷10000 metric-tracking).
- **F7 incluye** los patrones de seguridad de 08 §6: SSRF blocklist del git-fetcher, callback S2S con `run_id`, y `locked:true` en state machines — no solo las tools.

F1–F3 son LLM-free → pueden ir al core sin romper WASM. F4–F5 son la pieza nueva grande (~1 crate). F6–F7 opcionales en una segunda iteración.

## 4. Riesgos globales
- **Coste LLM**: cada flush L1/L1.5/L2 = 3 llamadas; local-first exige modo LLM-free y control de triggers (01, 05).
- **Compresión pierde detalle**: refs solo se re-leen a demanda; documentar el trade-off (05).
- **No romper prompt caching**: el patrón prepend/append es obligatorio desde F1 (04).
- **404 vs 403** anti-enumeración: documentar para debugging (06).
- **RRF client-side ignora scores absolutos** — el umbral no aplica en hybrid (baja precisión con candidateK pequeño) (04 §7.1); relevante para F1.
- **Heat lo mantiene el LLM, no un contador real** — limita la promesa de "navegación por heat" en L2/L3 (02 §8).
- **CreditCalculator inconsistente en TDAM** (÷1000 vs ÷10000): no copiar ambas, elegir una (spot-check clon).

## 5. Próximos pasos
1. ✅ Investigación + verificación multi-agente completadas (11 archivos: PLAN, 01–09, SYNTHESIS).
2. Confirmar este orden con el usuario (F1–F7).
3. ADR para el crate `vanta-memory` + decisiones de core (documentation-and-adrs).
4. Abrir tareas en `docs/Backlog.md` (F1 primero: search profile por namespace + MCP).
