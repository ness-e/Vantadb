---
title: Notion Sync 2026-09-24 — adiciones post-investigación (drafts listos para aplicar)
kind: concept
status: active
description: "Añadir sección: \"Evidencia 2026 (investigación integral)\" — re-baseline 2026-09-30: mapeo verificado + deltas post-campaña (F0-F5 completas, corte 0.8.0 preparado)"
tags: [vantadb, notion, sync, investigacion, post-investigacion]
---

# Notion Sync — 2026-09-24 (re-baseline 2026-09-30)

> **Propósito:** contenido listo para pegar en las páginas de Notion ("VantaDB Docs"). Ejecuta el owner/agente con Notion MCP.
> **Reglas:** cada claim verificado contra código o fuente externa citada (Regla 11) · marcas `[REAL]/[PARCIAL]/[PROPUESTA]` siempre · no romper enlaces.

## 0. Mapeo verificado (Notion MCP live, 2026-09-30)

| Draft | Página real | ID | Nota |
|---|---|---|---|
| hub | `VantaDB Docs` | `3d4d0445-9756-80f2-a5fc-e53ad196a8ba` | ✅ |
| `Problema` | (dentro del hub / sección) | — | verificar sección |
| `Propuesta` | `Propuesta` | `3d4d0445-9756-80cf-9b5e-cde4dd944c7c` | ✅ |
| `Roadmap` | `Roadmap, changelog y criterios de release` | `3dbd0445-9756-81ee-b7fd-e541a545c526` | **renombrada** |
| `SDKs y Quickstarts` | `SDKs y Quickstarts (Python + TypeScript)` | `3dbd0445-9756-8134-8746-d5ce6088f63c` | **renombrada** |
| `Benchmarks` | `Cómo Construir Benchmarks para VantaDB: Guía Completa 2026` | `3d6d0445-9756-801f-ad5f-cf9d7560f915` | **renombrada** |
| `Seguridad` | `Seguridad de la memoria` | `3dbd0445-9756-81ce-8fd7-e96e0b1c4a99` | **renombrada** |
| `Observabilidad` | (¿`Modulos`? verificar) | — | verificar |
| `Gobernanza` | (verificar) | — | verificar |
| `Casos de uso` | (verificar) | — | verificar |
| `Definición oficial` | **NO existe** | — | crear o fusionar en hub (decisión owner) |
| higiene | `VantaDB Docs (1)` `3dbd0445-9756-8046-9d38-d0e6b04f2f80` + `VantaDB OLD` ×2 (`224d0445-…44c3`, `3d0d0445-…cc21`) | | archivar |

## 1. Página `Problema` — agregar

**Añadir sección: "Evidencia 2026 (investigación integral)"**

- **Autoenvenenamiento — evidencia académica y de ataque:**
  - `AgentPoison` (NeurIPS 2024): backdoor vía memoria/KB envenenada en agentes RAG — ≥80% ASR con <0.1% de poison rate. https://arxiv.org/abs/2407.12784
  - `MINJA` (NeurIPS 2025): inyección de memoria con **queries normales**, sin privilegios. https://arxiv.org/html/2503.03704v2
  - Implicación: la cuarentena/abstención de MGR-13 no es research especulativa; es mitigación de ataques documentados. **[REAL 2026-09-30] SCH-05 implementada** (default-exclude + gates de inyección + abstención explícita).
- **Ventana ≠ memoria — tesis refinada 2026:**
  - `Lost in the Middle` (Liu et al., TACL 2024): el rendimiento cae en U según posición del contexto. https://arxiv.org/abs/2307.03172
  - Letta Filesystem: 74.0% en LoCoMo **solo con archivos** (gpt-4o-mini) — la capacidad del agente importa más que la herramienta de retrieval. https://www.letta.com/…
  - Implicación: el problema no es "falta vector DB"; es gobierno del ciclo de vida + formatos que el agente sabe usar (file-native, bloques con presupuesto). **[REAL] VER-06** (export MD git-friendly) + **[REAL] VER-04** (budget de inyección).
- **Benchmarks del sector están quemados (credibilidad como sub-problema):**
  - Auditoría de LoCoMo: 6.4% del answer-key erróneo; el juez acepta hasta 63% de respuestas incorrectas.
  - Lo que ningún benchmark mide: quality of writes, forgetting/consolidación, aislamiento multi-usuario, economía de tokens. **[REAL 2026-09-30] VER-08 lo mide** (write-quality 1.0, aislamiento 0/2500, tokens, ECE).
- **Dimensiones 6 (confianza) y 8 (meta-memoria):** **[REAL] MGR-12 + SCH-04** (confianza asserted/derived); N-07/N-08/N-09 siguen abiertas.

## 2. Página `Propuesta` — actualizar matriz (2026-09-30)

