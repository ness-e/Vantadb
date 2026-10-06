---
title: "MEMG-18 — Multimodalidad: decisión + spec (extensión transversal vs 9ª dimensión)"
kind: research
status: active
description: "Decisión documentada (extensión de modalidad transversal, no 9ª dimensión) + spec mínima anclada en payload/vector/MemoryExportLine + trigger refinado con evidencia 2026; sin implementación"
tags: [vantadb, research, estrategia]
---

# MEMG-18 — Multimodalidad: decisión + spec (extensión transversal vs 9ª dimensión)

- **Fecha:** 2026-10-06 · **Tipo:** research/spec (MEMG-18, Task 62, F5) · **Cero implementación** (este run no genera código ni símbolos)
- **Contrato (plan Task 62 L1790):** decisión documentada (extensión de modalidad transversal vs 9ª dimensión) + spec mínima (qué toca: contenido multimodal en episódica/semántica + embeddings multi-modal) + trigger de reevaluación refinado (hoy: MGR-25 + caso de uso) con la evidencia 2026; sin implementación.
- **Origen:** decisión owner 2026-09-14 ([`../backlog-futuro.md:28`](../backlog-futuro.md) FUT-15 / [`../Backlog.md:520`](../Backlog.md)) + validación externa 2026-09-30 ([`../Backlog.md:157`](../Backlog.md)) + marco 2.0 ([`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) §2.2(b)).
- **Alcance:** documentar la decisión de framing + spec mínima + trigger refinado. **No** reabre v1.0, **no** implementa, **no** rediseña el marco.

## Contenido

[§0 Resumen](#0-resumen-ejecutivo) · [§1 Decisión](#1-decisión) · [§2 Evidencia 2026](#2-evidencia-2026) · [§3 Spec mínima](#3-spec-mínima) · [§4 Trigger refinado](#4-trigger-refinado) · [§5 Límites y deuda](#5-límites-y-deuda) · [§6 Fuentes](#6-fuentes)

## §0. Resumen ejecutivo

| Pregunta | Respuesta |
|---|---|
| ¿9ª dimensión o extensión transversal? | **Extensión de modalidad transversal** — la modalidad es una propiedad de *forma* del contenido (eje sustrato/forma), no una función cognitiva ni una propiedad de ingeniería como D1-D8 |
| ¿Cambia el alcance v1.0? | **No** — multimodalidad sigue fuera de v1.0 con trigger (registro owner 2026-09-14, FUT-15, intacto) |
| ¿Qué toca cuando entre? | Contenido multimodal en episódica/semántica (payload + metadata) + embeddings multimodales (`vector`); ingesta vía extensión del trait `Ingestor`; export vía `MemoryExportLine` |
| ¿Cuándo se reevalúa? | Trigger refinado (§4): ingesta + caso de uso ICP + embedder multimodal disponible; más la cadencia anual del marco 2.0 |
| ¿Hay código? | **No** — decisión + spec mínima; la implementación espera al trigger |

## §1. Decisión

### 1.1 Opciones consideradas

| Opción | Descripción | Veredicto |
|---|---|---|
| **A — Extensión de modalidad transversal** | La modalidad (texto/imagen/audio/video) es un eje de forma del contenido; el contenido multimodal vive en registros de episódica/semántica con payload + metadata + embeddings multimodales; sin dimensión nueva | ✅ **Elegida** |
| **B — 9ª dimensión explícita (sensorial)** | Agregar "sensorial/modalidad" como dimensión del marco, junto a D1-D8 | ❌ Rechazada — mezcla ejes (§1.2) |
| **C — Almacén sensorial dedicado** | Store separado para percepciones sensoriales | ❌ Ya descartado (2026-09-14; el descarte vale solo para el almacén dedicado — documentado en [`memg-19-prospectiva-descartes.md`](memg-19-prospectiva-descartes.md) §2.1) |

### 1.2 Fundamento (por qué A)

1. **Las dimensiones D1-D8 son funciones y propiedades, no formas.** D1-D4 (trabajo, episódica, semántica, procedimental) son funciones cognitivas; D5-D8 (temporal, confianza, identidad, meta) son propiedades de ingeniería ([`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) §1.1, §2.4). La modalidad es una propiedad de *forma del contenido* — pertenece a los ejes ortogonales de presentación (sustrato/forma), no a la lista de dimensiones (MEMG-14 §2.3).
2. **La evidencia 2026 no pide una dimensión: pide capacidad.** El survey 2602.06052 lista "sensory" como **mecanismo cognitivo** (junto a working/episodic/semantic/procedural) — en la práctica de agentes eso se materializa como procesamiento de entrada multimodal + un store de recursos, no como función de memoria persistente. MIRIX (2507.07957) — la implementación real de referencia — resuelve multimodalidad con **Resource Memory** (componente de recursos) + embeddings visuales, manteniendo los tipos de memoria clásicos. Los tres competidores principales del segmento confirman el mismo patrón (§2.2): **Mem0** extrae facts de imágenes y los guarda como *memorias estándar*; **Cognee** convierte audio/imagen a texto y los ingesta *igual que documentos de texto*; **Letta** pasa imágenes como contenido de mensaje al LLM multimodal y su archival memory sigue siendo texto. Ninguno introduce una dimensión/tipo nuevo de memoria: la industria implementa multimodalidad como **capacidad transversal**.
3. **Las superficies existentes ya la soportan (extensión aditiva).** `MemoryInput`/`MemoryRecord` (payload `String` + `vector: Option<Vec<f32>>` + metadata) y `MemoryExportLine` no requieren cambio de modelo para contenido multimodal (§3). Una dimensión nueva no cambia nada en el motor — sería una etiqueta taxonómica sin sustrato técnico.
4. **Coherencia con el marco 2.0.** MEMG-14 §2.2 ya ubica multimodalidad como **ámbito candidato** (no dimensión) y §2.3 establece los ejes ortogonales como el lugar de las "formas"; esta decisión confirma esa ubicación.

