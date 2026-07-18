---
description: "Automatic rollback: recover from a failed ship or broken deployment by reverting to the last known-good state"
---

Invoke the agent-skills:shipping-and-launch skill (rollback section).

Execute a rollback to recover from a failed ship or broken deployment. Follow these steps in order.

> **Prerequisite:** `/ship` must have been run before. If there's no ship report, git history is the fallback.

## Phase 1 — Discover the last ship state

1. Buscá el último reporte de `/ship`:
   - `docs/ship-reports/ship-*.md` (más reciente por fecha)
   - `docs/last-ship-state.json` (state file resumido)
   - `git log --oneline -20 --grep:"ship:"` (fallback)
2. Identificá:
   - El commit que se shippeó (SHA)
   - El rollback plan documentado (si el ship fue GO y tiene rollback procedure)
   - Las condiciones de trigger que se cumplieron (¿por qué se necesita rollback?)
   - Feature flags, migraciones, o configuraciones que se aplicaron

## Phase 2 — Validate rollback preconditions

1. Verificá que el working tree esté limpio: `git status --porcelain`
   - Si hay cambios sin commit, preguntá al usuario si quiere stash o abortar
2. Determiná el target SHA:
   - Si existe rollback plan con SHA anterior → usá ese
   - Si no → `git log --oneline -5` y preguntá al usuario cuál es el target
3. Verificá que el target SHA sea un ancestor del HEAD actual
   - `git merge-base --is-ancestor <target> HEAD` debe devolver 0
4. **Unrevertible operations check:** si el ship incluyó migraciones destructivas (DROP TABLE, DELETE sin WHERE), abortá con advertencia — requiere intervención manual.

## Phase 3 — Execute the revert

1. **Git revert:** `git revert --no-commit <target-sha>..HEAD`
   - Si hay conflictos, abortá: `git merge --abort` y reportá los archivos en conflicto
2. **Restaurá configuraciones:** si el ship cambió feature flags, env vars, o configs, revertilos manualmente (`.env`, `config/*`, feature flags en DB)
3. **Deshacé migraciones de datos:** si el ship incluyó migraciones, ejecutá el rollback de migración si existe (buscar en `migrations/` o `docs/operations/`)
4. **Compilá y testeá:** `just verify` o `just verify-quick`
   - Si falla, abortá con `git reset --hard HEAD` y reportá el error

## Phase 4 — Finalize

1. **Commit the rollback:** `git commit -m "rollback: revert <descripción corta>"`
2. **Health check:** `just check` + servicios básicos funcionando
3. **Documentá en** `docs/rollback-reports/rollback-<timestamp>.md`:
   - Causa del rollback
   - SHA revertido y SHA restaurado
   - Lecciones aprendidas
   - Link al reporte de `/ship` original

4. **Próximo paso:** recomendá al usuario cómo reiniciar el ciclo:
   ```
   Rollback completado. Para continuar:
     /status          → dashboard post-rollback
     /pipeline plan   → re-planificar desde backlog
     /build           → retomar desarrollo
   ```

## Rules

1. Unrevertible operations (data loss, destructive migrations) must abort with a warning — manual intervention required.
2. If the rollback plan from `/ship` specified exact steps, follow them — they take priority over the generic procedure.
3. If any step in Phase 3 fails, stop immediately and report what was done vs undone.
