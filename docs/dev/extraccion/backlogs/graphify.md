---
title: "Backlog de extracción — Graphify"
kind: research
status: active
description: "Candidatas del informe Graphify: importación con evidencia por arista, contrato de productor, consultas con cobertura declarada, reconciliación por capa, publicación de derivados, relaciones paralelas e inspector"
tags: [extraccion, backlog, graphify]
---

# Backlog de extracción — Graphify

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/graphify.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.
> El scope canónico del informe es `FIND-299`/repo-map: conector opcional de evidencia, no un nuevo core de código ni parser en el motor.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-GRF-01 | 🟠 | Importación de evidencia code con evidencias por arista | `src/sdk/api/graph.rs:113-151` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: un export versionado de Graphify se admite como productor opcional; cada relación puede conservar evidencias rivales (origen, clase, score, localización) como registros asociados separados. Por qué: colapsar duplicados a un peso oculta evidencia contradictoria y `add_edge` no admite el objeto de evidencia completo; el score reportado no se promociona a confianza Vanta. Acción mínima: validar endpoints/dirección/fuentes del JSON permitido, mapear a IDs Vanta y publicar lote idempotente mediante el contrato C12. Trazabilidad: informe §Ficha A · evidencia `graphify/build.py:253-312` | X-GRF-02, X-GRF-06, C12 | Schema del conector, identidad de repo y soporte de evidencia asociada en `MemoryInput`/grafo |
| X-GRF-02 | 🟠 | Contrato de productor con identidad e invalidación por generación | `vantadb-mcp/src/code_index.rs:193-235` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: cada importación viaja como generación C12 con manifiesto de completitud, digest del worktree y bajas propiedad del productor como control de aplicación. Por qué: normalizar IDs no garantiza persistencia al mover/renombrar; mismo mtime/tamaño no asegura mismos bytes y un schema desconocido no debe escribir parcialmente. Acción mínima: conservar ID externo original y clave interna separada, exigir manifest completo o delta con tombstones e invalidar por cambio de fuente, gramática, modelo o prompt. Trazabilidad: informe §Ficha B · evidencia `graphify/ids.py:51-140`, `graphify/cache.py:41-86,235-284`, `graphify/watch.py:33-70` | X-GRF-01, X-GRF-04, X-GRF-05, C12, C14 | Hash/manifiesto por archivo existente, namespace por repo/productor; no iniciar un watcher desde este conector |
| X-GRF-03 | 🟡 | Consultas de código que declaran lo que no cubren | `vantadb-mcp/src/code.rs:272-301` | 🟢 1d | 🟡 P2 | ⬜ Candidata | Qué: `code_search`/`code_impact` devuelven un slice con seeds, dirección, omisiones, truncamiento y resolution_state; una consulta no resuelta no se presenta como «sin dependencias». Por qué: un BFS recortado parece cierre exhaustivo, una negativa por nombre ambiguo no es contexto útil y una relación semantic débil puede cambiar el blast radius. Acción mínima: resolver ID/nombre cualificado, conservar alternativas ambiguas, deduplicar por evidencia y entregar a context assembly con avisos de límite; no autorizar exclusión de tests desde `affected`. Trazabilidad: informe §Ficha C · evidencia `graphify/serve.py:317-354,1489-1565`, `graphify/affected.py:302-408` | X-GRF-01, X-GRF-07, C12, C7 | Retrieval híbrido y presupuesto existentes; generación fija de lectura vía X-GRF-02 |
| X-GRF-04 | 🟠 | Reconciliación con ownership por capa de evidencia | `vantadb-mcp/src/code_index.rs:152-327` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: el reemplazo por fuente se acota a la capa/origen del productor (AST vs explicación semántica o manual), con plan de reconcile condicionado a generación y autoridad. Por qué: un refresh de la capa AST no debe borrar explicación manual ni evidencia semántica que no puede regenerar; archivo excluido no equivale a borrado y un conflicto no resoluble falla, no sobrescribe. Acción mínima: extender el coordinador `index_repo` existente con separación de owners, validación de bajas explícitas y preview antes de publicar generación. Trazabilidad: informe §Ficha D · evidencia `graphify/watch.py:1324-1451` (§11.2 del informe) | X-GRF-01, X-GRF-02, X-GRF-06, C12 | Namespaces y publicación controlada de generación, coverage declarada por el export; legacy sin dueño queda diagnóstico |
| X-GRF-05 | 🟡 | Publicación de derivados y ACK de inputs comprobable | `vanta-memory/src/utils/task_checkpoint.rs` | 🟡 2-5d | 🔵 P3 | ⬜ Candidata | Qué: la generación primaria visible y el estado de cada derivado se publican con recibo que separa publicación, derivados listos y cleanup, sin prometer fsync por proyección. Por qué: varias escrituras sucesivas no son una transacción global; `True` del rebuild no certifica igualdad de cada derivado y un ACK emitido sin inputs completos falsea la vigencia. Acción mínima: preparar → validar → publicar condicionalmente a generación esperada → marcar proyecciones pending → actualizar cada una → ACK de inputs, con checkpoints existentes. Trazabilidad: informe §Ficha E · evidencia `graphify/watch.py:3247-3398,1970-2080`, `graphify/paths.py:84-205` | X-GRF-04, X-GRF-02, C9, C14 | Contrato de commit real por recurso y writer autorizado por repo/productor; condicional a pipeline aprobado |
| X-GRF-06 | 🟡 | Relaciones paralelas sin colapso por pareja de nodos | `src/sdk/api/graph.rs:113-151` | 🟢 1d | 🟡 P2 | ⬜ Candidata | Qué: `imports` y `calls` entre los mismos nodos y relaciones inversas sobreviven como tipos/direcciones distintos con sus evidencias asociadas al importar y reconciliar. Por qué: simplificar a un grafo por pareja pierde la relación inversa o el tipo; validar dirección y tipo antes de mapear endpoints evita adivinar orden en exports ambiguos. Acción mínima: port `map_relations` sobre el contrato C12 que asocie evidencias por relación y publique por generación; rechazar export ambiguo en lugar de inferir. Trazabilidad: informe §Ficha F · evidencia `graphify/export.py:415-474` | X-GRF-01, X-GRF-04, C12 | Labels/tipos de relación existentes en el grafo Vanta; depende del adapter X-GRF-01 y de X-GRF-04 |
| X-GRF-07 | 🟢 | Inspector read-only de evidencia code en Studio | `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx` | 🟡 2-5d | 🔵 P3 | ⬜ Candidata | Qué: una vista read-only muestra fuente/origen/revisión/límites de la evidencia seleccionada y permite navegar vecinos con presupuesto existente, distinguiendo stale, no-match, rechazo ambiguo y error. Por qué: la topología dibujada no es certeza y el ratio de reducción no es utilidad ni confianza; una selección vieja no debe pisar la nueva. Acción mínima: proponer `inspect_code_evidence` sin escritura con autorización y resolución C12; links versionados como alternativa mínima al panel. Trazabilidad: informe §Ficha G · evidencia `graphify/exporters/html.py:256-297` | X-GRF-01, X-GRF-03, C16 | Studio existente en `develop-desktop`, trabajo frontend separado; condicional a pipeline y UI aprobados |

