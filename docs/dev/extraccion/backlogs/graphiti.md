---
title: "Backlog de extracción — Graphiti"
kind: research
status: active
description: "Cinco candidatas del informe Graphiti: resolución temporal de hechos, identidad en dedup, retirada de soporte, traza por brazo e inspector Studio"
tags: [extraccion, backlog, graphiti]
---

# Backlog de extracción — Graphiti

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/graphiti.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-GRT-01 | 🔴 | Resolución temporal de intervalos de hecho con razón auditable | `vanta-memory/src/core/record/l1_dedup.rs`, `l1_writer.rs::record_input` | 🟡 | 🔴 P0 | ⬜ Candidata | Comparación pura de intervalos (KeepBoth/CloseExisting/CloseIncoming/Undetermined) para que una llegada tardía no invalide un hecho vigente · El delta es separar fecha efectiva del hecho del tiempo de conocimiento, no "añadir timestamps" · Acción mínima: función de dominio con decisión y razón; aplicación mutante solo tras garantías de publicación L1 · **Trazabilidad:** `informe §3.1 · evidencia graphiti_core/edges.py:263-291, graphiti_core/utils/maintenance/edge_operations.py:715-1035` | X-GRT-02 la consume como gate; converge con C4/C14 de `CONSOLIDATED.md` | Identidad/provenance existentes; `persist_planned` delete→put aclarado antes de modo mutante (informe §5.1.1) |
| X-GRT-02 | 🔴 | Elegibilidad de identidad en dedup: recurrencia no es duplicado | `vanta-memory/src/core/record/l1_dedup.rs` (precondición de decisión) | 🟡 | 🟠 P1 | ⬜ Candidata | Gate SameOccurrence/DistinctOccurrence/NeedsReview antes de dedup exacto, semántico y por lote · Fact idéntica con intervalos disjuntos describe ocurrencias distintas y no reutiliza ID · Acción mínima: extender decisiones actuales de dedup con gate puro y shadow outcomes, sin deduplicador paralelo · **Trazabilidad:** `informe §3.2 · evidencia graphiti_core/utils/maintenance/edge_operations.py:715-1035 · issue getzep/graphiti#1865` | X-GRT-01 (intervalos); converge con C4 de `CONSOLIDATED.md` | X-GRT-01; publicación concurrente necesita revisión segura (informe §3.6) |
| X-GRT-03 | 🟠 | Retirada de soporte por revisión con derivados marcados stale | `vanta-memory/src/core/record/l1_writer.rs` (metadata de fuentes), lifecycle, `src/cli_handlers/index.rs` | 🟠 | 🟠 P1 | ⬜ Candidata | Plan de retirada de una fuente: enumera soportes restantes, marca derivados stale y preserva el hecho con soporte vivo · Eliminar una conversación no debe borrar un hecho sustentado por otra ni servir resumen retirado como vigente · Acción mínima: plan read-only → estados de derivado → retirada con garantías de reemplazo → regeneración por jobs existentes · **Trazabilidad:** `informe §3.3 · evidencia graphiti_core/graphiti.py::remove_episode:2174-2251` | X-GRT-01 (revisión), X-GRT-05 (estado stale visible); converge con C5/C14 de `CONSOLIDATED.md` | Seguridad de reemplazo L1 antes de retirar físicamente (informe §5.1.1); scheduler/jobs existentes |
| X-GRT-04 | 🟡 | Trazabilidad por brazo y receta solicitada vs efectiva en recall | `vanta-memory/src/core/hooks/auto_recall.rs::RecallConfig`, `search_records_core`, `assemble_with_recall` | 🟡 | 🟠 P1 | ⬜ Candidata | Adjuntar a cada resultado origen de evidencia, estado por brazo (Complete/Skipped/Failed) y receta solicitada/efectiva · "No hay hechos" no debe confundirse con "falló embedding" ni un brazo omitido aparecer exitoso · Acción mínima: campos opcionales backwards-compatible en el recall existente; sin otro índice ni assembler · **Trazabilidad:** `informe §3.4 · evidencia graphiti_core/search/search.py:70-255, graphiti_core/search/search_config.py:32-139` | X-GRT-05 lo consume para explicar retrieval; converge con C7/C8 de `CONSOLIDATED.md` | Contrato de scopes/errores existente; independiente de X-GRT-01 y de la escritura mutante |
| X-GRT-05 | 🟢 | Inspector read-only de intervalos, soporte y razón de supersession | Studio existente: `MemoryLens.tsx`, `ActivityPanel.tsx`, `RetrievalLens.tsx` (worktree `develop-desktop`) | 🟠 | 🟡 P2 | ⬜ Candidata | Proyección autorizada de intervalos, fuentes y motivo de supersession en el Studio existente · Un usuario debe distinguir cambio de mundo, corrección, llegada tardía y fallo de derivado; Unknown no se dibuja como fecha cero · Acción mínima: proyección desde campos existentes y etiquetado de faltantes; acciones mutantes fuera del mínimo read-only · **Trazabilidad:** `informe §3.5 · evidencia graphiti_core/edges.py:263-291 · seams Studio snapshot 0ad38c542 (informe §8.2)` | X-GRT-03 (estado stale), X-GRT-04 (explicación); converge con C16 de `CONSOLIDATED.md` | X-GRT-04 para explicar retrieval; X-GRT-03 para estado stale; vista básica no bloqueada por migración temporal |

