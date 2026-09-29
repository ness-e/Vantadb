---
title: "MGR-12 — Confianza: jerarquía asserted/derived, scores por registro y derivación (Cierre MGR)"
kind: research
description: Dos ejes ortogonales por registro — nunca confundirlos
---

# MGR-12 — Confianza: jerarquía asserted/derived, scores por registro y derivación (Cierre MGR)

- **Fecha:** 2026-09-28 · **Tipo:** research/design (cero implementación productiva)
- **Contrato:** "research-doc cerrado con modelo de confianza asserted/derived (scores por registro + reglas de derivación + `last_validated` + calibración básica), tradeoffs y mapeo al `confidence_score` de nodo, listo para SCH-01."
- **Plan:** `docs/dev/plans/2026-09-26-master-roadmap.md` Task 24 (F3, dim 6) · **Destraba:** SCH-01 (Task 26) → SCH-02/SCH-04/SCH-05 · **Next:** SCH-01
- **Pre-mortem del plan:** F1 mitigado en §2 (taxonomía operacional + reglas verificables) · F2 mitigado en §6/§8 (jueces LLM excluidos explícitamente, `Backlog:934`) · F3 mitigado en §4 (semántica canónica en record + mapeo único a `confidence_score`).

## §0. Resumen ejecutivo (modelo en una página)

Dos ejes **ortogonales** por registro — nunca confundirlos:

| Eje | Pregunta | Valores | Fuente |
|-----|----------|---------|--------|
| `confidence_class` | ¿De dónde viene el contenido? | `asserted` (claim directo de un escritor) · `derived` (computado por el motor desde ≥1 registros padre) | Procedencia (PROV-O `wasDerivedFrom`; vocabulario asserted/inferred de triple stores) |
| `confidence` | ¿Cuánto se confía en él? | f32 en `[0.0, 1.0]` — rango de confianza declarado/computado | Escritor (asserted) o fórmula de derivación (derived) |

- **asserted** = el payload lo escribió un usuario/agente/import directo (no existe proceso del motor que lo compute). Score = declarado por el escritor o default `D_a` (Q1).
- **derived** = el motor lo computó como función de registros existentes y **declara sus padres** (`derived_from`). Score = `min(padres) × 0.9` (regla weakest-link, Q4) — determinista, explicable, recomputable.
- **`last_validated_at_ms`** = timestamp de la última re-verificación **exitosa** (`None` = nunca; Q2). No altera el score en 0.8.0.
- **Canónico = record.** `MemoryRecord` gana los campos; el nodo los proyecta/refleja por **un único punto de mapeo** (§4). Invariante anti-divergencia testeable.
- **Límites declarados (L1-L5, §5):** los scores son *rangos de confianza declarados/computados*, **no probabilidades calibradas** hasta VER-08. Cero jueces LLM en el slice (v1.0).

## §1. Gap verificado (código-real, re-verificado 2026-09-28 sobre HEAD)

### 1.1 El record no tiene confianza (el gap)
- `MemoryRecord` (`src/sdk/types/record.rs:102-138`): sin score, sin clase, sin `last_validated`. `MemoryInput` (`:59-78`): sin vía para declarar confianza.
- `rg last_validated` en `src/` = **0 hits** (repo-wide: solo refs previas en Backlog/plan/ROADMAP). `rg 'asserted'` de dominio = **0 hits** (solo comentarios de tests en `vfile_mmap.rs:580`, `engine/get.rs:87`).
- `vantadb-mcp/src/axioms.rs:22-30` = 4 Iron Axioms **hardcodeadas** (JSON). El axioma #2 "Confidence Constraint" ("Divergent vector mutations with high historical Confidence Score are rejected") referencia un score histórico, pero no existe clase por registro ni jerarquía de procedencia (el gap que citaba el plan).
- Escape actual: `metadata` es libre (`validate_metadata` solo bloquea el prefijo `__vanta_`, `src/sdk/serialization/mod.rs:12,136-150`) — pero **nada lee** claves de confianza en metadata: es un no-op observable, no un contrato.

