# REVIEW-18: Warning Next.js package-lock stray fuera del repo (turbopack root)

## Metadata
- **Plan file:** docs/plans/2026-08-23-backlog-triage.md (Task 13 — NO editar; estado trackea este task file)
- **Fuente:** review-full-20260822 H03-CODE-001 · Backlog triage Wave 2
- **Esfuerzo:** 🟢 30min
- **Prioridad:** 🟢
- **Tipo:** Frontend-web (config Next.js 16 / Turbopack)
- **Creado:** 2026-08-23
- **last-synced:** 2026-08-23 (cierre)
- **Estado:** ✅ COMPLETED (`6ea5e545`)

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `web/next.config.ts` (9L), `web/package.json`, `.opencode/rules/frontend-web.md`
- **Referencias hacia dentro:** ninguna en código fuente — `next.config.ts` se carga por convención de Next.js (build/dev/start). `rg -l "next.config"` = solo docs/reviews/plans (documentales).
- **Referencias hacia afuera:** importa `type { NextConfig } from "next"` (solo tipos, cero runtime)
- **Entorno verificado:** node_modules ✅ existe (no requiere `npm ci`) · package-lock legítimo en `web/package-lock.json` · lockfile stray detectado en `C:\Users\Eros\package-lock.json` (FUERA del repo git) · Next **16.3.0** con Turbopack default
- **Veredicto impacto:** BAJO — cambio config-only de 1 archivo; afecta únicamente la inferencia de workspace-root del bundler Turbopack; sin cambios en código, rutas ni output standalone

## DISCOVERY — Reproducción (GATE bug)

- **Repro CONFIRMADO** (`npm run build` en web/, 2026-08-23):
  ```
  ▲ Next.js 16.3.0 (Turbopack)
  - Environments: .env
  ⚠ Warning: Next.js ignored package-lock.json in C:\Users\Eros because it is outside the current Git repository (C:\Users\Eros\VantaDB Proyect\VantaDB).
   To use this directory, set `turbopack.root` in your Next.js config.
  ```
- **Causa raíz:** Turbopack infiere el workspace root caminando hacia arriba buscando lockfiles; encuentra el stray de `C:\Users\Eros\package-lock.json` (fuera del repo) y emite el warning. El propio output del compilador documenta la API del fix.
- **Fix documentado:** `turbopack: { root: __dirname }` en `web/next.config.ts` (patrón canónico Next.js para múltiples lockfiles).

## Contrato

"`npm run build` en web/ SIN el warning turbopack/package-lock (tail limpio como evidencia) + exit 0"

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — N/A: no toca trust boundaries, dependencias, auth, storage ni red (1 línea de config de bundler).
- [x] **PERFORMANCE** — N/A: no toca hot paths (vector/, engine.rs, serialización); no alego perf.

## Steps

### Step 1: Fix config
- **Archivos:** `web/next.config.ts`
- **Acción:** agregar `turbopack: { root: __dirname }`
- **Verify:** `npm run build` en web/ → tail sin warning + exit 0
- **Estado:** ✅ COMPLETED — warning eliminado, build exit 0, 35/35 páginas generadas
- **Evidencia post-fix (tail):**
  ```
  ▲ Next.js 16.3.0 (Turbopack)
  - Environments: .env
  ✓ Running next.config.ts took 24ms
  ...
  ✓ Generating static pages using 11 workers (35/35) in 1089ms
  ```
  Sin línea `⚠ Warning:` — EXIT_CODE=0. Re-verificado vía campaign_verify_cmd (taskId REVIEW-18).

### Step 2: Commit + memoria
- **Archivos:** `web/next.config.ts`, este task file
- **Acción:** commit conventional `fix(web): REVIEW-18 fija turbopack root — warning package-lock stray eliminado` + lesson
- **Verify:** commit `6ea5e545` (hooks pre-commit OK: sin Rust tocado, actionlint ok) · lesson TSYS-15 escrita en `.opencode/task-system/memory/lessons.md`
- **Estado:** ✅ COMPLETED

---

## Recitation final (canónica — plan file NO editado por instrucción del orquestador)

```
Campaign ID: 82c5ed20-2086-4619-b471-dbafeb63aead
Objetivo activo: REVIEW-18: warning turbopack package-lock stray eliminado en build de web/
Estado: completed
Última acción: Warning reproducido en DISCOVERY (Next 16.3.0 Turbopack ignora C:\Users\Eros\package-lock.json fuera del repo); fix de config turbopack.root=__dirname; build limpio exit 0; commit 6ea5e545; migración progreso hecha (fila Backlog eliminada + registro avance/web-frontend.md)
Resultado: OK
Próxima acción: Ninguna para REVIEW-18. Orquestador continúa Wave 2 (REVIEW-08 h2 RUSTSEC o según prioridad)
Contrato: verificacion: npm run build en web/ = tail SIN línea ⚠ Warning, EXIT_CODE=0, 35/35 páginas (re-verificado post-fix) || evidencia || claim: warning reproducido con config actual → output capturado en task file §DISCOVERY ("ignored package-lock.json in C:\Users\Eros") | confianza: alta || claim: fix elimina el warning sin romper el build → build post-fix limpio exit 0, output standalone intacto | evidencia: bash EXIT_CODE=0 | confianza: alta || artefactos: web/next.config.ts, .opencode/skills/campaign-executor/tasks/REVIEW-18.md, docs/Backlog.md (−1 fila), docs/avance/activo/web-frontend.md (+REVIEW-18), commits 6ea5e545 + <docs> || invariantes: output standalone intacto; reactStrictMode/output sin cambios de semántica; lockfile stray fuera del repo NO tocado (fuera de jurisdicción) || deuda: ninguna introducida || queda_pendiente: campaign_verify_cmd MCP inusable esta sesión por planes ambiguos (schema sin planFile) — verificación corrida directa en terminal (precedente MOD-17); fila Backlog de REVIEW-15 (completada hoy por otro worker) sigue presente — orquestador decide
Próxima tarea si completa: REVIEW-08 (Wave 2)
```

## Notas
- El lockfile stray en `C:\Users\Eros\` está FUERA del repo — no se puede (ni se debe) tocar desde aquí; `turbopack.root` es el fix correcto y portable (CI/otras máquinas sin el stray también quedan explícitas).
- No se usa `next lint` ni webpack — Next 16 es Turbopack-only por defecto; la opción aplica al build actual verificado.