## Fichas completas

> Las fichas expanden la tabla resumen con el contenido del informe
> `docs/dev/extraccion/agent-memory-2026-11-07/graphiti.md` (§3.1–3.5, tabla de
> ownership §3.6, contratos previos §5.1 y seams Studio §8.2). Todos los
> contratos son **hipotéticos** — interfaces deliberadamente no compilables cuyos
> tipos deben adaptarse a los existentes; no son una API implementada. Las
> referencias `archivo:línea` son del clon `C:\Users\Eros\graphiti` en SHA
> `747b480ec6b12ecb718f1e2bc588b20ee36ea9c2`; las rutas de VantaDB fijan el
> snapshot `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` y las de Studio el worktree
> `develop-desktop` @ `0ad38c542`.

### X-GRT-01 — Resolución temporal de intervalos de hecho con razón auditable

- **Problema:** un episodio recibido hoy describe una asignación anterior; el escritor no debe convertir una llegada tardía en autoridad para invalidar la asignación actual. El delta es separar fecha efectiva del hecho del tiempo de conocimiento — no «añadir timestamps»: `record_input` ya materializa `valid_at` desde `created_at`.
- **Contrato hipotético (tipos propuestos):** `struct FactClock { valid_from: Option<i64>, valid_until: Option<i64>, learned_at: i64, source_reference: Option<i64> }`; `enum IntervalAction { KeepBoth, CloseExisting, CloseIncoming, Undetermined }`; `fn compare_contradictory_facts(existing: &FactClock, incoming: &FactClock) -> Result<IntervalAction, TemporalError>`. Entrada: dos hechos previamente autorizados con contradicción ya establecida por el llamador; salida: decisión y razón auditable. Error por intervalo mal formado o fecha inválida, no por falta de LLM.
- **Fuente de evidencia:** `graphiti_core/edges.py:263-291` (G2, `EntityEdge`: `created_at`/`valid_at`/`invalid_at`/`expired_at`/`reference_time` — conocimiento, validez y ancla de extracción distintos; comentarios excluyen interpretar planes futuros como hechos ocurridos), `graphiti_core/utils/maintenance/edge_operations.py:715-1035` (G3, `_apply_temporal_invalidation`: compara fechas sin decidir verdad; el helper de contradicciones omite intervalos disjuntos — no es álgebra completa ni prueba de identidad temporal).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_dedup.rs` (decisión) y `vanta-memory/src/core/record/l1_writer.rs::record_input`, `write_memory`, lifecycle (aplicación); el SDK sigue dueño del storage [V1] [V2].
- **Acciones:**
  1. Definir semántica UTC/extremos y lectura compatible con campos ausentes.
  2. Incorporar fechas explícitas sin reinterpretar historia (no backfill desde `created_at` como fecha de mundo).
  3. Implementar la función de dominio pura con decisión y razón.
  4. Modo diagnóstico que compara decisiones antes de aplicar.
  5. Activar la aplicación mutante solo tras las garantías de publicación L1 (§5.1.1: `persist_planned:289-294` borra antes de `put_record` — secuencia estática a aclarar, no antes de modelar intervalos).
- **Dependencias:** identidad/provenance existentes; publicación segura para el modo mutante. La función pura y la UI no dependen de arreglar el checkpoint global. Cambios de semántica requieren versión de contrato.
- **FMEA:** fecha extraída errónea → invalida hecho correcto → detectar razón y fuente/revisión → exigir validación de entrada y resultado `Undetermined`; desconocido convertido en cero → orden falso → casos sin fecha; revisión que cambia entre decidir y aplicar → supersession equivocada → conflicto de revisión, replanificar sin reaplicar decisión antigua.
- **Criterios de aceptación propuestos:** llegada invertida converge semánticamente; fechas iguales no eliminan información sin regla explícita; intervalo disjunto intacto; límite exacto de fin/inicio documentado; error no muta. Observabilidad: conteos por acción/error y conflictos, sin nombres, contenido o fechas personales por defecto.
- **Criterio negativo:** desconocido nunca equivale a epoch; el fin no precede al inicio; no se extiende un fin histórico para acomodar una novedad; `learned_at` no decide verdad; el helper no crea fuente ni borra payload.
- **Independencia:** la comparación pura no requiere reindex (sí lo requeriría un filtro temporal); el rollback desactiva la aplicación conservando metadata y decisiones — no «restaura» hechos borrados sin fuente.
- **No objetivos:** temporalizar todos los atributos ni autorizar a un modelo a borrar; Neo4j como dependencia de una función pura queda rechazado. Abierto: representación de precisión/fecha aproximada y política ante contradicción sin fechas.

### X-GRT-02 — Elegibilidad de identidad en dedup: recurrencia no es duplicado

- **Problema:** la persona vuelve al mismo proyecto meses después: fact string idéntica no implica misma ocurrencia; un fast path que compara texto normalizado y endpoints sin gate temporal colapsa recurrencias disjuntas.
- **Contrato hipotético (tipos propuestos):** `enum OccurrenceMatch { SameOccurrence, DistinctOccurrence, NeedsReview }`; `fn dedup_eligibility(stored_id: &str, stored_revision: u64, old: &FactClock, new: &FactClock) -> Result<OccurrenceMatch, TemporalError>` — gate como precondición de `DedupDecision` antes de dedup exacto, semántico y por lote. El ID lógico de fuente no cambia por revisión; el ID de ocurrencia puede diferir aunque el texto coincida. El resultado enlaza revisión base, no autoriza write por sí mismo.
- **Fuente de evidencia:** `graphiti_core/utils/maintenance/edge_operations.py:715-1035` (G3; `_dedupe_extracted_edge:945-955` sigue comparando texto normalizado y endpoints sin gate temporal en el fast path — observación estática del clon) e issue pública [#1865](https://github.com/getzep/graphiti/issues/1865) (open; reproducción declarada en 0.30.2: mismo texto/endpoints reutiliza arista aunque intervalos disjuntos).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_dedup.rs` (precondición de `DedupDecision`); el escritor conserva su aplicación única `write_memory`/`apply_dedup_batch` [V1] [V2] — extender las decisiones actuales, no crear un deduplicador paralelo.
- **Acciones:**
  1. Documentar la identidad actual de dedup.
  2. Añadir el gate puro de elegibilidad.
  3. Registrar shadow outcomes.
  4. Activar sin reescribir IDs históricos.
  5. Reprocesar únicamente fuentes retenidas y autorizadas si se desea recuperar recurrencias colapsadas (un dato perdido no se reconstruye por reindex de vectores).
