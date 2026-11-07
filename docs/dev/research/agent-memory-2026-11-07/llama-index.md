---
title: "LlamaIndex: caché de transformación y expansión jerárquica verificable"
kind: research
status: proposed
description: "Análisis estático de ingesta, memoria y retrieval de LlamaIndex con extracciones acotadas para VantaDB"
tags: [research, memory, engineering]
---

# LlamaIndex: caché de transformación y expansión jerárquica verificable

## 1. Recomendación y arquitectura

**Dictamen:** extraer contratos de reutilización de transformaciones y de
expansión documental; no hacer del framework Python una dependencia del motor.
Clone leído: `C:\Users\Eros\llama_index`.
SHA congelado: `444fdf0684707de62ebb447d658dba137dd9cb99`.
Snapshot: **52 837 estrellas; MIT**, del [inventario](github-candidates.json).
La preparación y omisiones proceden del [manifiesto](clone-manifest.json).
No se consultaron contadores ni servicios externos.

El paquete central declara Python >=3.10, Pydantic, SQLAlchemy, fsspec,
tokenización, clientes HTTP y workflows; [L0] acredita dependencias, no su
utilización completa. Las integraciones viven separadas del núcleo.
Es una capa de aplicación que conecta documentos, transformaciones,
docstore, vector store y modelos; no un almacenamiento embebido equivalente.

```text
documentos/nodos → estrategia docstore → transformaciones/cache
               → vector_store.add → actualización docstore
query → vector_retriever → completar vecinos → sustituir hijos por padre
      → repetir mientras cambie → ordenar por score
memoria → contar historia/bloques → truncar por prioridad → formato de contexto
```

El primer flujo es visible en [L1]-[L3]; el segundo en [L4]-[L5].
La caché evita recomputación, pero no vuelve transaccionales los almacenes.
La expansión cambia el contenido recuperado: un padre no es otro hit ANN.
La memoria por bloques [L6] constituye política de contexto, no persistencia
semántica autónoma ni sustituto del ensamblador de Vanta.

