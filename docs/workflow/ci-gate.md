# `ci-gate.yml` — CI Gate

## ¿Qué hace?

Workflow reutilizable que actúa como compuerta de calidad para workflows pesados o programados. Antes de gastar horas de cómputo (benchmarks, certificación, fuzzing), consulta el estado de los check-runs del CI en el commit objetivo de `main` y aborta si el CI está en rojo.

## ¿Cómo lo hace?

Es un reusable workflow (`on: workflow_call`) con un solo input:

- **`event_name`** (string, requerido) — el evento que disparó al caller (`schedule`, `workflow_dispatch`, etc.). El caller lo pasa con `with: event_name: ${{ github.event_name }}`.

El job `ci-gate`:

1. Verifica el CI **solo si** `event_name == 'schedule'` (los schedules son automáticos; un dispatch manual debe poder forzar la ejecución).
2. Consulta `GET /repos/{owner}/{repo}/commits/{sha}/check-runs?per_page=100&filter=latest` con `gh api` usando el `GITHUB_TOKEN`.
3. Para cada uno de los 11 checks requeridos por el ruleset de `main`, filtra por nombre y `status == "completed"` y toma su `conclusion`.
4. Si cualquier conclusión es `failure`, `timed_out`, `cancelled` o `action_required`, el job falla y los jobs del caller con `needs: ci-gate` se saltan.
5. Checks ausentes o aún en progreso → el gate pasa (fail-open: no bloquea por CI no terminado).

## Checks requeridos (11)

`Format Check`, `Clippy Lints`, `Tests (Linux)`, `Tests (Windows)`, `Tests (macOS)`, `MSRV Check (1.94.1)`, `Experimental Crates Check`, `Security Audit`, `Miri (UB Detection)`, `Dependency Policy Check`, `Analyze`.

## ¿Cómo se usa?

Los callers (ej. `heavy-certification-50.yml`, `heavy-bench-nightly-51.yml`, `fuzz-40.yml`) lo invocan como job:

```yaml
ci-gate:
  uses: ./.github/workflows/ci-gate.yml
  permissions:
    checks: read
  with:
    event_name: ${{ github.event_name }}
```

Luego los jobs pesados declaran `needs: ci-gate`.

## ¿Qué verifica?

- Que el CI de `main` esté en verde antes de ejecutar tareas costosas
- Que la ejecución manual (`workflow_dispatch`) no quede bloqueada por el CI

## Funcionalidad final

Ahorro de cómputo y de runners: los workflows pesados no corren sobre un `main` que ya falla en CI. Es el equivalente a un `needs` entre workflows (que GitHub Actions no soporta).

## Permisos

- El reusable declara `permissions: contents: read, checks: read`.
- El **calling job** debe declarar `permissions: checks: read`, porque el token del caller solo puede degradarse (no elevarse) en un reusable workflow.