### 1.2 El nodo tiene confianza, pero está desconectada del record
- `UnifiedNode.confidence_score: f32` (`src/node/unified.rs:41-42`; default `0.5` `:92`; persistida en disk header offset 40, `src/node/disk.rs:18-19`; escrita `src/storage/ops.rs:164`; restaurada `src/storage/engine/get.rs:363,689`, `txn.rs:661`, `ops.rs:334`).
- **Mapeo roto en ambos sentidos:** `memory_record_to_node_owned` (`src/sdk/serialization/mod.rs:433-495`) **no setea** `node.confidence_score` → todo record queda en el default `0.5`; `record_from_node` (`:305`, construcción `:416-430`) **no la lee** → invisible para el SDK.
- Única vía pública que la setea: `Embedded::restore_graph_nodes` (`src/sdk/api/graph.rs:231`) vía `NodeRecord.confidence_score` (`src/sdk/serialization/graph_types.rs:69`) — path de grafo (CORE-02), no de memoria.
- Consumidores existentes del score de nodo (el activo a preservar, Hyrum):
  - **Eviction:** peso `eviction_weight_confidence` (`src/config.rs:285`, default `2.0` en `:380`; fórmula `src/eviction.rs:38-45`; variante Bayesian feature-gated `:91-110`).
  - **Executor:** descarta `SemanticSummary` con `confidence_score() < 0.4` en el path QUERY (`src/executor.rs:219-242`) — hoy nunca dispara para records (su `type` no es `SemanticSummary`).
  - **Prompt LLM:** `"Confidence Score: {:.2}"` en el contexto de summarización (`src/llm.rs:940`).
  - **MCP:** `get_node_neighbors` expone `target_confidence` (`vantadb-mcp/src/handlers/tools.rs:2011`).
  - **Server:** `NodeDTO.confidence_score` "for staleness detection" (`src/server/state.rs:73-84`).
- **Señal declarada sin productor:** `ExecutionResult::StaleContext(u128)` ("Signal that a context requires rehydration (low confidence score)", `src/executor.rs:33`) — ningún constructor en producción; solo consumidores (`server/handlers.rs:159`, `python.rs:47`, `sdk/api/graph.rs:264`). Candidato natural para SCH-05 (§6.3).

### 1.3 Multi-agente
- `OriginCollisionTracker` (`src/utils/confidence_metrics.rs:8-94`, exportado vía `pub mod utils` — `src/lib.rs:151`; `:203` exporta `compute_confidence_friction`): EMA de confianza por `_owner_role` + `slash_origin` + métrica de fricción. No está cableado a records ni a nodos. `_owner_role` en `relational` hoy se usa para RBAC pruning (`src/executor.rs:499-511`). Queda como insumo de MGR-18 (post-1.0), no del slice.

### 1.4 Mapeo a SCH-01 (por qué este doc es pre-requisito duro)
`Backlog:938` exige el Cierre MGR de MGR-10/12/13 antes de tocar `record.rs`. Los campos definidos en §3 viajan a la migración única v0.7.0 (SCH-02), junto con temporalidad (MGR-10) y cuarentena (MGR-13).

## §2. Taxonomía operacional asserted/derived (F1: sin ambigüedad)

### 2.1 Definición operacional (test de clasificación)
Una escritura es **`derived`** si y solo si:
1. existe un **proceso del motor** (consolidación/dream, summarización, extracción futura) que la produce como **función de ≥1 registros existentes**, y
2. la escritura **declara sus padres** (`derived_from` no vacío).

En cualquier otro caso la escritura es **`asserted`**: un actor (humano, agente, importador) introdujo el payload directamente. Inspirarse en otros registros no convierte una escritura en derived — la convierte la **procedencia computacional declarada**.

> Precedente de vocabulario: triple stores RDF distinguen **asserted (explicit)** vs **inferred (implicit)** statements producidos por el razonador (GraphDB: "inference rules are applied repeatedly to the asserted (explicit) statements until no further inferred (implicit) statements are produced"); Apache Jena expone `isInferred`. PROV-O modela la relación genérica con `prov:wasDerivedFrom` y el ciclo de vida con `prov:invalidatedAtTime`. VantaDB adopta la distinción **procedencia**, no la semántica lógica (no hay razonador/TMS en 0.8.0 — ver §7 "rechazados").