## Fichas completas

> Las fichas expanden la tabla resumen con el contenido del informe
> `docs/dev/extraccion/agent-memory-2026-11-07/graphify.md` (Fichas A–C de §2 y
> D–G de §12). Todos los contratos son **hipotéticos** — propuestas de
> comportamiento para VantaDB, no APIs existentes ni puertos de código upstream.
> Las referencias `archivo:línea` son del clon `C:\Users\Eros\graphify` en SHA
> `16218bfc80d4ca2799734d045039f7ed1a5fa9fb`; las rutas de VantaDB fijan el
> snapshot `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`. Scope canónico:
> `FIND-299`/repo-map — conector opcional de evidencia, no un nuevo core de
> código ni parser en el motor.

### X-GRF-01 — Importación de evidencia code con evidencias por arista

- **Problema:** colapsar duplicados de una misma relación a un peso oculta evidencia contradictoria; `add_edge` no admite el objeto de evidencia completo y el score reportado no se promociona a confianza VantaDB.
- **Contrato hipotético (tipos propuestos):** único `CodeEvidenceImport` C12 (CONSOLIDATED) como DTO persistente de importación: `also_seen` produce evidencias asociadas separadas de una misma relación; origen, dirección y referencias se preservan como registros asociados (memoria canónica asociada a la relación), no como peso. El score reportado no se convierte automáticamente a confianza derivada de VantaDB.
- **Fuente de evidencia:** `graphify/build.py:253-312` (G1, `edge_evidence`/`_fold_also_seen`: preservan confianza, score y localización al colapsar duplicados de igual relación); `graphify/build.py:1524` (`build_from_json(... directed=False)` por defecto — el modelo de almacenamiento puede ser no dirigido; no afirmar que todo export pierde dirección).
- **Seam VantaDB:** `src/sdk/api/graph.rs:113-151` (`Embedded::add_edge`) y `src/sdk/types/record.rs:184-224` (`MemoryInput`).
- **Acciones:**
  1. Fijar SHA y root del export.
  2. Leer el JSON permitido.
  3. Validar endpoints, dirección y fuentes antes de publicar.
  4. Importar AST y semantic como evidencias separadas.
  5. Mapear a IDs VantaDB (key por productor/repo/revisión/evidencia).
  6. Publicar lote idempotente mediante el contrato C12.
