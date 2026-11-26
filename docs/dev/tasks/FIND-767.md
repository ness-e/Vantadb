---
title: "FIND-767 - security_audit tenía 6 tests y ningún lane: ahora corre en un job semanal propio"
kind: task
status: completed
description: "`security_audit` (6 tests, 5 de seguridad real) no lo invocaba ningún workflow — `git log -S` en `.github/workflows/` → 0 — y estaba fuera del `default-filter`. Medido: 68-81 s de pared, así que no entra al gate por PR (umbral 30 s). Corre en un job programado propio de `certification.yml`, aislado de `other_heavy` (FIND-766), con `--ignore-default-filter` y `--no-tests fail`."
tags: [vantadb, ci, nextest, security, coverage]
---

# FIND-767 — `security_audit` no tenía lane

## Contexto

`FIND-752` lo contó: el binario `security_audit` tiene **6 tests** (5 de
seguridad real + un resumen), vive en la lista de exclusiones del
`default-filter` (`.config/nextest.toml:68`) y **no aparece en ningún
workflow** — `git log -S"security_audit" -- .github/workflows` → 0 commits.
Los dos gates locales también lo saltan por nombre (`dev-tools/verify.ps1:117`
en el fallback `cargo test`, `dev-tools/coverage-budget.ps1:79` por `--skip`).
La lane que *debía* cubrirlo (`certification.yml` → `other-heavy`) muere en
`TBH-01` antes de ejecutar un solo test (`FIND-766`). Efecto neto: **cero
ejecuciones** desde que existe — un test que existe y nunca corre cuenta como
cobertura, que es el modo de fallo que este ticket cierra.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `.github/workflows/certification.yml` (998 L),
  `.config/nextest.toml` (108 L), `tests/security/security_audit.rs` (613 L),
  `.github/actions/rust-setup/action.yml`, `.github/workflows/ci-gate.yml`,
  `dev-tools/git-commit-isolated.ps1` (155 L), `docs/dev/tasks/FIND-752.md`.
- **Referencias hacia dentro del fichero tocado:** `certification.yml` es
  llamado por sí mismo (schedule/push/PR) y su `name:` de job no se compara
  contra ninguna lista (la lista de check-runs required vive en
  `ci-gate.yml` + rulesets, y este job **no** es required).
- **Referencias hacia fuera:** `./.github/actions/rust-setup` (composite local),
  `actions/checkout` pinneado por SHA como el resto del fichero.
- **Veredicto:** cambio **aditivo** en un solo fichero de workflow + docs. No
  toca `src/**`, ni `.config/nextest.toml`, ni ningún job existente: 0 líneas
  modificadas de `other-heavy` (FIND-766 es su dueño en la wave siguiente).

## Decisión: qué lane

| Opción | Coste medido | Veredicto |
|---|---|---|
| (b) `ci-rust.yml` en cada PR | **68 s / 73 s / 81 s** de pared para los 6 tests (4 / 8 / 1 `--test-threads`, `target/debug` en Windows; `security_audit_resource_exhaustion` solo ≈ 65-89 s) | **rechazada** — el umbral del contrato era < 30 s; 68 s en cada PR es un minuto de runner para una suite que la mayoría de los diffs no puede afectar |
| (a) job nuevo en `certification.yml`, programado | ~70 s de tests + un build debug, **una vez por semana** | **elegida** |
| No correr en CI (borrar o `#[ignore]` con razón) | — | rechazada: son los 5 tests que verifican IQL injection, auth bypass, input validation, resource exhaustion y comparación en tiempo constante; la base de datos se queda sin medir eso |

**Por qué la opción (a) es defendible pese a correr `main`:** el run semanal
mide `main`, que va por detrás de `develop` (el mismo agujero que `FIND-752`
documentó para `other_heavy`). Se acepta a cambio de que la suite **exista en
un lane** y su verde/rojo sea observable; si el coste baja de 30 s, la lane por
PR es estrictamente mejor y queda escrita como upgrade en el propio YAML.

**Aislamiento frente a `FIND-766`** (pre-mortem c): job propio `security-audit`,
`name:` propio `Certification: security_audit`, añadido **al final** del
fichero (después de `fuzz-pr`), sin steps ni `needs` compartidos con
`other-heavy`. `FIND-766` puede editar `other-heavy` (líneas 296-378) sin
tocar una línea de este job.

**Sin `needs: ci-gate`** (decisión, no olvido): los otros jobs pesados lo
llevan para no gastar horas cuando `main` está rojo. Este es el **único** lane
que corre estos tests, y `needs: ci-gate` lo convierte en `skipped` —que GitHub
reporta como *success*— justo cuando `main` está rojo. Ese es exactamente el
modo de fallo que `FIND-752` midió (un verde que no mide). El coste de correrlo
igual: ~70 s de tests + un build debug, una vez por semana.

