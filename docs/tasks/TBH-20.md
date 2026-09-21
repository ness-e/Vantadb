# TBH-20 — ci-examples-12.yml: 3-OS matrix (ubuntu + windows + macos)

## Contexto (CI infra hardening — Phase 1)

Workflow `ci-examples-12.yml` corre solo en `ubuntu-latest` para los
examples de Rust + Python. La auditoría multi-agente del 2026-08-30 (gap
audit CI/CD) y la investigación `docs/research/validacion/07` (§"Smoke
de ejemplos en Windows/macOS") recomiendan explícitamente añadir la
matrix 3-OS al job de ejemplos Python (P1 / baja prioridad, ~2h esfuerzo).

Patrón ya probado y aprobado en el repo:
- `.github/workflows/release-wheels-60.yml:36-42` — `matrix.os: [ubuntu-latest, macos-latest, windows-latest]`
- `.github/workflows/ci-rust-10.yml` — tres jobs separados (`test`, `test-windows`, `test-macos`) en vez de matrix (variante válida)

## Contrato

- Ambos jobs (`rust-examples`, `python-examples`) deben tener
  `strategy.matrix.os: [ubuntu-latest, windows-latest, macos-latest]`
  y `runs-on: ${{ matrix.os }}`.
- `strategy.fail-fast: false` (consistente con `release-wheels-60.yml:40`)
- `name` del job debe incluir `(${{ matrix.os }})` para que el runner
  identifique el OS en la UI.
- NO TOCAR steps, paths, concurrency, permissions, env.
- NO añadir `continue-on-error` (Regla 2 — prohibido).
- NO añadir lógica nueva, scripts, tests.

## Archivos clave

- `.github/workflows/ci-examples-12.yml` (modificar — agregar matrix)
- `.github/workflows/release-wheels-60.yml` (referencia del patrón matrix 3-OS)
- `.github/actions/rust-setup/action.yml` (compatibilidad cross-OS — verified Linux-only safe)
- `.github/workflows/ci-rust-10.yml` (referencia alternativa con jobs separados)
- `docs/research/validacion/07-repo-esencial-y-fiabilidad-first-run.md` (gap original §52)

## Impacto mapeado (Regla 0)

- Archivo leído completo: `ci-examples-12.yml` (122 líneas, 2 jobs: `rust-examples`, `python-examples`)
- Referencias salientes: ninguna — es un workflow de CI standalone (no se importa desde otros workflows)
- Referencias entrantes: ningún otro workflow invoca `ci-examples-12.yml`
- `rust-setup` action es cross-OS safe (system-deps y swap son `runner.os == 'Linux'`)
- Veredicto: cambio aislado en workflow standalone; blast radius = 0 archivos aguas abajo
- Compatibilidad cross-OS de los examples:
  - **Rust examples**: usan storage local (`./examples_*_data`); pure Rust, cross-OS por diseño
  - **Python examples**: usan stdlib + `vantadb_py` + libs (langchain, crewai, autogen, etc.) — todas pure-Python o con wheels cross-OS (sentence-transformers, langchain, etc. publican wheels win/mac/linux)

## Plan

1. **S1 (ACT):** Edit `rust-examples` job — añadir matrix.os 3-OS, `runs-on: ${{ matrix.os }}`, name con OS
2. **S2 (ACT):** Edit `python-examples` job — mismo patrón
3. **S3 (VERIFY):** `python -c "import yaml; yaml.safe_load(...)"`
4. **S4 (VERIFY):** `Select-String` matches both OS labels (≥2 matches)
5. **S5 (VERIFY):** `actionlint .github/workflows/ci-examples-12.yml` → 0 errors
6. **S6 (COMMIT):** `ci(TBH-20):` conventional commit

## Spec

N/A (infra-only YAML edit, sin API pública, sin lógica nueva, sin spec).

## Steps

- [x] **S1 (ACT):** Edit `rust-examples` job — add `strategy.matrix.os` (3-OS) + `runs-on: ${{ matrix.os }}` + name suffix
- [x] **S2 (ACT):** Edit `python-examples` job — mismo patrón
- [x] **S3 (VERIFY):** `python -c "import yaml; yaml.safe_load(open('.github/workflows/ci-examples-12.yml')); print('OK')"` → OK
- [x] **S4 (VERIFY):** `Select-String -Path ci-examples-12.yml -Pattern "windows-latest|macos-latest"` → 2 matches (line 50, 86)
- [x] **S5 (VERIFY):** `actionlint .github/workflows/ci-examples-12.yml` → exit 0
- [x] **S6 (COMMIT):** `ci(TBH-20): extend ci-examples to 3-OS matrix (ubuntu+windows+macos)`