### 2.2 Ejemplos (tabla de calibración mental)
| Escritura | Clase | Razón |
|---|---|---|
| Agente guarda un hecho aprendido vía `memory_put` | `asserted` | Claim directo; el agente ES la fuente |
| Usuario guarda una preferencia vía SDK/HTTP | `asserted` | Claim directo |
| `import` JSONL / `wiki_ingest` de un documento | `asserted` | Contenido importado = claim del exportador; procedencia externa va en `metadata.source` |
| `write_axiom` (namespace `_axioms`) | `asserted` | Regla declarada por el agente |
| Dream/consolidación resume N episodios → 1 resumen | `derived` | Proceso del motor; padres = episodios |
| Summarización produce nodo `type=SemanticSummary` | `derived` | Ídem; hoy ya existe el marcador de tipo (`executor.rs:224-227`) |
| `supersede` (ADR-0028) | *(ninguna)* | Supersesión = invalidación temporal, eje ortogonal (MGR-10) |
| Iron Axioms (`axioms.rs:24-29`) | *(fuera)* | Constantes del motor, no records |
| Decaimiento por eviction | *(ninguna)* | Señal de mantenimiento, no procedencia |

### 2.3 Reglas de derivación verificables (invariantes testeables)
- **V1 — Padres obligatorios:** `derived` sin `derived_from` no vacío ⇒ `Error::Validation` (boundary) o bug de motor (interno). `asserted` con `derived_from` no vacío ⇒ rechazo (contradicción de clase).
- **V2 — Monotonía:** `confidence(derived) ≤ min(confidence(padres))`. Ninguna derivación puede *aumentar* confianza.
- **V3 — Aciclicidad:** `derived_from` no puede referenciar un descendiente de sí mismo. Validación de profundidad acotada (cap `MAX_DERIVATION_DEPTH = 16` en 0.8.0; cadena mayor ⇒ rechazo documentado).
- **V4 — Determinismo:** mismos padres + misma fórmula ⇒ mismo score (recomputable byte a byte).
- **V5 — Clase por escritura:** la clase la define **cada escritura** (re-`put` sobre una key existente reemplaza clase/score; no se hereda de la versión previa). El historial conserva las clases anteriores vía version-history.

## §3. Modelo de confianza por registro (contrato para SCH-02/SCH-04)

### 3.1 Campos propuestos (aditivos, One-Version Rule)
```rust
/// Clase de procedencia de un registro de memoria (D6). #[non_exhaustive]: puede crecer
/// (p. ej. `observed`, `inferred`) sin breaking.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConfidenceClass {
    #[default]
    Asserted, // claim directo de un escritor
    Derived,  // computado por el motor desde `derived_from`
}

pub struct MemoryInput {
    // ... campos existentes ...
    /// None = `asserted` (default). `Some(Derived)` exige `derived_from` no vacío (V1).
    #[serde(default)]
    pub confidence_class: Option<ConfidenceClass>,
    /// None = default del sistema (D_a para asserted; fórmula para derived). Debe ser finito en [0,1].
    #[serde(default)]
    pub confidence: Option<f32>,
    /// Keys (mismo namespace) de los registros padre. Requerido si class=Derived (V1).
    #[serde(default)]
    pub derived_from: Option<Vec<String>>,
}

pub struct MemoryRecord {
    // ... campos existentes ...
    #[serde(default)]
    pub confidence_class: ConfidenceClass, // Asserted en datos históricos (backfill)
    #[serde(default = "default_confidence")] // D_a (Q1)
    pub confidence: f32,                   // clamp [0.0, 1.0]; NaN/∞ rechazados
    /// Última re-verificación EXITOSA del contenido (None = nunca). Q2.
    #[serde(default)]
    pub last_validated_at_ms: Option<u64>,
    #[serde(default)]
    pub derived_from: Vec<String>, // vacío para asserted
}
```

Notas de contrato (Hyrum):
- **Aditivo puro:** `#[serde(default)]` en todo campo nuevo; consumidores viejos (JSON/HTTP/MCP) siguen leyendo sin cambios; exports v1 importan con defaults (backfill determinista, SCH-02).
- **Nombre:** `confidence` (plan) ↔ `confidence_score` (nodo, header existente). `last_validated_at_ms` sigue la convención `*_at_ms` del record (`created_at_ms`, `superseded_at_ms`).
- **Validación en boundaries** (SDK/HTTP/MCP/import): rango finito `[0,1]`, V1, V3, cap de profundidad. El core interno confía en el tipo.
- **Error mapping:** reusar `Error::Validation { field, reason }` (formato único, `serialization/mod.rs:89-150` ya usa ese patrón) — sin variantes nuevas de error.

