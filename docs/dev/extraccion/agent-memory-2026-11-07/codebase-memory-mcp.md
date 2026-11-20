---
title: "codebase-memory-mcp: evidencia estructural para memoria canónica"
kind: research
status: proposed
description: "Lectura estática de indexación, procedencia, consultas acotadas y selección de tests como conectores opcionales para VantaDB"
tags: [research, memory, engineering]
---

# codebase-memory-mcp: evidencia estructural para memoria canónica

## Índice

- [1. Dictamen y arquitectura](#1-dictamen-y-arquitectura)
- [2. Unidades adaptables](#2-unidades-adaptables)
- [3. Riesgos y defectos transferibles](#3-riesgos-y-defectos-transferibles)
- [4. Prerrequisitos VantaDB](#4-prerrequisitos-vantadb)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Método y ledger de evidencia](#6-método-y-ledger-de-evidencia)
- [7. Benchmarks existentes](#7-benchmarks-existentes)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades sugeridas](#9-debilidades-sugeridas)
- [10. Interfaz e inspección](#10-interfaz-e-inspección)
- [11. Dossier ampliado: ejecución, reconciliación e incidencias](#11-dossier-ampliado-ejecución-reconciliación-e-incidencias)
- [12. Contratos de adopción y cierre](#12-contratos-de-adopción-y-cierre)

## 1. Dictamen y arquitectura

**Propuesta:** consumir evidencia de un indexador externo cuando el usuario ya lo
utilice; no incorporar otro motor de grafos ni reproducir su universo de parsers.
El valor diferencial está en contratos de resolución explicable y cobertura,
no en reemplazar BM25/HNSW/RRF, memoria, procedencia o context assembly de Vanta.

- Repositorio: `DeusData/codebase-memory-mcp`.
- HEAD local verificado: `612eee2518ef84cc573e491fdee5fa5c90264cb5`.
- Snapshot [congelado](github-candidates.json): **48 254 estrellas**,
  `2026-11-07T06:56:07.0889641Z`, licencia detectada MIT.
- Código: backend C, tree-sitter y parsers vendorizados, resolución híbrida LSP,
  SQLite y FTS5, JSON-RPC/MCP, daemon/watcher; frontend TypeScript/React CBM Lens.
  La búsqueda semántica local no equivale a aristas inventadas por un LLM.
- Flujo observado: extracción → buffer de grafo → passes de resolución/enriquecimiento
  → persistencia por proyecto → consultas MCP → snippets y exploración visual.
  `emit_classified_edge` conserva `callee`, `strategy`, `confidence`, `candidates`.
  AST determinista no garantiza resolución estática completa de dispatch dinámico.

**Baseline revalidado, no asumido:** Vanta tiene `code_index`, `code_*`, registros
`file:`/`sym:`, `defines`, referencias heurísticas y reconciliación por hash.
La implementación revalidada en `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` de `extract_symbols_for_lang` también enruta
Python/TypeScript/JavaScript, aunque el comentario de cabecera aún dice Rust v0.
Véanse [code_index.rs](../../../../vantadb-mcp/src/code_index.rs), líneas 193–315,
514–520; [code.rs](../../../../vantadb-mcp/src/code.rs), líneas 1–36.
Eso no acredita paridad con un code graph AST multiidioma completo.
`FIND-299` (en `Backlog.md` (registro local, no versionado por git)), línea 285, y la
`mgr-22-repo-map` (registro historico, sin destino localizable en el repo; ver FIND-299 en `Backlog.md` (registro local, no versionado por git)) son el seam existente.
Las herramientas externas instaladas en el entorno del desarrollador no prueban
integración automática en el producto VantaDB.

## 2. Unidades adaptables

Las siguientes fichas son contratos propuestos, no código aprobado ni tareas nuevas.

### Ficha A — Importar evidencia de resolución, no IDs internos

- **Problema/unidad mínima:** transportar una arista y sus evidencias desde
  `emit_classified_edge` (`src/pipeline/pass_calls.c:837–874`, C1).
  `init_schema` (`src/store/store.c:363–410`, C3) usa IDs SQLite autoincrementales
  y unicidad `(project, qualified_name)`; no son identidad portátil entre rebuilds.
- **Seam real:** `MemoryInput` en
  [record.rs](../../../../src/sdk/types/record.rs), líneas 184–224;
  `Embedded::add_edge` en [graph.rs](../../../../src/sdk/api/graph.rs),
  líneas 113–151. La API de arista recibe label/peso/fecha, no un objeto
  arbitrario de procedencia: conservar una memoria-evidencia vinculada.
- **Contrato Rust propuesto:** exclusivamente `CodeEvidenceImport`, C12 de
  [CONSOLIDATED](CONSOLIDATED.md#c12--codeevidenceimport-contrato-canónico-único-de-evidencia-code-memory).
  Mapear qname/ruta/rango a `refs`, estrategia/candidatos a evidencia asociada,
  confianza reportada sin convertirla en confianza canónica. Resuelto, ambiguo
  y externo son estados distintos. No se define otro DTO de importación.
- **Pasos/dependencias:** (1) exportación o consulta read-only explícita;
  (2) validar proyecto, revisión y rutas; (3) resolver IDs externos a claves
  Vanta namespaced; (4) persistir evidencia antes de publicar relaciones;
  (5) registrar lote/generación y rechazos. Depende de identidad de repositorio,
  manifiesto de importación y reglas de procedencia canónicas.
- **FMEA:** reciclar un ID numérico enlaza símbolos ajenos; perder `strategy`
  convierte inferencia en hecho; mover un archivo cambia su contexto de nombre.
  Contención: conservar referencia original y revisión; no fusionar por nombre corto.
- **Aceptación futura:** importar dos veces no duplica evidencia; homónimos de
  proyectos distintos no colisionan; unresolved no crea un destino supuesto;
  una referencia permite abrir exactamente su fuente versionada.
- **No objetivos:** traducir parsers/LSP a Rust, copiar SQLite o hacer que
  `confidence=1` signifique certeza semántica.

### Ficha B — Respuesta acotada y snapshot explícito

- **Unidad:** `cbm_sb_*` y compactación de `src/mcp/compact_out.c` (C4);
  `TOOL_ANNOTATIONS` en `src/mcp/mcp.c:881–944` (C5) separa queries de indexado,
  borrado, ADR y traces. La lectura de una query no debe activar una reconstrucción.
- **Seam:** `code_search`/`code_explore` y ensamblado de contexto ya existentes;
  adaptar presentación, no crear una segunda búsqueda semántica.
- **Contrato Rust propuesto:** `CodeQueryPage { repo_id, generation,
  items, evidence_keys, omitted, continuation: Option<String>,
  budget_exhausted, freshness }`. El cursor pertenece a una generación y consulta;
  tamaño en bytes/caracteres no se anuncia como tokenización exacta.
- **Pasos/dependencias:** fijar snapshot → normalizar consulta sin borrar nombres
  cualificados → seleccionar resultados → cortar unidades completas → devolver
  omisiones y cursor; antes del siguiente page validar snapshot y ACL.
- **FMEA:** un cursor reutilizado tras sync salta filas; comprimir rutas sin regla
  de expansión destruye evidencia; presupuestar solo snippets omite overhead MCP.
  Mantener representación JSON literal para máquinas y referencias reversibles.
- **Aceptación:** paginar un índice sin cambios reconstruye el mismo conjunto;
  cambiar generación invalida continuación; una fila mayor al presupuesto no
  genera un cursor que apunte a sí mismo; omisión no se presenta como ausencia.
  CM-1: rechazar BM25+semantic simultáneos; fallback regex se declara como
  modo efectivo, no score BM25 ni híbrido implícito; no-indexado/error no son
  página vacía. Se conserva la interfaz propuesta, con diagnóstico del adapter.
- **No objetivos:** copiar un formato compacto propietario como schema canónico
  ni prometer ahorro de tokens sin tokenizer y captura del cliente.

### Ficha C — Separar vecindario visual de cierre para selección de tests

- **Unidad:** `cbm_impact_walk_*`, `src/store/store_impact.h:1–105` (C6), y
  `te_lift_fixpoint`, `src/mcp/test_impact_engine_reach.c:1202–1255` (C7).
  El walk inverso permite fixpoint (`max_hops=0`), ciclos con expansión única,
  padres deterministas y errores de asignación en lugar de respuestas recortadas.
  Los lifts incluyen roles/subprocesos; no es solo “tres saltos de callers”.
- **Seam:** `code_impact`, `IMPACT_MAX_DEPTH=10` en Vanta `code.rs:33–36,272–301`.
  Ese límite es correcto para exploración, pero no demuestra selección exhaustiva.
- **DTO propuesto:** `ImpactEvidence { revision, seeds, traversed_relations,
  selected_tests, explanation_paths, completeness, fallback_reason }`;
  `completeness` no puede ser `Complete` si se abortó por recursos o parse gaps.
- **Pasos/dependencias:** importar el informe upstream como evidencia opcional;
  mapear tests a fuentes; comprobar revisión/cobertura; mostrar fallback amplio;
  cualquier uso para excluir tests exige autorización y validación independiente.
- **FMEA:** cierre parcial excluye tests relevantes; una arista `CALL_REFERENCE`
  ignorada pierde callbacks; alcanzar el runner puede implicar ejecutar todo.
  Aplicar fail-open hacia suite amplia, no fail-open hacia menos tests.
- **Aceptación:** ciclos terminan; ambigüedad/cobertura ausente no producen
  selección vacía “segura”; explicación incluye relación y fuente de cada paso.
- **No objetivos:** diseñar aquí un runner de CI o política de selección nueva.

## 3. Riesgos y defectos transferibles

| Evidencia | Riesgo real de adaptación | Tratamiento |
|---|---|---|
| C1 suprime weak member matches con receiver no resuelto | Determinismo AST confundido con exactitud del enlace | Preservar estrategia y unresolved; no fabricar call edge |
| C2 borra nodos de archivos cambiados y cascada de edges | Invalidation local no basta para importadores y enlaces interarchivo | Exigir generación completa o delta con eliminaciones y dependientes |
| C3 IDs autoincrementales y nombres por proyecto | Rebuild/move rompe referencias externas | Mapear identidad por repo/revisión; no persistir ID upstream como ID Vanta |
| C5 expone herramientas de escritura además de lectura | Un conector de consulta obtiene poderes de indexado/borrado | Allowlist read-only y privilegio mínimo; annotations no son autorización |
| C6 distingue fixpoint de BFS limitado | Declarar “tests seguros” desde un vecindario | Informar incompletitud/fallback y no reducir gates |

La ampliación §11 sí consulta incidencias y comentarios upstream; las referencias
`#...` restantes del código no cuentan como incidencias examinadas. Se leyeron contratos de test C8 sin
ejecutarlos. Las garantías de aislamiento por proyecto en SQLite no equivalen a
autenticación multi-tenant: el proceso local conserva permisos del usuario.
“Local” describe indexación/consulta; integraciones, instalador, descargas de
modelos y agente consumidor tienen fronteras de red separadas, no auditadas aquí.

## 4. Prerrequisitos VantaDB

1. Baseline revalidado `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, limpio
   antes de esta edición. `c561158`/dirty es antecedente, no evidencia actual.
   Revalidar FIND-299 antes de una spec; no convertir su existencia en implementación.
2. Acordar repo identity, revisión, root permitido, schema del productor y estados
   fresh/stale/unknown. Un timestamp solo no prueba que el índice represente HEAD.
3. Usar metadatos planos de [ingestors.rs](../../../../src/wiki/ingestors.rs),
   líneas 1–27, como precedente de trazabilidad, no enviar código al ingestor
   documental que explícitamente lo excluye.
4. Separar peso relacional, evidencia y `ConfidenceClass` canónica: las escrituras
   `Derived` exigen padres y prohíben confidence explícita en `MemoryInput`.
5. Publicar generaciones completas y reconciliar borrados sin tocar memorias
   manuales; verificar límites de namespace y clave. No mantener dos verdades activas.

## 5. Qué no trasladar

No embutir daemon, watcher, parsers, LSPs, instalador de agentes o UI completa en
el motor embebido. No reutilizar la selección experimental de tests como gate
de Vanta sin evidencia equivalente. No duplicar FTS5/embeddings donde Vanta ya
tiene retrieval. Licencia local MIT leída; vendored y dependencias requieren
revisión propia de `THIRD_PARTY.md`. **Cambiar C/TypeScript por Rust no exime
de obligaciones de código o material derivado.**

## 6. Método y ledger de evidencia

Lectura estática conforme a [METHOD](METHOD.md), sin ejecutar código, tests,
scripts, benchmarks o servicios; sin indexar estos clones ni modificar MCPs.
Los enlaces siguientes fijan el SHA local comprobado, no una rama móvil.

| ID | Archivo fuente leído y alcance |
|---|---|
| C1 | [pass_calls.c:837–874](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/pipeline/pass_calls.c#L837-L874): emisión, estrategia, candidatos |
| C2 | [pipeline_incremental.c:1–140](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/pipeline/pipeline_incremental.c#L1-L140): delta disco y fallos de alloc |
| C3 | [store.c:363–410](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/store/store.c#L363-L410) y [6342–6370](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/store/store.c#L6342-L6370): esquema/versión `PRAGMA user_version` |
| C4 | [compact_out.c:454–480](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/compact_out.c#L454-L480), [902–910](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/compact_out.c#L902-L910): proxy de forma de tokens, no tokenizer |
| C5 | [mcp.c:881–944](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/mcp.c#L881-L944): anotaciones; corrección amplía a [query 5488–5580][C-query] y [delete_project 8054–8150][C-delete-project] |
| C6 | [store_impact.h:1–105](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/store/store_impact.h#L1-L105): contrato cierre/cancelación |
| C7 | [test_impact_engine_reach.c:21–38](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/test_impact_engine_reach.c#L21-L38), [1202–1255](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/test_impact_engine_reach.c#L1202-L1255): roles y fixpoint |
| C8 | [test_call_reference_contract.c:1–70](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/tests/test_call_reference_contract.c#L1-L70): helpers de contrato end-to-end, no resultado |
| C9 | [App.tsx:1–80](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/graph-ui/src/App.tsx#L1-L80): comentario/imports; no acredita flujo React completo |
| Corrección CM-2 | [watcher ACK y caller 1623–1750][C-watch-ack]; [scheduler real 25–119][V-scheduler] y [TaskKind 38–69][V-task-kinds]; no job code-index acreditado |

**Omisiones:** no auditoría exhaustiva de `internal/cbm`, `vendored`, LSPs,
subprocesos, instaladores, transports daemon/HTTP, trazas runtime ni todo Lens.
No se comprobó migración de un índice antiguo, stable IDs tras moves o crash
consistency. Schema SQLite y versión de formato no prueban compatibilidad universal.

## 7. Benchmarks existentes

- [benchmark-search-graph.sh:1–100](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/scripts/benchmark-search-graph.sh#L1-L100):
  recibe binario/repo; índice inicial no cronometrado, daemon caliente privado,
  consultas `name_pattern`, tiempo con `date`, conteo parseado con Python.
  Mide búsqueda, no calidad de respuesta, importación ni ahorro de llamadas LLM.
- [benchmark-index.sh:1–75](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/scripts/benchmark-index.sh#L1-L75):
  binario, lenguaje, repo y resultados; runtime/cache propios y handoff explícito.
  Arranca/para servicios: **no ejecutado y no apropiado para los servicios del usuario**.
- [MEASURING_SAVINGS.md:1–100](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/docs/MEASURING_SAVINGS.md#L1-L100)
  separa calidad, latencia y consumo del agente; requiere mismo SHA, modelo,
  preguntas, presupuesto y sesiones limpias. Contadores MCP no cuentan tokens
  totales del modelo ni todas las herramientas.
- Los ratios publicados en README/paper son claims upstream no reproducidos aquí.
  No se trasladan a Vanta ni se comparan con Paso 23. Para futura comparación rige
  exclusivamente [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks):
  importación, retrieval y respuesta separados; fallos/abstenciones en denominador;
  registrar tokens de entrada/salida/caché y llamadas del cliente, no solo bytes.

## 8. Matriz frente a VantaDB

| Capacidad | Estado Vanta revalidado | Delta candidato |
|---|---|---|
| Memoria canónica, retrieval y procedencia | Existe, baseline METHOD | Reutilizar, no duplicar |
| `file:`/`sym:`, `defines`, reconcile hash | Existe en `code_index.rs` | Conector con evidencia más rica |
| Extracción AST/LSP multiidioma equivalente | No acreditado; scanners actuales sí existen | Consumir productor externo opcional |
| Arista con estrategia y candidatos persistidos | Parcial: peso/label y memorias con metadata | Registro de evidencia asociado |
| Incremental + watcher integrado completo | Parcial; FIND-299 conserva pendientes | Manifiesto/generación, revalidar estado antes de tarea |
| Consulta BM25/regex y borrado de proyecto | Upstream trazado en §11.2; equivalencia del adapter Vanta no acreditada | B distingue modo efectivo; D retira solo proyección propia, no ADR/memoria humana |
| Job automático de code-index con ACK persistente | No acreditado | Scheduler real [V-scheduler] usa cola local y tareas de memoria, no un job code-index ya conectado; E requiere integrar sin otro coordinador |
| Cierre seguro para excluir tests | No acreditado; `code_impact` es acotado | Solo informe asesor, fallback amplio |
| Lens como escritorio Vanta | No aplicable | UI externa; Studio es otro worktree |

## 9. Debilidades sugeridas

En el baseline revalidado Vanta asigna `Some(1.0)` a referencias por coincidencia textual
y compara los símbolos acumulados de archivos procesados (`code_index.rs:310–315`).
Los archivos sin cambios salen antes (`193–202`). Esto justifica **hipótesis a
revalidar** sobre enlaces nuevos hacia símbolos saltados, enlaces viejos y borrado
de archivos completos; no una declaración de bug histórico reproducido.
Otra brecha concreta de contrato es que `add_edge` no recibe evidencia arbitraria.
La mejora sostenible es asociación a memoria canónica y generación, no otra base
de grafos. No se prescribe configuración de eval ni se modifica FIND-299.

## 10. Interfaz e inspección

**Fuente disponible:** de `graph-ui/src/App.tsx:1-132` solo se contrastaron
comentario e imports; no acreditan el flujo de conexión o navegación.
README describe selección de proyecto, Lens beta, lector, Galaxy, mapas,
fuente Monaco, diagnósticos y chat local opcional. Son superficies web servidas
por backend nativo C, no evidencia de un escritorio Tauri ni de paridad en Vanta.
La navegación devuelve nombre cualificado y posición; flechas visuales no son
trazas de ejecución real. Adoptar como criterio de diseño: abrir evidencia,
mostrar cobertura/staleness y no fingir grafo vacío cuando falta proyecto.
No se abrió el visor ni se validó accesibilidad, aislamiento o funcionamiento.

## 11. Dossier ampliado: ejecución, reconciliación e incidencias

### 11.1 Revisión, mapa y fronteras de datos

Ampliación documental observada el **2026-11-07**. `git rev-parse HEAD` y
`git status --short` confirmaron el SHA del manifest y clon limpio; no fetch,
pull, init ni ejecución del producto. CodeGraph respondió que este clon **no
está indexado**: se utilizó lectura directa, sin aceptar su sugerencia de init.
El mapa siguiente distingue código localizado de cobertura efectivamente leída.

| Área | Responsabilidad y límite |
|---|---|
| `src/discover`, `internal/cbm` | Descubrimiento, tree-sitter y extracción C; no se auditaron todas las gramáticas |
| `src/pipeline` | Orquestación, resolución de llamadas, enriquecimientos, manifiesto semántico, staging y publicación |
| `src/graph_buffer`, `src/store` | Grafo temporal frente a SQLite consultable; no son memoria conversacional |
| `src/semantic`, `src/simhash` | Rutas de búsqueda/similitud; no se acredita equivalencia de scores con Vanta |
| `src/mcp`, `src/cypher` | Contratos de herramientas/consultas; `manage_adr`, indexado y borrado no son lecturas |
| `src/daemon`, `src/watcher` | Ownership de procesos y refresco; el consumidor propuesto no adquiere ese lifecycle |
| `src/ui`, `graph-ui` | Servicio y Lens React; otro producto, no escritorio embebido Vanta |
| `src/traces`, `src/git` | Evidencia dinámica/Git separada de extracción; no se auditó cobertura runtime |

Stack acreditado: backend C con SQLite/FTS5 y tree-sitter vendorizado;
Lens declara React 19, Monaco, Three y `@huggingface/transformers`
([package.json:1–92](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/graph-ui/package.json#L1-L92)).
Esto acredita dependencias declaradas, no uso de todos los modelos ni garantía
de air-gap. MIT raíz no sustituye inventario por componente:
[THIRD_PARTY.md:1–55](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/THIRD_PARTY.md#L1-L55)
describe runtime MIT y gramáticas con MIT/CC0/Apache/ISC; su catálogo dice 160.
No trasladar al clon los 158 lenguajes de una skill instalada de otra versión.

Identidad: proyecto/qname e ID SQLite son identificadores del productor, no
identidad u128 canónica. El manifiesto de archivos, las filas de cobertura y
las referencias no resueltas son datos diferentes: tener un hash no prueba
que cada símbolo haya sido parseado o que cada llamada haya sido resuelta.
Una selección vacía de nodos no demuestra un archivo vacío ni ausencia de tests.

### 11.2 Flujos concretos: de bytes a generación consultable

**Resolución:** una llamada ya extraída entra en `emit_classified_edge`
(C1). Los patrones HTTP/async/config pueden emitir relaciones especializadas;
un match débil TS/JS con receptor no resuelto puede suprimir `CALLS`.
En el caso normal se conserva callee, strategy, confidence y número de
candidatos. Por tanto, **AST acredita forma sintáctica; el resolver decide el
destino**. Confidence reportada y selección de candidato no constituyen una
prueba del dispatch runtime. `calls_find_source:876–891` usa función envolvente
o nodo de archivo cuando no hay función: no inventar una función `main`.

**Actualización:** lectura nueva de
[pipeline_incremental.c:4039–4218](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/pipeline/pipeline_incremental.c#L4039-L4218)
corrige una interpretación demasiado simple de la ficha anterior:

1. Abre generación para consulta; si leer manifiesto falla, devuelve
   `CBM_PIPELINE_ABORT_PRESERVE_DB`, no interpreta el error como cero archivos.
2. `incr_generation_exact` compara el manifiesto semántico y metadatos.
   Igualdad exacta devuelve noop sin reabrir la generación sellada para escribir.
3. Si hay diferencia, intenta `closure_try_plan`: reparsea cambios y consumidores
   de superficies cambiadas. Si la planificación no es segura, fuerza full;
   una condición fatal no se degrada a éxito.
4. La clasificación por `mtime+size` de `classify_files:915–965` sigue en el
   archivo, pero `4137–4146` la ubica en la ruta legacy de test cuando no se usa
   closure. **No es correcto describirla como toda la ruta productiva actual.**
5. `find_deleted_files:968–1012` distingue deleted de mode-skipped; ENOENT/ENOTDIR
   puede justificar baja, EACCES/EIO/incertidumbre preserva. Menor cobertura de
   un modo no autoriza borrar todo lo que dejó de descubrir.
6. Antes de reconstruir se capturan filas de cobertura; fallo de lectura aborta.
   El comentario binario noop/full de `4079–4082` debe leerse junto al planner
   posterior: también existe reparación por closure.

**Publicación y ownership:** el código contempla stage separado, backup y sellado
antes del swap (`pipeline.c:5132–5280`); no se certificó crash safety del conjunto.
Sí se inspeccionó la protección concreta
[pipeline.c:4986–5048](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/pipeline/pipeline.c#L4986-L5048):
un sweep conserva stages de writer vivo o de lock que no pudo comprobar;
solo elimina el stage que puede reclamar. Adoptar “limpiar temporales” sin
ownership puede borrar el trabajo de otra ejecución.

**Estado humano:** [adr_migrate_before_delete:3191–3221](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/pipeline/pipeline.c#L3191-L3221)
migra ADR legacy al sidecar `<final>.adr.db`, no al stage descartable; si no
puede abrir un DB que existe, preserva/aborta. Es una defensa contra mezclar
estado humano durable con proyecciones reemplazables, no una recomendación
de crear sidecars Vanta.

**Watch:** [watcher.c:1–108](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/watcher/watcher.c#L1-L108)
define firmas dirty/HEAD pendientes. El cuerpo efectivo
[commit_baselines/poll_project:1623–1750][C-watch-ack] copia **lo observado**
(`pending_head/pending_dirty_sig`, o tree signature), no lo que haya cambiado
durante indexado. `poll_project:1733` invoca `index_fn`; solo `rc==0` llama
`commit_baselines:1737`. Busy (`rc>0`) conserva baseline; error no hace ACK.
El nombre efectivo al SHA es `commit_baselines`, no `commit_pending_changes`.
Git-clean no equivale a snapshot indexado. Un watcher configurado no
es evidencia de cobertura completa ni obliga a instalar otro watcher en Vanta.

**Consulta CM-1:** [handle_search_graph:5488–5580][C-query] resuelve
store/proyecto y exige proyecto indexado; limita `limit` a 1–500, offsets y
presupuesto de salida (estimación bytes/tokens, no tokenizer exacto).
Rechaza `query` con `semantic_query` simultáneos. Con `query` intenta BM25;
si FTS5 no está disponible o la tokenización queda vacía puede caer al camino
regex, no devuelve necesariamente ranking BM25. Fallo de resolución,
no-indexado o argumentos no equivale a “ningún símbolo”. B debe preservar
esa diferencia sin presentar modo efectivo como fusión de ambos.

**Borrado CM-1:** [handle_delete_project:8054–8150][C-delete-project]
exige proyecto, adquiere guard de mutación (puede cancelar/bloquear por
indexado), cierra store propio activo y toma pipeline lock. Intenta unlink
de DB y WAL/SHM. Si DB se elimina, retira sidecar `<db>.adr.db` y auxiliares
best-effort; si unlink de DB falla, conserva ADR, devuelve `delete_failed`
con error. DB ausente devuelve `not_found` con flag de error e intenta limpiar
ADR residual. Libera lock y **desregistra watcher incluso tras unlink fallido**.
`deleted` no certifica cada unlink auxiliar. Esto destruye proyecto/ADR
upstream, distinto de tombstones D de una proyección y de memorias manuales
Vanta: un conector no debe traducirlo automáticamente a borrado de éstas.
FMEA: fallo de unlink + unwatch → datos aún presentes sin refresco; detectar
estado por recurso, conservar diagnóstico y reintento explícito autorizado.

### 11.3 Incidencias upstream efectivamente leídas

Consulta keyless: `site:github.com DeusData codebase-memory-mcp issues incremental`.
Metasearch Bing/DDG con URLs de los cuatro proyectos retornó **0 resultados,
sin error**. `github-mcp-server.issue_read` para #867/#516 falló con
`Failed to connect to MCP server`; no es un fallo de autenticación demostrado
ni contradice que `get_me` funcionara en el lead. Fallback exitoso:
API pública `api.github.com/repos/DeusData/codebase-memory-mcp/issues/{n}`
y `/comments`, sin credenciales. Estados observados el 2026-11-07:

| Fuente leída | Reporte y camino | Estado y aplicabilidad |
|---|---|---|
| [#867](https://github.com/DeusData/codebase-memory-mcp/issues/867) + comentarios | En `74b27f3`: bootstrap con mtimes de otra máquina, persistencia por fila/re-stat posterior y reexport completo | Cerrado `not_planned`, 2026-09-25; cierre por redistribución del trabajo, no “no había bug” |
| [Comentario de cierre](https://github.com/DeusData/codebase-memory-mcp/issues/867#issuecomment-5841041178) | P1 atribuido a #1905/v0.11; P2 a SHA-256+torn-read guard; P3 continúa #1884; cross-path #885 | Declaración del maintainer. No se verificaron todos esos PRs; fuente actual §11.2 ya refuta atribuir todo a mtime |
| [#516](https://github.com/DeusData/codebase-memory-mcp/issues/516) + comentarios | En 0.8.1, dumps full **y** incremental cambiado reconstruían DB sin ADR; noop no perdía ADR | Cerrado `completed`, 2026-06-25; actualizado 2026-10-31 por carrera de swap |
| [Comentario #516](https://github.com/DeusData/codebase-memory-mcp/issues/516#issuecomment-6547746360) | Atribuye corrección de escritura ADR concurrente con swap a #2315/`7be2becf7321` | Código actual sí muestra migración sidecar; no se reprodujo carrera ni se revisó todo #2315 |

**Validación discriminante futura:** conservar anotación humana mientras se
actualizan símbolos, y provocar fallo de lectura de baseline sin permitir
reemplazo vacío. Para #867, dos clones del mismo SHA en rutas distintas no deben
confundir repo identity con root ni adoptar hashes de bytes no comparados.
Vanta calcula hash de contenido propio: no hereda automáticamente el bug mtime.
Sus anotaciones no deben quedar bajo ownership de una generación importada.
Las cifras de esos issues son reportes de sus autores, no benchmarks propios.

### 11.4 Benchmarks: población, tiempos y errores

Ampliación de C9/§7 con lectura de scripts completos relevantes:

| Runner | Qué calcula realmente | Errores, efectos y exclusiones |
|---|---|---|
| `benchmark-search-graph.sh:18–117` | Una medición wall-clock por request fijo; cinco patrones y tres queries BM25, `limit:20`; `date +%s%3N` incluye proceso/envelope, con daemon ya caliente | Runtime/cache privados, index full previo no medido, daemon start/cleanup. Query usa `\|\| true` y stderr descartado; parser devuelve `?`. Un tiempo impreso no acredita éxito ni percentiles |
| `benchmark-index.sh:74–183` | Setup monotónico separado de index y total; una ingesta por repo/etiqueta; archivos/LOC contados por `find`, no solo fuentes parseables | Escribe JSON/timings/conteos; fallback de parseo produce 0, pero exit de CLI y project vacío se comprueban. No dividir tiempo por “símbolos correctos” a partir de esos conteos |
| `docs/MEASURING_SAVINGS.md:1–110` | Receta de comparación de agente, no scorer ejecutado | Congela SHA/preguntas/modelo/budget, sesiones aisladas y todos los fallos. Advierte que `index_status` muestra HEAD actual, no prueba origen del índice |

El handoff `CBM_BENCH_KEEP_RUNTIME` solo se escribe después de stop exitoso;
stop fallido conserva runtime sin handoff para no borrar archivos de un daemon
potencialmente vivo. No se ejecutó ningún script. Dependencias: shell POSIX,
Python, Git, binario y corpus local. No hay split/gold/answerer/juez ni tokenizer
en estos dos microbenchmarks; no introducirlos por analogía con el paper.

**Adaptación Vanta, no autoría de evaluación:** un adapter C12 produciría
referencias repo/revisión/símbolo y respuestas con cobertura explícita para
comparar el baseline `code_*` con evidencia importada. El lead conserva preguntas,
anotaciones y protocolo METHOD; medir por separado ingesta, consulta y respuesta.
Los patrones `Controller/Service` no son corpus Rust representativo por sí solos.
Conservar no-match, error operativo y latencia de bootstrap; no importar el
`|| true` como una política de resultados válidos ni el proxy de compactación
como tokens reales del cliente.

## 12. Contratos de adopción y cierre

### 12.1 Reglas comunes y ampliación de fichas A–C

C12 es **el único formato canónico propuesto**. `CodeQueryPage` e
`ImpactEvidence` son respuestas de aplicación, no nuevos stores. Todos los
puertos siguientes son Rust **hipotético**, no APIs presentes.

| Ficha | Alternativas y delta | Persistencia, rollback y observabilidad |
|---|---|---|
| A: resolución | Extender scanners propios, adapter C12 o reescribir resolver. Elegir adapter opcional: importa estrategia/candidatos que el scanner no emite, sin sustituir índice propio | Original y mapping versionados; primero fuentes/evidencias, después relaciones. Revertir puntero de generación; desactivar productor, no borrar memoria manual. Contar ambiguos, externos, rechazos de esquema y bindings por productor |
| B: página/snapshot | Extender `code_search`/`code_explore`, no segundo retriever. Cursor firmado o estado server-side; nunca cursor libre que amplíe root | Cursor atado a consulta+generación+scope. Cambio de generación invalida cursor, no mezcla resultados. Contar omitidos, bytes/estimador declarado y fallos freshness, sin código en logs |
| C: impacto | Mostrar informe upstream frente a cerrar grafo propio. Mantener solo asesor hasta acreditar cobertura | Revisionar seeds/caminos; rollback elimina proyección asesoría, no cambia gates. Contar fallback, gaps y walks cancelados; prueba adversa de callbacks/macros/ciclos antes de excluir cualquier test |

Para A, error de endpoint o revisión desconocida rechaza el lote antes de publicar.
Para B, truncamiento se detecta por contadores y se mitiga con página posterior,
no salida “sin más dependencias”. Para C, error de alloc/cancelación produce
`Incomplete`/error, nunca conjunto vacío “seguro”. Aceptación común adicional:
reintentar tras fallo no duplica aristas; el lector no cruza generaciones;
rollback vuelve a evidencia anterior rotulada stale si ya no corresponde al árbol.

### 12.2 Ficha D — Reconciliación por ownership y cobertura

- **Problema/escenario:** un índice full se importa y después llega un delta
  moderate sin docs/tests; tratar esa omisión como baja destruye evidencia válida.
- **Fuente/unidad:** `find_deleted_files:968–1012` y routing `4039–4218` (§11.2).
  Extraer la distinción deleted/mode-skipped/error, no su motor ni heurísticas.
- **Valor/seam:** `code_index::index_repo:193–235` reconcilia símbolos del archivo;
  falta acreditar contrato de snapshot completo entre productores. Extender ese
  coordinador y jobs existentes, no otro storage.
- **Interfaz:** `plan_reconcile(previous: &Manifest, incoming: &CodeEvidenceImport)
  -> Result<ReconcilePlan, ReconcileError>`; `apply(plan, expected_generation)`.
  `Manifest/Plan/Error` son conceptos de aplicación pendientes de spec.
- **Invariantes:** tombstone necesita autoridad del productor, repo y generación;
  error de lectura no demuestra ausencia; lote parcial no declara eliminaciones
  de todo el repo; no cambia registros de otro productor o del usuario.
  Borrado de proyecto upstream incluye ADR humano; retirar una proyección D
  **no** autoriza ese alcance. `delete_failed/not_found` y watcher retirado
  deben conservarse en el recibo del adapter, no comprimirse a éxito.
- **Alternativas:** full reimport conservador (más coste, menor complejidad),
  delta con tombstones o watcher directo. Preferir full al principio si el export
  no acredita delta; no deducir tombstones desde un listado paginado.
- **Pasos/migración:** inventariar manifest existente → namespaces de ownership →
  modo dry-run sin bajas → validar cobertura/endpoints → publicar generación →
  garbage collection posterior. Datos legacy sin owner no se borran automáticamente.
- **Rollback/deps:** conservar manifest anterior y journal del plan; desactivar
  reconciliación y servir última generación marcada. Depende de A y publicación
  condicional; no bloquea inspector read-only.
- **FMEA:** cambio de modo → falsa baja → detectar diferencia de coverage/mode y
  preservar; dos importadores → borrado cruzado → owner estricto; delta incompleto
  → dangling edges → validar endpoints antes de commit.
- **Aceptación/observabilidad:** archivo ilegible preservado con error, eliminación
  explícita invalida relaciones incidentes propias, reimport idempotente; métricas
  de bajas propuestas/aplicadas/rechazadas, huérfanos y lag por generación.
- **Mínimo/no objetivos:** un productor y snapshot completo; no selector de tests,
  sincronización bidireccional ni promesa de rename identity sin evidencia.

### 12.3 Ficha E — Baseline del watcher solo después de publicación

- **Problema:** consumir un evento y avanzar hash aunque el job haya fallado
  deja un índice permanentemente viejo que el siguiente poll considera actual.
- **Fuente/unidad:** `commit_baselines:1623–1644` y caller
  `poll_project:1731–1750` [C-watch-ack], además del estado `64–108`.
  La unidad extraíble es ACK solo tras `index_fn==0`, no el proceso daemon.
- **Seam/valor real:** `code_index::index_repo:193–202` compara hash, **no**
  encola ni confirma un job. `services/scheduler.rs::run_pass:79–119`
  [V-scheduler] usa `LocalStateBackend`, TimerScanner y PipelineWorker;
  `core/state/types.rs::TaskKind:38–50` [V-task-kinds] solo enumera
  L1/L2/L3/Dream/Flush y `MemoryTaskHandler::handle:900–916` los despacha.
  No se acredita un caller/job de code-index ni ACK persistente conectado.
  El scheduler es del writer único y su cola/timers/leases son locales
  [V-scheduler:25–30]; TaskCheckpoint no hace durable esa cola por existencia.
  Integrar E en el coordinador existente es **trabajo nuevo no acreditado**,
  no construir otro scheduler/WAL. `observe_change` no es `publish_complete`.
- **Interfaz hipotética:** `observe(repo, fingerprint) -> PendingRevision`;
  `ack_published(pending, receipt) -> Result<(), SupersededOrFailed>`.
  El receipt identifica la generación C12, no confirma entrega de red.
- **Invariantes/alternativas:** el ACK solo corresponde al trabajo observado;
  retry idempotente; job antiguo no pisa nuevo. Preferir polling/reconcile único
  existente a segundo watcher; hooks solos pierden ediciones no committeadas.
- **Pasos/migración:** registrar intent/pending → encolar job → publicar → ACK
  condicional → conciliar pendientes al reiniciar; legacy desconocido fuerza
  comprobación, no baseline fresh. Desactivar auto-refresh conserva consulta manual.
  Es una secuencia propuesta: antes hay que definir el caller de code-index
  y cómo revalidar pendientes desde fuente al reiniciar, sin presumir que la
  cola local los persiste. No usar una tarea L1 como code-index por alias.
- **Dependencias/rollback:** D para cobertura y escritor condicional. Rollback
  cancela nuevos jobs y mantiene pending diagnosticable; no revierte el código fuente.
- **FMEA/detección/mitigación:** caída tras publish antes de ACK → repetición →
  dedup por generación; busy → perder evento → pending retenido; error persistente
  → tormenta de jobs → backoff observable sin avanzar baseline.
- **Aceptación:** cambio durante trabajo genera pendiente posterior; reinicio
  recupera pendiente; fallo no anuncia current. Observar retries, edad del pendiente,
  estados degraded y último publish, nunca paths sensibles como etiquetas métricas.
- **Mínimo/no objetivos:** conectar cambios a jobs; no timers/umbrales nuevos aquí,
  no instalar hooks, no declarar exactamente-una-vez global.

### 12.4 UI, contraste final y cobertura negativa

`graph-ui/src/App.tsx:1–132` (C9) acredita principalmente comentario/imports,
no un flujo React completo de Reader/Atlas/diagnóstico/ADR. Esos nombres no
sustituyen lectura de sus cuerpos/callers; integración visual queda parcial.
Propuesta: selección → evidencia → revisión → explicación de límite,
no Galaxy como prueba de cobertura. Studio ya tiene
`feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:138–164,193–228`
en `develop-desktop@0ad38c542b39a589035b0e17fdedb4c09dca24b9`:
search/filtros/score/contexto/inspector; fuente leída vía `git show`, solo lectura.
Fuente inmutable del cuerpo releído:
[RetrievalLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228)
(2026-11-07); no bytes dirty de `desktop/...`.
Oportunidad: superponer productor/revisión/resolver y razón de stale a ese flujo.
No crear Studio nuevo ni incluir Lens en el motor.
FMEA/aceptación UI CM-1/CM-2: distinguir regex de BM25, no-indexado de vacío,
borrado fallido de proyección retirada y pending de publicación confirmada.
El pin no acredita JobChip/jobs ni un panel de E ya implementado.
Aceptar E requeriría evidenciar caller/resultado/ACK; no basta hash igual o
cola vacía. No se ejecutan esas validaciones en esta corrección.

[C-query]: https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/mcp.c#L5488-L5580
[C-delete-project]: https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/mcp.c#L8054-L8150
[C-watch-ack]: https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/watcher/watcher.c#L1623-L1750
[V-scheduler]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/scheduler.rs#L25-L119
[V-task-kinds]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/state/types.rs#L38-L69

**Recibo [CM-1/CM-2](reviews/code-memory.md#codebase-memory-mcp):** consultas,
borrado, ACK/caller y límite del scheduler propagados a fichas/matriz/UI.
Se conservan interfaces propuestas; pendiente de reconfirmación independiente.

**Vanta revalidado:** `index_repo:152–327`, despacho multiidioma `514–520`,
`MemoryInput:184–224`, `add_edge:113–151`, `assemble_with_recall:252`
son seams reales. `add_edge` inserta extremos secuencialmente: no atribuir
atomicidad multirregistro por la mera firma; evidencias asociadas necesitan
publicación controlada. `all_indexed_symbols` solo acumula archivos procesados:
hipótesis acotada de enlaces hacia símbolos omitidos, no prueba runtime.

**Pendientes:** probar atomicidad/rollback e identidad sobre moves; leer todo
planner/manifest y transports; revisión adversarial independiente por el lead;
licencias transitivas y conformance del adapter. No auditoría exhaustiva de
parsers, LSP, trazas o seguridad. Esta ampliación no ejecuta validadores,
tests, apps, benchmarks, servicios ni commits. Al aprobar: separar specs de
importación/resolución (A/D), consulta (B), asesoría (C), refresco (E) y UI;
ninguna necesita reservar IDs de backlog durante esta investigación.
