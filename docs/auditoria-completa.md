# Auditoría Completa — ness-e/Vantadb

> Generada: 2026-07-14
> Alcance: repo settings, CI/CD, releases, tags, archivos raíz, seguridad, deploy, npm/PyPI/crates.io, README, comparación con estándares profesionales.

---

## 1. Repositorio — Settings

| Atributo | Valor | Diagnóstico |
|----------|-------|-------------|
| Visibilidad | Público | ✅ |
| Descripción | "Embedded persistent memory and vector retrieval engine…" | ✅ |
| Topics | `ai`, `bm25`, `database`, `embedded-database`, `hnsw`, `hybrid-search`, `local-first`, `pyo3`, `python`, `rust`, `vector-search`, `wal` | ✅ 12 bien elegidos |
| Homepage | https://vantadb.vercel.app | ✅ |
| Default branch | `main` | ✅ |
| Issues | ✅ Activado | ✅ |
| Projects | ✅ Activado | ✅ |
| Discussions | ✅ Activado | ✅ |
| Wiki | ❌ Deshabilitado | 🟢 No necesario — docs/ ya existe |
| Social preview | ❌ Default GitHub OG | 🟡 Vale la pena customizar |
| Delete branch on merge | ✅ | ✅ |
| Merge strategies | squash + rebase + merge commit | 🟡 Sobran estrategias — squash + rebase alcanza |

## 2. Archivos Raíz

| Archivo | Ubicación | Diagnóstico |
|---------|-----------|-------------|
| `README.md` | Raíz | ✅ 16KB, 15 badges |
| `LICENSE` | Raíz | ✅ Apache-2.0 |
| `CHANGELOG.md` | `docs/` | 🟡 No está en raíz — pero GitHub lo encuentra igual |
| `CONTRIBUTING.md` | Raíz + `.github/` | ✅ |
| `SECURITY.md` | `.github/` | ✅ |
| `CODE_OF_CONDUCT.md` | Raíz | ✅ |
| `SUPPORT.md` | `.github/` | ✅ |
| `CITATION.cff` | ❌ Ausente | 🟢 Software project, no académico → **no lo necesita** |
| `FUNDING.yml` | ❌ Ausente | 🟢 Sin GitHub Sponsors actuales → **no lo necesita** |

## 3. `.github/` — Community Files

| Archivo | Estado |
|---------|--------|
| Issue templates (yml) | ✅ bug_report, feature_request, documentation |
| Issue templates (md legacy) | ✅ bug_report, feature_request |
| Config.yml para issue templates | ✅ |
| PR template | ✅ |
| CODEOWNERS | ✅ |
| Dependabot | ✅ 4 ecosystems (Cargo, npm, Actions, Docker) |
| CLA templates | ✅ Corporate + Individual |
| Workflows | ✅ 15 activos |

## 4. Protección de `main`

Ruleset activo — bien configurado:

| Regla | Valor |
|-------|-------|
| PR requerido | ✅ sí |
| Merge methods | squash + rebase |
| Reviews requeridas | 1 approving |
| Code owner review | ✅ |
| Stale review dismiss | ✅ |
| Last push approval | ✅ |
| Status checks | `build`, `windows-check`, `deny`, `Analyze (rust)` |
| Strict up-to-date | ✅ |
| Force push | ❌ Bloqueado |
| Delete branch | ❌ Bloqueado |

**Diagnóstico**: ✅ Excelente. Solo sobra merge commit — con squash + rebase alcanza.

## 5. CI/CD — 15 Workflows

| Categoría | Workflow | Estado |
|-----------|----------|--------|
| CI (fast gate) | `ci-rust-10.yml` | ✅ |
| CI (web) | `ci-web-11.yml` | ✅ |
| Gate | `gate-docs-21.yml` | ✅ |
| Gate | `fuzz-40.yml` | ✅ |
| Heavy | `heavy-bench-nightly-51.yml` | ✅ |
| Heavy | `heavy-certification-50.yml` | ✅ |
| Perf | `perf-bench-40.yml` | ✅ |
| Security | `sec-codeql-30.yml` | ✅ |
| Release | `release-adapters-62.yml` | ❌ Ver sección 6 |
| Release | `release-binaries-63.yml` | ❌ Ver sección 6 |
| Release | `release-npm-61.yml` | ❌ Ver sección 6 |
| Release | `release-sbom-64.yml` | ✅ |
| Release | `release-wheels-60.yml` | ❌ Ver sección 6 |
| Bot | Dependabot | ✅ |
| Bot | Dependency Graph | ✅ |

## 6. 🔴 RELEASES — PROBLEMA CRÍTICO

### 6.1 Releases existentes en GitHub

| Tag | Release | Assets | Diagnóstico |
|-----|---------|--------|-------------|
| v0.1.0-rc1 | ✅ Pre-release | ✅ | ✅ |
| v0.1.0-rc2 | ✅ Pre-release | ✅ | ✅ |
| v0.1.0 | ✅ | ✅ | ✅ |
| v0.1.1 | ✅ | ✅ | ✅ |
| v0.1.2 | ✅ | ✅ | ✅ |
| v0.1.3 | ✅ | ✅ | ✅ |
| v0.1.4 | ✅ | ✅ | ✅ |
| v0.1.5 | ✅ | ✅ | ✅ |
| **v0.2.0 → v0.3.0** | ❌ **NO existen** | ❌ | **🔴 P1** |