### 3.2 Semántica del score
- Dominio `[0.0, 1.0]`, f32 finito. Significado: **rango de confianza** (ordinal hasta VER-08; §5).
- `asserted`: score declarado por el escritor; ausente ⇒ `D_a` (Q1, default de política — ver L1).
- `derived`: `score = clamp(min(score(padres)) × DERIVATION_DISCOUNT, 0.0, 1.0)`, `DERIVATION_DISCOUNT = 0.9` (Q4). Constante de módulo en 0.8.0 (sin config nueva — YAGNI; VER-08 puede promoverla a config si la calibración lo justifica).
- **Sin decaimiento temporal en 0.8.0**: el score no baja con el tiempo (decay = MGR-09/MGR-11); `last_validated_at_ms` es la señal de frescura, no un multiplicador.
- **Recomputación:** los derived **no se recomputan reactivamente** si un padre cambia (0.8.0); conservan su score almacenado hasta re-consolidación. Límite L5 + v1.0 (§8).

### 3.3 `last_validated_at_ms` (Q2)
- `Some(t)` = última vez que el sistema/operador **re-verificó y confirmó** el contenido; `None` = nunca re-verificado (distinto de "inválido").
- Validaciones **fallidas** no tocan el campo: son evento de auditoría (futuro: MGR-13/VER-01). Solo el éxito se estampa (recomendado Q2a) — evita "último intento" ambiguo.
- 0.8.0: el campo se **expone y persiste**; no hay job de revalidación automática (eso es MGR-18, post-1.0). Un `put` que reescribe el payload actualiza `updated_at_ms`, no `last_validated_at_ms`.

### 3.4 Clase/score por superficie de escritura (defaults 0.8.0)
| Superficie (verificada) | Clase default | Score default | Vía de declaración |
|---|---|---|---|
| SDK `put`/`put_batch` (`src/sdk/api/memory.rs:66-143`) | `asserted` | `D_a` | `MemoryInput.confidence*` |
| HTTP server (`src/server/handlers.rs:250,261`) | `asserted` | `D_a` | body `MemoryInput` |
| MCP `memory_put`/`memory_put_batch` (`vantadb-mcp/src/handlers/tools.rs:84,112,1300,1434`) | `asserted` | `D_a` | args del tool |
| `import` JSONL (`src/sdk/serialization/impl_export.rs`) | `asserted` (v1) / preservado (v2) | `D_a` / preservado | línea export v2 |
| `wiki_ingest` | `asserted` | `D_a` | metadata/args |
| Dream/consolidación (`vanta-memory/src/core/dream/`) | `derived` | fórmula §3.2 | `derived_from` obligatorio |

Regla: **el default es uniforme** (`asserted`/`D_a`) para toda escritura directa; la variación viene de la declaración explícita del escritor, no de tablas por superficie en 0.8.0 (simplicidad; per-surface tuning = v1.0 si hace falta).

### 3.5 Export/import + version history + schema
- `MemoryExportLine` + `EXPORT_SCHEMA_VERSION` (`src/sdk/serialization/mod.rs:35`, hoy `1`): bump a `2` con los campos nuevos; import v1 ⇒ defaults deterministas (backfill). `record_from_export_line` (`:522`) valida versión exacta — mantener semántica TooNew.
- `SnapshotRecord` (`src/sdk/version_history.rs:76-90`) omite hoy `superseded_*` (mismo hallazgo que MGR-10) — debe sumar los campos de confianza al mirror postcard + `From` impls (`:112-148`).
- `src/schema.rs`: `CURRENT_SCHEMA_VERSION 1 → 2` — **un solo bump** coordinado con MGR-10/13 (migración única, `Backlog:934`); header `.vanta.schema` + TooOld/TooNew ya existen (`:11,84-98`).
- Backfill v1→v2 (determinista, SCH-02): `confidence_class = Asserted`, `confidence = D_a`, `last_validated_at_ms = None`, `derived_from = []`. Misma DB ⇒ mismo resultado (test SCH-06).

## §4. Mapeo record↔nodo (F3: anti-divergencia)