- **Dependencias:** schema del conector, identidad de repo y soporte de evidencia asociada en `MemoryInput`/grafo; X-GRF-02, X-GRF-06, C12.
- **FMEA:** merge destructivo oculta evidencia contradictoria; grafo no dirigido invierte callers/callees; stub externo se interpreta como código disponible. Mantener `also_seen`, dirección y clase external/unresolved.
- **Criterios de aceptación propuestos:** reimportación no duplica evidencias; dos fuentes de una relación sobreviven; aristas inversas no se fusionan; un stub no ofrece snippet inventado; reimport converge.
- **Criterio negativo:** la cancelación previa a publicar deja la generación anterior; la posterior no implica undo. El rollback cambia la generación visible, nunca memoria humana.
- **Independencia:** adapter opcional — alternativas del informe: extender el scanner propio o resolver completo; el adapter preserva evidencia rival que `add_edge` no admite como metadata.
- **No objetivos:** replicar build/cluster, deduplicación semántica ni exportadores upstream.

### X-GRF-02 — Contrato de productor con identidad e invalidación por generación

- **Problema:** normalizar IDs no garantiza persistencia al mover/renombrar archivos; mismo mtime/tamaño no asegura mismos bytes; un schema desconocido no debe escribir parcialmente.
- **Contrato hipotético (tipos propuestos):** generación de C12 con manifiesto de completitud, digest del worktree y bajas propiedad del productor como control de aplicación. No se crea un segundo `CodeGraphSnapshot` persistente; una generación incompatible se rechaza o migra explícitamente, nunca se «arregla» silenciosamente.
- **Fuente de evidencia:** `graphify/ids.py:51-140` (G2, `normalize_id`/`DeclIdRegistry`: normalización Unicode idempotente, salts y colisiones — evita divergencia productor/consumidor, no persistencia de identidad), `graphify/cache.py:41-86,235-284` (G3, `_AST_CACHE_SCHEMA`, fingerprints de gramática/prompt y guards de mtime; la cabecera arquitectónica menciona versión anterior pero el código fija 39 — tomar la versión real del productor), `graphify/watch.py:33-70` (G4, colas `_queue_pending`/`_drain_pending`).
- **Seam VantaDB:** `vantadb-mcp/src/code_index.rs:193-235,476-486` (hash/manifiesto por archivo existente); `FIND-299` contempla watcher — no iniciar otro watcher desde este conector. Namespace por repo/productor.
- **Acciones:**
  1. Conservar ID externo original y clave interna separada.
  2. Exigir manifest completo o delta con tombstones.
  3. Invalidar por cambio de fuente, gramática, modelo o prompt.
  4. Validar entrantes a archivos cambiados antes de publish.
- **Dependencias:** hash/manifiesto por archivo existente, namespace por repo/productor, X-GRF-01/04/05, C12/C14. No iniciar un watcher desde este conector.
- **FMEA:** mismo mtime/tamaño no asegura mismos bytes; normalización casefold confunde símbolos distintos; pérdida de evento deja snapshot obsoleto; mover archivo parece alta+baja. Registrar alias/move solo con evidencia, nunca unir entidades por similitud de nombre.
- **Criterios de aceptación propuestos:** schema desconocido no escribe parcialmente; actualización de gramática no reutiliza evidencia antigua como fresh; delete elimina relaciones propiedad del productor y no borra memoria manual; un move conserva trazabilidad.
- **Criterio negativo:** un error de lectura del baseline no es una baja ni un grafo vacío exitoso; un job viejo no pisa uno nuevo.
- **Independencia:** alternativa conservadora: full snapshot frente a delta con tombstones; el delta real es coverage/origen/versionado — el hash de VantaDB ya existe. Migración legacy sin dueño queda diagnóstico.
- **No objetivos:** trasladar todos los caches, workers o hooks a VantaDB.

### X-GRF-03 — Consultas de código que declaran lo que no cubren

