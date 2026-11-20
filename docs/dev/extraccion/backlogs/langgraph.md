---
title: "Backlog de extracción — LangGraph"
kind: research
status: active
description: "Siete candidatas del informe LangGraph: conformidad versionada del adapter, writes intermedios, replay/aprobación, capacidades de store y serde, visibilidad de batch, borrado de thread y linaje de checkpoints"
tags: [extraccion, backlog, langgraph]
---

# Backlog de extracción — LangGraph

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/langgraph.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-LGR-01 | 🟠 | Conformidad versionada del adapter LangChain | `integrations/langchain/pyproject.toml:25-32` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | **Qué:** fijar y declarar la familia de versiones soportadas del adapter (Core, checkpoint, Python) y la matriz de conformidad antes de prometer compatibilidad. **Por qué:** los rangos `Core >=0.3,<1` declarados no intersectan con el `>=1.7.5,<2` del LangChain moderno del clon; no invalida el adapter en su familia, pero impide prometer el agente moderno sin migración explícita. **Acción mínima:** resolver/instalar versiones declaradas, documentar conformidad por versión y decidir migración explícita; prioridad P0 según el orden del informe. **Trazabilidad:** informe §2 "Versiones antes de conformidad" · evidencia `integrations/langchain/pyproject.toml:25-32` (V8). | Condiciona X-LGR-02..07, C11 | Resolución de dependencias no ejecutada; instancia desechable para pruebas de conformidad |
| X-LGR-02 | 🟠 | Escrituras intermedias con identidad estable y política de conflicto | `integrations/langchain/vantadb_langchain/checkpointer.py:232-296` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | **Qué:** clave de pending write (`tenant/thread/namespace/checkpoint/task/slot`) y política de conflicto declarada al persistir un batch; el slot especial se deriva del canal y de la versión del protocolo, el ordinario conserva posición. **Por qué:** el adapter hace get-before-put/skip para todo canal mientras el referente distingue `WRITES_IDX_MAP` y upsert de controles — diferencia estática de protocolo que puede perder un error actualizado o duplicar updates. **Acción mínima:** documentar semántica insert/replace por versión, comprobar atomicidad disponible del SDK y probar conformidad upstream en instancia desechable. **Trazabilidad:** informe §Ficha A · evidencia `libs/checkpoint-postgres/langgraph/checkpoint/postgres/__init__.py:352-384`, `libs/checkpoint-postgres/langgraph/checkpoint/postgres/base.py:134-162`, `libs/langgraph/langgraph/pregel/_loop.py:463-535`. | X-LGR-05 (usa estas keys), X-LGR-06 (conserva task/parent IDs), C11, C13 | Versión soportada fijada (X-LGR-01); atomicidad del SDK; `TaskCheckpointManager` sin fusionar dominios checkpointer/checkpoint de tarea |
| X-LGR-03 | 🟠 | Registro de decisión y recibo de efecto en resume | `vanta-memory/src/core/record/approval.rs` | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | **Qué:** aprobación durable (`ApprovalRecord` propuesto: operation_id, input_digest, decision_revision, actor, status) separada de la ejecución del efecto; resume valida identidad/revisión y el ejecutor externo recibe clave idempotente con recibo. **Por qué:** `interrupt` reanuda desde el principio del nodo y un efecto previo puede reejecutarse; persistir la decisión no equivale a ejecutar el efecto ni a una transacción con el servicio remoto. **Acción mínima:** separar intención, decisión y recibo en el host, enlazar provenance/checkpoint, declarar resultado desconocido cuando el remoto no soporta idempotencia. **Trazabilidad:** informe §Ficha B · evidencia `libs/langgraph/langgraph/types.py:913-935,1023-1049`, `libs/langgraph/langgraph/pregel/_runner.py:575-615`. | X-LGR-02, X-LGR-06, C8, C10, C13 | Provenance/checkpoint existentes; host autentica al aprobador; `CaptureApprovalQueue` existe pero no es aprobación durable de tool; condicionada a efectos sensibles |
| X-LGR-04 | 🟡 | Capacidades declaradas de store y serde versionado | `integrations/langchain/vantadb_langchain/store.py:97-139,231-271` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | **Qué:** contrato que enumera opciones soportadas del store (`semantic, ttl, field_indexing, ordered_batch`) y envelope `codec_version/type/payload`; las opciones no implementadas fallan explícitas, no se ignoran. **Por qué:** el adapter no debe aceptar opciones que no puede preservar: `index` no se traslada a `_put` en la lectura y `PutOp.ttl` ya se rechaza con `NotImplementedError`; un serializer permisivo sin allowlist degrada tipos silenciosamente. **Acción mínima:** enumerar opciones del contrato upstream congelado, traducir filtros soportados, rechazar las restantes con diagnóstico y distinguir payload recuperable de objeto reconstruido. **Trazabilidad:** informe §Ficha C · evidencia `libs/checkpoint/langgraph/store/base/__init__.py:160-240,475-520`, `libs/langgraph/langgraph/channels/binop.py:107-150`, `libs/checkpoint/langgraph/checkpoint/serde/jsonplus.py:83-101,104-147`. | X-LGR-05 (D usa capacidades), C15, C11 | Versión/política de serde del paquete instalado con escritor autorizado; namespace-tupla sin colisiones; ACL aplicada antes de buscar |
| X-LGR-05 | 🟠 | Visibilidad de batch declarada e indexación selectiva | `integrations/langchain/vantadb_langchain/store.py` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | **Qué:** modelo de visibilidad dentro de un batch (reads/search preparados antes de aplicar puts; última operación por namespace/key) y política de indexación que nunca envía campos excluidos al embedder. **Por qué:** un agente que guarda preferencia y consulta en el mismo batch no debe obtener resultado distinto por asumir aplicación en orden; `index=False` no debe filtrar campos secretos al proveedor. **Acción mínima:** validar todas las capacidades antes de mutar, seleccionar campos antes del embedding, aplicar primario y registrar estado de proyección por batch. **Trazabilidad:** informe §Ficha D · evidencia `libs/checkpoint/langgraph/store/memory/__init__.py:206-234,375-402,418-440`. | X-LGR-04, C7, C15 | Semántica de la versión upstream soportada; host valida scope e index policy; revisión/single-writer si el SDK no da CAS |
| X-LGR-06 | 🟠 | Borrado de thread con alcance y cierre verificables | `integrations/langchain/vantadb_langchain/checkpointer.py:298-313` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | **Qué:** retirada de thread (`ThreadDeletion` propuesta) que enumera familias — checkpoints y writes son distintas —, serializa con writers activos, borra reanudablemente, verifica ausencia y emite recibo `complete/remaining/blocked_writers`. **Por qué:** retirar una conversación no debe dejar writes reactivables ni borrar memorias compartidas por otros threads; paginar mientras se borra o sigue escribiendo deja restos. **Acción mínima:** autenticar, cerrar generación o serializar con writers, enumerar familias, borrar reanudablemente, verificar y emitir recibo. **Trazabilidad:** informe §Ficha E · evidencia `libs/checkpoint-sqlite/langgraph/checkpoint/sqlite/__init__.py:458-515`, adapter `integrations/langchain/vantadb_langchain/checkpointer.py:298-313`. | X-LGR-02 (keys), X-LGR-07 (linaje), C6, C13 | Lifecycle/provenance de `vanta-memory`; retención de auditoría como política aparte; `BaseStore` no se borra sin referencias y permiso explícitos |
| X-LGR-07 | 🟢 | Vista de linaje de checkpoint y modo de durabilidad | `integrations/langchain/vantadb_langchain/checkpointer.py` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | **Qué:** vista de inspección (`CheckpointView` propuesta: thread/namespace/checkpoint/parent/graph_revision/codec_revision/durability_mode/pending_task_ids) con desconocido explícito donde falte dato. **Por qué:** un operador debe distinguir reanudar el último estado, replay desde un padre y una nueva rama; un modo `exit` rotulado como durable por superstep daría falsa recuperación y un checkpoint previo a revocación no debe reinyectar contenido retirado. **Acción mínima:** exponer al adapter/Studio lo ya persistido, documentar campos disponibles, verificar compatibilidad antes de resume y consultar linaje paginado con ACL. **Trazabilidad:** informe §Ficha F · evidencia `libs/checkpoint/langgraph/checkpoint/base/__init__.py:93-360`, `libs/langgraph/langgraph/pregel/_loop.py:1300-1360`, adapter `integrations/langchain/vantadb_langchain/checkpointer.py:41-350`. | X-LGR-02/03 (identidades), C14, C4, C16 | Campos disponibles documentados; revisiones de host/mode registradas; UI lectora por defecto que nunca despacha tools |