- **Dependencias:** X-GRT-01 para intervalos; publicación concurrente sigue necesitando revisión segura (§3.6); no depende de comunidades ni de Studio. Converge con C4 de `CONSOLIDATED.md`.
- **FMEA:** par con tiempo incompleto aceptado como duplicado → pérdida de ocurrencia → detectar decisiones sin evidencia temporal → `NeedsReview`; cada retry crea otra ocurrencia → inflación → identidad estable de source event; distintos tenants colisionan → cruce → scope antes del gate.
- **Criterios de aceptación propuestos:** ida/vuelta al mismo proyecto conserva dos episodios y sus intervalos; duplicado realmente compatible reutiliza identidad; batch no cambia esa regla; conflicto de revisión no se resuelve con overwrite. Métricas: match/distinct/unknown, duplicados por fuente, sin payload.
- **Criterio negativo:** un error de identidad/scope nunca cae a merge; no se crea intervalo usando la hora de llegada; el desconocimiento produce revisión explícita, no afirmación inventada; dos decisiones sobre la misma revisión no hacen last-write-wins.
- **Independencia:** rollback = volver al gate anterior solo para futuras decisiones, preservando ocurrencias ya creadas — no fusionarlas masivamente al desactivar. Alternativa: adapter que registre ocurrencias como memorias independientes mientras se acuerda el contrato; rechazar normalizar todo por hash de texto.
- **No objetivos:** corregir la extracción temporal upstream ni definir pesos de similitud; no exigir que el LLM distinga intervalos conocidos. Abierto: tratamiento de precisión parcial antes de spec.