- **Problema:** un BFS recortado parece cierre exhaustivo; una negativa por nombre ambiguo no es contexto útil; una relación semantic débil puede cambiar el blast radius; `No matching nodes` no demuestra que el cuerpo de código no contenga la respuesta.
- **Contrato hipotético (tipos propuestos):** `CodeContextSlice { query, normalized_terms, revision, seeds, evidence_keys, omitted, truncated, traversal_direction, max_hops, resolution_state }`. Una consulta no resuelta no se presenta como «sin dependencias»; refusal ambiguo, no-match sentinel, error y subgrafo útil permanecen distintos en el recibo.
- **Fuente de evidencia:** `graphify/serve.py:317-354` (G5, `_query_terms`/`_bfs`/`_subgraph_to_text`), `graphify/serve.py:1489-1565` (`_query_graph_text`: acepta seeds externos; referencia ambigua se rehúsa y unresolved se informa; `:1532-1538` devuelve negativa textual ante nombre exacto ambiguo), `graphify/affected.py:302-408` (G6, `resolve_seed_outcome`/`affected_nodes`: recorrido acotado y referencia).
- **Seam VantaDB:** `vantadb-mcp/src/code.rs:12-20,272-301` (`code_search`/`code_impact`); VantaDB ya ensambla contexto con presupuesto — entregar al context assembly canónico, no crear presupuesto paralelo.
- **Acciones:**
  1. Resolver ID/nombre cualificado.
  2. Conservar alternativas ambiguas.
  3. Recuperar vecindario sobre la generación fija de lectura.
  4. Deduplicar por evidencia.
  5. Presupuestar la salida completa con avisos de límite.
  6. Entregar al context assembly canónico.
- **Dependencias:** retrieval híbrido y presupuesto existentes; generación fija de lectura vía X-GRF-02; X-GRF-01, X-GRF-07, C12/C7.
- **FMEA:** BFS recortado parece cierre exhaustivo; stemming/casefold pierde identidad; una relación semantic débil cambia blast radius; GF-1: una negativa por nombre ambiguo también es salida textual medible, no contexto útil — conservar `resolution_state` y no promover longitud baja a éxito de recuperación. Diferenciar `related` de `calls` y corte por presupuesto de ausencia.
- **Criterios de aceptación propuestos:** límites y omisiones se ven en texto y DTO; homónimos no se resuelven por orden del grafo; dirección inversa cambia el resultado explicado; refusal ambiguo, no-match sentinel, error y subgrafo útil permanecen distintos en el recibo/inspector. No cambiar el benchmark para forzar esta aceptación.
- **Criterio negativo:** cancelación o presupuesto produce `incomplete`, nunca una ausencia simulada; la consulta no publica cambios.
- **Independencia:** lectura sobre generación fija, sin commit; paginar solo dentro de la misma generación; desactivar el adapter mantiene las consultas VantaDB existentes sin reducir gates.
- **No objetivos:** autorizar exclusión de tests desde `affected` — no se acreditó un selector seguro con cierre transitivo y cobertura runtime; no expandir a otro retriever.

### X-GRF-04 — Reconciliación con ownership por capa de evidencia

- **Problema:** un refresh de la capa AST no debe borrar explicación manual ni evidencia semántica que no puede regenerar; archivo excluido no equivale a borrado; un conflicto no resoluble falla, no sobrescribe.
- **Contrato hipotético (tipos propuestos):** `plan_reconcile(previous, incoming: &CodeEvidenceImport, coverage) -> Result<ReconcilePlan, ImportError>` — los inputs de control no son otro schema de importación. Errores `UnknownCoverage`/`Conflict`/`BadEndpoint` no habilitan borrado.
- **Fuente de evidencia:** `graphify/watch.py:1324-1451` (§11.2: conserva archivos vivos excluidos sin regla explícita; AST reemplaza su capa de fuentes reconstruidas, no toda explicación semántica que no puede regenerar; colisión AST/semántica con evidencia independiente conserva el ID semántico y remapea los endpoints AST nuevos; conflicto no resoluble falla), `graphify/watch.py:1087-1138` (`_reconcile_existing_graph`: propaga fallo de lectura del baseline en vez de tomarlo por vacío — missing y unreadable son estados distintos).
- **Seam VantaDB:** `vantadb-mcp/src/code_index.rs:152-327` (`index_repo` ya reconcilia hash/símbolos) y `MemoryInput` (ya expresa procedencia); falta acreditar ownership por productor/capa importada — extender el coordinador existente, no llevar código al ingestor documental.
- **Acciones:**
  1. Capturar el original.
  2. Separar owner AST/explicación por capa/origen del productor.
  3. Validar bajas explícitas y fuentes fallidas.
  4. Preview del plan.
  5. Publicar generación.
  6. Cleanup propio de la capa.
