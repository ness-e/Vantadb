---
title: "Backlog de investigación — OpenViking → VantaDB"
kind: research
status: proposed
description: "Registro archivístico de la evaluación estática de OpenViking (b8e8895) y propuestas de reimplementación para VantaDB"
tags: [openviking, backlog, research]
---

# Backlog de investigación — OpenViking → VantaDB

> **Propósito:** registro archivístico ordenado de toda la investigación sobre OpenViking (sesiones del 2026-11-06/07) y del informe final de diez puntos, preservado para evaluación futura. **No es un plan de implementación ni una fila ejecutable del backlog**: cada propuesta requiere su propia especificación, revisión legal y validación antes de entrar al triage técnico.
> **Relación con el backlog activo:** [`Backlog.md`](./Backlog.md) sigue siendo la fuente del parser de `/pipeline plan`. Nada en este archivo entra al triage sin una fila `FIND-*`/`MEMG-*`/etc. propia conforme a la taxonomía oficial.
> **Estado de ejecución:** durante toda la investigación **no se ejecutó** código, tests, benchmarks, servicios ni instalaciones de ninguno de los dos repositorios; la fase de inspección original no produjo commits de implementación. Todo el contenido es evidencia estática o claims de terceros, con fecha de corte explícita.
> **Nota de extensión:** conservado en un archivo por solicitud del propietario; si se convierte en trabajo activo, dividir fichas en specs y enlazar este registro histórico, sin duplicar el backlog.
> **Advertencia de vigencia:** las rutas y líneas citadas de OpenViking corresponden al snapshot `b8e8895`; las de VantaDB a la base indicada en §0.2. El worktree actual de VantaDB tiene ~119 ficheros modificados sin commit (incluidos `checkpoint.rs`, `l1_writer.rs`, `context_engine`, `ingest`); **todo hallazgo marcado [Histórico] exige revalidación contra el worktree actual antes de actuar**. No se afirma que ningún defecto siga presente ni que esté corregido.

## Índice

