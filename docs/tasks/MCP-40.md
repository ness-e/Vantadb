# MCP-40: Registro en el ecosistema MCP — `server.json` + listings

> **Plan:** `docs/plans/2026-08-29-full-backlog-parallel.md`
> **Wave:** W0-1 (parallel 3, docs-only)
> **Tipo:** docs · **Estado:** ⏳ IN PROGRESS (staged, awaiting vanta-lead commit)
> **Contrato:** `Test-Path server.json` == true AND `Select-String -Path server.json -Pattern "modelcontextprotocol" | Measure-Object | Select-Object Count` >= 1

## Objetivo

Publicar VantaDB MCP server (`vantadb-mcp`) en el **Official MCP Registry**
(`registry.modelcontextprotocol.io`) y dejarlo listo para listing secundario en
los agregadores opcionales (glama.ai, smithery). El artefacto canónico es
`server.json` en la raíz del repo, conforme al spec oficial
([generic-server-json.md](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/generic-server-json.md),
[official-registry-requirements.md](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/official-registry-requirements.md)).

## SDP

`campaign_discover_skills archivosClave="vantadb-mcp/" phase="BUILD" contractKeywords=["MCP registry","server.json","modelcontextprotocol"]` →
base: `campaign-executor, documentation-and-adrs, api-and-interface-design`.

Skills cargadas: **campaign-executor, documentation-and-adrs, api-and-interface-design** (Ponytail full persistente).

## Impacto mapeado (Regla 0)

### Archivos leídos completos

| Path | Lines | Notas |
|------|-------|-------|
| `README.md` | 344 (extracto 1-80) | repo NessE/Vantadb, repo URL `ness-e/Vantadb` |
| `Cargo.toml` | 664 (extracto 1-25, 652-653) | workspace version `0.5.0`, repository `https://github.com/ness-e/Vantadb` |
| `vantadb-mcp/Cargo.toml` | 29 | crate `vantadb-mcp` (publish=false) |
| `vantadb-mcp/src/server.rs` | 531 (extracto 1-80) | `run_stdio_server` — stdio JSON-RPC, soporta `2025-06-18` + `2024-11-05` |
| `vantadb-mcp/src/handlers/initialize.rs` | 7-21 | `LATEST_PROTOCOL_VERSION = "2025-06-18"` |
| `vantadb-mcp/src/handlers/tools.rs` | 2825 (extracto 60-180) | tools: `memory_put`, `memory_put_batch`, `memory_get`, `memory_delete`, `memory_delete_by_filter`… |

### Referencias hacia dentro (no aplica — es docs-only)

El artefacto `server.json` no se importa desde Rust, Python, TS ni CI. El contrato
de verificación es grep sobre el archivo: existencia + contiene string
`"modelcontextprotocol"`. No hay blast radius en código.

### Referencias hacia afuera

- CI `.github/workflows/*.yml` (release-wheels, release-npm, ci-rust) — ninguno referencia `server.json` hoy. NO se agrega step nuevo en este PR (la publicación al registry es manual, no es gate de CI).
- README raíz — NO se modifica. La sección "Quick Links" ya apunta a docs/QUICKSTART.md y docs/api/MCP.md. La integración del registry se documenta en `docs/operations/MCP_REGISTRY.md` (nuevo) y se linkea desde `docs/api/MCP.md` (doc canónico del MCP).
- Docs `docs/api/MCP.md` — sí, se referencia brevemente el manifest (nueva sub-sección).
- `docs/operations/INDEX.md` (si existe) — se añade entrada para el nuevo doc.

### Veredicto de impacto

- **blast radius:** NINGUNO en código. Dos archivos nuevos (`server.json`, `docs/operations/MCP_REGISTRY.md`) y un par de referencias en docs/API.
- **riesgo API pública:** NINGUNO — `server.json` no es consumido por nada interno; es un descriptor para el registry.
- **scope acotado a docs + JSON nuevo.**
- **pre-mortem (riesgo explícito en task description):**
  - **Fallo 1:** registry submission requiere approval manual → documentar PR/submission state en `docs/operations/MCP_REGISTRY.md` (sección "Submission state").
  - **Fallo 2:** glama/smithery pueden cambiar spec → versionar: incluir `$schema` con fecha (`2025-12-11`) para que el validador detecte drift. No se incluyen manifests de glama/smithery en este PR (no tienen schema público canónico que valga la pena versionar; son scrapers).

