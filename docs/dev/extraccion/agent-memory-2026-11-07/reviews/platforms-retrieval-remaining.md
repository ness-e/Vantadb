---
title: "Revisión independiente: frameworks y recuperación restantes"
kind: research
status: proposed
description: "Contraste adversarial de siete informes con recibos individuales, fuentes congeladas, límites públicos y decisiones propuestas para VantaDB"
tags: [research, memory, engineering]
---

# Revisión independiente: frameworks y recuperación restantes

## Alcance y autoridad

Revisión de los siete informes completos, no de su longitud ni de una selección
de encabezados. El antecedente [platforms-frameworks](platforms-frameworks.md)
no se usa como revisión completa de estos siete proyectos: la revisión previa
reconocida para este encargo corresponde a Dify/Langflow.
Rigen [contrato](../EXPANSION-CONTRACT.md),
[referencia Vanta](../VANTA-REFERENCE.md), [método](../METHOD.md),
[notas de benchmarks](../BENCHMARK-NOTES.md) y
[cobertura](../REVIEW-COVERAGE.md), además del brief temporal del responsable.
La aceptación final pertenece al responsable principal.

**Corte de lectura:** 2026-11-07. Código estático y consultas públicas, sin
ejecutar aplicaciones, celdas, pruebas, builds, benchmarks, instalaciones o
servicios; sin iniciar/detener runtime, mantener grafos, alterar evaluación,
contratos, informes, índices o backlog. Sólo se escribe esta revisión.
No se ejecutaron gates, staging, commits ni push.

**Gravedad documental:** alta = puede inducir una decisión insegura o atribuir
garantías no probadas; media = falta de contrato/evidencia material para una
spec; baja = precisión que no cambia la recomendación. No es scoring del
producto ni prueba de vulnerabilidad explotada. Estados por eje: sustentado,
parcial, no acreditado o no aplicable con justificación; sin nota agregada.

## Índice