**Regla canónica:** la confianza vive en el **record**; el nodo es una **proyección** con un único punto de escritura y un único punto de lectura.

| Dato | Record (canónico) | Nodo (proyección) | Mecanismo |
|---|---|---|---|
| score | `confidence: f32` | `confidence_score: f32` (header, offset 40) | `memory_record_to_node_owned` setea `node.confidence_score = record.confidence`; `record_from_node` lo lee de vuelta |
| clase | `confidence_class` | `relational["__vanta_confidence_class"]` (String) | campo reservado, precedente `FIELD_SUPERSEDED_BY` (`mod.rs:28`) |
| last validated | `last_validated_at_ms` | `relational["__vanta_last_validated_at_ms"]` (Int) | idem |
| padres | `derived_from: Vec<String>` | `relational["__vanta_derived_from"]` (ListString) | idem |

- **Anti-colisión:** el prefijo `__vanta_` está reservado y rechazado en metadata de usuario (`serialization/mod.rs:12,137-142`) — los campos internos no son inyectables.
- **Invariante (test SCH-06):** `put(record)` → `get(key)` devuelve exactamente los mismos `confidence_class`/`confidence`/`last_validated_at_ms`/`derived_from`; y `node.confidence_score == record.confidence` tras el mapeo (roundtrip).
- **Preservación Hyrum:** el default 0.5 del nodo queda reemplazado por `D_a` para records nuevos — cambio observable **solo** vía los consumidores nuevos (los viejos ven el mismo f32 en el header, con otro valor; documentado como cambio intencional de la migración v2). `restore_graph_nodes` (`graph.rs:231`) sigue siendo el path de grafo; no lo toca este modelo (nota para SCH-02: no romper CORE-02).
- **Sinergia existente:** el filtro `SemanticSummary < 0.4` (`executor.rs:227`) pasa a ser coherente: un summary derived de padres débiles (`min×0.9`) puede cruzar el umbral y ser descartado — comportamiento deseado, no accidental.
- **Consumidores a actualizar en SCH-04:** `NodeDTO`/`server/state.rs` (exponer clase junto al score), MCP `get_node_neighbors` (opcional, v1.0), prompt LLM (`llm.rs:940` — ya lo usa; sin cambio).

## §5. Calibración básica y límites (stop condition: fórmula + defer VER-08)

### 5.1 Qué significa "calibrado" (y qué no)
Calibración = acuerdo entre el score y la frecuencia empírica de corrección (ECE — Guo et al., ICML 2017: "confidence calibration — the problem of predicting probability estimates representative of the true correctness likelihood"; las redes modernas están *poorly calibrated* y temperature scaling — un parámetro — las corrige). **En 0.8.0 los scores NO son probabilidades calibradas**: son rangos declarados (asserted) o computados por política (derived). Eso se documenta como límite, no se esconde.

### 5.2 Fórmula de calibración (spec para VER-08, F5 — dep cross-fase)
1. **Bins:** agrupar registros usados en respuestas evaluadas por score en B bins (B = 10 uniformes en 0.8.0).
2. **Ground truth:** corrección de la respuesta que usó el registro (harness VER-08: LongMemEval-S/LoCoMo; pares accuracy+tokens ya definidos en MGR-19 §3).
3. **ECE:** `ECE = Σ_b (n_b / N) · |acc(b) − conf(b)|` (conf(b) = score medio del bin).
4. **Baseline de corrección:** temperature scaling sobre el logit del score: `conf' = σ(logit(conf) / T)`, `T` ajustado minimizando NLL en validación; reportar ECE antes/después + reliability diagram.
5. **Consumidores del resultado:** revisión de `D_a` (Q1) y `DERIVATION_DISCOUNT` (Q4); posible promoción a config (tuner). La calibración aplicada (mapear scores a probabilidades) es **v1.0**.

### 5.3 Alternativas citadas (no implementadas en 0.8.0)
- **Verbalized confidence / P(IK)** (Lin-Hilton-Evans: GPT-3 aprende a expresar "90% confidence" calibrado; Kadavath et al.: los LMs "mostly know what they know"): válido como **input declarado** por agentes en `confidence` — no como garantía (Xiong et al. muestran que la elicitación varía fuerte por formato/modelo). No se valida en el slice.
- **Conformal prediction** (Angelopoulos & Bates): opción distribution-free para garantías de abstención en v1.0+ — fuera de scope.