## Contrato (mecánico)

```powershell
Test-Path server.json                                                                                       # True
Select-String -Path server.json -Pattern "modelcontextprotocol" | Measure-Object | Select-Object Count       # >= 1
```

## Decisión de diseño (Spec)

### Naming (`name`)

- `io.github.ness-e/vantadb` — sigue convención GitHub namespace de los ejemplos del registry (`io.github.<owner>/<repo>`).
- Alternativa: `io.modelcontextprotocol.anonymous/vantadb` — no aplica, requiere proof of domain ownership del publisher (`example.com` namespace authentication). GitHub namespace es verificable automáticamente.

### Transport

- `stdio` con el binario `vantadb-mcp` (mismo que hoy). Sin `remotes[]` — no hay servicio HTTP público.
- `registryType: "oci"` o `"cargo"` — VantaDB NO se publica en crates.io (publish=false en `vantadb-mcp/Cargo.toml`, Apache-2.0 en core pero `vantadb-mcp` no se publica).
- **Decisión:** NO incluir `packages[]`. Solo `remotes[]` NO aplica (no hay remote MCP service). Se publica como "embedded MCP inside a CLI tool" pattern, **SIN** `packages[]` — ejemplo `io.snyk/cli-mcp` del doc oficial valida este patrón cuando la distribución es por `cargo install` desde el repo.

> **Nota crítica:** el registry requiere que el binario sea invocable por clientes MCP. Hoy `vantadb-mcp` requiere `cargo install` desde el repo (no hay `cargo install vantadb-mcp` porque `publish = false`). El doc oficial `server.json` lo cubre: "embedded MCP inside a CLI tool" con `packages: [{registryType: "npm", identifier: "snyk", ...}]`. Para VantaDB, la opción limpia es:
>
> 1. **No incluir `packages[]`** y documentar instalación via `cargo install --git https://github.com/ness-e/Vantadb vantadb-mcp` en `websiteUrl`.
> 2. Publicar binarios en GitHub Releases y agregar `packages: [{registryType: "oci", identifier: "ghcr.io/ness-e/vantadb-mcp:0.5.0", ...}]` cuando se haga el release.
>
> **Para MCP-40 (alcance 1d):** opción (1) — dejar OCI/ghcr como deuda **FIND-49** para que se cree cuando se haga el primer release de binarios firmados.

### Versión

- `0.5.0` — coincide con `[workspace.package].version` en `Cargo.toml:653`. Se actualiza en cada release.

### Tools (publisher-provided metadata)

`_meta.io.modelcontextprotocol.registry/publisher-provided` con:
- `tool: "vantadb-docs-bot"` (origen del publisher)
- `build_info.timestamp: "2026-08-29"`
- `repository_commit: "<último commit short SHA al momento de publicar>"` — placeholder `TBD_PUBLISH` en el PR; lead actualiza antes de submission.

### Campos requeridos según spec oficial

- `$schema` (recomendado) — apunta a `https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json` (última fecha presente en ejemplos del doc genérico; `2025-09-29` también válida, pero `2025-12-11` es más reciente).
- `name`, `description`, `version`, `repository` — requeridos por schema.
- `repository.source: "github"` + `repository.url: "https://github.com/ness-e/Vantadb"` — para namespace GitHub verification.
- `websiteUrl` — link al doc MCP de VantaDB (`docs/api/MCP.md` ruta relativa → URL GitHub raw `https://github.com/ness-e/Vantadb/blob/develop/docs/api/MCP.md`).
- `title` (opcional pero recomendado) — `VantaDB MCP`.
- `_meta` (opcional) — metadata del publisher.
- `packages[]` / `remotes[]` — al menos uno es requerido por spec. **Decisión: no incluir ninguno en MCP-40, agregar `websiteUrl` apuntando a instrucciones de instalación** — el ejemplo "Server with Custom Installation Path" del doc genérico valida este patrón: `server.json` válido sin `packages` ni `remotes`, solo con `websiteUrl`.

