---
title: "Mem0: extraccion aditiva, entidades y limites del cliente alojado"
kind: research
status: proposed
description: "Estudio estatico de Mem0 OSS y su frontera con Platform, con candidatos incrementales para VantaDB"
tags: [research, memory, engineering]
---

# Mem0: extraccion aditiva, entidades y limites del cliente alojado

**Contrato:** [METHOD.md](METHOD.md). Investigacion estatica, sin instalacion,
ejecucion de codigo del proyecto, tests, servicios ni benchmarks. Las fichas son
hipotesis de diseno, no decisiones aprobadas ni promesas de mejora.

| Identidad congelada | Valor |
|---|---|
| `full_name` | `mem0ai/mem0` |
| Estrellas | **68542**, snapshot [github-candidates.json](github-candidates.json); no recalculadas |
| Captura | `2026-11-07T06:56:00.4053409Z` |
| SHA local, `git rev-parse HEAD` | `f1791daa86d469249c9b002f62644d58a1eb96a0` |
| Clon | `C:\Users\Eros\mem0`; `git status --short` sin cambios observados |
| Licencia declarada | Apache-2.0; estudiar no autoriza copiar sin revisar obligaciones |

## Indice

- [1. Recomendaciones y arquitectura](#1-recomendaciones-y-arquitectura)
- [2. Partes extraibles y flujo real](#2-partes-extraibles-y-flujo-real)
- [3. Riesgos e incidencias](#3-riesgos-e-incidencias)
- [4. Prerrequisitos en VantaDB](#4-prerrequisitos-en-vantadb)
- [5. Que no trasladar](#5-que-no-trasladar)
- [6. Advertencias y cobertura](#6-advertencias-y-cobertura)
- [7. Benchmarks existentes](#7-benchmarks-existentes)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades revalidadas, hipótesis y falsos positivos](#9-debilidades-revalidadas-hipótesis-y-falsos-positivos)
- [10. Interfaz e inspeccion](#10-interfaz-e-inspeccion)

**Plan de particion si crece:** mantener aqui los diez ejes; separar en una
investigacion posterior las fichas y el ledger tecnico, sin crear ahora archivos
fuera de la propiedad asignada.

## 1. Recomendaciones y arquitectura

**Dictamen:** estudiar el contrato de extraccion aditiva y sus fronteras de fallo,
no sustituir el motor VantaDB por un wrapper Python. VantaDB ya tiene dedup,
supersession, procedencia, BM25/HNSW/RRF y `context_assemble`; el valor incremental
seria una politica explicita de conservar afirmaciones y explicar su seleccion.
No equiparar el algoritmo OSS observado con Mem0 Platform ni con versiones
historicas que usaban `graph_memory.py`.

```text
mem0ai/mem0
├── mem0/memory/main.py          Memory / AsyncMemory OSS
│   ├── LLM + embeddings + VectorStoreFactory
│   ├── almacen vectorial: texto, hash, metadata
│   ├── entity_store lazy: entidades -> memory_ids
│   └── SQLiteManager: history + ventana de mensajes
├── mem0/client/main.py          MemoryClient / AsyncMemoryClient -> API externa
├── mem0-ts/                     SDK y superficie OSS TypeScript, no paridad auditada
├── server/main.py              REST FastAPI self-hosted
│   └── dashboard/              interfaz Next.js
├── cli/python/, cli/node/      clientes operativos
├── integrations/               adaptadores de agentes
└── evaluation                  gitlink; contenido no descargado
```

**Stack y coste de dependencias:** `mem0ai` 2.2.1, Python >=3.10,<4,
Hatch, Pydantic, Qdrant client, OpenAI, httpx, PostHog, SQLAlchemy y protobuf
son dependencias base. spaCy es extra `nlp`; FAISS, pgvector, Chroma, Redis,
Pinecone, Milvus, servicios cloud y otros son opciones de `vector-stores`,
no todos requisitos de un despliegue.
Fuente: [pyproject.toml:1-77](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/pyproject.toml#L1-L77).
La fabrica importa proveedores dinamicamente:
[factory.py:29-84](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/utils/factory.py#L29-L84).
No se midieron tamano instalado, RAM ni latencia.

### Licencia por superficie y límite de reutilización

Lectura complementaria del checkout fijado, no nueva captura de metadata:

| Componente estudiado | Evidencia de licencia | Decisión |
|---|---|---|
| Python OSS, servidor y documentación del monorepo | [LICENSE raíz Apache-2.0][M-license] | Estudiar contratos; cualquier material derivado exige conservar licencia/avisos y revisar las condiciones aplicables |
| SDK TypeScript y dashboard | Mismo LICENSE raíz; manifests [mem0-ts/package.json][M-ts-package] y [dashboard/package.json][M-ui-package] identifican paquetes distintos | La licencia raíz no acredita paridad Python/TS ni incluye el servicio Platform |
| Integración Strands, no motor OSS | [LICENSE][M-strands-license] y [NOTICE][M-strands-notice] propios, Apache-2.0 y atribución Mem0 2026 | No omitir NOTICE si se reutiliza material de ese componente; su código operativo no fue auditado |
| Otras integraciones/skills | El inventario de archivos localiza licencias propias en OpenClaw, OpenCode, Hermes, pi, DeepSeek, n8n y Zapier | Solo inventario, no dictamen legal por cada paquete |
| Dependencia `memory-benchmarks` | Gitlink y fuentes de §7 | La licencia del padre no se hereda automáticamente al submódulo ni al dataset; términos de ambos pendientes |
| Platform, modelos y proveedores | Fuente backend privada no leída; dependencias de terceros | Acceso al SDK no concede derechos sobre SaaS, pesos, conversaciones ni servicios |

No se copió código a Vanta. Reescribir en Rust no elimina posibles obligaciones
de derivación. No hay una certificación SPDX exhaustiva de dependencias.

### Corte de evidencia y decisiones prioritarias

Esta ampliación reemplaza la conclusión superficial «Mem0 = CRUD con LLM».
La revisión local sigue fijada al SHA de la tabla; no se actualizó el clon.
La referencia propia se revalidó sobre VantaDB
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, no sobre `c561158/dirty`.
Los issues se consultaron el **2026-11-07** por la API pública oficial después
de que GitHub MCP fallara al conectar. Su estado es una observación remota,
no parte del checkout ni prueba de corrección.

| Orden propuesto | Decisión | Motivo y coste oculto |
|---|---|---|
| Primero | Recibo honesto de extracción y mutación | El fallback por registro recupera disponibilidad, pero puede ocultar cobertura parcial |
| Primero, si se toca mutación | Pertenencia inmutable y revisión al aplicar | Metadata controlada por usuario no debe definir derechos ni destino |
| Después | Política aditiva explícita | Conservar discrepancias evita sobreescritura prematura, pero exige selección temporal y control de crecimiento |
| En paralelo, lectura | Diagnóstico de candidatura | Saber por qué algo nunca entró al pool vale más que añadir otra fórmula de fusión |
| Condicional | Proyección de entidades reparable | Útil para alias; añade un segundo estado derivado y obligaciones de borrado |
| Condicional | Retención y purga verificable | Un DELETE de la memoria no elimina el texto viejo del historial |

No son seis features aprobadas: son seis unidades diferentes, relacionadas
con cuatro candidatas previas que aquí se desarrollan y dos que faltaban.
La prioridad no presume mejora de calidad o rendimiento.

### Modelo de datos: cinco recursos, no una única «memoria»

| Recurso | Identidad y contenido observado | Invariante y límite |
|---|---|---|
| Mensaje de entrada | `role`, `content`, `name`; ventana SQLite por `session_scope` | Contexto de extracción; no es checkpoint de ejecución del agente |
| Memoria vectorial | ID UUID, payload `data`, MD5 del texto, fechas, campos de identidad y metadata | El hash sirve a dedup local, no a autenticación ni a identidad global |
| Entidad | Texto normalizado/tipo y conjunto de `memory_ids` en colección auxiliar | Coocurrencia/enlace inverso, no relaciones tipadas Neo4j en este SHA |
| Evento de historia | UUID propio, `memory_id`, `old_memory`, `new_memory`, evento, actor/rol y marca de borrado | Es otra escritura, con retención de contenido previo |
| Memoria procedural | Resumen LLM con `memory_type=procedural`, almacenado como memoria | Síntesis de conversación; no programa ejecutable ni workflow reanudable |

Anclajes: [creación y procedural, main.py:1970-2048][M-create],
[historia y mensajes, storage.py:102-191][M-storage],
[actualización/borrado:2052-2144][M-mutate]. La ventana de mensajes incorpora
texto original aunque no se extraigan hechos (`main.py:988-991`).
Por tanto, inventariar solo vectores para privacidad sería insuficiente.
`source_message_ids` no aparece como contrato equivalente al registro propio
en las rutas leídas: no se inventa una procedencia por turno que el API no entregue.

`_build_session_scope` ordena las claves user/agent/run y escapa `%`, `&`, `=`
antes de concatenarlas (`main.py:407-419`). Esta identidad de ventana es distinta
del ID de memoria y distinta de un principal autorizado. Una misma cadena
de usuario suministrada por dos clientes no demuestra aislamiento entre ellos.

### Superficies y despliegue

- **Biblioteca OSS:** `Memory`/`AsyncMemory` coordinan proveedores, SQLite y
  vector store en el proceso de la aplicación. Cada backend conserva sus propias
  garantías; una interfaz común no crea atomicidad transversal.
- **REST self-hosted:** FastAPI envuelve la biblioteca; `verify_auth` protege
  las rutas examinadas y `require_admin` configuración/reset/borrado global
  (`server/main.py:322-548`). No confundir autenticación con ownership por ID.
- **Platform:** `MemoryClient` llama HTTP a otra implementación. Features de
  decay, perfiles o fecha de referencia documentadas allí no prueban ejecución
  OSS. No se estudió el código privado de ese servicio.
- **Dashboard y CLI:** son consumidores; no añaden garantías de commit. La
  paginación cliente no es una enumeración completa del almacén.

El constructor OSS crea proveedor LLM aun cuando la siguiente operación use
`infer=False`. «Sin inferencia» significa no llamar al extractor en esa ruta;
no garantiza instalación/configuración libre de proveedor. Una reimplementación
Rust debe conservar la separación entre storage puro y adaptadores opcionales.

### Revalidación propia: qué ya está implementado

Las siguientes referencias son inmutables al nuevo corte:

- [V1: MemoryRecord:71-129][V-record] ya tiene versión, fuentes de mensajes,
  sesiones, team/user/agent, vector y `superseded_by`.
- [V2: writer:389-456][V-write] planifica, elimina destinos, hace `put_batch`
  y después registra efectos. El comentario `430-433` declara fallo parcial:
  no debe venderse como delete+replace indivisible.
- [V3: RecallConfig:302-334][V-recall] ofrece `core_search`, **false por
  defecto**. No atribuir BM25/HNSW/RRF del motor a toda llamada legacy.
- [V4: assemble_with_recall:252-348][V-context] comparte presupuesto restante
  entre MMD y recall y deduplica bloques por marcador. Mem0 top-k no lo sustituye.
- [V5: checkpoint:79-116][V-checkpoint] conserva clave global y RMW separado.
  Esto acredita el patrón estático; una pérdida concurrente sigue sin reproducirse.

Estas observaciones no dependen de que los tests pasen: no se ejecutó ninguno.
La presencia de `version` tampoco demuestra que se use como precondición atómica.

## 2. Partes extraibles y flujo real

### Ingesta, recall candidato y persistencia

1. `add` normaliza mensajes y construye metadata/filtros por `user_id`,
   `agent_id`, `run_id`. El helper elimina identidades de metadata libre y exige
   al menos un identificador:
   [main.py:314-404](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L314-L404).
2. Con `infer=False`, guarda cada mensaje no-system con embedding, sin llamada
   de extraccion. **No equivale a constructor libre de LLM:** `Memory.__init__`
   crea siempre el proveedor LLM, embedding y SQLite:
   [main.py:488-510](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L488-L510).
3. Con inferencia, lee diez mensajes previos del scope, recupera diez memorias
   por similitud, reemplaza UUID por indices locales para el prompt y ejecuta
   una extraccion JSON **ADD-only**. No es el antiguo resolver
   ADD/UPDATE/DELETE:
   [main.py:881-991](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L881-L991).
4. Embeddings batch con fallback individual; dedup MD5 contra los candidatos
   recuperados y el propio batch, no prueba de unicidad global. Persiste batch,
   reintenta individualmente y escribe history solo para registros confirmados:
   [main.py:993-1098](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L993-L1098).
5. Extrae entidades NLP en batch y mantiene enlaces de entidades a IDs. La
   extraccion devuelve vacio si spaCy no esta disponible:
   [entity_extraction.py:1-30](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/utils/entity_extraction.py#L1-L30).

### Actualizacion, borrado e historia

`update` por ID modifica texto/metadata, conserva identidad, reembebe y registra
old/new; `delete` borra el vector y conserva old text en el evento DELETE.
La limpieza de entidades es posterior y no fatal:
[main.py:2052-2144](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L2052-L2144).
`delete_all` exige scope y vuelve a listar por batches; detecta batches repetidos
y se detiene. `history(memory_id)` consulta SQLite, no reevalua ownership:
[main.py:1904-1973](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L1904-L1973).
El esquema history guarda ID, old/new, evento, fechas, actor y role, pero no
columnas tenant/user/agent/run:
[storage.py:102-127](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/storage.py#L102-L127).
Transacciones y mutex protegen SQLite individualmente, no una transaccion
distribuida vector/history/entity:
[storage.py:150-255](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/storage.py#L150-L255).

### Recuperacion y grafo: distincion decisiva

`search` exige identidad en `filters`, procesa operadores y pasa el filtro a
busqueda semantica y keyword. Obtiene hasta `max(4*k,60)` candidatos semanticos;
BM25 y entidades **puntuan esa lista**, no unen candidatos exclusivamente
lexicales. Excluye expirados salvo opcion expresa, agrega scores y permite
reranker con fallback:
[main.py:1642-1745](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L1642-L1745).
La formula es aditiva normalizada, con gate semantico antes de combinar; no RRF:
[scoring.py:60-139](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/utils/scoring.py#L60-L139).
El resultado son memorias, metadata y `score_details` opcional; no hay ensamblado
global de contexto/token budget acreditado en esa ruta.

En este SHA `MemoryConfig` no declara `graph_store` ni `enable_graph`:
[base.py:29-57](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/configs/base.py#L29-L57).
La busqueda acotada de archivos no localizo implementacion Python vigente de
`graph_memory.py`. Si existe grafo de entidades, es un **estado derivado
independiente** del vector principal, alojado mediante la fabrica vectorial, no
una transaccion Neo4j ni aristas tipadas. La documentacion Platform tambien
describe coocurrencia y boost, no relaciones tipadas:
[graph-memory.mdx:25-66](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/docs/platform/features/graph-memory.mdx#L25-L66).

`MemoryClient` exige API key, valida contra host externo y envia `add` a
`/v3/memories/add/`; esas rutas no ejecutan `Memory` local:
[client/main.py:240-295](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/client/main.py#L240-L295),
[321-361](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/client/main.py#L321-L361).
Filtros, perfiles, exportaciones y gobernanza del cliente no demuestran su
implementacion backend OSS. `reference_date` se rechaza en `Memory.search` OSS.

### Ficha A — política de conservación separada de la extracción

**Problema.** Un agente importa una sesión antigua que contradice una preferencia
actual. Extraer un hecho no autoriza borrar el actual. La unidad de Mem0 es el
paso ADD-only entre JSON y planificación, no su prompt ni todo `Memory`.
Fuente: [main.py:918-1098][M-add]. El código no ejecuta un resolver
UPDATE/DELETE en ese paso; la API explícita de actualización sigue existiendo.

**Delta y alternativas.** Vanta ya tiene `DedupAction::{Store,Update,Merge,Skip}`
([V1][V-record]) y contradicciones/supersession. Extender esa decisión con una
política conservadora es menor que crear un extractor independiente.
Un adapter que siempre emita Store es alternativa válida para una prueba de
dominio, pero no resuelve dedup literal ni selección temporal. Rechazar la
política aditiva universal evita crecimiento incontrolado.

**Contrato Rust hipotético.** `RetentionPolicy`, `FactProposal` y
`RetentionDecision` serían nombres nuevos: entrada scope autorizado, fuente
con revisión, afirmación y tiempo observado; salida Store/LinkDuplicate/
ProposeSupersession con razón. Identidad de hecho independiente de hash textual.
Errores diferenciados de fuente inválida, ámbito incompatible y conflicto de
revisión. Invariantes: política aditiva no elimina antecedentes; repetir una
operación identificada no duplica el efecto; contradicción no equivale a duplicado.

**Integración y fases.** (1) Comparar con `L1DedupConfig` y decisiones existentes;
(2) introducir la opción en dominio, no en ANN; (3) extender `plan_write`/
`apply_dedup_batch` [V2][V-write] y generation log; (4) verificar retrieval de
hechos coexistentes; (5) habilitar por scope explícito. Persistir versión de
política y motivo, con lectura compatible de registros anteriores. No exige
reindexar vectores si no cambia texto; cambiar política no reescribe el pasado.
Rollback: dejar de producir decisiones nuevas y seguir leyendo los registros.
Depende de C/D para recibos y pertenencia; no de E ni de una UI nueva.

**FMEA y aceptación.** Importación fuera de orden → hecho viejo parece vigente:
detectar tiempos de evidencia frente a ingesta, conservar ambos sin promover
automáticamente. Retry concurrente → duplicado: detectar operation ID y revisión,
resolver en frontera de commit, no con top-10. Crecimiento → contexto saturado:
observar cardinalidad y descartes del assembler existente; no inventar TTL.
Aceptación futura: repetición literal, contradicción fechada, dos scopes y
reintento concurrente conservan trazabilidad y no borran evidencia. Métricas por
acción/razón, sin texto por defecto. Mínimo: política de escritura; no decidir
ahora cuál es default ni construir razonamiento temporal completo.

### Ficha B — diagnóstico de admisión al pool, no solo score del ganador

**Problema y fuente.** Un nombre exacto aparece en keyword pero no entre vecinos
semánticos. Mem0 lo puntúa solo si también está en el pool semántico:
[main.py:1642-1745][M-search], [scoring.py:60-139][M-score].
`score_details` explica contribuciones del candidato, no todos los descartes.
La unidad extraíble es separar «encontrado», «admitido», «fusionado», «entregado».

**Delta/alternativas.** Vanta y Studio ya muestran explain. Extender la traza
existente es preferible a otro dashboard o al ranking aditivo de Mem0. Una
traza solo agregada protege privacidad pero no permite depurar un ID autorizado;
proponer ambos niveles bajo el permiso existente, no un endpoint abierto.

**Contrato hipotético.** `CandidateTrace { query_id, path, candidate_id,
revision, arm, rank, admission, contribution }` junto a resumen de truncamientos.
Entrada usa la misma consulta/ámbito del recall; salida no vuelve a ejecutar
otra búsqueda. Errores de rama no se codifican como lista vacía. IDs no autorizados
no aparecen ni siquiera como descartados; score no se etiqueta probabilidad.

**Seam/fases.** Instrumentar `perform_auto_recall_scored` [V3][V-recall] y la
fusión ya usada, distinguiendo `core_search` de legacy. Reutilizar explain del
SDK y `RetrievalLens.runSearch` (§10). Primero inventario de campos, después
captura opt-in, serialización compatible y visualización. No persistir payload
en la traza; retention explícita si se guarda. Sin reindexación ni migración de
memorias. Rollback: desactivar captura, no alterar candidatos/ranking.

**FMEA/aceptación.** Instrumentación cambia orden → diagnóstico deja de observar:
comparar IDs/orden con captura apagada. Rama no disponible presentada como cero
hits → falsa conclusión: exponer estado y error tipado. Trace cross-tenant →
fuga: autorización antes de emitir IDs. Aceptación: un hit por dos ramas tiene
dos contribuciones y una identidad; candidato lexical excluido se explica;
fallo, expirado y truncado son distinguibles. Observabilidad: cardinalidades y
razones, latencias de fase declaradas; sin pesos ni thresholds nuevos.
Mínimo read-only; no es una optimización de calidad acreditada.

### Ficha C — resultado de extracción y recibo de publicación parcial

**Problema/fuente.** Mem0 propaga `LLMError` para transporte, pero convierte
parseo fallido en extracción vacía (`main.py:957-991`). Tras persistir, historia
puede fallar y quedar solo registrada en logs (`1047-1098`). La unidad es un
resultado por frontera, no añadir SQLite a Vanta.

**Delta y alternativas.** `write_memory` llama persistencia y luego
`log_and_mark`; el batch propio hace deletes antes del put [V2][V-write].
Primero preservar resultados ya disponibles. Un booleano «ok» es insuficiente;
una transacción del motor puede resolver un subconjunto, pero no una llamada
LLM ni todos los efectos externos. No duplicar WAL con un coordinador genérico.

**Contrato propuesto.** `ExtractionOutcome::{NoFacts,Proposals,InvalidOutput,
ProviderFailed}` y `MutationReceipt { operation_id, source_revision,
primary: CommitState, projections }`. `CommitState` necesita distinguir aplicado,
no aplicado y resultado desconocido. `ProjectionStatus` identifica generación
esperada/confirmada y motivo. Retry conserva operation ID; el recibo no certifica
durabilidad en disco más fuerte que el storage usado.

**Seam y fases.** Mapear extractor, `L1Error`, writer y
`core/memory_generation_log`; extender resultado del job en
`services/pipeline_worker.rs` sin una cola paralela. Orden: inventario de efectos,
resultado tipado, identidad idempotente, reconciliación de proyección y UI.
Resolver indivisibilidad del reemplazo antes de prometer rollback de deletes.
Persistir estados añadidos de forma compatible y poder reanudar por operación.
Rollback de despliegue deshabilita productores nuevos, no borra operaciones
parciales; requieren reparación o compensación explícita.

**FMEA/aceptación.** Timeout después del commit → retry duplicado: detectar por
operation ID/lectura reconciliadora. Error de auditoría → primary ya existe:
marcar parcial, no reejecutar ciegamente extracción. JSON inválido → «no hechos»:
estado inválido diferenciado. Aceptación futura: fallo antes/después de cada
frontera deja recibo honesto; mismo retry no duplica memoria; cancelación conserva
lo confirmado. Observabilidad sin contenido: operación, fase, conteos,
revision y clase de error. Mínimo writer/resultado; no exactly-once global.

### Ficha D — pertenencia inmutable en creación y operaciones por ID

**Problema/fuente.** El issue #6655 muestra que reponer solo los campos recibidos
no elimina identidades inyectadas en campos omitidos. El SHA actual usa
`_strip_identity_keys` antes de fijar scope (`main.py:360-404`), y también en update.
La unidad es ese invariante, no un nuevo IAM.

**Delta/alternativas.** [V1][V-record] ya tiene team/user/agent/sesión.
Campos presentes no prueban que metadata libre ni todos los transports estén
gobernados. Extender validación en la frontera existente es preferible a
reconstruir scope desde cada adaptador. Un filtro de consulta no sustituye
autorización de get/update/delete/history por ID.

**Contrato hipotético.** `AuthorizedScope` solo construido por capa gobernada;
`ScopedMemoryId { scope, id }`; metadata de aplicación separada de identidad
reservada. Entrada de mutación: ID, revisión esperada y autoridad; salida
MutationReceipt de C. `NotFound/NotAuthorized/Conflict` se exponen según política
anti-enumeración. El modelo LLM nunca elige tenant ni obtiene permisos.

**Pasos y migración.** Inventariar rutas SDK/MCP/HTTP y jobs; clasificar claves
reservadas; rechazar o sanear según contrato explícito; validar pertenencia al
resolver ID y al commit; propagar scope inmutable al job. Registros legacy sin
identidad requieren política de compatibilidad antes de enforcement estricto,
no asignación silenciosa al usuario actual. Rollback de UI no elimina controles
de backend; no reabrir accesos por una flag de diagnóstico.

**FMEA/aceptación.** Campo omitido conserva metadata hostil → escritura en scope
ajeno: validar lista completa, no solo argumentos presentes. Scope cambia entre
validación y job → TOCTOU: snapshot autorizado y revalidación. Auditoría por ID
elude filtro → fuga: comprobar la misma pertenencia. Aceptación: matriz
crear/leer/update/delete/history con identidad cruzada, claves omitidas y
metadata anidada; no se exige negar búsquedas legítimas multi-scope. Contadores
de rechazo sin IDs ajenos. Mínimo validación transversal; no afirmar aislamiento
certificado antes de recorrer transportes y credenciales.

### Ficha E — índice auxiliar de entidades con ciclo de vida propio

**Problema/fuente.** La variante OSS enlaza entidades normalizadas con
`memory_ids`; el boost puede ayudar a alias repetidos sin inferir aristas
semánticas. [main.py:1100-1115][M-entity] y [update/delete][M-mutate] muestran
creación e invalidación. La extracción NLP puede devolver vacío sin spaCy.

**Delta/alternativas.** Vanta tiene grafo y búsqueda; no se propone «añadir grafo».
Comparar primero alias/metadatos existentes. Si una proyección inversa mejora
diagnóstico de identidad, materializarla en dominio; buscar entidades en cada
consulta sin persistencia evita desincronización pero repite trabajo.

**Contrato hipotético.** `EntityProjection { scope, normalized_entity,
extractor_version, members: [(memory_id,revision)] }`; entrada evento confirmado,
salida estado de proyección. Error NLP no es cero entidades certificado.
Un enlace a una revisión borrada nunca autoriza devolver su texto.

**Integración/fases.** Después de C/D, estudiar `l1_writer` post-commit y
auditoría existente: producir enlaces versionados, reconciliar y probar
invalidación antes de usar como señal. Cambiar extractor requiere reindexar
solo derivados, no cambiar IDs fuente. Doble lectura no es doble escritura
permanente: elegir generación activa, conservar rollback a la anterior o
desactivar boost. Nunca incorporar pesos de Mem0.

**FMEA/aceptación.** Homónimo normalizado une personas → observar colisiones y
mantener scope/tipo; entidad stale resucita memoria → validar fuente al leer;
reintento pierde miembro → update idempotente por identidad+revisión.
Aceptar creación, update textual, delete y reconstrucción con los mismos
miembros vivos. Métricas de lag, huérfanos y errores por versión, no nombres
personales. Mínimo índice reparable; no ontología, resolución universal ni
mejora de recall prometida.

### Ficha F — borrado operativo, historial y purga como contratos separados

**Problema/unidad.** `delete` elimina vector pero guarda `old_memory`; mensajes
e historia siguen siendo recursos distintos. `delete_all` pagina hasta vacío y
rompe ante batch repetido, pero devuelve mensaje genérico de éxito
(`main.py:1904-1958`). Extraer inventario de retención y recibo, no ese mensaje.

**Delta/alternativas.** Reutilizar lifecycle/GC/auditoría y recibo C.
Tombstone preserva explicación pero no cumple purga; borrar todo historial
elimina evidencia operativa. La política debe decidir qué metadatos mínimos
pueden sobrevivir sin contenido, y qué backups quedan fuera de garantía.

**Contrato hipotético.** `DeletionIntent::{Logical,PurgeContent}`,
`DeletionPlan { scope, target_revision, resources, retained_metadata }` y
`DeletionReceipt { completed,pending,unverified }`. Plan autorizado, ligado a
revisión; confirmación no convierte una estimación en snapshot atómico.
Error de proyección produce parcial reparable, no «nada pasó».

**Fases/seams.** Inventario en records, conversación, generation log, índices y
derivados; preview read-only; borrado idempotente; reconciliación; Studio.
No borrar derivados compartidos sin ownership (relación con Cognee).
Migración conserva interpretación de tombstones antiguos. Desactivar nuevas
purgas es rollback operativo; restaurar datos personales borrados no es
rollback permitido ni implícito.

**FMEA/aceptación.** Paginación truncada → quedan registros: contadores/fin real
y estado incompleto; batch repetido → no declarar purga completa. Historial
reintroduce texto → inventario y política de payload. Nueva escritura durante
purga → alcance explícito por revisión/manifest. Aceptación: dos scopes, más de
una página, fuente compartida, interrupción y reintento; recibo coincide con
lecturas autorizadas de cada recurso. Observabilidad solo conteos, estados y
correlación. Mínimo purga de contenido online; backups/retención legal y borrado
seguro del soporte físico requieren decisiones separadas.

### Fronteras operativas que completan las seis fichas

En la ruta sync leída no hay un token de cancelación que revierta las escrituras
ya confirmadas. El mutex SQLite no coordina el vector store ni protege una
revisión del documento durante una llamada al modelo. Estas propiedades no se
extrapolan a todos los backends o a `AsyncMemory`.

| Unidad | Ownership, concurrencia y visibilidad propuestos para Vanta | Compatibilidad y aceptación manual adicional |
|---|---|---|
| A: conservación | La autoridad y revisión se capturan antes de inferencia; el permiso no nace del texto extraído. Cancelar antes de commit no publica propuestas; después devuelve recibo C | Persistir razón/política sin reescribir el pasado. Consultar hechos contradictorios debe conservar ambas fuentes y no presentar fecha de importación como fecha de validez |
| B: candidatura | La traza observa la misma consulta autorizada; no prolonga locks ni cambia orden. Cancelación/fallo produce traza parcial identificada, no ranking completo ficticio | Consumidores antiguos ignoran campos opt-in. Comprobar IDs/orden con captura apagada/encendida y que un scope denegado no aparece en candidatos descartados |
| C: recibos | Confirmación pertenece al efecto primario; generación/historia/entidades tienen estados propios. Timeout posterior a put no se interpreta como «no aplicado» | Recuperar por ID de operación antes de repetir. Interrumpir entre primario e historial debe conservar el primero sin etiquetar todo como éxito |
| D: pertenencia | Validar identidad reservada al crear, resolver ID y publicar jobs; cambio de ACL invalida autoridad pendiente. Una revisión de contenido no reemplaza revisión de permisos | Legado sin scope queda clasificado, no reasignado al llamador. El mismo ID ajeno debe seguir protegido aunque se conozca por una traza anterior |
| E: entidades | El payload real es `linked_memory_ids`; lectura de conjunto → unión → update separado puede perder un miembro concurrente. La proyección nunca es autoridad sobre la fuente | Usar actualización condicional/reconciliación, no traducir ese RMW. Cancelar deja lag explícito. Reconstruir una generación y compararla con refs vivas antes de activar |
| F: purga | El plan fija alcance y revisión, no captura automáticamente futuras escrituras. Recibo incompleto sobrevive al cierre del cliente; no restaurar texto purgado como compensación | Distinguir DELETE lógico, purga online y backups. Reanudar tras una página fallida sin volver a borrar derivados compartidos |

**Evidencia que precisa E y C:** [main.py:1100-1220][M-entity-full]
normaliza nombres, intenta embeddings batch/individual, prioriza match exacto y
después match semántico, combina conjuntos y escribe entidades. Excepciones de
esa fase se registran sin invalidar el primario; el retorno enumera memorias
persistidas, no un recibo de cobertura NLP. Guardar mensajes sucede después.
El match por similitud no demuestra identidad de persona ni justifica copiar
su constante como política propia.

**Contraste con update/delete:** [main.py:2052-2144][M-mutate] confirma
primero el vector y luego escribe history; en estas dos funciones no se observa
un `try` que convierta el error de history en éxito. Por tanto, a diferencia de
ciertos fallbacks de add, un caller puede recibir excepción con el primario ya
cambiado. Además `_update_memory` registra texto en logs (`2053,2094`):
son superficies de retención distintas del API de historia. No habilitar logs
de payload como supuesto diagnóstico inocuo.

## 3. Riesgos e incidencias

| Evidencia | Frontera y efecto | Transferencia |
|---|---|---|
| Codigo confirmado | Metadata de recall previo se reduce a user/agent/run (`main.py:925-933`), mientras search acepta mas filtros | No asumir que todos los filtros de aplicacion acotan la dedup |
| Codigo confirmado | SQLite, vector y entidad se escriben en pasos separados; linking no fatal | Exito primario no certifica auditoria/proyeccion completa |
| Codigo confirmado | Dedup solo contra candidatos y batch; IDs nuevos aleatorios | No es garantia de exactly-once bajo concurrencia |
| Hipotesis | Dos escritores leen candidatos antes de insertar | Reproduccion discriminante pendiente, no race observada |
| Codigo confirmado | History guarda old/new tras DELETE y consulta por ID | Borrado operativo distinto de purga de datos personales |
| Operacional | En las firmas CRUD por ID examinadas no se recibe un scope de usuario | La cobertura de auth local es parcial: no acredita aislamiento por registro ni funcionamiento del despliegue. Requiere comprobar ownership en el servicio envolvente |

El adaptador Qdrant declara locks locales; esto no convierte el ciclo completo
read/dedup/write en transaccion:
[qdrant.py:1-65](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/vector_stores/qdrant.py#L1-L65).
La variante async incluye limpieza bulk para evitar carreras de entidades
en `delete_all` (`main.py:2342-2362`); no se audito paridad sync/async completa.

### Incidencias oficiales efectivamente leídas

Muestra dirigida de **tres issues**, no auditoría del tracker. Estado observado
2026-11-07 por `GET api.github.com/repos/mem0ai/mem0/issues/{n}`.
Se leyeron cuerpos completos; comentarios de #3245 y #4187 (este sin comentarios).
La petición de comentarios de #6655 devolvió respuesta vacía/no JSON en un
intento: no se le atribuyen comentarios ni PR confirmado.

1. **[#6655: Python OSS SDK: add() lets metadata set identity scope](https://github.com/mem0ai/mem0/issues/6655).**
   Cerrado, `completed`; creado 2026-07-29, cerrado 2026-08-04.
   Reporta 2.0.14/main `ea2ee075`: metadata define agent/run/actor cuando el
   argumento superior se omite. Incluye repro del helper e infer=False con store
   mock; no es una explotación observada por nosotros. El mecanismo está
   corregido estáticamente en el helper de este clon: strip completo en
   `main.py:360-364` antes de reponer los argumentos. **No** se afirma paridad
   completa TypeScript ni todas las APIs por ese cambio. Aplicabilidad: D,
   separar identidad de metadata libre. Repro pendiente: mismos casos en sync,
   async y transports propios con permisos distintos; no basta probar solo
   `metadata.user_id` si `user_id` explícito ya gana.
2. **[#3245: Memory deletion does not clean up Neo4j graph data](https://github.com/mem0ai/mem0/issues/3245).**
   Cerrado, creado 2025-07-29, cerrado 2026-03-23. Reporta mem0ai 0.1.115,
   Neo4j 5.23 y Qdrant: `_delete_memory` elimina vector/historia pero no grafo.
   Comentario del 2025-10-09 dice «Fixed ... PR incoming»; no identifica por sí
   solo un fix fusionado. El código actual no tiene ese `graph_memory.py`;
   mantiene entity store y limpieza no fatal `2116-2142`. No trasladar el
   defecto Neo4j al SHA nuevo. Aplicabilidad C/E/F: reconciliar toda proyección,
   verificar ausencia de retornos desde enlaces viejos. Repro propia pendiente.
3. **[#4187: Graph memory uses hard DELETE instead of soft-delete](https://github.com/mem0ai/mem0/issues/4187).**
   Cerrado, `completed`; creado 2026-03-02, cerrado 2026-03-21, sin comentarios
   devueltos. Reporta 1.0.4/Neo4j 5, `enable_graph=True`, y Cypher `DELETE r`
   frente a invalidación temporal descrita por el paper. El escenario es
   importar archivos antiguos que sustituyen relaciones actuales. No acredita
   que toda consulta temporal sea imposible ni defecto vigente en 2.2.1.
   No se comprobó PR de cierre. Aplicabilidad A/F: tiempo de evento separado de
   ingesta e historia; soft delete no equivale a purga. Repro pendiente con dos
   hechos fechados e importación inversa, sin depender de Neo4j en Vanta.

**Riesgo adicional propio de la ruta actual:** parseo inválido se degrada a
`[]` y guarda mensajes (`main.py:973-991`), mientras transporte LLM falla con
excepción. Es un hecho estático de clasificación de resultados, no issue
remoto inventado. El fallback de embeddings por texto también permite omitir
hechos individuales; el recibo debe explicar esos descartes.

**Seguridad y frontera:** el SDK embebido confía en su host. Que el helper fije
user/agent/run no autentica a quien lo invoca. REST tiene dependencias de auth;
la revisión no siguió por completo ownership y JWT hasta cada recurso.
Contenido de memoria, mensajes y custom instructions son entrada no confiable;
no deben convertirse en instrucciones privilegiadas ni registrar secretos en
errores. `_update_memory` loguea texto (`2053,2094`): considerar redacción antes
de adoptar una política similar de trazas.

## 4. Prerrequisitos en VantaDB

- Usar el corte revalidado V1–V5, no el estado histórico de METHOD.
- Mapear retention, supersession y dedup actuales a politicas, sin duplicarlas.
- Scope autorizado transversal a extraccion, recall candidato, history y UI.
- Contrato explicito entre commit primario, proyeccion y auditoria; medir despues.
- Presupuesto de `context_assemble` como autoridad; el top-k no lo sustituye.
- Extraccion LLM opcional y errores operativos distintos de cero hechos.

## 5. Que no trasladar

No trasladar dependencia LLM incondicional al constructor del motor, hashes MD5
como identidad de seguridad, pesos de ranking calibrados para otro backend,
fallos secundarios silenciosos, borrado con history textual sin retencion
explicita ni contratos Platform deducidos desde el SDK. No instalar todos los
extras para aprovechar una idea de dominio. Reescribir a Rust no evita licencia.

## 6. Advertencias y cobertura

**Leido, por rangos:** `mem0/memory/main.py` (helpers, constructor, add/infer,
search, update/delete/history/reset y limpieza async); `memory/storage.py`
(schema y transacciones); `configs/base.py`; `utils/factory.py`;
`utils/entity_extraction.py` (contrato); `utils/scoring.py`; `vector_stores/qdrant.py`
(constructor/locks); `client/main.py` (init/CRUD); `server/main.py`
(auth declarada y search/ID); `pyproject.toml`; `.gitmodules`; docs citadas;
CLI backend Platform y pagina dashboard de memorias.

**No examinado exhaustivamente:** proveedores vectoriales, prompts completos,
auth/session del server, REST y TypeScript completos, conectores, migraciones,
historia git, tests, fuente Platform y contenido del submodulo de evaluacion.
No se deduce cobertura total de esta seleccion. No se ejecutaron validadores
de enlaces/schema ni se regeneraron indices: propiedad y runtime lo prohiben.

## 7. Benchmarks existentes

`evaluation` es gitlink al commit `4b61c5d31b9c668a12b4f5e78064248a02c82d2b`
de `mem0ai/memory-benchmarks`, no carpeta con runner disponible en este clon:
[.gitmodules:1-4](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/.gitmodules#L1-L4).
El submodulo no esta materializado en el clon. La lectura remota complementaria del README, benchmarks/longmemeval/run.py, benchmarks/common/metrics.py y benchmarks/common/mem0_client.py se fijo al gitlink `4b61c5d31b9c668a12b4f5e78064248a02c82d2b`; vease [BENCHMARK-NOTES.md](BENCHMARK-NOTES.md). Esa evidencia distingue retrieval con juez de soporte y answerer, sin reproducir resultados ni auditar integramente LoCoMo/BEAM, entornos o efectos de todos sus runners.

La documentacion oficial declara LoCoMo (conversaciones/preguntas), LongMemEval
(memoria entre sesiones/actualizaciones) y BEAM, con scores de respuesta y
presupuesto top-200; no son medidas ANN ni resultados propios:
[memory-evaluation.mdx:65-90](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/docs/core-concepts/memory-evaluation.mdx#L65-L90).
La misma pagina describe componentes temporales Platform que OSS rechaza;
no comparar esas cifras con `Memory` local como si fueran el mismo sistema.
No se reproducen numeros de calidad.

**Pendiente manual:** ejecución y revisión de superficies no leídas del submodulo (entornos, efectos de runners, reset, delete-all, consumo de API y datos personales); no reobtención de datos.
La comparacion futura se rige por [protocolo comun](METHOD.md#protocolo-de-comparacion-de-benchmarks);
no se crea harness, grader, prompts, scoring ni umbral nuevo.

### Cómo funciona realmente el runner LongMemEval fijado

Lectura remota adicional de `run.py:90-140,194-219,279-324,407-443,
631-725,866-908,934-1047,1233-1375`
en el **gitlink**, no en `main` del submódulo:
[fuente inmutable][M-bench]. No se inicializó evaluation ni se ejecutó Python.

- **Entrada/unidad:** pregunta con `question_id`, `question_type`, `question`,
  `answer`, `question_date`, `answer_session_ids` y haystack de sesiones.
  La ingesta es por usuario aislado derivado de pregunta/run; el procesamiento
  llama `ingest_question`, después busca y evalúa. El esquema no es vector ANN.
  Preparación efectiva: descarga `longmemeval_s_cleaned.json` desde
  `resolve/main`, ordena sesiones por fecha, elimina `has_answer` de mensajes,
  forma pares consecutivos y omite pares con contenido vacío
  (`run.py:90-94,194-219,279-324,407-443` [M-bench]).
  No consta hash/split congelado del dataset descargado; `oss` y modelos del
  juez no identifican extracción, embeddings ni configuración efectiva del servidor.
- **Defaults observados:** modo `answerer`, backend `oss`, top-k 200, cortes
  10/20/50/200, modelos answerer/judge `gpt-5`, proveedor OpenAI, diez workers,
  muestreo `per-type=5`, seed 42 y resume activo. Son defaults **upstream**, no
  configuración propuesta para Vanta. `MEM0_BACKEND` puede sobreescribir backend.
- **Retrieval:** cronometra solo `mem0.search`; después el juez recibe pregunta,
  gold y memorias hasta cada corte y devuelve PASS/FAIL de soporte. Una salida
  no dict o judgment diferente de PASS cuenta cero. La gold se usa en juez,
  no en la llamada search de esa función. No puntúa intersección de IDs de
  sesiones como hace el agregado recall_all de Vanta.
- **Answerer:** es un modo aparte y añade generación; `predict-only` puede
  guardar retrieval sin veredictos. `user_profile` introduce otra fuente de
  contexto y otra llamada cuando se activa; no compararlo con variante sin perfil.
- **Denominador:** `compute_longmemeval_metrics` usa `len(evaluations)`, score
  ausente cero por cutoff y agregado por tipo. La cadena llama `asyncio.gather`;
  excepciones no capturadas dentro de pregunta pueden abortar antes del resumen.
  Ingesta `success=False` se loguea, pero la ruta continúa a search salvo shutdown.
  Por eso ni «todos los errores se descartan» ni «todos cuentan cero» son
  descripciones correctas del flujo completo.
- **Vacío operativo evaluable (M1):** el wrapper
  [mem0_client.py:254-344](https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/benchmarks/common/mem0_client.py#L254-L344)
  captura también errores HTTP y devuelve `[]` agotados los reintentos. No
  equivale a una excepción que escape al driver. El juez de abstención
  (`run.py:102-140` [M-bench]) puede devolver PASS con memorias vacías:
  una búsqueda fallida puede entrar en `evaluations` como vacío legítimo,
  **no necesariamente cero**. El denominador cuenta evaluaciones emitidas,
  no garantiza búsquedas exitosas; no se ha medido frecuencia ni impacto.
- **Resume/cache:** carga JSON previos y omite IDs ya presentes; agrega
  deduplicando por ID y última entrada. Reutilizar directorio tras cambiar modelo,
  backend o dataset puede mezclar experimentos si el operador no los separa.
  La presencia de seed no fija aleatoriedad de servicios LLM.
- **Concurrencia efectiva:** el wrapper `Mem0Client:53-100` acepta rpm pero
  construye `AsyncLimiter(100000,60)` con comentario sin limitación cliente y
  `TCPConnector(limit=0)`. Diez workers del driver no equivalen a un presupuesto
  global de requests ni a un servicio sin throttling.
- **Efectos:** escribe memorias en servicio OSS o cloud, archivos de resultados,
  checkpoints y usa APIs pagadas. El cliente tiene `delete_user:350-381`, pero
  en la cadena de pregunta examinada no se observó invocación de limpieza; no
  afirmar borrado automático. Cloud/OSS tienen rutas y autenticación diferentes.

**Adaptación concreta, sin crear harness:** distinguir la importación por
sesiones completas del eval Vanta de los pares filtrados de este runner;
conservar IDs/fechas, transformación y dataset efectivo, sin equipararlos.
Exponer adapter al store real con
scope de run/pregunta y retorno trazable a fuentes; comparar según el contrato
ya autorado en METHOD/BENCHMARK-NOTES. Mantener separados retrieval por evidencia,
soporte juzgado y QA. Modelos, importación, budgets y tiempos de extracción,
embeddings, búsqueda, generación y juez deben declararse por fase; search_latency
no incluye ingesta ni juez. No portar los defaults, prompts ni la fórmula como
un nuevo benchmark propio. LoCoMo/BEAM del submódulo siguen fuera de esta lectura.

**FMEA/aceptación documental M1–M2:** retries agotados → `[]` → PASS de
abstención puede ocultar fallo operativo; descarga mutable o pares omitidos →
corpus distinto sin cambio de etiqueta. Antes de comparar, el recibo debe
distinguir intentadas/evaluadas/error/abstención y preparación conocida o
Unknown conforme a METHOD; no deducir éxito de PASS ni modificar scoring,
configuración, prompts, harness o umbrales para esta corrección.

## 8. Matriz frente a VantaDB

| Capacidad | Estado VantaDB y fuente al corte fijado | Delta pertinente de Mem0 y decisión |
|---|---|---|
| Retrieval híbrido | Existe en motor; selección legacy/core en [V3][V-recall] y [referencia](VANTA-REFERENCE.md) | [Mem0 search][M-search] puntúa pool semántico: B, diagnosticar admisión, no copiar fusión |
| Dedup y supersession | Existe: [DedupAction/MemoryRecord][V-record], [writer][V-write] | [ADD-only][M-add]: A como política opcional, no reemplazo universal |
| Procedencia/auditoría | Existe: fuentes/version en [V1][V-record] y efectos post-commit [V2][V-write] | [History][M-storage]: C/F, distinguir conservación de contenido de consistencia |
| Contexto con presupuesto | Existe: [assemble_with_recall][V-context] | Search Mem0 no acredita ensamblado global equivalente; mantener el propio |
| Entidades como boost concreto | No acreditado equivalente en esta muestra propia, no ausencia de grafo | [Entity linking][M-entity-full]: E condicional a demostrar utilidad y reparación |
| Aislamiento extremo a extremo | Parcial en cobertura: gobernanza existente en [referencia](VANTA-REFERENCE.md); todos los transports no auditados | D exige validar pertenencia por ID; helper upstream no certifica servidor completo |
| Temporal Platform | No aplicable como extracción de código backend OSS | Cliente/docs no acreditan implementación privada ni paridad; rechazar esa atribución |
| Studio memoria/diagnóstico | Existe: componentes y SHA de §10 | B/C/E/F extienden inspectores; no nueva aplicación |
| Comparación LongMemEval equivalente | No acreditada: sesiones Vanta ≠ pares filtrados upstream | M1/M2: corpus/backend desconocidos y vacío por error potencialmente PASS; ningún score nuevo |

## 9. Debilidades revalidadas, hipótesis y falsos positivos

| Clase | Evidencia al corte 97c9623 | Consecuencia acotada |
|---|---|---|
| Hecho estático | Writer `430-446` borra antes del put y declara fallo parcial | C no puede prometer reemplazo indivisible sin cambiar esa frontera |
| Patrón / hipótesis | Checkpoint `79-116` usa clave global/RMW separado | Revisar writers y exclusión antes de un recibo basado en ese estado; lost update no reproducido |
| Hecho estático | `evals/eval_locomo.py:826-856` conserva sintético por defecto y FakeLocomoStore de fallback | No usar ese camino como certificación real; no significa ausencia de benchmarks medidos |
| Parcial | Explain y auditoría existen, pero no se acreditó traza de todos los descartes | B debe ser extensión, no reemplazo |
| No acreditado | Política de purga transversal e historial libre de contenido | F exige inventario, no afirmar incumplimiento legal |

Descartados: «falta dedup», «no hay RRF», «no hay procedencia», «no hay Studio»
y «no hay gate auth». `validate_auth_config:41-89` ya rechaza bind no-loopback
sin auth salvo override inseguro explícito. El valor de D es pertenencia y
metadata reservada, un problema distinto de autenticación al arrancar.

## 10. Interfaz e inspeccion

**Codigo disponible:** dashboard web Next.js self-hosted. `MemoriesPage`
lista por user_id opcional, pagina en cliente y permite delete con confirmacion:
[page.tsx:29-76](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/server/dashboard/src/app/(root)/dashboard/memories/page.tsx#L29-L76).
No acredita cliente desktop nativo ni equivalencia con dashboard Platform.
Graph view Pro/Enterprise es declaracion de producto, no fuente backend leida.
CLI Python tiene `PlatformBackend` HTTP:
[platform.py:1-54](https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/cli/python/src/mem0_cli/backend/platform.py#L1-L54).
Hay integraciones y SDK TypeScript inventariados, no certificados en ejecucion.

**Transferencia a Studio:** reusar memoria/actividad/diagnostico existentes;
considerar historial con retencion, desglose de candidaturas y estado de
proyecciones. Pendientes: validar permisos del dashboard, UX de purga y
correspondencia UI/API real; no se inicio ningun servidor ni navegador.

### Contraste real con Studio y cierre de cobertura

Studio leído en `develop-desktop` @
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`, sin editar:
`feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:138-164,193-228`
ya pide `explain:true`, abre registro/vecinos y consulta audit con filtrado
cliente por key;
`feature-freez/desktop/src/components/memory/MemoryLens.tsx:381-409` consulta
generation log y abre anchors;
`feature-freez/desktop/src/components/activity/ActivityPanel.tsx:52-89`
distingue carga y error, incluido audit no configurado. Cuerpos releídos con
`git show` del SHA completo, no bytes dirty de `desktop/...` ni UI ejecutada.

Flujo upstream leído: listar hasta 1000 → paginar de veinte en cliente → abrir
detalle → confirmar delete → refetch. El banner al alcanzar mil no demuestra
que la lista contenga toda la base. Oportunidad Studio: mostrar recibo C,
estado de proyecciones E y alcance/retención F en los inspectores existentes;
en B diferenciar «no candidato» de «score bajo». La UI debe mostrar transporte
sin capacidad, no aparentar historial vacío como si no hubiera eventos.
Ninguna pantalla puede otorgar permisos a partir de un user_id escrito a mano.
Si se visualizan evaluaciones, conservar estado operativo y abstención por
separado: un PASS no demuestra éxito de search (M1). Esta corrección de
[M1/M2/M3](reviews/memory.md#mem0) queda pendiente de reconfirmación independiente.
Fuentes correctivas (consulta pública 2026-11-07, sin ejecución): runner y
cliente al gitlink anterior; cuerpos Studio vía `git show`:
[MemoryLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L409),
[ActivityPanel](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L52-L89),
[RetrievalLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228).

**Dependencias entre unidades:** D gobierna A/C/E/F; C precede a garantías de
proyecciones E y purga F; B es read-only y puede evaluarse sin cambiar escritura.
A no necesita E. La referencia Cognee añade ownership compartido a F; Hindsight
añade precondición de revisión a C. No crear tres implementaciones del recibo.

**Cobertura de diez ejes:** 1 dictamen priorizado; 2 seis fichas y tres flujos;
3 tres issues leídos y riesgos estáticos; 4 prerequisitos por unidad; 5 rechazos;
6 límites/versiones; 7 runner LongMemEval; 8 matriz; 9 revalidación propia;
10 dashboard/Studio. No significa auditoría exhaustiva ni revisión adversarial
independiente completada: ésta corresponde al responsable.

**Pendientes concretos:** paridad sync/async/TS, todos los proveedores, auth/ID
end-to-end, limpieza real del benchmark, LoCoMo/BEAM, retención física,
licencias del submódulo/datasets y dependencias completas. La lectura de licencias
por superficies de §1 no equivale a un SBOM. Documentación histórica que atribuya Neo4j a este SHA debe
corregirse, no usarse como evidencia. Si se aprueban specs: dividir por política,
recibos/ownership, retrieval/entidades y purga/UI; conservar aquí la comparación.
No se ejecutaron apps/tests/builds/benchmarks/checks ni commits; no se editaron
fuentes, índices, método ni archivos fuera de estas tres fichas.

[M-add]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L918-L1098
[M-create]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L1970-L2048
[M-storage]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/storage.py#L102-L191
[M-mutate]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L2052-L2144
[M-search]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L1642-L1745
[M-score]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/utils/scoring.py#L60-L139
[M-entity]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L1100-L1115
[M-entity-full]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0/memory/main.py#L1100-L1220
[M-license]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/LICENSE
[M-ts-package]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/mem0-ts/package.json
[M-ui-package]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/server/dashboard/package.json
[M-strands-license]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/integrations/mem0-strands/LICENSE
[M-strands-notice]: https://github.com/mem0ai/mem0/blob/f1791daa86d469249c9b002f62644d58a1eb96a0/integrations/mem0-strands/NOTICE
[M-bench]: https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/benchmarks/longmemeval/run.py#L631-L725
[V-record]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/abstractions/types.rs#L71-L154
[V-write]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L389-L456
[V-recall]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L303-L334
[V-context]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L252-L348
[V-checkpoint]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/checkpoint.rs#L79-L116
