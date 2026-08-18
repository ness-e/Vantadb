# Plan de Investigación Profunda — TencentDB-Agent-Memory (TDAM)

> **Objetivo:** Extraer absolutamente todo del proyecto TDAM (lógica, algoritmos, procesos, flujos, funciones, funcionalidades, uso, diseño, arquitectura, UX/UI) con referencias y código, para decidir qué incluir en VantaDB y dónde.
>
> **Fuente:** `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` (rama `feat/server_team`, v2.0.0-beta.1, MIT, ~22.8k⭐ ago 2026)
> **Repositorio TDAM:** monorepo de 3 servicios + 1 proxy + 1 SDK: MemoryCore (:8420), MemoryKnowledge (fuente :8421 / deploy :8424), MemoryPanel (fuente :8123 / deploy :8125), MemoryProxy (:8096), sdk/memory-core (TS + Python).
> **Fecha:** 2026-08-18

---

## 1. Contexto previo (síntesis de lo ya investigado)

- TDAM es un **producto de memoria para agentes** (aplicación/orquestación), VantaDB es **el motor de almacenamiento embebido** que va años adelante en storage.
- La pirámide semántica L0–L3 (conversación cruda → resumen por sesión → entidades/escenas → conocimiento de equipo) es el modelo conceptual a extraer.
- Los 10 gaps priorizados ya identificados: **#1 Context offload/reclamation (killer), #2 Scene extraction, #3 Persona L3, #4 Skill memory, #5 Metadata plane, #6 Consolidación por triggers, #7 Telemetría por capa, #8 Proxy de inyección, #9 Billing, #10 Wiki auto-ingest+MCP**.
- Decisión arquitectónica adoptada: **core VantaDB se mantiene puro (sin LLM, WASM-compatible)**; las features LLM-driven viven en un crate nuevo `vanta-memory`; se extiende `vantadb-mcp`/`vantadb-server`/`integrations/`; no se copia el split interno de 4 servicios de MemoryCore (pipeline-worker / timer-scanner / worker-permit-pool / state-backend).

## 2. Método

1. **8 sub-agentes de investigación en paralelo** (`vanta-research`), cada uno con una porción del monorepo TDAM.
2. Cada agente **lee el código real** (Read/Grep/Glob sobre el path de TDAM), cita `archivo:línea`, extrae lógica/algoritmos/flujos, y **escribe su archivo de investigación** en `docs/research/tdam/`.
3. Regla de oro: los agentes **NO modifican fuente de VantaDB ni de TDAM** — solo escriben su `.md` de investigación.
4. **Verificación posterior multi-agente** (`vanta-review`, contexto fresco): cada reporte se contrasta contra el clon real; rutas/símbolos fabricados se corrigen (05/06/07/08 REVISADOS) y se añade el reporte 09 (`09-deploy-usage.md`) para las áreas de cobertura faltantes (deploy/uso/scripts/plugins).
5. Unificación: el lead lee los archivos completos y produce `SYNTHESIS.md` con los siguientes pasos.

## 3. División de trabajo (8 agentes + 1 de cobertura)

