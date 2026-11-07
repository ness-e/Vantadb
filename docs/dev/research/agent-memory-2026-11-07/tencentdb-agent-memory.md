---
title: "TencentDB Agent Memory: captura incremental y gobierno de activos"
kind: research
status: proposed
description: "Análisis estático de captura, checkpoints, gobierno y fronteras del hub TencentDB para ideas independientes en Rust"
tags: [research, memory, engineering]
---

# TencentDB Agent Memory: captura incremental y gobierno de activos

## Identidad y dictamen

- Repositorio `TencentCloud/TencentDB-Agent-Memory`, clon
  `C:\Users\Eros\TencentDB-Agent-Memory`.
- SHA `2002196f2d7c22ea267cfcaf67c5a6fa861df681`; rama **`feat/server_team`**,
  no confundirla con una release estable.
- Snapshot congelado: **30 589 estrellas**, licencia detectada **NOASSERTION**
  en [candidatos](github-candidates.json). No se alteró esa captura.
- `LICENSE:1-27` local contiene concesión MIT y copyright Tencent 2026 [T0].
  La diferencia es entre detector y texto, no permiso para inventar otro snapshot.
- [Manifest](clone-manifest.json) y [METHOD](METHOD.md) fijan el alcance offline.
- Vanta: base `c561158`, HEAD comunicado `167435a9e`; no reauditoría del motor.

**Dictamen:** no migrar el hub entero. Interesan la frontera captura/cursor y
el modelado explícito de compartir/importar activos. Vanta ya tiene las capas
de memoria, ingestors, scheduler, checkpoints, dedup, provenance y Studio.
El incremento útil es contractual, no reconstruir el pipeline en Rust.

## 1. Arquitectura, tecnologías, flujo y recomendaciones

**Documentación:** README presenta MemoryCore, hub/panel y proxy para compartir
memoria entre agentes; distingue activos Chat Memory, Skill, Wiki y Code-Graph.
La muestra de código acredita piezas, no todos los claims del producto.

**Código/manifiestos:** TypeScript ESM. MemoryCore declara pipeline L0→L1→L2→L3,
build con tsdown y pruebas Vitest [T1]. MemoryKnowledge es servicio independiente
con Hono, MCP, Drizzle/SQLite, AI SDK y dependencia `@colbymchenry/codegraph`
`^1.2.0` [T2]. No asumir que la dependencia resuelta corresponde al SHA del
clon CodeGraph estudiado en otra ficha.

Flujo de captura observado [T3][T4]:

1. `performAutoCapture` crea `CheckpointManager` y llama `captureAtomically`.
2. El callback recibe cursor de sesión, registra conversaciones y devuelve
   máximo timestamp/cantidad; sin mensajes devuelve `null`.
3. `captureAtomically` ejecuta callback dentro de `mutate`; solo con resultado
   avanza cursor y contadores. Puede usar un suelo temporal de arranque.
4. Indexación vectorial es una etapa posterior: hay capacidad de embedding
   diferido o camino síncrono, no una única transacción con L0.
5. Un registro opcional de promesas permite al propietario esperar trabajo
   pendiente. La notificación posterior al scheduler entrega array vacío:
   el runner debe leer storage, no depender del buffer de esta llamada.

No se inspeccionó todo `mutate` ni su protocolo de persistencia/locks.
El nombre `captureAtomically` no demuestra atomicidad crash-safe entre
JSONL, checkpoint, índice textual/vectorial y extracción LLM.

**Gobierno:** `ChatMemoryAgentRel` guarda compartir con equipo e IDs importados;
valida pertenencia al equipo, no autoimportar, no duplicados y máximo dos [T5].
El comentario reconoce almacenamiento provisional en `metadata_json`,
no columna dedicada. Lectura inválida cae al default de compartir `true`.
Es contrato específico de chat, no abstracción universal de todas las assets.

**Code-Graph:** el bridge carga paquete CodeGraph y su `ToolHandler`, resuelve
un módulo interno por plataforma y abre/indexa el proyecto [T6].
Es integración, no una implementación independiente del motor AST de Tencent.
La portabilidad del bridge depende también de empaquetado y exports upstream.

**Recomendación:** reutilizar scheduler y registros Vanta; explicitar qué
significa «capturado», «indexado» y «derivado disponible». Compartir contenido
debe ser política autorizada del backend, no inferencia de un toggle UI.

