# TSYS11-ENFORCE-05 — Alinear 9 permission: blocks a tabla canónica AGENTS.md

> Campaign: TSYS-11 (policy objetivo: ningún sub-agente escala a tools del lead)
> Estado: ⏳ IN PROGRESS
> Contrato: los 9 `permission:` alineados a tabla canónica AGENTS.md §Límites por rol
> (solo lead git push/commit/release; leafs audit/chaos/tuner/docs/review con task deny +
> edit restringido a su dominio; workers edit+bash solo su dominio),
> verificado por rg de deny en cada archivo y tabla intacta.
> D11: hoy todos otorgan allow amplio (deuda declarada) → escribir denies reales:
> lead único con git mutating, arch/worker/engine edit+bash dominio,
> audit/review solo notas + read-only bash, chaos/tuner solo scripts/bench dominio,
> docs solo docs. Regla 0 (leer 9 completos antes de editar), edits mínimos,
> verify con rg + git diff. NO commitear (handoff al orquestador/lead).

## Spec (mini — config-only, sin código productivo)
- Decisión 1: `allow` se conserva donde tabla dice ✅/⚠️; ⚠️ se documenta con comentario
  `# TSYS11:` en la misma línea (dominio/read-only/research) porque el schema
  allow/deny no expresa path-scoping. Evidencia: frontmatters actuales solo tienen
  allow/deny por tool (+ wildcard en task).
- Decisión 2: `deny` real donde tabla dice ❌ o ⚠️→superficie mínima:
  lead cargo-mcp/rust-analyzer deny→allow (tabla ✅, corrige drift);
  arch metasearch/argus allow→deny; worker metasearch allow→deny;
  chaos metasearch allow→deny + rust-analyzer deny→allow;
  review cargo/rust-analyzer/metasearch/argus allow→deny. Evidencia: tabla L259-269.
- Decisión 3: `pencil_*` explícito en los 9 (docs allow suyo; otros 8 deny) para que
  `rg deny` tenga señal en cada archivo y Extras quede cerrado. Evidencia: solo docs
  tenía pencil_* hoy.
- Decisión 4: `task:` intacto (orquestadores `* deny + vanta-* allow`; leafs `deny`) —
  ya enforced; no se toca. Evidencia: rg task en 9 archivos.
- Decisión 5: tabla AGENTS.md L253-276 INTACTA (contrato de referencia, no se edita).
- Decisión 6: git mutating solo lead se expresa como comentario en `bash:` (git vive en
  bash; no hay tool `git` en el schema) + enforcement rules 1-5 ya existentes en AGENTS.md.

## Impacto mapeado (Regla 0)
- Archivos leídos completos (9):
  - `.opencode/agents/vanta-lead.md` (546L, frontmatter L9-35)
  - `.opencode/agents/vanta-arch.md` (322L, frontmatter L8-33)
  - `.opencode/agents/vanta-worker.md` (457L, frontmatter L8-33)
  - `.opencode/agents/vanta-engine.md` (371L, frontmatter L8-33)
  - `.opencode/agents/vanta-audit.md` (214L, frontmatter L8-30)
  - `.opencode/agents/vanta-chaos.md` (213L, frontmatter L8-30)
  - `.opencode/agents/vanta-tuner.md` (243L, frontmatter L8-30)
  - `.opencode/agents/vanta-docs.md` (412L, frontmatter L8-31)
  - `.opencode/agents/vanta-review.md` (226L, frontmatter L12-34)
  - Referencia contrato: `.opencode/AGENTS.md` L253-276 (tabla + enforcement 1-5) — SOLO lectura.
- Referencias hacia dentro (cada agent file): frontmatter `permission:` → consumido por
  runtime OpenCode (allow/deny por tool); cuerpo usa `task` tool solo en
  orquestadores (lead §8, arch/worker/engine composition); leafs declaran
  `Do not invoke from another persona` / `task: deny`.
- Referencias entrantes: `.opencode/AGENTS.md` §Orchestration + §Límites (contrato),
  `docs/research/2026-08-10-agent-engineering/agent-03-orchestration.md` §9.2 (fuente),
  `.opencode/references/task-system.md` (integración), comandos `/pipeline task`
  (routing a sub-agentes por tipo).
- Veredicto: blast radius = 9 frontmatters YAML (solo comentarios + flips allow↔deny
  listados en Spec); cero código productivo, cero cambios de cuerpo, cero cambios en
  AGENTS.md; riesgo bajo (si un comment rompiera parse, verify con rg + read lo detecta).
  Edits: 1 por archivo (bloque permission: contiguo).

## Steps
- [x] S1 DISCOVERY: leer 9 completos + tabla, mapear impacto (Regla 0) — este archivo
- [x] S2 ACT: 1 edit por archivo (9 edits, solo bloque permission:) — 9/9 aplicados
- [x] S3 VERIFY: `rg deny` por archivo + `git diff --stat` + `git diff` (tabla intacta) + NO commit

## SDP
- SDP: base-only (config-only, sin keywords de código; skills de ingeniería no aplican a
  frontmatter YAML; contrato = tabla AGENTS.md ya citada)