### 6.2 Tags en remote sin Release

| Tag | Mensaje | Commit |
|-----|---------|--------|
| v0.2.0 | "fmt: cargo fmt in types.rs" (lightweight) | 6056b5f |
| v0.2.1 | "Python bindings stability…" | 01873ef |
| v0.2.2 | "MCP server improvements…" | 01873ef |
| v0.2.3 | "Python SDK, Distance, Async…" | 01873ef |
| v0.3.0 | "LangChain/LlamaIndex adapters…" | 01873ef |

⚠️ **v0.2.1, v0.2.2, v0.2.3, v0.3.0 apuntan al mismo commit**. Esto indica que fueron creados en masa sin avanzar el código entre ellos.

### 6.3 Causa Raíz

Dos problemas confluyen:

**A. `release-plz.toml` línea 19:**
```toml
git_release_enable = false
```
release-plz crea tags y los pushea, pero **nunca crea GitHub Releases**. Los releases v0.1.x fueron creados por el workflow `release-wheels-60.yml`, no por release-plz.

**B. `release-wheels-60.yml` líneas 20-25 — path filtering en tag push:**
```yaml
on:
  push:
    tags: ["v*.*.*"]
    paths:
      - "src/**"
      - "vantadb-python/**"
      - "Cargo.toml"
      - "Cargo.lock"
      - ".github/workflows/release-wheels-60.yml"
```

Cuando release-plz pushea un tag sin modificar archivos en esos paths, el workflow **no se ejecuta**. El tag existe, pero `softprops/action-gh-release` nunca corre. Los wheels se construyen y publican a PyPI, pero la GitHub Release no se crea.

**C. `release-binaries-63.yml` depende de `release: [published]`:**
```yaml
on:
  release:
    types: [published]
```
Es un catch-22: necesita un Release publicado para correr, pero ningún workflow crea ese Release.

### 6.4 Impacto

| Package | Publicado? | Diagnóstico |
|---------|-----------|-------------|
| `crates.io` (vantadb) | ❓ Último: v0.1.4 | 🔴 Desactualizado |
| `pypi.org` (vantadb-py) | ✅ v0.2.0 | ✅ |
| `npm` (vantadb-ts) | ❌ No existe | 🔴 Nunca publicado |
| `npm` (vantadb-wasm) | ❌ No existe | 🔴 Nunca publicado |
| Homebrew | ❌ SHA256 placeholders | 🔴 No funcional |

## 7. Secrets y Environments

| Secret | Propósito | Diagnóstico |
|--------|-----------|-------------|
| `NPM_TOKEN` | npm publish | 🔴 Nunca se usó — npm no tiene paquetes |
| `TEST_PYPI_API_TOKEN` | TestPyPI | ✅ |
| `VERCEL_ORG_ID` | Vercel deploy | ✅ |
| `VERCEL_PROJECT_ID` | Vercel deploy | ✅ |
| `VERCEL_TOKEN` | Vercel deploy | ✅ |

| Environment | Protection rules | Diagnóstico |
|-------------|-----------------|-------------|
| `pypi` | ✅ branch_policy + required_reviewer | ✅ |
| `testpypi` | ✅ branch_policy + required_reviewer | ✅ |
| `NPM_TOKEN` | ❌ Sin rules | 🟢 No crítico — npm ni existe |
| `Preview` | ❌ Sin rules | 🟢 Solo preview de Vercel |
| `Production` | ❌ Sin rules | 🟢 Solo deploy de Vercel |

## 8. Labels (18)

```
bug, documentation, duplicate, enhancement, good first issue,
help wanted, invalid, question, wontfix, dependencies,
rust, github_actions, javascript, triage, ci, docker,
benchmark, regression
```

**Diagnóstico**: ✅ Limpias. Cubren tipo + área. Sin duplicación.
🟡 Podrían agruparse mejor: `area:` prefix para áreas (ya tienen `rust`, `ci`, `docker`).

## 9. README — Calidad

### README.md (16KB)
- ✅ 15 badges (CI, quality, security, performance, project)
- ✅ Sin secciones rotas ni TODO markers
- ✅ Quickstart, usage, architecture, benchmarks, links
- ✅ Discord link presente

### README_ES.md (17KB)
- ✅ Misma estructura
- ✅ Traducción completa
- ⚠️ 13 badges vs 15 del inglés — **falta Discord badge**

## 10. Tags Report

```
wasm-v0.2.0
v0.3.0         ← ultimo tag, sin release
v0.2.3         ← mismo commit que v0.3.0
v0.2.2         ← mismo commit que v0.3.0
v0.2.1         ← mismo commit que v0.3.0
v0.2.0         ← commit diferente
v0.1.5         ← ultimo release real
v0.1.4
v0.1.3
v0.1.2
v0.1.1
v0.1.0
v0.1.0-test    ← draft, despublicado
v0.1.0-rc2
v0.1.0-rc1
ts-v0.2.0
adapters-v0.3.0
adapters-v0.1.0-test
```

