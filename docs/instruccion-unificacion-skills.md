# Instrucción de Trabajo: Unificación de 3 Skills de Auditoría/Revisión/Certificación

> **Fecha:** 2026-07-26
> **Proyecto:** VantaDB → Unificación de skills: `vantadb-full-review`, `vantadb-certify`, `vantadb-audit`
> **Plataforma destino:** OpenCode (agentes de IA)

---

## Objetivo Principal

Analizar, comparar y unificar **3 skills de IA** en una **única skill canónica y genérica**, que mantenga toda la potencia para VantaDB pero que esté diseñada desde cero para ser **reutilizable en cualquier proyecto de software**.

El entregable NO es un PDF de análisis. El entregable es la **skill unificada completa**, funcional, lista para instalarse en OpenCode.

---

## Requisito Crítico: Investigar el Ecosistema OpenCode

Antes de diseñar la skill, debes investigar cómo funcionan las skills en **OpenCode** (la plataforma donde se usará principalmente). Esto incluye:

### 1. Formato de Skills en OpenCode

Investiga y verifica:

- **Formato de frontmatter**: ¿OpenCode usa el mismo frontmatter YAML que Claude Code? ¿Tiene campos específicos adicionales?
- **Estructura de directorios**: ¿La skill va en `C:\Users\Eros\VantaDB Proyect\VantaDB\.opencode\skills\<nombre>\SKILL.md` o hay otra convención?
- **Compatibilidad**: ¿OpenCode soporta `compatibility: [opencode, claude-code]`? ¿Hay flags específicos?
- **Referencias a otras skills**: ¿Cómo se cargan skills anidadas? ¿Con `skill <nombre>`, con `require()`, con `import`?

Usa `websearch` y `webfetch` para consultar:
- Documentación oficial de OpenCode skills
- Repositorios de ejemplo de skills para OpenCode
- Diferencias con el formato Anthropic/Claude Code skills

### 2. Sistema de Sub-Agentes (Context Budget)

Las 3 skills actuales son monolíticas — el agente principal ejecuta TODO: cargo check, clippy, tests, review, reporte. Esto consume todo el contexto del agente y produce timeouts o truncamiento en projects grandes como VantaDB (~790 archivos, 17+ crates).

La skill unificada DEBE usar **sub-agentes en paralelo** para escalar sin consumir el contexto del agente principal.

Investiga y aplica:

- **Paralelismo por fase**: cada fase (Core, Bindings, Web, CI/CD, Docs, Security, etc.) se delega a un sub-agente independiente
- **Context budget**: el agente principal solo orquesta y consolida resultados. Los sub-agentes ejecutan y reportan hallazgos
- **Mecanismo de sub-agentes en OpenCode**: usa la tool `task(description, prompt, subagent_type)` para lanzar sub-agentes. Ejemplo:
  ```
  task("rust-core-check", "Ejecuta cargo check, clippy y tests en el workspace", "vanta-worker")
  task("python-sdk-check", "Build + test del SDK Python", "vanta-worker")
  task("security-audit", "Carga security-and-hardening y revisa unsafe blocks", "vanta-audit")
  ```
- **Fan-out / fan-in**: lanza N sub-agentes en paralelo, espera todos los resultados, luego consolida
- **Tipos de sub-agentes disponibles**: `vanta-worker` (implementación), `vanta-audit` (seguridad), `vanta-tuner` (performance), `vanta-chaos` (fuzzing), `vanta-docs` (documentación)
- **Recolección de resultados**: cada sub-agente devuelve un reporte estructurado (hallazgos, scores, pass/fail) que el orquestador consolida
- **Manejo de fallos**: si un sub-agente falla, el orquestador decide si abortar (fase crítica como compilación) o continuar con warning (fase cognitiva como code review)

### 3. Integración con el Sistema de Tasks de OpenCode

OpenCode tiene un sistema de tasks (`campaign_*` tools MCP). La skill debe:

