# VantaDB — AGENTS.md

> **🛡️ Validation Rule:** Si no estás 100% seguro de una respuesta, análisis o decisión técnica, DEBES validar contra internet (`websearch`/`webfetch`). Para herramientas, librerías o APIs, la fuente de verdad es su documentación oficial o GitHub. No confíes en conocimiento interno del modelo si hay duda.

## CodeGraph

CodeGraph tiene un índice pre-construido del código fuente de VantaDB (7.3K símbolos, 24.7K edges). **Úsalo SIEMPRE antes de grep/find/Read** para preguntas estructurales.

### Guía de decisión

CodeGraph expone tools MCP individuales **y** una tool consolidada. Prefiere la consolidada para ahorrar tokens.

#### 🚀 Tool Consolidada (Recomendada)

`codegraph_explore` — búsqueda semántica + call paths + blast radius en 1 llamada. Reduce consumo de tokens hasta 60%.

| Situación | Qué usar |
|-----------|----------|
| Cualquier pregunta estructural | `codegraph_explore "lenguaje natural o símbolo"` |

#### 🔍 Tools Individuales (Legacy)

Si tu entorno (Cursor, Claude Code) prefiere llamadas específicas:

| Tool | Propósito |
|------|-----------|
| `codegraph_search` | Búsqueda FTS5 + semántica para localizar definiciones |
| `codegraph_callers` | Qué llama a una función específica |
| `codegraph_callees` | Qué llama una función desde dentro |
| `codegraph_files` | Mapa y jerarquía de directorios indexados |
| `codegraph_dependencies` | Árbol de importaciones entre módulos |
| `codegraph_status` | Estado del índice (archivos, nodos AST, errores) |

### Reglas

- **Confía en el resultado** — no re-verifiques con grep/read. El source que devuelve es idéntico byte por byte al del Read tool.
- **No uses grep para buscar definiciones** — CodeGraph ya las tiene indexadas.
- **Staleness**: si ves `⚠️ Pending sync:` tras una edición, lee el archivo directamente. El auto-sync tarda ~2s.
- **Sin `.codegraph/`** → ignora CodeGraph, usa herramientas normales.

### Ejemplos VantaDB

```
codegraph_explore "how does a search query reach StorageEngine"
codegraph_search "StorageEngine"
codegraph_callers "VantaEmbedded::connect"
```

### CI / Hooks Integration

| Script | Qué hace |
|--------|----------|
| `dev-tools/verify.ps1` | Pre-flight completa (fmt → clippy → nextest → Python) — incluye `codegraph affected` al inicio |
| `dev-tools/verify_changed.ps1` | **Quick verify**: usa `codegraph affected` para testear solo archivos impactados. Ideal para iteración rápida |
| `.git/hooks/pre-commit` | Muestra preview no-blocking de tests afectados por staged changes |
| `.git/hooks/pre-push` | Corre `verify.ps1` completo antes de cada push |

**Flujo típico local:**
```
git add .                                           # stage changes
# pre-commit hook muestra preview de tests afectados
dev-tools/verify_changed.ps1                       # quick check (~30s)
git commit -m "feat: ..."                           # commit
# pre-push hook corre verify.ps1 completo (~2-5min)
git push
```
## Understand-Anything

Understand-Anything produce un **knowledge graph LLM-powered** (1917 nodos, 1120 edges, 32 capas, 14 tour steps) en `.understand-anything/knowledge-graph.json`. Complementa a CodeGraph para preguntas arquitectónicas y narrativa humana.

### CodeGraph vs Understand-Anything — Guía de decisión

| Situación | Herramienta | Por qué |
|-----------|------------|---------|
| "¿Dónde está definida la función X?" | **CodeGraph** | Index pre-construido, respuesta en ms |
| "¿Qué llama a esta función?" | `codegraph_explore` | Call paths precisos, resuelve dispatch dinámico |
| "¿Qué se rompe si cambio X?" | `codegraph_explore "X"` | Blast radius vía código fuente |
| "¿Cómo está estructurada la arquitectura?" | **Understand-Anything** | 32 capas con descripciones narrativas |
| "Dame un tour del código base" | **Understand-Anything** | Tour guiado de 14 pasos desde entry point |
| "Explica este módulo en detalle" | `skill understand-explain` | Análisis narrativo contextual |
| "¿Qué tests ejecutar?" | `git diff --name-only \| codegraph_explore` | Conectado al git diff |
| "Onboarding para nuevo dev" | `skill understand-onboard` | Genera guía de onboarding interactiva |
| "¿Cuál es el dominio de negocio?" | `skill understand-domain` | Extrae flujos de dominio del grafo |
| "¿Qué cambió en este PR?" | `skill understand-diff` | Analiza diff contra el grafo existente |

