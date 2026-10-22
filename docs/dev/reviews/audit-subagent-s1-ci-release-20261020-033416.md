---
title: "S1 — Auditoría CI + Release Mechanics (v0.9.0)"
kind: review
status: active
description: "Informe crudo del subagente S1: auditoría de la mecánica de CI y de publicación (paths filters, required checks, semver check, sync de versiones, fan-out, pins de actions, manifest, Dependabot). Base: origin/main 66f9a775, head auditado release-plz-fix-cargo-token. Sus 19 hallazgos son la fuente de `FIND-384` y `FIND-466..FIND-481`."
tags: [audit, reviews, ci, release, subagent]
---

# Informe del subagente — Auditoría CI + Release Mechanics · v0.9.0 · rama `release-plz-fix-cargo-token`

**Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
**Base:** `origin/main` (66f9a775 — merge PR #238 "chore(vantadb): release v0.9.0")
**Head auditado:** `origin/release-plz-fix-cargo-token` = 2e8152a3 (9 commits: 2e8152a3, 7c491b9f, f5b11dc7, 76ed8743, ec441d55, 162b4ee3, bb12f2c3, 0253a0c7, ea6a0b26)
**Modo:** READ-ONLY. Ningún fichero del repo modificado; ningún comando git de escritura; `gh` solo lectura (list/view/api GET).
**Fecha de la auditoría:** 2026-10-20 (estado real del repo en ese momento)

## Estado real del release 0.9.0 en vuelo (verificado, no supuesto)

| Artefacto | Estado real | Fuente |
|---|---|---|
| crates.io `vantadb` | **0.8.0** — 0.9.0 NO publicado | `GET https://crates.io/api/v1/crates/vantadb` |
| npm `vantadb` | 0.8.0 | `GET https://registry.npmjs.org/vantadb` |
| npm `vantadb-wasm` | 0.8.0 | idem |
| npm `vantadb-node` | **NO EXISTE** (404) — primer publish será 0.9.0 | idem |
| PyPI `vantadb-py` | 0.8.0 | `GET https://pypi.org/pypi/vantadb-py/json` |
| PyPI 7 adapters (`-langchain/-llamaindex/-mem0/-crewai/-dspy/-haystack/-letta`) | 0.5.0 | idem |
| PyPI 3 providers (`-openai/-ollama/-litellm`) | 0.8.0 | idem |
| Tag `v0.9.0` | **NO EXISTE** (`git ls-remote --tags` + `GET /releases`) | idem |
| GitHub Release `v0.9.0` | **NO EXISTE** (último = v0.8.0, 14 assets) | idem |
| `origin/main` `[workspace.package] version` | **0.9.0** | `git show origin/main:Cargo.toml` |
| Secret `CARGO_REGISTRY_TOKEN` | **EXISTE** (actualizado 2026-10-20T04:46:08Z) | `gh secret list` |
| Último run release.yml | `39368867477` → **failure** (`Release-plz release` job 118451918327) | `gh run view` |
| PR abierto #259 (esta rama → main) | open, MERGEABLE | `gh pr list` |
| PR abierto #258 (develop → main) | open, MERGEABLE | `gh pr list` |

Log del fallo (run 39368867477, job 118451918327, paso `Run release-plz`):
```
2026-10-20T04:25:03Z  Uploading vantadb v0.9.0 (/home/runner/work/Vantadb/Vantadb)
2026-10-20T04:25:03Z  error: failed to publish vantadb v0.9.0 to registry at https://crates.io
2026-10-20T04:25:03Z    please provide a non-empty token
```

---

## Resumen de hallazgos

| ID | Sev | Área | Título |
|----|-----|------|--------|
| S1-01 | **P1** | semver_check | El job "Semver Checks (public API)" no puede fallar por una violación semver |
| S1-02 | **P1** | required checks | El fix de `ci-fleet.yml` no se aplicó a `ci-rust.yml` ni a `docs-gates.yml` |
| S1-03 | **P1** | release.yml | El sync puede empujar al branch equivocado (`release-plz-fix-cargo-token` casa con el prefijo) |
| S1-04 | **P2** | CARGO_REGISTRY_TOKEN | Sin aserción de token no vacío antes de publicar (el fallo ya ocurrió una vez) |
| S1-05 | **P2** | sync script | `--write` nunca falla: target ilegible o transform que no casa ⇒ exit 0 |
| S1-06 | **P2** | sync script | Nada en CI ejecuta `--check`; `version_coherence.rs` no cubre los 3 targets nuevos |
| S1-07 | **P2** | dependabot guard | 18 required checks quedan vacíos en cada PR de Dependabot |
| S1-08 | **P2** | action pins | 2 pins apuntan a commits que no existen en upstream |
| S1-09 | **P2** | fan-out | Doble publicación concurrente (tag push + dispatch) ⇒ carrera TOCTOU |
| S1-10 | **P2** | release-plz.toml | `vantadb-wasm` sin `publish = false`: a una edición de publicar un 16º crate |
| S1-11 | **P2** | docs publish | PUBLISH.md y verify-release.ps1 siguen diciendo "vantadb-node nunca publicado" |
| S1-12 | **P3** | release.yml header | Cabecera sigue declarando Trusted Publishing (OIDC) |
| S1-13 | **P3** | release-wheels | `publish-pypi` no acepta `workflow_dispatch` (los hermanos sí) |
| S1-14 | **P3** | docs | `docs/dev/tasks/DIST-01.md` no existe (referenciado 2 veces) |
| S1-15 | **P3** | release-plz.toml | Comentario cita rutas del upstream que ya no existen |
| S1-16 | **P3** | release.yml | `git add -A` en el paso de sync |
| S1-17 | **P3** | sync script | `transformNodeLockJson` usa un contador posicional frágil |
| S1-18 | **P3** | manifest | `documented_only` no menciona `crates.io vantadb-wasm` |
| S1-19 | **P3** | dependabot | Ecosistema `github-actions` queda en solo-alertas (0 PRs) |

**No hay P0 confirmado.** El camino de publicación está desbloqueado por el secret real (verificado con `gh secret list`) y las 15 entradas del manifest son correctas y publicables (verificado contra los registries vivos). Los P1 son deuda de gate y una ruta de fallo concreta, no un bloqueo directo del publish de 0.9.0.

---

## Findings

### S1-01 | P1 | semver_check
- **Location:** `.github/workflows/ci-rust.yml:134-136` (paso), `.github/workflows/ci-rust.yml:94-98` (comentario), `release-plz.toml:43`
- **Claim/Break:** El comentario del job dice *"RELEASE-01: gate so a breaking change in the public API can never publish … this job closes that gap"* (`ci-rust.yml:95,97`). Pero el único paso sustantivo es `ci-rust.yml:136` con `continue-on-error: true` en `ci-rust.yml:135` ⇒ el job reporta **success** aunque `cargo semver-checks` detecte una violación. Verificado: `"Semver Checks (public API)"` **no** está en los 18 `required_status_checks` del ruleset `main` (`gh api repos/ness-e/Vantadb/rulesets/16844590`). El único mecanismo real es `semver_check = true` (`release-plz.toml:43`), y por diseño upstream **no puede rechazar el release**: `release-plz/website/docs/semver-check.md` → *"When release-plz detects API breaking changes, it updates the version of the package with a 'major semver Bump'".* Evidencia empírica del run 39353698789 (job 118404027216, paso `Run release-plz`):
  ```
  2026-10-20T01:35:03Z INFO Checking API compatibility with cargo-semver-checks...
  2026-10-20T01:52:03Z INFO vantadb: next version is 0.9.0 (⚠️ API breaking changes)
  ```
  El check **corrió** (17 min) y **detectó breaking changes**, y aun así el release continuó como 0.9.0.
- **Why it matters:** No existe en el repo ningún gate capaz de impedir que una ruptura de API pública se publique como 0.9.0. La mitigación es únicamente la corrección del número de versión (0.x → minor). `docs/api/COMPATIBILITY.md:20` y `docs/dev/avance/activo/operaciones.md:61` describen el mecanismo como "gate pre-publish obligatorio", y `release-plz.toml:23-26` lo describe con honestidad como INFORMATIONAL — el repo se contradice a sí mismo. Además, `release-plz/crates/release_plz_core/src/command/update/updater.rs:1078-1085` (`should_check_semver`) **omite el check con un simple warning** si `cargo-semver-checks` no está instalado; aquí sí lo está (`release-plz/action@b8d6b54b` → su `action.yml` instala `cargo-semver-checks@0.50`, verificado leyendo el `action.yml` en ese SHA), pero esa degradación silenciosa no está documentada en el repo.
- **Fix:** (a) Corregir el comentario de `ci-rust.yml:95-98` para que no afirme un gate inexistente, o (b) quitar `continue-on-error: true` de `ci-rust.yml:135` y promover el job a required check (implica resolver antes el rojo permanente de `develop→main` que el propio repo documenta). Añadir a `release-plz.toml` una nota de que `semver_check` **reescribe la versión, no bloquea**, y de que si `cargo-semver-checks` faltase el check se omite en silencio.
- **Verify:** `cargo semver-checks -p vantadb --release-type minor` en local; en un PR con un rename público, comprobar la conclusión del job `Semver Checks (public API)` en la UI (hoy: success) y la versión que release-plz propone.

### S1-02 | P1 | required checks (path filters)
- **Location:** `.github/workflows/ci-rust.yml:22-39` (`on.pull_request.paths`), `.github/workflows/docs-gates.yml:42-58` (`on.pull_request.paths`), `.github/workflows/ci-fleet.yml:34-35,53-54` (el fix aplicado)
- **Claim/Break:** El commit 0253a0c7 demuestra que un filtro `paths:` que no matchea ⇒ el check nunca reporta ⇒ "Expected - waiting for status to be reported" para siempre (PR #242 y #257, resueltos con bypass de admin). Amplió `ci-fleet.yml` con `.github/workflows/**` + `release-plz.toml`, pero **no** aplicó el mismo tratamiento a los otros dos workflows que producen required checks. Reparto real de los 18 contexts del ruleset `main` (verificado vía API): **11** de `ci-rust.yml`, **6** de `docs-gates.yml`, **1** de `ci-fleet.yml`. Faltan en los filtros:
  - `ci-rust.yml`: `.github/workflows/**` (sólo está `.github/workflows/ci-rust.yml`, línea 32), `release-plz.toml`, `release-manifest.json`.
  - `docs-gates.yml`: `release-plz.toml`, `release-manifest.json`, `integrations/**`, `vanta-proxy/**`.
- **Why it matters:** Un PR que toque **sólo** `release-plz.toml`, `release-manifest.json`, `integrations/**` o un workflow que no sea `ci-rust.yml` ⇒ `ci-rust.yml` no corre ⇒ 11 required checks no reportan; y si además no toca `scripts/**`/`docs/**`/`.github/workflows/**` ⇒ `docs-gates.yml` tampoco corre ⇒ 6 más. El PR queda bloqueado indefinidamente, exactamente el fallo que 0253a0c7 dice arreglar. El PR #259 actual se salva porque toca `ci-rust.yml` y `scripts/docs/**`.
- **Fix:** En `ci-rust.yml` `on.pull_request.paths`, añadir `.github/workflows/**`, `release-plz.toml`, `release-manifest.json`. En `docs-gates.yml` `on.pull_request.paths`, añadir `release-plz.toml`, `release-manifest.json`, `integrations/**`.
- **Verify:** Abrir un PR que toque únicamente `release-manifest.json` y comprobar que reportan los 18 contexts antes de pedir merge; `gh api repos/ness-e/Vantadb/rulesets/16844590 --jq '.rules[].parameters.required_status_checks[].context' | wc -l` ⇒ 18.

### S1-03 | P1 | release.yml (selección de branch)
- **Location:** `.github/workflows/release.yml:120`
- **Claim/Break:**
  ```bash
  PR_BRANCH=$(gh pr list --state open --base main --json headRefName \
    --jq '[.[] | select(.headRefName | startswith("release-plz-"))][0].headRefName // empty')
  ```
  El filtro `startswith("release-plz-")` también casa con **`release-plz-fix-cargo-token`**, que es precisamente el branch de esta rama. Además `[0]` toma el primero en el orden de la API (no el más reciente) ⇒ con dos PR `release-plz-*` abiertos, el destino es arbitrario.
- **Why it matters:** Si el publish de crates.io vuelve a fallar (o cualquier otro fallo de `release-plz-release`), `releases_created` no es `'true'` ⇒ corre el job `release-plz-pr` (`release.yml:71`) ⇒ el sync hace `git checkout -B "$PR_BRANCH" "origin/$PR_BRANCH"` (`release.yml:126`) y empuja el commit de sync a ese branch (`release.yml:142-143`). Si el branch elegido es `release-plz-fix-cargo-token` en vez del branch real del Release PR, los 22 manifiestos se sincronizan en el branch equivocado y **el Release PR queda sin sincronizar** — que es exactamente el incidente que ea6a0b26 dice arreglar.
- **Fix:** Filtrar por el patrón real de release-plz (`^release-plz-\d{4}-\d{2}-\d{2}T` o `startswith("release-plz-20")`) y seleccionar el PR **más reciente** (ordenar por `createdAt` desc) en lugar de `[0]`.
- **Verify:** Con dos PR `release-plz-*` abiertos, ejecutar el comando de `release.yml:120` y comprobar que devuelve el branch del Release PR y no `release-plz-fix-cargo-token`.

### S1-04 | P2 | CARGO_REGISTRY_TOKEN
- **Location:** `.github/workflows/release.yml:56-66`
- **Claim/Break:** `CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}` se define **incondicionalmente**. Si el secret no existiera, la variable quedaría presente y vacía y cargo falla con *"please provide a non-empty token"* — que es exactamente lo que pasó (run 39368867477, job 118451918327) y lo que el commit 162b4ee3 documentó al quitar la línea. f5b11dc7 la reañadió tras crear el secret. **Verificado**: `gh secret list` ⇒ `CARGO_REGISTRY_TOKEN 2026-10-20T04:46:08Z` (existe, valor no vacío). **No hay ninguna otra definición** de la variable en `.github/workflows/`, `scripts/` ni `docs/` (grep completo: única referencia operativa = `release.yml:66`; el resto son referencias históricas en `docs/CHANGELOG.md:200`, `docs/dev/avance/activo/ci-cd.md:138`, `2026-09-10-anti-stutter:287`, `docs/dev/tasks/AUD-027.md:41`). Ningún workflow la define como cadena vacía.
- **Why it matters:** El modo de fallo es *tardío y opaco*: el crate compila 2m22s y sólo entonces revienta en el publish, con el tag/release potencialmente ya creados (el propio `release.yml:14-17` avisa: *"A timeout here would leave the tag created but the crate unpublished"*). El repo ya demostró que este secret puede desaparecer/renombrarse. Nótese la asimetría: el job `fanout` **sí** aserta que `RELEASE_PLZ_TOKEN` esté puesto (`release.yml:172-175`, con `::error::` explícito), pero el publish a crates.io no tiene aserción equivalente.
- **Fix:** Añadir un paso de pre-flight en `release-plz-release` (o una línea al inicio del step `Run release-plz`) que falle rápido y con mensaje explícito:
  ```bash
  if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
    echo "::error::CARGO_REGISTRY_TOKEN is missing or empty - the crates.io publish would fail with 'please provide a non-empty token'"
    exit 1
  fi
  ```
- **Verify:** Quitar temporalmente el secret (o sobreescribir la variable a "") en un run de prueba ⇒ el job debe fallar en el pre-flight, no tras 2m22s de compilación.

### S1-05 | P2 | sync script (falla en silencio)
- **Location:** `scripts/docs/sync-workspace-version.mjs:496-527` (bucle), `:529-554` (modo write), `:553` (`process.exit(0)`)
- **Claim/Break:** El script documenta exit 1 = drift detectado (`:26`), pero **sólo en modo `--check`**. En modo `--write` (el que usa el release) el `exit 0` de la línea 553 es incondicional. Demostrado empíricamente (copia del árbol de la rama, `release-manifest.json` renombrado):
  ```
  FAIL: release-manifest.json: unable to read file (ENOENT: no such file or directory, ...)
  Sync complete: 0 file(s) updated.
  EXITCODE=0
  ```
  Además, si el regex de un `transform` no casa con el contenido real, `transform()` devuelve el texto **sin cambios**, se escribe el fichero igual, se cuenta como "updated" y el script sale con 0.
- **Why it matters:** `release.yml:127-143` usa `--write` y después sólo mira `git status --porcelain` para decidir si hace commit. Si el script entrega a medias (fichero ilegible, regex que no casa, fichero generado que el regex no reconoce), el commit se hace igualmente, el **step queda verde**, y los manifiestos del Release PR quedan desincronizados sin que nada lo señale. Con 22 targets, la probabilidad de que un regex no case no es trivial (p. ej. `transformNodeLockJson` cuenta posicionalmente, ver S1-17).
- **Fix:** En modo write, tras escribir, re-parsear y validar: si `spec.parse(transformed) !== spec.expected` o si algún target fue ilegible ⇒ acumular `driftedCount` y `process.exit(1)` al final.
- **Verify:** Con `release-manifest.json` ausente, `--write` debe salir con código ≠ 0; y con un `release-manifest.json` cuya "tag" no empiece por `v`, el transform debe reportar fallo y no reescribir en silencio.

### S1-06 | P2 | sync script (nadie ejecuta `--check`)
- **Location:** `scripts/docs/sync-workspace-version.mjs:23-31`; única invocación en CI = `.github/workflows/release.yml:127` (modo `--write`); `grep -rn "sync-workspace-version" .github/` ⇒ **una sola coincidencia**
- **Claim/Break:** El script ofrece `--check` con exit 1 (`:26`), pero **ningún workflow de CI lo ejecuta**. `tests/version_coherence.rs` (230 líneas, corre dentro de `cargo nextest run` en `ci-rust.yml`) cubre: `vantadb-python/pyproject.toml` (dynamic), `vantadb-python/Cargo.toml`, `vantadb-mcp/Cargo.toml`, `vantadb-server/Cargo.toml`, el bound `vantadb-py` de `integrations/*/pyproject.toml` y el header de `docs/api/SURFACE_MANIFEST.md`. **No** cubre: `release-manifest.json`, `vantadb-ts/package.json`, `vantadb-ts/package-lock.json`, `vantadb-node/package.json`, `vantadb-node/Cargo.toml`, `server.json`, `docs/api/openapi.yaml`, `docs/api/MCP.md`, `providers/*/Cargo.toml` — ni `vanta-memory/Cargo.toml`.
- **Why it matters:** Los **3 targets que esta rama añade al sync** (`release-manifest.json`, `vantadb-ts/package-lock.json`, `vanta-memory/Cargo.toml`) no tienen ninguna verificación. Su único mecanismo de sincronización es un `--write` que nunca falla (S1-05). El 0253a0c7 añadió `release-manifest.json` al sync precisamente porque "su propio comentario dice que hay que bumpearlo" — y sin gate, vuelve a derivar en el siguiente release silenciosamente.
- **Fix:** Añadir un paso `node scripts/docs/sync-workspace-version.mjs --check` a un job de `docs-gates.yml` (que ya es required check: p. ej. el job `check-format` o uno nuevo `check-version-sync`), y/o extender `tests/version_coherence.rs` con los targets no cubiertos.
- **Verify:** Bajar `release-manifest.json` a `"version": "0.8.0"` en una rama y comprobar que el nuevo gate falla.

### S1-07 | P2 | dependabot guard (required checks vacíos)
- **Location:** 68 guards añadidos: `ci-rust.yml:52,73,110,148,178,235,267,304,338,355,375,455,471,510,532,561,584,604,659,681,776` (21), `docs-gates.yml` (18), `ci-fleet.yml` (4), `certification.yml` (4), `nightly.yml` (6), `demo-acceptance.yml` (5), `electron-node.yml` (2), `platform-ops.yml` (2), `release-npm-61.yml` (2), `release-npm-node.yml` (2), `release-providers.yml` (1), `release-wheels.yml` (1)
- **Claim/Break:** El guard `if: github.actor != 'dependabot[bot]'` se aplicó a **los 18 jobs que producen los 18 required checks** del ruleset `main` (11 de ci-rust + 6 de docs-gates + 1 de ci-fleet) y a los 10 del ruleset `develop` (`gh api .../rulesets/23692587`). El propio commit bb12f2c3 declara la consecuencia: *"ningun PR de Dependabot reporta ya sus checks requeridos, asi que no se puede mergear solo."*
- **Why it matters:** La consecuencia real es la **inversa** de la declarada, y por tanto más peligrosa. GitHub trata un job omitido por `if:` como un check con conclusión *skipped*, que **satisface** el required status check. Es decir: cualquier PR de Dependabot queda *mergeable sin que se ejecute ni un solo check* — se pierde el `cargo audit`, `cargo deny`, `osv-scanner`, `cargo-machete`, `actionlint`, Miri, ASan y TSan en ese PR. El commit asume "sin reports = no mergeable"; el efecto real sería "sin reports = verde por omisión". *(NO VERIFICADO: no pude descargar la página oficial de GitHub sobre required status checks — los dos URLs que intenté devolvieron 404 y websearch devolvió HTTP 403. La afirmación se basa en el comportamiento documentado de checks `skipped`; confirmarla con `gh api repos/ness-e/Vantadb/commits/<sha>/check-runs` sobre un PR real de Dependabot.)*
- **Fix:** No aplicar el guard a los checks baratos y de seguridad (`audit`, `osv`, `deny`, `machete`, `actionlint`, `fmt`, `clippy`): esos consumen segundos. Alternativa: añadir un job agregador barato "Dependabot gate" que sea el único required y ejecute los checks críticos; o acotar el guard a los jobs caros (`test-windows`, `test-macos`, `coverage`, `sanitizer-*`, `miri`).
- **Verify:** Abrir un PR de Dependabot y listar `gh api repos/ness-e/Vantadb/commits/<sha>/check-runs --jq '.check_runs[] | "\(.name) \(.conclusion)"'` para ver si los 18 contexts quedan `skipped` (y entonces GitHub los cuenta como opción) o ausentes.

### S1-08 | P2 | action pins inexistentes
- **Location:** `.github/workflows/release-wheels.yml:198`, `.github/workflows/release-providers.yml:268`, `.github/workflows/release-adapters.yml:157`, `.github/workflows/release-npm-node.yml:245` → `actions/attest-build-provenance@78e6cbd37d0ac1a40113c04f2037dacf1ea3f12e # v4.1.1`; `.github/workflows/certification.yml:375` → `actions/upload-artifact@4e7e4d1bb8e5ae8b5b8e6dad68a47e8afbd4d76a # v4.5.0`
- **Claim/Break:** Ambos SHA **no existen** en sus repos upstream:
  - `GET /repos/actions/attest-build-provenance/commits/78e6cbd37d0ac1a40113c04f2037dacf1ea3f12e` ⇒ HTTP 422 *"No commit found for SHA"*; `GET .../git/commits/78e6cbd…` ⇒ HTTP 404. El tag real `v4.1.1` apunta a `0f67c3f4856b2e3261c31976d6725780e5e4c373`.
  - `GET /repos/actions/upload-artifact/commits/4e7e4d1bb8e5ae8b5b8e6dad68a47e8afbd4d76a` ⇒ HTTP 422; `.../git/commits/4e7e4d1b…` ⇒ HTTP 404. El tag real `v4.5.0` apunta a `6f51ac03b9356f520e9adb1b1b7802705f340c2b`.
  Los otros 18 pins del repo **sí** existen (verificados uno a uno vía API, incluido `release-plz/action@b8d6b54b…` ≡ tag `v0.5.139`, confirmado con `GET /repos/release-plz/action/compare/b8d6b54b…v0.5.139` ⇒ `identical`). Los dos pins rotos son **preexistentes en `origin/main`** (no los introduce esta rama).
- **Why it matters:** GitHub no puede resolver la acción ⇒ el step falla con *"Unable to resolve action …"*. Hoy el daño está enmascarado porque los 4 pasos de `attest-build-provenance` llevan `continue-on-error: true` (`release-wheels.yml:199`, `release-providers.yml:269`, `release-adapters.yml:156`, `release-npm-node.yml:246`) y el de `upload-artifact` está en el lane pesado. Pero la **atestación de procedencia de cada artefacto publicado** no se está haciendo, y el release de 0.9.0 publicará 15 artefactos sin provenance.
- **Fix:** Re-pinnejar a `actions/attest-build-provenance@0f67c3f4856b2e3261c31976d6725780e5e4c373 # v4.1.1` y `actions/upload-artifact@6f51ac03b9356f520e9adb1b1b7802705f340c2b # v4.5.0`. Considerar quitar `continue-on-error: true` de la atestación de los lanes de publicación (o al menos que su fallo sea visible como warning no bloqueante en el run summary).
- **Verify:** `gh api repos/actions/attest-build-provenance/commits/0f67c3f4…` ⇒ 200; en un run de `release-wheels.yml`, el paso de atestación debe aparecer como `success` (no como step fallido ignorado).

### S1-09 | P2 | fan-out (doble publicación concurrente)
- **Location:** `.github/workflows/release.yml:182-188` (los 7 `gh workflow run`), frente a los triggers por tag de `.github/workflows/release-wheels.yml:27`, `release-adapters.yml:17`, `release-providers.yml:26`, `release-npm-61.yml:23`, `release-npm-node.yml:18`, `release-sbom.yml:4`
- **Claim/Break:** La premisa del fan-out (`release.yml:150-152`) es *"Los tags creados por release-plz con GITHUB_TOKEN no disparan esos workflows (GitHub suprime eventos derivados)"*. Es **falsa aquí**: `release.yml:55` entrega a release-plz `secrets.RELEASE_PLZ_TOKEN` (un PAT, no un GITHUB_TOKEN) y los eventos creados con un PAT **sí** disparan workflows. Prueba empírica: run `37045932844` — `release-wheels.yml`, `event=push`, `ref=v0.8.0` (el tag del release 0.8.0) ⇒ success; y no existe **ningún** run `workflow_dispatch` de `release-wheels.yml` en el historial. El fan-out, por tanto, **duplica** el trabajo.
- **Why it matters:** Para v0.9.0 se dispararán 6 workflows por el push del tag **y** 6 dispatches por el fan-out ⇒ dos publicaciones concurrentes de la misma versión. `twine --skip-existing` (PyPI) y el paso "Check if version already published" (npm, `release-npm-61.yml:230-241`, `323-334`; `release-npm-node.yml`) son **TOCTOU**: consultan el registro y luego suben. Si ambas consultas ocurren antes de que ninguna suba, la perdedora recibe HTTP 400/403 y el job queda rojo aunque el artefacto quede publicado. Consecuencia: run rojo y ruido (y runner gastado) en el release en vuelo; en el peor caso, dos builds distintos compitiendo por la misma versión.
- **Fix:** Elegir **un** mecanismo. Si se queda el fan-out (que es lo necesario si el tag lo crea un GITHUB_TOKEN), eliminar los triggers `push: tags:` de los 6 workflows. Si se dejan los triggers por tag, eliminar el job `fanout` de `release.yml:156-188`.
- **Verify:** Tras el tag v0.9.0: `gh run list --workflow release-wheels.yml --event push` y `--event workflow_dispatch` deben mostrar **un** run, no dos.

### S1-10 | P2 | release-plz.toml (`vantadb-wasm` publicable)
- **Location:** `release-plz.toml:4-8` (`[workspace] publish = true`), `release-plz.toml:52-57` (`vantadb-wasm`, `release = false`), `vantadb-wasm/Cargo.toml:1-3` (sin `publish = false`)
- **Claim/Break:** `release-plz.toml:6` pone `publish = true` para todo el workspace y la exclusión de `vantadb-wasm` depende **sólo** de `release = false` en `release-plz.toml:57`. Su `Cargo.toml` **no** tiene `publish = false` (a diferencia de `vantadb-python:8`, `vantadb-server:7`, `vantadb-mcp:8`, `vantadb-ffi-core:8`, `vanta-proxy:7`). Verificado empíricamente que hoy el estado es correcto: crates.io sólo tiene `vantadb` (0.8.0); `vantadb-python`, `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`, `vantadb-ffi-core`, `vanta-memory` y `vanta-proxy` **no existen** en crates.io ⇒ los `release = false` / `publish = false` se respetan.
- **Why it matters:** Defence-in-depth: `vantadb-wasm` es publicable por semántica de Cargo y sólo queda fuera por una entrada de configuración. Si alguien quita ese `[[package]]` (o release-plz cambia la precedencia de `release=false` vs `publish=false`), `vantadb-wasm` se publica a crates.io en el release de 0.9.0 y `release-manifest.json` (15 entradas) queda incompleto sin que nada lo detecte. `vanta-memory` sí debe quedar publicable (DIST-01), así que no aplica.
- **Fix:** Añadir `publish = false` a `vantadb-wasm/Cargo.toml` (mismo patrón que `vantadb-ffi-core`), y dejar el `release = false` de `release-plz.toml` como segunda capa.
- **Verify:** `cargo metadata --no-deps --format-version 1 | jq '.packages[] | select(.name=="vantadb-wasm") | .publish'` ⇒ `[]` (no publicable) sin depender de la config de release-plz.

### S1-11 | P2 | docs de publish contradicen el manifest
- **Location:** `scripts/verify-release.ps1:17`, `docs/dev/workflow/PUBLISH.md:97`, `docs/dev/workflow/PUBLISH.md:225`, `docs/dev/workflow/PUBLISH.md:372`
- **Claim/Break:** El commit 7c491b9f cambió la premisa del manifest ("the fan-out publishes them with the core tag") y pasó `vantadb-node`, los 7 adapters y los 3 providers de `documented_only` a `registries`. Pero la documentación canónica del flujo de publish sigue afirmando lo contrario:
  - `verify-release.ps1:17` — *"`vantadb-node` was never published to npm - not checked."* (el propio script ahora **sí** lo verifica: `release-manifest.json:39`).
  - `PUBLISH.md:97` — *"Excepción `vantadb-node`: **nunca publicado** en npm (registry 404, 2026-10-03)."*
  - `PUBLISH.md:225` — *"npm es el canal; never published yet (404 documented)"*.
  - `PUBLISH.md:372` — *"Never published (404 documented) - excluded from the check."*
  - El `_comment` anterior de `release-manifest.json:2` era la fuente de esa premisa: *"entries that ride other tag namespaces (adapters-v*, providers-v*, node-v*) carry documented_only status instead of a core version"* — la rama cambió el `_comment` pero no la doc que lo reflejaba.
  Nota: `PUBLISH.md:97` también instruye *"la entrada `TARGETS` del script - `scripts/docs/check-npm-versions.mjs` - pasa manualmente a `policy: 'workspace'`"* — **ya está hecho** (`check-npm-versions.mjs:56-60` tiene `vantadb-node` con `policy: 'workspace'`), luego esa instrucción está obsoleta y no bloquea nada.
- **Why it matters:** `PUBLISH.md` es el runbook del release (referenciado desde `release-manifest.json:2` y `verify-release.ps1:24-26`). Quien ejecute el post-release verification después de 0.9.0 leerá "vantadb-node nunca publicado" mientras el manifest exige verificarlo. Y la doc de "nunca publicado" es la que justificó el `documented_only` que esta rama acaba de retirar: si alguien re-litera el texto antiguo, reintroduce el bug FIND-342.
- **Fix:** Actualizar `verify-release.ps1:14-17` y `PUBLISH.md:97,225,372` para reflejar que adapters/providers/vantadb-node se publican con el tag del core y se verifican; quitar la instrucción obsoleta de `policy: 'workspace'`.
- **Verify:** `pwsh scripts/verify-release.ps1 -Tag v0.9.0` tras el release ⇒ las 15 líneas de registry deben salir `OK`; y `grep -rn "never published" docs/dev/workflow/PUBLISH.md scripts/verify-release.ps1` ⇒ 0 coincidencias.

### S1-12 | P3 | release.yml (cabecera obsoleta)
- **Location:** `.github/workflows/release.yml:1-3`, `.github/workflows/release.yml:27`
- **Claim/Break:** La cabecera sigue diciendo *"publish to crates.io **via Trusted Publishing (OIDC)**. Requires: crates.io Trusted Publisher for vantadb with workflow file release.yml."* — pero `release.yml:56-66` publica **por token**. `id-token: write` (`release.yml:27`) sigue concedido sin uso. Y `docs/dev/tasks/AUD-027.md:41` afirma *"el workflow tiene `id-token: write` y NO define `CARGO_REGISTRY_TOKEN`"* — ya falso.
- **Why it matters:** Un mantenedor que lea la cabecera creerá que el OIDC está activo y que borrar el token no afecta al publish (justo el error que costó el run 39368867477).
- **Fix:** Actualizar la cabecera a "publish by crates.io API token (`CARGO_REGISTRY_TOKEN`); Trusted Publishing (OIDC) not used" y quitar `id-token: write` si nada más lo necesita.
- **Verify:** `grep -n "Trusted Publishing" .github/workflows/release.yml` ⇒ 0 (o la nota explícita).

### S1-13 | P3 | release-wheels (sin dispatch)
- **Location:** `.github/workflows/release-wheels.yml:179`
- **Claim/Break:** `publish-pypi` tiene `if: startsWith(github.ref, 'refs/tags/v')` **sin** rama `workflow_dispatch`, mientras sus hermanos sí la tienen: `release-adapters.yml:135-138` y `release-providers.yml:247-250` aceptan `(github.event_name == 'workflow_dispatch' && inputs.publish_pypi != 'false' && inputs.publish_testpypi != 'true')`.
- **Why it matters:** Quita la palanca manual de re-publicación de las wheels sin crear un tag — relevante si el fan-out falla y hay que reintentar a mano durante el release en vuelo. También es la incoherencia que hace dudar de si el input `publish_testpypi` de `release-wheels.yml` (único input suyo) es funcional.
- **Fix:** Replicar la condición de los hermanos: `startsWith(github.ref, 'refs/tags/v') || (github.event_name == 'workflow_dispatch' && inputs.publish_testpypi != 'true')`.
- **Verify:** `gh workflow run release-wheels.yml -f publish_testpypi=false` ⇒ el job `publish-pypi` debe planificarse.

### S1-14 | P3 | referencia rota a DIST-01
- **Location:** `release-plz.toml:68`, `vanta-memory/Cargo.toml:16`
- **Claim/Break:** Ambos referencian `docs/dev/tasks/DIST-01.md`, que **no existe** (búsqueda recursiva: sólo existe `docs/dev/tasks/DIST-05.md`). El contenido de DIST-01 está disperso en `docs/dev/tasks/MEMG-*.md`, `docs/dev/workflow/PUBLISH.md` y `docs/dev/avance/activo/vanta-memory.md`.
- **Why it matters:** El commit 2e8152a3 dice "el sync debe seguir el spec de vanta-memory" — el "spec" es el requisito de path-dep de `vanta-memory/Cargo.toml:21`, que **sí** está en el repo y el script **sí** sigue (verificado). Pero la referencia al documento normativo del hold es un enlace roto, justo en el fichero que gobierna si `vanta-memory` se publica.
- **Fix:** Reemplazar la referencia por el documento real que hoy contiene la decisión, o recrear `docs/dev/tasks/DIST-01.md` como índice.
- **Verify:** `Test-Path docs/dev/tasks/DIST-01.md` ⇒ True, o que las dos referencias apunten a un fichero existente.

### S1-15 | P3 | release-plz.toml (cita de rutas upstream)
- **Location:** `release-plz.toml:31-33`
- **Claim/Break:** El comentario cita `release_plz_core version.rs` (`SemverCheck::Incompatible` → `VersionIncrement::breaking`) y `next_version version_increment.rs`. En el upstream actual la lógica vive en `crates/release_plz_core/src/semver_check.rs` (enum `SemverCheck`, `parse_semver_check_output`) y `crates/release_plz_core/src/command/update/updater.rs` (`should_check_semver`, línea ~1078). La **sustancia** de la afirmación es correcta: `breaking(0.8.0)` → minor → 0.9.0 (verificado con el log del run 39353698789: *"vantadb: next version is 0.9.0 (⚠️ API breaking changes)"*).
- **Why it matters:** Referencia a rutas que no existen; un mantenedor que quiera verificar la afirmación no las encuentra. Impacto bajo (la afirmación es cierta).
- **Fix:** Actualizar las rutas citadas a `crates/release_plz_core/src/semver_check.rs` y `crates/release_plz_core/src/command/update/updater.rs`.
- **Verify:** Los paths existen en `release-plz/release-plz` (verificado vía API).

### S1-16 | P3 | release.yml (`git add -A`)
- **Location:** `.github/workflows/release.yml:131`
- **Claim/Break:** `git add -A` stagea todo el worktree. El script, además de los 22 targets, ejecuta `gen-surface-manifest.mjs --write` y `gen-index.mjs --write` (`sync-workspace-version.mjs:529-550`), que reescriben `docs/api/SURFACE_MANIFEST.md`, `docs/index.md` y `llms.txt`. El commit resultante incluye esos tres ficheros.
- **Why it matters:** `git add -A` es un "stage-everything": cualquier fichero generado o modificado por una de esas regeneraciones entra en el commit del Release PR. Hoy el checkout es limpio, así que el riesgo es bajo, pero el radio de acción del commit no está acotado.
- **Fix:** `git add` explícito con una lista, o `git add -- <los 22 targets> docs/api/SURFACE_MANIFEST.md docs/index.md llms.txt`.
- **Verify:** En un run de prueba, listar `git show --stat` del commit de sync y comprobar que sólo contiene los ficheros esperados.

### S1-17 | P3 | sync script (`transformNodeLockJson` posicional)
- **Location:** `scripts/docs/sync-workspace-version.mjs:184-192`
- **Claim/Break:** El transform cuenta las apariciones de `"version":` y sustituye **las 2 primeras** (`seen <= 2`), asumiendo que son la raíz y `packages[""]`. En el lockfile real (`vantadb-ts/package-lock.json`) el orden es correcto (línea 3 = raíz, línea 9 = `packages[""]`, línea 30+ = dependencias) — verificado. Pero la asunción es puramente posicional: si npm reordena el lockfile, o si el lockfile pierde el `"version"` raíz, la segunda sustitución caería sobre **la versión de la primera dependencia** (p. ej. `"../vantadb-node": {"version": "0.7.0"}` → "0.9.0"), corrompiendo el lockfile silenciosamente (el script no valida el resultado del transform).
- **Why it matters:** Es un transform destructivo cuyo error es silencioso y cuyo radio de impacto es un lockfile publicado dentro del tarball npm. La fragilidad se agrava por S1-05 (el modo write nunca falla).
- **Fix:** Reemplazar el contador por un parseo estructural: `JSON.parse` → `obj.version` y `obj.packages[""].version` → re-serializar preservando formato; o al menos verificar con un regex que el 2º match está dentro del bloque `packages[""]`.
- **Verify:** Con el lockfile real: tras el transform, `node -e "console.log(require('./vantadb-ts/package-lock.json').packages['../vantadb-node'].version)"` debe seguir siendo `0.7.0`.

### S1-18 | P3 | manifest (`documented_only` incompleto)
- **Location:** `release-manifest.json:41-46`
- **Claim/Break:** `documented_only` sólo lista `crates.io vanta-memory`. `vantadb-wasm` **tampoco** se publica a crates.io (`release-plz.toml:57`, `release = false`) y no aparece en ninguna lista.
- **Why it matters:** `verify-release.ps1:92` imprime cada `documented_only` como SKIP con su motivo — es el registro visible de "qué NO se publica y por qué". `vantadb-wasm` es un crate publicable por semántica de Cargo (S1-10) que queda fuera del registro.
- **Fix:** Añadir `{ "label": "crates.io vantadb-wasm", "reason": "release = false in release-plz.toml (npm-distributed via wasm-pack)" }`.
- **Verify:** `pwsh scripts/verify-release.ps1 -Tag v0.9.0` debe imprimir el SKIP de `vantadb-wasm` y de `vanta-memory`.

### S1-19 | P3 | dependabot (ecosistema actions sin PRs)
- **Location:** `.github/dependabot.yml:200-220`
- **Claim/Break:** `open-pull-requests-limit: 0` para `github-actions` (línea 220), alineado con npm/pip/providers. Decisión del owner, documentada en el comentario (líneas 214-219). Consecuencia: **ningún** bump de action se propondrá automáticamente — y este repo tiene 20 workflows con 20 pins SHA, dos de los cuales ya no existen (S1-08) y cuya corrección manual queda sin ninguna alerta que lo sugiera.
- **Why it matters:** El riesgo de seguridad de los pins (SHA inexistente) no se cierra por sí solo; la "alerta" de Dependabot es justamente el mecanismo que lo habría detectado. El propio repositorio tiene dos pins rotos que el modo alertas habría delatado.
- **Fix:** Aceptado por el owner — pero compensar: (a) añadir un check propio que valide que todo `uses: owner/repo@<sha>` resuelve (evita S1-08), y (b) revisar manualmente los pins mensualmente.
- **Verify:** Un script que itere los `uses:` de `.github/workflows/` y llame a la API de GitHub por cada SHA; debe dar 0 pines no resolubles (hoy da 2).

---

## Verificaciones positivas (cosas que están bien y conviene preservar)

1. **Sincronización YAML sin claves duplicadas.** Loader PyYAML estricto (constructor que lanza ante cualquier clave duplicada) sobre los 20 workflows de la rama: **0 duplicados, 0 errores de parseo**. El fix de ec441d55 (que duplicaba la clave `if:`) es real y está correctamente aplicado.
2. **`actionlint` pasa.** Ejecutado localmente con actionlint v1.7.7 (mismo binario y mismo `-ignore` que el job de `docs-gates.yml:596-604`): **exit 0**, sin hallazgos. El job `actionlint (workflows lint + inventory gate)` es required check y no bloqueará el merge. *(shellcheck no está instalado en mi máquina ⇒ el lint de shell no se ejecutó localmente; en `ubuntu-latest` sí lo está.)*
3. **El "inventory gate" cuadra.** `docs-gates.yml:606-611` compara `ls .github/workflows/*.yml | wc -l` con el número de `docs/dev/operations/CI_POLICY.md:16` → **20 == 20**. ✓
4. **Los 20 pins SHA del repo existen**, salvo los 2 de S1-08. Verificado uno a uno contra la API de GitHub. `release-plz/action@b8d6b54b…  # v0.5.139` es **correcto** (compare API ⇒ `identical` con el tag `v0.5.139`), y ese action **instala `cargo-semver-checks@0.50`**, por lo que el check semver de release-plz **sí** se ejecuta.
5. **El script de sync funciona y cubre los 22 targets.** Ejecutado sobre el árbol exacto de la rama: `--self-test` ⇒ **12/12 assertions**; `--check` ⇒ **"OK: All 22 manifests synchronized with workspace version 0.9.0"** (exit 0). Los 22 = 6 base + 9 `integrations/*/pyproject.toml` + 3 `providers/*/Cargo.toml` + `SURFACE_MANIFEST.md` + `release-manifest.json` + `vantadb-ts/package-lock.json` + `vanta-memory/Cargo.toml`. El recuento de 22 del enunciado es correcto.
6. **El sync sigue el "spec de vanta-memory".** `vanta-memory/Cargo.toml:21` = `vantadb = { path = "../", version = "0.9.0", default-features = false }`; el transform `transformVantaMemoryDepRequirement` (`sync-workspace-version.mjs:220-225`) casa exactamente con ese patrón y el `--check` lo confirma. Además, `vanta-memory/Cargo.toml:3` usa `version.workspace = true` (no requiere sync). Y el path-dep ya está en 0.9.0 en `origin/main` (verificado: `git diff origin/main…branch` sobre los 22 targets ⇒ **sólo** cambia `release-manifest.json`) ⇒ **el workspace de main resuelve** y el fix de 2e8152a3 protege el siguiente bump (0.10.0), que es el escenario que el commit reproduce.
7. **`release-manifest.json`: los 15 registries son correctos y completos.** Cross-check contra los workflows **y** contra los registries vivos:
   - 15 entradas en `release-manifest.json:25-39` = 1 crates.io + 2 npm TS/wasm + 1 npm node + 1 PyPI vantadb-py + 7 adapters + 3 providers.
   - Cada una la publica un workflow del fan-out (`release.yml:182-188`): wheels → `release-wheels.yml:174`, adapters → `release-adapters.yml:130`, providers → `release-providers.yml:242`, npm → `release-npm-61.yml:170,254`, node → `release-npm-node.yml:162`.
   - Cada workflow es disparado por el fan-out con la condición que lo habilita (p. ej. `release-wheels.yml:179` `startsWith(github.ref,'refs/tags/v')` ⇒ TRUE en un dispatch sobre el tag).
   - Nombres de distribución correctos: adapters `vantadb-<adapter>` (`integrations/*/pyproject.toml`), providers `vantadb-<provider>` (`providers/*/pyproject.toml`), wheels `vantadb_py` (maturin, `dynamic=["version"]` ⇒ hereda 0.9.0).
   - Todo registry realmente publicado está listado: crates.io sólo publica `vantadb` (verificado: los otros 7 crates del workspace **no existen** en crates.io); PyPI publica `vantadb-py` + 7 adapters + 3 providers; npm publica `vantadb`, `vantadb-wasm` y (a partir de 0.9.0) `vantadb-node`. No falta ni sobra ninguna entrada.
   - Los assets del manifest también cuadran: `github.binaries` (5 targets) == matriz de `release-binaries.yml:63-74`; `github.wheels.platforms` (4) == matriz de `release-wheels.yml` (4 entradas).
8. **`release-plz.toml` es coherente con los flags de publish.** `vantadb` (workspace, `version.workspace = true` ⇒ 0.9.0) se publica; `vantadb-wasm` y `vanta-memory` tienen `release = false`; los demás miembros del workspace tienen `publish = false` en su `Cargo.toml`. Los tres paquetes listados resuelven su versión desde `[workspace.package] version = "0.9.0"` (root `Cargo.toml`), así que release-plz los bumpea todos.
9. **Los environments no requieren aprobación manual** (`GET /repos/ness-e/Vantadb/environments/{pypi,npm,testpypi}` ⇒ `protection_rules: []`), luego los jobs de publish del fan-out corren desatendidos.
10. **El fix de la autenticación del push (ea6a0b26) es correcto.** `release.yml:141-143` usa el mecanismo `http.<url>.extraheader` estándar de `actions/checkout` con `base64 -w0` (GNU coreutils, presente en `ubuntu-latest`); `set -euo pipefail` (`:118`) hace que un fallo del push tumbe el step en vez de dejarse pasar.
11. **El guard de Dependabot no salta ningún job necesario en el camino del release.** La condición `github.actor != 'dependabot[bot]'` sólo es falsa para actor `dependabot[bot]`; el fan-out se dispara con el PAT del owner (`github.actor = ness-e`), y el push del tag lo hace release-plz con el PAT ⇒ ningún job de publicación queda omitido. Los 20 workflows parsean, actionlint pasa y no hay `if:` duplicados.

---

## NO VERIFICADO

| Afirmación | Por qué no se pudo verificar |
|---|---|
| Si un check `skipped` satisface o no un required status check (S1-07) | Las dos páginas oficiales de GitHub que intenté devolvieron 404 y websearch devolvió HTTP 403. Confirmar sobre un PR real de Dependabot con `gh api .../check-runs`. |
| Lint de shell (shellcheck) de los nuevos `run: |` | shellcheck no está instalado en la máquina de auditoría; el lint de shell de actionlint se saltó. En `ubuntu-latest` sí corre. El único bloque nuevo es `release.yml:141-143` (4 líneas, sin construcciones que shellcheck marque fuera de la lista ignorada SC2012\|SC2086\|SC2129\|SC2044). |
| Que release-plz vuelva a intentar el publish de 0.9.0 tras el fix | No ejecutable en modo read-only. Indicio fuerte: el run 39368867477 intentó publicar 0.9.0 desde un commit de merge que **no** bumpeaba la versión (`66f9a775` = "Merge PR #238 … release v0.9.0", con `Cargo.toml` ya en 0.9.0) ⇒ la selección de paquetes es **por versión publicada vs. actual**, no por último commit ⇒ tras mergear #259, crates.io (0.8.0) < main (0.9.0) ⇒ debe reintentarse. Conviene confirmarlo en el primer run post-merge. |
| Comportamiento de `cargo` ante `CARGO_REGISTRY_TOKEN` vacío vs. OIDC (preferencia por presencia) | Confirmado sólo el *error* (`"please provide a non-empty token"` en `rust-lang/cargo` → `crates/crates-io/lib.rs`, `TokenError::Empty`, `check_token()`). La lógica de decisión token-vs-OIDC no la localicé en `cargo` master (no encontré el módulo OIDC por búsqueda de código). La conclusión operativa no depende de ello: con el secret presente y no vacío (verificado), se usa el token. |
| Estado de `pypi`/`npm`/`testpypi` environments tras un posible cambio concurrente | Leído en el momento de la auditoría: `protection_rules: []` en los tres. |
| Efecto real de los 68 guards sobre la lista de required checks de `develop` | Se verificó el mecanismo y la lista de required checks; el comportamiento exacto de GitHub ante `skipped` es el punto NO VERIFICADO de arriba. |

---

## Coverage

### Revisado en profundidad (con evidencia primaria)
- **Los 9 commits** del branch, sus mensajes completos y su diff (`git diff origin/main…origin/release-plz-fix-cargo-token`, 926 líneas, 17 ficheros).
- **Los 20 workflows** de la rama: YAML parseado con loader estricto (duplicados), inventario de jobs/`if:`/`needs:`/`continue-on-error`/concurrency (script propio), **actionlint v1.7.7 ejecutado localmente** (exit 0), y los 20 pins SHA contrastados uno a uno con la API de GitHub.
- **`release.yml`** completo (188 líneas, numerado), incluida la lógica del fan-out, la autenticación del push y el paso de sync.
- **`ci-rust.yml`**: los 21 guards, el job `semver-checks` completo, el filtro `paths`.
- **`docs-gates.yml`**: los 18 guards, el job `actionlint` + inventory gate, el filtro `paths`.
- **`ci-fleet.yml`**: los 4 guards, la ampliación de `paths` (líneas 34-35, 53-54).
- **`release-plz.toml`** completo (105 líneas) contrastado con el upstream de release-plz (`website/docs/semver-check.md`, `crates/release_plz_core/src/semver_check.rs`, `crates/release_plz_core/src/command/update/updater.rs`, `crates/release_plz_core/src/command/update/packages_update.rs`) y con `rust-lang/cargo` (`crates/crates-io/lib.rs`).
- **`scripts/docs/sync-workspace-version.mjs`** completo (563 líneas): los 22 targets, los 12 self-tests, los transforms, y **ejecutado** (`--self-test` 12/12, `--check` OK, `--write` con target ausente ⇒ exit 0 — S1-05).
- **`release-manifest.json`** completo, contrastado con los 8 workflows publicadores y con los 3 registries vivos (crates.io, PyPI, npm) y con `scripts/verify-release.ps1:64-103`.
- **Reglas de GitHub**: `GET /repos/ness-e/Vantadb/rulesets/{16844590,23692587}` (18 y 10 required checks), `GET .../environments/{pypi,npm,testpypi}`, `gh secret list` (5 secrets).
- **Estado real del release**: runs `39353698789`, `39368867477` (logs completos), `37045932844`, `37045089945`; `GET /repos/ness-e/Vantadb/releases`; `git ls-remote --tags`; `GET /repos/ness-e/Vantadb/pulls`; `crates.io`/`pypi.org`/`registry.npmjs.org` (versiones vivas).
- **Estado de los manifiestos**: `git diff origin/main…branch` sobre los 22 targets (sólo cambia `release-manifest.json`), `Cargo.toml` de raíz y de los 8 miembros del workspace, `tests/version_coherence.rs` completo.

### Revisado superficialmente
- `certification.yml`, `nightly.yml`, `platform-ops.yml`, `demo-acceptance.yml`, `electron-node.yml`: YAML válido, sin duplicados, guards bien colocados, `actionlint` limpio. No se auditó la lógica de sus pasos.
- `release-sbom.yml`, `release-binaries.yml`, `release-verify.yml`, `perf-bench.yml`, `publish-mcp-registry.yml`, `ci-gate.yml`: leídos y sin hallazgos; no están en el diff.
- `scripts/verify-release.ps1`: leído hasta la línea 140; **no** auditado el smoke-test de binarios (líneas 140+).
- `scripts/docs/check-npm-versions.mjs`: leídas las 120 primeras líneas (policy + parser). **No** auditado el resto del evaluador.
- `docs/dev/workflow/PUBLISH.md`: revisadas las líneas relevantes (97, 225, 372); no auditado el resto del runbook.

### No revisado (fuera de alcance o sin acceso)
- Contenido de `docs/` en general (auditoría de docs: no es mi dominio). Sólo las líneas tocadas por la premisa del manifest.
- `integration/`, `provider/` y `adapter` **código** de runtime; sólo sus manifiestos de versión/empaquetado.
- La semántica interna de `release-plz` más allá de `semver_check` (pub, changelog, PR) — verificado sólo lo que afecta a la ruta de publish.
- Los `.github/actions/*` compuestos (sólo `rust-setup`, leído entero); no se revisó `sccache-action` ni otros actions externos.
- Configuración de `cargo`/`crates.io` de la máquina y credenciales reales (sólo presencia del secret vía API).
- Cualquier pipeline de CI/CD fuera de `.github/workflows/` y `.github/dependabot.yml`.