### X-GRT-03 — Retirada de soporte por revisión con derivados marcados stale

- **Problema:** eliminar una conversación no debe borrar un hecho sustentado por otra fuente, pero tampoco dejar el texto retirado en un resumen «vigente». Huérfanos, derivados stale y borrado físico son límites diferentes; «invalidado» no es «olvidado».
- **Contrato hipotético (tipos propuestos):** `struct SupportRemoval { source_id: String, expected_revision: u64 }`; `enum DerivedDisposition { Retain, MarkStale, RemoveUnsupported }`; `fn plan_source_withdrawal(request: SupportRemoval, supports: &[SourceSupport]) -> Result<WithdrawalPlan, WithdrawalError>`. Entrada desde la frontera autorizada de borrado; salida enumera soportes restantes, derivados afectados y revisión esperada. Errores: `NotAuthorized`, `RevisionConflict`, `UnresolvedReference`, `StorageFailure`.
- **Fuente de evidencia:** `graphiti_core/graphiti.py::remove_episode:2174-2251` (G11: elimina el UUID de `edge.episodes`; si quedan fuentes guarda la arista y sin ellas la agenda para borrar; elimina entidades mencionadas solo por ese episodio; reconstruye resúmenes con episodios restantes ordenados por tiempo y restaura `old_summary` en memoria si falla — no demuestra retiro del resumen persistido ni transacción única de toda la eliminación).
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs` (`source_message_ids`/metadata del escritor [V1]), lifecycle existente, scheduler/jobs y auditoría `src/cli_handlers/index.rs` para ampliar comprobación. El transporte MCP/HTTP no decide semántica de soporte.
- **Acciones:**
  1. Inventariar enlaces reales fuente→hecho→derivado.
  2. Construir el plan de retirada read-only.
  3. Añadir estados de derivado/versiones.
  4. Publicar la retirada con garantías de reemplazo (§5.1.1: seguridad de reemplazo L1 antes de retirar físicamente).
  5. Jobs reanudables recomponen desde las fuentes restantes.
  6. Auditar huérfanos; reindex solo de derivados afectados, no de todos los originales.
- **Dependencias:** seguridad de reemplazo L1 antes de retirar físicamente (informe §5.1.1); scheduler/jobs existentes; X-GRT-01 para revisión; equivalente transversal a derivados versionados, no sistema nuevo.
- **FMEA:** soporte añadido durante el plan → borrado injustificado → CAS/revalidar; rebuild fallido → resumen antiguo filtra el dato → estado stale excluido; huérfano fuera de enumeración → remanente invisible → auditoría desde fuentes y derivados, no solo desde relaciones disponibles.
- **Criterios de aceptación propuestos:** quitar una de dos fuentes preserva el soporte correcto; quitar la última retira del recall/preview; fallo intermedio reanudable; retry idempotente; otro tenant intacto. Métricas: pendientes/stale/huérfanos/conflictos por etapa, IDs opacos o agregados; nunca resúmenes sensibles en errores por defecto.
- **Criterio negativo:** un soporte vivo impide el borrado por «sin soporte»; una fuente retirada nunca genera nuevos derivados; `stale` no se anuncia `ready`; un fallo de regeneración no resucita contenido retirado; no se afirma borrado físico de backups o clientes con recibo lógico.
- **Independencia:** soportes históricos desconocidos quedan `Unknown`, no «cero»; el rollback deshabilita la regeneración nueva pero mantiene tombstone y retirada autorizada. Retirar primero de la elegibilidad de lectura; publicar derivado solo si fuentes/revisión siguen vigentes; cleanup parcial sigue pendiente (§3.6).
- **No objetivos:** GDPR completo, purga criptográfica ni borrado sin consentimiento — retirada lógica trazable de una fuente.

### X-GRT-04 — Trazabilidad por brazo y receta solicitada vs efectiva en recall

- **Problema:** distinguir «no hay hechos» de «falló embedding» y saber si el resultado procede de episodio, entidad o resumen; un brazo omitido no debe aparecer exitoso ni un fallo transformarse en lista vacía exitosa.
- **Contrato hipotético (tipos propuestos):** `enum EvidenceKind { RawMessage, Fact, Entity, DerivedContext }`; `enum ArmState { Complete, Skipped, Failed }`; `struct RecallTrace { kind: EvidenceKind, state: ArmState, requested_recipe: String, effective_recipe: String }`; `fn attach_recall_trace(result: ExistingRecallResult) -> TracedRecall`. Score con tipo/origen, no probabilidad; ID/revisión del registro canónico; scope preservado al expandir/hidratar.
- **Fuente de evidencia:** `graphiti_core/search/search.py:70-255` (G4: query vacía termina sin embedding; vector suministrado evita repetir la llamada; brazos edge/node/episode/community con `semaphore_gather`, cada uno con su config y filtros aplicables; `_trace_phase` marca error y relanza — no fabrica cero resultados como éxito), `graphiti_core/search/search_config.py:32-139` (G5: métodos/rerankers/resultados tipados por superficie — RRF, distancia y cross-encoder no producen probabilidades comparables).
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs::RecallConfig`, `search_records_core` y `vanta-memory/src/context_engine/engine.rs::assemble_with_recall` [V3] [V4] — el core híbrido existe y `core_search` es false por defecto: no afirmar BM25 efectivo por el simple hecho de que exista el flag.
- **Acciones:**
  1. Mapear recetas y errores actuales.
  2. Añadir campos opcionales backwards-compatible en el recall existente.
  3. Exponer receta solicitada/efectiva y estado por brazo.
  4. UI read-only.
  5. Decidir política de partial-success sin cambiar el algoritmo.
