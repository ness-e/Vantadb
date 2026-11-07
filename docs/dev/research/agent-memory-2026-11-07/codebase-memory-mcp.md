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
La implementación leída en la fase inicial (worktree `c561158`/dirty histórico) de `extract_symbols_for_lang` también enruta
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
- **DTO Rust propuesto:** `CodeEvidence { repo_id, revision, producer,
  producer_schema, external_qname, rel_path, span, content_digest,
  relation, target_ref, resolver, confidence: Option<f32>, candidates,
  resolution_state }`; `resolution_state` distingue resuelto/ambiguo/externo.
  Peso de arista y probabilidad calibrada no se intercambian.
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

No se consultaron issues remotos. Referencias `#...` dentro del código son contexto
histórico, no defectos vigentes demostrados. Se leyeron contratos de test C8 sin
ejecutarlos. Las garantías de aislamiento por proyecto en SQLite no equivalen a
autenticación multi-tenant: el proceso local conserva permisos del usuario.
“Local” describe indexación/consulta; integraciones, instalador, descargas de
modelos y agente consumidor tienen fronteras de red separadas, no auditadas aquí.

## 4. Prerrequisitos VantaDB

1. Revalidar FIND-299 y worktree vigente (lectura histórica de esta ficha: HEAD inicial `c561158` + dirty de la fase, no HEAD actual `b64383e`): HEAD de referencia Vanta
   `c561158253bb88192590fa406c61e0a27dd7df8d`; el árbol general tiene trabajo concurrente.
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
| C5 | [mcp.c:881–944](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/mcp.c#L881-L944): fronteras de herramientas |
| C6 | [store_impact.h:1–105](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/store/store_impact.h#L1-L105): contrato cierre/cancelación |
| C7 | [test_impact_engine_reach.c:21–38](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/test_impact_engine_reach.c#L21-L38), [1202–1255](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/src/mcp/test_impact_engine_reach.c#L1202-L1255): roles y fixpoint |
| C8 | [test_call_reference_contract.c:1–70](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/tests/test_call_reference_contract.c#L1-L70): helpers de contrato end-to-end, no resultado |
| C9 | [App.tsx:1–80](https://github.com/DeusData/codebase-memory-mcp/blob/612eee2518ef84cc573e491fdee5fa5c90264cb5/graph-ui/src/App.tsx#L1-L80): wiring de fuente y navegación |

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
| Cierre seguro para excluir tests | No acreditado; `code_impact` es acotado | Solo informe asesor, fallback amplio |
| Lens como escritorio Vanta | No aplicable | UI externa; Studio es otro worktree |

## 9. Debilidades sugeridas

En la lectura histórica del worktree de la fase inicial Vanta asigna `Some(1.0)` a referencias por coincidencia textual
y compara los símbolos acumulados de archivos procesados (`code_index.rs:310–315`).
Los archivos sin cambios salen antes (`193–202`). Esto justifica **hipótesis a
revalidar** sobre enlaces nuevos hacia símbolos saltados, enlaces viejos y borrado
de archivos completos; no una declaración de bug histórico reproducido.
Otra brecha concreta de contrato es que `add_edge` no recibe evidencia arbitraria.
La mejora sostenible es asociación a memoria canónica y generación, no otra base
de grafos. No se prescribe configuración de eval ni se modifica FIND-299.

## 10. Interfaz e inspección

**Fuente disponible:** `graph-ui/src/App.tsx` conecta selección de proyecto,
lector, seguimiento de símbolos y Galaxy; README describe Lens beta, mapas,
fuente Monaco, diagnósticos y chat local opcional. Son superficies web servidas
por backend nativo C, no evidencia de un escritorio Tauri ni de paridad en Vanta.
La navegación devuelve nombre cualificado y posición; flechas visuales no son
trazas de ejecución real. Adoptar como criterio de diseño: abrir evidencia,
mostrar cobertura/staleness y no fingir grafo vacío cuando falta proyecto.
No se abrió el visor ni se validó accesibilidad, aislamiento o funcionamiento.
