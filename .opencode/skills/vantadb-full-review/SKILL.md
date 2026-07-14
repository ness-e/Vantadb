---
name: vantadb-full-review
description: >
  Comprehensive multi-layer review of the entire VantaDB project.
  Covers Rust core, Python SDK, web frontend, TypeScript SDK, CI/CD, docs,
  security, performance, dependencies, architecture, design, and accessibility.
  Runs ALL available diagnostic tools and produces a structured report.
compatibility: opencode
---

# VantaDB Full Project Review

> **Orquestador de revisión integral.** Ejecuta análisis en paralelo usando
> todos los tools disponibles (cargo, codegraph, tsconfig, pytest, playwright)
> y carga skills especializadas según la capa que se revisa. Produce un reporte
> estructurado con scores, issues, y recomendaciones priorizadas.

---

## Marco Teórico — Sistemas de Evaluación de Proyectos

Esta skill unifica **5 sistemas de evaluación** de la industria para producir un review completo y calibrado contra estándares reales:

### 1. ISO/IEC 25010 (SQuaRE) — Modelo de calidad de producto

> Reemplaza a ISO 9126. Define 8 características y 31 subcaracterísticas de calidad de software.

| Característica | Subcaracterísticas | Mapa a capa del review |
|---------------|-------------------|----------------------|
| **Functional suitability** | completeness, correctness, appropriateness | F1 (Rust Core) — API contract testing |
| **Reliability** | maturity, availability, fault tolerance, recoverability | F1 (WAL, HNSW), F5 (Infra HA) |
| **Performance efficiency** | time behavior, resource utilization, capacity | F1 (benchmarks), F3 (bundle size) |
| **Compatibility** | coexistence, interoperability | F2 (Python SDK), F4 (TS SDK), F8 (adapters) |
| **Usability** | appropriateness recognizability, learnability, operability, user error protection, accessibility, UI aesthetics | F3 (UX), F7 (Design) |
| **Security** | confidentiality, integrity, non-repudiation, accountability, authenticity | F1 (unsafe), F3 (CSP/CORS), F5 (secrets) |
| **Maintainability** | modularity, reusability, analyzability, modifiability, testability | F8 (architecture, code smells) |
| **Portability** | adaptability, installability, replaceability | F5 (Docker, Vercel, WASM) |

ISO 25010 provee **qué** medir. Las siguientes proveen **cómo medirlo**.

### 2. SonarQube Quality Gate — Thresholds de calidad de código

> Basado en el quality gate "Sonar Way". Define condiciones binarias (pass/fail) y ratings A-E.

| Métrica | Threshold | Rating A | Rating B/C/D/E |
|---------|-----------|---------|----------------|
| **Reliability** (bugs) | 0 blocker/critical/major | A = 0 bugs | B=1+minor, C=1+major, D=1+critical, E=1+blocker |
| **Security** (vulnerabilities) | 0 blocker/critical/major | A = 0 vulns | B=1+minor, C=1+major, D=1+critical, E=1+blocker |
| **Maintainability** (tech debt %) | <5% debt ratio | A = <5% | B=6-10%, C=11-20%, D=21-50%, E=>50% |
| **Coverage** | ≥80% en código nuevo | ✅ | ❌ si <80% |
| **Duplication** | ≤3% en código nuevo | ✅ | ❌ si >3% |
| **Security Hotspots** | 100% reviewed | ✅ | ❌ si <100% |
| **Fudge factor** | Ignora duplicación si <20 líneas nuevas; ignora coverage si <20 líneas a cubrir | — | — |

El review adapta este sistema: cada capa tiene **condiciones de quality gate** que deben cumplirse.

### 3. OpenSSF CII Best Practices — Madurez de proyecto open source

> 3 niveles: Passing → Silver → Gold. Cada nivel agrega requisitos.

| Nivel | Requisitos clave que no están en niveles inferiores |
|-------|---------------------------------------------------|
| **Passing** | Licencia FLOSS, HTTPS, docs básicas, test suite, build system, changelog, control de cambios, proceso de vulnerabilidades (response < 14 días), static analysis, sin credenciales filtradas, fix vulns < 60 días |
| **Silver** (+ sobre Passing) | Coverage ≥80%, dynamic analysis + fuzzing, coding standards, dependencias monitoreadas, confirmación formal de reportes de vulnerabilidad |
| **Gold** (+ sobre Silver) | Coverage ≥90%, binary reproducibility, security audit formal cada 18 meses, todos los cambios revisados por segunda persona, coding standards documentados y seguidos |

### 4. OWASP ASVS v5.0 — Security verification levels

> Define 3 niveles de verificación de seguridad en 14 categorías (capítulos).

| Nivel | Tipo | Aplica a | Capítulo ASVS |
|-------|------|---------|--------------|
| **L1 (Opportunistic)** | Automatizado + verificación manual básica | Apps de baja sensibilidad | V1-V14, ~200 requisitos |
| **L2 (Standard)** | L1 + verificación manual completa | Apps con datos sensibles | L1 + controles adicionales por capítulo |
| **L3 (Advanced)** | L1+L2 + verificación arquitectónica profunda | Infraestructura crítica | L2 + revisión de diseño completo |

El review adapta ASVS: la capa de seguridad (F1, F3) se evalúa contra L1 como mínimo, L2 como target.

### 5. CodeClimate / Qlty — Maintainability scoring

> Basado en time-to-fix estimates de code smells.

| Rating | Issues de maintainability | Significado |
|--------|--------------------------|-------------|
| **A (verde)** | 0-4 issues | Código limpio y mantenible |
| **B (verde claro)** | 5-8 issues | Minor improvements |
| **C (amarillo)** | 9-12 issues | Atención necesaria |
| **D (naranja)** | 13-16 issues | Deuda técnica significativa |
| **F (rojo)** | 17+ issues (o 1 issue de ≥60min) | Deuda crítica |

Qlty agrega: linting, defects, formatting, duplication, security, complexity en un solo pipeline Rust.

---

### Unificación — El sistema de puntuación de esta skill

Cada capa se evalúa con **4 dimensiones** que mapean a los sistemas anteriores:

| Dimensión | Sistema fuente | Escala |
|-----------|---------------|--------|
| **Quality Gate** (✅/❌) | SonarQube | Pasa todas las condiciones → ✅; alguna falla → ❌ |
| **Rating** (A-E) | SonarQube + CodeClimate | A=no issues, B=minor, C=major, D=critical, E=blocker |
| **Score** (0-10) | Síntesis propia | Ver tabla por capa |
| **CII Level** | OpenSSF CII | Passing / Silver / Gold |

**Peso de cada sistema en el score total:**

| Sistema | Peso en score | Cómo se aplica |
|---------|--------------|----------------|
| ISO 25010 | 20% | ¿Cubre las 8 características? |
| SonarQube Quality Gate | 25% | ¿Pasa thresholds por capa? |
| CII Best Practices | 20% | ¿Cumple nivel? |
| OWASP ASVS (seguridad) | 15% | ¿Alcanza L1? ¿L2? |
| CodeClimate/Qlty | 20% | Maintainability rating |

## Arquitectura

El review se divide en **8 capas**. Cada capa tiene:
1. **Skills a cargar** (de las existentes en el proyecto)
2. **Comandos mecánicos** (herramientas CLI/MCP)
3. **Checklist de verificación** (ítems a revisar manualmente)
4. **Puntaje** (0-10 por capa, con criterios explícitos)