| # | Archivo de salida | Scope TDAM (directorios) | Temas |
|---|---|---|---|
| 01 | `01-core-pipeline.md` | `MemoryCore/src/core/abstractions`, `core/conversation`, `core/hooks`, `core/prompts`, `core/record`, `core/seed`, `core/state`, `core/store`, `core/tools` | TdaiCore fachada, HostAdapter/LLMRunner, pipeline L0→L3, auto-capture, consolidation, state machine, prompts |
| 02 | `02-scene-persona.md` | `MemoryCore/src/core/scene`, `core/persona`, `core/profile` | Scene extraction + scene index + MMD, persona generation L3, triggers por checkpoint, perfiles |
| 03 | `03-skill-memory.md` | `MemoryCore/src/core/skill` (+ `conversation-add`, `prompts`, `queue`) | Skill memory versionada: CRUD, versionado, recursos blob, RAG search, routing, extracción desde conversación, fast-path, permisos LLM |
| 04 | `04-storage-recall.md` | `MemoryCore/src/core/storage`, `core/store`, `core/record` (parcial), `core/seed` (parcial) | SQLite + sqlite-vec + FTS5, TCVDB, recall 3 modos (keyword/embedding/hybrid), RRF k=60, candidateK, BM25 sin embedding, multi-backend (SQLite/TCVDB para memoria + local/COS para archivos) |
| 05 | `05-offload.md` | `MemoryCore/src/offload`, `offload_server`, `offload-client`, `offload/hooks`, `offload/local-llm`, `offload/pipelines` | **Context window management (killer):** MMD injector, token counter, L3 aggressive compression, reclaimer GC por mtime, estimadores de tokens, state manager, session registry |
| 06 | `06-metadata-acl.md` | `MemoryCore/src/metadata`, `gateway`, `core/quota` | Metadata plane (teams/users/agents/tasks/assets + ACL, 55 endpoints /v3/meta, store dual SQLite/Mongo), auth 3 capas, quota/credits (CreditCalculator) — la telemetría real (core/report) se cubre en 01 §9 y 07 §8 |
| 07 | `07-proxy.md` | `MemoryProxy/src` completo | Proxy LLM transparente: injection, session init (state machine local), write-back, rate limiting (fail-open sin Redis), triple protocol (OpenAI Chat Completions / Anthropic Messages / Responses API), 5 agent-adapters (claude-code/codebuddy/codex/workbuddy/dsh), mem-command (mem:sync\|create-skill\|help), skill/knowledge bridges, report backends |
| 08 | `08-knowledge-panel-sdk.md` | `MemoryKnowledge/`, `MemoryPanel/`, `sdk/memory-core` | Wiki auto-ingest (fetcher→chunker→LLM→chunks), CodeGraph (@colbymchenry/codegraph), MCP server (12 tools code_*/wiki_*, mounts /v3/wiki/*), auto-sync scheduler, Panel web (UX/UI), SDK TS + Python |
| 09 | `09-deploy-usage.md` | `deploy/`, `assets/`, `.github/`, `scripts/`, `bin/`, plugins/ | Deploy (3 imágenes memory-core/hub/proxy, Panel+KS combined, puertos deploy), plugins hermes/openclaw, CLI seed, estructura SDK |

## 4. Formato estándar de cada archivo de investigación

```markdown
# TDAM — [Área] — Investigación profunda
> Fecha: 2026-08-18 · Agente: vanta-research · Scope: [directorios]

## 1. Resumen ejecutivo
## 2. Arquitectura y flujo (diagramas ASCII)
## 3. Lógica y algoritmos (con referencias archivo:línea)
## 4. Funcionalidades / Endpoints / APIs
## 5. UX/UI (si aplica)
## 6. Código clave (fragmentos citados + ref)
## 7. Integración en VantaDB (dónde/cómo/por qué — core puro vs vanta-memory vs MCP/server/integrations)
## 8. Riesgos / limitaciones / qué NO copiar
## RESULTADO
- Estado: ✅ COMPLETO
- Archivo: docs/research/tdam/XX-*.md
- Hallazgo principal: (1 línea)
- Ref clave: (archivo:línea más importante)
```

## 5. Criterios de calidad por agente

1. **Todo con referencias**: cada afirmación de lógica debe citar `ruta:línea`.
2. **Código real citado**: fragmentos relevantes (firmas, structs, algoritmos) con su ref.
3. **Flujos explícitos**: entrada → proceso → salida (state machines, pipelines, callbacks).
4. **UX/UI solo donde aplique**: MemoryPanel (08), scene MMD navigation (02), proxy UX (07).
5. **Recomendación de integración VantaDB obligatoria** en cada archivo (sección 7): qué va al core (LLM-free), qué a `vanta-memory` (LLM-driven), qué a MCP/server/integrations, qué NO copiar.
6. **Sin implementar nada**: solo investigación + escritura del .md.

## 6. Entregables

- 9 archivos de investigación (`docs/research/tdam/01..09-*.md`) — 01–08 por sub-agentes + 09 de cobertura deploy/uso
- Verificación multi-agente: 05/06/07/08 REVISADOS (rutas/símbolos fabricados reemplazados por refs reales), 01 ampliado (StatefulPipelineManager/telemetría/gateway/offload), 02/03/04 parcheados
- `SYNTHESIS.md` (lead): unión, análisis cruzado, decisiones y próximos pasos (ADR + tareas al backlog)

## 7. Cierre

Una vez completados los 9, el lead lee TODOS los archivos, cruza hallazgos, resuelve contradicciones y produce la síntesis con el plan de acción concreto (qué crate, qué features core, qué MCP tools, orden de implementación).