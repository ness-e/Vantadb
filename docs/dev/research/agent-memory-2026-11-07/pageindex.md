---
title: "PageIndex: intervalos documentales y publicación local de artefactos"
kind: research
status: proposed
description: "Lectura estática de PageIndex local, jerarquías de páginas y un runner documental sin extrapolar resultados externos"
tags: [research, memory, engineering]
---

# PageIndex: intervalos documentales y publicación local de artefactos

## Índice

- [1. Identidad y arquitectura](#1-recomendación-y-arquitectura)
- [2. Fuentes y candidatas A/B](#2-evidencia-y-extracciones-independientes)
- [3. Riesgos estáticos](#3-riesgos-y-defectos-nivel-de-evidencia)
- [4. Prerrequisitos](#4-prerrequisitos-vanta)
- [5. Rechazos](#5-qué-no-trasladar)
- [6. Método/licencias](#6-método-licencia-y-cobertura)
- [7. Benchmarks](#7-runner-benchmark-encontrado-y-realmente-leído)
- [8. Matriz](#8-matriz-vanta)
- [9. Negativos](#9-carencias-sugeridas-no-bugs-vanta)
- [10. UI](#10-ui-y-escritorio)
- [11. Flujos, contratos completos y fuentes públicas](#11-ampliación-de-cierre-flujos-contratos-y-fuentes-públicas)

## 1. Recomendación y arquitectura

**Dictamen:** estudiar el contrato de intervalos/citas y la publicación de
artefactos derivados; no sustituir ANN/BM25/sparse/RRF por una consigna
«vectorless» ni introducir llamadas LLM dentro del motor embebido.
Clone: `C:\Users\Eros\PageIndex`.
SHA: `2ac8694bf8fe89a3840dda9bdb88db5119b77fc4`.
Snapshot: **40 160 estrellas, MIT**, del [inventario](github-candidates.json).
Preparación: [manifiesto congelado](clone-manifest.json), captura 2026-11-07;
la primera lectura fue offline; §11 incorpora fuentes públicas autorizadas.

Stack observado: Python, LiteLLM/OpenAI/Agents/MCP, PyPDF2 y PDFium [P0].
El checkout contiene modo local y cloud; no es correcto describirlo sólo
como servicio remoto ni asumir que «local» implique cero llamadas externas.
`submit_document` elige Flash por defecto, valida entrada y extrae texto [P1].
El modo standard llama al constructor clásico; Flash extrae estructura,
aplica rechazo, asigna IDs y genera descripción mediante modelo [P2].

```text
PDF → texto por página (PyPDF2) ───────────────────┐
    → estructura Flash (PDFium) / standard       │
    → resumen/descripción y árbol final          │
    → tree.json + pages.json + doc.json + manifest
consulta/citas → árbol y rangos → contenido de páginas
```

El chequeo de límites [P2] existe precisamente porque dos parsers participan.
El árbol representa estructura documental; un intervalo no demuestra que una
respuesta sea correcta ni que el LLM cite la evidencia adecuada.
El almacenamiento JSON local [P4] muestra garantías por archivo, no una
transacción general entre cuatro artefactos.

**Vanta:** contraste fijado en `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`
([referencia](VANTA-REFERENCE.md)), no HEAD móvil. Los anclajes históricos
`c561158.../dirty` no acreditan defectos actuales; [V1–V3] se actualizan al pin.
Ya existen híbrido, provenance/sesiones, supersession/dedup, budgets,
scheduler/checkpoints/ingestors, cuotas y Studio. §11 precisa el delta.

## 2. Evidencia y extracciones independientes

| Ref. | Archivo y rango leído | Contrato observado |
|---|---|---|
| P0 | `requirements.txt:1-16` | Dependencias y parsers concretos |
| P1 | `pageindex/local_api.py:88-178` | Modos, entrada, validación y backend |
| P2 | `pageindex/local_api.py:219-286` | Límites, standard y Flash |
| P3 | `pageindex/tree_optimize.py:172-266` | Semántica unión, intro y reasignación de IDs |
| P4 | `pageindex/local_store.py:1-186` | Escritura atómica por archivo, lock, lectura y borrado |
| P5 | `pageindex/flash/api.py:1-100` | Validación PDF y fase de optimización |
| P6 | Notebook 10-K, celdas fuente indicadas en §7 | Runner de comparación con Gemini Embedding 2 |
| P7 | `README.md:37,163-176`; `LICENSE:1-21` | App externa, FinanceBench y MIT |

Los enlaces P6 apuntan al JSON del notebook, no a una versión ejecutada aquí.
No se ejecutó ni siquiera una celda de instalación.

### Candidata A — intervalo de fuente versionado y cobertura de introducción

- **Problema:** evitar que una jerarquía pierda el texto entre el comienzo de
  una sección y su primer hijo, o que una cita cambie al reindexar.
- **Unidad mínima:** intervalo de origen con revisión + relación padre/hijo;
  P3 acredita semántica unión e inserción explícita de introducción.
- **Seam Vanta:** `CandidatePage`, `sources` y merge por página [V1];
  contexto ensamblado con presupuesto ya existente [V2].
- **Contrato hipotético:** `SourceSpan { source_revision, start, end, unit }`
  y `SectionRef { stable_source_id, span, parent }`.
- **Acción 1:** inventariar spans/referencias ya persistidos por ingestors.
- **Acción 2:** distinguir página física, etiqueta impresa y offsets de texto;
  elegir conversión explícita al adaptar formatos, no números ambiguos.
- **Acción 3:** verificar intervalos antes de publicar estructura; rechazar
  inversión, fuera de rango o relación entre revisiones diferentes.
- **Acción 4:** conservar correspondencia de IDs al reconstruir un árbol;
  no usar la numeración secuencial P3 como identidad eterna.
- **Dependencias:** identidad de documento/revisión, texto original retenido,
  parser identificable, namespaces y autorización de fuente.
- **FMEA — introducción omitida:** búsqueda sólo en hojas pierde evidencia
  previa al primer encabezado; representar cobertura sin duplicarla al ensamblar.
- **FMEA — página compartida:** cortar antes del encabezado elimina texto útil;
  admitir solapamiento de fuente sin doble consumo de contexto.
- **FMEA — idiomas:** la heurística de inicio de página P3 exige letras latinas;
  no elevar esa regla a una verdad estructural universal.
- **Aceptación propuesta:** cada referencia resuelve sobre la revisión citada;
  el texto introductorio conserva una ruta de acceso y límites válidos.
- **Aceptación negativa:** sin evidencia de posición, reportar desconocido;
  no fabricar coordenadas a partir de un resumen del modelo.
- **Independencia:** puede trabajar con documentos ya publicados, sin B.
- **No objetivos:** nuevo parser PDF, modelo/juez ni reemplazar retrieval híbrido.

### Candidata B — publicación coherente de derivados documentales

- **Problema:** un lector no debería combinar árbol nuevo con páginas viejas
  ni interpretar una caché de manifiesto como fuente de verdad.
- **Unidad mínima:** descriptor de generación y receipt de publicación.
  P4 sirve como contraste: replace individual no equivale a commit conjunto.
- **Seam Vanta:** backend particionado y metadatos internos de salud [V3],
  junto con checkpoints/ingesta ya acreditados; no crear un docstore JSON.
- **Contrato hipotético:** `DocumentGeneration { source_revision, tree_digest,
  text_digest, state }` y `PublicationReceipt { generation, visibility }`.
- **Acción 1:** localizar las generaciones y receipts actuales; ampliar antes
  que duplicar un mecanismo existente.
- **Acción 2:** especificar qué puntero/registro hace visible una generación
  completa y cómo se distingue staging de publicada.
- **Acción 3:** ligar checkpoint y artefactos a una sola identidad de operación.
- **Acción 4:** definir relectura de una generación incompleta como diagnóstico,
  no borrado silencioso de la fuente primaria.
- **Dependencias:** batch/orden real del backend, validación de digests,
  semántica WAL y coordinación con recolector/rebuild existentes.
- **FMEA — crash intermedio:** derivados de generaciones distintas; impedir
  exposición conjunta hasta verificar el descriptor de publicación.
- **FMEA — último escritor:** dos ingestas de la misma fuente sobrescriben
  estado válido; exigir identidad/revisión en la transición visible.
- **FMEA — error confundido con ausencia:** una lectura fallida dispara
  reingesta destructiva; distinguir corrupción, permisos y documento inexistente.
- **Aceptación propuesta:** lectores ven una generación coherente o estado
  explícito no disponible; retry no vuelve a publicar una versión obsoleta.
- **Aceptación operativa:** fallo conserva diagnóstico y fuente original;
  no se anuncia durabilidad más fuerte que la proporcionada por el backend.
- **Independencia:** no requiere árbol de secciones ni candidata A.
- **No objetivos:** copiar almacenamiento plano, desplegar cloud o nuevo consenso.

## 3. Riesgos y defectos: nivel de evidencia

**Código:** P4 usa flush/fsync/replace por archivo, con limpieza del temporal.
`save_document` escribe árbol, páginas y metadatos en secuencia.
No se observa fsync del directorio ni atomicidad multiarchivo en ese tramo.
La consecuencia tras pérdida de energía es una **hipótesis**, no fallo reproducido.

P4 documenta que Windows carece del mutex `fcntl` y mantiene best-effort.
Es una limitación explícita pertinente al entorno del propietario; no afirmar
que toda operación de PageIndex sea insegura o que todo uso tenga concurrencia.
La lectura JSON convierte ciertos errores en ausencia y avisa ante JSON ilegible.
La caché de manifiesto silencia `OSError` al escribirse; existe recuperación
parcial por recorrido de documentos, no una garantía integral de restauración.

P2 comprueba límites externos cuando hay start/end, pero ese fragmento no
establece por sí solo orden start<=end ni validez completa de todos los nodos.
No se acredita un productor público que emita rangos invertidos: no se eleva
la observación a bug confirmado.
Frontera de confianza: PDF/estructura derivada/modelo → referencias persistidas;
el texto y los títulos deben seguir siendo datos, no permisos de ejecución.

## 4. Prerrequisitos Vanta

1. Revalidar qué spans, revisiones, receipts y generaciones existen hoy.
2. Mantener provenance y supersession; una jerarquía no reemplaza esos contratos.
3. Confirmar semántica de batch y marcadores de salud antes de proponer B.
4. Separar extracción opcional con modelos del núcleo determinista.
5. Preservar presupuesto compartido y acceso autorizado en expansión/citas.
6. Considerar Windows y crashes en el contrato futuro, sin afirmar resultados.

## 5. Qué no trasladar

No copiar el lock POSIX ausente en Windows como política del motor Rust.
No adoptar JSON multiarchivo como sustituto de WAL/backend.
No usar IDs renumerados como identidad durable de cita.
No importar un ciclo de agente completo sólo para navegar secciones.
No convertir resultados de QA financiera en promesas de recall general.
No cambiar defaults ni diseñar prompts, sampling o configuraciones de evaluación.

## 6. Método, licencia y cobertura

Lectura inicial estática offline de rutas P0-P7; ampliación pública en §11.
Sin instalaciones, scripts de repos, builds, tests, servicios, benchmarks,
commits ni push.
El manifiesto fija clone superficial, sin tags, historial restante ni LFS.
No auditados: algoritmos Flash completos, parser carácter por carácter,
MCP/cloud, chat local completo, límites de recursos PDF, OCR y red end-to-end.
Las celdas fuente P6 se leyeron como JSON, no como resultados reproducidos.

MIT raíz [PL] coincide con inventario; conservar avisos al reutilizar.
Datasets, PDFs NVIDIA y modelos tienen condiciones separadas no certificadas.
Una adaptación de ideas a Rust no autoriza copiar material sin revisar licencia.
§11 añade issues efectivamente leídos; no se atribuyen CVEs ni fixes sin prueba.

## 7. Runner benchmark encontrado y realmente leído

**P6:** `cookbook/pageindex-vs-vector-10k/pageindex-vs-vector-10k.ipynb`.
Se leyeron todas sus celdas de código: setup, descarga/verificación PDF,
preguntas, collect, ingesta, embeddings, respuestas, grading y cómputo de coste.
Entradas: cinco NVIDIA 10-K FY2022-2026 con SHA256 fijados en el notebook;
63 preguntas con referencia/key points en JSON; URL de preguntas apunta a main.
La fijación de PDFs no fija automáticamente la versión descargada del JSON.

Baseline: chunks 512 tokens/solape 64, normalización NumPy y producto escalar;
top-k 10/30/50/100. **No es HNSW ni el híbrido de Vanta.**
PageIndex consulta la biblioteca con `doc_id=None`, hasta 20 turnos.
Strings declarados: `openrouter/openai/gpt-6-sol`, `gpt-6-luna`,
`openrouter/google/gemini-embedding-2`, juez `claude-opus-5.5`.
Son valores del archivo; disponibilidad/precios actuales no se verificaron.

Métricas existentes: accuracy de cobertura total sin contradicción según juez
y USD estimados por 1 000 consultas mediante tabla de tokens/precios.
No son recall de motor, latencia p99, RSS, coste total de ingesta o calidad humana.
Dependencias: pageindex, tiktoken, PDFium, NumPy, requests/LiteLLM y clave/red
OpenRouter; outputs JSONL y cache de vectores locales.
`collect` reutiliza filas por ID y archivo: no versiona configuración completa.
Cambiar modelo/corpus conservando artefactos puede mezclar resultados: riesgo
de reproducibilidad inferido del código, no corrida defectuosa observada.
El coste medio usa respuestas presentes; accuracy usa total de preguntas.
Es importante no interpretar ambas poblaciones como idénticas tras errores.

**FinanceBench 98.7%** es una declaración README que remite a Mafin2.5 [P7].
No se atribuye a este runner NVIDIA; no se localizó su runner FinanceBench
dentro de la cobertura recorrida. Tampoco se leyó como runner el notebook
multi-embedding: sólo se localizaron referencias, sin atribuirle resultados.
No se reproducen como propios los outputs guardados.
Comparación futura: [protocolo existente](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz Vanta

| Capacidad | Estado | Delta condicionado |
|---|---|---|
| ANN/BM25/sparse/RRF | Existe | Árbol complementario, no sustitución |
| Provenance/sesiones/supersession/dedup | Existe | Citas ligadas a revisión |
| Ingestors/checkpoints/scheduler/auditoría | Existe | B no crea otra infraestructura |
| Assembler/compaction | Existe; V2 al pin 97c9623 | Consumir secciones con presupuesto común |
| Spans PDF exactos y mapping de IDs al reindexar | No acreditado aquí | Inventario previo a A |
| Garantía uniforme de generación documental | No acreditado aquí | Inventario previo a B |
| Integración A/B en todos los ingestors | Parcial: seams identificados | No trazado vigente completo |
| Servicio cloud PageIndex como parte del motor | No aplicable | Dependencia externa opcional del host |

## 9. Carencias sugeridas, no bugs Vanta

El valor diferencial es comprobar si una cita conserva documento, revisión y
coordenada, y si el operador ve la generación efectiva de sus derivados.
No se deduce que Vanta carezca de provenance, ingesta o publicaciones coherentes.
Priorizar A cuando se necesite inspección documental; B sólo si el contrato
actual no expresa una publicación que ya ocurre. No hay migración aprobada.

## 10. UI y escritorio

README anuncia **PageIndex App** en `app.pageindex.ai` [PA]: evidencia externa.
La cobertura local contiene APIs, árbol y notebooks; no se inspeccionó código
frontend de esa App ni se acredita escritorio nativo.
Notebook interactivo no equivale a aplicación de escritorio mantenida.
Studio ya existe: spans y generaciones serían datos de su inspector actual.
Runtime requerido: **ninguno**; no iniciar ni reiniciar procesos.

[P0]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/requirements.txt#L1-L16
[P1]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/local_api.py#L88-L178
[P2]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/local_api.py#L219-L286
[P3]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/tree_optimize.py#L172-L266
[P4]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/local_store.py#L1-L186
[P5]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/flash/api.py#L1-L100
[P6]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/cookbook/pageindex-vs-vector-10k/pageindex-vs-vector-10k.ipynb#L91-L630
[P7]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/README.md#L163-L176
[PA]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/README.md#L37
[PL]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/LICENSE#L1-L21
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L1-L65
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L228-L349
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/backend.rs#L1-L65

## 11. Ampliación de cierre: flujos, contratos y fuentes públicas

**Autoría entregada, pendiente de revisión independiente y aceptación.**
Lectura 2026-11-07 del pin de §1; se conservan A/B y el runner ya investigado.
No se toma Archipelago ni otro proyecto como implementación base de PageIndex.

### 11.1. Mapa de componentes y tres recorridos

`client.py` es fachada local/cloud y normalizador de chat/citas;
`local_api.py` indexa/lee; `local_store.py` persiste; `flash/` extrae layout;
`page_index_classic.py` construye TOC con modelos; `tree_optimize.py` combina y
expande; `agent_tools.py` expone navegación; `local_chat.py` adapta protocolos;
`mcp_bridge.py` es transporte cloud, `integrations/` adapters de frameworks;
`cookbook/` son notebooks, no frontend de App.
[pyproject.toml:1–60][P8] declara SDK 0.2.10, Python >=3.10, requests,
OpenAI, Agents, MCP, LiteLLM, PyPDF2/PDFium, regex, YAML y Pillow; Anthropic y
Claude Agent SDK son extras. El notebook dice probado con 0.4.1: **no prueba
que sus resultados guardados correspondan al SDK de este checkout**.
Los caps/comentarios de dependencia no se tratan como advisory verificado.
MIT raíz cubre código propio; condiciones de proveedores/modelos, PDFs,
datasets y librerías transitivas se revisan aparte antes de adoptar material.

**F1 — construir árbol/publicar documento.** `LocalAPI.submit_document:88–201`
valida PDF/opciones, extrae páginas PyPDF2, ejecuta standard/Flash, valida bounds
y sólo entonces asigna UUID `pi-...` y guarda meta completed bajo store lock.
Standard: `tree_parser` → `meta_processor` selecciona TOC con/sin números o
sin TOC, verifica índices, intenta reparación y cambia de estrategia; divide
nodos grandes recursivamente y añade introducciones [P9:1134–1247].
Flash: `extract_toc:124–155` parsea por página con PDFium, procesa geometría
y bloques; el cierre ensambla outline y puede incorporar bookmarks como
detected/bookmarks/hybrid. Optimización es una fase posterior [P10].
`optimize:925–1004` alterna merge/expand, congela decisiones para evitar
oscilación y renumera con `id_map`. Su coste estructural no es latencia medida
ni calidad QA [P11]. Fallo de modelo/estructura se propaga, no publica meta sana.

**F2 — navegar/leer/responder.** `agent_tools.py::_tool_specs:1524–1570`
usa bridge live para cloud y subconjunto local para biblioteca local.
Local: browse → `_resolve_document:401–465` por nombre entre IDs permitidos
→ `_get_document_structure:910–1008` (árbol sin texto, resúmenes y partes)
→ `_get_page_content:1011–1127` (páginas concretas) → answerer del host.
La estructura orienta selección; el contenido de página es evidencia distinta
del resumen. El umbral de guidance 20 páginas y `_CHAR_BUDGET=95000` son
defaults upstream, **no configuración propuesta para Vanta** [P12–P13].
La primera página de contenido puede superar el cap por la condición
`if not included`; cap por respuesta no implica budget total de conversación.
La salida enumera páginas devueltas/restantes/fuera de rango.
`local_chat.py:1686–1747` usa Agents `Runner.run`, convierte errores y añade
usage. `_chat_events_agen:1434–1501` cancela al abandonar y cierra backend en
finally. El bridge síncrono `_stream_sync:162–227` señala cancelación y espera
teardown; no prueba que proveedor remoto deje de consumir [P14].

**F3 — reemplazar/borrar/reconstruir.** En local `submit_document` crea nuevo ID
y nombre uniquificado, no actualiza una revisión immutable del mismo ID.
`delete_document:375–378` delega a DocStore; éste elimina `doc.json` primero,
retira directorio best-effort y actualiza manifest. Quitar visibilidad antes
del resto deja posible residuo, no rollback ni purge certificado [P4].
Lectura depende de meta existente. `save_document` publica tree, pages y meta
con replace separado; no hay commit multiarchivo. Lock protege name+save en
POSIX pero no Windows. Lectura concurrente, error de permiso y crash tienen
contratos diferentes: no atribuir atomicidad a `os.replace` más allá del archivo.
Rebuild de estructura/IDs exige remap de citas o nueva revisión explícita;
`id_map` de optimización no convierte ordinal en identidad durable.

### 11.2. A/B completadas: contratos y migración, sin duplicar Vanta

Los casos/fuentes/deltas/acciones/dependencias de §2 se conservan.

| Campo contractual | A — intervalo/cita | B — publicación de generación |
|---|---|---|
| Alternativas | Extender `CandidatePage.sources`/metadata, adapter de PDF o sólo página sin offset; rechazar nuevo parser general | Extender estado/checkpoint/backend actuales; adapter con generación externa; rechazar nuevo docstore o WAL |
| Rust **propuesto**, entrada/salida | `resolve_span(source_id, expected_revision, span, RecallScope) -> Result<ResolvedSpan, SpanError>`; salida texto autorizado + intervalo + revisión | `publish_generation(source_id, expected_revision, artifacts, operation_id) -> Result<GenerationPublication, PublicationError>`; salida generación visible, proyecciones pendientes y referencia de evidencia |
| Errores/invariantes | `Denied`, `InvalidInterval`, `MissingSource`, `StaleRevision`, `ReadFailed`; página física/etiqueta/offset nunca intercambiables; cita no acredita entailment | `RevisionConflict`, `IncompleteArtifacts`, `StorageFailed`, `Cancelled`, `ProjectionPending`; commit primario distinto de búsqueda completa |
| Seam real | `ingest/merge.rs::{CandidatePage,commit}:18–30,239–289`, `src/wiki/ingestors.rs::Ingestor:45–89`, `assemble_with_recall` [V1–V2] | `utils/task_checkpoint.rs::TaskCheckpointManager::{begin,advance,fail}`, `src/wiki/state.rs::WikiState`, merge existente; no transacción global asumida desde backend [V3] |
| Persistencia/rollback | Campo aditivo unidad/revisión; legacy unknown, backfill sólo con original verificable; desactivar resolución exacta sin borrar provenance | Preparar artefactos → verificar → registrar revisión esperada → publicar marcador existente si cubre unidad → proyectar → diagnosticar. Legado sin generación no se “certifica”; rollback impide nueva producción, preserva lectura/reparación |
| Owner/cancel/concurrencia | Reader posee revisión por llamada; cancela sin mutación. Renderer conserva identidad al recortar; actualización posterior no cambia cita ya emitida | Worker actual posee operación; modelo fuera de locks; CAS/serialización en seam existente. Cancel prepublicación abandona staging; postcommit devuelve aplicado/parcial, nunca rollback supuesto |
| FMEA/detección | Cache permutada → posición sustituye identidad → página incorrecta; validar cobertura/unicidad/page_index como lectura, no rehacer fuente automáticamente | Crash entre artefactos → mezcla generaciones → cita falsa; detectar descriptor/digests y dejar unavailable/reparable, no meta completed ficticia |
| Aceptación futura/observabilidad | Normal introducción/página compartida; adversa cache duplicada, rename, stale, Unicode, ACL. Log IDs/revisión/unidad/omisiones sin texto | Normal publicación completa; adversa dos writers, crash/cancel en frontera y error IO; phase/revisión/operation/conteos sin payload |
| Mínimo/abierto | Una unidad fuente y parser existente; no OCR nuevo; exactitud de offset/retención por decidir | Un ingestor/backend; no coordinador distribuido; unidad real de publicación y retención de staging abiertas |

`WriteReceipt` ya está implementado en [attestation.rs:654–738][PV4]:
status recorded, versión y SHA256 sobre una proyección, **no firma digital**.
No cubre automáticamente spans/linaje/proyecciones ni entrega al LLM.
Un `MutationReceipt` multi-etapa sería **propuesto**, no ausencia general de
recibos. B debe usar evidencia existente y añadir sólo semántica no expresada.

### 11.3. C — navegación documental estructurada como adapter de recall

1. **Problema agente:** encontrar sección dentro de un informe largo sin volcar
   todo el PDF ni asumir que top-k de fragmentos captura su introducción.
2. **Unidad/fuente:** browse de documento → árbol paginado → rangos dirigidos
   [P12–P13], con lectura de `_get_document_structure`/`_get_page_content`.
3. **Delta:** Vanta ya recupera y ensambla; añadir opción de explorar secciones
   ligadas a evidencia, no sustituir BM25/HNSW/RRF por un razonador.
4. **Alternativas:** navegación determinista desde hits existentes, herramientas
   del host sobre árbol importado o selección LLM opt-in; rechazar loop en core.
5. **Rust propuesto:** `list_sections(source_id, revision, cursor, scope) ->
   Result<SectionPage, NavigationError>`; `read_section(section_ref,
   revision, remaining_budget, scope) -> Result<SectionEvidence, ...>`.
   Cursors ligados a revisión; salida incluye returned/omitted y padre/rango.
   `Denied`, `UnknownCursor`, `StaleRevision`, `MissingSection`, `ReadFailed`,
   `BudgetExhausted`; nombre/path descriptivo no reemplaza ID autorizado.
6. **Seam:** `src/wiki/ingestors.rs::Ingestor`, `CandidatePage.sources` alimentan
   estructura; `auto_recall.rs::{RecallScope,perform_auto_recall_governed}` y
   `context_engine/engine.rs::assemble_with_recall` consumen evidencia.
7. **Pasos:** mapear A → importar árbol con B → validar ciclos/rangos/cobertura
   → listar sin texto → resolver sección bajo ACL → render existente. Persistir
   árbol como derivado con fingerprint, no duplicar fuente; migración aditiva,
   rebuild sólo desde revisión identificada. Rollback apaga adapter, no fuentes.
8. **Dependencias:** C4 identidad/A, C14 frescura/B y C7 budget; C16 inspector.
   Tripletas/ontología no son requisito para una jerarquía documental.
9. **FMEA:** homónimo/path truncado → documento equivocado → cita cruzada;
   detectar resolución no única y rechazar. Árbol cíclico/hub → loop/coste;
   validar estructura y consumir presupuesto ya existente, no un scheduler nuevo.
10. **Aceptación futura:** normal árbol→página correcta; adversa nombres iguales,
    cursor viejo, padre sin texto, revisión retirada, cancel entre lecturas.
    Observar fuente/revisión/nodos/bytes omitidos, no títulos ni contenido.
11. **Mínimo/no-goals/abierto:** read-only sobre un documento; no crear carpetas
    globales, permisos por path ni MCP cloud obligatorio. Política de selección
    del host y formato de árbol interoperable abiertos.

### 11.4. D — presupuesto de ejecución de navegación/modelo en el host

1. **Problema:** un agente respeta tokens de contexto pero sigue gastando turnos,
   retries y tiempo de herramientas después de que el caller abandone.
2. **Fuente/unidad:** `local_chat.py::_run_kwargs:1052–1059`,
   `_chat_events_agen:1434–1501`, `_stream_sync:162–227` [P14]; #569 leído
   distingue `max_turns` de deadline. Se extrae el contrato, no una solución
   a deadline que no está acreditada implementada en el pin.
3. **Delta:** presupuesto de `assemble_with_recall` es de contexto; no prueba
   límite global de red/modelo. Mantener ambas unidades explícitas.
4. **Alternativas:** extender budget/cancel del host/adapter actual, caller que
   controla cada llamada o rechazar navegación LLM; no introducir Tokio paralelo.
5. **Rust propuesto:** `navigate(request, run_context) -> Result<NavigationRun,
   NavigationRunError>`; run_context lleva identidad/revisión, cancelación y
   remanente del host, salida evidencia + uso observado/completitud/estado.
   `DeadlineExceeded`, `Cancelled`, `ProviderFailed`, `UsageUnknown`,
   `StaleRevision`; agotamiento impide llamadas nuevas, no promete cancelar pago.
6. **Seam:** `vanta-memory/src/ingest/worker.rs::{begin,execute}`,
   `TaskCheckpointManager` cuando es ingesta; en recall usar caller de
   `perform_auto_recall_governed`, no persistir estado de ejecución como memoria.
7. **Pasos/persistencia:** inventariar cancel/budget existentes → propagar remanente
   a adapter → finalizar recursos → emitir resultado parcial con razón. Lectura
   sin migración del primario; diagnóstico opcional con schema versionado.
   Rollback apaga navegación modelo, baseline híbrido sigue utilizable.
8. **Dependencias:** C7 distingue contexto/tiempo/coste, C9 owner/cancel,
   A/C4 fijan revisión; no bloquear C determinista ni ingesta sin LLM.
9. **FMEA:** timeout de request confundido con final de run → tarea sigue →
   coste/threads; detectar llamadas tras cancel y teardown pendiente; propagar
   señal al owner y no marcar concluido antes de conocer su límite efectivo.
10. **Aceptación futura:** normal navegación acotada; adversa modelo lento,
    herramienta bloqueada, abandono de stream y fallo de cierre. Observar
    turnos/tiempos/uso faltante/cancelación sin pregunta, respuesta ni secretos.
11. **Mínimo/no-goals/abierto:** adapter opt-in, un protocolo; no copiar Agents,
    nuevo runtime, prompts o políticas de precios. Límite de cancelación de
    proveedor y contrato de uso parcial deben explicitarse antes de implementar.

### 11.5. Incidencias públicas leídas: no todas son defectos acreditados

Consulta 2026-11-07. `websearch.search` DDG/Bing con
`site:github.com/VectifyAI/PageIndex/issues tree page index error` dio vacío
y Bing 301; fallback real `fetchWebContent` al listado oficial y cada issue
siguiente devolvió cuerpo y actividad sin truncamiento.

| Incidencia/estado observado | Revisión, causa y fix acreditados | Aplicación y discriminación |
|---|---|---|
| [#648](https://github.com/VectifyAI/PageIndex/issues/648), Open, creada 2026-11-04 | Reporte sobre `646deeb...`, cache construida permutada/duplicada. En pin local `_load_tree_with_text:290–297` aún usa posición; `_require_pages:342–356` no valida cobertura. Comentario ofrece arreglo, no merge probado | A valida page identity. No se afirma que writer normal permute caches; comprobar lectura de cache anormal sin modelo |
| [#561](https://github.com/VectifyAI/PageIndex/issues/561), Open, 2026-10-11 | `DocStore.lock` sin fcntl no bloquea; fuente P4 conserva mecanismo. #567/#593 aparecen vinculadas; no se leyeron diffs/merge | B no importa ese lock a Windows ni niega locks propios Vanta; discriminar writers mismo nombre/proceso/directorio |
| [#637](https://github.com/VectifyAI/PageIndex/issues/637), Open, 2026-10-26 | Miembro anuncia cambio de MCP a library path **no desplegado todavía** en el cuerpo. Pin usa doc_name; `_resolve_document` elige newest y targeting imprime nombre | C4: path no es identidad y ambigüedad debe ser visible. No afirmar API path local ya implementada; #632 sólo referencia |
| [#569](https://github.com/VectifyAI/PageIndex/issues/569), Open, 2026-10-12 | Reporte sobre 0.2.22 pide deadline; #575/#580 vinculadas, propuesta de timeout no merge acreditado. Pin leído tiene max_turns y cancel de stream | D distingue timeout request/run/turnos; no reproducido ni transferido como bug Vanta |
| [#585](https://github.com/VectifyAI/PageIndex/issues/585), Open, 2026-10-14 | Reporta ejecución vía default de schema. Pin `agent_tools.py:1422–1455` valida identificadores y usa `!r`; el string de su supuesto repro queda literal, no acredita breakout. No hay cadena demostrada documento→schema hostil | **Hipótesis de exploit descartada con esta evidencia**, no CVE. Sí revisar fuente de schemas/objetos no JSON antes de portar; #597 enlazada no acredita fix |

### 11.6. Benchmark, matriz y UI sin extrapolación

Complemento de §7 [P6]: `collect` (celda 10) reintenta filas con `error` pero
`done` se construye por ID, resultado final por `(id,system)`; un fallo sin
captura en vector/judge puede abortar el pool. No omitirlo del ledger de intento.
Celda 20 baraja respuestas por `random.Random(q["id"])`, juez recibe gold,
reintenta JSON y exige cobertura de todos los key points sin contradicción.
Eso no es fuga al retrieval por sí mismo ni recall@k. Celda 22 divide aciertos
por `len(QUESTIONS)` y costes sólo por respuestas; si no hay ninguna puede
fallar la división. Cache discount sólo PageIndex y tabla de precios fija:
coste de ingesta/juez/infra no incluido en ese USD/1000.
No reset automático acreditado: reutiliza biblioteca, PDFs, `vectors.npy`,
answers/grades JSONL; mezclar configuración con cache conservada altera ensayo.
Adapter Vanta futuro: mapear pregunta a recall autorizado, IDs→filing/páginas
y contexto→answerer del host; preservar gold sólo para juez, population y uso.
RRF actual no se etiqueta como el baseline NumPy. No nuevo harness/scoring.

| Capacidad | Upstream | Vanta pin 97c9623: fuente/estado | Delta/decisión |
|---|---|---|---|
| Recuperación | Navegación por árbol/P12 | Existe híbrido: `src/sdk/search/{hybrid,fusion}.rs` | C complementa, no reemplaza HNSW/BM25/RRF |
| Fuente y sesiones | PDF/meta/UUID, no memoria temporal de agente | Existe `MemoryRecord`, provenance/sesiones y `core/record/{l1_dedup,lifecycle}.rs` | A añade coordenada/revisión, no dedup nuevo |
| Budget y cuotas | Cap por tool y max_turns | Existen `assemble_with_recall`, `apply_recall_budget`, `src/config.rs:760–762` | D separa ejecución; chars/tokens/bytes no equivalentes |
| Lifecycle | Save terminal; delete sin revisión | Existen worker, scheduler/checkpoints, ingestors y progreso | B integra C9; coherencia global no acreditada, no negada |
| Receipt/derivados | JSON/manifest, no cert. global | Existe WriteReceipt PV4; parcial para derivados | C14 + B diagnostican, no WAL nueva |
| UI | App SaaS anunciada/cliente SDK | Existe Studio PS1; jobs no acreditados al pin | C16 reutiliza inspector; cancel/resultado por chunk sólo propuestos |

Studio, cuerpo releído con `git show` del pin separado
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`: ruta
`feature-freez/desktop/src/components/memory/MemoryLens.tsx` [PS1].
`ScenesPanel:57-96`, `PersonaPanel:172-206`, `SkillsPanel:266-297` y
`GenlogPanel/openAnchor:373-412` cargan escenas/snapshot/skills/genlog y
abren registro real vía `get(anchorId)`. **PI-1:** JobChip/jobs no están en
ese árbol; son `??` en `desktop/` del worktree, no bytes acreditados del pin.
Se retiran PS2/PS3 y la supuesta garantía de cancelación entre chunks.
Árbol/spans A/C y generación B serían vistas **propuestas** en Studio existente.
FMEA: cancel visual interpretado como retirada → cita a derivado aplicado;
resolver fuente/revisión autorizadas desde owner, no desde color/UI.
Aceptación futura: cancel antes/después de publicar, error parcial y restart
no ocultan IDs aplicados ni presentan árbol anterior como revisión vigente.
No se acredita persistencia de jobs ni render. App PageIndex es SaaS externa anunciada;
no se leyó su frontend ni se acredita escritorio nativo desde un SDK/MCP.
**Negativos Vanta:** no hay bug nuevo probado; quedan no acreditados end-to-end
spans exactos y publicación de todos los derivados. Se descartan ausencia de
budget/scheduler/receipts y el traslado de defectos históricos como actuales.

**Cobertura y continuidad:** leídos drivers Flash/classic/optimización, tools,
store, chat focal, notebook y cinco issues; no parser completo, todas las lanes,
auth SaaS, PRs enlazadas ni licencias transitivas. Sin apps/tests/builds/modelos.
Si pasa a specs (>300 líneas), separar A/C navegación/citas, B generación y D
adapter de ejecución; conservar aquí arquitectura, decisiones y fuentes.
No se crean specs ni se autoacepta la investigación.
PI-1 rectifica procedencia/fuente/UI y matriz; no niega Studio ni certifica
garantías de código dirty que aquí no se ha incorporado como evidencia.

[P8]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pyproject.toml#L1-L60
[P9]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/page_index_classic.py#L1134-L1247
[P10]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/flash/main.py#L124-L155
[P11]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/tree_optimize.py#L925-L1004
[P12]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/agent_tools.py#L910-L1008
[P13]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/agent_tools.py#L1011-L1127
[P14]: https://github.com/VectifyAI/PageIndex/blob/2ac8694bf8fe89a3840dda9bdb88db5119b77fc4/pageindex/local_chat.py#L1434-L1501
[PV4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L654-L738
[PS1]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L57-L412