## 2. Unidades mínimas para reimplementación independiente

### Unidad A — contrato de captura con cursor y visibilidad por etapa

- **Problema:** confirmar captura puede confundirse con que toda indexación
  y extracción haya terminado, especialmente con embeddings diferidos.
- **Unidad fuente:** callback de `captureAtomically:741-768` [T4];
  resultados y separación de etapas de `performAutoCapture` [T3].
- **Seam Vanta:** `vanta-memory/src/core/conversation/l0_recorder.rs`,
  checkpoints/scheduler existentes y `vanta-memory/src/ingest/merge.rs`.
- **DTO HIPOTÉTICO Rust:** `CaptureReceipt { source, session, cursor,
  durable_revision, lexical_state, embedding_state, derivation_state }`.
  No es un tipo acreditado del SDK ni una promesa de durabilidad actual.
- **Interfaz HIPOTÉTICA:** `capture_batch(ScopedBatch) -> CaptureReceipt`;
  un puerto de publicación posterior consume la revisión confirmada.
- **Acciones:** mapear confirmación actual; definir cursor estable frente a
  mensajes con timestamps iguales; usar commit/checkpoint existente;
  separar disponibilidad de evidencia e índice; registrar fallos recuperables;
  permitir al scheduler reintentar sobre IDs, no sobre buffers efímeros.
- **Dependencias/relaciones:** fuente/provenance y dedup ya presentes;
  cierre ordenado y cancelación para tareas diferidas; no crear segundo
  scheduler ni otro archivo de checkpoints por similitud upstream.
- **FMEA:** cursor avanza antes de write durable; mismo timestamp omite
  mensajes; retry duplica; cold-start floor pierde backlog deseado;
  cierre destruye índice mientras embedding sigue activo.
- **Aceptación funcional propuesta:** confirmación distingue persistencia de
  derivados; retry converge a las mismas evidencias; fallo de embeddings
  conserva recuperación textual; interrupción no hace aparecer un lote
  incompleto como íntegramente procesado; shutdown identifica trabajo pendiente.
- **No objetivo:** copiar IDs aleatorios o el suelo de arranque como política
  obligatoria para todos los ingestors Vanta.

### Unidad B — relación de préstamo con autorización y revisión

- **Problema:** permitir a un agente consultar activos de otro sin confundir
  pertenencia al equipo, visibilidad y equipamiento/importación.
- **Unidad fuente:** DTO y `validateImportedAgents`, lectura/escritura
  normalizada de relaciones en [T5].
- **Seam Vanta:** frontera de scopes de memoria/escenas/skills y adaptadores
  server/MCP; UI en Studio `develop-desktop`, no en el motor de índices.
- **DTO HIPOTÉTICO:** `AssetGrant { asset_id, owner_scope, reader_scope,
  asset_kind, revision, decision_source }`. `BorrowRequest` expresa actor,
  objetivo y revisión esperada. No se propone el límite de dos como universal.
- **Interfaz HIPOTÉTICA:** `authorize_borrow(request) -> Result<Grant,
  PolicyError>`; backend decide, UI solo muestra la decisión.
- **Acciones:** inventariar autorización actual; distinguir ver, usar y editar;
  validar pertenencia con identidad confiable; persistir cambios con revisión;
  aplicar el grant antes de búsqueda/expansión; revocar caches y derivados
  al retirar acceso; dejar auditoría explicable.
- **Dependencias:** catálogo de activos, identidad de actor, provenance,
  revisión y revocación. La Unidad A puede informar disponibilidad, no permisos.
- **FMEA:** default compartido abre datos; metadata corrupta habilita permisos;
  equipo cambia durante préstamo; UI oculta activo pero retrieval lo entrega;
  snapshot/cache conserva acceso revocado.
- **Aceptación propuesta:** actor de otro equipo no obtiene contenido;
  configuración inválida no amplía acceso; autoimportación y duplicados se
  rechazan; revocación afecta recall y preview; revisiones en conflicto no
  pisan concesiones; error explica política sin divulgar activos ajenos.
- **No objetivo:** trasladar el modelo provisional en JSON ni una plataforma
  de identidad/administración empresarial completa.

## 3. Defectos, riesgos e issues con evidencia diferenciada

- **Código:** `readChatMemoryRel` devuelve default con sharing `true` ante
  ausencia/JSON inválido ([T5], líneas 25-28,61-74). **Riesgo condicionado:** sería
  fail-open si esa relación fuese autoridad de acceso. No se auditó el
  enforcement de rutas ni se afirma una fuga reproducida.
