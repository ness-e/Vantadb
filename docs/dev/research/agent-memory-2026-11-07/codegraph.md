---
title: "CodeGraph: evidencia revisionada y actualización de grafos por archivo"
kind: research
status: proposed
description: "Lectura estática de persistencia, frescura, contexto y viewer de CodeGraph para ideas independientes en Rust"
tags: [research, memory, engineering]
---

# CodeGraph: evidencia revisionada y actualización de grafos por archivo

## Identidad y dictamen

- Repositorio **`colbymchenry/codegraph`**, no otros productos llamados CodeGraph.
- Clon `C:\Users\Eros\codegraph`; SHA
  `f72c9d2efb14454ff4f9fd7b8460abd82f19a287`, rama `main`.
- Snapshot: **76 956 estrellas**, MIT según
  [candidatos](github-candidates.json) y [manifest](clone-manifest.json).
- Método [METHOD](METHOD.md); Vanta base `c561158`, HEAD comunicado `167435a9e`.
- El uso de CodeGraph como **herramienta del desarrollo** de Vanta no acredita
  que el producto Rust tenga un indexador AST equivalente. No se reaudita Vanta.

**Dictamen:** priorizar comprobación de la revisión que sustenta una respuesta
y publicación coherente de una generación por fuente. No portar el producto,
su catálogo de lenguajes/frameworks ni el viewer. El principio útil excede al
código: un contexto no debe presentar rangos de una revisión sobre bytes de otra.

## 1. Arquitectura, tecnologías, flujo y recomendaciones

**Documentación AGENTS:** biblioteca/CLI/MCP local-first en TypeScript,
extracción tree-sitter, SQLite/FTS5, referencias/resolución, grafo y contexto.
El clon contiene además kernel de extracción y UI Svelte; no se inspeccionó
el kernel completo ni se acredita fidelidad semántica de todos los lenguajes.

**Código del esquema [C1]:** nodos guardan nombre, tipo, fichero y posiciones;
aristas guardan extremos, tipo, ubicación y provenance; archivos guardan
content hash y fecha de indexado. Referencias no resueltas conservan estado
pending/failed. FTS5 tiene triggers de insert/update/delete de nodos.
No es un almacén de recuerdos conversacionales ni un grafo LLM de hechos.

Flujo de actualización acreditado en la muestra [C2][C3]:

1. `CodeGraph.sync` entra en mutex del índice y adquiere lock de archivo.
2. Reabre base reemplazada; si está recién recreada sin estado, devuelve
   contención para que el watcher pueda reintentar, no sincroniza sobre
   un handle que ya no corresponde al archivo publicado.
3. `storeExtractionResult` no sustituye evidencia válida por un resultado
   de grammar-load failure; trata markers de error separadamente del «sin cambios».
4. Antes de borrar nodos de archivo captura aristas entrantes desde otros
   archivos e identidades previas; la identidad incluye línea, por lo que
   una inserción de líneas puede requerir reemparejar.
5. Caso que cabe en chunk: borrar/guardar bundle/reattach dentro de transacción.
   Caso grande: inserciones por chunks y yields; **no** misma atomicidad global.
   No se leyó todo el resolver ni toda la rama posterior de publicación.

Contexto [C4]: busca semillas, obtiene subgrafo, bloques de código, archivos
y resumen; entrega Markdown/JSON o DTO. Tiene nota explícita de baja confianza
y cadenas de llamadas acotadas a nodos recuperados. Los caminos son evidencia
estática; no trazas de ejecución ni prueba completa de despacho dinámico.
El código leído marca algunos saltos sintetizados en el render.

Frescura [C5]: `validateAnswerFiles` compara hashes usados para responder con
archivos actuales bajo root, distinguiendo `stale` de `unchecked`.
Acota cantidad, bytes y tiempo. No se leyó su llamador completo: existencia
del helper **no certifica** enforcement de todas las herramientas MCP.

**Recomendación Vanta:** usar revisiones y estados de evidencia al ensamblar,
no copiar SQLite o un segundo índice. Separar extractor de código, dominio
de memoria y adaptadores de presentación. BM25/HNSW/RRF y provenance ya existen.

## 2. Unidades mínimas para diseño independiente en Rust

### Unidad A — evidencia de respuesta ligada a revisión verificable

- **Problema:** resultado indexado puede apuntar a líneas que ya contienen
  otro símbolo o texto; un deadline vencido no significa que la fuente esté fresca.
