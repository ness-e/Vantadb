---
title: "Khoj: filtros explícitos, caché contextual y clientes de conocimiento"
kind: research
status: proposed
description: "Lectura estática acotada de búsqueda, configuración, evaluación y escritorio de Khoj para contrastar contratos con VantaDB"
tags: [research, memory, engineering]
---

# Khoj: filtros explícitos, caché contextual y clientes de conocimiento

## 1. Arquitectura, stack, flujo y recomendación

**Dictamen:** estudiar filtros separados del texto semántico y las claves de
identidad de caché; no incorporar el asistente personal ni su backend completo.
La interfaz de archivos y sincronización aporta ideas de inspección para Studio,
no un motivo para reemplazar Tauri por Electron.

- Repositorio: `khoj-ai/khoj`; clon `C:\Users\Eros\khoj`.
- SHA congelado: `ae229ca894c0b80ad84664afcfdde523b5e87057`.
- Snapshot: **38 033 estrellas**, `archived=false`, licencia API `AGPL-3.0`.
- Fuente: [candidatos](github-candidates.json) y [clones](clone-manifest.json).
- `pyproject.toml` declara `AGPL-3.0-or-later`; LICENSE contiene AGPLv3.
- Se conserva la distinción entre etiqueta de API y texto local, no se normaliza.

Stack leído: Python/FastAPI, modelos Django, PostgreSQL/pgvector,
sentence-transformers/Torch y clientes de inferencia; web React/Next y cliente
Electron con IPC y electron-store. No todo el asistente fue inspeccionado.
El scheduler global y configuración de modelos aparecen en código, pero no
se acredita aquí su comportamiento de automatizaciones o investigación profunda.

Flujo de búsqueda: API autenticada → `execute_search` → caché por usuario →
eliminar filtros del texto a embeber → embedding → consulta pgvector filtrada →
collation/dedup → cross-encoder opcional → límite → caché/respuesta.
Los filtros originales siguen hacia la capa ORM; no se pierden al defiltrar.
La consulta SQL expresa distancia coseno, no un pipeline híbrido BM25/RRF.
No se extrapola ese hecho a todas las herramientas o modos del asistente.

Vanta ya dispone de BM25/HNSW/RRF/sparse, provenance, sesiones, supersession,
dedup, assembler/compaction, scheduler/checkpoints, ingestors/auditoría y Studio.
La base `c561158253bb88192590fa406c61e0a27dd7df8d` es histórica;
HEAD informado `167435a9e` incluye trabajo ajeno. No se reaudita Vanta ni se
convierten hallazgos anteriores en defectos presentes.

## 2. Mecanismos mínimos extraíbles y seams Vanta

### A. Consulta estructurada con texto semántico separado

**Problema:** instrucciones de filtro no deberían distorsionar el embedding ni
confundirse con controles de acceso. Unidad: separación de S1/S2/S3, no Django.
S1 elimina términos de fecha/palabra/archivo solo para codificar la consulta.
S3 construye condiciones de palabras, globs y fechas sobre las entradas.
El scope propietario se aplica en la búsqueda; usuario y agente se combinan
por unión, después de comprobar acceso al agente en `execute_search`.
Sin usuario ni agente, S3 devuelve queryset vacío.

**Seam Vanta:** frontera de consulta de los ingestors/SDK → retrieval híbrido y
filtros existentes; explicación en diagnóstico retrieval de Studio.
No añadir un nuevo motor ni interpretar texto de usuario como namespace confiable.
**DTO hipotético, no existente:** `ParsedRetrievalQuery { semantic_text,
constraints, authorized_scope, parse_diagnostics, policy_revision }`.
`constraints` es un AST validado; `authorized_scope` lo resuelve el servidor,
no lo concede una expresión `file:` o el propio modelo.
Operaciones propuestas: `parse`, `authorize`, `lower_constraints`, `explain`.

