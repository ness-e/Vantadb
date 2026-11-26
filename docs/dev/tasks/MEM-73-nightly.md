---
title: "MEM-73-NIGHTLY: run the L0→L3 e2e on a schedule"
kind: task
status: completed
description: "Wire the ignored MEM-73 e2e into the daily nightly lane through the existing dev-tools/e2e-memory-cycle.ps1 entry point, with no continue-on-error, so the composed cycle cannot rot unobserved"
---

# MEM-73-NIGHTLY: run the L0→L3 e2e on a schedule

## Metadata

- **Origen:** recomendación del review P2-01 de `MEM-73` (`docs/dev/tasks/MEM-73.md` §Review)
- **Dependencias:** `MEM-73` ✅ `c3e9c098e` · `FIND-741` ✅ `e8df96232`
- **Tipo:** CI (workflow programado) + fila en el mapa de CI
- **Estado:** ✅ COMPLETED
- **Creado:** 2026-11-25

## Techo y contrato

`MEM-73` probó el ciclo compuesto L0→L3 en un proceso real, y lo dejó `#[ignore]`
con un carril manual (`dev-tools/e2e-memory-cycle.ps1`). Ese e2e sólo corre
cuando alguien se acuerda: un test que nadie dispara se rompe en silencio, y su
verde deja de significar algo. Esta tarea lo enchufa al lane **programado** —
sin meterlo en push/PR.

### Lo que ya existe y NO se re-diseña

| Pieza | Estado | Rol en esta tarea |
|---|---|---|
| `vantadb-mcp/tests/l0_l3_cycle_e2e.rs` | ✅ `c3e9c098e` | **no se toca**. Es el test que se cablea |
| `dev-tools/e2e-memory-cycle.ps1` | ✅ | **punto de entrada único** (build → gate de SHA → e2e). No se duplica en YAML |
| `FIND-741` (`--version` dice `commit <sha>`) | ✅ `e8df96232` | el gate de SHA del script funciona en CI porque `actions/checkout` provee `.git` |
| `nightly.yml` + `notify-failure` | ✅ HARD-02 | el job entra acá; el auto-issue ya existe |

### Contrato mecánico

```
actionlint .github/workflows/nightly.yml                # gate del repo (pre-commit/pre-push)
pwsh -NoProfile -File dev-tools/e2e-memory-cycle.ps1    # el comando que el job corre
```