## Fichas completas

> Expansión de cada candidata a partir del informe `docs/dev/extraccion/agent-memory-2026-11-07/langgraph.md` (§2 «Versiones antes de conformidad», recorrido del adapter y fichas A–F). Todos los contratos son **hipotéticos** — propuestas de comportamiento para VantaDB, no APIs existentes ni traducciones de código. Las referencias `archivo:líneas` citan el clon `C:\Users\Eros\langgraph` en SHA `e5e37b5d70dbfd93c7006bd90a658c6179a95946` (rutas relativas al repo LangGraph, prefijo `libs/`); el adapter Vanta citado es `integrations/langchain/vantadb_langchain/` en el snapshot `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` y requiere revalidación contra HEAD.

### X-LGR-01 — Conformidad versionada del adapter LangChain

- **Problema:** los rangos `Core >=0.3,<1` declarados por el adapter no intersectan con el `>=1.7.5,<2` del LangChain moderno del clon; eso no invalida el adapter en su familia declarada, pero impide prometer compatibilidad con el agente moderno sin migración explícita. No se resolvieron ni instalaron dependencias.
- **Contrato hipotético (tipos propuestos):** — (no hay tipo propuesto en el informe; la unidad es la declaración de familia de versiones y la matriz de conformidad, no un DTO).
- **Fuente de evidencia:** `integrations/langchain/pyproject.toml:25-32` (repo VantaDB, [V8]: declara Core `>=0.3,<1`, checkpoint `>=2,<5`, Python `>=3.11`); el informe §2 documenta el requisito `Core >=1.7.5,<2` del LangChain moderno del clon hermano (citado en `langchain.md` [C12]) — sin cita `archivo:líneas` del clon langgraph para este punto concreto.
- **Seam VantaDB:** `integrations/langchain/pyproject.toml` y el adapter `vantadb_langchain` existente — fijar y declarar la familia soportada antes de prometer compatibilidad.
- **Acciones:**
  1. Resolver/instalar las versiones declaradas.
  2. Documentar la conformidad por versión (Core, checkpoint, Python).
  3. Decidir la migración explícita si se quiere la familia moderna.
  4. Probar conformidad upstream en instancia desechable antes de prometer.
