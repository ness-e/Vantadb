---
title: "MEMG-19 — Prospectiva + descartes documentados (sensorial/emocional)"
kind: research
status: active
description: "Prospectiva/intencional como patrón de uso (working+temporal+procedimental; no almacén) con benchmarks PM-Bench/TriggerBench + descartes sensorial-como-almacén y emocional/motivacional con respaldos fetch-verificados y criterios de revisión; no normativo, sin implementación"
tags: [vantadb, research, estrategia]
---

# MEMG-19 — Prospectiva + descartes documentados (sensorial/emocional)

- **Fecha:** 2026-10-06 · **Tipo:** research (MEMG-19, Task 63, F5) · **Cero implementación** (este run no genera código ni símbolos)
- **Contrato (plan Task 63 L1818):** (a) prospectiva/intencional como patrón de uso documentado (working+temporal+procedimental; no almacén) con benchmarks de referencia (PM-Bench/TriggerBench); (b) descartes sensorial-como-almacén y emocional/motivacional con respaldos + criterio de revisión (emocional: re-evaluar en 6-12m); grep del repo > 0 para ambos.
- **Origen:** fila `MEMG-19` del Backlog (removida al cierre de esta tarea — registro en [`../avance/activo/operaciones.md`](../avance/activo/operaciones.md)) + dims hub §Validación (Notion — citada por ID, sin transcripción) + validación externa 2026-09-30 ([`../strategy/NOTION-SYNC-2026-09-24.md:132-144`](../strategy/NOTION-SYNC-2026-09-24.md)) + marco 2.0 ([`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) §2.2/§2.7).
- **Alcance:** documentar (1) la prospectiva como **patrón de uso** sobre superficies existentes y (2) los **descartes** sensorial/emocional con respaldos y criterios de revisión. **No** implementa, **no** reabre v1.0, **no** re-litiga los descartes registrados. El doc es **no normativo** ("extensión futura" del marco).

## Contenido

[§0 Resumen](#0-resumen-ejecutivo) · [§1 Prospectiva/intencional](#1-prospectivaintentacional--patrón-de-uso-no-almacén) · [§2 Descartes](#2-descartes-documentados) · [§3 Límites y deuda](#3-límites-y-deuda) · [§4 Fuentes](#4-fuentes)

## §0. Resumen ejecutivo

| Pregunta | Respuesta |
|---|---|
| ¿La prospectiva/intencional es un almacén nuevo? | **No** — es un **patrón de uso** sobre working (D1) + temporal (D5) + procedimental (D4); las superficies existentes lo soportan sin tipos/estructuras nuevas (§1.2-§1.3) |
| ¿Por qué no elevarla a dimensión/tipo? | Los benchmarks de referencia miden una **capacidad del agente**, no un store; el survey de referencia no la lista entre sus mecanismos de memoria; la industria la implementa como **intervención** (política), no como tipo (§1.2) |
| ¿Qué benchmarks la miden? | PM-Bench (máx 65.1% F1 — problema abierto), TriggerBench (PM vs RM; fragilidad atencional), Proactive Memory Agent (intervención selectiva: +8.3pp/+6.8pp) (§1.4) |
| ¿Sensorial como almacén? | **Descartado (2026-09-14)** — el material sensorial es procesamiento de entrada (buffer pre-tokenización) o ingesta multimodal → memorias estándar; ningún sistema de referencia tiene store sensorial (§2.1) |
| ¿Emocional/motivacional? | **Descartado hoy** — el estado del arte no es robusto (MemEmo: ningún sistema; PsychoAgent: sin significancia); **re-evaluar en 6-12m** (ventana 2027-03-30 → 2027-09-30) con triggers declarados (§2.2) |
| ¿Cambia algo en código? | **No** — decisión/documentación; cualquier materialización espera caso/trigger (§3) |

## §1. Prospectiva/intencional — patrón de uso (no almacén)

### 1.1 Qué es

**Prospective memory (PM)** = "the ability to execute an intention at a specific future cue or state while other activities are ongoing" — mantener intenciones del usuario, ejecutar intenciones diferidas y monitorear cambios latentes del entorno (PM-Bench, fetch-verificado 2026-10-06). TriggerBench la define complementariamente como "the critical ability to spontaneously recall and act on latent constraints **without direct prompts**" — es decir, lo opuesto al recall retrospectivo reactivo (queries explícitas): el sistema debe *traer* la intención cuando llega el cue, no esperar a que se la pidan.

### 1.2 Decisión: patrón de uso, no almacén

**Decisión:** la prospectiva/intencional se documenta como **patrón de uso** — una composición de memorias existentes (working + temporal + procedimental) más una política de disparo/recall. **No** se agrega tipo, dimensión ni almacén.

**Fundamento (4 puntos):**

1. **El survey de referencia no la lista como store.** El survey 2602.06052 (TMLR) organiza la memoria en sustrato + **mecanismos (sensory, working, episodic, semantic, procedural)** + sujeto — "prospective" no aparece entre los mecanismos de almacenamiento: es una capacidad del agente, no un tipo de memoria.
2. **Los benchmarks miden al agente, no al motor.** PM-Bench y TriggerBench evalúan si el agente *ejecuta* intenciones diferidas bajo carga (sección §1.4) — el fallo es del agente/contexto, no de un store faltante. Ningún resultado de esos benchmarks se resuelve agregando un almacén.
3. **La industria la implementa como intervención.** Proactive Memory Agent (2607.08716) corre un *memory agent* que decide **inyectar un recordatorio o callarse** ("memory as an active intervention mechanism rather than passive retrieval"); la ablación muestra que la intervención selectiva supera a la exposición pasiva del banco, la inyección always-on y el retrieval general. Es política de intervención sobre memorias existentes — no un tipo nuevo.
4. **Las superficies existentes bastan (extensión aditiva).** `MemoryType` ya tiene los portadores naturales de una intención (`Instruction`, `WorkTask`, `WorkMethod`), la ventana temporal existe (bitemporal SCH-02/03) y el recall gobernado existe (auto_recall + budget VER-04). Una dimensión nueva sería una etiqueta sin sustrato técnico (mismo argumento que multimodalidad, [`memg-18-multimodalidad.md`](memg-18-multimodalidad.md) §1.2).

### 1.3 Materialización sobre las superficies existentes (anclas)

| Capa del patrón | Superficie real (verificada worktree 2026-10-06) | Rol en el patrón |
|---|---|---|
| **Working (D1)** | MMD — `vanta-memory/src/context_engine/mmd.rs:3,30` ("The MMD is the agent's working memory for the current task") + checkpoints de tarea — `vanta-memory/src/utils/task_checkpoint.rs:1,74` (MEMG-20; API entregada, consumer pendiente `FIND-288`) | Mantener viva la intención en la tarea en curso; sobrevivir interrupciones (checkpoints `begin`/`advance`/`load`) |
| **Temporal (D5)** | Ventana de validez `valid_at_ms`/`invalid_at_ms` (`src/sdk/types/record.rs:287-297`) + `as_of_ms` en queries (`:402-406`) — SCH-02/SCH-03 (`AS OF` en 8 superficies) | El "cuándo": una intención con due-window queda consultable por ventana ("qué intenciones son válidas ahora") |
| **Procedimental (D4)** | `MemoryType::{Instruction, WorkTask, WorkMethod}` (`vanta-memory/src/core/abstractions/types.rs:31,35,37`) | El "qué hacer": la intención como instrucción/tarea con método asociado |
| **Disparo / recall** | `perform_auto_recall*` (`vanta-memory/src/core/hooks/auto_recall.rs:390-423`) + gobernanza de inyección (VER-04) + scopes `Agent`/`Team` (D22, `:653-657`); timers del scheduler (`vanta-memory/src/services/scheduler.rs:1-16`, `vanta-memory/src/utils/timer_scanner.rs:14,25` — pipeline interno, ADR-0054) | Reinyectar la intención cuando llega el cue (tiempo/estado), bajo presupuesto y governance |

**Lectura end-to-end (patrón):** (1) el usuario/agente expresa una intención ("recordar X cuando Y") → se registra como `Instruction`/`WorkTask` con ventana temporal (D4+D5); (2) el estado de la tarea vive en working memory (MMD/checkpoints, D1); (3) al llegar el cue, el recall gobernado la reinyecta (auto_recall + budget); (4) **la ejecución es del host/agente** — VantaDB provee las primitivas, no ejecuta la intención.

**Límites de la materialización (honestos):** hoy **no existe** una superficie de recordatorio/trigger de usuario (el scheduler dispara timers internos del pipeline `l1_idle`, no cues de usuario); la composición anterior es documentable con las piezas actuales pero su producto (recordatorios) requiere wiring de host — no se wirea especulativamente (patrón `FIND-288`/`FIND-290`: consumidor pendiente, sin acción de código hasta que exista host).

### 1.4 Benchmarks de referencia

| Benchmark (fetch-verificado 2026-10-06) | Qué mide | Hallazgo clave | Lectura para VantaDB |
|---|---|---|---|
| **PM-Bench** — arXiv [2607.12385](https://arxiv.org/abs/2607.12385) (COLM 2026; v1 2026-07-14) | Mantener/ejecutar intenciones diferidas + monitorear cambios latentes (simulación de 7 días, inspirada en el paradigma Virtual Week) | El mejor método (agente GPT-5.4) alcanza **solo 65.1% F1**; ninguna estrategia domina entre modelos | La prospectiva es un **problema abierto del agente** — no se resuelve con más almacenamiento; VantaDB aporta primitivas, el host aporta la política |
| **TriggerBench** — arXiv [2606.23459](https://arxiv.org/abs/2606.23459) (v1 2026-06-22) | Recall proactivo sin prompt + false alarms + robustez atencional (5 dimensiones; controles RM pareados) | **PM ≠ RM**: RM satura hasta 100K tokens, PM **decae con la longitud de contexto**; trade-off precisión-recall; overfit al heurístico "always-remind" | Si se adopta como feature, medir en **pares accuracy + false-alarm** (patrón VER-08: accuracy + economía) — un "siempre recordar" es un anti-patrón medido |
| **Proactive Memory Agent** — arXiv [2607.08716](https://arxiv.org/abs/2607.08716) (v1 2026-07-09) | Intervención activa (recordatorio selectivo vs silencio) sobre agente sin modificar | **+8.3pp** Terminal-Bench 2.0 y **+6.8pp** τ²-Bench; la **intervención selectiva** supera exposición pasiva, always-on, advisor-only y retrieval general; "behavioral state decay" | El valor está en la **política de intervención** (cuándo hablar/callarse), consistente con el recall gobernado + budget de VantaDB (VER-04) |

### 1.5 Límites de §1

- **No normativo:** no promete "recordatorios" ni trigger de usuario; documenta el patrón y su vara de medición.
- **No es feature:** si un caso ICP pide recordatorios/triggers como requisito de primera clase → crear fila de implementación (slice: intención con due-window + disparo por host + evaluación con PM-Bench-style accuracy/false-alarm); no se wirea especulativamente.

## §2. Descartes documentados

> Origen de los descartes: dims hub §Validación (Notion — `Las 8 dimensiones de la memoria` (`3dbd0445-…bc50`) y `Los 10 ámbitos afectados` (`3dbd0445-…d2f6`); IDs abreviados del patrón NOTION-SYNC §14; detalle no verificable desde repo → `[a verificar — Notion]`). Este doc los hace **auditables desde el repo** (respaldos fetch-verificados + criterios de revisión).

### 2.1 Sensorial como almacén — DESCARTADO (2026-09-14)

- **Decisión:** **no** hay dimensión ni almacén sensorial dedicado (registro owner 2026-09-14, P49; el descarte vale **solo** para el almacén dedicado — [`memg-18-multimodalidad.md`](memg-18-multimodalidad.md) §1.1 opción C). El material sensorial entra por (a) **procesamiento de entrada** (buffer pre-tokenización) o (b) **ingesta multimodal** → memorias estándar/referencias (extensión transversal, MEMG-18).
- **Respaldo (fetch-verificado 2026-10-06):**
  1. **Mem0** (biblioteca, actualizada 2026-09-03): el equivalente IA de la memoria sensorial es el *raw input buffer* — "this raw form usually isn't retained on purpose"; "Most AI systems don't need a dedicated sensory-memory analog". El filtrado ocurre en el punto de decidir qué guardar (short/long-term), no en un store sensorial.
  2. **Survey 2602.06052** (TMLR): "sensory" es un **mecanismo cognitivo** (procesamiento perceptual) junto a working/episodic/semantic/procedural — su materialización práctica es entrada multimodal + procesamiento, no persistencia dedicada.
  3. **MIRIX** (arXiv [2507.07957](https://arxiv.org/abs/2507.07957), v1 2025-07-10): la implementación multimodal de referencia resuelve con **Resource Memory** + 6 tipos estándar (Core/Episodic/Semantic/Procedural/Resource/Knowledge Vault) — **sin** store sensorial; +35% accuracy vs RAG y −99.9% storage en ScreenshotVQA.
  4. **Repo:** [`memg-18-multimodalidad.md`](memg-18-multimodalidad.md) §1.1(C)/§3.8 — el almacén dedicado ya estaba descartado; la modalidad se decidió como extensión transversal.
- **Criterio de revisión:** la reapertura **no** es del almacén sensorial per se, sino de la **materialización multimodal** — se rige por el trigger de MEMG-18 (ingesta + caso ICP + embedder) y por la cadencia anual del marco 2.0 ([`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) §4). Sin evidencia nueva → descarte vigente.

