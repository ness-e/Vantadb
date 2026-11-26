---
title: "Licencias del Corpus de Extracción (43 Proyectos)"
kind: research
status: active
description: "Clasificación legal verificada de los 43 proyectos analizados y gate previo para candidatas X-*"
tags: [extraccion, licenses]
---

# Licencias del Corpus de Extracción (43 Proyectos)

Auditoría legal estática (solo lectura) de los **43 repositorios** del corpus de extracción ([inventario general](./agent-memory-2026-11-07/README.md), [manifiesto base](./agent-memory-2026-11-07/clone-manifest.json), [manifiesto de adiciones](./agent-memory-2026-11-07/clone-additions-manifest.json), [informe OpenViking](./Backlog-OpenViking-VantaDB.md) e [formato canónico de backlogs](./BACKLOG-FORMAT.md)), contrastada contra los ficheros `LICENSE*`/`NOTICE*` de cada clon en `C:\Users\Eros\<proyecto>` y cruzada con las **265 candidatas `X-*`** y la licencia **Apache-2.0** de VantaDB.

> **Alcance normativo:** análisis técnico de ingeniería para control de provenance; no constituye asesoría jurídica formal. Reescribir código a Rust/TypeScript **no extingue** derechos de autor si existe traducción sustancial: donde rige gate *clean-room*, solo se trasladan invariantes y contratos abstractos.

## Índice

