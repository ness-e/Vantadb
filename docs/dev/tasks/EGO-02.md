---
title: "EGO-02 — query_vector Opcional en Búsqueda Vectorial"
kind: task
description: "Permitir que query_vector sea opcional en la API de búsqueda en bindings y SDK, facilitando búsquedas puramente léxicas o híbridas"
status: completed
owners: ["Bindings & SDK Team"]
tags: ["api-contract", "bindings", "ego-tasks", "search"]
---

# EGO-02: query_vector Opcional en Búsqueda Vectorial

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-15-pre-mercado-estabilizacion.md`
- **Fuente:** `VDB-BUG-02` (Ego client feedback)
- **Commit de Implementación:** `c82f4b2d`
- **Estado:** ✅ COMPLETED
- **Reglas aplicables:** `.opencode/rules/api-contract.md`, `.opencode/rules/python-bindings.md`, `.opencode/rules/js-ecosystem.md`

## Resumen y Contexto
Permitir que `query_vector` sea `None`/`Option::None` en las llamadas a búsqueda (`search`), de forma que clientes que realicen consultas de texto puro o con filtros estructurados sin vector no sean rechazados con `TypeError` o errores de validación.

## Evidencia Mecánica
- **Commit:** `c82f4b2d`
- **Verificación:** Tests unitarios y de integración de búsqueda en `tests/` y bindings.
- **Clippy y Fmt:** Limpios y conformes con las directrices del proyecto.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` (auditoría independiente de gobernanza).
- **Fecha:** 2026-10-19
- **Veredicto:** ✅ APPROVE EGO-02.
- **Checklist Anti-Hábitos Tóxicos:**
  - [x] Sin `.unwrap()` / `.expect()` en rutas de producción.
  - [x] Conserva retrocompatibilidad en el contrato de API público.
  - [x] Tipado consistente en stubs de Python (`.pyi`) y definiciones de TypeScript (`.d.ts`).
  - [x] No introduce deuda técnica oculta ni regresiones en índices de búsqueda.
