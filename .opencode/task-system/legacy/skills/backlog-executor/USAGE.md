# USAGE — Backlog Executor: Cómo generar y ejecutar prompts

## 3 formas de usarlo

### Forma 1: PowerShell Harness (auto-loop)

```powershell
.opencode\task-system\harness\harness-executor.ps1 -PlanFile docs\plans\mi-plan.md
```

El harness: lee plan → invoca `opencode run` → espera → detecta stall → repite.

### Forma 2: opencode-loop plugin (goal mode)

```bash
/loop-goal "Cargá backlog-executor. Ejecutá el plan file docs/plans/mi-plan.md una tarea a la vez. Después de cada tarea, actualizá el plan file y escribí la recitation."
```

### Forma 3: Manual (una iteración por turno)

```
/loop-goal "Cargá backlog-executor. Usá Prompt 1 del SKILL.md. Plan file: docs/plans/mi-plan.md"
```

## Cómo crear un plan file desde Backlog.md (Prompt 0)

```bash
/loop-goal "Cargá backlog-executor, brainstorming, writing-plans, ponytail (full). Ejecutá Prompt 0 del SKILL.md con las tareas de docs/Backlog.md que sean TIER 0 y tengan Estado ❌. Aplicá el Task Triage Gate a cada una y creá docs/plans/2026-07-14-campaign.md solo con las ✅ DO."
```

Esto:
1. Lee Backlog.md
2. Aplica Triage Gate a cada tarea (DO/DEFER/SKIP/BLOQUEADO)
3. Crea `docs/plans/YYYY-MM-DD-campaign.md` con las aprobadas
4. Cada tarea incluye: ID, archivos clave, contrato verificable, estado ⬜

## Formato de cada entrada en el plan file

```markdown
### Task 1: REV-004 — Fix tantivy rlib test build

- **Fuente:** Backlog.md línea 125
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-openai/Cargo.toml`
- **Gate Result:** ✅ DO
- **Contrato:** "cargo nextest run --no-run --workspace --build-jobs 2 pasa"
- **Estado:** ⬜ PENDING
- **Branch:** `fix/rev-004-tantivy`
```

## Cómo ejecutar UNA tarea específica

Si no querés un plan file completo, usá el Prompt 1 directamente:

```bash
/loop-goal "Cargá backlog-executor, writing-plans, incremental-implementation, ponytail (full). Usá Prompt 1 del SKILL.md. Plan file: docs/plans/mi-plan.md. La primera tarea a ejecutar es REV-004: Fix tantivy rlib test build con contrato 'cargo nextest run --no-run --workspace pasa'."
```

Esto ejecuta exactamente una iteración (Plan → Act → Verify → actualizar plan → recitation → yield).

## Mapa de prompts

| Prompt | Cuándo usarlo | Comando |
|--------|--------------|---------|
| **Prompt 0** | Crear plan file desde backlog | `"Ejecutá Prompt 0 del SKILL.md con <ruta>"` |
| **Prompt 1** | Una iteración (con harness) | `"Usá Prompt 1 del SKILL.md. Plan file: <ruta>"` |
| **Prompt 2** | Arrancar harness PowerShell | `.opencode\task-system\harness\harness-executor.ps1 -PlanFile <ruta>` |
| **/loop-goal** | Loop autónomo sin harness | `"/loop-goal <objetivo>"` |

## Contractos válidos (copiar/pegar)

| Tipo | Contrato |
|------|----------|
| Rust build | `"cargo check --workspace pasa"` |
| Rust full | `"cargo nextest run --profile audit --workspace --build-jobs 2 pasa"` |
| Rust clippy | `"cargo clippy --workspace --all-targets --all-features -- -D warnings pasa"` |
| Rust fmt | `"cargo fmt --check pasa"` |
| Web typecheck | `"npx tsc --noEmit pasa"` |
| Web lint | `"npx eslint . --ext .ts,.tsx pasa (0 errors, 0 warnings)"` |
| Rust + lint | `"cargo check --workspace && cargo clippy -- -D warnings && cargo fmt --check pasa"` |
| Python tests | `"python -m pytest vantadb-python/tests/ -v pasa"` |
