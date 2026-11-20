---
title: "Backlog de extracción — Hindsight"
kind: research
status: active
description: "Siete candidatas del informe Hindsight: publicación con fuentes vivas, traza de fusión, outcome de refresh, ventana temporal, partición por ámbito, validación de extracción y reflect con cancelación"
tags: [extraccion, backlog, hindsight]
---

# Backlog de extracción — Hindsight

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/hindsight.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-HIN-01 | 🔴 | Publicación de consolidación ligada a revisiones vivas de fuente | `vanta-memory/src/core/record/l1_writer.rs::write_memory:131-158`, `apply_dedup_batch:389` | 🟠 | 🔴 P0 | ⬜ Candidata | Precondición de commit: capturar `(id, versión)` de fuentes y metadata antes del LLM y revalidar al publicar (Applied/StaleSources/DeletedSources) · Un resultado tardío no debe resucitar evidencia borrada ni perder tags añadidos durante la inferencia · Acción mínima: validación condicional breve en el writer existente; reencolar solo conflictos recuperables · **Trazabilidad:** `informe §2 ficha A y complemento · evidencia engine/consolidation/consolidator.py:706-751 · tags consolidator.py:3445-3495` | X-HIN-05 la alimenta; X-HIN-03 y X-HIN-06 la usan; converge con C6/C14 de `CONSOLIDATED.md` | Primitiva condicional/atómica del SDK a identificar (no atribuir CAS a `write_memory`); checkpoints existentes |
| X-HIN-02 | 🟡 | Explicación y balance por brazo en la fusión de recall | `vanta-memory/src/core/record/l1_reader.rs:321-380` (pools + `rrf_merge`) | 🟡 | 🟠 P1 | ⬜ Candidata | Traza de fusión con contribuciones por brazo, límite previo por brazo y exclusiones registradas · Un brazo expansivo no debe consumir todo el pool ni un ID en dos brazos cobrar doble crédito · Acción mínima: ampliar la traza existente con límites antes de fusionar; sin otra implementación de RRF ni constantes copiadas · **Trazabilidad:** `informe §2 ficha B · evidencia engine/search/fusion.py:8-26 (cap), 29-109 (RRF)` | X-HIN-04 explica el brazo temporal; converge con C7 de `CONSOLIDATED.md` | IDs estables y scope; diagnóstico existente extensible |
| X-HIN-03 | 🟡 | Resultado tipado de refresh: preservado, publicado o rechazado | `vanta-memory/src/core/reflection/mod.rs`, `core/scene/auto_consolidate.rs`, `services/scheduler.rs:110-145`, `core/memory_generation_log/` | 🟡 | 🟠 P1 | ⬜ Candidata | Outcome de refresh con estado, revisión de fuente, ID preservado y razón, persistido en generation log/checkpoint · "Sin nueva evidencia", "candidato rechazado" y "escrito" deben distinguirse; un fallo no es una página vacía bien escrita · Acción mínima: añadir solo los estados faltantes al resultado ya emitido y exponerlos · **Trazabilidad:** `informe §2 ficha C · evidencia engine/mental_model_refresh.py:27-65` | X-HIN-01 aporta vigencia; X-HIN-07 relacionada; converge con C14 de `CONSOLIDATED.md` | X-HIN-01; generation log y checkpoint existentes; API de logging actual |
| X-HIN-04 | 🟡 | Ventana temporal explícita y semántica de fechas en recall | `vanta-memory/src/core/hooks/auto_recall.rs::RecallConfig`, `perform_auto_recall_governed` | 🟡 | 🟠 P1 | ⬜ Candidata | Resolución de ventana con origen explicit/inferred/none y errores de rango invertido o fecha ambigua sin zona · La fecha de ingesta no debe reemplazar la fecha del hecho ni el "ahora" del caller; lo desconocido no se completa silenciosamente · Acción mínima: resolver/normalizar fechas al borde de la ruta autorizada y propagar diagnóstico, sin backfill especulativo · **Trazabilidad:** `informe complemento ficha D · evidencia engine/search/retrieval.py:129-248, models.py:104-183 (occurred/mentioned)` | X-HIN-02 explica el brazo temporal; X-HIN-01 condiciona síntesis histórica; converge con C4 de `CONSOLIDATED.md` | Reloj del caller y provenance; compatibilidad del modo legacy por comprobar |
| X-HIN-05 | 🔴 | Particionado de consolidación por ámbito resuelto antes del LLM | `vanta-memory/src/core/record/l1_writer.rs::plan_write`, `default_tenancy`, `core/scene/auto_consolidate.rs` | 🟠 | 🔴 P0 | ⬜ Candidata | Agrupación canónica de lotes por scope resuelto, con locks breves en orden total · Dos fuentes autorizadas por separado no deben mezclarse en el mismo prompt por compartir tags; ámbito vacío o no autorizado falla cerrado · Acción mínima: resolver ámbitos antes de inferencia, capturar revisiones y publicar vía X-HIN-01 · **Trazabilidad:** `informe complemento ficha E · evidencia engine/consolidation/consolidator.py:655-703 (_consolidation_batch_key, _scope_sort_key)` | X-HIN-01 publica sus resultados; X-HIN-06 la requiere; converge con C3 de `CONSOLIDATED.md` | X-HIN-01 y X-HIN-03; permisos provienen del caller autorizado, no del lote |
| X-HIN-06 | 🟠 | Validación de extracción por item y contabilidad de rechazos | `vanta-memory/src/core/record/l1_writer.rs::plan_write`, `write_memory`, `core/memory_generation_log/` | 🟡 | 🔴 P0 | ⬜ Candidata | Resultado por item: aceptados y rechazados por ID con errores duplicado/desconocido/malformed; correlación ID↔embedding validada antes de persistir · Un lote con resultados duplicados o no solicitados no puede anunciar ingesta completa ni descartar evidencia sin diagnóstico · Acción mínima: validación determinista antes de embed y registro de parciales en el log existente · **Trazabilidad:** `informe complemento ficha F · evidencia engine/retain/orchestrator.py:1378-1485 · issue vectorize-io/hindsight#6439` | X-HIN-05 (scopes), X-HIN-01 (revisión), X-HIN-03 (outcome); converge con C13 de `CONSOLIDATED.md` | X-HIN-05, X-HIN-01, X-HIN-03; proveedor externo opcional; degradación sin embedding ya admitida por el writer |
| X-HIN-07 | 🟡 | Adapter de reflect con herramientas, citas y cancelación | `vanta-memory/src/core/reflection/mod.rs`, `context_engine/engine.rs::assemble_with_recall` | 🟠 | 🔵 P3 | ⬜ Candidata | Adapter de lectura que devuelve respuesta con IDs/versiones de evidencia y errores denied/cancelled/provider/budget · Un texto generado no se convierte en hecho automáticamente y ninguna herramienta amplía permisos ni persiste por defecto · Acción mínima: mapear herramientas a lectura autorizada y propagar cancelación cooperativa, sin runtime agentic nuevo · **Trazabilidad:** `informe complemento ficha G · evidencia engine/reflect/agent.py:503-592, models.py:68-74 (raise_if_cancelled)` | X-HIN-01 si hay publicación; X-HIN-02/X-HIN-04 aportan evidencia explicable; converge con C10 de `CONSOLIDATED.md` | Reflexión y assembly existentes; X-HIN-01 solo si se solicita publicación aparte |