**Referencia Vanta:** base de investigación `c561158253bb88192590fa406c61e0a27dd7df8d`;
HEAD concurrente comunicado por el propietario: `167435a9e`.
Los seams citados son históricos y acotados, no una auditoría renovada.
El [contrato](METHOD.md#vantadb-como-referencia) y el propietario acreditan ANN,
BM25/sparse/RRF, provenance, sesiones, supersession, dedup, assembler/compaction,
scheduler/checkpoints/ingestors/auditoría y Studio en `develop-desktop`.
Nada de ello se propone como capacidad inexistente.

## 2. Evidencia y extracciones mínimas independientes

### Registro de lectura

| Ref. | Ruta y rango efectivamente leídos | Evidencia |
|---|---|---|
| L0 | `llama-index-core/pyproject.toml:35-87` | Stack y dependencias centrales |
| L1 | `core/ingestion/pipeline.py:46-110` | Hash de entrada/transformación; get/put de caché |
| L2 | `core/ingestion/pipeline.py:451-507` | Duplicados y upsert con borrado previo |
| L3 | `core/ingestion/pipeline.py:580-662` | Estrategia efectiva, workers, vector store y docstore |
| L4 | `core/retrievers/auto_merging_retriever.py:35-164` | Relación padre/hijos y relleno de vecino |
| L5 | `core/retrievers/auto_merging_retriever.py:166-194` | Iteración hasta estabilizar y orden final |
| L6 | `core/memory/memory.py:481-545` | Truncamiento por prioridad y bloques protegidos |
| L7 | `core/evaluation/benchmarks/beir.py:1-109` | Runner BEIR leído completo |
| [L8] | `README.md:11-32` | Producto externo y composición |
| [LL] | `LICENSE:1-21` | MIT raíz y aviso |

Las rutas abreviadas `core/` están bajo
`llama-index-core/llama_index/`; enlaces inmutables al final.
La tabla no acredita módulos enteros ni las integraciones no recorridas.

### Candidata A — identidad y recibo de transformación

- **Problema:** distinguir un resultado reutilizado de otro recalculado,
  manteniendo identidad de fuente y versión efectiva de procesamiento.
- **Unidad mínima:** descriptor determinista de transformación + clave
  contextualizada + recibo de hit/miss, inspirado en L1.
- **Seam existente:** ingesta/merge de Vanta [V1], que ya agrupa por página,
  conserva `sources` y comunica fallos por entrada.
- **Contrato hipotético, no API acreditada:** `TransformIdentity { source_digest,
  processor_version, options_digest, scope }` y `TransformReceipt { identity,
  reused, output_digest }`.
- **Acción 1:** inventariar identidad/checkpoints actuales antes de añadir tipos.
- **Acción 2:** separar opciones que alteran contenido de opciones de ejecución;
  no serializar punteros, secretos ni un `Debug` inestable como identidad.
- **Acción 3:** incorporar el receipt al resultado de ingesta existente;
  una caché fallida debe conservar el error o la recomputación explícita.
- **Dependencias:** hash estable, serialización canónica, namespace, versión del
  extractor/modelo si participa y política de invalidación.
- **Persistencia:** reutilizar backend/checkpoint; no crear otro docstore.
- **FMEA — identidad incompleta:** dos configuraciones comparten clave y sirven
  contenido obsoleto; evitarlo haciendo explícitos todos los insumos semánticos.
- **FMEA — ámbito omitido:** cache hit puede revelar contenido de otro namespace.
- **FMEA — publicación parcial:** receipt exitoso antes del contenido durable
  convierte un retry en falso «ya procesado».
- **Aceptación propuesta:** misma identidad implica el mismo artefacto validado;
  cambio semántico invalida; fallo no publica receipt de éxito.
- **Aceptación de aislamiento:** ningún hit permite saltarse autorización o
  reutilizar provenance incompatible.
- **Independencia:** no depende de jerarquías, ANN ni de la candidata B.
- **No objetivos:** caché distribuida, nuevos modelos, nuevos parámetros de eval.

### Candidata B — expansión contextual con trazabilidad de sustitución

- **Problema:** explicar por qué un fragmento recuperado se amplió a su sección
  y qué evidencia quedó representada sin duplicar contenido en el presupuesto.
- **Unidad mínima:** relación fuente/fragmento/padre y registro de sustitución;
  tomar L4/L5 como contraste, no sus ratios ni media de scores.
- **Seam existente:** assembler puro y `AssembleOutput` de Vanta [V2];
  `ChatMessage`/`CompactionReport` ya dan contratos tipados [V3].
- **Contrato hipotético:** `ExpansionDecision { source_revision, selected_ids,
  replacement_id, reason, provenance }`, unido al reporte existente.
- **Acción 1:** comprobar qué jerarquía/secciones conserva ya el ingestor.
- **Acción 2:** resolver referencias dentro de la misma revisión y namespace.
- **Acción 3:** proponer expansión como paso opcional anterior al consumo del
  presupuesto compartido; el assembler sigue decidiendo qué cabe.
- **Acción 4:** conservar IDs originales y motivo; no presentar score agregado
  como similitud ANN medida del padre.
- **Dependencias:** padres direccionables, revisión de fuente, límites existentes
  del contexto y garantía de que cada relación proviene de entrada validada.
- **FMEA — ciclo:** padre que apunta a descendiente puede impedir convergencia;
  contrato de árbol validado y visita única, no confianza en JSON importado.
- **FMEA — pérdida de precisión:** un padre autorizado puede contener secciones
  no pertinentes; expansión no debe eliminar el acceso al fragmento original.
- **FMEA — mezcla de revisiones:** referencia vieja expande a documento distinto.
- **Aceptación propuesta:** decisiones reproducibles, sin duplicados padre/hijo,
  respetando presupuesto, permisos y pares tool-call/result ya protegidos.
- **Aceptación negativa:** relación no resoluble no inventa texto ni provenance;
  se conserva el fragmento o se comunica degradación.
- **Independencia:** funciona con documentos ya ingeridos, sin caché A.
- **No objetivos:** cambiar ranking híbrido, importar retriever Python o copiar
  el `simple_ratio_thresh=0.5` observado.

## 3. Riesgos, defectos e hipótesis

**Hechos estáticos:** L1 construye el hash concatenando contenido y una
representación textual del transformador; elimina patrones de direcciones.
Eso describe su implementación, no una especificación canónica interoperable.
L2 borra versiones anteriores en docstore/vector store antes de L3.
L3 añade vectores y después actualiza docstore; no hay transacción conjunta
visible en ese tramo. No se afirma que todo backend carezca de transacciones.

**Hipótesis discriminantes:** una excepción entre pasos puede dejar visibilidad
parcial; un retry de ingesta necesita reconocer qué paso ocurrió.
No se reprodujo el fallo ni se consultaron issues upstream.
L4 calcula score del padre promediando hijos y registra texto truncado en logs.
Es una frontera de privacidad real de ese código; la exposición concreta
depende del contenido sensible y de quién pueda leer esos logs.

L6 preserva prioridad cero y puede devolver contenido aún sobredimensionado
cuando sólo quedan bloques protegidos. No confundir esa política con prueba
de que el proveedor aceptará el contexto final.
No se certifica un defecto de Vanta por analogía con ninguna de estas rutas.

## 4. Prerrequisitos Vanta

1. Revalidar receipts, checkpoints y reportes actuales antes de abrir tareas;
   los commits concurrentes pueden cubrir ya A o B.
2. Preservar `sources`, supersession y dedup: caché no sustituye idempotencia.
3. Separar bytes originales, texto derivado e identidad de modelo/procesador.
4. Confirmar que expansión y compactación comparten presupuesto existente;
   no crear un segundo contador ni alterar protección de tool pairs.
5. Acordar cómo se expresa un paso fallido y qué garantía de durabilidad tiene.
6. Mantener diagnósticos sin contenido sensible por defecto.

## 5. Qué no trasladar

No importar Python/Pydantic/workflows al motor ni mantener dos pipelines.
No trasladar la concatenación textual como formato durable de identidad.
No adoptar borrado previo entre almacenes como contrato transaccional.
No promediar similitudes para cambiar el ranking sin justificar su semántica.
No reproducir logs con extractos de memoria del usuario.
No convertir la estrategia de bloques protegidos en promesa de cabida absoluta.

## 6. Método, licencia y cobertura

Lectura offline del checkout fijado; ninguna ejecución de código upstream,
instalación, build, test, benchmark, app, servicio o llamada de red.
Cobertura positiva: L0-L8 y seams históricos V1-V3; no auditoría exhaustiva.
Omitidos: agentes completos, conectores, callbacks, stores remotos, packaging,
seguridad de cada integración, notebooks y consistencia de backend end-to-end.
Historial shallow, otras ramas/tags, submódulos y LFS según manifiesto.

MIT de inventario concuerda con la licencia raíz leída [LL].
Copiar fragmentos sustanciales exige conservar aviso; revisar además archivos,
datos y dependencias concretos. Traducir a Rust no borra obligaciones.
Las propuestas anteriores describen contratos nuevos hipotéticos, no patches.
Los hallazgos históricos Vanta permanecen antecedentes, no bugs vigentes.

## 7. Benchmark upstream realmente leído

**L7 / `BeirEvaluator.run`: runner programático, no CLI autónoma.**
Descarga datasets BEIR si faltan; default `nfcorpus`, split `test`.
Carga corpus, queries y qrels mediante `GenericDataLoader`.
Recibe `create_retriever(documents)` y postprocesadores opcionales.
Conserva `doc_id` en metadatos y transforma hits en mapa documento→score.
Solicita NDCG, MAP, recall y precision para k=3/10 por defecto.
Esos valores describen el runner existente; aquí no se proponen ajustes.

Dependencias: paquete externo `beir`, tqdm, red/cache de datasets y las del
retriever elegido; modelos/embeddings dependen de esa función del llamador.
La deduplicación por clave de documento puede colapsar varios chunks.
El constructor contiene un `try: pass`; no comprueba realmente la importación
BEIR al construir el evaluador: los imports efectivos ocurren al usarlo.
No mide latencia de motor, p99, RSS ni recuperación de sesiones.
No implementa juez generativo en este runner.
La licencia del código no acredita licencia de corpus BEIR o modelos.

No se leyó como runner HotpotQA ni se atribuyen métricas a ese archivo.
ParseBench/ExtractBench aparecen anunciados en README [L8]; no se consideran
resultados de L7 ni evidencia ejecutada de este checkout.
Comparación futura remite al [protocolo](METHOD.md#protocolo-de-comparacion-de-benchmarks).
No se diseñaron prompts, jueces, thresholds, sampling ni configuraciones nuevas.

## 8. Matriz frente al estado existente

| Capacidad | Estado Vanta | Delta defendible |
|---|---|---|
| ANN/BM25/sparse/RRF | Existe, contrato del propietario/METHOD | No añadir otro retrieval básico |
| Provenance/sesiones/supersession/dedup | Existe | A debe conservar identidades existentes |
| Ingesta y checkpoints | Existe | Recibo/versionado de transformación por verificar |
| Assembler y compaction | Existe; V2/V3 histórico | B sólo como expansión explicable |
| Paridad exacta con cache key L1 | No acreditado | No implica necesidad de copiarla |
| Árbol documental importado validado | No acreditado en esta cobertura | Candidata B condicionada al inventario |
| Integración completa de A/B con todos los hosts | Parcial: seams, no trazado actual | Revalidación previa |
| Framework Python como requisito del motor | No aplicable | Mantener adaptadores fuera del núcleo |

## 9. Carencias sugeridas, no bugs certificados

El contraste plantea dos preguntas: ¿se puede explicar la reutilización de
una transformación y reconstruir la expansión de un hit hasta su fuente?
No demuestra que falten caché, jerarquías o compactación en Vanta.
Prioridad condicional: A si el operador pierde la versión efectiva de ingesta;
B si una ampliación ya existente no conserva decisiones y revisiones.
Una mejora puramente de reporte puede bastar; no se presupone nuevo subsistema.

## 10. UI, escritorio y evidencia externa

El README [L8] anuncia LlamaParse/LlamaCloud y enlaces de producto externos.
No se inspeccionó su frontend ni se acredita aquí una aplicación de escritorio.
El código leído ofrece APIs y logs, no evidencia de un dashboard incluido.
La búsqueda acotada no equivale a afirmar que no exista ninguna UI upstream.
Vanta ya tiene Studio Tauri/React en `develop-desktop`, según propietario.
A/B deberían alimentar su inspección de ingesta/contexto, no otra aplicación.
Runtime necesario para esta investigación: **ninguno**.

[L0]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/pyproject.toml#L35-L87
[L1]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/ingestion/pipeline.py#L46-L110
[L2]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/ingestion/pipeline.py#L451-L507
[L3]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/ingestion/pipeline.py#L580-L662
[L4]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/retrievers/auto_merging_retriever.py#L35-L164
[L5]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/retrievers/auto_merging_retriever.py#L166-L194
[L6]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/memory/memory.py#L481-L545
[L7]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/evaluation/benchmarks/beir.py#L1-L109
[L8]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/README.md#L11-L32
[LL]: https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/LICENSE#L1-L21
[V1]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/ingest/merge.rs#L1-L65
[V2]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/context_engine/engine.rs#L1-L70
[V3]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/context_engine/types.rs#L1-L65
