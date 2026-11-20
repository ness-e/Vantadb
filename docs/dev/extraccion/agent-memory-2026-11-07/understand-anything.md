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
- [11. Dossier ampliado de ejecución, persistencia e incidencias](#11-dossier-ampliado-de-ejecución-persistencia-e-incidencias)
- [12. Contratos de adopción y cierre](#12-contratos-de-adopción-y-cierre)

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

**Vanta (baseline revalidado `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`):** [code_index.rs](../../../../vantadb-mcp/src/code_index.rs),
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
- **Contrato Rust propuesto:** único `CodeEvidenceImport`, C12 de
  [CONSOLIDATED](CONSOLIDATED.md#c12--codeevidenceimport-contrato-canónico-único-de-evidencia-code-memory).
  Layers/tour son derivados asociados a sus referencias, no otro
  `CodeExplanationImport` ni otra memoria. Si el export no acredita origen,
  usar `Unknown`, no `Ast`; el peso queda reportado, no confianza canónica.
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

La ampliación §11 sí lee incidencias remotas; no se ejecutaron tests.
El dashboard genera token y protege solicitudes de datos; MCP permite resolver
proyecto desde argumento/env/cwd (`realpathSync`) y posee herramienta de arrancar
dashboard. El conector propuesto no debería conceder ese poder.
La persistencia local JSON no hace offline al análisis LLM, y el permiso de un
proceso local no es aislamiento multi-tenant. No se auditó red del agente anfitrión.

## 4. Prerrequisitos VantaDB

1. Fuente revalidada en baseline `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`;
   `c561158`/dirty es histórico. Revalidar FIND-299 antes de spec. No convertir reportes anteriores
   sobre bugs en estado vigente sin nueva comprobación.
2. Separar evidencia de fuente, explicación y relación inferida.
   `MemoryInput` es **struct**: `confidence_class: Option<ConfidenceClass>`
   puede ser `Some(ConfidenceClass::Derived)`; requiere `derived_from` y
   prohíbe confidence explícita. No existe constructor `MemoryInput::Derived`.
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

Runner real [scripts/benchmark-large-repo.mjs:1–64](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/scripts/benchmark-large-repo.mjs#L1-L64)
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

## 11. Dossier ampliado de ejecución, persistencia e incidencias

### 11.1 Mapa y contratos de almacenamiento

Continuación sobre `C:\Users\Eros\Understand-Anything`,
`a6ea4024767df85ce4f99140d4f12ecfc5a81eb6`, según manifest fijado.
No se ejecutó el plugin ni se refrescó el clon. Consulta pública: **2026-11-07**.

| Área | Responsabilidad y frontera comprobada |
|---|---|
| `packages/core/src/analyzer`, `languages/` | Estructura tree-sitter y composición del grafo; cobertura de todos los lenguajes no auditada |
| `core/src/types.ts`, `schema.ts` | Nodos, aristas, capas y tours; normalización/validación no acreditan verdad de una explicación |
| `core/src/persistence/index.ts` | JSON local, metadata/fingerprints/config separados; no motor transaccional de memoria |
| `core/src/fingerprint.ts`, `staleness.ts` | Comparación de fuentes y frescura; HEAD, bytes dirty y versión del productor son dimensiones distintas |
| `skills/understand`, `agents/`, helpers | Scan/batches/análisis/merge/layers/tour; instrucciones del agente anfitrión pueden invocar modelos y arrancar UI |
| `src/context-builder.ts`, `diff-analyzer.ts`, `onboard-builder.ts` | Vistas de consulta, impacto y guía; no aislamiento de ejecución |
| `packages/mcp/bin/understand-anything-mcp.mjs` | MCP stdio, carga de grafo, avisos freshness y lanzamiento opt-in del dashboard |
| `packages/dashboard`, `packages/viewer` | Web React/React Flow frente a distribución viewer; no escritorio nativo acreditado |
| `scripts/benchmark-large-repo.mjs`, `scripts/lib/` | Pipeline determinista para escala, distinto del pipeline LLM de producto |

`KnowledgeGraph` contiene version/kind/project/nodes/edges/layers/tour;
`ProjectMeta` declara commit y analyzedAt. `Layer` y `TourStep` mantienen listas
de nodeIds, no pruebas independientes de esos nodos. `AnalysisMeta` y
fingerprints son sidecars: tener meta reciente no prueba que JSON y todos los
resúmenes estén alineados. No se acreditaron tenant/ACL/sesiones/supersession
equivalentes a Vanta en este productor.

[persistence/index.ts:19–144](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/persistence/index.ts#L19-L144)
prioriza directorio legacy si existe y guarda con `writeFileSync` directo.
`saveGraph` sanea rutas absolutas; `saveMeta`/`saveFingerprints` escriben otros
archivos. No hay en esos helpers lock/CAS/rename/fsync ni commit compuesto.
Eso **no prueba** que todo caller carezca de protección, pero impide importar
su mera presencia como garantía de snapshot indivisible.
`loadGraph` devuelve null si falta, lanza por JSON inválido y valida por defecto;
`loadFingerprints` captura parseo y devuelve null. El consumidor debe retener
“ilegible” separado de “no existe”, aunque upstream use el mismo resultado.

MIT raíz leída: copyright Yuxiang Lin e Infinite Universe, Inc., 2026.
Core/dashboard/MCP/viewer pertenecen al árbol cubierto por esa declaración;
no se auditó cada cabecera ni las licencias de WASM/gramáticas/dependencias.
Redistribuir viewer añade dependencias/artefactos que no quedan autorizados
por citar solo MIT raíz. Reescritura Rust no elimina obligaciones derivadas.

### 11.2 Flujos trazados y puntos de fallo

1. **Selección de fuente:** [skills/understand/SKILL.md:47–73](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/skills/understand/SKILL.md#L47-L73)
   resuelve `PROJECT_ROOT`; al detectar worktree puede reasignarlo al checkout
   principal salvo opt-out. No confundir root de artefactos durables con root
   de bytes solicitados. Las instrucciones son evidencia documental del flujo,
   no ejecución observada del agente.
2. **Estructura/escritura:** U2 `GraphBuilder` reúne nodos/relaciones; agentes
   escriben batches que `merge-batch-graphs.py` normaliza. Su
   [normalize_node_id:233–276](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/skills/understand/merge-batch-graphs.py#L233-L276)
   elimina ciertos prefijos, trata alias func y añade prefix según tipo si falta.
   No equivale al esquema TypeScript ni prueba identidad estable.
3. **Publicación:** `saveGraph` serializa JSON y sidecars por separado (§11.1).
   Otro lector/proceso puede necesitar comprobar que todos corresponden a una
   generación; el helper no proporciona ese recibo. Kill a mitad de escritura
   o dos writers son casos pendientes, no resultados medidos.
4. **Consulta:** `SearchEngine.search` en
   [search.ts:14–64](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/core/src/search.ts#L14-L64)
   busca nombre/tags/summary/languageNotes con Fuse, OR de términos, tipos
   opcionales y límite default 50. Score bajo es mejor match, **no confidence**.
   U5 limita seeds y luego añade vecinos; el presupuesto debe aplicarse al
   resultado expandido. No sustituir BM25/HNSW/RRF por esta búsqueda.
5. **MCP/error:** [resolveProjectRoot/requireGraph/freshnessSummary:54–139](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/mcp/bin/understand-anything-mcp.mjs#L54-L139)
   canonicaliza root con realpath; carga fallida devuelve error MCP. Frescura
   fallida devuelve unknown; stale/dirty genera aviso. Es advertencia, no prueba
   de que toda herramienta rehúse contexto viejo ni autorización multitenant.
6. **Update/delete:** U4 `mergeGraphUpdate:472–507` elimina nodos de archivos
   cambiados y todas sus aristas incidentes, añade newNodes/newEdges y actualiza
   commit/fecha. Conserva `...existingGraph`, incluidas layers/tour. No demuestra
   recuperación de edges desde callers no cambiados ni remapeo de esos derivados.
   Delete se puede representar sin nodos nuevos; el helper no prueba el caller
   que detecta todas las bajas. Move necesita correspondencia explícita.
7. **Derivado:** [buildOnboardingGuide:7–123](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/src/onboard-builder.ts#L7-L123)
   renderiza arquitectura/conceptos/tour/mapa/hotspots desde el grafo.
   IDs no encontrados se filtran; una guía legible puede ocultar referencias
   faltantes. No hay LLM ni nueva persistencia en ese builder.
8. **Cancelación:** builders síncronos no acreditan propagación de cancelación
   a todos los agentes/subprocesos. UI cancela animation frame/timers de foco
   (U8); eso no cancela extracción compartida. Ninguna consulta read-only del
   conector propuesto recibe permiso de arrancar dashboard o reconstruir grafo.

### 11.3 Incidencias públicas efectivamente leídas

Router: comparación Bing/DDG de issues devolvió cero; API pública #167 devolvió
rate-limit. Fallback HTML leyó cuerpo #167/#758/#739 y listado oficial de issues.
El [listado](https://github.com/Egonex-AI/Understand-Anything/issues) confirmó
**open**, apertura **2026-11-05** para #758 y **2026-10-23** para #739,
observados 2026-11-07. La revisión independiente completa #167 mediante
[API oficial](https://api.github.com/repos/Egonex-AI/Understand-Anything/issues/167):
closed, creada 2026-05-21T17:14:29Z, cerrada 2026-05-24T13:12:59Z. Su
[comentario](https://github.com/Egonex-AI/Understand-Anything/issues/167#issuecomment-4513808773)
atribuye lectura literal de `inherit`, sin diff causal; cierre no prueba fix.
Fuente/fecha: [recibo independiente](reviews/memory-code-remaining.md#understand-anything),
2026-11-07, no nueva reproducción.

| Fuente | Mecanismo y alcance reportados | Contraste y validación futura |
|---|---|---|
| [#167](https://github.com/Egonex-AI/Understand-Anything/issues/167) | `ProviderModelNotFoundError` del project-scanner en OpenCode/DeepSeek v4; retry vuelve a fallar | CLAUDE local atribuye causa a `model: inherit` específico de Claude y dice omitir campo. El cuerpo solo acredita reporte, no esa causalidad ni merge. Adapter read-only evita dependencia de modelo; no fingir enriquecimiento completo si falla |
| [#758](https://github.com/Egonex-AI/Understand-Anything/issues/758) | Plugin 2.9.8/macOS/Python 3.13: `script:...` termina como `file:script:...`, edges caen como missing source, layers/tours heredan IDs | Función fijada conserva la secuencia de normalización descrita; no se ejecutó repro ni se auditó toda PR. Propuesta de PR/tests del autor **no acredita merge**. Validar tipo/ID/endpoints y mapping de derivados |
| [#739](https://github.com/Egonex-AI/Understand-Anything/issues/739) | En `e24fcf6d...`, 2.9.7, redirección worktree cambia fuente y omite archivo branch-only; repro de scanner, no calidad LLM | Snapshot estudiado aún contiene reasignación `PROJECT_ROOT` (§11.2); no se siguió toda ejecución anfitriona. Separar source/output y atar Git metadata a fuente. No extrapolar a cualquier root MCP |

Los resultados y tests que citan los autores son reportes, no ejecución propia.
#133 no se leyó: su cierre mencionado en #739 no demuestra solución de este
problema distinto. No se leyeron todos los comentarios ni PRs correctivos;
quedan lagunas de historia y compatibilidad de versiones.

### 11.4 Runner de escala: defaults, denominadores y efectos

Lectura adicional de `scripts/lib/large-repo-benchmark.mjs`, al SHA fijado:
[parseArgs:182–283](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/scripts/lib/large-repo-benchmark.mjs#L182-L283)
requiere repo y output, genera nombre Markdown asociado y rechaza outputs dentro
del sujeto. Concurrencia default **5**, rango permitido 1–32; `keep-artifacts`
es false. Son defaults upstream, no configuración de evaluación nueva.

[runBenchmark:1671–1741](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/scripts/lib/large-repo-benchmark.mjs#L1671-L1741)
crea temporal fuera del sujeto, metadata Git separada tool/subject, modo
deterministic y `llm.invoked:false`; fases scan/imports/batching/structure.
No sesiones/preguntas/gold/split/juez. Unidad es archivo/batch/entidad, no recall QA.
[1938–2058](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/scripts/lib/large-repo-benchmark.mjs#L1938-L2058)
registra batchesSucceeded/Failed, filesAnalyzed/Skipped, structure/callGraph
success/failure/skipped y distribución de duración de batches. `maxWorkerPeakRssBytes`
no es RSS total simultáneo. Digest determinista no prueba semántica completa.

Fallo de integrity produce failed/exit 1; warnings o archivos skipped pueden
producir **degraded/exit 0**. No usar exit 0 como cobertura íntegra.
Catch conserva error, finally termina reloj; duración general allí calculada
**precede cleanup y entrega de reportes**. Cleanup fallido agrega secondaryErrors,
marca failed y exit 1. Report write puede lanzar error de entrega, no entregar
un JSON completo por milagro. Escribe JSON+Markdown y borra temporales salvo
keep; etapas corren subprocesos Node. Ninguno se ejecutó.

Adaptación a Vanta bajo METHOD/BENCHMARK-NOTES: comparar coste de preparación
solo con pista motor/producto equivalente; C12 aporta refs revisionadas al
consumidor, no reemplaza extractor ni los denominadores de este runner.
No usar estos tiempos como ahorro LLM ni el generador de grafos falsos como
dataset de extracción. No se propusieron harness, prompts, scoring o umbrales.

## 12. Contratos de adopción y cierre

### 12.1 Terminación de A–C y priorización

**Todo puerto/tipo de esta sección es hipotético, no código Vanta presente.**
Único contrato canónico de importación: C12 `CodeEvidenceImport`. Snapshot,
invalidation y view slice de B/C son controles/vistas sobre ese contrato,
no importadores persistentes alternativos.

| Ficha / escenario | Alternativas y delta | Commit, ownership, cancelación y desactivación propuestos |
|---|---|---|
| A: agente consume explicación sin promoverla a hecho | Ampliar scanner, adapter opcional o rechazar export sin evidencia. Preferir adapter conservando Unknown cuando corresponda | Namespace productor/repo; original y mapping retenidos, validar antes de publicar. CAS de generación y evidencia primero. Cancelación previa descarta staging; posterior consulta recibo. Rollback a generación previa, nunca delete de memorias humanas |
| B: resumen viejo sobre cuerpo cambiado | Hash completo, fingerprint estructural o revisión Git sola. Hash de bytes más productor/revisión evita equiparar cosmetic con igualdad semántica | Invalidar propios derivados, no volver fresh antes de finish. Fallo parcial conserva anterior stale/unknown. Jobs/checkpoints ya existen. Migración sin owner solo diagnostica; medir razones/lag, no contenido |
| C: agente inspecciona contexto y diff | Links a fuente y vistas Studio frente a portar dashboard. Un salto es ayuda, no cierre exhaustivo ni selección segura de tests | Query ligada a generación y budget final; no commit. Cancelación da incomplete/descarta respuesta vieja, no cancela writer compartido. Desactivar panel conserva evidencia. Denegar roots ajenos y no devolver error como vacío |

Prioridad: **A/B/F antes de importación persistente**; **E antes de confiar
en IDs saneados**; **C/D después para contexto/enseñanza**. Read-only con estado
unknown puede evaluarse antes; no bloquear todo Vanta por trabajo opcional de UI.
Dependencias reales: `code_index`, `MemoryInput`, `add_edge`, assembly y jobs;
no construir nuevos BM25, budgets, scheduler, dedup ni checkpoints.

### 12.2 Ficha D — Capas y tours como derivados invalidables

- **Caso/fuente:** onboarding de un agente nuevo sin cargar todo repo. Unidad
  `onboard-builder.ts:7–123`, U1 layers/tour y U4 merge: guía reutilizable, pero
  IDs eliminados pueden desaparecer del texto sin aviso.
- **Delta/seam:** escenas/wiki/context assembly Vanta ya existen; delta es
  guía de código con dependencias a refs C12, no otra memoria conversacional.
- **Contrato:** `render_code_guide(generation, authorized_refs, budget) ->
  Result<GuideView, GuideError>`; referencias faltantes/stale se exponen.
  Si se materializa, el **struct** `MemoryInput` usa
  `confidence_class: Some(ConfidenceClass::Derived)` y `derived_from` con
  padres válidos del mismo namespace, sin confidence explícita
  [V-derived-input]; no se puede usar guía como su propia fuente.
- **Pasos/alternativas:** mapear nodes → verificar dependencias → proyectar
  layers/tour → aplicar budget global → renderizar refs/estado. Alternativa
  mínima: guía efímera; persistencia solo si lifecycle B está acordado.
- **Ownership/commit/deps:** D depende de A/B/E, writer de derivados existente;
  invalidar antes de declarar vigente. Cancelación no publica guía parcial como
  completa; rollback/desactivación elimina proyección propia, no fuentes.
- **FMEA:** ID borrado queda en tour → detectar referencia ausente y advertir;
  renumerar layer parece identidad estable → digest de miembros; explicación
  hostil instruye al agente → contenido citado, nunca autoridad de herramientas.
  Materializar con clase/linaje inválidos no se subsana inventando un enum o
  una confidence: conservar error del writer, sin anunciar guía publicada.
- **Aceptación manual:** delete/rename/body-change invalida guía; faltantes
  visibles; ciclos de derivación rechazados; observar conteos/estado sin resúmenes.
  La ficha identifica campos reales del struct, no un constructor invocable;
  rechazo de padres/clase no se presenta como guía vacía ni publicación completa.
- **No objetivos:** aprender arquitectura verdadera por clasificación de complejidad,
  copiar prompts de agentes o introducir evaluator/umbral pedagógico.

### 12.3 Ficha E — Mapping explícito y validación sin reparación silenciosa

- **Caso/unidad:** agente pide símbolo cuyo ID fue “corregido” y obtiene vacío.
  `normalize_node_id:233–276`, #758 y U7 sustentan separación original/canónico.
- **Delta/seam:** claves deterministas `file:`/`sym:` existen en `code_index`;
  no reinterpretar prefijos externos como claves internas ni exigir mismos IDs.
- **Contrato:** `map_producer_refs(&CodeEvidenceImport, mapping, schema) ->
  Result<MappingReport, ImportError>`. Conservar original, tipo, ruta, revisión,
  alias justificado y rechazos; `UnknownType/Collision/DanglingRef` no son éxito.
- **Pasos/alternativas:** validar export original → construir mapping una vez →
  comprobar endpoints/layers/tours → importar A → publicar B. Rechazo explícito
  es alternativa a corregir ambiguamente; normalización upstream no autoriza
  unir overloads ni rutas ajenas.
- **Deps/ownership/rollback:** A/B, namespace productor y schema acordado. Legacy
  sin mapping queda unknown; no migración destructiva por parecido. Dos writers
  no comparten mapping mutable; cancelar descarta plan, rollback usa anterior.
- **FMEA/aceptación:** doble prefijo crea entidad ajena → colisión/round-trip
  detectable; endpoint ausente → rechazar relación sin inventarlo. Caso manual:
  `script`, `config`, homónimos y reimport mantienen mapping estable o error
  trazable. Métricas de remapeos/rechazos sin rutas sensibles.
- **No objetivos:** copiar reparador Python completo ni deducir proveniencia AST
  desde un ID bien formado.

### 12.4 Ficha F — Separar identidad de fuente y destino de artefactos

- **Caso/fuente:** agente analiza branch feature y consume evidencia de main.
  §11.2 `SKILL.md:47–73`, #739 y MCP `resolveProjectRoot` muestran dos rutas
  distintas: no generalizar fallo de una instrucción a todas las APIs.
- **Delta/seam:** `index_repo` recibe root y namespace; C12 ya propone repo y
  revision. Falta especificar correspondencia repo/worktree/digest para conector.
- **Contrato:** `resolve_code_source(requested_root, declared_identity, auth)
  -> Result<AuthorizedSource, SourceError>`; destino de staging es control
  separado, nunca motivo para cambiar bytes analizados. Tipo de aplicación,
  no segundo DTO de importación.
- **Pasos/alternativas:** autorizar raíz física → registrar revisión/digest de
  esa fuente → leer export → comprobar identidad → importar A/B. Mínimo: export
  manual con fuente explícita; rechazar auto-redirect silencioso.
- **Concurrency/commit/rollback:** fijar root por trabajo; cambio de rama durante
  lectura produce conflicto/unknown, no evidencia fresh. Cancelar antes de
  publicación conserva previo; otro worktree tiene ownership separado. Desactivar
  auto-import no mueve ni elimina fuente; depende de A/B, no de instalar hooks.
- **FMEA/aceptación:** root principal confunde repo común con revisión → detectar
  fuente/commit discrepante; symlink escapa → autorización física. Futuro caso
  branch-only+dirty+dos worktrees conserva diferencias. Observar mismatch sin
  publicar roots secretos. No crear ni destruir worktrees desde el adaptador.

### 12.5 UI, matriz revisada, negativos y cobertura

Flujo web leído en [App.tsx:144–194](https://github.com/Egonex-AI/Understand-Anything/blob/a6ea4024767df85ce4f99140d4f12ecfc5a81eb6/understand-anything-plugin/packages/dashboard/src/App.tsx#L144-L194):
fetch con token → comprobar HTTP → validar grafo → Zustand → GraphView;
fallo de transporte y formato son distintos. U8 navega selección/tour y cancela
foco anterior. U9 permite fuente solo por path local/allowlist/tamaño/no binario;
stat/read no acredita defensa física completa frente a symlink. Cualquier
adopción sería extensión Studio, no incorporar dev server o compartir token en logs.
Seam Studio releído con `git show`:
[feature-freez/desktop/src/components/memory/MemoryLens.tsx:381-409](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L409)
consulta genlog y abre anchors; fuente inmutable 2026-11-07, no bytes dirty
de `desktop/...` ni una guía/tour de D ya implementada.

| Capacidad | Estado Vanta fijado | Decisión y fuente |
|---|---|---|
| Scanners/hash/símbolos/grafo | Existe | `code_index.rs:152–327,514–520`; conservar, no atribuir AST completo |
| Provenance y memoria derivada | Existe; parcial para export por arista | `record.rs:184–224`, `graph.rs:113–151`; asociar evidencia, no confianza arbitraria |
| Publicación completa por productor/root/generación | No acreditado | A/B/F requieren contrato; firma de `add_edge` no prueba commit compuesto |
| Guía por código con capas/tour y dependencias | Parcial | Escenas/wiki/assembly existen; D agrega vista específica; persistencia usa campos `confidence_class/derived_from` de `MemoryInput`, no enum nuevo |
| Budget, jobs, sesiones y Studio | Existe | VANTA-REFERENCE; no recrearlos desde plugin |
| Portar Fuse/React Flow/agent pipeline | No aplicable | Objetivo de conector, no reemplazo de motor o aplicación |

Relectura Vanta `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`:
`code_index.rs:230–233,310–315` ignora algunas mutaciones con `let _`; contador
de stale removidos no certifica éxito de delete. Además file manifest/hash se
guarda antes de todos los símbolos; fallo posterior podría impedir recuperación
por retry si éste salta hash igual. Son observación de código e hipótesis
discriminante, respectivamente, **no reproducciones ni bugs universales**.
Futuro caso: error entre file put y symbol put no permite recibo/ACK completo.

Ledger adicional: persistence 19–144; types 61–146; merge Python 233–276;
skill 47–73; MCP 54–139; onboard builder completo; App 144–194 y U9; benchmark
entry point completo y rangos del módulo enlazados; licencia y tres cuerpos de
issues. No auditados: todos los agentes/merge helpers, providers, parsers,
sanitización física completa, viewer empaquetado, delivery/recovery completo
del runner y todos los comentarios/PRs. No se certifica crash safety, UI, calidad
o ejecución. Autoría no es revisión independiente ni aceptación final.
Corrección [UA-1](reviews/memory-code-remaining.md#understand-anything)
propagada a prerrequisito/D/matriz/aceptación; inspector de guía debe mostrar
rechazo de linaje sin aparentar guía publicada. Pendiente de reconfirmación
independiente; “sin bloqueantes” en la revisión no es aceptación final.

[V-derived-input]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/types/record.rs#L184-L224
Si se aprueba implementación, separar specs A/E/importación, B/F/frescura e
identidad y C/D/Studio/guías; este dossier >300 líneas conserva TOC y evidencias.
