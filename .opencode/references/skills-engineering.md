# Skill Loading Guides

> Movido desde `.opencode/AGENTS.md` — referencia on-demand. Consultar cuando necesites decidir qué skill cargar. Si editas, actualiza también el puntero en AGENTS.md.

## Skill Loading Guide — Diseño & Creativo

- **Diseño UI/Frontend**: `vanta-design-orchestrator` → `impeccable` → `design-taste-frontend`
- **Animación**: `motion (motion.dev)` (preferido), `gsap-core` (alternativa GSAP)
- **Corrección de bugs**: `systematic-debugging` → `writing-plans`
- **Features multi-paso**: `brainstorming` → `writing-plans` → skills relevantes
- **SEO**: `ai-seo` → `seo-audit` → `audit-website`
- **Video/presentaciones**: `hyperframes` → deck skills según necesidad
- **Branding/Arte**: `brandkit`, `canvas-design`, `algorithmic-art`, `theme-factory`, `color-expert`, `platform-design`

## Skill Loading Guide — Ingeniería (agent-skills)

Skills de ingeniería instaladas desde [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) (`.opencode/`). Son **workflows obligatorios** — el agente DEBE usarlos cuando apliquen. No saltarse pasos.

**Lifecycle mapping (detección automática por contexto):**

| Fase | Skill | Cuándo se activa |
|------|-------|-------------------|
| **DEFINE** | `spec-driven-development` | Nueva feature, API, cambio significativo — escribe spec/PRD antes de código |
| **DEFINE** | `interview-me` | Requisitos ambiguos — extrae lo que el usuario realmente necesita |
| **DEFINE** | `idea-refine` | Concepto vago → propuesta concreta |
| **PLAN** | `planning-and-task-breakdown` | Spec listo → tareas pequeñas, verificables, con dependencias |
| **BUILD** | `incremental-implementation` | Implementar en slices verticales delgados (test → code → verify → commit) |
| **BUILD** | `test-driven-development` | Lógica nueva, bugs — Red-Green-Refactor, pirámide 80/15/5 |
| **BUILD** | `context-engineering` | Sesión nueva, tarea compleja — empaqueta contexto relevante para el agente |
| **BUILD** | `source-driven-development` | Decisiones de framework/library — verifica docs oficiales primero |
| **BUILD** | `doubt-driven-development` | Stakes altos (producción, seguridad) — verificación adversarial en contexto fresco |
| **BUILD** | `frontend-ui-engineering` | UI nueva o modificación en web/ |
| **BUILD** | `api-and-interface-design` | APIs, boundaries de módulos, interfaces públicas |
| **VERIFY** | `systematic-debugging` | Tests fallan, builds rotos, comportamiento inesperado — root cause first (Iron Law) |
| **VERIFY** | `browser-testing-with-devtools` | Depurar algo que corre en navegador (web/) |
| **REVIEW** | `code-review-and-quality` | Antes de mergear cualquier cambio — revisión en 5 ejes |
| **REVIEW** | `code-simplification` | Código funciona pero es más complejo de lo necesario |
| **REVIEW** | `security-and-hardening` | Input de usuario, auth, datos, integraciones externas |
| **REVIEW** | `performance-optimization` | Requisitos de performance o regresiones sospechadas |
| **SHIP** | `git-workflow-and-versioning` | Siempre — commits atómicos, trunk-based, ~100 líneas por cambio |
| **SHIP** | `ci-cd-and-automation` | CI/CD pipelines, Shift Left, feature flags |
| **SHIP** | `shipping-and-launch` | Antes de deploy — checklists, rollout gradual, rollback |
| **SHIP** | `documentation-and-adrs` | Decisiones arquitectónicas, cambios de API, features nuevas |
| **SHIP** | `deprecation-and-migration` | Remover sistemas viejos, migrar usuarios, sunset features |
| **SHIP** | `observability-and-instrumentation` | Telemetría, logging estructurado, métricas RED |
| **META** | `using-agent-skills` | Cómo usar este pack — consultar si hay dudas |

**Personas especializadas** (`.opencode/agents/`): `vanta-audit` (Security + Code Review), `vanta-chaos` (Fuzzing + Resilience), `vanta-tuner` (Performance + Observability). Legacy `code-reviewer`, `security-auditor`, `test-engineer` eliminados — reemplazados por vanta-*.

**Referencias** (`.opencode/references/`, **12 total**): `definition-of-done.md`, `testing-patterns.md`, `security-checklist.md`, `performance-checklist.md`, `accessibility-checklist.md`, `observability-checklist.md`, `orchestration-patterns.md`, `REFERENCE-SYNTHESIS.md`, `awesome-harness-engineering/`, `darwin-godel-machine/`, `deepclaude/`, `statewright/`.
