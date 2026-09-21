# TBH-11: Extend `heavy-bench-nightly-51.yml` from 5 → 8 benches

## Metadata
- **Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md`
- **Creado:** 2026-08-31T10:00
- **last-synced:** 2026-08-31T10:00
- **Estado:** 🔄 IN PROGRESS
- **Branch:** develop
- **Sub-agente:** vanta-lead

## Contexto
El workflow `heavy-bench-nightly-51.yml` corre nightly (cron `0 3 * * *`) con 5 benches:
`hnsw_pure`, `hybrid_queries`, `stress_test`, `bench_concurrent`, `high_density`. La auditoría
multi-agente del 2026-08-30 (gap audit benchmarks) detectó que 4 benches diagnósticos —
`canonical_p99`, `memory_budget`, `incremental_bench`, `ivf_bench` — existen en `benches/*.rs`
y están registrados como `[[bench]]` en `Cargo.toml`, pero NO se ejecutan en nightly. Esto
significa que las regresiones en P99 de insert/search, consumo de memoria RSS, comportamiento
de índices incrementales, y clustering IVF pasan desapercibidas entre releases.

## Archivos clave
| Path | Rol |
|---|---|
| `.github/workflows/heavy-bench-nightly-51.yml` | Modificar (config-only, aditivo) |
| `benches/canonical_p99.rs` | Existe (referencia, no se toca) |
| `benches/memory_budget.rs` | Existe (referencia, no se toca) |
| `benches/incremental_bench.rs` | Existe (referencia, no se toca) |
| `benches/ivf_bench.rs` | Existe (referencia, no se toca) |

## Contrato (verificable mecánicamente)
| Check | Comando | Esperado |
|---|---|---|
| YAML válido | `python -c "import yaml; yaml.safe_load(open('.github/workflows/heavy-bench-nightly-51.yml')); print('OK')"` | `OK` |
| 4 benches presentes | `Select-String -Path .github/workflows/heavy-bench-nightly-51.yml -Pattern "canonical_p99\|memory_budget\|incremental_bench\|ivf_bench"` | ≥ 4 matches (≥1 por bench) |
| actionlint | `actionlint .github/workflows/heavy-bench-nightly-51.yml` | 0 errors |
| Sin schedule change | `Select-String -Pattern "0 3 \* \* \*" .github/workflows/heavy-bench-nightly-51.yml` | schedule intacta |
| Benches originales intactos | `Select-String -Pattern "hnsw_pure\|hybrid_queries\|stress_test\|bench_concurrent\|high_density"` | ≥ 5 matches |

## Acceptance Criteria
1. Los 4 nuevos benches (`canonical_p99`, `memory_budget`, `incremental_bench`, `ivf_bench`)
   están agregados al job `light-benchmarks` (siguen el mismo patrón que los existentes)
2. NO se modifican ni quitan los 5 benches existentes
3. NO se cambia el `cron: '0 3 * * *'`
4. **Ponytail reflex:** config-only. NO se crean scripts. NO se tocan `benches/*.rs`

## Decisión

**Target placement:** los 4 nuevos steps se agregan al final del job `light-benchmarks`
(antes del step `Upload light results`). Mismo formato que los existentes:
```yaml
- name: Run <bench> benchmarks
  run: cargo bench --bench <bench> -- --nocapture 2>&1 | tee nightly_<bench>_results.txt
```

**Naming de archivo de output:** `nightly_<bench>_results.txt` (camelCase conservado —
`canonical_p99` → `nightly_canonical_p99_results.txt`, igual que los demás preservan el
guion bajo del nombre original).

**Razón:** el glob `nightly_*_results.txt` en el step `Upload light results` los recoge
automáticamente. Sin este patrón, los nuevos artifacts no se subirían.

## Cambios (esperados)
- `.github/workflows/heavy-bench-nightly-51.yml`: +4 steps en `light-benchmarks` job
  (después del step `Run concurrent benchmarks`, antes de `Upload light results`).
  Total diff: +16 líneas (4 steps × 4 líneas c/u = 16).

## Steps

### Step 1: Discovery
- **Acción:** Leer workflow completo + verificar existencia de los 4 `benches/*.rs`
- **Verify:** `glob benches/*.rs` muestra los 4 archivos
- **Estado:** ✅ DONE

### Step 2: Update task state
- **Acción:** Marcar TBH-11 in-progress en plan file / campaign system
- **Estado:** 🔄 PENDING

### Step 3: Edit workflow
- **Acción:** Insertar 4 steps `Run <bench> benchmarks` en `light-benchmarks` job
- **Verify:** Diff muestra +16 líneas, 0 changes en schedule o benches originales
- **Estado:** 🔄 PENDING

### Step 4: Verify YAML syntax
- **Acción:** `python -c "import yaml; yaml.safe_load(open('.github/workflows/heavy-bench-nightly-51.yml')); print('OK')"`
- **Verify:** Output = `OK`
- **Estado:** 🔄 PENDING

### Step 5: Verify bench names present
- **Acción:** `Select-String -Pattern "canonical_p99|memory_budget|incremental_bench|ivf_bench"`
- **Verify:** ≥ 4 matches (1 por bench name)
- **Estado:** 🔄 PENDING

### Step 6: Verify original benches intact + schedule intact
- **Acción:** `Select-String` sobre los 5 nombres originales + `0 3 * * *`
- **Verify:** 5 matches originales + schedule intacta
- **Estado:** 🔄 PENDING

### Step 7: actionlint
- **Acción:** `actionlint .github/workflows/heavy-bench-nightly-51.yml`
- **Verify:** Exit 0, sin errors
- **Estado:** 🔄 PENDING

### Step 8: Commit + close
- **Acción:** `git add` + commit conventional + update task state completed
- **Estado:** 🔄 PENDING

## Dependencias
- TBH-01..05, TBH-12..15, TBH-19, TBH-22, TBH-23: predecesoras (✅ completed)
- Ningún task dependiente explícito en el plan

## Notas
- Config-only. 0 cambios en código fuente. 0 dependencias nuevas.
- El bench `canonical_p99` es el canónico para Regla 9 (medición obligatoria de optimizaciones).
  Al incluirlo en nightly, las regresiones en P99 se detectan diariamente sin intervención manual.
- `incremental_bench` cubre el path de inserción incremental (vacío → 100k vectores en pasos),
  relevante para detectar leaks de memoria por chunks.
- `memory_budget` mide RSS en escenarios de carga sostenida.
- `ivf_bench` cubre el índice IVF (Inverted File) — alternativa a HNSW para casos de baja latencia.
- Total nightly runtime estimado: +~10-15 min para 4 benches adicionales (depende del runner).