- **Dependencias:** resolución de dependencias no ejecutada durante la investigación; instancia desechable para pruebas de conformidad. Condiciona X-LGR-02..07; converge con C11 de `CONSOLIDATED.md`.
- **FMEA:** prometer compatibilidad con el agente moderno sobre una familia Core no intersectante → soporte declarado falso; cambio silencioso de versión upstream → conformidad caduca → matriz versionada.
- **Criterios de aceptación propuestos:** la familia soportada queda fijada y documentada; las pruebas de conformidad corren sobre versiones declaradas; la migración se decide explícitamente, no por coincidencia parcial de API.
- **Criterio negativo:** ninguna documentación promete compatibilidad con una familia de versiones no verificada; una dependencia por rango no demuestra la versión instalada del usuario.
- **Independencia:** es la puerta de entrada — condiciona todas las demás candidatas (X-LGR-02..07 dependen de la versión soportada fijada); ella misma no requiere ninguna otra.
- **No objetivos:** reescribir el adapter para la familia moderna sin decisión explícita; ejecutar la resolución de dependencias como parte de esta investigación.

### X-LGR-02 — Escrituras intermedias con identidad estable y política de conflicto

- **Problema:** el adapter hace get-before-put/skip para todo canal mientras el referente distingue `WRITES_IDX_MAP` y upsert de controles — diferencia estática de protocolo que puede perder un error actualizado o duplicar updates; no es reproducción ni prueba sobre versiones posteriores del adapter.
- **Contrato hipotético (tipos propuestos):** `PendingWriteKey { tenant, thread, namespace, checkpoint, task, slot }` y operación `persist_writes(batch, conflict_policy)`. El slot especial se deriva del canal y de la versión del protocolo; el ordinario conserva posición.
- **Fuente de evidencia (clone):** `libs/checkpoint-postgres/langgraph/checkpoint/postgres/__init__.py:352-384` (`PostgresSaver.put_writes`) [L2], `libs/checkpoint-postgres/langgraph/checkpoint/postgres/base.py:134-162` (SQL de conflicto) [L3], `libs/langgraph/langgraph/pregel/_loop.py:463-535` (`PregelLoop.put_writes`) [L4]; adapter `integrations/langchain/vantadb_langchain/checkpointer.py:269-296` (`put_writes` enumera posiciones y hace get-before-put/skip para todo canal) [V6].
- **Seam VantaDB:** `integrations/langchain/vantadb_langchain/checkpointer.py::VantaDBCheckpointer.put/put_writes` (`:232-296`); `vanta-memory/src/utils/task_checkpoint.rs` (`TaskCheckpointManager:122-188`) — dominios checkpointer y checkpoint de tarea distintos que no deben fusionarse a ciegas.
- **Acciones:**
  1. Fijar la versión soportada del paquete (X-LGR-01).
  2. Documentar la semántica insert/replace por versión.
  3. Comprobar la atomicidad disponible del SDK.
  4. Definir la migración (versión de formato, lectura antigua, desactivación sin reinterpretar slots históricos).
  5. Probar manualmente la conformidad upstream en instancia desechable. Serializar primero evita error de codec a mitad de bucle; no resuelve un fallo de IO posterior.