**Acciones:** comprobar primero gramática actual y soporte de filtros; acordar
semántica de inclusión/exclusión y rangos; preservar consulta original para
diagnóstico redactado; hacer visibles filtros rechazados, no ignorarlos.
**Dependencias:** tipos de metadata, autorización, normalización temporal y
planificador de búsqueda existentes; ninguna dependencia LLM obligatoria.
**FMEA:** fechas ambiguas → resultados inesperados; glob mal interpretado →
alcance distinto; unión agente/usuario confundida con intersección → sobrelectura;
parser tolerante → filtro perdido. Mitigar con AST, errores explícitos y scope
independiente de la gramática, sin copiar regexes como política universal.
**Aceptación pendiente manual:** el embedding no incluye operadores; una
restricción inválida no amplía scope; la explicación preserva inclusiones y
exclusiones; un usuario no puede seleccionar un agente no autorizado.
**No objetivo:** convertir búsqueda semántica en lenguaje general de consultas.

### B. Identidad completa y vigencia de caché de retrieval

**Problema:** un resultado cacheado debe corresponder al mismo corpus, modelo,
política y scope, no solo al mismo texto. Unidad: entrada/salida de S1/S6.
La caché leída está particionada por `user.uuid`; su clave incluye consulta,
`n`, tipo, rerank, distancia y dedupe. No incluye agente ni revisión del modelo.
S6 invalida la caché del usuario al finalizar una ruta de configuración de contenido.
Eso acredita una invalidación, no cobertura de todos los cambios o procesos.

**Seam Vanta:** adaptador/cache de retrieval si existe, provenance/supersession,
reportes del assembler y checkpoints; no se afirma que falte una caché actual.
**DTO hipotético:** `RetrievalCacheIdentity { scope_id, corpus_revision,
query_digest, filter_digest, model_revision, ranking_revision }`;
`CachedRetrieval { identity, source_refs, expires_at, outcome }`.
Operaciones propuestas: `lookup_exact`, `validate_refs`, `insert_success`,
`invalidate_scope`; nunca almacenar autorización implícita en el texto.

**Acciones:** inventariar claves e invalidaciones existentes antes de crear
almacenamiento; preferir identidad por revisión a barridos sin coordinación;
registrar hit/miss y motivo sin guardar el prompt completo por defecto.
**Dependencias:** revisiones del corpus/config, IDs estables, límites de memoria
y política de retención; coordinación multi-proceso solo si realmente aplica.
**FMEA:** cambio de agente → mezcla de resultados; borrado → hit obsoleto;
cambio de encoder → distancia incompatible; caché local → invalidación desigual.
Mitigar con clave completa, validación actual de referencias y descarte de
entradas cuyo origen no pueda acreditarse.
**Aceptación pendiente:** mismo texto con scope/modelo distinto no comparte
entrada; retirada invalida refs; errores transitorios no se vuelven hits válidos;
la ausencia de caché conserva corrección del retrieval.
**No objetivo:** replicar la caché Python global ni agregar Redis al core.

## 3. Riesgos estáticos, hipótesis e issues

- **Código:** la clave S1 omite `agent`, aunque la consulta acepta ese parámetro.
  Es una divergencia observable de identidad, no exploit de aislamiento probado.
- **Hipótesis:** alternar agentes accesibles con la misma consulta podría
  reutilizar entradas de otro scope; faltan trazado de callers y reproducción.
- **Código:** S4 transforma score de cross-encoder en distancia y ordena con
  desempate por distancia bi-encoder; HTTPError produce scores cero.
- **Hipótesis operativa:** ese fallback mantiene orden bi-encoder, pero un cliente
  podría no distinguir degradación de ranking correcto; no se verificó UI.
- **Código:** umbrales usan truthiness en S2 y API (`max_distance or math.inf`).
  Un cero solicitado no conserva esa semántica en la ruta leída.
- **Código:** Desktop usa `nodeIntegration:true` (S9); hay puente IPC y host
  configurable. No basta para afirmar ejecución remota: faltan navegación,
  aislamiento efectivo y validación de contenidos completos.