- **Fuente mínima:** `AnswerFile` y `validateAnswerFiles` [C5];
  `SourceRequest.ondrift` de la frontera UI [C9].
- **Seam Vanta:** provenance/ingestors y
  `vanta-memory/src/context_engine/engine.rs`; conservar revision al entregar
  evidencia y en el diagnóstico retrieval de Studio.
- **DTO HIPOTÉTICO Rust:** `EvidenceSlice { source_id, revision, range,
  content_digest }`; `Freshness` distingue verified, stale y unchecked.
  No se afirma que estos sean tipos actuales de Vanta.
- **Interfaz HIPOTÉTICA:** `verify_sources(VerificationRequest) ->
  VerificationReport`, puerto de aplicación que usa lectores de fuente
  autorizados; dominio puro interpreta el resultado.
- **Acciones concretas:** mapear revision/provenance actuales; llevar el digest
  de la generación recuperada hasta el assembler; verificar con presupuesto
  de trabajo; no renderizar rango antiguo como si fuera actual; exponer
  alternativa «bytes actuales» rotulada sin marcas indexadas.
- **Dependencias/relaciones:** retención y acceso a fuente, lectura consistente,
  path guard y límites. Unidad B publica generaciones; A comprueba la que
  realmente sustentó la respuesta, también después de un salto de worker.
- **FMEA:** TOCTOU entre hash y lectura; timeout confundido con fresh;
  hash de bytes distinto al texto normalizado; symlink sale de raíz;
  presupuesto agotado oculta contribuyentes sin comprobar.
- **Aceptación funcional propuesta:** cambio de fuente produce stale o lectura
  explícitamente actual; error/timeout produce unchecked, no verified;
  rango antiguo nunca se aplica silenciosamente a otro texto;
  scope/root no se amplían; todos los contribuyentes quedan enumerados.
- **No objetivo:** copiar constantes 200 archivos/8 MiB/250 ms, garantizar
  snapshot de filesystem sin soporte o reindexar automáticamente sin permiso.

### Unidad B — reemplazo recuperable de generación por fuente

- **Problema:** reingest puede borrar evidencia válida o perder relaciones
  entrantes aunque solo haya cambiado una fuente.
- **Fuente mínima:** `storeExtractionResult` y reattach [C2]; lock/reapertura
  del índice [C3]. Se adopta el contrato, no la implementación SQLite.
- **Seam Vanta:** `vanta-memory/src/ingest/merge.rs`, writer/provenance,
  scheduler/checkpoints. Evitar una segunda cola o almacenamiento paralelo.
- **DTO HIPOTÉTICO:** `SourceGeneration { source_id, revision, state }`;
  `ImportBundle { records, relations, unresolved_refs }`;
  `PublishReceipt { previous_revision, published_revision, cleanup_state }`.
- **Interfaz HIPOTÉTICA:** `publish_generation(request) -> Result<Receipt,
  PublishError>`; extracción produce candidatos, aplicación coordina commit.
- **Acciones:** inventariar atomicidad actual; preparar nueva generación;
  validar relaciones y límites; publicar mediante commit vigente;
  invalidar referencias por revisión; recuperar cleanup/reintentos;
  diferenciar error del extractor de una fuente realmente vacía.
- **Dependencias:** IDs estables de fuente y relación, retención de generación
  previa, escritura condicional/checkpoint. Cambiar líneas no debe cambiar
  identidad semántica sin registrar la correspondencia.
- **FMEA:** marker vacío se presenta como importación correcta; chunk parcial
  queda visible como completo; dos writers publican generaciones invertidas;
  reattach por nombre ambiguo une sobrecargas equivocadas; borrado deja refs.
- **Aceptación propuesta:** extracción fallida conserva última evidencia
  utilizable con estado explícito; retry converge; relaciones entrantes se
  mantienen o invalidan de forma trazable, no desaparecen silenciosamente;
  publicación vieja no pisa nueva; cleanup pendiente no finge operación íntegra.
- **No objetivo:** afirmar que Vanta ya falla en estos casos, ni copiar el
  mecanismo de IDs por línea o los chunks upstream.

## 3. Defectos, riesgos e issues: nivel de evidencia