- **Dependencias:** contrato de scopes/errores existente; independiente de X-GRT-01 y de la escritura mutante; presupuesto único del assembler. Converge con C7/C8 de `CONSOLIDATED.md`.
- **FMEA:** config no propagada → receta distinta → comparar requested/effective; fallback que elimina filtro → fuga → error tipado antes de ejecutar; expansión que revive borrado → verificar revisión/estado en hidratación; doble evidencia que agota budget → dedup por identidad en el ensamblador.
- **Criterios de aceptación propuestos:** recipe y centro solicitados llegan al seam; empty query se distingue de fallo; brazo omitido no aparece exitoso; scopes sobreviven a fallback y expansión; mismo presupuesto. Observabilidad por fase, cardinalidad y códigos de error; no queries ni payloads por defecto.
- **Criterio negativo:** un fallo de brazo nunca se transforma en lista vacía exitosa; las trazas no prometen snapshot del corpus; una respuesta tardía de otra revisión no se añade a un contexto ya entregado (§3.6).
- **Independencia:** sin migración del corpus; el rollback omite los campos nuevos — no borra registros ni recompone el ranking. Alternativa primera: solo instrumentación de las rutas actuales.
- **No objetivos:** portar el catálogo de rerankers ni inventar un score normalizado universal; nuevas superficies solo con necesidad aprobada. Abierto: si partial-success es aceptable por el consumidor.

