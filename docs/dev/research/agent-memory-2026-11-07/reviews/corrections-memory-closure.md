---
title: "Reconfirmación independiente del rework de once informes de memoria y código"
kind: research
status: proposed
description: "Recibos estáticos por proyecto, mapa de diecinueve correcciones y límites de los dictámenes propuestos"
tags: [research, memory, engineering]
---

# Reconfirmación independiente del rework de once informes de memoria y código

**Índice:** [Mem0](#mem0) · [Cognee](#cognee) · [Graphiti](#graphiti) ·
[MemPalace](#mempalace) · [Hindsight](#hindsight) ·
[Agentmemory](#agentmemory) · [TencentDB](#tencentdb-agent-memory) ·
[Letta](#letta) · [Codebase Memory MCP](#codebase-memory-mcp) ·
[Graphify](#graphify) · [Understand Anything](#understand-anything) ·
[Mapa de diecinueve correcciones](#mapa-consolidado-de-las-diecinueve-correcciones) ·
[Límites y cierre](#límites-transversales-integridad-de-inputs-y-cierre).

## Alcance y criterio

Lectura independiente de autoría, 2026-11-07. Único archivo editable: este
recibo. Inputs: [contrato](../EXPANSION-CONTRACT.md),
[referencia propia](../VANTA-REFERENCE.md), [método](../METHOD.md),
[notas de benchmarks](../BENCHMARK-NOTES.md) y revisiones
[memory](memory.md), [memory-code-remaining](memory-code-remaining.md),
[memory-additions](memory-additions.md), [code-memory](code-memory.md).
Rework contrastado con
`C:\Users\Eros\AppData\Local\Temp\vantadb-fix-memory.diff`,
`vantadb-fix-memory.md` y los originales `vantadb-fix-memory-BASE/`.
El recibo del autor es un índice de afirmaciones, no prueba de corrección.

Vanta **V** = `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.
Studio **S** = `0ad38c542b39a589035b0e17fdedb4c09dca24b9`,
siempre `feature-freez/desktop/...`. Se leen cuerpos del commit mediante
`git show`; no se atribuyen a S rutas nuevas `desktop/...`, jobs ni bytes del
worktree dirty.
V ya tiene híbrido, scopes, checkpoints, quotas, budgets y Studio:
no se infieren carencias globales por silencio de una muestra.
`WriteReceipt`/SHA-256 no son firma; `MutationReceipt` sigue propuesto.

«Sustentado» = coherencia documental en la muestra contrastada; «parcial» =
laguna explícita o residual descrita, no certificación fallida universal.
Cerrar un finding documental no aprueba la propuesta ni acredita runtime,
seguridad, crash-safety, paridad entre backends o calidad medida.
Los once dictámenes son **propuestos al lead**, no aceptación final.
No se crean evaluadores, prompts, scoring, umbrales o configuraciones.
Se conserva un recibo por proyecto antes de avanzar al siguiente.

## Mem0

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe actual íntegro L1–799; diff Mem0 leído completo frente a BASE.
Clon: `C:\Users\Eros\mem0`, HEAD comprobado
`f1791daa86d469249c9b002f62644d58a1eb96a0` (**M**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Identidad/arquitectura/flujos | sustentado | L14–244,463–491: vector, entidad, history y Platform separados; no commit distribuido supuesto |
| 1. Recomendaciones | sustentado | L102–113: seis unidades, deltas frente a dedup/recall existentes |
| 2. Fichas | sustentado | L246–491: contratos propuestos, errores, ownership, FMEA y rollback; C no promete deshacer deletes |
| 3. Incidencias | parcial explícito | L510–545: reportes históricos delimitados; PR de cierre/paridad no certificados; no fueron reabiertos en esta pasada |
| 4. Prerrequisitos | sustentado | L157–174,561–568: RecallConfig real, core_search opt-in y autoridad por recurso |
| 5. Rechazos | sustentado | L570–576: no MD5 como seguridad, constructor LLM obligatorio ni ranking importado |
| 6. Método/límites | sustentado | L578–592,772–780: cobertura y licencias transitivas pendientes, no ausencia global |
| 7. Benchmarks | sustentado | L613–685: pares/sesiones, corpus mutable, wrapper fallido, abstención PASS y población emitida diferenciados |
| 8. Matriz | sustentado | L687–698: comparación equivalente no acreditada; incorpora M1/M2, no niega híbrido ni contexto propios |
| 9. Negativos propios | sustentado | L700–717: delete-before-put estático; concurrencia RMW sigue hipótesis, no bug demostrado |
| 10. UI/fuentes/cierre | sustentado | L719–799: flujo web y tres paneles S reales; genlog no sustituye recibo; PASS no acredita search exitoso |

| Finding previo | Resultado independiente | Evidencia de cierre y propagación |
|---|---|---|
| M1 alta | cerrado documentalmente | L648–655,680–685,698,752–754; cliente al gitlink retorna `[]` tras retries tanto OSS como cloud; prompt permite PASS de abstención. No se reduce a «siempre cero» ni a excepción del driver |
| M2 media | cerrado documentalmente | L624–629,669–678,680–685,698: resolve/main, orden cronológico, limpieza `has_answer`, pares y skip por contenido vacío; backend/split/modelos desconocidos se mantienen desconocidos |
| M3 baja | cerrado | L166 y enlace V-recall: `RecallConfig`, no tipo AutoRecallConfig |

**Contraste propio:** lectura pública raw del gitlink
[`4b61c5d…/mem0_client.py L254–344`](https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/benchmarks/common/mem0_client.py#L254-L344)
y [`run.py L90–140,194–219,279–324,407–443,631–663,866–908`](https://github.com/mem0ai/memory-benchmarks/blob/4b61c5d31b9c668a12b4f5e78064248a02c82d2b/benchmarks/longmemeval/run.py#L866-L908).
El denominador es `len(evaluations)` por cutoff, no búsquedas exitosas.
En sesiones de longitud impar queda un último chunk de un mensaje: «pares
consecutivos» no debe interpretarse como validación estricta user+assistant.
La corrección no afirma ese invariante ni cambia el runner.
M `utils/scoring.py L60–138` confirma pool semántico; V
`auto_recall.rs L302–335` confirma tipo/default. En S se leyeron
`MemoryLens.tsx L381–409`, `ActivityPanel.tsx L38–91` y
`RetrievalLens.tsx L138–228`; coinciden con los nuevos enlaces y semántica.

**Regresiones/límites:** no discrepancia material nueva en los cambios.
No se recalcularon scores ni se auditaron todos los providers, transports,
PRs históricos, licencias/datasets o ejecución de UI. Un primer intento de
impresión Python falló por encoding cp1252; se repitió con UTF-8 y se leyó
el contenido. Ningún fallo de red en estas lecturas raw.

## Cognee

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe íntegro L1–942 y diff completo Cognee/BASE leídos.
Clon `C:\Users\Eros\cognee`, HEAD comprobado
`8853378eae5366addb064e758e560a456146ae1a` (**C**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Arquitectura/tres flujos | sustentado | L14–231: add/cognify, búsqueda autorizada, update/delete; SQL, originales, grafo y vector separados |
| 1. Recomendaciones | sustentado | L103–111: A–H condicionadas, no otra cola, RRF o memoria de sesión |
| 2. Fichas | sustentado | L233–573: ocho unidades con contratos nuevos identificados, commit/cancelación, FMEA y migración |
| 3. Incidencias | sustentado acotado | L594–615: tres reportes y vigencia limitada; esta pasada no relee sus comentarios ni certifica PR causal |
| 4. Prerrequisitos | sustentado | L617–625: capacidades propias reconocidas, publicación y permisos por recurso |
| 5. Rechazos | sustentado | L627–635: no globals como IAM, auto-install ni graph-first como atomicidad |
| 6. Método | sustentado | L637–660 y cierre: cobertura parcial/propiedad ≠ ledger, licencias transitivas pendientes |
| 7. Benchmarks | sustentado | L662–763: LoCoMo defaults/unscored/reset; BEAM local/config/población numérica no equivalen a Modal |
| 8. Matriz | sustentado | L765–779: fila BEAM añade no equivalencia; híbrido, budgets, scheduler y Studio ya existen |
| 9. Negativos propios | sustentado | L781–803: writer estático distinto de crash reproducido; campos no prueban CAS |
| 10. UI/fuentes/cierre | sustentado | L805–942: web real y what-if visual, paneles S correctos, denominador visible sin score nuevo |

| Finding previo | Resultado independiente | Cierre end-to-end |
|---|---|---|
| CO-1 media | cerrado documentalmente | L703–724,777,869–870,889–891: corpus preingerido, split 100K/conv0, repetición1/concurrencia4, config obligatoria, questions_path manda y 10M exige export; medias solo con scores numéricos, no question_count |
| CO-2 media | cerrado | L852–876 y cierre: tres rutas feature-freez y rangos S leídos; error/no-config no se confunden con eventos vacíos; readiness/recibo siguen propuestas |

**Contraste propio:** C
[`run_sweep.py L148–177,309–404`](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/beam/eval/run_sweep.py#L309-L404)
y [`aggregate_cross_run.py L24–114`](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/beam/eval/aggregate_cross_run.py#L24-L114),
lectura local sin invocar runner: `get_score` filtra int/float,
`stats` usa únicamente listas numéricas y `question_count` cuenta entradas
del primer run. No se infiere que todas fueron respondidas/juzgadas; tampoco
se inventa un cero para ausentes. `validate_runs` exige mismos IDs/orden.
Paneles S ya contrastados en el recibo anterior, con mismos cuerpos/rangos.

**Regresiones/límites:** no discrepancia material introducida en la muestra.
Fichas A–H conservan separación fuente/chunk/derivado/run y sus dependencias;
CO-1 es precisión del runner, no nueva fórmula, harness o rechazo de todos
los resultados. No lectura nueva del orquestador Modal no publicado, todos
los retrievers/ACL/conectores/PRs; no ejecución ni conformance multitenant.

## Graphiti

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe íntegro L1–868 y diff Graphiti/BASE completos leídos.
Clon `C:\Users\Eros\graphiti`, HEAD comprobado
`747b480ec6b12ecb718f1e2bc588b20ee36ea9c2` (**G**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | §§0–1: episodio/entidad/hecho/saga; add/query/retirada y commits separados |
| 1. Recomendaciones | sustentado | §2: tiempo efectivo/ocurrencia/soportes, no timestamps desde cero |
| 2. Fichas | sustentado | §3.1–3.6: cinco contratos propuestos; funciones puras/vista no bloqueadas universalmente por escritura L1 |
| 3. Incidencias | sustentado acotado | §4: reportes y decorador/MCP actual diferenciados; no relectura de PR causal en esta pasada |
| 4. Prerrequisitos | sustentado | §5.1: valid_at y scopes existentes; error después de commit no implica rollback |
| 5. Rechazos | sustentado | §5.2: no driver externo, saga-checkpoint ni comunidades sin caso |
| 6. Método | sustentado | §§0/4/9: intervalos de aristas no historial global; group_id no auth |
| 7. Benchmarks | sustentado | L603–674: persistencia/repetición, oracle, zip/divisor y contexto string; AddEpisodeResults no portable directamente |
| 8. Matriz | sustentado | L676–700: eval construcción no aplicable directamente a registros Vanta; capacidad propia reconocida |
| 9. Negativos propios | sustentado | L702–718: writer estático, stale/RMW hipótesis separadas; no falso negativo de híbrido |
| 10. UI/fuentes/cierre | sustentado | L720–868: App/WorkspaceShell y paneles S, sin codec o JobChip atribuidos al pin |

| Finding previo | Resultado independiente | Cierre end-to-end |
|---|---|---|
| GI-1 media | cerrado | L732–768,798,864: se retira routes.ts/codec inexistente del pin; seams de §3.5 y aceptación conservados como propuesta, no implementación |
| GI-2 media | cerrado documentalmente | L616–626,654–667,697: grupos fijos reutilizados, episodios sin UUID suministrado, sin reset/restauración; candidate repetible con baseline anterior; FMEA y adaptación no equiparan pistas |

**Contraste propio:** se leyeron completos
[`eval_e2e_graph_building.py L1–180`](https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/tests/evals/eval_e2e_graph_building.py#L1-L180)
y [`eval_cli.py L1–40`](https://github.com/getzep/graphiti/blob/747b480ec6b12ecb718f1e2bc588b20ee36ea9c2/tests/evals/eval_cli.py#L1-L40).
El juez recibe objetos `AddEpisodeResults` reconstruidos del baseline y
producidos por candidate, no resultados retrieval Vanta. `zip(strict=False)`
recorta comparaciones, no el divisor candidate; sin listas/usuarios puede
haber división por cero. El valor por defecto de `candidate_is_worse` ausente
es false: no interpretar el resultado como calidad absoluta.
El error de contexto por indexar caracteres sigue explícito; no se tocó juez.
S `App.tsx L1–64` completo y `MemoryLens.tsx L488–550` leídos, además de
los tres paneles comunes. `ls-tree` no contiene las rutas app consultadas.

**Regresiones/límites:** no discrepancia material nueva. La adaptación al eval
Vanta existente se limita a su pista retrieval, no se vende como adapter del
juez Graphiti. Falta conformance drivers/ACL/cancelación, corpus oracle
íntegro y extracción natural; sin ejecución, scores, limpieza o artefactos.

## MemPalace

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe íntegro L1–776 y diff/BASE completos leídos. Clon
`C:\Users\Eros\mempalace`, HEAD comprobado
`f8e35362189bd0a5abf4c0df1dd5bf90ba059176` (**P**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | §§0–2: raw/closet/KG/snapshot nativo, write/query/delete distinguidos |
| 1. Recomendaciones | sustentado | §1: preservar raw y capacidades sin sustituir L1/escenas |
| 2. Fichas | sustentado | A–F, L160–494: C/F siguen caller/puertos reales; generación/preview explícitamente nuevos |
| 3. Incidencias | sustentado acotado | §3: parentless intencional no se interpreta como pérdida; PR abierta no implementación |
| 4. Prerrequisitos | sustentado | §4 y C/F: fuente, wiki y relación no comparten por nombre un commit |
| 5. Rechazos | sustentado | §5: no AAAK lossless, metáfora obligatoria ni exact scan sustituto de híbrido |
| 6. Método | sustentado | §6/§11: develop no release; MIT raíz no conformance/licencia transitiva |
| 7. Benchmarks | sustentado acotado | §7: any/all, user-only, relleno, SKIP y denominadores; no nueva lectura exhaustiva de variantes |
| 8. Matriz | sustentado | L656–669: wiki no prueba convergencia C ni preview F; put/add_edge son por unidad |
| 9. Negativos | sustentado | §9: preguntas de composición, no bugs inventados de Vanta |
| 10. UI/fuentes/cierre | sustentado | §10–11: landing no inspector; Studio S y recibos wiki/fuente/relación separados |

**MP-1 media: cerrado documentalmente.** Propagación L247–291 (C),
L438–444,464–492 (F),663–666 (matriz),708–720 (UI/fuentes).
Se conserva la propuesta pero se abandona la atribución de publicación de
fuentes/grafo a `CandidatePage`/`CommitReport`. La publicación por revisión
multirrecurso queda expresamente nueva/no acreditada, sin scheduler/WAL nuevo.

**Contraste propio:** V
[`merge.rs L16–49,228–288`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L228-L288),
[`worker.rs L109–151,244–301`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/worker.rs#L244-L301):
closures get_page/put_page, commit serial y complete/fail posterior.
[`memory.rs L1223–1240`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1223-L1240)
publica registro y audita resultado, no generación de todo el corpus.
[`graph.rs L113–151`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/graph.rs#L113-L151)
inserta source antes de cargar/escribir target: error del segundo extremo
no demuestra que no hubo efecto. P
[`knowledge_graph.py L94–193`](https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/knowledge_graph.py#L94-L193)
sí trata entidades/triples, preview en copia y commit con apply.

**Regresiones/límites:** no discrepancia material nueva. En D sigue citándose
V5 como recibo ingest, ahora sin presumir mapping lógico-chunk; matriz/UI
explicitan su límite. Contratos C/F, FMEA, ownership y aceptación no confunden
write wiki con hecho aplicado. No se ejecutan los casos de crash/fallback ni
se auditan todos los backends, variantes benchmark, auth, PyO3 o purga física.

## Hindsight

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe íntegro L1–653 y diff Hindsight/BASE completos leídos. Clon
`C:\Users\Eros\hindsight`, HEAD comprobado
`eb963cb5a929d92c081371df64c2d5c2fe3351af` (**H**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | §§1–2: retain/recall/reflect/update-delete, facts/observations/mental models y operaciones separados |
| 1. Recomendaciones | sustentado | §1: siete unidades condicionadas; no port de producto ni nomenclatura cognitiva como garantía |
| 2. Fichas | sustentado | A–G/complementos: once campos, fuente/revisión, metadatos actuales, cancelación cooperativa y recibos propuestos |
| 3. Incidencias | sustentado | L527–548: #4831/#2122 seguimiento técnico presente y contrastado; #6440/#6439 conservan límites de reporte/fork |
| 4. Prerrequisitos | sustentado | §4: writer/scheduler/derivados existentes; primitiva condicional pendiente no se finge implementada |
| 5. Rechazos | sustentado | §5: no servidor SQL, CLI-motor ni retry que oculte parcial |
| 6. Método | sustentado | §6/cierre: pin propio, lectura selectiva, MIT no licencia del despliegue completo |
| 7. Benchmarks | sustentado acotado | §7/complemento AMB: wrapper no es runner; reset/errores-vacío/agregado y fuentes remotas separados |
| 8. Matriz | sustentado | L588–604: nuevas filas tags/cancelación no heredan bug histórico ni paridad Vanta |
| 9. Negativos | sustentado | §9: composición bajo concurrencia es pregunta; WriteReceipt/hash no firma, C13 propuesto |
| 10. UI/fuentes/cierre | sustentado | L606–653: web real y paneles S; estado requested no equivale a proveedor detenido |

| Finding previo | Resultado independiente | Cierre end-to-end |
|---|---|---|
| H-1 media | cerrado documentalmente | L383–389,522–526,536–548,595–596,619–623: dos guards distintos en A, seguimiento #4893/#4897, cancelación #2127/07c85da y release solo reportados |
| H-2 media | cerrado | L610–627: rutas/rangos S correctos y cuerpos comunes contrastados; no JobChip/jobs ni bytes del worktree dirty atribuidos al pin |

**Contraste propio:** páginas públicas completas
[#4831](https://github.com/vectorize-io/hindsight/issues/4831) y
[#2122](https://github.com/vectorize-io/hindsight/issues/2122), vía
`websearch.fetchWebContent`, sin truncación/fallo: comentario mantenedor
2026-10-05 distingue #4893 y #4897; #2122 muestra #2127, `07c85da` y
Release v0.8.2. Son declaraciones/timeline, no artefacto probado.
H [`consolidator.py L3445–3495`](https://github.com/vectorize-io/hindsight/blob/eb963cb5a929d92c081371df64c2d5c2fe3351af/hindsight-api-slim/hindsight_api/engine/consolidation/consolidator.py#L3445-L3495)
revalida fuentes y obtiene `current_tags` por `lock_observation_tags`, usa
esos tags en historial y unión con tags de fuentes para rewrite.
La mera revisión de fuentes no acredita conservar ediciones de metadata.
No se extiende esa garantía SQL a stores externos por tener la misma interfaz.

**Regresiones/límites:** no discrepancia material nueva. Las propuestas A/G
ahora no parten de ausencia de guard/cancelación upstream. La unión con tags
de fuentes no prueba que toda retirada arbitraria de un tag sea permanente
si una fuente viva lo reintroduce; el informe no promete esa garantía global.
No se releen todos los loaders/jueces AMB, PRs completos, Oracle/auth,
artefactos release, cancelación de proveedor o borrado físico; nada ejecutado.

## Agentmemory

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe íntegro L1–550 y diff/BASE completos leídos.
Clon `C:\Users\Eros\agentmemory`, HEAD comprobado
`4f758eff36c8403570e1bf2ac6d986239fcdb662` (**A**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | §1/complemento: capture/inbox/observe/completed, search y evolve/forget; persistencias distintas |
| 1. Recomendaciones | sustentado | §1: recibos/ownership acotado, no iii-engine ni coordinación distribuida sin requisito |
| 2. Fichas | sustentado | A–D/complementos: contrato propuesto, errores, rollback, cancelación y privacy antes del truncamiento |
| 3. Incidencias | sustentado acotado | Tabla de cinco fuentes: abierta/cerrada-sin-merge/merge posterior separados; no reconsulta pública en esta pasada |
| 4. Prerrequisitos | sustentado | §4/complementos: ack durable, retención/tombstone, publicación y fencing condicionados |
| 5. Rechazos | sustentado | §5: no KV/runtime paralelo ni fingerprint como ID |
| 6. Método | sustentado | §6/cierre: Apache propio no licencia de engine/pesos, muestra no exhaustiva |
| 7. Benchmarks | sustentado acotado | §7/complementos: histórico any-hit distinto del scorer por sesiones; init/teardown/sandbox/pagos y denominadores |
| 8. Matriz | sustentado | L499–509: A/C y B en Studio son nueva proyección, no jobs existentes demostrados |
| 9. Negativos | sustentado | §9: preguntas de composición; no WriteReceipt firmado ni MutationReceipt implementado |
| 10. UI/fuentes/cierre | sustentado | L511–550: bearer/viewer real; S genlog/anchors/audit/explain no prueba lease ni entrega |

**AM-1 media: cerrado.** L517–539 corrige identidad/rangos S y propaga a
matriz L509, FMEA/aceptación L526–532 y fuentes inmutables L536–539.
No se limita a cambiar prefijo: elimina atribución de jobs, separa genlog,
recibo de entrega y lease vigente. La proyección A/C permanece propuesta.

**Contraste propio:** cuerpos S ya leídos en esta reconfirmación:
[`ActivityPanel L52–89`](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L52-L89),
[`MemoryLens L381–409,488–550`](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L550),
[`RetrievalLens L138–228`](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228).
El cierre de AM-1 depende de esos cuerpos propios, no del recibo del autor.

**Regresiones/límites:** no discrepancia material introducida. No se considera
«task checkpoint existe» prueba de lease distribuido; B lo condiciona y
requiere validación en publicación. La pasada no audita iii/KV, totalidad de
privacy/auth ni repite los runners o fuentes remotas no modificadas.

## TencentDB Agent Memory

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe íntegro L1–728 y diff/BASE completos leídos.
Clon `C:\Users\Eros\TencentDB-Agent-Memory`, HEAD comprobado
`2002196f2d7c22ea267cfcaf67c5a6fa861df681` (**T**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | §§1.1–1.4: append atrapado/cursor, RMW, timeout no cancelable, Wiki build/delete y cola RAM separados |
| 1. Recomendaciones | sustentado | §1.5: ocho unidades, no plataforma/AST/Redis obligatorios |
| 2. Fichas | sustentado | A–H: C protege cursor L0 con callers reales; firma/revisión condicional explícitamente propuesta |
| 3. Incidencias | sustentado acotado | §3/§11: dos reportes y fixes no acreditados; no reconsulta de PRs no modificadas |
| 4. Prerrequisitos | sustentado | L292–300: cursor captura distinto de checkpoint task/global; no queue nueva |
| 5. Rechazos | sustentado | §5: no metadata JSON permisiva como autoridad ni copia del hub |
| 6. Método | sustentado | §6/cierre: NOASSERTION congelado y MIT textual no se sustituyen |
| 7. Benchmarks | sustentado acotado | §7/L630–645: checkpoint QPS no calidad/cursor L0, tokens no tokenizer universal; sin ejecutar/calcular |
| 8. Matriz | sustentado | L652–662: fila cursor y task/global separadas, no CAS inferido; quota ≠ TTL/budget |
| 9. Negativos | sustentado | §9/§1.2: tie_break ya existe, carrera propia solo riesgo condicionado |
| 10. UI/fuentes/cierre | sustentado | L664–728: web/BFF real y S correcto; mensajes/cursor/progreso L1 visibles como estados distintos propuestos |

| Finding previo | Resultado independiente | Cierre end-to-end |
|---|---|---|
| T-1 media | cerrado documentalmente | L216–227,299–300,455–491,641–643,654–655,678–681: `record_turn/read_cursor/write_cursor`, callers y FMEA por sesión; no se «corrige» step/partial para proteger L0 |
| T-2 media | cerrado | L669–686: tres rutas/rangos S, mismas fuentes contrastadas; no JobChip/jobs ni bytes dirty afirmados |

**Contraste propio V:** leídos
[`l0_recorder.rs L274–372,421–430,471–495`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/conversation/l0_recorder.rs#L274-L372):
mensajes se publican antes de cursor; payload `after_timestamp_ms`, namespace
`l0_cursor/<sanitized-session>`, tie_break key+contenido presente. No hay
`CursorRevision` en ese payload. Callers
[`auto_capture.rs L111–139`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_capture.rs#L111-L139)
y [`conversation_hook.rs L45–78`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/conversation_hook.rs#L45-L78)
confirman captura previa a enqueue L1. No acreditan exclusión completa por
sesión ni fallo concurrente reproducido.
[`task_checkpoint.rs L129–188`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L129-L188)
modifica step/partial en otro namespace. Benchmark `markL1ExtractionComplete`
ya no se mapea al cursor L0 por analogía.

**Regresiones/límites:** no discrepancia material nueva. Se distingue riesgo
RMW de prueba de pérdida y helper global de ambos recursos. No inspección
íntegra de guards/ACL/colas/builders/modelos/erasure/UI upstream ni runtime.
Los alias T17/T19 no acreditan guard universal; el cierre lo limita.

## Letta

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe íntegro L1–427 y diff Letta/BASE completos leídos.
Clon `C:\Users\Eros\letta`, HEAD comprobado
`8816d3a8516893bfdc0fa14bfb1235226012e295` (**L**).
Complemento remoto separado **LC** =
`9176d8c0b16f408e9240422a98379a423e2176c3` de `letta-ai/letta-code`.

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Identidad/arquitectura/flujos | sustentado | §0: landing no servidor; complemento LC separado de V1; MemFS, recall JSONL/summary/remoto y deleteAgent acotados |
| 1. Recomendaciones | sustentado | §1: siete deltas; sin Git obligatorio, segundo runtime ni adaptación V1 |
| 2. Fichas | sustentado | §2 A–G: propuestos, fuentes/revisiones/errores/rollback; E separa fetch fallido de checkout ausente |
| 3. Incidencias | sustentado | L236–252: #4249 seguimiento/fix reportado y código LC contrastados; #5091 matizado |
| 4. Prerrequisitos | sustentado | §4: caller/publicación/fencing por ficha; wiki commit no atómico global |
| 5. Rechazos | sustentado | §5: V1/auto-commit ciego/Git primario/prompts rechazados |
| 6. Método | sustentado | §6: secuencia HEAD/show no snapshot conjunto certificado, documentación SDK no código auditado |
| 7. Benchmarks | sustentado acotado | §7: latencia CLI, no QA; resta API no overhead exclusivo, sin score de memoria |
| 8. Matriz | sustentado | L317–334: fila refresh padre distingue LC/fix reportado y no aplicación Git al motor |
| 9. Negativos propios | sustentado | §9: composición es hipótesis, tie_break existente, create_scope no fallo universal |
| 10. UI/fuentes/cierre | sustentado | L345–427: TUI real, cliente externo no auditado; S leído por cuerpos y publicación propuesta |

| Finding previo | Resultado independiente | Cierre end-to-end |
|---|---|---|
| L-1 media | cerrado documentalmente | L201–205,242–252,333,420–424: comentario 2026-09-09, #4277/v0.31.14 reportados, fetch usa runner autenticado del pin; mantiene issue open y límites GUI/auth/version |
| L-2 media | cerrado | L351–367: cuerpos/rangos S acreditan seam de E, no nueva proyección; GraphScene sigue solo localizado, no JobChip/jobs ni runtime |

**Contraste propio:** páginas completas, no truncadas, vía
`websearch.fetchWebContent`:
[#4249](https://github.com/letta-ai/letta-code/issues/4249) y
[#5091](https://github.com/letta-ai/letta-code/issues/5091).
#4249 contiene causa/fix/regresión/mutation control declarados y limitación
explícita de no abrir GCM real. #5091 distingue mount ausente/existente;
comentarios posteriores también describen recuperación manual de checkout
ausente. Eso no vuelve suficiente el fetch-in-place del cuerpo inicial
cuando no hay checkout, ni acredita resolución global.
LC [`memory-worktree.ts L1–58,276–301`](https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/memory-worktree.ts#L276-L301)
leído raw: import `runGit as runMemoryGit`, runner plain local distinto;
token solo con prefijo MemFS y fetch por `runMemoryGit`. No basta existencia
del helper: se comprobó su llamada. Paneles S comunes ya contrastados aquí.

**Regresiones/límites:** no discrepancia material nueva en el rework.
No se adoptó argumento del comentarista como ejecución propia ni se dio por
auditado #4277 completo, release v0.31.14, SaaS, App Server/SDK transitivo,
cliente desktop/web o todos los callers de locks/auth/repair. Sin ejecución.

## Codebase Memory MCP

**Dictamen propuesto: sin hallazgos bloqueantes, con una observación residual
de coherencia UI.** Informe íntegro L1–564 y diff/BASE completos leídos.
Clon `C:\Users\Eros\codebase-memory-mcp`, HEAD comprobado
`612eee2518ef84cc573e491fdee5fa5c90264cb5` (**CM**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Arquitectura/flujos | sustentado | §11.2: build/closure/stage, query y delete reales; error ≠ vacío |
| 1. Recomendaciones | sustentado | §1/§12: adapter opcional C12, no otro graph/daemon/parser |
| 2. Fichas | sustentado documentalmente | A–E: E señala aplicación efectiva ACK y nueva integración no acreditada en Vanta |
| 3. Incidencias | sustentado acotado | §11.3: #867/#516, versiones/cierres/fixes declarados; sin nueva auditoría íntegra de PRs |
| 4. Prerrequisitos | sustentado | §4/§12.3: snapshot/coverage/ownership; cola local no durable por TaskCheckpoint |
| 5. Rechazos | sustentado | §5: no parsers/LSP/CI gate/Lens como motor ni MIT transitivo supuesto |
| 6. Método | sustentado | §6/§11: AST no dispatch, hash no cobertura, solo lectura |
| 7. Benchmarks | sustentado acotado | §7/§11.4: scripts microbench, warm daemon/bootstrap/error `?`/0 distintos de calidad |
| 8. Matriz | sustentado | L231–233: query/delete/ACK no equivalencia adapter ni job code-index existente |
| 9. Negativos propios | sustentado | §9/cierre: hash skip/enlaces interarchivo hipótesis; add_edge no metadata/atomicidad global |
| 10. UI/fuentes/cierre | parcial residual | §12.4 corrige imports/comentarios y S; §10 L250–256 conserva atribución demasiado directa |

| Finding previo | Resultado independiente | Cierre end-to-end |
|---|---|---|
| CM-1 media | cerrado documentalmente | L111–113,355–371,459–461,534–539: B/D/matriz/FMEA/UI preservan modo efectivo, proyecto no indexado, delete_failed/not_found/ADR/unwatch |
| CM-2 media | cerrado documentalmente; capacidad E sigue no acreditada | L342–353,484–517,534–539: `commit_baselines`, index_fn/rc y scheduler/TaskKind reales; integración nueva explícita, no caller/job inventado |
| Observación UI de ledger previo (sin ID antiguo propio) | residual baja, no regresión de código | L250–256 frente a L521–524: §10 habla de conexión/navegación acreditada mientras §12.4 restringe evidencia a comentario/imports; límite correcto prevalece en este dictamen |

**Contraste propio CM:** leídos
[`mcp.c L5488–5580,8054–8150`](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/mcp.c#L5488-L5580):
query/semantic_query excluyentes, BM25 con fallback; delete guarda ADR si
unlink principal falla pero retira watcher después; auxiliares best-effort.
[`watcher.c L64–108,1623–1750`](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/watcher/watcher.c#L1623-L1750):
el nombre es **`commit_baselines`**, no `commit_pending_changes` del review
previo; copia pending observado, `index_fn` L1733 y ACK solo con rc==0.
Esto acredita código de aplicación, no mero comentario ni commit Git pendiente.
No demuestra por sí solo persistencia del ACK/recuperación crash end-to-end.

**Contraste propio V:**
[`scheduler.rs L25–119`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/scheduler.rs#L25-L119),
[`types.rs L38–69`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/state/types.rs#L38-L69),
[`pipeline_worker.rs L900–916`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L900-L916):
LocalStateBackend/TimerScanner/PipelineWorker y L1/L2/L3/Dream/Flush,
no job code-index. Primer intento sobre `memory_task_handler.rs` falló:
esa ruta no existe al pin; se corrigió leyendo el cuerpo en pipeline_worker.
CM [`App.tsx L1–132`](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/graph-ui/src/App.tsx#L1-L132)
son comentario/imports: no se acepta ese rango como caller React auditado.

**Límites:** residual UI no bloquea cierre ni se encarga otra corrección.
ACK/job/persistencia/recovery E son una laguna explícita de integración,
no capacidad implementada ni defecto probado del scheduler. No se certifican
transports/auth, publicación crash-safe, planners completos, parsers/LSP o CI.

## Graphify

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe íntegro L1–509 y diff/BASE completos leídos.
Clon `C:\Users\Eros\graphify`, HEAD comprobado
`16218bfc80d4ca2799734d045039f7ed1a5fa9fb` (**GF**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Identidad/arquitectura/flujos | sustentado | §1/§11.1–11.2: AST/semantic, query/refusal, reconcile por origen y publicaciones/ACK separados |
| 1. Recomendaciones | sustentado | §1/§12: adapter C12, evidencia rival, no NetworkX ni nuevo pipeline |
| 2. Fichas | sustentado | A–G: C conserva resolution_state, G diferencia negativa/no-match/error/útil; contratos futuros |
| 3. Incidencias | sustentado acotado | §11.3: estado/fecha de #5051/#4273 incorporados desde revisión previa, guard inode del pin discriminado |
| 4. Prerrequisitos | sustentado | §4: owner/repo/schema/coverage y Derived con fuentes; add_edge no admite evidencia arbitraria |
| 5. Rechazos | sustentado | §5: no URL/LLM/watcher obligatorio ni estimador como tokens reales |
| 6. Método | sustentado | §6/§11.1: Apache/porciones MIT no elección libre global; no offline universal por AST |
| 7. Benchmarks | sustentado | §7/§11.4: ratio de longitud no-sentinel, negativas ambiguas incluidas; no gold/juez/QA/coste |
| 8. Matriz | sustentado | L235: ratio no aplicable como utilidad Vanta; AST/scanners y evidence rich distinguidos |
| 9. Negativos propios | sustentado | §9/§12.5: hash antes de símbolos → hipótesis crash/retry, no bug reproducido |
| 10. UI/fuentes/cierre | sustentado | §11.4/§12.4 G: HTML inspector real; S existente, nuevo inspector de refusals no acreditado |

**GF-1 media: cerrado documentalmente.** L126–133,209–212,235,
365–381,479–487,505–509: corrección de población/estado propagada a C,
runner, FMEA/aceptación, matriz, UI y cierre, sin cambiar evaluación.

**Contraste propio GF:**
[`benchmark.py L1–121`](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/benchmark.py#L36-L103)
completo: `_query_subgraph_tokens` excluye exclusivamente texto igual a
`No matching nodes found.`; todo otro texto devuelve estimación al menos 1.
`questions or _SAMPLE_QUESTIONS` y `qt>0` determinan per_question; media entera
divide por `len(per_question)`. El objeto error significa cero filas
no-sentinel, no falta universal de conocimiento.
[`serve.py L1510–1565`](https://github.com/Graphify-Labs/graphify/blob/16218bfc80d4ca2799734d045039f7ed1a5fa9fb/graphify/serve.py#L1532-L1565)
retorna refusal de nombre exacto ambiguo antes de puntuar/BFS.
La cadena negativa corta→tokens bajos→ratio alto es posible por código;
no se ejecutó ni calculó frecuencia/score. No basta que la respuesta sea
«real del agente» para tratarla como subgrafo útil.

**Regresiones/límites:** no discrepancia material nueva. Las incidencias
incorporan procedencia del review, no supuesto merge ni reproducción propia.
No se releen aquí todos los resolvers, loaders, lock/fsync fallbacks,
providers, auth/XSS, tablas QA o licencias transitivas; el runner descrito no
certifica claims LoCoMo/LongMemEval/SWE-bench. S es seam, no aceptación visual.

## Understand Anything

**Dictamen propuesto: sin hallazgos bloqueantes en este alcance.**
Informe íntegro L1–544 y diff/BASE completos leídos.
Clon `C:\Users\Eros\Understand-Anything`, HEAD comprobado
`a6ea4024767df85ce4f99140d4f12ecfc5a81eb6` (**UA**).

| Eje | Estado actualizado | Evidencia y alcance |
|---|---|---|
| 0. Identidad/arquitectura/flujos | sustentado | §1/§11: estructura+explicación, JSON/sidecars separados, consulta/expansión y update/delete trazados |
| 1. Recomendaciones | sustentado | §1/§12: capas de evidencia Unknown si no consta origen; ni call graph runtime ni nuevo pipeline |
| 2. Fichas | sustentado | A–F: C12 único propuesto; D nombra campos reales y error de linaje, no constructor inexistente |
| 3. Incidencias | sustentado acotado | §11.3: #167 cierre no fix, #758 propuesta no merge, #739 fuente/output diferenciados; sin nueva consulta pública aquí |
| 4. Prerrequisitos | sustentado | L161–169: namespace/parents/Derived y root físico, no confidence inventada |
| 5. Rechazos | sustentado | §5: no Fuse/UI/Vite/agent pipeline como motor ni weight fijo como probabilidad |
| 6. Método | sustentado | §6/§11: caller no auditado no se declara inseguro por helper; MIT no SBOM transitiva |
| 7. Benchmarks | sustentado acotado | §7/§11.4: no gold/juez/QA; degraded exit 0, max RSS por worker, tiempo antes de cleanup/delivery |
| 8. Matriz | sustentado | L513–520: Derived por campos, publicación por productor/root aún no acreditada |
| 9. Negativos propios | sustentado | §9/L522–528: errores ignorados vs hipótesis hash/retry separados, no bug universal |
| 10. UI/fuentes/cierre | sustentado | §12.5: App/token/validate/store, allowlist no aislamiento físico; S genlog/anchor no guía D existente |

**UA-1 baja/no bloqueante: cerrado.** L161–164,432–436,445–450,518,
537–542: campo/clase real, fuente V, FMEA y aceptación; la notación
`MemoryInput::Derived` solo se conserva para negar su existencia, no como API.

**Contraste propio V:**
[`record.rs L184–225`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/types/record.rs#L184-L225)
es struct con `confidence_class: Option<ConfidenceClass>`,
`confidence: Option<f32>`, `derived_from: Option<Vec<String>>`;
Derived requiere padres no vacíos del mismo namespace, sin confidence
explícita. Corrección de nombre no exige compilar ni acredita ejecución.

**Muestra adversa adicional UA:** leídos
[`context-builder.ts L25–110`](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/src/context-builder.ts#L25-L110)
(cap solo de seeds, expansión un salto bidireccional),
[`staleness.ts L472–508`](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/staleness.ts#L472-L508)
(quita aristas incidentes y conserva layers/tour por spread),
[`graph-builder.ts L170–207`](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/analyzer/graph-builder.ts#L170-L207)
(pesos imports/calls fijos) y
[`vite.config.ts L125–184`](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/dashboard/vite.config.ts#L125-L184)
(allowlist léxica/stat/read, no prueba física).
Runner [`benchmark-large-repo.mjs L1–64`](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/scripts/benchmark-large-repo.mjs#L1-L64)
y [`large-repo-benchmark.mjs L1671–1741,1984–2058`](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/scripts/lib/large-repo-benchmark.mjs#L1984-L2058):
degraded puede exit 0, reloj acaba antes de cleanup y reports; delivery
puede lanzar. `llm.invoked:false` de ese runner no certifica todo el producto.

**Regresiones/límites:** no discrepancia material nueva. No se atribuye #167
como fix sin diff causal. Se mantienen Unknown/origen, IDs/normalización,
root fuente vs artefactos, vecinos no exhaustivos y publicación parcial.
No todos los helpers/callers/agents, seguridad física, viewers, modelos,
schemas o recovery/delivery se auditan por esta muestra; nada ejecutado.

## Mapa consolidado de las diecinueve correcciones

Estados de **hallazgo documental**, no del defecto upstream, de una feature
Vanta ni de un caso runtime. Los rangos siguientes son de los informes
actuales, no del review antiguo. Cada recibo superior añade evidencia propia,
límites, propagación ficha/matriz/FMEA/UI y dictamen propuesto.

| Finding / revisión previa | Estado propuesto | Informe y puntos de cierre contrastados |
|---|---|---|
| M1 — [memory](memory.md#mem0) | cerrado documentalmente | [Mem0](../mem0.md):648–655,680–685,698,752–754; error→vacío→posible PASS no se oculta |
| M2 — [memory](memory.md#mem0) | cerrado documentalmente | [Mem0](../mem0.md):624–629,669–685,698; preparación/población/backend y corpus mutable |
| M3 — [memory](memory.md#mem0) | cerrado | [Mem0](../mem0.md):166; RecallConfig real, core_search opt-in |
| CO-1 — [remaining](memory-code-remaining.md#cognee) | cerrado documentalmente | [Cognee](../cognee.md):685–764; BEAM local/preingerido, questions_path y denominador por métrica |
| CO-2 — [remaining](memory-code-remaining.md#cognee) | cerrado | [Cognee](../cognee.md):805–842; cuerpos S y aceptación/matriz no atribuyen bytes dirty |
| GI-1 — [remaining](memory-code-remaining.md#graphiti) | cerrado | [Graphiti](../graphiti.md):732–768,798,864; App/WorkspaceShell, sin routes.ts/codec del pin inventados |
| GI-2 — [remaining](memory-code-remaining.md#graphiti) | cerrado documentalmente | [Graphiti](../graphiti.md):616–626,654–667,697; acumulación/reset/UUID y AddEpisodeResults no portable |
| MP-1 — [remaining](memory-code-remaining.md#mempalace) | cerrado documentalmente | [MemPalace](../mempalace.md):247–291,438–492,663–666,708–720; wiki ≠ publicación de fuentes/grafo |
| H-1 — [additions](memory-additions.md#hindsight) | cerrado documentalmente | [Hindsight](../hindsight.md):383–389,522–548,595–596,619–623; tags actuales y fuentes distintas, fix/cancelación acotados |
| H-2 — [additions](memory-additions.md#hindsight) | cerrado | [Hindsight](../hindsight.md):610–627; cuerpos/rutas S y semántica de error |
| AM-1 — [additions](memory-additions.md#agentmemory) | cerrado | [Agentmemory](../agentmemory.md):509,517–539; genlog no entrega/lease, proyección A/C propuesta |
| T-1 — [additions](memory-additions.md#tencentdb-agent-memory) | cerrado documentalmente | [Tencent](../tencentdb-agent-memory.md):216–227,299–300,455–491,641–643,654–655,678–681; cursor L0/callers ≠ task checkpoint |
| T-2 — [additions](memory-additions.md#tencentdb-agent-memory) | cerrado | [Tencent](../tencentdb-agent-memory.md):669–686; cuerpos S y tres estados captura/cursor/progreso |
| L-1 — [additions](memory-additions.md#letta) | cerrado documentalmente | [Letta](../letta.md):201–205,242–252,333,420–424; seguimiento y fetch autenticado LC, no fix universal |
| L-2 — [additions](memory-additions.md#letta) | cerrado | [Letta](../letta.md):351–367; S por cuerpos, GraphScene solo localizado |
| CM-1 — [code-memory](code-memory.md#codebase-memory-mcp) | cerrado documentalmente | [Codebase Memory MCP](../codebase-memory-mcp.md):111–113,355–371,459–461,534–539; query/delete/ADR/unwatch y errores |
| CM-2 — [code-memory](code-memory.md#codebase-memory-mcp) | cerrado documentalmente | [Codebase Memory MCP](../codebase-memory-mcp.md):342–353,484–517,534–539; ACK real y E nueva no acreditada, sin job inventado |
| GF-1 — [remaining](memory-code-remaining.md#graphify) | cerrado documentalmente | [Graphify](../graphify.md):126–133,209–212,235,365–381,479–487; refusal incluido en longitud, no utilidad |
| UA-1 — [remaining](memory-code-remaining.md#understand-anything) | cerrado (era no bloqueante) | [Understand Anything](../understand-anything.md):161–164,432–450,518,537–542; struct/campos reales y rechazo de linaje |

**Balance propuesto:** diecinueve correcciones examinadas y cerradas en
sentido documental; ninguna de esas diecinueve queda «no resuelta».
Permanece **residual baja** la coherencia UI de Codebase Memory MCP:
§10 L250–256 atribuye más implementación de la acreditada por App.tsx L1–132,
mientras §12.4 L521–524 limita correctamente la lectura a comentario/imports.
Es una observación sin ID antiguo propio, no una vigésima corrección aceptada,
ni una regresión de código. Se registra aquí sin editar el informe fuente,
sin bloquear el cierre ni encargar otro rework.

La integración E de Codebase Memory MCP conserva una **laguna de capacidad**
explícita: caller code-index, publicación/ACK, recuperación y persistencia
end-to-end no acreditados. Eso no reabre automáticamente CM-2: su corrección
pedía localizar el seam **o** declarar integración nueva, y ahora lo hace.
No se confunde ese cierre con un job implementado ni con garantía de crash
recovery. Análogamente, MP C/F, Tencent C y MutationReceipt siguen propuestas
o contratos pendientes, aunque se haya corregido la atribución documental.

## Límites transversales, integridad de inputs y cierre

### Integridad de la comparación

Lectura íntegra de los **once informes actuales**: 7 360 líneas en total
(alcances individuales indicados arriba), no lectura por encabezados.
Lectura del recibo de rework y de todos los hunks correspondientes; BASE
byte a byte contrastada con los actuales mediante comparación textual en
memoria, sin escribir otro archivo ni ejecutar software de los proyectos.
La recomposición de `difflib.unified_diff` sobre los trece pares del recibo
(once informes más los dos registros) produce **112 255 bytes idénticos**
al diff archivado. SHA-256 comprobado en ambos:
`c5e2c5968d6d981b4001c316b93d7a47835b2f8f1de53d4234d88cbc16db04ec`.
Esto acredita correspondencia BASE→actual con ese diff, no veracidad de los
claims ni firma/autenticidad de los documentos. Los dos registros se leyeron
como contexto del diff; no se editaron ni se autoriza desde aquí cambiar el
estado de aceptación del lead.

### FMEA y límites de lo que se cierra

- **Commit/publicación:** primario, cursor, checkpoint, derivado y delivery
  son fronteras distintas. Fallo después de una escritura puede ser parcial,
  no rollback; cancelación no deshace un commit ni una factura remota.
- **Concurrencia:** version/provenance no prueban CAS. Caller real y recurso
  protegido deben coincidir; fuentes vivas no sustituyen metadata actual,
  cursor L0 no es task checkpoint y lock de proceso no es fencing distribuido.
- **Benchmarks:** se describieron runners existentes, preparación efectiva,
  unidades/denominadores/defaults/exclusiones/errores/cache/reset/modelos en
  los alcances de cada recibo. Ningún score se ejecutó, reprodujo o recalculó.
  PASS, salida corta, idle, exit 0 o retain aceptado no prueban calidad,
  cobertura, búsqueda exitosa, publicación completa ni portabilidad.
- **Seguridad/ownership:** scope no autentica; root léxico no prueba
  confinamiento físico; hash no firma ni anonimiza por sí solo. Bajas propias
  de un productor no autorizan borrar ADR/memoria manual, historial o backups.
- **UI:** solo cuerpos S bajo `feature-freez/desktop/...`; no Studio dirty
  como fuente inmutable. Panel existente no implementa el nuevo contrato;
  error/stale/no configurado no deben convertirse en vacío legítimo.
- **Evidencia pública:** páginas/estados/comentarios son observaciones vivas
  separadas de los pins. Fix reportado, issue cerrado o PR cerrado no son
  por sí solos prueba de merge/presencia en pin/artefacto release verificado.
  Donde no hubo nueva lectura pública se dice expresamente; no se convierte
  el recibo del autor en auditoría propia del historial completo.
- **Cobertura no exhaustiva:** no se certifican todos los providers,
  backends/ACL, parsers, SDKs, callers, crash recovery, erasure física,
  licencias transitivas, UI dinámica o comparabilidad de datasets.
  «No acreditado» nunca se convierte en «inexistente».

### Operaciones y alcance de escritura

En esta reconfirmación solo se escribió
`docs/dev/research/agent-memory-2026-11-07/reviews/corrections-memory-closure.md`,
mediante patches. Lecturas: terminal (`sed`, `nl`, `grep`, `wc`, hashes),
Git read-only (`show`, `ls-tree`, `cat-file`, `rev-parse`, `status`,
`check-ignore`, `ls-files`), lectura pública raw/HTML y comparación de textos
con biblioteca estándar en memoria. Sin nuevos ficheros auxiliares.
Fallos propios de impresión cp1252 y ruta `memory_task_handler.rs` se
documentaron y resolvieron por relectura UTF-8/cuerpo real; no son fallos del
producto. No se usaron subagentes ni `code_search` en esta ejecución.

No apps, tests, builds, benchmarks, instalaciones, servicios o arranques/
paradas de runtime; no validadores de proyecto, nuevos harnesses, scoring,
prompts, configuraciones o umbrales. No mantenimiento/reindexado de grafos,
fetch/pull/modificación de clones, código, JSON, contratos, índices, backlog,
otros informes ni Forge de Meilisearch. **Sin staging, commits ni push**,
por la restricción específica de esta revisión, aunque el flujo general de
implementaciones ordinarias disponga otra cosa.

El status final contiene cambios ajenos concurrentes; no se atribuyen ni se
revierten. Este recibo no está trackeado: `git ls-files -- <ruta>` no devuelve
entrada y `git check-ignore -v --no-index` señala `.gitignore:268`
(`docs/dev/research/`). Su ausencia en status no significa pérdida del archivo.
No se fuerza add ni se cambia el ignore.

**Propuesta de cierre al responsable principal:** once dictámenes guardados,
diecinueve correcciones documentales reconfirmadas y una observación UI
residual explícita. Sin regresión material nueva demostrada en la muestra.
Esto **no proclama aceptación del lead**, no cambia los registros y no
autoriza implementación. La decisión final de aceptación pertenece al lead;
el límite estático no obliga a iniciar runtime para cerrar este encargo.