### 5.4 Límites documentados (contrato público)
- **L1:** `D_a` y `0.9` son **política declarada**, no medición (VER-08 los mide).
- **L2:** no hay probabilidades calibradas; consumidores deben tratar los scores como rangos y usar **umbrales configurables**, no significancia estadística.
- **L3:** sin decaimiento temporal (MGR-09/11).
- **L4:** `derived_from` es same-namespace (cross-namespace = v1.0).
- **L5:** sin recomputación reactiva de derivados (re-consolidación manual/programada).

## §6. Consumo: SCH-04 (scores) y SCH-05 (trust-aware/abstención)

### 6.1 SCH-04 — scores consumibles (0.8.0)
- Exponer `confidence_class`, `confidence`, `last_validated_at_ms`, `derived_from` en SDK (`MemoryRecord`), HTTP (`MemoryRecord` JSON), MCP (`memory_get`/`memory_search` results).
- Filtro **opt-in** aditivo: `min_confidence: Option<f32>` en `MemoryListOptions`/search params (`record.rs:142-160` precedent: `exclude_superseded`). Default `None` = comportamiento actual (sin breaking).
- docs/api/ mismo PR (Regla 3).

### 6.2 SCH-05 — trust-aware retrieval + abstención (0.8.0)
- **Retrieval trust-aware:** excluir `quarantined` (MGR-13) y aplicar `min_confidence` opt-in en search; el contrato de ranking ponderado por confianza queda v1.0 (Q3).
- **Abstención selectiva:** umbral configurable (default off / `0.3` sugerido) — si el mejor hit < umbral, responder señal explícita de confianza insuficiente en vez de contenido (linaje: selective classification; en RAG: Self-RAG — tokens de reflexión; CRAG — evaluación del retrieval con acciones correctivas).
- **Reuso (ponytail):** `ExecutionResult::StaleContext` (`executor.rs:33`) ya es la señal declarada "requires rehydration (low confidence score)" sin productor — SCH-05 debería cablearla como resultado de abstención del query path en vez de inventar una variante nueva. (No se toca en este doc; hallazgo lateral sin fila FIND para no editar Backlog en co-batch.)

### 6.3 Excluido explícito del slice (v1.0 — `Backlog:934`)
- **Jueces LLM / grounding:** FACTS Grounding (benchmark de grounding sobre documento fuente), FaithJudge/Vectara (faithfulness RAG con leaderboard evolutivo), TruthfulQA — **no entran** (rabbit hole del plan). Solo quedan citados como trabajo futuro.
- Derivación completa automatizada (extracción de facts desde texto), recomputación reactiva, probabilidades calibradas, TMS/conflictos (MGR-06), decaimiento (MGR-09/11), cross-namespace, ranking ponderado.

## §7. Tradeoffs y alternativas consideradas

| # | Decisión | Alternativas | Elegido (rationale) |
|---|---|---|---|
| 1 | Clase + score separados | solo score / solo clase | **Ambos ortogonales**: la clase responde procedencia (auditable, PROV-O), el score responde confianza; un derived puede ser muy confiable y un asserted dudoso |
| 2 | Almacenamiento del score | record-only / nodo-only / header + campos reservados | **Header `confidence_score` (ya existe) + `__vanta_*` en relational** (precedente ADR-019/ADR-0028): cero cambios de layout en disk header, roundtrip por KV existente |
| 3 | Fórmula de derivación | media / min / min×factor / sin cómputo | **min×0.9** (Q4): weakest-link es explicable y monótona (V2); el factor descuenta la pérdida de la transformación; sin calibración empírica no se justifica nada más complejo |
| 4 | Default asserted | 1.0 declarado / 0.5 neutral / obligatorio | **Q1** — 1.0 "trust the writer" (semántica limpia, límites L1); 0.5 mantiene continuidad con el default del nodo pero es "no dice nada" |
| 5 | Calibración | ahora (temperatura en 0.8.0) / diferir | **Diferir a VER-08**: stop condition del plan; sin harness no hay ground truth; fórmula ya especificada (§5.2) |
| 6 | `last_validated` | éxito / intento / éxito+estado | **Q2** — éxito-only (`Option`), fallos a auditoría |
| 7 | Vía de declaración | metadata libre / campos tipados | **Campos tipados**: la metadata es observable pero no contractual (Hyrum); nada la lee hoy — tipado elimina la ambigüedad |
| 8 | Ranking | exponer+filtro / ranking ponderado | **Q3** — exponer+filtro opt-in en 0.8.0; ponderado tras calibración (v1.0): ponderar con scores no calibrados = regresión de relevancia |

