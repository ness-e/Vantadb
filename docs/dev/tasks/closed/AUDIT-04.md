---
title: "AUDIT-04: Unificación de Motores y Reducción de Default Features en Cargo"
kind: task
status: completed
created_at: "2026-10-31T06:00:00Z"
updated_at: "2026-10-31T06:00:00Z"
owner: "vanta-engine"
category: "architecture-refactor"
tags: [engine, inmemory, storage, cargo, refactor]
description: "Unificación de Motores (Eliminación de InMemoryEngine en src/engine.rs y migración de tests a StorageEngine con BackendKind::InMemory) y reducción de default features en Cargo.toml."
---

# AUDIT-04: Unificación de Motores y Reducción de Default Features en Cargo

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-31-audit-remediation-master.md`
- **Fuente:** Hallazgos FIND-503, deuda técnica de doble motor
- **Esfuerzo:** 🟡 2d
- **Prioridad:** 🟠 Media
- **Tipo:** Rust (core architecture & packaging)
- **Turns estimados:** 15-20
- **Creado:** 2026-10-31T06:00:00Z
- **last-synced:** 2026-10-31T06:00:00Z
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 4

## Objetivo

Eliminar la dualidad arquitectónica en el motor core de VantaDB retirando definitivamente `InMemoryEngine` de `src/engine.rs`. Toda la funcionalidad en memoria (incluyendo pruebas y entornos transitorios) debe canalizarse a través de `StorageEngine` configurado con `BackendKind::InMemory`. Asimismo, se deben depurar los `default` features en `Cargo.toml` para evitar compilar por defecto dependencias pesadas innecesarias (como crates de cifrado complejo, proveedores remotos o herramientas de debug) al consumir `vantadb` como librería embebida.

## Archivos en scope

- `src/engine.rs`: remoción de `InMemoryEngine` y redirección hacia `StorageEngine`.
- `src/storage/engine/mod.rs` y `src/storage/engine/init.rs`: soporte de `BackendKind::InMemory`.
- `Cargo.toml`: reconfiguración de `[features]` y aligeramiento de `default`.
- `tests/**`: actualización de tests unitarios y de integración que instanciaban `InMemoryEngine`.

## Blast Radius

| Dirección | Módulos |
|---|---|
| Callers | Suites de tests, `Client::in_memory()`, SDK callers que solicitaban DB efímera |
| Callees | `StorageEngine`, `BackendKind::InMemory`, `HnswIndex`, `WAL` (modo bypass/noop) |
| Implicaciones | Se elimina código duplicado de indexación y escaneo; el tiempo de compilación inicial disminuye; se garantiza paridad de comportamiento entre tests y producción. |

## Criterios de aceptación

1. `InMemoryEngine` completamente removido o convertido en un alias obsoleto que construye `StorageEngine::open_with_backend(BackendKind::InMemory)`.
2. Todos los tests existentes en `tests/` y `src/` compilan y pasan utilizando `StorageEngine`.
3. `cargo build --no-default-features -p vantadb` compila limpiamente en tiempo reducido.
4. Las dependencias opcionales (`remote-providers`, `server`, etc.) no están activas en `default`.

## Contrato

```bash
cargo check --workspace --no-default-features
cargo test -p vantadb --features "in-memory"
cargo clippy -p vantadb --all-targets -- -D warnings
```

## Steps

### Step 1: Verificar completitud de `StorageEngine` con `BackendKind::InMemory`
- Auditar que `BackendKind::InMemory` admita todas las operaciones requeridas por las pruebas (inserción, traversal, búsqueda híbrida, transacciones).
- Habilitar bypass de persistencia en disco de manera segura.

### Step 2: Refactorizar `src/engine.rs`
- Sustituir la implementación separada de `InMemoryEngine` por la inicialización parametrizada de `StorageEngine`.
- Actualizar `VantaDb::in_memory()` o constructores análogos en el SDK.

### Step 3: Migrar suite de pruebas
- Reemplazar imports y fixtures que dependían de la API vieja de `InMemoryEngine`.
- Validar que no existan carreras de datos ni fugas de memoria en la ejecución de tests.

### Step 4: Reducir `default` features en `Cargo.toml`
- Ajustar la lista de `default = [...]` en la raíz y en el crate `vantadb`.
- Probar compilación con combinaciones mínimas de flags.

## Invariantes de dominio

- La semántica de las operaciones en memoria debe ser 100% idéntica a la del motor persistente.
- El rendimiento de las pruebas no debe degradarse al unificar bajo `StorageEngine`.

