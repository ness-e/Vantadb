# CI/CD Integration Plan

> Integración de workflows de `web/` dentro del monorepo VantaDB.

---

## Situación Actual

### VantaDB/.github/workflows/ (6 workflows activos)

| Workflow | Trigger | Scope |
|----------|---------|-------|
| `rust_ci.yml` | push/PR main | Rust build, clippy, tests, audit |
| `release.yml` | tag v*.*.* | Build + release binaries |
| `python_wheels.yml` | tag, PR, dispatch | Build + publish Python wheels |
| `nightly_bench.yml` | cron 3am | Criterion benchmarks |
| `heavy_certification.yml` | cron weekly | Stress/certification tests |
| `bench.yml` | push/PR main | Python SDK benchmarks |

### web/.github/workflows/ (1 workflow, obsoleto)

| Workflow | Trigger | Scope |
|----------|---------|-------|
| `deploy.yml` | push/PR feat/landing-v2 | Build + deploy a GitHub Pages |

**Problema**: Este workflow deploya a GitHub Pages, pero el target real es Vercel.
La rama `feat/landing-v2` no es la rama principal.

## Plan de Acción

### Paso 1: Crear web-deploy.yml en .github/workflows/

Mover el workflow de la web a `VantaDB/.github/workflows/web-deploy.yml` con:
- Trigger: `push` a main + `pull_request` a main
- `paths: ["web/**"]` — solo corre cuando cambia web/
- Usar Vercel CLI o dejar que Vercel auto-deploy (recomendado)

**Recomendación**: No necesitas workflow de deploy para Vercel si está conectado via Git.
Vercel auto-deploya en cada push a main. Solo necesitas un workflow si quieres
Prettier + ESLint + TypeScript check en CI.

### Paso 2: Filtrar workflows existentes

Agregar `paths-ignore: ["web/**"]` a los 6 workflows de Rust/Python para que
no corran cuando solo cambia la web.

### Paso 3: Eliminar workflow obsoleto

- Eliminar `web/.github/workflows/deploy.yml`
- Eliminar `web/.github/` si queda vacío (excepto skills/)

### Paso 4: Vercel rootDirectory

En el dashboard de Vercel:
1. Ir a tu proyecto en vercel.com
2. Settings → General → Root Directory
3. Cambiar de `/` a `web/`
4. Framework Preset: `Other` (ya está `framework: null` en vercel.json)

El `web/vercel.json` existente ya configura todo lo demás (rewrites, cleanUrls).

## Workflow Propuesto: web-ci.yml

```yaml
name: Web CI
on:
  push:
    branches: [main]
    paths: ["web/**"]
  pull_request:
    branches: [main]
    paths: ["web/**"]

jobs:
  build:
    defaults:
      run:
        working-directory: web
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: "npm"
          cache-dependency-path: web/package-lock.json
      - run: npm ci
      - run: npm run lint
      - run: npm run build
```

Nota: No incluye deploy porque Vercel auto-deploya desde Git.
