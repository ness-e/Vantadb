---
title: "Dify: configuración efectiva y trazabilidad del retrieval"
kind: research
status: proposed
description: "Análisis estático parcial del retrieval y diagnóstico de Dify, sus runners y las fronteras de adaptación a VantaDB"
tags: [research, memory, engineering]
---

# Dify: configuración efectiva y trazabilidad del retrieval

## 1. Arquitectura, stack, flujo y recomendación

**Dictamen:** extraer contratos de configuración efectiva y diagnóstico
redactado, no una plataforma de workflows. La composición de búsqueda/rerank
enseña fronteras útiles, pero no prueba superioridad sobre el híbrido de Vanta.
La licencia impide tratar el repositorio como Apache-2.0 sin condiciones.

- Repositorio: `langgenius/dify`; clon `C:\Users\Eros\dify`.
- SHA congelado: `65b4e17d6ebeff39c7e66c832649afff01666adf`.
- Snapshot: **160 588 estrellas**, `archived=false`, licencia API `NOASSERTION`.
- Fuente: [github-candidates.json](github-candidates.json) y [clone-manifest.json](clone-manifest.json).
- `LICENSE:1-22` declara Apache 2.0 modificado con condiciones adicionales.
- No se actualizaron metadatos, stars ni el snapshot de código.

El manifiesto API leído declara Python 3.13, Flask, SQLAlchemy a través del
código, Celery, Redis, PostgreSQL y dependencias de tracing/modelos.
La web usa React/TypeScript y tiene scripts Next/vinext; no se ejecutaron.
El inventario también contiene `dify-agent` y runtime separado: esta ficha sigue
el retrieval RAG de `api/`, no certifica toda la arquitectura de agentes.

Flujo leído: selección de datasets disponibles por tenant → filtros metadata →
estrategia single/multiple → retrieval por texto o attachment → búsqueda
keyword/semántica/fulltext según método → dedup híbrido → postproceso
ponderado o modelo reranker → documentos/segmentos para contexto y referencias.
`RetrievalService` propaga contexto de tracing a sus pools de threads.
Hit testing usa ese servicio, permite configuración de prueba y registra consultas.
Estado documental, configuración retrieval y spans son superficies separadas.

Vanta ya tiene BM25/HNSW/RRF/sparse, provenance, sesiones, supersession, dedup,
assembler/compaction, scheduler/checkpoints e ingestors/auditoría.
Studio Tauri/React existe en `develop-desktop`, incluido diagnóstico retrieval.
Base histórica `c561158253bb88192590fa406c61e0a27dd7df8d`;
HEAD informado `167435a9e` incluye commits ajenos. No se reaudita Vanta.

## 2. Mecanismos mínimos extraíbles y seams Vanta

### A. Plan efectivo de retrieval y resultado de cada etapa

**Problema:** configuración solicitada, resuelta y aplicada no son equivalentes.
Unidad mínima: resolución/explicación del plan, no pools Flask ni proveedores.
S2:119-164 elige override, configuración del dataset o default y aplica
excepción economy → keyword; el método mostrado en una pantalla no basta.
S1:815-877 enruta keyword, semántica y fulltext según método.
S1:891-924 deduplica híbrido y aplica postproceso o threshold alternativo.
S3 permite ausencia de runner cuando no hay modelo válido o autorización.

**Seam Vanta:** configuración de retrieval híbrido existente → reporte de
búsqueda y assembler; SDK/MCP/servidor como frontera de validación, no core cloud.
**DTO hipotético, no API actual:** `EffectiveRetrievalPlan { scope,
config_revision, requested_mode, applied_stages, score_domains, exclusions }`;
`StageOutcome { stage, status: Skipped|Applied|Failed, reason, candidate_count }`.
Operaciones propuestas: `resolve_plan`, `validate_capabilities`, `execute_stage`,
`record_outcome`. Los scores conservan dominio, no se fusionan por nombre.

