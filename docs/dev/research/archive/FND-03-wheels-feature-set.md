# FND-03 — Feature set mínimo + wheels CI compile matrix verde

**Fecha:** 2026-08-16 · **Wave:** P20a · **Prio:** 🟡 · **Tipo:** verificación (estado OK, sin cambios)
**Owner:** vanta-lead · **Contrato:** "feature set mínimo compila (`--no-default-features`) + job CI compile matrix verde"
**Complementa:** FND-16 (multi-target CI — los wheels ya corren por PR a main, 3 OS + smoke tests)

---

## 1. Objetivo

Los wheels del release deben ser útiles para el dev de Show HN que prueba con un comando. Verificar que:
1. El feature set mínimo del core compila (`cargo check -p vantadb --no-default-features`).
2. El CI compila la matrix de wheels (linux/macos/windows) sin features que falten en la config mínima.
3. abi3 y muchoslinux mínimo están declarados.

## 2. Features por target

### 2.1 Core `vantadb` (`Cargo.toml` workspace)

| Feature | Tipo | En wheels |
|---|---|---|
| `cli`, `arrow`, `roaring`, `advanced-tokenizer`, `fs2`, `sysinfo` | default | ❌ NO (excluido por `default-features = false`) |
| `fjall`, `memmap2`, `rayon` | default | ✅ **SÍ — set mínimo del wheel** |
| `rocksdb`, `prometheus`, `server`, `tls`, `tui`, `python_sdk`, `failpoints`, `remote-inference`, `wasm`, `custom-allocator`, `jemalloc`, `opentelemetry`, `hot-reload`, `encryption`, `wal-shipping`, `pitr`, `bayesian_decay`, `async-ingestion`, `async-io` | opt-in | ❌ NO |

**Default del core:** `cli + arrow + fjall + roaring + advanced-tokenizer + memmap2 + fs2 + sysinfo + rayon` (`Cargo.toml:97`).

### 2.2 Binding `vantadb_py` (`vantadb-python/Cargo.toml`)

El crate PyO3 fija explícitamente el feature set mínimo del core que se empaqueta:

```toml
vantadb = { path = "../", default-features = false, features = ["fjall", "memmap2", "rayon"] }
```

- `default-features = false` → **sin CLI, sin arrow, sin tokenizer avanzado, sin sysinfo, sin roaring**.
- `fjall` (backend de storage embebido persistente) + `memmap2` (archivos mmap) + `rayon` (paralelismo) = el mínimo útil para el dev que hace `pip install vantadb-py` y prueba con un comando.

### 2.3 Job de wheels (`release-wheels-60.yml`)

| Aspecto | Valor | Cita |
|---|---|---|
| Matrix | `os: [ubuntu-latest, macos-latest, windows-latest]` | `release-wheels-60.yml:39-42` |
| Build | maturin-action, `--release --out dist --manifest-path ./vantadb-python/Cargo.toml` | `release-wheels-60.yml:82-90` |
| Features extra | **ninguno** (`--features` no se pasa) → empaqueta el set mínimo de `vantadb_py` | `release-wheels-60.yml:86` |
| manylinux | `2_28` | `release-wheels-60.yml:87` |
| musllinux | `1_2` | `release-wheels-60.yml:88` |
| abi3 | `abi3-py311` (en `vantadb-python/Cargo.toml:15`) → wheels `cp311-abi3`, compat CPython 3.11+ | `vantadb-python/Cargo.toml:15` |
| Trigger PR | `pull_request` branches main, paths `src/**`, `vantadb-python/**`, `Cargo.toml`, `Cargo.lock`, workflow | `release-wheels-60.yml:10-17` |
| Smoke test | por OS: import + `pytest vantadb-python/tests/test_sdk.py` | `release-wheels-60.yml:92-114` |
| Coherencia | `cargo test --test version_coherence --no-default-features --features fjall` antes del build | `release-wheels-60.yml:80` |

**Consistencia cross-file:** el job de coherencia (step pre-build) usa `--no-default-features --features fjall` — consistente con el set mínimo que empaqueta el wheel. No hay feature consumida por `vantadb_py` que falte en la config mínima del core.

## 3. Verificación mecánica

| Comando | Resultado |
|---|---|
| `cargo check -p vantadb --no-default-features` | ✅ exit 0 (7 warnings pre-existentes en `src/storage/vfile_mmap.rs`, 0 errores) |
| `cargo check -p vantadb --no-default-features --features fjall,memmap2,rayon` | ✅ exit 0 (set real del wheel compila) |

Los 7 warnings (`unnecessary unsafe block` con memmap2 0.9) son pre-existentes, no bloquean el contrato y no son de esta tarea.

## 4. abi3 / manylinux declarado

- **abi3:** ✅ `abi3-py311` en `vantadb-python/Cargo.toml:15` (pyo3 0.29). Los wheels se taggean `cp311-abi3` y sirven para CPython ≥3.11 — coherente con `requires-python = ">=3.11"` en `pyproject.toml:10` y los classifiers 3.11/3.12/3.13.
- **manylinux mínimo:** ✅ `2_28` (y `musllinux: 1_2`) en el action de maturin — declarado el mínimo soportado.

## 5. Observación sobre `[tool.maturin] features` (pyproject.toml)

`pyproject.toml:39` declara `features = ["pyo3/extension-module"]`. Validado contra la documentación oficial de maturin (`maturin.rs/config` — sección Cargo options): la key `features` pasa `--features` a cargo sobre el crate construido. La sintaxis `dep/feature` es válida, y el feature ya está activado en `vantadb-python/Cargo.toml:15` (`pyo3 = { ..., features = ["extension-module", ...] }`). **Redundante pero inofensivo — no es un gap.**

## 6. Veredicto

**ESTADO OK — SIN CAMBIOS.**

| Punto del contrato | Estado |
|---|---|
| `cargo check -p vantadb --no-default-features` pasa | ✅ |
| Reporte documenta features por target | ✅ (sección 2) |
| abi3/manylinux declarado | ✅ (sección 4) |
| Job CI compile matrix verde (3 OS) | ✅ por PR (FND-16) + smoke tests por OS |
| Changes de CI + actionlint | N/A — no se tocó ningún workflow |

**No se modificó** `Cargo.toml` (workspace), `vantadb-python/Cargo.toml`, `vantadb-python/pyproject.toml` ni `release-wheels-60.yml`: la config actual ya empaqueta el feature set mínimo y la matrix ya es verde por PR. Inventar cambios habría sido deuda.

## 7. Alcance y límites

- El alcance P20a (Backlog.md:485 DoD) verifica "feature set mínimo compila + job CI verde" — cumplido.
- El aislamiento fino de features (vector-only sin grafos) del enunciado original queda delegado a **FND-23** (decisión default-on vs opt-in de grafos con telemetría real, post-launch) — no se decidió acá por diseño (no intuición).
- Deuda opcional anotada (no bloqueante): `[tool.maturin] features = ["pyo3/extension-module"]` es redundante; se puede limpiar en un futuro PR de pyproject sin impacto.