- **Código:** captura atrapa fallo de L0 y continúa a etapas posteriores;
  scheduler puede ser notificado incluso sin mensajes ([T3], líneas 143-145,301-326).
  No equivale a pérdida probada: faltan contratos de caller, storage y retry.
- **Código:** ID de L0 vectorial incorpora reloj/aleatoriedad ([T3], líneas 42-44).
  No acredita idempotencia de reenvío; tampoco invalida la deduplicación
  que pueda existir en otra capa.
- **Código:** `resolveToolsPath` depende de un path interno por plataforma
  ([T6], líneas 25-38). **Hipótesis:** futuras versiones empaquetadas pueden romperlo
  aunque el API principal siga siendo compatible.
- **Defecto estático acotado:** el runner token importa
  `../src/offload/fast-token-estimate.ts` desde `src/offload/`, que resuelve
  a `src/src/offload/...`, ruta no presente ([T8], línea 4). No se ejecutó.
- **Issues:** no consultados; no se adjudican números ni estado vigente.

## 4. Prerrequisitos Vanta

Revalidar contratos existentes antes de abrir implementación: persistencia L0,
semántica de checkpoints y publicación de derivados; identidad de equipo/actor
si realmente se necesita colaboración. Usar políticas explícitas de retención,
borrado y revocación. Mantener funciones puras de dominio separadas de HTTP,
proveedores LLM y drivers. No suponer que un hub multiagente exige varios servicios.

## 5. Qué no trasladar

No portar MemoryCore/Knowledge/Panel/Proxy juntos ni acoplar Vanta a OpenClaw,
Redis, COS, Tencent VectorDB o MongoDB. No duplicar BM25/HNSW/RRF.
No copiar `metadata_json` provisional como fuente de permisos.
No convertir «compartir true» en default del motor embebido.
No importar rutas internas de CodeGraph ni el protocolo de despliegue como
requisito para una unidad de memoria.

## 6. Método, licencia y cobertura

NOASSERTION congelado se conserva; MIT local es evidencia adicional [T0].
Revisar notices/archivos/dependencias antes de cualquier derivación:
Rust no elimina obligaciones ni hace independiente una traducción literal.
Clon de una rama superficial, sin historial completo, tags ni servicios.
No auditoría exhaustiva de auth, almacenamientos, wiki, proxy, borrado o cuotas.

## 7. Runners upstream, datos, métricas y efectos

Se leyó **`scripts/bench-checkpoint-lock.ts:1-212`** [T7]:
usa `CheckpointManager` real; entradas sintéticas `bench-sess-*` y preload,
no LoCoMo ni LongMemEval. Cronometra `markL1ExtractionComplete`, calcula QPS y
p50/p95/p99; escenarios de concurrencia y tamaño están definidos upstream.
Depende de Node/TypeScript y, con `REDIS_HOST`, `ioredis`/Redis real.
El cuerpo crea directorio temporal y pasa storage `undefined`: el comentario
sobre COS no acredita que este runner mida COS.

Efectos: crea/elimina directorio `cp-bench-*`, escribe checkpoints, crea locks
Redis con TTL y libera por owner mediante Lua; `finally` elimina directorio y
cierra Redis. No ejecutado. Es contención de checkpoint en una ejecución,
no prueba de escritores en procesos distintos, fencing ni calidad de memoria.
La conclusión de capacidad L1 usa supuestos comentados, no medición end-to-end.

También se leyó **`src/offload/benchmark-token-estimate.ts:1-89`** [T8]:
compara estimación con `js-tiktoken` `cl100k_base`, toma archivos multilingües
si existen y agrega escenarios JSON/código sintéticos. Calcula error relativo,
tiempos y speedup; el total es diferencia de sumas, no error absoluto medio
por documento. No acredita presupuesto para cualquier tokenizer/modelo.
Efectos visibles: lectura de corpus y salida a consola; hay un problema de
ruta de importación descrito arriba.