- **Dependencias:** X-GRF-01/02/06, C12; namespaces y publicación controlada de generación; coverage declarada por el export. Conservar el manifest anterior; ninguna migración deduce ownership de nombre/peso. Full reimport sin bajas es alternativa mínima si el export no declara coverage; legacy sin dueño queda diagnóstico.
- **FMEA:** ID igual que reemplaza evidencia humana → detectar origen distinto y rechazar/alias explícito; archivo excluido que parece borrado → coverage; delete que deja dangling → verificar endpoints; cancelación → no publicar plan incompleto.
- **Criterios de aceptación propuestos (manuales):** AST+semántica sobreviven refresh; falla de lectura no borra; baja propia no toca otra fuente; métricas por altas/bajas/rechazos.
- **Criterio negativo:** `UnknownCoverage`/`Conflict`/`BadEndpoint` nunca habilitan borrado; una cancelación no publica un plan incompleto.
- **Independencia:** extiende `index_repo` existente; rollback conserva el manifest anterior y la generación retenida.
- **No objetivos:** merge bidireccional, recuperar evidencia nunca exportada ni asumir que la explicación conservada sigue semánticamente fresh.

### X-GRF-05 — Publicación de derivados y ACK de inputs comprobable

- **Problema:** varias escrituras sucesivas no son una transacción global; `True` del rebuild no certifica igualdad de cada derivado y un ACK emitido sin inputs completos falsea la vigencia.
- **Contrato hipotético (tipos propuestos):** `publish_code_projection(expected_generation, prepared_refs) -> Result<ProjectionReceipt, PublishError>`; el receipt referencia C12 y separa publicación, estado de derivados y cleanup — no promete fsync por proyección.
- **Fuente de evidencia:** `graphify/watch.py:3247-3398` (construye temporal, compara baseline, guard de shrink; marca HTML stale antes de avanzar JSON; después reporte/labels/firmas, manifiesto, HTML opt-in y stamp — varias escrituras, no transacción global; fallos de manifest capturados y callflow puede quedar sin actualizar), `graphify/watch.py:1970-2080` (`_current_build_state`/`_write_build_state`: incorpora root, conjunto escaneado, exclusiones, modo y epoch — no solo mtime; no emite stamp si falta JSON completo o HTML está stale; el check de extremos `{`/`}` es deliberadamente barato), `graphify/paths.py:84-205` (`_atomic_replace`: rename con fallback copia/renombra y restauración best-effort; documenta ausencia de fsync — rename visible no acredita power-loss durability).
- **Seam VantaDB:** `vanta-memory/src/utils/task_checkpoint.rs` — jobs/checkpoints y hashes VantaDB ya existen; distinguir generación primaria visible de derivados listos no exige nuevo scheduler.
- **Acciones:**
  1. Preparar.
  2. Validar.
  3. Publicar condicionalmente a la generación esperada.
  4. Marcar proyecciones pending.
  5. Actualizar cada una.
  6. ACK de inputs con checkpoints existentes.
- **Dependencias:** contrato de commit real por recurso y writer autorizado por repo/productor; X-GRF-04 y X-GRF-02, C9/C14. Condicional a pipeline aprobado.
- **FMEA:** fallo entre JSON y UI → detectar generación distinta y mostrar stale; retry que duplica; cleanup fallido que queda invisible → sigue visible. Medir estados, no contenido. No copiar el fallback rename como garantía transaccional.
- **Criterios de aceptación propuestos:** la generación distinta se detecta y se muestra stale; retry no duplica; cleanup fallido sigue visible. Negativo local del informe (hipótesis derivada del orden de `index_repo`, no pérdida reproducida): hash de archivo escrito antes de todos sus símbolos — aceptación futura: fallo entre file put y symbol put debe impedir ACK completo.
- **Criterio negativo:** cancelar antes no adelanta el ACK; cancelar después se reconcilia el recibo — nunca se anuncia vigencia sin inputs completos.
- **Independencia:** rollback vuelve a la generación retenida y marca derivados incompatibles; alternativa mínima: no importar HTML, generar vista read-only desde VantaDB.
- **No objetivos:** atomicidad de filesystem universal, exactly-once ni portar el HTML.