- **Código:** estado global contiene caché LRU y telemetría con desactivación
  `KHOJ_TELEMETRY_DISABLE`; no se inspeccionó cada evento enviado.

Issues/PR remotos **no consultados**; sin CVEs inventadas ni atribución de bugs
históricos al SHA actual. Riesgos de prompt injection requieren contexto de
herramientas/privilegios no auditado aquí.

## 4. Prerrequisitos Vanta

Revalidar capacidades actuales sin reabrir una auditoría en esta ficha.
Precisar scope autorizado y momento de comprobación en lectura de referencias.
Documentar semántica de filtros antes de exponer una gramática en Studio.
Separar revisión documental de revisión de embedding/configuración.
Comprobar si el caché/reporte actual ya cubre esos campos y sus invalidaciones.
Conservar el funcionamiento offline y la opcionalidad del proveedor.
Revisión legal obligatoria antes de derivar código AGPL; estudiar el contrato
no equivale a autorizar copiarlo ni a resolver obligaciones de una obra derivada.

## 5. Qué no trasladar

No llevar Django/PostgreSQL/Torch/Electron como requisitos del motor Rust.
No copiar una unión usuario/agente como regla de permisos sin modelo de dominio.
No convertir términos `file:` en ACL ni confiar en el cliente para conceder scope.
No suponer que una invalidación local cubre workers, réplicas y todos los writers.
No portar automatizaciones, billing, investigación web o asistente monolítico.
No importar corpus/prompt/juez de evaluaciones como criterio de aceptación Vanta.
No reemplazar scheduler/checkpoints o dedup actuales por variantes equivalentes.

## 6. Método, licencia y ledger de evidencia

Análisis estático **parcial de una plataforma grande**, no auditoría integral.
No hubo red, instalaciones, código de proyecto ejecutado, servicios, apps,
tests, builds, benchmarks, commits o push. Se leyeron fuentes locales.
Enlaces GitHub son identificadores inmutables construidos sin acceder a la red.
Clon superficial sin otras ramas/tags/historia ni LFS descargado; manifiesto
sin submódulos declarados. No se examinaron dependencias transitivas completas.
AGPLv3 requiere evaluación de redistribución/modificaciones/servicio de red
según el uso concreto; no se emite dictamen jurídico de compatibilidad.