## §8. 0.8.0 vs v1.0 (scope split explícito)

| Capacidad | 0.8.0 (SCH-02/04/05) | v1.0 |
|---|---|---|
| Campos + backfill + export v2 | ✅ | — |
| Derivación `min×0.9` + reglas V1-V5 | ✅ | — |
| `last_validated` persistido/expuesto | ✅ (estampado manual) | Revalidación automática (MGR-18) |
| Exposición + filtro `min_confidence` | ✅ | — |
| Abstención por umbral | ✅ (opt-in) | Umbral calibrado |
| Calibración aplicada (temperatura/ECE) | ⏳ spec §5.2 → **VER-08** (F5) | ✅ |
| Jueces/grounding (FACTS/FaithJudge/TruthfulQA) | ❌ excluido | ✅ evaluación (no necesariamente runtime) |
| Recomposición reactiva / extracción derivada | ❌ | ✅ |
| Ranking ponderado por confianza | ❌ | ✅ (post-calibración) |
| Cross-namespace `derived_from` | ❌ (L4) | ✅ |

## §9. Preguntas owner (Cierre MGR — no bloquean SCH-01; rutea LEAD)

1. **Q1 — Score default de `asserted`:** (a) `1.0` declarado con límites L1 documentados (**Recomendado** — semántica "trust the writer"; el class ya separa procedencia) · (b) `0.5` neutral (continuidad con el default actual del nodo) · (c) obligatorio explícito (fricción alta, rompe ergonomía del put).
2. **Q2 — Semántica de `last_validated_at_ms`:** (a) solo re-verificación exitosa (`Option`, **Recomendado**) · (b) último intento (incluye fallos, ambiguo) · (c) éxito + campo de estado de validación (2 campos, postergable — One-Version Rule).
3. **Q3 — Consumo en 0.8.0:** (a) exponer + filtro opt-in `min_confidence` (**Recomendado**) · (b) solo exponer (más conservador) · (c) ranking ponderado ya (riesgo de regresión con scores no calibrados — no recomendado).
4. **Q4 — Derivación:** (a) `min(padres) × 0.9` (**Recomendado**) · (b) media ponderada (oculta el eslabón débil) · (c) score fijo `0.5` para derived hasta v1.0 (no computa nada — decorativo).

## §10. Plan de implementación (consume SCH-01 → SCH-02/04/05/06 + VER-08)

1. **SCH-01 (Task 26 — consolidación):** fusionar §3 de este doc + modelo temporal MGR-10 + estados MGR-13 en **un** diff de schema v2; ADR en `docs/dev/architecture/adr/` (autoría humana, Regla 5 — este doc aporta la evidencia). Gate: Cierre MGR de los 3 research-docs (`Backlog:938`).
2. **SCH-02 (schema v2 + migración):** `src/sdk/types/record.rs` (enum + campos + serde defaults) · `src/sdk/serialization/mod.rs` (consts `__vanta_*`, mapping bidireccional, export v2) · `src/sdk/version_history.rs` (mirror postcard) · `src/schema.rs` (bump único) · backfill determinista. Tests: roundtrip + backfill + V1/V3. Verify: `cargo nextest run --profile audit -p vantadb`.
3. **SCH-04 (scores consumibles):** SDK/HTTP/MCP + `min_confidence` + docs/api mismo PR. Verify: tests de exposición + `validate-docs-coverage.ps1`.
4. **SCH-05 (trust-aware + abstención):** umbral + exclusión quarantined + cableado de `StaleContext` (o variante equivalente) + test de contención. Verify: test "contenido dudoso no se inyecta por defecto".
5. **SCH-06 (tests migración/time-travel/roundtrip + chaos):** incluye el invariante §4 (record↔nodo) y determinismo de backfill.
6. **VER-08 (F5 — calibración empírica):** harness con §5.2 (ECE + temperatura + reliability diagram); resultados revisan Q1/Q4.

