# Plan de Ejecución: Auditoría docs/api/ — Correcciones

> **Campaign ID:** f763f627-518b-4846-b403-56e831278594
> **Inicio:** 2026-07-21
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** Auditoría docs/api/ (vanta-lead, Jul 21)
> **Score actual:** 6.4/10

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 6  | 0     | 0    | 0         |

## Contexto

La auditoría de `docs/api/` encontró 9 incidencias (5 críticas, 4 medias) con un score de 6.4/10.
Este plan cubre las correcciones priorizadas para llevar la documentación a ~8.5+/10.

**Problemas principales:**
- Tipos `u64` vs `u128` desactualizados en EMBEDDED_SDK.md y openapi.yaml
- Referencias rotas: `vantadb-cli` no existe, `query_lisp` renombrado a `query`
- `openapi.yaml` contiene campos inventados (`semantic_cluster`) que no existen en el código real
- PYTHON_SDK.md falta métodos: `search_batch`, `flush`, `VantaVector`, `delete_by_filter`, `similar_to_key`, `count`
- PYTHON_SDK.md tipo `vector: List[float]` incorrecto — acepta `List[float] | VantaVector | np.ndarray | buffer protocol`
- TS_SDK.md no menciona `connect_idb()`
- Falta documentación de sintaxis IQL
- `last_reviewed` desactualizado, versiones inconsistentes

---

### Task DOC-API-01: Fix EMBEDDED_SDK.md — tipos u64→u128 + VantaConfig + last_reviewed

- **Archivos clave:** `docs/api/EMBEDDED_SDK.md`
- **Gate Justificación:** Error de tipo que causa bugs en producción. `node_id: u64` vs código real `u128`. `Edge.target: u64` vs `u128`.
- **Contrato:** "grep 'u64' en EMBEDDED_SDK.md solo encuentra referencias válidas (sin falsos positivos para node/edge)"
- **Task file:** `tasks/DOC-API-01.md`
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-07-21T00:00

---

### Task DOC-API-02: Fix openapi.yaml — remover semantic_cluster, alinear NodeDTO, bump version

- **Archivos clave:** `docs/api/openapi.yaml`
- **Gate Justificación:** `semantic_cluster`, `relational`, `hits`, `confidence_score` no existen en `VantaNodeRecord` real. `version: 1.0.0` debería coincidir con MCP.md `0.1.5`.
- **Contrato:** "NodeDTO en openapi.yaml coincide con campos de VantaNodeRecord real (src/sdk/types.rs)"
- **Task file:** `tasks/DOC-API-02.md`
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-07-21T00:00

---

### Task DOC-API-03: Fix MCP.md — comandos rotos, tool renombrado, last_reviewed

- **Archivos clave:** `docs/api/MCP.md`
- **Gate Justificación:** `cargo install vantadb-cli` falla (el crate no existe). `query_lisp` ya no existe, es `query`. last_reviewed desactualizado.
- **Contrato:** "MCP.md no contiene referencias a `vantadb-cli` ni `query_lisp`"
- **Task file:** `tasks/DOC-API-03.md`
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-07-21T00:00

---

### Task DOC-API-04: Fix PYTHON_SDK.md — métodos faltantes + tipos correctos

- **Archivos clave:** `docs/api/PYTHON_SDK.md`
- **Gate Justificación:** 6 métodos existentes sin documentar. `vector: List[float]` incorrecto (acepta VantaVector/np.ndarray/buffer). last_reviewed desactualizado.
- **Contrato:** "PYTHON_SDK.md documenta search_batch, flush, VantaVector, delete_by_filter, similar_to_key, count con firmas correctas"
- **Task file:** `tasks/DOC-API-04.md`
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-07-21T00:00

---

### Task DOC-API-05: Fix TS_SDK.md — connect_idb, searchVector vs search_vector, types

- **Archivos clave:** `docs/api/TS_SDK.md`
- **Gate Justificación:** `connect(path)` no existe en WASM — es `new()`, `open()`, `connect_idb()`. `searchVector()` es en realidad `search_vector` (el TS wrapper lo camelCasa). u128 serialización a string no documentada.
- **Contrato:** "TS_SDK.md menciona connect_idb() y usa searchVector/search_vector correctamente según código real"
- **Task file:** `tasks/DOC-API-05.md`
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-07-21T00:00

---

### Task DOC-API-06: Crear IQL.md + HTTP_API.md completeness + bump last_reviewed general

- **Archivos clave:** `docs/api/IQL.md`, `docs/api/HTTP_API.md`
- **Gate Justificación:** IQL no tiene documentación standalone. HTTP_API.md solo cubre 3 endpoints — verificar contra código real si hay más. Bump last_reviewed en todos los archivos restantes.
- **Contrato:** "docs/api/IQL.md existe con sintaxis básica. HTTP_API.md verificado contra cli_server.rs."
- **Task file:** `tasks/DOC-API-06.md`
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-07-21T00:00

---

## Dependencias

```
DOC-API-01 ──┐
DOC-API-02 ──┤
DOC-API-03 ──┤── (independientes, pueden ejecutarse en paralelo)
DOC-API-04 ──┤
DOC-API-05 ──┤
DOC-API-06 ──┘
```

Todas las tasks son independientes entre sí. El orden sugerido es por prioridad (1→6), pero se pueden ejecutar en cualquier orden.

## Post-Condición

- Score docs/api/ sube de 6.4/10 a ~8.5/10
- No quedan referencias a `u64` donde el código usa `u128`
- No quedan referencias a `vantadb-cli` ni `query_lisp`
- Todos los métodos públicos de Python SDK están documentados
- openapi.yaml coincide con el código real
- IQL.md existe como referencia de sintaxis
- last_reviewed actualizado en todos los archivos

## Recitation

=== RECITATION ===
Objetivo activo: Plan de corrección docs/api/ — 6 tasks
Estado: plan
Última acción: Creación del plan file
Resultado: ✅ Plan creado
State: PLAN (desde: IDLE)
Próxima acción: Crear task files DOC-API-01 a DOC-API-06
Contrato: "Plan file existe en docs/plans/ con 6 tasks definidas"
Próxima tarea si completa: Ejecutar DOC-API-01
last-synced: 2026-07-21T00:00
=== END RECITATION ===
