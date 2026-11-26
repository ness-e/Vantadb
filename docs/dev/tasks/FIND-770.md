---
title: "FIND-770 - ci-rust filtra pull_request: 10 de los 14 required de develop quedan en Pending"
kind: task
status: in-progress
description: "`ci-rust.yml` filtra `pull_request` por 22 patrones: 1280 ficheros (54,4 %) -959 bajo `docs/`- no disparan el workflow, y 10 de los 14 contexts required de `develop` salen de ahi. Fix: `paths` fuera de `pull_request` + job `rust-scope` fail-open que omite por `if:` los jobs caros (9 saltan en cascada por `needs`). Coste medido: ~570 s frente a 5670 s de los 3 tests."
tags: [vantadb, ci, github-actions, required-checks, paths-filter]
---

# FIND-770 - `ci-rust.yml` filtra `pull_request`: 10 de los 14 required contexts de `develop` quedan en `Pending`

## Contexto

`FIND-765` (`49ced681e`) quitó `paths` de `on.pull_request` en `docs-gates.yml`
y midió el coste: 16 jobs, 967 s de runner, 10m38s de pared, y los 3 required
son 37 s. El mismo commit **declaró el residual** (`ci-cd.md:1131`): "10 de los
14 contexts salen de `ci-rust.yml`, que conserva su `pull_request.paths` (22
patrones) y **no** es escribible ahí por conflicto de carril". Este es ese
carril.

El ruleset `develop` (`23692587`, leído por API hoy) tiene **14** contexts
required; **10** vienen de este fichero:

```
Format Check                    Clippy Lints
Tests (Linux)                   Tests (Windows)
Tests (macOS)                   MSRV Check (1.95.0)
Experimental Crates Check       Security Audit
Miri (UB Detection)             Dependency Policy Check
```