### X-GRT-05 — Inspector read-only de intervalos, soporte y razón de supersession

- **Problema:** un usuario ve dos versiones y no sabe si hubo cambio de mundo, corrección, llegada tardía o fallo del derivado; `Unknown` no se dibuja como fecha cero ni un score como probabilidad.
- **Contrato hipotético (tipos propuestos):** `struct FactInspection { id: String, revision: u64, clock: FactClock, support_ids: Vec<String>, derived_state: String }`; `fn inspect_fact(scope: AuthorizedScope, id: &str) -> Result<FactInspection, InspectError>` — contrato de vista del backend, no estado autoritativo del frontend; `NotFound` no revela existencia de activos ajenos.
- **Fuente de evidencia:** `graphiti_core/edges.py:263-291` (G2, campos temporales y soporte plural del hecho — upstream aporta campos, no UI operativa acreditada); seams Studio releídos con `git show` en `0ad38c542` (informe §8.2): `feature-freez/desktop/src/App.tsx:17-64` (`WorkspaceShell`; no existe `app/routes.ts` en ese árbol — se retira la atribución del codec `?s/?ns/?inspect`), `components/memory/MemoryLens.tsx:381-409,488-550` (genlog/anchor/sesión), `components/activity/ActivityPanel.tsx:52-89` (audit/errores), `components/lens/retrieval/RetrievalLens.tsx:138-164,193-228` (search/contexto/audit). Son seams de componentes existentes, no una navegación temporal ya implementada.
- **Seam VantaDB:** Studio existente en `develop-desktop`: `MemoryLens.tsx`, `ActivityPanel.tsx`, `RetrievalLens.tsx` — proyección autorizada sobre datos del escritor/lifecycle; añadir explicación, no otro escritorio. Alternativa del informe: ampliar el detalle actual antes de crear una vista nueva.
- **Acciones:**
  1. Proyección desde campos existentes y etiquetado de faltantes.
  2. Mostrar separado «cuándo fue cierto», «cuándo se conoció» y «estado de proyección»; `Unknown` visible en vez de timestamp sintetizado.
  3. Expandir soporte autorizado al original retenido, o explicar retirada, retención vencida o referencia sin resolver — sin filtrar otros tenants.
  4. Agregar intervalos cuando X-GRT-01 esté aprobado; desde `retrieval`, mostrar receta solicitada/efectiva y clase de evidencia.
  5. Vista read-only primero; acciones mutantes (editar/retirar) requieren revisión esperada y confirmación humana — fuera del mínimo.
- **Dependencias:** X-GRT-04 para explicar retrieval; X-GRT-03 para estado stale; la vista básica no queda bloqueada por la migración temporal completa. El backend decide el acceso; Studio posee únicamente estado de navegación/cache (§3.6).
- **FMEA:** cliente cachea fuente revocada → exposición → invalidar preview y consultar autorización al abrir; UI infiere fecha → mentira visual → `Unknown`; motivo con contenido sensible → fuga en notificación → códigos de motivo y redacción; score dibujado como probabilidad → etiqueta de score.
- **Criterios de aceptación propuestos:** distinguir válido/conocido/retirado, fuente faltante, revisión conflictiva y error de carga; cambio de conexión/namespace elimina el estado anterior; `failed`/`stale` no se representan como «sin memoria». Observabilidad de carga/errores sin payload ni IDs personales.
- **Criterio negativo:** una inspección nunca se convierte en permiso de escritura; una respuesta tardía tras cambio de conexión/scope se descarta — no pisa la selección nueva.
- **Independencia:** no requiere migración de datos para empezar; el rollback oculta el panel sin cambiar hechos; al cambiar conexión/scope se cancela la carga e ignora la respuesta antigua.
- **No objetivos:** renderizar todo el grafo, importar Neo4j Browser ni acciones destructivas por LLM. Abierto: diseño humano de la navegación temporal.

[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L112-L620
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_dedup.rs#L112-L250
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L133-L335
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L150-L264
