---
title: "MGR-23/24 — Memoria de proyecto/ingeniería: taxonomía de lo memorable, ADRs/convenciones y linker decisión→código→test"
kind: research
description: "Spec de MGR-23 (detección de cambios significativos + ADRs MADR con aprobación humana + memoria de convenciones) y MGR-24 (linker issue↔commit↔test + post-mortems), con la taxonomía de lo memorable declarada y la industria validada (ICSA-2024, DRAFT-2025, LinkAnchor, KGCompass)"
---

# MGR-23/24 — Memoria de proyecto/ingeniería: taxonomía de lo memorable, ADRs/convenciones y linker decisión→código→test

- **Fecha:** 2026-10-05 · **Tipo:** research/spec (MEMG-09, Task 53; cero implementación en este run — specs + diseño verificable)
- **Contrato (plan Task 53 L1529):** "taxonomía de lo memorable declarada (mapa, convenciones, ADRs, historia, dependencias, deuda, patrones de fallo); grafo decisión→código→test (linker issue↔commit↔test) como diseño verificable"
- **Origen:** `../Backlog.md` filas MGR-23 (PI-2) y MGR-24 (PI-3); Notion track PI (no accesible a workers — el resumen del Backlog manda)
- **Estado:** spec ✅ por este doc · **Implementaciones:** fuera de este run (los MGR-23/24 son "investigación + spec" por contrato) · **Consume:** MGR-01, MGR-07 (skills), MGR-22 (repo-map — doc hermano `mgr-22-repo-map.md`), MGR-05 (entidades, dep declarada de MGR-24)

## §0. Resumen ejecutivo

La memoria de proyecto/ingeniería de VantaDB tiene hoy piezas reales pero **desconectadas**: los ADRs viven como archivos MADR escritos a mano ([`../architecture/adr/`](../architecture/adr/README.md), 56 ADRs), las convenciones viven en `AGENTS.md`/`rules` (docs vivos sin detector de deriva), los links issue↔commit viven como texto en mensajes de commit (sin recuperación ni consulta) y los patrones de fallo viven en tests (`tests/`, `vanta-memory/tests/`) y reviews, sin loop de post-mortem.

Este doc entrega lo que el contrato pide para el "track PI restante":

1. La **taxonomía de lo memorable declarada** (§1) — las 7 categorías y su superficie/mecanismo/estado.
2. La **spec de MGR-23** (§2): detección de cambios arquitectónicamente significativos (git diff → juez LLM), generación de ADRs en formato MADR con **aprobación humana** (patrón dream approve/reject), ventana de contexto 3–5 ADRs y supersession; memoria de convenciones versionada con detector de deriva.
3. La **spec de MGR-24** (§3): **linker issue↔commit↔test** con reglas de resolución ordenadas, modelo de grafo (edges `fixes`/`tested_by`), recuperación de links faltantes como asistencia (no auto-archivo) y loop de post-mortems — como **diseño verificable** (plan de validación con split temporal en §3.4).

## §1. Taxonomía de lo memorable (declarada)

| # | Categoría | Qué es memorable | Superficie actual | Mecanismo objetivo | Estado |
|---|-----------|------------------|-------------------|--------------------|--------|
| 1 | **Mapa** | estructura del repo, stack, comandos, owners | inexistente como artefacto | `repo-map` (MGR-22): grafo file-per-node + scene `repo-map` | slice v0 ✅ (`code_index`); scene → FIND-299 |
| 2 | **Convenciones** | estilo, lints, AGENTS.md, runbooks | `AGENTS.md`, `.opencode/rules/`, frontmatter de docs | store versionado + detector de deriva (§2.4) | spec ✅ · impl pendiente |
| 3 | **ADRs** | decisiones arquitectónicas con alternativas y consecuencias | `docs/dev/architecture/adr/` (MADR, escritura humana) | `adr_propose`/`adr_approve` + supersession (§2.1–2.3) | spec ✅ · impl pendiente |
| 4 | **Historia** | commits, issues, PRs, sesiones | git log + threads/scenes/dream (memoria) | linker issue↔commit↔test en el grafo (§3) | spec ✅ · impl pendiente |
| 5 | **Dependencias** | grafo de módulos/crates/imports | `Cargo.toml`/`package.json` + edges `defines` (v0) | edges `imports`/`calls` (MGR-22 v1) + manifests ingestados | parcial · v1 pendiente |
| 6 | **Deuda** | hallazgos, shortcuts, deuda diferida | `../Backlog.md` (FIND-*), comentarios `ponytail:`, skills `ponytail-debt` | deuda consultable por grafo (FIND→archivo→símbolo) | parcial (texto) · wiring v2 |
| 7 | **Patrones de fallo** | bugs, incidentes, regresiones, sus fixes+tests | tests de caos (`tests/chaos_integrity.rs`, `vanta-memory/tests/scheduler_crash.rs`), reviews | post-mortem loop + edges `fixes`/`tested_by` (§3.3) | spec ✅ · impl pendiente |