- **Dependencias:** versión soportada fijada (X-LGR-01); atomicidad del SDK; `TaskCheckpointManager` sin fusionar dominios. X-LGR-05 usa estas keys; X-LGR-06 conserva task/parent IDs; converge con C11/C13 de `CONSOLIDATED.md`. Alternativa mínima documentada: mantener el protocolo Python y ampliar pruebas de conformidad; Rust solo si falta una primitiva real de persistencia.
- **FMEA:** read-before-write no atómico; cambio de slots al filtrar controles; carrera entre workers; pérdida de un error actualizado. Requiere comparar la revisión actual de Vanta — no se declara bug reproducido.
- **Criterios de aceptación propuestos:** repetir un batch no duplica updates ordinarios; los reemplazos de control son los previstos por esa versión; namespace/thread no colisionan; reinicio entre batch y snapshot conserva las tareas ya persistidas. IDs de task/checkpoint y código de fallo bastan para diagnóstico; no volcar valores sensibles.
- **Criterio negativo:** el adapter nunca reasigna slots tras filtrar mensajes (el runtime es owner de task/channel e índice especial); cancelar no implica rollback de writes confirmados.
- **Independencia:** parcial — requiere la versión fijada de X-LGR-01; la unidad de commit debe declararse por batch o por write con estado parcial recuperable.
- **No objetivos:** exactly-once de herramientas ni nuevo workflow engine; no extraer el scheduler completo.

### X-LGR-03 — Registro de decisión y recibo de efecto en resume

