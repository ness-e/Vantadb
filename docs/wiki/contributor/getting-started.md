---
title: "Contribuir: primeros pasos"
kind: concept
status: active
description: "Cómo empezar a contribuir al repo: guía canónica, áreas por crate y convenciones que debes conocer antes del primer cambio."
---

# Contribuir: primeros pasos

## Guía canónica

`[HECHO]` La guía de contribución oficial es [`.github/CONTRIBUTING.md`](../../../.github/CONTRIBUTING.md) (enlazada desde el README raíz); el canal de vulnerabilidades es [`.github/SECURITY.md`](../../../.github/SECURITY.md). Esta página solo orienta la primera lectura.

## Antes de cambiar código

`[HECHO]` Convenciones con gate: `clippy::unwrap_used`/`expect_used` en deny fuera de tests (`Cargo.toml:771-772`); `missing_docs` warn en crates FFI (`vantadb-ffi-core/src/lib.rs:60`); `unsafe_op_in_unsafe_fn` deny (`src/lib.rs:2`); errores públicos solo vía `vantadb::Error`. `[HECHO]` El quality bar está en `docs/specs/CONSTRAINTS.md` y el modelo operativo en `docs/dev/OPERATING-MODEL.md`.

## Por dónde empezar

`[INFERENCIA]` Ruta sugerida: [code-map](../developer/code-map.md) para ubicar el módulo → [contracts](../developer/contracts.md) para saber qué gates toca el cambio → [change-checklist](change-checklist.md) antes de abrir el PR. Cambios en bindings exigen fila en la matriz de BINDINGS_NAMESPACES + evidencia same-PR.

## Fuentes

`.github/CONTRIBUTING.md` · [CONSTRAINTS](../../specs/CONSTRAINTS.md) · [OPERATING-MODEL](../../dev/OPERATING-MODEL.md) · [developer/](../developer/code-map.md)
