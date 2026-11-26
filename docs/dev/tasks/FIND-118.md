---
title: "FIND-118 — `vantadb_py` sigue vivo: removerlo es un rename del módulo nativo (deferido a 1.0.0)"
kind: task
status: stable
description: "Medido vivo en 0.9.0: `import vantadb_py` funciona y el import canónico carga `vantadb_py`. No es un alias removible sino el paquete que aloja el .pyd; la remoción es legal en un MINOR 0.x pero exige renombrar el módulo nativo, así que se difiere a 1.0.0 (decisión pendiente del owner) y se registra en DEPRECATIONS.md"
---

# FIND-118 — el alias `vantadb_py` sigue vivo, y no es lo que la fila decía

> **Ruta:** `vanta-worker` · **Carril:** C (Python/docs, sin `cargo`) · **Desenlace:** **(b) sigue vivo**; la remoción queda **deferida a 1.0.0, pendiente de firma del owner**
> **Origen de la fila (Backlog):** `FIND-118` — *"Remover alias legacy `vantadb_py` en 0.6.0 (cierra migración temporizada H-09/PY-03)"* · Dep: `H-09`/`PY-03`

## Veredicto (leer primero)

La fila describía `vantadb_py` como un **alias** removible cuando pasara la
ventana 0.6.0. Las dos mitades son falsas:

1. **La ventana no "pasó" — nunca existió.** La deprecación (PY-03) se publicó
   **en** 0.6.0 (`CHANGELOG.md:1516` — `## [0.6.0] … 2026-09-20`, con la entrada
   del alias en `:1676`) y el mensaje prometía la remoción **en** 0.6.0
   (`vantadb_py/__init__.py:17`). Deprecar y remover en la misma release
   contradice la política (`docs/api/VERSIONING.md:99` — "at least one MINOR").
2. **No es un alias, es la implementación.** `vantadb/__init__.py` es un shim de
   45 líneas cuyo único trabajo es `from vantadb_py import *`. El paquete
   `vantadb_py/` aloja el módulo compilado (`vantadb_py.pyd`, 7.3 MB), el
   wrapper de 669 líneas (`AsyncClient`, `SearchRequest`, `error_to_dict`) y el
   subpaquete `migrate/`. Borrar el "alias" es renombrar el módulo nativo.

**Sobre el target (corregido en la ronda 1 de review):** la remoción **era legal
en `0.10.0`** — `VERSIONING.md:19-20` deja que un MINOR pre-1.0 lleve breaking
changes y la ventana de 1 MINOR se cumplió desde 0.6.0. `1.0.0` **no** es una
imposición de ADR-0045: D1 congela los **9 nombres de artefacto** (crate /
distribuciones / binarios), y el módulo de import no está entre ellos. Deferir a
`1.0.0` es una **decisión** (el rename toca `module-name`, `#[pymodule]`, 52
sitios de import y 3 workflows) y **queda pendiente de firma del owner**
(§Incógnitas 1). Para que nadie repita el error de atribución, `VERSIONING.md`
§Deprecation policy ahora declara el alcance del freeze por escrito.

## Impacto mapeado (Regla 0)

**Archivos leídos completos antes de editar:**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `vantadb-python/vantadb_py/__init__.py` | 669 L completas | el "alias" (docstring, warning, API real) |
| `vantadb-python/vantadb/__init__.py` | 45 L completas | el shim canónico (dirección real de la dependencia) |
| `vantadb-python/pyproject.toml` | 70 L completas | `[tool.maturin] module-name = "vantadb_py"` + `include` |
| `vantadb-python/tests/test_import_alias.py` | completo | el contrato PY-03 que había que corregir |
| `docs/api/DEPRECATIONS.md` | completo | registro obligatorio (ADR-0045 D2.3) |
| `docs/api/PYTHON_SDK.md` | §Import name | doc de superficie afectada |
| `docs/api/VERSIONING.md` | §Pre-1.0, §11 surfaces, §Deprecation policy | la normativa real del target |
| `docs/dev/architecture/adr/ADR-0045-naming-freeze.md` | completo | **alcance real** de D1 (9 artefactos, no el módulo de import) |
| `CHANGELOG.md` | 0.6.0 + entradas PY-03 | fecha real de la deprecación |

