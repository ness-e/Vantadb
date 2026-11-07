---
title: "Flowise: composición de recuperación y configuración observable"
kind: research
status: proposed
description: "Análisis estático parcial del Flowise archivado, sus rutas de retrieval, evaluación e interfaz y contratos mínimos para VantaDB"
tags: [research, memory, engineering]
---

# Flowise: composición de recuperación y configuración observable

## 1. Arquitectura, stack, flujo y recomendación

**ARCHIVADO según el snapshot del 2026-11-07: `archived=true`.**
No se oculta ni se infiere mantenimiento activo a partir de estrellas o código.
**Dictamen:** referencia histórica de composición/diagnóstico, no dependencia
estratégica del core. Estudiar identidad de resultados y precedencia de config,
sin portar flows visuales, marketplace o el monolito Node.

- Repositorio: `FlowiseAI/Flowise`; clon `C:\Users\Eros\Flowise`.
- SHA congelado: `9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb`.
- Snapshot: **55 562 estrellas**, **archivado**, licencia API `NOASSERTION`.
- Fuente: [candidatos](github-candidates.json) y [clone-manifest.json](clone-manifest.json).
- LICENSE distingue contenido Apache 2.0, enterprise/comercial y terceros.
- `NOASSERTION` no autoriza copiar ni significa ausencia de licencia.

Monorepo Node/TypeScript con componentes LangChain, servidor Express/TypeORM y
UI React/MUI/Redux; paquetes y scripts de Turbo/pnpm constan en manifiestos.
Los nodos construyen proveedores a partir de configuraciones persistidas,
con imports dinámicos y credenciales que resuelve el entorno de aplicación.
No se auditaron todos los proveedores, agentes ni mecanismos enterprise.

Flujo leído: Retriever Agentflow → lista de document stores → cargar config
embedding/vector store → construir retriever → `invoke` con cancelación →
texto o JSON metadata → estado de ejecución/SSE.
La consulta diagnóstica del servidor usa un flujo similar y trata de reconciliar
los resultados con chunks persistidos por store y contenido.
RRF es otro nodo: genera variantes LLM, busca por variante y fusiona; no se
demuestra que esté activo en el flujo Agentflow inspeccionado.

Vanta ya tiene BM25/HNSW/RRF/sparse, provenance/sesiones/supersession/dedup,
assembler/compaction, scheduler/checkpoints, ingestors/auditoría y Studio.
Studio Tauri/React vive en `develop-desktop`; no se propone otra aplicación.
Base histórica `c561158253bb88192590fa406c61e0a27dd7df8d`;
HEAD informado `167435a9e` contiene trabajo ajeno. No se reaudita Vanta.

## 2. Mecanismos mínimos extraíbles y seams Vanta

### A. Resultado compuesto con identidad y outcome por fuente

**Problema:** componer varias fuentes exige conservar qué devolvió cada una,
su identidad estable y sus errores; concatenar texto no lo garantiza.
Unidad mínima: envelope de resultados inspirada en fronteras S1/S2/S3,
**no copiar el merge observado**, que tiene limitaciones estáticas.
S1 permite seleccionar varios stores, pero asigna `docs = ...` en cada vuelta.
S3 intenta resolver ID por contenido y genera UUID si no encuentra chunk.
S4 usa `relevancy_score` para distinguir resultados; score no es identidad.

**Seam Vanta:** retrieval/RRF existente → provenance → assembler/reporte;
ingestors o conexiones externas como adaptadores, no nuevo agregador en core.
**DTO hipotético, no API existente:** `SourceRetrievalBatch { scope,
source_id, source_revision, config_revision, status, hits }`;
`EvidenceHit { evidence_id, revision, rank, score_domain, source_ref }`.
Operaciones propuestas: `retrieve_scoped`, `record_source_outcome`,
`merge_by_identity`, `select_existing_rrf`; sin LLM obligatorio.

