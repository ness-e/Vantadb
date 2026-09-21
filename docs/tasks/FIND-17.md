# FIND-17: Identidad de marca inconsistente — auditoría de nombres + convención única pre-launch

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-desktop-ux-core.md (Task 6, Wave 1)
- **Fuente:** Backlog FIND-17 (P33)
- **Esfuerzo:** 🟡 3h
- **Prioridad:** 🟢
- **Tipo:** docs/auditoría de marca (vanta-docs) — NO código de negocio
- **Creado:** 2026-08-25
- **last-synced:** 2026-08-25
- **Estado:** ✅ COMPLETED (auditoría + ADR-030 + nota README; commit del lead)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3 (auditoría → ADR → nota README)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Artefactos auditados | `Cargo.toml` (crate `vantadb` + workspace), `vantadb-python/pyproject.toml` (PyPI `vantadb-py`), `vantadb-ts/package.json` (npm `vantadb`), `vantadb-node/package.json` (npm `vantadb-node`), `vantadb-wasm/Cargo.toml`, `vantadb-server/Cargo.toml`, `vantadb-mcp/Cargo.toml`, `vanta-memory/Cargo.toml`, `vanta-proxy/Cargo.toml`, `README.md`, `README_ES.md`, `CONTRIBUTING.md`/`SECURITY.md`/`SUPPORT.md` (referencias repo), `docs/` (65+ citas `ness-e/Vantadb`) |
| Registries (verificación live) | crates.io `vantadb` ✅ · PyPI `vantadb-py` ✅ · npm `vantadb` ✅ · npm `vantadb-node` ❌ (404, nunca publicado) · GitHub `ness-e/Vantadb` ✅ · `vantadb.dev` ❌ (DNS muerto) · GitHub About → `vantadb.vercel.app` (live) |
| Implicaciones | NO renombrar crates/packages (rompería semver/usuarios — STOP CONDITION del plan). Cambios = solo docs (ADR + nota README). Cero cambios de código o metadata de packaging. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `Cargo.toml` (661L), `vantadb-python/pyproject.toml` (53L), `vantadb-ts/package.json` (66L), `vantadb-node/package.json` (43L), `README.md` (441L), `vantadb-wasm/Cargo.toml` (1-40), `vantadb-server/Cargo.toml` (1-30), `vantadb-mcp/Cargo.toml` (25L), `vanta-memory/Cargo.toml` (1-30), `vanta-proxy/Cargo.toml` (1-30), `SECURITY.md` (grep), `SUPPORT.md` (grep), `README_ES.md` (grep), `vantadb-ts/README.md` (1-40), `.github/workflows/*` (glob — todos los archivos de badges existen), `docs/_templates/adr.md`.
- **Archivos referenciados hacia dentro:** `ness-e/Vantadb` citado en 65+ lugares en `docs/` (Backlog, blog, QUICKSTART, FAQ, master-index, operations, strategy, glosario, reviews, historial). `vantadb.dev` en Cargo.toml (homepage), vantadb-ts/package.json (homepage), vantadb-wasm/Cargo.toml (homepage), SUPPORT.md (email enterprise@vantadb.dev).
- **Archivos que referencian a los editados:** `docs/architecture/adr/` (índice de ADRs) + README (enlace a la nota de convención). ADR-030 es nuevo (número 030 libre).
- **Veredicto impacto:** **bajo** — solo se CREA `docs/architecture/adr/ADR-030-brand-identity-naming-convention.md` y se EDITA `README.md`/`README_ES.md` (nota corta). No se toca metadata de packaging ni código. STOP CONDITION respetada: cero renames.

## Contrato
1. Auditoría de nombres documentada (tabla artefacto → nombre actual → nombre decidido).
2. Convención decidida y documentada (ADR o nota) — IA aporta evidencia; decisión final del owner (Regla 5). ADR marcado PROPOSED para confirmación del owner.
3. Links/URLs actualizados donde no rompen — hallazgo: NO hay badges rotos (todos los workflows referenciados existen en `.github/workflows/`); única URL muerta = `vantadb.dev` (decisión de dominio del owner, NO se cambia en este batch).
4. NO renombrar artefactos. NO tocar código. NO commit (lead).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) nombres de crates/packages/imports intactos (renames = breaking semver, STOP CONDITION); (2) repositorio canónico `ness-e/Vantadb` intacto (AUD-29 ya unificó; GitHub es case-insensitive en routing pero la URL canónica publicada usa `Vantadb`); (3) homepage `vantadb.dev` NO se cambia hasta decisión del owner (dominio comprado, DNS pendiente); (4) docs técnicos en inglés (español solo planning); (5) ADR escrito por IA = evidencia + propuesta, NO decisión final (Regla 5 forcing function).
- **Comandos de verificación:** `scripts/validate-docs-coverage.ps1` (docs coverage) · grep `vantadb.dev` (solo en metadata de packaging + email — documentado, no editado) · lectura de ADR-030 (contenido).
- **Deuda pendiente:** decisión del owner sobre (a) convención ADR-030 (PROPOSED → ACCEPTED); (b) dominio canónico `vantadb.dev` vs `vantadb.vercel.app`; (c) rename futuro opcional de repo a `VantaDB` (case) si se quiere consistencia visual exacta (GitHub redirects, cero riesgo); (d) PyPI owner `DevpNess` vs `ness-e`; (e) `vantadb-node` nunca publicado en npm.

