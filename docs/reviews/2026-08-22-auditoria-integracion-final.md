# Auditoría Final de Integración del Producto — post roadmap TDAM

> **Fecha:** 2026-08-22 · **Modo:** read-only multi-agente (vanta-research ×2 en paralelo)
> **Alcance:** verificación de funcionamiento conjunto en 3 escenarios de consumo + re-evaluación
> de los 7 descartes deliberados del port TDAM + revisión human-facing-db-ui
> **Estado del código auditado:** P27+P29+P30+P31+P32 completas (54 tareas), suites verdes
> (vanta-memory 472 · vanta-proxy 52 · vantadb-mcp 30 · TS 246 · Python 105 · workspace 2568+)

---

## 1) Veredicto ejecutivo

**Los módulos están construidos y testeado individualmente, pero la integración producto-level tiene 3 huecos críticos** donde capacidades terminadas no llegan al usuario final:

| # | Hallazgo | Severidad |
|---|---|---|
| H1 | Write-back del proxy desconectado: `WriteBack` se construye y flushea pero nadie llama `track()` en el camino de request → los writes L0 nunca se encolan | 🔴 |
| H2 | Tools L0/L1 (`vanta_memory_capture/search`) anunciadas al modelo pero sin ejecutor: el cliente recibe un tool_call que nadie maneja | 🔴 |
| H3 | Wiki sin trigger productivo de ingest: `WikiStore` completo + worker listo + tools MCP query-only, pero no existe ruta HTTP ni tool MCP que dispare el ingest | 🔴 |
| H4 | Desktop sin acceso al pipeline vanta-memory (cero referencias en src-tauri); lente CONSOLIDAR es heurística client-side, no usa L1/L2/L3 | 🟠 |
| H5 | Skills CRUD solo vía MCP; server HTTP solo tiene GET listing | 🟠 |
| H6 | `/conversation/add` guarda threads pero NO dispara extracción L1 | 🟠 |
| H7 | providers/ (openai/ollama/litellm) e integrations/ huérfanos del ecosistema nuevo; embeddings para vanta-memory sin runner productivo fuera de adapters | 🟠 |
| H8 | `vantadb-node` superficie mínima sin graph/explain (coherente con P32 pero incompleto vs wasm/ts) | 🟢 |
| H9 | Sin sesión válida (headers ausentes) el request pasa verbatim sin memoria — correcto por diseño pero sin documentar | 🟢 |

## 2) Flujos por escenario

### Escenario 1 — Desktop Studio embebido: ⚠️
Data-plane completo y pulido (Inspector, Grafo+IQL, espacio, undo+papelera, deep-links —
41 componentes). Pero **el pipeline de memoria inteligente es invisible desde la UI**: sin
comandos Tauri IPC que expongan capture/recall/persona/scenes/skills/wiki. El requisito de
consolidación asistida de human-facing-db-ui quedó en heurística local (D16a).

### Escenario 2 — Coding agent → proxy: ⚠️ cadena rota en 2 puntos
El forward path funciona verbatim (auth→rate-limit→session→inject→forward, streaming OK).
Roto ANTES y DESPUÉS del forward: write-back sin caller (H1) y tool-calls sin executor (H2).
Alternativa parcial existente: comando in-band `mem:` opt-in.

### Escenario 3 — Server HTTP API: ❌ para las capacidades nuevas
Las rutas legacy funcionan, pero wiki (sin rutas), skills (solo lectura), memory pipeline
(sin exposición) y conversation/add (sin L1) no llegan al protocolo HTTP.

## 3) human-facing-db-ui — logrado vs faltante
✅ Workspace unificado, HOME overview, Inspector master-detail + Historial/Diff, filtros
compuestos, undo+papelera, Ctrl+K, grafo+IQL, timeline, cursor pagination, import drag&drop,
deep links.
❌ Consolidación asistida conectada a pipeline real · skills/wiki en UI · servir consola
desde proceso embebido OPFS.

## 4) Re-evaluación de los 7 descartes deliberados

| Descarte | Re-veredicto |
|---|---|
| Redis | ✅ Justificado — rol cubierto in-process (rate_limit.rs, session.rs, LocalStateBackend) |
| SQLite metadata | ✅ Justificado y superado — EntityStore/InternalMetadata ya implementa lo que SYNTHESIS marcaba como propuesta |
| TencentVDB | ✅ Obvio — HNSW/DiskANN/IVF propios |
| COS refs offload | ⚠️ Mayormente justificado — matiz: binarios tras result_ref sin hogar; revisar con server-mode |
| **Opik/Langfuse/CH/Kafka** | ⚠️ **Prematuro en parte** — `ReportHook` ya existe (report.rs); un hook OTLP→Langfuse/OTel es esfuerzo S con valor medio-alto para agentes productivos |
| **agent-adapters claude-code** | ⚠️ **Prematuro en parte** — `classifyCcRequest` + `extractLastUserText` (~2 funciones, esfuerzo S) son necesarios para tráfico Claude Code real (routing forks + extracción de user text); D26 no los sustituye. Los otros 4 adapters sí eran stubs innecesarios |
| MemoryPanel | ✅ Razonable — falta admin remota multi-equipo, irrelevante hasta server-mode |

## 5) Acciones derivadas → registradas en Backlog

MEM-50 (H1) · MEM-51 (H2) · MEM-52 (H3) · MEM-53 (H4) · MEM-54 (H5) · MEM-55 (H6) ·
MEM-56 (Langfuse hook) · MEM-57 (claude-code parse) · MEM-58 (consolidación UI↔pipeline) ·
BND-05 (vantadb-node graph/explain).

## 6) Conclusión

**La extracción TDAM está completa al nivel crate/test. Lo que falta es la ÚLTIMA MILLA de
integración producto**: cablear piezas terminadas entre sí (proxy↔memory writes, wiki→fachada,
desktop→pipeline). Son tareas de integración pequeñas individualmente pero que definen si el
producto funciona como sistema. Se recomienda planificarlas como campaña "Última Milla" antes
de cualquier release público.