```
┌─────────────────────────────────────────────────────┐
│               VANTAFULL REVIEW                       │
├─────────────────────────────────────────────────────┤
│  FASE 0 — SETUP  (tools + contexto)                 │
│  FASE 1 — RUST CORE LAYER                            │
│  FASE 2 — PYTHON SDK LAYER                           │
│  FASE 3 — WEB FRONTEND LAYER                         │
│  FASE 4 — TS SDK LAYER                               │
│  FASE 5 — CI/CD + INFRA LAYER                        │
│  FASE 6 — DOCS + SEO LAYER                           │
│  FASE 7 — DESIGN + UX LAYER                          │
│  FASE 8 — ARCHITECTURE + DEPENDENCIES LAYER          │
│  FASE 9 — HALLAZGOS ENCONTRADOS (all findings)       │
│  FASE 10 — REPORTE (score agregado + prioridades)    │
└─────────────────────────────────────────────────────┘
```

## FASE 0 — Setup

Cargá las skills base que aplican transversalmente:
- `ponytail-audit` — detectar over-engineering en todo el repo
- `code-review-and-quality` — framework de revisión multi-eje
- `doubt-driven-development` — adversarial review para hallazgos críticos
- `ponytail-review` — revisión de over-engineering en diff
- `code-simplification` — detectar código innecesariamente complejo
- `codegraph_explore` — mapear estructura y dependencias entre módulos

### Análisis del Repo Git

Ejecutá estos comandos de diagnóstico del repositorio:

```bash
# Estado del repo
git status --short
git log --oneline -30
git diff --stat

# Análisis de ramas
git branch -a
git log --oneline --graph --all --decorate -20

# Commits problemáticos (merge conflicts, revert commits)
git log --oneline --grep="revert\|fixup\|WIP\|wip\|fixme\|hack\|workaround"

# Commits sin conventional commit
git log --oneline --format="%s" | Select-String -NotMatch "^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?:"

# Archivos con más cambios (churn)
git log --name-only --pretty=format: | Sort-Object | Group-Object | Sort-Object Count -Descending | Select-Object -First 20

# Dead branches (sin commit en >30 días)
git for-each-ref --sort=-committerdate refs/heads/ --format="%(refname:short) %(committerdate:relative)"

# Salud del workspace
cargo check --workspace 2>&1 | tail -20
cargo fmt --check 2>&1 | tail -20

# Index de símbolos
codegraph_explore "WorkspaceOverview top-level modules"
```

## FASE 1 — Rust Core Layer

**Skills:** `code-review-and-quality`, `code-simplification`, `security-and-hardening`, `performance-optimization`, `api-and-interface-design`

### Comandos mecánicos

| Comando | Propósito | Pasa/Falla |
|---------|-----------|------------|
| `cargo check --workspace` | Compilación completa | |
| `cargo fmt --check` | Formato | |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Lints | |
| `cargo nextest run --profile audit --workspace --build-jobs 2` | Tests | |
| `cargo deny check` | Licencias + advisories | |
| `cargo audit` | Security advisories | |
| `cargo machete` | Dependencias no usadas | |
| `cargo outdated --exit-code 1` | Dependencias desactualizadas | |
| `cargo bloat --crates 2>&1 \| Select-Object -First 20` | Tamaño del binario | |

### Checklist de revisión manual

**Correctitud:**
- [ ] `unsafe` blocks revisados uno por uno (SAFETY docs presentes, invariantes validados)
- [ ] Error handling: todos los `unwrap()`/`expect()` justificados o reemplazados con `?`
- [ ] Edge cases en colecciones vacías, `None`, concurrent access
- [ ] Serialization/deserialization: forward compatibility checks

**Performance:**
- [ ] Hot paths identificados sin lock contention (`parking_lot`, `dashmap` uso correcto)
- [ ] Allocaciones en hot paths minimizadas
- [ ] WAL: sharded o single mutex? (ver P2 en bitacora)
- [ ] HNSW: insert_lock bottleneck? (ver P1 en bitacora)
- [ ] Benchmarks pasan sin regresiones: `cargo bench 2>&1 | tail -30`

**Security:**
- [ ] Input validation en API pública (VantaEmbedded methods)
- [ ] Path traversal en file operations
- [ ] Config con secrets manejada correctamente (env vars, no hardcode)
- [ ] `unsafe` blocks auditados con `cargo-geiger` si disponible

**Arquitectura:**
- [ ] Circular dependencies? (codegraph_explore para mapear imports)
- [ ] Módulos >1000 líneas candidatos a split
- [ ] Feature flags correctamente gateados (no leakage entre features)
- [ ] API pública documentada con docstrings

### SonarQube Quality Gate (Rust Core)

| Condición | Threshold | Pasa/Falla |
|-----------|-----------|------------|
| Reliability Rating on new code | = A (0 bugs) | |
| Security Rating on new code | = A (0 vulns) | |
| Maintainability Rating on new code | = A (<5% debt ratio) | |
| Coverage on new code | ≥ 80% | |
| Duplication on new code | ≤ 3% | |
| All Security Hotspots reviewed | 100% | |
| No leaked credentials | 0 secrets | |

### CII Best Practices (Rust Core)

| Criterio | Nivel | Pasa/Falla |
|----------|-------|------------|
| Build system working | Passing | |
| Test suite | Passing | |
| Static analysis applied | Passing | |
| Vulnerability response < 14 days | Passing | |
| Warnings fixed | Passing | |
| Secure design knowledge | Passing | |

### CodeClimate Maintainability (Rust Core)

| Métrica | Rating | Umbral |
|---------|--------|--------|
| Cargo clippy --D warnings | Debe ser 0 | A = 0, B = 1-8, C = 9-12, D = 13-16, F = 17+ |
| Large files (>1000L) | <3 archivos | A <3, B <5, C <8, D <12, F ≥12 |
| unsafe blocks | <10 con SAFETY docs | A <5, B <10, C <20, D <50, F ≥50 |
| unwrap/expect sin justificar | 0 | A = 0 |

### Score: __/10

**Fórmula de score:**
- Base = cumplir Quality Gate (✅ todas las condiciones = 5pts, cada ❌ resta 1)
- +1 si CII = Passing, +2 si CII = Silver, +3 si CII = Gold
- +1 si CodeClimate rating = A
- +1 si ISO 25010 maintainability bien modularizada
- +0.5 si benchmarks sin regresiones

| Score | Criterio |
|-------|----------|
| 10 | Quality Gate ✅ (6/6), CII Gold, CodeClimate A, ISO modular |
| 8-9 | Quality Gate ✅ (5-6/6), CII Silver+, CodeClimate A-B |
| 6-7 | Quality Gate ✅ (4-5/6), CII Passing, CodeClimate B-C |
| 4-5 | Quality Gate ❌ (2-3/6), CII con fallas, CodeClimate D |
| 0-3 | Quality Gate ❌ (0-1/6), sin CII, CodeClimate F, no compila |

## FASE 2 — Python SDK Layer

**Skills:** `security-and-hardening`, `code-review-and-quality`

### Comandos mecánicos

```bash
# Build + test del SDK Python
dev-tools/setup_venv.ps1 2>&1 | tail -5
target/audit-venv/Scripts/python -m pytest vantadb-python/tests/ -v 2>&1 | tail -30

# Mypy type checking si configurado
target/audit-venv/Scripts/python -m mypy vantadb-python/ 2>&1 | tail -20

# Verificar imports (no runtime errors)
target/audit-venv/Scripts/python -c "import vantadb_py; print(vantadb_py.__version__)"
```

### Checklist manual

- [ ] Async concurrency limit? (B9 — Semaphore presente?)
- [ ] Type stubs (.pyi) para autocompletado en IDEs
- [ ] Error handling: panics en Rust no llegan como panic a Python
- [ ] Memory management: objetos grandes liberados correctamente
- [ ] API parity con Rust SDK (mismas operaciones, mismos parámetros)
- [ ] Thread safety: Python GIL + Rust threading

### Score: __/10

## FASE 3 — Web Frontend Layer

**Skills:** `frontend-ui-engineering`, `performance-optimization`, `security-and-hardening`, `seo-audit`, `audit-website`, `visual-review`

### Comandos mecánicos

