# Plan de Reparación de Workflows GitHub Actions

> **Meta:** Auditar, diagnosticar y arreglar los 13 workflows + 1 custom action que fallan en CI.

**Diagnóstico inicial:** Basado en análisis estático de los 13 YAML + revisión de logs de ejecuciones reales en `main` obtenidos via `gh run view --log-failed`.

## Errores Reales Detectados en Ejecuciones (main branch)

| Workflow | Jobs Fallidos | Error Concreto (del log) |
|----------|--------------|--------------------------|
| `sec-codeql-30.yml` | `Initialize CodeQL` | `Rust does not support the manual build mode. Please try using one of the following build modes instead: none.` — CodeQL ≥2.26 cambió su API para Rust |
| `ci-web-11.yml` | `Build & Test` | 26 tests e2e fallan en `design-audit-pipeline.spec.ts:248` — `expect(result.cssIssues.length).toBe(0)` → Received: 1. No hay issues de CSS, es el test de auditoría de diseño que encuentra issues reales en páginas |
| `ci-rust-10.yml` | `TSan (ThreadSanitizer)` | `the option 'Z' is only accepted on the nightly compiler` — usa `dtolnay/rust-toolchain@stable` pero `-Z sanitizer=thread` requiere nightly |
| `ci-rust-10.yml` | `ASan (AddressSanitizer)` | Mismo error: `-Z sanitizer=address` requiere nightly, no stable |
| `ci-rust-10.yml` | `Tests (macOS)` | `failed to run custom build command for 'librocksdb-sys v0.17.3+10.4.2'` — `process didn't exit successfully (signal: 6, SIGABRT)`. Build de RocksDB aborta en macOS |
| `ci-rust-10.yml` | `Tests (Windows)` | `FAIL executor::tests::test_consume_io_accumulates` — test específico de Rust que falla en Windows |
| `ci-rust-10.yml` | `Minimal Versions` | `missing field 'namespace' in initializer of 'VantaDBVectorStore'` + `Option<Vec<Option<&pyo3::Bound>>> cannot be used as a Python function argument` |
| `ci-rust-10.yml` | `Clippy Lints` | Errores de linting, `cargo clippy --workspace --all-targets --all-features -- -D warnings` falla |
| `ci-rust-10.yml` | `MSRV Check (1.94.1)` | `cargo check --workspace` falla con la MSRV declarada |
| `ci-rust-10.yml` | `Miri (UB Detection)` | `cargo miri setup` falla |
| `ci-rust-10.yml` | `Tests (Linux)` | Tests fallan con Audit profile |
| `ci-rust-10.yml` | `Code Coverage` | `cargo llvm-cov nextest` falla |

## Workflows Exitosos (no requieren reparación)
- `PERF: Benchmarks — Python Integration` — ✅ success
- `GATE: Docs — Lint & Frontmatter` — ✅ success
- `Dependabot Updates` — ✅ success

## Hallazgos Transversales

| # | Problema | Archivos Afectados |
|---|----------|-------------------|
| T1 | **Node 20 deprecation** — logs muestran `Node.js 20 is deprecated. This workflow is running with Node 24 by default` | Todos con `actions/checkout` + Node 20 |
| T2 | **Toolchain incorrecto en sanitizers** — ASan/TSan usan `@stable` pero necesitan `@nightly` | `ci-rust-10.yml:341-401` |
| T3 | **CodeQL build-mode obsoleto** — `build-mode: manual` ya no funciona para Rust | `sec-codeql-30.yml:33` |
| T4 | **Librocksdb-sys falla en macOS** — abort signal en build script | `ci-rust-10.yml:163-166` |
| T5 | **Test de auditoría de diseño demasiado sensible** — CSS issues falsos positivos | `ci-web-11.yml` + `web/e2e/design-audit-pipeline.spec.ts` |
| T6 | **Scripts externos frágiles** | `download_benchmark_datasets.sh`, `bench_regression.py`, `vantadb_local_bench.py` |
| T7 | **Versiones SHA inconsistentes** de `Swatinem/rust-cache` y `taiki-e/install-action` | `rust-setup/action.yml`, `ci-rust-10.yml`, `fuzz-40.yml` |
| T8 | **Acciones sin pin SHA** (`dtolnay/rust-toolchain`, `pypa/gh-action-pypi-publish`) | Todos los workflows |
| T9 | **`npm ci` sin caché** en gate-docs | `gate-docs-21.yml:29` |
| T10 | **Timeout sin margen** | `ci-rust-10.yml`, `heavy-*.yml` |
| T11 | **Sin caché Playwright browsers** | `ci-web-11.yml` |

