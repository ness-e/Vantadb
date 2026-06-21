# Auditoría de Workflows, Tests y Relaciones — VantaDB

**Fecha:** 2026-06-20
**Última actualización:** 2026-06-20
**Tipo:** Revisión estática de configuración CI/CD y declaraciones de tests
**Alcance:** `.github/workflows/*.yml`, `.config/nextest.toml`, `Cargo.toml` (`[[test]]`), `docs/operations/CI_POLICY.md`

---

## Estado Actual — 7/9 Hallazgos Corregidos

| # | Hallazgo | Severidad | Estado |
|---|----------|-----------|--------|
| 1 | `hnsw_recall` ID mismatch | 🔴 Crítico | ✅ CORREGIDO |
| 2 | Tests sin `[[test]]` explícito | 🟡 Alto | ✅ CORREGIDO |
| 3 | `multilingual_tokenizer_integration` no excluido | 🟡 Alto | ✅ CORREGIDO |
| 4 | `mcp_tests` sin clasificación CI | 🟡 Alto | ✅ CORREGIDO |
| 5 | `--features cli` implícito en storage-persistence | 🟠 Medio | ✅ CORREGIDO |
| 6 | `chaos_integrity` sin `required-features` | 🟠 Medio | ❌ PENDIENTE |
| 7 | `test-threads = 2` global | 🔵 Bajo | ❌ PENDIENTE |
| 8 | `columnar` nunca corre en CI | 🔵 Bajo | ✅ CORREGIDO |
| 9 | Filtro `not test(...)` frágil | 🔵 Bajo | ✅ CORREGIDO |

---

## ✅ Hallazgos Corregidos

### 1. 🔴 `hnsw_recall` → `hnsw_recall_certification`
- `.config/nextest.toml:28` ahora usa `not binary(hnsw_recall_certification)`

### 2. 🟡 Tests con `[[test]]` explícito agregados
- `Cargo.toml:400-414` — se agregaron entradas para `fjall_cold_copy_restore`, `property_durability`, `fuzz_proptest`, `multilingual_tokenizer_integration`

### 3. 🟡 `multilingual_tokenizer_integration` excluido y clasificado
- `.config/nextest.toml:54` → `not binary(multilingual_tokenizer_integration)`
- `heavy_certification.yml:198` → agregado al job `other-heavy`
- `CI_POLICY.md:60` → documentado

### 4. 🟡 `mcp_tests` clasificado
- `.config/nextest.toml:53` → `not binary(mcp_tests)`
- `heavy_certification.yml:210-214` → nuevo bloque `--package vantadb-mcp --test mcp_tests`
- `CI_POLICY.md:60` → documentado

### 5. 🟠 Documentación de features implícitos
- `heavy_certification.yml:115` → comentario que `prefetch_benchmark` y `file_locking_stress` requieren `cli`

### 8. 🔵 `columnar` ahora corre en CI
- `heavy_certification.yml:186` → `--features cli,arrow` (antes solo `cli`)
- `heavy_certification.yml:199` → `--test columnar` agregado

### 9. 🔵 Filtro frágil eliminado
- `not test(integrations_certification)` ya no existe en el filter

### Otros cambios adicionales detectados
- `binary_id(...)` → `binary(...)` en todo `.config/nextest.toml`
- `integration` agregado al filtro audit (`nextest.toml:38`)
- `memory_telemetry` y `concurrent_insert_preserves_hnsw_invariants` agregados al filtro audit y a `heavy_certification.yml`

---

## ❌ Hallazgos Pendientes (2)

### 6. 🟠 `chaos_integrity` sin `required-features = ["failpoints"]`

| Campo | Valor |
|-------|-------|
| **Archivo** | `Cargo.toml:199` |
| **Problema** | Corre en `heavy_certification.yml` con `--features failpoints`, pero el `[[test]]` en Cargo.toml no declara el requirement. |
| **Impacto** | Compila sin failpoints (pasa vacío o falla distinto) |

### 7. 🔵 `test-threads = 2` global en nextest (no OS-específico)

| Campo | Valor |
|-------|-------|
| **Archivo** | `.config/nextest.toml:64` |
| **Problema** | El límite aplica también a Linux, donde podría usar más paralelismo. El comentario dice que es por Windows MSVC. |

---

## 📊 Resumen Final

| Estado | Cantidad |
|--------|----------|
| ✅ Corregidos | 7 |
| ❌ Pendientes | 2 |
| **Total hallazgos** | **9** |