```bash
# TypeScript check
npx tsc --noEmit 2>&1 | tail -20

# Lint
npx eslint . --ext .ts,.tsx 2>&1 | tail -20

# Bundle size
npx vite build 2>&1 | tail -10

# Squirrelscan audit (si está instalado)
squirrelscan --url http://localhost:5173 --format json 2>&1 | tail -30
```

### Checklist manual

**Performance:**
- [ ] Bundle JS < 150KB gzip, CSS < 25KB gzip
- [ ] Animations: una lib (GSAP) vs 3 (GSAP + Motion + AnimeJS)? (W13)
- [ ] React.memo en componentes pesados (Three.js, Nav, benchmarks)
- [ ] `useMemo`/`useCallback` solo donde hay rerender medible
- [ ] Images: lazy loading, dimensions, format moderno (avif/webp)
- [ ] Fonts: variable fonts, preconnect, display swap

**Accessibility:**
- [ ] Touch targets >= 44px (W17 — Apple HIG)
- [ ] prefers-reduced-motion en animaciones
- [ ] Keyboard navigation: focus visible, tab order
- [ ] Color contrast WCAG AA (4.5:1 texto normal, 3:1 texto grande)
- [ ] Alt text en todas las imágenes
- [ ] ARIA labels en interactive elements
- [ ] Skip-to-content link presente

**Security:**
- [ ] Security headers en Vercel/Nginx (W6 — HSTS, X-Content-Type-Options, CSP)
- [ ] CORS configurado correctamente
- [ ] No secrets en frontend bundle
- [ ] `innerHTML`/`dangerouslySetInnerHTML` revisados

**SEO (si aplica):**
- [ ] Twitter cards con site/creator (W9)
- [ ] Sitemap con todas las rutas
- [ ] JSON-LD completo (url, image, softwareVersion)
- [ ] Canonical URLs en blog
- [ ] Meta descriptions únicas por página
- [ ] Open Graph tags completos

**UX:**
- [ ] Three.js hero: error boundary, mobile touch, responsive (W15)
- [ ] Direct DOM mutation → React state? (W14)
- [ ] Loading states en lazy routes (Suspense)
- [ ] Empty states en listas/datasets vacíos
- [ ] Error boundaries por sección