**Acciones:** comprobar composición actual antes de agregar tipos; preservar
resultados por fuente hasta ensamblado; distinguir error, vacío y cancelación;
mantener desempates por identidad sin descartar documentos de igual score.
El DTO externo se traduce a provenance ya existente, no a metadatos arbitrarios
que el motor tenga que interpretar como política.
**Dependencias:** IDs/revisiones, autorización por fuente, cancelación y
presupuesto compartido existentes; revisión del adaptador si usa red/modelos.
**FMEA:** asignación por iteración → fuentes anteriores desaparecen; empate
deduplicado por score → pérdida de documentos; match por contenido → identidad
ambigua; fallo parcial → falso éxito total. Mitigar con batches por fuente,
clave revisionada, outcome tipado y política explícita de parcialidad.
**Aceptación pendiente manual:** dos fuentes conservan sus hits; scores iguales
no colapsan IDs distintos; mismo contenido con distinta procedencia se representa;
cancelar no se informa como éxito vacío; no mezclar scopes.
**No objetivo:** implementar RRF de nuevo o copiar generación multi-query.

### B. Configuración efectiva de observabilidad con dueño único

**Problema:** entorno y UI pueden activar trazadores duplicados o hacer que una
preferencia aparente no coincida con el proveedor realmente usado.
Unidad mínima: resolución de precedencia y ownership de emisión de S5.
El resolver cachea configuración de entorno durante la vida del proceso.
UI gana cuando su proveedor tiene `status:true`; en otros casos el entorno
habilitado puede aportar entrada. Un UI desactivado no es veto universal.
Al adoptar LangSmith, elimina flags del auto-tracer para evitar doble emisión.
Eso describe una decisión upstream; no se recomienda mutar env global de Vanta.

**Seam Vanta:** configuración/diagnóstico y auditoría existentes → actividad y
conexiones Studio; exportador externo opt-in fuera del motor.
**DTO hipotético:** `EffectiveObservationConfig { revision, enabled,
origin: User|Environment|Default, emitter_owner, destination_ref, redaction }`.
Nunca incluir token, URL con secreto ni texto recuperado por defecto.
Operaciones propuestas: `resolve`, `inspect_redacted`, `emit_once`,
`report_reload_requirement`; no construir un servicio nuevo.

**Acciones:** cotejar precedencia actual; hacer visible origen y cuándo entra
en vigor un cambio; elegir un dueño de eventos y no un segundo auto-tracer;
mantener valores secretos en el dueño existente de credenciales.
**Dependencias:** configuración versionada, lifecycle, redacción y trazas locales;
conectividad solo si el usuario aprueba exportación externa.
**FMEA:** dos emisores → costes/trazas duplicadas; caché eterna → config obsoleta;
disable de UI sobreescrito por env → expectativa de privacidad rota;
error exportador → bloqueo del retrieval. Mitigar con política explícita,
estado efectivo inspeccionable y fallo de exportación separado.
**Aceptación pendiente:** un evento tiene un dueño; origen efectivo es visible;
desactivación tiene semántica documentada; inspección no revela credenciales;
retrieval local funciona sin exportadores ni red.
**No objetivo:** adoptar LangSmith/analytics o reiniciar servicios en esta tarea.

## 3. Riesgos estáticos frente a hipótesis

- **Código:** S1:153-176 sobrescribe `docs` por store. Para dos stores exitosos,
  el valor final conserva la última asignación, no la unión. No ejecutado.
- **Código:** S4:83-90 usa score en `seenScores`, no ID documental.
  Dos documentos distintos con igual score no se conservan ambos en esa función;
  inferencia lógica del bucle, no medición de recall en una aplicación real.
- **Código:** `RRFRetriever.ts:112` elige default de `c` usando `topK`, no `c`.
  Configuraciones no predeterminadas requieren comprobación antes de copiar.
- **Código:** S3 reconcilia contenido de resultados con chunks; fallback UUID
  no acredita continuidad de identidad ni revisión de fuente.
- **Hipótesis:** contenido igual en varios chunks puede producir atribución
  ambigua; requiere fixtures del store y consulta completa para demostrarlo.
- **Código:** S2/S3 resuelven por ID; no se revisó la cadena completa de
  controladores/ACL. No se afirma fuga cross-workspace ni vulnerabilidad.
- **Código:** S5 cachea env y borra flags de auto-tracing; un cambio de env
  no actualiza el valor cacheado. El comentario exige reinicio para recargar.