**Regla general**: CodeGraph primero para todo lo que sea símbolos/código preciso. Understand-Anything para contexto arquitectónico, narrativa, onboarding y visualización.

### Slash Commands (Understand-Anything nativo)

El proyecto [Egonex-AI/Understand-Anything](https://github.com/Egonex-AI/Understand-Anything) expone estos comandos que el agente escribe directamente en la consola:

| Comando | Qué hace |
|---------|----------|
| `/understand` | Escanea repo, construye grafo en `.understand-anything/knowledge-graph.json` |
| `/understand --auto-update` | Activa hook post-commit para actualizaciones incrementales |
| `/understand --full` | Rebuild completo del grafo |
| `/understand-chat [pregunta]` | Chat contextualizado en la arquitectura del sistema |
| `/understand-dashboard` | Panel visual interactivo en navegador |
| `/understand-explain [ruta]` | Análisis aislado de un archivo específico |
| `/understand-diff` | Examina cambios staged/unstaged y predice impacto |
| `/understand-onboard` | Genera Guided Tours para onboarding |
| `/understand-domain` | Agrupa código por entidades de negocio |
| `/understand-knowledge [ruta]` | Analiza documentación Markdown externa |

### Alternativa: Agent Skills

Los skills en `C:\Users\Eros\.agents\skills\` envuelven la misma funcionalidad vía `skill <nombre>`:

| Skill | Comando OpenCode | Qué hace |
|-------|-----------------|----------|
| `understand` | `skill understand` | Pipeline completo: escanea, analiza y genera grafo |
| `understand-chat` | `skill understand-chat` | Chat contextual sobre el codebase |
| `understand-explain` | `skill understand-explain` | Explicación profunda de archivo/módulo |
| `understand-diff` | `skill understand-diff` | Analiza git diff contra el grafo |
| `understand-domain` | `skill understand-domain` | Extrae conocimiento de dominio de negocio |
| `understand-knowledge` | `skill understand-knowledge` | Analiza wikis Markdown → grafo |
| `understand-onboard` | `skill understand-onboard` | Guía de onboarding interactiva |
| `understand-dashboard` | `skill understand-dashboard` | Visor web interactivo del grafo |

### Estado actual

El grafo ya está generado en `.understand-anything/knowledge-graph.json`:

```
/understand --auto-update        # incremental post-commit
/understand --full               # rebuild completo
skill understand                 # misma funcionalidad vía skill
```

### Flujo recomendado: CodeGraph + Understand-Anything sin conflictos

1. **Para navegación diaria** → usa CodeGraph (`codegraph_explore`). Es más rápido, determinístico, y no gasta tokens LLM en re-análisis.
2. **Para entender arquitectura** → `/understand-chat "pregunta"` o `skill understand-chat`. El grafo ya existe, no necesita regenerarse.
3. **Para onboarding/review** → `/understand-explain` o `skill understand-explain`. Usan el grafo existente.
4. **Solo regenera si**: cambia la estructura del proyecto (nuevos módulos grandes) o quieres un análisis más fresco.
5. **NUNCA** ejecutes `/understand --full` a menos que sea necesario — el pipeline actual ya cubre 790 archivos y consumió ~158s de subagentes.

### Referencia del grafo

```json
{
  "nodes": [{"id": "file:src/engine.rs", "type": "file", "name": "engine.rs", "summary": "In-memory storage engine", "tags": ["storage", "core"]}],
  "edges": [{"source": "file:src/engine.rs", "target": "file:src/storage/mod.rs", "type": "imports", "direction": "directed", "weight": 0.7}],
  "layers": [{"id": "layer:core-engine", "name": "Core Engine", "description": "In-memory engine and storage backends", "nodeIds": ["file:src/engine.rs", ...]}],
  "tour": [{"order": 1, "title": "Project Overview", "description": "Start with README", "nodeIds": ["document:README.md"]}]
}
```

### Capas arquitectónicas (32 total)

Las principales: `core-engine`, `storage-backends`, `vector-index`, `web-frontend`, `python-bindings`, `typescript-sdk`, `integration-wrappers`, `dev-tooling`, `tests`, `documentation`, `ci-cd`, `wasm`, `enterprise`, `mcp`.

## Rust MCP Servers

Dos MCP servers para operaciones Rust (ver tabla completa en [MCP Servers Disponibles](#mcp-servers-disponibles)):

### Guía de uso para el agente

| Situación | Qué usar |
|-----------|----------|
| "Verifica que el código compile" | `cargo-mcp cargo_check` |
| "Ejecuta clippy" | `cargo-mcp cargo_clippy` |
| "Corre los tests" | `cargo-mcp cargo_test` |
| "Agrega la dependencia serde" | `cargo-mcp cargo_add` con `dependencies: ["serde"]` |
| "Formatea el código" | `cargo-mcp cargo_fmt_check` |
| "¿Qué símbolos hay en este archivo?" | `rust-analyzer-mcp rust_analyzer_symbols` con `file_path` |
| "Llévame a la definición de X" | `rust-analyzer-mcp rust_analyzer_definition` con `file_path`, `line`, `character` |
| "¿Qué errores tiene este archivo?" | `rust-analyzer-mcp rust_analyzer_diagnostics` con `file_path` |
| "Dame los errores de todo el workspace" | `rust-analyzer-mcp rust_analyzer_workspace_diagnostics` |
| "Refactoriza / renombra este símbolo" | `rust-analyzer-mcp rust_analyzer_code_actions` |

### Flujo recomendado

1. **Durante desarrollo**: usa `cargo-mcp` para build/test/clippy rápidos desde el chat
2. **Para navegación de código**: `rust-analyzer-mcp` es más preciso que grep para goto-def, references, hover
3. **Al finalizar**: corre `cargo fmt --check` + `cargo clippy` + `cargo nextest` via cargo-mcp antes de commit
4. **rust-mcp-server**: ignorar — no funcional y redundante

## Dev Tools (Instalados)

Herramientas de desarrollo instaladas globalmente para optimizar el workflow de un solo dev.

### Cargo Tools

| Herramienta | Instalada | Comando | Propósito |
|-------------|-----------|---------|-----------|
| **cargo-watch** | ✅ | `cargo watch -x check` | Feedback loop sub-second. Re-ejecuta comandos en cada cambio de archivo |
| **cargo-machete** | ✅ | `cargo machete` | Detecta dependencias no usadas |
| **cargo-bloat** | ✅ | `cargo bloat --crates` | Analiza qué engorda el binario release |
| **cargo-outdated** | ✅ | `cargo outdated` | Lista dependencias desactualizadas |
| **cargo-nextest** | ✅ | `cargo nextest run` | Test runner ~3× más rápido que cargo test |
| **cargo-deny** | ✅ | `cargo deny check` | Auditoría de licencias + advisory + bans |
| **cargo-audit** | ✅ | `cargo audit` | Security advisory checker |
| **release-plz** | ✅ | `release-plz release` | Automatiza bump de versiones, changelog, y publish |
| **git-cliff** | ✅ | `git-cliff -o CHANGELOG.md` | Generador de changelog desde conventional commits |

### Justfile

El **Justfile** en la raíz del proyecto es el reemplazo moderno de Makefile. Instalación: `cargo install just`

Comandos principales:

```bash
just check            # cargo check --workspace (feedback rápido)
just test             # cargo nextest run --profile audit
just verify           # fmt + clippy + test + deny (pre-flight completo)
just verify-quick     # dev-tools/verify_changed.ps1 (30s, CodeGraph-optimized)
just watch            # cargo watch -x check -x 'nextest run' (loop infinito)
just fmt-fix          # cargo fmt (aplica formato)
just machete          # cargo machete (deps no usadas)
just size             # cargo bloat --crates (tamaño binario)
just outdated         # cargo outdated (deps stale)
just audit            # cargo audit (seguridad)
just changelog        # git-cliff -o docs/CHANGELOG.md
just ci               # fmt + clippy + test + deny + audit (mismo orden que CI)
just certify          # nocturnal_suite.ps1 (certificación pesada local)
just release          # cargo build --release
just run-cli          # cargo run --features cli
just run-server       # cargo run --features server --bin vantadb-server
```

### Git Aliases

Configurados globalmente en `~/.gitconfig`:

| Alias | Comando real |
|-------|-------------|
| `git lg` | `log --oneline --graph --all --decorate` |
| `git st` | `status -sb` |
| `git ci` | `commit` |
| `git co` | `checkout` |
| `git br` | `branch` |
| `git rb` | `rebase -i` |
| `git up` | `push -u origin HEAD` |
| `git fixup` | `commit --fixup` |
| `git amend` | `commit --amend --no-edit` |
| `git undo` | `reset --soft HEAD~1` |
| `git unstage` | `reset HEAD --` |

### VS Code Setup

Archivos en `.vscode/`:

| Archivo | Propósito |
|---------|-----------|
| `extensions.json` | Recomienda rust-analyzer, CodeLLDB, crates, Even Better TOML, GitLens, cSpell, markdownlint, ShellCheck |
| `settings.json` | Config: rust-analyzer con clippy + features del proyecto, formatOnSave, exclude patrones |
| `tasks.json` | 10 tareas: check, clippy, nextest, fmt, deny, verify, build release, run cli/server |
| `mcp.json` | cargo-mcp + rust-analyzer-mcp para GitHub Copilot Chat |

### Dependabot

Configurado en `.github/dependabot.yml` para 4 ecosistemas:

| Ecosistema | Schedule | Límite PR |
|------------|----------|-----------|
| **Cargo** | Weekly (lunes) | 10 PRs |
| **npm (web/)** | Weekly (lunes) | 5 PRs |
| **GitHub Actions** | Weekly (lunes) | Ilimitado |
| **Docker** | Weekly (lunes) | Ilimitado |

Las PRs se agrupan por tipo (patch, minor) para reducir ruido.

### release-plz

Configurado en `release-plz.toml`. Automatiza:

1. Análisis de conventional commits desde el último tag
2. Bump semántico de versiones (feat → minor, fix → patch, breaking → major)
3. Actualización de `docs/CHANGELOG.md`
4. Creación de tag `v{{ version }}` en git
5. Publicación a crates.io (en orden de dependencias del workspace)

Uso: `release-plz release` (desde la rama main, después de mergear)

### CI: sccache

Agregado al workflow `ci-rust-10.yml` mediante `mozilla-actions/sccache-action`. Acelera compilaciones en CI reutilizando objetos compilados entre runs (~40-60% más rápido en rebuilds).

### Flujo diario recomendado

```bash
# Desarrollo iterativo
just watch-check                    # terminal 1: feedback instantáneo

# Antes de commit
just verify                         # fmt + clippy + test + deny

# Commit
git add -p && git ci -m "feat: ..."
git up

# Release (cuando toca)
release-plz release                 # bump + changelog + tag + publish
```

## Web Frontend (Vite + React + TanStack Router)

Stack: **Vite 8 + React 19 + TanStack Router v1 + GSAP 3.15 + Tailwind CSS v4**

### Estructura

```
web/
  src/
    routes/        ← 27 rutas TanStack (lazy-loaded)
    components/    ← nb/ (18 design system components) + compuestos (NbTrustBar, NbArchSection...)
    styles/        ← 46 CSS → 31 tras cleanup. nb-base.css (base + grid) + nb-components.css (componentes + utilitarias)
    lib/           ← gsap.ts (plugins registrados), utils.ts (cn)
    hooks/         ← useScrollReveal (IntersectionObserver + "is-visible")
```

### Stack decisions

| Decisión | Por qué |
|----------|---------|
| **Vite 8** | Última major, esbuild nativo, HMR instantáneo |
| **React 19** | Server Components no usados (SPA), pero aprovecha use() y mejoras de rendering |
| **TanStack Router v1** | Type-safe first class, lazy routes, search params |
| **GSAP 3.15** | ScrollTrigger + TextPlugin + DrawSVG registrados. Plugins gratuitos desde 2024 |
| **Tailwind v4** | CSS-first config (tokens.css importa tailwindcss). NO tailwind.config.js |
| **@tanstack/react-query** | Para fetching si se agrega API |
| **split-type** | Text reveal animations (hero, section headers) |
| **@observablehq/plot** | Benchmarks (lazy-load, ~45KB gzip) |
| **simple-icons** | Logos de tecnologías (tree-shakeable) |

### Design System (nb/)

18 componentes en `src/components/nb/`. Calidad promedio auditada: 7.9/10.

| Componente | Propósito |
|------------|-----------|
| NbSectionHeader | Hero + section titles con `nb-section-header` + `--bordered`/`--center` |
| NbCardFrame | Tarjetas con border + offset shadow (engine, architecture) |
| NbDitherImage | Imagen con filtro SVG dithering (about/team) |
| NbCursor | Cursor parpadeante terminal |
| NbSplitFlap | Efecto split-flap display |
| NbMarquee | Marquee horizontal infinito |
| NbFeatureGrid | Grid asimétrico (7fr-5fr) con iconos |
| NbPricingCard | Card de pricing con lista de features |
| NbFaqAccordion | Acordeón FAQ |
| NbTerminalBlock | Bloque de terminal con sintaxis |
| NbBenchmarkGrid | Grid de benchmarks |
| NbArchSection | Sección de arquitectura con spec table |
| NbDataTrust | Trust bar animado |
| NbEcosystem | Grid de ecosistema |

### Sistema de animación

- GSAP registrado en `src/lib/gsap.ts` (ScrollTrigger, TextPlugin, DrawSVGPlugin, useGSAP)
- ScrollTrigger para animaciones basadas en scroll (pin, scrub, reveal)
- TextPlugin para typewriter/heor text reveals
- DrawSVGPlugin para SVG draw animations
- `useScrollReveal` hook para reveal básico vía IntersectionObserver (clase `is-visible`)
- Animaciones existentes en varias rutas (engine, latency, hero)

### CSS Architecture

- **nb-base.css**: Reset, layout helpers (`.nb-section`, `.nb-grid`, `.nb-inner`), tipografía base (`.nb-title`, `.nb-sub`)
- **nb-components.css**: Componentes concretos (`.nb-card`, `.nb-btn`, `.nb-frame`, `.nb-bento`, `.nb-table`, `.nb-cmd-block`, `.nb-marquee`, `.nb-trust-*`, `.nb-metric-*`, `.nb-card-frame`, `.nb-num-marker`)
- **tokens.css**: Variables CSS + Tailwind v4 theme
- **index.css**: Entry point que `@import`a todos los CSS base
- Archivos de ruta: cada ruta lazy importa su propio CSS

### Patrones a seguir

- **Variantes de clase**: `nb-card--amber`, `nb-card--strong` (modificador BEM)
- **Estados**: Preferir data attributes (`[data-state="active"]`) sobre clases de estado
- **CSS Modules**: No usar. Preferir CSS plano con naming BEM + `cn()` para composición
- **Media queries**: Breakpoints: 960px (tablet), 768px (small tablet), 640px (mobile)
- **prefers-reduced-motion**: Siempre incluir en animaciones nuevas
- **Tailwind**: Solo para prototyping rápido. Preferir variables CSS + clases nb/ para producción

### Performance Budget

| Recurso | Límite actual | Target |
|---------|--------------|--------|
| Bundle JS (gzip) | ~150KB | <120KB |
| CSS (gzip) | ~25KB | <20KB |
| Fonts (gzip, 3 variables) | ~500KB | ~500KB (no cambiar) |
| GSAP (gzip) | ~100KB | ~100KB (necesario para animaciones existentes) |

### Contenido

- **NO** usar "ONNX", "Sled", "LangChain", "LlamaIndex" — ya no existen en el código real
- Stack real: **Rust 1.94**+ | **Python 3.11**+ | Fjall + RocksDB + InMemory backends
- Integraciones reales: CrewAI + DSPy + Haystack + Mem0 + OpenAI + Ollama + LiteLLM
- Versión: **0.2.0** (no 0.1.5)
- Embedding providers: OpenAI, Ollama, LiteLLM (no "any ONNX model")

## Skills Manifest

**Todas las skills están centralizadas en:**
- `.agents/skills/` (proyecto, 116 skills esenciales)
- Referencia completa en: `SKILLS-MANIFEST.md` (raíz del proyecto)

**Siempre preferir la copia del proyecto sobre la global.**
Para cargar: `skill <nombre>` o leer el SKILL.md correspondiente.

### Skill Loading Guide — Diseño & Creativo

- **Diseño UI/Frontend**: `vanta-design-orchestrator` → `impeccable` → `design-taste-frontend`
- **Animación**: `motion (motion.dev)` (preferido), `gsap-core` (alternativa GSAP)
- **Corrección de bugs**: `systematic-debugging` → `writing-plans`
- **Features multi-paso**: `brainstorming` → `writing-plans` → skills relevantes
- **SEO**: `ai-seo` → `seo-audit` → `audit-website`
- **Video/presentaciones**: `hyperframes` → deck skills según necesidad
- **Branding/Arte**: `brandkit`, `canvas-design`, `algorithmic-art`, `theme-factory`, `color-expert`, `platform-design`

### Skill Loading Guide — Ingeniería (agent-skills)

Skills de ingeniería instaladas desde [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) (`.agents/agent-skills/`). Son **workflows obligatorios** — el agente DEBE usarlos cuando apliquen. No saltarse pasos.

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
| **VERIFY** | `debugging-and-error-recovery` | Tests fallan, builds rotos, comportamiento inesperado |
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

**Personas especializadas** (`.agents/agents/`): `code-reviewer` (Staff Engineer), `test-engineer` (QA), `security-auditor` (Security), `web-performance-auditor` (Web Perf).

**Referencias** (`.agents/references/`): `definition-of-done.md`, `testing-patterns.md`, `security-checklist.md`, `performance-checklist.md`, `accessibility-checklist.md`, `observability-checklist.md`.

**Reglas:**
1. Antes de cualquier acción, evaluar qué skill de ingeniería aplica
2. Si aplica una skill, DEBE cargarse con `skill <nombre>` y seguirse exactamente
3. No implementar sin spec (para features nuevas) ni mergear sin review
4. No saltarse pasos con excusas — las skills tienen tablas anti-racionalización
5. Skills de diseño/creativo y de ingeniería son complementarias — ambas pueden aplicarse
6. **Relaciones, dependencias e implicaciones:** cada cambio DEBE analizar:

   ```
   1. USAR codegraph_explore para mapear callers/callees/blast radius del cambio
   2. IDENTIFICAR módulos aguas arriba (dependen de lo que cambia)
   3. IDENTIFICAR módulos aguas abajo (de los que depende el cambio)
   4. EVALUAR implicaciones: ¿rompe contratos existentes? ¿cambia comportamiento público?
      ¿afecta performance/memoria? ¿introduce nuevos errores? ¿require migración de datos?
   5. DOCUMENTAR hallazgos en el commit message o ADR
   ```

## Ritual de Inicio de Sesión (MUST DO)

Al empezar cada sesión, ejecutar en orden:

1. **Cargar skills base**:
   ```
   skill progreso                 # lee backlog, chequea WIP
   skill writing-plans            # si la tarea tiene múltiples pasos
   skill systematic-debugging     # si la tarea es corregir un bug
   ```
2. **Revisar estado del repo**:
   ```bash
   git status --short             # ¿hay cambios sin commit?
   git log --oneline -5           # ¿qué se hizo en la última sesión?
   ```
3. **Cargar skills adicionales** según el tipo de tarea (ver [Skill Loading Guide](#skill-loading-guide--diseño--creativo))
4. **Verificar entorno rápido**: solo si la tarea involucra cambios en infraestructura
   ```bash
   rustc --version && cargo --version
   just check                     # feedback rápido
   ```

Al **finalizar** la sesión:
```
skill progreso                   # mueve tareas completadas a docs/progreso/
ponytail-review                   # revisa over-engineering residual
just verify                       # fmt + clippy + test + deny (o just verify-quick)
```

## Ponytail — Lazy Senior Dev Mode

Integrado vía plugin OpenCode desde `~/.agents/ponytail/` (v4.8.4, MIT, 80k stars).

**Filosofía:** antes de escribir código, subir esta escalera y detenerse en el primer peldaño que aplica:

```
1. ¿Esto necesita existir?       → no: skip (YAGNI)
2. ¿Ya existe en el codebase?    → reusar, no reescribir
3. ¿Lo resuelve la stdlib?       → usarla
4. ¿Feature nativa del platform? → usarla
5. ¿Dependency ya instalada?     → usarla
6. ¿Se puede en una línea?       → una línea
7. Recién acá: el mínimo que funciona
```

**No recorta:** validación de trust boundaries, data-loss handling, seguridad, accesibilidad. Solo over-engineering.

### Comandos

| Comando | Qué hace |
|---------|----------|
| `/ponytail` | Reporta nivel actual |
| `/ponytail lite` | Moderado — solo corta lo obvio |
| `/ponytail full` | Default — escalera completa |
| `/ponytail ultra` | Máxima intensidad |
| `/ponytail off` | Desactivado |
| `/ponytail-review` | Revisa el diff actual por over-engineering |
| `/ponytail-audit` | Audita todo el repo |
| `/ponytail-debt` | Cosecha deuda técnica diferida |

### Modo default

El modo default es `full`. Se puede cambiar con `PONYTAIL_DEFAULT_MODE` (lite/full/ultra/off) o persistir con `/ponytail <nivel>`.

### Skills integradas

Las 6 skills de ponytail están disponibles como skills del proyecto:
- `ponytail` — lazy mode activo
- `ponytail-review` — revisión de diff
- `ponytail-audit` — auditoría completa
- `ponytail-debt` — deuda técnica
- `ponytail-gain` — scoreboard de impacto
- `ponytail-help` — referencia rápida

## Progreso Skill (MUST USE)

Load `progreso` at start and before completing every task:
- **Start**: `skill progreso` — reads backlog, checks for in-progress work
- **Complete**: `skill progreso` (Trigger 1) — moves done tasks from `docs/Backlog.md` → `docs/progreso/README.md` BEFORE any summary

## Reference Files

Archivos de referencia externos para no saturar este AGENTS.md. Son auto-contenidos, el agente los consulta solo cuando aplica el contexto.

| Archivo | Cuándo consultar | Cómo editar |
|---------|------------------|-------------|
| `docs/references/troubleshooting.md` | Error inesperado de compilación, test, Python SDK, web, git o herramienta en Windows | Agregar nuevo síntoma al final de la sección correspondiente con: síntoma, causa raíz, solución, comando exacto |
| `docs/references/bug-workflow.md` | Reporte de bug, test failure, comportamiento inesperado — antes de implementar cualquier fix | Modificar pasos si hay un patrón nuevo que documentar. NO cambiar las fases sin discusión |
| `docs/references/reading-nextest-output.md` | Falla de nextest, SLOW, LEAK, test flaky, o cualquier output de test runner | Agregar ejemplos de output con explicación si encuentras un patrón nuevo |

**Reglas:**
- NO leer estos archivos si no aplican al contexto actual
- Si lees un archivo para resolver un issue y la solución no está documentada, AGREGA la entrada faltante
- Si editas, mantener el mismo formato: tabla de secciones al inicio, bloques de código para comandos

## Doc Language Split

| Language | Content |
|----------|---------|
| **English** (source of truth) | `docs/api/`, `docs/architecture/`, `docs/operations/`, `docs/QUICKSTART.md` |
| **Spanish** (planning only) | `docs/VantaDB-MPTS/`, `docs/Backlog.md`, `docs/progreso/`, `docs/Investigaciones/` |

Technical docs stay in English. Never duplicate technical content in Spanish.

**Doc-Driven Development**: For new features, write/update `docs/api/` or `docs/operations/` docs FIRST, then implement. Never leave docs behind code.

## Pre-Flight Checks

```bash
:: Order matters — stop on first failure
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --profile audit --workspace --build-jobs 2
scripts/validate-docs-coverage.ps1   # final step before marking done

:: Quick local verify (CodeGraph-optimized, ~30s)
dev-tools/verify_changed.ps1
```

## Build System

- **Rust**: stable (rust-toolchain.toml: `1.94.1`+)
- **Profile `ci`** (no LTO, opt-level=2, 16 codegen-units) — used by CI Fast Gate
- **Profile `release`** (thin LTO, opt-level=3, 1 codegen-unit)
- **Profile `dev`** (opt-level=1, debug=0) — faster local iteration
- **Profile `test`** (opt-level=0, debug=0)
- **Profile `audit`** — used by nextest for pre-flight/release validation
- **Windows MSVC stack overflow workaround**: Always pass `--build-jobs 2` to nextest
- **Windows linker**: `.cargo/config.toml` forces `link.exe` (rust-lld causes STATUS_STACK_BUFFER_OVERRUN with large crates)

### Rust Build Optimization

`jobs = 2` en `.cargo/config.toml` es necesario por RAM (sin cambios de código posibles).
Estrategias para mantener `cargo check` rápido:

**Sin cambiar código (workflow):**

| Comando | Por qué es más rápido |
|---------|----------------------|
| `cargo check -p vantadb` | Solo la crate core, ignora las otras 15 del workspace |
| `cargo check -p vantadb -p vantadb-server -p vantadb-mcp` | Solo las 3 que tocas |
| `cargo check -p vantadb --no-default-features -F "fjall,cli"` | Excluye rocksdb, arrow, tantivy, server, prometheus |
| `cargo check -p vantadb` (sin flag) | El profile `check` nativo ya usa opt-level=0, debug=0, codegen-units=256 |
| `cargo check --timings -p vantadb` | Genera HTML con el desglose exacto de cada crate |
| `cargo check --workspace --exclude vantadb-langchain --exclude vantadb-ollama --exclude vantadb-openai --exclude vantadb-litellm --exclude vantadb-haystack --exclude vantadb-dspy --exclude vantadb-crewai --exclude vantadb-letta --exclude vantadb-mem0 --exclude vantadb-llamaindex` | Workspace completo sin los 10 adapters (cada uno tira pyo3) |

**Prioridad: `-p vantadb` es el que más impacto da.** Los adapters casi nunca cambian.

## Default Features

`cli` + `arrow` + `rocksdb` + `fjall` + `sysinfo` + `memmap2` + `fs2` + `prometheus` + `rayon` + `advanced-tokenizer`

Key optional features:
- `failpoints` — required for `chaos_integrity` test
- `remote-inference` — enables `llm` module (reqwest-based)
- `server` — enables axum HTTP server + tokio
- `python_sdk` — enables PyO3 bindings

## Test Suite

```bash
:: Fast Gate (audit profile)
cargo nextest run --profile audit --workspace --build-jobs 2

:: Single test (adapt to use nextest or cargo test as needed)
cargo nextest run --profile audit -p vantadb --test <test_name>

:: Tests that require specific features:
cargo nextest run --profile audit --features "failpoints" --test chaos_integrity
cargo nextest run --profile audit --features "cli" --test cli_tests
cargo nextest run --profile audit --features "arrow" --test columnar

:: Experimental tests (parser, executor, governor) — NOTE: experimental-lisp and experimental-governance deleted Jul 2026

:: Fuzzing (requires nightly + Linux, in fuzz/ dir excluded from workspace)
cd fuzz && cargo +nightly fuzz run fuzz_parser -- -max_total_time=300
```

Test categories: `tests/core/`, `tests/storage/`, `tests/logic/`, `tests/api/`, `tests/certification/`, `tests/memory/`, plus root-level integration tests.

## CI Architecture (Two-Tier)

1. **Fast Gate** (every PR/push): fmt, clippy, unit + fast integration tests. Must stay <5 min, deterministic, offline.
2. **Heavy Certification** (manual/scheduled): stress_protocol, hnsw validation, SIFT, competitive_bench, chaos_integrity, wal_resilience. Takes up to 2hrs. Never in Fast Gate.

See `docs/operations/CI_POLICY.md`.

## Python SDK

```bash
:: Hermetic venv (tests MUST use this — never a global install)
dev-tools/setup_venv.ps1         # creates target/audit-venv + maturin build
target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py -v

:: Editable install from source
pip install -e ./vantadb-python

:: PyPI name differs from import
pip install vantadb-py      # distribution
import vantadb_py            # module (underscore)
```

Built via `maturin` with PyO3. Requires Python ≥3.11.

## Architecture

```
vantadb/ (src/)            ← core library (primary crate)
  sdk/                     ← primary embedded API (VantaEmbedded, connect(), Vanta* types)
  engine.rs                ← in-memory engine
  storage/                 ← persistent backends (Fjall default, RocksDB fallback)
  wal.rs                   ← Write-Ahead Log
  vector/                  ← HNSW, distance metrics
  node.rs                  ← UnifiedNode, FieldValue
  cli.rs                   ← vanta-cli binary (#[cfg(feature = "cli")])
  api/                     ← HTTP routes (feature-gated, stub)
vantadb-python/            ← PyO3 bindings
vantadb-server/            ← standalone HTTP server binary
vantadb-wasm/              ← WASM build
vantadb-mcp/               ← MCP integration
vantadb-{openai,ollama,mem0,letta,crewai,dspy,haystack,litellm}/  ← thin integration crates
packages/                  ← LangChain + LlamaIndex adapter packages
fuzz/                      ← cargo-fuzz targets (nightly Linux only, excluded from workspace)
benches/                   ← Criterion benchmarks ([[bench]] in Cargo.toml)
```

## Key Conventions

- **Commit style**: Conventional Commits (`feat:`, `fix:`, `docs:`, `test:`, `perf:`) — see `.github/CONTRIBUTING.md`
- **Changelog**: `docs/CHANGELOG.md` via `git-cliff` (config: `cliff.toml`)
- **Licensing**: `cargo-deny` configured in `deny.toml` (MIT/Apache-2.0 only)
- **Markdown linting**: `.markdownlint-cli2.yaml` — line length disabled, HTML `div`/`h1`/`p`/`br` allowed
- **WASM**: vanta-wasm binary uses `opt-level = "s"` + strip in release
- **OpenCode MCP config**: `opencode.jsonc` at root (CodeGraph MCP server)
- **CodeGraph CI hooks**: verify.ps1/verify_changed.ps1 + pre-commit/pre-push hooks

## MCP Servers Disponibles

Configurados globalmente en `%USERPROFILE%\.config\opencode\opencode.json`.

### Activos

| MCP | Comando | Propósito |
|-----|---------|-----------|
| **CodeGraph** | `codegraph serve --mcp` | Grafo de conocimiento del código (7.3K símbolos). Resuelve símbolos, flujos, blast radius |
| **Pencil** | `mcp-server-windows-x64.exe` | Editor de archivos `.pen` — diseño UI visual, reemplazo de Figma |
| **Playwright** | `@playwright/mcp` | Automatización de navegador: navegar, click, screenshot, snapshot, redes |
| ~~**Recraft**~~ | ~~`@recraft-ai/mcp-recraft-server`~~ | ❌ Eliminado — sin API key |
| **cargo-mcp** | `cargo-mcp serve` | Ejecutar comandos Cargo: `check`, `clippy`, `test`, `build`, `fmt`, `add`, `remove`, `bench`, `run` |
| **rust-analyzer-mcp** | `rust-analyzer-mcp` | LSP completo: goto def, hover, references, completions, diagnostics, rename, format |
| ~~**rust-mcp-server**~~ | ~~`rust-mcp-server`~~ | ❌ Deshabilitado — bug MCP handshake en v0.2.4. Redundante: cargo-mcp + rust-analyzer-mcp cubren todo |

### Referencia rápida para agentes

- Para preguntas de código → **CodeGraph** (siempre primero, antes de grep/read)
- Para diseño UI/visual → **Pencil** (archivos `.pen`)
- Para web scraping/testing → **Playwright**
- Para generar/editar imágenes → ~~**Recraft**~~ (❌ sin API key — eliminado)
- Para tareas Rust → **cargo-mcp** (build/test/clippy/fmt), **rust-analyzer-mcp** (LSP: goto def, hover, diagnostics, completions)
