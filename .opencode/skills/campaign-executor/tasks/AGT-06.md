# AGT-06: Script anti-drift de referencias (check-agents-refs.ps1)

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md (Task 13)
- **Fuente:** docs/Backlog.md (Wave 4)
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟢
- **Tipo:** Tooling / Docs (vanta-docs)
- **Turns estimados:** 5
- **Creado:** 2026-08-25
- **last-synced:** 2026-08-25
- **Estado:** ✅ COMPLETED (implementación — lead verifica mecánico y commitea)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `dev-tools/verify_changed.ps1` (nuevo hook que invoca el script cuando AGENTS.md cambia) |
| Callees | Ninguno — script standalone de solo lectura; no importa módulos |
| Implicaciones | NO es código de negocio. Añade un check anti-drift que falla (exit 1) si una ruta citada en backticks de AGENTS.md no existe. No rompe builds ni API; solo documenta |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/AGENTS.md` (contenido completo inyectado en contexto + extract de backticks vía regex), `AGENTS.md` (raíz, 5 tokens), `dev-tools/verify_changed.ps1` (45 líneas, patrón de hook `run` + docs-coverage), `dev-tools/verify.ps1` (pre-flight), `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md`, task files AGT-02.md/AGT-04.md (formato)
- **Archivos referenciados hacia dentro:** `.opencode/AGENTS.md` y `AGENTS.md` citan rutas en backticks (`.opencode/commands/*.md`, `docs/*`, `dev-tools/*.ps1`, `references/*.md`, etc.) — el script las valida contra el repo
- **Archivos que referencian a los editados (referencias entrantes):** `dev-tools/check-agents-refs.ps1` (nuevo) — nadie lo referencia todavía; `dev-tools/verify_changed.ps1` lo invocará (hook condicional). El fix de `.opencode/AGENTS.md:78` (referencia rota `references/understand-anything.md` → `.opencode/references/understand-anything.md`) no tiene otros referenciadores — es un literal markdown; grep de "understand-anything" solo aparece en línea 78
- **Veredicto impacto:** BAJO — archivo nuevo standalone (script de tooling) + 1 línea editada en verify_changed.ps1 + 1 literal corregido en .opencode/AGENTS.md. Nada se rompe

## Contrato
"Script existe, valida refs, enganchado a verify_changed.ps1" — verify:
1. `dev-tools/check-agents-refs.ps1` existe y corre: `pwsh -NoProfile dev-tools/check-agents-refs.ps1` → exit 0 con "OK" (0 stale reales)
2. Anti-drift detecta drift: inyectar una ruta fake y comprobar que reporta `MISSING` + exit 1 (test manual del mecanismo)
3. Hook en `dev-tools/verify_changed.ps1` presente (invoca el script cuando `AGENTS.md`/`.opencode/AGENTS.md` cambió)
4. Comando documentado en el task file / salida del script

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** el script es SOLO LECTURA (no muta nada). NO edita AGENTS.md (salvo el fix puntual de la referencia rota, fuera del mecanismo). No colisiona con AGT-03 (PENDING, scope = deuda P2 Regla 6, sección disjunta de Understand-Anything). `verify_changed.ps1` mantiene su semántica: el nuevo check corre SOLO si `AGENTS.md`/`.opencode/AGENTS.md` cambió (mantiene fast gate ~30s)
- **Comandos de verificación:** `pwsh -NoProfile dev-tools/check-agents-refs.ps1` → `check-agents-refs: OK (N refs)` exit 0; test negativo con ruta fake → `MISSING: ...` exit 1
- **Deuda pendiente:** filtro conservador solo valida rutas con extensión de archivo; refs de directorios sin extensión (ej `.githooks/pre-push`) se validan en silencio (si existen) pero NO se reportan si faltan — evita falsos positivos deliberadamente

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (script standalone de tooling, 0 lógica de negocio). Fix de deuda: corrige 1 referencia rota real (`.opencode/AGENTS.md:78` → `.opencode/references/understand-anything.md`). El mecanismo en sí previene futura deuda de refs stale (el caso "917 líneas" del backlog).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: script existe + corre exit 0 + hook en verify_changed.ps1 + mecanismo anti-drift probado (test negativo) |
| **Commit** | Lead verifica mecánico y commitea (sub-agente NO commitea — regla del plan) |
| **Release** | N/A — tooling interno, sin release (justificado en Notas) |

## Herramientas necesarias
- PowerShell 7+ (pwsh) — regex + Test-Path stdlib, sin dependencias externas

## Investigation Notes
- Probe de extracción de backticks sobre `.opencode/AGENTS.md` + `AGENTS.md`: 21 candidatos con extensión; 5 MISSING pero 4 son placeholders (`prompts/X.md`, `tasks/X.md`, `.opencode/task-system/prompts/X.md`, `docs/plans/X.md` — patrones de ejemplo con `X`), el único stale REAL es `references/understand-anything.md` (el archivo existe en `.opencode/references/understand-anything.md`; la referencia en AGENTS.md omite el prefijo `.opencode/`).
- Filtro anti-falsos-positivos: descarta tokens con whitespace (comandos/prosa), URLs (`://`), `~` (externo), `/` inicial (absoluto), `[<>*?#]` (placeholder/glob), sin `/` (no es ruta: `P2-1`, `cargo`, `cli,fjall`), leaf placeholder `X.md`, y sin extensión de archivo.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — diseño validado con probe real |
| Pendientes de ejecución (downhill) | 1 step |
| % completado | 50% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — NO aplica: script de solo lectura sobre markdown de docs; sin input de usuario, auth, dependencias nuevas, storage ni red.
- [ ] **PERFORMANCE** — NO aplica: no toca hot paths ni índices; corre en <1s sobre 2 archivos markdown; enganchado solo cuando AGENTS.md cambia.

## Steps

### Step 1: Crear script + hook + fix ref rota + verificar
- **Archivos:** `dev-tools/check-agents-refs.ps1` (nuevo), `dev-tools/verify_changed.ps1` (hook), `.opencode/AGENTS.md:78` (fix ref rota)
- **Acción:** (1) crear script anti-drift standalone; (2) enganchar a verify_changed.ps1 (check condicional cuando AGENTS.md cambió, patrón docs-coverage); (3) corregir `references/understand-anything.md` → `.opencode/references/understand-anything.md` para que el script reporte 0 stale reales
- **Verify:** `pwsh -NoProfile dev-tools/check-agents-refs.ps1` → `check-agents-refs: OK (22 refs)` exit 0 ✅; test negativo: inyección temporal de `` `fake/path/does-not-exist.md` `` → `MISSING: fake/path/does-not-exist.md` exit 1 ✅ (revertido, re-run exit 0); `Select-String -Path dev-tools/verify_changed.ps1 -Pattern 'agents-refs'` → hook presente (L41-47) ✅; `Select-String -Path .opencode/AGENTS.md -Pattern 'understand-anything'` → L78 apunta a `.opencode/references/understand-anything.md` ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna — tarea independiente (Wave 4). AGT-03 (PENDING) es disjunto (scope P2 Regla 6)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review / lead (verifica mecánico al commitear — regla del plan: sub-agentes NO commitean)
- **Enfoque:** ¿el script evita falsos positivos y detecta el drift real (refs rota de AGENTS.md)? ¿el hook preserva la semántica del fast gate?
- **Cómo se probó:** corrida real exit 0 + test negativo (ruta fake → MISSING exit 1); hook verificado por grep
- **Checklist anti-hábitos tóxicos:** N/A para revisión del lead — comando reproducible
- **Veredicto:** pendiente del lead

## Notas
- Scope estricto: NO es código de negocio; es tooling de docs (vanta-docs). El único edit a AGENTS.md es el fix de la referencia rota (necesario para que el script reporte 0 stale y el hook no rompa verify_changed); disjunto de AGT-03.
- El filtro es deliberadamente conservador (solo extensiones de archivo) para priorizar 0 falsos positivos sobre cobertura total — techo conocido documentado en Deuda pendiente.
- Release N/A: herramienta interna sin release.