- [1. Resumen por categoría legal](#1-resumen-por-categoría-legal)
- [2. Matriz verificada de los 43 proyectos](#2-matriz-verificada-de-los-43-proyectos)
- [3. Obligaciones concretas y cruce con filas X-*](#3-obligaciones-concretas-y-cruce-con-filas-x-)
- [4. Gate previo a especificación e implementación](#4-gate-previo-a-especificación-e-implementación)

---

## 1. Resumen por categoría legal

Los **8 valores `NOASSERTION`** del snapshot congelado de la API de GitHub quedaron resueltos al inspeccionar los textos de licencia locales en disco (`0` proyectos `NOASSERTION` en raíz; únicamente 4 submódulos no inicializados sin ninguna fila `X-*` asociada).

| Categoría | Proyectos | Filas `X-*` | Régimen frente a VantaDB (`Apache-2.0`) |
|---|---:|---:|---|
| **Permissive** (`MIT`, `Apache-2.0`, `PostgreSQL`) | 34 | 209 | Compatible con `Apache-2.0` (respetando avisos de copyright y `NOTICE` si hubiera obra derivada; la campaña mantiene reimplementación propia) |
| **Copyleft fuerte** (`AGPL-3.0`, `GPL-3.0`) | 2 | 21 | **Incompatible** con `Apache-2.0`. Prohibido portar o traducir código/prompts; solo especificación *clean-room* |
| **Source-available** (`BSL-1.1`, `RSALv2 / SSPL-1.0 / AGPL-3.0`, `Apache-2.0` modificada) | 3 | 14 | **Incompatible** con `Apache-2.0`. Prohibido reutilizar o derivar código/diseño; solo contratos funcionales abstractos |
| **Mixta por componente** (`MIT`+`BSL-1.1`, `BSD-3-Clause`+`Enterprise`, `Apache-2.0`+`Commercial`, `MIT`+`CC-BY-4.0`) | 4 | 21 | **Permitida solo sobre subárbol permisivo verificado** (`0/21` filas pisan rutas privativas/BSL); bloqueo duro de directorios restringidos |
| **NOASSERTION** (raíz tras verificación local) | 0 | 0 | Submódulos no clonados (`mem0/evaluation`, `tidb/pkg/extension/enterprise`, `anything-llm/{browser-extension,embed}`) permanecen sin acreditar y vedados |
| **Total** | **43** | **265** | — |

---

## 2. Matriz verificada de los 43 proyectos

| Proyecto (`owner/repo`) | SHA pin | API GitHub | Licencia verificada en clon (`C:\Users\Eros\...`) | Categoría | Backlog y filas `X-*` | Gate para VantaDB |
|---|---|---|---|---|---|---|
| `rohitg00/agentmemory` | `4f758eff36` | `Apache-2.0` | `Apache-2.0` (`LICENSE`, `packages/mcp/LICENSE`) | Permissive | [agentmemory/BACKLOG.md](./agentmemory/BACKLOG.md) (`X-AGM-01..04`, 4) | Permisivo (`Apache-2.0`) |
| `agno-agi/agno` | `f45ccee83d` | `Apache-2.0` | `Apache-2.0` (`LICENSE`, `libs/*/LICENSE`) | Permissive | [agno/BACKLOG.md](./agno/BACKLOG.md) (`X-AGN-01..05`, 5) | Permisivo (`Apache-2.0`) |
| `Mintplex-Labs/anything-llm` | `308b27662b` | `MIT` | `MIT` (`LICENSE`; submódulos `embed`/`browser-extension` no clonados) | Permissive | [anything-llm/BACKLOG.md](./anything-llm/BACKLOG.md) (`X-ALL-01..07`, 7) | Permisivo (`MIT`; no tocar submódulos) |
| `microsoft/autogen` | `d07c667a32` | `CC-BY-4.0` | Código `MIT` (`LICENSE-CODE`); docs `CC-BY-4.0` (`LICENSE`) | Mixta por componente | [autogen/BACKLOG.md](./autogen/BACKLOG.md) (`X-ATG-01..05`, 5) | `X-ATG-01..05` citan `.py` (`MIT`); no copiar docs `CC-BY-4.0` |
| `chroma-core/chroma` | `1a3ac2c7c6` | `Apache-2.0` | `Apache-2.0` (`LICENSE`) | Permissive | [chroma/BACKLOG.md](./chroma/BACKLOG.md) (`X-CHR-01..06`, 6) | Permisivo (`Apache-2.0`) |
| `DeusData/codebase-memory-mcp` | `612eee2518` | `MIT` | `MIT` (`LICENSE`; gramáticas vendored con licencias propias no citadas) | Permissive | [codebase-memory-mcp/BACKLOG.md](./codebase-memory-mcp/BACKLOG.md) (`X-CBM-01..06`, 6) | Permisivo (`MIT` en `src/` y `graph-ui/`) |
| `colbymchenry/codegraph` | `f72c9d2efb` | `MIT` | `MIT` (`LICENSE`) | Permissive | [codegraph/BACKLOG.md](./codegraph/BACKLOG.md) (`X-CGR-01..05`, 5) | Permisivo (`MIT`) |
| `topoteretes/cognee` | `8853378eae` | `Apache-2.0` | `Apache-2.0` + `NOTICE.md` (`LICENSE`, `NOTICE.md`) | Permissive | [cognee/BACKLOG.md](./cognee/BACKLOG.md) (`X-COG-01..08`, 8) | Permisivo (`Apache-2.0`; preservar `NOTICE` si hay derivación) |
| `crewAIInc/crewAI` | `73f48320db` | `MIT` | `MIT` (`LICENSE`) | Permissive | [crewAI/BACKLOG.md](./crewAI/BACKLOG.md) (`X-CRW-01..05`, 5) | Permisivo (`MIT`) |
| `langgenius/dify` | `65b4e17d6e` | `NOASSERTION` | `Apache-2.0` modificada — LangGenius License (`LICENSE`) | Source-available | [dify/BACKLOG.md](./dify/BACKLOG.md) (`X-DIF-01..05`, 5) | **BLOQUEO código/UI**: solo *clean-room* |
| `facebookresearch/faiss` | `277b3c76b7` | `MIT` | `MIT` (`LICENSE`) | Permissive | [faiss/BACKLOG.md](./faiss/BACKLOG.md) (`X-FAI-01..05`, 5) | Permisivo (`MIT`) |
| `FlowiseAI/Flowise` | `9291856d1e` | `NOASSERTION` | `Apache-2.0` general (`LICENSE.md`) + `FlowiseAI Commercial` en `packages/server/src/enterprise/` e `IdentityManager.ts` | Mixta por componente | [Flowise/BACKLOG.md](./Flowise/BACKLOG.md) (`X-FLW-01..05`, 5) | `X-FLW-01..05` en rutas `Apache-2.0`; vedado `enterprise/**` |
| `Graphify-Labs/graphify` | `16218bfc80` | `Apache-2.0` | `Apache-2.0` + `MIT` previo + `NOTICE` (`LICENSE`, `LICENSE-MIT`, `NOTICE`) | Permissive | [graphify/BACKLOG.md](./graphify/BACKLOG.md) (`X-GRF-01..07`, 7) | Permisivo (`Apache-2.0`; preservar `NOTICE` si hay derivación) |
| `getzep/graphiti` | `747b480ec6` | `Apache-2.0` | `Apache-2.0` (`LICENSE`) | Permissive | [graphiti/BACKLOG.md](./graphiti/BACKLOG.md) (`X-GRT-01..05`, 5) | Permisivo (`Apache-2.0`) |
| `microsoft/graphrag` | `34b2b67ae4` | `MIT` | `MIT` (`LICENSE`) | Permissive | [graphrag/BACKLOG.md](./graphrag/BACKLOG.md) (`X-GRR-01..06`, 6) | Permisivo (`MIT`) |
| `vectorize-io/hindsight` | `eb963cb5a9` | `MIT` | `MIT` (`LICENSE`) | Permissive | [hindsight/BACKLOG.md](./hindsight/BACKLOG.md) (`X-HIN-01..07`, 7) | Permisivo (`MIT`) |
| `khoj-ai/khoj` | `ae229ca894` | `AGPL-3.0` | `AGPL-3.0` raíz (`LICENSE`) + `GPL-3.0` (`src/interface/obsidian/LICENSE`) | Copyleft fuerte | [khoj/BACKLOG.md](./khoj/BACKLOG.md) (`X-KHO-01..07`, 7) | **BLOQUEO código**: solo *clean-room* |
| `lancedb/lancedb` | `bff93ff314` | `Apache-2.0` | `Apache-2.0` (`LICENSE`, `python/LICENSE`) | Permissive | [lancedb/BACKLOG.md](./lancedb/BACKLOG.md) (`X-LAN-01..07`, 7) | Permisivo (`Apache-2.0`) |
| `langchain-ai/langchain` | `d9caf08357` | `MIT` | `MIT` (`LICENSE`, `libs/*/LICENSE`) | Permissive | [langchain/BACKLOG.md](./langchain/BACKLOG.md) (`X-LCH-01..07`, 7) | Permisivo (`MIT`) |
| `langflow-ai/langflow` | `60999cc9a7` | `MIT` | `MIT` (`LICENSE`) | Permissive | [langflow/BACKLOG.md](./langflow/BACKLOG.md) (`X-LFL-01..05`, 5) | Permisivo (`MIT`) |
| `langchain-ai/langgraph` | `e5e37b5d70` | `MIT` | `MIT` (`LICENSE`, `libs/*/LICENSE`) | Permissive | [langgraph/BACKLOG.md](./langgraph/BACKLOG.md) (`X-LGR-01..07`, 7) | Permisivo (`MIT`) |
| `letta-ai/letta` (+ `letta-code`, `letta-server`) | `8816d3a851` | `Apache-2.0` | `Apache-2.0` en `letta`, `letta-code` (`17436c8e4`) y `letta-server` (`56ba9c2`) | Permissive | [letta/BACKLOG.md](./letta/BACKLOG.md) (`X-LET-01..12`, `X-LET-20..22`, 15) | Permisivo (`Apache-2.0`; V1 `archive` deprecado upstream) |
| `HKUDS/LightRAG` | `aaebfe1bb8` | `MIT` | `MIT` (`LICENSE`) | Permissive | [LightRAG/BACKLOG.md](./LightRAG/BACKLOG.md) (`X-LRA-01..07`, 7) | Permisivo (`MIT`) |
| `run-llama/llama_index` | `444fdf0684` | `MIT` | `MIT` (`LICENSE`, `llama-index-core/LICENSE`) | Permissive | [llama_index/BACKLOG.md](./llama_index/BACKLOG.md) (`X-LLI-01..07`, 7) | Permisivo (`MIT`) |
| `meilisearch/meilisearch` | `c18c81973a` | `NOASSERTION` | `MIT AND BUSL-1.1` (`LICENSE`, `LICENSE-MIT`, `LICENSE-EE` en `*enterprise_edition*`) | Mixta por componente | [meilisearch/BACKLOG.md](./meilisearch/BACKLOG.md) (`X-MEI-01..05`, 5) | `X-MEI-01..05` en `MIT`; vedado `*enterprise_edition*` (Rust) |
| `mem0ai/mem0` | `f1791daa86` | `Apache-2.0` | `Apache-2.0` (`LICENSE`; submódulo `evaluation` no clonado) | Permissive | [mem0/BACKLOG.md](./mem0/BACKLOG.md) (`X-M00-01..06`, 6) | Permisivo (`Apache-2.0`; no tocar submódulo) |
| `MemPalace/mempalace` | `f8e3536218` | `MIT` | `MIT` (`LICENSE`) | Permissive | [mempalace/BACKLOG.md](./mempalace/BACKLOG.md) (`X-MPL-01..06`, 6) | Permisivo (`MIT`) |
| `milvus-io/milvus` | `eba8be50f8` | `Apache-2.0` | `Apache-2.0` (`LICENSE`) | Permissive | [milvus/BACKLOG.md](./milvus/BACKLOG.md) (`X-MIL-01..07`, 7) | Permisivo (`Apache-2.0`) |
| `openai/openai-agents-python` | `27c75c5f9c` | `MIT` | `MIT` (`LICENSE`) | Permissive | [openai-agents-python/BACKLOG.md](./openai-agents-python/BACKLOG.md) (`X-OAP-01..05`, 5) | Permisivo (`MIT`) |
| `volcengine/OpenViking` | `b8e88959f3` | `AGPL-3.0` | `AGPL-3.0` raíz + `web-studio/` + `frontend/packages/react`; `crates/` y `sdk/typescript` `Apache-2.0`; `bot/` `MIT` | Copyleft fuerte | [OpenViking/BACKLOG.md](./OpenViking/BACKLOG.md) (`X-OVK-01..14`, 14) | **BLOQUEO código**: `14/14` filas vienen de árbol `AGPL-3.0` |
| `VectifyAI/PageIndex` | `2ac8694bf8` | `MIT` | `MIT` (`LICENSE`) | Permissive | [PageIndex/BACKLOG.md](./PageIndex/BACKLOG.md) (`X-PGI-01..04`, 4) | Permisivo (`MIT`) |
| `pgvector/pgvector` | `724588a550` | `NOASSERTION` | `PostgreSQL License` (`LICENSE`) | Permissive | [pgvector/BACKLOG.md](./pgvector/BACKLOG.md) (`X-PGV-01..05`, 5) | Permisivo (`PostgreSQL License`) |
| `qdrant/qdrant` | `222b2e44ab` | `Apache-2.0` | `Apache-2.0` (`LICENSE`) | Permissive | [qdrant/BACKLOG.md](./qdrant/BACKLOG.md) (`X-QDR-01..05`, 5) | Permisivo (`Apache-2.0`) |
| `The-Vibe-Company/quivr` | `bb1b19ecf4` | `MIT` | `MIT` (`LICENSE`) | Permissive | [quivr/BACKLOG.md](./quivr/BACKLOG.md) (`X-QUI-01..06`, 6) | Permisivo (`MIT`) |
| `infiniflow/ragflow` | `3c6d2255af` | `Apache-2.0` | `Apache-2.0` (`LICENSE`) | Permissive | [ragflow/BACKLOG.md](./ragflow/BACKLOG.md) (`X-RAG-01..05`, 5) | Permisivo (`Apache-2.0`) |
| `redis/redis` | `5b69ba2ce9` | `NOASSERTION` | Tri-licencia `RSALv2 OR SSPL-1.0 OR AGPL-3.0` (`LICENSE.txt`) + `BSD-3-Clause` `<=7.2` (`REDISCONTRIBUTIONS.txt`) | Source-available | [redis/BACKLOG.md](./redis/BACKLOG.md) (`X-RED-01..05`, 5) | **BLOQUEO código**: ninguna opción compatible con `Apache-2.0` |
| `pathwaycom/serviette` | `84e92a3372` | `MIT` | `MIT` en repo (`LICENSE`); depende en runtime de `pathway` (`BSL-1.1` externo) | Permissive | [serviette/BACKLOG.md](./serviette/BACKLOG.md) (`X-SRV-01..04`, 4) | Fuente `serviette` `MIT`; prohibido depender de `pathway` |
| `supermemoryai/supermemory` | `16ed414e0e` | `MIT` | `MIT` en SDKs/MCP (`LICENSE`); motor SaaS remoto es privativo externo | Permissive | [supermemory/BACKLOG.md](./supermemory/BACKLOG.md) (`X-SUP-01..08`, 8) | Permisivo en clientes `MIT`; `X-SUP-08` exige contrato externo |
| `surrealdb/surrealdb` | `9577ac5f0c` | `NOASSERTION` | `BSL-1.1` (`LICENSE`; Change Date `2030-01-01` → `Apache-2.0`) | Source-available | [surrealdb/BACKLOG.md](./surrealdb/BACKLOG.md) (`X-SUR-01..04`, 4) | **BLOQUEO código (Rust)**: riesgo máximo de copia accidental |
| `TencentCloud/TencentDB-Agent-Memory` | `2002196f2d` | `NOASSERTION` | `MIT` con preámbulo Tencent (`LICENSE`, `package.json` en `MemoryCore`/`MemoryKnowledge`/`sdk`) | Permissive | [TencentDB-Agent-Memory/BACKLOG.md](./TencentDB-Agent-Memory/BACKLOG.md) (`X-TAM-01..08`, 8) | Permisivo (`MIT` local; puentes npm/cloud externos aparte) |
| `pingcap/tidb` | `41efd866c3` | `Apache-2.0` | `Apache-2.0` (`LICENSE`; submódulo `pkg/extension/enterprise` no clonado) | Permissive | [tidb/BACKLOG.md](./tidb/BACKLOG.md) (`X-TID-01..03`, 3) | Permisivo (`Apache-2.0`; no tocar submódulo enterprise) |
| `Egonex-AI/Understand-Anything` | `a6ea402476` | `MIT` | `MIT` (`LICENSE`) | Permissive | [Understand-Anything/BACKLOG.md](./Understand-Anything/BACKLOG.md) (`X-UND-01..06`, 6) | Permisivo (`MIT`) |
| `weaviate/weaviate` | `85bd085e6f` | `NOASSERTION` | `BSD-3-Clause` fuera de `wl/` (`LICENSE-BSD`) + `Weaviate License` en `wl/` (`wl/LICENSE-WEAVIATE`) | Mixta por componente | [weaviate/BACKLOG.md](./weaviate/BACKLOG.md) (`X-WEA-01..06`, 6) | `X-WEA-01..06` fuera de `wl/` (`BSD-3-Clause`); vedado `wl/**` |

---

## 3. Obligaciones concretas y cruce con filas `X-*`

### 3.1 Copyleft fuerte (21 filas afectadas — bloqueo total de código)

- **OpenViking (`AGPL-3.0` raíz, mixta en subárboles menores) → `X-OVK-01..14` (14 filas):**
  - *Evidencia en clon:* `LICENSE` raíz, headers `SPDX-License-Identifier: AGPL-3.0` en `openviking/**/*.py`, `integrations/frameworks/langchain/LICENSE` (`AGPL-3.0`) y `frontend/packages/react` (`AGPL-3.0-only`); `web-studio/` carece de licencia propia y hereda `AGPL-3.0`. Solo `crates/LICENSE` (`ragfs`, `ov_cli`), `examples/LICENSE` y `sdk/typescript` son `Apache-2.0`, y `bot/license/LICENSE` es `MIT`.
  - *Cruce exacto:* las **14 filas** (`X-OVK-01..09` sobre `openviking/*` y `X-OVK-10..14` sobre `web-studio/*` + `openviking/session/*`) referencian exclusivamente archivos `AGPL-3.0`. Ninguna proviene de `crates/`.
  - *Obligación:* `AGPL-3.0` §13 exige ofrecer el *Corresponding Source* completo bajo `AGPL-3.0` a todo usuario que interactúe por red (`vantadb-server`, `vantadb-mcp`, `vanta-proxy`) si incluye código derivado. Prohibido traducir funciones, prompts o componentes React; implementar desde especificación funcional propia.
- **Khoj (`AGPL-3.0` + `GPL-3.0`) → `X-KHO-01..07` (7 filas):**
  - *Evidencia en clon:* `LICENSE` raíz es `AGPL-3.0`; `src/interface/obsidian/LICENSE` es `GPL-3.0`.
  - *Cruce exacto:* `X-KHO-01..06` citan `src/khoj/{database,routers,processor,search_type}/*` (`AGPL-3.0`); `X-KHO-02` y `X-KHO-07` citan además `src/interface/obsidian/src/utils.ts` (`GPL-3.0`) e `interface/desktop/main.js` (`AGPL-3.0`).
  - *Obligación:* ambas licencias son copyleft recíproco incompatible con `Apache-2.0` en distribución combinada. `X-KHO-01..07` solo pueden ejecutarse como rediseño *clean-room* sobre los seams existentes de VantaDB.

### 3.2 Source-available y licencias modificadas (14 filas afectadas — bloqueo total de código)

- **SurrealDB (`BSL-1.1`, Rust) → `X-SUR-01..04` (4 filas):**
  - *Evidencia en clon:* `LICENSE` fija *Licensor: SurrealDB Ltd.*, *Licensed Work: SurrealDB 3.0*, *Change Date: `2030-01-01`* hacia `Apache-2.0`, y prohíbe todo uso productivo como *Database Service* (ofrecer funcionalidad de base de datos a terceros que creen/gestionen esquemas o tablas).
  - *Cruce exacto:* `X-SUR-01..04` citan `surrealdb/datastore/src/{triggers,index_state,tx,values/index_build}.rs` y `surrealdb/core/src/kvs/{write_barrier,rollback,index/builder}.rs`. Al compartir lenguaje **Rust** con VantaDB, existe riesgo crítico de trasvase sintáctico involuntario: el autor de código no debe tener abierto el clon de SurrealDB al implementar.
- **Redis (`RSALv2 OR SSPL-1.0 OR AGPL-3.0`) → `X-RED-01..05` (5 filas):**
  - *Evidencia en clon:* `LICENSE.txt` establece tri-licencia desde Redis 8.0 a elección del receptor (`RSALv2`, `SSPLv1` o `AGPLv3`); `REDISCONTRIBUTIONS.txt` mantiene `BSD-3-Clause` solo para código `<=7.2`, pero `src/{t_stream,aof,replication,db,expire,evict}.c` en `unstable` (`5b69ba2ce9`) entrelaza commits post-7.2 inseparables sin auditoría línea por línea.
  - *Obligación:* `RSALv2` prohíbe distribuir o exponer a terceros un producto cuyo valor derive del software; `SSPLv1` §13 exige licenciar bajo `SSPLv1` todo el stack de servicio/gestión; `AGPLv3` contamina con copyleft de red. `X-RED-01..05` quedan restringidas a diseño conceptual puro.
- **Dify (`Apache-2.0` modificada — LangGenius) → `X-DIF-01..05` (5 filas):**
  - *Evidencia en clon:* `LICENSE` añade a `Apache-2.0`: (1a) prohibición de operar entorno *multi-tenant* (= múltiples workspaces) sin licencia comercial escrita; (1b) prohibición de quitar/alterar LOGO o copyright en el frontend (`web/`); (2a) reserva de endurecimiento unilateral; y reclamo expreso de **patente de diseño/apariencia** sobre el diseño interactivo.
  - *Cruce exacto:* `X-DIF-01`, `X-DIF-03`, `X-DIF-04` y `X-DIF-05` citan `api/**` (sujeto a 1a); `X-DIF-02` cita además `web/app/components/datasets/hit-testing/index.tsx` (sujeto a 1a, 1b y patente de apariencia). Prohibido portar código Python/TS o replicar el layout visual de `web/` en VantaDB Studio.

### 3.3 Mixtas por componente y casos `NOASSERTION` despejados

- **Meilisearch (`MIT AND BUSL-1.1`, Rust + `no-agents.md`) → `X-MEI-01..05` (5 filas):** `LICENSE-EE` (`BSL-1.1`, no producción, conversión a `MIT` a los 4 años) gobierna 11 ficheros `*enterprise_edition*` (sharding, federated/proxy/routes/tasks network, `scheduler/enterprise_edition/{mod,network,s3}.rs`); el resto es `MIT` (`LICENSE-MIT`). Las 5 filas `X-MEI-01..05` citan únicamente rutas `MIT` (en `X-MEI-05` solo se menciona históricamente un PR sobre snapshot S3 sin extraer de `enterprise_edition/s3.rs`). Se respeta además `AGENTS.md` (*no-agents*): cero modificaciones al clon, cero entrenamiento y reimplementación propia sin copiar Rust literal.
- **Weaviate (`BSD-3-Clause` + `wl/` privativa) → `X-WEA-01..06` (6 filas):** fuera de `wl/` rige `BSD-3-Clause`; dentro de `wl/` rige *Weaviate License Agreement* (*All rights reserved*, sujeta a license key comercial). `X-WEA-01..06` citan solo `adapters/repos/db/**`, `entities/vectorindex/**` y `tools/dev/bench/**` (`0` referencias a `wl/`). Queda prohibido inspeccionar o derivar de `wl/**`.
- **Flowise (`Apache-2.0` + `FlowiseAI Commercial`) → `X-FLW-01..05` (5 filas):** `packages/server/src/enterprise/` e `IdentityManager.ts` prohíben copia/distribución/fusión comercial y asignan titularidad de parches a FlowiseAI. `X-FLW-01..05` citan solo rutas `Apache-2.0` fuera de esos ficheros; `enterprise/**` e `IdentityManager.ts` quedan bloqueados.
- **AutoGen (`MIT` código + `CC-BY-4.0` docs) → `X-ATG-01..05` (5 filas):** `README.md:228-235` separa código bajo `LICENSE-CODE` (`MIT`) y documentación bajo `LICENSE` (`CC-BY-4.0`). Las 5 filas citan exclusivamente `.py` bajo `MIT`; no copiar prosa ni diagramas `CC-BY-4.0`.
- **TencentDB Agent Memory (`MIT`) → `X-TAM-01..08` (8 filas) y pgvector (`PostgreSQL License`) → `X-PGV-01..05` (5 filas):** ambos figuraban `NOASSERTION` en el snapshot de GitHub por formato de encabezado, pero sus `LICENSE` locales son permisivas OSI (`MIT` y `PostgreSQL License`). En `X-TAM-*` no incorporar dependencias externas de su bridge npm/SaaS sin auditar sus propias licencias; en `X-SRV-01..04` ([serviette/BACKLOG.md](./serviette/BACKLOG.md), `MIT`) queda prohibido añadir `pathway` (`BSL-1.1`) como dependencia; en `X-SUP-08` ([supermemory/BACKLOG.md](./supermemory/BACKLOG.md), `MIT`) recordar que el motor SaaS remoto es privativo.

---

## 4. Gate previo a especificación e implementación

Antes de promover cualquier fila `X-*` a `/spec` o a `docs/dev/Backlog.md` (véase [CONSOLIDATED.md](./agent-memory-2026-11-07/CONSOLIDATED.md)):

1. **Bloqueo *clean-room* obligatorio (35 filas: `X-OVK-01..14`, `X-KHO-01..07`, `X-SUR-01..04`, `X-RED-01..05`, `X-DIF-01..05`):** la spec se redacta únicamente desde la descripción funcional en voz VantaDB y los seams propios; queda prohibido abrir el clon upstream durante la escritura del código Rust/TS o copiar nombres internos no estándar, tablas de constantes, prompts o layouts UI.
2. **Guardia de subárbol en repositorios mixtos (21 filas: `X-MEI-01..05`, `X-WEA-01..06`, `X-FLW-01..05`, `X-ATG-01..05`):** toda revalidación adicional contra el clon debe verificar antes que la ruta objetivo no pertenezca a `*enterprise_edition*`, `wl/**`, `packages/server/src/enterprise/**`, `IdentityManager.ts` ni documentación `CC-BY-4.0`.
3. **Política general en permisivos (209 filas):** aunque `MIT`, `Apache-2.0`, `BSD-3-Clause` y `PostgreSQL` permiten obras derivadas, la regla arquitectónica de VantaDB exige reimplementación idiomática sobre `vanta-memory`/`vantadb`. Si excepcionalmente se incorporase un fragmento literal de un proyecto permisivo, deberá añadirse su copyright y licencia (y el contenido de `NOTICE` para `Apache-2.0`, p. ej. `cognee` o `graphify`) en el archivo `THIRD_PARTY_NOTICES` o cabecera correspondiente antes del merge.