**Guarda contra el vacío** (`FIND-738`): `--ignore-default-filter` +
`--no-tests fail`. Sin la primera, el selector matchea 0 tests y sale 0; sin la
segunda, un 0 sería verde. Con ambas, una selección vacía es roja.

## Medición

```text
# conteo (contrato) — el comando de FIND-752; en esta sesión quedó bloqueado
# esperando el lock de cargo (ver §"Lo que NO se pudo verificar")
cargo nextest list -p vantadb --ignore-default-filter -E 'binary(security_audit)'
# → 6 tests, 0 benchmarks  (medido en FIND-752 con este comando; reconfirmado
#   con el binario: security_audit-…exe --list)

# runtime real, binario ya compilado (target/debug/deps/security_audit-355292c3ca9ce7f7.exe)
--test-threads=1  → 6 passed, 80.84 s (pared 80.9 s)
--test-threads=4  → 6 passed, 68.22 s (pared 68.5 s)
--test-threads=8  → 6 passed, 72.51 s (pared 72.7 s)

# por test (una corrida cada uno)
security_audit_iql_injection             8.0 s
security_audit_auth_bypass               3.1 s
security_audit_input_validation          4.7 s
security_audit_resource_exhaustion      88.9 s   ← el poste largo (10k batch + 30 open/close + 8 hilos × 250 puts)
security_audit_timing_attack             0.5 s
security_audit_summary_report            1.6 s
```

`resource_exhaustion` pasa del `slow-timeout = 60s` del `profile.audit`: eso
solo lo **reporta** como slow; `terminate-after = 3` lo mata a los 180 s, así
que el margen es holgado (medido 65-89 s como test único).

## Verificación

- `actionlint .github/workflows/certification.yml` → **exit 0** (baseline
  previo también 0).
- YAML parseado (pyyaml, no a ojo): el job existe con
  `if = workflow_dispatch || schedule '0 3 * * 0'`,
  `name = "Certification: security_audit"`, 3 steps (checkout, rust-setup,
  run), **22 jobs** (antes 21), **0 `name:` duplicados**.
- `cargo nextest list -p vantadb --ignore-default-filter -E 'binary(security_audit)'`
  → **6 tests** (ver §Medición).
- `python -c yaml.safe_load` + conteo de jobs arriba: el bloque `run:` es
  exactamente
  `cargo nextest run -p vantadb --test security_audit --ignore-default-filter --profile audit --no-tests fail --locked`.

## Lo que NO se pudo verificar (declarado)

- **El workflow no se disparó**: no se puede lanzar un `schedule` ni un
  `workflow_dispatch` desde acá, y el push es acto del owner. Se verificó la
  **causa** (el job existe, el filtro está puesto, el YAML parsea, `actionlint`
  verde) y la **medición local** del binario; el primer run real (domingo
  03:00 UTC o dispatch manual) es lo que confirma el verde en runner Linux.
- **El tiempo en Linux/CI no está medido**: los 68-81 s son de `target/debug`
  en Windows. En un runner Linux con más caché el número debería bajar, no
  subir; el `timeout-minutes: 45` cubre el peor caso (build debug frío).
- **El conteo de `nextest list` no llegó a completarse aquí**: se lanzó
  `cargo nextest list -p vantadb --ignore-default-filter -E 'binary(security_audit)'`
  y quedó **17 m 28 s** en `Blocking waiting for file lock on artifact directory`
  (`src/**` en edición por `FIND-758`, y tres `cargo-nextest` de otros carriles
  compitiendo por el mismo `target/`); se abortó en vez de sumar una cuarta
  compilación al lock. El conteo se verificó dos vías independientes:
  el binario ya construido (`security_audit-…exe --list` → `6 tests, 0
  benchmarks`) y el `security_audit` = **6** que `FIND-752` midió con el
  comando exacto del contrato sobre las features por defecto.

## Deuda / lo que queda

- Si el coste baja de 30 s, mover la lane a `ci-rust.yml` por PR (mejor
  cobertura que un semanal sobre `main`).
- `FIND-766` sigue siendo el dueño de que el run semanal pruebe `main` y no
  `develop`.
- `docs/dev/operations/CI_POLICY.md` §2c es de `FIND-752` (en vuelo); el dato
  "lane = `certification.yml` → `Certification: security_audit`" queda acá y en
  `ci-cd.md` hasta que ese carril lo absorba. **NOTICED BUT NOT TOUCHING.**