- `[REAL]` v0.7.0: núcleo embebido, WAL, híbrido BM25+HNSW+RRF, CRUD/TTL/supersession, grafo+PageRank, IQL (JOIN), SDKs py/ts/node/wasm, server, MCP.
- `[REAL 2026-09-30 — post-campaña]`: `query_sparse` + text-only en 3 bindings (WIRE-03) · perfil MCP enforceado + fusión 79 (WIRE-02) · TTL server + sweeper (WIRE-04) · entity linking + RRF boost (WIRE-05) · batching segmentado (WIRE-06, 5.92×) · schema v2 bitemporal+confianza+cuarentena (SCH-02) · `AS OF` IQL v2 (SCH-03) · hash-chain WAL + `vanta-cli verify` (VER-01) · borrado certificado (VER-02) · redacción-on-write + AEAD (VER-03) · importadores Mem0/Zep/Letta (VER-05) · export MD (VER-06) · dreams promote real (VER-07) · harness de evals (VER-08) · governance de inyección (VER-04).
- `[PARCIAL]`: scheduler con host · cost-tracking output (WIRE-01) · bench §2 (DEF-06 ✅ regenerado).
- `[PROPUESTA]`: `auto_resolve_entities`, `detect_conflicts`, `extract_skills` (v1.0).
- **Corte 0.8.0 preparado** (ADR-046 firmado; guía UPGRADE.md §0.8.0; release-plz con marcador breaking).

**Anexo A (specs):** research-docs MGR-10/12/13 cerrados con Cierre MGR ✅ (enlazar `docs/dev/research/mgr-1*.md`).
**Anexo B (benchmarks propios):** **[REAL] VER-08** — protocolo = pares accuracy+tokens, write-quality, abstención, aislamiento; LongMemEval-S subset MIT commiteado; `recall_all@5 0.7617` / `recall_any@5 0.9213`; ECE 0.0003 (T*=5.63).
**Anexo C (primitiva estrella):** **Memory Contracts** — schema versionado + retención + conflicto + tenancy en metadata (PROPUESTA v1.0).
**Tabla "qué copiamos de quién":** Mem0 (ADD-only, entity linking, Dream ✅ VER-07) · Zep/Graphiti (bitemporalidad ✅ SCH-02) · Letta (bloques con presupuesto ✅ VER-04).

## 3. Página `Roadmap, changelog y criterios de release` — agregar

- **Fases:** P52 Verificabilidad ✅ (VER-01..08) · P53 Esquema ✅ (SCH-01..08, corte **0.8.0** preparado) · P54 Tracks ICP ✅ (ICP-01..03) · P55 Frontera de producto ✅ (DEF-01..08) · P56 Anuncio (VER-09/EXE-01/N-17 en cierre).
- **v0.8.0 = governance manual + migración única de esquema** (bitemporal+confianza+cuarentena); v1.0 = automática + federación.
- **Gates de anuncio:** EXE-01 (demos CI) + VER-09 (head-to-head publicado) — en cierre de campaña.

## 4. Página `SDKs y Quickstarts (Python + TypeScript)` — agregar

- **Paridad [REAL]:** `query_sparse` y text-only en los 3 bindings ✅ · filtros avanzados ✅ · **campos v2** (bitemporal+confianza+quarentena) + params (`AS OF`/`valid_window`/`min_confidence`/`include_quarantined`) en las 8 superficies ✅ (SCH-07).
- **Adapters:** 9 paquetes listos, **publicación PyPI pendiente (lane owner; 9/9 → 404 live 2026-09-30)**; importadores Mem0/Zep/Letta→VantaDB ✅ (VER-05) + 3 guías de migración.
- Nota de naming: congelar nombres actuales hasta 1.0 (DEF-04 ✅ ADR-045).

## 5. Página `Cómo Construir Benchmarks…` — agregar

- **Caveats del sector:** leer scores en pares (accuracy+tokens); exigir protocolo (modelo-juez, stack, reranking); LoCoMo auditado (6.4%/63%).
- **[REAL 2026-09-30] Harness propio VER-08:** `evals/memory_harness.py` + `calibration.py`; LongMemEval-S (MIT) subset commiteado con pins; números publicados en `BENCHMARKS.md` §19 (recall_all@5 0.7617 · p99 2.446ms · write-quality 1.0 · ECE 0.0787→0.0003). §2 regenerada (DEF-06 ✅) con hardware/protocolo.
- **Head-to-head VER-09** (Mem0/Zep/Letta, protocolo publicado) — en cierre.
- **Lo que medimos y nadie mide:** calidad de escritura, olvido/consolidación, aislamiento, economía de tokens ✅.

## 6. Página `Seguridad de la memoria` — agregar