- Usar `campaign_get_next_task` / `campaign_update_task_state` para trackear progreso
- Usar `campaign_memory_write` para registrar decisiones y lecciones aprendidas
- Integrarse con el plan file system (`docs/plans/plan-*.md`)

### 4. Modo Ponytail (Lazy Mode)

OpenCode tiene integrado Ponytail (lazy senior dev mode). La skill debe respetar:

- `/ponytail lite|full|ultra` afecta qué tan agresiva es la revisión de over-engineering
- `ponytail-review` para revisar el diff por complejidad innecesaria
- `ponytail-audit` para auditoría completa de sobreingeniería
- No cargar estas skills si el modo es `lite` o `off`

---

## Requisito: Arquitectura de Sub-Agentes en Paralelo (Obligatorio)

**Problema actual:** Las 3 skills cargan todo en el contexto del agente principal. Un `cargo check --workspace` en VantaDB ya son ~2 minutos de output. Sumado a clippy, tests, Python, Web, docs, security review — el agente se queda sin contexto y trunca resultados.

**Solución obligatoria:** La skill unificada debe usar **fan-out con sub-agentes paralelos**.

### Arquitectura

```
AGENTE PRINCIPAL (orquestador — contexto mínimo)
│
│  FASE 0: Pre-check (git status, diff analysis — rápido, local)
│
│  LANZA EN PARALELO (task tool):
│  ├── Sub-agente 1: Core Language Check (compile + lint + test)
│  ├── Sub-agente 2: Bindings/SDKs (Python/TS/WASM)
│  ├── Sub-agente 3: Web Frontend (si existe)
│  ├── Sub-agente 4: CI/CD + Dependencies
│  ├── Sub-agente 5: Documentation Coverage
│  └── Sub-agente 6: Architecture Review
│
│  ESPERA TODOS (await parallel)
│  │
│  FASE 7: Consolidación de resultados (solo metadata + hallazgos)
│
│  FASE 8: Reporte final (estructurado, sin logs crudos)
│
│  OPCIONAL (si modo == full):
│  ├── Sub-agente 7: Security Audit
│  ├── Sub-agente 8: Performance Audit
│  └── Sub-agente 9: Code Review (skill-based)
```

### Reglas de Paralelismo

| Condición | Acción |
|-----------|--------|
| Fase crítica (core language check) falla | Abortar — no tiene sentido revisar bindings si no compila |
| Fase no crítica (docs, architecture) falla | Continuar — registrar hallazgo, no bloquear |
| Sub-agente no responde en 5 minutos | Timeout — marcar como fallido, continuar |
| Máximo de sub-agentes paralelos | 4 simultáneos (para no saturar contexto del orquestador) |
| Resultado de cada sub-agente | Estructura JSON/YAML: `{phase, status, score, findings: [{id, severity, file, description, recommendation}]}` |

### Budget de Contexto

| Componente | Límite estimado |
|------------|----------------|
| Orquestador (agente principal) | < 10% del contexto total (solo coordinación) |
| Cada sub-agente | 15-20% del contexto (ejecuta una fase, reporta, termina) |
| Consolidación | < 5% (solo tabla de resultados + hallazgos priorizados) |
| Reporte final | < 5% (formato estructurado, sin verbose) |

---

## Requisito Crítico II: Generalización Obligatoria

**Las 3 skills actuales están acopladas a VantaDB** — mencionan paths concretos (`vantadb-python/tests/`, `web/src/app/`, `crates/vantadb-server/`), comandos específicos (`cargo nextest run --profile audit`, `dev-tools/setup_venv.ps1`), y configuraciones hardcodeadas (Quality Gates, scores, layers). Esto las hace **inservibles para otros proyectos**.

### 1. Separar Configuración de Comportamiento