Los otros 4 son de `docs-gates.yml` (3 de `FIND-765` + `rustdoc` de
`FIND-763`). Sin `paths` fuera de `pull_request`, un PR que sólo toque
`docs/**` **no arranca** este workflow, no crea check-run, y esos 10 contexts
quedan en `Pending` para siempre: el deadlock de `FIND-714`, por la puerta del
disparador.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-rust.yml` (1018 líneas),
  `dev-tools/git-commit-isolated.ps1` (155 líneas),
  `docs/dev/tasks/FIND-765.md` (precedente), `docs/dev/avance/activo/ci-cd.md`
  §FIND-765 (`:1100-1136`).
- **Referencias hacia dentro:** `on.push.paths` y `on.pull_request.paths` son
  la única fuente del disparo; `.github/actions/rust-setup` es el composite que
  usan los jobs (no se toca). `ci-gate.yml` tiene una lista `REQUIRED` de 19
  contexts y **tolera** un context ausente con `WARN` (política EST-03), así
  que no se toca.
- **Referencias entrantes:** ruleset `develop` `23692587` (14 contexts, por
  nombre exacto de job), `docs/dev/operations/CI_POLICY.md` (describe el
  filtro), `docs/dev/avance/activo/ci-cd.md:1131` (declara el residual).
- **Veredicto impacto:** **medio**. El riesgo no es de código sino de
  satisfacibilidad: si un job **required** dejara de reportar (por `if:` mal
  puesto o por una cadena de `needs` que no cascada), se reabre el deadlock por
  la otra puerta. Mitigación verificable: los 14 contexts se comprueban
  nombre-a-nombre contra el YAML parseado (§Contrato).

## Contrato

(a) `on.pull_request` sin `paths`, `on.push` con sus 24 patrones intactos;
(b) `actionlint` exit 0; (c) YAML parseado: los **14** contexts del ruleset
`23692587` siguen teniendo un job que los reporte; (d) el coste del fix medido
contra el run más reciente que corrió los 3 tests.

## Spec (tabla de decisiones)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Filtro en `pull_request` | (a) quitarlo y guardar por job · (b) quitarlo y aceptar el coste · (c) ampliar los 22 patrones | (a) | **(a)**. (b) medido: 5670 s de runner sólo en los 3 tests (658+3426+1586) y 1h09m de pared por PR docs-only — el remedio peor que la enfermedad. (c) es el treadmill que `FIND-765` ya rechazó: re-enumerar con cada directorio nuevo, y el patrón que faltaba siempre es el que no conocías. |
| 2 | Granularidad del guard | (a) `if:` en cada job caro · (b) `if:` sólo en el primero de la cadena + cascadeo por `needs` | (b) | **(b)**, y no por elegancia: `workflow-syntax §jobs.<job_id>.needs` dice *"If a job fails or is skipped, all jobs that need it are skipped"*. Un guard en `clippy` omite 9 jobs (tests Linux/Windows/macOS, embeddings, coverage, miri, asan, experimental, release-combo) porque los 9 lo llevan en `needs`. 1 condición probada contra 9 sin probar. |
| 3 | Dirección del fallo del scope | fail-open (correr de más) · fail-closed (omitir) | fail-open | **fail-open**, calcado de `wasm-scope` (`FIND-725`): un falso positivo cuesta minutos de runner; un falso negativo pierde una suite entera **en silencio** (`skipped`, no `failure`). |
| 4 | Lista de patrones del scope | los 22 de PR · los 24 de push | 24 | **24** (`on.push.paths` es superset: `.cargo/**` y `.github/actions/rust-setup/**` eran sólo de push). Un PR que toque `.cargo/**` hoy **no** disparaba nada — con esto dispara los gates. Cambio de comportamiento declarado, en la dirección estricta. |
| 5 | `is_required` de los omitidos | quitarlos del ruleset · dejarlos required | dejarlos | **dejarlos**: `status-checks` dice *"A job that is skipped will report its status as 'Success'. It will not prevent a pull request from merging, even if it is a required check"*. El ruleset es configuración del repo por API, no un fichero: fuera de alcance. |

## Steps

1. [x] Medir el coste real por job (run `45821093876`) y contar el residual.
2. [x] Quitar `paths` de `on.pull_request`; dejar `on.push` intacto (24 patrones).
3. [x] Añadir el job `rust-scope` (fail-open) y el guard en `clippy`,
       `semver-checks`, `public-api-snapshot`. Los 9 jobs con `needs: clippy` se
       omiten por cascada, no por 9 `if:` copiados.
4. [x] Verificar: `actionlint` + YAML parseado + 14 contexts nombre-a-nombre.
5. [ ] Cerrar: entrada en `ci-cd.md`, fila fuera del Backlog, commit.

## Medición (run `45821093876`, `3a7e225ad`, push a `develop`, success)

| Job | s | ¿required? |
|---|---:|---|
| Tests (Windows) | 3426 | sí |
| Tests (macOS) | 1586 | sí |
| ASan | 812 | no |
| Clippy Lints | 671 | sí |
| Tests (Linux) | 658 | sí |
| Code Coverage | 537 | no |
| Tests (embeddings) ×2 | 476 | no |
| Miri | 222 | sí |
| MSRV | 114 | sí |
| Release Combo | 111 | no |
| Minimal Versions | 111 | no |
| Experimental Crates | 136 | sí |
| Unused Deps | 70 | no |
| Security Audit | 36 | sí |
| Dependency Policy | 32 | sí |
| Secret Scan | 25 | no |
| Format Check | 23 | sí |
| OSV-Scanner | 9 | no |
| WASM Scope | 7 | no |

Run completo: **4175 s de pared** (12:35:36 → 13:45:11). Los 3 tests required
solos: **5670 s** de runner y **~57 min** de pared, y no arrancan hasta que
`fmt`+`clippy`+`secret-scan` terminan.

Residual fuera del filtro (medido sobre `HEAD`, 2353 ficheros tracked, con la
semántica de GitHub: `*` no cruza `/`, `**` sí): **1280 (54,4 %)**, de ellos
**959 bajo `docs/`**.

## Verificación

| Check | Comando | Resultado |
|---|---|---|
| Lint de workflow | `actionlint .github/workflows/ci-rust.yml` | **exit 0** (baseline previo también 0) |
| YAML parseado | `pyyaml` sobre el fichero | `on.pull_request` = `{branches: [main, develop]}` **sin** `paths`; `on.push.paths` = **24** intactos; **24 jobs**; 0 `name:` duplicados |
| 14 contexts nombre-a-nombre | `find770_check.py` contra el ruleset `23692587` | **14/14** con job; 6 de los 10 de `ci-rust` `skipped`, 4 `runs`; los 4 de `docs-gates` vivos |
| ERE del scope | `grep -E` real sobre la regex **extraída del workflow** | **46/46** (24 que deben disparar, 17 que no, 6 trampas de prefijo) |
| Script del scope, E2E | el `run:` real extraído del YAML, `git fetch` neutralizado por PATH | **5/5**: push→true, dispatch→true, sin base→true, PR docs-only→false, PR con `src/`→true |
| Docs | `check-docs.mjs` | exit 0 (FIND-770 aparece como *orphan* listado, no gateado — igual que `FIND-118`) |
| Links | `check-links.mjs` | exit 1 por **3 roturas preexistentes** (`ADR-0066` → `../../tasks/FIND-756.md` ×3), ninguna de esta tarea |

**No se corrió `just verify`**: el carril es CI y no compila (cero `.rs` tocados).

### Lo que NO se pudo verificar (declarado)

No se puede abrir el PR de prueba ni disparar un `pull_request` desde acá: el
contrato **vivo** (`gh pr checks` sin ningún `Expected` a los 10 min) queda
**sin observar**. Se verificó la causa (el filtro ya no está en el disparador) y
la coincidencia nombre-a-nombre, no el efecto en un PR real. Tampoco se observó
la primera corrida de `rust-scope` (nunca ha corrido).

## Notas de implementación

- **Una condición, nueve jobs omitidos.** El guard vive en `clippy` (primer
  eslabón de la cadena de compilación); los 9 jobs que lo llevan en `needs`
  (tests Linux/Windows/macOS, embeddings, coverage, miri, asan, experimental,
  release-combo) se omiten **por dependencia**. Poner 9 `if:` a mano habría sido
  9 superficies donde equivocarse contra 1 probada.
- **`!cancelled()` no es decorativo:** sin una función de status en el `if:`, un
  `needs` omitido omite siempre al dependiente y el guard nunca se evalúa.
- **`grep -qE` con `<<<`, no con pipe:** bajo `set -o pipefail`, un `grep -q`
  que sale en el primer match cierra el pipe, `printf` muere con SIGPIPE (141) y
  la respuesta sería `rust=false` con un PR que **sí** toca Rust — fallo
  silencioso en la dirección catastrófica. Y `rc=2` (regex rota) se trata como
  fail-open: tomarlo por "sin match" apagaría todos los gates por un typo.
- **El scope no lleva `if:` propio:** en `push` sale por el primer branch del
  script. Un `if: event_name == 'pull_request'` lo dejaría `skipped` en cada
  push y arrastraría a `clippy` y su cadena: el deadlock por la puerta del guard.
- **Concurrencia de carriles:** al cerrar, el worktree tenía **dos filas más
  retiradas** por otros carriles (`FIND-118`, cuya task file nació a las 12:18;
  y `FIND-771`, la sección entera). Este commit se construyó desde `HEAD` menos
  **solo** mi sección para no llevarse trabajo ajeno; esas dos retiradas siguen
  en el worktree, intactas, para sus dueños.

## Review

Pendiente de la ronda P2-01 (contexto fresco).
