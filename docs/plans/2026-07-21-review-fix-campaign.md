# Plan: Resolver Hallazgos del Review Externo

**Fuente:** `docs/reviews/review.md`
**Fecha:** 2026-07-21
**Backlog referencia:** `docs/Backlog.md`

## Resumen

| Prioridad | Count | Estado |
|-----------|-------|--------|
| P0 | 3 | ⬜ PENDING |
| P1 | 2 | ⬜ PENDING |
| P2 | 4 | ⬜ PENDING |
| Estratégico | 1 | ⬜ PENDING |

## Tareas

### P0 — Debe resolverse antes de claims de producción más fuertes

| ID | Hallazgo | Esfuerzo | Sub-Agente | Verificación | Estado |
|----|----------|----------|------------|-------------|--------|
| P0-3 | `vantadb-python/vantadb_py/vantadb_py.abi3.so` suelto en tree → añadir a `.gitignore` + verificar otros artefactos build | 🟢 15min | vanta-lead | `git status` limpio para build artifacts | ✅ DONE 2026-07-22 |
| P0-2 | Advisory-ignore register: cada advisory ignorado en `cargo audit` necesita owner + razón + expiry + issue link | 🟡 1-2h | vanta-audit | `cargo audit` report con register documentado | ⬜ PENDING |
| P0-1 | `continue-on-error: true` en workflows — auditar cada instancia, dividir en hard gates vs experimental documentado | 🟡 1-2h | vanta-lead | Cada instancia auditada, documentada, decidida | ⬜ PENDING |

### P1 — Alta prioridad mantenibilidad

| ID | Hallazgo | Esfuerzo | Sub-Agente | Verificación | Estado |
|----|----------|----------|------------|-------------|--------|
| P1-4 | Clasificar adaptadores por tier (oficial/comunidad/experimental) + documentar en ADR | 🟡 2h | vanta-arch | ADR con clasificación | ⬜ PENDING |
| P1-3b | WASM storage — validar que OPFS/IndexedDB tenga tests dedicados | 🟡 1-2h | vanta-engine | Tests WASM existentes documentados | ⬜ PENDING |

### P2 — Limpieza media prioridad

| ID | Hallazgo | Esfuerzo | Sub-Agente | Verificación | Estado |
|----|----------|----------|------------|-------------|--------|
| P2-1 | Renombrar `README.MD` → `README.md` en git index (casing) | 🟢 5min | vanta-lead | `git ls-files README*` muestra `README.md` | ✅ DONE (commit 8e3bfe6) |
| P2-4 | Política para `fuzz/Cargo.lock` — commit o gitignore + enforce en `.gitignore` | 🟢 15min | vanta-lead | Decisión documentada + `.gitignore` actualizado | ✅ DONE (commit 8e3bfe6, fuzz/Cargo.lock en .gitignore) |
| P2-2 | Mapa contribuidor: "qué test corre para qué claim" — archivo conciso | 🟡 1-2h | vanta-docs | `docs/TEST_MAP.md` creado | ⬜ PENDING |
| P2-3 | Inventario de unsafe/unwrap en APIs públicas y storage hot paths | 🟡 1-2h | vanta-audit | Inventario en `docs/UNSAFE_INVENTORY.md` | ✅ DONE (commit 8e3bfe6) |

### Estratégico — Discusión + ADR

| ID | Hallazgo | Esfuerzo | Sub-Agente | Verificación | Estado |
|----|----------|----------|------------|-------------|--------|
| P1-1 | Surface-area expansion: decisión sobre política de expansión de superficie | 🟡 1h | vanta-arch | ADR de decisión | ⬜ PENDING |

## Ejecución

Una tarea a la vez, ordenada por prioridad. Si una tarea requiere discusión → marcarla como BLOQUEADA y continuar.

```yaml
fail_mode: stop   # parar al primer error, corregir, continuar
parallel: false   # una tarea por vez
```