**Design:**
- [ ] Tokens consistentes (--amber, --dark, --white en todo el CSS)
- [ ] OG image colores correctos (W5 — brand #ff5500, #0a0a0a)
- [ ] Tipografía: variables cargadas correctamente
- [ ] Responsive: mobile (640px), tablet (768px, 960px)

### Score: __/10

## FASE 4 — TypeScript SDK Layer

**Skills:** `code-review-and-quality`, `security-and-hardening`

### Comandos mecánicos

```bash
# Buscar el directorio del TS SDK
cd packages/
npx tsc --noEmit 2>&1 | tail -20
npx vitest run 2>&1 | tail -20

# Bundle size (si configurado)
npx tsc --noEmit --extendedDiagnostics 2>&1 | tail -10
```

### Checklist manual

- [ ] Test coverage expandido a 50+ tests (B16)
- [ ] Type stubs completos
- [ ] Error handling: errores tipados con códigos
- [ ] API documentation en JSDoc
- [ ] Browser + Node dual compatibility
- [ ] Bundle size monitoreado

### Score: __/10

## FASE 5 — CI/CD + Infra Layer

**Skills:** `ci-cd-and-automation`, `security-and-hardening`, `observability-and-instrumentation`

### Comandos mecánicos

```bash
# Verificar workflows de CI
Get-ChildItem .github/workflows/*.yml | ForEach-Object { Write-Output "--- $($_.Name) ---"; Get-Content $_ }

# GitHub Actions: estado de runs recientes
gh run list --limit 20 --json databaseId,workflowName,conclusion,status,headBranch,createdAt 2>&1

# GitHub Actions: runs fallidos
gh run list --limit 50 --json databaseId,workflowName,conclusion,headBranch,createdAt | ConvertFrom-Json | Where-Object { $_.conclusion -eq "failure" -or $_.conclusion -eq "cancelled" -or $_.conclusion -eq "timed_out" }

# GitHub Actions: detalle de cada workflow fallido
gh run view <FAILED_RUN_ID> --log --failed 2>&1 | tail -50

# GitHub Actions: workflows totales y health
$wf = gh run list --limit 100 --json conclusion 2>&1 | ConvertFrom-Json
$total = $wf.Count
$failed = ($wf | Where-Object { $_.conclusion -eq "failure" }).Count
$passed = ($wf | Where-Object { $_.conclusion -eq "success" }).Count
Write-Output "Total: $total, Passed: $passed, Failed: $failed, Rate: $([math]::Round($passed/$total*100,1))%"

# Dockerfile si existe
Test-Path Dockerfile && Get-Content Dockerfile

# Dependabot config
Get-Content .github/dependabot.yml

# Vercel config
Get-Content web/vercel.json
```

### Checklist manual

**CI/CD:**
- [ ] Fast Gate < 5 min (fmt → clippy → test)
- [ ] Heavy Certification manual (no bloquea PRs)
- [ ] Code coverage job existente (NUEVO-15)
- [ ] Dependabot para Cargo + npm + Actions + Docker
- [ ] Secret scanning en PRs
- [ ] sccache caching configurado
- [ ] Test splitting (--build-jobs 2 para Windows)

**Infra:**
- [ ] Docker multi-stage build optimizado
- [ ] Vercel: HSTS, redirects, headers configurados
- [ ] Release workflow (release-plz configurado)
- [ ] Pre-commit + pre-push hooks activos
- [ ] `.env.example` presente y actualizado

**Monitoring:**
- [ ] `/metrics` endpoint: auth opcional (P12)
- [ ] Tracing (opentelemetry) configurado
- [ ] Logging estructurado (no println!)
- [ ] Health check endpoint

### SonarQube-style Quality Gate (CI/CD)

| Condición | Threshold | Pasa/Falla |
|-----------|-----------|------------|
| CI pipeline < 5 min (Fast Gate) | ≤ 300s | |
| Secret scanning en PRs | activo | |
| Dependabot para todos los ecosistemas | Cargo + npm + Actions + Docker | |
| Release workflow automatizado | release-plz configurado | |
| Image size (Docker) | < 200MB | |

### CII Best Practices (CI/CD + Infra)

| Criterio | Nivel requerido | Pasa/Falla |
|----------|----------------|------------|
| Build system working | Passing | |
| CI implemented | Passing | |
| Vulnerability report process public | Passing | |
| Reproducible build | Silver | |
| Dependencies monitored | Silver | |
| Security audit < 18 months | Gold | |

### Score: __/10

**Fórmula:** Quality Gate (5pts) + CII level (1/2/3pts) + Coverage job (1pt) + Monitoring (1pt)

## FASE 6 — Docs + SEO Layer

**Skills:** `documentation-and-adrs`, `ai-seo`, `writing-guidelines`

### Comandos mecánicos

```bash
# Validar cobertura de docs
pwsh scripts/validate-docs-coverage.ps1 2>&1

# Verificar que todos los README existen
Get-ChildItem -Recurse -Filter "README.md" | ForEach-Object { $_.FullName }
```

### Checklist manual

**Technical docs:**
- [ ] `docs/api/` — todas las APIs documentadas
- [ ] `docs/operations/` — configuración, deployment, backup
- [ ] `docs/architecture/` — ADRs, diagramas, decisiones
- [ ] README.md actualizado (badges, features, quickstart)
- [ ] CHANGELOG.md actualizado (git-cliff)
- [ ] Docstrings en Rust (pub fn documentadas)
- [ ] Doc-drive: docs escritos ANTES del código

**Spanish docs:**
- [ ] Backlog.md actualizado
- [ ] bitacora.md issues resueltos marcados
- [ ] progreso/README.md migrado
- [ ] MPTS docs cross-referenciados con inglés

**Website SEO:**
- [ ] llms.txt presente (AI search optimization)
- [ ] robots.txt correcto
- [ ] Sitemap.xml actualizado
- [ ] Open Graph + Twitter cards en todas las páginas
- [ ] JSON-LD structured data
- [ ] Blog con canonical URLs
- [ ] Performance audit (squirrelscan o Lighthouse)

### SonarQube-style Quality Gate (Docs)

| Condición | Threshold | Pasa/Falla |
|-----------|-----------|------------|
| API documentation exists | todas las pub fn documentadas | |
| README.md badges + quickstart | presente y actualizado | |
| CHANGELOG.md from git-cliff | presente | |
| llms.txt presente | sí | |
| sitemap.xml + robots.txt | presente | |
| JSON-LD structured data | implementado | |

### CII Best Practices (Docs)

| Criterio | Nivel | Pasa/Falla |
|----------|-------|------------|
| Project description on website | Passing | |
| How to obtain, contribute, report bugs | Passing | |
| Contribution process explained | Passing | |
| Documentation in English | Passing | |
| Release notes with vuln info | Passing | |

### Score: __/10

## FASE 7 — Design + UX Layer

**Skills:** `plan-design-review`, `visual-review`, `platform-design`, `ux-heuristics`

### Checklist manual

**Design audit (plan-design-review):**
- [ ] Visual hierarchy: clear, scannable layouts
- [ ] Color palette: OKLCH consistency, contrast ratios
- [ ] Typography: scale, pairing, readability
- [ ] Spacing: consistent rhythm, whitespace
- [ ] Component consistency: reuse vs custom
- [ ] No generic AI slop patterns

**UX heuristics (Nielsen):**
- [ ] Visibility of system status (loading, progress)
- [ ] Match between system and real world (metaphors, language)
- [ ] User control and freedom (undo, cancel, back)
- [ ] Consistency and standards (platform conventions)
- [ ] Error prevention (validation, confirmations)
- [ ] Recognition rather than recall (visible options)
- [ ] Flexibility and efficiency (shortcuts, power user)
- [ ] Aesthetic and minimalist design (no noise)
- [ ] Help users recognize, diagnose, recover from errors
- [ ] Help and documentation (contextual help)

**Platform design (Apple HIG + Material 3):**
- [ ] Touch targets >= 44pt
- [ ] Navigation patterns estándar
- [ ] Motion: purposeful, not decorative
- [ ] Dark mode soportado

### Score: __/10

## FASE 8 — Architecture + Dependencies Layer

**Skills:** `doubt-driven-development`, `code-simplification`, `database-schema-designer`

### Comandos mecánicos

```bash
# Mapear estructura completa del workspace
codegraph_explore "vantadb workspace modules dependencies"

# Dependencias no usadas
cargo machete 2>&1

# Dependencias desactualizadas
cargo outdated 2>&1

# Licencias
cargo deny check 2>&1 | tail -20
```

### Checklist manual

**Architecture:**
- [ ] Modules: single responsibility, clean boundaries
- [ ] Circular dependencies? (codegraph_dependencies)
- [ ] Feature flags: correct gating, no leakage
- [ ] Error hierarchy: ChainedError pattern, no String variants
- [ ] Public API: semver-aware, breaking changes flagged
- [ ] Duplication: append_to_vstore / write_node_to_vstore (P6)
- [ ] Large files candidate to split: serialization.rs 1827L (P5)
- [ ] init_telemetry masivo (PERF-14)
- [ ] insert_hnsw 177L monolítica (DOC-02)

**Dependencies:**
- [ ] Dependencias no usadas (cargo machete limpio)
- [ ] Dependencias desactualizadas (cargo outdated mínimo)
- [ ] Licencias compatibles (cargo deny check limpio)
- [ ] Adapters (langchain, llamaindex, crewai, etc.): todos build?
- [ ] WASM: tamaño optimizado (opt-level = "s")
- [ ] Python: maturin build exitoso

**Database schema:**
- [ ] Fjall config: column families, compresión
- [ ] RocksDB config: block cache, bloom filters
- [ ] WAL: fsync policy, recovery testing
- [ ] Index: HNSW config (ef, M, flat_threshold)

### CodeClimate Maintainability Rating (Architecture)

| Métrica | Threshold para A | Rating actual |
|---------|-----------------|---------------|
| Circular dependencies | 0 | A/B/C/D/E |
| Large files (>1000L) | <3 | A/B/C/D/E |
| Single-responsibility modules | >80% de módulos | A/B/C/D/E |
| Tech debt ratio | <5% | A/B/C/D/E |
| Dependencies: unused | 0 | A/B/C/D/E |
| Dependencies: outdated | <3 crates | A/B/C/D/E |

A = 0-4 métricas fuera de rango, B = 5-8, C = 9-12, D = 13-16, F = 17+

### ISO 25010 Mapping (Architecture)

| Característica ISO 25010 | Cómo se evalúa en esta capa |
|--------------------------|---------------------------|
| **Maintainability** (modularity, reusability, analyzability, modifiability, testability) | Circular deps, large files, error hierarchy, public API semver |
| **Compatibility** (coexistence, interoperability) | Adapters build, WASM, Python bindings |
| **Portability** (adaptability, installability, replaceability) | Docker, Vercel deploy, cross-platform |

### Score: __/10

## FASE 9 — Hallazgos Encontrados (Findings)

> **Taxonomía completa de hallazgos.** Después de ejecutar Fases 1-8, clasificá
> cada hallazgo en una o más de estas categorías. Cada hallazgo debe tener:
> - **ID único** (ej: `H01-LOGIC-001` para Rust Core)
> - **Categoría** (de la taxonomía abajo)
> - **Severidad** (Critical / High / Medium / Low / Info)
> - **Capa** (Rust Core / Python SDK / Web Frontend / TS SDK / CI-CD / Docs / Design / Architecture)
> - **Archivo:Línea** exacto
> - **Descripción** del problema
> - **Evidencia** (log, stack trace, snippet, screenshot)
> - **Recomendación** de fix y referencia a herramienta/skill si aplica
> - **Esfuerzo estimado** (XS < 1h / S < 4h / M < 1d / L < 1w / XL > 1w)
> - **Relación con otros hallazgos** (duplicado, causa raíz, dependiente)
> - **Validación** (cómo verificar que está corregido: test, lint, comando)

**Skills a cargar adicionalmente para hallazgos:**
- `debugging-and-error-recovery` — para fallas lógicas y errores
- `code-review-and-quality` — ya cargada, aplicar checklist de hallazgos
- `doubt-driven-development` — adversarial review para hallazgos críticos/altos
- `ponytail-review` / `ponytail-audit` — detectar over-engineering en hallazgos
- `impeccable` / `plan-design-review` — para fallas de diseño (F7)
- `security-and-hardening` — para fallas de seguridad
- `performance-optimization` — para fallas de algoritmo/performance

### ID y Nomenclatura

```
H<FASE>-<CAT>-<NNN>
```

| Componente | Valores |
|-----------|---------|
| FASE | 01-08 (capa donde se encontró) |
| CAT | LOGIC / PATTERN / ARCH / DIRECTION / CLARITY / CODE / DESIGN / ERROR / MISSING / FEATURE / ALGO / ANY |
| NNN | 001, 002, ... secuencial |

Ejemplo: `H03-DESIGN-007` = Web Frontend, falla de diseño, hallazgo #7.

### Categorías de Hallazgos

Cada hallazgo debe clasificarse en UNA categoría primaria y puede tener categorías secundarias.

#### 1. Fallas Lógicas (`LOGIC`)

Errores en la lógica de negocio que causan comportamiento incorrecto.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **Off-by-one** | Índices fuera de rango, iteraciones incorrectas | `codegraph_explore` para flujo |
| **Race condition** | Estado compartido sin sincronización, TOCTOU | `security-and-hardening` |
| **Incorrect branching** | Condiciones booleanas invertidas, `if` faltante | `systematic-debugging` |
| **State management** | Transiciones de estado inválidas, estado inconsistente | `code-review-and-quality` |
| **Edge case no manejado** | Empty collections, None/Null, 0, max values | `test-driven-development` |
| **Input validation** | Validación insuficiente o incorrecta | `security-and-hardening` |
| **Error handling** | Errores ignorados, `unwrap()` injustificado, panic reachable | `code-review-and-quality` |
| **Type confusion** | Conversiones incorrectas, asumir subtipo sin check | `rust-analyzer diagnostics` |
| **Incorrect serialization** | Forward/backward compat, field missing, format mismatch | `cargo test` |
| **Async/sync mismatch** | Sync en async context, `.block()` sin cuidado | `performance-optimization` |

**Checklist de validación lógica:**
- [ ] `cargo test --workspace` pasa (tests existentes)
- [ ] Tests específicos para el edge case del hallazgo
- [ ] Fuzzing (si aplica): `cd fuzz && cargo +nightly fuzz run fuzz_parser`
- [ ] Verificación manual: código revisado línea por línea

#### 2. Fallas de Patrón (`PATTERN`)

Violaciones de patrones de diseño establecidos o anti-patrones.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **GoF pattern violation** | State/Strategy implementado con if-else gigante, Singleton mal implementado | `code-review-and-quality` |
| **Rust anti-pattern** | `clone()` everywhere, `Rc<RefCell>` en vez de `Arc<Mutex>`, `unsafe` innecesario | `code-review-and-quality` |
| **Python anti-pattern** | Mutable default args, `*args`/`**kwargs` para todo, falta type hints | `code-review-and-quality` |
| **JS/TS anti-pattern** | `any` everywhere, `null` vs `undefined` inconsistente, callback hell | `code-review-and-quality` |
| **React anti-pattern** | `useEffect` sin deps, key incorrecta, direct DOM mutation | `frontend-ui-engineering` |
| **CSS anti-pattern** | `!important`, inline styles, selector explosivo | `visual-review` |
| **API design anti-pattern** | Verbos en URL, error sin código, versioning incorrecto | `api-and-interface-design` |
| **Database anti-pattern** | N+1 queries, sin índices, full table scans frecuentes | `database-schema-designer` |
| **CQRS/CQRS violation** | Query commands en misma ruta, write en read path | `api-and-interface-design` |
| **Adapter pattern mal aplicado** | Integración acoplada al provider, sin abstracción | `doubt-driven-development` |

**Checklist de validación de patrones:**
- [ ] El patrón usado es el mínimo necesario (no over-engineering)
- [ ] No hay patrones mezclados (Strategy + Factory + Decorator en mismo lugar)
- [ ] El patrón se aplica consistentemente en todo el proyecto
- [ ] Si se desvía del patrón, hay `ponytail:` comment explícito

#### 3. Fallas de Arquitectura (`ARCH`)

Problemas estructurales en la organización del código.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **Circular dependencies** | Módulo A importa B que importa A | `codegraph_explore dependencies` |
| **Layering violation** | Capa superior llama a capa no adyacente | `codegraph_explore` |
| **God object / God module** | Archivo >1000L, módulo con demasiadas responsabilidades | `code-simplification` |
| **Feature leakage** | Funcionalidad de una feature en módulo de otra feature | `code-review-and-quality` |
| **Tight coupling** | Módulo conoce internas de otro módulo | `api-and-interface-design` |
| **Missing abstraction** | Código duplicado que debería abstraerse | `code-simplification` |
| **Premature abstraction** | Interface con 1 implementation, factory para 1 producto | `ponytail-audit` |
| **Leaky abstraction** | Abstracción que expone detalles de implementación | `doubt-driven-development` |
| **Incorrect module boundary** | Módulo mezcla concerns no relacionados | `codegraph_explore` |
| **Architecture mismatch** | Stack elegido vs stack necesario (ej: REST para streaming) | `idea-refine` |

**Checklist de validación arquitectónica:**
- [ ] `codegraph_explore "arch dependencies modules"` — sin ciclos
- [ ] Todas las dependencias apuntan en una dirección (no bidireccional entre layers)
- [ ] El diagrama de paquetes coincide con la arquitectura documentada
- [ ] No hay imports entre módulos que no deberían conocerse

#### 4. Fallas de Dirección del Proyecto (`DIRECTION`)

Problemas estratégicos, de producto o de gestión del proyecto.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **Scope creep** | Features que no están en el backlog/MPTS | `git log --oneline` + `docs/Backlog.md` |
| **Abandoned effort** | Código escrito, committeado, pero nunca terminado ni eliminado | `git log --oneline --grep="WIP\|wip\|draft"` |
| **Tech debt no priorizado** | Issues conocidos pero sin plan de resolución | `docs/bitacora.md`, `docs/Backlog.md` |
| **Strategic misalignment** | Feature implementada que no aporta al roadmap | `docs/VantaDB-MPTS/` |
| **Missing validation** | Feature shipping sin tests ni benchmarks | `cargo nextest` |
| **Over-engineering** | Solución demasiado compleja para el problema actual | `ponytail-audit` |
| **Under-engineering** | Sin tests, sin logging, sin error handling | `code-review-and-quality` |
| **No dogfooding** | El equipo no usa su propio producto | — |
| **Missing roadmap** | No hay dirección pública clara | `docs/CHANGELOG.md`, README |
| **Release cadence** | Releases demasiado espaciados o sin proceso | `git tag --sort=-creatordate`, release-plz |

**Checklist de validación de dirección:**
- [ ] Revisar `docs/Backlog.md` vs lo implementado — ¿hay features fuera de plan?
- [ ] Revisar `docs/bitacora.md` — ¿issues conocidos sin plan?
- [ ] Revisar `docs/VantaDB-MPTS/` — ¿el proyecto sigue el MPTS?
- [ ] `git log --oneline --grep="fix\|bug\|hotfix"` — ¿mucho bug fixing post-release?

#### 5. Fallas de Claridad (`CLARITY`)

Código, docs o comunicación que son difíciles de entender.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **Naming confuso** | Variables/funciones con nombre engañoso, abreviaturas | `writing-guidelines` |
| **Magic numbers/strings** | Constantes literales sin nombre | `code-review-and-quality` |
| **Lógica demasiado anidada** | >3 niveles de indentación, >10 líneas en una función | `code-simplification` |
| **Missing comments** | `unsafe` blocks sin SAFETY docs, lógica compleja sin explicación | `code-review-and-quality` |
| **Dead code** | Código comentado, funciones no llamadas, exports no usados | `cargo machete`, `ponytail-audit` |
| **Test incomprensible** | Test sin nombre descriptivo, arrange/act/assert mezclados | `test-driven-development` |
| **Doc desactualizada** | Documentación que no coincide con el código | `documentation-and-adrs` |
| **Error message confuso** | Mensajes de error que no ayudan a diagnosticar | `observability-and-instrumentation` |
| **Config oculta** | Valores hardcodeados que deberían ser configurables | `ci-cd-and-automation` |
| **Formatting inconsistente** | Mezcla de tabs/spaces, llaves inconsistente | `cargo fmt --check` |

**Checklist de validación de claridad:**
- [ ] Un dev nuevo puede entender el código solo leyendo
- [ ] `cargo doc --no-deps` sin warnings
- [ ] README.md tiene quickstart funcional
- [ ] No hay `TODO` sin issue asociado

#### 6. Fallas de Código (`CODE`)

Violaciones de estándares de calidad de código.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **Compiler warnings** | Warnings ignorados o suprimidos | `cargo check`, `npx tsc` |
| **Clippy lints** | Lints no corregidos | `cargo clippy` |
| **Lint supressions** | `#[allow()]` sin justificación | `cargo clippy` |
| **Unsafe sin SAFETY** | Todo `unsafe` block debe tener SAFETY comment | `security-and-hardening` |
| **Clone innecesario** | `clone()` en hot path o en tipos Copy | `performance-optimization` |
| **Alloc innecesaria** | Box/Arc/String donde no se necesita | `performance-optimization` |
| **Casting incorrecto** | `as` casts inseguros, `transmute` innecesario | `security-and-hardening` |
| **TOCTOU bugs** | Time-of-check-time-of-use en I/O | `security-and-hardening` |
| **Error swallowing** | `let _ =`, `ok()`, `ignore()` en Result/Error | `code-review-and-quality` |
| **Panic en library code** | `panic!()` en código que no es binary | `code-review-and-quality` |

**Checklist de validación de código:**
- [ ] `cargo check --workspace` sin warnings (0)
- [ ] `cargo clippy --workspace -- -D warnings` pasa
- [ ] `cargo fmt --check` pasa
- [ ] `npx tsc --noEmit` sin errors

#### 7. Fallas de Diseño (`DESIGN`)

Problemas de UI/UX y diseño visual.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **Inconsistencia visual** | Componentes similares se ven diferentes | `plan-design-review` |
| **Accesibilidad** | Contraste insuficiente, focus missing, aria faltante | `platform-design` |
| **Responsive broken** | Layout roto en mobile/tablet | Playwright MCP resize + screenshot |
| **Motion excesiva** | Animaciones sin propósito, sin `prefers-reduced-motion` | `design-motion-principles` |
| **AI slop** | Diseño genérico que parece template | `impeccable` |
| **Touch targets** | Botones < 44px, muy juntos | `platform-design` |
| **Loading states** | Sin skeleton/spinner en operaciones largas | `frontend-ui-engineering` |
| **Empty states** | Lista vacía sin mensaje útil | `frontend-ui-engineering` |
| **Error states** | Error mostra-do como raw JSON/stack trace al usuario | `frontend-ui-engineering` |

**Checklist de validación de diseño:**
- [ ] Playwright MCP snapshot + screenshot en 1440×900 y 390×844
- [ ] `plan-design-review` ejecutado (si aplica a web/)
- [ ] No hay layout shift en transiciones
- [ ] Modo claro/oscuro soportado

#### 8. Errores (`ERROR`)

Errores concretos que rompen build, tests, o runtime.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **Compilation error** | Cargo/tsc/build falla | `cargo check`, `npx tsc` |
| **Test failure** | Test(s) que fallan consistentemente | `cargo nextest`, `pytest`, `vitest` |
| **Flaky test** | Test que falla intermitentemente | `cargo nextest --retries 3` |
| **Build warning como error** | `-D warnings` causa falla | `cargo clippy -- -D warnings` |
| **Runtime panic** | `panic!` o `unreachable!` alcanzable en runtime | `codegraph_explore` |
| **Deadlock** | Mutex/RWLock sin orden consistente | `performance-optimization` |
| **Memory leak** | Arc cycles, forgot `drop()`, global state | `performance-optimization` |
| **CI failure** | Pipeline de CI fallando | `gh run list --conclusion failure` |
| **Security vulnerability** | CVE en dependencias | `cargo audit` |
| **Dependency conflict** | Versiones incompatibles | `cargo deny check` |

**Checklist de validación de errores:**
- [ ] `just verify` pasa completo
- [ ] `cargo audit` sin advisories críticos
- [ ] `gh run list --conclusion failure` = 0 runs fallidos en últimas 50
- [ ] Test flaky identificados y fixeados

#### 9. Cosas que Faltan (`MISSING`)

Elementos que deberían estar presentes pero no están.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **Missing test** | Función pública sin test | `test-driven-development` (coverage gap) |
| **Missing validation** | Input sin sanitizar, sin bounds check | `security-and-hardening` |
| **Missing error handling** | Result/Error ignorado | `code-review-and-quality` |
| **Missing docs** | `pub fn` sin docstring, API endpoint sin doc | `documentation-and-adrs` |
| **Missing CI gate** | Sin clippy, sin test gate, sin fmt check | `ci-cd-and-automation` |
| **Missing monitoring** | Sin logging, sin metrics, sin health endpoint | `observability-and-instrumentation` |
| **Missing recovery** | WAL sin replay, sin retry en fallos transitorios | `debugging-and-error-recovery` |
| **Missing migration** | Schema change sin migration path | `database-schema-designer` |
| **Missing env config** | `.env.example` desactualizado o faltante | — |
| **Missing license** | Archivos sin header de licencia | `cargo deny check` |

#### 10. Features que Faltan (`FEATURE`)

Capacidades que deberían existir para completitud del producto.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **Feature gap vs competitor** | Chroma/Pinecone/Milvus tienen X, VantaDB no | `docs/Backlog.md`, `docs/VantaDB-MPTS/` |
| **Missing integration** | Adapter faltante (ej: LangChain listo, pero no LlamaIndex) | `cargo check --workspace` |
| **Missing API endpoint** | CRUD incompleto, faltan filtros/paginación | `docs/api/` |
| **Missing storage backend** | Solo Fjall, falta RocksDB/InMemory para prod | `src/storage/` |
| **Missing auth** | Sin API key, sin JWT, sin RBAC | `security-and-hardening` |
| **Missing telemetry** | Sin tracing, sin metrics export | `observability-and-instrumentation` |
| **Missing cli command** | CLI sin subcomando necesario (ej: no hay `backup`) | `src/cli.rs` |
| **Missing python binding** | Feature de Rust sin binding en Python | `vantadb-python/` |
| **Missing WASM target** | Feature que no build para WASM | `vantadb-wasm/` |
| **Missing enterprise feature** | Multi-tenancy, audit log, SSO | `docs/VantaDB-MPTS/` |

#### 11. Fallas de Algoritmo (`ALGO`)

Ineficiencias algorítmicas o elección incorrecta de estructura de datos.

| Subcategoría | Qué buscar | Herramienta/Skill |
|-------------|-----------|-------------------|
| **N+1 queries** | Query en loop que debería ser batch | `database-schema-designer` |
| **Wrong data structure** | `Vec` para búsqueda, `HashMap` para orden, `String` para path | `performance-optimization` |
| **Unnecessary allocation** | String/vec creado y dropeado en hot path | `performance-optimization` |
| **Inefficient algorithm** | O(n²) donde O(n log n) es posible, bubble sort en prod | `performance-optimization` |
| **Cache missing** | Resultado costoso recalculado sin cache | `performance-optimization` |
| **Lock contention** | Mutex global en hot path | `performance-optimization` |
| **Serialization overhead** | JSON donde binary es posible, serde sin optimizar | `performance-optimization` |
| **Index missing** | Query sin índice, full table scan | `database-schema-designer` |
| **Inefficient batch** | Insert one-by-one en vez de batch insert | `code-review-and-quality` |
| **Memory bloat** | Cargar todo en memoria cuando streaming es posible | `performance-optimization` |

**Checklist de validación algorítmica:**
- [ ] Benchmarks: `cargo bench 2>&1 | tail -30` — sin regresiones
- [ ] Hot paths identificados y optimizados
- [ ] No hay allocaciones innecesarias en loops calientes

#### 12. Otras Fallas (`ANY`)

Catch-all para hallazgos que no encajan en categorías anteriores.

| Subcategoría | Qué buscar |
|-------------|-----------|
| **Security** | Derivar a `security-and-hardening` skill |
| **Performance** | Derivar a `performance-optimization` skill |
| **Dependency** | Licencia incompatible, CVE, deprecado |
| **Build** | Build lento (>5min), sin caché, sin parallel |
| **DevX** | Developer experience pobre: hot reload roto, tooling ausente |
| **Deploy** | Deploy manual, sin rollback, sin canary |
| **Scale** | Sin horizontal scaling, sin sharding, sin replication |
| **Cost** | Infraestructura cara vs necesaria |

### Priorización de Hallazgos

| Severidad | Definición | SLA sugerido |
|-----------|-----------|-------------|
| **Critical** | Data loss, security breach, build broken, prod down | Fix < 24h |
| **High** | Feature broken, major perf regression, test suite red | Fix < 72h |
| **Medium** | UX roto en path común, warning en CI, sin docs | Fix < 2 semanas |
| **Low** | Code smell, deuda menor, mejora no urgente | Backlog |
| **Info** | Sugerencia, observación, mejora futura | Cuando se pueda |

### Procesamiento de Hallazgos

1. **Recolectar** de todas las fases (F1-F8)
2. **Clasificar** en categorías (1-12)
3. **Priorizar** por severidad
4. **Deduplicar** (mismo root cause, diferente síntoma)
5. **Agrupar** por capa y por equipo responsable
6. **Verificar** cada hallazgo (que sea reproducible, no falso positivo)
7. **Documentar** en el reporte final (FASE 10)

### Formato de Salida

Cada hallazgo se documenta como:

```markdown
### [H<FASE>-<CAT>-<NNN>] <Título corto>

| Campo | Valor |
|-------|-------|
| **Categoría** | LOGIC / PATTERN / ARCH / DIRECTION / CLARITY / CODE / DESIGN / ERROR / MISSING / FEATURE / ALGO / ANY |
| **Subcategoría** | |
| **Severidad** | 🔴 Critical / 🟡 High / 🔵 Medium / ⚪ Low / ℹ️ Info |
| **Capa** | Rust Core / Python SDK / Web Frontend / TS SDK / CI-CD / Docs / Design / Architecture |
| **Archivo** | `ruta/al/archivo.rs:42` |
| **Evidencia** | ```
error[E0382]: use of moved value
``` |
| **Root Cause** | Descripción de por qué ocurre |
| **Impacto** | Qué se rompe o degrada |
| **Recomendación** | Cómo fixearlo |
| **Esfuerzo** | XS / S / M / L / XL |
| **Relacionado con** | H<FASE>-<CAT>-<NNN> (si aplica) |
| **Validación** | Comando o test que verifica el fix |
```

## FASE 10 — Reporte Final

Después de ejecutar todas las fases (incluyendo FASE 9 Hallazgos), producí un reporte estructurado:

```markdown
# VantaDB Full Review — <YYYY-MM-DD>

## Resumen Ejecutivo

### Scores por capa

| Capa | Score | Quality Gate | Rating | CII Level | ISO 25010 |
|------|-------|-------------|--------|-----------|-----------|
| Rust Core | _/10 | ✅/❌ | A-E | _ | _ |
| Python SDK | _/10 | ✅/❌ | A-E | _ | _ |
| Web Frontend | _/10 | ✅/❌ | A-E | _ | _ |
| TS SDK | _/10 | ✅/❌ | A-E | _ | _ |
| CI/CD + Infra | _/10 | ✅/❌ | A-E | _ | _ |
| Docs + SEO | _/10 | ✅/❌ | A-E | _ | _ |
| Design + UX | _/10 | ✅/❌ | A-E | _ | _ |
| Architecture | _/10 | ✅/❌ | A-E | _ | _ |
| **Total** | **__/80** | **__/8 ✅** | | | |

> 🟢 Score >= 8 | 🟡 Score 5-7 | 🔴 Score < 5

### ISO 25010 Coverage Heatmap

| Característica | Cubierta por | Nivel |
|---------------|-------------|-------|
| Functional suitability | F1 (API), F2 (Python), F4 (TS) | 🟢/🟡/🔴 |
| Reliability | F1 (WAL, HNSW), F5 (HA) | 🟢/🟡/🔴 |
| Performance efficiency | F1 (benchmarks), F3 (bundle) | 🟢/🟡/🔴 |
| Compatibility | F2, F4, F8 (adapters) | 🟢/🟡/🔴 |
| Usability | F3 (UX), F7 (Design) | 🟢/🟡/🔴 |
| Security | F1 (unsafe), F3 (CSP), F5 (secrets) | 🟢/🟡/🔴 |
| Maintainability | F8 (architecture) | 🟢/🟡/🔴 |
| Portability | F5 (Docker, WASM) | 🟢/🟡/🔴 |

### SonarQube-style Quality Gate Summary

| Condición global | Aplica a | Resultado |
|-----------------|----------|-----------|
| No new reliability issues | Todo el workspace | ✅/❌ |
| No new security issues | Todo el workspace | ✅/❌ |
| No new maintainability issues | Todo el workspace | ✅/❌ |
| Coverage ≥ 80% on new code | Rust + Python + TS | ✅/❌ |
| Duplication ≤ 3% on new code | Rust + Python + TS | ✅/❌ |
| All hotspots reviewed | Rust + Web | ✅/❌ |
| No leaked credentials | Todo el repo | ✅/❌ |
| Vulns fixed < 60 days | Dependencias | ✅/❌ |
| **Overall Quality Gate** | | **✅/❌** |

## Issues Prioritizados

### 🔴 Críticos (arreglar antes del próximo release)
1. [ ] **Issue** — Archivo:Línea — Recomendación
2. [ ] ...

### 🟡 Altos (arreglar esta iteración)
3. [ ] ...

### 🔵 Medios (backlog)
4. [ ] ...

### ⚪ Buenos para tener (cuando se pueda)
5. [ ] ...

## Capa por Capa

### Rust Core (_/10)
**Quality Gate:** ✅/❌ — Rating: A-E — CII Level: _ — ISO 25010: _

**Commands:**
- `cargo check`: ✅⬜❌
- `cargo clippy`: ✅⬜❌ — N warnings
- `cargo nextest`: ✅⬜❌ — N passed / N failed / N skipped
- `cargo deny`: ✅⬜❌
- `cargo audit`: ✅⬜❌
- `cargo machete`: ✅⬜❌

**Quality Gate conditions:**
- [ ] Reliability Rating = A (✅/❌)
- [ ] Security Rating = A (✅/❌)
- [ ] Maintainability Rating = A (✅/❌)
- [ ] Coverage ≥ 80% (✅/❌)
- [ ] Duplication ≤ 3% (✅/❌)
- [ ] All hotspots reviewed (✅/❌)

**Issues:**
- ...

**Recomendaciones:**
- ...

### [resto de capas... mismo formato]

## Resumen de Hallazgos (de FASE 9)

| Categoría | Critical | High | Medium | Low | Info | Total |
|-----------|----------|------|--------|-----|------|-------|
| LOGIC | _ | _ | _ | _ | _ | _ |
| PATTERN | _ | _ | _ | _ | _ | _ |
| ARCH | _ | _ | _ | _ | _ | _ |
| DIRECTION | _ | _ | _ | _ | _ | _ |
| CLARITY | _ | _ | _ | _ | _ | _ |
| CODE | _ | _ | _ | _ | _ | _ |
| DESIGN | _ | _ | _ | _ | _ | _ |
| ERROR | _ | _ | _ | _ | _ | _ |
| MISSING | _ | _ | _ | _ | _ | _ |
| FEATURE | _ | _ | _ | _ | _ | _ |
| ALGO | _ | _ | _ | _ | _ | _ |
| ANY | _ | _ | _ | _ | _ | _ |
| **Total** | **_** | **_** | **_** | **_** | **_** | **_** |

### Top 5 Hallazgos por Severidad

1. [H01-ERROR-001] ...
2. [H03-DESIGN-001] ...
3. [H08-ARCH-001] ...
4. [H05-MISSING-001] ...
5. [H06-CODE-001] ...

## Hallazgos Transversales

- Patrones que se repiten en múltiples capas
- Oportunidades de unificación
- Tech debt compartido
- [H01-LOGIC-001] — mismo pattern en Rust, Python y TS

## Recomendaciones Generales

1. Prioridad más alta:
2. Próximo release:
3. Largo plazo:

## Resumen ISO 25010

| Característica | Score | Brecha |
|---------------|-------|--------|
| Functional suitability | _/10 | |
| Reliability | _/10 | |
| Performance efficiency | _/10 | |
| Compatibility | _/10 | |
| Usability | _/10 | |
| Security | _/10 | |
| Maintainability | _/10 | |
| Portability | _/10 | |
| **Total** | **__/80** | |

## CII Best Practices Assessment
- **Current level:** None / Passing / Silver / Gold
- **Gaps for next level:** ...
- **Target level:** ...

---

_Generado por vantadb-full-review, usando code-review-and-quality, security-and-hardening,
performance-optimization, audit-website, visual-review, plan-design-review,
ci-cd-and-automation, seo-audit, database-schema-designer, writing-guidelines,
documentation-and-adrs, doubt-driven-development, code-simplification,
ponytail-audit, ponytail-review, debugging-and-error-recovery, systematic-debugging,
impeccable, platform-design, design-motion-principles, observability-and-instrumentation,
git-workflow-and-versioning, api-and-interface-design, test-driven-development,
frontend-ui-engineering.
Basado en ISO/IEC 25010, SonarQube Quality Gates, OpenSSF CII Best Practices,
OWASP ASVS v5.0, CodeClimate/Qlty maintainability scoring, y GitHub Actions CI status._
```

## Cómo ejecutar el review

### Review completo (todas las capas)

```
# OpenCode TUI
/loop-goal --max-turns 50 --check "cargo check --workspace" --safe --prompt-file .opencode/skills/vantadb-full-review/loop-prompt.md Ejecutá vantadb-full-review contra el proyecto VantaDB. Revisá todas las 10 fases (F0-Setup, F1-F8 capas técnicas, F9-Hallazgos, F10-Reporte). Usá sub-agentes para paralelizar. Producí el reporte final en docs/reviews/YYYY-MM-DD-full-review.md.
```

### Review de una sola capa

Cargá la skill y especificá la capa:

```
skill vantadb-full-review
Revisá solo la capa Rust Core Layer del proyecto VantaDB.
```

### Review desde terminal

```powershell
.\harness-executor.ps1 -PlanFile docs/plans/YYYY-MM-DD-full-review.md
```

## Herramientas de referencia

### CLI / MCP

| Herramienta | Para qué se usa en el review |
|-------------|------------------------------|
| `cargo check` | Compilación de todo el workspace |
| `cargo clippy` | Lints de Rust (CodeClimate maintainability) |
| `cargo fmt --check` | Formato de código |
| `cargo nextest` | Tests de Rust |
| `cargo deny` | Licencias y advisories |
| `cargo audit` | Security advisories |
| `cargo machete` | Dependencias no usadas |
| `cargo outdated` | Dependencias desactualizadas |
| `cargo bloat` | Tamaño del binario |
| `codegraph_explore` | Análisis estructural, blast radius |
| `rust-analyzer-mcp diagnostics` | Errores de compilación por archivo |
| `npx tsc --noEmit` | TypeScript check |
| `npx vitest run` | Tests de TS/web |
| `dev-tools/setup_venv.ps1` | Build Python SDK |
| `pytest` | Tests de Python SDK |
| `just verify` | Pre-flight completo |
| `pwsh scripts/validate-docs-coverage.ps1` | Cobertura de docs |
| Playwright MCP | Visual review, screenshots, responsive testing |
| `audit-website` (squirrelscan) | Website audit (230+ reglas) |
| `gh run list` / `gh run view` | Estado de GitHub Actions (workflows fallidos) |
| `git log --oneline --graph` | Análisis de historial y ramas |
| `git for-each-ref` | Detección de dead branches |
| `git log --oneline --grep="revert\|fixup\|WIP"` | Commits problemáticos |
| `git log --format="%s"` | Validación de conventional commits |
| `git log --name-only` \| churn analysis | Archivos con más cambios |

### Sistemas de Evaluación de Referencia

| Sistema | URL | Propósito en el review |
|---------|-----|----------------------|
| **ISO/IEC 25010 (SQuaRE)** | https://iso.org/standard/35733.html | Modelo de calidad de producto: 8 características, 31 subcaracterísticas |
| **SonarQube Quality Gate** | https://docs.sonarsource.com/sonarqube-server/2026.3/quality-standards-administration/managing-quality-gates/introduction-to-quality-gates | Quality gates con thresholds de coverage, duplicación, ratings, hotspots |
| **OpenSSF CII Best Practices** | https://www.bestpractices.dev/en/criteria | 3 niveles (Passing/Silver/Gold) de madurez open source |
| **OWASP ASVS v5.0** | https://owasp.org/www-project-application-security-verification-standard/ | 3 niveles (L1/L2/L3) de verificación de seguridad en 14 categorías |
| **CodeClimate / Qlty** | https://qlty.sh | Maintainability scoring A-F basado en time-to-fix estimates |

### Skills instalables para mejorar el review

> Skills que no están instaladas pero se pueden agregar para profundizar el review.

| Skill | Instalación | Propósito | Fuente |
|-------|-------------|-----------|--------|
| **trailofbits/static-analysis** | `npx skills add trailofbits/skills/static-analysis` | Análisis de seguridad contextual (auth, data flow, trust boundaries) | Trail of Bits (AgenticSkills) |
| **trailofbits/differential-review** | `npx skills add trailofbits/skills/differential-review` | Security review enfocado en git diff — solo código cambiado | Trail of Bits (AgenticSkills) |
| **code-review-expert** | `npx skills add code-review-expert` | Patrones de code review cubriendo arquitectura, perf y seguridad | AgenticSkills |
| **Snyk MCP** | Add `@snyk/mcp-server` a MCP config | Escaneo de vulnerabilidades en dependencias en tiempo real | Snyk |
| **thinkingpatterns** | `npx skills add thinkingpatternsai/skills -s thinkingpatterns` | Plataforma de scanning agents: arquitectura, seguridad, reliability, escalabilidad | ThinkingPatterns.ai |

### Skills relacionadas

| Skill | Propósito |
|-------|-----------|
| `code-review-and-quality` | Framework de revisión multi-eje |
| `ponytail-audit` | Detección de over-engineering en todo el repo |
| `ponytail-review` | Revisión de over-engineering en diff actual |
| `doubt-driven-development` | Adversarial review para hallazgos críticos |
| `debugging-and-error-recovery` | Root-cause debugging para fallas lógicas y errores |
| `systematic-debugging` | Metodología de debugging paso a paso |
| `code-simplification` | Detección de complejidad innecesaria |
| `plan-design-review` | Senior Designer Review: puntúa diseño 0-10 |
| `impeccable` | Design critique, anti AI-Slop, accesibilidad |
| `platform-design` | Apple HIG + Material 3 + WCAG 2.2 (300+ reglas) |
| `design-motion-principles` | Auditoría de animaciones anti-slop |
| `seo-audit` | Auditoría SEO técnica |
| `audit-website` | Website audit (230+ reglas con squirrelscan) |
| `visual-review` | Visual review pipeline (Playwright + ImageMagick + pixelmatch) |
| `performance-optimization` | Análisis de performance, bottlenecks |
| `security-and-hardening` | Revisión de seguridad y hardening |
| `database-schema-designer` | Review de schema de base de datos |
| `observability-and-instrumentation` | Detección de falta de logging/metrics/tracing |
| `ci-cd-and-automation` | Revisión de CI/CD pipelines y quality gates |
| `git-workflow-and-versioning` | Análisis de historial git y versionado |
| `writing-guidelines` | Revisión de claridad en docs y comunicación |
| `api-and-interface-design` | Validación de diseño de APIs y boundaries |
| `test-driven-development` | Detección de falta de cobertura de tests |
| `frontend-ui-engineering` | Review de componentes React, estados, UX |