- El **framework de revisión** (fases, capas, scoring, quality gates) debe ser genérico y configurable
- Los **detalles específicos de VantaDB** (paths, comandos, umbrales) deben vivir en un perfil/config externo
- La skill debe detectar automáticamente el lenguaje del proyecto (Rust/Python/TS/Go/etc.) y ajustar su comportamiento

### 2. Sistema de Perfiles (Profiles)

- **Perfil por defecto**: revisión genérica que funciona para cualquier proyecto de software
- **Perfil VantaDB**: hereda el perfil por defecto y agrega las capas específicas (WAL, HNSW, PyO3 bindings, WASM, etc.)
- Los perfiles deben ser archivos YAML/TOML externos que la skill carga, no código hardcodeado
- Mecanismo de `--profile <name>` o detección automática (si detecta `Cargo.toml` con workspace → Rust multi-crate)

### 3. Capas Abstractas, No Concretas

En vez de "FASE 1 — Rust Core Layer" (VantaDB-specific), debe ser un sistema de capas que se auto-configuran:

```yaml
# profiles/default.yml
phases:
  - id: core_language
    name: "Core Language Check"
    description: "Compile + lint + unit tests"
    auto_detect: true            # detecta lenguaje del proyecto
    critical: true               # si falla, aborta
    parallel: true               # se lanza como sub-agente
    subagent_type: "vanta-worker"
    commands:
      rust: ["cargo check --workspace", "cargo clippy -- -D warnings", "cargo test"]
      python: ["python -m pytest", "mypy src/"]
      typescript: ["npx tsc --noEmit", "npx vitest run"]
      go: ["go vet ./...", "go test ./..."]

  - id: bindings
    name: "SDK/Bindings Check"
    description: "Python, WASM, TypeScript SDKs"
    auto_detect: true
    critical: false
    parallel: true
    subagent_type: "vanta-worker"
    detect:
      - pyproject.toml with maturin
      - Cargo.toml with wasm-pack
      - package.json with tsc

  - id: web_frontend
    name: "Web Frontend Check"
    auto_detect: true
    subagent_type: "vanta-worker"
    detect:
      - package.json + next.config   → next build
      - package.json + vite.config   → vite build
      - package.json + astro.config  → astro build
```

### 4. Trigger Universal

| Comando | Acción | Sub-agentes |
|---------|--------|-------------|
| `skill unified-review` | Carga y ejecuta con detección automática | Todos los detectados |
| `/review` | Modo interactivo, perfil por defecto | Detectados |
| `/review quick` | Solo core language checks | 1 sub-agente |
| `/review certify` | Todas las capas, aborta en fallo crítico | Todos en paralelo |
| `/review full` | Revisión completa + scoring + security + perf | Todos + 2 opcionales |
| `/review --profile vantadb` | Usa perfil VantaDB | Perfil VantaDB |
| `/audit` | Alias legacy para compatibilidad | Según modo |

### 5. Sin Pérdida de Potencia para VantaDB

El perfil VantaDB debe ser **estrictamente igual o superior** a lo que las 3 skills actuales ofrecen por separado:

| Capacidad | Hoy en | En skill unificada |
|-----------|--------|-------------------|
| Quality Gates ISO 25010 + SonarQube + CII + OWASP + CodeClimate | full-review | Perfil VantaDB → modo full |
| Pre-push certification gate de 8 capas | certify | Perfil VantaDB → modo certify |
| Pipeline de auditoría con 4 modos | audit | Entry point unificado |
| Scoring multi-dimensional con pesos configurables | full-review | Sistema de scoring genérico + overrides VantaDB |
| CI/CD parity check | certify (L7a) | Fase dedicada en perfil VantaDB |
| Findings taxonomy con 12 categorías | full-review (F9) | Taxonomía genérica + extensiones VantaDB |
| Reporte estructurado con executive summary | full-review (F10) | Formato único de reporte |
| Pre-push hook generator | certify | Genera hook para cualquier plataforma |
| Security audit con skills | audit (P2) | Sub-agente con vanta-audit |
| Performance audit con skills | audit (P3) | Sub-agente con vanta-tuner |