- [0. Contexto y evidencia](#0-contexto-y-evidencia)
- [1. Recomendaciones para VantaDB](#1-recomendaciones-para-vantadb)
- [2. Fichas de extracción y reimplementación en Rust](#2-fichas-de-extracción-y-reimplementación-en-rust)
- [3. Issues upstream aplicables a VantaDB](#3-issues-upstream-aplicables-a-vantadb)
- [4. Prerequisitos a corregir en VantaDB antes de extraer](#4-prerequisitos-a-corregir-en-vantadb-antes-de-extraer)
- [5. Lista de rechazo — qué no trasladar](#5-lista-de-rechazo--qué-no-trasladar)
- [6. Advertencias metodológicas](#6-advertencias-metodológicas)
- [7. Benchmarks aplicables a VantaDB](#7-benchmarks-aplicables-a-vantadb)
- [8. Matriz de brechas de capacidades](#8-matriz-de-brechas-de-capacidades)
- [9. Negativos y deuda detectados en VantaDB](#9-negativos-y-deuda-detectados-en-vantadb)
- [10. Evaluación de escritorio / UI](#10-evaluación-de-escritorio--ui)
- [Apéndice A. Inventario de fuentes](#apéndice-a-inventario-de-fuentes)
- [Apéndice B. Decisiones corregidas por la revisión adversarial](#apéndice-b-decisiones-corregidas-por-la-revisión-adversarial)
- [Apéndice C. Roadmap de dependencias](#apéndice-c-roadmap-de-dependencias)
- [Apéndice D. Verificación pendiente — solo manual](#apéndice-d-verificación-pendiente--solo-manual)
- [Apéndice E. Incógnitas y lagunas de evidencia](#apéndice-e-incógnitas-y-lagunas-de-evidencia)

---

## 0. Contexto y evidencia

### 0.1 Alcance ejecutivo

El usuario pidió clonar `volcengine/OpenViking` fuera del repo, evaluarlo con múltiples agentes en paralelo (qué hace, cómo, estructura, benchmarks, qué migrar y qué issues aplican a VantaDB) y, en una segunda fase, una síntesis en diez puntos incluyendo la app de escritorio. Se ejecutaron revisiones paralelas de: arquitectura, motor/memoria/retrieval, seguridad defensiva, incidencias públicas, escritorio/UI y una revisión adversarial independiente (red-team) del dictamen; el agente principal revisó personalmente la metodología de benchmarks.

**Conclusión estratégica consolidada:** OpenViking es un competidor relevante en la *experiencia integral* de memoria/contexto para agentes (filesystem semántico navegable + resúmenes materializados + ciclo de vida de agente), no un sustituto del motor embebido de VantaDB. Lo más aprovechable son **patrones de diseño** (identidad de contexto, derivados versionados, presupuesto de contexto, ledger de entrega, merge declarativo, verificador de consistencia, pipeline observable, experiencias estructuradas, watches incrementales, inspector de evidencia), nunca código (AGPL-3.0). VantaDB ya tiene mecanismos propios de ANN, híbrido y durabilidad; no hay comparación equivalente medida para establecer superioridad. El modelo "LLM en hot path" no debe hacerse obligatorio en el core.

### 0.2 Snapshots y bases de código

| Repositorio | Base analizada | Estado al archivar este documento |
|---|---|---|
| OpenViking | `b8e88959f3ab437b4d08514eceaa7221cca470dc` (`main`, commit 2026-11-06, `feat(langchain): preserve owned-client authentication and Store cleanup (#7039)`) | Clon completo (no shallow), limpio, sin submódulos, 6.017 ficheros en `C:\Users\Eros\Open Vikings\OpenViking` |
| VantaDB | Base consolidada `632f49187ef8fce8e21abea78fba7ecbb8c5c463` (la auditoría de incidencias registró `f6ed51d058467226b073af71eea045fca4fa875e` en su momento de ejecución) | HEAD `c561158253bb88192590fa406c61e0a27dd7df8d` con ~119 ficheros modificados sin commit, incluidos `vanta-memory/src/utils/checkpoint.rs`, `vanta-memory/src/core/record/l1_writer.rs`, `vanta-memory/src/context_engine/*`, ingest y storage engine |

**Regla de lectura:** toda afirmación de defecto de VantaDB en este documento es **[Histórico]** — describe la base estática analizada, no el worktree actual. Revalidar antes de crear filas de backlog.

### 0.3 Leyenda de evidencia

| Etiqueta | Significado |
|---|---|
| **[Hecho]** | Evidencia directa en código en la línea citada del snapshot fijado |
| **[Inferencia]** | Conclusión derivada del código; mecanismo plausible, no demostrado dinámicamente |
| **[Hipótesis]** | Requiere verificación manual/dinámica; no demostrado |
| **[Claim]** | Afirmación publicada por terceros (README, blog, issues), no reproducida |
| **[Histórico]** | Hallazgo fechado; el worktree actual puede haberlo alterado |

### 0.4 Arquitectura verificada de OpenViking

OpenViking es una **plataforma de contexto para agentes** organizada como filesystem virtual `viking://` — no una base vectorial pura ni comparable 1:1 con el motor embebido de VantaDB.

```text
CLI `ov` (Rust) / SDKs Python·Go·TS / MCP / plugins / Studio React
                        │
                        ▼
OpenViking Server (FastAPI + auth plugins: dev/api_key/trusted/OIDC/LDAP)
   └─ OpenVikingService
        ├─ VikingFS: ls/tree/read/write/grep/abstract/overview/find/search
        ├─ ResourceService → ingest queue
        ├─ SessionService → archive + commit + memory extraction
        ├─ CompileService → runtime/VikingBot externo
        └─ QueueManager → colas persistentes at-least-once
             (add_resource / semantic / embedding / reindex /
              session_commit / external_task)
                        │
                        ▼
RAGFS (Rust `ragfs`, binding `ragfs-python` vía maturin)
   filesystem lógico, locks, cache opcional, S3/SQLite/MySQL/Redis opcionales
                        │
                        ▼
Vector backends: local flat / VikingDB / Volcengine / openGauss / cuVS
```

- Composición del servicio: `openviking/service/core.py:82-157`; inicialización de RAGFS, QueueManager y VikingDBManager: `core.py:187-235` — **[Hecho]** en `b8e8895`.
- Registro de routers REST (fs, resources, search, sessions, skills, compile, watches, MCP, ACL, admin, metrics, WebDAV): `openviking/server/app.py:675-702` — **[Hecho]**.
- `VikingFS` como fachada: `openviking/storage/viking_fs/__init__.py:110-126` — **[Hecho]**.
- **Gateway, bot y Studio son productos separados** del servicio core: `openviking_gateway/` (proxy LLM + captura + management API), `bot/` (VikingBot, canales, sandbox), `web-studio/` (SPA React).
- **Materialización de resúmenes:** sidecars por directorio `.abstract.md` (L0) y `.overview.md` (L1) generados por LLM bottom-up (`storage/queuefs/semantic_processor.py:1560-1724`); L0 es literalmente la primera línea del L1 (`:1560-1589`). L2 = fuente.
- **Matiz crítico:** `HierarchicalRetriever.retrieve` (`retrieve/hierarchical_retriever.py:102-279`) ejecuta **una única búsqueda vectorial global** filtrada por `level`/`scope` + dedup por URI + rerank opcional — **no** hay cascada automática L0→L1→L2. Lo jerárquico es el *layout de storage*, no el retrieval — **[Hecho]**, confirmado por la revisión adversarial.
- **Los nombres no son intercambiables con los niveles cognitivos L0–L3 de VantaDB:** en OV, L0/L1/L2 son *representaciones materializadas del mismo contenido*; en VantaDB, L0–L3 es un lifecycle de memoria. Cualquier analogía directa es un error de mapeo.

Rutas citadas en todo el documento son relativas al checkout `C:\Users\Eros\Open Vikings\OpenViking` salvo indicación contraria; las referencias `archivo:línea` son **[Histórico]** a `b8e8895` y deben reverificarse antes de usarse como enlace normativo.

### 0.5 Mapa de código — referencias históricas (OV `b8e8895`)

| Área | Componente | Referencia de lectura |
|---|---|---|
| Servicio | Composición e init | `openviking/service/core.py:82-157`, `:187-235` |
| API | Routers REST | `openviking/server/app.py:675-702` |
| FS semántico | Fachada VikingFS | `openviking/storage/viking_fs/__init__.py:110-126` |
| Namespaces/scopes | URI `viking://`, scopes, `default_target_directories` | `openviking/core/namespace.py:187-253` |
| Scope filter | AST `PathScope` pushdown al motor | `openviking/storage/scope_filter.py:36-68`, `openviking/storage/vectordb/index/local_index.py:163-165` |
| Retrieval | Búsqueda global filtrada + rerank | `openviking/retrieve/hierarchical_retriever.py:102-279`, `rerank_scoring.py:281-310` |
| Assembler | Pipeline de contexto: gather/budget/tiers/params/render/ledger | `openviking/retrieve/context_assembler/{pipeline,gather,budget,tiers,params,render,ledger,models,expansion,recall_preset,rewrite}.py` |
| Derivados L0/L1 | Generación bottom-up de sidecars | `openviking/storage/queuefs/semantic_processor.py:1560-1724` |
| Consistencia | Verificador FS↔índice por nivel | `openviking/storage/index_consistency.py:14-50` (`IndexExpectation.level :25-41`) |
| Memoria | Extracción ReAct | `openviking/session/memory/extract_loop.py` |
| Memoria | Merge ops declarativas | `openviking/session/memory/merge_op/{base,factory,patch,patch_handler,replace,sum,link_merge,immutable}.py`; plantillas `openviking/system_assets/memory/memory_templates/builtin/*.yaml` |
| Memoria | Dedup (solo firma textual para session-skills) | `openviking/session/skill/dedup.py:15-48` |
| Memoria | Hotness/retención/TTL | `openviking/retrieve/memory_lifecycle.py:18-63`, `openviking/session/retention.py:462-479`, `openviking/ttl/README.md:132-135` |
| Watches | Resource watch (store, runtime, pagination, types) | `openviking/resource/watch_{manager,runtime,pagination,storage,store,store_file,store_mysql,types}.py`, `service/resource_service.py`, `storage/resource_diff.py` |
| Colas | QueueFS persistente at-least-once | `openviking/storage/queuefs/` (`add_resource_processor.py`, `semantic_processor.py`, …) |
| Gateway | Management API, capture | `openviking_gateway/management.py:191`, `app.py:127-176,292-378`, `config.py:102-104,162-168` |
| Seguridad | Auth plugins, SSRF guard, path/zip safety | `openviking/server/auth/plugins/{dev,trusted}.py`, `openviking/utils/network_guard.py:123-135`, `openviking/utils/path_safety.py`, `openviking/utils/zip_safe.py`, `openviking/server/routers/webdav.py:45-61,162-171` |
| UI | Studio MemoryImpact | `web-studio/src/features/workbench/sessions/components/memory-impact.tsx` |
| Docs | Helper (agent integration) | `docs/en/agent-integrations/14-openviking-helper.md` |
| Benchmarks | Harnesses | `benchmark/README.md`, `benchmark/locomo/*`, `benchmark/tau2/*`, `benchmark/RAG/*` (`pipeline.py:187-202`) |

### 0.6 Licencia y provenance

- **[Hecho]** La raíz del repo es **AGPL-3.0**: `LICENSE` y headers `SPDX-License-Identifier: AGPL-3.0` muestreados en `openviking/*.py` (p.ej. `hierarchical_retriever.py:2`, `index_consistency.py:2`, `scope_filter.py:2`). El cambio de licencia a AGPL se introdujo en el commit `ce998873f` (historial).
- **[Hecho]** `crates/` (RAGFS, `ov_cli`) declara **Apache-2.0** — licencias mixtas por subárbol.
- **[Hipótesis]** El inventario por archivo está **incompleto**: solo hubo muestreo, no barrido SPDX completo. Antes de cualquier reutilización directa/derivada hay que completarlo.
- **Regla:** reescribir a otro lenguaje **no** exime de la licencia — una traducción/port sustancial puede ser obra derivada. Estudiar ideas y reimplementarlas limpiamente es distinto de trasladar código; el gate legal es obligatorio para cualquier reutilización directa o derivada.
- **No se filtró la investigación por licencia** (registrar todo lo relevante); sí se registra provenance por hallazgo.

### 0.7 Auditoría de seguridad estática de OpenViking

Alcance declarado por el subagente: lectura estática únicamente; sin ejecución, sin secrets; cobertura parcial (~123 ficheros con egress HTTP, 40+ `unsafe` en `crates/ragfs`, muestreados). Sin exploits dinámicos.

| ID | Severidad | Hallazgo | Estado |
|---|---|---|---|
| H1 | High | Gateway management API con escritura sin autenticar si `management_key` vacío: `management.py:191` `can_write = manager or not configured`, con `management_key=""` por defecto (`config.py:104`), `management_api=True` (`config.py:102`) y docker-compose exportando default vacío → ruta de código configurada por la que cualquier caller podría escribir upstreams/profiles en config por defecto. Solo existe warning de arranque; la validación de longitud no rechaza `""`. | **[Hecho]** ruta configurada confirmada estáticamente; no explotada dinámicamente |
| H2 | Medium | CORS wildcard + credentials + métodos/headers wildcard (`server/config.py:404`, `app.py:655-662`): postura condicional — el wildcard no otorga la API key al atacante ni demuestra lectura cross-origin autenticada; el riesgo depende de que la key sea conocida/adivinable y del despliegue. Sin validación de navegador ni runtime. | **[Hecho]** default débil; sin validación browser/runtime |
| H3 | Medium | SSRF guard con ventana TOCTOU DNS-rebinding: valida IPs resueltas antes del fetch (`network_guard.py:128-135`) pero httpx re-resuelve al conectar; además allowlist de code-hosting salta la verificación de IP (`:123-126`). | **[Hipótesis fuerte]** |
| H4/H5 | Medium | Riesgo FFI nativo: confianza en punteros del provider dlopen; revisión parcial (`mem::zeroed` muestreado, no exhaustivo). | **[Hipótesis]** revisión parcial |
| H6 | Medium | `encryption_enabled=False`, `api_key_hashing_enabled=False` por defecto (`server/config.py:407-408`); `derive_seeded_api_key_secret` genera secretos deterministas `sha256(user_id‖seed)` (`openviking/server/api_keys/key_mgr.py:51-54`). | **[Hecho]** defaults |
| H7 | Low-Medium | Identidad de tenant por headers `X-OpenViking-Account/User` spoofeables en dev/trusted si el proxy no strippea (riesgo operativo; trusted mode mitiga con root key + validación header↔URL). | **[Inferencia]** depende del despliegue |
| H8 | Low/Info | Supply chain: nativos vendored sin provenance visible (`third_party/`: croaring, leveldb-1.23, rapidjson, spdlog-1.14.1, krl) + `Cargo.lock` ~647 entradas + `setup.py` descarga binarios prebuilt. La licencia AGPL-3.0 se trata en §0.6 como cuestión legal, no como vulnerabilidad. | **[Hecho]** sin auditoría completa |

**Observaciones positivas [Hecho]:** sanitización de paths exhaustiva (`path_safety.py`), confinamiento WebDAV (`Destination` al prefijo), anti Zip-Slip + salto de symlinks + verificación post-extracción (`zip_safe.py`, `git_accessor.py:692-713`), fail-closed en bind no-loopback del plugin dev (`dev.py:46-64`, `sys.exit(1)`), `hmac.compare_digest` en trusted, SSRF con hooks por request que revalidan redirects.

**Recomendaciones de la auditoría:** (1) gateway `management_key` obligatorio o `management_api=false` por defecto; (2) patrón adoptable para VantaDB — gate "dev-mode solo si bind loopback" (ver §9: resultó ser **falso positivo** en VantaDB, ya existía); (3) SSRF con pinning de IP en conexión o resolver inyectado, eliminar exención allowlist; (4) hashing de keys ON por defecto + redacción de campos secretos en dumps; (5) formalizar `// SAFETY:` y contract-testing del ABI, `cargo audit`/`deny` en CI.

**Cobertura no revisada:** `openviking_gateway` completo (lua, capture, protocols), `crates/ragfs` multibackend/shell, parsers individuales, frontend/web-studio, totalidad de `unsafe` de `provider.rs`. No se ejecutó `cargo audit`, semgrep ni tests.

### 0.8 Capacidades existentes de VantaDB — no duplicar

Verificado contra el repo; cualquier propuesta debe tratar esto como base, no como carencia:

- **Retrieval híbrido:** BM25 + HNSW + RRF (con variantes de búsqueda y MMR en `src/sdk/search/`).
- **Provenance, sesiones, supersession:** dedup L1 con juicio LLM opcional `store|update|merge|skip` + `contradicts` → supersession, degradando a `store` sin LLM ([`vanta-memory/src/core/record/l1_dedup.rs`](../../vanta-memory/src/core/record/l1_dedup.rs)).
- **Ensamblado de contexto existente:** [`vantadb-mcp/src/context.rs`](../../vantadb-mcp/src/context.rs) (recall + compaction + presupuesto compartido) y [`vanta-memory/src/context_engine/engine.rs`](../../vanta-memory/src/context_engine/engine.rs) — **extender, no crear un assembler nuevo**.
- **Auditoría de índice existente:** audit/rebuild/repair + drift del text index en [`src/cli_handlers/index.rs`](../../src/cli_handlers/index.rs).
- **Auth:** gate de arranque que rechaza bind no-loopback sin auth (`validate_auth_config`, [`src/server/bootstrap.rs:41-87`](../../src/server/bootstrap.rs)) — el hallazgo inicial "falta gate loopback" fue **falso positivo** corregido por la revisión adversarial.
- **Ingestors wiki con claves deterministas** para reingesta idempotente ([`src/wiki/ingestors.rs`](../../src/wiki/ingestors.rs)).
- **Desktop:** worktree `develop-desktop` con VantaDB Studio (Tauri v2, Rust/React/Vite): memoria, conexiones, skills, escenas, actividad, diagnóstico de retrieval — ya existe; no proponer otra consola.
- **Benchmarks propios ya medidos:** existen resultados previos de la serie Paso 23 (`benchmarks/`, `vantadb-python/tests/test_step23_demos.py`, [`docs/user/operations/BENCHMARKS.md`](../user/operations/BENCHMARKS.md)) — queda **descartada** la vieja afirmación "VantaDB no tiene benchmarks medidos". Lo que está en cuestión es la **certificación LoCoMo** (§9-C), que es distinto.

---
## 1. Recomendaciones para VantaDB

Ordenadas por prioridad consolidada (todas son propuestas; ninguna imita código AGPL):

1. **Corregir garantías propias antes de añadir capacidades** — atomicidad de reemplazo L1, RMW del checkpoint global y blindaje del harness LoCoMo (§4). Sin esto, cualquier capa nueva hereda pérdida silenciosa.
2. **Adoptar el patrón de contexto navegable, no el retriever "jerárquico"**: el diferencial de OV es storage/indexación (identidad URI→niveles materializados L0/L1/L2 + lectura progresiva), con retrieval *flat* filtrado por nivel. Reescritura corregida por la revisión adversarial (ficha F1/F2).
3. **Extender los sistemas existentes, nunca duplicarlos**: `context_assemble`/`context_engine`, `cli_handlers/index.rs` (audit), scheduler de jobs, `l1_dedup` post-decisión. Cada ficha §2 declara su punto de anclaje explícito.
4. **Scope por subárbol con depth como evidencia para MEMG-25** (query-scope, **no** authz): prefijo canónico + filtro por profundidad en consulta.
5. **Ledger de contexto entregado por sesión+revisión**: distinguir seleccionado/renderizado/entregado, con confirmación idempotente (ficha F4); prerequisito: carrera de checkpoint.
6. **Merge declarativo post-`DedupDecision` (opcional, no Required)**: operador de aplicación determinista y auditable sobre campos tipados — OV decide *cómo* fusionar declarativamente, Vanta decide *si* fusionar con LLM; son complementarios (ficha F5).
7. **Extender la auditoría de índice al eje registro-canónico ↔ vector ↔ derivado por revisión** (ficha F6), no presentarla como ausente.
8. **Pipeline de ingesta observable con estados explícitos** (ficha F7): `ACK capture ≠ ready`.
9. **Evaluar experiencias estructuradas y watches incrementales** (fichas F8/F9) solo después de las garantías base; conectan con reflection/skills/scenes/dreaming e ingestors existentes.
10. **Onboarding/inspector de evidencia en VantaDB Studio existente** (ficha F10): timeline captura→extracción→recall→inyección; instalado ≠ funcionando.
11. **Disciplina de benchmark antes de claims**: dataset real explícito, backend real obligatorio, CSV por pregunta, todo pineado (§7). Ningún número publicado por OV está reproducido.
12. **Nada de LLM obligatorio en el core ni en hot path**: OV encadena hasta ~4 llamadas de modelo por ciclo escritura+query (intent planning, generación L0/L1, extracción, rerank); eso rompería el presupuesto de latencia de VantaDB. Principio ya documentado en `l1_dedup.rs` ("el LLM es opcional, el pipeline nunca pierde datos") — mantenerlo.
13. **Escritorio**: extraer *ideas de UX* (detección de integraciones con consentimiento, inspector de evidencia, sync selectivo) para Studio existente; no construir un Helper propio ni atribuir a OV funciones que ya tiene VantaDB Studio.

---

## 2. Fichas de extracción y reimplementación en Rust

> Todas las fichas son **propuestas de diseño** (status `proposed`), con DTO/contratos **hipotéticos explícitos** — no describen APIs existentes de VantaDB. Origen: diseño consolidado del informe final; lecturas de código sobre `b8e8895`. Requieren spec propia (`/spec`) antes de entrar al backlog.

### Ficha F1 — Identidad lógica de contexto y navegación por subárbol

- **Objetivo:** identidad lógica de contexto estable e independiente del path físico, con navegación paginada por hijos/ancestros y filtrado por subárbol+profundidad.
- **Lectura de código (OV):** `core/namespace.py:187-253` (scopes `resources`/`agent`/`user/{id}`/`session`/`temp`, `default_target_directories`), `storage/scope_filter.py:36-68` (AST `PathScope("uri", target, depth)` inyectado como `req.dsl`), `viking_vector_index_backend.py:2007` (`visible_roots(ctx)` ACL), `retrieve/hierarchical_retriever.py:204-223`.
- **Anclaje en VantaDB:** namespaces planos actuales; registro de memoria/contexto existente; `FilterBitset`/filtros del motor; ingestors con claves deterministas.
- **Contrato propuesto (hipotético):** `ContextId { source, parent, revision, kind, scope }`; `resolve(context_id) -> record existente` sin duplicar payload; `children(id, page)`, `ancestors(id)`; canonicalización de path + metadatos de subtree/depth en el registro para filtro pushdown.
- **Pasos:** (1) definir esquema de identidad y canonicalización; (2) persistir `parent/scope/depth` en registros; (3) índice de prefijo para scope-filter; (4) resolución id↔registro sin copia; (5) paginación estable.
- **Dependencias:** esquema de registro; filtros existentes; spec MEMG-25.
- **FMEA:** colisión de identidad entre tenants → id incluye scope; escape de subárbol por path no canónico → canonicalización obligatoria en escritura y consulta; rename que rompe provenance → identidad separada del path (el rename actualiza `parent`, no el id); profundidad inconsistente tras mover subárbol → recálculo o derivación al vuelo.
- **Tests de aceptación:** colisión entre tenants, intento de escape por `..`/encoding, límites de profundidad, rename preserva provenance, paginación estable bajo inserciones concurrentes.
- **Observabilidad:** métricas de resolución id→registro (hit/miss), cardinalidad de scope, profundidad máxima registrada.
- **Non-goals:** scope **no** es autenticación ni enforcement de permisos (eso queda en middleware); no cambia el modelo físico de almacenamiento.
- **Enlace backlog:** MEMG-25 (spec de scopes), FIND-320 (trust/scope boundary) — como evidencia de diseño, no fila nueva.

### Ficha F2 — Derivados versionados abstract/overview/source

- **Objetivo:** materializar por unidad de agrupación derivados L0-abstract/L1-overview/L2-source **versionados** por revisión de fuente y versión de generador, con generación asíncrona e invalidación.
- **Lectura de código (OV):** `storage/queuefs/semantic_processor.py:1560-1724` (generación bottom-up; L0 = primera línea del L1 `:1560-1589`), `core/context.py:34-39` (`ContextLevel`), `storage/index_consistency.py:25-41` (`IndexExpectation.level` — niveles como registros separados del mismo índice).
- **Anclaje en VantaDB:** subsistemas existentes de wiki/scenes/jobs para generación asíncrona; registros en partición interna + `FilterBitset` por nivel (propuesta del dictamen); **no** replicar en lifecycle L0–L3 cognitivo (mapeo erróneo).
- **Contrato propuesto (hipotético):** `Derivative { context_id, level: Abstract|Overview|Source, source_revision, generator_version, state: Pending|Ready|Stale|Failed }`; publicación por CAS sobre `(context_id, level, source_revision)` que impide que una generación vieja sobrescriba una nueva.
- **Pasos:** (1) esquema derivado + revisiones; (2) job async de generación con invalidación por nueva revisión; (3) publicación CAS; (4) indexación del derivado con `level`; (5) lectura progresiva en consulta (fuente→overview→abstract según necesidad).
- **Dependencias:** **escrituras L1 seguras como prerequisito** antes de cualquier borrado en cascada de derivados (§4-A); jobs existentes; ficha F1.
- **FMEA:** job lento deja derivados stale → marcar `Stale` por `source_revision`, nunca borrar fuente; actualización durante generación → descartar resultado viejo en CAS; fallo de generador → `Failed` reintentable, fuente intacta; borrado en cascada con L1 no atómico → prohibido hasta prerequisito.
- **Tests de aceptación:** generación concurrente de dos revisiones (gana la nueva); cancelación a mitad; fuente nunca desaparece por fallo de derivado; invalidación correcta tras update; derivado huérfano detectable por auditoría (F6).
- **Observabilidad:** estados por nivel, lag fuente→derivado listo, ratio stale, fallos por generador/versión.
- **Non-goals:** no es compresión ni summarización en hot path; el generador (LLM o no) es intercambiable; no es cascade de retrieval.
- **Enlace backlog:** derivado de la revisión adversarial — "adoptar esquema URI→niveles materializados", no `HierarchicalRetriever`.

### Ficha F3 — Presupuesto de contexto explícito (extensión del assembler existente)

- **Objetivo:** dotar al ensamblado de contexto **existente** de presupuesto total explícito, reserva de respuesta, cuotas por categoría, redistribución de remanente y contabilidad de descartes — **no** crear un assembler nuevo.
- **Lectura de código (OV):** `retrieve/context_assembler/` completo: `pipeline.py`, `gather.py`, `budget.py`, `tiers.py`, `params.py`, `render.py`, `models.py`, `expansion.py`, `recall_preset.py`, `rewrite.py`, `ledger.py`. Leer pipeline→gather→budget→tiers→params→render como flujo, sin portarlo mecánicamente.
- **Anclaje en VantaDB:** [`vantadb-mcp/src/context.rs`](../../vantadb-mcp/src/context.rs) (`context_assemble`: recall + compaction + presupuesto compartido) y [`vanta-memory/src/context_engine/engine.rs`](../../vanta-memory/src/context_engine/engine.rs) (`assemble_with_recall` o equivalente — verificar nombre actual en worktree).
- **Contrato propuesto (hipotético):** `ContextBudget { total, reserved_response, category_quotas: Map<Category, usize> }`; resultado `AssembledContext { selected: Vec<Item>, discarded: Vec<(Item, DiscardReason)>, per_category_count }`; estimator de tokens **declarado** (nunca afirmar tokenización exacta); tier `explicit|detail`.
- **Pasos:** (1) leer pipeline OV como referencia; (2) mapear `context.rs`/`engine.rs` actuales; (3) añadir presupuesto/cuotas/redistribución al flujo existente; (4) registrar descartes con razón; (5) citations/wrappers consistentes.
- **Dependencias:** estimator declarado; esquema de ítems con categoría y nivel (F1/F2 ayudan); ninguna dependencia de OV en runtime.
- **FMEA:** presupuesto diminuto → comportamiento definido con mínimos por categoría y degradación documentada; empates de score → desempate estable y determinista; Unicode/multibyte en estimación → medir en el estimador declarado, no en bytes; ítems superseded o sin permiso → excluidos antes de presupuestar; port mecánico del pipeline → prohibido, el diseño se adapta al seam existente; fuente→overview→abstract como orden de consumo del mismo contexto.
- **Tests de aceptación:** presupuesto exacto, presupuesto mínimo degenerado, redistribución de remanente documentada, empates estables entre runs, exclusión por auth/supersession previa al presupuesto, descartes con razón y conteo, estimator declarado en salida.
- **Observabilidad:** tokens estimados vs reservados, % de presupuesto usado por categoría, conteo de descartes por razón, items por tier.
- **Non-goals:** no es compresión de prompts ni reranking; no cambia la API de recall existente; no garantiza tokens exactos (estimador declarado).
- **Enlace backlog:** extensión de `context_assemble` existente; relacionado con trabajo de `context_engine` en curso en el worktree — **revalidar solapamiento**.

### Ficha F4 — Ledger de contexto entregado por sesión y revisión

- **Objetivo:** registrar qué contexto (id+revisión) se seleccionó, renderizó y **entregó** por sesión y ventana de turnos, con confirmación idempotente — evita reinyectar lo ya entregado y permite auditar la inyección.
- **Lectura de código (OV):** `retrieve/context_assembler/ledger.py` (`_record_turn`, `_resolve_turn`), `session/session.py:3270-3286` (contexto para search), `session/session.py:1489-1498`.
- **Anclaje en VantaDB:** `CheckpointManager` ([`vanta-memory/src/utils/checkpoint.rs`](../../vanta-memory/src/utils/checkpoint.rs)) y sesiones existentes; el ledger se apoya en el mecanismo de checkpoint pero requiere **CAS atómico** que hoy no tiene (§4-B).
- **Contrato propuesto (hipotético):** `DeliveryLedger { session_id, entries: [{context_id, revision, state: Selected|Rendered|Delivered, turn}] }`; update atómico/CAS; `confirm_delivery(entry) idempotente`; ventana de turnos configurable.
- **Pasos:** (1) resolver la carrera RMW del checkpoint (§4-B, **prerequisito duro**); (2) esquema de entradas por sesión+context_id+revisión; (3) transiciones de estado atómicas; (4) consulta "¿qué se entregó ya en esta ventana?"; (5) integración con el assembler (F3).
- **Dependencias:** §4-B (checkpoint CAS/fusión bajo lock); F1 (context_id+revision); F3 (puntos de selección/render).
- **FMEA:** cancelación antes de entrega → **no** suprime (estado queda Selected/Rendered, no Delivered); dos llamadas concurrentes → ambas preservadas por CAS, no lost-update; nueva revisión del mismo contexto → no suprimida por entrega previa de la vieja; fallo a mitad de update → idempotencia en retry; ventana mal dimensionada → reinjección excesiva o supresión excesiva (parámetro observable).
- **Tests de aceptación:** dos recalls concurrentes preservan ambas entregas; cancel-before-delivery no suprime; nueva revisión no suprimida; confirmación repetida es idempotente; ventana de turnos respeta límite; restart no pierde el ledger.
- **Observabilidad:** entregas por sesión, ratio supresión, conteo por estado, edad del ledger.
- **Non-goals:** no es dedup semántico ni política de calidad; no persiste contenido, solo referencias id+revisión.
- **Enlace backlog:** issue upstream [#7170](https://github.com/volcengine/OpenViking/issues/7170) como lección (§3); prerequisito §4-B.

### Ficha F5 — Merge declarativo post-`DedupDecision`

- **Objetivo:** operador declarativo y determinista para **aplicar** la fusión una vez que `l1_dedup` ya decidió `update|merge` — nunca sustituir la decisión LLM.
- **Lectura de código (OV):** `session/memory/merge_op/`: `base.py` (registry ABC), `factory.py`, `patch.py`, `patch_handler.py`, `replace.py`, `sum.py`, `link_merge.py`, `immutable.py`; plantillas por tipo en `system_assets/memory/memory_templates/builtin/*.yaml`.
- **Anclaje en VantaDB:** [`l1_dedup.rs`](../../vanta-memory/src/core/record/l1_dedup.rs) — pipeline de dos fases (keyword recall + LLM judge) que hoy aplica `update/merge` sin álgebra declarativa; el operador se inserta **después** de `DedupDecision`.
- **Contrato propuesto (hipotético):** enum cerrado `MergeOp = Patch | Replace | Sum | Immutable | Link`, permitido por esquema del registro; campos tipados; precedencia explícita `(authority, revision, policy, tie_estable)`; rechazo de mutaciones desconocidas/incompatibles/protegidas; guard anti-replay para `Sum`.
- **Pasos:** (1) definir enum y esquema por tipo de memoria; (2) handlers deterministas por op sobre campos tipados; (3) resolver precedencia y rechazo; (4) replay guard para `Sum`; (5) integrar tras la decisión LLM en `apply_dedup_batch` o equivalente.
- **Dependencias:** **reemplazo L1 seguro como prerequisito** (§4-A) — un merge que elimina antes de reemplazar hereda la pérdida; contrato de campos tipados por tipo.
- **FMEA:** op desconocida o incompatible con el esquema → rechazo explícito, nunca aplicación a ciegas; replay de `Sum` → guard por idempotency key; mutación de campo protegido → rechazo; precedencia ambigua → regla declarada + desempate estable; merge sobre registro eliminado entre decisión y aplicación → recheck bajo la misma guard.
- **Tests de aceptación:** replay de `Sum` no duplica; conflictos de precedencia resueltos por la regla declarada; campos protegidos intactos; tipos incompatibles rechazados; decisión LLM intacta (el operador solo aplica).
- **Observabilidad:** merges por op, rechazos por razón, replays detectados, merges aplicados vs decididos.
- **Non-goals:** no decide *si* fusionar (eso es el LLM/judge); no introduce plantillas YAML en VantaDB salvo que la spec lo justifique; no es dedup semántico.
- **Enlace backlog:** reformulación adversarial del "sistema de dedup" (ya existente); Optional, no Required.

### Ficha F6 — Auditoría de consistencia registro ↔ vector ↔ derivado

- **Objetivo:** extender el checker existente al eje **registro canónico ↔ vector ↔ revisión de derivado**: detectar missing/orphan/stale/invalid por revisión.
- **Lectura de código (OV):** `storage/index_consistency.py:14-50` — job que reconstruye expectativas desde storage canónico y verifica `Eq(uri)` por nivel en el índice.
- **Anclaje en VantaDB:** [`src/cli_handlers/index.rs`](../../src/cli_handlers/index.rs) — `cmd_audit_index`, drift de text index, `cmd_repair_text_index`. El checker actual audita postings/BM25; el eje propuesto es contenido↔vector por revisión, que **no es el mismo** (delimitación de la revisión adversarial).
- **Contrato propuesto (hipotético):** `ConsistencyExpectation { record_id, revision, expected_derivatives }`; `AuditReport { missing, orphan, stale_revision, invalid }`; diagnóstico separado de reparación; jobs de reparación idempotentes guardados por `current_revision`.
- **Pasos:** (1) mapear comandos actuales; (2) definir expectativa por registro+revisión; (3) auditoría diagnóstico-only; (4) reparación como job separado con guard de revisión; (5) tratar **fallo de lectura ≠ ausencia** (lección [#7132](https://github.com/volcengine/OpenViking/issues/7132)).
- **Dependencias:** F2 (revisiones de derivados); jobs existentes; §4 si la reparación escribe L1.
- **FMEA:** reparación concurrente con publicación → guard de `current_revision` (nunca reparar hacia una revisión vieja); cancelación a mitad de job → idempotente/reanudable; paginación grande → job por páginas con progreso persistido; lectura fallida reportada como ausente → error categorizado, no "missing".
- **Tests de aceptación:** publicación concurrente durante auditoría; cancelación y reanudación por página; error de lectura clasificado distinto de ausencia; reparación no retrocede revisiones.
- **Observabilidad:** conteos por categoría (missing/orphan/stale/invalid), duración por página, reparaciones aplicadas/omitidas por guard.
- **Non-goals:** no es rebuild completo; no es GC de vectores; diagnóstico nunca muta.
- **Enlace backlog:** reformulación del "checker de índice" (parcialmente existente).

### Ficha F7 — Pipeline de ingesta con estados observables

- **Objetivo:** hacer observable el scheduler/ingesta existente con estados explícitos `Captured → ExtractionQueued → Extracted → Indexing → Ready` más `Failed | Retryable | Cancelled`, revisiones, timestamps e idempotencia.
- **Lectura de código (OV):** QueueFS = colas persistentes at-least-once (`storage/queuefs/`: `add_resource_msg.py`, `add_resource_processor.py`, `semantic_processor.py`, colas embedding/reindex/session_commit/external_task); jobs con ACK de captura.
- **Anclaje en VantaDB:** scheduler/jobs existentes + ingestors wiki de claves deterministas ([`src/wiki/ingestors.rs`](../../src/wiki/ingestors.rs)).
- **Contrato propuesto (hipotético):** `IngestItem { key, revision, state, timestamps por transición, idempotency_key, error?, partial_result? }`; máquina de estados explícita; **ACK de captura ≠ ready**; reintentos acotados; "lock/deferral" ≠ "ejecución fallida".
- **Pasos:** (1) inventariar transiciones actuales del scheduler; (2) modelo de estados + persistencia; (3) reintentos acotados con clasificación retryable/failed; (4) reanudación en **todos** los límites tras restart; (5) superficie de consulta por estado.
- **Dependencias:** claves deterministas existentes; persistencia de jobs.
- **FMEA:** at-least-once duplica trabajo → handlers idempotentes por `idempotency_key`; lock deferral reportado como fallo → estado `Retryable` separado; restart a mitad → reanudar en cada frontera Captured/Queued/Extracted/Indexing; retry infinito → tope explícito (lección [#7034](https://github.com/volcengine/OpenViking/issues/7034); VantaDB ya limita — mantenerlo).
- **Tests de aceptación:** restart en cada frontera reanuda correctamente; duplicado por reintento no duplica efecto; deferral por lock no marca Failed; tope de reintentos respeta límite; estado consultable en cualquier instante.
- **Observabilidad:** items por estado, latencia por transición, reintentos por clase, backlog de cola, edad del item más viejo.
- **Non-goals:** no cambia la semántica de las colas; no añade orquestador externo; no promete exactly-once.
- **Enlace backlog:** lección upstream de colas; relacionado con watches (F9).

### Ficha F8 — Experiencias estructuradas del agente

- **Objetivo:** registro de experiencias (situación, objetivo, precondiciones, acciones, resultado, evidencia, fallo, expiración) que alimente reflexión, skills y escenas — conectando subsistemas existentes.
- **Lectura de código (OV):** `session/memory/extract_loop.py` (extracción ReAct, cap 32k tokens), skills persistentes y su procesamiento (`server/routers/skills.py`, `server/skill_ingest.py`, `utils/skill_processor.py`), dedup textual de session-skills (`session/skill/dedup.py:15-48`).
- **Anclaje en VantaDB:** reflection (FIND-290), skills, scenes/dreaming existentes; l1_dedup para consolidación.
- **Contrato propuesto (hipotético):** `Experience { situation, objective, preconditions, actions, outcome: Success|Failure|Partial, evidence_refs, failure_detail?, expiry? }`; `agent_claim ≠ verified_outcome`; política de promoción experiencia→skill; retrieval nunca ejecuta instrucciones recuperadas.
- **Pasos:** (1) esquema + ciclo de vida con expiración; (2) ingestión desde sesiones/escenas con evidencia enlazada; (3) política de promoción a skill con umbral declarado; (4) recuperación contextual; (5) barrido anti-secretos y anti-contaminación train/test.
- **Dependencias:** §4 y F4/F7 para evidencia confiable; reflection (FIND-290), hooks (FIND-295), FIND-288.
- **FMEA:** claim del agente tomado como verdad → `outcome` requiere `evidence_refs` verificables o queda `agent_claim`; experiencias fallidas descartadas → conservarlas (son útiles); instrucciones recuperadas ejecutadas automáticamente → prohibido por contrato (retrieval ≠ ejecución); secretos en experiencias → barrido previo a persistir; contaminación train/test → aislamiento por scope/expiry (riesgo real en benchmarks §7); promoción prematura a skill → política con evidencia mínima.
- **Tests de aceptación:** experiencia sin evidencia no marca outcome verificado; experiencia fallida recuperable; instrucción recuperada nunca se ejecuta; barrido de secretos bloquea persistencia; promoción exige evidencia declarada.
- **Observabilidad:** experiencias por outcome, ratio claim/verificado, promociones, hits de retrieval de experiencias.
- **Non-goals:** no es replay de episodios ni RL; no ejecuta acciones; no es journaling de sesión crudo.
- **Enlace backlog:** FIND-290 (reflection), FIND-288, FIND-295 (hooks).

### Ficha F9 — Watches incrementales de fuentes

- **Objetivo:** reingesta incremental por watches: manifest de revisiones, debounce, unidad de cambio + ancestros, exclusiones, permisos, rename/delete, versión de parser, conectores remotos.
- **Lectura de código (OV):** `resource/watch_{manager,runtime,pagination,storage,store,store_file,store_mysql,types}.py`, `service/resource_service.py`, `storage/resource_diff.py`, parsers por tipo; `routers/resources.py`.
- **Anclaje en VantaDB:** ingestors con claves deterministas ([`src/wiki/ingestors.rs`](../../src/wiki/ingestors.rs)) — la reingesta idempotente ya existe; lo nuevo es la detección incremental por watch.
- **Contrato propuesto (hipotético):** `Watch { root, excludes, debounce_ms, parser_version }`; `ChangeEvent { unit_id, kind: Upsert|Rename|Delete, ancestors, revision }`; eventos idempotentes; conector remoto separado del parsing (fetch ≠ parse).
- **Pasos:** (1) manifest de revisiones por unidad; (2) debounce + coalescing de eventos; (3) propagación a ancestros (F1); (4) rename/delete como eventos de primera clase; (5) `parser_version` en el manifest para reprocesar al cambiar parser; (6) conectores remotos con validación de fuente.
- **Dependencias:** F1 (ancestros), F7 (estados), FIND-299 (watches ya en backlog); §4 para escrituras seguras.
- **FMEA:** tormenta de eventos → debounce + límite; auto-ingest (escribir sobre lo que el propio watch genera) → exclusión de artefactos derivados; symlink fuera del root → rechazo (lección `zip_safe`/`path_safety`); SSRF en conectores remotos → guard equivalente al TOCTOU de §0.7-H3, sin allowlist sin verificación IP; evento para revisión ya inválida → descartar por `source_revision`; permisos de origen cambian → re-evaluar scope.
- **Tests de aceptación:** rename mantiene identidad+provenance; delete propaga; tormenta coalescida; evento idempotente ante reintento; parser nuevo dispara reproceso selectivo; symlink/SSRF rechazados.
- **Observabilidad:** eventos por tipo, debounce rate, lag watch→ready, reprocesos por parser_version.
- **Non-goals:** no es FUSE ni montaje; no es sync bidireccional; el fetch remoto no ejecuta parsers en el conector.
- **Enlace backlog:** FIND-299 (watches).

### Ficha F10 — Inspector de evidencia y onboarding en Studio existente

- **Objetivo:** en VantaDB Studio existente (worktree `develop-desktop`, Tauri v2): onboarding de integraciones con consentimiento + inspector del ciclo captura→extracción→recall→inyección con evidencia real.
- **Lectura de código (OV):** `web-studio/src/features/workbench/sessions/components/memory-impact.tsx` (MemoryImpact), `docs/en/agent-integrations/14-openviking-helper.md` (perfiles cloud/selfhost, detección de agentes, setup MCP/hooks/plugins, inspección de sesiones, inventario, sync selectivo, captura de evidencia + commit + recall + inject — **solo documentado, no verificado en runtime**; el fuente de Helper no está en el checkout).
- **Anclaje en VantaDB:** VantaDB Studio existente (conexiones, memoria, skills, escenas, actividad, diagnóstico de retrieval) — **extenderlo, no crear producto nuevo**.
- **Contrato propuesto (hipotético):** `IntegrationCandidate { kind, evidence, config_diff }` con detección → preview → consentimiento → aplicación sin sobrescritura; `TimelineEvent { stage: Capture|Extract|Recall|Inject, ref, ts }`; memory diff + provenance + presupuesto de descartes (F3); sync selectivo por categoría.
- **Pasos:** (1) inspector de timeline sobre datos reales (read-only primero); (2) memory diff/provenance en UI; (3) detección de integraciones con preview + consentimiento, nunca escritura silenciosa; (4) sync selectivo; (5) superficie de errores reales de transporte/capabilities.
- **Dependencias:** F4/F7 para evidencia del ciclo; Studio existente; API de lectura ya disponible.
- **FMEA:** "instalado ≠ funcionando" → toda capacidad mostrada requiere evidencia real o se marca no-verificada; detección que sobrescribe config del usuario → preview+consentimiento obligatorio; tokens en UI/transporte → nunca persistir secretos, transporte seguro, errores reales propagados; diff con datos parciales → indicar cobertura.
- **Tests de aceptación:** timeline muestra solo eventos con evidencia; detección no muta nada sin consentimiento; errores de transporte visibles; sync selectivo respeta exclusión.
- **Observabilidad:** (UI) eventos mostrados por stage, acciones de onboarding aceptadas/rechazadas.
- **Non-goals:** no es marketplace de integraciones; no es configurador remoto; primera iteración read-only.
- **Enlace backlog:** evaluación de escritorio §10.

---

## 3. Issues upstream aplicables a VantaDB

**Metodología de la muestra [Hecho]:** consulta `gh api` del 2026-11-06 (22:31–22:37 UTC), 3 páginas de `issues?state=all&per_page=100` → 300 objetos = **79 issues** (221 PR excluidos); búsquedas temáticas adicionales (`refresh stale`, `tenant isolation`, `Windows build`, `Windows|build in:title`) que añadieron #2015, #5873, #5613, #6552. Se leyeron cuerpo/comentarios de **15 candidatos**; las reproducciones citadas son de terceros, no ejecuciones propias.

**Semántica de estados:** «existente» = protección implementada en VantaDB; «pendiente» = deuda/riesgo equivalente identificado; «no demostrado» = evidencia insuficiente para afirmar el mismo mecanismo; «no aplicable» = contrato o mecanismo diferente. Ninguno equivale a certificación dinámica.

| Issue | Síntoma upstream | Transferencia a VantaDB |
|---|---|---|
| [#7170](https://github.com/volcengine/OpenViking/issues/7170) — concurrent recalls overwrite session dedup ledger | Abierto; dos recalls parten del mismo ledger, la última escritura pierde URIs (repro determinista publicada; PR #7175 abierto: releer+fusionar bajo lock) | Análogo directo del hallazgo §9-B: checkpoint global con RMW separado → **pendiente** (hipótesis de carrera, requiere test determinista) |
| [#7099](https://github.com/volcengine/OpenViking/issues/7099) — extracción elimina memorias raíz | Abierto; el fix upstream es sobre raíz de extracción — **mecanismo distinto** del L1 de VantaDB | La transferencia real es la unidad de atomicidad de reemplazo: delete-before-put en L1 → **pendiente** (§9-A), no copiar el fix de raíz |
| [#6923](https://github.com/volcengine/OpenViking/issues/6923) — knob clamped por default hard-coded (5 candidatos) | **Abierto con fix fusionado** (estado cerrado ≠ resuelto) | Equivalente concreto ya registrado: **FIND-313 (IQL, k=5)** — pendiente propio, no diagnóstico importado |
| [#7132](https://github.com/volcengine/OpenViking/issues/7132) — error de lectura interpretado como ausencia | Leer-fallo vs no-existe | En VantaDB las rutas persona/perfil examinadas **ya propagan errores** → no importar el diagnóstico; sí aplicar la regla en auditoría (F6): error de lectura ≠ ausencia |
| [#7134](https://github.com/volcengine/OpenViking/issues/7134) — falso éxito de backend | Confirmar escritura rechazada | **No demostrado** en VantaDB; guardar L1 sin vector ante fallo de embedding es degradación explícita, no falso éxito |
| [#7034](https://github.com/volcengine/OpenViking/issues/7034) — reintentos ilimitados | Retry infinito de jobs | VantaDB **ya tiene** intentos acotados y distingue "sesión ocupada" de "ejecución fallida" → existente; un retry exitoso puede satisfacer el contrato |
| [#5873](https://github.com/volcengine/OpenViking/issues/5873) — fuente actualizada, semántica/vectores obsoletos | Derivado stale tras update | Refuerza F2/F6: consistencia de derivados ante fallos intermedios → **pendiente** como deuda de diseño |
| [#6552](https://github.com/volcengine/OpenViking/issues/6552) — tempfile abierto en Windows | Limpieza de fichero abierto | **Sin análogo probado**; incorporar el escenario solo cuando exista una ruta homóloga |
| [#5613](https://github.com/volcengine/OpenViking/issues/5613) — expansión de scope | Posible fuga por scope | Diseño distinto; **no probada** la fuga; relevante para F1 (scope ≠ auth) |
| [#7162](https://github.com/volcengine/OpenViking/issues/7162) — índice local sparse se corrompe bajo escritura | Abierto; `get view out of bounds` persistente; PR #4991 abierto sin fusionar (evita insertar términos durante búsquedas — mecanismo relacionado, no prueba de causa) | **No demostrado**: VantaDB almacena postings por dimensiones numéricas y namespace (`src/sdk/serialization/impl_sparse_index.rs:48-58,137-148`); no se acreditó el mismo diccionario mutable en consultas |

**Cierre operativo de la auditoría de incidencias:** no transportar diagnósticos por similitud de síntomas ni marcar resuelto por estado cerrado — #6923 permanece abierto con fix fusionado; #7048/#6963 cerrados sin fix; #2015 cerrado con recurrencia reportada. Bases locales usadas por el subagente: OV `b8e8895`, VantaDB `f6ed51d` en ese momento.

---

## 4. Prerequisitos a corregir en VantaDB antes de extraer

Orden estricto — todo [Histórico], revalidar contra el worktree actual:

1. **Reemplazo L1 no indivisible** — rutas secuencial y batch eliminan registros antes de escribir los nuevos; [`l1_writer.rs:430-446`](../../vanta-memory/src/core/record/l1_writer.rs) documenta que un fallo intermedio deja eliminaciones aplicadas. Limitación confirmada **estáticamente**; sin reproducción dinámica de pérdida. Relacionar con **FIND-221, TXN-01, CRASH-01** sin confundir sus alcances (no es el mismo problema que el WAL de CRASH-01). Bloquea F2 (borrados en cascada) y F5 (merge = reemplazo).
2. **Checkpoint global susceptible de lost update** — `CheckpointManager` mantiene JSON global (`checkpoint.rs:79-84`) y `mutate()` separa lectura/modificación/escritura (`:111-116`) sin CAS ni guard compartido; el worker bloquea **por sesión** pero la clave del checkpoint es **global** → hipótesis de carrera que requiere prueba determinista. El comentario que promete atomicidad in-process no es garantía mecánica. Bloquea F4.
3. **Blindaje del harness LoCoMo** — `eval_locomo.py:826-856`: puede generar datos sintéticos por defecto y degradar a `FakeLocomoStore` fuera de `--self-test`; además el loader no normaliza el esquema original de conversaciones LoCoMo al de preguntas esperado. Resolver dentro de **MEM0-05** antes de usar LoCoMo como evidencia competitiva (§7).
4. **Recovery/atomicidad propios** — `CRASH-01` y `TXN-01` (`Backlog.md`): recovery abre el segmento por shard y reproduce ese reader (`src/storage/engine/init.rs:618-649`); commit aplica mutaciones antes del marcador durable `Commit` (`src/storage/engine/txn.rs:333-381`). Justifica priorizarlos; **no demuestra** pérdida tras crash por sí solo.
5. **FIND-313 (IQL k=5)** — equivalente propio del defecto #6923.

---

## 5. Lista de rechazo — qué no trasladar

| Rechazado | Razón |
|---|---|
| LLM obligatorio en el core o en hot path | OV encadena intent planning + generación L0/L1 + extracción + rerank (~4 llamadas por ciclo); incompatible con el presupuesto de latencia y con el principio documentado en `l1_dedup.rs` ("el LLM es opcional, el pipeline nunca pierde datos") |
| Sistemas duplicados (assembler, dedup, auditoría, scheduler) | Todo existe; las fichas extienden seams existentes. Duplicar = deuda inmediata |
| Migración total del producto o del modelo de servicio | OV es un *servicio* con dependencias VLM externas; VantaDB es motor embebido. Scope y modelo operativo incompatibles |
| Reescritura "source-to-source" como bypass de licencia | Traducir/portar no exime; la raíz es AGPL-3.0. Solo reimplementación limpia de diseño tras gate legal |
| Defaults permisivos (auth vacío, CORS `*`, hashing off) | H1/H2/H6 de §0.7 son exactamente los defaults a no copiar; VantaDB debe mantener fail-closed |
| Runners/scripts destructivos de OV (`--step all` → borrado recursivo, reinicio de servicios, restore de estado) | `benchmark/RAG/pipeline.py:187-202` termina en `db.clear` — **nunca** sobre datos compartidos; ninguna de estas mecánicas se adopta |
| Índice/ANN de OV | El índice local es `flat`/`flat_hybrid` (fuerza bruta); HNSW solo en backends remotos. Ninguna idea de índice es adoptable — VantaDB ya es superior |
| Retrieval "jerárquico" como feature | Ya verificado flat con filtro; la adopción es el esquema de materialización (F1/F2) |
| Dedup semántico de OV | No existe en OV (firma textual solo para skills; el LLM decide merges) — no hay nada que importar; el enfoque `l1_dedup` es más robusto |
| Bot/gateway/producto completo | VikingBot, gateway LLM y Studio son productos separados; fuera de scope de VantaDB |

---

## 6. Advertencias metodológicas

1. **Evidencia estática fechada, no dinámica:** ningún hallazgo de este documento fue verificado en ejecución. Las líneas citadas son del snapshot `b8e8895` (OV) y de la base indicada (VantaDB); el worktree actual difiere — **revalidar antes de actuar**.
2. **No transportar diagnósticos por similitud de síntomas:** un issue upstream con síntoma parecido no implica el mismo mecanismo (ej. #7162 sparse ≠ postings de VantaDB; #7099 raíz ≠ reemplazo L1).
3. **Estado cerrado ≠ resuelto / abierto ≠ roto:** #6923 abierto con fix fusionado; #7048/#6963 cerrados sin fix; #2015 cerrado con recurrencia.
4. **No confundir alcances de deuda propia:** L1 delete-before-put, checkpoint RMW, CRASH-01 (WAL/recovery) y TXN-01 son problemas distintos relacionados, no el mismo bug.
5. **`ACK de captura ≠ dato listo`:** en colas at-least-once la confirmación de recepción no certifica procesamiento; igual para "instalado ≠ funcionando" en integraciones/UI.
6. **Fallo de lectura ≠ ausencia:** clasificar errores por separado (lección #7132; regla adoptada en F6).
7. **Claims publicados ≠ resultados:** todas las cifras de benchmarks de OV son [Claim] no reproducida (§7).
8. **Muestreo no es inventario:** headers SPDX, issues leídos (15/79) y unsafe (muestreados) son muestras; las conclusiones se limitan a lo muestreado.
9. **Hipótesis de carrera ≠ bug confirmado:** checkpoint RMW y TOCTOU DNS son patrones presentes con intercalación plausible; requieren reproducción determinista antes de "fix".
10. **Degradación explícita ≠ falso éxito:** almacenar L1 sin vector ante fallo de embedding es comportamiento declarado; no equipararlo al defecto #7134.
11. **Historial vs actualidad:** el estado de LoCoMo/LongMemEval/NIAH en backlog puede haber evolucionado tras esta fecha; los enlaces a filas son históricos, no aserciones de incompletitud actual.

---

## 7. Benchmarks aplicables a VantaDB

> **Protocolo transcrito del informe final — propuestas metodológicas, no umbrales nuevos.** No se autorizan aquí rúbricas, prompts de judge, harnesses ni configuraciones de benchmark; todo requiere spec propia.

### 7.1 Separaciones obligatorias

- **Memoria y RAG son familias separadas** con pipelines, datasets y métricas distintas — no mezclar denominadores.
- **Cuatro métricas distintas que no deben fusionarse:** calidad de *retrieval*, respuestas *generadas*, scores *juzgados*, y *éxito de tarea*.
- Los pipelines LoCoMo de OV (**VikingBot, OpenClaw, Hermes, Claude Code, mem0-singleturn**) **no son idénticos**: ingestión, presupuestos, prompts de QA y de judge difieren entre ellos. Compararlos exige normalizar esas variables.

### 7.2 Claims publicados por OV — no reproducidos [Claim]

Fuente: `README.md` + `benchmark/README.md` + [blog.openviking.ai/post/openviking-benchmark-results](https://blog.openviking.ai/post/openviking-benchmark-results/).

| Familia | Claim publicado |
|---|---|
| LoCoMo accuracy | OpenClaw 24.20 → 82.08; Hermes 33.38 → 82.86; Claude Code 57.21 → 80.32 (con OpenViking); input tokens −34.3…−91.0 %, latencia −58.45…−66.10 % |
| tau2-bench task success | Airline +11.87 pp, Retail +6.87 pp (Retail 70.94→77.81; Airline 54.38→66.25) |
| HotpotQA | 91 (top-20), latencia 0.23 s |
| RAG | avg 66.87, latencia 0.19 s |

**Ninguna cifra fue reproducida ni verificada localmente.**

### 7.3 Defectos metodológicos detectados en el harness de OV [Hecho estático]

- **Judge permisivo mezclado con estricto:** criterios como partial-lists, `date14d`, `duration50%` deben reportarse separados del judge estricto; fusionarlos infla accuracy.
- **Tool disabling en QA:** deshabilitar herramientas del agente cambia la tarea medida; declararlo por configuración.
- **El question-index diagnóstico puede usar evidencia groundtruth** — es diagnóstico, **no** una evaluación end-to-end válida.
- **Errores de API/parsing del judge separados de corrección:** un fallo del judge no es una respuesta incorrecta ni una correcta; contarlos aparte.
- **Preguntas RAG fallidas no pueden salir del denominador silenciosamente** — el denominador declarado es obligatorio.
- **`--step all` de `benchmark/RAG` termina en borrado recursivo** (`pipeline.py:187-202`, `db.clear`) → **NUNCA sobre datos compartidos**.
- **Algunos scripts reinician servicios o restauran estado** — aislarlos de cualquier entorno compartido.

### 7.4 Requisitos de reproducibilidad (aplicables a cualquier corrida propuesta)

Pin de: commits, datasets, agente, modelos, prompts, presupuestos, configs y concurrencia. Datos/cuenta **aislados y desechables**. Smoke inicial: **una** muestra/pregunta solo como prueba de flujo. Registro por pregunta en CSV: attempted/completed/graded/error/abstention, costos por fase, y **latencia de retrieval vs latencia total**. **Ninguna afirmación sobre modelos a partir de la mera existencia del harness.**

### 7.5 Estado del harness propio de VantaDB [Histórico]

- [`evals/eval_locomo.py:826-856`](../../evals/eval_locomo.py): genera datos sintéticos por defecto y degrada a `FakeLocomoStore` fuera de `--self-test`; el loader no normaliza el esquema original de conversaciones LoCoMo al esquema de preguntas esperado → usar LoCoMo como **certificación** está bloqueado por esto (filas históricas MEM0-05).
- [`evals/eval_longmemeval.py`](../../evals/eval_longmemeval.py): variantes `vector_only|bm25_only|hybrid_rrf|hybrid_graphrag`, gate declarado `recall_all@5 >= 0.85` — describe el harness, **no demuestra resultados**.
- [`evals/memory_harness.py`](../../evals/memory_harness.py): proxies de abstención, economía de tokens y fidelidad de escritura — no evalúan respuestas generadas/juzgadas por LLM.
- Existen resultados medidos previos de la serie Paso 23 (`benchmarks/`, `docs/user/operations/BENCHMARKS.md`) — la afirmación antigua "sin benchmarks medidos" queda descartada; LoCoMo-certificación es lo pendiente.

### 7.6 Escenarios propuestos para VantaDB (metodología, sin umbrales inventados)

- **MEM0-05 (hardening LoCoMo):** dataset real y explícito; adaptador del esquema original; backend real obligatorio para certificación — nada de `Fake*` salvo en self-test explícito; sintético solo declarado como sintético.
- **Pista A (retrieval puro):** recall/MRR/nDCG donde estén justificados, sin LLM en la medición.
- **Pista B (memoria end-to-end):** sesiones → extracción → recall → respuesta → judge, con presupuestos de generación **idénticos** entre variantes y **sin fuga de evidencia** (evidence/groundtruth no entra al prompt del sistema evaluado).
- **EVAL-02:** calibración del judge sobre holdout.
- **ACT-02:** corrida LongMemEval de ~500 preguntas; umbral declarado ≠ prueba de superación.
- **tau2 (adaptado):** variante "solo memoria": baseline sin-memoria vs con-memoria con **mismo agente, herramientas, modelo, seeds y presupuestos**; experiencias solo de entrenamiento, test congelado, sin persistencia de test, repeticiones por tarea reportadas, simulador modificado declarado.
- **RAG:** empezar por retrieval sobre Qasper/FinanceBench/SyllabusQA; después QA-sobre-recuperado y QA-oracle como diagnóstico separado; ablación flat actual vs summaries/progressive/rerank.
- **Escenarios adicionales propuestos:** presupuestos y cobertura factual, reinyección, revisiones, consistencia, crash, scopes, contenido recuperado malicioso — **sin umbrales fabricados**.
- **Self-tests existentes:** `python evals/memory_harness.py --self-test`, `python evals/eval_longmemeval.py --self-test`, `python evals/eval_locomo.py --self-test` — **SOLO MANUAL, no son evidencia de calidad** (prueban el arnés offline, no certifican nada).

---

## 8. Matriz de brechas de capacidades

Capacidades OV verificadas en código vs estado VantaDB (fecha de la investigación). "Existente" no certifica paridad de calidad.

| Capacidad OV | Estado VantaDB | Disposición |
|---|---|---|
| Retrieval híbrido BM25+ANN+RRF | Existente (BM25/HNSW/RRF) | Mantener — superior |
| Provenance, sesiones, supersession | Existente (`l1_dedup`, supersession) | Mantener |
| Filesystem virtual `viking://` navegable (ls/tree/grep) | Ausente como modelo | Evaluar F1 (identidad+subárbol) |
| Resúmenes materializados L0/L1/L2 por unidad | Ausente como entidades indexables por nivel (L0–L3 de Vanta es lifecycle, no documentos por URI) | F2 (derivados versionados) |
| Scope por subárbol+depth en consulta | Parcial (namespaces planos; MEMG-25 en backlog) | F1 |
| Presupuesto de contexto con cuotas/tiers | Parcial (context_assemble con presupuesto compartido) | F3 extensión |
| Ledger de contexto entregado por sesión | Ausente | F4 (tras prerequisito §4-B) |
| Merge declarativo post-decisión | Parcial (decisión LLM sí; operador declarativo no) | F5 opcional |
| Verificador FS↔índice por nivel | Parcial (auditoría de text-index existe; eje contenido↔vector no confirmado) | F6 extensión |
| Pipeline de ingesta con estados observables | Parcial (jobs existentes, estados no explícitos) | F7 |
| Experiencias estructuradas | Parcial (reflection/skills/scenes existentes; no hay esquema unificado) | F8 |
| Watches incrementales | Ausente (FIND-299 en backlog) | F9 |
| Inspector/onboarding de integraciones | Parcial (Studio existe con diagnóstico; no hay onboarding ni timeline de evidencia) | F10 |
| Extracción de memoria por LLM ReAct | Parcial (dedup LLM existe; extracción ReAct completa no equivalente) | Solo si spec lo justifica; nunca obligatorio en core |
| Intent/query planning por LLM | Ausente | Rechazado (hot path) — §5 |
| Reranker con fallback a scores vectoriales | Ausente (no hay cross-encoder) | Evaluar solo cuando exista reranker |
| Skills como contexto persistente | Existente (skills propias) | Mantener |
| Gateway LLM / bot / compilación externa | Ausente | Fuera de scope — §5 |
| Watches ACL/multitenant, WebDAV, Helm | Ausente | Fuera de scope |
| TTL/expiración de memoria | Existente y más estricto (`expires_at` + GC) | Mantener — el TTL Redis de OV no garantiza borrar vectores |
| Hotness/tiering cold-warm-hot | `heat` existe (`lifecycle.rs`), política host-driven | Tiering explícito barato sobre lo construido — evaluar |
| `time_decay` en inner loop de scoring ANN | Decay solo en memoria/post-filtrado | Evaluar en `CPIndex` si medición lo justifica |

---

## 9. Negativos y deuda detectados en VantaDB

Todo **[Histórico]** — hallazgos estáticos sobre la base analizada; el worktree actual (HEAD `c561158`, ~119 ficheros sucios incluidos `checkpoint.rs`, `l1_writer.rs`, `context_engine`, `ingest`) puede haberlos alterado. Ninguno fue verificado dinámicamente; ninguno se afirma corregido ni vigente sin revalidación.

### 9-A. Reemplazo L1 no indivisible [Hecho estático]

Las rutas secuencial y batch eliminan registros antes de persistir los nuevos; [`l1_writer.rs:430-446`](../../vanta-memory/src/core/record/l1_writer.rs) lo documenta explícitamente (un fallo intermedio deja eliminaciones ya aplicadas). MEMG-11 (group-commit/throughput, entregado) **no contradice** el riesgo: mejorar throughput no hace indivisible delete+replace. Relacionar — sin equiparar — con **FIND-221** (atomicidad real del lote, parcial), **TXN-01** y **CRASH-01** (WAL/recovery; alcance distinto).

### 9-B. Checkpoint global con RMW separado [Hecho patrón / Hipótesis de carrera]

`mutate()` en [`checkpoint.rs:111-116`](../../vanta-memory/src/utils/checkpoint.rs) hace lectura, modificación y escritura separadas sobre la clave global `checkpoint.json` (`:79-84`); el lock es por sesión pero la clave es compartida → intercalación susceptible de lost update **[Hipótesis]**, requiere reproducción determinista. Análogo conceptual al issue upstream #7170 (ledger sobrescrito por recalls concurrentes).

### 9-C. Harness LoCoMo no apto para certificación [Hecho estático]

`eval_locomo.py:826-856`: dataset sintético por defecto, `FakeLocomoStore` fuera de `--self-test`, loader sin normalización al esquema LoCoMo original. Resolver en **MEM0-05**. Distinto de "no hay benchmarks medidos" — sí existen (Paso 23); lo pendiente es la certificación.

### 9-D. Deuda relacionada ya registrada (no duplicar)

Supersession (**FIND-277**, **FIND-577**), spill (**FIND-294**), hooks (**FIND-295**), reflection (**FIND-290**), watches (**FIND-299**), trust/scope (**FIND-320**), scopes spec (**MEMG-25**), IQL k=5 (**FIND-313**), enforcement fronteras estrictas/trust-en-retrieval/L3 (`Backlog.md`); provisioning/cuotas restantes. Todas con estado fechado — revalidar.

### 9-E. Falsos positivos descartados (registro)

- **"Falta gate de bind loopback sin auth"** → **descartado**: `validate_auth_config` ya existe y es fail-closed ([`src/server/bootstrap.rs:41-87`](../../src/server/bootstrap.rs)). El subagente de seguridad lo propuso sin verificar; la revisión adversarial lo eliminó (falso positivo FIND-07).
- **"Falta sistema de dedup"** → descartado: `l1_dedup.rs` existe (decisión LLM + supersession); reformulado como merge declarativo post-decisión (F5).
- **"Falta auditoría de índice"** → reformulado: existe audit/drift/repair de text-index; lo propuesto es el eje contenido↔vector↔derivado (F6).
- **"VantaDB sin benchmarks medidos"** → descartado: existen resultados Paso 23; LoCoMo-certificación es lo pendiente.

---

## 10. Evaluación de escritorio / UI

**Tres productos separados — no confundirlos ni atribuir stack entre ellos:**

| Producto | Naturaleza | Stack verificado |
|---|---|---|
| **OpenViking Helper** | App nativa descargable, documentada beta Windows x64/macOS | **Fuente no localizado en el checkout — no atribuir stack.** El directorio `dsh-desktop-smoke` (Electron) pertenece a **otro** producto (DeepSeek Harness Desktop), no a Helper |
| **Web Studio** | SPA servida por el backend | React 19 / Vite / TypeScript / TanStack Router+Query / Tailwind / cliente OpenAPI + adapter; depende del servidor (chat VikingBot) — `web-studio/` |
| **`frontend/`** | Biblioteca de UI, **no** runtime | workspace Node `^20.19 \|\| >=22.12`, React 18/19 peer |

**Helper (solo documentación [Claim], no runtime):** `docs/en/agent-integrations/14-openviking-helper.md` + [docs oficiales](https://docs.openviking.ai/en/agent-integrations/14-openviking-helper): perfiles cloud/selfhost, detección de agentes, setup de MCP/hooks/plugins, inspección de sesiones locales, inventario, sync selectivo, captura de evidencia + commit + recall + inject. Nada verificado en ejecución.

**MemoryImpact:** `web-studio/src/features/workbench/sessions/components/memory-impact.tsx` — componente de impacto de memoria por sesión, fuente de la idea del inspector (F10).

**VantaDB Studio (ya existente):** worktree `develop-desktop`, Tauri v2 Rust/React/Vite con memoria, conexiones, skills, escenas, actividad y diagnóstico de retrieval; bridge tipado (`desktop/src/vanta.ts`, `vanta-http-map.ts`). **No proponer funciones que ya existen.**

**Evaluación:** el valor incremental para VantaDB está en (1) onboarding de integraciones con detección+consentimiento+preview y (2) inspector de evidencia del ciclo captura→extracción→recall→inyección (F10) — no en construir otra consola ni un Helper propio. La evaluación de UI en runtime **no se realizó** (sin ejecución de ninguna app).

---

## Apéndice A. Inventario de fuentes

| Fuente | Tipo | Contenido aportado |
|---|---|---|
| Checkout `C:\Users\Eros\Open Vikings\OpenViking` @ `b8e8895` | Primario | Toda la evidencia de código OV (6.017 ficheros, limpio, full history) |
| Informe de evaluación arquitectónica (subagente) | Derivado | Mapa de arquitectura, componentes/stack por `pyproject.toml`/`Cargo.toml`, catálogo de capacidades, compatibilidad Windows, features declaradas vs ejecutables |
| Auditoría defensiva estática (subagente) | Derivado | Hallazgos H1–H8, observaciones positivas, límites de cobertura |
| Investigación estática engine/memoria (subagente `985f28d9`) | Derivado | Claims vs implementación (L0/L1/L2, intent planning, scopes, ANN flat, rerank, memoria, colas), ideas adoptables, riesgos |
| Dictamen P2-01 — segunda opinión (subagente `334c6055`) | Derivado | Red-team de hipótesis, correcciones de duplicación, evidencia faltante, recomendaciones reescritas |
| Incidencias públicas (subagente `4deb8d43`) | Derivado | Muestra `gh api` de 79 issues, matriz de aplicabilidad, hallazgos prioritarios |
| Revisión de benchmarks (agente principal) | Derivado | `benchmark/README.md`, harnesses LoCoMo/tau2/RAG, defectos metodológicos |
| Evaluación escritorio/UI (subagente `b6b41d8d` + revisión directa) | Derivado | Separación Helper/WebStudio/frontend, MemoryImpact, contraste con VantaDB Studio |
| `history_0d9354c789ee4723.md` / `history_fa79ee29c0154ae9.md` (`devin/cli/summaries/`) | Transcripción | Conversación completa de la investigación (sesiones 1 y 2) |
| Repo VantaDB (base `632f491`/`f6ed51d`, HEAD `c561158` dirty) | Primario | Contrastes de capacidades, hallazgos §9, backlog cruzado |

## Apéndice B. Decisiones corregidas por la revisión adversarial

Veredicto del dictamen: **REQUEST CHANGES** condicional — 6 hipótesis mayoritariamente correctas, 2 supuestos que no resistían lectura directa y 2 líneas de adopción por delimitar.

| Decisión inicial | Corrección impuesta | Estado final |
|---|---|---|
| "Adoptar retrieval jerárquico de OV" | El retriever es flat con filtro `level`; lo adoptable es el **esquema URI→niveles materializados** (storage/indexación), no `HierarchicalRetriever` | Reescrito → F1/F2 |
| "index_consistency ausente en VantaDB" | Existe audit/drift/repair de text-index; el eje propuesto (contenido↔vector por nivel/revisión) es distinto | Reformulado → F6 (extensión) |
| "Sistema de dedup de memorias" | Ya existe `l1_dedup.rs` con juicio LLM; el valor incremental es el **operador declarativo post-decisión**, Optional | Reformulado → F5 |
| "Bloquear bind no-loopback sin auth" | Ya existe `validate_auth_config` fail-closed en `bootstrap.rs` | **Eliminado — falso positivo** |
| "Scopes de memoria" | Ya en backlog (MEMG-25 spec, FIND-301/305 enforcement) | Adjuntado como evidencia, no fila nueva |
| "Búsqueda flat + filtro" | VantaDB ya tiene BM25/HNSW/RRF | No presentado como adopción |
| "Superioridad de ragfs vs WAL de Vanta" | Sin medición → "superior" prohibido; la idea evaluable es lock granularity/cola de escritura, no el storage | Delimitado |
| Estado de issues por label | Estado cerrado ≠ resuelto; verificación por fix fusionado real | Regla adoptada (§3, §6) |

Evidencia aún pendiente exigida por el dictamen: contrato verificable de la capa L0/L1 materializada en VantaDB, barrido SPDX completo de `openviking/`, enforcement real de `check_access` (FIND-305 declara pendiente — `middleware.rs:178-180` es intención, no enforcement), comparación medida de garantías ragfs vs WAL.

## Apéndice C. Roadmap de dependencias

```text
Garantías de escritura + checkpoints + evaluación confiable (§4: L1, CAS, MEM0-05, recovery)
        │
        ▼
Scopes/retrieval  ──────────────────────────────┐
        │                                        │
        ▼                                        │
Identidad (F1) + Derivados versionados (F2)      │
        │                                        │
        ▼                                        ▼
Extensión del assembler (F3) + Ledger de entrega (F4)
        │
        ▼
Consistencia (F6) + Experiencias (F8) + Watches (F9)   [F5 paralelo tras §4-A; F7 soporte transversal]
        │
        ▼
Inspector/onboarding en Studio (F10)   ← la UI read-only puede paralelizarse desde el inicio
```

Garantías: nada que borre en cascada antes de §4-A; ningún ledger antes de §4-B; ningún benchmark como certificación antes de §4-3; la lectura de UI (F10 read-only) no depende de las fichas de escritura.

## Apéndice D. Verificación pendiente — solo manual

Nada de lo siguiente se ejecutó ni debe presumirse verificado:

1. **Reproducción determinista** de la carrera de checkpoint (dos sesiones, `mutate` concurrente sobre la clave global) — prueba a diseñar, ejecutar manualmente y reportar.
2. **Reproducción de pérdida L1** (fallo inyectado entre delete y put en ruta batch/secuencial).
3. **Self-tests offline** (solo prueban arneses, no calidad): `python evals/memory_harness.py --self-test`, `python evals/eval_longmemeval.py --self-test`, `python evals/eval_locomo.py --self-test`.
4. **Verificación manual de H1/H3 de seguridad OV**: levantar gateway con `management_key` vacío y confirmar el bypass de `/management/*`; ensayo DNS-rebinding del TOCTOU — hipótesis, no demostrada.
5. **Barrido SPDX completo** de `openviking/` antes de cualquier reutilización "inspirada en".
6. **Contrato verificable por ficha adoptada** (comando + salida esperada), no auto-reporte.
7. **Revalidación de cada hallazgo [Histórico]** contra el worktree actual antes de convertirlo en fila de backlog.

## Apéndice E. Incógnitas y lagunas de evidencia

- **Fuente de OpenViking Helper no localizada** en el checkout → stack, arquitectura interna y realización de funciones documentadas, desconocidas. `dsh-desktop-smoke` pertenece a otro producto (DeepSeek Harness Desktop).
- **Inventario SPDX por archivo incompleto** — muestreo de headers, no barrido; el reparto AGPL/Apache-2.0 por subárbol no está cerrado.
- **Ninguna medida de rendimiento comparativa** (ragfs vs WAL, ANN flat vs HNSW, latencias de colas): toda comparativa de garantías es cualitativa.
- **Cobertura de seguridad parcial**: `openviking_gateway` (lua/capture/protocols), `crates/ragfs` multibackend/shell, parsers individuales, frontend/web-studio y la mayoría de los 40 `unsafe` de `provider.rs` sin revisar; sin `cargo audit` ni semgrep.
- **Muestra de issues acotada**: 15 de 79 leídos a fondo; selección por recencia+búsquedas temáticas, no exhaustiva.
- **Estado de `check_access`/enforcement HTTP** — `middleware.rs:178-180` es intención declarada, no enforcement verificado (FIND-305 pendiente).
- **Evaluación de UI en runtime no realizada**: ni OV Studio/Helper ni VantaDB Studio se ejecutaron.
- **Evolución posterior del backlog**: estado de LoCoMo/LongMemEval/NIAH/FIND-* fechado; puede haber cambiado desde el corte.
- **`memory_harness` proxies**: abstención/economía/fidelidad no equivalen a evaluación juzgada por LLM — laguna metodológica propia declarada.