| ID | Evidencia inmutable y rangos leídos | Contrato |
|---|---|---|
| S1 | [routers/helpers.py:1484-1575](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/helpers.py#L1484-L1575) | Acceso agente, caché, defilter y ranking |
| S2 | [text_search.py:99-206](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/search_type/text_search.py#L99-L206) | Consulta y collation |
| S3 | [adapters:2090-2178](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/database/adapters/__init__.py#L2090-L2178) | ORM, owner y distancia |
| S4 | [text_search.py:235-257](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/search_type/text_search.py#L235-L257) | Cross-encoder y fallback |
| S5 | [models:544-587](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/database/models/__init__.py#L544-L587) y [state.py:1-41](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/utils/state.py#L1-L41) | Config/modelos y estado global |
| S6 | [helpers.py:3170-3181](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/helpers.py#L3170-L3181) y [api.py:46-77](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/khoj/routers/api.py#L46-L77) | Invalidación y endpoint |
| S7 | [eval.py:1-230](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L1-L230), [347-431](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L347-L431), [450-585](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L450-L585), [623-709](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/tests/evals/eval.py#L623-L709) | Datos, mutaciones, evaluadores y reporte |
| S8 | [search/page.tsx:365-440](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/web/app/search/page.tsx#L365-L440) y [665-726](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/web/app/search/page.tsx#L665-L726) | Carga, archivos, descarga |
| S9 | [desktop/main.js:1-145](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/desktop/main.js#L1-L145), [440-486](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/desktop/main.js#L440-L486) y [preload.js:12-90](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/src/interface/desktop/preload.js#L12-L90) | Host, archivos y capacidades IPC |
| S10 | [pyproject.toml:1-110](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/pyproject.toml#L1-L110) y [LICENSE:1-18](https://github.com/khoj-ai/khoj/blob/ae229ca894c0b80ad84664afcfdde523b5e87057/LICENSE#L1-L18) | Stack/licencia |

También leídos `text_search.py:1-98,209-232`, UI `search/page.tsx:1-90`,
`eval.py:288-346`; búsquedas por inventario y símbolos sobre los clones locales.
No auditados agentes, memoria personal, chat completo, OAuth, IPC completo,
sincronización completa, todos los filtros ni emisores de telemetría.

## 7. Benchmarks/runners realmente leídos

S7 es un runner upstream de evaluación del asistente vía `/api/chat`, no del
motor vectorial aislado. Carga FRAMES de HuggingFace y opcionalmente su KB
Wikipedia; también enruta SimpleQA, GPQA y MATH500.
Entradas contienen pregunta, respuesta y tipo de razonamiento; el runner
dispone de opciones upstream de tamaño, orden/semillas y modo, no elegidas aquí.
FRAMES-IR consulta referencias de archivos y calcula precisión/recall/F1;
**la función devuelve recall**, aunque el agregado se imprime como accuracy.
GPQA usa matching de opción; otros modos usan evaluación Gemini y coste.
El agregado descarta decisiones ausentes: éxito aparente exige leer errores
y denominador, no solo la media. No certifica memoria longitudinal.
Salidas: CSV de respuestas/referencias/uso y resumen TXT con medias y coste.
Dependencias: servidor Khoj, API key según despliegue, datasets/pandas/requests,
red para corpus y Gemini en modos que lo utilizan, más modelos del asistente.
Efectos: `frames_ir` puede indexar KB vía PATCH; chat crea conversaciones;
hay ficheros de reporte, coste de inferencia y posible ejecución de herramientas.
No se acredita rollback ni sandbox universal del contenido evaluado.
No ejecutado; no cifras propias, harness, jueces, prompts, umbrales,
configuraciones ni sampling nuevos. Ver [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz capacidades Vanta

| Capacidad | Estado baseline | Diferencial a revalidar |
|---|---|---|
| BM25/HNSW/RRF/sparse | Existe | No portar búsqueda pgvector |
| Provenance/sesiones/supersession/dedup | Existe | Vigencia de referencias cacheadas |
| Assembler/compaction | Existe | No sustituir por historial del asistente |
| Scheduler/checkpoints e ingestors/auditoría | Existe | No duplicar automatizaciones |
| Studio: conexiones, memoria, actividad, retrieval | Existe | Integración de filtros efectivos: parcial por verificar |
| Clave cacheada con todas las revisiones | No acreditado | No deducible de baseline; revisar antes de actuar |
| Electron/Django obligatorios | No aplicable | Adaptadores fuera del core |

## 9. Deuda sugerida sin falsos bugs

¿Existe explicación única de consulta original, texto semántico y filtros?,
¿la identidad de caché cubre modelo y agente?, ¿las invalidaciones atraviesan
todas las rutas de escritura? Son preguntas, no ausencias demostradas en Vanta.
La acción futura depende del código actual y puede ser solo documentación.
No se propone migración de índices ni una tarea aprobada por esta comparación.

## 10. Interfaz/escritorio e ideas Studio

**Verificado por código:** cliente Electron, host/token/archivos/carpetas
persistidos, selección y recursión de directorios, puentes IPC de sincronización;
ventana con atajo y trabajo periódico. No es solo un anuncio comercial.
Web contiene carga con progreso, gestión de archivos, selección de documento
y descarga de texto. No se ejecutó UI ni se evaluó accesibilidad.
Para Studio existente: vista previa de exclusiones antes de sincronizar,
scope/host destino visible y estado por archivo; explicación de filtros efectivos
en RetrievalLens. Conservar permisos de filesystem y secretos en frontera Tauri.
No trasladar el puente Electron ni abrir Node a contenido remoto.