- **Defecto estático del scorer:** `scoreSearchNodes` fija `firstRank` con
  el primer símbolo de la lista esperada que encuentra, no el menor rank
  relevante ([C7], líneas 18-29). Ejemplo lógico: expected `[A,B]`, results `[B,X,A]`
  devuelve MRR `1/3` en lugar de `1`. No se ejecutó ni se cuantificó efecto
  en casos actuales; casos con un solo esperado no exponen esa diferencia.
- **Código:** recall compara solo nombres case-insensitive, no archivo,
  qualified name o revisión [C7]. Puede dar crédito a homónimos;
  es un límite del runner, no prueba de error del motor de resolución.
- **Código:** rama grande de almacenamiento es chunked, no una transacción
  total [C2]. **Hipótesis:** crash puede dejar visibilidad parcial y requerir
  recuperación; no se declara defecto reproducido ni fallo del caso pequeño.
- **Código:** fuente demasiado grande puede validarse por stamp de tamaño
  en helper de frescura ([C5], líneas 40-46). No confundir ese criterio de marcador
  con digest del contenido íntegro ni con certificación byte a byte.
- **Hipótesis:** el archivo puede cambiar tras validación y antes de entregar
  texto. No se auditó toda composición de lectura para determinar esa ventana.
- **Issues:** no investigados. Números citados por comentarios del código son
  contexto histórico, no auditoría de estado actual ni defectos Vanta.

## 4. Prerrequisitos Vanta

Revalidar semántica actual de revisiones, borrado, supersession y checkpoints.
Determinar qué fuentes siguen legibles y qué significa verificar una fuente
remota o borrada. No añadir filesystem reads al core sin puerto y autorización.
Definir límite de trabajo y conducta degradada sin inventar umbrales de calidad.
Si se añade extractor de código, separar evidencia AST de relaciones inferidas.

## 5. Qué no trasladar

No portar resolver multilenguaje, framework patterns, MCP daemon, instaladores,
watcher, kernel y UI como una sola feature de memoria.
No sustituir BM25/HNSW/RRF de Vanta por FTS5 por similitud con este producto.
No interpretar AST determinista como análisis semántico completo de programas.
No equiparar densidad de aristas con corrección ni «local» con ausencia de
superficies de confianza. No copiar instrucciones de uso como diseño de producto.

## 6. Método, licencia y cobertura

MIT congelado y `LICENSE:1-21`, copyright Colby Mchenry 2026 [C11].
Revisar licencias de gramáticas, kernel y dependencias antes de adoptar código;
una traducción a Rust no elimina derivación.
Clon superficial de rama única, sin historial completo ni ejecución.
No se certifica soporte de todos los lenguajes, crash safety ni auth UI/MCP.
Se inspeccionaron unidades acotadas, no el archivo `src/index.ts` íntegro.

## 7. Runner upstream, entradas, métricas, dependencias y efectos

`__tests__/evaluation/runner.ts:1-123` [C6] exige `EVAL_CODEBASE` o path,
comprueba `.codegraph/codegraph.db` y abre índice existente. No mide coste
de una indexación desde cero. Usa test cases con símbolos concretos
(`TransportService`, `RestController`, etc.) [C8]; no corpus agnóstico que
pueda ejecutarse sobre Vanta sin cambiar la pregunta que se evalúa.

Dos rutas: `searchNodes` con límite y `findRelevantContext` con opciones
prefijadas upstream. Scoring [C7]: recall de nombres, recíproco descrito arriba
y `edges/nodes` como densidad; pass definido por `PASS_THRESHOLD=0.5`.
Se documentan valores existentes, **no se proponen ni modifican**.
No hay LLM reader/juez en este runner ni QA de memoria conversacional.

Dependencias: Node/TypeScript, biblioteca CodeGraph y un índice compatible
preparado de antemano; no se ejecutó ni instaló nada.
Efectos: lee el índice, ejecuta `git rev-parse --short HEAD` en cwd del runner,
crea `evaluation/results` y escribe JSON timestampado. El SHA reportado
no fija por sí solo commit/revisión del corpus target.
`cg.close()` ocurre en éxito; no hay `finally` de cierre ni estado por caso
fallido en el cuerpo leído. Un throw termina el run.

Media MRR filtra resultados por valor positivo o ID `search-*`; no asumir
denominador uniforme con todas las consultas. No se midieron aquí latencias,
consumo, calidad AST ni ahorro de tokens. No se alteraron prompts, evaluadores,
sampling, thresholds o configuraciones. Comparación futura según
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz frente a VantaDB