**Principio de arquitectura (transversal):** un hecho, un artefacto fuente. El grafo/memoria **proyecta** (no duplica) lo que ya vive en git/docs; la fuente de verdad sigue siendo el archivo/commit (patrón PROJECTMEM: log append-only + proyección regenerable).

## §2. MGR-23 — ADRs automáticas + memoria de convenciones

### 2.1 Detección de cambios significativos

- **Señales de candidatura (git diff → score):** cambios en `docs/api/**`, `src/sdk/**` (frontera pública), features Cargo con impacto de build, nuevas deps, cambios de schema/storage, migraciones, cambios en `AGENTS.md`/rules. La lista es configurable y **auditable** (cada ADR propuesto cita las señales que lo dispararon).
- **Juez LLM:** clasificador sobre el diff + contexto. Evidencia de industria: ICSA-2024 muestra GPT-4 0-shot **relevante pero sub-humano** para decidir "¿este cambio amerita ADR?"; few-shot/RAG (DRAFT-2025) mejora la generación. Conclusión: el juez **propone**, nunca archiva.
- **Trigger:** post-commit hook (batch nocturno como fallback); idempotente por `(commit_range, señal)`.

### 2.2 Generación y ventana de contexto

- **Ventana 3–5 ADRs previos** (localidad temporal + conceptual) para el prompt de generación — "Context Matters" (2026) midió Last_K 3–5 ≈ All_N para esta tarea; presupuesto acotado (spec MGR-22 §3.5).
- **Formato MADR** (el del repo: Contexto → Opciones consideradas (≥2) → Decisión + consecuencias positivas/negativas) — un ADR sin alternativas no es ADR (regla del repo).
- **Evidencia adjunta:** el draft cita archivos/símbolos del grafo MGR-22 (`code_index`) y los commits que lo dispararon.

### 2.3 Aprobación humana, archivo y supersession

- **Patrón `dream`:** `adr_propose` es read-only (preview, como `dream_promote` hoy: `vantadb-mcp/src/dreams.rs` `{preview_count, mutated:false}`) y `adr_approve` es el único path que muta — escribe el archivo en `docs/dev/architecture/adr/`, lo registra en el grafo y dispara `gen-index`. Nunca auto-archivo (mismo invariante que dream/approve).
- **Supersession:** un ADR nuevo nombra al reemplazado (`supersedes:`/`superseded_by:` frontmatter + edge `superseded_by`), inmutable una vez aceptado (regla MADR del repo).
- **API propuesta:** `adr_propose {diff_range, signals?}` → `{draft_id, madr_markdown, evidence[]}` · `adr_approve {draft_id, number?}` → `{path, adr_node_id}`. Perfil `full` (writer).

### 2.4 Memoria de convenciones + detector de deriva

- **Store versionado:** convenciones como records (namespace `conventions`; escrita por humano/agente aprobador) con `metadata.source={file}` y versión — reutiliza el mecanismo de memoria, sin store nuevo.
- **Detector de deriva:** diff programático entre convención declarada y artefacto (p. ej. regla de estilo ↔ lint/fmt config; "no unwrap en prod" ↔ grep; comando documentado ↔ Justfile) → reporte (read-only) con severidad. Las convenciones no se auto-modifican.
- **Consumo:** inyección en context (patrón OpenHands `repo.md`, ver doc MGR-22 §2).

## §3. MGR-24 — Trazabilidad bug→fix + minería de patrones

### 3.1 Linker issue↔commit↔test (reglas de resolución, ordenadas)

| Prioridad | Regla | Fuente | Confianza |
|-----------|-------|--------|-----------|
| 1 | Trailers/body de commit: `Closes #N`, `Fixes #N`, `Refs #N` | git log (offline) | alta (explícito) |
| 2 | Convención de tests: archivo/nombre de test referencia el símbolo/módulo + edges tipo `TESTS` | repo + grafo MGR-22 | media (heurística → validable) |
| 3 | PR body / metadata (GitHub API, opcional v2) | red (opt-in) | alta pero requiere red/credenciales |
| 4 | **Recuperación de links faltantes** (candidatos ranked, asistida) | grafo commits+tests (patrón LinkAnchor: agente + búsqueda en grafo de commits; KGCompass: multi-hop) | **baja por diseño → requiere confirmación humana** |

- **Nunca auto-vincula** los casos 2/4: propone candidatos (Hit@1) que un humano/agente aprueba (mismo patrón propose/approve que MGR-23).
- **Split temporal obligatorio** para cualquier evaluación/minería aprendida (LinkFormer): train/eval por tiempo, nunca por random.

### 3.2 Modelo de grafo (decisión→código→test)

Records + edges en un namespace `history` (mecanismo MGR-22, sin core nuevo):