- **Hipótesis de seguridad:** texto con metadata → estado/plantillas de agente;
  no debería conceder privilegios. No se auditaron todas las herramientas.

Issues/PR remotos **no revisados**, no CVEs inventadas ni explotación atribuida.
ARCHIVADO eleva riesgo de continuidad/mantenimiento, pero no prueba inseguridad
de una ruta concreta. No se recomienda adoptar dependencia de un fork sin evaluar.

## 4. Prerrequisitos Vanta

Revalidar la implementación actual antes de aprobar tareas; RRF ya existe.
Conservar scope y provenance al mezclar conectores, sin merge por score.
Definir resultado parcial y cancelación en el reporte actual.
Precisar quién fija configuración efectiva y quién emite/exporta diagnósticos.
Comprobar políticas existentes de redacción/retención y cierre de conexiones.
Separar revisión documental, revisión de embedding y configuración de búsqueda.
Revisión legal por archivo: enterprise y avisos explícitos tienen condiciones
distintas; reescribir en Rust no elimina obligaciones si se deriva expresión.

## 5. No trasladar

No portar marketplace, canvas, cloud, billing/enterprise ni monolito Node al core.
No copiar esta implementación RRF ni justificar duplicar la de Vanta.
No usar score como identidad, ni contenido como sustituto de fuente revisionada.
No elevar `UPSERTED` o `SYNC` a garantía transaccional universal.
No mutar variables de entorno globales para coordinar librerías del motor.
No activar inferencia LLM para fusionar listas que Vanta ya puede fusionar.
No arrastrar evaluadores/jueces del producto como criterio de aceptación Vanta.
No recomendar dependencia estratégica ignorando el archivo del repositorio.

## 6. Límites, licencia, método y ledger

Lectura estática acotada del snapshot **archivado**; no auditoría completa de
Flowise, proveedores, flujos enterprise, auth, colas o integraciones.
No red, scripts, instalaciones, apps/tests/builds/benchmarks/servicios,
commits o push. Enlaces inmutables construidos sin navegar a GitHub.
Clon superficial sin historia/otras ramas/tags; omisiones LFS del manifiesto.
LICENSE: enterprise y archivos con aviso explícito son comerciales; resto
Apache 2.0 salvo terceros. No sustituir ese reparto por etiqueta genérica MIT.