### 1.3 Alcance v1.0 (registro owner 2026-09-14 intacto)

- La decisión registrada el 2026-09-14 es de **alcance**: "multimodal fuera de alcance v1.0, trigger MGR-25 + caso" (FUT-15). **Esta decisión no la modifica**: multimodalidad sigue fuera de v1.0; lo que se documenta es el *framing* (transversal, no dimensión) y la spec mínima para cuando el trigger dispare.
- Cualquier cambio de alcance es del owner (no unilateral — Gate D evaluado en el task file, no disparado).
- **Estado:** decisión documentada; sin implementación.

## §2. Evidencia 2026

> Fuentes externas fetch-verificadas 2026-10-06 (TSYS-13; títulos y cifras tomados de las fuentes). Fecha por fila: 2026-10-06 (verificación).

### 2.1 Surveys y frontera

| Fuente | Hallazgo relevante | Lectura para VantaDB |
|---|---|---|
| arXiv [2602.06052](https://arxiv.org/abs/2602.06052) — *A Survey of Agent Memory in the Second Half* (v4, TMLR) | Tres ejes: sustrato (paramétrico/externo), **mecanismo cognitivo (sensory, working, episodic, semantic, procedural)**, sujeto | "sensory" aparece como mecanismo; su materialización práctica es entrada multimodal + procesamiento, no store persistente dedicado |
| arXiv [2507.07957](https://arxiv.org/abs/2507.07957) — *MIRIX* | Seis tipos: Core, Episodic, Semantic, Procedural, **Resource Memory**, Knowledge Vault; en ScreenshotVQA: **+35% accuracy vs baseline RAG** y **−99.9% storage** | La multimodalidad real = store de recursos + embeddings; los tipos clásicos se conservan → extensión transversal |
| arXiv [2512.13564](https://arxiv.org/abs/2512.13564) — *Memory in the Age of AI Agents* (v2) | Formas (token/paramétrico/latente), funciones (factual/experiencial/working), dinámicas; **"multimodal memory" listada como frontera emergente** | Multimodalidad = frontera de investigación (dirección futura), consistente con "fuera de v1.0 con trigger" |

### 2.2 Implementaciones del segmento (competidores)

| Implementación | Cómo trata la multimodalidad (fuente oficial) | Lectura para VantaDB |
|---|---|---|
| **Mem0** ([docs.mem0.ai — multimodal support](https://docs.mem0.ai/open-source/features/multimodal-support)) | Imágenes → modelo de visión extrae "text and key details" → "Extracted information is stored as **standard memories** so search, filters, and analytics continue to work"; inputs por URL o base64 (JPEG/PNG/WebP/GIF) | Transversal: sin tipo/dimensión nueva — facts extraídos entran al modelo de memoria estándar |
| **Cognee** ([docs.cognee.ai — multimedia processing](https://docs.cognee.ai/guides/multimedia-audio-image-processing)) | Audio → transcripción; imagen → modelo de visión → **texto** ("turns audio and images into the text it then processes"); "both are ordinary text, so the same chunking, extraction, and summarization steps build one graph across the two files" | Transversal (convert-to-text): el pipeline de memoria no distingue modalidad tras la conversión |
| **Letta** ([docs.letta.com — image inputs](https://docs.letta.com/v1-sdk/messages/image-inputs) · [archival memory](https://docs.letta.com/v1-sdk/memory/archival-memory)) | Imágenes = **contenido de mensaje** para LLMs con visión ("supports image inputs… multi-modal capabilities depend on the underlying language model"); archival memory = store semántico de **texto** (facts/knowledge) | Transversal en la capa de mensaje; la memoria persistente sigue siendo texto — sin dimensión multimodal |

### 2.3 Stack de embeddings multimodales (disponibilidad para §3.3)

| Fuente | Hallazgo relevante | Lectura para VantaDB |
|---|---|---|
| [jina-clip-v2](https://jina.ai/models/jina-clip-v2/) (jina.ai; modelo abierto) | Embedder multilingüe texto+imagen (865M, 89 idiomas, 512×512); **Matryoshka 1024→64**; permite búsqueda entre modalidades en una representación coherente | Confirma que `vector` puede venir de un embedder conjunto; Matryoshka dialoga con FUT-02 (truncamiento dinámico) |
| [Voyage multimodal embeddings](https://www.mongodb.com/docs/voyageai/models/multimodal-embeddings/) (MongoDB/Voyage) | "Transform unstructured data from multiple modalities into a **shared vector space**" (texto/imagen/video); `voyage-multimodal-3.5`: 32k tokens, dims 1024/256/512/2048; un solo backbone reduce el sesgo de CLIP en búsqueda mixta | El espacio vectorial compartido es el invariante de §3.3 (una colección = un modelo); dimensiones configurables |

**Anclas repo:** decisión owner 2026-09-14 (FUT-15: [`../backlog-futuro.md:28`](../backlog-futuro.md), mirror [`../Backlog.md:520`](../Backlog.md)) · registro de la validación ([`../Backlog.md:157`](../Backlog.md) — "candidata más fuerte a dimensión faltante según los 5 surveys 2026; MIRIX +35% ScreenshotVQA, −99.9% storage") · marco 2.0 ([`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) §2.2(b), §2.3) · sync dims hub ([`../strategy/NOTION-SYNC-2026-09-24.md:138`](../strategy/NOTION-SYNC-2026-09-24.md): "multimodal candidata").

## §3. Spec mínima (qué toca)

> **No vinculante hasta que el trigger (§4) dispare.** Base para el slice futuro; anclas verificadas contra el worktree 2026-10-06 (HEAD `84bbc276` + WIP MEMG-15 en vuelo desplaza líneas de `record.rs`/`types.rs`; re-verificar líneas post-commit de MEMG-15).

### 3.1 Ingesta (extiende MGR-25)

- El trait `Ingestor` (MGR-25, `src/wiki/ingestors.rs:50`; registry `default_ingestors():125`) es el punto de extensión: un ingestor multimodal (imagen/audio/video) produce chunks `MemoryInput` con `metadata.source={file,page,chunk}` (patrón MGR-25/MEMG-08) **más** campos de modalidad: `metadata.modality` (`text|image|audio|video`) y `metadata.content_type` (MIME).
- El contenido no-textual entra como: (a) **referencia** (URI/ruta) o (b) **extracción acompañante** (caption/OCR/transcript) cuando exista; binarios grandes **no inline** en `payload` (política de blob store = pregunta abierta, §5).
- MGR-25 hoy: trait + txt/json/csv end-to-end; html/pdf/docx = FIND-298. La ingesta multimodal extiende el mismo trait (no lo reemplaza).

### 3.2 Almacenamiento (sin cambio de modelo de registro)

- `MemoryInput` (`src/sdk/types/record.rs:178`) y `MemoryRecord` (`:252`) conservan `payload: String` + `metadata` (`MemoryMetadata = Fields`, `src/sdk/types.rs:150`) + `vector: Option<Vec<f32>>`. La modalidad vive en **metadata + representación del payload** — sin campos nuevos obligatorios.
- Blob store content-addressed (para binarios) = extensión de storage a decidir (Arch) cuando el trigger dispare; fuera de esta spec.

### 3.3 Embeddings multimodales

- `vector` pasa a poder generarse con un **embedder multimodal** (jina-clip-v2: conjunto texto+imagen, Matryoshka 1024→64; voyage-multimodal-3.5: backbone único texto+imagen+video, dims 256–2048 — §2.3); el índice HNSW no cambia (mismo `Vec<f32>`).
- **Invariante de espacio vectorial:** una colección/namespace = un modelo de embeddings (no mezclar espacios); registrar `metadata.embedding_model` + `metadata.embedding_dim` para procedencia.
- Multi-vector por registro (p. ej. texto+imagen del mismo ítem) = extensión de esquema **no decidida** (pregunta abierta, §5).

### 3.4 Recuperación

- Búsqueda híbrida (dense + sparse + BM25 + RRF) sin cambios; filtros por modalidad vía metadata (`modality`); cross-modal retrieval = mismo espacio de embeddings (query texto → ítems imagen si el modelo es conjunto).

### 3.5 Portabilidad / export

- `MemoryExportLine` (`src/sdk/types/record.rs:562`): payload `String` + `vector` — multimodal export = referencias/base64 en payload + metadata; si se requieren campos nuevos → bump `schema_version` (v3) en la spec de implementación (no aquí).

### 3.6 Gobernanza / privacidad

- Proveniencia y confianza sin cambios (metadata.source MGR-12; `confidence_class` ADR-046; quarantine SCH-07 aplican igual). Contenido multimodal sensible (capturas de pantalla, audio) → aplican el ámbito candidato #1 (privacidad/compliance) y MEMG-17 (erasure criptográfica) como requisitos de diseño.

### 3.7 Evaluación

- Benchmark de referencia: ScreenshotVQA (MIRIX: +35% vs RAG, −99.9% storage) o equivalente propio; métricas VER-08 (accuracy + tokens/costo) como base.

### 3.8 Lo que NO toca

- **Dims D1-D8** — no se agrega D9 (decisión §1). · **`MemoryType`** (`vanta-memory/src/core/abstractions/types.rs:25`, 7 tipos L1) — sin tipo "sensorial". · **Wire format actual** — sin cambios obligatorios. · **Hot paths / HNSW** — sin cambios de algoritmo. · **Almacén sensorial dedicado** — descartado (2026-09-14; [memg-19](memg-19-prospectiva-descartes.md) §2.1). · **Memoria paramétrica** — fuera de alcance (MEMG-14 §2.6).

## §4. Trigger refinado

**Original (registro owner 2026-09-14):** MGR-25 + caso de uso (FUT-15).

**Refinado (2026-10-06) — lectura operativa del mismo trigger:**

| Condición | Lectura refinada | Estado hoy |
|---|---|---|
| 1. Ingesta | Trait `Ingestor` estable + **primer ingestor multimodal** (imagen/audio) — la extensión multimodal de MGR-25, no su cierre total (html/pdf/docx = FIND-298) | Trait estable (`src/wiki/ingestors.rs:50`); ingestor multimodal: no existe |
| 2. Caso de uso | Caso ICP concreto donde la **recuperación de contenido no-textual** sea requisito de primera clase (p. ej. agente computer-use que recupera pantallas; research con PDFs con figuras; asistente con notas de audio) | No registrado en repo |
| 3. Embedder multimodal | Modelo/proveedor disponible en el stack (local ONNX o remoto) + decisión de espacio vectorial por colección (§3.3) | No decidido |

- **Qué dispara:** las condiciones owner refinadas son (1)+(2); (3) es prerrequisito técnico declarado que se verifica al abrir (si falta, el primer slice incluye conseguir el embedder). Con (1)+(2)+(3) → crear fila de implementación con esta spec como base (slice: ingestor multimodal + embedder + retrieval por metadata + evaluación).
- **Qué NO dispara:** literatura sola (los 5 surveys 2026 no bastan — el marco exige evidencia de producto: [`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) §2.1 "la elevación requiere evidencia de producto"); paridad de features con competidores sin caso.
- **Cadencia:** la revisión anual del marco 2.0 ([`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) §4) re-evalúa esta fila; un cambio de evidencia (survey/estándar nuevo) la revisa antes.

## §5. Límites y deuda

- **Qué NO es este doc:** no implementa (cero código/símbolos); no es un ADR (Regla 5 — el ADR de una decisión de implementación lo articula el owner; aquí no hay decisión de implementación sancionada); no reabre v1.0; no rediseña el marco.
- **Preguntas abiertas (para la spec de implementación, cuando dispare el trigger):** blob store (¿content-addressed interno o referencias externas?), multi-vector por registro (¿esquema nuevo o un vector por registro?), export schema v3 (¿campos de modalidad?).
- **Deuda:** ninguna nueva. El run elimina deuda de trazabilidad (decisión + spec + trigger quedan auditables desde el repo).

## §6. Fuentes

**Repo (anclas verificadas contra worktree 2026-10-06; HEAD `84bbc276` + WIP MEMG-15 en vuelo):** [`../backlog-futuro.md:28`](../backlog-futuro.md) (FUT-15) · [`../Backlog.md:157`](../Backlog.md) (fila MEMG-18 + evidencia) · [`../Backlog.md:520`](../Backlog.md) (FUT-15 mirror) · [`memg-14-marco-2.0.md`](memg-14-marco-2.0.md) (§1.1, §2.2(b), §2.3, §2.6) · [`../strategy/NOTION-SYNC-2026-09-24.md:138`](../strategy/NOTION-SYNC-2026-09-24.md) · `src/sdk/types/record.rs:178,252,562` · `src/sdk/types.rs:150` · `src/wiki/ingestors.rs:50,125,147` · `vanta-memory/src/core/abstractions/types.rs:25` · `docs/dev/tasks/MEMG-08.md` (precedente MGR-25).

**Externas (fetch-verificadas 2026-10-06, TSYS-13):**

| URL | Título (verificado) |
|---|---|
| https://arxiv.org/abs/2602.06052 | A Survey of Agent Memory in the Second Half: Towards Self-Evolving and Long-Horizon Agents (v4, TMLR) |
| https://arxiv.org/abs/2507.07957 | MIRIX: Multi-Agent Memory System for LLM-Based Agents |
| https://arxiv.org/abs/2512.13564 | Memory in the Age of AI Agents (v2) |
| https://docs.mem0.ai/open-source/features/multimodal-support | Open Source Multimodal Support (Mem0 — facts de imágenes como memorias estándar) |
| https://docs.cognee.ai/guides/multimedia-audio-image-processing | Multimedia Processing (Cognee — audio/imagen → texto, mismo pipeline) |
| https://docs.letta.com/v1-sdk/messages/image-inputs | Image inputs (Letta — imágenes como contenido de mensaje) |
| https://docs.letta.com/v1-sdk/memory/archival-memory | Archival memory (Letta — store semántico de texto) |
| https://jina.ai/models/jina-clip-v2/ | jina-clip-v2 — Multilingual multimodal embeddings (texto+imagen, Matryoshka 1024→64) |
| https://www.mongodb.com/docs/voyageai/models/multimodal-embeddings/ | Multimodal Embeddings (Voyage AI/MongoDB — shared vector space, dims 256–2048) |