---

# Prompts Reutilizables

Cada prompt está diseñado para copiarse y pegarse en una sesión con el agente, cargando **writing-plans** primero. Incluyen qué skills cargar, qué archivos leer y qué verificar.

---

## PROMPT A: Auditoría Completa de Workflows

> **Propósito:** Escanear todos los workflows, detectar problemas comunes y producir un reporte.
> **Skills a cargar:** `ci-cd-and-automation`, `systematic-debugging`, `writing-plans`

```
Carga las skills ci-cd-and-automation, systematic-debugging y writing-plans.

Necesito que audites TODOS los workflows en .github/workflows/ y la custom action en .github/actions/rust-setup/.

Para cada archivo YAML, verifica:

1. **Pin SHA seguro** — cada `uses:` debe tener SHA completo de 40 caracteres, NO version tags (v1, v2, @stable, @nightly, @release/v1). Ej: `actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11` en vez de `actions/checkout@v4`.
2. **Consistencia de SHA** — el MISMO action debe tener el MISMO SHA en todos los archivos. Si `Swatinem/rust-cache` aparece con SHA distinto en dos lugares, es un bug.
3. **Node.js version** — usa Node 22 (no 20, que está EOL desde abril 2026).
4. **Caching** — npm/Rust debe usar actions/cache o setup-* nativo (cache: npm). Sin caché = lento, lento = timeouts.
5. **Timeouts** — timeout-minutes debe tener margen (1.5x-2x sobre el tiempo esperado). Timeouts ajustados causan fallos intermitentes.
6. **Scripts referenciados** — existe cada script? Tiene manejo de errores? (set -e, try/except).
7. **Dependencias entre jobs** — si job B necesita output de job A, usa `needs:` y actions/upload-artifact / download-artifact.
8. **Path filters** — `on: push: paths:` debe reflejar los paths reales que afectan el workflow.
9. **Shell consistency** — scripts bash deben tener `shell: bash` explícito, especialmente en Windows.

Produce un reporte en docs/workflow-audit-report.md con secciones:
- Problemas transversales (afectan ≥2 workflows)
- Problemas por workflow (tabla con archivo, línea, riesgo, severidad alta/media/baja)
- Recomendaciones ordenadas por impacto

Usa glob para listar workflows, read para leer cada uno, grep para buscar patrones como "uses:" sin SHA, "node-version: 20", "timeout-minutes:".
```

---

## PROMPT B: Diagnosticar un Workflow Específico (con logs reales)

> **Propósito:** Dado un workflow que falla en GitHub, obtener sus logs reales y diagnosticar.
> **Skills a cargar:** `systematic-debugging`, `ci-cd-and-automation`

```
Carga las skills systematic-debugging y ci-cd-and-automation.

El workflow [NOMBRE.yml] está fallando en GitHub. Sigue estos pasos:

PASO 0 — Setup
```bash
mkdir -p logs
```

PASO 1 — Obtener el estado actual de los runs
```bash
gh run list --workflow [NOMBRE.yml] --limit 10 --json name,conclusion,headBranch,createdAt,url,databaseId \
  --jq '.[] | [.databaseId, .conclusion, .headBranch, .createdAt] | @tsv'
```

PASO 2 — Identificar qué jobs fallaron
```bash
RUN_ID=<último run ID fallido>
gh run view $RUN_ID --json jobs --jq '.jobs[] | select(.conclusion=="failure") | .name + " — " + (.steps[]? | select(.conclusion=="failure") | .name)'
```

PASO 3 — Obtener logs completos de los pasos fallidos
```bash
gh run view $RUN_ID --log-failed > logs/[NOMBRE]-failed-$RUN_ID.log
```

PASO 4 — Obtener logs completos del workflow (especialmente útil para ver setup, caché, steps previos)
```bash
gh run view $RUN_ID --log > logs/[NOMBRE]-full-$RUN_ID.log
```

PASO 5 — Extraer los errores específicos
```bash
# Errores de Rust (compilación, test failures)
rg -i "error\[|error:|FAILED|test.*FAILED|could not compile|failed to run" logs/[NOMBRE]-failed-$RUN_ID.log

