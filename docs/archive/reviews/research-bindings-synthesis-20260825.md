# INV-SYNTHESIS — Síntesis global del programa de investigación INV-* (2026-08-25)

> Generada por `/research synthesis` (INV-DECIDE, sesión 2026-08-26) como artefacto de cierre.
> Insumos: los 9 reportes `docs/reviews/research-<modulo>-20260825.md` + sus fases D por-módulo
> ya materializadas (Backlog P40-P46, `docs/avance/investigaciones.md`, wontfix, 7 planes quick-wins).

## 1. Scores globales por módulo

| Módulo | Score | Hallazgos | Informe |
|---|---|---|---|
| `vantadb-python` | 8.1 | 9 | research-vantadb-python-20260825.md |
| `vantadb-server` | 8.0 | 14 | research-vantadb-server-20260825.md |
| `desktop` | 7.4 | 15 | research-desktop-prod-20260825.md |
| `vantadb-ts` | 7.2 | 14 | research-vantadb-ts-20260825.md |
| `web` | 7.2 | 11 | research-web-prod-20260825.md |
| `vantadb-wasm` | 6.8 | 23 | research-vantadb-wasm-20260825.md |
| `integrations` | 6.3 | 11 | research-integrations-20260825.md |
| `vantadb-node` | 4.8 | 10 | research-vantadb-node-20260825.md |
| `providers` | 4.0 | 14 | research-providers-20260825.md |
| **Promedio** | **6.6** | **121** | — |

Posición competitiva resumida: capacidad técnica ≥7.0 en 5 de 9 superficies (server único con rate-limit fail-closed nativo; python SDK maduro; wasm único browser engine con persistencia real + híbrido + grafo). Los dos scores bajos (`node` 4.8, `providers` 4.0) son brechas de **distribución/estado**, no de diseño.

## 2. Patrones transversales (solo visibles en síntesis)

1. **Distribución > capacidad**: `vantadb-node` 404 en npm (BND-08, único P0 🔴 del programa); `vantadb-ts` 12 descargas/semana vs 35K-465K competidores; `wasm` 187/mes vs Orama 5.44M.
2. **Providers roto, no flojo**: `vantadb-openai` no compila contra core actual (E0063 `exclude_superseded`) — regresión activa (PROV-01 Critical).
3. **Pérdida de trazabilidad ×4**: MOD-22..24, MOD-25..28, MOD-41..45, MOD-46..50 derivados al Backlog y desaparecidos sin completar ni archivar. Violación repetida del invariant de `progreso`.
4. **Regla 11 bloquea marketing**: ningún SDK publica benchmark reproducible (PY-02, TS-09, PERF-BENCH-01, DESKTOP H-15).
5. **Paridad de bindings dispersa**: sparse_vector `None`, `remove_edge` ausente, límites FFI divergentes, semántica score/distance inconsistente entre transports (WSM-06/09/10, TS-04, BND-10, PY-01).

## 3. Decisiones HITL de la sala global (sesión 2026-08-26)

| # | Pregunta | Decisión |
|---|---|---|
| Q1 | Apuesta estratégica prioritaria | **Paridad de bindings**, con política de excepciones documentadas (BINDINGS_NAMESPACES.md = matriz canónica; todo hueco nuevo = fila Backlog o excepción con motivo) |
| Q2 | Dimensiones inaceptables hoy | Providers roto · npm `vantadb-node` 404 · trazabilidad sistémica · CSP desktop `null` |
| Q3 | ¿Contradicción con roadmap? | Ninguna — refinamiento del norte existente (preparación Show HN) |
| Q4-Q5 | Ejecución | **Los 7 planes quick-wins se ejecutan**, paralelizando sub-agentes solo entre directorios disjuntos; verificaciones pesadas serializadas al cierre por el lead |
| Q6 | Política de paridad | Paridad con excepciones documentadas |
| Q7 | PY H-09 identidad import | **Consolidar en `import vantadb`**; deprecar alias interno `vantadb_py` (nueva fila PY-03) |
| Q8 | Contramedida trazabilidad | **Derivación atómica**: derivar = crear la fila en el mismo commit del registro; chequeo mecánico periódico en Trigger 4 de `progreso` (regla en `docs/avance/meta.md`) |
| Q9 | Síntesis formal | Generada (este documento) + fila en `docs/reports/INDEX.md` |

## 4. Plan de ejecución aprobado

| Wave | Plan | Directorio | Notas |
|---|---|---|---|
| 1 | 2026-08-25-research-providers-quickwins | `providers/` | PROV-01 Critical primero |
| 1 | 2026-08-25-py-quickwins | `vantadb-python/` | — |
| 1 | 2026-08-25-research-desktop-quickwins | `desktop/` | Incluye CSP mínima Tauri (H-01 🔴) |
| 1 | 2026-08-25-research-vantadb-ts-quickwins | `vantadb-ts/` | Único agente autorizado a tocar `.github/workflows/` en wave 1 (TS-06) |
| 2 | 2026-08-25-wasm-quickwins | `vantadb-wasm/` | — |
| 2 | 2026-08-25-integrations-research-wins | `integrations/` | QW-1..5 absorben MOD-46..50 |
| 2 | 2026-08-25-vantadb-node-hardening | `vantadb-node/` | Wave 1 del plan; BND-08 requiere tokens npm del owner (fuera de alcance agentes) |

Restricción transversal: cada agente corre solo checks livianos y acotados a su crate/dir (`cargo check -p`, tests filtrados, build del paquete). Prohibido en paralelo: `just verify`, `dev-tools/verify.ps1`, `cargo nextest --workspace`. El lead ejecuta la verificación completa una vez al cierre de las 2 waves.

---

*INV-DECIDE ejecutado por `/research synthesis`. Fuente de decisiones: sesión HITL 2026-08-26
(9 preguntas, 0 defaults aplicados). Registro vivo: `docs/avance/investigaciones.md` §INV-DECIDE.*