- [Referencias congeladas](#referencias-congeladas)
- [LangChain](#langchain)
- [LangGraph](#langgraph)
- [CrewAI](#crewai)
- [LightRAG](#lightrag)
- [PageIndex](#pageindex)
- [Serviette](#serviette)
- [Meilisearch](#meilisearch)
- [Límites transversales y entrega](#límites-transversales-y-entrega)

## Referencias congeladas

HEAD de los siete clones reconsultado con `git rev-parse`, sin fetch ni escritura.
El [manifiesto](../clone-manifest.json) conserva captura, rutas y omisiones;
no se actualizan estrellas ni inventarios. Una revisión shallow no acredita
fecha de introducción/fix ni versión publicada que contenga un cambio.

| Alias | Proyecto / clon bajo `C:\Users\Eros\` | SHA |
|---|---|---|
| LC | `langchain-ai/langchain`, `langchain` | `d9caf083577c43a91c5bf6bd1f7dfc289581184d` |
| LG | `langchain-ai/langgraph`, `langgraph` | `e5e37b5d70dbfd93c7006bd90a658c6179a95946` |
| CA | `crewAIInc/crewAI`, `crewAI` | `73f48320dbe6e2dc7a9bc54fefc6a13637b9485c` |
| LR | `HKUDS/LightRAG`, `LightRAG` | `aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a` |
| PI | `VectifyAI/PageIndex`, `PageIndex` | `2ac8694bf8fe89a3840dda9bdb88db5119b77fc4` |
| SE | `pathwaycom/serviette`, `serviette` | `84e92a33728cf843026f4011d86568b403e38fb4` |
| ME | `meilisearch/meilisearch`, `meilisearch` | `c18c81973ad5d3e126f4583cdd16a9ec1719aa4a` |
| V | Motor/adapters Vanta, `VantaDB Proyect\VantaDB` | `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` |
| S | Studio, `VantaDB Proyect\develop-desktop` | `0ad38c542b39a589035b0e17fdedb4c09dca24b9` |

Identidad adicional por recibo, procedente del manifiesto; ramas reconsultadas
localmente. Son timestamps de **metadata**, no fecha de release ni del commit.
`LICENSE` raíz se releyó; la columna no sustituye auditoría per-file/transitiva.

| Alias | Rama | Captura UTC 2026-11-07 | Licencia por superficie y límite |
|---|---|---|---|
| LC | master | 06:56:40.3815325Z | MIT de código propio; Core/Classic/proveedores/servicios no constituyen una licencia universal. |
| LG | main | 06:56:20.7153879Z | MIT raíz; checkpoint/CLI declaran MIT en el informe; savers/deps y producto LangSmith requieren revisión separada. |
| CA | main | 06:56:32.1155273Z | MIT de biblioteca/cliente; términos de CrewAI Plus/telemetría alojada no acreditados por esa licencia. Adapter Vanta declara Apache-2.0, no MIT upstream. |
| LR | main | 06:56:23.2154091Z | MIT raíz; WebUI, parsers, modelos/corpus y adaptadores no auditados per-file en su totalidad. |
| PI | main | 06:56:24.0841950Z | MIT raíz del SDK; App SaaS, PDFs/datos y modelos con condiciones separadas no acreditadas. |
| SE | main | 06:56:25.9836250Z | MIT wrapper/HTML; Pathway xpack, conectores y modelos no quedan licenciados por el wrapper. |
| ME | main | 06:56:19.7819824Z | Raíz MIT AND BUSL-1.1 con frontera LICENSE-EE; mini-dashboard/CDN/ZIP y transitivas no certificados per-file. Metadata NOASSERTION no es permiso. |

Identificación de los informes leídos: SHA256 de bytes al cierre de esta lectura.
Los siete conteos coinciden con los recorridos completos indicados en los
recibos. Cambios posteriores del autor exigirán comparar nuevamente; estos hashes
no identifican los clones ni prueban ausencia de edición concurrente durante toda
la sesión.

| Informe | SHA256 de archivo |
|---|---|
| langchain.md | `3FCF4EE43184F8A7387E233863893ED824A2EBC69C28B8C305FDEFFE7F308883` |
| langgraph.md | `FE1A8AC0851A38D9FBAE389BFA5F1B042E08A70ABC1A483A986529FAE412FAAD` |
| crewai.md | `99FEFF240FBC6CF40B4CADD52F828274D001976AE8C76E640243DDC65D98C337` |
| light-rag.md | `A81E58A6F1E91039F6FF049C8189BD1A0AF789A12AB477707A85912C7890EB0D` |
| pageindex.md | `8FCEAB966E78145BB9FAC33DD7838148112F450E2B1F86D5CCC0ED6A554F9FE9` |
| serviette.md | `FCC3050733E8E008397E89287C9FBDF84FFECF8F2D2326947FAB95BA09703A26` |
| meilisearch.md | `BAEABDBF483A905D439ABC1D947876640B0D658FD18C2478A1CE02575F216D85` |

Los alias fijan revisión para las rutas/símbolos que siguen. Vanta se leyó
mediante `git show V:ruta`, no atribuyendo el worktree concurrente al pin.
BM25/HNSW/RRF, filtros, `RecallScope`/`RecallConfig`/`InjectionPolicy`,
provenance, dedup/supersession, scheduler/checkpoints, budgets y cuotas existen:
no se convierten las propuestas en carencias desde cero. `WriteReceipt`
(`src/attestation.rs:654–738`) usa hash de proyección store/WAL; no firma
digital, prueba universal de derivados ni ACK del modelo. `MutationReceipt`
es propuesto cuando aparece en los informes.

## LangChain

**Recibo:** [langchain.md](../langchain.md), lectura completa 1–493; pin LC.
**Dictamen propuesto: cambios requeridos.** La separación contexto/historial/
checkpoint es acertada; falta el recorrido material de mutación del vector store
y el contraste temporal relevante para memoria.

### Ledger 0 + 10

| Eje / sección | Estado | Evidencia o laguna |
|---|---|---|
| 0. Identidad/arquitectura/flujos, §§0/2 | parcial | Mensajes, middleware y history separados; escritura/update vectorial no trazados: LC-1. |
| 1. Recomendaciones, §1 | sustentado | Conformidad del adapter actual antes que framework Rust; compactación causal y recibo de tool son dominios distintos. |
| 2. Fichas Rust A–F, §2 | parcial | Problema, unidad, delta, alternativas, tipos propuestos, seams, pasos/migración, deps, FMEA, aceptación y no-goals presentes; F omite identidad/commit de escritura: LC-1. |
| 3. Incidencias/seguridad, §3 | sustentado | #41323 abierto; #38867 cerrado; no se extrapola cierre a fix ni bug a Vanta. Consulta pública propia y límite de actividad abajo. |
| 4. Prerrequisitos, §4 | sustentado | Dependencias por candidata; conserva assembler, approval de capturas y checkpoints existentes. |
| 5. Rechazos, §5 | sustentado | No portar Runnable/providers/legacy memory; resumen derivado no sustituye fuente. |
| 6. Método/licencia, §§0/6/11 | parcial | MIT y dependencias separadas; “no auditado legacy” no justifica por sí solo excluir el contraste temporal relevante: LC-2. |
| 7. Benchmarks, §7 | sustentado | Runner de schema conversion, 20 tools; warm/cold no son recall ni QA. |
| 8. Matriz, §8 | parcial | Descarta inversión cosine ya corregida; conformidad de update/IDs pendiente LC-1. |
| 9. Negativos Vanta, §9 | sustentado | Riesgos de append/score no se declaran bugs reproducidos; excepción del prefijo protegido preservada. |
| 10. UI, §10 | sustentado | Documentación LangSmith Studio externa, no frontend OSS acreditado ni desktop nuevo. Delta para Studio existente, no certificación visual. |

### Hallazgos

1. **LC-1 · Media — “update/delete siguen storage” no cubre el contrato de mutación.**
   Informe `langchain.md:126–133,294–320,414–420`.
   [Core `VectorStore.add_documents:234–258`][lc-core] se declara add/update,
   prioriza IDs explícitos y delega en `add_texts`. En V,
   `integrations/langchain/vantadb_langchain/vectorstore.py::add_texts:430–446`
   realiza embeddings y puts individuales; `_build_key:84–85` depende de texto/posición
   cuando no se suministran IDs. `delete` también itera; el retorno no es una
   transacción de conjunto. La ficha de score no analiza ese recorrido.
   **Mecanismo/consecuencia:** contenido editado sin identidad estable puede
   representar otra clave; fallo después de un put deja prefijo aplicado y
   retry puede volver a mutar. No se ha demostrado pérdida ni CAS inexistente
   en todo el motor; es una laguna del informe sobre el adapter.
   **Corrección verificable:** documentar add con/sin IDs, reemplazo de
   texto/metadata/vector, validación antes/después de embeddings, retorno y
   error parcial/retry/cancel en ese símbolo; ligar conformidad a la versión
   soportada. Ampliar F o justificar candidata distinta, sin crear otra DB.

2. **LC-2 · Media — Falta evaluar la memoria time-weighted, no sólo decir “legacy”.**
   Informe `langchain.md:55–56,126–133,363–370,454–465`.
   [TimeWeightedVectorStoreRetriever:64–198][lc-time] combina recencia,
   salience y claves extra; resuelve `buffer_idx` contra `memory_stream`,
   modifica `last_accessed_at` sólo en el buffer al leer (`115–132`), y
   extiende ese buffer **antes** de persistir (`164–198`).
   **Consecuencia:** copiar “memoria temporal” sin esa distinción puede
   aparentar persistencia/recall tras reinicio que el código no promete;
   fallo del store deja estados distintos entre buffer y vector store.
   **Corrección verificable:** añadir lectura acotada y decisión explícita
   frente al scoring opcional ya existente de Vanta, sin importar pesos.
   Justificar rechazo, extensión o adapter con identidad, touch, fallo y
   persistencia diferenciados. No hace falta ejecutar ni adoptar esta API.

### Contrastes y límites

- Tres fronteras verificadas: Core add/update delegado → storage; history
  entrada/salida sin CAS común; middleware resumen/interrupt con publicación
  posterior al modelo. Cancelación del caller no implica undo remoto.
- Bench leído: `libs/core/tests/benchmarks/test_tool_schema_conversion.py:1–69`.
  El bloque mide conversión de 20 schemas, warm prepopula memo; cold incluye
  creación. Rondas/warmup de pytest-benchmark no fijados allí; error propaga,
  sin dataset QA, split, juez, embeddings ni reset de corpus. CPU/RAM y
  artefactos del fixture, no coste end-to-end de memoria.
- Fuentes públicas reabiertas: [#41323](https://github.com/langchain-ai/langchain/issues/41323)
  completo, abierto; [#38867](https://github.com/langchain-ai/langchain/issues/38867)
  cuerpo/estado cerrado, actividad truncada. #39268 aparece relacionado;
  diff y versión publicada corregida no acreditados por esta revisión.
  La versión reportada del issue no identifica el pin LC ni un entorno instalado.
- MIT es licencia de código propio, no de todos los proveedores/servicios.
  No auditados proveedores completos, todos los middleware ni frontend SaaS.
  LC-1/2 son correcciones documentales, no permiso de implementación.

## LangGraph

**Recibo:** [langgraph.md](../langgraph.md), lectura completa 1–551; pin LG.
**Dictamen propuesto: cambios requeridos**, por explicitud de privacidad al
reanudar, no porque falte memoria semántica ni porque SQLite sea el backend Vanta.

### Ledger 0 + 10

| Eje / sección | Estado | Evidencia o laguna |
|---|---|---|
| 0. Arquitectura, §§1/2 | sustentado | `BaseCheckpointSaver` y `BaseStore` distintos; superstep/pending writes no son hechos ni ACL. |
| 1. Recomendaciones, §1 | sustentado | Prioriza conformidad y no portar Pregel. |
| 2. Fichas A–F, §2 | parcial | Once campos distribuidos entre ficha/cierre; falta invariante discriminante de privacidad al resume: LG-1. |
| 3. Incidencias, §3 | sustentado | SQLite #9563 y scans #8855 contrastados; reporte/fix propuesto no equivale a release integrado. |
| 4. Prerrequisitos, §4 | parcial | Identidad y retención sí; autorización vigente antes de rehidratar/reanudar insuficientemente concreta. |
| 5. Rechazos, §5 | sustentado | No serializar clases Python no confiables como contrato de memoria Rust. |
| 6. Método/licencias, §§6/11 | sustentado | MIT por componentes declarados; serde privilegiado y limitaciones explícitos. |
| 7. Benchmark, §7 | sustentado | pyperf/durability exit; bug objeto contra strings confirmado en 478–485. |
| 8. Matriz, §8 | sustentado | TTL rechazado, index ignorado, namespaces por tupla y Core incompatible descritos sin falsas ausencias. |
| 9. Negativos, §9 | sustentado | `put_writes` y wrappers sync son hechos del adapter al pin, no una ejecución de fallo. |
| 10. UI, §10 | sustentado | Producto web externo documentado; no código de frontend ni auth completa acreditados. |

### Hallazgo

1. **LG-1 · Media — Privacidad al resume no tiene aún una aceptación propia.**
   Informe `langgraph.md:176–205,275–325,362–375,503–508`.
   [Vanta saver `put/get_tuple/delete_thread`][v-saver] conserva checkpoint
   completo y writes; [store `_delete`][v-store] opera sobre otra familia.
   El texto distingue correctamente ambos, pero B/F se concentran en
   identidad/revisión de operación y compatibilidad de grafo, y E en thread.
   **Mecanismo/consecuencia:** retirar un hecho de `BaseStore` o revocar permiso
   no demuestra que un checkpoint anterior deje de contenerlo. Reanudar desde
   un padre puede rehidratar esa copia aunque el retrieval actual esté autorizado.
   Es un escenario de contrato, no una fuga reproducida ni un bug probado Vanta.
   **Corrección verificable:** declarar dueño/política de acceso al checkpoint
   antes de decode, y de revalidación antes de resume/nueva entrega/efecto; añadir a B/E/F caso
   “checkpoint previo + permiso/fuente retirados” con resultado permitido,
   denegado o redacción explícita. No prometer borrar backups ni convertir
   delete_thread en borrado global de memorias compartidas.

### Contrastes y límites

- LG `checkpoint-sqlite/.../__init__.py:188–208,458–515`: commit en finally,
  `executemany`, slots especiales y delete en ambas tablas. `checkpoint-postgres/
  .../base.py:149–162`: upsert frente a insert-do-nothing.
  V `checkpointer.py:269–313`: enumera/skip y borrado paginado; no equivale
  automáticamente a semántica de controles PostgreSQL ni a transacción con tools.
- LG `store/memory/__init__.py:206–234,375–440`: reads preparados antes de puts,
  último put por clave e indexación selectiva. V `store.py:97–159`: secuencial,
  TTL explícitamente rechazado, `op.index` no trasladado, payload al embedder.
  No se hereda una garantía universal de batch desde InMemoryStore.
- LG `serde/jsonplus.py:104–147`: escritor directo puede cruzar frontera de
  reconstrucción; `pickle_fallback=False` no desactiva msgpack permisivo.
  Base64 no es cifrado, firma ni autorización.
- Bench `bench/__main__.py:1–85,467–520`: UUID, stream consumido, exit y pyperf;
  excepción no es QA miss; saver in-memory puede acumular estado, sin fsync DB.
  El guard compara `graph` con nombres strings y no acredita first-event medido.
- Reabiertos [#9563](https://github.com/langchain-ai/langgraph/issues/9563),
  completo y abierto, versiones reportadas 3.1.2/3.1.3; y
  [#8855](https://github.com/langchain-ai/langgraph/issues/8855), cuerpo abierto,
  actividad posterior truncada, referencia reportada `81bf17b`. No se ejecutaron
  repros; PRs/fixed release no acreditados. No trasladar cifras del reportero.
- Seams compartidos V: `TaskCheckpointManager`, assembler y approval de
  capturas existen; no son automáticamente autorización durable de una tool.
  Conformidad, crash, cifrado, HTTP auth y frontend externo no certificados.

## CrewAI

**Recibo:** [crewai.md](../crewai.md), lectura completa 1–807; pin CA.
**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
No equivale a aceptación final, compatibilidad ejecutada ni seguridad global.

### Ledger 0 + 10

| Eje / sección | Estado | Evidencia o laguna |
|---|---|---|
| 0. Arquitectura/trazas, §§0/2 | sustentado | Memoria unificada, outputs SQLite y replay de tareas separados; tres flujos y error/cancel explícitos. |
| 1. Recomendaciones, §1 | sustentado | Reconoce adapter CrewAI existente en V; prioriza propiedad/barrera antes de planner. |
| 2. Fichas A–E, §2/cierre | sustentado | Once campos distribuidos: tipos propuestos, seams reales, alternativas, migración/rollback, FMEA y aceptación adversa. |
| 3. Incidencias, §3/cierre | sustentado | #8659 abierto y reconstrucción sin private/source verificada al pin; fix/version publicada no acreditados. |
| 4. Prerrequisitos, §4 | sustentado | Scope confiable distinto de metadata inferida; no bloqueo universal por CAS/WAL. |
| 5. Rechazos, §5 | sustentado | No pesos/pool/Flow obligatorios ni `include_private` como ACL. |
| 6. Método/licencias, §§0/6 | sustentado | MIT no resuelve providers/enterprise/telemetría; no tests leídos tratados como ejecutados. |
| 7. Benchmarks, §7 | sustentado | Evaluación de agente, denominadores y excepción fuera del try distinguidos; no recall ni dataset fijo. |
| 8. Matriz, §8 | sustentado | Scoring opcional, provenance, scheduler y adapter existentes; delta de capacidades no universal. |
| 9. Negativos Vanta, §9/cierre | sustentado | `get_record/search` silencian excepción en ese adapter; no se extiende a todo Vanta ni a corrupción. |
| 10. UI, §10/cierre | sustentado | HTML Flow y clientes de tracing no son frontend enterprise ni desktop OSS. Studio propuesto sin atribuir bytes a su pin. |

### Evidencia discriminante y recibo de hallazgos

**No se encontró discrepancia bloqueante nueva** en la muestra contrastada.
Esto no rellena lagunas con un resultado de runtime.

- CA `memory/encoding_flow.py:388–517`: dedup first-wins, reembedding antes
  de mutar; delete/update/save separados. `477–490` conserva scope y metadata,
  **omite private/source de primer nivel**; la inserción sí los suministra.
  `unified_memory.py:862–909` usa `model_copy`, conserva campos omitidos y
  reemplaza metadata explícita: no es el mismo camino ni “todo update filtra”.
- CA `unified_memory.py:302–375`: submit guarda Future; callback elimina y emite
  error best-effort; drain espera sin relanzar. Recall no queda certificado como
  watermark de todos los writes durables. Cancelar tarea no revierte herramientas.
- V `integrations/crewai/vantadb_crewai/memory.py:165–226`: save usa puts;
  update reutiliza save; get/search convierten excepciones en None/[]; scope por
  segmento se filtra tras top-k. El mapper no repara la pérdida upstream
  si ya recibe `private=False`. El informe conserva esa frontera correctamente.
- Bench CA `experimental/evaluation/experiment/runner.py:26–165`: casos
  secuenciales, reset del evaluator, no de toda DB; faltan campos antes del try
  puede abortar; error capturado devuelve cero/failed; media de scores presentes;
  comparación dict usa intersección. No se confunde con QA/recall.
  Costes/modelo efectivo, contaminación entre casos y escritura de baseline
  siguen límites, no resultados propios ni nuevos criterios de evaluación.
- [#8659](https://github.com/crewAIInc/crewAI/issues/8659) reabierto íntegro:
  abierto, reporte `bb4d892e6`, helper no ejecutado; desarrollo #8660 no prueba
  merge ni versión corregida. La misma omisión permanece en CA.
- No auditados Qdrant, todos los locks/consumidores, exportación/retención remota
  y frontend enterprise. El ledger no acredita autenticación entre tenants.

## LightRAG

**Recibo:** [light-rag.md](../light-rag.md), lectura completa 1–641; pin LR.
**Dictamen propuesto: cambios requeridos**, por evidencia Studio atribuida
a una revisión que no contiene dos de los componentes citados.

### Ledger 0 + 10

| Eje / sección | Estado | Evidencia o laguna |
|---|---|---|
| 0. Arquitectura/flujos, §§1/2/11/12 | sustentado | Anchors antes de derivados, purge por fases y multimodalidad separados de memoria de agente; no transacción distribuida. |
| 1. Recomendaciones, §§1/11 | sustentado | Enriquecimiento opcional; no importar cuatro stores ni reemplazar RRF. |
| 2. Fichas A–G, §§2/11/12 | sustentado | Once campos distribuidos; G opt-in, B dato/E protocolo, A consumo/F producción. Studio requiere LR-1. |
| 3. Incidencias, §3/12 | sustentado | #3659 RFC abierto, #3978 abierto con tabla histórica; no fixes/versiones publicados acreditados. |
| 4. Prerrequisitos, §4 | sustentado | ACL transitiva, revisión/contribución, budget compartido; no LLM obligatorio en Vanta. |
| 5. Rechazos, §5 | sustentado | Nombres normalizados no identidad universal; caché no amplía permisos. |
| 6. Método/licencia, §6 | parcial | Pin upstream correcto; pin Studio mal atribuido LR-1. MIT no licencia modelos/corpus. |
| 7. Benchmarks, §7 | sustentado | RAGAS y scoring lexical offline distintos; NaN, fallos y denominadores separados. |
| 8. Matrices, §§8/11/12 | parcial | Capacidades existentes conservadas, pero fila Studio usa referencias inválidas. |
| 9. Negativos, §9 | sustentado | Linaje integral no acreditado no se convierte en ausencia de provenance o bug Vanta. |
| 10. UI, §§10/12.4 | parcial | Cliente web real leído; atribución de jobs al pin S incorrecta. |

### Hallazgo

1. **LR-1 · Alta — Fuente Studio no pertenece al SHA declarado.**
   Informe `light-rag.md:614–621,639–641`. `git ls-tree -r S` localiza
   `feature-freez/desktop/src/components/memory/MemoryLens.tsx`, no
   `desktop/src/...`; tampoco contiene `JobChip.tsx` ni `commands/jobs.rs`.
   `git show S:feature-freez/desktop/src-tauri/src/commands/jobs.rs` falla
   porque esa ruta no existe. No basta anteponer `feature-freez` a los tres enlaces.
   **Mecanismo/consecuencia:** bytes del worktree actual se presentan como
   comportamiento inmutable del pin; cancelación parcial y persistencia de jobs
   no quedan acreditadas por esa evidencia.
   **Corrección verificable:** citar MemoryLens con ruta/rangos reales del
   commit; para JobChip/jobs declarar lectura del worktree con identidad
   separada, o retirar esa atribución al pin. No afirmar que Studio no existe ni
   que todo su lifecycle falla. Este mismo error reaparece en PI/SE/ME.
   `git status --short` los muestra como `??` en `desktop/`: son archivos no
   rastreados, no bytes recuperables mediante `git show S:ruta`.

### Evidencia y límites

- LR `operate.py:3514–3568,3666–3725`: superset de anchors y flush antes de
  primera mutación; hook después. `lightrag.py:6015–6095,6188–6250`:
  proof/journal, reparación de derivados antes de chunks, anchors al final;
  caché LLM es recurso separado. No se certifica crash físico por docstring.
- `operate.py:4693–4796`: keywords, fallback corto y context-only; no fallback
  universal ANN ni eliminación de toda llamada LLM en los modos KG.
  `pipeline.py:4805–4830,7956–7988`: cancel antes de analizar y sidecars
  success/skipped/failure; opt-out no debe reutilizar éxito anterior.
- V `core/ontology/mod.rs::apply_triplet_with_source:147–212`,
  `src/wiki/ingestors.rs::Ingestor:45–89` y assembler son seams reales;
  spans versionados/retirada integral son deltas, no capacidades negadas.
- Bench LR `eval_rag_quality.py:129–183,535–564,789–887`: código default
  embedding 3-large frente a docstring 3-small; caso all-NaN falla; tasa sobre
  `len(results)` y medias por conteo válido de cada métrica. Offline
  `offline_retrieval_check.py:182–254` usa ficheros gold/top-k 2, no grafo.
  Se conservan coste de API/juez/embedder, artefactos sensibles y ausencia de
  mediciones propias; no se modifica evaluación.
- [#3659](https://github.com/HKUDS/LightRAG/issues/3659) y
  [#3978](https://github.com/HKUDS/LightRAG/issues/3978) reabiertos:
  abiertos, cuerpos/diseño parcialmente extraídos, actividad truncada.
  El segundo declara “Current state” sin actualizar; no describe uniformemente
  el pin actual. PRs y fixed release siguen no acreditados.
- WebUI `src/api/lightrag.ts:879–908` pasa AbortSignal a fetch; no prueba undo
  backend ni cese de coste remoto. Auth completa, parsers/backends completos,
  recovery físico y todos los comentarios de issues no auditados.

## PageIndex

**Recibo:** [pageindex.md](../pageindex.md), lectura completa 1–499; pin PI.
**Dictamen propuesto: cambios requeridos**, por identidad de evidencia Studio;
no por el supuesto exploit #585, que no queda acreditado.

### Ledger 0 + 10

| Eje / sección | Estado | Evidencia o laguna |
|---|---|---|
| 0. Identidad/arquitectura, §§1/11.1 | sustentado | Local/cloud, dos parsers, árbol/páginas/meta y tres recorridos distinguidos. |
| 1. Recomendaciones, §1 | sustentado | Navegación complementaria, no sustituto vectorless de híbrido. |
| 2. Fichas A–D, §§2/11 | sustentado | Once campos, revisión/unidades, navegación y budget de ejecución separados; PI-1 afecta presentación. |
| 3. Issues, §11.5 | sustentado | Cinco cuerpos reabiertos; #585 no demuestra breakout de string; no repro propio ni versión corregida acreditada. |
| 4. Prerrequisitos, §4 | sustentado | Spans/generación por comprobar, no ausencia de provenance/checkpoints. |
| 5. Rechazos, §5 | sustentado | No copiar lock POSIX/JSON como sustituto de WAL ni equiparar QA financiera a recall. |
| 6. Método/licencias, §§6/11 | parcial | MIT/datasets/proveedores separados; links Studio no corresponden a S. |
| 7. Benchmark, §§7/11.6 | sustentado | NVIDIA, celdas de código, población/costes/cache leídos; FinanceBench externo no atribuido al runner. |
| 8. Matriz, §§8/11.6 | parcial | Capacidades V existentes; fila Studio depende de PI-1. “V2 histórico” en 238 es etiqueta residual frente al pin actualizado. |
| 9. Negativos, §9 | sustentado | Spans exactos/generación integral no acreditados, no bugs Vanta fabricados. |
| 10. UI, §§10/11.6 | parcial | App anunciada externa, no frontend inspeccionado; Studio mal atribuido a pin. |

### Hallazgo

1. **PI-1 · Alta — Mismo error de atribución Studio que LR-1.**
   Informe `pageindex.md:472–477,497–499`: PS1 usa ruta `desktop/` inexistente
   en S; PS2/PS3 citan JobChip/jobs que ni siquiera aparecen en `git ls-tree S`.
   **Consecuencia:** se infiere cancelación entre chunks desde bytes ajenos al
   SHA. **Corrección verificable:** separar fuente inmutable
   `feature-freez/desktop/.../MemoryLens.tsx` de la lectura del worktree actual;
   para jobs no basta corregir prefijo. Conservar existencia de Studio y declarar
   revisión/rangos realmente inspeccionados, sin iniciar UI.

### Fuentes, contrastes y límites

- PI `local_store.py:15–40,96–125,168–186`: fsync/replace por archivo,
  fcntl ausente en Windows, meta publicada después de tree/pages, delete
  quita meta primero y limpia directorio best-effort. No commit multiarchivo.
- `local_api.py:290–347` usa posición de lista para texto del árbol,
  `_require_pages` sólo comprueba ausencia/vacío; `349–356` anuncia ready.
  `agent_tools.py:1082–1127` resuelve por page_index y admite primera página
  aunque exceda presupuesto. Identidad y cobertura no son conteo de tokens.
- `agent_tools.py:1422–1455`: identificadores validados y defaults con `!r`.
  El string de #585 permanece literal; no se ejecutó `exec` ni repro. Objetos
  Python arbitrarios y origen completo del schema no se certifican seguros.
- Cinco fuentes oficiales leídas: [#648](https://github.com/VectifyAI/PageIndex/issues/648),
  [#561](https://github.com/VectifyAI/PageIndex/issues/561),
  [#637](https://github.com/VectifyAI/PageIndex/issues/637),
  [#569](https://github.com/VectifyAI/PageIndex/issues/569) y
  [#585](https://github.com/VectifyAI/PageIndex/issues/585), todas abiertas
  en la consulta. #648 declara caches anormales construidas, no desorden espontáneo;
  #637 dice cambio cloud no desplegado; #569 reporta 0.2.22 y distingue deadline
  de max_turns. PRs asociadas no prueban merge/release corregida.
- Notebook `cookbook/pageindex-vs-vector-10k/pageindex-vs-vector-10k.ipynb`,
  celdas de código 2–22 leídas como JSON, no ejecutadas: cinco PDFs con SHA256,
  preguntas desde main, 63 configuradas, NumPy top-k 10/30/50/100, chunks
  512/64, modelos strings declarados y SDK probado 0.4.1 distinto del paquete
  del checkout. Accuracy divide por QUESTIONS; coste sólo respuestas presentes,
  cache discount asimétrico y costes de ingesta/juez fuera de ese USD/1000.
  `collect` identifica done por ID y conserva última fila por (ID,sistema);
  vector/judge pueden abortar pool, PageIndex guarda error. No reset automático.
- V ingestors/merge/assembler/checkpoints son seams verificables; no se
  certifican todos los parsers, lanes de chat, permisos cloud o App SaaS.

## Serviette

**Recibo:** [serviette.md](../serviette.md), lectura completa 1–483; pin SE.
**Dictamen propuesto: cambios requeridos**, por evidencia Studio mal atribuida.

### Ledger 0 + 10

| Eje / sección | Estado | Evidencia o laguna |
|---|---|---|
| 0. Arquitectura/flujos, §§1/2/11 | sustentado | Metadata→parse→split→embedding→sinks/retractions; indexer/API con owners distintos. |
| 1. Recomendaciones, §1 | sustentado | Identidad reproducible/frescura, no memoria autobiográfica ni runtime Pathway en Rust. |
| 2. Fichas A–D, §§2/11 | sustentado | Once campos, manifest/fingerprint/lifecycle/coverage distintos; SE-1 afecta JobChip. |
| 3. Incidencias, §12.1 | sustentado | #53/#46 abiertos de 2023, sin versión/fix acreditados; no se usan como defectos actuales del pin. |
| 4. Prerrequisitos, §4 | sustentado | Parse fallido distinto de vacío; revisión/manifest y autorización existentes antes de propuesta. |
| 5. Rechazos, §5 | sustentado | No duplicar BM25 RAM/RRF; licencia wrapper no licencia Pathway xpack. |
| 6. Método/licencia, §§6/11/12 | parcial | MIT wrapper acotado; identidad de evidencia Studio requiere corrección. |
| 7. Benchmarks, §§7/12.2 | sustentado | FRAMES QA/recall artículos y realtime hit-fichero/PSS distintos; down -v advertido. |
| 8. Matriz, §§8/12.3 | parcial | Se preservan capacidades V; fila Studio depende de SE-1. |
| 9. Negativos, §§9/12.3 | sustentado | No fingerprint integral acreditado no significa ausencia de ingesta/dedup. |
| 10. UI, §§10/12.4 | parcial | HTML same-origin real; Studio/jobs no pertenecen al pin declarado. |

### Hallazgo

1. **SE-1 · Alta — Atribución de JobChip y jobs ajenos al pin.**
   Informe `serviette.md:455–461,481–483`. Evidencia discrepante: árbol S y
   `git show` descritos en LR-1. **Consecuencia:** se acredita lifecycle/cancel
   con una revisión que no tiene esos archivos. **Corrección verificable:**
   ruta/rangos reales de MemoryLens al pin y lectura del worktree separada para jobs;
   mantener desconocida durabilidad UI, no inventar otro frontend.

### Fuentes, contrastes y límites

- SE `indexer/graph.py:550–665,721–761`: ParseError→texto vacío→cero chunks;
  UDF memoizada y whitelist de splitter puro, SHA256 sobre metadata canónica
  sin seen_at + texto; no ID de ocurrencia/offset. Metadata cambiante produce
  retract/alta; no implica commit atómico común de los sinks.
- `indexer/fingerprint.py:145–225`: identidad de splitter/parser/embedder/
  librerías/esquema; archivo ilegible se reescribe y drift permite confirmación.
  El informe no normaliza ese comportamiento como garantía segura.
- `server/hybrid.py:1–110`: BM25 RAM/RRF opcional y refresh con índice previo;
  omisión por tamaño no equivale a “sin evidencia”. Consulta no controla
  cancelación del indexer; materialized views no son memoria personal.
- FRAMES `benchmarks/frames/run_bench.py:1–139`: defaults k=8/concurrency=6,
  judge 4o-mini; timeout/retry de HTTP, gold vacío→1; promedia judged, pool
  puede abortar, tokens sólo juez y artefactos sensibles. No QA propia medida.
- Realtime `run_bench.py:226–250,288–335,393–423`: hit por sufijo expected_file
  sobre todas las preguntas; error HTTP propaga; default 100mb/k=5; preparación
  fuera de medida y **compose down -v** destructivo antes del arranque.
  PSS del indexer no RAM de Qdrant/API; exact-count no fecha de HNSW ready.
- [#53](https://github.com/pathwaycom/serviette/issues/53) y
  [#46](https://github.com/pathwaycom/serviette/issues/46) leídos íntegros:
  abiertos, sin SHA/fixed release; anidamiento JSON y streaming del wrapper
  antiguo no diagnostican el código de 2026. Parse→vacío es observación estática.
- `frontend/static/index.html:422–458` hace POST, maneja error/answer/notice/
  sources, sin AbortController allí. No certifica permisos backend ni cese de
  trabajo al cerrar pestaña. V ingestors, assembler, worker/progreso y checkpoints
  son seams existentes; Pathway interno, ocho accessors y auth integral no auditados.

## Meilisearch

**Recibo:** [meilisearch.md](../meilisearch.md), lectura completa 1–603; pin ME,
rama main. **Dictamen propuesto: cambios requeridos**, por ME-1/ME-2.
La prohibición de forge impide acreditar incidencias y no es defecto del producto.

### Restricción efectiva de esta revisión

Se leyó localmente `C:\Users\Eros\meilisearch\AGENTS.md` y se respetó su
prohibición: ninguna modificación/sugerencia de modificación al clon y ningún
acceso propio a issues, PRs, discussions, Actions o releases Meilisearch.
Los títulos/listados que el autor declaró haber visto **no se reabrieron**.
No se hizo fallback a mirrors de incidencias ni descarga del build.zip
desde releases. Las correcciones siguientes se dirigen exclusivamente al
informe de VantaDB. Leer código local o el cliente CDN no acredita la forge.

### Ledger 0 + 10

| Eje / sección | Estado | Evidencia o laguna |
|---|---|---|
| 0. Arquitectura/trazas, §§0/11 | sustentado | API→payload→cola→batch→índice; settings/search read txn y cancel/upgrade separados. |
| 1. Recomendaciones, §1 | sustentado | Outcome/explicación, no sustituir RRF por ratios ni importar daemon. |
| 2. Fichas A–D, §§2/11 | sustentado | Once campos, seams de worker/checkpoint/receipt; A outcome, B/C explicación/filtro, D generación. |
| 3. Incidencias públicas, §§3/12.1 | no acreditado | Prohibición expresa. Títulos observados por el autor no son cuerpos, causa, vigencia, fix o release revisados. Observaciones estáticas se mantienen separadas. |
| 4. Prerrequisitos, §4 | sustentado | Ya existen híbrido/scheduler/checkpoints/recibos; no aceptación desde task succeeded. |
| 5. Rechazos, §5 | sustentado | No autorización por filterable ni undo universal por cancelación. |
| 6. Método/licencias, §§6/11 | parcial | MIT AND BUSL-1.1/EE explícitos; no SBOM per-file completo. Studio mal atribuido ME-1. |
| 7. Benchmarks, §§7/12.2 | parcial | movies/xtask/WaitForTask leídos; se omiten dos workloads de embeddings relevantes: ME-2. |
| 8. Matriz, §§8/12.3 | parcial | Capacidad existente respetada; filas UI/generación necesitan evidencia ME-1/ME-2, no otra cola. |
| 9. Negativos Vanta, §§9/12.3 | sustentado | Preguntas de consistencia, no bugs reproducidos ni performance propia. |
| 10. UI, §§10/12.4 | parcial | App.jsx CDN es código real a tag, build ZIP/SHA frontend no resueltos; Studio incorrecto ME-1. |

### Hallazgos

1. **ME-1 · Alta — JobChip/jobs no están en el pin Studio citado.**
   Informe `meilisearch.md:577–584,599–601`. Evidencia local S descrita en
   LR-1: MemoryLens sólo bajo `feature-freez/desktop/`; no JobChip/jobs en
   el árbol. **Consecuencia:** cancelación/IDs parciales se acreditan con
   bytes de otra identidad. **Corrección verificable:** ruta correcta para
   MemoryLens y recibo de lectura del worktree separado para los dos archivos,
   o retirar la atribución al pin; no fingir falta de Studio.

2. **ME-2 · Media — El benchmark no cubre cambio de embedder pese a proponer D.**
   Informe `meilisearch.md:288–290,427–458,491–528`.
   Evidencia ME local: `workloads/embeddings-settings-add.json:1–65` y
   `embeddings-movies-subset-hf.json:1–61`. Ambos fijan cinco runs, cuatro
   threads y movies-100 SHA256
   `d215e395e4240f12f03b8f1f68901eac82d9e7ded5b462cbf4a6b8efde76c6c6`.
   En el primero documentos/settings son **precommands** y la medición
   añade embedder HuggingFace; en el segundo el embedder se prepara antes
   de medir la inserción. Modelo/revisión null o defaults no son identidad
   congelada del artefacto del modelo.
   **Mecanismo/consecuencia:** extrapolar movies a D oculta qué preparación,
   descarga/caché y embedding quedan dentro/fuera de spans. No se declara gasto
   LLM inexistente sólo porque no haya juez QA.
   **Corrección verificable:** añadir ambos recorridos al informe o justificar
   exclusión de uno con alcance preciso; defaults de modelo no resueltos,
   población, fase medida, reset, red/caché y coste de recursos explícitos.
   No ejecutar workloads, crear harness ni modificar código/config upstream.

### Fuente, commit, cancelación y readiness

- ME `index-scheduler/src/lib.rs:960–1018`: registra en write txn y commit
  antes de notify; la señal must_stop para cancel se solicita antes del commit
  de esa petición. ACK de cola no confirma commit del índice ni undo previo.
  `scheduler/process_batch.rs:174–219` commit de índice y estadísticas
  independientes; `1234–1295` rollback de upgrade específico, no undo de ingesta.
- `milli/src/search/hybrid.rs:28–112`: comparación secuencial de scores con
  ratios y sorts, no RRF ni probabilidad de verdad; brazos keyword/semantic
  pueden omitirse. Read txn no ve obligatoriamente commit posterior.
- Bench `xtask/src/bench/workload.rs:22–85,139–207`: precommands fuera
  de captura; delete_db, start y kill son **efectos del runner leído**, nunca
  ejecutados aquí. `xtask/src/common/command.rs:93–185` espera desaparición
  global de enqueued/processing, no valida éxito de la tarea exacta.
  `movies.json:1–43` fija diez runs y SHA256 del asset, no datos materializados.
  Agregados de spans no son percentiles HTTP/recall/QA; errores pueden impedir
  reporte. Dashboard de métricas externo no auditado.
- UI local `crates/meilisearch/build.rs:19–76`, Cargo metadata `170–172`:
  embebe recursos del ZIP, comprueba SHA1
  `6a8a76d9ed79357959bc6d4e3816f753a605a27b`. Eso identifica bytes esperados,
  **no SHA de árbol frontend**, procedencia criptográfica ni licencia per-file.
  Release ZIP no descargado ni comparado por restricción.
  [App.jsx CDN v0.4.2](https://cdn.jsdelivr.net/gh/meilisearch/mini-dashboard@v0.4.2/src/App.jsx)
  reabierto: React/SDK, health, índices/stats, clave en memoria y aceptada en URL.
  No código completo de frontend ni equivalencia byte-a-byte ZIP↔tag acreditada.
  El tag puede cambiar; se preserva el límite, sin sortear forge para resolverlo.
- V worker/progreso/TaskCheckpointManager/WriteReceipt ya existen; no nuevo
  scheduler/receipt desde cero. Cobertura uniforme accepted→applied→visible,
  propagación por bindings y auth/retención de diagnósticos siguen parciales.
  No auditados kernels completos, EE, todos los providers ni recuperación física.

## Límites transversales y entrega

- **Siete recibos guardados**, cada uno con 0+10, hallazgos o ausencia acotada,
  evidencia, fuentes y límites. Se proponen cambios en seis; CrewAI queda sin
  hallazgos bloqueantes en la muestra. No se acepta ningún informe finalmente.
- LR-1/PI-1/SE-1/ME-1 son cuatro ocurrencias de **una causa de procedencia**.
  No cuatro defectos runtime ni una razón para negar la existencia de Studio.
  El responsable debe solicitar rectificación de evidencia al autor de esos
  informes, conservando archivos no rastreados del worktree separados del commit inmutable.
- LC-1 afecta un adapter secuencial; LG-1 es política del host al resume.
  No se fusionan con RMW de `utils/checkpoint.rs::mutate`, escritura L1, WAL recovery
  o atomicidad de transacciones. No se sostiene lock durante llamada LLM.
- Seguridad acreditada aquí = frontera y mecanismo identificados, no auditoría
  exhaustiva ni paridad. Licencia de wrapper/raíz no cubre SDKs remotos, modelos,
  corpus, servicios ni todos los componentes transitivos.
- Resultados publicados/repros de terceros no son resultados propios. Leer una
  prueba no acredita que pasó. Fallo de acceso o texto truncado no acredita lo
  omitido. Las métricas se mantienen por pista y unidad; caracteres, tokens,
  bytes de cuota/almacenamiento y presupuesto de llamadas no se intercambian.
- Verificación final permitida: inspección estática de este archivo, sus rutas,
  enlaces y consistencia de recibos. No se ejecuta validación global de docs ni
  comando de prueba/build/benchmark. La prohibición específica de
  staging/commit/push prevalece para esta revisión documental; no implementación
  satisfactoria ni autorización de publicar.
- El archivo queda guardado en el proyecto, pero `.gitignore:268` ignora
  `docs/dev/research/`; no aparece en el status ordinario por ese motivo.
  No se cambia esa regla ni se fuerza staging. Versionar/publicar corresponde
  al responsable cuando exista autorización expresa.
- División futura: esta entrega conserva un único ledger por restricción del
  encargo. Si se autoriza otra organización, separar por proyecto manteniendo
  procedencia común y enlaces; no se crean archivos ni índices ahora.

[lc-core]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/vectorstores/base.py#L234-L291
[lc-time]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain/langchain_classic/retrievers/time_weighted_retriever.py#L64-L198
[v-saver]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/checkpointer.py
[v-store]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/store.py#L97-L174