## Fuentes (verificadas mecánicamente 2026-09-28)

| Fuente | URL | Verificación |
|---|---|---|
| Guo et al., *On Calibration of Modern Neural Networks* (ICML 2017) — ECE + temperature scaling | https://arxiv.org/abs/1706.04599 | fetch 200 (abstract) |
| Lin, Hilton, Evans, *Teaching Models to Express Their Uncertainty in Words* (P(IK), 2022) | https://arxiv.org/abs/2205.14334 | fetch 200 (abstract) |
| Kadavath et al., *Language Models (Mostly) Know What They Know* (2022) | https://arxiv.org/abs/2207.05221 | HEAD 200 |
| Xiong et al., *Can LLMs Express Their Uncertainty?* (confidence elicitation, 2023) | https://arxiv.org/abs/2306.13063 | HEAD 200 |
| Geifman & El-Yaniv, *Selective Classification for Deep Neural Networks* (2017) | https://arxiv.org/abs/1705.08500 | HEAD 200 |
| Angelopoulos & Bates, *A Gentle Introduction to Conformal Prediction* (2021) | https://arxiv.org/abs/2107.07511 | HEAD 200 |
| Lin et al., *TruthfulQA* (2021) | https://arxiv.org/abs/2109.07958 | HEAD 200 |
| Google DeepMind, *FACTS Grounding* (benchmark/leaderboard) | https://deepmind.google/blog/facts-grounding-a-new-benchmark-for-evaluating-the-factuality-of-large-language-models/ | HEAD 200 (URL canónica; la variante `discover/blog/...` dio 404 y fue descartada) |
| Tamber et al., *Benchmarking LLM Faithfulness in RAG with Evolving Leaderboards* (FaithJudge, 2025) | https://arxiv.org/abs/2505.04847 | HEAD 200 |
| W3C, *PROV-O: The PROV Ontology* (2013) | https://www.w3.org/TR/prov-o/ | fetch 200 (texto completo) |
| Ontotext GraphDB, *Reasoning* — asserted (explicit) vs inferred (implicit) | https://graphdb.ontotext.com/documentation/11.0/reasoning.html | HEAD 200 |
| Apache Jena, *Inference API* (`isInferred`) | https://jena.apache.org/documentation/inference/ | HEAD 200 |
| Park et al., *Generative Agents* (retrieval = recencia × importancia × relevancia, 2023) | https://arxiv.org/abs/2304.03442 | fetch 200 (abstract) |
| Rasmussen et al., *Zep/Graphiti* (grafo temporal valid_at/invalid_at, 2025) | https://arxiv.org/abs/2501.13956 | HEAD 200 |
| Chhikara et al., *Mem0* (sistema de memoria, 2025) | https://arxiv.org/abs/2504.19413 | HEAD 200 |
| Asai et al., *Self-RAG* (reflexión/abstención en retrieval, 2023) | https://arxiv.org/abs/2310.11511 | HEAD 200 |
| Yan et al., *CRAG — Corrective RAG* (evaluación del retrieval, 2024) | https://arxiv.org/abs/2401.15884 | HEAD 200 |

Nota de método: MetaSearchMCP quedó wedgeado (todos los providers en timeout, 2026-09-28) → router en cascada a keyless (`webfetch`/`websearch` nativos). Ninguna cita de este doc está sin verificar.

## Deuda y notas

- **Deuda nueva:** ninguna (research + docs-only; saldo neto Regla 6 = 0).
- **NOTICED BUT NOT TOUCHING:** `StaleContext` sin productor (`src/executor.rs:33`) → candidato natural de SCH-05 (señal de abstención/rehydration); sin fila FIND nueva (evitar editar `Backlog.md` en co-batch — SCH-05 lo consume igual). `OriginCollisionTracker` sin cablear → MGR-18 (post-1.0). Warnings pre-existentes `src/sdk/search/debug_ops.rs` (ya registrados en MGR-19).
- **WIP ajeno intacto:** MGR-10/MGR-13 (co-batch F3) — archivos no tocados. `src/` no tocado.
- **Skills (SDP v3, DEFINE):** campaign-executor + progreso (base) · source-driven-development · security-and-hardening · spec-driven-development · interview-me · idea-refine · documentation-and-adrs · writing-guidelines · api-and-interface-design · coordinated-web-search.