### 2.2 Emocional/motivacional — DESCARTADO hoy (re-evaluar en 6-12m)

- **Decisión:** **no** es dimensión ni almacén de memoria. La **emoción** es un *cualificador del contenido* (saliencia/sentimiento) y el dominio afectivo (reconocer/evocar/expresar) pertenece a *affective computing*, no a un store de memoria; lo **motivacional** (drives/goals) se representa como intenciones/instrucciones — cubierto por el patrón de §1, no por un store nuevo.
- **Respaldo (fetch-verificado 2026-10-06):**
  1. **MemEmo / HLME** (arXiv [2602.23944](https://arxiv.org/abs/2602.23944), v1 2026-02-27): benchmark de emoción en sistemas de memoria con 3 dimensiones (extracción emocional, actualización emocional, QA emocional); **"none of the evaluated systems achieve robust performance across all three tasks"** — el estado del arte no es robusto ni siquiera en evaluación.
  2. **PsychoAgent** (arXiv [2608.07438](https://arxiv.org/abs/2608.07438), v2 2026-08-31; BICA 2026): retrieval afectivo mejora conflict-recall (0.933 vs 0.500/0.667) pero con **"corrected pairwise differences were not significant"** (5 raters ciegos, 27 outputs) — sin validación de producto; el resultado es un mecanismo inspeccionable, no una capacidad lista.
  3. **Survey 2511.20657** (v2 2026-05-02): la inteligencia emocional en agentes = reconocer/evocar/expresar emociones (affective computing) con desafíos abiertos — no se trata como dimensión/almacén de memoria.
  4. **Repo/Notion:** dims hub §Validación (candidata evaluada y descartada) `[a verificar — Notion]`; "emocional" en el repo aparece solo en contextos distintos (JTBD/marketing — `FND-24`), sin relación con el motor de memoria.
- **Criterio de revisión (contractual):** **re-evaluar en 6-12 meses** desde la validación (2026-09-30) → ventana **2027-03-30 → 2027-09-30**, o antes si dispara alguno de estos triggers:
  1. un benchmark emocional donde **≥1 sistema logre robustez en las 3 dimensiones** HLME (extracción + actualización + QA);
  2. **evidencia de producto** (caso ICP donde la continuidad afectiva sea requisito de primera clase, no adorno);
  3. evidencia experimental con **significancia** (no solo retrieval interno — el caso PsychoAgent).
  La re-evaluación se registra en la cadencia del marco 2.0 ([`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) §4) — la ventana emocional (6-12m) corre por delante de la anual.

### 2.3 Registro de descartes (auditable)

| Candidata | Decisión | Fecha | Respaldo (repo + fuentes) | Criterio de revisión |
|---|---|---|---|---|
| **Sensorial como almacén** | ❌ Descartada | 2026-09-14 (P49) | Mem0 (raw input buffer, sin retención) · survey 2602.06052 (mecanismo, no store) · MIRIX 2507.07957 (Resource Memory) · [`memg-18`](memg-18-multimodalidad.md) §1.1(C)/§3.8 | Trigger MEMG-18 (ingesta multimodal + caso + embedder) + cadencia anual (marco 2.0 §4) |
| **Emocional/motivacional como dimensión/almacén** | ❌ Descartada | 2026-09-30 (validación externa) | MemEmo 2602.23944 (ningún sistema robusto) · PsychoAgent 2608.07438 (sin significancia) · survey 2511.20657 (affective computing ≠ store) · dims hub §Validación `[a verificar — Notion]` | **Re-evaluar en 6-12m** (2027-03-30 → 2027-09-30) + 3 triggers (§2.2) |

## §3. Límites y deuda

- **Qué NO es este doc:** no implementa (cero código/símbolos); no es un ADR (Regla 5 — no hay decisión de implementación sancionada); no reabre v1.0; no re-litiga los descartes registrados; **no es normativo** (extensión futura del marco, "núcleo + extensiones").
- **Notion:** citado por ID de página (patrón NOTION-SYNC §14; sin transcripción — pre-mortem #1 del plan); detalle no verificable desde repo marcado `[a verificar — Notion]`. La aplicación de notas a Notion es del **lane owner** (canal: draft de [`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) §5).
- **Deuda:** ninguna nueva. El run **elimina** deuda de trazabilidad: prospectiva y descartes dejan de vivir solo en Notion y quedan auditables desde el repo (respaldos + criterios + grep > 0).

## §4. Fuentes

**Repo (anclas verificadas contra worktree 2026-10-06, HEAD `55b9a532` + cierre MEMG-15 `c29e2cb9` — sin drift en anclas):** [`../strategy/NOTION-SYNC-2026-09-24.md:132-144`](../strategy/NOTION-SYNC-2026-09-24.md) (§14 sync #2 + IDs) · [`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) (§2.2/§2.3/§2.7/§4) · [`memg-18-multimodalidad.md`](memg-18-multimodalidad.md) (§1.1(C)/§1.2/§3.8) · [`mgr-10-bitemporalidad.md`](mgr-10-bitemporalidad.md) (D5) · `vanta-memory/src/core/abstractions/types.rs:31,35,37` · `vanta-memory/src/utils/task_checkpoint.rs:1,74` · `vanta-memory/src/context_engine/mmd.rs:3,30` · `vanta-memory/src/services/scheduler.rs:1-16` · `vanta-memory/src/utils/timer_scanner.rs:14,25` · `vanta-memory/src/core/hooks/auto_recall.rs:390-423,653-657` · `src/sdk/types/record.rs:287-297,402-406` · `FIND-288`/`FIND-290` (consumidores pendientes dim1 — [`../Backlog.md`](../Backlog.md)) · registro de cierre en [`../avance/activo/operaciones.md`](../avance/activo/operaciones.md).

**Externas (fetch-verificadas 2026-10-06, TSYS-13):**

| URL | Título (verificado) | Fecha (verificada) |
|---|---|---|
| https://arxiv.org/abs/2607.12385 | PM-Bench: Evaluating Prospective Memory in LLM Agents | v1 2026-07-14 (COLM 2026) |
| https://arxiv.org/abs/2606.23459 | TriggerBench: Investigating Prospective Memory for Large Language Models | v1 2026-06-22 |
| https://arxiv.org/abs/2607.08716 | Remember When It Matters: Proactive Memory Agent for Long-Horizon Agents | v1 2026-07-09 |
| https://arxiv.org/abs/2602.06052 | A Survey of Agent Memory in the Second Half: Towards Self-Evolving and Long-Horizon Agents (TMLR) | v4 2026-08-04 |
| https://arxiv.org/abs/2602.23944 | MemEmo: Evaluating Emotion in Memory Systems of Agents (HLME) | v1 2026-02-27 |
| https://arxiv.org/abs/2608.07438 | PsychoAgent: An Affect-Sensitive Cognitive Architecture for Conflict-Aware Memory in LLM Agents (BICA 2026) | v2 2026-08-31 |
| https://arxiv.org/abs/2511.20657 | Intelligent Agents with Emotional Intelligence: Current Trends, Challenges, and Future Prospects | v2 2026-05-02 |
| https://arxiv.org/abs/2507.07957 | MIRIX: Multi-Agent Memory System for LLM-Based Agents | v1 2025-07-10 |
| https://mem0.ai/library/agent-memory/how-memory-shapes-us-a-deep-dive-into-the-types-of-memory | Types of AI Agent Memory: Sensory to Long-Term Explained (Mem0) | actualizado 2026-09-03 |
