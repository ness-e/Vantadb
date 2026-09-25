---
title: "Notion Sync 2026-09-24 — adiciones post-investigación (drafts listos para aplicar)"
type: strategy
status: active
tags: [vantadb, notion, sync, investigacion, post-investigacion]
last_reviewed: 2026-09-24
aliases: []
related: [GO_TO_MARKET.md, ../Backlog.md]
---

# Notion Sync — 2026-09-24

> **Propósito:** contenido listo para pegar en las páginas de Notion ("VantaDB Docs") tras la investigación integral 2026-09-24. Ejecuta el owner/agente con Notion MCP; el trabajo queda trackeado como **N-17** en `docs/dev/backlog-notion.md`.
> **Reglas:** cada claim verificado contra código o contra fuente externa citada (Regla 11) · marcas `[REAL]/[PARCIAL]/[PROPUESTA]` siempre · no romper enlaces existentes · fecha en cada sección añadida.
> **Páginas detectadas (2026-09-24, Notion MCP):** hub `VantaDB Docs` · `Propuesta` · `SDKs y Quickstarts (Python + TypeScript)` · `Cómo Construir Benchmarks para VantaDB: Guía Completa 2026` · duplicado `VantaDB Docs (1)` (consolidar) · `VantaDB OLD` (archivar). Las páginas `Problema`, `Roadmap`, `Seguridad`, `Observabilidad`, `Gobernanza`, `Casos de uso`, `Definición oficial` se referencian en el repo — verificar nombres exactos al aplicar.

---

## 1. Página `Problema` — agregar

**Añadir sección: "Evidencia 2026 (investigación integral)"**

- **Autoenvenenamiento — evidencia académica y de ataque:**
  - `AgentPoison` (NeurIPS 2024, ~550 citas): backdoor vía memoria/KB envenenada en agentes RAG — un atacante escribe una mentira en la memoria y controla respuestas futuras. https://papers.nips.cc/paper_files/paper/2024/hash/eb113910e9c3f6242541c1652e30dfd6-Abstract-Conference.html
  - `MINJA` (2025): inyección de memoria con **queries normales**, sin privilegios — la defensa está abierta. https://arxiv.org/html/2503.03704v2
  - Implicación: la cuarentena/abstención de MGR-13 no es research especulativa; es mitigación de ataques documentados (fila SCH-05).
- **Ventana ≠ memoria — tesis refinada 2026:**
  - `Lost in the Middle` (Liu et al., TACL 2024, ~6.5K citas): el rendimiento cae en U según posición del contexto. https://arxiv.org/abs/2307.03172
  - Letta Filesystem: 74.0% en LoCoMo **solo con archivos** (gpt-4o-mini) — la capacidad del agente importa más que la herramienta de retrieval. https://www.letta.com/blog/benchmarking-ai-agent-memory/
  - Implicación: el problema no es "falta vector DB"; es gobierno del ciclo de vida + formatos que el agente sabe usar (file-native, bloques con presupuesto).
- **Benchmarks del sector están quemados (credibilidad como sub-problema):**
  - Auditoría de LoCoMo: 6.4% del answer-key erróneo; el juez (gpt-4o-mini) acepta hasta 63% de respuestas intencionalmente incorrectas. https://dev.to/penfieldlabs/we-audited-locomo-64-of-the-answer-key-is-wrong-and-the-judge-accepts-up-to-63-of-intentionally-33lg
  - Lo que ningún benchmark mide: **quality of writes, forgetting/consolidación, aislamiento multi-usuario, economía de tokens** (mem0.ai/blog/ai-memory-benchmarks-in-2026).
- **Dimensiones 6 (confianza) y 8 (meta-memoria)** siguen sin research específica: N-07 (P(IK)/abstención), N-08 (ranking temporal), N-09 (fórmula de decaimiento), N-10 (benchmark longitudinal de identidad) → enlazar cada una con MGR-18/11/12/19.
- **Direcciones futuras** ya registradas: FUT-15 (ontología + multimodal + memoria ejecutable) con triggers.