## Fichas completas

> Expansión de cada candidata a partir del informe `docs/dev/extraccion/agent-memory-2026-11-07/hindsight.md` (§2 fichas A–C y «Complemento de autoría» complementos A–C + fichas D–G). Todos los contratos son **hipotéticos** — nombres de interfaz propuestos, no APIs existentes de VantaDB. Las referencias `archivo:líneas` citan el clon `C:\Users\Eros\hindsight` en SHA `eb963cb5a929d92c081371df64c2d5c2fe3351af` (rutas relativas a `hindsight-api-slim/hindsight_api/`); las rutas VantaDB citan el baseline `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` y requieren revalidación contra HEAD. Priorización del informe §1: primero A/E/F; después B/D/C; G solo ante requisito real de razonamiento.

### X-HIN-01 — Publicación de consolidación ligada a revisiones vivas de fuente

- **Problema:** evitar que un resultado LLM tardío resucite evidencia borrada o ya modificada: una referencia de procedencia describe origen; una precondición de commit verifica que ese origen todavía sea válido.
- **Contrato hipotético (tipos propuestos):** `SourceRevision { id, version }`, `ConsolidationProposal { sources, output }` y resultado `Applied | StaleSources | DeletedSources`; firma propuesta `apply_consolidation(scope: &RecallScope, p: ConsolidationProposal) -> Result<ApplyOutcome, ConsolidationError>` con errores `Unauthorized | Storage | Cancelled`. Aplicar solo si revisiones y ámbito siguen coincidiendo; tras commit devolver identidad durable, no fingir rollback de LLM. Estos nombres no existen acreditados.
- **Fuente de evidencia (clone):** `engine/consolidation/consolidator.py:706-751` (`_filter_live_source_memories`, `_sources_changed_since_read`: relee fuentes vivas bajo bloqueo y detecta `updated_at` cambiado antes de aplicar la decisión del LLM) [H3]; pin de tags `consolidator.py:3445-3495` (`lock_observation_tags`/`current_tags`, distinto del guard de fuentes) [H-tags, vía fix upstream #4897 del issue #4831].
- **Seam VantaDB:** `vanta-memory/src/core/abstractions/types.rs` (`MemoryRecord:71-138`, fuentes/versión/supersession); `vanta-memory/src/core/record/l1_writer.rs::write_memory:131-158` y `apply_dedup_batch:389`, que ya persisten decisiones.
- **Acciones:**
  1. Revalidar el writer actual contra HEAD.
  2. Identificar la primitiva condicional/atómica adecuada del SDK — no atribuir CAS a `write_memory`, cuyo persist/log son etapas separadas.
  3. Capturar `(id, versión)` de fuentes **y** metadata actual (tags) antes del LLM.
  4. Validar revisiones y metadata al commit dentro de la misma sección condicional.
  5. Reencolar solo conflictos recuperables con checkpoint; versionar metadata de propuesta sin reindexar vectores (lectores antiguos ignoran campos opcionales; rollback desactiva publicación y conserva fuentes).
- **Dependencias:** primitiva condicional/atómica del SDK por identificar; checkpoints existentes. X-HIN-05 la alimenta; X-HIN-03 y X-HIN-06 la usan; converge con C6/C14 de `CONSOLIDATED.md`. Alternativas documentadas: solo provenance no impide TOCTOU; locks durante el LLM son inaceptables → validación condicional breve en el writer existente.
- **FMEA:** check separado del write permite TOCTOU → mitigar dentro de la misma sección condicional; locks durante el LLM provocan contención (alternativa rechazada); reintentos sin invalidación repiten decisiones obsoletas; snapshot de recall anterior al LLM → perder tags añadidos o resucitar retirados → revalidar fuentes **y** metadata actual en publicación (una revisión de fuentes no cubre ambas).
- **Criterios de aceptación propuestos:** borrar o editar una fuente mientras se calcula una síntesis impide publicar el candidato antiguo; repetir el mismo trabajo no duplica ni pierde procedencia; namespaces diferentes no comparten fuentes; la aceptación separa edición de fuentes de edición de tags durante el LLM. Contadores `applied/stale/deleted/storage/cancelled`, sin texto.
- **Criterio negativo:** un resultado tardío nunca publica sobre fuentes editadas o borradas; no se publica tras cancelación previa al commit; tras commit no se finge rollback del trabajo del LLM.
- **Independencia:** primera tanda del informe (A/E/F); es prerequisito de publicación para X-HIN-03/X-HIN-05/X-HIN-06 pero no requiere ninguna otra candidata para su validación condicional en el writer existente.
- **No objetivos:** introducir PostgreSQL; mantener locks durante la inferencia; añadir otro runtime. Abierto: primitiva condicional adecuada del SDK.

### X-HIN-02 — Explicación y balance por brazo en la fusión de recall

- **Problema:** un brazo expansivo puede consumir todo el pool de rerank; contribuciones por brazo deben conservar ranking y scores separados sin normalizar métricas distintas como iguales.
- **Contrato hipotético (tipos propuestos):** `ArmCandidate { id, arm, rank, raw_score }` y `FusionTrace { contributions, exclusions }`; firma propuesta `trace_fusion(candidates: &[ArmCandidate], selected: &[String]) -> Result<FusionTrace, TraceError>` con errores de identidad/ordinal inválido; invariante: una identidad con múltiples contribuciones y score bruto no normalizado entre métricas. Presupuesto por brazo configurado en la capa de dominio; resultado compatible con el diagnóstico existente.
- **Fuente de evidencia (clone):** `engine/search/fusion.py:8-26` (`cap_per_source`) y `:29-109` (`reciprocal_rank_fusion`, que conserva ranks y scores por brazo; alternativa interleave para problemas de deduplicación) [H2].
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_reader.rs:321-380` (`recall_candidates` con pools keyword/vector y `rrf_merge` ya existentes) — no proponer otra implementación paralela de RRF.
- **Acciones:**
  1. Inventariar las trazas actuales de fusión.
  2. Ampliar la traza que ya exista con contribuciones y exclusiones.
  3. Imponer límites por brazo antes de fusionar.
  4. Registrar el truncamiento con razón.
  5. Medir únicamente con el protocolo de METHOD; persistencia de traza efímera por defecto con campos opcionales en API (rollback omite la traza y conserva el orden actual).
- **Dependencias:** IDs estables y scope; diagnóstico existente extensible. X-HIN-04 explica el brazo temporal; converge con C7 de `CONSOLIDATED.md`. Abierto: contrato de trace existente del core vs legacy.
- **FMEA:** normalizar scores de distintas métricas como si fueran iguales; duplicado intrabrazo → doble crédito → atribución engañosa (detectar contribuciones repetidas y rechazarlas); límites que excluyan evidencia rara; ordinal de brazo equivocado → atribución errónea (hipótesis del informe, requiere recorrido del llamador); cancelación descarta la traza incompleta, no se convierte en fallo de escritura.
- **Criterios de aceptación propuestos:** un ID presente en dos brazos aparece una vez con dos contribuciones; brazo ausente se explica; cambiar el orden de entrada no cambia su identidad; un brazo fallido no equivale a brazo vacío. Observabilidad con cantidades, tiempos y enums — no query ni payload.
- **Criterio negativo:** el score bruto nunca se presenta normalizado entre métricas; una contribución repetida se rechaza en lugar de cobrar doble crédito; un brazo fallido no se reporta como vacío.
- **Independencia:** segunda tanda del informe (diagnóstico); útil sin X-HIN-01 y no requiere las candidatas de publicación; X-HIN-04 explica su brazo temporal.
- **No objetivos:** afirmar mejora de recall; copiar constantes upstream como configuración de evaluación; cambiar constantes ni presupuesto de evaluación; otra implementación de RRF.

### X-HIN-03 — Resultado tipado de refresh: preservado, publicado o rechazado

- **Problema:** distinguir «sin nueva evidencia», «candidato rechazado» y «escrito»: el éxito de ejecución no implica éxito de publicación y un fallo no debe confundirse con una página vacía escrita correctamente.
- **Contrato hipotético (tipos propuestos):** `RefreshOutcome { status, source_revision, preserved_id, reason }`; firma propuesta `finish_refresh(expected_version: u64, result: RefreshOutcome) -> Result<RefreshOutcome, RefreshError>` con estados preservado/publicado/rechazado y errores `Stale | Cancelled | Storage`. Conservar contenido previo cuando no haya candidato válido. No se afirma que Vanta carezca de todos estos estados.
- **Fuente de evidencia (clone):** `engine/mental_model_refresh.py:27-65` (`RefreshMode`, `ModeFallbackReason`, `RefreshOutcome`, `RefreshOperationOutcome`: distinguen full/delta, fallback, contenido preservado y candidato fallido) [H7].
- **Seam VantaDB:** `vanta-memory/src/core/reflection/mod.rs` y `vanta-memory/src/core/scene/auto_consolidate.rs`; `vanta-memory/src/services/scheduler.rs:110-145` (recupera tareas y ejecuta mantenimiento); `vanta-memory/src/core/memory_generation_log/` para persistir outcome.
- **Acciones:**
  1. Comparar los resultados ya emitidos por reflexión/escenas.
  2. Añadir solo los estados faltantes.
  3. Persistirlos en generation log/checkpoint existentes.
  4. Exponerlos en Studio.
  5. Migración aditiva de estado/causa sin reindexar fuentes; consulta de resultado antes de repetir trabajo; rollback deja de producir nuevos estados y conserva versiones válidas.
- **Dependencias:** X-HIN-01 para vigencia de fuentes; API de logging existente. X-HIN-07 relacionada; converge con C14 de `CONSOLIDATED.md`. Alternativa documentada y rechazada: solo log textual no sirve al consumidor → resultado tipado.
- **FMEA:** éxito de ejecución no implica éxito de publicación; un resultado vacío puede borrar contenido útil → nunca reemplazar una página válida por vacío; el historial puede revelar datos borrados; excepción tras persistencia → retry ciego → duplicado (detectar por identidad/revisión y checkpoint; mitigar consultando el resultado antes de repetir).
- **Criterios de aceptación propuestos:** refresh vacío/fallido conserva la versión válida, queda auditable y no se presenta como nuevo conocimiento; el operador distingue preservación de escritura; reinicio entre persistencia y log conserva la decisión durable o muestra reconciliación pendiente — no éxito falso. Métricas por outcome sin contenido.
- **Criterio negativo:** una página válida nunca se reemplaza por un resultado vacío; un refresh rechazado/fallido no se reporta como publicado; no se promete transacción global entre writer y log.
- **Independencia:** segunda tanda del informe; requiere X-HIN-01 solo para la vigencia de fuentes en publicación — los estados y su persistencia en generation log son un delta propio.
- **No objetivos:** un agente reflect generalista; copiar prompts de inferencia/evaluación; visibilidad de estado entre writer/log como transacción global (abierto).

### X-HIN-04 — Ventana temporal explícita y semántica de fechas en recall

- **Problema:** un agente pregunta «qué prefería entonces»; la fecha de ingesta no debe reemplazar la fecha del hecho ni el «ahora» del caller, y lo desconocido no se completa silenciosamente.
- **Contrato hipotético (tipos propuestos):** `resolve_time(input: TemporalRecallInput) -> Result<ResolvedWindow, TemporalError>`; entrada = referencia UTC y ventana opcional; salida con origen `explicit | inferred | none`; errores de rango invertido y fecha inválida. Una ventana explícita no se reinterpreta y no autoriza acceso.
- **Fuente de evidencia (clone):** `engine/search/retrieval.py:129-248` (`retrieve_all_fact_types_parallel` resuelve `TemporalWindow` antes de `recall_unified`) [H5]; `engine/models.py:104-183` (`Document`/`MemoryUnit`: `occurred_start/end` cuándo pasó, `mentioned_at` cuándo se mencionó, `created_at/updated_at` persistencia/revisión — no inferir bitemporalidad transaccional) [H12].
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs::RecallConfig` y `perform_auto_recall_governed` [V2], más filtros del SDK ya existentes; la compatibilidad del modo legacy debe comprobarse.
- **Acciones:**
  1. Identificar los campos temporales disponibles.
  2. Normalizar fechas al borde de la ruta autorizada.
  3. Conservar los campos ausentes como desconocidos.
  4. Aplicar el filtro en la ruta autorizada.
  5. Propagar el diagnóstico de resolución; metadata aditiva/versionada, sin backfill especulativo (rollback ignora campos nuevos).
- **Dependencias:** X-HIN-02 explica el brazo temporal; X-HIN-01 condiciona la síntesis histórica; reloj del caller y provenance, sin proveedor obligatorio. Converge con C4 de `CONSOLIDATED.md`.
- **FMEA:** timezone omitida → conversión local → hecho fuera de ventana (detectar fecha ambigua, rechazar/explicitar la zona; no completar silenciosamente).
- **Criterios de aceptación propuestos:** el caso normal respeta los límites; el adverso distingue evento sin fecha de evento fuera de rango; la cancelación no publica derivados; métricas por causa/origen, nunca timestamps personales en logs.
- **Criterio negativo:** una fecha desconocida o ambigua nunca se completa ni reinterpreta silenciosamente; una ventana explícita no autoriza acceso adicional.
- **Independencia:** segunda tanda del informe; contrato temporal auditable sobre filtros/budgets existentes — no requiere X-HIN-01 salvo para síntesis histórica publicada.
- **No objetivos:** bitemporalidad MVCC; nuevo modelo cognitivo; mínimo declarado: entrada explícita. Abierto: tratamiento contractual de fechas desconocidas.

### X-HIN-05 — Particionado de consolidación por ámbito resuelto antes del LLM

- **Problema:** dos fuentes autorizadas separadamente no deben mezclarse en el mismo prompt por compartir tags originales; el límite es la entrada al LLM de consolidación, no solo la salida de recall.
- **Contrato hipotético (tipos propuestos):** `partition_consolidation(items: &[ScopedSource]) -> Result<Vec<ScopedBatch>, ScopeError>`; entrada = revisiones + ámbitos resueltos; salida = lotes homogéneos; errores scope inválido/no autorizado. Una fuente fanout no cambia de ámbito por tomar tags del primer elemento.
- **Fuente de evidencia (clone):** `engine/consolidation/consolidator.py:655-703` (`_consolidation_batch_key:655-694` distingue combined/scope/fanout; `_scope_sort_key:697-703` — orden de locks total) [H3].
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs::plan_write` y `default_tenancy` [V1]; `vanta-memory/src/core/scene/auto_consolidate.rs` [V3]; sellos agent/team e `InjectionPolicy` existentes [V1][V2].
- **Acciones:**
  1. Resolver ámbitos antes de la inferencia.
  2. Ordenar claves canónicamente.
  3. Capturar revisiones.
  4. Ejecutar la inferencia fuera de locks.
  5. Publicar vía X-HIN-01; el checkpoint incluye la versión del particionado, sin reindexar (rollback pausa esos jobs, no mezcla).
- **Dependencias:** X-HIN-01 (vigencia) y X-HIN-03 (outcomes); los permisos vienen del caller autorizado, no del lote. Converge con C3 de `CONSOLIDATED.md`. Alternativas documentadas: filtrar la respuesta llega tarde; serializar todo desperdicia paralelismo → agrupación canónica con locks breves ordenados.
- **FMEA:** ámbito vacío interpretado de forma distinta → lote compartido → exfiltración al proveedor (detectar igualdad de scopes canónicos y mitigar fail-closed).
- **Criterios de aceptación propuestos:** fuentes de tenants distintos nunca comparten lote; scopes solapados adquieren locks en el mismo orden; la cancelación libera ownership; métricas de cardinalidad/denegados sin tags ni textos.
- **Criterio negativo:** ninguna fuente se agrupa fuera de su ámbito canónico resuelto; un scope no autorizado falla cerrado, nunca degrada a lote compartido.
- **Independencia:** primera tanda del informe (A/E/F); depende de X-HIN-01 para publicar y de X-HIN-03 para outcomes, pero el particionado/validación es un delta propio sobre `plan_write`/`default_tenancy`.
- **No objetivos:** mínimo = particionado y validación; no una ACL paralela. Abierto: fanout permitido.

### X-HIN-06 — Validación de extracción por item y contabilidad de rechazos

- **Problema:** un lote con resultado duplicado o no solicitado no puede anunciar ingesta completa ni descartar evidencia sin diagnóstico; abortar todo pierde parciales válidos e ignorar errores oculta pérdida.
- **Contrato hipotético (tipos propuestos):** `validate_extraction(expected: &[SourceRevision], returned: &[ExtractionItem]) -> Result<ExtractionBatchOutcome, ExtractError>`; salida = aceptados + rechazados por ID; errores duplicado/desconocido/malformed; ningún resultado crea identidad que no fue solicitada.
- **Fuente de evidencia (clone):** `engine/retain/orchestrator.py:1378-1485` (`_extract_and_embed:1378-1450`, `RetainFactFilter.review:1453-1485`) [H1]; issue [vectorize-io/hindsight#6439](https://github.com/vectorize-io/hindsight/issues/6439) (diccionario por `custom_id` sobrescribe duplicado — límite de evidencia declarado en el informe).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs::plan_write` y `write_memory` [V1] — el writer ya admite embedding opcional fallido, degradación legítima a preservar; `vanta-memory/src/core/memory_generation_log/` [V3].
- **Acciones:**
  1. Validar antes del embed.
  2. Conservar la correlación ID↔resultado al filtrar.
  3. Publicar solo válidos autorizados.
  4. Registrar el parcial con causa.
  5. Metadata opcional sin reindexar; rollback suspende el extractor nuevo, no elimina registros previamente válidos.
- **Dependencias:** X-HIN-05 (scopes), X-HIN-01 (revisión), X-HIN-03 (outcome); proveedor externo opcional. Converge con C13 de `CONSOLIDATED.md`.
- **FMEA:** el índice cambia al filtrar → embedding de otro fact → corrupción semántica (detectar correspondencia ID/dimensión; mitigar con zip validado por identidad).
- **Criterios de aceptación propuestos:** el caso normal produce un ID por resultado; adversos cubiertos: duplicados, omitidos, unknown y cancelación antes de persistir; conteos por causa sin payload.
- **Criterio negativo:** ningún resultado crea una identidad no solicitada; un lote con rechazos no se anuncia como ingesta completa; la degradación sin embedding (ya admitida) no se confunde con extracción incoherente.
- **Independencia:** primera tanda del informe (A/E/F); requiere el ámbito de X-HIN-05, la revisión de X-HIN-01 y el outcome de X-HIN-03 para el circuito completo — la validación determinista por item es su núcleo propio.
- **No objetivos:** mínimo = validación determinista; no prompt ni evaluador nuevo. Abiertos: política de retry parcial y quién autoriza retención de errores.

### X-HIN-07 — Adapter de reflect con herramientas, citas y cancelación

- **Problema:** una respuesta necesita combinar observación y evidencia original sin convertir un texto generado en un hecho automáticamente; reflect es un bucle con herramientas, no simplemente devolver vecinos.
- **Contrato hipotético (tipos propuestos):** `reflect_evidence(request: ReflectEvidenceRequest) -> Result<ReflectEvidenceResult, ReflectError>`; entrada = scope + fuentes + budget; salida = respuesta con IDs/versiones y estado; errores `denied | cancelled | provider | budget`. Ninguna herramienta amplía permisos ni persiste por defecto.
- **Fuente de evidencia (clone):** `engine/reflect/agent.py:503-592` (`run_reflect_agent`: herramientas de mental models, observations y facts; contrato jerárquico) [H4]; `engine/models.py:68-74` (`RequestContext.raise_if_cancelled`, cancelación cooperativa) [H12].
- **Seam VantaDB:** `vanta-memory/src/core/reflection/mod.rs` [V3] y `vanta-memory/src/context_engine/engine.rs::assemble_with_recall` [V4] — reflexión y context assembly ya existen; solo un adapter opcional que diferencia consulta de publicación.
- **Acciones:**
  1. Mapear herramientas a lectura autorizada.
  2. Propagar la cancelación cooperativa existente.
  3. Retornar citas con IDs/versiones dentro del scope.
  4. Log de estado opcional, sin migración de fuente ni reindexación; rollback deshabilita el adapter y conserva el recall directo.
- **Dependencias:** X-HIN-01 solo si se solicita publicación aparte; X-HIN-02/X-HIN-04 aportan evidencia explicable. Converge con C10 de `CONSOLIDATED.md`. Alternativas documentadas: usar recall directamente para preguntas simples; reservar herramientas para requisito explícito; rechazar un nuevo runtime agentic.
- **FMEA:** desconexión → el LLM continúa y el retry duplica trabajo → agotamiento (detectar operación abandonada; mitigar con cancelación cooperativa/admisión existente — sin prometer cancelar la factura de una solicitud remota ya enviada).
- **Criterios de aceptación propuestos:** las citas resuelven dentro del scope; una fuente borrada impide la publicación; cancelar libera recursos locales; métricas de llamadas/outcome sin prompts, respuesta ni claves del proveedor. Aceptación pendiente por etapa/proveedor/transporte — no se reprodujo desconexión.
- **Criterio negativo:** un texto generado nunca se convierte en hecho por defecto; ninguna herramienta amplía permisos ni persiste sin solicitud explícita.
- **Independencia:** tercera tanda del informe — solo ante un requisito real de razonamiento; adapter de lectura opcional sobre reflexión/assembly existentes, no un reflect autónomo generalista.
- **No objetivos:** reflect autónomo generalista; nuevo runtime agentic. Abiertos: utilidad real y contrato de salida por SDK.