# Errores de Node/JS
rg -i "Error:|FAIL|failed|npm ERR" logs/[NOMBRE]-failed-$RUN_ID.log

# Errores de GitHub Actions (setup, actions, steps)
rg -i "##\[error\]|Exit code|process didn.t exit|action failed" logs/[NOMBRE]-failed-$RUN_ID.log
```

PASO 6 — Diagnóstico root-cause
- Lee el workflow completo: .github/workflows/[NOMBRE.yml]
- Lee cualquier action referenciada (ej: rust-setup)
- Para errores de compilación: lee los logs completos e identifica el crate/módulo exacto
- Categoriza: sintáctico (YAML) | dependencia (action/version) | entorno (OS/tool/missing dep) | lógico (test/build command)
- Determina línea exacta del .yml que causa el error

PASO 7 — Guard contra recurrencia
- Propón un cambio que evite que este error vuelva
- Si es de action version: pin SHA
- Si es de toolchain: corregir toolchain@nightly vs @stable
- Si es de test frágil: proponer tolerancia o skip condicional por OS

Devuélveme: archivo exacto + línea + causa raíz (con snippet del log) + fix propuesto.
```

---

## PROMPT C: Arreglar un Workflow Específico + Push + Monitorear

> **Propósito:** Aplicar correcciones a un workflow, pushear y monitorear resultado.
> **Skills a cargar:** `ci-cd-and-automation`, `code-review-and-quality`

```
Carga las skills ci-cd-and-automation y code-review-and-quality.

Voy a arreglar el workflow [NOMBRE.yml]. Las causas de falla detectadas son:
[PEGAR ERROR DEL LOG DE GITHUB]

PARTE 1 — CORRECCIÓN

1. **Lee el workflow actual:** .github/workflows/[NOMBRE.yml]
2. **Pin SHA:** Reemplaza TODOS los `uses:` sin SHA completo.
3. **Node 22:** Cambia node-version: 20 por node-version: 22.
4. **Caché:** npm cache, Swatinem/rust-cache, etc.
5. **Workflow dispatch:** Si falta, agrega `workflow_dispatch:` debajo de `on:`.
6. **Timeouts:** Verifica 1.5x-2x el tiempo esperado.

Después, valida sintaxis:
```bash
python -c "import yaml; yaml.safe_load(open('.github/workflows/[NOMBRE.yml]'))"
```

PARTE 2 — PUSH

```bash
git add .github/workflows/[NOMBRE.yml]
git commit -m "fix(ci): reparar [NOMBRE.yml] — [causa raíz breve]"
git push
```

PARTE 3 — MONITOREO POST-PUSH (espera activa, timeout ~15 min)

```bash
echo "Monitoreando workflow [NOMBRE.yml] después del push..."
POLL=0
while [ $POLL -lt 30 ]; do
  sleep 30
  STATUS=$(gh run list --workflow [NOMBRE.yml] --limit 1 \
    --json status,conclusion,databaseId --jq '.[0]')
  STATE=$(echo $STATUS | jq -r '.status')
  if [ "$STATE" = "completed" ]; then
    CONC=$(echo $STATUS | jq -r '.conclusion')
    RUN_ID=$(echo $STATUS | jq -r '.databaseId')
    if [ "$CONC" = "success" ]; then
      echo "✅ WORKFLOW PASA (run $RUN_ID)"
    else
      echo "❌ WORKFLOW FALLA (run $RUN_ID)"
      mkdir -p logs
      gh run view $RUN_ID --log-failed > logs/[NOMBRE]-postfix-failed-$RUN_ID.log
      echo "Log fallido guardado. Correr PROMPT B."
    fi
    break
  fi
  echo "  esperando (${POLL}/30)... status=$STATE"
  POLL=$((POLL+1))
done
```

Devuélveme: diff de cambios + resultado validación YAML + resultado monitoreo (pasó o falló).
```

---

## PROMPT D: Reemplazar Todos los SHA por Versiones Consistentes

