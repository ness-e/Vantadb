---
title: "Understand Anything: semántica y exploración sobre evidencia canónica"
kind: research
status: proposed
description: "Lectura estática de grafos enriquecidos, fingerprints, contexto y visor de Understand Anything para conectores opcionales de VantaDB"
tags: [research, memory, engineering]
---

# Understand Anything: semántica y exploración sobre evidencia canónica

## Índice

- [1. Dictamen y arquitectura](#1-dictamen-y-arquitectura)
- [2. Fichas candidatas](#2-fichas-candidatas)
- [3. Riesgos y defectos transferibles](#3-riesgos-y-defectos-transferibles)
- [4. Prerrequisitos VantaDB](#4-prerrequisitos-vantadb)
- [5. Componentes que no trasladar](#5-componentes-que-no-trasladar)
- [6. Método y ledger de alcance](#6-método-y-ledger-de-alcance)
- [7. Benchmarks existentes](#7-benchmarks-existentes)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades sugeridas](#9-debilidades-sugeridas)
- [10. Interfaz y disponibilidad](#10-interfaz-y-disponibilidad)

## 1. Dictamen y arquitectura

**Propuesta:** consumir el grafo pedagógico como enriquecimiento opcional sobre
memorias y evidencias ya canónicas; no usar sus aristas narrativas como call graph
determinista ni trasladar su pipeline de agentes al motor Rust.

- Repo `Egonex-AI/Understand-Anything`; HEAD local verificado
  `a6ea4024767df85ce4f99140d4f12ecfc5a81eb6`.
- [Snapshot](github-candidates.json): **91 147 estrellas**,
  `2026-11-07T06:57:17.6691986Z`, licencia detectada MIT.
- Monorepo TypeScript/pnpm: core con web-tree-sitter/WASM, registros de lenguajes,
  fingerprints, Zod, persistencia JSON y búsqueda Fuse; skill/agents para análisis
  LLM; MCP stdio; dashboard React/React Flow/Zustand y viewer distribuible.
- Flujo: estructura analizada + resúmenes de agente → `GraphBuilder` → grafo con
  nodos/aristas/layers/tour → validación/persistencia → MCP/contexto/dashboard.
  `GraphEdge` incluye tipo, dirección, descripción y weight, pero no procedencia
  AST/LLM, span probatorio ni confidence calibrada por arista (U1/U2).
- El builder construye IDs como `function:${filePath}:${fn.name}` y pesos fijos
  de relación; por tanto weight no es una probabilidad de verdad.
  El producto mezcla estructura verificable y explicación model-enriched:
  no hay base para clasificar toda arista `calls` como evidencia de ejecución.

**Vanta (lectura histórica del worktree inicial `c561158`):** [code_index.rs](../../../../vantadb-mcp/src/code_index.rs),
líneas 193–315,514–520, ya indexa símbolos, `defines`, referencias heurísticas,
hash/reconcile y despacha scanners Rust/Python/TS/JS. [code.rs](../../../../vantadb-mcp/src/code.rs)
expone consultas sobre grafo propio. Esto es producto real, pero no paridad
AST/semántica completa. `FIND-299` (en `Backlog.md` (registro local, no versionado por git)), línea 285, y
`mgr-22-repo-map` (registro historico, sin destino localizable en el repo; ver FIND-299 en `Backlog.md` (registro local, no versionado por git)) siguen siendo referencia para el resto.
Tener CodeGraph/Understand Anything instalados como developer tooling no implica
que Vanta los ingiera automáticamente. Tampoco justifica incorporar otro indexer.

## 2. Fichas candidatas

### Ficha A — Importación en dos capas: estructura y explicación

- **Problema/unidad mínima:** `GraphBuilder.addFileWithAnalysis`,
  `addImportEdge` y `addCallEdge`, `analyzer/graph-builder.ts:102–207` (U2),
  más `KnowledgeGraph`/`GraphEdge` en `types.ts:61–122` (U1).
  Conservar resúmenes y tours sin hacerlos pasar por relaciones deterministas.
- **Seam real:** [MemoryInput](../../../../src/sdk/types/record.rs),
  líneas 184–224, y [Embedded::add_edge](../../../../src/sdk/api/graph.rs),
  líneas 113–151. La arista no admite metadata arbitraria: usar evidencia asociada.
  Wiki y scene existentes sirven para navegación/documentación, no otro store.
- **Contrato Rust propuesto:** `CodeExplanationImport { repo_id, revision,
  graph_version, producer, nodes, relations, layers, tour, provenance }`;
  cada `ImportedRelation { external_source, external_target, kind, direction,
  reported_weight, evidence_keys, origin: Option<Origin> }`.
  Si el export no acredita origen, `origin=None`, no `Ast`.
- **Pasos/dependencias:** validar JSON/schema → conservar original y problemas
  de saneamiento → mapear IDs namespaced → contrastar rutas/spans contra revisión
  permitida → persistir explicación como derivada de evidencia → publicar relaciones
  con origen conocido o estado no acreditado.
- **FMEA:** `autoFixGraph` vuelve aceptable el formato, no verdadera la afirmación;
  homónimos/overloads colisionan si solo hay nombre/archivo; move cambia ID.
  No enlazar por nombre corto ni usar resúmenes como prueba circular.
- **Aceptación futura:** importación idempotente; arista sin fuente no se muestra
  como AST; schema desconocido no produce escrituras parciales; los IDs originales
  se conservan para auditoría y los scopes de repo no se mezclan.
- **No objetivos:** portar Zod/tree-sitter/pipeline de subagentes, ni copiar
  automáticamente pesos fijos al sistema de confianza de Vanta.

### Ficha B — Freshness independiente de “cambio cosmético”

- **Unidad:** `contentHash`, `compareFingerprints` en `fingerprint.ts:80–89,142–250`
  (U3), `GraphFreshnessResult` y `mergeGraphUpdate` en `staleness.ts` (U4).
  La frescura diferencia fresh/dirty/stale/unknown y ahead/behind/diverged;
  falta de evidencia no se declara fresh.
- **Seam:** manifiesto/hash por archivo de `code_index.rs:193–235`, procedencia
  de ingestors y FIND-299. No crear otro watcher paralelo al mecanismo elegido.
- **DTO propuesto:** `CodeSnapshotState { revision, head_seen, generation,
  worktree_digest, schema_version, changed_files, state, reason }`;
  `state` mantiene unknown distinto de stale. `SemanticInvalidation {
  evidence_keys, changed_paths, producer_version, model_prompt_fingerprint }`.
- **Pasos/dependencias:** recibir snapshot del productor → validar hash de bytes
  referenciados → invalidar explicación si cambió evidencia → reconciliar bajas
  y relaciones entrantes → reconstruir derivaciones/layers/tours afectados →
  publicar nueva generación sin mezclar revisiones.
- **FMEA:** firma idéntica y cuerpo distinto puede cambiar semántica;
  “COSMETIC” estructural no basta para resumen LLM; borrar nodos puede borrar
  aristas entrantes que el delta no repone; layers/tour retienen IDs antiguos.
  `mergeGraphUpdate` conserva el resto del objeto: no acredita regeneración de
  todas las derivaciones ni cierre transitivo de invalidación.
- **Aceptación:** cambios de cuerpo invalidan evidencia semántica aunque no
  cambie firma; un commit no disponible produce unknown; delete/move no deja
  navegación a IDs inexistentes; delta incompleto no se anuncia como snapshot completo.
- **No objetivos:** copiar heurísticas upstream de clasificación como política
  de seguridad o garantía de no regresión funcional.

### Ficha C — Adaptador visual y contexto acotado por evidencia

- **Unidad:** `buildChatContext`, `src/context-builder.ts:25–76` (U5);
  `buildDiffContext`, `src/diff-analyzer.ts:22–87` (U6);
  `GraphView.tsx` y `readSourceFile` del dashboard (U8/U9).
  Chat expande un salto bidireccional después de limitar seeds; diff también
  devuelve vecinos de un salto. No son cierre transitivo de impacto.
- **Seam:** context assembly con presupuesto compartido de Vanta y Studio
  separado en `develop-desktop`; seleccionar evidencia del store observado (worktree de la fase inicial).
- **DTO propuesto:** `CodeViewSlice { repo_id, revision, nodes, relations,
  layers, evidence_links, omitted, freshness, navigation_targets }`.
  Referencias de fuente son `{path, span, revision}`, nunca rutas absolutas abiertas.
- **Pasos/dependencias:** autorizar repo → recuperar seeds → expandir dentro de
  presupuesto global → devolver límites → abrir fuente solo en raíz permitida.
  Requiere contrato de API/UI y autorización ya existentes; no empaquetar servidor.
- **FMEA:** límite de 15 seeds no limita vecinos de un hub; el JSON pesa más que
  la selección inicial; una ruta symlink puede escapar aunque sea léxicamente local.
  Limitar salida total y verificar ruta física al abrir fuente.
- **Aceptación:** hub truncado declara omisiones; seleccionar nodo muestra evidencia,
  revisión y estado; token inválido/ruta fuera de raíz se deniegan; no usar vecinos
  de diff para excluir tests.
- **No objetivos:** migrar toda UI, emitir prompts de evaluación ni diseñar
  selección de tests; tours/layers son ayudas de comprensión, no ground truth.

## 3. Riesgos y defectos transferibles

| Evidencia local | Conclusión limitada | Riesgo al adaptar |
|---|---|---|
| U1 `GraphEdge` sin origen/confidence/span | El esquema no acredita procedencia por arista | Mezclar AST y explicación como hechos iguales |
| U2 IDs por archivo/nombre | Moves alteran identidad; overloads requieren más discriminación | Referencias históricas mal enlazadas |
| U5 expansión tras límite de seeds | No hay cap total de vecinos en esa función | Contexto de hub crece pese a `maxNodes` |
| U4 merge elimina edges incidentes a nodos reemplazados | El helper no demuestra reextracción de todos los dependientes | Pérdida de relaciones desde archivos no modificados |
| U7 `version` validada como string | No acredita negociación/migración exhaustiva de versiones | Aceptar export incompatible como equivalente |
| U9 ruta normalizada + allowlist; `stat/readFile` | Control lexical localizado, no validación física demostrada | Hipótesis symlink a revisar, no vulnerabilidad reproducida |

No se consultaron issues remotos ni se ejecutaron tests.
El dashboard genera token y protege solicitudes de datos; MCP permite resolver
proyecto desde argumento/env/cwd (`realpathSync`) y posee herramienta de arrancar
dashboard. El conector propuesto no debería conceder ese poder.
La persistencia local JSON no hace offline al análisis LLM, y el permiso de un
proceso local no es aislamiento multi-tenant. No se auditó red del agente anfitrión.

## 4. Prerrequisitos VantaDB

1. Revalidar fuente y FIND-299 en árbol concurrente; baseline HEAD
   `c561158253bb88192590fa406c61e0a27dd7df8d`. No convertir reportes anteriores
   sobre bugs en estado vigente sin nueva comprobación.
2. Separar evidencia de fuente, explicación y relación inferida. `Derived`
   en `MemoryInput` exige `derived_from` y prohíbe confidence explícita.
3. Namespace por repo/productor, revisión inmutable, mapping de IDs y manifiesto
   de ownership. No borrar memorias manuales al reconciliar un snapshot externo.
4. Reutilizar [wiki/ingestors.rs](../../../../src/wiki/ingestors.rs), líneas 1–27,
   como precedente de metadata plana; código no es ingesta documental genérica.
5. Definir schema/version del conector y fallo explícito para incompatibilidades,
   paths físicos permitidos y presupuesto único de contexto. No aprobar
   implementación antes de contrastar el seam existente.

## 5. Componentes que no trasladar

No incorporar pipeline de agentes/LLM, sistema de plugins/parsers, servidor Vite,
viewer completo, búsqueda Fuse o persistencia JSON paralela al motor.
No adoptar `weight=0.8` como exactitud de llamadas ni fingerprint estructural
como garantía de semántica idéntica. MIT local leída; revisar atribución y
dependencias por archivo antes de reutilizar. **Reescribir TypeScript en Rust
no elimina obligaciones de material derivado ni de licencias transitivas.**

## 6. Método y ledger de alcance

Se aplicó [METHOD](METHOD.md), documentación del proyecto y lectura estática
del SHA local. No ejecución, indexación, servicios, instalaciones o cambios MCP.
Los prefijos U1–U9 son referencias de este informe, no IDs de backlog nuevos.

| Ref. | Archivos de fuente leídos; enlaces fijados al SHA |
|---|---|
| U1 | [types.ts:61–122](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/types.ts#L61-L122): node/edge/project/graph |
| U2 | [graph-builder.ts:60–207](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/analyzer/graph-builder.ts#L60-L207): IDs, contains/imports/calls |
| U3 | [fingerprint.ts:80–110](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/fingerprint.ts#L80-L110), [142–250](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/fingerprint.ts#L142-L250): hash y comparación |
| U4 | [staleness.ts:1–110](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/staleness.ts#L1-L110), [472–508](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/staleness.ts#L472-L508): freshness y merge |
| U5 | [context-builder.ts:25–76](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/src/context-builder.ts#L25-L76): expansión contextual |
| U6 | [diff-analyzer.ts:22–87](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/src/diff-analyzer.ts#L22-L87): mapeo y vecinos |
| U7 | [schema.ts:440–499](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/schema.ts#L440-L499), [normalizeGraph/validateGraph](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/schema.ts#L516-L590): validación y normalización |
| U8 | [GraphView.tsx:97–239](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/dashboard/src/components/GraphView.tsx#L97-L239): tipos visibles, tour y selección |
| U9 | [vite.config.ts:15–83](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/dashboard/vite.config.ts#L15-L83), [125–184](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/dashboard/vite.config.ts#L125-L184): token y lectura de fuente |

Además se leyeron `core/src/search.ts:1–73` (Fuse/OR), `persistence/index.ts:1–195`
(resolución legacy `.understand-anything` antes de `.ua`, JSON validado al cargar),
`analyzer/llm-analyzer.ts` (constructores de prompt/parseo), MCP `resolveProjectRoot`
y helpers, y comienzo de `core/src/__tests__/staleness.test.ts`; nada ejecutado.
**Omisiones:** `languages/`, plugins/Figma, todos los agentes/skills/hooks, viewer
empaquetado, pruebas completas de sanitización, embedding search, red y runtime.
No se certifica migración entre schemas o identidad estable tras movimientos.

## 7. Benchmarks existentes

Runner real [scripts/benchmark-large-repo.mjs:1–67](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/scripts/benchmark-large-repo.mjs#L1-L67)
delega en [large-repo-benchmark.mjs:288–302](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/scripts/lib/large-repo-benchmark.mjs#L288-L302).
Entrada: repo local, salida JSON requerida, etiqueta/concurrencia y opción de
conservar artefactos. V1 declara helpers deterministas únicamente, sin LLM;
etapas scan/imports/batching/structure, artefactos temporales fuera del sujeto.
[Métricas de etapas](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/scripts/lib/large-repo-benchmark.mjs#L1938-L1969)
incluyen duración y máximo RSS de workers; un máximo por worker no es RSS
agregado simultáneo del sistema. Depende de Node, paquetes/helpers y parsers.
No mide calidad de resúmenes, costes LLM, ahorro de llamadas o exactitud QA.

`scripts/generate-large-graph.mjs` se identifica como generador sintético en la
documentación del proyecto: no sustituye un corpus real ni demuestra extracción.
No se ejecutó ninguno ni se reportan cifras de performance.
Para comparar tokens/llamadas de agentes, seguir
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks): capturas de cliente,
mismo modelo/presupuesto y sesiones aisladas, separación importación/retrieval/QA,
fallos y no-match visibles. No se diseñó harness o política de scoring adicional.

## 8. Matriz frente a VantaDB

| Capacidad | Estado Vanta | Delta candidato |
|---|---|---|
| Memoria/procedencia/retrieval/context assembly | Existe, baseline METHOD | Reutilizar |
| Símbolos y grafo de código | Existe slice/scanners, AST completo no acreditado | Productor externo opcional |
| Explicación/layers/tours sobre código | Parcial: escenas/wiki; esta UX no acreditada | Vistas derivadas con evidencia |
| Proveniencia por arista AST/LLM | Parcial en Vanta; export U1 tampoco la acredita | Enriquecer contrato, no copiar omisión |
| Hash/reconciliación | Existe por archivo | Fresh/dirty/stale/unknown y generación |
| Budget final tras expansión de hubs | Context assembly existe; integración code requiere validar | Reusar presupuesto global |
| Selección exhaustiva de tests | No acreditada por diff de un salto | No usarlo como gate |
| Dashboard en producto Vanta | Studio existe en otro worktree | Adaptador visual, no segundo servidor |

## 9. Debilidades sugeridas

Vanta no debe degradar su procedencia canónica para encajar en un export más pobre.
La ausencia de metadata rica en `add_edge` sugiere evidencia asociada, no otro core.
En `code_index.rs:310–315` las referencias tienen peso 1 sin resolver AST:
etiquetarlas claramente evita que interfaz o agente las interpreten como llamadas
probadas. Revalidar invalidación interarchivo, borrados y moves antes de adoptar
semántica persistente. Son líneas de revisión, no bugs históricos confirmados.

## 10. Interfaz y disponibilidad

La fuente del dashboard está incluida: `packages/dashboard/src/components/`
contiene `GraphView`, `DomainGraphView`, `KnowledgeGraphView`, `CodeViewer` y
`LearnPanel`; U8 permite selección/tour y organización por capas.
El dev server U9 ofrece fuente con token y allowlist del grafo; esto no autoriza
un endpoint de lectura arbitraria. MCP también puede iniciar viewer/dashboard,
pero esa herramienta no se llamó.
Hay paquete viewer separado y dashboard web, no escritorio nativo acreditado.
Lo reutilizable es mostrar revisión/freshness, navegar a evidencia y explicar
agrupaciones; no portar React Flow entero ni negar la UI existente de Studio.