No se diseñaron ni modificaron prompts, métricas, umbrales, sampling o
configuraciones. Otros runners e2e/Mongo se localizaron, **no se analizaron**.
Claims de memoria del README no quedan certificados por estos microbenchmarks.
Comparación futura solo según [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz frente a VantaDB

| Capacidad | Estado usado | Contraste útil |
|---|---|---|
| Captura/sesiones/provenance/dedup | Existe | recibo por etapa |
| Scheduler/checkpoints/ingestors | Existe | cursor/publicación y cierre |
| BM25/HNSW/RRF | Existe | no otro índice por portabilidad |
| Escenas/personas/skills | Existe | no importar toda la ontología |
| Pipeline con estados visibles entre capas | Parcial para este contraste | existen piezas; composición no certificada aquí |
| Préstamo gobernado de assets entre equipos | No acreditado | requisito primero, no bug supuesto |
| Tencent COS/Redis como requisito | No aplicable | elección de despliegue upstream |
| Calidad de captura/recall del producto | No acreditado | runners leídos no la miden |

## 9. Deuda sugerida, no defectos actuales de Vanta

Preguntar si una confirmación actual distingue raw/indexado/derivado; si hay
eventos estables frente a timestamps iguales; si shutdown drena trabajo
de ingestión; y si permisos/revocación se aplican también al contexto derivado.
Son verificaciones futuras, no afirmaciones de fallos del HEAD `167435a9e`.
Evitar nuevas filas si ya hay tareas que cubren estas garantías.

## 10. UI/escritorio y Studio

**Código real:** `MemoryPanel/web/src/App.tsx:1-70` usa React, RouterProvider,
store de auth, LoginGate e i18n [T9]. `ChatMemoryPage/index.tsx:1-10` monta
`ChatMemoryPanel` dentro de `ResourcePage` [T10]; no solo capturas de marketing.
No se auditó su aplicación de permisos ni todas sus acciones.
No se localizó un empaquetado desktop propio en la muestra; el panel es web.

Studio Tauri/React ya tiene memoria, actividad, escenas, skills y diagnóstico.
Aplicar allí estados por etapa y explicación de grants/revocación cuando
existan contratos backend, no crear otro panel ni copiar su stack visual.

### Ledger de archivos/rangos efectivamente leídos

| Ref. | Ruta/rango | Superficie |
|---|---|---|
| T0 | `LICENSE:1-27` | MIT local frente a detector |
| T1 | `MemoryCore/package.json:1-70` | pipeline declarado y scripts |
| T2 | `MemoryKnowledge/package.json:1-80` | stack y dependencia CodeGraph |
| T3 | `MemoryCore/src/core/hooks/auto-capture.ts:1-180,285-347` | captura/capacidades/scheduler |
| T4 | `MemoryCore/src/utils/checkpoint.ts:735-768` | callback y avance del cursor |
| T5 | `MemoryPanel/src/panel/domain/chat-memory-governance.ts:1-107` | modelo/normalización |
| T6 | `MemoryKnowledge/src/engines/code/bridge.ts:1-140` | carga y apertura del índice |
| T7 | `MemoryCore/scripts/bench-checkpoint-lock.ts:1-212` | runner completo |
| T8 | `MemoryCore/src/offload/benchmark-token-estimate.ts:1-89` | estimación de tokens |
| T9 | `MemoryPanel/web/src/App.tsx:1-70` | shell UI |
| T10 | `MemoryPanel/web/src/pages/ChatMemoryPage/index.tsx:1-10` | composición de página |

README raíz leído `1-90`; listado de rutas no equivale a lectura de cuerpos.
Pendientes: enforcement auth/equipos, implementación completa de `mutate`,
storage/cleanup, calidad del pipeline y ejecución manual de runners.
Sin scripts de proyectos, builds, tests, apps, instalaciones, servicios, red,
benchmarks, validadores documentales, commits ni push.

[T0]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/LICENSE#L1-L27
[T1]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/package.json#L1-L70
[T2]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryKnowledge/package.json#L1-L80
[T3]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/core/hooks/auto-capture.ts#L1-L347
[T4]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/utils/checkpoint.ts#L735-L768
[T5]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryPanel/src/panel/domain/chat-memory-governance.ts#L1-L107
[T6]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryKnowledge/src/engines/code/bridge.ts#L1-L140
[T7]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/scripts/bench-checkpoint-lock.ts#L1-L212
[T8]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryCore/src/offload/benchmark-token-estimate.ts#L1-L89
[T9]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryPanel/web/src/App.tsx#L1-L70
[T10]: https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/2002196f2d7c22ea267cfcaf67c5a6fa861df681/MemoryPanel/web/src/pages/ChatMemoryPage/index.tsx#L1-L10
