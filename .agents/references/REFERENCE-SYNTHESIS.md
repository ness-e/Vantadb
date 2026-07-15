# REFERENCE SYNTHESIS — Campaign Executor v2

Fecha: 2026-07-15
Fuentes: `.agents/references/` (11 entries)
Propósito: Compilar toda la información de referencia para mejorar el Campaign Executor v2

---

## Tabla de Contenidos

1. [awesome-harness-engineering](#1-awesome-harness-engineering)
2. [darwin-godel-machine](#2-darwin-godel-machine)
3. [statewright](#3-statewright)
4. [deepclaude](#4-deepclaude)
5. [Checklists individuales](#5-checklists-individuales)
   - 5.1 Definition of Done
   - 5.2 Security Checklist
   - 5.3 Observability Checklist
   - 5.4 Orchestration Patterns
   - 5.5 Testing Patterns
   - 5.6 Performance Checklist
   - 5.7 Accessibility Checklist
6. [Priorización de mejoras](#6-priorización-de-mejoras)

---

## 1. awesome-harness-engineering

**Ubicación:** `C:\Users\Eros\VantaDB Proyect\VantaDB\.agents\references\awesome-harness-engineering\`

**Definición central:** La práctica de dar forma al entorno alrededor de los agentes de IA para que puedan trabajar de manera confiable. No se trata del modelo, sino de todo lo demás: prompts, herramientas, estado, contexto, guardrails, verificación, observabilidad, orquestación.

### Archivos analizados

| Archivo | Contenido |
|---------|-----------|
| `README.md` (168L) | Lista curada de recursos de harness engineering |
| `LICENSE` | CC0 1.0 Universal (Dominio público) |
| `CONTRIBUTING.md` | Guía de contribución |
| `.github/pull_request_template.md` | Template PR |
| `.github/workflows/link-check.yml` | CI con lychee para links rotos |

### Secciones del README.md (recursos)

**Foundations (10 artículos):**
- OpenAI: "Harness engineering" — restricciones arquitectónicas, instrucciones repo-locales, validación en browser, telemetría
- Anthropic: "Effective harnesses for long-running agents" — initializer agents, feature lists, init.sh, self-verification, handoff artifacts
- Anthropic: "Harness design for long-running app development" — mejor diseño de estado de tarea y evaluadores
- LangChain: "Anatomy of an Agent Harness" — Modelo + Harness = prompts, tools, middleware, orchestration, runtime
- Thoughtworks/Martin Fowler: "Harness Engineering" — context engineering, restricciones arquitectónicas, garbage collection contra entropía
- Anthropic: "Building effective agents" — Workflows, agents, tools, structured systems vs raw prompting
- HumanLayer: "Skill Issue" — resultados débiles de agentes son problemas de harness, no del modelo
- Inngest: "Your Agent Needs a Harness, Not a Framework" — estado, retries, traces, concurrencia como infraestructura first-class
- "Greenfield AI, Brownfield AI": taxonomía de codebases con CLAUDE.md en capas, pre-commit hooks ratcheted, lint violations baselineadas
- "Harness Engineering for Language Agents: CAR decomposition" — Control-Agency-Runtime, introduce HarnessCard
- "Many Hands Engineering": cómo múltiples agentes comparten un commons

**Context, Memory & Working State (8 artículos):**
- Anthropic: ventana de contexto = presupuesto de memoria de trabajo, no dumping ground
- Manus: KV-cache locality, tool masking, filesystem memory, mantener fallos útiles en contexto
- OpenHands: bounded conversation memory — solo preserva goals, progress, critical files, failing tests
- HumanLayer: reducir context drift, session resumability, backpressure contra low-value context burns
- HumanLayer: writing good CLAUDE.md

**Constraints, Guardrails & Safe Autonomy (9 recursos):**
- Anthropic: reducir fricción de aprobación via sandboxing/policy design
- Anthropic: code execution con MCP — tool boundaries explícitos e inspeccionables
- Anthropic: writing effective tool interfaces for models
- OpenHands: mitigación de prompt injection — confirmation mode, analyzers, sandboxing, hard policies
- Thoughtworks: moving quality checks into the loop, anchoring agents to reference applications, humans-in-the-loop
- **Lurkr**: Static scanner (CI) para riesgos de capability de agentes: shadow capabilities, credenciales en contexto, eval/subprocess en @tool, prompt interpolation directa, MCP endpoints no verificados

**Specs, Agent Files & Workflow Design (6 recursos):**
- AGENTS.md: formato abierto ligero para instrucciones repo-locales
- GitHub Spec Kit: spec-driven development
- Thoughtworks: why strong specs make AI-assisted delivery more dependable
- **12 Factor Agents** (HumanLayer): principios de producción — prompts explícitos, state ownership, clean pause-resume
- **12-Factor AgentOps**: context discipline, validation, reproducible agent workflows

**Evals & Observability (14 recursos):**
- OpenAI: turning traces into repeatable evals con JSONL logs
- OpenHands: measuring skill effectiveness con bounded tasks, deterministic verifiers, no-skill baselines
- **Inspect AI** (UK AISI): eval framework open-source: solver, scorer, sandboxing, tool-use, MCP, log-viewer
- **OpenTelemetry Semantic Conventions for GenAI**: spans, metrics, events estandarizados para agent workflows
- **AgentOps**: SDK open-source Python: session replay, cost tracking, benchmarking, tracing
- **agenttrace**: TUI/CLI local-first: auditoría de traces de sesiones, health gates, cost spikes, tool failures, latency gaps
- OpenHands: layered verification con trajectory critics, reranking, early stopping
- Anthropic: quantifying infrastructure noise in benchmarks (runtime config afecta scores más que el modelo)

**Runtimes, Harnesses & Reference Implementations (16 recursos):**
- **HEAAL**: safety constraints via grammar-enforced AIL
- **Claude Agent SDK**: sessions, tools, orchestration
- **deepagents** (LangChain): agentes long-running con middleware/harness patterns
- **SWE-agent**: agente de código maduro con inspectable harness, prompt, tools, environment
- **SWE-ReX**: sandboxed code execution infrastructure
- **AgentKit** (Inngest): TypeScript durable workflow-aware agents en event-driven infrastructure
- **browser-use/browser-harness**: CDP-based browser harness, self-healing web-task workflows
- **Citadel**: harness para Claude Code/Codex con isolated worktrees, multi-agent coordination, persisted memory, campaign state
- **Harbor**: harness generalizado para evaluar/mejorar agentes a escala
- **"Ralph Wiggum" pattern**: `while :; do cat PROMPT.md | claude-code; done` — single-task loops, deterministic prompt stacking, bounded subagent parallelism
- **skills.sh**: marketplace comunitario de skills reutilizables
- **Uni-CLI**: hub CLI universal con 134 sites, 711 pipelines YAML declarativos, 8-phase Karpathy-style self-repair loop, eval harness, cost ledger, sensitive-path deny list, MCP serve (~80 tokens/invocación)

### Patrones clave extraídos

| Patrón | Descripción | Fuente |
|--------|-------------|--------|
| CAR Decomposition | Control-Agency-Runtime: documentar el harness en 3 dimensiones | Position paper |
| Initializer agent | Agente separado que setup context + task list → handoff a workers | Anthropic |
| init.sh pattern | Script que bootstrapea el environment del agente al inicio | Anthropic |
| Self-verification | Agente verifica su propio output antes de declarar completo | Anthropic |
| Handoff artifacts | Artefactos estructurados entre ventanas de contexto | Anthropic |
| while loop harness | `while :; do cat PROMPT.md \| agent; done` — minimalista, determinista | Ralph Wiggum |
| Bounded conversation memory | Solo preserva: goals, progress, critical files, failing tests | OpenHands |
| Filesystem memory | Usar filesystem como memoria persistente entre turns | Manus |
| Tool masking | Mostrar solo tools relevantes al paso actual | Manus |
| Layered verification stack | Múltiples capas: unit tests, critics, sandbox execution | OpenHands |
| Ratcheted pre-commit hooks | Calidad que solo se endurece, nunca se relaja | Greenfield/Brownfield |
| 8-phase self-repair loop | Plan → Execute → Validate → Diagnose → Fix → Retry → Report → Escalate | Uni-CLI |
| 12-Factor Agents | Prompts explícitos, state ownership, clean pause-resume, logs como JSONL | HumanLayer |
| Spec-driven development | Ejecutar agentes contra specs explícitas para resultados predecibles | GitHub Spec Kit |
| Lurkr scanning | Static scanning CI-time: shadow capabilities, credenciales, MCP endpoints | Lurkr |
| HarnessCard | Documentar el harness usando CAR framework | Position paper |

### Principios de context management

1. **Context window = working memory budget, not dumping ground**
2. **Useful failures should stay in-context** — mantener lo que salió mal para diagnóstico
3. **Prevenir context drift** — resumability requiere estructura de contexto estable
4. **Backpressure on low-value work** — no quemar contexto en ruido
5. **Bounded memory** — solo goals, progress, critical files, failing tests

---

## 2. darwin-godel-machine

**Ubicación:** `C:\Users\Eros\VantaDB Proyect\VantaDB\.agents\references\darwin-godel-machine\`

**Definición:** Sistema de auto-mejora para agentes. Implementa un ciclo evolutivo donde los agentes diagnostican su propio rendimiento, proponen modificaciones, las implementan, evalúan los resultados y archivan el aprendizaje.

### Archivos analizados (~50+ archivos)

**Raíz (8):** README, research paper, controller, executors, pytest config, gitignore, contributing, security, roadmap

**Core (4 módulos, ~14 archivos):**
- `agent/`: Agente principal (`agent.py` 55k chars), interfaz FM (`fm_interface/` con 3 providers), tools (`tools/` con 3 tools)
- `archive/`: Sistema de archivo (`agent_archive.py`, `parent_selector.py` con fórmula exacta, `lineage_visualizer.py`)
- `evaluation/`: Evaluación (`benchmark_runner.py` 43k chars, `scorer.py` múltiples estrategias, `agent_validator.py` 30k chars)
- `self_modification/`: Pipeline de auto-modificación (`performance_diagnosis.py` → `modification_proposal.py` → `implementation.py`)

**Infraestructura:**
- `sandbox/sandbox_manager.py`: Ejecución aislada vía Docker con staging/sync-back, 462 líneas
- `utils/agent_loader.py`: Carga dinámica de agentes con paquetes sintéticos, 203 líneas

**Configuración (7 archivos YAML/JSON):**
- `dgm_config.yaml`: Config central, 102 líneas (providers, DGM settings, archive, parent selection, evaluation, sandbox, agents, benchmarks, logging, safety)
- `live_dgm_proof.yaml`, `eval_model_matrix.yaml`, `live_model_matrix.yaml`, `live_score_movement.yaml`
- 3 benchmarks: `humaneval_calibrated` (233L), `humaneval_headroom` (97L), `humaneval_style` (72L)

**Documentación:** Research paper, architecture designs (3), live-run proofs (30 dirs), demos, runbook, telemetry format

**Scripts (17):** `run_dgm_in_sandbox.py`, `summarize_archive_scores.py`, `summarize_live_run_telemetry.py`, `verify_demo_path.py` + cost estimation, model matrix, seed archive, lineage scripts

### Patrones clave extraídos

| Patrón | Descripción | Aplicación |
|--------|-------------|------------|
| **DGM Loop** | Diagnose → Propose → Modify → Evaluate → Archive | Auto-mejora del pipeline cada N tareas |
| **Parent selection** | Sigmoid(rendimiento × novelty) con muestreo | Elegir qué versión del pipeline usar |
| **Sandbox Docker sin red** | Aislar ejecución, staged-project con sync-back | Aislar runs de agentes de prueba |
| **Config-driven** | Toda parametrización en YAML, no hardcode | Reemplazar variables en prompts por YAML |
| **Telemetría estructurada** | Parseo de logs → provider tokens, latencias, costos | Cost tracking por tarea |
| **Score movement rehearsal** | Probar cambios en modo "rehearsal" antes de default | A/B testing del pipeline |

### Fórmula de selección de padres

```
P(p) = σ(α · performance(p) + β · novelty(p))
donde σ es sigmoid, performance = score normalizado, novelty = distancia al padre actual
```

---

## 3. statewright

**Ubicación:** `C:\Users\Eros\VantaDB Proyect\VantaDB\.agents\references\statewright\`

**Definición:** State machine guardrails que controlan qué tools puede usar un agente en cada fase. "Agents are suggestions, states are laws."

### Estructura

```
statewright/
├── crates/
│   ├── engine/     → Evaluador de state machine en Rust puro. Determinista. Sin LLM.
│   ├── cli/        → sw-agent: ejecutor de workflows contra Ollama
│   ├── mcp-gateway/→ Gateway MCP para integrar con Claude Code, Codex, Pi, opencode, Cursor
│   └── tui/        → Interfaz ratatui terminal
├── plugins/        → Plugins para cada agente soportado
├── docs/specs/     → Especificaciones: arquitectura, state-machine-pipeline, model-compatibility, etc.
├── templates/      → Workflow templates
└── tests/          → Tests
```

### Arquitectura (3 capas)

1. **Engine** (`crates/engine`) — Rust puro. Evalúa estados, transiciones, guards, tool restrictions. Sin LLM. Sin runtime dependencies.
2. **Agent binary** (`sw-agent`) — Ejecutor directo-a-Ollama. Carga workflow, ejecuta LLM en loop constrainido, enforce tool access. Soporta per-state model routing.
3. **Plugin layer** (MCP gateway) — Se integra con coding agents. Cuando se activa un workflow, los hooks enforcean tool restrictions por estado.

### gen_sm → llm_solve Pipeline

El concepto más importante de Statewright:

```
Task Input → [Generator Model] → StateMachineDefinition → [Executor Model] → Result
```

**Stage 1: Generate State Machine (gen_sm)** — Un modelo (fronterizo o fine-tuned) genera un `StateMachineDefinition` JSON: estados, transiciones, guards, allowed_tools, max_iterations. **Un API call. Bajo temperature. Cacheable.**

**Stage 2: Execute Within State Machine (llm_solve)** — Cualquier modelo ejecuta dentro de las constraints. Modelos pequeños obtienen más guardrails.

**Beneficios de la separación:**
- Planificar cuesta una vez, se reusa
- Modelo ejecutor puede ser pequeño (el plan se provee)
- Plan es artefacto explícito y determinista, no implícito en el prompt
- El plan es entrenable (fine-tune planner en triples exitosos)

### Guardrails (12)

| Guardrail | Qué hace |
|-----------|----------|
| Per-state tool enforcement | El agente no ve ni puede llamar tools fuera de `allowed_tools` |
| Bash discernment | Bloquea `echo > file`, `rm -rf`, `sed -i`, scripting interpreters |
| Edit guards | Rechaza diffs > `max_edit_lines`, limita archivos por estado |
| Command allow-lists | Solo comandos prefix-matching (pytest, cargo test) |
| Conditional transitions | Guards programáticos: `test_result eq pass` |
| Approval gates | `requires_approval` pausa para revisión humana |
| Interrupts | Editar un archivo → auto-transición a validation state |
| Fork/join | Ramas secuenciales o paralelas, join all/any |
| Environment scoping | Ocultar `PROD_DB_URL` vía `blocked_env`, sustituir con `env_overrides` |
| Session isolation | Estado por sesión vía `CLAUDE_SESSION_ID` |
| Per-state model routing | Estados baratos → modelos chicos, estados caros → frontier |
| Thinking level control | Per-state `thinking_level`: high, medium, low, off |
| Tool escalation detection | Warning cuando un estado salta 2+ niveles de privilegio sin approval |

### Model Traits Registry

Registro que describe cómo cada modelo maneja tool calling, razonamiento, contexto y output:

| Campo | Descripción |
|-------|-------------|
| `tool_mode` | native / raw / auto |
| `reasoning` | bool |
| `response_field` | content / reasoning |
| `history_window` | Turns a retener (3 para chicos, 10 para grandes) |
| `max_full_read_lines` | Límite de líneas para reads sin rango |
| `max_diff_lines` | Máximo de líneas cambiadas antes de considerar oversize |
| `unescape_tool_args` | Si el modelo double-escapea JSON |
| `single_quote_json` | Si el modelo usa single quotes en JSON |
| `num_ctx` | Context window override |

Resolución jerárquica: defaults → familia → tamaño → tag.

### MoM (Mixture of Models) Escalation Pattern

```
Tier 1: qwen3:8b (5GB) — history_window=5, max_read=200
Tier 2: devstral-small-2:24b (15GB) — history_window=10, max_read=400
Tier 3: Frontier API — sin constraints locales
```

Cada nivel resuelve sus propios traits. El harness cambia tool mode, prompt format y context limits automáticamente.

### Fork/Join

Paralelismo con branches: lint, test y docs pueden ejecutarse simultáneamente. Soporta `join: "all"` y `join: "any"`. Máx 8 branches, 4 subprocesos concurrentes.

### Descubrimientos experimentales clave

1. **num_ctx es load-bearing para Ollama** — default 2048 es catastróficamente pequeño. Mínimo 8192, recomendado 32768.
2. **Modelos de razonamiento** (gpt-oss, deepseek-r1) outputean en formatos no estándar: mixed JSON+texto, reasoning field, Harmony tokens.
3. **Default a raw JSON mode para todos los modelos** — native es optimización, raw funciona universalmente.
4. **Context minimization per-state** es el factor más importante para que modelos chicos funcionen en tareas no-triviales.

---

## 4. deepclaude

**Ubicación:** `C:\Users\Eros\VantaDB Proyect\VantaDB\.agents\references\deepclaude\`

**Definición:** Usar el loop autónomo de Claude Code con DeepSeek V4 Pro, OpenRouter, o cualquier backend Anthropic-compatible. Mismo UX, 17x más barato.

### Archivos analizados

| Archivo | Contenido |
|---------|-----------|
| `README.md` (326L) | Documentación completa |
| `deepclaude.ps1` | Script PowerShell |
| `deepclaude.sh` | Script bash |
| `proxy/README.md` | Documentación del proxy |
| `proxy/model-proxy.js` | Proxy Node.js |

### Cómo funciona

Claude Code lee variables de entorno para determinar adónde enviar API calls. deepclaude las configura por sesión, lanza Claude Code, y restaura al salir.

```
ANTHROPIC_BASE_URL → endpoint API
ANTHROPIC_AUTH_TOKEN → API key
ANTHROPIC_DEFAULT_OPUS_MODEL → nombre del modelo para Opus-tier
ANTHROPIC_DEFAULT_SONNET_MODEL → nombre del modelo para Sonnet-tier
ANTHROPIC_DEFAULT_HAIKU_MODEL → nombre del modelo para Haiku-tier (subagentes)
CLAUDE_CODE_SUBAGENT_MODEL → modelo para subagentes
```

### Backends soportados

| Backend | Input/M | Output/M | Notas |
|---------|---------|----------|-------|
| DeepSeek (default) | $0.44 | $0.87 | Auto context caching (120x más barato en repeat turns) |
| OpenRouter | $0.44 | $0.87 | Más barato, menor latencia desde US/EU |
| Fireworks AI | $1.74 | $3.48 | Inferencia más rápida |
| Anthropic | $3.00 | $15.00 | Claude Opus original |

### Live switching (proxy)

Proxy en `localhost:3200` que intercepta API calls y permite cambiar backend mid-session sin reiniciar:

```
/_proxy/mode POST → cambiar backend
/_proxy/status GET → backend activo + uptime
/_proxy/cost GET → token usage + savings
```

### Qué funciona y qué no

**Funciona:** Read, Write, Edit, Bash, Glob, Grep, multi-step tool loops, subagent spawning, git, /init, thinking mode
**No funciona:** Image/vision input, parallel tool use (DeepSeek lo soporta pero Claude Code envía secuencial), MCP server tools (compatibility layer), prompt caching savings (DeepSeek tiene auto-caching pero Anthropic cache_control se ignora)

---

## 5. Checklists individuales

### 5.1 Definition of Done

**Archivo:** `accessibility-checklist.md` → NO, es `definition-of-done.md`

**Estructura:** 5 ejes con checks específicos:

| Eje | Checks clave |
|-----|-------------|
| **Correctness** (5) | AC met, runtime-verified, tests fail without change, no regressions, edge/error paths handled |
| **Quality** (5) | Intent-revealing code, no duplicated logic, no dead code/debug output, scoped changes, linting passes |
| **Integration** (3) | Works with rest of system, migrations/config/feature flags accounted for, backward compatibility |
| **Documentation** (3) | Public interfaces documented, ADRs for architectural decisions, timeless language |
| **Ship-readiness** (4) | Security review, observability, rollback path, human review before merge |

**6 Red flags** que indican que el DoD se está subvirtiendo.

**Aplicación:** El pipeline debe tener un DoD checklist como validación final. "Unverified work is not done" — runtime-verified es obligatorio.

---

### 5.2 Security Checklist

**Archivo:** `security-checklist.md` (179L)

**Estructura:**

| Sección | Checks |
|---------|--------|
| Threat Modeling (4) | Trust boundaries, assets, STRIDE, abuse cases |
| Pre-commit (3) | No secrets in code, .gitignore, .env placeholders |
| Authentication (7) | bcrypt, cookies seguras, rate limiting, reset tokens, lockout, MFA |
| Authorization (5) | Every endpoint checks auth, IDOR prevention, admin role, scoped API keys, JWT validation |
| Input Validation (10) | Allowlists, validación por tipo, file upload, SQL parametrizado, HTML encoding, SSRF prevention |
| Security Headers | CSP, HSTS, X-Content-Type-Options, etc. |
| CORS | Config example, warning contra wildcard |
| Data Protection (5) | Sensitive fields excluded, no logging, PII encrypted, HTTPS, backups encrypted |
| Dependency Security (4) | npm audit, lockfile, CI=ci, typosquatting |
| **AI/LLM Security (5)** | Model output untrusted, prompt injection assumed, secrets out of context, scoped tool permissions, token/rate/recursion limits |
| Error Handling | Generic errors only, no stack/sql/internals |
| OWASP Top 10 | Tabla 10 vulnerabilidades + prevención |
| OWASP Top 10 for LLMs | Tabla 10 riesgos: prompt injection, sensitive disclosure, supply chain, data poisoning, improper output handling, excessive agency, system prompt leakage, vector weaknesses, misinformation, unbounded consumption |

**Principios críticos para Campaign Executor:**
- "Treat model output as untrusted" — toda salida generada debe validarse/sanitizarse
- "Prompt injection assumed; permissions enforced in code, not in system prompt"
- "System Prompt Leakage (LLM07)" — no poner secrets en prompts
- "Excessive Agency (LLM06)" — scope tool permissions; acciones destructivas requieren confirmación

---

### 5.3 Observability Checklist

**Archivo:** `observability-checklist.md` (91L)

**Estructura:**

| Señal | Checks |
|-------|--------|
| Structured Logging (8) | JSON con stable event names, correlation IDs, no secrets, allowlisted fields |
| Metrics (6) | RED por endpoint, USE por recurso, latency histograms p50/p95/p99, no unbounded labels |
| Distributed Tracing (6) | OpenTelemetry, W3C trace context, async survival, manual spans |
| Alerting (7) | Symptom-based (error rate, p99), actionable, links to runbook, 2 severities |
| Dashboards (4) | Service health, dependency health, answers on-call questions |
| Verify the Telemetry (4) | Forced error + found it, metric series appear, E2E trace, diagnosed from telemetry |
| Pre-Launch Gate (5) | Logs flowing, RED metrics visible, alert configured + test-fired, traceable request, runbooks known |

**Principios:**
- "Telemetry without a question is noise" — escribir las 2-4 preguntas que hará un on-call
- Metrics = that something is wrong, traces = where, logs = why
- No unbounded label values (no user/tenant IDs)
- Instrumentation can be wrong — verify it end-to-end

---

### 5.4 Orchestration Patterns

**Archivo:** `orchestration-patterns.md` (370L)

**Estrategias endorsadas:**

| Patrón | Uso |
|--------|-----|
| 1. Direct invocation | Single persona, single perspective, single artifact (default, cheapest) |
| 2. Single-persona slash command | Wraps one persona with project skills |
| 3. Parallel fan-out with merge | Múltiples personas mismo input → merge |
| 4. Sequential pipeline | User-driven slash commands, user ES el orchestrator |
| 5. Research isolation | Sub-agente para leer material grande → solo digest |

**Anti-patrones:**
- A. **Router persona ("meta-orchestrator")** — capa de routing sin valor, añade costo y pierde info
- B. **Persona that calls another persona** — destruye single-perspective design, esconde costo, multiplica failure modes
- C. **Sequential orchestrator that paraphrases** — pierde checkpoints humanos, acumula drift
- D. **Deep persona trees** — cada capa añade latencia/tokens sin valor

**Regla principal:** "The user (or a slash command) is the orchestrator. Personas do not invoke other personas."

---

### 5.5 Testing Patterns

**Archivo:** `testing-patterns.md` (235L)

**Estructura:**
- AAA (Arrange-Act-Assert)
- Naming conventions: `[unit] [expected behavior] [condition]`
- Mocking patterns: **Mock at Boundaries Only** (mock DB/HTTP/FS/external APIs → NO mock internal utils/business logic/validation)
- React/Component Testing (render, screen, findByRole)
- API/Integration Testing (supertest)
- E2E Testing (Playwright, getByRole)

**Anti-patrones:** Testing implementation details, snapshot everything, shared mutable state, skipping tests

---

### 5.6 Performance Checklist

**Archivo:** `performance-checklist.md` (153L)

**Estructura:**
- Core Web Vitals: LCP ≤2.5s, INP ≤200ms, CLS ≤0.1
- TTFB Diagnosis: DNS, TCP/TLS, server processing
- Frontend: Images, JavaScript, CSS, Fonts, Network, Rendering
- Backend: Database, API, Infrastructure
- Measurement: Lighthouse CLI, bundle analyzers, web-vitals

---

### 5.7 Accessibility Checklist

**Archivo:** `accessibility-checklist.md` (160L)

**Estructura:**
- Keyboard Navigation (7 checks)
- Screen Readers (7 checks)
- Visual (5 checks): contrast 4.5:1, color not sole channel, 200% text resize, no flashing >3/sec
- Forms (5 checks): visible labels, required indicators, specific errors, autocomplete
- Content (5 checks): lang attribute, descriptive title, 44x44px touch targets, meaningful empty states

---

## 6. Priorización de mejoras

### 🔴 ALTA PRIORIDAD

| # | Mejora | Explicación | Referenciado en | Esfuerzo | Valor | Ya cubierto | Lo que falta | Patrón | Aplicación |
|---|--------|-------------|-----------------|----------|-------|-------------|--------------|--------|------------|
| 1 | **Correlation ID por campaña** | Generar un UUID al inicio de cada campaña y propagarlo en logs, recitation, commits y traces | Observability Checklist, awesome-harness-engineering (12-Factor Agents) | Bajo | Alto | Recitation block existe | No hay correlation ID trazable entre iteraciones | 12-Factor Agents: logs como event streams | `harness.ps1` genera GUID → inyecta en iter.md → recitation → commit message |
| 2 | **Validación de output generado (LLM05)** | Sanitizar HTML, SQL, shell commands, file paths generados por el agente antes de ejecutarlos | Security Checklist (AI/LLM Security: "model output is untrusted", "prompt injection assumed") | Medio | Alto | Capa Determinista en RULES.md | No hay validación de output del agente; se confía en lo que genera | OWASP LLM05: Improper Output Handling | Nueva capa de verificación en RULES.md: sanitizar output antes de write |
| 3 | **Rate limiting + Budget tracking real** | Limitar tool calls por tarea, tokens consumidos, tiempo de ejecución. Trackear contra presupuesto | Security Checklist (LLM10: unbounded consumption), awesome-harness-engineering (Inngest: state/retries/traces) | Medio | Alto | Budget table en SKILL.md (iteraciones, sub-agentes, fails) | Los límites son declarativos, no se enforcean en runtime; no hay tracking de tokens real | Token bucket / rate limiter pattern | `harness.ps1` trackea tool calls y tiempo; MCP server enforcea hard limits |
| 4 | **Output untrusted — sandbox execution** | Ejecutar código generado en entorno aislado (Docker sin red) antes de aplicarlo al proyecto real | Security Checklist, darwin-godel-machine (sandbox Docker), awesome-harness-engineering (SWE-ReX, sandboxing) | Alto | Alto | No hay sandbox | No hay aislamiento de ejecución | Sandbox pattern (DGM: sandbox sin red, staged-project con sync-back) | Nuevo `sandbox/` script: Docker sin red → stage → validate → sync-back |
| 5 | **Structured JSONL logging** | Emitir logs estructurados por evento (campaign.step.start, .complete, .error) con correlation ID | Observability Checklist (structured logging, stable event names) | Bajo | Alto | Recitation block en texto plano | No hay logs JSONL, no hay eventos estables, no hay métricas RED | OpenTelemetry Semantic Conventions for GenAI | `harness.ps1` emite eventos JSONL a `traces/<campaign-id>.jsonl` |

### 🟡 MEDIA PRIORIDAD

| # | Mejora | Explicación | Referenciado en | Esfuerzo | Valor | Ya cubierto | Lo que falta | Patrón | Aplicación |
|---|--------|-------------|-----------------|----------|-------|-------------|--------------|--------|------------|
| 6 | **Filesystem memory (traces/ + memory/)** | Estructura de directorios persistente: `traces/<id>.jsonl`, `memory/lessons.md`, `artifacts/` por tarea | awesome-harness-engineering (Manus: filesystem memory, OpenHands: bounded memory) | Bajo | Medio | Task files guardan estado | No hay persistencia estructurada de decisiones, lecciones, ni traces | Filesystem as persistent memory | Nuevos `traces/` y `memory/` dirs; pipeline escribe lecciones al cerrar tarea |
| 7 | **Research isolation pattern** | Para tareas que requieren ingerir mucho contexto: spawn sub-agente que lee y devuelve digest | Orchestration Patterns (Pattern 5: research isolation), awesome-harness-engineering | Bajo | Medio | Sub-agentes existen en pipeline-run.md | No hay sub-agente específico de "research" | Pattern 5: Research isolation | Agregar modo RESEARCH a pipeline-run.md: sub-agente lee → digest → tarea principal |
| 8 | **Ratcheted Definition of Done** | Los thresholds de calidad suben con el tiempo, nunca bajan. Cada release exige más | awesome-harness-engineering (Greenfield/Brownfield: ratcheted pre-commit hooks) | Medio | Medio | DoD versionado en cada workflow JSON + accept state con checks condicionales | ✅ 2026-07-15: dod_version en 5 workflows; accept state referencia versión y aplica checks progresivos (V1: 5 checks base; V2: +ponytail-review+test-coverage; V3: +no secrets+conventional-commit) | Ratcheted quality gates | workfows/*.json: `dod_version: 1` en cada definition; accept state ejecuta según versión |
| 9 | **Model traits registry** | Registry de traits por modelo (context window, tool mode, response field, etc.) | statewright (model-compatibility.md, model-traits.md) | Medio | Alto | No hay | No hay configuración por modelo; el harness trata todos igual | Model traits per-model | Nuevo `config/model-traits.yaml` con traits por modelo; harness resuelve según `-m` |
| 10 | **Mixture of Models (MoM) ladder** | Escalar a modelo más caro cuando el actual falla: Tier 1 (barato) → Tier 2 (medio) → Tier 3 (frontier) | statewright (MoM escalation, per-state model routing) | Alto | Alto | Retry ladder (4 escalones) en iter.md | Los escalones usan el mismo modelo, no escalan a uno mejor | MoM: per-state model routing | Agregar tiers de modelo al retry ladder; MCP enforcea cambio de modelo |
| 11 | **Per-state tool enforcement** | En cada estado de la state machine C0, restringir tools a solo las permitidas para ese estado | statewright (core concept: "agents are suggestions, states are laws") | Alto | Muy alto | State machine C0 en iter.md con 12 estados y 4 transiciones inválidas | Las tools no están restringidas por estado; el agente ve todas siempre | Per-state tool enforcement | Agregar `allowed_tools` por estado en la state machine; MCP gateways enforcean |
| 12 | **Static scanning con Lurkr** | CI-time scan de shadow capabilities, credenciales en prompts, MCP endpoints no verificados | awesome-harness-engineering (Lurkr scanner) | Bajo | Medio | Security checklist menciona "no secrets in code" | No hay scanning automatizado de riesgos de agente | Lurkr static scanning | Agregar paso a CI: `npx lurkr scan .opencode/` |
| 13 | **Fork/Join para tareas independientes** | Lint + test + docs en paralelo en vez de secuencial | statewright (fork-join.md: max 8 branches, join all/any) | Medio | Medio | Parallel mode en harness.ps1 (waves, 4 sub-agentes) | No hay fork/join dentro de una tarea; solo paralelismo entre tareas | Fork/Join pattern | pipeline-run.md: detectar sub-tareas independientes → fork ear |
| 14 | **Symptom-based alerting** | Alertar basado en síntomas (error rate, p99 latency) no en causas (CPU). Thresholds justificados por SLO | Observability Checklist (symptom-based, actionable, runbook-linked) | Bajo | Medio | Stagnation detection en iter.md | No hay alerting de salud del pipeline | Symptom-based alerting | harness.ps1: emitir métricas RED; alertar si campaign.error_rate > 5% |

### 🟢 BAJA PRIORIDAD

| # | Mejora | Explicación | Referenciado en | Esfuerzo | Valor | Ya cubierto | Lo que falta | Patrón | Aplicación |
|---|--------|-------------|-----------------|----------|-------|-------------|--------------|--------|------------|
| 15 | **HarnessCard** | Documentar el pipeline usando CAR (Control-Agency-Runtime): qué controla, qué decide, cómo ejecuta | awesome-harness-engineering (CAR decomposition, HarnessCard) | Bajo | Medio | SKILL.md describe arquitectura | No está estructurado como CAR Card | CAR Decomposition | Nuevo apéndice en SKILL.md con HarnessCard formal |
| 16 | **Score movement rehearsal** | Probar cambios de pipeline en modo "rehearsal" antes de hacerlos default | darwin-godel-machine (score movement rehearsal) | Medio | Medio | No hay | No hay forma de A/B testear cambios de pipeline | Score movement rehearsal | Nuevo modo `--rehearsal` en harness: ejecuta con nuevo config sin modificar producción |
| 17 | **Parent selection para pipeline versions** | Elegir versión del pipeline basado en rendimiento × novelty | darwin-godel-machine (sigmoid parent selection) | Alto | Bajo | No hay | No hay versionado del pipeline ni evolución | Evolutionary parent selection | Sistema de versionado de pipeline con selección evolutiva |
| 18 | **gen_sm → llm_solve pipeline** | Generar state machine específica de la tarea con modelo frontier, ejecutar con modelo commodity | statewright (gen_sm → llm_solve: state-machine-pipeline.md) | Muy alto | Muy alto | State machine C0 fija en iter.md | La state machine es fija, no se genera por tarea | gen_sm → llm_solve | Generar workflow JSON por tipo de tarea en DISCOVERY |
| 19 | **Template library de state machines** | 10-20 templates de state machine por tipo de tarea (bug_fix, feature_add, refactor, deploy) | statewright (Phase 1: template library) | Medio | Alto | No hay | Templates de workflow no existen | Template library | `.opencode/task-system/workflows/` con templates; keyword classifier selecciona |
| 20 | **Training data collection** | Recolectar triples (task, state_machine, outcome) para fine-tuning de planner | statewright (Phase 3: training data) | Alto | Bajo | No hay | No hay recolección de datos de ejecución | Training triple collection | Instrumentar cada tarea para emitir JSON de training |
| 21 | **Dynamic model switching (proxy)** | Poder cambiar de modelo mid-session sin reiniciar el harness | deepclaude (proxy model-switching) | Medio | Medio | -m flag en harness | No hay switching mid-session | Model proxy pattern | Agregar proxy ligero o endpoint MCP para cambiar modelo en caliente |
| 22 | **Per-state conversation retention** | Clear per cycle para modelos ≤10B, clear per phase para 10-30B, keep all para 30B+ | statewright (model-compatibility.md: conversation retention by model size) | Medio | Medio | Recitation persiste estado | No hay adaptación de retención por tamaño de modelo | Conversation strategy by model size | Agregar `conversation_strategy` al model traits registry |
| 23 | **HEAAL-style grammar constraints** | Restricciones vía gramática (AIL) a nivel de tool calls, no solo prompt | awesome-harness-engineering (HEAAL) | Muy alto | Alto | Per-state tool enforcement (idea) | No hay enforcement a nivel de gramática | Grammar-enforced constraints | Investigar HEAAL para enforcement en vez de solo prompt |

### Resumen de prioridades

| Prioridad | Count | Items |
|-----------|-------|-------|
| 🔴 Alta | 5 | Correlation ID, Output validation, Rate limiting, Sandbox, JSONL logging |
| 🟡 Media | 9 | Filesystem memory, Research isolation, Ratcheted DoD, Model traits, MoM ladder, Per-state tools, Lurkr scan, Fork/join, Symptom alerting |
| 🟢 Baja | 9 | HarnessCard, Score rehearsal, Parent selection, gen_sm→llm_solve, Template library, Training data, Dynamic model switching, Conversation retention, Grammar constraints |

### Mejoras SIN PRIORIDAD (no aplican al Campaign Executor)

| Mejora | Razón |
|--------|-------|
| Accessibility checks | El executor no genera UI directamente |
| Performance (Core Web Vitals) | No aplica a pipelines de backend |
| CSS/JS optimization | No genera bundles de frontend |

---

## Mapa de patrones a ubicaciones en el código

| Patrón | Archivo destino actual | Referencia |
|--------|-----------------------|------------|
| State machine C0 | `iter.md:105-125` | Statewright |
| Recitation / handoff | `iter.md:recitation block` | Anthropic handoff artifacts |
| Retry ladder | `iter.md:4 escalones` | Karpathy self-repair loop |
| Evaluator-optimizer | `iter.md:198-199` | Lilian Weng |
| Self-Harness Gate | `iter.md:208-218` | Anthropic self-harness |
| Pre-commit gate | `iter.md:220-227` | Greenfield/Brownfield ratchet |
| Budget controls | `SKILL.md:216-226` | explainx.ai |
| Stagnation detection | `iter.md | harness.ps1` | Anthropic no-progress |
| Parallel waves | `pipeline-run.md:waves` | Anthropic parallel workers |
| Capa Determinista | `RULES.md:124-139` | Propia |
| Rust safety rules | `RULES.md:124-129` | Propia |
| Progreso / doc coverage | `progreso/SKILL.md` | Propia |

### Mejoras técnicas específicas de statewright transferibles

| Concepto statewright | Aplicación en Campaign Executor v2 | Archivo candidato |
|---------------------|-----------------------------------|-------------------|
| Per-state `allowed_tools` | Cada estado C0 (PLAN, ACT, VERIFY, etc.) define qué tools permite | `iter.md` state machine |
| `num_ctx` configuración | Pasar `--num-ctx 32768` en llamadas Ollama vía MCP | `harness.ps1` o MCP server |
| Raw JSON mode default | Forzar raw JSON tool calling para todos los modelos | MCP server |
| `response_field` parsing | Si el modelo outputea en `reasoning` field, parsear de ahí | MCP server |
| Fork/join para sub-tareas | Cuando lint + test + docs son independientes, fork ear | `pipeline-run.md` |
| Bash command allow-lists | Solo comandos prefix-matching permitidos en VERIFY | `iter.md` VERIFY state |
| Approval gates | Estados que requieren confirmación humana (CLOSE, DEPLOY) | `iter.md` ACCEPT→CLOSE |
| Per-state model routing | PLAN usa Haiku, ACT usa Sonnet, REVIEW usa Opus | `iter.md` + MCP server |

---

## Glosario de términos

| Término | Definición |
|---------|------------|
| Harness | Entorno alrededor del agente: prompts, tools, estado, guardrails, orquestación |
| CAR | Control-Agency-Runtime: 3 dimensiones para documentar un harness |
| Gen_SM | Generate State Machine: modelo genera definición de workflow |
| LLM_Solve | Ejecutar dentro de state machine constraints |
| MoM | Mixture of Models: escalar entre modelos según dificultad |
| Model Traits | Características de cada modelo (tool mode, context window, response field) |
| RED metrics | Rate / Errors / Duration |
| USE metrics | Utilization / Saturation / Errors |
| Handoff artifact | Estado estructurado entre ventanas de contexto |
| Bounded memory | Solo preservar lo esencial (goals, progress, critical files, failing tests) |
| Ratchet | Mecanismo que solo permite movimiento en una dirección (calidad sube, nunca baja) |
| Score movement rehearsal | Probar cambios de pipeline en modo simulación |
| Lurkr | Static scanner para riesgos de agentes AI |
| Fork/Join | Ejecutar ramas en paralelo, unir cuando todas (o cualquier) completen |

---

*Documento generado el 2026-07-15. Fuentes: `.agents/references/` (11 entries).*