---

## Material de Entrada: 3 Skills Adjuntas

### Skill 1: `vantadb-full-review` (1,236 líneas)

**Propósito actual:** Revisión integral multi-capa de todo el proyecto VantaDB. Produce un reporte estructurado con scores numéricos, quality gates, y hallazgos clasificados.

**Fortalezas a conservar:**
- Sistema de 4 dimensiones de scoring (Quality Gate ✅/❌, Rating A-E, Score 0-10, CII Level)
- Taxonomía de hallazgos con 12 categorías y subcategorías detalladas
- Formato de reporte final estructurado (tablas de scores, heatmap ISO 25010, issues priorizados)
- Referencia a sistemas de evaluación industriales (ISO 25010, SonarQube, CII, OWASP, CodeClimate)

**Debilidades a eliminar:**
- Todo el contenido VantaDB-specific: paths, comandos, nombres de crates
- Skills de diseño cargadas (plan-design-review, impeccable, platform-design) — solo relevantes si el proyecto tiene frontend
- Peso de scoring hardcodeado (ISO 20%, SonarQube 25%, etc.) — debe ser configurable por proyecto
- 8 capas fijas que asumen Rust + Python + TS + Web — debe detectar qué existe
- Ejecución secuencial monolítica — debe usar sub-agentes paralelos

**Patrón de generalización:**
```
Capas actuales (VantaDB, secuencial):
  F1 Rust Core → F2 Python SDK → F3 Web Frontend → F4 TS SDK → F5 CI/CD → F6 Docs → F7 Design → F8 Architecture

Capas genéricas (paralelas vía sub-agentes):
  L1 Core Language (detecta: Cargo.toml / pyproject.toml / package.json / go.mod)
  L2 Bindings/SDKs (detecta: pyproject.toml con maturin, wasm-pack, tsconfig.json)
  L3 Web Frontend (detecta: next.config, vite.config, astro.config)
  L4 CI/CD + Dependencies (detecta: .github/, Cargo.lock, package-lock.json)
  L5 Documentation (detecta: docs/, README.md, CHANGELOG.md)
  L6 Architecture (análisis estructural con codegraph u otras tools)
  L7 Security (sub-agente vanta-audit, opcional)
  L8 Performance (sub-agente vanta-tuner, opcional)
```

### Skill 2: `vantadb-certify` (158 líneas)

**Propósito actual:** Pre-push certification gate que ejecuta 8 layers secuenciales. Falla si alguna capa mecánica no pasa.

**Fortalezas a conservar:**
- Flujo de 8 layers con orden de dependencias y aborto controlado
- CI/CD parity check (verificar que cambios en Cargo.toml/package.json tengan correlato en workflows CI)
- Cognitive review con skills que pueden vetar pero no bloquear el pipeline

**Debilidades a eliminar:**
- Layer 0 "CodeGraph Impact Analysis" asume VantaDB — detectar si existe `.codegraph/`
- Comandos hardcodeados — usar herramientas estándar del lenguaje detectado
- Paths fijos de scripts — mover a perfil VantaDB
- Ejecución secuencial — las layers independientes deben ir en paralelo

**Patrón de generalización:**
```
Layer 0: Diff Impact Analysis (sub-agente: codegraph si existe, o git diff analysis)
Layer 1: Core Language (sub-agente vanta-worker, crítico)
Layer 2: Bindings (sub-agente vanta-worker, si existen)
Layer 3: Web (sub-agente vanta-worker, si existe)
Layer 4: CI Parity (sub-agente vanta-lead)
Layer 5: Docs Coverage (sub-agente vanta-docs)
Layer 6: Code Review (sub-agente vanta-audit, opcional)
```

### Skill 3: `vantadb-audit` (151 líneas)

**Propósito actual:** Orquestador de auditoría con 4 modos (quick/certify/review/full). Se invoca con `/audit`.

