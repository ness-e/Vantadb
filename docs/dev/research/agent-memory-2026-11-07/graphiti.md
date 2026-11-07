---
title: "Graphiti: hechos temporales, procedencia y recuperación para VantaDB"
kind: research
status: proposed
description: "Estudio de arquitectura, flujos, incidencias públicas, evaluación y mecanismos Rust candidatos de Graphiti sobre revisiones congeladas"
tags: [research, memory, engineering]
---

# Graphiti: hechos temporales, procedencia y recuperación para VantaDB

## Índice

- [0. Identidad, evidencia y arquitectura](#0-identidad-evidencia-y-arquitectura)
- [1. Tres flujos trazados](#1-tres-flujos-trazados)
- [2. Recomendaciones priorizadas](#2-recomendaciones-priorizadas)
- [3. Fichas Rust candidatas](#3-fichas-rust-candidatas)
- [4. Incidencias públicas y seguridad](#4-incidencias-públicas-y-seguridad)
- [5. Contratos previos y rechazos](#5-contratos-previos-y-rechazos)
- [6. Benchmarks y adaptación](#6-benchmarks-y-adaptación)
- [7. Matriz y negativos VantaDB](#7-matriz-y-negativos-vantadb)
- [8. UI y Studio](#8-ui-y-studio)
- [9. Cobertura, fuentes y cierre](#9-cobertura-fuentes-y-cierre)

## 0. Identidad, evidencia y arquitectura

### 0.1 Revisión y límites del dictamen

| Campo | Evidencia congelada |
|---|---|
| Proyecto | `getzep/graphiti`, biblioteca de grafo temporal para agentes |
| Clon local, solo lectura | `C:\Users\Eros\graphiti` |
| Commit completo | `747b480ec6b12ecb718f1e2bc588b20ee36ea9c2`, rama `main` |
| Captura de metadata | 2026-11-07T06:56:01.9641319Z; 32 614 estrellas; no archivado |
| Preparación | Clon superficial, sin actualizar ni completar historia; [manifest](clone-manifest.json) |
| Licencia propia | Apache-2.0 en `LICENSE` y `pyproject.toml`; no implica licencia de modelos/servicios |
| Paquete leído | `graphiti-core` 0.31.2, Python >=3.10,<4 [G0] |
| VantaDB de comparación | `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; no la antigua base `c561158` dirty |
| Studio | Worktree `develop-desktop`, HEAD observado `0ad38c542`; lectura de rutas, no runtime |
| Fuentes públicas | Consulta adicional 2026-11-07; issues y PR son evidencia mutable, no parte del clon |

**Dictamen:** el diferencial aprovechable es la semántica de tiempo del hecho,
identidad temporal y soporte por episodios; no la mera existencia de búsqueda
híbrida, dedup o procedencia. Vanta ya tiene esas primitivas. No se recomienda
portar Graphiti, introducir Neo4j, ni convertir extracción LLM en requisito del
motor. La propuesta es extender contratos existentes con unidades independientes.

**Etiquetas:** **Código** acredita estructura estática; **documentación** es
declaración upstream; **reporte** es síntoma publicado, no reproducción propia;
**inferencia** deriva del camino leído; **propuesta** no existe como API actual.
No se ejecutaron servicios, consultas, tests, builds ni evaluadores. Se aplica
[EXPANSION-CONTRACT](EXPANSION-CONTRACT.md), [METHOD](METHOD.md) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md); OpenViking se usa como referencia de
profundidad, no como prueba de defectos del nuevo baseline.

### 0.2 Mapa de componentes y dependencias

| Superficie | Archivos/stack | Responsabilidad y frontera |
|---|---|---|
| Orquestación | `graphiti_core/graphiti.py`, Python async | `Graphiti`, add/search/remove, coordinación entre componentes |
| Dominio | `nodes.py`, `edges.py`, Pydantic | Episodios, entidades, relaciones, comunidades, saga; UUID y grupo |
| Extracción/resolución | `utils/maintenance/edge_operations.py`, operaciones de nodos | Extraer candidatos, resolver identidad/contradicción, fechar e invalidar |
| Retrieval | `search/search.py`, `search_config.py`, `search_filters.py` | Brazos tipados, recetas, filtros, reranking; no lista universal |
| Persistencia | `driver/` | Neo4j; extras FalkorDB, Neptune/OpenSearch y Kuzu deprecado |
| Modelos | `llm_client/`, `embedder/`, `cross_encoder/` | Puertos independientes de texto estructurado, vectores y reranking |
| Servicio | `server/graph_service/routers/`, DTO y FastAPI | Transporte REST; no redefine semántica del dominio |
| Agentes | `mcp_server/` | Adaptación MCP con dependencias/config propias |
| Evaluación | `tests/evals/` | Comparación de construcción de grafo; no certificación ANN o QA por presencia |
| Instrumentación | `tracer`, `telemetry`, `OTEL_TRACING.md` | Observabilidad; revisar datos y exportadores habilitados |

`pyproject.toml:13-42` declara Pydantic, neo4j, OpenAI, tenacity, numpy,
python-dotenv, posthog y certifi; extras Anthropic, Groq, Gemini, Voyage,
sentence-transformers y tracing [G0]. Que exista una dependencia no acredita que
una llamada concreta la use. La elección de driver también cambia qué significa
particionar: propiedad de nodo en un grafo frente a grafo físico separado.
Esa diferencia explica por qué `group_id` no puede tratarse como una etiqueta
decorativa ni como autorización.

**Despliegue observado:** biblioteca consumible por aplicación; wrappers REST y
MCP; grafos externos según backend. No hay fundamento para imponer ese despliegue
a Vanta embebido. Instalar extras, levantar bases y llamar proveedores queda
fuera del estudio. La disponibilidad de modelos nombrados por el código no se
verificó contra sus proveedores.

### 0.3 Modelo de datos y semántica de memoria

| Unidad | Identidad/atributos observados | Consecuencia |
|---|---|---|
| `Node` | UUID, nombre, labels, group_id, created_at [G8] | Entidad y sesión no son la misma unidad |
| `EpisodicNode` | Contenido, fuente, fecha válida y referencias a entity edges [G1][G9] | Evidencia de hechos; contenido puede no conservarse |
| `EntityNode` | Entidad reutilizable, resumen/atributos y embedding de nombre | Resumen mutable no es historial íntegro |
| `EntityEdge` | source/target UUID, nombre, fact, embedding, episodes, atributos [G2] | Hecho con soporte plural; identidad no es solo texto |
| Tiempos de arista | created_at, valid_at, invalid_at, expired_at, reference_time [G2] | Conocimiento, validez y ancla de extracción distintos |
| Comunidad | Agrupación y resumen para consulta [G4][G5] | No sustituye evidencia ni garantiza alcance autorizado |
| Saga | HAS_EPISODE/NEXT_EPISODE, primero/último [G9] | Orden conversacional, no checkpoint de ejecución durable |

`valid_at` significa comienzo del hecho; `invalid_at`, fin de su validez;
`expired_at`, momento en que se registró invalidación. `reference_time` pertenece
al episodio productor, no debe sustituirse silenciosamente por reloj de ingesta.
Los comentarios excluyen interpretar planes/promesas futuras como hechos ya
ocurridos o soft deadlines como fin de validez [G2:274-286].

**Alcance de “bitemporal”:** los campos soportan dos ejes de tiempo; su existencia
no prueba consultas históricas completas de atributos de nodos, consistencia de
todas las rutas ni restauración de estados previos. La unidad analizada es el
hecho-relación. Memoria conversacional es evidencia y hechos extraídos; no se
acreditó un runtime de ejecución resumible a partir de estas clases.

### 0.4 Features verificadas y límites de configuración

- `add_episode` permite entity/edge types Pydantic, exclusiones, episodios
  anteriores explícitos, instrucciones de extracción y saga [G1].
- `_extract_and_resolve_edges` separa resueltos, invalidados y **nuevos**;
  evita alimentar resúmenes con duplicados como nueva información [G10].
- `_apply_temporal_invalidation` puede expirar el entrante si ya existe otro
  posterior; llegada más reciente no equivale a hecho más nuevo. El helper
  de contradicciones omite intervalos disjuntos [G3].
- Search configura entidades, hechos, episodios y comunidades por separado.
  BM25/coseno/BFS y RRF/MMR/distancia/menciones/cross-encoder son opciones por
  superficie, no todos activos en cada petición [G4][G5].
- El vector puede proporcionarse; solo se genera cuando la receta necesita
  semántica/MMR. Query whitespace devuelve `SearchResults()` [G4].
- `_process_episode_data` borra `episode.content` cuando
  `store_raw_episode_content` es falso [G9:1031-1035]. “Tiene episodios” no
  garantiza conservación del original literal.
- `remove_episode` preserva hechos soportados por otros episodios y reconstruye
  resúmenes supervivientes [G11]. No prueba borrado físico completo.

## 1. Tres flujos trazados

### 1.1 Escritura de un episodio y publicación de hechos

**Escenario:** una conversación informa de un cambio de proyecto de una persona.

1. `add_episode` resuelve grupo/driver y recupera contexto anterior por
   reference_time/tipo; con `previous_episode_uuids`, carga los especificados.
   Reutiliza UUID si corresponde o construye el episodio [G1].
2. Extrae entidades, resuelve identidad y remapea endpoints. La extracción LLM
   pertenece a esta fase: no hay aún commit de toda la operación.
3. `_extract_and_resolve_edges` obtiene relaciones, incorpora endpoints
   materializados y aplica uuid_map; devuelve resolved/invalidated/new [G10].
4. Procesa atributos/resúmenes con nuevas relaciones; entrega el conjunto a
   `_process_episode_data` [G1].
5. Este construye MENTIONS y `episode.entity_edges`; delega
   `add_nodes_and_edges_bulk`. Es la frontera de escritura agrupada leída,
   **no** una transacción entre LLM remoto y almacenamiento [G9].
6. Saga agrega NEXT_EPISODE/HAS_EPISODE y guarda primero/último mediante
   llamadas posteriores. Comunidades se actualizan después si se solicitaron.

**Fallo/cancelación:** antes de persistir, un error de proveedor no debe contarse
como episodio publicado; después del bulk puede quedar saga/comunidad pendiente.
No se inspeccionaron transacciones de todos los drivers ni compensaciones.
La docstring aconseja episodios secuenciales y esperar cada uno [G1:1404-1413];
no certifica concurrencia. Reintentar con UUID estable reduce ambigüedad, pero
no acredita exactamente-una-vez de todos los efectos. Vanta debe preservar
original y publicar derivados por revisión, no copiar esta secuencia como
garantía atómica.

### 1.2 Recuperación por superficies hasta el consumidor

**Escenario:** buscar hechos de una entidad alrededor de un nodo central.

1. `Graphiti.search` elige receta de aristas; `search_` expone la superficie
   ampliada. Resultado simple de hechos no equivale a cuatro tipos [G12].
2. `search/search.py` valida group_ids y query; selecciona driver por petición.
   Query vacía termina sin embedding. Vector suministrado evita repetir esa
   llamada; receta lexical no necesita generarlo [G4].
3. Ejecuta edge/node/episode/community search con `semaphore_gather`.
   Cada brazo recibe su config y filtros aplicables; comunidad no recibe
   el mismo argumento `SearchFilters` en esta agregación [G4:160-220].
4. Rerank según receta. RRF, distancia y cross-encoder no producen
   probabilidades comparables entre clases [G5].
5. REST transforma aristas a facts; `/get-memory` compone texto desde mensajes
   y llama search [G13]. Campo presente en DTO no prueba propagación.

**Fallo:** `_trace_phase` marca error y relanza; no fabrica cero resultados
como éxito [G4:79-94]. No se acreditó contrato global de partial-success;
no asumir que gather salva todos los brazos tras un error. **Concurrencia:**
no se acreditó snapshot único entre búsqueda, expansión e hidratación.
**Borrado:** un resultado ya entregado no desaparece del consumidor. La propuesta
Vanta conserva revisión/origen y declara estado por brazo sin cambiar ranking.

### 1.3 Corrección temporal y borrado con soporte compartido

**Escenario:** llega tarde un episodio histórico; luego se retira una fuente.

1. El resolver recibe candidatos duplicados/contradictorios.
   `_apply_temporal_invalidation` no decide verdad: compara fechas sobre ese
   conjunto; si el entrante trae fin registra expiración; un candidato
   posterior puede cerrar el intervalo entrante [G3].
2. El helper no invalida intervalos sin solapamiento; para anterior
   contradictorio puede fijar invalid_at al comienzo del nuevo. No es álgebra
   completa ni prueba de identidad temporal.
3. `remove_episode` carga entity_edges y elimina su UUID de `edge.episodes`.
   Si quedan fuentes guarda la arista; sin ellas la agenda para borrar
   [G11:2174-2190].
4. Obtiene entidades por MENTIONS y elimina las mencionadas solo por ese
   episodio. Guarda aristas supervivientes con embedding cargado, borra
   aristas sin soporte, nodos y episodio mediante llamadas separadas.
5. Reconstruye resúmenes con episodios restantes ordenados por tiempo y
   procesados por lotes. Si falla, restaura `old_summary` en memoria y registra
   excepción: no demuestra retiro del contenido del resumen persistido.

**Fallo/ownership:** no hay transacción única visible de toda la eliminación.
Entidad sin MENTIONS queda fuera de enumeración; nueva fuente concurrente exige
revalidar soporte antes de borrar. Rebuild LLM puede fallar después del borrado
canónico. Huérfanos, derivados stale y borrado físico son límites diferentes.
No convertir “invalidado” en “olvidado”.

## 2. Recomendaciones priorizadas

| Orden | Recomendación | Valor incremental y condición |
|---|---|---|
| Primero | Revalidar frontera de reemplazo L1 antes de invalidación destructiva | `persist_planned` borra antes de put; aplicar intervalos no debe ampliar esa ventana [V1] |
| Primero | Contrato de tiempo efectivo separado de llegada | Vanta ya escribe valid_at; no se propone “añadir timestamps”, sino evitar confluir nacimiento del registro y validez del hecho [V1:545-557] |
| Después | Identidad temporal antes de dedup de relaciones | Mismo texto y endpoints pueden describir recurrencias disjuntas; issue #1865 |
| Después | Retirada de soporte y stale derivado explícitos | Extender provenance/supersession; no borrar hecho con otras fuentes ni servir resumen retirado como vigente |
| En paralelo | Trazabilidad por brazo y propagación de parámetros | Comparar receta solicitada y efectiva, sin otro índice o assembler |
| Condicional | Inspector de intervalos en Studio | Primero contrato backend; lectura UI puede avanzar sin migración de escritura |
| No prioritario | Comunidades/sagas como sistema nuevo | Ya hay escenas, sesiones, scheduler; exigir caso que no cubran |

No se prometen mejoras de recall, latencia o coste. Los candidatos siguientes
separan lógica temporal pura, publicación, retrieval y UX; no son cinco formas
de nombrar el mismo DTO. Son especificaciones preliminares, sin IDs de backlog.

## 3. Fichas Rust candidatas

Los fragmentos son **interfaces hipotéticas**, deliberadamente no compilables
como módulo autónomo: los tipos de identidad/error deben adaptarse a los ya
existentes. No son una API implementada. No fijan pesos, prompts ni umbrales.

### 3.1 Intervalos de hechos y resolución temporal pura

**Problema/escenario:** un episodio recibido hoy describe una asignación anterior;
el escritor no debe convertir llegada tardía en autoridad para invalidar la
asignación actual. Unidad mínima: comparación de intervalos y decisión con razón,
inspirada en [G2][G3], separada de extracción de fechas y verdad semántica.

**Valor frente a Vanta:** `record_input` ya materializa valid_at desde created_at;
`lifecycle` y dedup ya modelan supersession. El incremento es representar fecha
efectiva suministrada y tiempo de conocimiento sin forzar equivalencia [V1].
Alternativas: metadata tipada sobre SDK existente, adapter de dominio, o mantener
solo timestamps operativos. Preferir extensión de dominio; rechazar Neo4j como
dependencia de una función pura.

```rust
// Propuesta: nombres y representación finales sujetos a spec.
struct FactClock { valid_from: Option<i64>, valid_until: Option<i64>,
                   learned_at: i64, source_reference: Option<i64> }
enum IntervalAction { KeepBoth, CloseExisting, CloseIncoming, Undetermined }
fn compare_contradictory_facts(
    existing: &FactClock, incoming: &FactClock
) -> Result<IntervalAction, TemporalError>;
```

- **Inputs/outputs:** dos hechos previamente autorizados y contradicción ya
  establecida por el llamador; devuelve decisión y razón auditable. Error por
  intervalo mal formado o fecha inválida, no por falta de LLM.
- **Invariantes:** desconocido no equivale a epoch; fin no precede inicio; no
  extender fin histórico para acomodar novedad; intervalos disjuntos coexisten;
  learned_at no decide verdad; no crear fuente ni borrar payload desde helper.
- **Seam real:** `l1_dedup.rs` para decisión, `l1_writer.rs::record_input`,
  `write_memory`, lifecycle para aplicar; SDK sigue dueño del storage [V1][V2].
- **Fases:** definir semántica UTC/extremos; lectura compatible con campos
  ausentes; incorporar fechas explícitas sin reinterpretar historia; modo
  diagnóstico compara decisiones; activar aplicación solo tras garantías L1.
  No reindex requerido por comparación pura; sí si se añade filtro temporal.
- **Rollback:** desactivar aplicación, conservar metadata adicional y decisiones;
  no “restaurar” hechos borrados sin fuente. No backfill desde created_at como
  si fuera fecha de mundo. Cambios de semántica requieren versión de contrato.
- **Dependencias:** identidad/provenance existente; publicación segura para modo
  mutante. UI y función pura no dependen de arreglar checkpoint global.
- **FMEA:** fecha extraída errónea → invalida hecho correcto → detectar razón y
  fuente/revisión → exigir validación de entrada y resultado Undetermined;
  desconocido convertido en cero → orden falso → casos sin fecha;
  revisión cambia entre decidir/aplicar → supersession equivocada → conflicto
  de revisión, replanificar sin reaplicar decisión antigua.
- **Aceptación:** llegada invertida converge semánticamente; fechas iguales no
  eliminan información sin regla explícita; intervalo disjunto intacto; límite
  exacto de fin/inicio documentado; error no muta. Observabilidad: conteos por
  acción/error y conflictos, sin nombres, contenido o fechas personales por defecto.
- **Mínimo/no objetivos:** lógica y metadata de un hecho; no temporalizar todos
  los atributos ni autorizar a un modelo a borrar. Abierto: representación
  de precisión/fecha aproximada y política ante contradicción sin fechas.

### 3.2 Identidad temporal en dedup: recurrencia no es duplicado

**Problema:** la persona vuelve al mismo proyecto meses después. Fact string
idéntica no implica misma ocurrencia. Unidad mínima: elegibilidad de reutilizar
ID antes de dedup exacto, semántico y dentro del lote. Issue #1865 aporta
reproducción con fechas suministradas; [G3] muestra por qué invalidación sola
no resuelve identidad.

**Incremento/alternativas:** extender decisiones actuales de `l1_dedup`, no
crear un deduplicador paralelo. Un adapter puede registrar ocurrencias como
memorias independientes mientras se acuerda contrato; rechazar normalizar todo
por hash de texto. No exigir que LLM distinga intervalos conocidos.

```rust
enum OccurrenceMatch { SameOccurrence, DistinctOccurrence, NeedsReview }
fn dedup_eligibility(
    stored_id: &str, stored_revision: u64,
    old: &FactClock, new: &FactClock
) -> Result<OccurrenceMatch, TemporalError>;
```

- **Identidad:** ID lógico de fuente no cambia por revisión; ID de ocurrencia
  puede diferir aunque texto coincida. El resultado debe enlazar revisión base,
  no autorizar write por sí mismo. Error de identidad/scope no cae a merge.
- **Invariantes:** dedup exacto y LLM respetan misma elegibilidad; batch y
  secuencial no colapsan recurrencia; no crear intervalo usando hora de llegada;
  desconocimiento produce revisión explícita, no afirmación inventada.
- **Seam:** precondición de `DedupDecision` en `l1_dedup.rs`; escritor conserva
  su aplicación única `write_memory`/`apply_dedup_batch` [V1][V2].
- **Fases/migración:** documentar identidad actual; añadir gate puro; registrar
  shadow outcomes; activar sin reescribir IDs históricos. Reprocesar únicamente
  fuentes retenidas y autorizadas si se desea recuperar recurrencias colapsadas.
  Dato perdido no se reconstruye por reindex de vectores.
- **Rollback:** volver al gate anterior solo para futuras decisiones y preservar
  ocurrencias ya creadas; no fusionarlas masivamente al desactivar.
- **Dependencias:** 3.1 para intervalos; no depende de comunidades ni de Studio.
  Publicación concurrente sigue necesitando revisión segura.
- **FMEA:** par con tiempo incompleto se acepta como duplicado → pérdida de
  ocurrencia → detectar decisiones sin evidencia temporal → NeedsReview;
  cada retry crea otra ocurrencia → inflación → identidad estable de source
  event; distintos tenants colisionan → cruce → scope antes del gate.
- **Aceptación:** ida/vuelta al mismo proyecto conserva dos episodios y sus
  intervalos; duplicado realmente compatible reutiliza identidad; batch no
  cambia esa regla; conflicto de revisión no se resuelve con overwrite.
  Métricas: match/distinct/unknown, duplicados por fuente, sin payload.
- **No objetivos/abiertos:** no corregir extracción temporal upstream ni definir
  pesos de similitud. Resolver tratamiento de precisión parcial antes de spec.

### 3.3 Soporte por fuentes y retirada de derivados

**Problema:** eliminar una conversación no debe borrar un hecho sustentado por
otra, pero tampoco dejar el texto retirado en un resumen “vigente”. Unidad
mínima: detach de fuente + plan de invalidación por revisión [G11], no copia de
todo el grafo. Incremento: composición verificable de provenance, supersession
y jobs existentes; no presentar provenance como ausente.

```rust
struct SupportRemoval { source_id: String, expected_revision: u64 }
enum DerivedDisposition { Retain, MarkStale, RemoveUnsupported }
fn plan_source_withdrawal(
    request: SupportRemoval, supports: &[SourceSupport]
) -> Result<WithdrawalPlan, WithdrawalError>;
```

- **Contrato:** entrada desde frontera autorizada de borrado; salida enumera
  soportes restantes, derivados afectados y revisión esperada. Errores:
  NotAuthorized, RevisionConflict, UnresolvedReference, StorageFailure.
- **Invariantes:** un soporte vivo impide borrado por “sin soporte”; fuente
  retirada nunca genera nuevos derivados; stale no se anuncia ready; un fallo
  de regeneración no resucita contenido retirado; no afirmar borrado físico de
  backups o clientes con recibo lógico.
- **Seam:** `source_message_ids`/metadata del escritor [V1], lifecycle existente,
  scheduler/jobs y auditoría `src/cli_handlers/index.rs` para ampliar comprobación.
  Transporte MCP/HTTP no decide semántica de soporte.
- **Fases:** inventariar enlaces reales; construir plan read-only; añadir
  estados de derivado/versiones; publicar retirada con garantías de reemplazo;
  jobs reanudables recomponen desde fuentes restantes; auditar huérfanos.
  Reindex solo de derivados afectados, no de todos los originales.
- **Migración/rollback:** soportes históricos desconocidos quedan Unknown, no
  “cero”; rollback deshabilita regeneración nueva pero mantiene tombstone y
  retirada autorizada. Restaurar requiere autorización y fuentes retenidas.
- **Dependencias:** seguridad de reemplazo L1 antes de retirar físicamente;
  equivalente transversal a derivados versionados OpenViking, no sistema nuevo.
- **FMEA:** soporte añadido durante plan → borrado injustificado → CAS/revalidar;
  rebuild fallido → resumen antiguo filtra dato → estado stale excluido;
  huérfano fuera de enumeración → remanente invisible → auditoría desde fuentes
  y derivados, no solo desde relaciones disponibles.
- **Aceptación:** quitar una de dos fuentes preserva soporte correcto; quitar
  última retira del recall/preview; fallo intermedio reanudable; retry idempotente;
  otro tenant intacto. Métricas: pendientes/stale/huérfanos/conflictos por etapa,
  IDs opacos o agregados; nunca resúmenes sensibles en errores por defecto.
- **Mínimo/no objetivos:** retirada lógica trazable de una fuente; no prometer
  GDPR completo, purga criptográfica ni borrar sin consentimiento.

### 3.4 Resultado tipado y estado efectivo por brazo de retrieval

**Problema:** distinguir “no hay hechos” de “falló embedding” y saber si el
resultado procede de episodio, entidad o resumen. Unidad mínima: origen,
revisión y resultado de ejecución por brazo [G4][G5]. No portar su catálogo
de rerankers ni inventar un score normalizado universal.

```rust
enum EvidenceKind { RawMessage, Fact, Entity, DerivedContext }
enum ArmState { Complete, Skipped, Failed }
struct RecallTrace { kind: EvidenceKind, state: ArmState,
                     requested_recipe: String, effective_recipe: String }
fn attach_recall_trace(result: ExistingRecallResult) -> TracedRecall;
```

- **Incremento/seam:** extender `auto_recall.rs::RecallConfig`,
  `search_records_core` y `assemble_with_recall`; el core hybrid existe y
  `core_search` es false por defecto [V3][V4]. No afirmar BM25 efectivo por
  defecto del simple hecho de que exista el flag.
- **Contrato/invariantes:** ID/revisión del registro canónico, scope preservado
  al expandir/hidratar; score con tipo/origen, no probabilidad; fallo no se
  transforma en lista vacía exitosa; presupuesto único en assembler.
- **Alternativas:** solo instrumentación de rutas actuales primero; adapter
  para superficie nueva después. Rechazar otro assembler o índice de hechos.
- **Fases/migración:** mapear recetas y errores actuales; añadir campos
  opcionales backwards-compatible; exponer requested/effective; UI read-only;
  decidir partial policy sin cambiar algoritmo. Sin migración del corpus.
  Rollback omite campos nuevos, no borra registros ni recompone ranking.
- **Dependencias:** contrato de scopes y errores; independiente de 3.1 y de
  checkpoint mutante. Relacionado con diagnóstico retrieval ya existente.
- **FMEA:** config no propagada → receta distinta → comparar requested/effective;
  fallback elimina filtro → fuga → error tipado antes de ejecutar; expansión
  revive borrado → verificar revisión/estado en hidratación; doble evidencia
  agota budget → dedup por identidad en ensamblador.
- **Aceptación:** recipe y centro solicitados llegan al seam; empty query se
  distingue de fallo; brazo omitido no aparece exitoso; scopes sobreviven a
  fallback y expansión; mismo presupuesto. Observabilidad por fase, cardinalidad
  y códigos de error; no queries ni payloads por defecto.
- **Mínimo/no objetivos:** explicar rutas actuales; nuevas superficies solo con
  necesidad aprobada. Abierto: si partial-success es aceptable por consumidor.

### 3.5 Inspector temporal y de soporte en Studio existente

**Problema:** un usuario ve dos versiones y no sabe si hubo cambio de mundo,
corrección, llegada tardía o fallo del derivado. Unidad: proyección read-only de
intervalos, fuentes y razón de supersession; no timeline construida desde logs
con texto privado. Upstream aporta campos [G2], no UI operativa acreditada.

```rust
// Contrato de vista hipotético del backend, no estado autoritativo del frontend.
struct FactInspection { id: String, revision: u64, clock: FactClock,
                        support_ids: Vec<String>, derived_state: String }
fn inspect_fact(scope: AuthorizedScope, id: &str)
    -> Result<FactInspection, InspectError>;
```

- **Seam/valor:** superficies `memorias`, `actividad`, `retrieval` del Studio
  leído [S1], datos del escritor/lifecycle existente. Añadir explicación, no
  otro escritorio. Alternativa: ampliar detalle actual antes de nueva vista.
- **Invariantes/errores:** revisión mostrada identificable; Unknown no se dibuja
  como fecha cero; acceso se decide backend; NotFound no revela existencia de
  activos ajenos; preview respeta retirada y retención.
- **Fases/migración:** proyección desde campos existentes; etiquetar faltantes;
  agregar intervalos cuando 3.1 esté aprobado; drill-down de fuente autorizada;
  después acciones con confirmación y revisión, fuera del mínimo read-only.
  No migración de datos para empezar; rollback oculta panel, no cambia hechos.
- **Dependencias:** 3.4 para explicar retrieval, 3.3 para estado stale; no bloquear
  vista básica por toda la migración temporal.
- **FMEA:** cliente cachea fuente revocada → exposición → invalidar preview y
  consultar autorización al abrir; UI infiere fecha → mentira visual → Unknown;
  motivo con contenido sensible → fuga en notificación → códigos y redacción.
- **Aceptación:** distinguir válido/conocido/retirado, fuente faltante, revisión
  conflictiva y error de carga; cambio de conexión/namespace elimina estado
  anterior. Observabilidad de carga/errores sin payload ni IDs personales.
- **No objetivos/abiertos:** no renderizar todo el grafo, no importar Neo4j Browser,
  no acciones destructivas por LLM. Falta diseño humano de navegación temporal.

### 3.6 Ownership y frontera de commit de las cinco extracciones

Los tipos anteriores son pseudocontratos de diseño; `AuthorizedScope`,
`ExistingRecallResult` y `SourceSupport` no se acreditan como tipos existentes.
Una futura spec debe enlazarlos a los tipos reales, no crear aliases públicos
sin necesidad. La aceptación descrita en cada ficha es **manual futura**,
no prueba ejecutada ni nuevo protocolo de benchmark.

| Ficha | Dueño del estado y commit propuesto | Concurrencia, cancelación y error |
|---|---|---|
| 3.1 | Dominio puro calcula; `l1_writer::plan_write` conserva ownership de publicación y lifecycle de supersession | Plan lleva revisión base; cancelación antes de persistir no cambia hechos; después debe devolver estado persistido y efectos pendientes |
| 3.2 | Gate dentro de dedup; escritor asigna identidad de ocurrencia, fuente conserva ID de evento | Dos decisiones sobre misma revisión no hacen last-write-wins; retry del evento no crea tercera ocurrencia; RevisionConflict exige releer |
| 3.3 | Registro canónico de soporte y retirada; jobs son dueños de regeneración, no de autorización | Retirar primero de la elegibilidad de lectura; publicar derivado solo si fuentes/revisión siguen vigentes; cleanup parcial sigue pendiente |
| 3.4 | Recall produce traza; assembler posee presupuesto; caller elige política partial | Trazas no prometen snapshot del corpus; respuesta tardía de otra revisión no se añade a un contexto ya entregado |
| 3.5 | Backend devuelve proyección autorizada; Studio posee únicamente estado de navegación/cache | Al cambiar conexión/scope cancelar carga e ignorar respuesta antigua; no convertir una inspección en permiso de escritura |

**Relaciones:** 3.1 → 3.2; soporte/revisión → 3.3; 3.4 → diagnóstico
de 3.5. La vista básica de 3.5 y la función pura de 3.1 son independientes
del arreglo de escritura L1. Solo las aplicaciones mutantes quedan
condicionadas a una publicación segura. Esto evita un bloqueo universal.

## 4. Incidencias públicas y seguridad

### 4.1 Qué se pudo comprobar en la red pública

Consulta documental del **2026-11-07**: cuerpos y campos de las APIs GitHub,
no ejecución de los ejemplos. Las fechas siguientes son las devueltas por
GitHub; no se deducen de una etiqueta ni de la fecha del clon.

| Fuente | Estado/fechas acreditados | Mecanismo y límite |
|---|---|---|
| [Issue #1865](https://github.com/getzep/graphiti/issues/1865) | Open; creado 2026-09-09T21:01:40Z; actualizado 2026-09-30T15:39:09Z; closed_at=null | Reproducción declarada en 0.30.2, SHA `eaa4128681bc53487138a4bbc22d58336ebe70d2`: mismo texto/endpoints reutiliza arista aunque intervalos disjuntos |
| [Issue #1676](https://github.com/getzep/graphiti/issues/1676) | Closed el 2026-09-08T14:25:53Z; creado 2026-07-23T05:03:51Z | Reporte en 0.29.2/FalkorDB: mutación compartida de driver entre awaits cruza grafos; no es un fallo genérico de todo grafo temporal |
| [PR #1699](https://github.com/getzep/graphiti/pull/1699) | Closed y **merged_at=2026-09-08T14:25:52Z**, merge `08834609936e85a1bbed4158773b1f4641f48d6f` | Bundle driver/clients por llamada para add/bulk; el merge está probado por `/pulls/1699`, no solamente por cierre del issue |
| [Issue #1876](https://github.com/getzep/graphiti/issues/1876) | Open; creado 2026-09-10T17:45:25Z; actualizado 2026-09-30T15:35:51Z | Reporta write en grafo group_id y read en database configurada; su conclusión global debe contrastarse con el decorador del SHA congelado |

APIs efectivamente leídas: [1865](https://api.github.com/repos/getzep/graphiti/issues/1865),
[1676](https://api.github.com/repos/getzep/graphiti/issues/1676),
[1699](https://api.github.com/repos/getzep/graphiti/pulls/1699) y
[1876](https://api.github.com/repos/getzep/graphiti/issues/1876).
El estado público abierto no demuestra que el defecto siga presente en
todas las rutas ni versiones. Tampoco un merge prueba integración real.

**#1865 contra el clon:** `_dedupe_extracted_edge:945-955` [G3] sigue
comparando texto normalizado y endpoints sin gate temporal en ese fast path;
el camino semántico elige `related_edges[duplicate_fact_id]` antes de resolver
contradicción. La observación estática sustenta la oportunidad de 3.2,
no una reproducción propia del caso. El mismo archivo ahora separa datado
e invalidación y finaliza ciertos intervalos en `_apply_temporal_invalidation`;
por eso no se trasladan íntegros todos los síntomas de la versión del reporte.
El arreglo propuesto en un fork no se presenta como merge upstream.

**#1876, falso positivo global descartado:** `search`, `search_` y
`retrieve_episodes` están decorados [G12][G14]. En FalkorDB, el decorador
clona driver por un solo grupo distinto del default y por cada grupo cuando
hay varios. No es correcto afirmar que *todo* read ignora group_id en este
SHA. Queda por revisar la ruta MCP que invoque directamente
`EpisodicNode.get_by_group_ids` en la lectura inicial; la ampliación siguiente
cierra esa laguna estática, sin certificar ejecución. El principio transferible es consistencia
entre routing de write/read/delete, no copiar un supuesto fix indiscriminado.

**Ampliación de cierre — MCP directo sí inspeccionado [G16]:**
`_driver_for_group:191-203` cachea por `(id(base_driver), group_id)` y llama
`clone(database=group_id)`; `_client_for_group:206-219` copia el cliente y
su bundle de clients, sin mutar el driver compartido. `get_episodes:795-840`
normaliza escalar/lista, aplica grupo configurado si se omite, consulta cada
grupo con ese helper y recorta **después** de ordenar globalmente por
`created_at`. El bucle es secuencial: el comentario documenta la limitación
de consultas concurrentes sobre conexión FalkorDB compartida. Sin grupos ni
default devuelve lista vacía (`841-844`), no busca indiscriminadamente.

`get_entity_edge:763-791` y `delete_entity_edge:696-725` usan driver ligado al
grupo; `delete_episode:729-759` usa cliente ligado y `remove_episode`, no
`EpisodicNode.delete` aislado. Excepciones se convierten en `ErrorResponse`
con texto de excepción: no confundir error con lista vacía ni considerar ese
texto seguro para un cliente no confiable. La cache y la copia superficial
no acreditan lifecycle/cierre de conexiones ni ACL.

La [página pública #1876](https://github.com/getzep/graphiti/issues/1876)
se releyó en este cierre el 2026-11-07, cuerpo y comentarios visibles:
permanece Open y describe el camino antiguo con `client.driver` directo.
**Ese camino ya no coincide con el MCP del SHA congelado.** No se atribuye
el cambio a una PR por inferencia de fechas. El issue sigue siendo útil para
contrastar rutas y defaults, no para abrir un “fix de reads” ya presente.
Queda pendiente confirmar semántica real de `clone` por backend y permisos de
actor; filtrar por group_id no los demuestra.

### 4.2 FMEA upstream y frontera de confianza

| Disparador | Mecanismo observado o hipótesis | Consecuencia/validación discriminante | Decisión para Vanta |
|---|---|---|---|
| Misma relación en otro período | Fast path de dedup por texto/endpoints [G3] | Colapso de ocurrencia; fixtures con fechas suministradas separan bookkeeping de calidad LLM | Gate temporal 3.2, no cambiar similitud sin evidencia |
| Dos grupos concurrentes | Reporte histórico #1676; bundle local presente [G1] | Auditar destino físico con interleaving; tests fake del PR no certifican todos los drivers | Scope inmutable por operación en adaptadores; no serializar globalmente por defecto |
| Borrar soporte mientras llega otro | Lectura→modificación de episodes→save [G11] | Lost update o retiro incorrecto; comprobar revisión al aplicar | Plan de retirada y conflicto tipado |
| Falla LLM al reconstruir resumen | `old_summary` se restaura y excepción se registra [G11] | Puede quedar resumen persistido anterior; comprobar recall/preview tras retirar fuente | Stale no elegible, no restauración silenciosa |
| Fallo durante rebuild de comunidades | Se eliminan comunidades antes de construirlas [G15] | Proyección temporalmente ausente; fallo no prueba pérdida de episodios | Generación nueva y publicación condicionada si se adopta el principio |
| Instrucción maliciosa en episodio | Contenido no confiable alimenta prompts y contexto | Riesgo de prompt injection, no CVE comprobada; separar datos y control y probar el consumidor | Mantener `InjectionPolicy` y taint existentes |

`group_id` es partición/routing, no autorización del actor. Un DTO con
grupo y UUID no acredita ACL en cada ruta REST/MCP. No se afirma
vulnerabilidad explotable: falta auditoría de autenticación, deployment
y enforcement. La telemetría y trazas deben revisarse antes de enviar
payloads o identificadores a terceros; la dependencia posthog no prueba por
sí sola qué datos se exportaron.

## 5. Contratos previos y rechazos

### 5.1 Dependencias locales específicas

Baseline único de esta ampliación: [VANTA-REFERENCE](VANTA-REFERENCE.md),
SHA `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.

1. **Publicación L1:** `persist_planned:289-294` borra targets y después
   llama `put_record` [V1]. Es una secuencia estática; no constituye prueba
   de crash loss ni de una transacción envolvente. Debe aclararse antes de
   activar reemplazo/retirada nuevos, no antes de modelar intervalos.
2. **Éxito parcial:** `mark_contradicted_targets` documenta error propagado
   después del nuevo registro persistido [V1:332-348]. Un Err puede coexistir
   con commit primario; retry necesita ID/revisión, no asumir rollback.
3. **Tiempo existente:** `valid_at_ms` se materializa desde created_at
   del registro [V1:484-558]. El delta es distinguir tiempo del hecho y del
   conocimiento, no “Vanta no tiene temporalidad”.
4. **Scope existente:** `RecallScope`, `RecallConfig`, `InjectionPolicy` [V3]
   ya gobiernan inyección. Cualquier expansión por soporte debe conservarlos;
   no inventar un tipo implementado `MemoryScope`.
5. **Contexto existente:** assembler/reportes/presupuesto [V4] se amplían.
   El prefijo protegido puede exceder presupuesto por contrato; no prometer
   que una nueva vista de hechos hará caber todo contexto.

### 5.2 Mecanismos examinados que no justifican otra extracción

| Mecanismo Graphiti | Alternativa y coste oculto | Dictamen |
|---|---|---|
| Neo4j/FalkorDB/Neptune como soporte físico | Introduce servicios, auth, compatibilidad y backup propios | No trasladar al motor embebido |
| BM25/coseno/RRF/MMR | Vanta ya tiene híbrido; semántica del score cambia por receta | Instrumentar 3.4, no reescribir índices |
| Comunidades y resúmenes globales [G15] | Escenas/derivados existentes; rebuild destruye proyección anterior antes del éxito | Conservar lección de revisión de 3.3; no adoptar clustering sin caso agente |
| Saga/NEXT_EPISODE/HAS_EPISODE | Agrupación narrativa; no scheduler durable ni rollback de herramientas | Relacionar fuentes si hace falta; no llamarlo memoria de ejecución |
| LLMRuntime y esquemas Pydantic | Puertos/modelos opcionales ya separables; traducción literal no añade garantía | Mantener providers fuera de primitivas; exigir schema-validación al adaptar |
| Persistir solo hechos extraídos | Ahorra raw pero impide verificar extracción/borrado por fuente | No imponer `store_raw_episode_content=false` a Vanta |
| Catálogo de drivers y modelos | Coste de conformance y licencias por componente | Adapter por necesidad, no portabilidad ficticia |

Cambiar Python por Rust no elimina obligaciones Apache-2.0 ni convierte
una traducción de implementación en diseño independiente. Revisar notices,
dependencias y modelos concretos antes de derivar código.

## 6. Benchmarks y adaptación

### 6.1 Runner realmente leído

`tests/evals/eval_cli.py:1-40` y
`eval_e2e_graph_building.py:32-181` [G6][G7] implementan una evaluación de
**construcción de grafo respecto de otro grafo generado**, no Recall@k,
LoCoMo QA ni éxito de agente. No se ejecutaron ni se “corrigieron”.

- **Entrada:** ruta relativa fija `data/longmemeval_data/longmemeval_oracle.json`.
  Hay que conservar variante oracle: no presentarla como `_s` completo.
  Toma primeras `multi_session_count` filas por `iloc`; `session_length`
  cuenta mensajes globalmente por usuario, no longitud por cada sesión.
  Importa ambos roles como `role: content`, a diferencia del raw MemPalace
  que concatena user. CLI exige ambos enteros; no fija defaults para ellos.
- **Escritura:** usuarios `lme_oracle_experiment_user_N_baseline/candidate`;
  add_episode secuencial por usuario y usuarios concurrentes vía gather.
  Usa credenciales Neo4j importadas de tests de integración.
  Serializa `baseline_graph_results.json` y `candidate_graph_results.json`;
  elimina vectores de objetos de resultado, no acredita borrado de la base.
  Los nombres de grupo son persistentes y se reutilizan: no hay reset ni
  restauración en el runner leído [G6:32-180]. No suministra UUID a los
  episodios; repetir contra el mismo destino puede acumular datos, no produce
  necesariamente un corpus limpio. `eval_cli.py:25-35` [G7] permite repetir
  candidate sin reconstruir baseline. El coste de un baseline reutilizado
  queda fuera de esa pasada; no confundirlo con coste total de ambos grafos.
- **Modelos:** baseline y juez por defecto nombran `gpt-6-luna` en el código;
  la existencia del nombre no acredita disponibilidad comercial. Extracción,
  embedding, grafo externo y juez cuestan por fases; no hay medición monetaria.
  No convertir el texto de configuración en aval de un proveedor.
- **Juez:** `EvalAddEpisodeResults` devuelve `candidate_is_worse` y reasoning;
  el score suma uno cuando no es peor. No es una métrica de exactitud absoluta
  ni una demostración de equivalencia semántica.
- **Denominador:** promedio por usuario de `user_raw_score /
  len(add_episode_results[user_id])`, luego `/ user_count`. `zip(...,
  strict=False)` compara solo hasta el mínimo de las tres listas, mientras
  el divisor mantiene longitud candidate. Con cero usuarios/resultados
  puede dividir por cero; un error de proveedor no tiene fila de error
  normalizada en el bucle leído.
- **Defecto de contexto estático:** `add_episode_context` almacena strings
  `msg['content']` [G6:67-68]; en el juez, `episodes` es ese string y
  `episodes[0]`/`episodes[1:]` seleccionan primer carácter y resto
  [G6:155-165], no mensaje actual/lista previa. Esto compromete la
  interpretación del score; no se cambió prompt, juez ni runner.

### 6.2 Qué transferencia es admisible

Aplicar exclusivamente el [protocolo existente](METHOD.md#protocolo-de-comparacion-de-benchmarks):
separar pistas motor/retrieval/memoria/respuesta/agente/producto; congelar
dataset/commit/modelos/budget y registrar intentadas, completadas, errores y
abstenciones. No hay nueva rúbrica, umbral, muestreo ni harness en esta tarea.
El fallo del juez descrito impide interpretar sus scores como comparación
validada, no autoriza inventar un reemplazo.

**Portabilidad concreta GI-2:** el juez consume `AddEpisodeResults` de
Graphiti (episodios, entidades, aristas y fuentes), no registros Vanta.
Ese eval comparativo **no es portable directamente** ni se fuerza un DTO
Graphiti sobre el motor. Para retrieval ya existe
`evals/eval_longmemeval.py::run_longmemeval_run:469-535`, con sesiones completas,
IDs y `answer_session_ids` (BENCHMARK-NOTES §1): una futura adaptación del
adapter existente conservaría episodios como fuentes L0, IDs de sesión y
referencias de entidades/hechos sin equiparar sus scores al juez de grafos.
No se construye harness/adapter ni se altera juez o scoring en esta corrección.
FMEA: reusar grupo → corpus acumulado → comparación contaminada; exigir recibo
del destino/estado inicial, baseline usado y fases incluidas. Aceptación
documental: repetición no se describe como limpia sin evidencia de aislamiento;
el defecto de contexto y la incompatibilidad de resultados siguen visibles.

Una futura aceptación de 3.1–3.3 usa casos funcionales con fechas/soportes
conocidos: valida contratos, no calidad de extracción. Retrieval requiere
evidencia anotada y generación QA separada. Importación y reconstrucción,
embeddings, búsqueda, respuesta y juez deben conservar costes separados;
latencia de una query no mide el coste de mantener un grafo.

## 7. Matriz y negativos VantaDB

### 7.1 Capacidades: significado acotado de cada estado

“Existe” acredita código en el snapshot Vanta, no conformidad runtime.
“Parcial” significa que hay piezas pero no se acredita la composición
concreta; “No acreditado” no equivale a inexistencia.

| Capacidad | Vanta en 97c9623… | Evidencia y decisión |
|---|---|---|
| L0/sesiones/procedencia | Existe | `core/conversation/l0_recorder.rs` y `source_message_ids` [V1]; conservar evidencia |
| Dedup y supersession | Existe | `l1_dedup::run_l1_dedup`, escritor y lifecycle [V1][V2]; gate temporal incremental |
| Fecha de validez en write | Existe | `put_record`/`record_to_data` materializan valid_at [V1]; no anunciar temporalidad desde cero |
| Separación efectiva de tiempo del hecho y conocimiento en todo recall | Parcial | Hay timestamps y supersession; no se acredita álgebra bitemporal end-to-end; 3.1 |
| Recurrencias disjuntas protegidas en todos los modos dedup | No acreditado | 3.2 necesita análisis de exacto/semántico/lote, no se infiere desde #1865 |
| BM25/HNSW/RRF/filtros | Existe | `src/sdk/search/`, `src/index/`; no reemplazar por Graphiti |
| Receta híbrida efectiva en toda ruta de memoria | Parcial | `RecallConfig.core_search=false` por defecto [V3]; traza de 3.4 |
| Inyección gobernada y taint | Existe | `InjectionPolicy` [V3]; no equivale a ACL universal |
| Context assembly y reportes | Existe | `assemble_with_recall`, `IntegratedContext`, `CompactionReport` [V4] |
| Retirada de una fuente y reconstrucción segura de todos sus derivados | Parcial | Provenance/lifecycle/jobs existen; cierre completo no certificado; 3.3 |
| Log de generación | Existe | `core/memory_generation_log/`; distinto de ledger de entrega al modelo |
| Checkpoint por tarea | Existe | `utils/task_checkpoint.rs::TaskCheckpointManager`; no importar Saga como reemplazo |
| Auditoría de índices | Existe | `src/cli_handlers/index.rs`; ampliar por fuente/proyección, no negar diagnóstico |
| Inspector de intervalos/soportes de 3.5 | No acreditado | Studio existente [S1]; no se acredita la nueva proyección |
| Comparación directa del eval de construcción | No aplicable a registros Vanta sin adaptación | GI-2: `AddEpisodeResults`, grupos persistentes sin reset y baseline reutilizable; usar eval Vanta existente para su propia pista |
| Driver Neo4j y comunidades Graphiti | N/A | Elección de otro producto, no requisito faltante en Vanta |

### 7.2 Negativos sustentados, hipótesis y falsos positivos

| Tipo | Hallazgo local | Qué falta antes de actuar |
|---|---|---|
| Código | Delete→put explícito en `persist_planned` [V1:289-294] | Examinar garantías del caller/SDK y fallo entre operaciones; no anunciar pérdida reproducida |
| Código | Error de marcado puede propagarse después de persistir [V1:345-348] | Contrato de recibo/commit y retry discriminante; no tratar Err como rollback |
| Código | valid_at nace de created_at [V1:547-558] | Determinar semántica por productor; no reinterpretar todo created_at como fecha histórica |
| Código | Core híbrido opt-in en ruta de recall [V3] | Confirmar configuración efectiva en consumidor, no decir que no existe híbrido |
| Hipótesis | Derivado retirado sigue en contexto/cache | Inspección de retirada, hidratación y preview por revisión; no bug atribuido sin ruta |
| Hipótesis separada | RMW global de checkpoint puede perder actualizaciones | `utils/checkpoint::mutate:112-117` del snapshot; seguir callers/serialización; no confundir con delete→put |
| Falso positivo descartado | Vanta no tiene presupuesto, tenancy, dedup ni auditoría | Sí hay código según [VANTA-REFERENCE](VANTA-REFERENCE.md); los deltas son de composición |
| Falso positivo descartado | #1876 demuestra todos los reads rotos en el clon Graphiti | Decorador [G14] y helpers/MCP directo [G16] contradicen esa generalización; permisos y ejecución siguen pendientes |

No se extrapola desde una incidencia de FalkorDB a seguridad WAL, recovery
o authz de Vanta. Cada frontera requiere evidencia propia. La relevancia
del contraste está en especificar revisiones y resultados efectivos, no en
sumar un motor temporal externo.

## 8. UI y Studio

### 8.1 Producto realmente localizado

El árbol leído acredita biblioteca Python, FastAPI y MCP, no un escritorio
Graphiti propio. REST expone `/search`, `/get-memory`, `/episodes/{group_id}`
y `/entity-edge/{uuid}` en `routers/retrieve.py:16-62` [G13].
Esos endpoints no constituyen visor de grafo ni prueban permisos.
Los clientes de bases externas y el producto alojado Zep son superficies
distintas: no se auditó su fuente ni se los contabiliza como UI del clon.
No se afirma ausencia absoluta de ejemplos visuales fuera de la muestra.

### 8.2 Integración concreta con el Studio existente

Relectura con `git show` del Studio separado
`0ad38c542b39a589035b0e17fdedb4c09dca24b9` [S1]:
`feature-freez/desktop/src/App.tsx:17-64` monta `WorkspaceShell`.
No existe `app/routes.ts` en ese árbol: se retira la atribución del codec
`?s/?ns/?inspect` al pin; no se sustituye por bytes dirty de `desktop/...`.
Los cuerpos de `feature-freez/desktop/src/components/` sí acreditan
`memory/MemoryLens.tsx:381-409,488-550` (genlog/anchor/sesión),
`activity/ActivityPanel.tsx:52-89` (audit/errores) y
`lens/retrieval/RetrievalLens.tsx:138-164,193-228` (search/contexto/audit).
Son seams de componentes existentes, no una navegación temporal ya implementada.
Fuentes inmutables de esos cuerpos (relectura 2026-11-07):
[MemoryLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L550),
[ActivityPanel](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L52-L89),
[RetrievalLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228).

Flujo propuesto para 3.5, usando esos seams:

1. Abrir registro en `memorias` con conexión/namespace/ID y revisión.
2. Mostrar separado “cuándo fue cierto”, “cuándo se conoció” y “estado de
   proyección”; Unknown visible en vez de un timestamp sintetizado.
3. Expandir soporte autorizado al original retenido, o explicar retirada,
   retención vencida o referencia sin resolver sin filtrar otros tenants.
4. Desde `retrieval`, mostrar receta solicitada/efectiva y clase de evidencia;
   desde `actividad`, proponer vínculo al recibo de reconstrucción cuando el
   backend lo exponga; no hay JobChip/jobs acreditados en este pin.
5. Vista read-only primero. Acciones de editar o retirar necesitan revisión
   esperada y confirmación humana; el frontend no decide cuál hecho es cierto.

FMEA de UI: respuesta tardía tras cambio de conexión, preview cacheado tras
revocación, razón de contradicción con datos sensibles y score dibujado como
probabilidad. Mitigación: reset/cancelación por scope, reautorización al abrir,
códigos de motivo y etiqueta de score. Aceptación futura de la navegación
propuesta: conservar identidad sin mostrar caché de otro namespace; failed/stale no
se representan como “sin memoria”.

## 9. Cobertura, fuentes y cierre

### 9.1 Inventario de lectura y enlaces inmutables

Todas las referencias G fijan
`747b480ec6b12ecb718f1e2bc588b20ee36ea9c2`. Los rangos son evidencia de
superficies concretas; el enlace a un rango amplio no afirma lectura
exhaustiva de todos los drivers ni todos los prompts.

| Ref. | Archivo/símbolo/rangos revisados | Cobertura/laguna |
|---|---|---|
| G0 | `pyproject.toml:1-100`, LICENSE | Stack/licencia; dependencias transitivas no auditadas |
| G1 | `graphiti.py::_resolve_request_scope`, `add_episode:1304-1587` | Orquestación/ownership; no transacción de todos los drivers |
| G2 | `edges.py::EntityEdge:263-291` | Modelo de hechos/tiempos/soportes |
| G3 | `edge_operations.py:715-786,834-855,864-1035` | Invalidación, datado, dedup; no calidad de extracción |
| G4 | `search/search.py:70-255` | Query vacía, embedding condicional, brazos, trazas |
| G5 | `search_config.py:32-139` | Métodos/rerankers/resultados tipados; no equivalencia de scores |
| G6 | `tests/evals/eval_e2e_graph_building.py:17-181` | Runner completo; dataset completo no auditado |
| G7 | `tests/evals/eval_cli.py:1-40`, `prompts/eval.py:30-46,123-170` | CLI y contrato del juez; sin modificaciones |
| G8 | `nodes.py::Node/EpisodicNode/EntityNode/CommunityNode/SagaNode` en 282-334,499-506,727-732,907-922 | Campos y tipos; no todas las operaciones CRUD |
| G9 | `graphiti.py::_process_episode_data:984-1095` | MENTIONS, bulk, saga; raw opcional |
| G10 | `graphiti.py::_extract_and_resolve_edges:916-983` | Remapeo/resolución; LLM externo |
| G11 | `graphiti.py::remove_episode:2174-2251` | Soporte plural, borrados separados, rebuild y error |
| G12 | `graphiti.py:1250-1302,1912-2018` | Decoradores y búsqueda simple/ampliada |
| G13 | `server/graph_service/routers/retrieve.py:1-62` | REST leído; auth no acreditada |
| G14 | `decorators.py::handle_multiple_group_ids:29-132` | Routing Falkor uno/múltiples grupos |
| G15 | `graphiti.py::build_communities:1866-1910` | Retiro previo/build/save, no clustering exhaustivo |
| G16 | `mcp_server/src/graphiti_mcp_server.py:188-219,695-855` | Routing de get/delete y episodios; no auditoría completa de transporte/auth/cache |
| V1–V4 | Fuentes Rust del snapshot 97c9623… enlazadas abajo y [referencia](VANTA-REFERENCE.md) | Lectura histórica, no reauditoría del HEAD concurrente |
| S1 | `feature-freez/desktop/src/App.tsx:17-64` y componentes de §8.2 al SHA completo | Cuerpos vía git show; codec/rutas de navegación no acreditados en el pin |

### 9.2 Cobertura de los diez ejes y división futura

| Eje contractual | Lugar donde se responde |
|---|---|
| 1 Recomendaciones/contexto | §0 y §2 |
| 2 Extracción Rust | §1, §3.1–3.6 |
| 3 Issues/riesgos upstream | §4 |
| 4 Prerrequisitos Vanta | §5.1 y dependencias por ficha |
| 5 Qué no trasladar | §5.2 |
| 6 Advertencias metodológicas | §0.1, §4.1 y §6 |
| 7 Benchmarks | §6.1–6.2 |
| 8 Matriz de capacidades | §7.1 |
| 9 Negativos Vanta | §7.2 |
| 10 UI/Studio | §8 |

**Split plan (>300 líneas):** mantener este informe como evidencia autónoma;
si se aprueban specs activas, separar contrato temporal/identidad (3.1–3.2),
soportes/retirada (3.3), traza de recuperación (3.4) e inspector Studio (3.5).
No crear los archivos ni tareas en esta investigación. Conservar enlaces al
SHA, las exclusiones y relaciones con derivados versionados/diagnóstico ya
estudiados; no duplicar candidatos transversales por cambiarles el nombre.

**Lagunas:** auth y lifecycle completo de MCP/cache de drivers,
transacciones por driver, carreras reales de remove_episode, precisión de
extracción natural y coste de modelos, datos completos del eval, clientes
Zep y UI de bases externas. Quedan manuales los casos adversos de las fichas,
los validadores documentales acotados y cualquier medición del protocolo.
No se ejecutó ningún runner, prueba, build, instalación ni servicio; no se
actualizaron clones, metadata, contratos o backlog ni se hizo git add/commit/push.

**Conclusión:** el mayor valor es un contrato de **ocurrencia temporal +
soportes + publicación por revisión**, complementado por diagnóstico de
lectura. No se acredita ventaja de calidad/latencia por adoptar grafos ni
necesidad de reemplazar las primitivas Rust existentes.

**Cierre de autoría por archivo:** se conserva la investigación y las cinco
fichas; se completa el read/delete MCP que faltaba y se corrige el negativo
histórico de #1876. Fuentes públicas de la ampliación: página #1876 leída,
además de las APIs previamente registradas en §4.1. No hubo reproducción,
revisión independiente ni aceptación final. No requiere otro candidato Rust:
el contrato de scope por operación pertenece a 3.3/3.4 y al enforcement
existente de Vanta, no a un nuevo router de bases.

[G0]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/pyproject.toml#L1-L100
[G1]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/graphiti.py#L1304-L1587
[G2]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/edges.py#L263-L291
[G3]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/utils/maintenance/edge_operations.py#L715-L1035
[G4]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/search/search.py#L70-L255
[G5]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/search/search_config.py#L32-L139
[G6]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/tests/evals/eval_e2e_graph_building.py#L17-L180
[G7]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/tests/evals/eval_cli.py#L1-L40
[G8]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/nodes.py#L282-L334
[G9]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/graphiti.py#L984-L1095
[G10]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/graphiti.py#L916-L983
[G11]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/graphiti.py#L2174-L2251
[G12]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/graphiti.py#L1912-L2018
[G13]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/server/graph_service/routers/retrieve.py#L1-L62
[G14]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/decorators.py#L29-L132
[G15]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/graphiti_core/graphiti.py#L1866-L1910
[G16]: https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/mcp_server/src/graphiti_mcp_server.py#L188-L219
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L112-L620
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_dedup.rs#L112-L250
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L133-L335
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L150-L264
[S1]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/App.tsx#L17-L64

**Recibo de corrección:** [GI-1/GI-2](reviews/memory-code-remaining.md#graphiti)
propagados a eval, matriz, seam y aceptación UI. Pendiente de closure
independiente; no aceptación ni resultado runtime.
