# VantaDB — AGENTS.md

> **🛡️ Validation Rule:** Si no estás 100% seguro de una respuesta, análisis o decisión técnica, DEBES validar contra internet (`websearch`/`webfetch`). Para herramientas, librerías o APIs, la fuente de verdad es su documentación oficial o GitHub. No confíes en conocimiento interno del modelo si hay duda.

## Manual de Operación

Todo el detalle del sistema de tareas, agentes, skills, MCP servers, y su integración está en:

📖 **`.opencode/VANTADB-OPERATING-MANUAL.md`** — Manual de Operación completo (917 líneas, 14 secciones)

📖 **`SKILLS-MANIFEST.md`** — Catálogo completo de las 111 skills del proyecto (82 + 29, raíz)

Consultar para: entender cómo se relacionan los componentes del sistema, flujos de integración, troubleshooting, y reglas avanzadas.

## Pipeline & Command System

El sistema de pipeline vive en `.opencode/` y se activa cuando el usuario envía un comando.

### Entry Points

| Mensaje del usuario | Archivo a leer | Modos |
|---|---|---|
| `/pipeline ...` | `.opencode/commands/pipeline.md` | Plan / Task / Run / Interactive / Pipeline / Ejecución |
| `/audit ...` | `.opencode/commands/audit.md` | Full / Quick / Certify / Review |
| `/build ...` | `.opencode/commands/build.md` | Default / Auto / Prove |
| `/ship` | `.opencode/commands/ship.md` | Fan-out GO/NO-GO |
| `/rollback` | `.opencode/commands/rollback.md` | Revertir ship fallido |
| `/status` | `.opencode/commands/status.md` | Dashboard de un vistazo |
| `/backlog` | `.opencode/commands/backlog.md` | Revisar backlog + listar tareas activas + recomendar prioridad |
| `/spec` | `.opencode/commands/spec.md` | Spec-Driven Development |
| `/webperf` | `.opencode/commands/webperf.md` | Web performance audit |
| `/code-simplify` | `.opencode/commands/code-simplify.md` | Simplify code |

### Path Resolution

Todas las rutas relativas en comandos y prompts se resuelven así:

| Referencia en el archivo | Resuelve a |
|---|---|
| `prompts/X.md` | `.opencode/task-system/prompts/X.md` |
| `skills/X` | Buscar ↓ en orden: `.opencode/skills/X/` → `.agents/skills/X/` → `~/.agents/skills/X/` (usar la primera que exista; preferir la copia del proyecto → `.opencode/` sobre `.agents/` sobre global) |
| `tasks/<ID>.md` | `.opencode/skills/campaign-executor/tasks/<ID>.md` (si no existe: `.opencode/skills/campaign-executor/tasks/complete/<ID>.md` → `.opencode/skills/campaign-executor/tasks/closed/<ID>.md`) |
| `docs/plans/X.md` | `docs/plans/X.md` (ruta directa) |

**Nota:** aunque la regla de resolución para `tasks/<ID>.md` arranca en la raíz (`tasks/`), las tareas **completadas** viven en `tasks/complete/` y las **cerradas sin resolver** en `tasks/closed/`. Un `grep` o `Read` sobre la raíz sola se pierde esas. Buscar en los tres niveles cuando el ID no aparezca en la raíz.

### Cómo ejecutar un comando

1. **Detectar:** el usuario manda un mensaje que coincide con un patrón de comando (`/pipeline task P1-2`, `/audit quick`, `/status`, etc.)
2. **Leer entry point:** el agente LEE el archivo de comando correspondiente en `.opencode/commands/` con Read tool
3. **Interpretar modo:** el archivo describe cómo rutear según el argumento (plan / task / run / etc.)
4. **Resolver rutas:** cada referencia a `prompts/X.md`, `skills/X`, `tasks/X.md` se resuelve según la tabla de Path Resolution
5. **Cargar prompts:** cada prompt listado se carga con Read tool y se ejecuta secuencialmente
6. **Seguir sin saltar:** los prompts son instrucciones activas — seguir fases y pasos en orden
7. **Handoff:** al finalizar, escribir recitation y detenerse (no continuar a la siguiente tarea sin que el usuario lo pida)

### Skills base para pipeline

| Skill | Cuándo cargar |
|---|---|
| `progreso` | Al inicio de sesión y al completar cada tarea |
| `ponytail (full)` | Siempre activo (modo lazy default) |
| `campaign-executor` | En modo task / run (no es skill instalado, leer `.opencode/skills/campaign-executor/SKILL.md`) |

## CodeGraph

Índice pre-construido del código de VantaDB (7.3K símbolos, 24.7K edges). **Úsalo SIEMPRE antes de grep/find/Read** para preguntas estructurales.

- **Tool consolidada**: `codegraph_explore "pregunta o símbolo"` — búsqueda + call paths + blast radius en 1 llamada (hasta 60% menos tokens).
- **Tools individuales (legacy, Cursor/Claude Code)**: `codegraph_search`, `codegraph_callers`, `codegraph_callees`, `codegraph_files`, `codegraph_dependencies`, `codegraph_status`.
- **Reglas**: confía en el resultado (no re-verifiques); no uses grep para definiciones; si ves `⚠️ Pending sync:` lee el archivo directo (~2s); sin `.codegraph/` → usa herramientas normales.
- **CI/Hooks**: `dev-tools/verify.ps1` (pre-flight), `dev-tools/verify_changed.ps1` (quick ~30s). Pre-push barrier instalado como `.githooks/pre-push` (automático, Regla 1).