**Fortalezas a conservar:**
- Sistema de modos con matriz de fases vs modos
- Aborto controlado (Phase 1 falla → stop; fases cognitivas fallan → continúa con veto)
- Reporte estructurado en timestamp + modo

**Debilidades a eliminar:**
- Invocación con `/audit` solo — múltiples entry points
- Modalidad fija de 4 modos — modos personalizables por proyecto
- Skills cargadas por nombre fijo — configurables en perfil
- Sin paralelismo — todo secuencial en el agente principal

**Patrón de generalización:**
```
La skill unificada absorbe este orquestador como entry point, pero con paralelismo.
Los modos quick/certify/review/full definen QUÉ sub-agentes lanzar.
Cada modo es un array de fases en el perfil YAML.
```

---

## Matriz de Solapamientos

| Área | full-review | certify | audit | Skill unificada |
|------|-------------|---------|-------|----------------|
| Compilación + lint + tests | F1 (cargo) | L1 (cargo) | P1 (cargo) | 1 sub-agente: Core Language |
| Security audit | Checklist F1 | — | P2 (skill) | 1 sub-agente: Security (vanta-audit) |
| Performance | Benchmarks F1 | — | P3 (skill) | 1 sub-agente: Performance (vanta-tuner) |
| Code review | Skills F1-F8 | L7 | P4 | 1 sub-agente: Code Review |
| CI/CD review | F5 | L6 parity | — | Fusión: 1 sub-agente |
| Docs review | F6 | L5 | — | 1 sub-agente (vanta-docs) |
| Design/UX | F7 | — | — | Solo si perfil incluye frontend |
| Architecture | F8 | — | — | 1 sub-agente |
| Hallazgos | F9 (12 cat.) | — | — | Consolidación post sub-agentes |
| Reporte | F10 | — | Reporte | Fase final del orquestador |
| Pre-push hook | — | Sí | — | Generación de hook por plataforma |
| CI/CD parity | — | L7a | — | Fase en perfil VantaDB |
| Modos | — | — | 4 modos | Entry point del orquestador |
| Scoring | ISO+Sonar+CII+OWASP+CC | — | — | Configurable en perfil |

---

## Arquitectura Propuesta de la Skill Unificada

