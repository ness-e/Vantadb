---
title: "Plan Maestro de Remediación Post-Auditoría v0.9.0 (AUDIT-01..AUDIT-06)"
kind: plan
status: active
created_at: "2026-10-31T06:00:00Z"
updated_at: "2026-10-31T06:00:00Z"
owner: "vanta-lead"
category: "remediation"
tags: [audit, remediation, governance, memory, mcp, engine]
description: "Plan Maestro de Remediación integral en 6 tareas técnicas (AUDIT-01 a AUDIT-06) tras la auditoría profunda v0.9.0."
---

# Plan Maestro de Remediación Post-Auditoría v0.9.0 (AUDIT-01..AUDIT-06)

## Objetivo

Consolidar y ejecutar la remediación estructural de los hallazgos críticos de la auditoría profunda v0.9.0 y del post-mortem de release en el repositorio VantaDB. El plan establece seis tareas atómicas e interdependientes (`AUDIT-01` a `AUDIT-06`) para restaurar la calidad de recuperación semántica/léxica, resolver fallos de consistencia en el filtrado y almacenamiento particionado, unificar el ciclo de vida y borrado de memoria (`forget()`), consolidar el motor de almacenamiento bajo una arquitectura única sin duplicidad in-memory, exponer el perfil cognitivo de alta densidad (`McpProfile::Brain`) y sanear el sistema de gobernabilidad `.opencode`, la documentación pública y el backlog de negocio.

## Archivos en scope

- `evals/memory/**`, `evals/eval_longmemeval.py`
- `src/embeddings/**`, `src/sdk/search/hybrid.rs`, `src/sdk/api/memory.rs`
- `src/storage/engine/**`, `src/storage/query_planner.rs`, `src/shred/**`
- `src/sdk/api/**`, `vantadb-python/src/**`, `vantadb-mcp/src/**`
- `src/engine.rs`, `src/lib.rs`, `Cargo.toml`
- `vantadb-mcp/src/profile.rs`, `vantadb-mcp/src/tools/**`, `scripts/install.*`
- `.opencode/**`, `docs/**`, `README.md`, `README_ES.md`

## Criterios de aceptación

1. **AUDIT-01 (Recuperación LongMemEval):** Embeddings locales predeterminados con `fastembed` en Python/evals; chunking por turnos conversacionales en lugar de límites ciegos por caracteres; ponderación RRF diferenciada por modalidad (`dense_weight` vs `sparse_weight`).
2. **AUDIT-02 (Correctness de filtrado y almacenamiento):** Eliminación del bug de bitset vacío en `InFilter` (`node.bitset`); caché de estadísticas de colección BM25 en memoria compartida sin recálculo O(N) por query; desacople de `ShreddedRowStore` en `memory.put` para no penalizar latencia de inserción en caliente.
3. **AUDIT-03 (API Unificada forget()):** Exposición de método canónico `forget(key, mode)` con modos `Soft` (tombstone), `Hard` (purga física de vstore/índices) y `Certified` (emisión de certificado criptográfico de eliminación) en Rust SDK, Python SDK y protocolo MCP.
4. **AUDIT-04 (Unificación de Motores):** Eliminación definitiva de `InMemoryEngine` en `src/engine.rs`; migración de toda la suite de pruebas a `StorageEngine` con `BackendKind::InMemory`; eliminación de dependencias pesadas en los `default` features de `Cargo.toml`.
5. **AUDIT-05 (Perfil Cognitivo Brain & Distribución):** Implementación de `McpProfile::Brain` con 12 herramientas consolidadas más meta-despachador `vanta_dispatch`; soporte de transición de niveles de memoria L0→L3; empaquetado de binarios con feature `embed-local` embebida y scripts de instalación autosuficientes.
6. **AUDIT-06 (Gobernabilidad, Posicionamiento y Rieles de Negocio):** Saneamiento de reglas `.opencode/` en rama `develop` (aislando `desktop/` en `develop-desktop`); alineación estratégica `STRAT-01` como "SQLite for agents" (memoria cognitiva local-first); actualización de `Backlog-negocio.md` priorizando Creem.io, Binance Pay USDT y Payoneer→Airtm para residencia VE (descartando Polar.sh por bloqueo de Stripe Connect Express en Venezuela).

## Comando de verificación

```bash
cargo check --workspace --all-targets
cargo test --workspace --no-default-features --features "embed-local,mcp"
python -m pytest evals/
node scripts/docs/check-links.mjs
node scripts/docs/check-docs.mjs
```

---

## Desglose de Tareas

| Tarea | Título | Módulos Principales | Esfuerzo | Prioridad |
|---|---|---|---|---|
| **AUDIT-01** | Reparación de Calidad de Recuperación en LongMemEval | `evals/`, `src/embeddings/`, `src/sdk/search/` | 🟡 2-3d | 🔴 Alta |
| **AUDIT-02** | Corrección InFilter bitset, caché BM25 y desacople ShreddedRowStore | `src/storage/engine/`, `src/shred/`, `src/sdk/` | 🟡 2d | 🔴 Alta |
| **AUDIT-03** | API Unificada Embedded::forget() (Soft, Hard, Certified) | `src/sdk/api/`, `vantadb-python/`, `vantadb-mcp/` | 🟡 1-2d | 🔴 Alta |
| **AUDIT-04** | Unificación de Motores (Deprecación InMemoryEngine) y Trim Cargo | `src/engine.rs`, `src/storage/`, `Cargo.toml` | 🟡 2d | 🟠 Media |
| **AUDIT-05** | Perfil Cognitivo McpProfile::Brain y Reparación de Instaladores | `vantadb-mcp/`, `scripts/install.*` | 🟡 2d | 🔴 Alta |
| **AUDIT-06** | Limpieza .opencode, Posicionamiento STRAT-01 y Rieles Negocio VE | `.opencode/`, `README*.md`, `docs/dev/` | 🟢 1d | 🔴 Alta |