> **Propósito:** Estandarizar todas las acciones a un SHA único por action.
> **Skills a cargar:** `ci-cd-and-automation`

```
Carga la skill ci-cd-and-automation.

Necesito estandarizar los SHA de todas las GitHub Actions en el repo.

Reglas:
1. Cada action (por nombre, ej: Swatinem/rust-cache) debe tener el MISMO SHA en TODOS los archivos
2. NO usar version tags (@v4, @stable, @release/v1)
3. Buscar el SHA más RECIENTE de cada acción desde el repositorio oficial

Pasos:
1. Glob todos los .yml en .github/workflows/ y .github/actions/
2. grep de todos los `uses:` para listar acciones únicas
3. Para cada acción:
   a. dtolnay/rust-toolchain — el SHA más reciente en tags del repo
   b. Swatinem/rust-cache — SHA más reciente
   c. taiki-e/install-action — SHA más reciente
   d. actions/checkout — SHA más reciente
   e. actions/setup-node — SHA más reciente
   f. actions/setup-python — SHA más reciente
   g. actions/cache — SHA más reciente
   h. actions/upload-artifact — SHA más reciente
   i. actions/download-artifact — SHA más reciente
   j. PyO3/maturin-action — SHA más reciente
   k. pypa/gh-action-pypi-publish — SHA más reciente
   l. softprops/action-gh-release — SHA más reciente
   m. github/codeql-action/* — SHA más reciente
   n. actions/github-script — SHA más reciente
   o. actions-rust-lang/setup-rust-toolchain — SHA más reciente
   p. jetli/wasm-pack-action — SHA más reciente
   q. al-cheb/configure-pagefile-action — SHA más reciente
   r. actions/attest-build-provenance — SHA más reciente
4. Edita CADA archivo con replaceAll para el SHA viejo → nuevo
5. Valida sintaxis YAML de cada archivo editado

Devuélveme: tabla de acciones con SHA old → SHA new + archivos editados.
```

---

## PROMPT E: Arreglar Scripts Referenciados por Workflows

> **Propósito:** Hacer robustos los scripts bash/Python que los workflows ejecutan.
> **Skills a cargar:** `ci-cd-and-automation`, `debugging-and-error-recovery`

```
Carga las skills ci-cd-and-automation y debugging-and-error-recovery.

Revisa y fortalece los scripts que los workflows ejecutan:

1. `scripts/download_benchmark_datasets.sh` — Agrega:
   - Verificación de curl disponible (`command -v curl`)
   - Manejo de error si la URL de Stanford cambia (curl --fail --retry 3)
   - Checksum SHA256 de validación después de descargar
   - Tolerancia a interrupciones (trap SIGINT)

2. `scripts/bench_regression.py` — Verifica:
   - Manejo de error si target/criterion no existe
   - Parseo robusto de JSON de criterion
   - Si falla, que no rompa el workflow (exit code 0 con warning)

3. `benchmarks/vantadb_local_bench.py` — Verifica:
   - `vantadb_py` import falla con mensaje claro (ya lo tiene)
   - Cachea el dataset si es muy grande
   - Maneja OOM (out of memory) en GitHub runners pequeños

4. `benchmarks/update_markdown.py` — Verifica:
   - Manejo de error si benchmark_results.json no existe o está vacío
   - Si falla, que no rompa el workflow

Para cada script, lee el archivo, edítalo si es necesario, y verifica que los cambios sean sintácticamente válidos.
```

---

## PROMPT F: Reparación Completa de un Workflow Release (Ejemplo)

> **Propósito:** Arreglar un workflow de release específico (ej: release-wheels-60.yml)
> **Skills a cargar:** `writing-plans`, `ci-cd-and-automation`, `doubt-driven-development`