```
┌────────────────────────────────────────────────────────────────────┐
│                   UNIFIED REVIEW & AUDIT SKILL                      │
│                     (OpenCode — sub-agentes paralelos)               │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  PROFILE SYSTEM (YAML externo)                                │  │
│  │  ├── profiles/default.yml  → generic project                  │  │
│  │  ├── profiles/vantadb.yml  → VantaDB overrides                │  │
│  │  └── profiles/<custom>.yml → cualquier proyecto               │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────── ENTRY POINTS ───────────────────────────────┐   │
│  │  skill unified-review    → carga y ejecuta con auto-detect   │   │
│  │  /review [mode] [--profile] → orquestador con sub-agentes    │   │
│  │  /audit                 → alias legacy (→ /review full)      │   │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌────── DETECTION ENGINE (agente principal, rápido) ─────────┐   │
│  │  1. git diff --name-only HEAD → archivos tocados            │   │
│  │  2. Detectar lenguaje: Cargo.toml / pyproject / package     │   │
│  │  3. Detectar bindings: maturin / wasm-pack / tsc            │   │
│  │  4. Detectar web: next/vite/astro config                    │   │
│  │  5. Detectar CI: .github/ / .gitlab-ci.yml                 │   │
│  │  6. Detectar docs: docs/ README CHANGELOG                   │   │
│  │  7. Cargar perfil (default + overrides de proyecto)         │   │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌────── FAN-OUT: LANZAR SUB-AGENTES EN PARALELO ─────────────┐  │
│  │                                                              │  │
│  │  task("core-check",         cmd, "vanta-worker") ──────┐    │  │
│  │  task("bindings-check",     cmd, "vanta-worker") ──────┤    │  │
│  │  task("web-check",          cmd, "vanta-worker") ──────┤──→ │  │
│  │  task("ci-cd-check",        cmd, "vanta-lead")   ──────┤    │  │
│  │  task("docs-check",         cmd, "vanta-docs")   ──────┤    │  │
│  │  task("arch-check",         cmd, "vanta-arch")   ──────┘    │  │
│  │                                                              │  │
│  │  (Opcional, modo full):                                      │  │
│  │  task("security-audit",     cmd, "vanta-audit")              │  │
│  │  task("performance-audit",  cmd, "vanta-tuner")              │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌────── FAN-IN: CONSOLIDACIÓN (agente principal) ────────────┐   │
│  │  Por cada sub-agente:                                       │   │
│  │  ├── ¿Falló? → si es crítico, abortar; si no, registrar    │   │
│  │  ├── Extraer findings del reporte JSON del sub-agente       │   │
│  │  └── Consolidar en tabla unificada                          │   │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌────── REPORTE FINAL ───────────────────────────────────────┐   │
│  │  docs/reviews/review-<mode>-<timestamp>.md                  │   │
│  │  - Executive Summary + Scoreboard por fase                  │   │
│  │  - Findings consolidados (severidad/categoría/esfuerzo)     │   │
│  │  - Recomendaciones priorizadas                              │   │
│  │  - Logs de sub-agentes linkeados (no inline)                │   │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

---

## Entregables

### 1. Archivo SKILL.md Final — `unified-review/SKILL.md`

**Este es el entregable principal.** Un solo archivo que:

- Cumpla con el formato de skills de **OpenCode** (investigar primero)
- Incluya frontmatter YAML con `name`, `description`, `compatibility: [opencode, claude-code]`
- Defina la arquitectura de sub-agentes paralelos con `task()` calls
- Tenga las fases configurables mediante perfil YAML externo
- Incluya ejemplos de uso para proyectos genéricos Y para VantaDB
- Funcione sin configuración inicial (modo por defecto detecta el proyecto solo)
- Taxonomía de hallazgos genérica (severidad, categoría, esfuerzo)
- Integración con Ponytail (respetar modo lazy del usuario)
- Integración con Campaign task system (trackear progreso con `campaign_*` tools)

**Estructura esperada del SKILL.md:**

```markdown
---
name: unified-review
description: >
  Universal project review, audit, and certification gate.
  Uses parallel sub-agents for scalability.
  Detects project language/tooling automatically.
compatibility: [opencode, claude-code]
---

# Unified Review & Audit

## Profile System
...

## Detection Engine
...

## Phase Definitions (sub-agent per phase)
...

## Fan-Out Orchestration
...

## Fan-In Consolidation
...

## Report Format
...