- Nodos: `issue:{n}`, `commit:{sha}`, `test:{path::test_name}`, `postmortem:{id}`.
- Edges: `fixes` (commit→issue), `tested_by` (código→test), `guards` (test→issue/bug), `superseded_by` (ADR), `postmortem_of` (postmortem→issue/commit).
- Anclaje a código: reutiliza los nodos de `code_index` (file/symbol) para unir decisión↔código (p. ej. `commit` → archivo tocado → símbolo MGR-22).

### 3.3 Loop de post-mortems

- Incidente/bug con fix → registro estructurado (qué se rompió, causa raíz, fix, test guard, tiempo a detección/MTTR) → record en `history` + edges `fixes`/`guards`.
- Retrieval: similitud + grafo (bug nuevo → fixes análogos y tests guard existentes) — el caso "soluciones de referencia" del Backlog.
- Alimenta "patrones de fallo" (§1 #7) y prioriza tests de caos dirigidos.

### 3.4 Plan de verificación (diseño verificable)

1. **Muestra etiquetada:** ≥50 links reales del historial del repo (tráilers parseados como ground truth parcial + revisión manual de una sub-muestra).
2. **Métricas:** precision/recall y Hit@1 por regla (1–4) y combinadas; baseline = solo regla 1.
3. **Criterio de aceptación:** reglas 1–2 con precision ≥0.95 (bajo costo de falso positivo: solo consulta); recuperación (4) reportada como asistencia (Hit@1@k), sin gate de precision.
4. **Reproducibilidad:** el harness de evaluación corre offline sobre el repo (sin red) para reglas 1–2; cualquier componente con red o aprendido declara split temporal.

## §4. Interacciones y dependencias

- **MGR-22 → MGR-23/24:** el grafo file-per-node (`code_index`) es el anclaje "código" de ambos (evidencia de ADRs; símbolos tocados por commits).
- **MGR-07 (skills):** el store de convenciones reutiliza la forma `SkillRecord` (MCP `skill_*`) como precedente, sin acoplarse.
- **MGR-05 (entidades):** dep declarada de MGR-24 en Backlog; el linker no la requiere para v1 (issues/commits/tests son nodos propios), la aprovecha cuando exista.
- **FIND-120:** la escena repo-map/onboarding consume MGR-22; los post-mortems consumen FIND-296/297-style rows como insumo (deuda/flujo ya registrado).

## §5. Cobertura del DoD y próximos pasos

| MGR | DoD (Backlog) | Este doc |
|-----|----------------|----------|
| MGR-23 | spec (`adr_propose`/`adr_approve`, convenciones versionadas) + research-doc + Cierre MGR | spec ✅ (§2) + research ✅ — Cierre MGR registrable |
| MGR-24 | spec (linker + grafo + post-mortem loop) + research-doc + Cierre MGR | spec ✅ (§3) + research ✅ — Cierre MGR registrable |

**Próximos pasos (implementación, orden sugerido):** (1) MGR-23 detección + `adr_propose` (read-only) sobre el grafo MGR-22; (2) MGR-24 linker reglas 1–2 offline + métricas §3.4; (3) post-mortem loop; (4) convenciones + deriva. Cada bloque, un task con tests (TDD) como cualquier feature.

## §6. Límites declarados

- Sin cambio de core en este run; todo el diseño reutiliza mecanismos existentes (memoria + grafo + approve/reject).
- La calidad del juez LLM (MGR-23) se asume sub-humana 0-shot (ICSA-2024): el diseño lo compensa con aprobación humana obligatoria.
- `cADR` y `repo-rag` **NO verificados** (sin fuente canónica localizada 2026-10-05) — no se citan como base; candidato cercano ADR-context: arXiv 2604.03826 (no equivalente a cADR).

## §7. Fuentes

**Industria (verificadas 2026-10-05):**
1. ICSA-2024 — "Can LLMs Generate Architectural Design Decisions?" — https://doi.org/10.1109/icsa59870.2024.00016
2. DRAFT-2025 (ADR generation, RAG/few-shot) — https://arxiv.org/abs/2504.08207
3. Context Matters — context strategies for ADR generation (Last_K 3–5 ≈ All_N) — https://arxiv.org/abs/2604.03826
4. LinkAnchor (FSE 2026, commit-graph agent) — https://arxiv.org/abs/2508.12232 · https://github.com/ISE-Research/LinkAnchor
5. LinkFormer — https://doi.org/10.5281/zenodo.6524460
6. KGCompass (KG multi-hop repair) — https://github.com/GLEAM-Lab/KGCompass · (candidato paper https://arxiv.org/abs/2503.21710, confianza media)
7. EALink (ASE 2023) — https://arxiv.org/abs/2308.10759

**Internas:** `../architecture/adr/README.md` (MADR del repo), `vanta-memory/src/core/dream/` (patrón approve/reject), `vantadb-mcp/src/dreams.rs` (preview read-only), `../Backlog.md` (MGR-23/24, FIND rows), `../tasks/MEMG-09.md`, `mgr-22-repo-map.md` (doc hermano).