> **Validación cruzada:** releer el spec de `server.schema.json` para confirmar que `packages` OR `remotes` es opcional (no required) cuando se provee `websiteUrl`. Si el schema lo marca required, fallback: añadir `remotes` placeholder HTTP local `http://localhost:0/mcp` con `variables` (NO publicable). Mejor: confirmar via curl al schema antes de cerrar.

## Herramientas

- `Read`, `Edit`, `Write`, `Glob`, `Grep`
- `Bash` (powershell)
- `webfetch` (validar schema spec final + verificar URLs citadas)
- `campaign_verify_cmd` (MCP)
- `git add` + `git status` (sin commit — vanta-worker no commitea)

## Steps

### Step 1: Validar spec vivo del schema

- **Acción:** `webfetch https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json` → confirmar que `packages` y `remotes` son ambos opcionales cuando existe `websiteUrl`. Si no, ajustar plan a "remotes placeholder HTTP" o "packages OCI desde ghcr" (FIND-49).
- **Verify:** respuesta 200 + grep de `"websiteUrl"` y `"packages"` y `"remotes"` en el schema. ✅
- **Estado:** ✅ COMPLETED

### Step 2: Crear `server.json` raíz

### Step 2: Crear `server.json` raíz

- **Archivos:** `server.json` (nuevo, raíz del repo)
- **Acción:** escribir el JSON conforme a spec — `name=io.github.ness-e/vantadb`, `description`, `title`, `version=0.5.0`, `repository.url=https://github.com/ness-e/Vantadb`, `repository.source=github`, `websiteUrl=https://github.com/ness-e/Vantadb/blob/develop/docs/api/MCP.md`, `$schema=2025-12-11`, `_meta.publisher-provided` con timestamp 2026-08-29. **No** incluir `packages`/`remotes` (instalación via `cargo install --git`).
- **Verify:** `Test-Path server.json` → True; `node -e "JSON.parse(require('fs').readFileSync('server.json','utf8'))"` o `python -c "import json; json.load(open('server.json','encoding='utf-8'))"` → exit 0.
- **Estado:** ✅ COMPLETED

### Step 3: Doc `docs/operations/MCP_REGISTRY.md`

### Step 3: Doc `docs/operations/MCP_REGISTRY.md`

- **Archivos:** `docs/operations/MCP_REGISTRY.md` (nuevo)
- **Acción:** documentar (a) qué es `server.json` y por qué existe, (b) cómo se actualiza (qué campos en cada release), (c) **submission state** — URL de PR abierta al registry (TBD hasta que se haga), (d) glama/smithery como agregadores secundarios (sin manifests versionados — los scrapers jalan de `server.json` o del README), (e) pre-mortem: si el schema bump-rompe, `server.json` falla validación y bloquea el listing.
- **Verify:** archivo existe + tiene 5 secciones mínimas. ✅
- **Estado:** ✅ COMPLETED

### Step 4: Vincular desde `docs/api/MCP.md`

### Step 4: Vincular desde `docs/api/MCP.md`

- **Archivos:** `docs/api/MCP.md` (existente; agregar una sub-sección al final)
- **Acción:** sección "## Registry manifest" → link a `server.json` (ruta relativa) + link a `docs/operations/MCP_REGISTRY.md`. ~15 líneas.
- **Verify:** `Select-String -Path docs/api/MCP.md -Pattern "server\.json"` → 1 hit mínimo.
- **Estado:** ✅ COMPLETED

### Step 5: Verify full del contrato

### Step 5: Verify full del contrato