- **Threat model write-time** (AgentPoison/MINJA): validación en ingesta + cuarentena + trust-aware retrieval ✅ (SCH-05).
- **Audit WORM:** hash-chain sobre el WAL (VER-01 ✅) + certificado de purga (VER-02 ✅) + audit de inyección `outcome=denied` (VER-04 ✅).
- **Redacción-on-write + namespaces cifrados (AEAD)** ✅ (VER-03): 0 PII en store/índices/export; auditoría PII binary-safe fail-closed.
- P0 previos: sandbox export/import/snapshots ✅ (WIRE-09); `/snapshot` con auth ✅ (API-05).

## 7. Página `Observabilidad` — agregar

- **Token-economy por retrieval** (tokens/llamada, p50/p99) ✅ (VER-08/N-16).
- **Cost tracking del proxy:** output tokens ✅ (WIRE-01).
- Guardrails North Star: 0 hallazgos high sin parche ≤7d; 0 regresión p99 >15% (workflow informativo + Q5 owner-open).

## 8. Página `Gobernanza` — agregar

- **Namespaces trusted/tainted + RBAC por acción** (MGR-04): PROPUESTA (FIND-196… pendiente research).
- **Memory Contracts** (Anexo C) como primitiva de gobernanza por namespace (v1.0).
- **Pipeline de consolidación:** dreams con dry-run + diff + promote real ✅ (VER-07; ADD/UPDATE/DELETE/NOOP idempotente).
- **Presupuesto + ACLs + audit de inyección** ✅ (VER-04).

## 9. Página `Casos de uso` — agregar

- **3 tracks ICP con métricas** ✅ (ICP-01/02/03): AI-IDEs vía MCP (`AI_IDES.md`) · local-LLM/privacidad (`PRIVACY.md`) · frameworks (`FRAMEWORKS.md`).
- Cada track: one-pager + demo CI + métrica medible; entrada en `COMPARISON.md` (§7/§8/§9).

## 10. Página `Definición oficial` — crear (decisión owner)

- **Tagline candidata:** "la memoria verificable y gobernable que vive en tu proceso" (complementa "SQLite para agentes").
- **North Star:** agentes activos que recuperan una memoria con éxito en ventana de 7 días (métrica instrumentada ✅ ICP-01).
- **Frontera verificable:** `EXPERIMENTAL_FEATURES.md` regenerada (DEF-02/03 ✅) manda; nada REAL sin evidencia en código.

## 11. Higiene de páginas

- `VantaDB Docs (1)` (`3dbd0445-9756-8046-9d38-d0e6b04f2f80`): duplicado — consolidar en el hub y archivar.
- `VantaDB OLD` ×2 (`224d0445-9756-826e-8c85-01e1765a44c3`, `3d0d0445-9756-8098-b82c-ec71fa8acc21`): archivar (histórico).
- `Seguridad de la memoria`: contiene refs stale `v0.5.0`/v0.6.1 — actualizar a 0.7.0/0.8.0-pending (contrato: "0.7.0 no stale").
- Al aplicar todo: registrar fecha de sync en este archivo.

## 12. Proceso de aplicación (checklist N-17)

1. [x] `Problema` §1 · 2. [x] `Propuesta` §2 · 3. [x] `Roadmap…` §3 · 4. [x] `SDKs…` §4 · 5. [x] `Benchmarks…` §5 · 6. [x] `Seguridad de la memoria` §6 · 7. [x] `Observabilidad` §7 · 8. [x] `Gobernanza` §8 · 9. [x] `Casos de uso` §9 · 10. [x] `Definición oficial` §10 (crear) · 11. [x] Higiene §11.
> Al cerrar cada página: verificación de claims contra código (Regla 11) + nota de fecha + link cruzado al doc del repo correspondiente.
> **Ejecución:** Notion MCP (`notion-update-page`/`notion-create-pages` disponibles; lane owner para el workspace).

## 13. Sync EJECUTADO — 2026-09-30 (aprobación owner via question)

- **9 páginas actualizadas** (append, sin borrados): `Problema` (`3d4d0445-…e7b7`) · `Propuesta` (`…44c7c`) · `Roadmap…` (`…c526`) · `SDKs…` (`…f63c`) · `Benchmarks…` (`…f915`) · `Seguridad de la memoria` (`…a99`) · `Observabilidad…` (`…649e`) · `Gobernanza…` (`…adb`/`…f39` según mapeo) · `Casos de uso` (`…d4b`).
- **Creada:** `Definición oficial` → `3ebd0445-9756-81bb-a6f2-c3b67157b866` (bajo el hub).
- **Higiene:** `[ARCHIVED 2026-09-30] VantaDB Docs (1) — ver hub` (`…2f80`) · `[ARCHIVED 2026-09-30] VantaDB OLD (Plantilla)` ×2 (`…44c3`, `…cc21`).
- **Nota histórica:** `Seguridad de la memoria` conserva su informe verbatim 2026-09-08 (refs v0.5.0 históricas) + sección "Estado 2026-09-30" que marca el estado vigente (0.7.0 / 0.8.0-pending).
- **Estado:** N-17 ejecutado; verificación independiente pendiente (reviewer).