| Capacidad | Estado usado | Diferencia bajo estudio |
|---|---|---|
| BM25/HNSW/RRF | Existe | no adoptar FTS5 como reemplazo |
| Provenance/supersession/dedup | Existe | digest/revisión de evidencia entregada |
| Ingestors/checkpoints/scheduler | Existe | generación por fuente y cleanup |
| Assembly/compaction | Existe | frescura de los contribuyentes |
| UI de memoria/diagnóstico en Studio | Existe | estados stale/unchecked |
| Coherencia completa de reingest | Parcial para este contraste | mecanismos presentes; garantía compuesta no auditada |
| Verificación de revisión al responder | No acreditado | no se reaudita HEAD actual |
| Indexador AST multilenguaje en producto Rust | No acreditado | herramienta de desarrollo no equivale a producto |
| Reutilizar instalador/daemon completo | No aplicable | fuera de unidad extraíble |

## 9. Deuda sugerida sin inventar bugs de Vanta

Preguntas: ¿la respuesta conserva la revisión exacta de cada fuente?,
¿se distinguen no verificado y fresco?, ¿un extractor fallido parece una
fuente vacía?, ¿se preservan relaciones al sustituir una generación?,
¿se puede diagnosticar cleanup pendiente?
No son defectos constatados de `167435a9e`; revalidar y deduplicar tareas
antes de implementar. El estudio base no autoriza resucitar bugs históricos.

## 10. UI/escritorio y aplicación a Studio

**Código real:** `ui/src/App.svelte:1-80` importa vistas Symbol/File/Map/Flow,
etc., conecta canal live y recarga stats/entries/trails al cambiar el índice
[C10]. No es solo una imagen de marketing ni prueba de app desktop nativa.
El source `ui/src/lib/adapter.ts:1-125` define seam de datos/errores
y petición de source con comportamiento de drift [C9]. Describe transporte
HTTP para viewer e implementación in-process posible; esa descripción no
acredita que el producto Pro externo haya sido auditado.

Studio Tauri/React ya existe en `develop-desktop`: adaptar el principio de
DTO/errores y aviso de revisión en pantallas actuales. No portar Svelte,
otro servidor loopback ni el producto Pro. En un visor de evidencia, desactivar
marcas por línea si se muestran bytes actuales fuera de la revisión indexada.

### Ledger de lectura efectiva

| Ref. | Archivo/rangos | Evidencia |
|---|---|---|
| C1 | `src/db/schema.sql:19-143` | nodos/aristas/files/refs/FTS |
| C2 | `src/extraction/index.ts:3319-3480` | replace, markers, snapshot/reattach |
| C3 | `src/index.ts:1110-1169` | lock y base reemplazada |
| C4 | `src/context/index.ts:283-435,461-565` | contexto, confianza, paths y semillas |
| C5 | `src/mcp/answer-freshness.ts:1-66` | verificación acotada |
| C6 | `__tests__/evaluation/runner.ts:1-123` | runner |
| C7 | `__tests__/evaluation/scoring.ts:1-82` | scoring y defecto MRR |
| C8 | `__tests__/evaluation/test-cases.ts:1-92` | ejemplos anotados |
| C9 | `ui/src/lib/adapter.ts:1-125` | seam y drift |
| C10 | `ui/src/App.svelte:1-80` | UI implementada |
| C11 | `LICENSE:1-21` | MIT |

AGENTS/CLAUDE leídos; listados/búsquedas no se cuentan como módulos auditados.
No cubiertos: kernel completo, parsers por lenguaje, resolución completa,
watcher/transportes, caller de frescura, transacciones del backend y auth UI.
Pendientes: recuperación en crash, TOCTOU, indexación de corpus y conformance UI.
No scripts de proyectos, tests/builds/apps/installs, benchmarks, servicios,
red, validadores documentales, commits ni push.

[C1]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/db/schema.sql#L19-L143
[C2]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/extraction/index.ts#L3319-L3480
[C3]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/index.ts#L1110-L1169
[C4]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/context/index.ts#L283-L565
[C5]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/mcp/answer-freshness.ts#L1-L66
[C6]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/__tests__/evaluation/runner.ts#L1-L123
[C7]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/__tests__/evaluation/scoring.ts#L1-L82
[C8]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/__tests__/evaluation/test-cases.ts#L1-L92
[C9]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/ui/src/lib/adapter.ts#L1-L125
[C10]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/ui/src/App.svelte#L1-L80
[C11]: https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/LICENSE#L1-L21