**Referencias entrantes (quién depende del nombre):** 52 sentencias
`import`/`from vantadb_py` medibles — 36 en `vantadb-python/` (paquete +
`migrate/` + 20 tests + `conftest.py` + `sanity_check.py` +
`verify_published_wheel.py`), 12 en `integrations/`, 2 en `providers/`+`scripts/`,
2 en snippets de `docs/user/tutorials/`. Además consumidores **no editables en
esta tarea**: `.github/workflows/release-wheels-60.yml`, `dev-tools/setup_venv.*`,
`dev-tools/scripts/validate_python_sdk.*`, `dev-tools/verify_staged.ps1:45`.

**Veredicto de impacto:** la remoción real es una **migración breaking de
empaquetado** (`pyproject.toml module-name` + `#[pymodule]` en
`vantadb-python/src/lib.rs:3197` + 52 consumidores + workflows + dev-tools), i.e.
fuera del alcance escribible de esta tarea. Lo que **sí** era deuda dentro de
alcance: el mensaje de deprecación prometía una versión ya publicada y el
registro D2.3 no existía.

## Medición ejecutada

```bash
rg -n 'vantadb_py' --glob '!target' --glob '!.venv' --glob '!node_modules' --glob '!dist' .
# → 550 líneas con match en 169 archivos (docs/api y docs/user dominan; el resto:
#   vantadb-python, integrations, dev-tools, scripts, providers, benchmarks, CHANGELOG)

rg -n '^\s*(import|from)\s+vantadb_py' --glob '!target' --glob '!.venv' .
# → 52 sentencias ejecutables (36 vantadb-python · 12 integrations · 2 providers/scripts
#   · 2 snippets docs)

vantadb-python/.venv/Scripts/python.exe -c "import vantadb_py; print(vantadb_py.__version__)"
# → DeprecationWarning: ... removed in ... (0.6.0) ...
# → IMPORT OK -> vantadb_py 0.9.0

python -c "import vantadb, sys; print('vantadb_py' in sys.modules)"
# → True   (el import canónico DEPENDE del nombre "deprecado")
```

Repetido idéntico en los tres venvs presentes (`vantadb-python/.venv`,
`target/audit-venv`, `.venv`): los tres resuelven `import vantadb` →
`vantadb-python/vantadb/__init__.py` y reportan `vantadb_py 0.9.0`.

## Steps

| # | Step | Verify | Resultado |
|---|---|---|---|
| 1 | Medir liveness + dirección de la dependencia (rg + 3 venvs) | salida de arriba | ✅ vivo en 0.9.0; `vantadb` carga `vantadb_py` |
| 2 | Leer la normativa que gobierna el target (VERSIONING, ADR-0045) | alcance de D1 | ✅ D1 cubre 9 artefactos, **no** el módulo de import ⇒ 0.10.0 era legal |
| 3 | Corregir el contrato falso en `vantadb_py/__init__.py` (docstring + `warnings.warn`) y en el shim `vantadb/__init__.py:6` ("removal in 0.6.0") | `pytest test_import_alias.py` + `rg '0\.6\.0' vantadb-python/vantadb*` | ✅ 3/3 · 0 promesas del target viejo |
| 4 | Endurecer el contrato: ventana real + alias sirviendo el objeto **compilado** (`vantadb_py.Client is vantadb_py.vantadb_py.Client`) | `pytest` | ✅ 3/3 (antes 2) |
| 5 | Registrar la deprecación en `docs/api/DEPRECATIONS.md` (D2.3, estaba ausente) + target en `PYTHON_SDK.md` + **alcance del freeze** en `VERSIONING.md` | gates de docs | ✅ 0 broken links / 0 docs errors / 0 gaps de cobertura |
| 6 | Review P2-01 ronda 1 (agente distinto) → 🔴 changes required (1C/3R/4O) → aplicado | ver §Review | ✅ C1 y R1-R3 resueltos; R1 escalado al owner |
| 7 | Cerrar la fila (Backlog) + entrada en `bindings.md` + commit aislado | helper `git-commit-isolated.ps1` | ✅ `19fbb5b87` + `08093505a` (reparación, ver §Incógnitas 6) |

## Commits

| Commit | Contenido |
|---|---|
| `19fbb5b87` | `fix(python): FIND-118` — mensaje/docstrings, test endurecido, registro `DEPRECATIONS.md`, `PYTHON_SDK.md`, alcance del freeze en `VERSIONING.md`, task file, entrada de `bindings.md`, fila del Backlog |
| `08093505a` | `fix(backlog): FIND-118` — reparación de la copia stale del Backlog (ver §Incógnitas 6) |

## Contrato

