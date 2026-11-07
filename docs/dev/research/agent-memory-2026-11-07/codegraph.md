---
title: "CodeGraph: evidencia revisionada y actualización de grafos por archivo"
kind: research
status: proposed
description: "Lectura estática de persistencia, frescura, contexto y viewer de CodeGraph para ideas independientes en Rust"
tags: [research, memory, engineering]
---

# CodeGraph: evidencia revisionada y actualización de grafos por archivo

## Índice

- [Identidad y dictamen](#identidad-y-dictamen)
- [1. Arquitectura, tecnologías, flujo y recomendaciones](#1-arquitectura-tecnologías-flujo-y-recomendaciones)
- [2. Unidades mínimas para diseño independiente en Rust](#2-unidades-mínimas-para-diseño-independiente-en-rust)
- [3. Defectos, riesgos e issues: nivel de evidencia](#3-defectos-riesgos-e-issues-nivel-de-evidencia)
- [4. Prerrequisitos Vanta](#4-prerrequisitos-vanta)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Método, licencia y cobertura](#6-método-licencia-y-cobertura)
- [7. Runner upstream, entradas, métricas, dependencias y efectos](#7-runner-upstream-entradas-métricas-dependencias-y-efectos)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Deuda sugerida sin inventar bugs de Vanta](#9-deuda-sugerida-sin-inventar-bugs-de-vanta)
- [10. UI/escritorio y aplicación a Studio](#10-uiescritorio-y-aplicación-a-studio)
- [11. Dossier ampliado y revalidación](#11-dossier-ampliado-y-revalidación)
- [12. Fichas adicionales y cierre](#12-fichas-adicionales-y-cierre)

## Identidad y dictamen

- Repositorio **`colbymchenry/codegraph`**, no otros productos llamados CodeGraph.
- Clon `C:\Users\Eros\codegraph`; SHA
  `f72c9d2efb14454ff4f9fd7b8460abd82f19a287`, rama `main`.
- Snapshot: **76 956 estrellas**, MIT según
  [candidatos](github-candidates.json) y [manifest](clone-manifest.json).
- Método [METHOD](METHOD.md); baseline Vanta revalidado
  `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; `c561158`/dirty es histórico.
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
- **Contrato HIPOTÉTICO:** `CodeEvidenceImport` C12 de
  [CONSOLIDATED](CONSOLIDATED.md#c12--codeevidenceimport-contrato-canónico-único-de-evidencia-code-memory).
  La generación, revisión y productor pertenecen a su identidad; registros,
  relaciones y evidencia no constituyen un segundo `ImportBundle`.
  El recibo de publicación describe visibilidad/limpieza, no otro almacén.
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
- **Issues:** muestra efectivamente leída en §11; los demás números en
  comentarios siguen siendo contexto histórico, no auditoría de estado.

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
No son defectos constatados del snapshot `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; revalidar y deduplicar tareas
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

## 11. Dossier ampliado y revalidación

### 11.1 Mapa físico, stack y modelo persistente

Lectura complementaria **2026-11-07**, sobre el clon y SHA completo de
§Identidad. `git rev-parse HEAD` coincidió y `git status --short` no emitió
cambios. CodeGraph MCP contestó que **este clon no está indexado**; no se
ejecutó `init`. Fast Context expiró; se continuó con lectura directa.
El índice Vanta sí respondió, pero se contrastó el baseline con
`git show 97c9623...:<ruta>` para no confundir snapshots.

| Área | Implementación y frontera |
|---|---|
| `src/index.ts`, `lazy-engine.ts`, `engine.ts` | API pública CodeGraph y coordinación; AGENTS prescribe carga perezosa del engine para lecturas |
| `src/extraction/` | AST tree-sitter/WASM y workers; nodos, edges, referencias pendientes y errores |
| `src/resolution/` | Imports/nombres/frameworks y síntesis; sintaxis determinista no garantiza destino runtime |
| `src/db/` | SQLite, prepared statements, esquema/migraciones; no servidor multitenant |
| `src/graph/`, `src/context/` | Traversal, límites dinámicos y ensamblado; no memoria conversacional |
| `src/sync/`, `src/mcp/` | Watcher/transporte; historial de emisiones de sesión distinto del grafo |
| `src/ui-server/`, `ui/` | API JSON/lector web Svelte; `src/ui/` es la UI terminal |
| `codegraph-kernel/`, `telemetry-*` | Localizados, no auditados íntegramente; “local-first” no acredita air-gap |

[package.json:1–74](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/package.json#L1-L74)
declara 1.6.2, TypeScript, commander, ignore/picomatch y web-tree-sitter.
Distingue runtime anunciado `>=20 <25` de desarrollo `>=22.12`.
El backend leído es **`node:sqlite`/DatabaseSync**, no better-sqlite3 ni SQLite
WASM: [sqlite-adapter.ts:1–100](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/db/sqlite-adapter.ts#L1-L100)
expone wrapper de forma compatible y documenta Node >=22.5 desde fuente.
Las WASM son del parser, no un fallback de persistencia.

C1 separa `nodes.id TEXT` (tipo, nombre/qname, fuente, posiciones, firma y
docstring), `edges.id INTEGER` (extremos/kind/metadata/posición/provenance,
cascada FK), `files.path` (hash/tamaño/mtime/errores/generated) y
`unresolved_refs` (nombre original, tipo, posición, archivo/lenguaje,
candidatos, pending/failed/name_tail). IDs de fila no son identidad portátil.
Resolver elimina refs resueltas y conserva failed para retry. FTS indexa
**nombres/qnames/docstrings/firmas**, no acredita todos los cuerpos; triggers
acompañan insert/update/delete. El file node_count puede ser anterior al
filtrado, no un denominador de calidad.

La muestra no acredita memoria de usuario/sesiones/supersession/ACL por tenant
equivalente a Vanta: N/A para este productor, no una razón para construir otro
motor. MIT raíz no certifica licencias de gramáticas/kernel/transitivas.

### 11.2 Traza indexación → grafo → consulta → refresh/delete

1. **Admisión:** `sync` toma mutex local y file lock (C3); fallo produce
   `LockUnavailableError`. Reabre DB reemplazada y fuerza reconciliación
   completa. Archivo recién recreado sin estado significa rebuild en curso,
   no un baseline vacío que pueda considerarse listo.
2. **Store:** C2 no reemplaza evidencia por grammar-load failure. Captura nodos
   y edges entrantes antes de cascada. Bundle pequeño usa una transacción;
   grande divide nodos/aristas/refs en bloques **2000** y yields. Exclusión del
   writer no implica atomicidad entre todos esos commits para lectores.
   [3503–3515](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/extraction/index.ts#L3503-L3515)
   escribe file record al final.
3. **Refresh:** se resuelven refs de archivos cambiados y se reabren afectadas
   por imports/barrels/default exports/nuevos candidatos. El barrido
   [index.ts:1735–1768](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/index.ts#L1735-L1768)
   recupera pendientes incluso sin cambios nuevos. No se auditó cada rama
   multiidioma del resolver.
4. **Delete:** [removeTracked:3896–3940](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/extraction/index.ts#L3896-L3940)
   comprueba existencia física: Git puede seguir listando un borrado unstaged.
   Antes de `deleteFile` resucita refs originales de edges entrantes para
   rebind/failed. C2 advierte que edges legacy/sintetizados sin `refName`
   pueden caer: inventar short-name quitaría contexto receptor/qualifier.
5. **Query:** C4 selecciona seeds/subgrafo/bloques/archivos/resumen. Markdown
   añade call paths y baja confianza. Es evidencia estática, no ejecución;
   preservar provenance de wiring sintetizado.
6. **MCP:** [tools.ts:3114–3203](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/mcp/tools.ts#L3114-L3203)
   espera recovery, puede offload al pool y recibe `_cgAnswerFiles`; lo elimina
   del resultado. **Solo con watcher antes/ahora degraded**, sin error y con
   contribuyentes, llama a `validateAnswerFiles`. Stale/unchecked rehúsan el
   cuerpo antes de registrar emisiones de sesión. Esto concreta el caller
   pendiente en §1: no es verificación universal byte-a-byte de todo MCP.
7. **Estado:** [getIndexState:2165–2193](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/index.ts#L2165-L2193)
   distingue indexing/complete/partial/failed/null y build/extraction version.
   Complete del run no certifica todas las resoluciones. Cancelación/crash
   entre chunks y recuperación íntegra quedan pendientes manuales.

### 11.3 Incidencias públicas concretas

Observadas **2026-11-07** por API pública GitHub `/issues/{n}`: cuerpo y
metadata leídos, cero comentarios en estos tres; ninguna reproducción propia.

| Fuente, fecha y estado | Prueba reportada y causa acreditada/pendiente | Aplicabilidad |
|---|---|---|
| [#3251](https://github.com/colbymchenry/codegraph/issues/3251), 2026-11-06, open | Repro Rust `format!`, `vec!`, `write!`, `json!`; plain sí enlaza. Autor identifica npm 1.6.2 y **f72c9d2**, mismo SHA. token_tree es su explicación; no auditamos expansión de macros | “Sin callers” no prueba dead code ni permite excluir tests. Futuro caso macro/plain; revisar extractor Rust antes de atribuir causa definitiva |
| [#3238](https://github.com/colbymchenry/codegraph/issues/3238), 2026-11-04, open | Globals `parseInt`/`new Response()` enlazan exports homónimos no importados; dos archivos, `resolvedBy: exact-match`; `Date.now` es control | Exact-match es estrategia, no certeza. Preservar binding y no promocionar confidence a confianza Vanta; matcher y PR correctivo no auditados |
| [#3228](https://github.com/colbymchenry/codegraph/issues/3228), 2026-11-04, open | Init anuncia “Already initialized” tras run abortado; autor declara lectura estática, no repro; referencia f72c9d2/getIndexState | Installed/present/complete/usable son estados distintos. GetIndexState se leyó; no acredita fallo runtime de init ni bug Vanta |

Websearch/DDG `site:github.com/colbymchenry/codegraph/issues` dio **0 resultados**;
metasearch GitHub Graphify también cero. API pública funcionó para estos cuerpos,
después agotó cuota (`API rate limit exceeded`); GitHub MCP falló al conectar.
Fetch HTML keyless funcionó como fallback para Graphify #5051. No se iniciaron
servicios ni actualizaron clones. No hay corrección/PR leído para estos tres;
open no prueba que todas las versiones/lenguajes sufran el problema.

### 11.4 Evaluador: defaults, población y adaptación

C6–C8 relectura: **12 casos** hard-coded, seis search/seis explore, con nombres
del dominio Elasticsearch; sin descarga o SHA obligatorio del corpus.
`EVAL_CODEBASE` precede al argumento. Search usa limit 10; explore usa
searchLimit 8, traversalDepth 3, maxNodes 80, minScore 0.2; overrides por caso
al final. Son defaults upstream, no parámetros propuestos.

Cada esperado cuenta por nombre case-insensitive; gold vacío da recall 0.
Explore comprueba conjunto de nombres y densidad edges/nodes, no pares posibles.
PASS_THRESHOLD 0.5 es del runner. Media recall incluye resultados emitidos;
MRR filtra valor positivo o ID search. Un throw aborta sin recibo para cada
caso restante. El reloj rodea query, no scoring; excluye open/indexación.
No hay warmup explícito, percentiles, tokenizer/modelo/embeddings/answerer/juez,
split ni sampling en esta suite fija. Gold solo va al scorer.

SHA corto del cwd no fija revisión del target. `npm run eval` compilaría y
escribiría JSON: **no se ejecutó**. Adaptación bajo [METHOD](METHOD.md) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md): mapear `code_search`/contexto/C12 a
refs repo/revisión/símbolo, conservar errores y separar ingesta/query/QA/agente.
Los doce casos no son portables literalmente a Vanta Rust; selección/anotación
de corpus o cambio del scorer compete al protocolo existente del responsable.
No se diseñan aquí harness, prompts, métricas o umbrales nuevos. El defecto MRR
no invalida por sí solo todas las cifras publicadas ni mide calidad del resolver.

## 12. Fichas adicionales y cierre

### 12.1 Contrato compartido y terminación de A/B

**Propuestas Rust, no implementadas.** Importar solo mediante
[`CodeEvidenceImport`, C12](CONSOLIDATED.md#c12--codeevidenceimport-contrato-canónico-único-de-evidencia-code-memory):
identidad repo/revision/producer/schema_version/generation; refs, relation,
direction; evidencia Ast/Heuristic/Model/Unknown, span/hash/reported_weight.
`EvidenceSlice`, receipts e informes de consulta son vistas de aplicación de
esa evidencia, no otros DTO persistentes de importación.

Baseline inmutable para todas las fichas:
[index_repo:152–327](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vantadb-mcp/src/code_index.rs#L152-L327),
[code_impact:272–309](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vantadb-mcp/src/code.rs#L272-L309),
[MemoryInput:184–224](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/types/record.rs#L184-L224)
y [VANTA-REFERENCE](VANTA-REFERENCE.md). Los enlaces relativos previos son
navegación, no nueva revisión acreditada.

| Unidad existente | Delta/alternativas | Ownership, concurrencia, errores, commit y rollback propuestos |
|---|---|---|
| A: frescura | Vanta ya hashea al indexar; verificar **lo entregado** es delta. Verificar al leer, snapshot retenido o aviso unchecked; preferir fuentes retenidas si existen | `verify_sources` toma generación inmutable y bytes de la misma lectura. No escribe ni reindexa; permisos/timeout/oversize son razones explícitas. Publicar informe solo si corresponde a esa generación; rollback desactiva verificación sin anunciar verified. Métricas por estado/causa, nunca snippets |
| B: publicación | Vanta ya reconcilia símbolos; no es nueva ingesta. Extender coordinador actual, adapter C12 o full snapshot; rechazar copia de chunks como garantía atómica | Solo producer/repo dueño publica y baja; CAS de generación evita writer antiguo. Validar fuente/endpoints antes del commit de visibilidad; si API no permite batch indivisible, staging lógico y puntero pendiente de diseño, no garantía asumida. Failed extractor conserva anterior stale; cleanup posterior reintentable |

A depende del catálogo/mapping de B pero no de instalar watcher; B depende de
identidad y del writer real, no del RMW global de checkpoints por analogía.
Aceptación manual adicional: reader concurrente solo ve generación publicada;
cancelación antes de commit no adelanta baseline; repetir el lote converge;
fallo de cleanup es visible sin borrar memorias humanas. No se propone migrar
datos legacy sin owner por inferencia; modo inicial sería diagnóstico sin bajas.

### 12.2 Ficha C — Referencias pendientes con contexto suficiente para retry

- **Caso/unidad:** borrar una definición sin tocar callers no debe convertir
  sus referencias en “nunca existieron”. C1 y removeTracked (§11.2) conservan
  refName/refKind/posición; C2 no adivina cuando ese material falta.
- **Delta/seam:** `all_indexed_symbols` de Vanta solo reúne archivos procesados
  y `references` no lleva contrato de unresolved. Extender `index_repo` y
  consumo `code_callers/code_callees`, no un segundo resolver obligatorio.
- **Contrato:** `reconcile_refs(import: &CodeEvidenceImport,
  expected_generation) -> Result<ResolutionReport, ReconcileError>`.
  Report separa bound/ambiguous/pending/failed/unsupported; candidatos no son
  edges confirmados. `WrongRevision`, `MissingSource`, `IncompleteManifest` y
  `Conflict` no se traducen a cero callers.
- **Pasos/estado:** mapear refs originales → invalidar bindings propios cuyo
  target cambió → pedir evidencia al productor → validar receptor/import →
  publicar resolución en generación → reintentar solo pendientes relevantes.
  Legacy sin callee original queda unknown, no se reconstruye desde display name.
- **Alternativas/deps:** full reimport si no hay ref portable; índice invertido
  incremental solo después. Depende de B/C12 y coverage del productor, no de
  portar parsers. Rollback vuelve al mapping previo marcado por revisión.
- **FMEA:** homónimos tras delete → rebind equivocado → detectar candidatos
  múltiples y retener ambigüedad; retry infinito → jobs repetidos → estado
  observable y retry gobernado; otro productor borra endpoint → ownership estricto.
- **Aceptación futura:** delete/reintroducción, overloads y line shift mantienen
  identidad/evidencia o explican unknown; retry idempotente; logs solo conteos
  por estado/generación. **No objetivos:** inferir dispatch runtime o recuperar
  información que el export nunca conservó.

### 12.3 Ficha D — Límite dinámico como resultado, no arista inventada

- **Caso/fuente:** cuando un flow termina en dispatch, “no hay camino” es demasiado
  fuerte. [dynamic-boundary-report.ts:1–109](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/graph/dynamic-boundary-report.ts#L1-L109)
  define BoundarySite/BoundaryCandidate/continuations, lectura acotada de cuerpos
  y representación compartida MCP/viewer; dice explícitamente que no muta grafo.
- **Delta/seam:** `code_impact` ya tiene depth/direction, no acredita reporte de
  cierre semántico. Extender respuesta y assembly existente, no añadir calls
  especulativas. Reutilizar C12 para refs a evidencia; reporte es efímero.
- **Interfaz:** `explain_boundary(snapshot, seeds, authorized_source_reader,
  budget) -> Result<BoundaryReport, QueryError>` con stopping_ref, razón,
  candidatos y coverage. Un candidato nunca puede participar como call confirmado.
- **Pasos/commit:** fijar generación → recorrer relaciones admitidas → informar
  dónde termina evidencia → shortlist opcional autorizada → presupuestar salida.
  Sin escrituras/commit; cancelación devuelve incomplete, no path inexistente.
- **Alternativas/deps:** solo avisar unsupported es mínimo sostenible; shortlist
  opcional si el productor conserva dispatch key. Depende de A y de tipado de
  relaciones, no de un LLM. Desactivar shortlist deja el límite informado.
- **FMEA:** regex confunde strings con llamadas → candidato erróneo → rótulo
  heurístico y fuente; hub agota presupuesto → omission explícita; stale body
  → rechazo/unchecked por A. Métricas de límites/omisiones, sin código en logs.
- **Aceptación futura:** callback desconocido devuelve límite; llamada real se
  separa de candidato; cancelación y query vacía no prueban ausencia. **No
  objetivos:** copiar constantes upstream, inventar edges o seleccionar tests.

### 12.4 Ficha E — Puerto visual con errores y drift explícitos

- **Caso/fuente:** C9 `GraphAdapter`/`ApiFailure`/`SourceRequest.ondrift` permite
  mismo lector con transportes diferentes; no obliga a HTTP. Aporta distinción
  not-found/refused/drift/current sin presentar otro símbolo bajo rango antiguo.
- **Delta/seam:** Studio ya existe y tiene diagnóstico retrieval según
  VANTA-REFERENCE. Proponer `read_code_view(auth, generation, selection) ->
  Result<CodeView, ViewError>` y `open_evidence(ref)` sobre API existente;
  jamás importar Svelte o declarar endpoint Studio ya disponible.
- **Pasos/ownership:** autorizar repo → resolver C12 → obtener bytes/rango de
  revisión → renderizar estado/errores → desactivar marcas si se eligen bytes
  actuales. Lectura sin commit; selección concurrente descarta respuesta de
  generación anterior. Cancelar navegación no cancela indexador compartido.
- **Alternativas/deps:** enlaces a fuente son mínimo; adapter visual requiere A/B
  y trabajo frontend separado. Desactivar panel no toca evidencia persistida.
- **FMEA/aceptación:** symlink/ruta ajena se deniega; error no parece pantalla
  vacía; stale no aplica anotaciones indexadas a bytes actuales; selección vieja
  no pisa nueva. Observar errores por clase, sin roots ni contenido sensible.
- **Límite/no objetivos:** [viewer-gate.ts:1–29](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/src/bin/viewer-gate.ts#L1-L29)
  exige `CODEGRAPH_UI=1`; [ui/package.json](https://github.com/colbymchenry/codegraph/blob/f72c9d2efb14454ff4f9fd7b8460abd82f19a287/ui/package.json#L1-L62)
  es private, Svelte5/XYFlow/Vite. Fuente presente ≠ viewer lanzado o paquete
  publicado. No acreditar app desktop, Pro externo, a11y o seguridad frontend.

### 12.5 Matriz final, hallazgo local y cobertura

| Capacidad | Vanta en snapshot | Decisión/evidencia |
|---|---|---|
| Símbolos/hash/reconcile/scanners Rust/Python/TS/JS | Existe | code_index 193–235,514–520; conservar |
| AST multilenguaje/resolver imports equivalente | No acreditado | Adapter opcional, no inferir del tooling developer |
| Evidence source/Derived y grafo | Existe/parcial para import code | MemoryInput existe; metadata por relación no queda acreditada por `add_edge` |
| Estado generación/completitud por productor | No acreditado | B/C, no atribuir atomicidad por nombre de API |
| Verificación al entregar contexto | Parcial | Assembly/compaction existen; A amplía provenance de contribuyentes |
| Cierre exhaustivo para tests | No acreditado | D explica límites; depth no demuestra cierre |
| Visor Svelte integrado o memoria conversacional upstream | N/A | Producto/propósito distinto; Studio existente se amplía |

**Negativo local sustentado:** code_index 230–233 ignora fallos de remove/delete
con `let _` y aumenta stale_symbols_removed tras `Ok(Some(old))`; 310–315
ignora resultado de add_edge heurística. Contador no acredita que toda mutación
haya tenido éxito. Es observación de esas ramas, no reproducción de pérdida ni
diagnóstico del writer global. Aceptación futura: inducir fallo aislado de
mutación y comprobar recibo incompleto antes de usarlo para ACK.

Fuentes adicionales efectivamente leídas: package manifests, sqlite-adapter,
removeTracked, getIndexState, caller MCP y dynamic-boundary types/rationale,
además C1–C11. No lectura exhaustiva de kernel, matcher Rust/TS, síntesis,
transacciones backend, transports, telemetry o auth. Revisión adversarial por
autor independiente y casos manuales pendientes; no validación global ni
ejecución. Si pasa a specs activas, dividir A/frescura, B/C/importación,
D/query y E/Studio; este dossier >300 líneas conserva TOC y trazabilidad
sin crear archivos adicionales ni reservar backlog.