- **Problema:** `interrupt` reanuda desde el principio del nodo y un efecto previo puede reejecutarse; persistir la decisión no equivale a ejecutar el efecto ni a una transacción con el servicio remoto — la reanudación debe distinguir pendiente, aprobada y aplicada.
- **Contrato hipotético (tipos propuestos):** `ApprovalRecord { operation_id, input_digest, decision_revision, actor, status }`; `resume` valida identidad/revisión. El ejecutor externo recibe una clave idempotente y registra su recibo.
- **Fuente de evidencia (clone):** `libs/langgraph/langgraph/types.py:913-935,1023-1049` (`interrupt(value)` exige checkpointer y reanuda desde el principio del nodo; la correspondencia entre múltiples interrupciones usa el orden dentro de la tarea) [L5]; `libs/langgraph/langgraph/pregel/_runner.py:575-615` (`PregelRunner.commit`) [L6].
- **Seam VantaDB:** `vanta-memory/src/core/record/approval.rs` (`CaptureApprovalQueue` existe pero aprueba capturas/memorias, no es aprobación durable de una tool [V9]); `TaskCheckpointManager` y scheduler existentes; adaptador Python para la semántica LangGraph — no reinterpretación de nodos en Rust.
- **Acciones:**
  1. Separar intención, decisión y recibo en el host.
  2. Fijar quién autentica al aprobador.
  3. Enlazar provenance/checkpoint.
  4. Acordar la recuperación cuando el remoto acepta pero no llega el recibo — declarar resultado desconocido y reconciliar.
  5. Antes de dispatch validar revisión/claim; tras dispatch la cancelación solo bloquea intentos nuevos.
- **Dependencias:** provenance/checkpoint existentes; el host autentica al aprobador; condicionada a efectos sensibles. X-LGR-02 y X-LGR-06 conservan task/parent IDs; converge con C8/C10/C13 de `CONSOLIDATED.md`. Preferencia documentada: `interrupt/Command` upstream con registro de decisión en el host — compartir referencias/provenance, no asumir intercambiabilidad.
- **FMEA:** repetir pago/envío; aprobación sobre input modificado; orden de interrupts cambiado por una nueva versión; tenant incorrecto. Si el remoto no soporta idempotencia, declarar resultado desconocido y reconciliar.
- **Criterios de aceptación propuestos:** crash antes/después de aprobación y antes/después de recibo no presenta un efecto no confirmado como completado; decisión obsoleta rechazada; el replay conserva identidad de operación y auditoría. Caso LG-1: checkpoint creado antes de retirar una fuente o revocar permiso — autorizar acceso antes de decode y revalidar en host antes de resume/recall/efecto; resultado permitido solo para datos aún autorizados; denegación o redacción explícita para lo revocado, nunca reinyección automática desde la copia histórica.
- **Criterio negativo:** una decisión persistida nunca se presenta como efecto ejecutado; la migración aditiva deja decisiones antiguas sin recibo como desconocidas; el rollback deshabilita dispatch — no borra auditoría ni revierte una operación remota.
- **Independencia:** parcial — X-LGR-02/X-LGR-06 conservan las identidades que usa; no depende de inventar un scheduler ni un motor de agentes.
- **No objetivos:** prometer exactly-once distribuido ni adoptar Studio.

### X-LGR-04 — Capacidades declaradas de store y serde versionado