**Acciones:** cotejar planes/reportes actuales; preservar configuración efectiva
con referencia a revisión; distinguir deshabilitado de no disponible; mostrar
fallback sin prometer que mantenga orden o calidad equivalentes.
No crear valores de evaluación ni copiar los defaults top-k del producto.
**Dependencias:** metadata/config versionada, límites de trabajo y cancelación
existentes; proveedores LLM/rerank solo adaptadores opcionales.
**FMEA:** configuración inválida → etapa omitida silenciosamente; score de
distancia usado como similitud → filtrado invertido; error parcial → falso vacío;
override mutante → diagnóstico irreproducible. Mitigar con enums/dominios,
validación previa y revisión inmutable, no con diccionarios sin procedencia.
**Aceptación pendiente manual:** mismo plan explica qué se ejecutó; deshabilitar
rerank no requiere proveedor; fallo de etapa es distinguible de cero hits;
consulta de diagnóstico no modifica por accidente configuración persistente.
**No objetivo:** reemplazar RRF por suma ponderada ni integrar la plataforma.

### B. Diagnóstico de recuperación con privacidad explícita

**Problema:** explicar retrieval sin convertir observabilidad en copia secundaria
de todos los documentos. Unidad: evento de recuperación y política de exportación.
S4:20-65 serializa contenido, metadata, score e ID; S4:86-105 coloca consulta y
documentos en atributos de span. Eso acredita payload, no destino efectivo.
S2 registra consultas de hit testing mediante `DatasetQuery` y `session.flush`.
S5 expone historial de consultas y resultados, diferenciando fuente externa.

**Seam Vanta:** auditoría/diagnóstico retrieval y actividad existentes →
RetrievalLens/MemoryLens de Studio; exportador externo opt-in fuera del core.
**DTO hipotético:** `RetrievalDiagnostic { trace_id, scope, config_revision,
source_refs, stage_outcomes, visibility: MetadataOnly|PreviewAuthorized,
redaction_revision }`. El DTO contiene referencias; previews se resuelven aparte.
Operaciones: `capture_metadata`, `authorize_preview`, `redact`, `export_opt_in`.
Retención de auditoría y trazas no debe heredar la del historial conversacional.

**Acciones:** comprobar si la auditoría actual ya cubre el evento; clasificar
metadata sensible; mantener identidad estable sin exportar texto por defecto;
explicar qué consultas se guardan y quién puede inspeccionarlas.
**Dependencias:** scope, autenticación/autorización, provenance, política de
retención y canal de actividad existentes; sin requisito de OpenTelemetry remoto.
**FMEA:** traza cruza tenant → fuga; referencia firmada expira → vista rota;
documento retirado → preview histórico no autorizado; exportador falla →
consulta bloqueada. Mitigar con resolución autorizada actual, redacción previa
y separación entre éxito del retrieval y fallo de observabilidad.
**Aceptación pendiente:** metadata-only no contiene texto/secretos; retirar
acceso impide preview; error de exportación es visible sin perder resultados;
se identifica revisión de configuración sin guardar credenciales.
**No objetivo:** construir nuevo dashboard de métricas o un juez de respuestas.

## 3. Riesgos estáticos y separación de hipótesis

- **Código:** S4 puede poner consulta/contenido/metadata en spans. El destino,
  redacción global y sampling efectivos no se auditaron; no se afirma fuga.
- **Hipótesis de privacidad:** exportar esos spans a tercero ampliaría frontera
  de confianza documento → proveedor de observabilidad; requiere configuración.
- **Código:** S3 devuelve `None` en ausencia de modelo o InvokeAuthorizationError.
  S1 contiene filtro alternativo de score si el híbrido carece de rerank runner.
  No es prueba de error actual ni de equivalencia semántica del fallback.
- **Código:** S1 usa pools anidados y `cancel()` de futures. No se acredita que
  una llamada de SDK ya iniciada se interrumpa; el coste residual es hipótesis.
- **Código:** hit testing persiste consultas; no es una operación puramente
  de lectura aunque la interfaz la presente como prueba de recuperación.
- **Frontera de autorización:** helper de bajo nivel resuelve por dataset ID;
  selección de datasets del caller recibe tenant. No se infiere bypass de
  permisos sin recorrer controladores, repositorios y verificación completa.
