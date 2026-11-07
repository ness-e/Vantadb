---
title: "LlamaIndex: caché de transformación y expansión jerárquica verificable"
kind: research
status: proposed
description: "Análisis estático de ingesta, memoria y retrieval de LlamaIndex con extracciones acotadas para VantaDB"
tags: [research, memory, engineering]
---

# LlamaIndex: caché de transformación y expansión jerárquica verificable

## Índice

- [0. Identidad, mapa, modelos y trazas](#0-identidad-mapa-modelos-y-trazas)
- [1. Recomendación y arquitectura](#1-recomendación-y-arquitectura)
- [2. Evidencia y extracciones mínimas independientes](#2-evidencia-y-extracciones-mínimas-independientes)
- [3. Riesgos, defectos e hipótesis](#3-riesgos-defectos-e-hipótesis)
- [4. Prerrequisitos Vanta](#4-prerrequisitos-vanta)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Método, licencia y cobertura](#6-método-licencia-y-cobertura)
- [7. Benchmark upstream realmente leído](#7-benchmark-upstream-realmente-leído)
- [8. Matriz frente al estado existente](#8-matriz-frente-al-estado-existente)
- [9. Carencias sugeridas, no bugs certificados](#9-carencias-sugeridas-no-bugs-certificados)
- [10. UI, escritorio y evidencia externa](#10-ui-escritorio-y-evidencia-externa)
- [11. Inventario ampliado y cierre](#11-inventario-ampliado-y-cierre)

## 0. Identidad, mapa, modelos y trazas

**Corte:** `run-llama/llama_index`, clon
`C:\Users\Eros\llama_index`, SHA completo
`444fdf0684707de62ebb447d658dba137dd9cb99`, confirmado mediante lectura
Git sin actualizar el checkout. Captura original 2026-11-07 y 52.837
estrellas/MIT conservadas en [github-candidates.json](github-candidates.json);
política shallow, rama única, sin tags/submódulos/LFS según
[clone-manifest.json](clone-manifest.json). No se recalcula metadata.
La licencia raíz MIT [LL] permite estudiar contratos, no evita revisar
licencias por paquete, conectores, corpus, modelos y servicio remoto.

Este documento conserva las candidatas A/B anteriores y las completa con
clasificación de ingesta, receipts por etapa, proyección de metadatos y
evidencia de copia recuperable durante compactación. No sustituye esas
ampliaciones por una ficha nueva.
La referencia comparativa **ahora** es
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612` [V4–V10], no el snapshot
histórico `c561158`. Los issues se consultaron públicamente el 2026-11-07;
no son observaciones de runtime propio.

### 0.1 Mapa y superficies inspeccionadas

| Área | Ruta / fuente fijada | Responsabilidad y límite |
|---|---|---|
| Núcleo Python | `llama-index-core/pyproject.toml:35-87` [L0] | Python ≥3.10, Pydantic, fsspec, SQLAlchemy, tokenización, HTTP y workflows |
| Modelos | `core/schema.py:207-318,337-359,791-818` [L9][L10] | Nodos, relaciones, modos de metadata, identidad de texto |
| Ingesta | `core/ingestion/{pipeline,cache}.py` [L1–L3][L11][L12] | Reader→transformación→caché→almacenes |
| Índice | `core/indices/{base,vector_store/base}.py` [L13][L14] | Insert/refresh/delete y proyección hacia backend |
| Almacenamiento | `core/storage/storage_context.py:149-202` [L15] | Composición/persistencia de docstore, índice, grafo y vectors |
| Retrieval | `core/retrievers/auto_merging_retriever.py` [L4][L5] | Hidratación de padre/vecino y sustitución iterativa |
| Construcción de jerarquía | `core/node_parser/relational/hierarchical.py:14-229` [L20] | Lista plana de todos los niveles; enlaces padre/hijo |
| Contexto | `core/memory/memory.py:339-544` [L6][L16] | Estimación multimodal y productores de bloques |
| Memoria/archivo | `core/memory/memory.py:546-845`; `core/storage/chat_store/sql.py:372-406` [L22], [L23] | ACTIVE→ARCHIVED y fan-out posterior; no commit de todos los receptores |
| Evaluación | `core/evaluation/{benchmarks/beir,retrieval/*}.py` [L7][L17–L19] | BEIR delegado y métricas por IDs propias |
| Integraciones | `llama-index-integrations/`, `llama-index-packs/` | Mapa del monorepo, no auditoría de cada plugin |
| Producto externo | README [L8] | LlamaCloud/LlamaParse; no código del servicio acreditado |

Las rutas abreviadas `core/` comienzan en
`llama-index-core/llama_index/`. El framework es una biblioteca de aplicación,
no una base transaccional ni un runtime obligatorio de Vanta.
API programática: `IngestionPipeline.run/arun`, `IngestionCache`,
`VectorStoreIndex`, `refresh_ref_docs`, `delete_ref_doc`, retrievers y
memoria. La API HTTP del servicio externo no hereda garantías de estas clases.

### 0.2 Modelo, formatos e identidades

`BaseNode.id_` genera UUID por defecto [L9]; reingerir el mismo texto con
nuevos IDs no es actualización por identidad. `SOURCE`, `PREVIOUS`, `NEXT`,
`PARENT`, `CHILD` son relaciones tipadas hacia `RelatedNodeInfo`; su presencia
no valida un árbol ni que los nodos compartan revisión/tenant.
`ref_doc_id` identifica fuente; node ID, hash, vector ID y sesión no son
sinónimos. `TextNode.hash` usa SHA-256 de texto + representación de metadata
[L10:801-803], distinto del hash de transformación por secuencia de nodos [L1].

Formato central Pydantic/JSON: texto, embedding opcional, metadata plana,
relaciones, información de tipo. `IngestionCache` guarda nodos serializados
por `doc_to_json` en KV y los reconstruye por `json_to_doc` [L11:27-46].
La caché usa collection configurable; no constituye política de autorización.
`StorageContext.persist` invoca stores secuencialmente [L15]; los vector
stores pueden ser externos. No hay manifest de commit conjunto en ese tramo.
No se auditó pickle, cada serializer ni cada implementación de store.

`MetadataMode::{ALL,EMBED,LLM,NONE}` decide qué metadata entra en el texto;
las listas de exclusión de embedding y LLM son separadas [L9:337-359].
Excluir una clave del prompt **no la borra** del docstore, caché o logs.
`NodeWithScore` transporta puntuación del retriever; el padre de auto-merge
recibe media de hijos, no distancia ANN medida. Chat y memoria por bloques
son estado conversacional/contextual, no checkpoint durable de ejecución.
LLM/embeddings dependen de las transformaciones y retriever elegidos:
se pueden suministrar vectores o pasos puros; un default de Settings puede
activar proveedor. No ejecutar/importar el paquete para comprobar defaults.

### 0.3 Traza de escritura, commit e índice

1. `_prepare_inputs` reúne documentos/nodos y readers;
   `run:572-604` decide estrategia efectiva. Con docstore sin vector store,
   upsert/delete cae a duplicates-only con warning sin cambiar la
   configuración del objeto [L3]. No llamarlo upsert aplicado.
2. `_handle_duplicates:451-467` escribe hash **durante clasificación**;
   `_handle_upserts:469-507` lee hash por ref_doc_id y borra docstore/vector
   anteriores si cambió. `UPSERTS_AND_DELETE` interpreta entrada como snapshot
   completo y elimina fuentes ausentes. Una página parcial no es ese contrato.
3. Transformaciones consultan caché por etapa; en miss ejecutan y guardan.
   La clave concatena contenido `ALL` y `str(to_dict())` normalizado por regex
   [L1]. No es framing canónico ni acredita todo parámetro pasado por kwargs.
4. Con workers, usa pool `spawn`, divide lotes y fusiona cache entries en
   padre (`606-636`). Sin workers es secuencial; cambiar lote puede cambiar
   clave y reutilización. No inferir caché compartida segura entre procesos.
5. `vector_store.add` recibe solo nodos con embedding (`650-653`).
   `_update_docstore` sucede después (`655-660`). El retorno de `run` no es
   una transacción conjunta ni prueba durabilidad del backend remoto.
   Fallo entre ambos exige reconciliar, no repetir ciegamente.
6. El camino de índice `VectorStoreIndex._add_nodes_to_index` [L14:220-258]
   añade vectores, recibe nuevos IDs y almacena copias sin embedding cuando
   el vector store no guarda texto o hay override; image/index nodes son
   excepciones. Evita duplicación de vector, pero introduce ownership de
   texto condicionado por capacidades del backend.
7. Persistencia explícita [L15:182-202] escribe docstore, index store, graph
   y cada vector namespace por separado. Caída intermedia no equivale a
   snapshot consistente. No se inspeccionó fsync ni atomización de cada store.

Async `arun` [L12:693-882] repite decisión→transformación→async_add→docstore;
los workers se reúnen mediante `asyncio.gather`. Hay hash anticipado también
en `_ahandle_duplicates:698`. Cancelar un await no revierte stores externos
o procesos que ya actuaron; el esquema no acredita cancelación transaccional.

### 0.4 Traza de consulta y contexto

`AutoMergingRetriever._retrieve` llama al vector retriever, rellena huecos
entre vecinos y agrupa hijos por padre [L4][L5]. Hidrata desde docstore;
compara proporción de hijos y, si supera el threshold upstream, reemplaza
hijos por padre. Repite mientras hubo cambios y ordena por score.
No contiene detección de ciclos en ese bucle leído. Una relación importada
hostil necesita validación; no se afirma que todo parser genere ciclos.
Padre no resoluble puede fallar la lectura; no hay fallback universal leído.
El logger INFO incluye extracto de texto aunque verbose sea falso.

`Memory._get_memory_blocks_content` [L16:446-479] obtiene bloques por prioridad
con `session_id`; tipo no admitido produce `ValueError`.
`_estimate_token_count` cuenta texto con tokenizer y usa estimaciones para
imagen/audio/vídeo/documento; ignora CachePoint/ToolCallBlock en ese bucle,
cuenta additional_kwargs textualizados. No certifica tokens de wire.
Trunca bloques no protegidos [L6]; prioridad cero puede dejar overflow.
El ensamblador final de host conserva responsabilidad sobre permisos,
presupuesto, tool pairs y coste de ejecutar productores externos.

### 0.5 Traza de refresh, update y borrado

`BaseIndex.refresh_ref_docs:429-452` [L13] devuelve un booleano por documento:
hash ausente inserta; diferente ejecuta `update_ref_doc`; igual conserva.
No revisa aquí cambios de modelo si el hash documental no cambió.
`update_ref_doc:372-410` declara delete-then-insert, sync y async.
`VectorStoreIndex.delete_ref_doc:423-459` [L14] borra vector, referencias y
docstore opcional; async usa gather y luego actualiza index struct.
Error/cancelación puede dejar efectos parciales; no prueba borrado físico
de todas las copias, cachés o backups. `delete_nodes` y `delete_ref_doc`
son contratos distintos; no usar node IDs como IDs de fuente sin mapping.

### 0.6 De dónde procede la jerarquía y dónde vive cada nivel

`HierarchicalNodeParser.from_defaults` [L20:113-154][L20] compone
`SentenceSplitter` por nivel; defaults observados `[2048,512,128]` y overlap
20, **no parámetros recomendados para Vanta**. Rechaza combinar chunk_sizes
con parser IDs o IDs sin mapa. `_recursively_get_nodes_from_nodes:160-205`
subdivide nivel a nivel y agrega enlaces recíprocos PARENT/CHILD solo desde
el segundo nivel. Devuelve `sub_nodes + sub_sub_nodes`: una **lista plana**
con texto solapado entre niveles, no una tabla de hojas sin duplicación.
`get_leaf_nodes` clasifica por ausencia de CHILD; no valida revisión/ACL,
integridad referencial ni aciclicidad de datos importados.

Para consulta B, el vector retriever puede recuperar hojas y auto-merge
hidrata padres desde **docstore**, no desde cualquier vector store que tenga
`stores_text=True`. Deben publicarse padres y relaciones en el docstore de
ese `StorageContext`; guardar hojas en Vanta no puebla automáticamente aquel
almacén. La regla de ownership [L14] hace especialmente importante esta
distinción. La traza leída acredita construcción y consumidor, no una
integración jerárquica Vanta end-to-end ya configurada.

`node_to_metadata_dict:38-73` [L21] conserva JSON del nodo en `_node_content`,
tipo en `_node_type`, suprime embedding y opcionalmente texto, y duplica
ref_doc_id como `document_id/doc_id/ref_doc_id`. La deserialización
`76-97` reconstruye tipo y texto. Así pueden viajar relaciones y exclusiones
de metadata; **no** constituyen validación ni garantía de que todos los
bindings conserven valores anidados. Una metadata ajena no vacía sin
`_node_content` produce `ValueError` en ese helper. La compatibilidad se
debe validar, no convertir todo metadata JSON en nodo por confianza.

### 0.7 Memoria conversacional: lectura, truncamiento y archivo

`Memory.aget` [L22:624-670][L22] lee mensajes ACTIVE de la sesión, obtiene
productores secuencialmente [L16:459-463], estima tokens, trunca, formatea
template y después inserta contenido en mensajes. No contabiliza otra vez
el mensaje final completo en ese tramo: overhead de formato/wire no queda
certificado por la suma previa. `_insert_memory_content:574-622` hace copia
superficial de la lista y modifica bloques del mensaje system/user elegido;
no inferir de ello persistencia automática del cambio en SQL.

`Memory.aput/aput_messages:811-829` guarda ACTIVE y llama `_manage_queue`.
Al superar su presupuesto de historia, esta selecciona mensajes antiguos,
intenta conservar un turno completo y llama a `archive_oldest_messages`
**antes** de `asyncio.gather(block.aput(...))` para los receptores
(`781-797`). La implementación SQL leída [L23:372-406][L23] selecciona por
timestamp/ID y cambia estado a ARCHIVED con `session.commit()`: no borra
físicamente esos mensajes ni confirma que cada bloque los haya procesado.

Fallo de un receptor o cancelación después del archivo puede dejar estados
diferentes por bloque. La función no aporta en ese tramo un receipt por
receptor o reconciliación. No se afirma que los bloques sean todos remotos ni
que todos fallen, ni que el archivo se pierda. Dos managers de una sesión
requieren examinar ownership/aislamiento: seleccionar un conteo y volver a
consultar «los más antiguos» no acredita identidad causal bajo concurrencia.
`BaseMemoryBlock.aput:151-165` además puede ignorar el offload por
`accept_short_term_memory=False`; éxito del await no significa derivación
creada. Mutar `additional_kwargs["session_id"]` no autentica al llamador.

### 0.8 Adapter Vanta ya implementado: delta real, no integración nueva

Fuente **fijada** Vanta `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`:
`integrations/llamaindex/vantadb_llamaindex/vectorstore.py` [V13].
CodeGraph localizó el símbolo; la lectura `git show <SHA>:<ruta>` fija la
evidencia aunque avance el worktree. No se reindexó ni escribió tooling.

1. `VantaDBVectorStore` declara `stores_text=True`, `flat_metadata=False`;
   abre `vantadb_py.Client` con namespace, read_only y backend (`27-58`).
   Es integración existente, no propuesta.
2. `add:118-150` exige embedding del nodo, guarda texto con MetadataMode.NONE,
   serializa metadata vía [L21], usa node_id como key y llama `put_batch`.
   No genera embeddings ni publica los padres en un docstore externo.
3. `_record_to_node:102-114` reconstruye el envelope existente; `delete:152-169`
   pagina por ref_doc_id y borra cada key. No acreditar una transacción de
   todo ese recorrido ni el borrado de caché/docstore externos.
4. `query:322-381` ofrece vector/híbrido del SDK, modo híbrido cliente
   opcional y MMR. Ya hay fusión: no introducir otro RRF. El modo MMR y el
   híbrido cliente retornan antes del postfiltro complejo del camino común.
5. `_build_vanta_filters:383-406` manda EQ al SDK y separa operadores
   complejos. No consulta `MetadataFilters.condition` en el tramo leído;
   ese hecho del adapter no prueba que el motor carezca de OR o filtros.
6. **Discrepancia estática concreta:** tras construir `nodes`, `ids` y
   `similarities`, el postfiltro modifica **solo nodes** (`377-381`).
   Si rechaza un hit, pueden divergir longitudes y correspondencia posicional.
   Es un caso condicionado al rechazo efectivo; no un fallo ejecutado.
7. `get_nodes:460-471` acepta `filters` pero solo hace gets por node_ids.
   No tratar ese argumento como ACL. Hidratación de padre debe conservar
   permisos/revisión en el host y no solo confiar en el filtro del ANN.

Tests leídos [V14:148-166,221-241][V14] cubren EQ, roundtrip textual y documentan
que top_k=0 vuelve al default 4. Su existencia **no** acredita ejecución,
cobertura de postfiltros complejos ni un bug por el fallback de k=0:
es intención explícita del test. Filtros/tuplas alineadas y supervivencia de
relaciones/metadata se deben resolver antes de atribuir resultados a B o
a una adaptación BEIR. No se modifica código del adapter en este encargo.

## 1. Recomendación y arquitectura

**Dictamen:** extraer contratos de reutilización de transformaciones y de
expansión documental; no hacer del framework Python una dependencia del motor.
Clone leído: `C:\Users\Eros\llama_index`.
SHA congelado: `444fdf0684707de62ebb447d658dba137dd9cb99`.
Snapshot: **52 837 estrellas; MIT**, del [inventario](github-candidates.json).
La preparación y omisiones proceden del [manifiesto](clone-manifest.json).
No se actualizaron contadores. Se añadieron consultas públicas de issues (§3).

El paquete central declara Python >=3.10, Pydantic, SQLAlchemy, fsspec,
tokenización, clientes HTTP y workflows; [L0] acredita dependencias, no su
utilización completa. Las integraciones viven separadas del núcleo.
Es una capa de aplicación que conecta documentos, transformaciones,
docstore, vector store y modelos; no un almacenamiento embebido equivalente.

```text
documentos/nodos → estrategia docstore → transformaciones/cache
               → vector_store.add → actualización docstore
query → vector_retriever → completar vecinos → sustituir hijos por padre
      → repetir mientras cambie → ordenar por score
memoria → contar historia/bloques → truncar por prioridad → formato de contexto
```

El primer flujo es visible en [L1]-[L3]; el segundo en [L4]-[L5].
La caché evita recomputación, pero no vuelve transaccionales los almacenes.
La expansión cambia el contenido recuperado: un padre no es otro hit ANN.
La memoria por bloques [L6] constituye política de contexto, no persistencia
semántica autónoma ni sustituto del ensamblador de Vanta.

**Referencia Vanta:** base fijada `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.
V1–V3 se conservan como antecedentes históricos; V4–V10 son seams revalidados.
El [contrato](METHOD.md#vantadb-como-referencia) y el propietario acreditan ANN,
BM25/sparse/RRF, provenance, sesiones, supersession, dedup, assembler/compaction,
scheduler/checkpoints/ingestors/auditoría y Studio en `develop-desktop`.
Nada de ello se propone como capacidad inexistente.

| Prioridad condicionada | Decisión específica |
|---|---|
| Previa a benchmarks/adopción | Verificar fidelidad del adapter existente (§0.8 y §9); no medir nuevas políticas sobre resultados desalineados |
| Primera, si hay fallos parciales de ingesta | D y semántica honesta de progreso; A añade identidad/reuse sin confundir caché con commit |
| Primera, si se exporta metadata no controlada | E sobre gobernanza existente, antes de nuevas llamadas externas |
| Siguiente, si hace falta sincronización inspeccionable | C read-only; nunca aplicar snapshot-delete a entrada parcial |
| Condicionada a corpus jerárquico autorizado | B tras validar padres, ownership y presupuesto; no beneficio universal sobre todo corpus |
| Incremental de diagnóstico de compactación | F sobre SpillSink existente; independiente de adoptar el framework o una segunda memoria |

## 2. Evidencia y extracciones mínimas independientes

### Registro de lectura

| Ref. | Ruta y rango efectivamente leídos | Evidencia |
|---|---|---|
| L0 | `llama-index-core/pyproject.toml:35-87` | Stack y dependencias centrales |
| L1 | `core/ingestion/pipeline.py:46-110` | Hash de entrada/transformación; get/put de caché |
| L2 | `core/ingestion/pipeline.py:451-507` | Duplicados y upsert con borrado previo |
| L3 | `core/ingestion/pipeline.py:580-662` | Estrategia efectiva, workers, vector store y docstore |
| L4 | `core/retrievers/auto_merging_retriever.py:35-164` | Relación padre/hijos y relleno de vecino |
| L5 | `core/retrievers/auto_merging_retriever.py:166-194` | Iteración hasta estabilizar y orden final |
| L6 | `core/memory/memory.py:481-545` | Truncamiento por prioridad y bloques protegidos |
| L7 | `core/evaluation/benchmarks/beir.py:1-109` | Runner BEIR leído completo |
| [L8] | `README.md:11-32` | Producto externo y composición |
| [LL] | `LICENSE:1-21` | MIT raíz y aviso |

Las rutas abreviadas `core/` están bajo
`llama-index-core/llama_index/`; enlaces inmutables al final.
La tabla no acredita módulos enteros ni las integraciones no recorridas.

### Candidata A — identidad y recibo de transformación

- **Problema:** distinguir un resultado reutilizado de otro recalculado,
  manteniendo identidad de fuente y versión efectiva de procesamiento.
- **Unidad mínima:** descriptor determinista de transformación + clave
  contextualizada + recibo de hit/miss, inspirado en L1.
- **Seam existente:** ingesta/merge de Vanta [V1], que ya agrupa por página,
  conserva `sources` y comunica fallos por entrada.
- **Contrato hipotético, no API acreditada:** `TransformIdentity { source_digest,
  processor_version, options_digest, scope }` y `TransformReceipt { identity,
  reused, output_digest }`.
- **Acción 1:** inventariar identidad/checkpoints actuales antes de añadir tipos.
- **Acción 2:** separar opciones que alteran contenido de opciones de ejecución;
  no serializar punteros, secretos ni un `Debug` inestable como identidad.
- **Acción 3:** incorporar el receipt al resultado de ingesta existente;
  una caché fallida debe conservar el error o la recomputación explícita.
- **Dependencias:** hash estable, serialización canónica, namespace, versión del
  extractor/modelo si participa y política de invalidación.
- **Persistencia:** reutilizar backend/checkpoint; no crear otro docstore.
- **FMEA — identidad incompleta:** dos configuraciones comparten clave y sirven
  contenido obsoleto; evitarlo haciendo explícitos todos los insumos semánticos.
- **FMEA — ámbito omitido:** cache hit puede revelar contenido de otro namespace.
- **FMEA — publicación parcial:** receipt exitoso antes del contenido durable
  convierte un retry en falso «ya procesado».
- **Aceptación propuesta:** misma identidad implica el mismo artefacto validado;
  cambio semántico invalida; fallo no publica receipt de éxito.
- **Aceptación de aislamiento:** ningún hit permite saltarse autorización o
  reutilizar provenance incompatible.
- **Independencia:** no depende de jerarquías, ANN ni de la candidata B.
- **No objetivos:** caché distribuida, nuevos modelos, nuevos parámetros de eval.

### Candidata B — expansión contextual con trazabilidad de sustitución

- **Problema:** explicar por qué un fragmento recuperado se amplió a su sección
  y qué evidencia quedó representada sin duplicar contenido en el presupuesto.
- **Unidad mínima:** relación fuente/fragmento/padre y registro de sustitución;
  tomar L4/L5 como contraste, no sus ratios ni media de scores.
- **Seam existente:** assembler puro y `AssembleOutput` de Vanta [V2];
  `ChatMessage`/`CompactionReport` ya dan contratos tipados [V3].
- **Contrato hipotético:** `ExpansionDecision { source_revision, selected_ids,
  replacement_id, reason, provenance }`, unido al reporte existente.
- **Acción 1:** comprobar qué jerarquía/secciones conserva ya el ingestor.
- **Acción 2:** resolver referencias dentro de la misma revisión y namespace.
- **Acción 3:** proponer expansión como paso opcional anterior al consumo del
  presupuesto compartido; el assembler sigue decidiendo qué cabe.
- **Acción 4:** conservar IDs originales y motivo; no presentar score agregado
  como similitud ANN medida del padre.
- **Dependencias:** padres direccionables, revisión de fuente, límites existentes
  del contexto y garantía de que cada relación proviene de entrada validada.
- **FMEA — ciclo:** padre que apunta a descendiente puede impedir convergencia;
  contrato de árbol validado y visita única, no confianza en JSON importado.
- **FMEA — pérdida de precisión:** un padre autorizado puede contener secciones
  no pertinentes; expansión no debe eliminar el acceso al fragmento original.
- **FMEA — mezcla de revisiones:** referencia vieja expande a documento distinto.
- **Aceptación propuesta:** decisiones reproducibles, sin duplicados padre/hijo,
  respetando presupuesto, permisos y pares tool-call/result ya protegidos.
- **Aceptación negativa:** relación no resoluble no inventa texto ni provenance;
  se conserva el fragmento o se comunica degradación.
- **Independencia:** funciona con documentos ya ingeridos, sin caché A.
- **No objetivos:** cambiar ranking híbrido, importar retriever Python o copiar
  el `simple_ratio_thresh=0.5` observado.

### Contratos adicionales de A/B tras revalidar Vanta

**A:** integrar en `merge_page` [V4] y en chunks deterministas de
`src/wiki/ingestors.rs` [V5], no en el viejo stub de V1. Entradas con
framing, scope autorizado e identidad de modelo; salida de caché lleva
digest, versión de schema y provenance. Error IO/caché corrupta no equivale a
«documento ya ingerido». Concurrencia: calcular fuera de locks y publicar
solo para revisión esperada; un receipt es de artefacto, no de lote entero.
Migración aditiva, legado `Unknown`; recomputación selectiva; rollback
deshabilita reuse y conserva contenido primario. Alternativa simple:
recomputar sin caché hasta demostrar coste. Telemetría hit/miss/error y
versión de procesador sin contenido ni secretos. A no usa prompt ni nuevo
modelo ni promete reutilización entre tenants.

**B:** seam actual `ContextEngine::assemble/assemble_with_recall` [V6] y
`CompactionReport` [V7], no segundo assembler. Contrato de expansión recibe
hits ya autorizados y una vista/revisión de fuente; errores tipados
MissingParent, InvalidRelation, RevisionConflict, Denied. Validar relaciones,
deduplicar cobertura de origen, expandir, luego presupuestar con el
estimador existente. No persistir texto duplicado; reporte aditivo,
relaciones antiguas sin revisión no se usan para expansión automática.
Rollback deshabilita expansión y conserva hits iniciales. Alternativas:
vecinos contiguos sin padre o mostrar enlace sin inyectar sección completa.
Aceptación manual añade padre borrado/revocado durante hidratación,
IDs iguales en namespaces distintos y contexto protegido que excede budget:
resultado explícito, no truncamiento de seguridad ni promesa de cabida.
Ownership B: un resolver autorizado por ejecución, sin cache global de padres
por ID desnudo; cancelar descarta expansiones pendientes y libera referencias.
La expansión es una lectura, no un commit de memoria nueva. La publicación
de un árbol pertenece a ingesta C/D y no puede autorizarse desde un hit.
Agregar casos de árbol plano [L20] con padre no publicado, metadata corrupta
[L21] y desalineación de adapter [V13]; no «arreglar» el ranking para ocultarlos.

### Candidata C — preview de ingesta con semántica de snapshot explícita

- **Caso:** sincronización paginada de una carpeta de conocimiento; el
  operador quiere inspeccionar altas/cambios/borrados sin gastar embeddings
  ni eliminar documentos que faltan solo en esa página.
- **Evidencia:** `_handle_upserts` [L2:469-507] mezcla clasificación y
  mutación; `refresh_ref_docs` [L13:429-452] compara hashes. Requisito público
  #23768 (§3) solicita separar preview; no es implementación upstream.
- **Delta/seam:** Vanta ya separa `plan_write` y `persist_planned`
  [V8:160-176], pero su plan privado L1 es por decisión de dedup.
  `merge_page` [V4] y chunks [V5] son seams de importación documental.
  Delta: plan read-only de sincronización, no otro clasificador LLM.
- **Contrato Rust propuesto:** `IngestPreview { scope, basis_revision,
  input_manifest_digest, mode, added, changed, unchanged, deletion_candidates }`;
  mode distingue delta de snapshot completo. Devuelve plan o errores
  IncompleteManifest, IdentityConflict, Denied, ReadConflict.
  Lista ausente no autoriza borrar; el principal y modo vienen del host.
- **Pasos:** inventariar manifests/receipts actuales; estabilizar IDs
  fuente/chunk; evaluar sin transformaciones ni escritura; mostrar diff;
  al aplicar comprobar basis o recalcular y exigir nueva aprobación para
  borrados adicionales. Mantener mismos comparadores de plan y ejecución.
- **Concurrencia/commit:** preview no reserva estado ni garantiza ejecución
  posterior. Un escritor concurrente invalida basis; no hacer check y write
  separados y llamarlo CAS. La publicación utiliza garantías existentes.
- **Persistencia/deps:** preview mínimo efímero, digest/revisión en receipt
  de ingesta si ya existe; no requiere nuevo store. A solo si se desea
  estimar reutilización. No depende de B. Migración de IDs debe preceder a
  cualquier delete-by-absence; desactivar preview no borra contenido.
- **Alternativas:** mera lista de archivos cambiados evita acceso al motor
  pero no descubre drift de chunks; adapter externo si controla toda la
  ingesta. Rechazar convertir cualquier batch en snapshot completo.
- **FMEA:** página parcial→borrado masivo→modo/manifiesto completo validado;
  ID inestable→alta+baja falsas→mapping estable; preview antiguo→TOCTOU→
  revisión esperada y replan; ruta de fuente manipulada→scope externo→
  resolver identidad bajo raíz autorizada, no desde texto del documento.
- **Aceptación manual futura:** preview no llama embeddings/LLM ni muta
  stores; delta nunca borra ausentes; snapshot completo sí los propone;
  cambio concurrente detectado; ref_doc_id agrupa hijos de una fuente sin
  mezclar tenants. Métricas de decisiones/conflictos, no paths sensibles.
- **No objetivos:** scheduler, algoritmo de dedup alternativo, aprobación
  automática por modelo ni snapshot global multialmacén.

### Candidata D — evidencia de etapa aplicada y reconciliación de ingesta

- **Caso:** proceso añade vectores y falla antes del docstore; reintento
  debe distinguir «transformé», «publiqué», «proyección disponible» y «no sé».
- **Evidencia/unidad:** [L3:648-662], [L12:868-882], [L14:230-258] y
  persistencia secuencial [L15:182-202]; #22638 muestra por qué un hash
  publicado durante decisión no equivale a éxito.
- **Delta/seam:** `TaskCheckpointManager` [V10] ya ofrece progreso at-least-once
  y ownership single-writer; scheduler [V9] ejecuta jobs; [V8] escribe L1.
  Delta: evidencia por fuente/etapa y reconciliación tras outcome desconocido,
  no checkpoint básico ni segunda cola.
- **Contrato propuesto:** `IngestStageReceipt { operation_id, scope,
  source_revision, processor_identity, stage, artifact_ids, outcome }`;
  outcome distingue Applied, Failed, Unknown, PendingProjection.
  `reconcile(receipt)` consulta por IDs/revisión y devuelve evidencia, no
  un retry implícito ni «exactly once».
  Es observación de **una etapa**, no recibo total: varias etapas/receptores
  pueden tener resultados distintos. Para escrituras referencia por operación
  al [C13 MutationReceipt propuesto](CONSOLIDATED.md#c13--mutationreceipt-recibo-de-mutación-multialmacén),
  conservando aceptación, commit, durabilidad y proyecciones independientes.
  `Applied` de transform no acredita commit; `PendingProjection` puede coexistir
  con primario confirmado/durable y otro receptor fallido. La espera cancelada
  postdispatch conserva Unknown salvo evidencia, no Failed/rollback automático.
- **Pasos:** trazar commits de backend real; fijar IDs antes de side effects;
  reutilizar checkpoint para intención; guardar progreso después de efecto
  verificable; en reinicio inspeccionar primario/proyección; reparar solo
  etapas pendientes. Hash de dedup exitoso se publica tras contenido válido.
- **Commit/concurrencia:** caída entre efecto y receipt sigue siendo posible;
  resolver mediante idempotencia/reconciliación, no prometer atomicidad
  distribuida. Un dueño por operación/revisión o publicación condicional;
  [V8:430-446] admite fallo parcial delete-before-put: conservarlo visible.
- **Persistencia/migración:** schema aditivo en checkpoint/resultado existente;
  registros previos sin recibo son desconocidos. No reingestar masivamente
  por falta de receipt. Rollback detiene nuevas etapas y mantiene evidencia.
- **Deps/alternativas:** identidad estable A/C; backend consultable por ID.
  A y C aportan contratos de identidad/preview, no obligación de implementar
  ambas candidatas completas para usar D: reutilizar IDs existentes si bastan.
  C13 sólo gobierna afirmaciones de mutación, no cada transformación pura.
  Si todo cabe en primario transaccional existente, usarlo antes que saga.
  Si el adapter posee todo el flujo, mantener allí coordinación.
- **FMEA:** receipt antes de dato→retry omitido→confirmar evidencia; dato sin
  receipt→duplicado→reconciliar; cancelación→resultado remoto incierto→Unknown;
  borrado entre etapas→resurrección→revalidar vigencia y authz al publicar.
- **Aceptación manual futura:** interrupción antes/después de cada etapa,
  reintentos duplicados, cache hit corrupto y cambio de fuente; un fallo no
  «envenena» hash ni borra evidencia anterior sin estado explícito.
  Observabilidad por etapa/código sin payload, vector o identificador sensible.
- **Mínimo/no objetivos:** semántica veraz de progreso, no consenso,
  transacciones distribuidas o garantía de rollback de proveedores.

### Candidata E — proyección de metadata por destino, no por registro entero

- **Caso:** timestamp/URI/provenance sirven al operador y al filtro, pero un
  secreto operativo no debe enviarse al embedder o modelo por estar en metadata.
- **Evidencia:** `BaseNode.get_metadata_str` y listas
  `excluded_embed_metadata_keys`/`excluded_llm_metadata_keys` [L9].
  La separación por destino es útil; las listas denylist no son la política
  recomendada para entrada desconocida. `ALL` usado por caché [L1] conserva
  frontera distinta.
- **Delta/seam:** `InjectionPolicy`, `RecallConfig` y presupuesto gobernado
  de `auto_recall.rs` [V11] ya existen; `ContextEngine` [V6] ensambla contexto.
  Delta condicionado: descriptor explicable de campos enviados a cada
  destino y versión efectiva, no añadir ACL por prefijos desde cero.
- **Contrato propuesto:** `MetadataProjection { policy_revision, target,
  allowed_fields }`; target Embedding, ModelContext o Diagnostic.
  `project_metadata(record, principal, projection)` devuelve campos más
  razones de omisión sin valores secretos; errores UnknownPolicy/Denied.
  Tipos nuevos hipotéticos, no API implementada.
- **Pasos:** inventariar qué campos se exportan hoy; reutilizar gobernanza;
  aplicar allowlist por destino antes de serializar/red; propagar versión
  a identidad A si cambia embedding; reportar omisiones en diagnóstico.
- **Commit/concurrencia:** no cambia registro primario; cambio de política
  invalida caché de proyección. Revocación se comprueba al envío, no solo al
  recuperar hit. Cache de embedding no habilita acceso a texto revocado.
- **Migración/deps:** política conservadora para metadata desconocida,
  compatibilidad de adaptadores explícita; reindexar solo cuando cambie
  contenido semántico del embedding. Depende de inventario de exportación y
  proveedor; no requiere C/D para diagnosticar.
- **Alternativas/rollback:** mantener metadata fuera de texto y referenciar
  solo source ID; allowlist en adapter si es único exporter. Rollback no
  restaura secretos ya enviados: preservar política de no exportación y
  retirar feature, no hacer fail-open.
- **FMEA:** default ALL→exfiltración→allowlist; diagnóstico→segunda fuga→
  redacción de valores; política mutable durante envío→TOCTOU→revalidación;
  metadato hostil→instrucción→tratarlo como datos, nunca permisos.
- **Aceptación manual futura:** canario sensible ausente de payload a
  embedder/modelo/log, provenance autorizado visible, filtros siguen
  funcionando; cambio de política no reutiliza proyección incompatible.
  No se enviaron canarios ni llamadas de proveedor durante este estudio.
- **No objetivos:** clasificador de PII infalible, cambiar ranking,
  eliminación retroactiva de datos externos o reglas nuevas de evaluación.

### Candidata F — evidencia de copia recuperable durante compactación

- **Caso agente:** el contexto puede seguir funcionando tras compactar, pero
  el operador necesita saber si el original se conservó realmente para
  inspección/rellamada o si solo queda el stub.
- **Fuente/unidad mínima:** orden archivo→fan-out de [L22:781-797][L22] y commit
  ARCHIVED en [L23:372-406][L23]. Lección: «retirado del contexto activo»,
  «archivado» y «aceptado por cada derivación» no son un estado único.
  No copiar su waterfall ni el presupuesto.
- **Delta/seam revalidado:** Vanta **ya tiene** `SpillSink`, `DbSpillSink`,
  `SpillStorage` [V16] y cursor persistente de offload [V17]. El trait devuelve `()`;
  DbSpillSink cuenta nuevos guardados y registra/absorbe errores
  (`spill.rs:249-259`). Propuesta: hacer visible esa evidencia en el reporte
  actual; no crear offloader, almacén de chat ni cursor nuevos.
- **Contrato Rust propuesto, no existente:** extensión aditiva
  `RecoveryCopyReport { attempted, stored, already_present, failed, unknown }`
  asociada a la ejecución de ensamblado y scope autorizado. Si se requiere
  detalle privado: identidad de mensaje/artefacto y outcome tipado; nunca
  texto original por defecto. `stored` significa éxito de API bajo su
  garantía declarada, no fsync ni retención perpetua.
- **Pasos:** inventariar métricas que ya publica el caller; recoger outcomes
  en el sink concreto, no IO en assembler puro; asociar resumen al reporte;
  exponer estado degradado al host/Studio; solo después decidir si algún
  flujo exige copia recuperable antes de sustituir contenido.
- **Commit/ownership/cancelación:** un dueño de reporte por ejecución;
  conservar `Unknown` si no se puede confirmar la escritura al interrumpir.
  No avanzar un cursor como si fuera acuse de todos los receptores:
  [V17:88-96][V17] es RMW del estado, no transacción multi-receptor. No se
  demuestra aquí un avance erróneo del cursor en Vanta.
- **Dependencias/relaciones:** usa IDs/scope existentes y [V7]; comparte la
  semántica de outcome de D, pero D es ingesta documental y F es copia de
  recuperación de mensajes sustituidos. No obliga a A/B/C ni a derivaciones
  LLM. Mantener un solo vocabulario de evidencia si ambas se aprueban.
- **Migración/alternativas/rollback:** primero reporte efímero opcional,
  sin reindexación ni reescritura de originales. Alternativa: contador actual
  + warning si basta para operación; rechazar un ledger durable nuevo sin
  caso de uso. Desactivar reporte no cambia compactación ni borra copias.
- **FMEA:** compaction exitosa + spill fallido→falsa recuperabilidad→outcome
  degradado; dedup→cero nuevos→falso fallo→separar already_present;
  dos tareas→evidencia mezclada→correlación por ejecución; retención/GC→
  copia posteriormente ausente→revalidar al recuperar, no prometer eternidad.
- **Aceptación manual futura:** sink sano, dedup, error IO y cancelación
  producen estados distinguibles sin payload; compaction/report no confunde
  ninguna de esas situaciones con copia durable confirmada. Permisos se
  revalidan al recuperar original. No se ejecutaron los casos.
- **Mínimo/no objetivos:** evidencia de la garantía actual; no cambiar por
  defecto fail-open a fail-closed, ni exigir todos los receptores, ni diseñar
  otra política de presupuesto. La política de bloqueo queda decisión abierta.

## 3. Riesgos, defectos e hipótesis

**Hechos estáticos:** L1 construye el hash concatenando contenido y una
representación textual del transformador; elimina patrones de direcciones.
Eso describe su implementación, no una especificación canónica interoperable.
L2 borra versiones anteriores en docstore/vector store antes de L3.
L3 añade vectores y después actualiza docstore; no hay transacción conjunta
visible en ese tramo. No se afirma que todo backend carezca de transacciones.

**Hipótesis discriminantes:** una excepción entre pasos puede dejar visibilidad
parcial; un retry de ingesta necesita reconocer qué paso ocurrió.
No se reprodujo el fallo. La ampliación consultó los siguientes issues.
L4 calcula score del padre promediando hijos y registra texto truncado en logs.
Es una frontera de privacidad real de ese código; la exposición concreta
depende del contenido sensible y de quién pueda leer esos logs.

L6 preserva prioridad cero y puede devolver contenido aún sobredimensionado
cuando sólo quedan bloques protegidos. No confundir esa política con prueba
de que el proveedor aceptará el contexto final.
No se certifica un defecto de Vanta por analogía con ninguna de estas rutas.

### Incidencias públicas leídas el 2026-11-07

| Incidencia / estado observado | Reporte, código y aplicabilidad |
|---|---|
| [#22638: docstore duplicates_only strategy poisoned hash](https://github.com/run-llama/llama_index/issues/22638), **open**, creada 2026-08-11T13:20:29Z, actualizada 2026-08-28T03:47:27Z | Reporta core 0.14.23: transformación falla tras `set_document_hash`, retry omite documento. El SHA fijado aún muestra escritura anticipada sync [L2:463] y async [L12:698], aunque tiene `_update_docstore` posterior. Evidencia estática consistente con mecanismo, no ejecución propia ni prueba para todos los stores |
| [#23768: read-only preview of IngestionPipeline upsert changes](https://github.com/run-llama/llama_index/issues/23768), **open**, creada 2026-10-28T15:07:23Z, actualizada 2026-10-31T12:07:31Z | Solicita clasificación separada sin transformaciones/escritura, delta vs snapshot y respeto de ref_doc_id. Su propio texto advierte que preview no garantiza aplicación posterior. Es requisito, no fix ni función ya implementada; fundamenta C |

**Validaciones discriminantes futuras:** para #22638, docstore limpio +
transformación que falla, inspeccionar hash/nodos/vectores, reintentar con
transformación sana; controles sin docstore y con upsert efectivo, sync/async.
Separar hash anticipado de delete-before-put: tienen ventanas distintas.
En Vanta, [V8] planifica antes de persistir; no atribuirle automáticamente
hash poisoning. Revisar específicamente etapa que publica progreso D.
Para #23768, comprobar cero side effects y replan tras cambio concurrente;
batch parcial no puede proponer bajas si mode=delta.

**Acceso:** `websearch.search` DuckDuckGo para issues ingestion/docstore
devolvió cero resultados; `metasearch.search_web` DuckDuckGo/Bing también.
Fallback positivo `websearch.fetchWebContent` a API pública
`search/issues?q=repo:run-llama/llama_index+is:issue+ingestion+docstore&per_page=3`.
Se leyó cuerpo completo de #23768 en primer item y GET individual completo
de #22638. La búsqueda se truncó después: no se declaran leídos los 116 hits.
La lectura de comentarios de #22638 se truncó; el primero es un bot con
conjeturas históricas y texto promocional. No se ejecutó ninguna instrucción
contenida allí, ni se toma como confirmación de PRs/fixes. No se auditó la
historia de reversiones ni se atribuye corrección sin evidencia.

**Seguridad:** URLs/readers/metadata de documentos son entrada no confiable.
Un loader HTTP puede ampliar superficie SSRF según adapter, pero esta
cobertura no acredita un exploit. Namespaces del KV no son autorización.
Texto de padres y recuerdos puede contener prompt injection: no autoriza
tools ni borrado. Logs INFO de [L4] constituyen salida de contenido por
configuración del logger; no se certifica acceso de terceros a esos logs.

## 4. Prerrequisitos Vanta

1. Revalidar receipts, checkpoints y reportes actuales antes de abrir tareas;
   los commits concurrentes pueden cubrir ya A o B.
2. Preservar `sources`, supersession y dedup: caché no sustituye idempotencia.
3. Separar bytes originales, texto derivado e identidad de modelo/procesador.
4. Confirmar que expansión y compactación comparten presupuesto existente;
   no crear un segundo contador ni alterar protección de tool pairs.
5. Acordar cómo se expresa un paso fallido y qué garantía de durabilidad tiene.
6. Mantener diagnósticos sin contenido sensible por defecto.

Las dependencias son **por candidata**, no gate universal: A requiere
identidad/procesador; B relaciones/revisión y presupuesto compartido; C modo
delta/snapshot y basis; D evidencia de commit y reconciliación; E inventario
de exportación. Diagnósticos read-only C/E no exigen resolver una transacción
multialmacén. Para D activo sí hay que respetar la ventana parcial de
`apply_dedup_batch` [V8:390-456]; no mezclarla con recovery del WAL.
`TaskCheckpointManager` [V10:11-27,113-121] documenta single-writer por
tarea y resume at-least-once: no usarlo como scheduler distribuido o CAS.

## 5. Qué no trasladar

No importar Python/Pydantic/workflows al motor ni mantener dos pipelines.
No trasladar la concatenación textual como formato durable de identidad.
No adoptar borrado previo entre almacenes como contrato transaccional.
No promediar similitudes para cambiar el ranking sin justificar su semántica.
No reproducir logs con extractos de memoria del usuario.
No convertir la estrategia de bloques protegidos en promesa de cabida absoluta.

No trasladar UUID aleatorio como única identidad de fuente reimportable,
snapshot-delete sobre páginas parciales, exclusiones de metadata como ACL,
`str(dict)` como wire identity interoperable ni `persist()` como commit
conjunto de servicios. Reutilizar un adapter actual antes que incorporar
todo LlamaIndex; añadir framework implicaría más proveedores, defaults,
serializadores y superficie de actualización para capacidades ya existentes.

## 6. Método, licencia y cobertura

Lectura estática del checkout fijado; consultas públicas de issues únicamente.
Ninguna ejecución de código upstream, instalación, build, test, benchmark,
app ni servicio. No se inició ni detuvo runtime.
Cobertura positiva: L0-L8 y seams históricos V1-V3; no auditoría exhaustiva.
Omitidos: agentes completos, conectores, callbacks, stores remotos, packaging,
seguridad de cada integración, notebooks y consistencia de backend end-to-end.
Ampliación leída: modelos, persistencia, índice/refresh/delete, async y métricas
por IDs (§11); no confundir la cobertura original L0–L8 con la renovada.
Historial shallow, otras ramas/tags, submódulos y LFS según manifiesto.

MIT de inventario concuerda con la licencia raíz leída [LL].
Copiar fragmentos sustanciales exige conservar aviso; revisar además archivos,
datos y dependencias concretos. Traducir a Rust no borra obligaciones.
Las propuestas anteriores describen contratos nuevos hipotéticos, no patches.
Los hallazgos históricos Vanta permanecen antecedentes, no bugs vigentes.

## 7. Benchmark upstream realmente leído

### 7.1 BEIR

**L7 / `BeirEvaluator.run`: runner programático, no CLI autónoma.**
Descarga datasets BEIR si faltan; default `nfcorpus`, split `test`.
Carga corpus, queries y qrels mediante `GenericDataLoader`.
Recibe `create_retriever(documents)` y postprocesadores opcionales.
Conserva `doc_id` en metadatos y transforma hits en mapa documento→score.
Solicita NDCG, MAP, recall y precision para k=3/10 por defecto.
Esos valores describen el runner existente; aquí no se proponen ajustes.

Dependencias: paquete externo `beir`, tqdm, red/cache de datasets y las del
retriever elegido; modelos/embeddings dependen de esa función del llamador.
La deduplicación por clave de documento puede colapsar varios chunks.
El constructor contiene un `try: pass`; no comprueba realmente la importación
BEIR al construir el evaluador: los imports efectivos ocurren al usarlo.
No mide latencia de motor, p99, RSS ni recuperación de sesiones.
No implementa juez generativo en este runner.
La licencia del código no acredita licencia de corpus BEIR o modelos.

No se leyó como runner HotpotQA ni se atribuyen métricas a ese archivo.
ParseBench/ExtractBench aparecen anunciados en README [L8]; no se consideran
resultados de L7 ni evidencia ejecutada de este checkout.
Comparación futura remite al [protocolo](METHOD.md#protocolo-de-comparacion-de-benchmarks).
No se diseñaron prompts, jueces, thresholds, sampling ni configuraciones nuevas.

**Denominadores y errores:** [L7] delega las fórmulas/agregados a
`beir.retrieval.evaluation.EvaluateRetrieval`, dependencia no fijada/leída
en este clon. Se conoce la población que entrega: queries del split test,
qrels y un diccionario doc_id→score por pregunta; varios chunks de mismo
documento colapsan y prevalece el último asignado, no necesariamente máximo.
La política exacta BEIR de gold vacío/queries sin relevancia queda pendiente;
no atribuirle una fórmula desde conocimiento externo. Excepción de retrieval,
metadata `doc_id` ausente o score inválido no se maneja por pregunta: puede
abortar sin reporte de completitud. No hay abstención generativa.
La descarga/cache se prepara fuera del loop retrieval; no mide tiempos ni
costes en ese loop. Embeddings/modelo/tokenizer, concurrencia efectiva,
budgets y caches del retriever vienen del llamador, no están congelados por
este evaluador. No se atribuye semilla o warmup inexistente.

**Adaptación Vanta bajo METHOD:** adapter `create_retriever(documents)` hacia
ingesta/retrieval actual [V5][V12], conservar doc_id de corpus y separar
node/chunk ID de documento. Mantener corpus/query/qrels separados: qrels no
entran en embedding ni selección. El postprocesado B altera unidad de
resultado y debe conservar mapping; no contar padre y sus hijos dos veces.
Preservar runner/evaluador y métricas upstream; registrar por fuera
intentadas/completadas/errores conforme a METHOD, sin nueva rúbrica.
Esta pista es retrieval, no memoria conversacional, QA ni éxito de agente.
BEIR/modelos/red pueden descargar datos y usar APIs pagadas según retriever;
reset/borrado quedan bajo contrato del adapter y sobre datos desechables.

### 7.2 Evaluación programática por IDs: HitRate y MRR

Cadena leída: `BaseRetrievalEvaluator.evaluate`→`aevaluate`
[L17:103-166]→`RetrieverEvaluator._aget_retrieved_ids_and_texts`
[L18:35-50]→retriever async→postprocesadores→listas IDs/textos→
`metric.compute` [L19]. No es CLI con dataset incorporado; el llamador aporta
query y expected_ids, y elige métricas por nombre. No hay modelo de juez ni
answerer en esta cadena; no se inspeccionó runner de dataset completo.

| Métrica de [L19] | Cálculo real y denominador |
|---|---|
| HitRate default, `use_granular_hit_rate=False` | 1 si **algún** retrieved ID pertenece a expected_ids, 0 si no; unidad pregunta |
| HitRate granular | Cuenta cada retrieved ID que está en set(expected); divide por longitud original de expected_ids; no deduplica retrieved antes de contar |
| MRR default, `use_granular_mrr=False` | Recíproco de posición 1-based del primer hit; 0 si ningún hit |
| MRR granular | Suma recíprocos de cada hit y divide por número de hits **recuperados**, no por número de gold esperado |
| Vacíos | HitRate/MRR rechazan expected_ids o retrieved_ids vacíos con `ValueError`, no los puntúan automáticamente como cero |

Un duplicado en retrieved puede afectar granular: propiedad estática de
esas funciones, no evidencia de que todos los retrievers produzcan duplicados.
No hay aquí promedio global ni denominador de preguntas del dataset: depende
del caller no leído. No declarar scores incompletos como tasa de éxito total.
No se configura k en la función de métrica: la cardinalidad la entrega el
retriever. Modelo/embeddings/tokenizer/seed/cache/budget/servicios y coste
son del caller; no se inventa default para completar una tabla.

Adaptación: mapear namespace+key/revisión a IDs externos estables, mantener
gold solo en evaluación y conservar resultado vacío/error según el protocolo
ya fijado. HitRate default no es `recall_all` de LongMemEval; MRR granular no
es cobertura de todos los soportes. No modificar fórmulas, sampling o prompts
para hacer coincidir porcentajes. Consultar también
[BENCHMARK-NOTES](BENCHMARK-NOTES.md). No se ejecutó ninguno de los evaluadores.

## 8. Matriz frente al estado existente

| Capacidad | Estado Vanta | Delta defendible |
|---|---|---|
| ANN/BM25/sparse/RRF | **Existe**, [V12] y referencia fijada | No añadir otro retrieval básico |
| Provenance/sesiones/supersession/dedup | **Existe**, [V5][V8] | A conserva identidades existentes |
| Ingesta y checkpoints | **Existe**, [V4][V9][V10] | D solo evidencia/reconciliación específica |
| Assembler y compaction | **Existe**, [V6][V7] | B solo expansión explicable |
| Paridad exacta con cache key L1 | No acreditado | No implica necesidad de copiarla |
| Árbol documental importado validado | No acreditado en esta cobertura | Candidata B condicionada al inventario |
| Integración completa de A/B con todos los hosts | Parcial: seams, no trazado actual | Revalidación previa |
| Framework Python como requisito del motor | No aplicable | Mantener adaptadores fuera del núcleo |
| Planificación L1 sin persistir | **Existe**, `plan_write` [V8] | C no propone «plan» desde cero |
| Preview documental delta/snapshot | **No acreditado** en seams leídos | C condicionado a inventario, no ausencia global |
| Evidencia de etapa frente a mutación total | **Parcial**, checkpoint/L1 [V8][V10] | D referencia C13 propuesto; varios receipts de etapa no forman commit conjunto |
| Fidelidad del adapter LlamaIndex existente | **Existe / parcial**, [V13][V14] | Tuplas nodes/ids/similarities y filters requieren corrección antes de B/BEIR; no adapter nuevo |
| Spill y cursor de offload | **Existe**, [V15][V16][V17] | F expone stored/already_present/failed/unknown; no crea almacenamiento ni confirma fsync |
| Gobernanza de inyección | **Existe**, [V11] | E solo proyección por destino/versionado |
| Historial de tarea vs memoria de chat | **Existe** separación [V10][V6] | No equipararlos |
| Servicio LlamaCloud dentro del motor | **N/A** | Producto externo, distinto despliegue |
| Inspección de memoria/actividad | **Existe**, [S1] | Extender Studio, no app paralela |

## 9. Carencias sugeridas, no bugs certificados

El contraste plantea dos preguntas: ¿se puede explicar la reutilización de
una transformación y reconstruir la expansión de un hit hasta su fuente?
No demuestra que falten caché, jerarquías o compactación en Vanta.
Prioridad condicional: A si el operador pierde la versión efectiva de ingesta;
B si una ampliación ya existente no conserva decisiones y revisiones.
Una mejora puramente de reporte puede bastar; no se presupone nuevo subsistema.

**Hechos negativos acotados:** [V8:430-446] conserva delete-before-put en
lote L1 y comentario de fallo parcial; D debe expresar ese outcome.
[V10] documenta resume at-least-once, no exactly-once de side effects;
es límite declarado, no bug descubierto. [V6:197-218] conserva prefijo
protegido aun si excede budget: no afirmar cabida absoluta.
**No acreditado:** paridad de A/C/E con todos los hosts/adaptadores.
**Hipótesis:** hash anticipado de #22638 no se traslada a Vanta sin trazar
la etapa concreta. **Falsos positivos descartados:** ausencia de jobs,
provenance, compactación, presupuesto, authz o UI. Nada aquí prueba peor
calidad, rendimiento, seguridad global o inexistencia de benchmarks propios.

## 10. UI, escritorio y evidencia externa

El README [L8] anuncia LlamaParse/LlamaCloud y enlaces de producto externos.
No se inspeccionó su frontend ni se acredita aquí una aplicación de escritorio.
El código leído ofrece APIs y logs, no evidencia de un dashboard incluido.
La búsqueda acotada no equivale a afirmar que no exista ninguna UI upstream.
Vanta ya tiene Studio Tauri/React en `develop-desktop`, según propietario.
A/B deberían alimentar su inspección de ingesta/contexto, no otra aplicación.
Runtime necesario para esta investigación: **ninguno**.

Se releyó con `git show` el cuerpo de
`feature-freez/desktop/src/components/memory/MemoryLens.tsx:1-90` del commit
`0ad38c542b39a589035b0e17fdedb4c09dca24b9` [S1]:
escenas/persona/skills/genlog y navegación al inspector; read-only no
significa permisos universales ni restore. Flujo propuesto: preview C →
coste estimado sin invocar modelos → ejecución y receipt D → inspección de
reuse A → query/expansión B → campos omitidos E en diagnóstico redactado.
No diseñar UI nueva; conectar reportes al inspector/actividad existente.
No se leyó el frontend alojado de LlamaCloud/LlamaParse, ni se infiere su
stack, authz o función de escritorio del README.
Las rutas actuales `desktop/...` están movidas/modificadas localmente;
sus bytes no se atribuyen al SHA, ni JobChip/jobs ausentes en ese pin.

## 11. Inventario ampliado y cierre

| Referencias nuevas | Archivo/rangos efectivamente usados |
|---|---|
| [L9][L10] | `schema.py:207-318,337-359,791-818`; modelos/metadata/hash |
| [L11] | `ingestion/cache.py:1-78`; KV/serde/persist limitado a SimpleCache |
| [L12] | `ingestion/pipeline.py:693-743,790-882`; async/errores/progreso |
| [L13] | `indices/base.py:229-253,372-485`; insert/update/refresh |
| [L14] | `indices/vector_store/base.py:220-258,385-480`; ownership/delete |
| [L15] | `storage/storage_context.py:149-216`; persist secuencial, Simple stores |
| [L16] | `memory/memory.py:339-479`; estimación y productores |
| [L17–L19] | `evaluation/retrieval/base.py:35-166`, `evaluator.py:1-103`, `metrics.py:1-195`; no métricas restantes |
| [L20], [L21] | `node_parser/relational/hierarchical.py:14-229`; `vector_stores/utils.py:40-98`; parser y envelope |
| [L22], [L23] | `memory/memory.py:546-845`; `storage/chat_store/sql.py:372-406`; archivo antes de fan-out |
| [V4–V12] | Seams Vanta fijados, citados por símbolo/rango en el texto; CodeGraph y lectura local |
| [V13], [V14] | `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:27-471`; `tests/test_vectorstore.py:148-166,221-241` |
| [V15], [V16], [V17] | `context_engine/engine.rs:62-79`; `context_engine/spill.rs:65-120,224-261`; `offload/state_manager.rs:78-96` |
| [S1] | `feature-freez/desktop/src/components/memory/MemoryLens.tsx:1-90`, cuerpo Studio fijado; no ejecución |

No leídos: implementación de `beir`/pytrec_eval, todos los aggregators de
datasets, generadores de preguntas, QA judges, integración completa con
Vanta LlamaIndex, plugins remotos y Cloud, locks/commits de cada backend,
árboles importados end-to-end, licencias transitivas. No se declara lectura
exhaustiva ni rendimiento medido.

**Relaciones:** A identidad/reuse; B expansión independiente de A; C planea
sin gastar modelos; D usa identidad de A/C si se aprueba publicación;
E modifica identidad A cuando altera inputs semánticos. B/E comparten
gobernanza/budget existente, no nuevo pipeline. Las fichas no reservan IDs ni
modifican backlog; se reconciliarán con familias transversales de
[OpenViking](../../Backlog-OpenViking-VantaDB.md) antes de specs.

**División futura (>300 líneas):** specs de identidad/proyección A/E,
sincronización/progreso C/D y relaciones/contexto B; conservar este dossier
como evidencia común sin delegar sus diez ejes. No se crean specs ahora.
Pendientes: aceptación manual de cada ficha, versiones/contratos concretos de
stores, reproducción #22638, revisión adversarial independiente y decisión
del responsable. No validadores globales, commits, staging ni push.

[L9]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/schema.py#L207-L359
[L10]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/schema.py#L791-L818
[L11]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/ingestion/cache.py#L1-L78
[L12]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/ingestion/pipeline.py#L693-L882
[L13]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/indices/base.py#L372-L485
[L14]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/indices/vector_store/base.py#L220-L258
[L15]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/storage/storage_context.py#L149-L216
[L16]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/memory/memory.py#L339-L479
[L17]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/evaluation/retrieval/base.py#L35-L166
[L18]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/evaluation/retrieval/evaluator.py#L1-L103
[L19]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/evaluation/retrieval/metrics.py#L16-L158
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L128-L186
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/ingestors.rs#L1-L89
[V6]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L93-L218
[V7]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/types.rs#L1-L98
[V8]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L160-L176
[V9]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/scheduler.rs#L1-L118
[V10]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L1-L121
[V11]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L115-L142
[V12]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/hybrid.rs#L7-L60
[S1]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L1-L90
[L20]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/node_parser/relational/hierarchical.py#L14-L229
[L21]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/vector_stores/utils.py#L40-L98
[L22]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/memory/memory.py#L546-L845
[L23]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/storage/chat_store/sql.py#L372-L406
[V13]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/llamaindex/vantadb_llamaindex/vectorstore.py#L27-L471
[V14]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/llamaindex/tests/test_vectorstore.py#L148-L241
[V15]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L62-L79
[V16]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/spill.rs#L65-L261
[V17]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/offload/state_manager.rs#L78-L96

[L0]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/pyproject.toml#L35-L87
[L1]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/ingestion/pipeline.py#L46-L110
[L2]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/ingestion/pipeline.py#L451-L507
[L3]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/ingestion/pipeline.py#L580-L662
[L4]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/retrievers/auto_merging_retriever.py#L35-L164
[L5]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/retrievers/auto_merging_retriever.py#L166-L194
[L6]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/memory/memory.py#L481-L545
[L7]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/evaluation/benchmarks/beir.py#L1-L109
[L8]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/README.md#L11-L32
[LL]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/LICENSE#L1-L21
[V1]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/ingest/merge.rs#L1-L65
[V2]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/context_engine/engine.rs#L1-L70
[V3]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/context_engine/types.rs#L1-L65