```
Carga writing-plans, ci-cd-and-automation, y doubt-driven-development.

Voy a arreglar el workflow [release-wheels-60.yml].

Plan de tareas:

TASK 1: Pin SHA + Node 22
- Lee el workflow
- Reemplaza node-version: 20 por 22
- Pin `pypa/gh-action-pypi-publish@release/v1` a un SHA concreto (busca el último release en https://github.com/pypa/gh-action-pypi-publish/releases)
- Pin `dtolnay/rust-toolchain@stable` a SHA
- Verifica consistencia de SHA en Swatinem/rust-cache con el custom action

TASK 2: Caché en compilación
- El build de maturin demora ~20-30 min, se puede cachear el target/
- Agrega caching con `Swatinem/rust-cache` en el job build-wheels
- El cache-bin debe ser false (los wheels son el producto)

TASK 3: Robustez en smoke test
- Lee `vantadb-python/verify_published_wheel.py`
- Los shell scripts deben manejar error si el wheel no existe:
  - Linux: `if [ ! -f "$(ls -t ./dist/vantadb_py-*.whl | head -1)" ]; then echo "No wheel found"; exit 1; fi`
  - Windows: si Get-ChildItem no encuentra archivos

TASK 4: Dependencias entre jobs
- `verify-pypi-install` necesita `needs: publish-pypi` (ya existe, bien)
- `verify-testpypi-install` necesita `needs: publish-testpypi` (ya existe, bien)

Valida: `python -c "import yaml; yaml.safe_load(open('.github/workflows/release-wheels-60.yml'))"`
```

---

## PROMPT G: CI Web — Fix + Node 22 + Playwright Cache

> **Propósito:** Arreglar ci-web-11.yml específicamente.
> **Skills a cargar:** `ci-cd-and-automation`

```
Carga ci-cd-and-automation.

Arregla ci-web-11.yml:

1. Node 20 → 22
2. Verifica que `cache: "npm"` con `cache-dependency-path: web/package-lock.json` funciona (setup-node nativo)
3. Agrega caché de Playwright browsers entre runs:
   Usa `actions/cache@<SHA>` con path: `~/.cache/ms-playwright` y key basada en hash de package-lock.json
4. Agrega `shell: bash` explícito en steps que corren comandos npx
5. Verifica que `npx vitest run` y `npx tsc --noEmit` tengan timeout adecuado
6. Agrega `workflow_dispatch:` si no está (ya existe)
7. Reemplaza SHA de actions/checkout y actions/setup-node por los más recientes

Valida YAML después de editar.
```

---

## PROMPT H: CI Rust — Agregar Caché + Feature Matrix + Robustez

> **Propósito:** Arreglar ci-rust-10.yml.
> **Skills a cargar:** `ci-cd-and-automation`, `doubt-driven-development`

```
Carga ci-cd-and-automation y doubt-driven-development.

Arregla ci-rust-10.yml:

PROBLEMAS IDENTIFICADOS:
- 11 jobs que compilan el mismo workspace 11 veces cada push
- Sin feature matrix (todas las tasks usan --all-features)
- GloVe download puede fallar (URL externa)
- Swatinem/rust-cache SHA inconsistente con rust-setup/action.yml
- dtolnay/rust-toolchain sin pin SHA
- taiki-e/install-action SHA inconsistente

CAMBIOS:

1. Feature matrix: crea una matrix de features para test/coverage
   - feature-set: ["default", "cli,arrow,tls,opentelemetry", "cli,arrow,tls,opentelemetry,failpoints"]
   - coverage usa solo el feature set principal

2. Caché cross-job: usa `actions/cache` con key compartida para target/ y ~/.cargo
   - O usa `Swatinem/rust-cache` con shared-key en jobs que no usan rust-setup

3. GloVe download: 
   - Cambia a curl --retry 3 --retry-delay 5 --fail
   - Agrega validación SHA256 del archivo descargado
   - Skip si el archivo ya existe y el hash coincide

4. Consistency: 
   - rust-setup/action.yml y ci-rust-10.yml deben usar el mismo SHA para Swatinem/rust-cache
   - Ídem para taiki-e/install-action

5. Pin SHA: dtolnay/rust-toolchain y taiki-e/install-action a SHA concreto

Valida YAML. Verifica que `cargo nextest run --profile audit` existe en .config/nextest.toml.

Si .config/nextest.toml no existe, adviértelo (el workflow lo referencia pero el archivo no está).
```

---

## PROMPT I: Release Pipeline — Fix All 5 Release Workflows

> **Propósito:** Reparar todos los workflows de release en una sola sesión.
> **Skills a cargar:** `writing-plans`, `ci-cd-and-automation`, `doubt-driven-development`