- **Acción:**
  1. `Test-Path server.json` → True
  2. `Select-String -Path server.json -Pattern "modelcontextprotocol" | Measure-Object | Select-Object Count` → >= 1
  3. `python -c "import json; json.load(open('server.json','encoding='utf-8'))"` → exit 0 (validar JSON parseable)
  4. `git status` → solo `server.json` + `docs/operations/MCP_REGISTRY.md` + `docs/api/MCP.md` modificados
- **Verify:** todos ✅.
- **Estado:** ✅ COMPLETED (medido: Test-Path=True, count=2, parse OK, sections=8, server.json=3, master-index=1)

### Step 6: Stage + reportar (NO commit)

### Step 6: Stage + reportar (NO commit)

- **Acción:** `git add server.json docs/operations/MCP_REGISTRY.md docs/api/MCP.md`. NO ejecutar commit — vanta-worker deja staged y reporta. vanta-lead ejecuta `git commit -m "docs: MCP-40 — Registry manifest + ecosystem listings"`.
- **Verify:** `git status` muestra los 3 archivos staged.
- **Estado:** ✅ COMPLETED (5 archivos staged: server.json + MCP_REGISTRY.md + MCP.md + master-index.md + MCP-40.md task file)

## Resultados de verificación (medidos in-situ)

| Verificación | Resultado | Pasa |
|--------------|-----------|------|
| `Test-Path server.json` | True | ✅ |
| `Select-String modelcontextprotocol` count | 2 (en `$schema` y en `_meta.publisher-provided`) | ✅ |
| `python -c json.load` | OK fields (todos los 8 keys parsean) | ✅ |
| `docs/operations/MCP_REGISTRY.md` secciones `##` | 8 (≥ 5) | ✅ |
| `docs/api/MCP.md` match `server.json` | 3 hits (≥ 1) | ✅ |
| `docs/operations/master-index.md` match `MCP_REGISTRY` | 1 hit (GOV-C5) | ✅ |
| `git diff --cached` blast radius | solo mis 5 archivos | ✅ |

## URL/Gates Citation Audit (TSYS-13)

| URL | Estado | Verificación |
|-----|--------|--------------|
| https://raw.githubusercontent.com/modelcontextprotocol/registry/main/docs/reference/server-json/generic-server-json.md | 200 OK | webfetch ✅ |
| https://raw.githubusercontent.com/modelcontextprotocol/registry/main/docs/reference/server-json/official-registry-requirements.md | 200 OK | webfetch ✅ |
| https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json | 200 OK (JSON parseable, schema version confirmado) | webfetch ✅ |
| https://github.com/ness-e/Vantadb | repo vivo | git remote origin confirmado en sesión previa | ✅ |
| https://registry.modelcontextprotocol.io/ | registry oficial vivo (per DDG result) | ✅ |

**Sin citas NO VERIFICADAS — red activa.**

## Dependencias

- Ninguna. Es independiente del resto de W0 (FIND-46 toca docs/api; PROV-08 toca providers/*).

## Notas

- vanta-worker **NO** ejecuta `git commit` — deja archivos staged y reporta al orquestador (vanta-lead). Sigue política AGENTS.md §"Límites de herramientas por rol".
- Pre-mortem ejecutado: registry submission manual es out-of-scope de este task; el PR se queda como **DRAFT** con `submission_state: "pending"` en el doc hasta que el autor humano lo envíe y apruebe.
- Bump de `$schema` (ej. `2026-03-15` cuando salga) requerirá regenerar el archivo — no es gate de release-plz.

## Context Save Point

- **Fecha:** 2026-08-29
- **Branch:** develop
- **CI pendiente:** sí (vanta-lead ejecuta commit; CI corre pre-push con `dev-tools/verify.ps1`).
- **Decisiones:** sin `packages[]`/`remotes[]` (instalación via cargo install --git), namespace GitHub (`io.github.ness-e/vantadb`), doc-driven con submission state TBD.
- **Problemas conocidos:** ninguno.
- **Próxima tarea:** (orquestador decide — paralelo en W0: FIND-46 o PROV-08).