Lo que **no** se puede verificar en local: disparar el `schedule`. Se verifica
sintaxis, ubicación del job en el lane programado, y que el comando del job sea
exactamente el que corre en local.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.github/workflows/nightly.yml` (826L),
  `.github/actions/rust-setup/action.yml` (136L), `dev-tools/e2e-memory-cycle.ps1` (44L),
  `vantadb-mcp/tests/l0_l3_cycle_e2e.rs` (459L, lectura), `vantadb-server/build.rs`,
  `.config/nextest.toml` (`[profile.audit]`), `docs/dev/operations/CI_POLICY.md`
  (§Inventory, §2b, §External Dependencies), `.githooks/pre-commit`/`pre-push`
  (bloque actionlint), `docs/dev/workflow/TRIGGERS.md` (fila `nightly.yml`).
- **Referencias hacia dentro de lo tocado:** `nightly.yml` es referenciado por
  `TRIGGERS.md` (matriz por archivo, no por job: **no cambia**), `CI_POLICY.md` §2b
  (tabla job→certification, **sí cambia**) y el badge del README (por workflow, no
  por job). `dev-tools/e2e-memory-cycle.ps1` no lo importa nadie (se invocaba a
  mano); pasa a tener un segundo invocador y sigue siendo la única fuente de verdad.
- **Referencias entrantes / blast radius de producción:** ninguna. Cero cambios en
  `src/**`, `vanta-memory/**`, `vantadb-mcp/**`. Un job nuevo en un workflow
  programado no corre en PR ni en push.
- **Veredicto:** BAJO. Un job aditivo + una fila de tabla + un task file.

## Spec (decisiones)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | ¿Workflow nuevo o `nightly.yml`? | A) `memory-e2e-nightly.yml` nuevo (otra programación que mantener, otro badge, otro `notify`) / B) job en `nightly.yml` | **B** — `nightly.yml` es el lane programado diario y el punto de recepción designado de gates lentos; su `notify-failure` ya abre issue deduplicado |
| 2 | ¿Trigger? | A) push/PR (envenena el CI de todos con un e2e que arranca procesos — clase `FIND-727`/`FIND-739`) / B) `schedule`/`dispatch` | **B** — gate `if: github.event_name != 'pull_request'`, el patrón que ya usan los heavies del archivo (el `pull_request:` de `nightly.yml` matchea `src/**`, así que sin el gate correría en cada PR) |
| 3 | ¿`continue-on-error`? | A) sí (decorativo) / B) no | **B** — un e2e que se puede ignorar es el gate que no mide. Si es ruidoso se arregla el test |
| 4 | ¿Reusar el `.ps1` o portarlo a bash? | A) portar (dos implementaciones que se desincronizan) / B) `pwsh -NoProfile -File` sobre el guion existente | **B** — PowerShell 7 viene en `ubuntu-latest` y este mismo workflow ya lo usa (`coverage-budget`, `check_completions`). Una sola fuente de verdad para build+gate |
| 5 | Plataforma | A) Linux / B) Linux + Windows | **A** — el e2e es OS-agnóstico (spawnea el binario, `git`, un puerto efímero en 127.0.0.1); el carril local en Windows ya lo cubre. Duplicar el runner duplica superficie de flake (spawn+socket) sin señal nueva |
| 6 | `timeout-minutes` | A) 60 "por si acaso" / B) con ancla explícita | **B** — 30 min: el e2e está medido (1,45 s local; MEM-73 registró 3,5-4,2 s wall), el build **no** (no hay corrida de CI todavía). Ancla disponible: la corrida local completa del comando del job (2026-11-25, Windows/MSVC, `target/` caliente) = **525 s**, y la mitad de build es más que un binario porque nextest construye los 27 targets de `vantadb-mcp` antes de filtrar, más un compile de dependencias en un checkout frío. 30 min = ~3,4× ese piso y coincide con el techo de `failpoint-tests` (30 min), que linkea menos binarios de test. La primera corrida programada es la medición para ajustar |

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Job `memory-cycle-e2e` en `nightly.yml` (gate PR, `timeout`, `install-nextest`, reuso del `.ps1`) | ✅ |
| 2 | Job en `needs` de `notify-failure` (el fallo abre issue) | ✅ |
| 3 | Cabecera de `nightly.yml` + fila en `CI_POLICY.md` §2b | ✅ |
| 4 | Verify: `actionlint` + correr el comando del job una sola vez | ✅ |
| 5 | Review P2-01 (agente distinto) | ✅ |

## Verificación

```
actionlint .github/workflows/nightly.yml                         → exit 0
python -c "yaml.safe_load(...)" (PyYAML, 2do parser)             → 16 jobs; el job nuevo:
   name "Nightly: memory L0 -> L3 e2e (shipped binary)", if github.event_name != 'pull_request',
   timeout 30, continue-on-error AUSENTE; notify-failure.needs lo incluye
node scripts/docs/check-docs.mjs                                 → GATING: all clear; orphans 0
node scripts/docs/check-links.mjs                                → 0 broken (1 wikilink pre-existente, budget)
npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml    → 0 files (docs/dev/tasks/** excluido por config)

comando del job, corrido UNA vez en local (declarado):
  pwsh -NoProfile -File dev-tools/e2e-memory-cycle.ps1
  → [1/3] build → [2/3] "MEM-73 e2e: L0→L3 in 1.4504179s (pass ran, queue drained, scene=1, persona=true)"
  → Summary [8.410s] 1 passed, 200 skipped → [3/3] ok → exit 0
  → ELAPSED_TOTAL_SEC=525.4   (de ahí sale el timeout-minutes: 30)
```

Gate de SHA bajo `fetch-depth: 1` (preocupación de CI, verificada aparte): un
clone `--depth 1` de `develop` resuelve `HEAD` con el mismo valor que el repo
completo (la **longitud** de la abreviatura cambia — 7 vs 9 caracteres —, pero
los **dos lados** del gate, `--version` y `git rev-parse --short HEAD`, la toman
de la misma copia, así que comparan lo mismo).

**NO verificado (no se puede desde acá):** disparar el `schedule`. No hay forma de
ejecutar el nightly desde este entorno; lo que sí se verificó es que el job está en
el workflow que tiene el `cron` (`0 5 * * *`), que su `if:` lo excluye de
`pull_request`, y que el comando del job es el mismo que corre en local.

**Observación fuera de scope (no tocado):** `node scripts/docs/gen-index.mjs --check`
falla por drift **pre-existente** en la lista de ADRs de `docs/index.md`
(`docs/index.md` no está modificado en el working tree, así que el drift es de HEAD,
no de este cambio). `NOTICED BUT NOT TOUCHING: docs/index.md → regenerar con gen-index.mjs --write, tarea aparte.`

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (contexto fresco, sin autoría sobre el cambio), 3 rondas cortas.
  Tier **Fast** (paths: `.github/workflows/**`, `docs/dev/**`) → spot-check mecánico, sin
  protocolo adversarial completo.
- **Ejes:** (a) ¿puede correr en un PR y envenenar el fast gate? (b) ¿puede quedar verde sin
  correr el e2e, o contra un binario stale? (c) ¿el `timeout` tiene fuente? (d) ¿algún
  comando duplica al `.ps1`? (e) ¿un rojo llega a un humano? (f) ¿la fila de CI_POLICY dice
  la verdad?
- **Cómo se probó:** lectura del diff + `nightly.yml` completo, traza de `needs` →
  `notify-failure` → `github-script`, lectura de `vantadb-server/build.rs` y de
  `.config/nextest.toml` `[profile.audit]`, y conteo de targets (`vantadb-mcp/tests/*.rs` = 27).
- **Veredicto ronda 1:** 🔴 REQUEST CHANGES — 2 required:
  1. El comentario del timeout decía "measured not guessed" midiendo sólo el e2e (1,4 s), y
     anclaba en `failpoint-tests` "compilando los targets del paquete `vantadb`" — falso:
     nombra 3 targets (`nightly.yml:132-138`). Además se autocontradecía ("el primer run es
     la medición que reemplaza esta estimación").
  2. `Step 5 Review ✅` con la sección `## Review` vacía: gate marcado sin dictamen.
- **Aplicado:** timeout reescrito con procedencia explícita (525 s = corrida local
  **caliente**, "a floor, not the CI figure"; el build de CI es estimación hasta la primera
  corrida), mismo encuadre en `CI_POLICY.md`, Step 5 desmarcado, y esta sección escrita.
- **Re-review:** ✅ **approve** — 4/4 claims re-probados sobre los bytes actuales (`if:`
  gating; `pwsh` en ubuntu por precedente en el mismo archivo; nextest construye todos los
  targets antes de filtrar; 27 targets; y el issue automático vía `needs` + cuerpo que lista
  los jobs fallidos). **Opcional non-finding aplicado:** se quitó "same dependency graph"
  (era aproximado: `failpoint-tests` activa `--features failpoints`) — el cambio sólo
  elimina una afirmación, no agrega ninguna.

## Artefactos

- `.github/workflows/nightly.yml` — job `memory-cycle-e2e` (+ `needs` de `notify-failure` + cabecera).
- `docs/dev/operations/CI_POLICY.md` — §2b: fila del job + bloque de justificación.
- `docs/dev/tasks/MEM-73-nightly.md` — este archivo.
- `docs/dev/tasks/MEM-73.md` — 4 líneas al cierre de §Review apuntando acá.
