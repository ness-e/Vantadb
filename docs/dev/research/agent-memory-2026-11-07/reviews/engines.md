---
title: "Revisión adversarial independiente de motores"
kind: research
status: proposed
description: "Contraste estático de Qdrant, Faiss, Redis, TiDB y SurrealDB contra sus clones congelados y la referencia VantaDB"
tags: [research, memory, engineering]
---

# Revisión adversarial independiente de motores

## Alcance y método

Revisión independiente de la autoría; no aceptación final. Se aplica el
[contrato](../EXPANSION-CONTRACT.md), la [referencia Vanta](../VANTA-REFERENCE.md),
[METHOD](../METHOD.md), [BENCHMARK-NOTES](../BENCHMARK-NOTES.md) y
[REVIEW-COVERAGE](../REVIEW-COVERAGE.md), más el brief local asignado.
Lectura de los informes íntegros; contraste estático dirigido, no auditoría
exhaustiva. Los rangos de informe corresponden al documento observado durante
esta revisión, no a una versión publicada. Fecha de consulta pública: 2026-11-07.

HEAD Vanta comprobado: `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.
Los cinco HEAD upstream coinciden con el manifiesto, sin fetch.
Las rutas upstream de cada sección son relativas al clon
`C:\Users\Eros\<proyecto>`; las rutas Vanta son relativas a este repositorio.
Se cargaron documentation-skill, coordinated-web-search y codebase-memory;
se consultó CodeGraph en lectura. No se reindexó ni se modificó configuración.
La revisión adversarial se realiza en este contexto independiente; no se invocó
otra CLI/modelo ni se inició un ciclo anidado de subagentes.

No se ejecutaron apps, tests, builds, benchmarks, servicios, instalaciones,
validadores globales ni operaciones git de escritura. No se iniciaron ni
detuvieron runtimes. No hay cifras propias ni certificación de seguridad,
durabilidad física, calidad, rendimiento o paridad de bindings.

## Índice

- [Qdrant](#qdrant)
- [Faiss](#faiss)
- Las secciones restantes se incorporan al terminar su contraste.

## Qdrant

**Informe:** [qdrant.md](../qdrant.md), 654 líneas leídas.
**SHA:** `222b2e44ab1c3f17e8246396ead6558e45041df7`.
**Veredicto propuesto: cambios requeridos.**

### Ledger 0 + 10

| Eje | Estado | Sección del informe y evidencia/laguna |
|---|---|---|
| 0 Arquitectura/identidad | parcial | §1, Q5/Q10/Q11: fence, WAL, worker y tombstone diferenciados; falta asimetría de espera sin timeout (Q-R1). SHA comprobado; Apache-2.0 en manifests segment/edge y LICENSE. |
| 1 Recomendaciones | sustentado | §1/§4: no reemplazar ANN, extender explicación y recibos; dependencias por candidato. |
| 2 Fichas Rust | parcial | §2 A-E: alternativas, seams, FMEA, rollback y aceptación futura concretos; contrato de aceptación sin representación explícita (Q-R2). |
| 3 Incidencias | sustentado | §3: páginas #9575, #10851 y #9869 leídas independientemente; cerrado no se usa como prueba única; PoC y merge/release no acreditados. |
| 4 Prerrequisitos Vanta | sustentado | §4 V1-V7; `put_certified`, `delete_inner`, `delete_by_filter`, explain y rebuild existentes; no bloqueo universal por C13. |
| 5 Rechazos | sustentado | §5: clúster, defaults, sidecar y Edge rechazados con costes y fronteras; no negar Edge embebido. |
| 6 Método/licencia | sustentado | §6: raíz/segment/edge separados de transitivas y UI externa; no ejecutar ni derivar corrección desde tests. |
| 7 Benchmarks | sustentado | §7 Q7/Q14 contrastados: población completa frente a 1 000 offsets, división entera, plain no top-10, caché y trabajo fuera/dentro del closure; no QA ni resultados propios. |
| 8 Matriz Vanta | sustentado | §8 y V1-V7: existe/parcial/no acreditado/no aplicable con decisiones; explain y certificados no se reinventan. |
| 9 Negativos propios | sustentado | §9: comentario dry-run contradicho por llamadas; `fusion_report: None` en ramas concretas; batches/delete no se elevan a corrupción ni resurrección Vanta. |
| 10 UI/escritorio | sustentado | §10 y `src/actix/web_ui.rs:13-43`: servido web comprobado, frontend/escritorio no certificado; destino Studio existente, no nueva app. |

### Hallazgos

1. **Media — Q-R1, pérdida de estado en la espera sin timeout omitida.**
   Informe `qdrant.md:119-124,253-273` (§1.2 y B).
   Fuente: `lib/collection/src/shards/local_shard/shard_ops.rs:180-202`:
   `(Some(receiver), None)` descarta el `InternalUpdateResult` y fabrica
   `Completed`; con timeout conserva `status`. En
   `lib/collection/src/update_workers/update_worker.rs:211-240`, la espera
   deferred puede emitir `WaitTimeout` como resultado exitoso del canal.
   No basta enumerar los cuatro estados sin mostrar esta asimetría.
   Consecuencia: una adaptación de B puede elevar feedback parcial a finalización
   completa. **Corrección verificable:** describir ambas ramas y la emisión
   deferred; dejar revisión/configuración de alcanzabilidad y reproducción
   pendientes. No afirmar fallo de Vanta ni power-loss por este contraste.

2. **Media — Q-R2, aceptación declarada independiente pero no definida en el DTO.**
   Informe `qdrant.md:32-43,196-199,253-273`.
   El `MutationReceipt` propuesto enumera commit, durabilidad, proyecciones,
   generación y `outcome`, pero no define si aceptación reside en `outcome`,
   qué valores admite ni cómo representa aceptado con commit desconocido.
   Fuente discriminante: Q5 `SubmitOutcome`/`await_update_result:165-218`
   distingue admisión, rechazo de clock y espera vencida; Vanta
   `src/sdk/api/memory.rs:1261-1277` aporta una atestación diferente.
   Consecuencia: los SDK pueden volver a codificar una escalera excluyente
   dentro de `outcome`, pese a la advertencia narrativa correcta.
   **Corrección verificable:** definir localmente el campo o vista de aceptación,
   su desconocimiento y relación con errores, sin alterar el contrato común en
   esta revisión; incluir caso aceptado/commit desconocido, primario durable con
   proyección pendiente y timeout posterior a dispatch. No exigir fsync de índices.

### Evidencias contrastadas y límites

- Código: Q1 `dispatch.rs:40-190`; Q4 `reciprocal_rank_fusion.rs:13-98`;
  Q5 `append_and_dispatch`/`await_update_result:100-218`;
  Q10 `update_worker_internal:345-399`, feedback deferred `:133-244`,
  `CollectionUpdater::update:41-107`; Q11 `resolve_submit.rs:1-124`.
  El fence drena antes de resolver y no retiene el lock WAL durante el drain.
- Runners leídos: `conditional_search.rs:1-212`,
  `hnsw_search_graph.rs:1-116`, `fixture.rs:1-79`.
  No se leyeron internals de Criterion ni se certificaron resultados.
- Vanta: CodeGraph aportó `vector_memory_search`, `put_certified`,
  `delete_inner`, rebuild; lectura adicional `explain.rs:9-117` y
  `namespaces.rs:251-312`. Error tras primario/derivados no significa rollback.
- Red pública, MCP descubierto `websearch.fetchWebContent`, lectura efectiva:
  [#9575](https://github.com/qdrant/qdrant/issues/9575),
  [#10851](https://github.com/qdrant/qdrant/issues/10851),
  [#9869](https://github.com/qdrant/qdrant/issues/9869).
  #10851 llegó truncado: se leyó cuerpo/revisión/PoC inicial, no discusión
  completa. Las otras dos páginas no llegaron truncadas. No hubo fallo de
  acceso que exigiese escalar el router para estas URLs conocidas.
- Dos lecturas locales iniciales usaron rutas erróneas del revisor
  (`update_workers.rs`, `lib/shard/src/collection_updater.rs`); se corrigieron a
  las rutas Q10 del informe y se leyeron. **No son defectos del informe.**
- No se contrastaron todos los campos/ramas de Q2/Q3/Q6/Q9/Q12/Q13 ni la cadena
  transitiva de `wal.flush`, consenso, UI externa, ACL end-to-end o crash físico.
  La coincidencia de símbolos no sustituye esos límites.

## Faiss

**Informe:** [faiss.md](../faiss.md), 563 líneas leídas.
**SHA:** `277b3c76b7a1507915eb8f5010dcb367cbc95494`.
**Veredicto propuesto: sin hallazgos bloqueantes en este alcance.**
No es aceptación definitiva ni validación dinámica del negativo CPIndex.

### Ledger 0 + 10

| Eje | Estado | Sección del informe y evidencia/laguna |
|---|---|---|
| 0 Arquitectura/identidad | sustentado | §0: biblioteca CPU, train/add/search/remove/reopen, ID externo frente a posición, ownership y ausencia de commit de aplicación; MIT raíz y cabecera del runner verificadas. |
| 1 Recomendaciones | sustentado | §1: prioridad por negativo propio condicionado, no catálogo ANN; rechaza sustitución sin evidencia. |
| 2 Fichas Rust | sustentado | §2 A-E/2.1: operaciones, errores, publicación, ownership, rollback, FMEA y aceptación; no impone epochs Faiss como feature observada. |
| 3 Incidencias | parcial | §3: #4655/#4656 confirmadas por HTML; #4535 cuerpo correcto pero estado/discusión pendientes en autoría; ahora legibles (F-R1). FAQ distingue CPU/GPU y caller lock. |
| 4 Prerrequisitos Vanta | sustentado | §4: generación, capacidad y DTO por ficha; recheck exacto existente; no equipara checkpoint con índice. |
| 5 Rechazos | sustentado | §5: FFI/motor nuevo, delete-add atómico, autotuning con gold y dedup geométrico rechazados con alternativas. |
| 6 Método | sustentado | §6: representación/precisión, no runtime ni GPU inferidos; issues 1.11 no declarados reproducidos en el SHA. |
| 7 Benchmarks | sustentado | §7: `bench_hnsw.evaluate`, `eval_setting`, criterio autotune y dataset OSS contrastados; any primer vecino, intersección y huecos usan denominadores distintos; no QA ni cifras propias. |
| 8 Matriz Vanta | sustentado | §8: algoritmos, filtros y lectores existentes; frescura y diagnóstico parciales, no ausencias globales. |
| 9 Negativos propios | sustentado | §9: condición cardinalidad en cache frente a update mismo ID y persistencia condicional; no declara corrupción ni fallo en todos los callers del SDK. |
| 10 UI/escritorio | sustentado | §10: Studio existente; `RetrievalLens.tsx`/`ActivityPanel.tsx` efectivamente AM en worktree desktop; evidencia de disco no atribuida al commit congelado. |

### Hallazgos

No se encontró contradicción bloqueante en los mecanismos contrastados.

1. **Baja — F-R1, laguna pública ahora resoluble, no falsedad de autoría.**
   Informe `faiss.md:220-229,324,528-531,556-557`.
   La página [#4535](https://github.com/facebookresearch/faiss/issues/4535)
   permitió leer estado cerrado, etiqueta `wontfix` y discusión del 2025-10-08.
   Allí se documenta un mecanismo más específico que cardinalidad:
   IDMap usa `index->add_ex` sin IDs; IVF genera `ntotal+i`; remove/re-add
   puede duplicar IDs internos. El primer intento del contributor no reproducía
   porque repetía remove sin re-add; el autor aporta esa diferencia el 7 de octubre.
   Cierre no equivale a fix: la resolución recomienda no envolver IVF.
   **Consecuencia:** la ficha C podría ganar una discriminación más precisa,
   pero ya separa identidad/cardinalidad y no inventa una reparación.
   **Corrección verificable recomendada:** ampliar el registro de #4535 con esa
   discusión y fecha/estado observados; conservar pendiente el resultado runtime
   y la aplicabilidad a Vanta. No es motivo para reescribir la matriz como bug Vanta.

### Evidencias contrastadas y límites

- `IndexIVF.cpp:529-615` (selector/presupuesto/modos), `:1385-1410`
  (remove y error después de remove en hash), `:1543-1626`
  (entrenamiento/merge/ownership); `IndexIDMap.cpp:275-296` y
  `IndexFlatCodes.cpp:61-80` (compactación).
- `ProductQuantizer.cpp:58-78` y `impl/index_read.cpp:1190-1260`:
  validaciones concretas, no prueba de seguridad integral de todos los readers.
- Runners: `benchs/bench_hnsw.py:1-134`,
  `bench_all_ivf.py:65-98,355-396,423-460`,
  `datasets_oss.py:1-23`, `contrib/datasets.py:147-190`.
  El import preferente interno y el uso del gold para elegir operating points
  impiden llamar holdout independiente a esa reevaluación.
- Vanta: CodeGraph `search_ivf/search_scann` y lecturas
  `graph/core.rs:321-374`, `serialize/bytes.rs:224-239,611-626`,
  `ivf.rs:104-150`. Reutilizar el mismo CPIndex después de update con igual
  cardinalidad es la condición acreditada; caller productivo/reproducción no.
- Desktop: `RetrievalLens.tsx:121-164` y `ActivityPanel.tsx:54-83`;
  estado AM verificado sin git writes. Filtro visual no es autorización.
- Red MCP `websearch.fetchWebContent`: páginas
  [#4656](https://github.com/facebookresearch/faiss/issues/4656),
  [#4655](https://github.com/facebookresearch/faiss/issues/4655),
  [#4535](https://github.com/facebookresearch/faiss/issues/4535) y
  [FAQ Markdown](https://raw.githubusercontent.com/wiki/facebookresearch/faiss/FAQ.md).
  Lectura efectiva de cuerpos/discusión visibles y apartados FAQ de precisión,
  concurrencia/ownership/IDs; no se certifican timestamps UTC exactos de la API
  del autor ni toda la FAQ. No hubo error de acceso del revisor a estas fuentes.
- No se auditó GPU/cuVS, cadena legal transitiva, readers completos, todas las
  ramas OpenMP, ni extremo a extremo del SDK. Receipt durable/readiness no
  aplica al contrato de `Index`: el informe correctamente no fabrica un ACK
  transaccional desde una operación C++ terminada o un lock del integrador.