- **Problema:** el adapter no debe aceptar opciones que no puede preservar: `index` no se traslada a `_put` en la lectura del informe y `PutOp.ttl` ya se rechaza con `NotImplementedError`; un serializer permisivo sin allowlist degrada tipos silenciosamente.
- **Contrato hipotético (tipos propuestos):** `StoreCapabilities { semantic, ttl, field_indexing, ordered_batch }` y envelope `codec_version/type/payload`. `ordered_batch` significa declarar el modelo de visibilidad, no imponer read-your-writes como contrato universal. Namespace es una tupla sin colisiones; ACL se aplica antes de buscar.
- **Fuente de evidencia (clone):** `libs/checkpoint/langgraph/store/base/__init__.py:160-240,475-520` (`SearchOp`/`PutOp`, `index` en `:490-520`) [L7]; `libs/langgraph/langgraph/channels/binop.py:107-150` (`BinaryOperatorAggregate.update`) [L8]; `libs/checkpoint/langgraph/checkpoint/serde/jsonplus.py:83-101,104-147` (fallback de reconstrucción de `JsonPlusSerializer`; `pickle_fallback=False` por defecto no implica seguridad general — msgpack es permisivo salvo strict/allowlist) [L9][L13].
- **Seam VantaDB:** `integrations/langchain/vantadb_langchain/store.py:97-139,231-271` (`VantaDBStore.batch`/`_search_semantic`); Vanta delega serde al paquete instalado [V6] — fijar versión, política y escritor autorizado es prerrequisito; context engine existente para budgets sin asignar presupuesto independiente al adapter.
- **Acciones:**
  1. Enumerar las opciones del contrato upstream congelado.
  2. Traducir los filtros soportados.
  3. Rechazar las restantes con diagnóstico.
  4. Distinguir payload recuperable de objeto reconstruido.
  5. Migración: probar lectura de checkpoints viejos, mantener bytes originales y rechazar tipos incompatibles antes de ejecutar nodos.
- **Dependencias:** versión/política de serde del paquete instalado con escritor autorizado; namespace-tupla sin colisiones; ACL aplicada antes de buscar. X-LGR-05 usa estas capacidades; converge con C15/C11 de `CONSOLIDATED.md`. Alternativa documentada: envelope opaco conservado en Rust y decode solo en host autorizado.
- **FMEA:** namespace como prefijo textual ambiguo (mitigado: el adapter compara prefijos por tupla — colisión `user/1`–`user/12` no demostrada); serializer con clases peligrosas; filtros post-top-k que reducen recall; pérdida silenciosa de tipos. No se afirma que todo ello ocurra en Vanta.
- **Criterios de aceptación propuestos:** round-trip de namespace y mensajes; rechazo de codec/tipo desconocido con diagnóstico; la búsqueda nunca cruza tenant; opciones no implementadas fallan explícitamente, no se ignoran.
- **Criterio negativo:** ninguna opción no implementada se acepta silenciosamente; un error de decode/cancelación nunca se representa como checkpoint vacío exitoso; no reindexar memoria por cambiar la allowlist de checkpoints.
- **Independencia:** sí como contrato declarativo de capacidades sobre el adapter existente; X-LGR-05 la consume para su modelo de visibilidad.
- **No objetivos:** portar clases Python ni reducers arbitrarios a Rust; sustituir reducers Python por JSON.

### X-LGR-05 — Visibilidad de batch declarada e indexación selectiva

- **Problema:** un agente que guarda preferencia y consulta en el mismo batch no debe obtener un resultado distinto por asumir aplicación en orden; `index=False` no debe filtrar campos secretos al embedder.
- **Contrato hipotético (tipos propuestos):** `StoreBatchPlan { read_revision, mutations, index_policy } -> BatchReceipt { applied_ids, visibility, projection_state }`; errores `UnsupportedIndexPolicy | Conflict | PartialCommit`.
- **Fuente de evidencia (clone):** `libs/checkpoint/langgraph/store/memory/__init__.py:206-234` (`InMemoryStore.batch/abatch`: reads/search se preparan antes de aplicar puts; la última operación por namespace/key queda en el map), `:375-402` (`_prepare_ops`), `:418-440` (`_extract_texts`) [L11] — el referente es una implementación, no prueba de garantía universal de `BaseStore`; Vanta recorre secuencialmente [V7].
- **Seam VantaDB:** `integrations/langchain/vantadb_langchain/store.py` (`VantaDBStore.batch`/`_put`) — delta sobre el store existente, no un nuevo KV.
- **Acciones:**
  1. Fijar la semántica de la versión soportada.
  2. Caracterizar put/get/delete/search dentro de un batch.
  3. Validar todas las capacidades antes de mutar.
  4. Seleccionar campos antes del embedding — nunca enviar campos excluidos al embedder.
  5. Aplicar el primario y registrar el estado de proyección por batch; reindexación solo de registros cuya política cambie, sin enviar retrospectivamente campos excluidos.
