---
title: "Revisión independiente de cinco informes de memoria"
kind: research
status: proposed
description: "Dictámenes adversariales delimitados de Hindsight, Agentmemory, Supermemory, TencentDB Agent Memory y Letta"
tags: [research, memory, engineering]
---

# Revisión independiente de cinco informes de memoria

## Índice

- [Alcance y recibos](#alcance-y-recibos)
- [Hindsight](#hindsight)
- [Agentmemory](#agentmemory)
- [Supermemory](#supermemory)
- [TencentDB Agent Memory](#tencentdb-agent-memory)
- [Letta](#letta)
- [Cierre de revisión](#cierre-de-revisión)

## Alcance y recibos

Revisión independiente de la autoría, el 2026-11-07. Se aplican el
[contrato](../EXPANSION-CONTRACT.md), la [referencia Vanta](../VANTA-REFERENCE.md),
el [método](../METHOD.md), las [notas](../BENCHMARK-NOTES.md) y la
[cobertura](../REVIEW-COVERAGE.md). Dictámenes **propuestos**, no aceptación
final, certificación de ejecución ni autorización de implementación.
La aceptación final de **cada uno de los cinco** permanece con el responsable
principal; ningún dictamen de esta revisión autoaprueba un informe o candidato.
Único archivo editado: esta revisión. Sin agentes anidados, fetch Git,
instalaciones, aplicaciones, tests, builds, benchmarks, servicios ni git writes.

Referencia motor: `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, leída con
`git show`; los archivos concurrentes no se atribuyen a ese commit.
Se cargaron documentation-skill y coordinated-web-search y se consultó el
catálogo de skills. No se hizo mantenimiento de grafos ni diseño de harness,
prompts, métricas, umbrales, rúbricas o configuración.

## Hindsight

**Recibo:** leído íntegro [hindsight.md](../hindsight.md), líneas 1–622;
clon `C:\Users\Eros\hindsight`, HEAD comprobado
`eb963cb5a929d92c081371df64c2d5c2fe3351af`.
**Veredicto propuesto: cambios requeridos.**

### Ledger 0 + 10

| Eje | Estado | Evidencia / límite |
|---|---|---|
| 0. Arquitectura | sustentado | §§1–2: retain/documento/facts/derivados, recall multibrazo, reflect y borrado; CAS de delta y locks SQL delimitados; no transacción LLM global |
| 1. Recomendaciones | sustentado | Priorización A/E/F frente a B/D/C/G, sin sustituir capacidades existentes |
| 2. Fichas Rust | sustentado | A–G y complementos: problema, unidad, delta, alternativas, contratos propuestos, seam, pasos/migración/rollback, dependencias, FMEA, aceptación/observabilidad y no objetivos; primitiva condicional sigue abierta |
| 3. Incidencias | parcial | Cuatro fuentes públicas efectivamente leídas; el informe omite evidencia concreta de correcciones en dos de ellas, H-1 |
| 4. Prerrequisitos | sustentado | §§4 y fichas: validación por candidato, no runtime faltante como bloqueo universal |
| 5. Rechazos | sustentado | §5 separa producto SQL/Python, prompts y nomenclatura; lectura no obliga a portar |
| 6. Método | sustentado | §6 y cierre distinguen estático, reporte, propuesta e hipótesis; MIT no cubre pesos/proveedores |
| 7. Benchmarks | sustentado | §7 y complemento: AMB separado del runner antiguo, denominadores QA/retrieval, reset remoto, cachés, errores y costes sin resultado propio |
| 8. Matriz | sustentado | §§8 y matriz trazable: híbrido, scopes, versiones, scheduler, budgets y WriteReceipt existentes; SHA-256 no firma, C13 propuesto |
| 9. Negativos Vanta | sustentado | §9 formula preguntas, no inexistencias globales ni fallos heredados de OpenViking |
| 10. UI | parcial | Código upstream web real; atribución de rutas Studio al commit separado incorrecta, H-2 |

### Hallazgos

1. **Media — H-1: historial de correcciones incompleto, no mera ausencia de prueba.**
   Informe `hindsight.md:525-526`, símbolos
   `_sources_changed_since_read` / `RequestContext.raise_if_cancelled`.
   [#4831](https://github.com/vectorize-io/hindsight/issues/4831), leído con
   comentarios, contiene declaración del mantenedor del 2026-10-05:
   cambios #4893 y #4897, este último preserva tags actuales bajo bloqueo.
   En el SHA `eb963cb5a929d92c081371df64c2d5c2fe3351af`,
   `hindsight-api-slim/hindsight_api/engine/consolidation/consolidator.py:3445-3495`
   confirma `lock_observation_tags` y merge desde `current_tags`, protección
   distinta de revisar fuentes. [#2122](https://github.com/vectorize-io/hindsight/issues/2122)
   presenta cierre por #2127, commit `07c85da` y etiqueta Release v0.8.2.
   **Mecanismo:** leer solo cuerpo/cierre y señalar únicamente la protección
   adyacente pierde la corrección específica ya documentada y presente.
   **Consecuencia:** análisis de vigencia y transferencia menos preciso; no
   invalida A/G ni prueba por sí solo paridad runtime.
   **Corrección verificable:** registrar ambos PR de tags y el PR de cancelación,
   diferenciar declaración del mantenedor, código confirmado y versión no
   verificada mediante artefacto; mantener pendientes los límites end-to-end.

2. **Media — H-2: fuente Studio de disco atribuida a rutas que no existen en el SHA.**
   Informe `hindsight.md:592-598`: `desktop/src/components/{memory/MemoryLens,
   activity/ActivityPanel,lens/retrieval/RetrievalLens}.tsx` al SHA
   `0ad38c542b39a589035b0e17fdedb4c09dca24b9`.
   `git ls-tree -r --name-only` de ese commit ubica los tres bajo
   **`feature-freez/desktop/src/components/`**; `git show SHA:desktop/...`
   falla. Las tres rutas nuevas figuran `AM` en el worktree.
   **Mecanismo:** confundir lectura de archivos movidos/modificados con contenido
   inmutable de HEAD. **Consecuencia:** seam no reproducible en la revisión
   afirmada, aunque Studio sí exista.
   **Corrección verificable:** citar rutas `feature-freez/desktop/...` y comparar
   sus cuerpos a ese SHA, o declarar explícitamente lectura dirty separada sin
   atribuirla al commit; no crear otra UI por este error de evidencia.

### Fuentes contrastadas y límites

- **Clon leído por cuerpos/rangos:** `engine/retain/orchestrator.py:1378-1485,
  4133-4177,4280-4335`; `engine/search/fusion.py:1-110`;
  `search/retrieval.py:129-248`; `consolidation/consolidator.py:655-776,
  3445-3495`; `memories/pg/documents.py:321-348`;
  `mental_model_refresh.py:1-65`; `reflect/agent.py:503-560`;
  `models.py:15-74`; `worker/poller.py:1286-1325`;
  `hindsight_system_evals/amb.py:1-123`, `retrieval_metrics.py:1-85`;
  control plane `observation-history-view.tsx:1-100`, `api/recall/route.ts:1-65`.
- **Vanta leído al snapshot:** `l1_writer.rs:117-315` distingue plan/persist/log,
  degradación de embedding y tenancy; `auto_recall.rs:302-335` conserva
  `RecallScope`, budgets y `core_search=false`; `src/attestation.rs:657-735`
  explicita hash, exclusiones y ausencia de firma. Studio: árbol Git y
  `ActivityPanel.tsx:38-89` dirty, **no** cuerpo completo del snapshot.
- **Públicas leídas:** #6440, #6439, #4831 y #2122 mediante HTML GitHub.
  #6440/#6439 siguen open, fixes solo declarados en fork/parche; no merge
  upstream inferido. API keyless de #6440/#6439 devolvió rate limit; cuatro
  llamadas `github-mcp-server.issue_read` fallaron conexión; fallback HTML sí
  permitió lectura, no se trata el fallo como evidencia del contenido.
  URLs REST fallidas por `urllib`:
  `https://api.github.com/repos/vectorize-io/hindsight/issues/6440` y
  `https://api.github.com/repos/vectorize-io/hindsight/issues/6439`;
  error `API rate limit exceeded for 161.22.33.161`.
- **AMB leído remoto**, SHA `28b644bd4dd653c0207c7507280439390c829155`:
  `src/memory_bench/runner.py` (respuesta HTTP completa y tramos 185-260,
  400-475), `memory/hindsight.py:157-299,899-939`,
  `llm/__init__.py:21-40`. Se confirmó reset `banks.delete`, captura de fallo
  de delete, recall que degrada a vacío y agregado booleano/media de scores.
  Un intento de impresión local falló por encoding; se repitió con UTF-8.
- **No releído:** AMB loaders/jueces/modelos completos, Oracle, parser completo
  de extracción, SDKs, migraciones y autorización end-to-end. No se certifican
  seguridad multitenant, borrado físico, exactitud, coste ni latencia.

## Agentmemory

**Recibo:** leído íntegro [agentmemory.md](../agentmemory.md), líneas 1–535;
HEAD del clon `C:\Users\Eros\agentmemory` comprobado
`4f758eff36c8403570e1bf2ac6d986239fcdb662`.
**Veredicto propuesto: cambios requeridos.** No se encontró otro hallazgo
bloqueante en los cuerpos contrastados, además de la atribución visual AM-1.

### Ledger 0 + 10

| Eje | Estado | Evidencia / límite |
|---|---|---|
| 0. Arquitectura | sustentado | §§1/Modelo: capture→inbox→observe→completed, search y evolve/forget; persistencias separadas y cancelación no garantizada correctamente delimitadas |
| 1. Recomendaciones | sustentado | No portar iii; ack/fencing solo donde haya requisito, sin asumir coordinación distribuida |
| 2. Fichas Rust | sustentado | A–D con complementos cubren los 11 campos; contratos propuestos, no SDK actual; C invalidación y D privacidad no se sustituyen por DTO genérico |
| 3. Incidencias | sustentado | §Incidencias distingue reportes, PR abierta, cerrada sin merge y merge posterior; cinco fuentes leídas, con límites HTML abajo |
| 4. Prerrequisitos | sustentado | §4 y A/B: idempotencia, ack, política de retención y ownership dependen del candidato; no bloqueo universal |
| 5. Rechazos | sustentado | §5: no KV paralelo, runtime iii ni fingerprint como ID |
| 6. Método | sustentado | §6 y cierre: Apache-2.0 propio separado de SDK/motor/pesos; muestra no exhaustiva |
| 7. Benchmarks | sustentado | §7 y complemento: runner antiguo any-hit vs scorer por sesiones, teardown/init, sandbox y pagos separados; sin QA ni resultados propios |
| 8. Matriz | sustentado | Existentes híbrido/dedup/scheduler/budget; fencing no acreditado; WriteReceipt no firma, C13 propuesto |
| 9. Negativos Vanta | sustentado | §9 pregunta por contratos; no diagnostica defecto actual desde similitud |
| 10. UI | parcial | Viewer y bearer reales contrastados; fuente Studio incorrectamente atribuida a SHA, AM-1 |

### Hallazgos

1. **Media — AM-1: mismo cruce snapshot/worktree que H-2.**
   `agentmemory.md:517-521` atribuye `desktop/src/components/...` a
   `0ad38c542b39a589035b0e17fdedb4c09dca24b9`; ese commit contiene
   `feature-freez/desktop/src/components/...`, no las tres rutas afirmadas.
   **Fuente discrepante:** árbol Git del worktree `develop-desktop` a ese SHA;
   rutas de disco nuevas con estado `AM`, no evidencia inmutable.
   **Mecanismo/consecuencia:** seam visual leído fuera del commit declarado
   no se puede reproducir con sus referencias.
   **Corrección verificable:** releer/citar los cuerpos bajo `feature-freez`
   o etiquetar contenido dirty y su identidad separadamente; conservar el
   reconocimiento correcto de Studio existente.

### Fuentes contrastadas y límites

- **Leídas:** `capture.ts:226-356`, `keyed-mutex.ts` completo,
  `leases.ts:1-155`, `remember.ts:266-440`, `relations.ts:146-230`,
  `search.ts:116-155`, `privacy.ts:1-95`, `_capture-filter.ts:1-95`,
  `eval/runner/run.ts`, `score.ts` y `adapters/agentmemory.ts` completos;
  `sandbox.ts:59-110`; `benchmark/longmemeval-bench.ts:160-215`;
  viewer `index.html:2021-2075`, `server.ts:130-210`.
  `Map`/AsyncLocalStorage no es lease distribuido; forget marca deleted antes
  del borrado y expone fallos de limpieza; evolve puede fallar entre etapas.
- **Vanta al SHA de referencia:** `task_checkpoint.rs:129-188`,
  `ingest/merge.rs:220-265`, más writer/recall/attestation ya leídos.
  No se certifica CAS por la existencia de checkpoint ni ack fsync por un DTO.
- **Públicas efectivamente leídas por HTML:** [#1273](https://github.com/rohitg00/agentmemory/issues/1273)
  y [#1603](https://github.com/rohitg00/agentmemory/issues/1603) completas, open;
  [#1704](https://github.com/rohitg00/agentmemory/pull/1704) cabecera/body, open;
  [#1699](https://github.com/rohitg00/agentmemory/pull/1699) cabecera/body, closed,
  no merged observado; [#1701](https://github.com/rohitg00/agentmemory/pull/1701)
  cabecera/body, merged. Las tres conversaciones PR quedaron truncadas:
  no se acredita lectura íntegra de comentarios/diffs ni el timestamp exacto
  `15:53:22Z`/SHA de merge afirmados por el autor; no se contradicen sin evidencia.
  Los commits enlazados desde #1273 no prueban que su fix esté publicado.
- **No releído:** adapters vector/CLI completos, loaders, otros runners, iii
  externo, privacidad universal, spool y auth global. No se exige ejecutar
  nada para sostener esta revisión estática selectiva.

## Supermemory

**Recibo:** leído íntegro [supermemory.md](../supermemory.md), líneas 1–451;
clon `C:\Users\Eros\supermemory`, HEAD comprobado
`16ed414e0e0c61c946b7687ec813b2064fe75dd4`.
**Veredicto propuesto: sin hallazgos bloqueantes en este alcance.**
No implica aceptación final del responsable ni auditoría del servicio privado.

### Ledger 0 + 10

| Eje | Estado | Evidencia / límite |
|---|---|---|
| 0. Arquitectura | sustentado | §0 separa OSS cliente/middleware/MCP de internals privados; recall, post-stream capture, update/delete, errores y cancelación realmente trazados |
| 1. Recomendaciones | sustentado | §1 prioriza outcomes, revisión de caché/readiness; adapter SaaS condicionado, no motor obligatorio |
| 2. Fichas Rust | sustentado | A–G y §12 cubren 11 campos; delta de presentación ≠ dedup L1; readiness ≠ durabilidad; pasos y seams Vanta explícitos salvo transporte futuro declarado abierto |
| 3. Incidencias | sustentado | Tres páginas leídas completas, open; versiones LiveKit/v3/self-hosted separadas; PR referenciada no tratada como merge |
| 4. Prerrequisitos | sustentado | Tabla por unidad y §12: recall/L0/worker/erasure reutilizados; no segundo scheduler/assembler |
| 5. Rechazos | sustentado | §5 separa SaaS, servidor binario, SDK, componente MIT y riesgos de egress |
| 6. Método | sustentado | §6 diferencia API efectiva de dependencia declarada y publicación de medición propia |
| 7. Benchmarks | sustentado | §7: docs-test no QA; MemoryBench fijado; clear no-op, denominador condicionado, recallAtK=hitAtK solo en ese runner, no generalizado al producto |
| 8. Matriz | sustentado | §8/§12 preservan capacidades reales, `core_search` no default, WriteReceipt no firma, C13 propuesto |
| 9. Negativos Vanta | sustentado | §9 distingue hechos de L0/budget, hipótesis de caché y falsos positivos descartados |
| 10. UI | sustentado | §10 web redirect/grafo no consola privada; §0.1 declara expresamente Studio dirty y evidencia de worktree, no release |

### Hallazgos y observación no bloqueante

No se encontró discrepancia bloqueante en las fuentes contrastadas.
**Baja — SM-1, precisión de navegación:** `supermemory.md:328` usa las rutas
Studio `desktop/...` junto al SHA `0ad38c542b39a589035b0e17fdedb4c09dca24b9`;
el árbol de ese commit tiene `feature-freez/desktop/...`. A diferencia de H-2
y AM-1, `supermemory.md:38` **sí etiqueta expresamente cambios ajenos y
evidencia de worktree**; no se transforma esa limitación declarada en hallazgo
bloqueante. Consecuencia residual: el lector no puede usar `git show` con
la ruta de disco para reproducirla. Mejora verificable: nombrar ambas rutas y
separar navegación actual de cuerpo del snapshot, sin presentar UI nueva ausente.

### Fuentes contrastadas y límites

- **Leídas en clon:** `shared/cache.ts` completo, `shared/memory-client.ts:30-74`,
  `tools-shared.ts:64-168`, `vercel/middleware.ts:151-188,311-379`,
  `vercel/index.ts:241-289`, `apps/mcp/src/server/auth/index.ts:68-207`,
  `livekit-sdk-python/.../memory.py:249-282`, web `page.tsx:1-60` y
  `memory-graph/.../use-graph-data.ts:43-69`. Se confirma Promise.all sin
  parciales, guardado no esperado y borrado con preview/IDs, no CAS probado.
- **Vanta pin:** `L0Recorder::record_turn:274-372`, `erasure.rs:86-108,
  191-253`, además de recall/writer/attestation ya contrastados.
  `erase_scope` destruye DEK y emite recibo; su existencia no convierte el
  preview hipotético en API actual ni certifica eliminación de backups.
- **Públicas completas por HTML:** [#1860](https://github.com/supermemoryai/supermemory/issues/1860),
  [#1849](https://github.com/supermemoryai/supermemory/issues/1849),
  [#1856](https://github.com/supermemoryai/supermemory/issues/1856).
  Estados/versiones y límites del informe corroborados; no se leyeron #1863,
  #1850, #1312/#1302/#1299 ni se acredita versión publicada de sus fixes.
- **MemoryBench remoto leído por urllib, no ejecutado**, SHA
  `063b86bfb347336def3a084153f40bfea1e2dfd6`:
  `src/providers/supermemory/index.ts:55-102,140-216`,
  `src/orchestrator/phases/retrieval-eval.ts:1-147`,
  `evaluate.ts:16-38`, `report.ts:96-170`.
  Confirmados ocho retries HTTP posibles, polling sin deadline global,
  failedIds sin throw final, `clear` que solo advierte, precisión sobre
  resultados efectivos y recall binario, exclusión de no evaluadas.
- **No releído:** docs-test completo, loader/CLI/answer/jueces completos,
  todos los wrappers, layouts, auth global ni internals del servicio.
  La evidencia estática selectiva basta para los dictámenes acotados;
  no demuestra calidad, seguridad, coste, rendimiento o equivalencia SaaS/local.

## TencentDB Agent Memory

**Recibo:** leído íntegro [tencentdb-agent-memory.md](../tencentdb-agent-memory.md),
líneas 1–688; clon `C:\Users\Eros\TencentDB-Agent-Memory`, HEAD comprobado
`2002196f2d7c22ea267cfcaf67c5a6fa861df681`.
**Veredicto propuesto: cambios requeridos.**

### Ledger 0 + 10

| Eje | Estado | Evidencia / límite |
|---|---|---|
| 0. Arquitectura | sustentado | §§1.2–1.4: append atrapado/cursor, RMW local/externo opcional, timeout no cancelable, generación Wiki y borrado parcial correctamente diferenciados |
| 1. Recomendaciones | sustentado | §1.5 y H: ocho mecanismos; no clonar plataforma/AST ni volver obligatorios Redis/COS |
| 2. Fichas Rust | parcial | A–H/complementos cubren 11 campos, pero C mezcla seam de checkpoint de tarea con cursor L0, T-1 |
| 3. Incidencias | sustentado | Dos páginas open leídas; propuestas de fix no declaradas merge; no se diagnostica Studio desde una carrera React ajena |
| 4. Prerrequisitos | parcial | Condiciones por candidato y contratos existentes reconocidos; revisar separación de recursos en C, T-1 |
| 5. Rechazos | sustentado | §5: no plataforma ni permisos derivados de JSON permisivo |
| 6. Método | sustentado | §6/cierre distingue MIT textual de NOASSERTION congelado, dependencia CodeGraph de implementación propia |
| 7. Benchmarks | sustentado | §7/complemento: QPS de checkpoint no calidad ni COS medido; token estimate no tokenizer universal y ruta de import errónea reconocida |
| 8. Matriz | parcial | Cita TaskCheckpoint como parte de composición no acreditada, pero seam de C necesita precisión; resto no niega híbrido/budget/quotas existentes |
| 9. Negativos Vanta | sustentado | §9/§1.2 descarta falsa novedad de tie_break; no bugs actuales inventados |
| 10. UI | parcial | React/BFF real; atribución Studio al snapshot falla, T-2 |

### Hallazgos

1. **Media — T-1: el seam de C no es el cursor de captura que se propone proteger.**
   `tencentdb-agent-memory.md:451-471`, especialmente 456/462, y 619-620:
   `TaskCheckpointManager::advance` se propone para `advance_capture`.
   **Fuente discrepante Vanta** al SHA
   `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`:
   `vanta-memory/src/utils/task_checkpoint.rs:129-188` avanza `step` y
   `partial` de una tarea; el cursor real lo lee/escribe
   `core/conversation/l0_recorder.rs:421-430,471-495` en `l0_cursor/<sesión>`
   como `after_timestamp_ms`. `record_turn:274-372` usa este último.
   **Mecanismo:** parecido RMW no establece acoplamiento de recursos; añadir
   revisión al checkpoint de tarea no condiciona el write del cursor L0.
   **Consecuencia:** una tarea derivada de C puede “corregir” el recurso
   equivocado sin proteger la frontera que motiva A/C.
   **Corrección verificable:** mapear C a `record_turn/read_cursor/write_cursor`
   y caller concreto; mantener TaskCheckpoint solo para progreso/reconciliación
   si se demuestra ese vínculo. Distinguir además el helper global de
   `utils/checkpoint.rs`. No afirmar fallo concurrente Vanta sin ese análisis.

2. **Media — T-2: rutas Studio no pertenecen al SHA afirmado.**
   `tencentdb-agent-memory.md:644-649`, mismo mecanismo que H-2/AM-1.
   Árbol `0ad38c542b39a589035b0e17fdedb4c09dca24b9`:
   `feature-freez/desktop/src/components/...`, no `desktop/src/components/...`;
   las rutas nuevas están `AM`. **Consecuencia:** evidencia de integración
   visual no reproducible como snapshot. **Corrección verificable:** releer
   cuerpos/rangos al SHA bajo `feature-freez` o etiquetar lectura dirty;
   no negar por ello que Studio ya existe.

### Fuentes contrastadas y límites

- **Leídas en clon:** `l0-recorder.ts:274-313`, `auto-capture.ts:92-145`,
  `checkpoint.ts:364-375,735-768`, `auto-recall.ts:92-134`,
  `embedding.ts:520-599`, `offload/reclaimer.ts:62-130`,
  `state-reporter.ts:65-80`, `memory-cleaner.ts:69-100`,
  `wiki-service.ts:423-462,1350-1425`, `sqlite-store.ts:547-559`,
  BFF `chat-memory.ts:589-640`, UI `App.tsx:1-70`.
  Se confirma que shape “effective” no demuestra entrega, TTL usa políticas
  distintas, dimensions es condicional y sort por index no valida identidad.
- **Benchmarks leídos:** `bench-checkpoint-lock.ts:1-100,107-128,140-212`,
  `benchmark-token-estimate.ts:1-89`. No ejecución: comentarios COS no
  sustituyen storage efectivo, ni supuestos de demanda LLM son throughput
  medido del producto; el import token resuelve a `src/src/offload`.
- **Vanta:** cuerpos L0 y TaskCheckpoint indicados, recall/writer/erasure
  contrastados en recibos anteriores. No se trató un scope como autorización
  ni un hash WriteReceipt como firma; C13 sigue propuesto.
- **Públicas leídas:** [#1937](https://github.com/TencentCloud/TencentDB-Agent-Memory/issues/1937)
  y [#1935](https://github.com/TencentCloud/TencentDB-Agent-Memory/issues/1935),
  ambas open, 2026-11-03. Se leyeron cuerpo chino/inglés y mecanismo; la
  presentación de salida abrevia requisitos finales. No se leyeron PR
  #1942/#1960/#1941 ni #1936; no fix/version publicada acreditados.
- **No releído:** drivers, guard end-to-end, cola/build/cancel completos,
  materialización de todos los archivos, MemoryCleaner completo, UI de
  préstamo, todos los e2e/Mongo y modelos resueltos. Son límites de esta
  revisión, no inexistencia de capacidad ni exigencia de runtime universal.

## Letta

**Recibo:** leído íntegro [letta.md](../letta.md), líneas 1–395; HEAD del clon
`C:\Users\Eros\letta` comprobado `8816d3a8516893bfdc0fa14bfb1235226012e295`.
Su README y AGENTS completos confirman landing y retiro de V1. Se contrastó
fuente complementaria actual `letta-ai/letta-code` al SHA
`9176d8c0b16f408e9240422a98379a423e2176c3`, por raw público, sin nuevo clon.
**Veredicto propuesto: cambios requeridos.**

### Ledger 0 + 10

| Eje | Estado | Evidencia / límite |
|---|---|---|
| 0. Arquitectura | sustentado | §0 distingue landing, MemFS/commit/sync/compilación, recall JSONL/summaries/remoto y deleteAgent; no atribuye servidor privado/retirado al clon |
| 1. Recomendaciones | sustentado | §1 A–G: revisión, política confiable, repair, compactación, outcomes y linaje; sin Git/runtime obligatorio |
| 2. Fichas Rust | sustentado | §2 A–G cubren 11 campos y propuestas marcadas; no se sustituyen por dos DTO genéricos; no se confunde task checkpoint con archivo de experiencias |
| 3. Incidencias | parcial | Cuatro páginas efectivamente leídas, open; seguimiento de #4249 aporta corrección/causa reportada omitida, L-1 |
| 4. Prerrequisitos | sustentado | §4 por ficha, caller/publicación/fencing pendientes; assembler/ingest/scheduler existentes reconocidos |
| 5. Rechazos | sustentado | §5 V1/Git obligatorio/auto-commit ciego descartados; no adoptar prompts/modelos |
| 6. Método | sustentado | §6 fuente complementaria separada, snapshot no certificado desde comandos Git separados; cobertura selectiva explícita |
| 7. Benchmarks | sustentado | §7 CLI latency, no QA; exitCode y costes no medidos, sin reset/warmup; ni adaptador V1 ni escenarios convertidos en score |
| 8. Matriz | sustentado | §8 local FTS ≠ remoto/híbrido; `rrf_score` local no RRF; capacidades actuales Vanta y límites de paridad preservados |
| 9. Negativos Vanta | sustentado | §9 mezcla de revisiones es hipótesis; tie_break y capacidad existente no omitidos; create_scope no fallo universal |
| 10. UI | parcial | §10 TUI/scanner reales, desktop externo no auditado; Studio solo localizado y ruta/snapshot confundidos, L-2 |

### Hallazgos

1. **Media — L-1: omisión de seguimiento técnico que cambia la evaluación de #4249.**
   `letta.md:240,275,393` conserva solo hipótesis sobre helpers/repos existentes
   y deja el estado efectivo como pendiente, aunque afirma lectura del cuerpo.
   **Fuente discrepante:** [#4249](https://github.com/letta-ai/letta-code/issues/4249),
   comentario público de `jibramnobhill`, 2026-09-09, observado open:
   reporta fix #4277 publicado en v0.31.14, causa en el runner Git plain de
   `memory-worktree.ts` para fetch del padre, y regresión/mutation control.
   **Hecho estático complementario:** al pin Letta Code,
   `src/agent/memory-worktree.ts:286-293` obtiene token solo para origen MemFS
   y llama `runMemoryGit(...["fetch","origin","main"],token,...)`.
   No equivale a comprobar toda autenticación o abrir GUI Windows.
   **Mecanismo/consecuencia:** omitir esa respuesta deja al lector con una
   explicación inicial menos precisa y una laguna mayor que la evidencia viva.
   **Corrección verificable:** incorporar causa/fix/versión **reportados**,
   enlazar comentario/PR y comparar runner con el pin; conservar open,
   distinguir afirmación upstream de reproducción y no afirmar fix universal.
   El seguimiento de #5091 también matiza checkout ausente frente a existente;
   su propuesta inicial no debe tomarse como diagnóstico único confirmado.

2. **Media — L-2: evidencia Studio solo localizada y ruta no fijada al snapshot.**
   `letta.md:340` asocia rutas `desktop/...` al snapshot documental
   `0ad38c542b39a589035b0e17fdedb4c09dca24b9`; árbol real:
   `feature-freez/desktop/...`. Además reconoce **localización**, no lectura
   de cuerpos/rangos de esos paneles. **Consecuencia:** no acredita el seam
   visual específico de E mediante ese snapshot, aunque no niega Studio.
   **Corrección verificable:** leer y citar ActivityPanel/MemoryLens/
   RetrievalLens bajo la ruta del commit, o identificar lectura dirty y sus
   límites; mantener GraphScene solo localizado si no se leyó su cuerpo.

### Fuentes contrastadas y límites

- **Clon:** README/AGENTS completos; se mantiene exclusión V1/archive,
  no se usó histórico para inferir API actual. Apache-2.0 declarado no
  acredita licencia de todo servicio/SDK/dependencia.
- **Letta Code raw al SHA fijado:** `system-prompt-compilation.ts:65-100`,
  `memory-git.ts:1230-1268,1860-1877`, `transcript-search.ts:558-585,606-630`,
  `conversation-search.ts:44-78`, `memory-constraints.ts:119-155`,
  `memory-conflict-repair.ts:175-234`, `memory-operation.ts:1-58`,
  `local-store.ts:482-522`, `compaction.ts:43-90`,
  `memory-scanner.ts:25-55`, `memory-worktree.ts:286-293`.
  Lock abarca escritores del harness, **no las ediciones ordinarias del
  primario** (`memory-operation.ts:25-30`); refuerza el límite del propio
  informe sobre snapshot conjunto, no se declara una carrera reproducida.
- **Runner leído:** `latency-benchmark.ts:28-70,120-185,260-340`.
  Spawn/close/SIGTERM y defaults corroborados; no ingestion gold/juez ni
  metric retrieval. API durations sumadas no son wall time por fase ni
  garantía de overhead no solapado. No ejecución ni cambio de harness.
- **Páginas públicas:** #4831 (2026-09-28, open) y #4832 (2026-09-28, open)
  completas; #5091 (2026-10-14, open) cuerpo y seguimiento visible, salida
  truncada al final; #4249 (2026-09-05, open) cuerpo y comentario correctivo,
  truncación final de metadata. No se presume lectura completa de PRs
  #4725/#4277/#5248/#1702; ninguna marca de estado prueba su merge por sí sola.
- **No releído:** SDK/sesiones D1–D3, App Server, SaaS, lexer JSONL completo,
  file-lock completo, planners/callers/repair completos, MemFS remoto,
  clientes desktop/web ni notices transitivos. Las guías siguen siendo
  documentación del autor, no hechos del SDK verificados por esta revisión.

## Cierre de revisión

| Informe | Dictamen propuesto | Corrección antes de aceptar el informe |
|---|---|---|
| Hindsight | cambios requeridos | H-1 seguimiento de incidencias; H-2 snapshot Studio |
| Agentmemory | cambios requeridos | AM-1 snapshot Studio |
| Supermemory | sin hallazgos bloqueantes en este alcance | SM-1 mejora no bloqueante de navegación dirty/snapshot |
| TencentDB Agent Memory | cambios requeridos | T-1 cursor L0 frente a checkpoint de tarea; T-2 snapshot Studio |
| Letta | cambios requeridos | L-1 seguimiento de #4249; L-2 cuerpo/ruta Studio |

No hay puntuación agregada ni aceptación automática de features. Las
capacidades Vanta citadas por los cinco —híbrido BM25/HNSW/RRF, scopes,
governance, provenance, dedup/supersession, sesiones/personas/escenas/skills,
scheduler/checkpoints/ingestors, compaction, quotas/budgets, Studio y
WriteReceipt— no se han reclasificado como ausentes por silencio de la muestra.
`MutationReceipt` C13 permanece propuesto; integridad SHA-256 no es firma.
Se deben revalidar los cuerpos documentales corregidos y, solo cuando una
decisión concreta dependa de una garantía dinámica no acreditada, delimitar
una validación posterior autorizada. No se exige runtime para aprobar toda
observación estática ni se certifica auditoría exhaustiva, seguridad,
crash-safety, borrado físico, paridad visual, rendimiento o calidad medida.

**Recibo de operaciones:** lecturas locales con cat/sed/nl y `git show`/
`rev-parse`, consultas públicas `websearch.fetchWebContent` y raw por urllib;
ediciones únicamente por apply_patch en este archivo. Accesos fallidos y
lecturas truncadas quedan en sus recibos, sin reemplazarlos por resultados
inventados. Esta revisión no modificó informes, contratos, backlog, código, índices,
prompts, rúbricas, scoring, umbrales o configuración; sin procesos de producto,
tests/builds/benchmarks/install/servicios, commits, staging ni push.

**Persistencia local frente a versionado:** el archivo existe en disco, pero
`git check-ignore -v` confirma que `.gitignore:268` ignora `docs/dev/research/`.
No se alteró esa regla ni se forzó staging: la incorporación a Git requiere
una decisión posterior autorizada del responsable. `git status` muestra
modificaciones ajenas en cuatro informes; no se revirtieron ni se atribuyen
a las ediciones de esta revisión.
