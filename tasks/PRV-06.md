# PRV-06: All providers — docstrings + error handling

## Metadata
- **Plan file:** `docs/plans/2026-07-22-adapter-10of10-campaign.md`
- **Creado:** 2026-07-22T12:00
- **Estado:** ⬜ PENDING

## Steps

### Step 1: Python docstrings for all 3 providers
- **Archivos:** `providers/*/src/python.rs`
- **Acción:** En PyO3, los docstrings se agregan como `///` antes de `#[pyclass]`, `#[pymethods]`, y `#[pyfunction]`. Agregar docstrings descriptivos a cada método público expuesto.
- **Verify:** `cargo check --workspace`