- **Dependencias:** semántica de la versión upstream soportada (X-LGR-01); el host valida scope e index policy (el modelo no elige ACL); revisión/single-writer si el SDK no da CAS. X-LGR-04 declara las capacidades que consume; converge con C7/C15 de `CONSOLIDATED.md`. Alternativas: documentar diferencias permitidas o rechazar operaciones compuestas no soportadas; no copiar `InMemoryStore` como DB.
- **FMEA:** `index` ignorado → PII al proveedor (detectar llamadas del embedder con campos centinela; mitigar validando antes); batch divergente → contexto viejo/nuevo inesperado (detectar con secuencia mixta y declarar el modelo).
- **Criterios de aceptación propuestos:** `index=False`/lista, fallo de embedding, dos writers y put/get/delete conservan el contrato declarado; logs por IDs y política, no texto.
- **Criterio negativo:** cancelar el embedding antes del put no crea valor; un batch parcialmente aplicado se informa, no se «deshace» inventando transacción.
- **Independencia:** parcial — requiere las capacidades declaradas de X-LGR-04 y la semántica de versión de X-LGR-01; X-LGR-02 no sustituye este contrato de memoria.
- **No objetivos:** otro ANN, TTL nuevo ni runtime.

### X-LGR-06 — Borrado de thread con alcance y cierre verificables

- **Problema:** retirar una conversación no debe dejar writes reactivables ni borrar memorias compartidas legítimamente por otros threads; checkpoints y writes son familias distintas, y retirar un hecho de `BaseStore` no sanea checkpoints anteriores.
- **Contrato hipotético (tipos propuestos):** `ThreadDeletion { scope, thread, generation, families } -> DeletionReceipt { complete, remaining, blocked_writers }`; errores `Unauthorized | GenerationConflict | Partial`.
- **Fuente de evidencia (clone):** `libs/checkpoint-sqlite/langgraph/checkpoint/sqlite/__init__.py:458-515` (`SqliteSaver.delete_thread:498-515`) [L12]; adapter `integrations/langchain/vantadb_langchain/checkpointer.py:298-313` (`delete_thread` pagina y borra ambos namespaces por thread; no toca `BaseStore`, trazas, archivos ni proveedor — reiniciar/cancelar puede dejar un subconjunto, no declara transacción global) [V6].
- **Seam VantaDB:** `integrations/langchain/vantadb_langchain/checkpointer.py:298-313` + lifecycle/provenance de `vanta-memory` — mejorar el ciclo de vida del saver existente sin convertir `delete_thread` en borrado global.
- **Acciones:**
  1. Autenticar (principal fuera del LLM).
  2. Cerrar la generación o serializar con writers activos.
  3. Enumerar las familias (checkpoints y writes son distintas).
  4. Borrar reanudablemente.
  5. Verificar ausencia.
  6. Emitir el recibo `complete/remaining/blocked_writers`.