## Recitation (canónico — estructura única)

- `activeGoal`: FIND-17 — auditar consistencia de nombres en artefactos públicos y documentar convención única pre-launch (sin renames).
- `lastAction`: DISCOVERY completo — lectura directa de todos los artefactos (CodeGraph sync deshabilitado), verificación live de 6 registries (crates.io/PyPI/npm×2/GitHub/dominio), mapeo de 65+ citas del repo en docs; task file creado con Regla 0 mapeada.
- `result`: `OK`
- `nextAction`: lead verifica + acepta ADR-030 (PROPOSED → ACCEPTED) + commitea (NO COMMIT del worker).
- `contract`:
  - `verificacion`: `scripts/validate-docs-coverage.ps1` ✅ · ADR-030 presente con tabla de auditoría ✅ · nota README presente ✅ · cero renames (git diff solo docs) ✅
  - `evidencia`:
    - claim: crates.io `vantadb` existe (0.5.0, homepage vantadb.dev, repo ness-e/Vantadb)
      evidencia: https://crates.io/api/v1/crates/vantadb (fetch 2026-08-25)
      confianza: alta
    - claim: PyPI `vantadb-py` existe (0.5.0, owner DevpNess, URLs → ness-e/Vantadb)
      evidencia: https://pypi.org/pypi/vantadb-py/json (fetch 2026-08-25)
      confianza: alta
    - claim: npm `vantadb` existe (0.5.0, homepage vantadb.dev, repo ness-e/Vantadb); npm `vantadb-node` NO existe (404)
      evidencia: https://registry.npmjs.org/vantadb + https://registry.npmjs.org/vantadb-node (fetch 2026-08-25)
      confianza: alta
    - claim: repo GitHub `ness-e/Vantadb` público y live; GitHub About website = `vantadb.vercel.app` (≠ vantadb.dev de los packages)
      evidencia: https://github.com/ness-e/Vantadb (fetch 2026-08-25)
      confianza: alta
    - claim: `vantadb.dev` NO resuelve (dominio comprado sin DNS)
      evidencia: GET https://vantadb.dev → Transport error (2026-08-25)
      confianza: alta
    - claim: README badges NO están rotos — workflows `ci-rust-10.yml`, `gate-docs-21.yml`, `sec-codeql-30.yml`, `heavy-certification-50.yml` existen en `.github/workflows/`
      evidencia: `Get-ChildItem .github/workflows` (2026-08-25)
      confianza: alta
  - `artefactos`: `.opencode/skills/campaign-executor/tasks/FIND-17.md`, `docs/architecture/adr/ADR-030-brand-identity-naming-convention.md`, `README.md`, `README_ES.md`
  - `invariantes`: cero renames; docs en inglés; ADR PROPOSED (owner decide)
  - `deuda`: decisión owner (convención + dominio + PyPI owner + vantadb-node publish)
  - `queda_pendiente`: lead verifica + acepta ADR + commitea; owner resuelve dominio canónico y PyPI ownership
- `nextTask`: FIND-11 (Task 8, mismo plan) o la que el lead asigne.

## Definition of Done (contrato multi-nivel)

| Nivel | Gate |
|-------|------|
| Task | Auditoría documentada (tabla) + ADR/nota creada + links sin roturas + docs coverage 0 gaps |
| Commit | Lo ejecuta el lead (NO COMMIT del worker). Conventional `docs:` |
| Release | No aplica (sin cambios de código ni metadata de packaging) |

## Investigation Notes (auditoría FIND-17 — evidencia)

**Mapeo de nombres por artefacto (2026-08-25, verificación live):**

| # | Artefacto | Nombre actual | Nombre decidido (propuesta) | Estado registry | Repo URL | Homepage |
|---|---|---|---|---|---|---|
| 1 | Producto (display) | **VantaDB** | VantaDB (sin cambio) | — | — | — |
| 2 | Repo GitHub | `ness-e/Vantadb` | `ness-e/Vantadb` (sin cambio; case `Vantadb` documentado como URL canónica publicada) | ✅ live (2⭐) | — | About = `vantadb.vercel.app` |
| 3 | Rust crate core | `vantadb` | `vantadb` (sin cambio) | ✅ crates.io 0.5.0 | ness-e/Vantadb ✅ | `vantadb.dev` ❌ dead |
| 4 | Rust CLI binary | `vanta-cli` | `vanta-cli` (sin cambio) | — | — | — |
| 5 | Crates experimentales | `vantadb-server`, `vantadb-mcp`, `vantadb-wasm` | sin cambio (publish=false; wasm no se publica a crates.io) | — | ness-e/Vantadb (wasm) | `vantadb.dev` ❌ dead (wasm) |
| 6 | Crates internos | `vanta-memory`, `vanta-proxy` | sin cambio (publish=false) | — | — | — |
| 7 | PyPI distribución | `vantadb-py` | `vantadb-py` (sin cambio) | ✅ live 0.5.0 | ness-e/Vantadb ✅ | GitHub |
| 8 | Python módulo | `vantadb_py` (import `vantadb` canónico según README local) | sin cambio | — | — | — |
| 9 | npm TS SDK | `vantadb` | `vantadb` (sin cambio) | ✅ live 0.5.0 | ness-e/Vantadb ✅ | `vantadb.dev` ❌ dead |
| 10 | npm native bindings | `vantadb-node` | `vantadb-node` (sin cambio; NOTA: nunca publicado en npm — 404) | ❌ 404 | (ausente en package.json) | — |
| 11 | Dominio | `vantadb.dev` | decisión owner (DNS pendiente; alternativas: `vantadb.vercel.app` live) | ❌ DNS dead | — | — |

