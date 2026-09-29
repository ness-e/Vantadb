---
title: "ADR-046: Schema v2 — migración única hacia 0.8.0 (bitemporalidad + confianza + cuarentena)"
type: adr
status: accepted
tags: [vantadb, architecture, adr, schema, migration, bitemporal, confidence, quarantine, semver]
created: 2026-09-28
last_reviewed: 2026-09-28
---

# ADR-046: Schema v2 — migración única hacia 0.8.0 (bitemporalidad + confianza + cuarentena)

> **ACEPTADO — firmado por el owner el 2026-09-28 (Regla 5).** Este ADR consolida los tres
> Cierre MGR ✅ (MGR-10 `b6614e1e`, MGR-12 `50df4efd`, MGR-13 `72f29720`) en UNA migración de
> schema (**v2**, un único breaking) con corte **0.8.0**. Base ratificada (16 defaults) + las
> 7 decisiones nuevas firmadas; **SCH-02 desbloqueada**. Firma y articulación en
> [§Owner sign-off](#owner-sign-off-regla-5).

## Context

Tres investigaciones cerradas en F3 (dim 5 temporal + dim 6 confianza/cuarentena) decidieron,
por mandato del owner (2026-09-24, Backlog:934), converger en **un solo breaking change de
schema** en vez de tres migraciones: "migración única de MGR-10 (bitemporalidad) + MGR-12
(confianza) + MGR-13 (cuarentena), condicionada a que cada MGR cierre su research-doc" —
con alcance 0.8.0 **re-baselined**: 0.7.0 ya está publicado (`docs/CHANGELOG.md:10`,
`[0.7.0] - 2026-09-25`) y el corte activo de F3 es **0.8.0** (plan `:35`, gate F3).

Estado del código hoy (verificado contra HEAD; detalle en cada research-doc):

- El record SDK (`src/sdk/types/record.rs:103-138`) no tiene ventana de validez, ni confianza
  por registro, ni estado de cuarentena; `valid_at|invalid_at|bitemporal|as_of` = 0 hits de
  dominio en `src/` (`mgr-10-bitemporalidad.md:14`).
- La infra reutilizable existe y no se rediseña: transaction-time parcial por key
  (`version_history.rs`, retención 32/key), invalidación por `supersede`/`superseded_at_ms`
  (ADR-028), header `.vanta.schema` con `TooOld`/`TooNew` (`src/schema.rs:11,84-98`),
  comando `vanta migrate` (`src/cli_handlers/migrate.rs:175-279`) y export/import JSONL
  versionado (`serialization/mod.rs:35,498-531`).
- La confianza existe a nivel **nodo** (`unified.rs:42`, default `0.5` `:92`) y alimenta
  eviction y prompts, pero el mapeo record↔nodo está roto en ambos sentidos
  (`serialization/mod.rs:433-495` no la setea; `:416-430` no la lee).
- Cuarentena de contenido no existe (`quarantine` en `src/` = solo salvage de WAL,
  `src/wal.rs:611-642`, semántica de storage, no de contenido — `mgr-13-cuarentena.md:28`).

Restricciones de encuadre:

- **Rails de breaking (HARD-01 ✅):** 0.x MINOR = frontera de breaking
  (`docs/api/VERSIONING.md:16-27`); todo breaking se marca `feat!:`/`BREAKING CHANGE:`;
  rails mecánicos en `docs/api/COMPATIBILITY.md:17-30` + `release-plz.toml:22`
  (`semver_check = true`).
- **Pre-launch:** "breaking ilimitado pre-lanzamiento" (Backlog:934); los deltas ya
  acumulados se cierran con el corte 0.8.0 (`COMPATIBILITY.md:53-76`).
- **Stop conditions del bloque (respetadas):** nada que exija rediseño del engine
  (MVCC, índices temporales, append-only) entra a 0.8.0; si algo lo exige → recortar + FIND.
- **Insumos:** `mgr-10-bitemporalidad.md` (238L), `mgr-12-confianza.md` (286L),
  `mgr-13-cuarentena.md` (238L) — reconciliación explícita en [D8](#d8--reconciliación-de-insumos-mgr-101213--este-adr).

## Decision

### D0 — Base ratificada por el owner (2026-09-28)

Los **16 defaults recomendados** de los tres research-docs fueron aprobados por el owner vía
question el 2026-09-28 ("Aprobar defaults y avanzar"). Son decisión base de este ADR — **no se
re-abren**:

| Doc | Ratificado |
|-----|-----------|
| MGR-10 Q1–Q6 (`mgr-10-bitemporalidad.md:207-212`) | queries sin filtro por defecto (opt-in, sin imitar `AS OF now`); `invalid_at` retroactivo = API v1.0 (0.8.0 lo deriva de `supersede`); retención global 32/key documentada; snapshots migración **in-place**; crash-exactitud (P27) diferida a v1.0/VER-01; **edges (SCH-09) se decide en este ADR** (ver D1c). |
| MGR-12 Q1–Q4 (`mgr-12-confianza.md:243-246`) | `D_a = 1.0` para `asserted`; `last_validated_at_ms` = solo éxito (`Option`); exponer + filtro opt-in `min_confidence`; derivación `min(padres) × 0.9`. |
| MGR-13 Q1–Q6 (`mgr-13-cuarentena.md:206-211`) | import con cuarentena **opt-in** + recomendación documentada; expiración `keep` + señal de vencido (deadline default se fija en D5d); separación reviewer/writer **auditable** (no dura) en 0.8.0; cuarentena **sticky** (I2); `get` por key devuelve cuarentenados con estado visible; `quarantine_apply` en 0.8.0. |

Las decisiones **nuevas** de este ADR (D4b, D4c, D5d, D5e, D6, D7, D8) +
los canónicos consolidados (D1–D3, plan de migración) quedaron **firmados por el owner**
(2026-09-28; status `accepted` — ver §Owner sign-off).

### D1 — Corte único: schema v2 dentro de 0.8.0 (sin segundo breaking)

**D1a — Versión.** `CURRENT_SCHEMA_VERSION 1 → 2` (`src/schema.rs:11`) y
`EXPORT_SCHEMA_VERSION 1 → 2` (`serialization/mod.rs:35`), ambos en el mismo corte **0.8.0**.
El bump del header es el **marcador de migración completa** y va **último** (D7). `MIN_COMPAT_VERSION`
se mantiene en `1`: un binario v2 lee DBs v1 vía normalización; un binario v1 sobre DB v2 ve
`TooNew` (correcto: no sabe decodificar snapshots V2).

**D1b — Nada más entra.** Todo breaking de persistencia del corte F3 viaja bajo el **mismo**
`SCHEMA_VERSION = 2`. Prohibido un segundo bump (v3) antes/para 0.8.0; cualquier necesidad
futura de breaking va a un release posterior con su propio ADR.

**D1c — Edges (SCH-09), riesgo de segundo breaking — anotado y acotado.**
`Edge` (`src/node/edge.rs:9`) no tiene ventana de validez; Backlog:946 propone
`properties` + `valid_at_ms`/`invalid_at_ms` + backfill, **dentro de la misma migración v2**
("misma migración v2, sin segundo breaking"). Esquema de edges NO se diseña acá (stop condition
de MGR-10 §4.5, `mgr-10-bitemporalidad.md:201-203`). Decisión de frontera:

1. Si SCH-09 aterriza antes del corte, sus campos viajan **dentro del envelope v2** (mismo
   `SCHEMA_VERSION=2`, mismas reglas D7 de migración/mirrors) — nunca un v3.
2. Si no aterriza (no tiene task en el plan hoy; solo Backlog:946), nace una fila `FIND-*`
   con dueño y el corte 0.8.0 no se bloquea.
3. SCH-09 no toca `record.rs`; consume este ADR para las reglas de migración.

**D1d — Alcance 0.8.0 vs v1.0 (estricto; anti-scope-creep).** Consolidado de las tres
secciones de scope (`mgr-10-bitemporalidad.md:126-137`, `mgr-12-confianza.md:226-239`,
`mgr-13-cuarentena.md` §8):

| Capacidad | 0.8.0 (corte F3) | v1.0 / diferido (dueño) |
|---|---|---|
| Campos v2 en record + backfill determinista | ✅ (SCH-02) | — |
| Semántica valid/transaction + intervalo `[start, end)` | ✅ (D3) | bitemporal per-fact completo (`recorded_from/to`, Snodgrass) |
| Time-travel transaction **por key** (`get_version`/`versions`) | ✅ reutiliza lo existente | `as_of` transaction cross-key (índice temporal) |
| `AS OF T` valid-time (IQL + search/list) + filtros de ventana + `exclude_superseded` extendido | ✅ (SCH-03) | MVCC / índices temporales |
| Confianza por registro (clase + score + `last_validated` + `derived_from`) + derivación `min×0.9` | ✅ (SCH-02/04) | recomputación reactiva; extracción automática de facts |
| Exposición + filtro `min_confidence` + abstención opt-in | ✅ (SCH-04/05) | umbral calibrado; ranking ponderado por confianza |
| Calibración (ECE/temperatura) | ⏳ spec → **VER-08** (F5) | ✅ aplicada |
| Jueces/grounding LLM (FACTS/FaithJudge/TruthfulQA) | ❌ excluido | ✅ evaluación |
| Cuarentena (estado + transiciones + default-exclude + gates de inyección) | ✅ (SCH-02/05) | detección ML de poison; namespaces trusted/tainted (MGR-04); T5 re-validación; aislamiento L0 |
| Crash-exactitud del historial (WAL-exact, P27) | ❌ (gap documentado) | v1.0 / VER-01 |
| Retención per-namespace; setter retroactivo de `invalid_at`; cross-namespace `derived_from` (L4) | ❌ | v1.0 |
| Hash-chain de auditoría (PROV-O) | ❌ (citado — lo cubre VER-01, F4) | F4 |
| Edges con validez temporal (SCH-09) | envelope v2 o FIND (D1c) | — |

Rabbit holes explícitamente NO (stop conditions del plan): MVCC, segundo store de eventos
(CQRS), gramática temporal completa, migrar índices derivados in-place (rebuild-on-mismatch
ya existe, `impl_index.rs:52`).

### D2 — Campos v2 consolidados (contrato para SCH-02)

**`MemoryRecord` — 10 campos nuevos** (todos `#[serde(default)]` — `confidence` con `default = "default_confidence"` (D_a = 1.0); aditivo puro — One-Version:
se extiende el record, nunca un `MemoryRecordV2`):

| Campo | Tipo | Normalización v1 (compat) | Semántica | Fuente |
|---|---|---|---|---|
| `valid_at_ms` | `u64` | ausente ⇒ `created_at_ms` | inicio de validez; intervalo `[valid, invalid)` | `mgr-10:63`, D8 |
| `invalid_at_ms` | `Option<u64>` | `None` (abierto, ∞) | fin de validez; puede ser retroactivo (setter v1.0) | `mgr-10:64` |
| `confidence_class` | `ConfidenceClass` (`#[non_exhaustive]`, default `Asserted`) | `Asserted` | procedencia: claim directo vs computado por el motor | `mgr-12:110` |
| `confidence` | `f32` finito en `[0,1]`; NaN/∞ rechazados | `D_a = 1.0` | rango de confianza declarado/computado (NO probabilidad calibrada — L1/L2) | `mgr-12:111-112` |
| `last_validated_at_ms` | `Option<u64>` | `None` | última re-verificación **exitosa** (None = nunca; fallos no tocan el campo) | `mgr-12:113-115` |
| `derived_from` | `Vec<String>` | `[]` | keys padre (same-namespace) de un `derived`; vacío para `asserted` | `mgr-12:116-117` |
| `quarantined_at_ms` | `Option<u64>` | `None` | `None` = activo; `Some(t)` = en cuarentena desde t (sticky) | `mgr-13:53` |
| `quarantine_reason` | `Option<String>` | `None` | código estable: `explicit_write` \| `unreviewed_import` \| `derived_promotion` \| `policy_match` (reservado) | `mgr-13:54` |
| `quarantined_by` | `Option<String>` | `None` | principal que la aplicó, o `system:<op>` (ej. `system:dream_promote`) | `mgr-13:55` |
| `quarantine_review_due_ms` | `Option<u64>` | `None` | deadline de revisión (señal T3; **nunca** promueve solo) — default en D5d | `mgr-13:56` |

Contrato Rust mínimo (nombres finales; `#[non_exhaustive]` obligatorio — puede crecer con
`observed`/`inferred` sin breaking, `mgr-12:84-92`):

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConfidenceClass {
    #[default]
    Asserted,
    Derived,
}
```

**`MemoryInput` — campos nuevos** (aditivos, la vía de declaración del escritor):

- `valid_at_ms: Option<u64>` — `None` ⇒ default de insert (D8).
- `confidence_class: Option<ConfidenceClass>` — `None` ⇒ `asserted` (default del sistema).
- `confidence: Option<f32>` — `None` ⇒ `D_a` para asserted; **prohibido** si `Derived` (D4b).
- `derived_from: Option<Vec<String>>` — requerido no-vacío si `Derived` (V1); prohibido si `Asserted`.
- `quarantine: bool` (`#[serde(default)]` → `false`) — entrada write-time T1 (`mgr-13:66`).

**Wire de consulta** (aditivo, opt-in; el default de TODAS las queries queda como hoy):

- `MemoryListOptions` / `MemorySearchRequest`: `include_quarantined: bool` (default `false`,
  `mgr-13:58`), `min_confidence: Option<f32>` (default `None`, `mgr-12:201`), hasheados en el
  fingerprint del cursor (`page.rs:157`, mismo patrón que `exclude_superseded`).
- Params temporales de SCH-03: **semántica fija en D3**; nombres de wire finales en SCH-03
  (con `AS OF` en IQL). No se fijan acá para no duplicar el contrato de esa task.
- **Abstención** (config + wire, nombre final fijado acá porque `mgr-13:196` lo delega):
  config `confidence_threshold: Option<f32>` (default `None` = OFF; distinto del filtro
  por request `min_confidence`, que no dispara abstención); cuando está seteado y no quedan
  candidatos sobre el umbral, la respuesta lleva `abstained: true` + `abstention_reason` con
  códigos estables `no_candidates_above_threshold` \| `all_quarantined`. Nunca "menos
  resultados" silenciosos.

**Hyrum (forma de emisión):** `#[serde(default)]` es obligatorio en todo campo nuevo;
la emisión (`null` vs `skip_serializing_if`) **no** se garantiza — los consumidores no deben
depender de la presencia de nulls. Consumidores viejos ignoran campos desconocidos (serde),
pero los v1-imports se normalizan (D7).

### D3 — Semántica valid vs transaction (los dos ejes)

Dos ejes ortogonales por registro (`mgr-10:40-48`; SQL:2011 / Snodgrass / Zep-Graphiti):

- **Valid time** (`valid_at_ms`, `invalid_at_ms`) — cuándo el contenido refleja la realidad
  *según el usuario*; modificable retroactivamente (bitemporal real). Cambia lo que el dato
  *dice sobre el mundo*.
- **Transaction time** (`version`, `updated_at_ms`, `superseded_at_ms`, snapshots
  `Versions`) — cuándo el sistema *registró* cada estado; append-only por key. Registra
  cómo cambió *nuestro conocimiento*. `updated_at_ms` es el bump del estado del registro —
  **no** es una ventana de verdad; `superseded_at_ms` es el evento de invalidación.

**Contrato temporal (canónico):**

1. Intervalo **cerrado-abierto** `[valid_at_ms, invalid_at_ms)`; predicado "vale en T":
   `valid_at_ms <= T && invalid_at_ms.map_or(true, |inv| inv > T)` (`mgr-10:66-70`).
2. Invariante de escritura: `valid_at_ms <= invalid_at_ms` cuando ambos están set
   (validación en boundary + test).
3. `supersede()` escribe por defecto `invalid_at_ms = Some(now)` **junto a**
   `superseded_at_ms = Some(now)` → en 0.8.0 quedan **alineados** (invariante 3,
   `mgr-10:90`); divergen solo con un setter retroactivo de `invalid_at` (API v1.0).
4. **`AS OF T` = eje valid** (la pregunta "el spec decía X en fecha T", `evidence-before-belief`).
   No confundir con transaction: el time-travel de transaction en 0.8.0 es **por key**
   (`get_version`/`versions`, retención 32/key, gap de crash documented) y **no** existe
   `AS OF` transaction cross-key en search/list (v1.0, sin índice de versiones).
5. **Default sin cambios:** las queries actuales siguen devolviendo todo; `AS OF`/ventana/
   `exclude_superseded` son opt-in — cero breaking silencioso (ratificado, D0; difiere a
   propósito del default `AS OF CURRENT_TIMESTAMP` de SQL:2011, `mgr-10:84`).
6. La extensión de `exclude_superseded` a la semántica nueva (excluir también
   `invalid_at_ms <= now`) la decide SCH-03 con flag aditivo o extensión documentada
   (`mgr-10:83`); el ADR fija el predicado, no la forma del flag.

### D4 — Confianza: reglas, decisiones y tests

**Taxonomía operacional** (`mgr-12:51-58`): una escritura es `derived` si y solo si (1) un
proceso del motor la produce como función de ≥1 registros existentes **y** (2) declara sus
padres (`derived_from` no vacío). En cualquier otro caso es `asserted` (humano/agente/import
directo). Cuadro de ejemplos: `mgr-12:60-71`.

**Reglas V1–V5** (`mgr-12:73-78`), con la resolución de los carry-overs:

- **V1 — Padres obligatorios.** `derived` sin `derived_from` ⇒ `Error::Validation`
  (boundary); `asserted` con `derived_from` no vacío ⇒ rechazo.
- **V2 — Monotonía.** `confidence(derived) ≤ min(confidence(padres))`. Test nuevo (abajo).
- **V3 — Aciclicidad.** Validación de profundidad acotada, `MAX_DERIVATION_DEPTH = 16`;
  cadena mayor ⇒ rechazo documentado.
- **V4 — Determinismo.** Mismos padres + misma fórmula ⇒ mismo score (recomputable).
  Test nuevo (abajo).
- **V5 — Clase por escritura.** La clase la define cada escritura; re-`put` sobre una key
  reemplaza clase/score (no se hereda de la versión previa); el historial conserva las clases
  anteriores vía version-history. Test nuevo (abajo).

**D4a — Score de `derived`.** `score = clamp(min(score(padres)) × 0.9, 0.0, 1.0)`;
`DERIVATION_DISCOUNT = 0.9` constante de módulo (sin config en 0.8.0; VER-08 puede promoverla).
Sin decaimiento temporal (señal de frescura = `last_validated_at_ms`); sin recomputación
reactiva (L5) — el score almacenado es la verdad hasta re-consolidación (`mgr-12:127-132`).

**D4b — [`Derived` + `Some(confidence)`] ⇒ RECHAZO en boundary** *(carry-over P2-01 #1;
`mgr-12` no lo especificaba)*. Decidido: **(a) rechazo**, no clamp ni solo-interno.

- Predicado: `confidence_class == Derived && confidence.is_some()` ⇒
  `Error::Validation { field: "confidence", reason: "derived score is computed from parents; declared scores are not allowed on derived records" }`
  (mismo formato de error existente, `serialization/mod.rs:522-531`).
- Rationale: (b) clamp silencioso descarta input del caller sin señal (Hyrum: comportamiento
  mudo y depurable fatal); (c) solo-interno duplica la computación en dos caminos y permite
  que un escritor interno contradiga V2. Con (a), la fórmula tiene **un único productor**
  (el punto de materialización del record, D6) y V2 se cumple por construcción.
- Nota: los escritores internos (dream/consolidación) tampoco pasan el score: pasan
  `derived_from` + clase y el core computa en el punto único.

**D4c — Backfill `confidence := D_a = 1.0` uniforme; cambio observable ACEPTADO** *(carry-over
P2-01 #2)*. Decidido: **aceptar el cambio (pre-launch)**, sin caso especial 0.5.

- Backfill: **todo** registro histórico ⇒ `confidence_class = Asserted`, `confidence = 1.0`
  (D_a), `last_validated_at_ms = None`, `derived_from = []` (`mgr-12:155`). Uniforme: una sola
  semántica para el campo.
- Consumidores viejos citados y su delta esperado:
  - **Eviction (Weighted):** fórmula `hits + confidence×w.confidence + importance×…` en
    `src/eviction.rs:38-45`; peso `eviction_weight_confidence` default **2.0**
    (`config.rs:285,380`). Hoy todo record llega con el default 0.5 del nodo (mapeo roto);
    post-v2 llegarán scores reales (fórmula `min(padres)×0.9`) ⇒ el peso en eviction cambia según clase/padres (más conservador para `asserted` con D_a=1.0; menos para `derived` con `min(padres) < 0.556`) — la dirección depende de la clase; recalibrar es de
    eviction (retención más conservadora). El peso sigue configurable; recalibrar es de
    VER-08/owner, fuera de este ADR.
  - **Prompt LLM:** `"Confidence Score: {:.2}"` con `node.confidence_score`
    (`src/llm.rs:939-944`) ⇒ pasará a mostrar valores reales (1.00 asserted / min×0.9 derived)
    en lugar de 0.50. Cambio intencional documentado.
  - **Executor:** filtro `SemanticSummary < 0.4` (`src/executor.rs:219-242`) — solo aplica a
    `SemanticSummary`; con la fórmula, un summary derived de padres débiles puede cruzar el
    umbral (sinergia deseada, `mgr-12:171`), no regresión.
  - **MCP `get_node_neighbors` / `NodeDTO`:** mismos números nuevos; exposición de clase en
    SCH-04.
- Alternativa rechazada: backfill 0.5 "por continuidad" — crea dos semánticas para el mismo
  campo sin lifetime (histórico neutral vs nuevo declarado); contradice la decisión Q1
  ratificada (1.0 "trust the writer", `mgr-12:243`).
- Registro documental: la guía `UPGRADE.md` 0.8.0 (SCH-08) lista este delta como cambio
  observable del corte. Fundamento semver: 0.x MINOR = frontera de breaking
  (`VERSIONING.md:16-27`) + pre-launch (Backlog:934).

**D4d — Tests V2/V4/V5 enumerados** *(carry-over P2-01 #1b; hoy solo V1/V3 estaban cubiertos
en el plan de tests)*. Se suman al contrato de SCH-02 (unidad/boundary) y SCH-06
(persistencia/mirror):

| Regla | Test | Nivel |
|---|---|---|
| V2 | `derived_score_is_min_parents_times_0_9` — padres [0.8, 0.5] ⇒ score 0.45; y `derived_score_le_min_parent` (monotonía) | SCH-02 unit |
| V2 | `derived_with_declared_confidence_is_rejected` — boundary (D4b) | SCH-02 integration |
| V4 | `derived_score_recomputable_deterministically` — mismos padres ⇒ mismo score; recomputo desde padres almacenados == score persistido | SCH-02 unit |
| V4 | `backfill_is_deterministic` — misma DB v1 ⇒ mismo output (cubierto además por SCH-06 doble corrida) | SCH-06 |
| V5 | `reput_replaces_class_and_score` — asserted(0.9) → derived ⇒ clase/score recalculados, sin herencia; derivada→asserted idem | SCH-02 integration |
| V5 | `snapshot_history_preserves_previous_class` — el mirror `SnapshotRecord`/version-history retiene la clase anterior | SCH-06 |

### D5 — Cuarentena: estado, transiciones y defaults

Máquina de estados y threat model completos en `mgr-13:80-165` (estados §3.1; transiciones
§3.2 T1/T1b/T1c/T1d/T2/T3/T4; invariantes §3.3 I1–I6). Este ADR fija solo lo que es
contrato de schema/compat:

- Estado binario: `active` (implícito, `quarantined_at_ms = None`) · `quarantined`
  (`Some(t)`). Default-exclude de search/list/retrieval con opt-in `include_quarantined`;
  `get` por key devuelve el registro **con estado visible** (ratificado Q5). Nunca 404
  silencioso. Ortogonal a `superseded_by` y `expires_at_ms` (I3).
- **Sticky (I2):** un `put` sobre una key cuarentenada conserva el estado. Solo T2
  (`quarantine_promote`) o T4 (`quarantine_reject`) salen.
- **Nunca auto-promoción (I1):** ni TTL ni deadline promueven. Gates duros de inyección
  (`auto_recall`, `inject_context`, `recall` nunca inyectan cuarentenado — `mgr-13:185-192`).
- Entradas 0.8.0 (`mgr-13:109-119`): flag explícito write-time; promoción derivada de dream
  default ON (T1b); opción de import; `quarantine_apply` (T1d). Sin heurísticas automáticas
  (I6 — cero falsos positivos por contenido).
- **D5d — Deadline default de revisión: 30 días** *(carry-over P2-01 #6; única sub-pregunta
  de MGR-13 sin recomendación, `mgr-13:207`)*. Decisión firmada (2026-09-28):
  - Al entrar en cuarentena sin deadline explícito:
    `quarantine_review_due_ms := quarantined_at_ms + 30 días`; override por registro;
    config `quarantine_review_default_days: u32 = 30` (`0` = sin deadline default).
  - Acción de expiración (T3): `keep` = default (señal métrica `quarantine_overdue` +
    audit; el registro sigue cuarentenado); `purge` = opt-in explícito del operador.
  - Rationale: (i) cierra I4 automáticamente — toda entrada tiene clave de revisión, sin
    esperar a que el operador la setee (mitiga "cuarentena permanente", riesgo #2 del bloque);
    (ii) costo cero por defecto: la señal no muta datos (`keep`) y nunca promueve (I1);
    (iii) 30d coincide con el ejemplo de Q2 y con cadencias de revisión típicas (patrón
    reporte+expiración de Azure Quarantine, `mgr-13:234`); (iv) no afecta el determinismo del
    backfill: los registros v1 no pueden estar cuarentenados (todos `None` al migrar).
- **D5e — Default de import (cierre del "a decidir" de Q1).** Uniforme: `quarantine=false`
  por defecto en **todos** los caminos (`import_records`/`import_file`/`bulk_import_*`/
  `import_v2`) + recomendación documentada para archivos externos; `quarantine=true` opt-in
  por operación. Es el default compatible (import sin flag se comporta como hoy).
- Checkpoint de contrato futuro (anotado, no resuelto acá): cuando MEM-65 conecte el merge
  real de dream, `dream_promote` MCP pasa de `readOnlyHint: true` a `false` — breaking
  deliberado documentado (`mgr-13:146`).

### D6 — Mapeo record↔nodo y punto de escritura residual *(carry-over P2-01 #3)*

Regla canónica (`mgr-12:157-172`): **la confianza vive en el record; el nodo es una
proyección** con un único punto de escritura (`memory_record_to_node_owned`,
`serialization/mod.rs:433-495`) y un único punto de lectura (`record_from_node`,
`:305,416-430`):

| Dato | Record (canónico) | Nodo (proyección) |
|---|---|---|
| score | `confidence: f32` | `confidence_score: f32` (header) — el write path lo setea, el read path lo lee |
| clase / validated / padres | campos tipados | `relational["__vanta_confidence_class"]` / `["__vanta_last_validated_at_ms"]` / `["__vanta_derived_from"]` (prefijo reservado no inyectable, `mod.rs:12`) |

**Precedencia del segundo escritor — `restore_graph_nodes` (`src/sdk/api/graph.rs:203-240`,
`:231`).** Ese path restaura grafos exportados (CORE-02) y setea `node.confidence_score`
con el valor transportado por `NodeRecord`. No se rompe CORE-02 (no se recomputa ni se
descarta nada). Precedencia definida:

1. **Dominio memory:** el record manda; el write path de memoria es el normalizador único
   (score + campos `__vanta_*` según la tabla). Invariante roundtrip testeable.
2. **`restore_graph_nodes` es un restore de dominio grafo, no un escritor de memoria
   records:** preserva el valor transportado tal cual (verbatim); no sintetiza campos
   `__vanta_*` nuevos si el export no los traía.
3. **Conflicto entre ambos:** *last-writer-wins físico* sobre el nodo (store único). Un
   `put` de memoria posterior a un restore re-normaliza los campos v2 desde los inputs del
   record; un restore posterior reproduce el estado exportado (que era consistente al
   exportar). Sin merge implícito; no hay camino que lea "mitad y mitad".
4. **Nota + test (SCH-02, cross-check SCH-06):**
   `restore_graph_nodes` roundtrip preserva `confidence_score` (CORE-02 intacto); un
   `memory put` posterior al restore deja `record.confidence == node.confidence_score` y
   campos `__vanta_*` coherentes (invariante §4, `mgr-12:169`).

### D7 — Compat export/import y boundaries de normalización v1 *(carry-over P2-01 #5, anotado para SCH-02)*

**Predicado de import (reconciliación resuelta, D8):** `record_from_export_line`
(`serialization/mod.rs:522-531`) pasa de "versión exacta" a **aceptar `schema_version ∈ {1, 2}`
/ rechazar `> 2`** (TooNew; 0 u otro inválido ⇒ rechazo). v1 se **normaliza** al importar;
export siempre emite `EXPORT_SCHEMA_VERSION = 2`. Errores con el formato único existente
(`Error::Validation { field: "schema_version", … }`).

**Boundaries de deserialización v1 → v2 (los 4 formatos que un campo nuevo atraviesa,
`mgr-10:27-36`):**

| # | Formato | Punto de normalización v1 | Regla |
|---|---|---|---|
| 1 | Node fields KV/WAL (postcard) | `record_from_node` (`:322-357`): campos ausentes ⇒ defaults de la tabla D2 (en particular `valid_at_ms ⇒ created_at_ms`) | Aditivo; WAL forward-compat por header (`wal.rs:18,23`); binario v2 lee DB v1 sin migrar |
| 2 | Mirror `SnapshotRecord` (postcard oculto) | decode **V2-first → V1-fallback** ordenado (postcard `from_bytes` ignora bytes sobrantes → V2-decoding-V1 falla por faltantes; fallback determinista, `mgr-10:155`); al decodificar V1 aplicar defaults; **re-encode siempre V2** | Append de los campos al final del mirror + `From` bidireccional (incluye por fin `superseded_*` — fix del gap `version_history.rs:144-145`). **Verificar en SCH-02 con bytes V1 reales**; si postcard cambiase esa semántica → prefijo de versión en el value |
| 3 | Export JSONL | `record_from_export_line`: v1 normalizado (valid=created, invalid=superseded_at, class=Asserted, conf=D_a, last_validated=None, derived_from=[], quarantine=None×4) | Predicado de arriba; tests roundtrip v2 + import v1 existente (contrato SCH-02) |
| 4 | Header `.vanta.schema` | — (marcador, no datos) | `CURRENT_SCHEMA_VERSION → 2` **al final** de la migración; `MIN_COMPAT_VERSION` queda en 1; v1-binary sobre v2-DB ⇒ `TooNew` |

**Nota para SCH-02 (queda anotado como pendiente de esa task, no de este ADR):** el predicado
`≤2 / >2` y la normalización v1 aplican a los caminos JSONL — MCP (`import` tool), HTTP
(`import_v2` path JSONL) y CLI (`vanta import`), porque todos pasan por
`record_from_export_line`/`import_records` — verificar esa convergencia con test por path.
**Boundary propio (quinto):** el transporte **bulk** (`import_v2` `format:"bulk"` →
`bulk_import_file`/`bulk_import_stream`, `VDBJSON\n` + version byte `0x01` + `Vec<MemoryInput>`)
NO pasa por `record_from_export_line`: su compat es serde-aditiva de `MemoryInput` y su version
byte es independiente del predicado `≤2/>2` — test propio en SCH-02 (bulk v0x01 → v2).

### D8 — Reconciliación de insumos (MGR-10/12/13 → este ADR)

Tabla campo×spec (mitigación del riesgo #1 del bloque: "insumos divergentes → ADR ambiguo").
"Coincide" = los tres docs son consistentes; las divergencias/sub-preguntas quedan resueltas
en la columna final con la decisión tomada acá:

| Ítem | MGR-10 | MGR-12 | MGR-13 | Resolución (este ADR) |
|---|---|---|---|---|
| `valid_at_ms` tipo wire | `u64` + normalización (prefiere sobre `Option`, `:72`) | — | — | **`u64`** + default de insert `:= created_at_ms` (D8) |
| `invalid_at_ms` / valid window | `Option<u64>`; alineado con `superseded_at` por `supersede` (`:76-78`) | — | — | D3 (invariantes 2–3) |
| `confidence_class`/`confidence`/`last_validated`/`derived_from` | — | §3.1 (`:83-119`); D_a=1.0 Q1 | — | D2 + D4 (D4a–D4c) |
| Score `derived` declarado por caller | — | no especificado | — | **D4b: rechazo en boundary** (carry-over #1) |
| `D_a` 1.0 vs default nodo 0.5 | — | Q1=1.0; nota de cambio observable (`:170`) | — | **D4c: 1.0 uniforme, delta aceptado (pre-launch)** (carry-over #2) |
| Punto de escritura nodo | — | "único punto" (§4) sin definir el 2.º escritor | — | **D6: precedencia `restore_graph_nodes` + test** (carry-over #3) |
| Campos cuarentena (`quarantined_at_ms` etc.) | — | — | §2.2 (`:51-58`) | D2 (nombres y códigos finales) |
| Deadline de revisión | — | — | Q2 sub-pregunta abierta (`:207`) | **D5d: 30d default + `keep`** (carry-over #6) |
| Default de import | — | — | Q1 "opt-in; bulk/stream a decidir" (`:206`) | **D5e: opt-in uniforme** |
| `include_quarantined` / default-exclude | (filtros opt-in por defecto, Q1) | — | `:58,74-76` | D2/D5 (coincide; ejes distintos: quarantine excluye, valid no) |
| `min_confidence` | — | §6.1 | — | D2 (opt-in) |
| Nombre config abstención | — | "umbral configurable" | "nombre final en ADR SCH-01" (`:196`) | **D2: `confidence_threshold` (default None=OFF)** |
| Export predicate | acepta {1,2} (`:156`) | "mantener semántica TooNew" (`:152`) | — | **D7: aceptar ∈{1,2} / rechazar >2** (ambas satisfechas: v1 normaliza, >N rechaza) |
| Backfill determinista | §4.1 (`:143-150`) | §3.5 (`:155`) | `None` defaults (`:53-56`) | D4c + §Migration (reglas unificadas) |
| Target de versión | 0.8.0 | "v0.7.0" en §1.4 (`:47`) — **stale** | 0.8.0 | **0.8.0** (`CHANGELOG.md:10`; plan `:35`) — re-baseline |
| Edges (SCH-09) | §4.5 recomendación (`:201-203`) | — | — | **D1c: envelope v2 o FIND; nunca v3** |

Coincidencias sin conflicto: semántica de dos ejes (MGR-10 §1) ↔ ortogonalidad de estados
(MGR-13 I3); "canónico en el record" (MGR-12 §4) ↔ "estado canónico en el storage record"
(MGR-13 §2.3); snapshots/mirror a migrar (MGR-10 §4.2 ↔ MGR-12 §3.5 ↔ MGR-13 §7-2).

## Plan de migración y backfill (0.8.0, con comandos)

> Fuente canónica: `mgr-10-bitemporalidad.md:139-199` (§4). DoD Backlog:938: "ADR aceptado +
> plan de implementación con comandos".

**Reglas del backfill (función pura, determinista por construcción — `mgr-10:143-150`):**

```text
valid_at_ms    := created_at_ms
invalid_at_ms  := superseded_at_ms        # si superseded_by.is_some(); si no, None
confidence_class := Asserted              # MGR-12 :155
confidence       := 1.0                   # D_a (D4c)
last_validated_at_ms := None
derived_from     := []
quarantine (4 campos) := None             # MGR-13 (v1 no puede estar cuarentenado)
```

- **Prohibido** leer reloj (`now_ms()`), aleatoriedad u orden de iteración en el backfill.
- **Idempotente** (re-ejecutar recalcula lo mismo; los campos migrados no alimentan el
  cálculo) y **order-independent** (misma DB v1 ⇒ resultado byte-idéntico — gate SCH-02/SCH-06).

**Orden invariable: expand → backfill → bump** (`mgr-10:195-199`): un crash mid-backfill deja
header v1 + nodos parcialmente backfilled; ambos binarios siguen leyendo (v2 normaliza, v1
ignora campos extra y decodifica snapshots V2 de forma degradada sin corrupción); re-run completa.

**Secuencia con comandos:**

```bash
# 0. Baseline (regla dura -p; nunca nextest sin -p desde la raíz — HARD-05)
cargo nextest run --profile audit -p vantadb --build-jobs 2

# 1. Plan (dry-run informativo; se extiende con "records: N")
vanta migrate plan <dir>

# 2. Integridad pre-migración
vanta migrate check <dir>

# 3. Copia de respaldo (para la doble corrida determinista)
Copy-Item -Recurse <dir> <dir>.v1-copy

# 4. Backfill en seco (reporta N + muestra)
vanta migrate run <dir> --format records --dry-run

# 5. Backfill real (idempotente; crash-safe: re-run completa)
vanta migrate run <dir> --format records

# 6. Cierre: resto de formatos + bump de header (SIEMPRE después del backfill)
vanta migrate run <dir> --format all

# 7. Determinismo: dos copias de la DB v1 → migrar ambas → comparar byte a byte
vanta migrate run <copyA> --format all && vanta migrate run <copyB> --format all

# 8. Roundtrip export/import (v2 y v1)
vanta export --namespace <ns> --out export-v2.jsonl
vanta import --in export-v2.jsonl
vanta import --in tests/fixtures/export-v1.jsonl    # v1 sigue importable (D7)
```

- `records` es un `FormatKind`/paso **nuevo a definir en SCH-02** (hoy `FormatKind` =
  vfile|index|wal|schema, `src/migration.rs:11-56`); si SCH-02 lo integra como sub-paso de
  `schema`, mantener la semántica "header al final".
- Crash-safety y determinismo se instrumentan/verifican en SCH-06 (failpoints existentes +
  SIGKILL mid-migración + doble corrida byte-idéntica).

## Plan de implementación único (SCH-02 → SCH-08; gate F3)

| Task | Ruta | Consume | Entrega | Verificación |
|---|---|---|---|---|
| SCH-02 | vanta-worker | este ADR | Campos + 4 formatos + backfill determinista + import v1 + tests V1–V5 | `cargo nextest run --profile audit -p vantadb --build-jobs 2` |
| SCH-03 | vanta-engine | SCH-02 | `AS OF` (IQL + params) + filtros de ventana + `exclude_superseded` extendido + cursor | tests deterministas de time-travel |
| SCH-04 | vanta-engine | SCH-02 | Scores consumibles (SDK/HTTP/MCP) + `min_confidence` | roundtrip serde + stubs `.pyi`/`.d.ts` + snapshots |
| SCH-05 | vanta-worker | SCH-02 + MGR-13 | Cuarentena operativa + abstención + gates de inyección + ops T1d/T2/T4 | `quarantine_containment` + sticky |
| SCH-06 | vanta-chaos | SCH-02..05 | Migración determinista (doble corrida), time-travel, roundtrip, chaos (failpoints/SIGKILL) | suite + `chaos.yml` |
| SCH-07 | vanta-worker | SCH-02..06 | 8 superficies (Py/TS/Node/WASM + HTTP + MCP + IQL) + `docs/api/` mismo PR | `validate-docs-coverage` + `openapi_yaml_parity` + `sdk_serialization` |
| SCH-08 | vanta-docs | SCH-07 | `UPGRADE.md` §0.8.0 + release notes + corte | Release PR release-plz revisado; smoke upgrade v1→v2 |

**Gate F3 (plan `:35`):** "migración determinista verde + corte **0.8.0** (release-plz)".
El corte se ejecuta **solo** vía release-plz (Regla 7; nunca tags/versión/CHANGELOG a mano).

## Alternatives Considered

### Sistema append-only / system-versioned completo (bitemporal Snodgrass en 0.8.0)
- Pros: AS OF en ambos ejes sin límite de retención; sin gaps de snapshot.
- Cons: **rediseño del engine** (store append-only + GC; los índices HNSW/text/scalar
  apuntan al nodo vivo); rompe el modelo live-record y el hot path de escritura.
- **Rechazada:** stop condition del plan/bloque ("si exige rediseño del engine → acotar +
  FIND"; `mgr-10:105,137`). Queda v1.0.

### Tres migraciones separadas (una por dimensión)
- Pros: PRs más chicos; menor superficie por cambio.
- Cons: tres breakings de schema consecutivos para los mismos consumidores; tres bumps de
  versión; riesgo de inconsistencia entre cortes.
- **Rechazada:** decisión owner 2026-09-24 (Backlog:934) — migración **única**.

### `Derived + Some(confidence)`: clamp silencioso a la fórmula (en vez de rechazo)
- Pros: tolerante con escritores; nunca falla.
- Cons: descarta input del caller sin señal (mudo, difícil de depurar); oculta bugs del
  escritor; rompe la expectativa de que la fórmula tiene un productor único.
- **Rechazada:** D4b — rechazo en boundary con error explícito.

### Mantener `confidence = 0.5` en el backfill (continuidad del default del nodo)
- Pros: neutralidad para consumidores viejos (eviction idéntica).
- Cons: dos semánticas para un campo (histórico "neutral" vs nuevo "declarado"); 0.5 no
  significa nada bajo la semántica nueva; contradice Q1 ratificada (D_a=1.0).
- **Rechazada:** D4c — uniforme 1.0 y deltas documentados (pre-launch).

### Predicado de import "versión exacta" (descartar v1)
- Pros: simplicidad del validador.
- Cons: rompe el contrato "v1 sigue importable"; obliga a re-exportar todo antes de upgradear.
- **Rechazada:** D7 — aceptar ∈{1,2} (v1 normalizado), rechazar >2.

### Cuarentena default ON en import (seguro por defecto)
- Pros: cadena de suministro cubierta sin acción del operador.
- Cons: cambia el flujo de import actual (compat); los registros recién importados
  desaparecerían del retrieval por defecto hasta revisión.
- **Rechazada:** D5e — opt-in uniforme + recomendación documentada.

### Migración de snapshots "lossy" (dejar snapshots históricos sin migrar + FIND)
- Pros: menos superficie frágil en el corte (el mirror postcard es la parte más delicada).
- Cons: el contrato de migración no cierra (historial retenido sin campos nuevos ni fix de
  `superseded_*`); deuda silenciosa en el formato más opaco.
- **Rechazada:** Q4 ratificada (in-place); costo acotado por el cap de retención.

## Consequences

- **Pros:** un solo breaking para consumidores (un bump 0.8.0 con guía única); base
  verificable de "qué era verdad en T" (evidence-before-belief) y de confianza declarada/
  computada por registro; confianza record↔nodo unificada (se cierra el mapeo roto
  bidireccional); cuarentena como contención de entrada sin motor de políticas; todo
  aditivo en wire (`#[serde(default)]`) con rails mecánicos de breaking ya activos.
- **Cons / deuda asumida:** historia transaction acotada (cap 32/key, gap de crash —
  documentado, P27→v1.0); snapshots históricos exigen migración in-place del mirror postcard
  (frágil; failpoints en SCH-06); scores no calibrados (L1/L2 hasta VER-08); sin
  recomputación reactiva ni decaimiento; cuarentena sin detección automática (I6).
- **Cambios observables del corte (Hyrum, documentar en SCH-08/UPGRADE):**
  exports pasan a v2 (v1 sigue importable); `confidence` real en records (eviction más
  conservadora con la config actual; prompt muestra scores reales; D4c); v1-binary sobre
  DB v2 ⇒ `TooNew`; `quarantine` default-exclude **solo** afecta registros que 0.8.0 marque
  (cero efecto sobre datos legacy, `mgr-13:76`); `AS OF`/filtros opt-in (default igual a hoy).
- **Riesgo residual controlado:** edges (SCH-09) no genera segundo breaking — D1c lo ancla
  al envelope v2 o a FIND; insumos divergentes quedaron reconciliados en D8 (tabla explícita).

## Owner sign-off (Regla 5)

> **[OWNER]** **Firmado** — status `accepted` (2026-09-28). Firma y articulación abajo;
> el registro vive también en `docs/dev/tasks/SCH-01.md`.
>
> **Base ya ratificada (2026-09-28, question "Aprobar defaults y avanzar"):** los 16 defaults
> recomendados de MGR-10/12/13 (D0) — no se re-abren.
>
> **Firmado con este ADR (decisiones nuevas):**
>
> 1. **D4b** — rechazo en boundary de `Derived + Some(confidence)` + tests V2/V4/V5.
> 2. **D4c** — backfill uniforme `confidence = D_a = 1.0`; deltas observables de eviction/
>    prompt aceptados (pre-launch).
> 3. **D6** — precedencia del punto de escritura residual `restore_graph_nodes` + test.
> 4. **D8/D7** — default de insert `valid_at_ms := created_at_ms` salvo valor explícito.
> 5. **D7** — import v2 acepta ≤ 2 / rechaza > 2 (v1 normalizado; boundaries anotados SCH-02).
> 6. **D5d** — deadline default de revisión de cuarentena: **30 días** (`keep`; nunca
>    auto-promoción).
> 7. **D5e** — cierre del "bulk/stream a decidir" (mgr-13 Q1): el transporte bulk se declara
>    boundary propio (compat serde-aditiva; ver nota D7/SCH-02).
> 8. Canónicos consolidados: tabla de campos (D2), semántica temporal (D3), alcance (D1d),
>    plan de migración + implementación (§ arriba), reconciliación (D8).
>
> - **Articulación:** el owner aprueba el **corte único v2** (una sola migración, un solo breaking pre-launch) con el set completo: los 16 defaults ratificados + las 7 decisiones nuevas (D4b · D4c · D6 · D8/D7 · D7 · D5d · D5e) tal como quedaron articuladas con evidencia en este ADR.
> - **Firma:** Eros (owner) — 2026-09-28
> - **Riesgos aceptados:** deltas observables de confianza (D4c: eviction/prompt, pre-launch); fragilidad del mirror postcard en la migración in-place (mitigada: V2-first→V1-fallback + tests SCH-02/SCH-06); edges dentro del envelope v2 (D1c, sin segundo breaking).

## References

- Contrato origen: master roadmap Task 26 (`docs/dev/plans/2026-09-26-master-roadmap.md:677-701`);
  gate F3 `:35`. Backlog:938 (pre-req Cierre MGR) + :934 (migración única / breaking
  pre-launch) + :939-946 (SCH-02..09).
- Insumos (Cierre MGR ✅): `docs/dev/research/mgr-10-bitemporalidad.md` (commit `b6614e1e`) ·
  `mgr-12-confianza.md` (`50df4efd`) · `mgr-13-cuarentena.md` (`72f29720`).
- Precedentes: ADR-028 (supersession), ADR-044 (acumulado breaking 0.x), ADR-045 (naming
  freeze + régimen de breaking pre-usuarios).
- Política/rails: `docs/api/VERSIONING.md:16-27` · `docs/api/COMPATIBILITY.md:17-30,53-76` ·
  `docs/api/DEPRECATIONS.md` · `release-plz.toml:22` · `docs/CHANGELOG.md:10`.
- Código citado: `src/sdk/types/record.rs:59-78,103-138,142-160,231-261` ·
  `src/sdk/serialization/mod.rs:12-35,305-357,433-495,498-531` ·
  `src/sdk/version_history.rs:76-90,112-148` · `src/schema.rs:11-13,84-98` ·
  `src/sdk/api/graph.rs:203-240` · `src/eviction.rs:38-45,63-68` · `src/llm.rs:939-944` ·
  `src/executor.rs:219-242` · `src/config.rs:285,380` · `src/node/unified.rs:42,92` ·
  `src/cli_handlers/migrate.rs:175-279` · `src/migration.rs:11-56` · `src/wal.rs:18,23,611-642`.
- Task file de esta decisión: `docs/dev/tasks/SCH-01.md`.
