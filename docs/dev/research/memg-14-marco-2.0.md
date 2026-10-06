---
title: "MEMG-14 — Marco 2.0: núcleo + extensiones (reformulación del marco 8/6/10/PI)"
kind: research
status: active
description: "Marco reformulado 'núcleo + extensiones' (7 sub-decisiones a-g), respaldo de dims/áreas/ámbitos en repo, validaciones externas y cadencia anual; detalle Notion citado por página"
tags: [vantadb, research, estrategia]
---

# MEMG-14 — Marco 2.0: núcleo + extensiones (reformulación del marco 8/6/10/PI)

- **Fecha:** 2026-10-05 · **Tipo:** research/spec (MEMG-14, Task 60, F5) · **Cero implementación** (este run no genera código ni símbolos)
- **Contrato (plan Task 60 L1734):** marco reformulado ("núcleo + extensiones") con las 7 sub-decisiones del backlog (a)-(g); revisión documentada; respaldo en `docs/` con enlaces a evidencia.
- **Origen:** validación externa 2026-09-30 (4 research, ≈100 fuentes, ~50 fetch-verificadas; registrada en `../Backlog.md:120,140`; sync en [`../strategy/NOTION-SYNC-2026-09-24.md`](../strategy/NOTION-SYNC-2026-09-24.md) §14, commits `076d2bb1`/`fc3ec237`) + marco núcleo en repo (`../../../SPEC.md:9`, `../Backlog.md:776`).
- **Alcance:** reformular las 7 sub-decisiones sancionadas — **no rediseñar el marco** (pre-mortem #1 del plan). El detalle de cada página Notion no se transcribe: se cita por ID (patrón NOTION-SYNC) y lo no verificable se marca `[a verificar — Notion]`.

## Contenido

[§1 Marco núcleo (respaldo)](#1-marco-núcleo-respaldo-verificado) · [§2 Reformulación (a)-(g)](#2-reformulación-núcleo--extensiones-7-sub-decisiones) · [§3 Registro de validaciones externas](#3-registro-de-validaciones-externas-de-apuestas-propias) · [§4 Cadencia de re-validación](#4-cadencia-de-re-validación) · [§5 Draft de sync a Notion](#5-draft-de-sync-a-notion-pendiente-de-aplicación--lane-owner) · [§6 Límites y deuda](#6-límites-y-deuda) · [§7 Fuentes](#7-fuentes)

## §0. Resumen ejecutivo

El marco de memoria de VantaDB (8 dimensiones · 6 áreas · 10 ámbitos · track PI) nace en el programa P49 (2026-09-14, `../Backlog.md:773-776`) y vive **principalmente en Notion** (páginas `Las 8 dimensiones…`, `Las 6 áreas…`, `Los 10 ámbitos…`, `Memoria de Proyecto e Ingeniería (PI)`). La validación externa 2026-09-30 mostró que la taxonomía **no es cerrada**: un survey de seguridad (EMNLP 2026) propone un lifecycle de 6 fases distinto —con Share y Forget/Rollback— y marcos de referencia (DAMA-DMBOK) tratan interoperabilidad como área de primera clase.

**Decisión de presentación:** el marco se publica como **núcleo + extensiones** — el núcleo (D1-D8 / C1-C6 / AM1-AM10 / PI) queda estable y respaldado en repo; las candidatas (áreas 7/8, ámbitos +4, ampliaciones) quedan **abiertas y documentadas con evidencia**; y se registra una lista explícita de **"no elevar"**. Se retira el claim "cerrada en seis" (riesgo de credibilidad, Regla 11).

| Sub-decisión | Qué resuelve | Estado en este doc |
|---|---|---|
| (a) Áreas 7/8 + meta-área observabilidad | Portabilidad/interoperabilidad y compartir/multi-agente como candidatas; observabilidad como meta-área | §2.1 — documentada (candidatas, sin elevación formal) |
| (b) Ámbitos +4 y ampliaciones | 4 candidatos a ámbito; AM1 + abstención/calibración; AM9 + test-time learning | §2.2 — documentada |
| (c) Ejes ortogonales + cadencia L0→L3 | Sustrato/forma y sujeto como ejes; cadencia multiescala real del pipeline | §2.3 — documentada (cadencia ya implementada) |
| (d) Corrección "cerrada en seis" | Retirar el claim; separar dims cognitivas (1-4) de ingeniería (5-8) | §2.4 — corrección aplicada a la presentación |
| (e) Sync + respaldo + registro + cadencia | Este doc (respaldo), §3 (registro), §4 (cadencia), §5 (draft sync) | §2.5 — entregada como draft (aplicación: lane owner) |
| (f) Confianza calibrada + paramétrica | Confianza calibrada = extensión de D6; memoria paramétrica = fuera de alcance | §2.6 — documentada |
| (g) "No elevar" + nota bitemporal | Lista de no-elevación; AM4 con validación externa débil declarada | §2.7 — documentada |

## §1. Marco núcleo (respaldo verificado)

> **Anclas repo (verificadas HEAD 2026-10-05):** `../../../SPEC.md:9` (6 áreas + 8 dims con nombres) · `../Backlog.md:776` (D1-D8 / C1-C6 / AM1-AM10 completos) · `../Backlog.md:773-776` (origen P49 + "validación Fase 1: se mantienen las 8"). **Páginas Notion:** citadas por ID (los IDs abreviados con `…` provienen de NOTION-SYNC §14; el ID completo vive en Notion/sync doc).

### 1.1 Las 8 dimensiones (D1-D8)

| # | Dimensión | Rol en la presentación (d) | Ancla |
|---|-----------|----------------------------|-------|
| D1 | trabajo | Función cognitiva (memoria de trabajo) | `../Backlog.md:776` · `../../../SPEC.md:9` |
| D2 | episódica | Función cognitiva | ídem |
| D3 | semántica | Función cognitiva | ídem |
| D4 | procedimental | Función cognitiva | ídem |
| D5 | temporal | Propiedad de ingeniería | ídem |
| D6 | confianza | Propiedad de ingeniería | ídem |
| D7 | identidad | Propiedad de ingeniería | ídem |
| D8 | meta | Propiedad de ingeniería | ídem (alias "meta-memoria" en `../../../SPEC.md:9`) |

- **Detalle por dimensión (cooperación, precedencia, estados REAL/PARCIAL/PROPUESTA):** `[a verificar — Notion: "Las 8 dimensiones de la memoria" (`3dbd0445-…bc50`)]`. En repo: D5 → [`mgr-10-bitemporalidad.md`](mgr-10-bitemporalidad.md); D6 → [`mgr-12-confianza.md`](mgr-12-confianza.md); D8 → MGR-18 (`../Backlog.md:825`).

### 1.2 Las 6 áreas del ciclo (C1-C6)

| # | Área | Ancla |
|---|------|-------|
| C1 | persistencia | `../Backlog.md:776` · `../../../SPEC.md:9` |
| C2 | recuperación | ídem |
| C3 | selección | ídem |
| C4 | evolución | ídem |
| C5 | gobernanza | ídem |
| C6 | seguridad | ídem |

- **Detalle y claim "seis áreas fundamentales":** `[a verificar — Notion: "Las 6 áreas del ciclo de vida" (`3dbd0445-…3140`)]` — la calificación inline del claim se aplicó en Sync #2 (`../strategy/NOTION-SYNC-2026-09-24.md:136`).

### 1.3 Los 10 ámbitos (AM1-AM10)

| # | Ámbito | # | Ámbito |
|---|--------|---|--------|
| AM1 | hechos | AM6 | identidad |
| AM2 | relaciones | AM7 | contexto |
| AM3 | tiempo | AM8 | consistencia |
| AM4 | sistema-vs-realidad | AM9 | aprendizaje |
| AM5 | vigencia | AM10 | seguridad |

- **Ancla:** `../Backlog.md:776`. **Detalle + candidatas descartadas (§Validación):** `[a verificar — Notion: "Los 10 ámbitos afectados" (`3dbd0445-…d2f6`)]`.

### 1.4 Track PI (Memoria de Proyecto e Ingeniería)

- **Notion:** `Memoria de Proyecto e Ingeniería (PI)` (`3dbd0445-…69ce`) — industria: scopes, budgets, ranking por tarea, hooks, churn de formatos (`../strategy/NOTION-SYNC-2026-09-24.md:141`). **No accesible a workers** — el resumen del Backlog manda (precedente [`mgr-23-24-memoria-proyecto.md`](mgr-23-24-memoria-proyecto.md):11).
- **Respaldo repo:** taxonomía de lo memorable declarada (mapa, convenciones, ADRs, historia, dependencias, deuda, patrones de fallo) en [`mgr-23-24-memoria-proyecto.md`](mgr-23-24-memoria-proyecto.md) §1; fila `MEMG-09` (`../Backlog.md:139`).

## §2. Reformulación: núcleo + extensiones (7 sub-decisiones)

> **Verificación 1:1:** cada subsección (a)-(g) mapea a la sub-decisión correspondiente de `../Backlog.md:140` (fila `MEMG-14`) y al contrato del plan (L1734). "Núcleo" = §1 (estable, respaldado); "extensiones" = candidatas documentadas + lista "no elevar".

### 2.1 (a) Áreas candidatas 7/8 + meta-área observabilidad

- **Decisión:** C7 **portabilidad/interoperabilidad** y C8 **compartir/colaboración multi-agente** quedan como **áreas candidatas** (referencia tentativa C7/C8; sin elevación formal). **Meta-área observabilidad/evaluación**: transversal al ciclo, no una fase más.
- **Evidencia:**
  - W3C Community Group *AI Agent Memory Interoperability* (charter v1.0, adoptado 2026-06-19; fetch-verificado 2026-10-05): cell portable con metadata canónica, identity binding post-cuántico (ML-DSA-65/FIPS-204), **sharing contracts** (temporary/permanent/syndicate + revocación), erasure GDPR Art.17, crosswalks NIST/ISO/EU AI Act/MCP — es decir, interoperabilidad y sharing como problemas de primera clase del ecosistema.
  - Survey de seguridad (arXiv 2604.16548, EMNLP 2026; fetch-verificado): fase **Share & Propagate** en su lifecycle.
  - Repo: `MEMG-15` (`../Backlog.md:141`) ya planifica adoptar los drafts de frontera (AAIF/AIMEM/ALF/AMP); la observabilidad tiene instrumentación real (VER-08, `docs/user/operations/MEMORY_TELEMETRY.md`, eventos `proxy-memory-events` de ICP-01, N-12..N-16).
- **Estado:** candidatas documentadas. La **elevación** a área requiere evidencia de producto (no se eleva por literatura) — criterio declarado aquí.

### 2.2 (b) Ámbitos candidatos +4 y ampliaciones

- **Decisión — 4 candidatos a ámbito** (numeración AM11-AM14 tentativa; sin número asignado en el marco):
  1. **privacidad/compliance + portabilidad** — evidencia: W3C CG (erasure GDPR Art.17, cross-vendor migration), `MEMG-17` (rollback + erasure + recibos; [`../avance/activo/vanta-memory.md`](../avance/activo/vanta-memory.md):169-173), VER-03 (redacción-on-write + AEAD).
  2. **economía operacional** — evidencia: pares accuracy+tokens de VER-08; eRAG citado en `MEMG-23` (`../Backlog.md:170`). (Nota: economía **no** se eleva a área — ver §2.7(g).)
  3. **multimodalidad** — evidencia fetch-verificada: arXiv 2602.06052 (survey; mecanismo "sensory"), 2507.07957 (MIRIX), 2512.13564 (frontera multimodal); trigger registrado en `FUT-15` (`../backlog-futuro.md:28`) y decisión+spec **entregadas** en [`memg-18-multimodalidad.md`](memg-18-multimodalidad.md) (2026-10-06; extensión transversal, no 9ª dim).
  4. **multi-agente** — evidencia: fase Share & Propagate del survey; cascadas multiagente como problema declarado (`../../../SPEC.md:9`); sharing contracts W3C.
- **Decisión — ampliaciones:**
  - **AM1 (hechos) + abstención/calibración** — evidencia: SCH-05 (cuarentena + abstención explícita), VER-08 (subscore de abstención).
  - **AM9 (aprendizaje) + test-time learning** — evidencia: arXiv 2507.05257 (MemoryAgentBench; fetch-verificado), STATE-Bench vía `MEMG-23` (`../Backlog.md:170`).
- **Estado:** candidatos documentados; `MEMG-18`/`MEMG-19`/`MEMG-23` (F5) los desarrollan como decisión/spec.

### 2.3 (c) Ejes ortogonales + cadencia multiescala L0→L3

- **Decisión — 2 ejes de presentación (ortogonales al ciclo/dims):**
  - **Eje 1 — sustrato/forma:** token-paramétrico-latente. Ubica la **memoria paramétrica** (Titans/Memory Layers) como fuera de alcance (§2.6).
  - **Eje 2 — sujeto:** single-agente vs multi-agente. Conecta con C8 candidata (§2.1) y el ámbito multi-agente (§2.2).
- **Decisión — cadencia multiescala L0→L3:** L0 conversación cruda → L1 memorias estructuradas (extracción + dedup) → L2 escenas → L3 persona/doctrina + consolidación (dream). **Ya es real en código** (`vanta-memory`): `../../../README.md:37,229`; host del planificador: [`ADR-0054`](../architecture/adr/ADR-0054-scheduler-host-vantadb-server.md); tareas WIRE-14..18; estado por superficie: `../../user/operations/EXPERIMENTAL_FEATURES.md` §Scope Budget.
- **Estado:** cadencia implementada (con residuales declarados: scheduler host v1 = HTTP; dream timers sin productor); ejes = lente de presentación documentada.

### 2.4 (d) Corrección "cerrada en seis" + separación cognitivas/ingeniería

- **Decisión:** retirar el claim de taxonomía "cerrada en seis" / "seis áreas fundamentales" (calificado inline en Sync #2) y presentar el marco como **núcleo + extensiones**. Separar en la presentación:
  - **D1-D4 = funciones cognitivas** (trabajo, episódica, semántica, procedimental — taxonomía clásica de memoria).
  - **D5-D8 = propiedades de ingeniería** (temporal, confianza, identidad, meta — cómo el sistema las garantiza).
- **Evidencia de que la taxonomía no es cerrada:** arXiv 2604.16548 (6 fases: Write, Store, Retrieve, Execute, **Share & Propagate**, **Forget & Rollback** — fetch-verificado, incluye fases ausentes del ciclo C1-C6); DAMA-DMBOK 2ª ed. (fetch-verificado: "data integration & interoperability" entre sus contenidos/knowledge areas); registro de la validación (≈100 fuentes) en `../Backlog.md:140`.
- **Estado:** corrección aplicada a la presentación (este doc + §5 draft). El núcleo no cambia: solo se retira el claim de cierre.

### 2.5 (e) Sync Notion + respaldo repo + registro de validaciones + cadencia anual

- **Decisión (4 piezas):**
  1. **Respaldo al repo** = este doc (marco núcleo §1 + reformulación §2 + límites §6).
  2. **Sync a Notion** = draft append-ready en §5 (pendiente de aplicación — lane owner; Sync #2 requirió aprobación owner via question).
  3. **Registro de validaciones externas** de apuestas propias = §3.
  4. **Cadencia anual de re-validación** = §4.
- **Evidencia:** Sync #2 ejecutado y verificado (6/6 notas + 2/2 calificaciones inline, `../strategy/NOTION-SYNC-2026-09-24.md:132-144`); pendiente diferido a este run: nota de `Seguridad de la memoria` (MEMG-17) — incluida en el draft §5.

### 2.6 (f) Confianza calibrada como extensión + memoria paramétrica fuera de alcance

- **Decisión — confianza (D6):** el núcleo tiene confianza por registro (SCH-02/04: `confidence`/`confidence_class` + `min_confidence`; [`mgr-12-confianza.md`](mgr-12-confianza.md)). La **confianza calibrada** (ECE/temperatura) es **extensión** de D6, no una dimensión nueva: evidencia VER-08 (ECE 0.0003, T*=5.63 — `../strategy/NOTION-SYNC-2026-09-24.md:57`). Fórmula de decaimiento sin canónico validado → **N-09 abierto** (`../backlog-notion.md:28`) — se declara, no se inventa.
- **Decisión — memoria paramétrica (Titans / Memory Layers):** **fuera de alcance**; ubicada en el eje sustrato (§2.3). Sin implementación ni spec en este run (no sancionada). Evidencia de la decisión: `../Backlog.md:140(f)` (validación externa 2026-09-30; sin URL registrada en repo → si se cita en público, fetch primero — Regla 11).
- **Estado:** documentada.

### 2.7 (g) Lista "no elevar" + nota bitemporal

- **Decisión — lista "no elevar"** (candidatas evaluadas y NO elevadas):
  - **Idempotencia / effect semantics** — sin respaldo estable en la validación.
  - **Economía** — transversal de gobernanza, **no** área (sí permanece como ámbito candidato AM12, §2.2).
- **Decisión — nota bitemporal (ámbito #4):** AM4 (sistema-vs-realidad) tiene **validación externa débil** declarada (`../strategy/NOTION-SYNC-2026-09-24.md:140`, "nota #4"). Contexto repo: bitemporalidad **implementada** (SCH-02: `valid_at`/`invalid_at` + `AS OF`; [`mgr-10-bitemporalidad.md`](mgr-10-bitemporalidad.md)); ranking temporal sin resolver → N-08 (`../backlog-notion.md:27`). La debilidad es de *respaldos externos del ámbito*, no del mecanismo → se declara y se re-evalúa en la cadencia (§4).
- **Estado:** documentada.

## §3. Registro de validaciones externas de apuestas propias

> Decisión (e): registrar dónde la validación 2026-09-30 encontró **equivalencia externa** de apuestas propias. Criterio: apuesta → equivalente externo → evidencia → estado. Donde la evidencia registrada no tiene URL en repo, se dice explícitamente (Regla 11: no se inventa cita).

| Apuesta propia | Equivalente externo (validación 2026-09-30) | Evidencia repo | Estado |
|---|---|---|---|
| Consolidación tipo **dream** (sleep-time) | OpenAI dreaming · Letta sleep-time agents | VER-07 (dream promote real), ADR-0040; registro `../Backlog.md:140(e)` (sin URL en repo) | ✅ validada externamente; implementación real |
| **Bitemporalidad** | Zep/Graphiti (grafo bitemporal) | SCH-02 (`valid_at`/`invalid_at` + `AS OF` en 8 superficies), [`mgr-10-bitemporalidad.md`](mgr-10-bitemporalidad.md) | ✅ mecanismo validado; **ámbito #4 con respaldo externo débil** (§2.7) |
| **Grafo decisión→código→test** | Sin equivalente combinado; piezas en industria (ICSA-2024, DRAFT-2025, LinkAnchor, KGCompass) | [`mgr-23-24-memoria-proyecto.md`](mgr-23-24-memoria-proyecto.md) §2-§3; `MEMG-09` | 🟡 sin equivalente directo — diferenciador a validar con evidencia de producto |
| **Erasure/rollback verificables** | VMG (arXiv 2604.16548), ChronoMem (arXiv 2607.27773), W3C CG (GDPR Art.17) | MEMG-17 (rollback + erasure DEK + recibos; `../avance/activo/vanta-memory.md:169-173`), VER-01/02 | ✅ validada; firmas ML-DSA-65 fuera de alcance declarado |
| **Interoperabilidad de memoria** | W3C CG + draft IETF `draft-saihm-memory-protocol` + AAIF/AIMEM/ALF/AMP | `MEMG-15` (`../Backlog.md:141`) | 🟡 validada como necesidad; adopción pendiente (F5) |

## §4. Cadencia de re-validación

- **Base:** revisión **anual** (los surveys 2024-26 mueven el mapa — `../Backlog.md:140(e)`).
- **Triggers de revisión anticipada:** (1) survey/estándar nuevo que proponga una taxonomía alternativa al núcleo; (2) cambio de evidencia sobre un candidato (se eleva o se descarta); (3) drift detectado entre el marco público y el estado real del producto.
- **Mecanismo:** actualizar §2/§3 de este doc (fecha + evidencia) → actualizar el draft §5 → aplicar sync (lane owner) → registrar en avance. Si una decisión genera trabajo, crear fila en Backlog (vía orchestrator).
- **Relación con N-11:** `N-11` (re-validar "Análisis de cobertura" de las dims tras cada slice IMPL-MGR, `../backlog-notion.md:49`) es **complementaria y más frecuente** — se mantiene; esta cadencia es la revisión *de la taxonomía*, no del estado por dim.

## §5. Draft de sync a Notion (pendiente de aplicación — lane owner)

> **Estado: DRAFT — no aplicado.** Los workers no operan Notion (política `pipeline-full.md` 0c-context; precedente `mgr-23-24:11`); Sync #2 requirió aprobación owner via question. Contenido **append-only**, con marcas y verificación contra repo (Regla 11).

| Página (ID) | Append propuesto |
|---|---|
| `Las 6 áreas del ciclo de vida` (`3dbd0445-…3140`) | Nota "Marco 2.0 — núcleo + extensiones": C7 portabilidad/interoperabilidad y C8 compartir/multi-agente como candidatas + meta-área observabilidad/evaluación; respaldo en repo: `docs/dev/research/memg-14-marco-2.0.md` (2026-10-05). |
| `Las 8 dimensiones de la memoria` (`3dbd0445-…bc50`) | Nota: separación de presentación D1-D4 (cognitivas) / D5-D8 (ingeniería); confianza calibrada = extensión de D6; memoria paramétrica fuera de alcance (eje sustrato); cadencia L0→L3 real (ADR-0054). |
| `Los 10 ámbitos afectados` (`3dbd0445-…d2f6`) | Nota: 4 candidatos (privacidad/compliance+portabilidad, economía, multimodalidad, multi-agente) + ampliaciones AM1 (abstención/calibración) y AM9 (test-time learning); lista "no elevar"; nota #4 (bitemporal) ratificada. |
| `Memoria de Proyecto e Ingeniería (PI)` (`3dbd0445-…69ce`) | Nota: taxonomía de lo memorable respaldada en repo (`mgr-23-24-memoria-proyecto.md` §1); industria ya incorporada en `MEMG-09`. |
| `Propuesta` (`3d4d0445-…44c7c`) | Línea en §"Estado 2026-10-05": marco núcleo+extensiones con respaldo en repo + validaciones registradas + cadencia anual. |
| `Seguridad de la memoria` (`…a99`) | Nota diferida de §14:143 (MEMG-17): rollback semántico + erasure criptográfica (DEK) + recibos verificables; firma ML-DSA-65 fuera de alcance declarado. |

- **Al aplicar:** registrar fecha en `../strategy/NOTION-SYNC-2026-09-24.md` (nueva sección de sync) + verificación post-sync (patrón Sync #2: fetch live de cada nota).

## §6. Límites y deuda

- **Qué NO es este doc:** no es un rediseño del marco (las 7 decisiones estaban sancionadas en el Backlog); no es una nueva validación (registra la de 2026-09-30); no implementa nada.
- **No verificable desde el repo (marcado):** el detalle de cada página Notion (§1, `[a verificar — Notion]`); URLs de dream/sleep-time y Titans/Memory Layers (no registradas en repo — fetch antes de citar en público).
- **Deuda:** ninguna nueva. Si la aplicación del sync o la elevación de una candidata excede este run → fila `FIND-*` (no se hace aquí).

## §7. Fuentes

**Repo (verificadas HEAD 2026-10-05, `64eee3f7`):** `../../../SPEC.md:9` · `../Backlog.md:120,139-141,159-160,170,773-776` · `../strategy/NOTION-SYNC-2026-09-24.md:132-144` · `../backlog-notion.md:26-29,49` · `../backlog-futuro.md:28` · [`mgr-10-bitemporalidad.md`](mgr-10-bitemporalidad.md) · [`mgr-12-confianza.md`](mgr-12-confianza.md) · [`mgr-23-24-memoria-proyecto.md`](mgr-23-24-memoria-proyecto.md) · [`../architecture/adr/ADR-0054-scheduler-host-vantadb-server.md`](../architecture/adr/ADR-0054-scheduler-host-vantadb-server.md) · `../avance/activo/vanta-memory.md:169-173` · `../../../README.md:37,229` · `../../user/operations/EXPERIMENTAL_FEATURES.md`.

**Externas (fetch-verificadas 2026-10-05, TSYS-13):**

| URL | Título (verificado) |
|---|---|
| https://arxiv.org/abs/2604.16548 | A Survey on Long-Term Memory Security in LLM Agents… (6 fases: Write, Store, Retrieve, Execute, Share & Propagate, Forget & Rollback; EMNLP 2026) |
| https://www.w3.org/community/ai-agent-memory-interop/ | AI Agent Memory Interoperability Community Group (charter v1.0, 2026-06-19) |
| https://dama.org/learning-resources/dama-data-management-body-of-knowledge-dmbok/ | DAMA-DMBOK (2ª ed.: data integration & interoperability) |
| https://arxiv.org/abs/2602.06052 | A Survey of Agent Memory in the Second Half… |
| https://arxiv.org/abs/2507.07957 | MIRIX: Multi-Agent Memory System for LLM-Based Agents |
| https://arxiv.org/abs/2512.13564 | Memory in the Age of AI Agents |
| https://arxiv.org/abs/2607.12385 | PM-Bench: Evaluating Prospective Memory in LLM Agents |
| https://arxiv.org/abs/2606.23459 | TriggerBench: Investigating Prospective Memory for Large Language Models |
| https://arxiv.org/abs/2602.23944 | MemEmo: Evaluating Emotion in Memory Systems of Agents |
| https://arxiv.org/abs/2507.05257 | Evaluating Memory in LLM Agents via Incremental Multi-Turn Interactions |
| https://arxiv.org/abs/2607.27773 | ChronoMem: Version Control and Semantic Rollback for LLM Agent Memory |

**Commits de la validación (registrados):** `076d2bb1` · `fc3ec237` (`../strategy/NOTION-SYNC-2026-09-24.md:134`).