## Usage Examples
...
```

### 2. Perfiles YAML

Dos archivos de configuración:

- **`profiles/default.yml`** — Perfil genérico con:
  - Detección automática de lenguaje (Rust/Python/TS/Go)
  - Comandos estándar (cargo test, pytest, npm test, go test)
  - 5 fases base: Core, Dependencies, CI, Docs, Report
  - Scoring simplificado (sin ISO 25010)
  - Modos: quick (1 fase), certify (3 fases críticas), full (5 fases)

- **`profiles/vantadb.yml`** — Perfil VantaDB que hereda y extiende default.yml:
  - Paths de bindings: `vantadb-python/`, `vantadb-ts/`, `vantadb-wasm/`
  - Comandos específicos: `cargo nextest run --profile audit --workspace --build-jobs 2`
  - Umbrales de scoring ISO 25010 + SonarQube + CII + OWASP + CodeClimate
  - Skills adicionales: `security-and-hardening`, `performance-optimization`, `code-review-and-quality`
  - 4 modos: quick/certify/review/full (heredados de vantadb-audit)
  - Taxonomía de 12 categorías (heredada de full-review F9)
  - Sub-agentes específicos: vanta-audit para security, vanta-tuner para performance
  - Pre-push hook generator para PowerShell

### 3. Documento de Arquitectura (Markdown, opcional)

Si es necesario para claridad, incluir un breve `unified-review/ARCHITECTURE.md` con:

- Diagrama de flujo de sub-agentes
- Formato de intercambio (JSON entre sub-agente y orquestador)
- Estrategia de fallos y timeouts
- Budget de contexto estimado por fase

---

## Preguntas de Scoping (Respondidas)

### 1. ¿Propósitos iguales o adyacentes?

**Adyacentes.** Cada skill tiene un rol distinto:

| Skill | Rol | En skill unificada |
|-------|-----|-------------------|
| `full-review` | Profundidad + scoring | Modo `full` del perfil |
| `certify` | Gate pre-push | Modo `certify` del perfil |
| `audit` | Orquestador de modos | Entry point principal |

**Estrategia:** Orquestador con sub-commands y perfiles. La skill unificada absorbe `audit` como entry point, `full-review` como modo `full`, `certify` como modo `certify`. Todos con sub-agentes paralelos.

### 2. ¿Formato de la skill?

**Formato OpenCode** (investigar primero). Se espera algo como:

```yaml
---
name: unified-review
description: "..."
compatibility: [opencode, claude-code, github-copilot]
---
```

### 3. ¿Archivos a entregar?

| Archivo | Obligatorio | Propósito |
|---------|-------------|-----------|
| `unified-review/SKILL.md` | ✅ **Sí** | Skill instalable en OpenCode |
| `unified-review/profiles/default.yml` | ✅ **Sí** | Perfil genérico para cualquier proyecto |
| `unified-review/profiles/vantadb.yml` | ✅ **Sí** | Perfil VantaDB (hereda de default) |
| `unified-review/ARCHITECTURE.md` | ❌ Opcional | Documentación del diseño de sub-agentes |

---

## Flujo de Trabajo para la Herramienta

```
1. INVESTIGAR → Documentación de OpenCode skills (formato, estructura, convenciones)
2. INVESTIGAR → Mecanismo de task() / sub-agentes en OpenCode
3. INVESTIGAR → Campaing tools MCP para task tracking
4. INVESTIGAR → Integración con Ponytail
5. ANALIZAR → Las 3 skills adjuntas en profundidad
6. IDENTIFICAR → Todo el contenido VantaDB-specific (paths, comandos, configs)
7. DISEÑAR → Sistema de perfiles YAML
8. DISEÑAR → Detección automática de proyecto/lenguaje
9. DISEÑAR → Arquitectura de sub-agentes paralelos con fan-out/fan-in
10. DISEÑAR → Sistema de scoring genérico
11. GENERAR → profiles/default.yml
12. GENERAR → profiles/vantadb.yml
13. GENERAR → unified-review/SKILL.md
```

---

## Instrucciones Finales

1. **INVESTIGA** OpenCode primero — no asumas que usa el mismo formato que Claude Code. Usa `websearch` y `webfetch` para verificar.
2. **INVESTIGA** el sistema de sub-agentes (`task()` tool) en OpenCode — cómo lanzar, cómo recolectar resultados, cómo manejar timeouts.
3. **ANALIZA** los 3 SKILL.md adjuntos en profundidad — no te saltes ningún detalle.
4. **IDENTIFICA** TODO el contenido VantaDB-specific.
5. **DISEÑA** la abstracción genérica con perfiles YAML.
6. **IMPLEMENTA** el paralelismo con sub-agentes como eje central (no opcional).
7. **GENERA** los archivos listos para copiar/instalar.
8. **La skill debe funcionar sin configuración** — auto-detect debe cubrir el 80% de los casos.
9. **El perfil VantaDB debe ser 1:1 o superior** en capacidad vs las 3 skills originales.
10. **Fecha del documento:** 2026-07-26.