---

### Tareas en Detalle

### AUDIT-01: Reparación de Calidad de Recuperación en LongMemEval
- **Problema:** En las evaluaciones de `LongMemEval`, los embeddings sintéticos o modelos remotos no acoplados introducen latencias elevadas o fallas de contexto. El chunking previo por longitud de caracteres fragmentaba mensajes conversacionales entre turnos, degradando el recall de aguja en pajar.
- **Acción:** Adoptar `fastembed` como provider local primario en el harness de evaluación; segmentar las sesiones en chunks semánticos basados en turnos de usuario/asistente; calibrar los pesos de fusión RRF según modalidad (ponderando el componente denso sobre el léxico en diálogos con lenguaje natural).

### AUDIT-02: Corrección InFilter bitset, caché BM25 y desacople ShreddedRowStore
- **Problema:** El filtro `InFilter` calculaba un bitset de nodos pre-filtrados que, bajo ciertas ramas de optimización, quedaba vacío descartando matches legítimos (`node.bitset`). BM25 recalculaba estadísticas de frecuencia documental en cada query. `memory.put` ejecutaba la trituración a filas tabulares (`ShreddedRowStore`) en la ruta crítica síncrona.
- **Acción:** Corregir la inicialización e intersección del bitset en `InFilter`. Introducir caché invalidable para estadísticas BM25 por colección. Mover la persistencia tabular de `ShreddedRowStore` a un worker desacoplado o paso de background pos-WAL.

### AUDIT-03: API Unificada Embedded::forget() (Soft, Hard, Certified)
- **Problema:** Las operaciones de borrado estaban fragmentadas entre `delete()`, `soft_delete()` y `purge()`, con interfaces divergentes entre bindings de Rust, Python y MCP.
- **Acción:** Diseñar e implementar `Embedded::forget(key: &str, mode: ForgetMode) -> Result<ForgetReport>`. `ForgetMode::Soft` establece la lápida (tombstone); `ForgetMode::Hard` desvincula el nodo de los índices HNSW/BM25 y purga el storage; `ForgetMode::Certified` genera un certificado firmado/hasheado comprobando la inexistencia de residuos en disco. Exponerlo de forma idéntica en Python y en la tool `vanta_forget` de MCP.

### AUDIT-04: Unificación de Motores (Deprecación InMemoryEngine)
- **Problema:** `src/engine.rs` mantenía una bifurcación de mantenimiento costosa con `InMemoryEngine`, duplicando lógica de índices y colecciones respecto a `StorageEngine`. Además, `Cargo.toml` incluía features pesadas por defecto.
- **Acción:** Retirar `InMemoryEngine`. Migrar todas las pruebas que requerían ejecuciones sin disco para utilizar `StorageEngine` respaldado por `BackendKind::InMemory`. Recortar los `default` features en `Cargo.toml` para aligerar la compilación por defecto.

### AUDIT-05: Perfil Cognitivo McpProfile::Brain y Reparación de Instaladores
- **Problema:** El servidor MCP ofrecía perfiles dispersos o con demasiadas tools atómicas sin jerarquía cognitiva, lo que agotaba el context window de los agentes de IA. Los binarios instalados carecían a menudo del feature de embeddings locales o dependían de dependencias de sistema dinámicas.
- **Acción:** Introducir `McpProfile::Brain` limitando la interfaz a 12 herramientas de alto impacto más el router meta `vanta_dispatch`, formalizando la jerarquía L0 (raw facts) a L3 (persona/insights). Reconstruir instaladores `vanta-install.ps1` y `install.sh` garantizando soporte `embed-local`.

### AUDIT-06: Gobernabilidad, Posicionamiento STRAT-01 y Rieles de Negocio
- **Problema:** El árbol `.opencode/` en `develop` fallaba en linters estáticos (`ast-grep`, `semgrep`) por referenciar el directorio `desktop/` que reside en el worktree `develop-desktop`. En el `README.md` persistían siglas internas (`ADR-0047`, `FIND-105`), un bloque confuso de métrica North Star y enlaces 404 a `docs/dev/`. El posicionamiento del producto oscilaba entre vector DB genérica y sucesor de Kuzu. En `Backlog-negocio.md`, se asumía Polar.sh sin considerar que Stripe Connect Express rechaza cuentas bancarias en Venezuela (`VE`).
- **Acción:** Sanitizar configs y reglas de `.opencode/` aislando `desktop/` al worktree `develop-desktop`. Limpiar `README.md` y `README_ES.md`, fijando la visión `STRAT-01`: "The local-first cognitive memory engine for AI agents ('SQLite for agents')". Resolver `STRAT-01` en `Backlog-negocio.md`, posponer `BIZ-18`, y reestructurar los rieles de pago para Venezuela priorizando Creem.io (USDC Polygon), Binance Pay USDT y Payoneer→Airtm. Marco operativo detallado en [OPERATING-MODEL](../OPERATING-MODEL.md).