Flujo típico: `git add .` → `dev-tools/verify_changed.ps1` → `git commit` → `dev-tools/verify.ps1` → `git push`

## Understand-Anything

> Ver `references/understand-anything.md` — knowledge graph, capas arquitectónicas, decisión de uso vs CodeGraph, slash commands y flujo de consulta. Editar allá, no aquí.

## Rust MCP Servers

**Deshabilitados por default** (ahorro de contexto, preferencia del usuario): el agente usa la terminal para operaciones Rust, no los MCPs de cargo/rust-analyzer. Para reactivarlos puntualmente, ver [MCP Servers Disponibles](#mcp-servers-disponibles).

```bash
cargo check -p vantadb           # compilar
cargo clippy -p vantadb          # lint
cargo nextest run --profile audit --workspace --build-jobs 2   # tests
cargo add serde                  # dependencias
```

## Dev Tools (Instalados)

> Ver `references/dev-tools.md` — cargo tools, justfile, git aliases, VS Code setup, dependabot, release-plz, CI sccache y flujo diario. Editar allá, no aquí.

## Web Frontend (Next.js 16 + shadcn/ui + framer-motion)

> Ver `references/frontend-web.md` — estructura, stack decisions, design system, i18n, animación y contenido. Editar allá, no aquí.

## Skills Manifest

**Todas las skills están centralizadas en:**
- `.agents/skills/` (proyecto, 82 skills) + `.opencode/skills/` (29 skills)
- Referencia completa en: `SKILLS-MANIFEST.md` (raíz del proyecto)

**Siempre preferir la copia del proyecto sobre la global.**
Para cargar: `skill <nombre>` o leer el SKILL.md correspondiente.

### Skill Loading Guide

> Ver `references/skills-engineering.md` — guías de carga por categoría (diseño, corrección de bugs, features multi-paso, SEO, video). Editar allá, no aquí.

### Anti-Rationalization (MUST)

Las siguientes excusas son incorrectas y DEBEN ser ignoradas:

- "Esto es muy chico para una skill"
- "Yo lo implemento rápido, no necesito spec/test/review"
- "Ya conozco el código, no hace falta cargar skills"
- "Después agrego los tests"

**Comportamiento correcto:** siempre evaluar qué skills aplican y cargarlas antes de cualquier acción. Si hay duda, cargar la skill igual.

### Regla 0: Análisis de Impacto Antes de Modificar/Eliminar (MUST)

**Nunca modifiques o elimines un archivo sin antes:**

1. **Leer su contenido completo**
2. **Mapear todas sus referencias** — grep por el nombre del archivo en TODO el workspace (no solo `.opencode/`)
3. **Mapear qué archivos referencia hacia afuera** — imports, includes, depends
4. **Evaluar impacto** — ¿qué se rompe si el archivo desaparece? ¿qué depende de él?
5. **Presentar hallazgos al usuario** con: contenido, referencias entrantes, referencias salientes, veredicto propuesto

**Prohibido:** eliminar basado en "parece duplicado" o "no se usa (sin grep)". Siempre verificar con grep primero.

### Orchestration: Personas, Skills, Commands

Tres capas componibles con roles distintos:

- **Skills** (`.agents/skills/<name>/SKILL.md`) — workflows con pasos y criterios de salida. El *how*. Obligatorios cuando un intento matchea.
- **Personas** (`.opencode/agents/<role>.md`) — roles con perspectiva y formato de output. El *who*.
- **Comandos** (`.opencode/commands/*.md`) — entry points del usuario. El *when*. Capa de orquestación.

**Regla de composición:** el usuario (o un comando) es el orquestador. Las personas pueden invocar skills. El único patrón multi-persona endorsed es **parallel fan-out con merge step**.

**Excepción asimétrica:** algunas personas *sí* invocan otras personas vía `task` tool, pero solo en una dirección:
- **Orquestadores** (vanta-arch, vanta-worker, vanta-lead, vanta-engine) pueden invocar especialistas
- **Especialistas** (vanta-audit, vanta-chaos, vanta-tuner, vanta-docs) tienen `task: * deny` — son leaf nodes
- Esto refleja el patrón real: el que diseña/implementa delega al que revisa/testea/optimiza, nunca al revés. La regla general ("personas no invocan otras personas") aplica estrictamente a especialistas.

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

### Límites de herramientas por rol

> Fuente: `docs/Investigaciones/2026-08-10-agent-engineering/agent-03-orchestration.md` §9.2 — *worker = solo tools de su dominio; orquestador = delegación + verificación; evaluador = verificación, nunca implementa*. Política objetivo (TSYS-11): **ningún sub-agente escala a tools del lead**. Estado actual: los `permission:` de `.opencode/agents/*.md` otorgan todo `allow` (deuda a corregir al implementar TSYS-11); la tabla es el contrato de referencia.

Leyenda: ✅ permitido · ⚠️ solo uso read-only / delimitado a su dominio · ❌ prohibido.

| Rol (archivo) | File Read (read/glob/grep/list) | File Edit (edit) | Bash (build/test/bench) | Git push/commit/release | codegraph\* (intel) | cargo-mcp\* / rust-analyzer-mcp\* | Web search (webfetch/websearch/metasearchmcp/argus) | campaign\* (task system) | task (delegar sub-agentes) | Extras (playwright/discord/lottiefiles-creator/pencil) |
|---|---|---|---|---|---|---|---|---|---|---|
| **vanta-lead** (mode: all) | ✅ | ✅ | ✅ | ✅ **único rol que hace git push/commit/release** | ✅ | ✅ | ✅ | ✅ | ✅ (solo `vanta-*`) | ✅ |
| **vanta-arch** | ✅ | ✅ (diseño/arquitectura) | ✅ | ❌ | ✅ | ✅ | ⚠️ research | ✅ | ✅ (solo `vanta-*`) | ❌ |
| **vanta-worker** | ✅ | ✅ (código core/bindings) | ✅ | ❌ | ✅ | ✅ | ⚠️ research | ✅ | ✅ (solo `vanta-*`) | ❌ |
| **vanta-engine** | ✅ | ✅ (índices/algoritmos) | ✅ | ❌ | ✅ | ✅ | ⚠️ research | ✅ | ✅ (solo `vanta-*`) | ❌ |
| **vanta-audit** (leaf) | ✅ | ⚠️ solo notas/reportes de auditoría, **nunca fix** | ⚠️ read-only (cargo check/clippy/test) | ❌ | ✅ | ✅ | ⚠️ CVE lookup | ⚠️ solo reportar verdict | ❌ | ❌ |
| **vanta-chaos** (leaf) | ✅ | ✅ (solo scripts fuzz/estrés de su dominio) | ✅ (fuzzers, kill, stress) | ❌ | ✅ | ✅ | ⚠️ | ⚠️ | ❌ | ❌ |
| **vanta-tuner** (leaf) | ✅ | ✅ (solo telemetría/bench) | ✅ (bench/profile) | ❌ | ✅ | ✅ | ⚠️ | ⚠️ | ❌ | ❌ |
| **vanta-docs** (leaf) | ✅ | ✅ (solo docs/docstrings) | ⚠️ read-only (cargo doc/test --doc, pytest) | ❌ | ✅ | ⚠️ solo doc/test | ✅ | ⚠️ | ❌ | ⚠️ (pencil — suyo; resto ❌) |
| **vanta-review** (leaf) | ✅ | ⚠️ solo notas de review, **nunca implementa** | ⚠️ read-only (verificar) | ❌ | ✅ | ⚠️ solo test/check | ⚠️ | ⚠️ (verdict approve/changes) | ❌ | ❌ |

**Reglas de enforcement:**
1. **Solo vanta-lead** toca git mutating (`git commit`, `git push`, tags, release-plz) y packaging/publish (cargo publish, pip, npm). El resto corre git solo en modo lectura (status/log/diff).
2. **Workers** (arch/worker/engine): edit + bash limitados a su dominio de implementación; cualquier commit lo prepara el worker y lo ejecuta el lead.
3. **Especialistas leaf** (audit/chaos/tuner/docs/review): `task: deny` ya enforced en sus archivos; su edit queda restringido a entregables de su dominio (reportes, scripts de fuzz, docs) — nunca código core.
4. **Review nunca implementa**: vanta-review solo verifica y emite verdict; si encuentra fixes, los delega al worker.
5. Los `permission:` reales de `.opencode/agents/*.md` deben alinearse a esta tabla al implementar TSYS-11 (hoy todos otorgan `allow` amplio).

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
3. **Cargar skills adicionales** según el tipo de tarea (ver `references/skills-engineering.md`)
4. **Verificar entorno rápido**: solo si la tarea involucra cambios en infraestructura
   ```bash
   rustc --version && cargo --version
   just check                     # feedback rápido
   ```
5. **Validar feature stack** (si la sesión toca Rust):
   ```bash
   cargo check --no-default-features --features fjall   # feature set mínimo compila
   ```

Al **finalizar** la sesión:
```
skill progreso                   # mueve tareas completadas a docs/progreso/
ponytail-review                   # revisa over-engineering residual
just verify                       # fmt + clippy + test + deny (o just verify-quick)
```

## Ponytail — Lazy Senior Dev Mode

Integrado vía plugin OpenCode desde `~/.agents/ponytail/` (v4.8.4). **Modo default: `full`** (persistir con `PONYTAIL_DEFAULT_MODE`).

| Comando | Efecto |
|---------|--------|
| `/ponytail [lite\|full\|ultra\|off]` | Cambiar nivel de la escalera YAGNI→reusar→stdlib→nativo→dep→una línea→mínimo |
| `/ponytail-review` | Revisa el diff por over-engineering |
| `/ponytail-audit` | Audita el repo completo |
| `/ponytail-debt` / `/ponytail-gain` / `/ponytail-help` | Deuda diferida / impacto / referencia |

Regla: no simplificar trust boundaries, seguridad, accesibilidad, ni lo pedido explícitamente. Código primero, explicación ≤3 líneas.

## Progreso Skill (MUST USE)

Load `progreso` at start and before completing every task:
- **Start**: `skill progreso` — reads backlog, checks for in-progress work
- **Complete**: `skill progreso` (Trigger 1) — **elimina la fila** de `docs/Backlog.md` (no tacharla; el registro de completado vive en `docs/progreso/README.md`) y migra a `docs/progreso/README.md` BEFORE any summary; items removidos sin completar van a `docs/progreso/BACKLOG_HISTORY.md`. Plans completados → `docs/plans/archive/`

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

## Reglas del Proyecto (`.opencode/rules/`) — LAZY-LOADING OBLIGATORIO

Las reglas normativas por **área del sistema** viven en `.opencode/rules/`. Son reglas duras (must/must-not/por-qué), separadas del material de referencia. El índice y el instructivo de formato están en `.opencode/rules/README.md`.

**CUÁNDO LEER:** antes de **crear, editar, modificar, mejorar o borrar** código dentro de un área, carga el archivo de reglas de ESA área. Ejemplos de disparadores:

| Si vas a tocar... | Archivo de reglas a leer |
|---|---|
| WAL, storage/engine, backends, vfile, gc, lsm, schema, migration | `.opencode/rules/durability.md` |
| Índices vectoriales / text_index / tokenizer / vector quantization | `.opencode/rules/indexes.md` |
| Cualquier `async`/Tokio, `spawn_blocking`, mutexes, semáforos, ingestion async | `.opencode/rules/concurrency-async.md` |
| `node.rs`, `engine.rs`, `config.rs`, `error.rs`, parser, planner, executor | `.opencode/rules/core-engine.md` |
| `sdk/`, API pública, `VantaError`, semver, compat de bindings | `.opencode/rules/api-contract.md` |
| `vantadb-python/`, providers | `.opencode/rules/python-bindings.md` |
| `vantadb-server/`, `vantadb-mcp/` | `.opencode/rules/server-mcp.md` |
| `vantadb-wasm/`, `vantadb-ts/`, `vantadb-node/` | `.opencode/rules/js-ecosystem.md` |
| `web/` (Next.js, Tailwind, motion, i18n) | `.opencode/rules/frontend-web.md` |
| release, versionado, CI, changelog, publish de cualquier crate | `.opencode/rules/release-ci.md` |

**Cómo usarlas:**
- Leer SOLO el archivo del área tocada; no cargar toda la carpeta (lazy-loading por contexto).
- El contenido es **obligatorio** — no se puede saltar ni relajar una regla.
- Si una regla no aplica a la sub-tarea exacta, ignorarla sin borrarla.
- Si vas a AÑADIR/MODIFICAR una regla: seguir `.opencode/rules/README.md` → "Reglas para las reglas" (formato de cabecera, Must/Must-not/Por-qué, status, sin solapamiento con `references/`, `skills/`, `AGENTS.md`, ADRs).

## Doc Language Split

| Language | Content |
|----------|---------|
| **English** (source of truth) | `docs/api/`, `docs/architecture/`, `docs/operations/`, `docs/QUICKSTART.md` |
| **Spanish** (planning only) | `docs/Backlog.md`, `docs/progreso/`, `docs/Investigaciones/` |

Technical docs stay in English. Never duplicate technical content in Spanish.

**Doc-Driven Development**: For new features, write/update `docs/api/` or `docs/operations/` docs FIRST, then implement. Never leave docs behind code.

## Pre-Flight Checks

```bash
:: Full check (6 steps, ~2-5min)
dev-tools/verify.ps1

:: Quick check (3 steps, ~30s)
dev-tools/verify_changed.ps1
```

## Build System

> Ver `references/build-system.md` — build optimizado, profile, features por defecto y Rust Build Optimization. Editar allá, no aquí.

## Default Features

`cli` + `arrow` + `fjall` + `roaring` + `advanced-tokenizer` + `memmap2` + `fs2` + `sysinfo` + `rayon`

(`rocksdb` y `prometheus` NO están en default — activarlos opt-in cuando se necesiten.)

Key optional features:
- `failpoints` — required for `chaos_integrity` test
- `remote-inference` — enables `llm` module (reqwest-based)
- `server` — enables axum HTTP server + tokio
- `python_sdk` — enables PyO3 bindings

## Test Suite

> Ver `references/test-suite.md` — runner, categorías y comandos de test. Editar allá, no aquí.

## CI Architecture (Two-Tier)

1. **Fast Gate** (every PR/push): fmt, clippy, unit + fast integration tests. Must stay <5 min, deterministic, offline.
2. **Heavy Certification** (manual/scheduled): stress_protocol, hnsw validation, SIFT, competitive_bench, chaos_integrity, wal_resilience. Takes up to 2hrs. Never in Fast Gate.

See `docs/operations/CI_POLICY.md`.

## Python SDK

> Ver `references/python-sdk.md` — instalación, uso y bindings PyO3. Editar allá, no aquí.

## Architecture

> Ver `references/architecture.md` — estructura del workspace, invariantes y convenciones clave. Editar allá, no aquí.

## Key Conventions

- **Commit style**: Conventional Commits (`feat:`, `fix:`, `docs:`, `test:`, `perf:`) — see `CONTRIBUTING.md`
- **Changelog**: `docs/CHANGELOG.md` via `git-cliff` (config: `cliff.toml`)
- **Licensing**: `cargo-deny` configured in `deny.toml` (MIT/Apache-2.0 only) — applies to the Apache-2.0 core only; `vantadb-pro` proprietary is excluded
- **Markdown linting**: `.markdownlint-cli2.yaml` — line length disabled, HTML `div`/`h1`/`p`/`br` allowed
- **WASM**: vanta-wasm binary uses `opt-level = "s"` + strip in release

## Open Core Licensing (Model)

VantaDB follows an **Open Core** model (decision 2026-08-06, see `docs/plans/archive/2026-08-06-oc-vantadb-pro.md` and `.opencode/rules/open-core-licensing.md`):

- **Core `vantadb` stays Apache-2.0** — never relicensed, never gains Pro-only features.
- **`vantadb-pro`** (commercial Pro/Enterprise) lives in a **separate private repo** (`C:\Users\Eros\VantaDB Proyect\vantadb-pro`), **NOT** a workspace member. Never add it to `Cargo.toml` `[workspace] members`/`default-members`.
- **Delivery is compiled artifacts only** (private registry / signed on-prem `vantadb.license`), never source. Each Pro feature validates its license offline (expiry + max nodes).
- `cargo-deny` (`deny.toml`, MIT/Apache-2.0) gates the core only; Pro is excluded.
- Full normative rules: `.opencode/rules/open-core-licensing.md`.
- **OpenCode MCP config**: `opencode.jsonc` at root (CodeGraph MCP server)
- **CodeGraph CI hooks**: verify.ps1/verify_changed.ps1 (invocados por los hooks git instalados — `.githooks/pre-commit`/`.githooks/pre-push`, ver Regla 1)

## MCP Servers Disponibles

Config: activos en `opencode.jsonc` (proyecto); deshabilitados en `%USERPROFILE%\.config\opencode\opencode.json` (global, todos los proyectos). **OpenCode no filtra MCPs por agente** — los MCPs deshabilitados no cargan sus tools (ahorra contexto). Reactivar puntualmente: `"<mcp>": { "enabled": true }` en la config global + reiniciar OpenCode.

### Activos

| MCP | Comando | Propósito |
|-----|---------|-----------|
| **CodeGraph** | `codegraph serve --mcp` | Grafo de conocimiento del código (7.3K símbolos). Resuelve símbolos, flujos, blast radius |
| **Campaign** | `bun .opencode/task-system/mcp/campaign-server.mjs` | Task system: 30+ tools para plan, task, verify, state machine |
| **MetaSearchMCP** | `metasearchmcp-mcp` | Búsqueda multi-provider: web, GitHub, académico, código. DuckDuckGo gratis |
| **Argus** | `argus mcp serve` | 14 providers, extracción 12-step, dead URL recovery |

### Deshabilitados (default)

| MCP | Estado | Por qué |
|-----|--------|---------|
| **Pencil** | ❌ off | Editor `.pen` — solo cuando se trabaja en diseño UI |
| **Playwright** | ❌ off | Navegador — solo para testing web/devtools |
| **Discord** | ❌ off | Integración social no usada |
| **LottieFiles Creator** | ❌ off | Animaciones Lottie — solo cuando aplica |
| **cargo-mcp** | ❌ off | Terminal preferida para Rust (ver [Rust MCP Servers](#rust-mcp-servers)) |
| **rust-analyzer-mcp** | ❌ off | Terminal preferida para Rust |
| ~~**rust-mcp-server**~~ | ❌ off | Bug MCP handshake v0.2.4. Redundante con cargo-mcp + rust-analyzer-mcp |
| ~~**Recraft**~~ | ❌ eliminado | Sin API key |

### Referencia rápida para agentes

- Para preguntas de código → **CodeGraph** (siempre primero, antes de grep/read)
- Para tareas Rust → terminal (ver [Rust MCP Servers](#rust-mcp-servers))

## VantaDB Development Protocol & AI Guardian Rules

Como agente de IA asistiendo en VantaDB, DEBES auditar el código y las peticiones basándote estrictamente en las siguientes reglas. Cuestiona cualquier desviación, no asumas que el código es correcto y corrige malas prácticas de forma directa.

### Regla 1: Pre-push Gate Estricto

NUNCA sugieras mergear a `main` o pushear código sin antes ejecutar el pipeline local de certificación.

**Prohibido `--no-verify`**: aunque los hooks git se pueden saltar con `--no-verify` (o no estén reinstalados), la verificación previa es obligatoria antes de push. Si `dev-tools/verify.ps1` falla (tests, clippy, fmt, deny), NO se puede pushear saltándolo. Hay que arreglar el error y reintentar hasta que pase. Error → arreglar → reintentar, tantas veces como sea necesario. `--no-verify` solo se permite si el usuario lo ordena explícitamente.

| Si el usuario hace... | Debes responder... |
|---|---|
| `git push` o `git merge` | "¿Ya ejecutaste `dev-tools/verify.ps1` (build sin warnings, `cargo nextest --profile audit`, `cargo clippy --deny warnings`, `cargo fmt --check`)?" |
| Modifica `src/` o bindings | Recordar que `just verify` cubre fmt + clippy + test + deny |
| Salta la verificación "porque es un cambio chico" | Bloquear: "El pre-push gate corre en CI aunque sea 1 línea. Lo barato es verificarlo local." |

### Regla 2: Tolerancia Cero a Flaky Tests e Ignorancia de Errores

**Prohibición absoluta:** Está estrictamente prohibido sugerir o escribir `continue-on-error: true` en cualquier GitHub Action nuevo o existente (se heredan 7 instancias, todas con `# CATEGORY:` explícita). Ver taxonomía en `docs/operations/CI_POLICY.md` (secciones EXPERIMENTAL / BEST-EFFORT / NON-CRITICAL / INFORMATIONAL). Cualquier nueva exención requiere justificación + CATEGORY tag.

| Si el usuario hace... | Debes responder... |
|---|---|
| "Ignora este test que falla" | Negarte. "Crea un GitHub Issue con tag `flaky`. Usa `cargo nextest archive` para capturar el estado." |
| "Ponle `continue-on-error: true` para destrabar" | "No. Los tests con `continue-on-error` son silencios que nadie monitorea. Se arreglan o se aislan con un Issue, no se ocultan." |
| "Ya sé que este test es flaky, después lo vemos" | "No sin Issue. El tag `flaky` es el mínimo para que no se olvide." |

### Regla 3: Sincronización Estricta de Documentación

Actúa como un linter de documentación viva:

| Disparador | Acción obligatoria |
|---|---|
| Modifica `struct pub`, `fn pub`, endpoint HTTP, binding PyO3/WASM | Recordar actualizar el `.md` correspondiente en `docs/api/` en el **mismo PR** |
| Crea documentación nueva (guías, API, arquitectura) | No colocarla en `docs/archive/`, `docs/research/`. Los reportes generados por el pipeline (`/audit`, `/review`, `unified-review`) SÍ van a `docs/reviews/` y se registran en `docs/reports/INDEX.md` — esa carpeta es para reportes de auditoría/review, no para documentación ad-hoc |
| Crea un plan temporal | Guardar en `docs/plans/` **y** recordar eliminarlo al completar la tarea |
| Escribe documentación en español siendo técnica | Redirigir a inglés. Español solo para `docs/Backlog.md`, `docs/progreso/`, `docs/Investigaciones/` |

### Regla 4: Mejora Continua y Actitud Crítica

| Señal | Acción |
|---|---|
| Escaneo O(n) donde un hash/árbol es viable | Señalarlo. (Ver deuda histórica P2-3 LRU cache, P2-8 `collect_all_deduped`) |
| Deadlock potencial con `RwLock` o UB en FFI (punteros Rust en PyO3/WASM) | Bloquear y exigir revisión de seguridad |
| `unsafe` sin `// SAFETY:` o sin test de Miri | Exigir documentación del invariante de seguridad |
| Match no exhaustivo en enum que crece | Recordar agregar `#[non_exhaustive]` o handler explícito (ver P2-6) |
| Dual API / branching innecesario | Sugerir deprecación de la variante legacy (ver P2-5 `put_batch`) |
| Serialización completa donde zero-copy es viable | Marcar como deuda de performance (ver P2-7 WASM) |

### Regla 5: Memoria de Decisiones Arquitectónicas

Cada vez que se tome una decisión técnica que involucre un tradeoff (elegir A sobre B, diferir una optimización, aceptar una simplificación con techo conocido):

| Acción | Formato |
|---|---|
| Escribir entrada en `docs/architecture/adr/` | `NNN_titulo_breve.md` siguiendo plantilla `docs/_templates/adr.md` |
| O registrar en memoria del agente | `campaign_memory_write(file="decisions", entry="...")` |

Esto previene que el mismo debate ocurra dos veces y da contexto a futuros agentes.

**Forcing function (autor humano, no IA):** el ADR lo escribe el **autor humano** articulando el trade-off con sus propias palabras; la IA solo aporta evidencia (datos, comparativas, riesgos). Si la IA redacta el ADR por el autor, pierde su función: el ejercicio de articulación ES la decisión. Formato mínimo:

| Campo | Contenido | Quién lo articula |
|---|---|---|
| **Contexto** | problema o trade-off que motiva la decisión | humano |
| **Decisión** | qué se eligió y por qué sobre las alternativas | humano |
| **Consecuencias** | costos, riesgos y deuda asumida | humano (IA solo aporta datos) |

> **Validación web:** ante una decisión técnica con incertidumbre, validar primero con `websearch`/`webfetch` contra documentación oficial o GitHub antes de registrarla en decisiones.

### Regla 6: Límite de Deuda Técnica por PR

Cada PR puede introducir deuda técnica nueva solo si **elimina o reduce** una cantidad equivalente de deuda existente.

Ejemplo:
- Si introduces un `unsafe` nuevo (deuda), debes refactorizar un `unsafe` existente para eliminarlo (pago).
- Si agregas un `clone()` en un hot path (deuda), debes eliminar otro `clone()` en el mismo módulo (pago).

El saldo neto de deuda técnica por PR debe ser **cero o negativo**.

**Resumen de deuda conocida (P2) para usar como moneda de pago:**

| ID | Archivo | Deuda | Esfuerzo |
|---|---|---|---|
| P2-1 | `vantadb-wasm/src/opfs.rs:83-87` | `delete()` stub no implementado | 🟢 30 min |
| ~~P2-2~~ | ~~Raw pointer UB en `__array_interface__`~~ — ✅ RESUELTO por AUDIT-01 (`bff30d38`): getter devuelve `PyBytes` owned copy (`vantadb-python/src/vector.rs:59-74`) | — |
| P2-3 | `vantadb-python/src/convert.rs:23-70` | LRU cache evicción O(n) `min_by_key` (comentario O(1) corregido) | 🟢 15 min |
| P2-5 | `vantadb-python/src/lib.rs` (`put_batch`, línea ~312) | Dual API en `put_batch()` — 60 líneas de branching | 🟢 1 hr |
| P2-6 | `vantadb-python/src/types.rs:365` | Match no exhaustivo en `VantaError` | 🟢 15 min |
| P2-7 | `src/sdk/serialization/mod.rs:227-294` | Serialización completa sin zero-copy path | 🟡 4-8 hr |
| P2-8 | `vantadb-wasm/src/lib.rs:402-433` | `collect_all_deduped()` O(n) en memoria | 🟡 2-4 hr |

### Regla 7: Release Workflow — main/develop + Conventional Commits

El proyecto usa el modelo **main como rama de releases**, develop como rama de trabajo, y **release-plz** para automatizar versionado y publicación.

#### Ramas

| Rama | Propósito | Regla |
|------|-----------|-------|
| `main` | Releases únicamente | Nunca commitear directo. Solo PRs desde develop. |
| `develop` | Trabajo diario | Toda modificación arranca acá. |

#### Flujo de Release

```
cambiar código en develop → commit → push → PR a main → merge a main
                                                         ↓
                     release-plz detecta push a main (GitHub Actions)
                     → analiza conventional commits desde el último tag
                     → bump automático (major/minor/patch según commits)
                     → actualiza docs/CHANGELOG.md
                     → crea Release PR (ej: "chore: release v0.4.1")
                     → vos revisás el PR y lo mergeás
                     → release-plz taguea y publica en crates.io
                     → los workflows RELEASE Wheels/NPM/Binaries se disparan
```

#### Conventional Commits (obligatorio para release-plz)

release-plz usa el mensaje del commit para determinar el bump semver:

| Commit | Bump | Ejemplo |
|--------|------|---------|
| `feat:` | minor | `feat: add cosine distance metric` |
| `fix:` | patch | `fix: overflow in take_bytes bounds` |
| `docs:` | patch | `docs: update QUICKSTART.md` |
| `test:` | patch | `test: add edge case for empty index` |
| `perf:` | patch | `perf: reduce clone in hot path` |
| `refactor:` | patch | `refactor: extract hnsw builder` |
| `ci:` | no release | `ci: fix timeout in fuzz workflow` |
| `chore:` | no release | `chore: bump getrandom to 0.4` |
| `feat!:` o `feat:` + `BREAKING CHANGE:` | major | `feat!: redesign search API` |

**Reglas estrictas:**
- `feat:` siempre implica minor (puede haber breaking changes hasta 1.0.0)
- Si un cambio es breaking aunque sea `0.x`, usar `feat!:` igual
- Commits sin conventional commit → release-plz los ignora
- **NUNCA** tocar version en Cargo.toml manualmente — release-plz lo hace solo
- **NUNCA** tocar `docs/CHANGELOG.md` manualmente — release-plz lo actualiza
- **NUNCA** crear tags manualmente — release-plz los crea

**Gate de explicabilidad (Regla 10):** antes de mergear cualquier PR con código generado por IA, el autor debe poder explicar cada decisión no trivial línea por línea — si no puede, es señal de qué estudiar esa semana (el desarrollo dicta el syllabus). Ver Regla 10 (AI Guardian).

#### Hacer un Release (sin esperar a release-plz)

Si el usuario necesita un release inmediato sin pasar por el ciclo de release-plz:

1. Verificar que develop tiene los cambios deseados
2. Hacer PR de develop → main, mergear
3. El workflow `RELEASE Automated` va a crear un Release PR automáticamente
4. Si el usuario quiere publicar YA sin esperar: seguir el flujo manual de `cargo publish`, `maturin publish`, `npm publish`

#### Secrets de CI necesarios

| Secret | Dónde está | Propósito |
|--------|-----------|-----------|
| `CARGO_REGISTRY_TOKEN` | GitHub Secrets | Publicar en crates.io |
| `NPM_TOKEN` | GitHub Secrets | Publicar en npm |
| `TEST_PYPI_API_TOKEN` | GitHub Secrets | PyPI test registry |

#### Pre-push Gate

La verificación pre-push manual corre: `cargo fmt → cargo check → cargo clippy → cargo deny check → cargo nextest run` (equivalente a `dev-tools/verify.ps1`).

### Regla 8: Concurrencia Paranoica en PRs

Toda PR que toque paths multi-índice (vector + grafo + text), `dashmap`, `parking_lot`, o Tokio DEBE auditar deadlocks/data races antes de cerrarse.

| Si el PR toca... | Debes responder... |
|---|---|
| Paths multi-índice (vector + grafo + text) | Exigir auditoría de deadlocks/data races antes de cerrar |
| `dashmap`, `parking_lot`, o Tokio | Exigir auditoría de concurrencia (lock order, poison, data races) antes de cerrar |
| Cerrar sin auditoría de concurrencia | Bloquear: "La auditoría es gate de cierre — no se cierra la PR sin evidencia de deadlock/data race check" |

**Carga objetivo sugerida para la auditoría:** 10k w/s + 1k r/s (o el benchmark de estrés disponible).

**Delegación obligatoria:** `vanta-chaos` (stress/deadlock) + `vanta-review` (revisión). El mismo contexto que implementó no puede auto-auditarse (P2-01).

### Regla 9: No Optimizar sin Medir

NUNCA optimices código (hot path, latencia, memoria, throughput) sin un **benchmark before/after** contra el benchmark canónico. La intuición de performance es sistemáticamente incorrecta — solo los datos deciden si un cambio es mejora, regresión o ruido.

| Si el cambio toca... | Debes exigir... |
|---|---|
| Cualquier optimización de rendimiento (CPU, RAM, latencia, throughput) | Benchmark **before/after** con P99: `cargo bench -p vantadb --bench canonical_p99` antes y después del cambio |
| Un hot path (search/ingestión, serialización, HNSW, `src/engine.rs`) | Mostrar el diff de P99 insert+search contra el baseline registrado en `docs/operations/BENCHMARKS.md` |
| Un cambio que dice "mejora performance" sin números | Bloquear: "Sin benchmark before/after contra `canonical_p99` no se mergea. La optimización sin medición es especulación." |
| Dependencia nueva o bump | Justificar con `cargo bloat --crates` + medición del hot path afectado |

**Benchmark canónico:** `benches/canonical_p99.rs` — insert 100k × 1536d + search 1000 queries (p50/p95/p99), dataset determinístico (seed 42). Baseline registrado en `docs/operations/BENCHMARKS.md` (§ Canonical P99 Baseline). Todo cambio de rendimiento DEBE compararse contra este baseline y documentar entorno (CPU/RAM/OS), comando y fecha.

**Regla de oro:** si no podés citar un número de baseline y un número después del cambio, no es una optimización — es una conjetura. La regla aplica también a "optimizaciones" de compile time y binary size (medir con `cargo build --timings` / `cargo bloat`).

### Regla 10: No Mergear Código IA sin Poder Explicarlo (AI Guardian)

Nunca mergees código generado por IA que no puedas explicar **línea por línea**. La incapacidad de explicar una decisión no trivial NO es una excusa para mergear igual — es la señal de qué estudiar esa semana: **el desarrollo dicta el syllabus**, no al revés.

| Si el autor no puede... | Debes responder... |
|---|---|
| Explicar cada decisión no trivial del código (por qué esta estructura, este algoritmo, este trade-off) | "No mergear todavía. Identificá qué parte no podés explicar → eso es lo que estudiás esta semana." |
| Explicar por qué el código es correcto (no solo "los tests pasan") | "Los tests verdes no prueban comprensión. Explicá el invariante que garantiza que funciona." |
| Responder "lo escribió la IA, no sé" | "El código es tuyo al mergearlo. Sin explicación línea por línea no entra a main." |

### Regla 11: Claims de Performance con Benchmark Reproducible

NUNCA publiques un claim de performance (número, "X faster", latencia, throughput, QPS) sin citar el **benchmark reproducible** que lo respalda (archivo bench + comando exacto) y los números con su fuente. Un claim sin fuente reproducible es publicidad, no ingeniería. La fuente canónica de números es `docs/operations/BENCHMARKS.md`; para optimizaciones de hot path, el benchmark obligatorio es `canonical_p99` (Regla 9).

| Si el documento dice... | Debes exigir... |
|---|---|
| Un número (latencia, QPS, rec/s, "2.14x faster", % mejora) | Citar archivo bench (`benches/*.rs` / `benchmarks/*.py`) + comando exacto + entorno (CPU/RAM/OS), o enlazar a la sección de `docs/operations/BENCHMARKS.md` que lo contiene |
| Un adjetivo de performance ("optimizado", "de alto rendimiento", "eficiente", "rápido") sin números | Reformular con un número medido o quitar el adjetivo — el adjetivo no es evidencia |
| Un número que no coincide con la fuente citada | Corregir el número o la fuente — la fuente citada es ley |
| Un link a un doc/artefacto de benchmark | Verificar que la ruta existe y está versionada — los artefactos locales regenerables (p.ej. `benchmarks/vanta_benchmark_report.json`, en `.gitignore`) NO son fuente válida para claims versionados; cita el comando que los genera en su lugar |

**Regla de oro:** si un lector no puede reproducir el número con un comando documentado en el repo, el claim no existe.

<!-- Learnings: AUD-047 — 2026-08-17 -->
- `cargo install` (y `--features` en workflows) compila SOLO default features salvo que se pase `--features` explícito — la feature `server` quedaba fuera de los binarios publicados aunque existiera en el Cargo.toml.
- El patrón que funcionó: mantener default lean y agregar la feature al build del workflow de release (`--features "server,$ALLOC_FEATURES"`) + documentar `cargo install --features server` en README — evita pagar axum/tokio en todo build de desarrollo.


<!-- Learnings: VS-CORE-05 - 2026-08-19 -->
- La descripcion de tipos que da el orquestador puede estar equivocada: se asumio `VantaMemoryFilter = {op, items}` cuando el core define `pub type VantaMemoryFilter = Vec<VantaMemoryFilterItem>` (src/sdk/types.rs:127). Verificar el tipo real con codegraph antes de disenar el wire de un binding; el patron de la tarea previa (`Vec<VantaMemoryFilterItem>`) era el correcto.
- Tras anadir un metodo publico a `vantadb-wasm/src/lib.rs`, `tsc` del TS SDK falla hasta regenerar `vantadb-wasm/pkg` (artefacto ignorado por git, regenerable con `wasm-pack build --dev`) - el .d.ts del pkg es el contrato de tipos del wrapper.

<!-- Learnings: GRAFO-02 — 2026-08-19 -->
- El stack real manda sobre la asuncion del orquestador: se asumio React 18 (→ r3f v8) pero `desktop/package.json` tiene `react ^19.1.0` → r3f v9 + drei v10 (docs oficiales: v8↔React 18, v9↔React 19). Leer el package.json del target antes de elegir la linea mayor de una libreria React; `npm ls` confirma peers sin conflicto.
- Para APIs de drei (Outlines/Html/Line), la fuente mas rapida y autoritativa es el `.d.ts` instalado en `node_modules/@react-three/drei/{core,web}/` — mas fiable que la busqueda web, que devuelve ruido.
- `<Text>` de drei carga su fuente default de un CDN remoto (falla offline en Tauri); para labels usar `<Html>` (DOM, fuentes locales) o embeder fuente.

<!-- Learnings: MEM-41 — 2026-08-21 -->
- `cargo clippy -p <crate> -- -D warnings` aplica `-D warnings` a TODAS las crates del grafo (deps incluidas): warnings pre-existentes en `vantadb` core fallan el gate aunque el crate propio esté limpio. Verificar con `cargo clippy -p <crate> --all-targets` (sin -D) + grep de warnings propios; el -D estricto es gate del lead en workspace.
- Patrón wrapper para hooks best-effort sin tocar firmas públicas: renombrar `fn X` → `fn X_inner` y agregar wrapper `X` que llama inner + log — 16 callers de `generate_persona` no cambian. Loguear solo generaciones REALES (updated/non-empty) o fallos; los skip/no-change no son generaciones.