```
Carga writing-plans, ci-cd-and-automation, doubt-driven-development.

Necesito arreglar los 5 workflows de release (release-*.yml) y validar que la pipeline de publicación completa funcione.

Lee TODOS los workflows release:
- release-wheels-60.yml
- release-npm-61.yml
- release-adapters-62.yml
- release-binaries-63.yml
- release-sbom-64.yml

TASK 1: Pin SHA + Node 22 en todos (aplica PROMPT C a cada uno)

TASK 2: Dependencias entre releases
- Si release-wheels (PyPI) y release-npm (npm) se publican para el mismo tag v*:
  - Deberían poder ejecutarse en paralelo sin conflictos
  - Verifica que usen distintos concurrency groups

TASK 3: release-adapters-62.yml
- Los 9 adapters compilan Rust en cada job de test sin caché compartido
- Solución: job separado "build-vantadb-python" que compile una vez,
  suba artifact, y los test-adapters lo descarguen
- Alternativa: agregar Swatinem/rust-cache en cada job (simpler)

TASK 4: release-sbom-64.yml
- `cargo install cargo-cyclonedx` demora 5-10 min compilando desde source
- Solución: usa `taiki-e/install-action@<SHA>` con tool: cargo-cyclonedx
  (es mucho más rápido porque descarga binario precompilado)

TASK 5: release-binaries-63.yml
- Cross-compile aarch64-linux: verifica que gcc-aarch64-linux-gnu se instala
- gh release upload: si falla porque release no existe, el workflow debe fallar claramente
- Verifica que `actions-rust-lang/setup-rust-toolchain` tenga SHA correcto

TASK 6: release-npm-61.yml
- publish-ts necesita WASM build de publish-wasm
- Verifica que el job publish-ts tenga `needs: [publish-wasm]` (NO lo tiene actualmente!)

Para CADA workflow, después de editar:
1. Valida sintaxis YAML: `python -c "import yaml; yaml.safe_load(open('.github/workflows/[FILE]'))"`
2. Verifica que no haya `uses:` con version tag sin SHA

Devuélveme: resumen de cambios por workflow + tabla de SHA viejo→nuevo.
```

---

## PROMPT J: Estresar y Validar Workflow Localmente Antes de Push

> **Propósito:** Validar cambios en workflows localmente antes de pushear.
> **Skills a cargar:** `ci-cd-and-automation`, `test-driven-development`

```
Carga ci-cd-and-automation y test-driven-development.

Validación pre-push de cambios en workflows.

1. Sintaxis YAML de todos los workflows:
```bash
python -c "
import yaml, glob, sys
files = glob.glob('.github/workflows/*.yml') + glob.glob('.github/actions/**/action.yml')
ok = True
for f in sorted(files):
    try:
        with open(f) as fp:
            yaml.safe_load(fp)
        print(f'  OK: {f}')
    except Exception as e:
        print(f'  FAIL: {f} — {e}')
        ok = False
sys.exit(0 if ok else 1)
"
```

2. Verifica que todos los `uses:` estén pineados a SHA:
```bash
grep -rn 'uses:' .github/ | grep -vE 'uses: [a-zA-Z0-9]+/[a-zA-Z0-9_-]+@[a-f0-9]{40}' | grep -v 'Binary' | grep -v 'node12' || echo "Todos los actions tienen SHA completo"
```

3. Verifica que los scripts referenciados existen:
```bash
for script in $(grep -rohP 'run: (.+?\.(sh|py))' .github/workflows/ | sed 's/run: //'); do
  if [ ! -f "$script" ]; then echo "MISSING: $script"; fi
done
```

4. Si puedes, ejecuta `act` (necesita Docker) para simular workflows localmente:
   - `act -W .github/workflows/ci-web-11.yml --job build --container-architecture linux/amd64`
   - `act -W .github/workflows/gate-docs-21.yml --job lint-markdown`

Si `act` no está disponible, solo haz las verificaciones 1-3.
```

---

## PROMPT K: Snapshot de Fallos + Monitoreo Continuo

> **Propósito:** Obtener el estado actual de TODOS los workflows, detectar fallos, monitorear cambios.
> **Skills a cargar:** `ci-cd-and-automation`

