# SDKB-04 — Docs + gate backward-compat final

> Plan: `docs/plans/2026-08-22-vantadb-bindings-sdk.md` · Ruta: vanta-docs · Cynefin: 🟦 obvio

## Steps

- ✅ **S1 DISCOVERY** — task file creado, impacto mapeado (Regla 0)
- ✅ **S2** — `vantadb-ts/README.md`: sección "Domain Sub-clients" (`db.memory.*`, `db.graph.*`, `db.wiki.*`, `db.system.*`), inglés, backward-compat explícito ("flat API unchanged")
- ✅ **S3** — `docs/api/PYTHON_SDK.md`: sección "Domain Sub-clients" equivalente
- ✅ **S4** — `docs/api/BINDINGS_NAMESPACES.md`: referencia cruzada hacia ambos docs en el header canon
- ✅ **S5** — Gate backward-compat: `npm test` → **246 passed / 246** · pytest → **105 passed, 4 skipped** (pre-existentes)
- ✅ **S6** — validate-docs-coverage → **0 gaps**. Fix requerido: 4 getters PyO3 internos (`memory_client`/`graph_client`/`system_client`/`wiki_client`, renombrados vía `#[pyo3(name=...)]`) agregados a `$pyInternals` del validador — no son API Python visible; documentar esos nombres habría sido falso.
- ✅ **S7 CIERRE** — task file sync + recitation §3. Sin commit (instrucción explícita del orquestador).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `docs/plans/2026-08-22-vantadb-bindings-sdk.md`, `docs/api/BINDINGS_NAMESPACES.md`, `vantadb-ts/README.md`, `docs/api/PYTHON_SDK.md`; fuente verificada vía codegraph: `vantadb-ts/src/vantadb.ts:247-343` (getters memory/graph/wiki/system frozen) y `vantadb-python/src/lib.rs:276-329` (`forward_to_db!` Memory 15 / Graph 10 / System 17 / Wiki 1) + getters `#[pyo3(name=...)]` en lib.rs:1972-2006.
- **Referencias hacia dentro:** README TS lista métodos planos; PYTHON_SDK.md lista métodos planos; BINDINGS_NAMESPACES.md es el mapa canon.
- **Referencias entrantes:** `scripts/validate-docs-coverage.ps1` escanea docs/api; SKILL `vantadb`; planes previos citan PYTHON_SDK.md.
- **Veredicto:** cambios solo aditivos en 3 archivos de doc (secciones nuevas + links). Cero código. Backward-compat se prueba con suites existentes intactas.

## Contrato

"`npm test` + `pytest` completos exit 0; READMEs actualizados; validate-docs-coverage 0 gaps"

## Context Save Point

- (vacío — sin interrupciones)