```bash
python -m pytest vantadb-python/tests/test_import_alias.py -v
python -m pytest vantadb-python/tests/test_stub_drift.py -v      # el .pyi no quedó desalineado
node scripts/docs/check-links.mjs
node scripts/docs/check-docs.mjs
pwsh scripts/validate-docs-coverage.ps1
```

## Verificación medida

| Comando | Resultado |
|---|---|
| `pytest tests/test_import_alias.py -v` (venv `vantadb-python/.venv`) | ✅ 3 passed (canónico silencioso · legacy avisa con target `1.0.0` · legacy sirve el `Client` compilado) |
| `pytest tests/test_stub_drift.py -v` | ✅ 7 passed (el `.pyi` sigue espejando el runtime) |
| `pytest tests/test_sdk.py -q` | ✅ 77 passed (regresión del SDK: tocar el docstring del shim no rompió nada) |
| Control **RED** del pin nuevo (aplicado al mensaje viejo `0.6.0`) | ✅ falla: `warning must state removal timeline` — el test sí caza una reversión |
| `python -c "import vantadb_py"` (3 venvs) | ✅ importa, avisa con `1.0.0`, `vantadb_py 0.9.0` |
| `node scripts/docs/check-links.mjs` · `check-docs.mjs` | ✅ exit 0 · 0 broken · exit 0 |
| `pwsh scripts/validate-docs-coverage.ps1` | ✅ 0 gaps |
| `rg -n '0\.6\.0' vantadb-python/vantadb vantadb-python/vantadb_py vantadb-python/tests/test_import_alias.py` | ✅ 0 **promesas** del target viejo; solo menciones históricas que explican por qué 0.6.0 era inalcanzable (los `0.6.0` de `docs/api/*` son de AST-010, otro rename ya consumado) |

## Qué queda y por qué (alcance real de la remoción)

| Bloque | Archivos | Estado |
|---|---|---|
| Nombre del módulo nativo | `vantadb-python/pyproject.toml:44` (`module-name`), `vantadb-python/src/lib.rs:3197` (`#[pymodule]`) | bloqueado en esta tarea: `src/**` prohibido (`FIND-758` en vuelo) |
| Publicación / validación | `.github/workflows/release-wheels-60.yml`, `dev-tools/setup_venv.*`, `validate_python_sdk.*`, `verify_staged.ps1` | prohibido: `.github/workflows/**` (`FIND-770`) y `dev-tools/**` |
| Consumidores | 12 en `integrations/`, 2 en `providers/`+`scripts/`, 36 en `vantadb-python/` (tests + `migrate/`) | fuera del alcance escribible declarado; es exactamente el trabajo del rename |
| Decisión | target `0.10.0` (legal) vs `1.0.0` (deferido) | **PENDIENTE_OWNER** — §Incógnitas 1 |

## Invariantes

- **`import vantadb` no puede romperse**: `vantadb_py` es su implementación, no al revés.
- La deprecación no puede prometer una versión ya publicada ni una ventana de 0 releases.
- El alias permanece **funcional** y sirviendo los objetos compilados hasta su target.
- `vantadb_py/*.pyi` y el runtime no pueden divergir (`test_stub_drift.py`).

## Incógnitas / deuda

1. **El target `1.0.0` está PENDIENTE de firma del owner.** `0.10.0` era legal
   (`VERSIONING.md:19-20`) y sigue siéndolo; `1.0.0` es la opción elegida por
   costo de migración (rename del módulo nativo + 52 sitios + 3 workflows). Un
   patch de worker no debe extender por su cuenta una fecha pública ya emitida:
   queda registrado como deferral **provisional** en `DEPRECATIONS.md` y esta
   fila no lo da por ratificado. `decisions.md` no se tocó (`.opencode/**`
   prohibido en esta tarea) — la firma debe ir ahí o en un ADR.
2. **El rename real no es "borrar un archivo"** (ver tabla de arriba). Queda
   con target en `docs/api/DEPRECATIONS.md`.
3. **La propia suite importa el nombre deprecado** (36 sentencias en
   `vantadb-python/`, incluido `tests/conftest.py:30`): cada corrida de pytest
   emite el warning. Pre-existente y fuera de alcance; candidato a fila
   `FIND-*` cuando se aborde el rename (no se creó fila: esta tarea sólo podía
   tocar su propia fila del Backlog).
4. **Artefacto local obsoleto**: `vantadb-python/vantadb_py/` contiene
   `vantadb_native.pyd` (6.6 MB) además de `vantadb_py.pyd` (7.3 MB) — residuo
   del rename `#[pymodule] vantadb_native -> vantadb_py` (`d12d00226`). Es build
   artifact gitignored, no se tocó; un `maturin develop` fresco lo resuelve.