| ID | Evidencia inmutable y rangos leídos | Uso |
|---|---|---|
| S1 | [Retriever.ts:138-218](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/nodes/agentflow/Retriever/Retriever.ts#L138-L218) | Bucle de stores, estado y SSE |
| S2 | [DocStoreVector.ts:40-173](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/nodes/vectorstores/DocumentStoreVS/DocStoreVector.ts#L40-L173) | Config persistida e imports |
| S3 | [documentstore/index.ts:1489-1576](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/documentstore/index.ts#L1489-L1576) | Consulta, timing y reconciliación |
| S4 | [ReciprocalRankFusion.ts:1-96](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/nodes/retrievers/RRFRetriever/ReciprocalRankFusion.ts#L1-L96) y [RRFRetriever.ts:103-137](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/nodes/retrievers/RRFRetriever/RRFRetriever.ts#L103-L137) | Multi-query y fusión |
| S5 | [tracingEnv.ts:1-111](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/src/tracingEnv.ts#L1-L111) | Caché, precedencia y emisión |
| S6 | [DocumentStore.ts:1-43](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/database/entities/DocumentStore.ts#L1-L43) | Estado y configuraciones persistidas |
| S7 | [EvaluationRunner.ts:1-235](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/evaluation/EvaluationRunner.ts#L1-L235) | Runner real de aplicación |
| S8 | [evaluations/index.ts:160-196](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/index.ts#L160-L196) y [250-310](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/index.ts#L250-L310) | Persistencia y evaluadores |
| S9 | [DocumentStoreStatus.jsx:1-100](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/ui/src/views/docstore/DocumentStoreStatus.jsx#L1-L100) y [EvaluationResultSideDrawer.jsx:1-115](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/ui/src/views/evaluations/EvaluationResultSideDrawer.jsx#L1-L115) | Estado y detalle UI |
| S10 | [LICENSE.md:1-32](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/LICENSE.md#L1-L32) y [package.json:1-85](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/package.json#L1-L85) | Licencia mixta y paquetes |

Lectura adicional: `RRFRetriever.ts:1-102`, `packages/server/package.json:1-38`,
`packages/ui/package.json:1-36`; inventario de UI/evaluaciones/retrievers.
No inspeccionados todos los evaluadores simples/LLM, workers de predicción,
checkpointers, memory nodes, controladores de permisos ni frontend completo.
El código de evaluación no acredita resultados reproducidos.

## 7. Benchmarks/runners realmente leídos

S7 ejecuta evaluación por filas de dataset `{input, output, sequenceNo}` y
lista de chatflow IDs; llama `/api/v1/prediction/{id}` con request/evaluation ID.
Puede reutilizar `sessionId` configurado upstream: el runner no garantiza
aislamiento de memoria entre filas bajo cualquier configuración.
Recoge salida real, error, latencia de API y métricas de tokens/proveedor/modelo;
estima coste con catálogo y métricas anidadas de Agentflow cuando existen.
S8 agrega pass/fail/error de evaluadores adicionales y permite evaluadores LLM.
No se inspeccionó su implementación/prompt completo: no se certifica el juez.
Persiste EvaluationRun con entrada, esperado, salidas, errores y métricas.
Dependencias: Flowise operativo, DB, dataset/chatflows configurados,
credenciales y proveedores que cada flow utilice; axios/Node y modelo si aplica.
Efectos: ejecuta flows reales, escribe evaluaciones y potencialmente historial,
estado/herramientas; puede gastar tokens/dinero y consultar servicios externos.
No se acredita rollback universal ni sandbox de efectos de los flows.
Latencia end-to-end no aísla índice, y coste estimado depende del catálogo.
Pass/fail configurable de aplicación no prueba recall ni memoria longitudinal.
No runner de microbenchmark RRF acreditado en esta lectura acotada.
No se creó harness, prompt/juez, threshold, configuración o sampling.
No ejecutado; comparación futura bajo [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz capacidades Vanta

| Capacidad | Estado baseline | Diferencial a revalidar |
|---|---|---|
| BM25/HNSW/RRF/sparse | Existe | No copiar la fusión Flowise |
| Provenance/sesiones/supersession/dedup | Existe | Batches externos deben conservar identidad |
| Assembler/compaction | Existe | Presupuesto global, no texto concatenado sin recibo |
| Scheduler/checkpoints, ingestors/auditoría | Existe | No duplicar jobs/estado de aplicación |
| Studio conexiones/memoria/actividad/retrieval | Existe | Integración outcome por fuente: parcial por verificar |
| Precedencia observable única de trazadores | No acreditado | Revisar contrato actual antes de actuar |
| Plataforma visual/enterprise en core | No aplicable | Adaptadores y UI opt-in |

## 9. Deuda sugerida sin falsos bugs

¿Toda fusión conserva IDs y revisión por fuente?, ¿se ven resultados parciales?,
¿desactivar un exportador tiene una precedencia única?, ¿config efectiva es
inspeccionable sin secretos? No son bugs demostrados de Vanta.
Los fallos lógicos observados en Flowise son advertencias para no copiar
algoritmos por reputación; no invalidan ni reabren RRF existente de Vanta.
No se crean tareas ni se atribuyen ausencias a HEAD sin lectura autorizada.

## 10. Interfaz/escritorio e ideas Studio

**Código verificado:** UI React/MUI de estado documental distingue STALE,
EMPTY, SYNCING, UPSERTING, SYNC, UPSERTED y NEW; drawer de evaluación muestra
entrada, esperado y salidas por flow. Colores/status no certifican durabilidad.
Consulta diagnóstica S3 devuelve docs/timing para inspección, no calidad medida.
No se localizó shell Electron/Tauri propio en inventario acotado; frontend web
no equivale a escritorio nativo. No se ejecutó UI ni auditoría de accesibilidad.
Para Studio: origen/revisión por resultado, estado parcial por fuente y
config efectiva redactada en actividad/RetrievalLens; no una consola nueva.
Conservar Tauri/React y no importar canvas/marketplace a la distribución embebida.