### X-GRF-06 — Relaciones paralelas sin colapso por pareja de nodos

- **Problema:** `imports` y `calls` entre los mismos nodos y relaciones inversas deben sobrevivir como tipos/direcciones distintos; simplificar a un grafo por pareja pierde la relación inversa o el tipo.
- **Contrato hipotético (tipos propuestos):** `map_relations(&CodeEvidenceImport) -> Result<RelationPlan, _>` — validar dirección/tipo → mapear endpoints → asociar evidencias → publicar por generación.
- **Fuente de evidencia:** `graphify/export.py:415-474` (`to_json` preserva `original_links`, multigraph y `_src`/`_tgt` — la dirección sobrevive al intercambio).
- **Seam VantaDB:** `src/sdk/api/graph.rs:113-151` — VantaDB ya tiene labels/tipos de relación; no necesita otro multigraph, el adapter no colapsa por pareja.
- **Acciones:**
  1. Validar dirección y tipo antes de mapear endpoints.
  2. Mapear endpoints a IDs VantaDB.
  3. Asociar evidencias por relación.
  4. Publicar por generación (vía X-GRF-02).
  5. Rechazar export ambiguo en lugar de adivinar orden.
- **Dependencias:** labels/tipos de relación existentes en el grafo VantaDB; depende del adapter X-GRF-01 y de X-GRF-04, C12.
- **FMEA:** simplificación de NetworkX pierde relación inversa → comparar lista original y endpoints; cancelación descarta el plan; rollback retira solo la generación propia.
- **Criterios de aceptación propuestos (manuales):** ida/vuelta y dos tipos distintos sobreviven sin duplicados tras retry.
- **Criterio negativo:** un export ambiguo se rechaza — nunca se infiere orden ni dirección; no calibrar confidence desde defaults numéricos.
- **Independencia:** la cancelación descarta el plan y el rollback retira solo la generación propia; no requiere otro multigraph.
- **No objetivos:** — *el informe no declara un campo de no-objetivos separado para F; se conservan los de A/C (no replicar build/exportadores, no portar NetworkX).*

### X-GRF-07 — Inspector read-only de evidencia code en Studio

- **Problema:** la topología dibujada no es certeza y el ratio de reducción no es utilidad ni confianza; una selección vieja no debe pisar la nueva.
- **Contrato hipotético (tipos propuestos):** `inspect_code_evidence(auth, generation, selection) -> Result<CodeView, _>` — puerto sin escritura: autorizar → resolver C12 → mostrar fuente/origen/revisión/límites → navegar vecinos con presupuesto existente.
- **Fuente de evidencia:** `graphify/exporters/html.py:256-297` (`showInfo`/`focusNode`: selecciona nodo → obtiene vecinos → muestra tipo/comunidad/fuente/grado → foco en vecino; escapa campos y usa listener delegado — controles localizados, no auditoría XSS); el marcador `.graph.html.stale` prueba que vigencia de vista y JSON son cosas distintas.
- **Seam VantaDB:** `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx` (`:138-164,193-228`, releído con `git show` en `0ad38c542`: búsqueda explain y get/vecinos/audit existentes) — Studio ya tiene diagnóstico retrieval; no crear otra aplicación.
- **Acciones:**
  1. Autorizar al usuario sobre el repo.
  2. Resolver la evidencia vía C12.
  3. Mostrar fuente/origen/revisión/límites.
  4. Navegar vecinos con el presupuesto existente.
  5. Descartar respuestas de una selección de generación anterior; links versionados como alternativa mínima al panel.
- **Dependencias:** X-GRF-01/X-GRF-03 y trabajo frontend separado; no depende de X-GRF-05 si la vista consulta datos canónicos; Studio existente en `develop-desktop`; condicional a pipeline y UI aprobados.
- **FMEA:** selección vieja pisa nueva → descartar respuesta anterior; texto hostil → render seguro; ruta fuera de root → denegar.
- **Criterios de aceptación propuestos (manuales):** stale, no-match, rechazo ambiguo y error distintos; ratio no se presenta como utilidad ni confianza; cancelación de UI no cancela el job compartido.
- **Criterio negativo:** el inspector nunca escribe; la cancelación de la UI no cancela el job compartido.
- **Independencia:** desactivar el panel no borra datos; la alternativa mínima (links versionados) no requiere el panel completo.
- **No objetivos:** portar vis.js ni prometer accesibilidad validada; no inspector de refusals ya implementado se declara en Studio.