5. **`README.md:105` / `README_ES.md:105`** dicen "still works but emits a
   `DeprecationWarning`" sin versión: siguen siendo ciertos, no se editaron
   (fuera del alcance declarado y sin dato falso).
6. **Incidente del árbol compartido (clase `FIND-753`):** mi primer commit
   (`19fbb5b87`) llevó una copia **stale** de `docs/dev/Backlog.md` y borró 11
   líneas ajenas (secciones `FIND-770` y `FIND-771`, commiteadas por otro carril
   entre mi lectura y mi commit) además de mi propia fila. El exit code fue 0; lo
   cazó la comprobación **por contenido** del helper. Reparado en `08093505a`,
   reconstruyendo desde `19fbb5b87^` a nivel de byte. Lección: en checkout
   compartido, un archivo compartido se re-verifica contra `HEAD` inmediatamente
   antes de `git add`; y `git show` por pipe no es byte-safe para acentos (mangle
   silencioso) — `git checkout <rev> -- <path>` sí.

## Review

**P2-01 ronda 1 (agente distinto, contexto fresco): 🔴 CHANGES REQUIRED — 1
Critical, 3 Required, 4 Optional.** El reviewer no aceptó la justificación
central del cambio y tenía razón: yo había atribuido el target `1.0.0` a
ADR-0045 D1, que congela 9 **artefactos** y no el módulo de import; además
`VERSIONING.md:19-20` deja que un MINOR 0.x lleve breaking changes, así que
`0.10.0` era legal. Resolución de cada punto:

| # | Hallazgo | Resolución en este commit |
|---|---|---|
| **C1** | "v1.0.0 - the earliest legal window" / "no puede caer en un MINOR 0.x" es **normativamente falso** y estaba publicado en el registro | Reescrito en los 6 lugares (`DEPRECATIONS.md`, `PYTHON_SDK.md`, docstrings de `vantadb_py` y `vantadb`, docstring del test, task file, `bindings.md`): `1.0.0` es **deferral**, `0.10.0` también era legal. Añadida la nota de alcance del freeze en `VERSIONING.md` §Deprecation policy para cerrar el agujero de atribución de raíz |
| **R1** | Una fecha pública se movió de 0.6.0 a 1.0.0 sin ratificación del owner (no hay ADR ni entrada en `decisions.md`) | **Escalado, no auto-aprobado**: `PENDIENTE_OWNER` explícito en §Incógnitas 1, en `DEPRECATIONS.md` ("pending owner sign-off") y en `bindings.md`. `decisions.md` no se tocó por prohibición de `.opencode/**` |
| **R2** | El task file marcaba la review como ✅ antes de que existiera (auto-reporte de gate) | Step marcado con el dictamen real (🔴 → aplicado) y §Review poblada con este dictamen |
| **R3** | Las cifras de medición no reproducían (508 líneas/15 áreas; el desglose sumaba 178 y mezclaba archivos con líneas) | Re-medido con el comando publicado: **550 líneas / 169 archivos**; el desglose por área se eliminó (mezclaba unidades) y se conservaron las cifras que reproducen exacto (52/36/12/2/2) |
| O1 | La aserción de identidad era casi tautológica (`from … import *`) | Reforzada: ahora también `vantadb_py.Client is vantadb_py.vantadb_py.Client` (prueba que es la clase **compilada**) y el docstring dice qué guarda cada una. Se descartó el subproceso redundante |
| O2 | El task file quedaba huérfano (fila del Backlog borrada, sin enlace entrante) | Enlazado desde la entrada de `bindings.md` |
| O3 | `-` ASCII donde las entradas hermanas usan `—` | Normalizado a `—` en la entrada nueva de `DEPRECATIONS.md` |
| O4 | "wrapper de 659 líneas" (son 669) | Corregido a 669 en task file y `bindings.md` |

Verificado tras los cambios: `pytest test_import_alias.py` **3/3** ·
`test_stub_drift.py` **7/7** · `test_sdk.py` **77/77** · `check-links` **0
broken** · `check-docs` **exit 0** · `validate-docs-coverage` **0 gaps**.

## SDP

`SDP:` campaign-executor, progreso, documentation-skill, source-driven-development,
incremental-implementation, test-driven-development, deprecation-and-migration,
api-and-interface-design, doubt-driven-development.