- **Dependencias:** X-LGR-02 define las keys; X-LGR-07 el linaje; retención de auditoría/recibos es política aparte; no borrar `BaseStore` sin referencias y permiso explícitos; backups/exportaciones tienen su propio lifecycle. Converge con C6/C13 de `CONSOLIDATED.md`. Alternativa: bloqueo de thread en host y procedimiento de mantenimiento si no existe primitiva transaccional.
- **FMEA:** paginar mientras se borra o sigue escribiendo → restos (reescaneo y generation guard); thread ID ajeno → borrado cruzado (validar principal fuera del LLM); error parcial nunca es borrado completo.
- **Criterios de aceptación propuestos:** más de una página, crash/cancelación y writer tardío no dejan el thread reanudable tras recibo completo; otro thread/store intacto; métricas de familias/IDs, no payload. Caso LG-1: fuente retirada/permiso revocado con checkpoint previo retenido — aunque el thread no se borre por política de auditoría, su recall/entrega no reinyecta contenido revocado; denegar resume o redactar explícitamente según host (no es una fuga reproducida).
- **Criterio negativo:** un error parcial nunca se reporta como borrado completo; `delete_thread` no se convierte en borrado global de fuentes/backups/BaseStore.
- **Independencia:** parcial — usa keys de X-LGR-02 y linaje de X-LGR-07; versión aditiva del recibo con progreso reanudable; desactivar no restaura contenidos borrados.
- **No objetivos:** borrado certificado de todos los servicios externos ni versionado nuevo de Pregel.

### X-LGR-07 — Vista de linaje de checkpoint y modo de durabilidad

- **Problema:** un operador debe distinguir reanudar el último estado, replay desde un padre y una nueva rama; un modo `exit` rotulado como durable por superstep daría falsa recuperación y un checkpoint previo a revocación no debe reinyectar contenido retirado.
- **Contrato hipotético (tipos propuestos):** `CheckpointView { thread, namespace, checkpoint, parent, graph_revision, codec_revision, durability_mode, pending_task_ids }`, con «desconocido» explícito donde falte dato.
- **Fuente de evidencia (clone):** `libs/checkpoint/langgraph/checkpoint/base/__init__.py:93-360` (`CheckpointTuple` devuelve checkpoint, padre y pending writes) [L1]; `libs/langgraph/langgraph/pregel/_loop.py:1300-1360` (modos de persistencia en el loop) [L4]; adapter `integrations/langchain/vantadb_langchain/checkpointer.py:41-350` (parent guardado por `put`) [V6].
- **Seam VantaDB:** `integrations/langchain/vantadb_langchain/checkpointer.py` — exponer al adapter/Studio lo ya persistido, no implementar time travel en Rust; UI lectora por defecto que nunca despacha tools.
- **Acciones:**
  1. Documentar los campos disponibles.
  2. Registrar las versiones del host/mode.
  3. Verificar compatibilidad antes de resume.
  4. Consultar el linaje paginado con ACL.
  5. Mostrar separado de hechos/decisiones/recibos.
- **Dependencias:** campos disponibles documentados; revisiones de host/mode registradas; X-LGR-02/X-LGR-03 aportan identidades; no exigir embedding. Converge con C14/C4/C16 de `CONSOLIDATED.md`. Alternativa: inspector de metadatos en Python mientras Rust conserva envelopes opacos.
- **FMEA:** modo `exit` rotulado durable por superstep → falsa recuperación (detectar el modo efectivo); reducer/código cambiado → replay divergente (detectar revisión incompatible y pedir migración explícita); checkpoint anterior conserva copia de hecho retirado → rehidratación no autorizada (revalidar en host, no tratar el checkpoint como fuente factual).
- **Criterios de aceptación propuestos:** padre/namespace y rama visibles sin revelar payload ajeno; revisión desconocida no afirma replay equivalente; reanudar un padre previo a revocación/retirada prueba la política LG-1: acceso antes de decode y filtro/redacción/denegación antes de recall, nueva entrega o efecto, incluso si grafo/codec son compatibles.
- **Criterio negativo:** seleccionar un checkpoint nunca despacha tools ni afirma equivalencia de replay con revisión desconocida; migración aditiva no inventa revisión de grafo histórica.
- **Independencia:** sí como vista de inspección — el host decide rama/durability y confirma snapshots; cancelar una consulta no cancela el workflow; rollback solo oculta la vista nueva.
- **No objetivos:** determinismo de LLM, undo remoto ni duplicar Studio.
