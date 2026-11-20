---
title: "Backlog de extracción — codebase-memory-mcp"
kind: research
status: active
description: "Candidatas del informe codebase-memory-mcp: evidencia de resolución importable, respuesta por snapshot, impacto asesor, reconciliación por ownership, baseline tras publicación y overlay de procedencia en Studio"
tags: [extraccion, backlog, codebase-memory-mcp]
---

# Backlog de extracción — codebase-memory-mcp

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/codebase-memory-mcp.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-CBM-01 | 🔴 | Evidencia de resolución importable como memoria canónica | `src/sdk/api/graph.rs`, `src/sdk/types/record.rs` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: cada arista importada conserva estrategia, candidatos y confianza reportada como evidencia asociada a memoria canónica, con estados resuelto/ambiguo/externo/pendiente y referencia repo/revisión/rango. Por qué: los IDs SQLite del productor no son identidad portable entre rebuilds; perder `strategy` convierte inferencia en hecho y un match débil suprimido no debe fabricar una call edge. Acción mínima: usar exclusivamente `CodeEvidenceImport` (C12) — mapear qname/ruta/rango a `refs`, conservar referencia original y revisión, persistir evidencia antes de publicar relaciones, registrar lote/generación y rechazos. Trazabilidad: informe §Ficha A · evidencia `src/pipeline/pass_calls.c:837-874`, `src/store/store.c:363-410` | X-CBM-04 (reconciliación por generación), X-CBM-05, C12, C14 | Identidad de repositorio, manifiesto de importación y reglas de procedencia canónicos acordados |
| X-CBM-02 | 🟠 | Respuesta acotada ligada a snapshot de generación | `vantadb-mcp/src/code.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: `code_search`/`code_explore` devuelven página ligada a consulta+generación+scope con items, `evidence_keys`, `omitted`, `continuation`, `budget_exhausted` y `freshness`; el cursor se invalida al cambiar la generación. Por qué: un cursor reutilizado tras sync salta filas o mezcla generaciones; un fallback regex no puede presentarse como ranking BM25 ni un error como página vacía. Acción mínima: extender la presentación existente con `CodeQueryPage { repo_id, generation, items, evidence_keys, omitted, continuation, budget_exhausted, freshness }`, declarar modo efectivo y estimador de tamaño; no crear segundo retriever. Trazabilidad: informe §Ficha B · evidencia `src/mcp/compact_out.c:454-480`, `src/mcp/mcp.c:5488-5580` | X-CBM-01 (`evidence_keys`), C7, C14 | Generación e identidad de C12; presupuesto/cancelación del host |
| X-CBM-03 | 🟠 | Informe de impacto asesor con completitud declarada | `vantadb-mcp/src/code.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: la selección de tests o el impacto devuelven seeds, relaciones recorridas, paths de explicación, `completeness` y `fallback_reason`; `completeness` nunca es `Complete` si el recorrido abortó por recursos o gaps de parseo. Por qué: un vecindario BFS limitado no prueba cierre; excluir tests por cierre parcial pierde cobertura real. Acción mínima: `ImpactEvidence { revision, seeds, traversed_relations, selected_tests, explanation_paths, completeness, fallback_reason }` sobre `code_impact` existente; uso asesor con fail-open hacia suite amplia, nunca gate sin validación independiente. Trazabilidad: informe §Ficha C · evidencia `src/store/store_impact.h:1-105`, `src/mcp/test_impact_engine_reach.c:1202-1255` | X-CBM-01 (aristas con estrategia), C12 | Cobertura del productor; `IMPACT_MAX_DEPTH=10` actual documentado como límite, no como cierre |
| X-CBM-04 | 🔴 | Reconciliación por ownership y cobertura con tombstones autorizados | `vantadb-mcp/src/code_index.rs` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: un plan de reconciliación distingue deleted/mode-skipped/error antes de dar de baja evidencia; los tombstones exigen autoridad del productor, repo y generación; un lote parcial no declara eliminaciones de todo el repo ni toca registros de otro productor o del usuario. Por qué: tratar una omisión por cambio de modo o un error de lectura como baja destruye evidencia válida; `index_repo` reconcilia símbolos por archivo pero no acredita contrato de snapshot completo entre productores. Acción mínima: `plan_reconcile(previous: &Manifest, incoming: &CodeEvidenceImport) -> Result<ReconcilePlan, ReconcileError>` + `apply(plan, expected_generation)` sobre el coordinador existente; dry-run sin bajas; conservar manifest anterior y journal del plan para rollback. Trazabilidad: informe §Ficha D · evidencia `src/pipeline/pipeline_incremental.c:968-1012`, `src/pipeline/pipeline_incremental.c:4039-4218` | X-CBM-01, X-CBM-05, C6, C12 | Manifiesto por generación de C12; publicación condicional por generación |
| X-CBM-05 | 🔴 | Baseline de cambios comprometida solo tras publicación confirmada | `vanta-memory/src/services/scheduler.rs`, `vantadb-mcp/src/code_index.rs` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: observar un cambio registra un pendiente; la baseline solo avanza cuando el job publica y el recibo identifica la generación; busy conserva el pendiente, error persistente aplica backoff observable sin avanzar baseline, y el reinicio concilia pendientes en lugar de asumir fresh. Por qué: confirmar un cambio cuyo job falló deja un índice permanentemente viejo que el siguiente poll considera actual; `observe_change` no es `publish_complete` y una cola local no es ledger durable. Acción mínima: `observe(repo, fingerprint) -> PendingRevision` + `ack_published(pending, receipt) -> Result<(), SupersededOrFailed>` integrados en el coordinador existente; secuencia registrar intent → encolar → publicar → ACK condicional → conciliar. El ACK upstream solo compromete baselines tras veredicto terminal (proceso limpio + respuesta parseada), no por exit code. Trazabilidad: informe §Ficha E · evidencia `src/watcher/watcher.c:1627-1764`, `src/daemon/application.c:4436-4467` | X-CBM-04, C9, C13, C14 | Caller de job code-index definido (hoy no acreditado en Vanta); registro de intent/pending durable, no solo `LocalStateBackend`; cobertura de X-CBM-04 |
| X-CBM-06 | 🟡 | Overlay de productor/revisión/resolver y razón de stale sobre evidencia | `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: la vista de evidencia en Studio superpone productor, revisión, estrategia de resolución y razón de stale; distingue pendiente de publicación confirmada, regex de BM25 y no-indexado de vacío. Por qué: una selección visual sin revisión presenta evidencia de otra generación como actual; un grafo vacío no debe parecer ausencia de proyecto. Acción mínima: extender el flujo selección→evidencia existente en RetrievalLens con badges de revisión/resolver/stale-reason y estados de error explícitos; no crear Studio nuevo ni embeber Lens en el motor. Trazabilidad: informe §12.4 (candidata UI) · evidencia `graph-ui/src/App.tsx:1-132`, `src/mcp/mcp.c:7828-7951` | X-CBM-01, X-CBM-02, X-CBM-05, C16 | Frescura/revisión aportada por X-CBM-05/C14; Studio existente en `develop-desktop`; trabajo frontend separado del motor |

## Fichas completas

> Expansión de cada candidata a partir del informe `agent-memory-2026-11-07/codebase-memory-mcp.md` (§Fichas A–C y §12.1–12.4). Todos los contratos son **hipotéticos**: propuestas, no APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `612eee25`; las rutas VantaDB citan la baseline `97c9623` (Studio en `develop-desktop@0ad38c54`) y requieren revalidación contra HEAD.

### X-CBM-01 — Evidencia de resolución importable como memoria canónica

- **Problema:** los IDs SQLite autoincrementales del productor y la unicidad `(project, qualified_name)` no son identidad portable entre rebuilds; perder `strategy` convierte inferencia en hecho y un match débil suprimido no debe fabricar una call edge.
- **Contrato hipotético:** exclusivamente `CodeEvidenceImport` (C12 de CONSOLIDATED — único formato canónico propuesto, no se define otro DTO de importación): mapear qname/ruta/rango a `refs`, estrategia/candidatos a evidencia asociada, confianza reportada sin convertirla en confianza canónica; `resuelto`/`ambiguo`/`externo`/`pendiente` como estados distintos, con referencia repo/revisión/rango.
- **Fuente de evidencia (clone):** `src/pipeline/pass_calls.c:837-874` (`emit_classified_edge`: arista y sus evidencias, supresión de weak member matches con receiver no resuelto); `src/store/store.c:363-410` (`init_schema`: IDs autoincrementales, unicidad por proyecto/qname).
- **Seam VantaDB:** `src/sdk/types/record.rs:184-224` (`MemoryInput`) y `src/sdk/api/graph.rs:113-151` (`Embedded::add_edge` — la API de arista recibe label/peso/fecha, no un objeto arbitrario de procedencia: conservar una memoria-evidencia vinculada; inserta extremos secuencialmente, sin atomicidad multirregistro por la mera firma).
- **Acciones:**
  1. Exportación o consulta read-only explícita al productor.
  2. Validar proyecto, revisión y rutas.
  3. Resolver IDs externos a claves Vanta namespaced.
  4. Persistir evidencia antes de publicar relaciones (primero fuentes/evidencias, después relaciones).
  5. Registrar lote/generación y rechazos; error de endpoint o revisión desconocida rechaza el lote antes de publicar.
- **Dependencias:** identidad de repositorio, manifiesto de importación y reglas de procedencia canónicos acordados (prerrequisito §4); X-CBM-04 (reconciliación por generación), X-CBM-05, C12, C14.
- **FMEA:** reciclar un ID numérico enlaza símbolos ajenos; perder `strategy` convierte inferencia en hecho; mover un archivo cambia su contexto de nombre; determinismo AST confundido con exactitud del enlace. Contención: conservar referencia original y revisión; no fusionar por nombre corto.
- **Criterios de aceptación propuestos:** importar dos veces no duplica evidencia; homónimos de proyectos distintos no colisionan; `unresolved` no crea un destino supuesto; una referencia permite abrir exactamente su fuente versionada; reintentar tras fallo no duplica aristas; rollback vuelve a evidencia anterior rotulada stale si ya no corresponde al árbol.
- **Criterio negativo:** un ID interno del productor nunca se persiste como identidad Vanta; una arista `unresolved` nunca se materializa como call edge; la confianza reportada nunca se anuncia como certeza semántica.
- **Independencia:** adapter opcional que importa estrategia/candidatos que el scanner propio no emite — sin sustituir el índice propio (alternativas documentadas: extender scanners propios, adapter C12 o reescribir resolver).
- **No objetivos:** no traducir parsers/LSP a Rust, no copiar SQLite ni hacer que `confidence=1` signifique certeza semántica; desactivar el productor no borra memoria manual.

### X-CBM-02 — Respuesta acotada ligada a snapshot de generación

- **Problema:** un cursor reutilizado tras sync salta filas o mezcla generaciones; un fallback regex no puede presentarse como ranking BM25 ni un error como página vacía; la lectura de una query no debe activar una reconstrucción.
- **Contrato hipotético:** `CodeQueryPage { repo_id, generation, items, evidence_keys, omitted, continuation, budget_exhausted, freshness }` — página ligada a consulta+generación+scope; el cursor pertenece a una generación y consulta y se invalida al cambiar la generación; tamaño en bytes/caracteres con estimador declarado, nunca anunciado como tokenización exacta. Cursor firmado o estado server-side; nunca cursor libre que amplíe root.
- **Fuente de evidencia (clone):** `src/mcp/compact_out.c:454-480` (`cbm_sb_*`, compactación de salida); `src/mcp/mcp.c:881-944` (`TOOL_ANNOTATIONS` separa queries de indexado, borrado, ADR y traces — annotations no son autorización) y `src/mcp/mcp.c:5488-5580` (handler de query).
- **Seam VantaDB:** `vantadb-mcp/src/code.rs` (`code_search`/`code_explore` y ensamblado de contexto ya existentes — extender la presentación, no crear una segunda búsqueda semántica ni segundo retriever).
- **Acciones:**
  1. Fijar snapshot de generación.
  2. Normalizar consulta sin borrar nombres cualificados.
  3. Seleccionar resultados y cortar unidades completas.
  4. Devolver omisiones y cursor.
  5. Antes del siguiente page, validar snapshot y ACL; representación JSON literal para máquinas y referencias reversibles.
- **Dependencias:** X-CBM-01 (`evidence_keys`), C7, C14; generación e identidad de C12; presupuesto/cancelación del host.
- **FMEA:** cursor reutilizado tras sync salta filas (ligar a generación); comprimir rutas sin regla de expansión destruye evidencia (referencias reversibles); presupuestar solo snippets omite overhead MCP (estimador declarado sobre la respuesta completa); BM25+semantic simultáneos (rechazado en CM-1); fallback regex declarado como modo efectivo, nunca score BM25 ni híbrido implícito.
- **Criterios de aceptación propuestos:** paginar un índice sin cambios reconstruye el mismo conjunto; cambiar generación invalida continuación; una fila mayor al presupuesto no genera un cursor que apunte a sí mismo; omisión no se presenta como ausencia; no-indexado/error no son página vacía; truncamiento se detecta por contadores y se mitiga con página posterior, no con salida «sin más dependencias»; contar omitidos, bytes/estimador declarado y fallos freshness, sin código en logs.
- **Criterio negativo:** un error o un índice ausente nunca se devuelven como página vacía; un cursor de otra generación nunca se acepta; una omisión nunca se presenta como ausencia de evidencia.
- **Independencia:** delta de presentación sobre el retriever existente; el lector no cruza generaciones.
- **No objetivos:** no copiar un formato compacto propietario como schema canónico ni prometer ahorro de tokens sin tokenizer y captura del cliente.

### X-CBM-03 — Informe de impacto asesor con completitud declarada

- **Problema:** la selección de tests o el impacto desde un vecindario BFS limitado no prueba cierre; excluir tests por cierre parcial pierde cobertura real. `completeness` nunca puede ser `Complete` si el recorrido abortó por recursos o gaps de parseo.
- **Contrato hipotético:** `ImpactEvidence { revision, seeds, traversed_relations, selected_tests, explanation_paths, completeness, fallback_reason }` — reporte de aplicación, no nuevo store; uso asesor con fail-open hacia suite amplia, nunca gate sin validación independiente.
- **Fuente de evidencia (clone):** `src/store/store_impact.h:1-105` (`cbm_impact_walk_*`: walk inverso con fixpoint `max_hops=0`, ciclos con expansión única, padres deterministas y errores de asignación en lugar de respuestas recortadas); `src/mcp/test_impact_engine_reach.c:1202-1255` (`te_lift_fixpoint`; los lifts incluyen roles/subprocesos — no es solo «tres saltos de callers»).
- **Seam VantaDB:** `vantadb-mcp/src/code.rs` (`code_impact`, `IMPACT_MAX_DEPTH=10` en `code.rs:33-36,272-301` — ese límite es correcto para exploración, pero no demuestra selección exhaustiva; documentarlo como límite, no como cierre).
- **Acciones:**
  1. Importar el informe upstream como evidencia opcional.
  2. Mapear tests a fuentes.
  3. Comprobar revisión/cobertura y mostrar fallback amplio.
  4. Revisionar seeds/caminos; rollback elimina la proyección asesoría, no cambia gates.
  5. Cualquier uso para excluir tests exige autorización y validación independiente; prueba adversa de callbacks/macros/ciclos antes de excluir cualquier test.
- **Dependencias:** X-CBM-01 (aristas con estrategia), C12; cobertura del productor.
- **FMEA:** cierre parcial excluye tests relevantes; una arista `CALL_REFERENCE` ignorada pierde callbacks; alcanzar el runner puede implicar ejecutar todo; error de alloc/cancelación produce `Incomplete`/error — nunca conjunto vacío «seguro»; fail-open hacia suite amplia, no hacia menos tests.
- **Criterios de aceptación propuestos:** ciclos terminan; ambigüedad/cobertura ausente no producen selección vacía «segura»; la explicación incluye relación y fuente de cada paso; contar fallback, gaps y walks cancelados.
- **Criterio negativo:** `completeness` nunca es `Complete` tras un aborto por recursos o gaps de parseo; un informe asesor nunca actúa como gate de CI sin validación independiente.
- **Independencia:** solo asesor hasta acreditar cobertura (alternativa documentada: mostrar informe upstream frente a cerrar el grafo propio).
- **No objetivos:** no diseñar un runner de CI ni una política de selección nueva; no reutilizar la selección experimental de tests como gate de Vanta sin evidencia equivalente.

### X-CBM-04 — Reconciliación por ownership y cobertura con tombstones autorizados

- **Problema:** un índice full importado y después un delta moderate sin docs/tests: tratar esa omisión como baja destruye evidencia válida; el plan debe distinguir deleted/mode-skipped/error antes de dar de baja evidencia, y un lote parcial no declara eliminaciones de todo el repo ni toca registros de otro productor o del usuario.
- **Contrato hipotético:** `plan_reconcile(previous: &Manifest, incoming: &CodeEvidenceImport) -> Result<ReconcilePlan, ReconcileError>` y `apply(plan, expected_generation)` (`Manifest`/`Plan`/`Error` son conceptos de aplicación pendientes de spec). Invariantes: tombstone exige autoridad del productor, repo y generación; error de lectura no demuestra ausencia; retirar una proyección **no** autoriza el alcance de un borrado de proyecto upstream (que incluye ADR humano); `delete_failed`/`not_found` y watcher retirado se conservan en el recibo del adapter, no se comprimen a éxito.
- **Fuente de evidencia (clone):** `src/pipeline/pipeline_incremental.c:968-1012` (`find_deleted_files`) y `src/pipeline/pipeline_incremental.c:4039-4218` (routing) — extraer la distinción deleted/mode-skipped/error, no su motor ni heurísticas.
- **Seam VantaDB:** `vantadb-mcp/src/code_index.rs` (`index_repo:193-235`, revalidado `:152-327` con despacho multiidioma `:514-520`) — reconcilia símbolos del archivo pero no acredita contrato de snapshot completo entre productores; extender ese coordinador y los jobs existentes, no otro storage.
- **Acciones:**
  1. Inventariar el manifest existente y namespaces de ownership.
  2. Modo dry-run sin bajas.
  3. Validar cobertura/endpoints antes del commit.
  4. Publicar generación y garbage collection posterior.
  5. Conservar manifest anterior y journal del plan para rollback; datos legacy sin owner no se borran automáticamente.
- **Dependencias:** X-CBM-01, X-CBM-05, C6, C12; manifiesto por generación de C12 y publicación condicional por generación; no bloquea el inspector read-only.
- **FMEA:** cambio de modo → falsa baja (detectar diferencia de coverage/mode y preservar); dos importadores → borrado cruzado (owner estricto); delta incompleto → dangling edges (validar endpoints antes de commit).
- **Criterios de aceptación propuestos:** archivo ilegible preservado con error; eliminación explícita invalida relaciones incidentes propias; reimport idempotente; métricas de bajas propuestas/aplicadas/rechazadas, huérfanos y lag por generación.
- **Criterio negativo:** una omisión por cambio de modo o un error de lectura nunca se convierte en baja; un lote parcial nunca declara eliminaciones de todo el repo; tombstones sin autoridad del productor/repo/generación se rechazan.
- **Independencia:** un productor y snapshot completo como mínimo; alternativas: full reimport conservador (preferido al principio si el export no acredita delta), delta con tombstones o watcher directo; no deducir tombstones desde un listado paginado.
- **No objetivos:** no selector de tests, no sincronización bidireccional ni promesa de rename identity sin evidencia; desactivar la reconciliación sirve la última generación marcada.

### X-CBM-05 — Baseline de cambios comprometida solo tras publicación confirmada

- **Problema:** consumir un evento y avanzar el hash aunque el job haya fallado deja un índice permanentemente viejo que el siguiente poll considera actual; `observe_change` no es `publish_complete` y una cola local no es un ledger durable.
- **Contrato hipotético:** `observe(repo, fingerprint) -> PendingRevision` y `ack_published(pending, receipt) -> Result<(), SupersededOrFailed>`; el receipt identifica la generación C12 — no confirma entrega de red. Secuencia: registrar intent → encolar → publicar → ACK condicional → conciliar pendientes al reiniciar. El ACK upstream solo compromete baselines tras veredicto terminal (proceso limpio + respuesta parseada), no por exit code.
- **Fuente de evidencia (clone):** `src/watcher/watcher.c:1623-1644` (`commit_baselines`) y `:1731-1750` (caller `poll_project`), estado `:64-108` — la unidad extraíble es ACK solo tras `index_fn==0`, no el proceso daemon; `src/daemon/application.c:4436-4467` (admisión/trabajo).
- **Seam VantaDB:** `vanta-memory/src/services/scheduler.rs` (`run_pass:79-119`, `LocalStateBackend`, TimerScanner, PipelineWorker — scheduler del writer único, cola/timers/leases locales `:25-30`; TaskCheckpoint no hace durable esa cola por existencia), `core/state/types.rs::TaskKind:38-50` (solo L1/L2/L3/Dream/Flush — no usar una tarea L1 como code-index por alias) y `vantadb-mcp/src/code_index.rs` (`index_repo:193-202` compara hash, no encola ni confirma job). **No se acredita un caller/job de code-index ni ACK persistente conectado**: integrar es trabajo nuevo, no construir otro scheduler/WAL.
- **Acciones:**
  1. Definir el caller de code-index (hoy no acreditado en Vanta).
  2. Registrar intent/pending durable, no solo `LocalStateBackend`.
  3. Encolar job, publicar y ACK condicional con dedup por generación.
  4. Conciliar pendientes al reiniciar desde la fuente, sin presumir que la cola local los persiste.
  5. Legacy desconocido fuerza comprobación, no baseline fresh; desactivar auto-refresh conserva consulta manual.
- **Dependencias:** X-CBM-04 para cobertura y escritor condicional; C9, C13, C14; rollback cancela nuevos jobs y mantiene pending diagnosticable — no revierte el código fuente.
- **FMEA:** caída tras publish antes de ACK → repetición (dedup por generación); busy → perder evento (pending retenido); error persistente → tormenta de jobs (backoff observable sin avanzar baseline); job antiguo no pisa al nuevo.
- **Criterios de aceptación propuestos:** cambio durante trabajo genera pendiente posterior; reinicio recupera pendiente; fallo no anuncia current; observar retries, edad del pendiente, estados degraded y último publish — nunca paths sensibles como etiquetas métricas.
- **Criterio negativo:** la baseline nunca avanza por un job fallido, ocupado o no confirmado; un `PendingRevision` nunca se descarta por reinicio; hash igual o cola vacía no acreditan publicación.
- **Independencia:** preferir polling/reconcile único existente a un segundo watcher; hooks solos pierden ediciones no committeadas; retry idempotente.
- **No objetivos:** no timers/umbrales nuevos, no instalar hooks ni declarar exactly-once global; conectar cambios a jobs es el mínimo.

### X-CBM-06 — Overlay de productor/revisión/resolver y razón de stale sobre evidencia

- **Problema:** una selección visual sin revisión presenta evidencia de otra generación como actual, y un grafo vacío no debe parecer ausencia de proyecto; pendiente de publicación confirmada, regex de BM25 y no-indexado de vacío deben distinguirse.
- **Contrato hipotético:** la vista de evidencia superpone productor, revisión, estrategia de resolución y razón de stale sobre el flujo selección→evidencia existente; badges de revisión/resolver/stale-reason y estados de error explícitos (CM-1/CM-2: distinguir regex de BM25, no-indexado de vacío, borrado fallido de proyección retirada y pending de publicación confirmada).
- **Fuente de evidencia (clone):** `graph-ui/src/App.tsx:1-132` (acredita principalmente comentario/imports, no un flujo React completo — los nombres Reader/Atlas/diagnóstico/ADR no sustituyen lectura de cuerpos/callers); `src/mcp/mcp.c:7828-7951` (endpoints servidos a la UI).
- **Seam VantaDB:** `feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:138-164,193-228` en `develop-desktop@0ad38c54` (search/filtros/score/contexto/inspector ya existentes — superponer el overlay a ese flujo; leído vía `git show`, no bytes dirty).
- **Acciones:**
  1. Extender el flujo selección→evidencia existente con badges de productor/revisión/resolver.
  2. Añadir razón de stale y estados de error explícitos.
  3. Distinguir pendiente de publicación confirmada y modo efectivo de búsqueda.
  4. Mantener el trabajo frontend separado del motor — no crear Studio nuevo ni embeber Lens en el motor.
- **Dependencias:** X-CBM-01, X-CBM-02, X-CBM-05, C16; frescura/revisión aportada por X-CBM-05/C14; el pin no acredita JobChip/jobs ni un panel de baseline ya implementado.
- **FMEA:** — (el informe no documenta FMEA específico para la candidata UI; los riesgos cubiertos son los del CM-1/CM-2 de presentación: confundir regex con BM25, no-indexado con vacío, borrado fallido con proyección retirada, pending con publicación confirmada).
- **Criterios de aceptación propuestos:** la vista distingue regex de BM25, no-indexado de vacío, borrado fallido de proyección retirada y pending de publicación confirmada; aceptar el panel de baseline requeriría evidenciar caller/resultado/ACK — no basta hash igual ni cola vacía.
- **Criterio negativo:** una evidencia de otra generación nunca se presenta como actual sin marca de revisión; un grafo vacío nunca se renderiza como ausencia de proyecto.
- **Independencia:** trabajo frontend separado del motor sobre el Studio existente; requiere los campos aportados por X-CBM-01/X-CBM-02/X-CBM-05.
- **No objetivos:** no crear Studio nuevo, no embeber Lens en el motor, no Galaxy como prueba de cobertura.
