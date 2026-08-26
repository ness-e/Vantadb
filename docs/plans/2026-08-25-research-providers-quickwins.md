# Plan: Quick wins Research providers (INV-providers-01)

> **Fecha:** 2026-08-25 · **Origen:** `/research providers` → Fase D (decisiones HITL)
> **Informe:** `docs/reviews/research-providers-20260825.md` (score 4.0/10)
> **Backlog:** filas PROV-01..12 en `docs/Backlog.md` P45
> **Ejecutar con:** `/pipeline run docs/plans/2026-08-25-research-providers-quickwins.md`

## Alcance

Solo hallazgos 🟢 mecánicos aprobados. Los estratégicos (PROV-04/05/10/11/12) quedan
en Backlog para planificación separada.

## Wave 1 — Fixes mecánicos (<2h total)

| Task | Backlog | Descripción | Contrato de verificación |
|------|---------|-------------|--------------------------|
| 1 | PROV-01 | Fix compile openai: añadir `exclude_superseded: false` en `providers/openai/src/python.rs:296-302` | `cargo check --manifest-path providers/openai/Cargo.toml` exit 0 |
| 2 | PROV-06 | Pasar `timeout` a kwargs de `litellm.embedding()` cuando esté seteado (`python.rs` embed) | grep timeout en embed kwargs; crate compila |
| 3 | PROV-03 | Regenerar los 3 `.pyi` desde firmas reales (namespace, get/list/delete/list_namespaces, model/base_url/timeout) | Firmas .pyi == pymethods (revisión manual) |
| 4 | PROV-07 | ValueError en distance_metric inválido; warning en metadata descartada (los 3 crates) | Compila + caso de test manual documentado |
| 5 | PROV-08 | READMEs ×3 completos: tabla 7 métodos, quickstart, requisito pip del SDK proveedor | README menciona todos los métodos y el requisito |

## Wave 2 — Tests verdes

| Task | Backlog | Descripción | Contrato de verificación |
|------|---------|-------------|--------------------------|
| 6 | PROV-02 | Actualizar tests ×3 a firma actual (`search(ns, emb, ...)`), eliminar `create_namespace` fixture ollama | pytest de cada crate pasa localmente (build maturin manual necesario) |
| 7 | PROV-09 | `pytest.importorskip` + test embed() mockeado + job CI que corra tests Python de providers | Workflow CI incluye step pytest providers |

## Notas

- Los crates están FUERA del workspace (MSVC linker, Cargo.toml:638) — compilar con
  `cargo check --manifest-path`, no `cargo check -p`.
- Los tests Python requieren wheel compilada (maturin develop) — sin CI previo,
  primera corrida puede revelar problemas de build.
- Deuda técnica nueva = 0 (todos los cambios son fixes/reemplazos).