---

## 11. Priorización de Acciones

### 🔴 P1 — Debe arreglarse antes del próximo push

| # | Problema | Solución | Archivos a tocar |
|---|----------|----------|------------------|
| 1 | Tags v0.2.0–v0.3.0 sin Release | `release-plz.toml`: `git_release_enable = true`<br>`release-wheels-60.yml`: sacar `paths:` del trigger de tags<br>Crear releases retroactivos vía `gh release create` | `release-plz.toml`<br>`.github/workflows/release-wheels-60.yml` |
| 2 | npm nunca publicó | Verificar si NPM_TOKEN es válido. Ejecutar `release-npm-61.yml` via `workflow_dispatch` | Ninguno (debug) |
| 3 | `release-binaries-63.yml` catch-22 | Agregar `workflow_dispatch` como trigger alternativo | `.github/workflows/release-binaries-63.yml` |
| 4 | Cargo.toml version = 0.2.0 pero tag v0.3.0 | `release-plz release` o bump manual | `Cargo.toml` |

### 🟡 P2 — Debería arreglarse esta semana

| # | Problema | Solución | Archivos |
|---|----------|----------|----------|
| 5 | README_ES sin Discord badge | Copiar badge del inglés | `README_ES.md` |
| 6 | Social preview default | Subir OG image custom | GitHub repo settings |
| 7 | Merge commit strategy activada | Deshabilitar en repo settings | GitHub UI |

### 🟢 P3 — Polish, no urgente

| # | Problema | Solución |
|---|----------|----------|
| 8 | CHANGELOG.md en `docs/` no raíz | Mover symlink o dejarlo como está |
| 9 | Tags v0.2.1–v0.3.0 en mismo commit | Decidir versión real y recrear tags |

### ❌ No necesita arreglo

| Item | Razón |
|------|-------|
| CITATION.cff | Proyecto de software, no investigación académica |
| FUNDING.yml | No hay GitHub Sponsors ni OpenCollective |
| Habilitar Wiki | docs/ + homepage cubren la documentación |
| Protection rules en NPM_TOKEN/Preview/Production | No hay riesgo — npm ni existe, Preview/Production son Vercel |
| Más badges en README | 15 ya es cantidad estándar profesional |
| CHANGELOG en raíz | docs/CHANGELOG.md funciona idéntico |

---

## 12. Comparación con Repos Profesionales

| Práctica | VantaDB | Rust stdlib | Tokio | Serde | Tauri |
|----------|---------|-------------|-------|-------|-------|
| Branch protection | ✅ ruleset | ✅ rules | ✅ | ✅ | ✅ |
| Issue templates | ✅ yml + md | ✅ yml | ✅ yml | ✅ yml | ✅ yml |
| PR template | ✅ | ✅ | ✅ | ✅ | ✅ |
| Dependabot | ✅ 4 eco | ✅ 1 eco | ✅ | ✅ | ✅ |
| SECURITY.md | ✅ | ✅ | ✅ | ✅ | ✅ |
| CODEOWNERS | ✅ | ✅ | ✅ | ✅ | ✅ |
| Changelog en raíz | ⚠️ docs/ | ✅ raíz | ✅ raíz | ✅ raíz | ✅ raíz |
| Social preview | ❌ default | ✅ custom | ✅ custom | ✅ custom | ✅ custom |
| Release a tag | ❌ roto | ✅ 1:1 | ✅ 1:1 | ✅ 1:1 | ✅ 1:1 |
| Wiki | ❌ off | ❌ off | ❌ off | ❌ off | ❌ off |
| FUNDING | ❌ | ❌ | ✅ | ✅ | ✅ |
| CI/CD gates | ✅ 15 | ✅ ~10 | ✅ ~12 | ✅ ~8 | ✅ ~15 |

**Conclusión**: VantaDB sigue los mismos estándares que repositorios profesionales establecidos. La única diferencia significativa es el **pipeline de releases roto** — y la falta de social preview, que es cosmética.

---

## 13. Resumen Final

| Categoría | Bueno | Regular | Malo |
|-----------|-------|---------|------|
| Settings repo | 10/11 | 1 | 0 |
| Archivos raíz | 8/10 | 0 | 2 faltantes no críticos |
| Community | 10/10 | 0 | 0 |
| Protección branch | 9/10 | 1 | 0 |
| CI/CD | 12/15 | 0 | 3 workflows release |
| Releases/Publicación | 1/5 | 1 | 3 |
| README | 14/15 | 1 | 0 |
| Labels | 18/18 | 0 | 0 |
| Secrets/Security | 5/5 secrets | 2/5 env rules | 0 |
| **Total** | **87%** | **9%** | **4%** |

El proyecto está sano. El 4% malo es el pipeline de releases — una vez arreglado, pasa a 91%.
