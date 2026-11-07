---
title: "Quivr 2: engine Go de contenido versionado y monitoring"
kind: research
status: proposed
description: "Contraste del Quivr actual con VantaDB: aceptacion durable, proyecciones de busqueda y alertas sensibles a correcciones"
tags: [research, memory, engineering]
---

# Quivr 2: engine Go de contenido versionado y monitoring

## 1. Recomendaciones para VantaDB y contexto arquitectonico del proyecto.

**Dictamen:** estudiar versionado/withdrawal y alertas con correcciones; no
trasladar el antiguo relato de «brain Python». Este HEAD es un **engine Go de
contenido que sigue llegando**, busqueda y monitoring. No es un gestor de memoria
personal ni un producto completo de chat.

- Repo `The-Vibe-Company/quivr`; clon `C:\Users\Eros\quivr`.
- HEAD verificado `bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f`.
- [Snapshot](github-candidates.json): **39 633 estrellas**, no archivado,
  licencia API MIT. [LICENSE](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/LICENSE#L1-L21)
  MIT, copyright 2026 The Vibe Company; revisar plugins/dependencias aparte.
- [go.mod](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/go.mod#L1-L35):
  Go 1.27.3, PostgreSQL/pgx, Temporal, OpenTelemetry, MCP y plugins HTTP.

Mapa: HTTP/collectors → aceptacion con receipt e idempotency key → contenido
canonico/versiones en PostgreSQL + object storage → workers/Temporal →
normalizacion/segmentacion/embeddings por plugins → proyeccion de busqueda →
resultados con pasaje/version. Journal de cambios → intents de monitoring →
evaluador fijado → match/notice/outbox → lectura o webhook firmado.
El [README actual](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/README.md#L5-L54)
declara engine sin UI propia; `quivr-search/` es demo separado dentro del repo.
No atribuir al core limites comerciales o aislamiento basados en facturacion.

## 2. Partes extraibles, flujo real, lectura de codigo y mapeo a Rust/VantaDB.

| Fuente leida | Contrato observado |
|---|---|
| [S1 content.go:117-190](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/content.go#L117-L190) | `Command`: revision/position/provenance; withdrawal separado y absorbente; Record distinto de Version |
| [S2 content.go:331-385](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/content.go#L331-L385) | `Accept` exige action/corpus; limpia attribution confiable y valida procedencia/blobs |
| [S3 content.go:422-453](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/content.go#L422-L453) | `Withdraw` puede cercar identidad antes de materializar; receipt oculta IDs si falta lectura |
| [S4 submission.go:8-35](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/submission.go#L8-L35) | Submit/Batch autorizan envelope, pero conservan validacion e idempotencia por entrada |
| [S5 serving_projection.go:12-54](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/processing/serving_projection.go#L12-L54) | `RunServing` comprueba elegibilidad y generation; trabajo de publicacion separado de evaluacion opcional |
| [S6 engine.go:16-55](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/engine.go#L16-L55) | Outcomes distintos: no_match/not_ready/ineligible/superseded/withdrawn |
| [S7 engine.go:104-155](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/engine.go#L104-L155) | `EvaluationStore`: fanout con checkpoints, lease, commit atomico de match/notice/outbox; error se reintenta, no se vuelve negativo |
| [S8 match_committer.go:22-65](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/match_committer.go#L22-L65) | `CommitMatches`: batching de positivos validados, queues por Organization |
| [S9 webhook.go:11-37](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/webhook.go#L11-L37) | `ParseSecret`/`Sign`: firma id+timestamp+body, no autenticacion por URL secreta |
| [S10 online/search.go:14-46](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/online/search.go#L14-L46) | CLI sirve busqueda lexical/semantic/hybrid con corpus y procedencia |

El origen durable no se confunde con el indice reconstruible. Actualizar una
fuente produce Version; la disponibilidad y proyeccion no son el contenido
canonico. Withdrawal cerca la identidad incluso si llega antes que ingestion.
S5 separa experimentos de evaluacion de slots necesarios para servir.
S7 diferencia correccion que deja de hacer match de retirada y de error
operativo. No se ha auditado cada adaptador SQL para certificar estas garantias
en ejecucion; aqui se identifican contratos y caminos concretos.

Retrieval devuelve evidencia de pasaje/documento/version segun README/CLI; no
se acredita presupuesto de prompt ni generador QA obligatorio. Segmentar y
embeddings son plugins; sus politicas no son una unica estrategia fija del core.

### Ficha candidata A — retiro cercado frente a materializacion tardia

- **Objetivo / unidad minima:** evitar que un job antiguo reviva una fuente.
  `Service.Accept`/`Withdraw` (S2:331-385, S3:425-453) y `RunServing` (S5:24-51).
- **Seam Vanta:** lifecycle/supersession y rollback en
  `vanta-memory/src/core/record/`, wiki store/state y scheduler existentes.
- **Contrato propuesto:** `SourceIdentity { scope, namespace, key }`,
  `RevisionReceipt { revision, accepted, materialized }`; `accept`,
  `retire_source`, `publish_if_eligible`, `inspect_receipt`.
- **Acciones:** comprobar equivalencia con supersession actual; definir si
  retiro es reversible y como se autoriza nueva identidad; vincular derivado
  y referencia a revision, sin introducir otra base canonica.
- **Dependencias:** transaccion, idempotencia, procedencia y lectura autorizada.
- **FMEA:** baja antes de alta ignorada → resurreccion; receipt revela existencia
  → fuga; cambio a texto antiguo confundido con replay → actualizacion perdida.
  Fence independiente, redaccion de receipt y revision separada de hash textual.
- **Aceptacion pendiente:** baja antes/despues de job; replay no altera resultado;
  correccion que vuelve a texto anterior sigue siendo revision nueva; no-lector
  no recibe IDs. **No objetivos:** Temporal obligatorio ni retencion ilimitada.

### Ficha candidata B — observacion sensible a correcciones

- **Objetivo / unidad minima:** que una alerta previa no siga siendo «verdadera»
  tras una correccion o retiro. Outcomes/`EvaluationStore.CommitNoMatch`/
  `CommitWithdrawal` (S6/S7:120-133), no la plataforma completa.
- **Seam Vanta:** scheduler y checkpoints de `vanta-memory`, procedencia y
  actividad de Studio. Es una extension de observacion, no reemplazo de recall.
- **Contrato propuesto:** `Observation { subscription_revision, source_revision,
  evidence_ids, decision }`; `evaluate`, `commit_if_current`, `retract`,
  `list_changes`. `Pending/Error/NoMatch` distintos; delivery desacoplado.
- **Acciones:** comprobar watchers actuales; fijar snapshot de reglas/modelo;
  persistir resultado+evento junto a checkpoint; entregar por adaptador opt-in.
- **Dependencias:** journal durable, alcance autorizado y backpressure;
  evaluador puro o LLM opcional. No imponer webhooks ni servidor al core.
- **FMEA:** timeout tratado negativo; regla cambia durante job; reentrega duplica
  notificacion. Outcome explicito, revision de subscripcion y clave idempotente.
- **Aceptacion pendiente:** correccion genera retractacion; retry no duplica;
  error no se presenta como no-match; scope retirado deja de revelar fuentes.
  **No objetivos:** reglas comerciales, cuotas o SLA distribuido.

## 3. Defectos, riesgos e issues transferibles, con su nivel de evidencia.

No se establece un defecto runtime. **Codigo:** S7 evita deliberadamente que un
error sea negativo; S5 omite publicacion no elegible. Son patrones preventivos,
no prueba de que otra ruta este rota. **Hipotesis:** una integracion que conserve
solo el primer match y descarte notices podria seguir citando contenido retirado.
La frontera afectada es engine → aplicacion consumidora, no necesariamente core.

**Hipotesis de seguridad:** plugins y destinos webhook cruzan frontera de red;
firma de S9 prueba integridad del mensaje, no autorizacion del destino ni
prevencion completa de SSRF. No se audito transporte de delivery ni se ejecutaron
ataques. Issues/PR remotos **no consultados**; instrucciones del clone no
autorizan escribir en servicios de tareas ajenos.

## 4. Prerrequisitos en VantaDB antes de incorporar la idea.

Revalidar supersession, tombstones, rollback, dedup y scheduler actuales.
Definir diferencias entre borrar memoria, retirar fuente y revocar acceso.
Preservar referencias historicas sin servir contenido no autorizado. Contrastar
observacion con capacidades existentes antes de crear tareas. Revision de licencia
MIT por fichero y dependencias; lenguaje Rust no elimina derivacion.

## 5. Componentes y practicas que no deben trasladarse.

No incrustar PostgreSQL/Temporal/object storage/plugins HTTP como requisitos del
motor. No confundir append-only ilimitado con una politica de privacidad; retencion
y olvido necesitan contrato propio. No convertir alertas tematicas en extraccion
obligatoria de memoria personal ni traducir el viejo brain inexistente en este HEAD.

## 6. Advertencias metodologicas.

**Ledger:** rangos S1-S10, README/go.mod/LICENSE, documentacion y runner de §7.
No leidos integralmente adaptadores PostgreSQL, acquisition RSS/M365/X, plugins,
transportes HTTP, demo UI ni todas las migrations. La arquitectura nueva se
afirma por codigo/manifest actuales, no por popularidad historica.
Pendiente manual: ingestion incremental, retirada, corte de generacion,
reentrega/aislamiento, demo y costes. Sin servicios, tests ni benchmark ejecutados.

## 7. Benchmarks existentes: runner, entrada, metricas, dependencias y riesgos.

[scripts/eval/run.py:1-54](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/run.py#L1-L54)
y [guia de evaluacion](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/docs/agents/evaluation.md#L1-L60):
`make eval` crea Corpus, ingiere por API, espera vectores, consulta modos/perfiles
y calcula nDCG@10, Recall@10, MRR@10, latencia y paid calls por query.
Entradas publicas FR/EN fijadas en registro; privadas en formato TREC con corpus,
queries y qrels. No hay juez generativo fijo para esas metricas: relevancia
anotada y scoring de rankings. Reranker/evaluation plugin puede introducir llamadas
pagadas y otra segmentacion; se deduplica por Record.

Stack local aislado requiere Linux x86_64/dependencias de despliegue; API existente
requiere permisos de corpora/content/changes/search. **Cada run crea Corpora y no
los borra**, segun guia; cache/reportes quedan en `.scratch/eval/`.
No usar un tenant compartido ni tomar sampled scores como full-corpus. El runner
falla por errores de harness/dependencias, no convierte cualquier score bajo en
test fallido. No se ejecuta ni se adoptan sus gates/prompts/configuraciones;
comparacion solo por [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz existe/parcial/no acreditado/no aplicable frente a VantaDB.

| Superficie Vanta | Estado baseline | Diferencial a comprobar |
|---|---|---|
| BM25/HNSW/RRF y procedencia | Existe | Pasajes con revision, no otro search stack |
| Sesiones, supersession, dedup opcional | Existe | Retiro previo a materializacion: no acreditado |
| Assembly/compactacion compartida | Existe | Quivr no demuestra sustituto |
| Scheduler/checkpoints/wiki | Existe | Eventos correctivos de observaciones: no acreditado |
| Studio Tauri conexiones/memoria | Existe | Receipt/generacion como extension |
| Studio escenas/skills/actividad/retrieval | Existe | Actividad puede mostrar retractacion; no nueva consola |
| Engine de monitoring como requisito core | No aplicable | Adaptador de dominio opcional |

## 9. Debilidades de VantaDB sugeridas por el contraste, no bugs inventados.

Preguntar si una referencia citada conserva revision y si retirar una fuente
invalida todas sus observaciones/derivados pendientes. No se afirma que Vanta
carezca de memoria, procedencia o scheduler: estan en la baseline. La utilidad
de alertas persistentes depende de necesidad real de producto, no de imitar Quivr.

## 10. Escritorio, interfaz e inspeccion: implementado, externo o no localizado.

Engine sin UI propia segun README; `quivr-search/` es fuente de demo web
localizada, no un desktop nativo auditado ni el antiguo frontend «brain».
Documentacion alojada tiene fuente en `docs-site/`; el hosting no acredita otras
capacidades. UI/runtime no abiertos. Vanta mantiene Studio Tauri existente:
integrar estados y evidencia en memoria/actividad/retrieval, sin otra consola.