## 2. Página `Propuesta` — agregar/actualizar

**Actualizar matriz REAL/PARCIAL/PROPUESTA (fecha 2026-09-24)** con:
- `[REAL]` v0.7.0: núcleo embebido, WAL, híbrido BM25+HNSW+RRF, CRUD/TTL/supersession, grafo+PageRank, IQL (con JOIN), SDKs py/ts/node/wasm, server, MCP (~87 tools), memoria L0→L3 parcial con dreams/skills/scenes, proxy 16 etapas, desktop, embeddings e5 locales.
- `[PARCIAL]`: scheduler con host, `query_sparse` consultable, cost-tracking output, perfil MCP enforceado, TTL en server, bench §2.
- `[PROPUESTA]`: `auto_resolve_entities`, `detect_conflicts`, `extract_skills` (v0.7 manual) y todo el slice automático (v1.0).

**Añadir sección: "Decisiones owner 2026-09-24"**
1. **3 tracks ICP a profundidad**: AI-IDEs vía MCP (métrica: sesiones con put+search misma semana) · devs local-LLM/privacidad (métrica: 0 PII en store) · frameworks (métrica: installs/semana de adapters). → filas P54.
2. **Migración única de esquema en 0.7.0**: bitemporalidad + scores asserted/derived + cuarentena en un solo breaking change. → filas P53.
3. **Harness completo**: canonical_p99 + LoCoMo + LongMemEval-S + BEAM-subset con dataset commiteado + write-quality/abstención/tokens/p99-CI + head-to-head contra Mem0/Zep/Letta. → VER-08/VER-09.
4. **Cobro fase 1**: carriles PayPal + Binance (USDT) + Payoneer→Airtm. → BIZ-10..12 (negocio).

**Anexo A (specs):** al cerrar cada research-doc MGR-10/12/13 con su Cierre MGR, enlazar la spec.
**Anexo B (benchmarks propios):** protocolo = pares accuracy+tokens, protocolo documentado (modelo-juez/stack/reranking), write-quality, abstención, aislamiento por usuario, dataset commiteado.
**Anexo C (primitiva estrella):** **Memory Contracts** — schema versionado + política de retención + política de conflicto + tenancy declaradas en la metadata del namespace (nadie lo tiene; combina huecos 5/7/8 + memory cubes de MemOS).
**Tabla "qué copiamos de quién" (2026):** Mem0 (ADD-only, entity linking, Dream) · Zep/Graphiti (bitemporalidad, invalidación trazable) · Letta (bloques con presupuesto, filesystem, self-editing) · MIRIX (dry-run de dreams, skills desde tool-traces) · Cognee (importadores de rivales, forget API) · ReMe (file-native Markdown+wikilinks) · MemOS (cubes, viewer) · A-MEM (evolución retroactiva de notas) · Qdrant Edge (UpdateOperation log = exponer WAL).

## 3. Página `Roadmap` — agregar

- **Fases P52–P56** (2026-09-24): P52 Verificabilidad (VER-01..09) · P53 Esquema 0.7.0 (SCH-01..08) · P54 Tracks ICP (ICP-01..03) · P55 Frontera de producto (DEF-01..08) · P56 Cableado (WIRE-01..10). Gates: W0 estabilización → W1 APIs → W2 P0 seguridad → W3 cableado → W4 esquema → W5 verificabilidad → W6 ICP+harness → W7 head-to-head + gate Fase A.
- **v0.7 = governance manual + migración única de esquema**; v1.0 = automática + federación.
- **Gates de anuncio:** EXE-03 (Fase A: 5 installs limpias + 2-3 stranger-test + 0 críticos) + EXE-02/VER-09 publicado.

## 4. Página `SDKs y Quickstarts` — agregar