- **Hipótesis:** attachment/documento debe conservar ownership al expandir
  referencias; S1:928-956 verifica tenant y dataset en la ruta leída.

No CVEs atribuidas. Issues/PR remotos **no revisados**. Comentarios con números
de incidencias no certifican vigencia, corrección ni cronología.
No se auditó el sistema de plugins ni la seguridad global de la plataforma.

## 4. Prerrequisitos Vanta

Revalidar el estado actual antes de crear tareas, no usar ausencia de prueba
como prueba de ausencia. Mantener sparse/BM25/HNSW/RRF existentes.
Precisar qué capa puede introducir overrides y quién fija sus límites.
Conservar distinción entre score de cada etapa y orden final.
Verificar reportes existentes de assembly, auditoría y diagnóstico Studio.
Definir ownership/retención de una consulta guardada y previews revocables.
Revisión legal de licencia/archivo/diseño antes de derivar expresión o UI.
No copiar componentes visuales por asumir que todo está bajo Apache estándar.

## 5. No trasladar

No importar marketplace, plugin daemon, workflows visuales, billing/cloud ni
monolito web/API como dependencia del core Rust embebido.
No convertir Celery/Redis/SQL/pools Flask en nuevos requisitos de scheduler.
No reemplazar RRF por weighting sin evidencia específica del caso de uso.
No registrar texto completo en trazas por conveniencia.
No equiparar índice completo, documento habilitado y fuente autorizada:
el modelo de estado leído los distingue y una UI debe mantener esa separación.
No copiar defaults, límites o prompts de pruebas como contrato de producto Vanta.

## 6. Alcance, licencia y ledger

Lectura estática acotada; **no auditoría completa de Dify** ni análisis de todas
las rutas de autorización, datasets, herramientas, modelos o despliegues.
Sin red, ejecución de scripts, instalaciones, apps/tests/builds/benchmarks,
servicios, commits o push. Enlaces construidos a fuentes del snapshot local.
Clon superficial sin otras ramas/tags/historia; omisiones LFS según manifiesto.
`NOASSERTION` en GitHub no es ausencia de licencia.
LICENSE exige autorización comercial para uso multi-tenant salvo permiso escrito
y limita retirar/modificar logo/copyright del frontend; contiene además una
declaración sobre protección del diseño interactivo. No es asesoría jurídica.