```
Carga la skill ci-cd-and-automation.

PARTE 1 — SNAPSHOT: estado actual de todos los workflows

```bash
mkdir -p logs/snapshots
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Tabla resumen de los últimos 3 runs por workflow
echo "📊 Snapshot: $TIMESTAMP" > logs/snapshots/snapshot-$TIMESTAMP.md
echo "" >> logs/snapshots/snapshot-$TIMESTAMP.md

for wf in .github/workflows/*.yml; do
  NAME=$(basename $wf .yml)
  gh run list --workflow $NAME --limit 3 \
    --json databaseId,conclusion,headBranch,createdAt,url,triggeredBy \
    --jq ".[] | [.databaseId, .conclusion, .headBranch, .createdAt[0:19]] | @tsv" \
    > logs/snapshots/$NAME-runs.tsv 2>/dev/null
  
  echo "## $NAME" >> logs/snapshots/snapshot-$TIMESTAMP.md
  cat logs/snapshots/$NAME-runs.tsv >> logs/snapshots/snapshot-$TIMESTAMP.md 2>/dev/null || echo "  (sin runs)" >> logs/snapshots/snapshot-$TIMESTAMP.md
  echo "" >> logs/snapshots/snapshot-$TIMESTAMP.md
done

echo "Snapshot guardado en logs/snapshots/snapshot-$TIMESTAMP.md"
```

PARTE 2 — DETECTAR QUÉ WORKFLOWS FALLAN EN main

```bash
# Resumen de workflows fallidos en main en los últimos 30 días
echo "## Fallos en main (últimos 30 días)" >> logs/snapshots/snapshot-$TIMESTAMP.md
for wf in .github/workflows/*.yml; do
  NAME=$(basename $wf .yml)
  FAILS=$(gh run list --workflow $NAME --branch main --limit 20 \
    --json conclusion --jq '[.[] | select(.conclusion=="failure")] | length')
  if [ "$FAILS" -gt "0" ]; then
    echo "- $NAME: $FAILS fallos en últimos 20 runs" >> logs/snapshots/snapshot-$TIMESTAMP.md
    echo "  Obteniendo logs de último fallo..."
    LAST_FAIL=$(gh run list --workflow $NAME --branch main --limit 20 \
      --json databaseId,conclusion --jq '.[] | select(.conclusion=="failure") | .databaseId' | head -1)
    if [ -n "$LAST_FAIL" ]; then
      gh run view $LAST_FAIL --log-failed > logs/snapshots/$NAME-last-fail.log 2>/dev/null
      echo "  Log fail: logs/snapshots/$NAME-last-fail.log" >> logs/snapshots/snapshot-$TIMESTAMP.md
      # Extraer errores clave del log
      rg -i "error\[|error:|FAILED|test.*FAILED|failed to run|process didn.t exit" \
        logs/snapshots/$NAME-last-fail.log | head -5 >> logs/snapshots/snapshot-$TIMESTAMP.md
    fi
  fi
done
```

PARTE 3 — MONITOREO DE UN RUN ESPECÍFICO (cuando un workflow está corriendo)

```bash
# Monitorear un run en tiempo real
RUN_ID=<ID del run>
echo "Monitoreando run $RUN_ID"
sleep 30  # dar tiempo para que arranque

gh run watch $RUN_ID  # watch se queda viendo hasta completar
echo "Conclusión: $(gh run view $RUN_ID --json conclusion --jq '.conclusion')"

# Si falló, logs
if [ "$(gh run view $RUN_ID --json conclusion --jq '.conclusion')" = "failure" ]; then
  echo "❌ Falló — extrayendo logs..."
  gh run view $RUN_ID --log-failed > logs/run-$RUN_ID-failed.log
  gh run view $RUN_ID --log > logs/run-$RUN_ID-full.log
  echo "Ver: logs/run-$RUN_ID-failed.log"
  rg -i "error\[|error:|FAILED|test.*FAILED" logs/run-$RUN_ID-failed.log | head -20
fi
```

PARTE 4 — RETRY DE UN WORKFLOW FALLIDO (después de aplicar fix)

```bash
# Re-ejecutar el último run fallido de un workflow desde main
RUN_ID=$(gh run list --workflow [NOMBRE.yml] --branch main --limit 1 \
  --json databaseId,conclusion --jq '.[] | select(.conclusion=="failure") | .databaseId')
if [ -n "$RUN_ID" ]; then
  echo "Re-ejecutando run $RUN_ID con los cambios actuales..."
  gh run rerun $RUN_ID
  echo "Esperando 30s para que arranque..."
  sleep 30
  gh run watch $RUN_ID
fi
```

Devuélveme: archivo snapshot-$TIMESTAMP.md con el estado completo + lista de workflows fallidos.
```

---

## Orden de Ejecución Recomendado

```
FASE 0 — Capturar estado actual de fallos
  PROMPT K → snapshot de todos los runs fallidos + logs por workflow

FASE 1 — Auditoría
  Usa PROMPT A → docs/workflow-audit-report.md

FASE 2 — Correcciones Transversales
  PROMPT D → SHA consistency + Node 22 en todos los workflows
  PROMPT E → Scripts robustos

FASE 3 — Correcciones por Workflow (prioridad)
  1. PROMPT G → ci-web-11.yml (más simple, quick win)
  2. PROMPT H → ci-rust-10.yml (el más usado — múltiples issues)
  3. PROMPT C → gate-docs-21.yml (rápido)
  4. PROMPT C → fuzz-40.yml (quick win)
  5. PROMPT C → sec-codeql-30.yml (rápido — cambiar build-mode)
  6. PROMPT I → Todos los release-*.yml (complejo, requiere cuidado)
  7. PROMPT C → perf-bench-40.yml
  8. PROMPT C → heavy-bench-nightly-51.yml
  9. PROMPT C → heavy-certification-50.yml

FASE 4 — Validación pre-push
  PROMPT J → validación local (YAML syntax, SHA pins, scripts)

FASE 5 — Push y ciclo de monitoreo
  1. git push
  2. PROMPT K después de cada cambio → verificar si el workflow que arreglamos ahora pasa
  3. Si falla de nuevo, PROMPT B para el diagnóstico fino del error nuevo
  4. Repetir hasta que pase
  5. Una vez que pase, PROMPT K para verificar que no se hayan roto otros workflows

FASE 6 — Mantenimiento continuo
  1. PROMPT K semanal para detectar nuevas fallas
  2. Configurar notificaciones en GitHub (Settings → Notifications → Actions)
  3. Agregar workflow health check en el dashboard
```

---

## Checklist por Workflow

```
| Workflow            | SHA pinned | Node 22 | Caché  | Timeout OK | Scripts OK | workflow_dispatch |
|---------------------|------------|---------|--------|------------|------------|-------------------|
| ci-rust-10.yml      | [ ]        | N/A     | [ ]    | [ ]        | [?]        | [✅]              |
| ci-web-11.yml       | [ ]        | [ ]     | [ ]    | [ ]        | N/A        | [✅]              |
| fuzz-40.yml         | [ ]        | N/A     | [ ]    | [ ]        | N/A        | [✅]              |
| gate-docs-21.yml    | [ ]        | [ ]     | [✅]?  | N/A        | N/A        | [✅]              |
| heavy-bench-51.yml  | [ ]        | N/A     | [ ]    | [ ]        | [?]        | [✅]              |
| heavy-cert-50.yml   | [ ]        | N/A     | [ ]    | [ ]        | [?]        | [✅]              |
| perf-bench-40.yml   | [ ]        | N/A     | [ ]    | [ ]        | [?]        | [✅]              |
| release-adapters-62 | [ ]        | N/A     | [ ]    | [ ]        | [?]        | [✅]              |
| release-binaries-63 | [ ]        | N/A     | [ ]    | [ ]        | N/A        | [✅]              |
| release-npm-61.yml  | [ ]        | [ ]     | [ ]    | [ ]        | N/A        | [✅]              |
| release-sbom-64.yml | [ ]        | N/A     | [ ]    | N/A        | N/A        | [✅]              |
| release-wheels-60   | [ ]        | N/A     | [ ]    | [ ]        | [?]        | [✅]              |
| sec-codeql-30.yml   | [ ]        | N/A     | [ ]    | [ ]        | N/A        | [✅]              |
```

> **Nota:** `[?]` significa que el workflow referencia scripts externos que deben verificarse.