- **Paridad pendiente (post-API-01..09):** `query_sparse` y text-only en los 3 bindings; filtros intercambiables (`$and`/`$or`, range/datetime); score vs distance unificado. → WIRE-03.
- **Adapters:** 9 paquetes listos, publicación PyPI pendiente (MKT-18f); importadores Mem0/Zep/Letta→VantaDB en VER-05.
- Nota de naming: congelar nombres actuales hasta 1.0 (DEF-04).

## 5. Página `Benchmarks` (guía 2026) — agregar

- **Caveats del sector:** leer scores en pares (accuracy+tokens); exigir protocolo (modelo-juez, stack, reranking); LoCoMo auditado (6.4%/63%).
- **Planificado (sin números aún):** regeneración §2 (DEF-06) + harness propio VER-08 + head-to-head VER-09. Mientras tanto, citar `BENCHMARKS.md` §1/§8 (Rust canonical) y no mezclar pipelines.
- **Lo que mediremos y nadie mide:** calidad de escritura, olvido/consolidación, aislamiento por usuario, economía de tokens (N-12..N-16).

## 6. Página `Seguridad` — agregar

- **Threat model write-time** (AgentPoison/MINJA): validación en ingesta + cuarentena + trust-aware retrieval (SCH-05).
- **Audit WORM**: hash-chain sobre operaciones de memoria (VER-01) + certificado de purga (VER-02).
- **P0 pendiente:** sandbox de paths export/import/snapshots + refuse-to-start del proxy (WIRE-09); `/snapshot` con auth (API-05).

## 7. Página `Observabilidad` — agregar

- **Token-economy por retrieval** (tokens/llamada, p50/p99) como métrica de producto (VER-08/N-16).
- **Cost tracking del proxy:** output tokens hoy sin cablear (`record_response_usage`) → WIRE-01.
- Guardrails North Star: 0 hallazgos high sin parche ≤7d; 0 regresión p99 >15%.

## 8. Página `Gobernanza` — agregar

- **Namespaces trusted/tainted + RBAC por acción** (MGR-04).
- **Memory Contracts** (Anexo C) como primitiva de gobernanza por namespace.
- **Pipeline de consolidación:** dreams con dry-run + diff + promote real (VER-07); operaciones ADD/UPDATE/DELETE/NOOP (MGR-08/VER-07).

## 9. Página `Casos de uso` — agregar

- **3 tracks ICP con métricas** (ICP-01/02/03): AI-IDEs vía MCP · local-LLM/privacidad · frameworks.
- Cada track: one-pager + demo + métrica medible; entrada en `COMPARISON.md`.

## 10. Página `Definición oficial` — agregar

- **Tagline candidata:** "la memoria verificable y gobernable que vive en tu proceso" (complementa "SQLite para agentes").
- **North Star:** agentes activos que recuperan una memoria con éxito en ventana de 7 días.
- **Frontera verificable:** la nueva tabla de `EXPERIMENTAL_FEATURES.md` regenerada (DEF-02/03) manda; nada se presenta como REAL sin evidencia en código.

---

## 11. Higiene de páginas

- `VantaDB Docs (1)`: duplicado — consolidar en el hub y archivar.
- `VantaDB OLD`: archivar (marcar como histórico).
- Al aplicar todo: registrar en `backlog-notion.md` N-17 (checklist por página) + fecha de sync.

## 12. Proceso de aplicación (checklist N-17)

1. [ ] `Problema` §1 · 2. [ ] `Propuesta` §2 · 3. [ ] `Roadmap` §3 · 4. [ ] `SDKs y Quickstarts` §4 · 5. [ ] `Benchmarks` §5 · 6. [ ] `Seguridad` §6 · 7. [ ] `Observabilidad` §7 · 8. [ ] `Gobernanza` §8 · 9. [ ] `Casos de uso` §9 · 10. [ ] `Definición oficial` §10 · 11. [ ] Higiene §11.
> Al cerrar cada página: verificación de claims contra código (Regla 11) + nota de fecha + link cruzado al doc del repo correspondiente.