| ID | Ruta/rango leído con enlace inmutable | Uso |
|---|---|---|
| S1 | [retrieval_service.py:93-173](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/datasource/retrieval_service.py#L93-L173), [203-267](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/datasource/retrieval_service.py#L203-L267), [792-956](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/datasource/retrieval_service.py#L792-L956) | Dispatch, dedup, postproceso, attachments |
| S2 | [hit_testing_service.py:65-192](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/services/hit_testing_service.py#L65-L192) | Config efectiva y log de consulta |
| S3 | [data_post_processor.py:1-134](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/data_post_processor/data_post_processor.py#L1-L134) | Modelo, weighting y reorder opcional |
| S4 | [otel/parser/retrieval.py:1-105](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/extensions/otel/parser/retrieval.py#L1-L105) | Datos serializados de tracing |
| S5 | [hit-testing/index.tsx:1-165](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/web/app/components/datasets/hit-testing/index.tsx#L1-L165) | Permisos UI, historial, cambios y resultado |
| S6 | [dataset_retrieval.py:82-155](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/retrieval/dataset_retrieval.py#L82-L155) y [470-525](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/core/rag/retrieval/dataset_retrieval.py#L470-L525) | Caller tenant, estrategia y filtros |
| S7 | [models/dataset.py:100-132](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/models/dataset.py#L100-L132) | Contrato de estado documental |
| S8 | [sse_benchmark.py:1-210](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/scripts/stress-test/sse_benchmark.py#L1-L210), [364-448](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/scripts/stress-test/sse_benchmark.py#L364-L448), [718-750](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/scripts/stress-test/sse_benchmark.py#L718-L750) | Runner SSE y reporte |
| S9 | [LICENSE:1-22](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/LICENSE#L1-L22), [api/pyproject.toml:1-75](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/api/pyproject.toml#L1-L75), [web/package.json:1-55](https://github.com/langgenius/dify/blob/65b4e17d6ebeff39c7e66c832649afff01666adf/web/package.json#L1-L55) | Licencia y stack |

Lectura adicional: `retrieval_service.py:1-92,175-202,269-320`;
inventario de runners/tests/tracing. No se leyeron todos los workers ni el
runtime Go; su presencia no acredita comportamiento que esta ficha no describe.

## 7. Benchmarks y runners realmente leídos

S8 es un **stress runner de transporte SSE/workflow**, basado en Locust.
No mide recall, memoria longitudinal ni fidelidad de citas.
Entradas: API key a través de `ConfigHelper`, ruta workflow, preguntas de fichero
o lista incorporada; POST con `response_mode=streaming`.
Métricas declaradas: conexiones, eventos, tasas, TTFE, duración de stream,
latencias inter-evento, percentiles y categorías de error.
El tracker conserva muestras acotadas de ciertas distribuciones; no representa
necesariamente toda la historia de una ejecución prolongada.
El reporte exporta JSON local y estadísticas agregadas de Locust.
Dependencias: Python, requests/Locust, helper de configuración y Dify operativo,
workflow publicado y sus proveedores/herramientas; no basta disponer del clon.
Efectos: ejecuta workflows reales, puede generar coste LLM, logs, mensajes o
mutaciones de herramientas; escribe `scripts/stress-test/reports/`.
No se inspeccionó `ConfigHelper` completo ni el parser de eventos entero.
TTFE no equivale a primer token útil; éxito de stream no mide corrección.

S2/S5 ofrecen hit testing interactivo con registro de consulta, no corpus golden
ni benchmark de calidad por sí mismos. Los tests localizados no se ejecutaron.
No se creó harness, prompt/juez, threshold, configuración ni sampling.
No resultados propios; comparación futura bajo [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz capacidades Vanta

| Capacidad | Estado baseline | Diferencial a revalidar |
|---|---|---|
| BM25/HNSW/RRF/sparse | Existe | No reemplazar por stack Dify |
| Provenance, sesiones, supersession, dedup | Existe | Referencias diagnósticas y autorización actual |
| Assembler/compaction | Existe | Etapas aplicadas y score domain: no acreditado integralmente |
| Scheduler/checkpoints, ingestors/auditoría | Existe | No incorporar workers/colas equivalentes |
| Studio: actividad y diagnóstico retrieval | Existe | Config solicitada vs efectiva: parcial por verificar |
| Exportación metadata-only con política revisionada | No acreditado | Baseline no certifica todas las integraciones |
| Marketplace/workflows/cloud dentro del core | No aplicable | Fuera del objetivo embebido |

## 9. Deuda sugerida sin falsos bugs

Preguntas abiertas: ¿un reporte conserva la configuración realmente aplicada?,
¿es visible cuándo se omite reranking?, ¿los previews respetan retirada de acceso
y las trazas tienen redacción independiente? No se afirma que Vanta carezca de ello.
El contraste sugiere revisar contratos en una tarea posterior autorizada.
No se heredan hallazgos históricos ni se crean bugs por omisión de esta lectura.

## 10. Interfaz/escritorio e ideas Studio

**Código verificado:** pantalla web de hit testing con consultas anteriores,
resultados internos/externos, modificación de retrieval y estado loading/error;
la UI calcula permiso de ejecución, sin que eso sustituya controles backend.
No se acredita un shell desktop nativo en las superficies inspeccionadas.
Tampoco se ejecutaron UI, navegación o comprobaciones de accesibilidad.
Para Studio existente: explicar plan efectivo, etapa omitida y score domain,
con preview autorizado y exportación explícita en RetrievalLens/actividad.
Inspirarse en los conceptos no autoriza copiar diseño o componentes licenciados.
No crear consola alternativa, runtime web obligatorio ni editor visual de flows.
