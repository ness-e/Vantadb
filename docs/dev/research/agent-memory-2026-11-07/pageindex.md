---
title: "PageIndex: intervalos documentales y publicación local de artefactos"
kind: research
status: proposed
description: "Lectura estática de PageIndex local, jerarquías de páginas y un runner documental sin extrapolar resultados externos"
tags: [research, memory, engineering]
---

# PageIndex: intervalos documentales y publicación local de artefactos

## 1. Recomendación y arquitectura

**Dictamen:** estudiar el contrato de intervalos/citas y la publicación de
artefactos derivados; no sustituir ANN/BM25/sparse/RRF por una consigna
«vectorless» ni introducir llamadas LLM dentro del motor embebido.
Clone: `C:\Users\Eros\PageIndex`.
SHA: `2ac8694bf8fe89a3840dda9bdb88db5119b77fc4`.
Snapshot: **40 160 estrellas, MIT**, del [inventario](github-candidates.json).
Preparación: [manifiesto congelado](clone-manifest.json); sin red en esta tarea.

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

**Vanta:** referencia histórica `c561158253bb88192590fa406c61e0a27dd7df8d`;
HEAD concurrente comunicado: `167435a9e`.
Se conserva el [baseline](METHOD.md#vantadb-como-referencia) y las capacidades
acreditadas por el propietario: retrieval híbrido, provenance/sesiones,
supersession/dedup, assembler/compaction, scheduler/checkpoints/ingestors,
auditoría y Studio Tauri/React en `develop-desktop`.
No se vuelve a auditar Vanta ni se reabren hallazgos antiguos como actuales.

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

Lectura estática offline de rutas P0-P7; sin instalaciones, scripts de repos,
builds, tests, servicios, benchmarks, red, commits ni push.
El manifiesto fija clone superficial, sin tags, historial restante ni LFS.
No auditados: algoritmos Flash completos, parser carácter por carácter,
MCP/cloud, chat local completo, límites de recursos PDF, OCR y red end-to-end.
Las celdas fuente P6 se leyeron como JSON, no como resultados reproducidos.

MIT raíz [PL] coincide con inventario; conservar avisos al reutilizar.
Datasets, PDFs NVIDIA y modelos tienen condiciones separadas no certificadas.
Una adaptación de ideas a Rust no autoriza copiar material sin revisar licencia.
No se revisaron issues externos ni se atribuyen CVEs o fixes históricos.

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
| Assembler/compaction | Existe; V2 histórico | Consumir secciones con presupuesto común |
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
[V1]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/ingest/merge.rs#L1-L65
[V2]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/context_engine/engine.rs#L1-L70
[V3]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/backend.rs#L1-L65