**Inconsistencias detectadas:**
1. **Homepage dispar:** packages (crate/npm/wasm) dicen `vantadb.dev` (muerto) vs GitHub About dice `vantadb.vercel.app` (live).
2. **`vantadb-node` nunca publicado** en npm (package.json existe, registry 404).
3. **PyPI owner `DevpNess`** vs crates.io owner `ness-e` vs GitHub `ness-e` — identidad de cuenta dispar entre registries.
4. **Metadata PyPI stale:** summary "Source-installed Python bindings…" + description en español (README viejo publicado en 0.5.0).
5. **Case del repo:** `Vantadb` vs producto `VantaDB` (GitHub routing case-insensitive; no rompe links — solo cosmético).
6. **Branch drift README:** local (develop) dice import canónico `vantadb`; README live (main) dice `import vantadb_py` — drift entre ramas, no naming.

**Links rotos: NO hay badges rotos** (todos los workflow files referenciados existen localmente y el repo es live). Única URL muerta = `vantadb.dev` (metadata de packaging + email enterprise@vantadb.dev en SUPPORT.md) — depende de decisión de dominio del owner, NO se edita en este batch (no romper = no cambiar a ciegas).

## Steps

### Step 1: Auditoría de nombres (DISCOVERY)
- **Archivos:** todos los artefactos públicos + registries
- **Acción:** leer cada artefacto, verificar cada registry live, mapear tabla (arriba).
- **Verify:** evidencia con fetch a registries (2026-08-25) ✅
- **Estado:** ✅ COMPLETED

### Step 2: ADR-030 — convención de identidad de marca
- **Archivos:** `docs/architecture/adr/ADR-030-brand-identity-naming-convention.md` (nuevo)
- **Acción:** ADR con Context (evidencia/tabla de auditoría), Decisión (convención propuesta: "el producto es VantaDB; el crate Rust es `vantadb`; el paquete PyPI es `vantadb-py` (módulo `vantadb_py`, import `vantadb`); los paquetes npm son `vantadb` (TS, WASM) y `vantadb-node` (nativo); el repo GitHub es `ness-e/Vantadb`"), Consecuencias + decisiones pendientes del owner. Status: PROPOSED (Regla 5 — la IA aporta evidencia, el owner articula/confirma la decisión).
- **Verify:** lectura de contenido + número 030 libre ✅
- **Estado:** ✅ COMPLETED

### Step 3: Nota de convención en README (+ README_ES)
- **Archivos:** `README.md`, `README_ES.md`
- **Acción:** extender la nota existente de `vantadb-py`/import con referencia compacta a la convención completa (enlaza a ADR-030). Sin renames, sin cambios de código.
- **Verify:** grep de la nota en ambos README ✅
- **Estado:** ✅ COMPLETED

### Step 4: Verify full + cierre
- **Archivos:** — (verify) + task file
- **Acción:** `scripts/validate-docs-coverage.ps1` ✅ + git diff = solo docs (cero renames) ✅ + task file actualizado. NO commit (lead).
- **Verify:** contrato completo ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna. Wave 1. CodeGraph auto-sync deshabilitado → lectura directa (hecho).

## Review (GATE — agente distinto, P2-01)
> Lo ejecuta un agente DISTINTO al implementador — delegado por el lead al cierre.
- **Revisor:** vanta-review (designado por el lead)
- **Enfoque:** ¿la propuesta de convención es completa y precisa? ¿la tabla de auditoría refleja la realidad de los registries? ¿la decisión quedó correctamente marcada como del owner (PROPOSED)?
- **Cómo se probó:** evidencia mecánica: fetch live a 6 registries (2026-08-25) + lectura de todos los artefactos + glob de workflows.
- **Veredicto:** pendiente

## Notas
- **STOP CONDITION respetada:** renames NO se hacen en este batch (solo auditoría + decisión documentada). Cero cambios en Cargo.toml/package.json/pyproject.toml.
- **Regla 5:** ADR lo articula el owner; la IA entrega la evidencia del mapeo + una propuesta lista para confirmar. El ADR queda PROPOSED hasta que el owner lo acepte.
- **Colateral ruteado:** `vantadb-node` sin publicar → candidato a fila FIND-* (release/publish decision, NO docs) — el lead decide si abre ticket.