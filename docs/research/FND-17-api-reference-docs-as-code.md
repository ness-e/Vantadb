# Investigación — FND-17: API reference automatizada (docs-as-code)

> **ID:** `FND-17`
> **Categoría:** Docs / DevOps (CI/CD)
> **Fecha:** 2026-08-16
> **Estado:** ✅ Análisis completado — plan propuesto (Fase 1 de bajo costo), Python/TS diferidos
> **Fuente:** `docs/Backlog.md` P20c, plan `docs/plans/2026-08-16-wave-r2-r7-fnd.md` Task 6
> **Contrato:** análisis + plan o pipeline entregado (no implementación forzada) — ver §6 Decisión

---

## 1. Contexto y Objetivos

P20c: investigar si rustdoc/pydoc/typedoc se generan en CI y se versionan junto al
código — *"lo primero que evalúa un dev antes de adoptar la DB"*. La API reference
es el primer artefacto que un adoptante abre: si no está automatizada, se desactualiza
o simplemente no existe. Esta investigación verifica el estado real del repo y propone
el pipeline mínimo viable.

---

## 2. Estado actual (verificado contra workflows reales)

### 2.1 NO existe generación de API reference en CI

Grep dirigido sobre `.github/workflows/` (17 workflows) y `scripts/`:

```
rg -i 'cargo doc|rustdoc|typedoc|pydoc|mkdocs|docs-generate|generate-docs' .github/ scripts/
→ 0 matches (exit 1)
```

Ningún workflow invoca `cargo doc`, typedoc, pydoc/mkdocstrings ni despliega un
docs site. Los únicos artefactos subidos son wheels (`release-wheels-60.yml:116-117`),
WASM (`release-npm-61.yml:109-110`), SBOM (`release-sbom-64.yml:37-45`) y benchmarks
(`heavy-bench-nightly-51.yml:64`).

### 2.2 Los workflows de docs existentes son de VALIDACIÓN, no de generación

| Workflow / script | Qué hace | Evidencia |
|---|---|---|
| `gate-docs-21.yml` | markdownlint sobre `docs/**` + check de versiones en headers (`docs/api/openapi.yaml`, `docs/api/MCP.md`) | `gate-docs-21.yml:30,62-78` |
| `ci-rust-10.yml` | Gate de ADR en PRs que tocan core | `ci-rust-10.yml:154-167` |
| `scripts/validate-docs-coverage.ps1` | Paridad métodos públicos ↔ docs markdown (busca el nombre del método como texto en el .md) — **no genera nada**, solo valida que el doc manual mencione el símbolo | `validate-docs-coverage.ps1:64-175` |

La "API reference" hoy es markdown **escrito a mano** en `docs/api/`:
`EMBEDDED_SDK.md`, `PYTHON_SDK.md`, `TS_SDK.md`, `HTTP_API.md`, `IQL.md`, `MCP.md`,
`GRAPH_RAG.md`, `openapi.yaml`. Es un vault Obsidian (`docs/README.md:15`), no un
docs site generado.

### 2.3 Estado del material fuente por lenguaje

| Lenguaje | Tooling instalado | Docstrings en el código | Capacidad de generación hoy |
|---|---|---|---|
| **Rust (core + crates)** | rustdoc viene con la toolchain (`cargo doc` sin deps) | 9 archivos con `#![warn(missing_docs)]`: `providers/{openai,ollama,litellm}/src/lib.rs:1`, `vantadb-wasm/src/lib.rs:2`, `vantadb-mcp/src/lib.rs:1`, `vantadb-server/src/{lib,main}.rs:1`, `vantadb-python/src/{lib,vector}.rs` (lib.rs:5, vector.rs:2). **Core `src/lib.rs` NO tiene el atributo** — solo ~20 docstrings `///` | ✅ **`cargo doc --no-deps` ya genera** HTML navegable |
| **Python** | `pyproject.toml` sin deps de docs (`vantadb-python/pyproject.toml:1-53`) | Stubs `.pyi` con firmas pero **0 docstrings** (solo el docstring de módulo, `vantadb_py.pyi:1`; `rg '"""' *.pyi` → 0) | ❌ pdoc/mkdocstrings generarían solo firmas, sin descripción |
| **TypeScript** | `package.json` sin typedoc en devDependencies, sin script de docs (`vantadb-ts/package.json:23-29,50-60`) | **0 bloques JSDoc** (`rg '/\*\*' src/*.ts` → 0) | ❌ typedoc requeriría dep nueva y docstrings que no existen |
| **Docs site** | Sin `mkdocs.yml`, sin workflow Pages/deploy | — | ❌ el vault Obsidian no se sirve como site |

### 2.4 Lo que ya existe gratis: docs.rs

El core publicado ya tiene API reference pública vía docs.rs:
`Cargo.toml:11` → `documentation = "https://docs.rs/vantadb"`. Esto cubre el crate
publicado, pero NO cubre: el estado de `develop` (pre-release), ni los bindings
Python/TS, ni crates del workspace no publicados.

---

## 3. Herramientas disponibles vs requeridas

| Herramienta | Deps nuevas | Costo | Calidad del output HOY |
|---|---|---|---|
| **rustdoc (`cargo doc`)** | 0 — incluida en toolchain | 🟢 trivial (1 job CI) | Alta para crates con `warn(missing_docs)`; media para core (sin atributo, docstrings parciales) |
| **pdoc / mkdocstrings** | Sí (dev-deps Python) | 🟡 media | Baja — sin docstrings en `.pyi`, genera solo firmas |
| **typedoc** | Sí (`npm i -D typedoc`, dep nueva) | 🟡 media | Baja — sin JSDoc en `src/`, genera solo tipos |
| **GitHub Pages / docs site (mkdocs)** | Sí (workflow + branch + config + template) | 🔴 alta | N/A — infraestructura nueva pesada |

Fuentes verificadas: [rustdoc](https://doc.rust-lang.org/rustdoc/) (parte del toolchain),
[typedoc.org](https://typedoc.org/) ✅ (convierte comentarios TS en HTML; `npm install --save-dev typedoc`),
[pdoc.dev](https://pdoc.dev/) ✅ (auto-genera API docs desde docstrings, sin config).

---

## 4. Análisis por opción

### Opción A — Fase 1: rustdoc en CI (RECOMENDADA, costo 🟢)

**Qué:** job CI que corre `cargo doc --no-deps --workspace`, sube el HTML como
artifact y lo deja disponible en cada PR/merge.

- **Costo:** ~1 job, 10-15 líneas de YAML. Cero deps nuevas (rustdoc incluido).
- **ROI inmediato:** el dev adopta viendo la API del estado actual, no la de docs.rs
  (que puede ir atrasada en `develop`).
- **Gate de calidad acoplado:** `cargo doc --no-deps` falla si un docstring Rust
  tiene sintaxis inválida de markdown/links (`[broken]`) — convierte docstrings
  rotos en error de CI.
- **Complemento natural:** subir `#![deny(missing_docs)]` al core (hoy solo `warn`
  en crates periféricos; core sin atributo) para que la cobertura sea verificable.
  Esto es trabajo de autoría de docstrings — se puede diferir como follow-up.
- **Deploy opcional (Fase 2, defer):** `actions/deploy-pages` + Pages apuntando al
  artifact de rustdoc. Requiere decisión de infra (branch/pages config), no bloquea
  la Fase 1.

### Opción B — Fase 2: docs site completo (mkdocs + Pages) — DEFER 🔴

- Requiere: `mkdocs.yml`, tema, workflow de Pages, migrar el vault Obsidian a
  estructura de site. Infraestructura nueva pesada.
- **No bloquea** la Fase 1: el rustdoc + `docs/api/` markdown (vault) coexisten hoy.

### Opción C — typedoc (TS) — DEFER 🟡 con prerequisito de autoría

- **Prerequisito bloqueante:** 0 JSDoc en `src/` hoy. typedoc sobre código sin
  docstrings produce una lista de tipos sin explicación — peor que `docs/api/TS_SDK.md`
  (que al menos tiene prosa curada).
- **Orden correcto:** primero escribir JSDoc en `vantadb-ts/src/*.ts` (trabajo de
  autoría, ~1-2 días), después agregar typedoc como devDep y job CI.
- typedoc además es una **dep nueva** en el SDK — el gate de deuda (Regla 6) exige
  justificación.

### Opción D — pdoc/mkdocstrings (Python) — DEFER 🟡 con prerequisito de autoría

- **Prerequisito bloqueante:** 0 docstrings en los `.pyi`. El wheel ya incluye
  `py.typed` + `.pyi` (`vantadb-python/pyproject.toml:40-43`), así que la fuente
  de verdad para generar es el stub — pero sin docstrings el output es solo firmas.
- **Alternativa de menor esfuerzo que pdoc:** documentar los `.pyi` con docstrings
  (los stubs ya son el contrato público) y generar con pdoc (sin config, ver
  [pdoc.dev](https://pdoc.dev/)) — pero sigue siendo autoría primero.

---

## 5. Plan propuesto (Fase 1 — bajo costo, sin infra nueva)

> ⚠️ **NO implementado** — requiere confirmación del lead antes de tocar workflows
> (regla de la task: análisis primero).

**Job CI** (nuevo workflow `docs-reference.yml`, o job adicional en `gate-docs-21.yml`):

```yaml
# docs-reference — genera API reference Rust (rustdoc) en CI, sin deps nuevas
name: Docs Reference
on:
  pull_request:
    paths: ["src/**", "vantadb-*/src/**", "providers/**", "Cargo.toml", "Cargo.lock"]
  push:
    branches: [develop, main]

jobs:
  rustdoc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable        # rustdoc incluido
      - uses: Swatinem/rust-cache@v2               # reutiliza cache existente
      - name: Generate rustdoc
        run: cargo doc --no-deps --workspace       # falla si hay docstring roto
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: api-reference-rust
          path: target/doc
          retention-days: 7
```

**Decisión de Fase 2 (defer, requiere lead):** deploy del artifact a GitHub Pages
(`actions/deploy-pages`) cuando exista decisión de docs site; hoy el vault Obsidian
en GitHub cumple.

**Follow-ups de autoría (backlog, no pipeline):**
1. `#![deny(missing_docs)]` en core `src/lib.rs` (hoy ni siquiera `warn`) + completar
   docstrings públicos.
2. JSDoc en `vantadb-ts/src/*.ts` → habilita typedoc (Opción C).
3. Docstrings en `vantadb_py/*.pyi` → habilita pdoc (Opción D).

---

## 6. Decisión

**Plan (no defer):** Fase 1 — job CI `cargo doc --no-deps --workspace` + artifact.
Costo 🟢 (0 deps, rustdoc incluido), ROI alto (API reference del estado actual para
el dev evaluador, gate de docstrings rotos en CI).

**Diferido con razón:** typedoc (Opción C) y pdoc/mkdocstrings (Opción D) — el
material fuente (docstrings) no existe en los SDKs; generar sin él produce listas
de firmas sin valor sobre la prosa curada de `docs/api/`. El bloqueante es autoría,
no pipeline. Docs site/Pages (Opción B) — infraestructura nueva, no bloquea Fase 1.

**Acción requerida:** confirmación del lead para implementar la Fase 1 en un
workflow (no se toca CI sin aprobación — ver §5 nota).

---

## 7. Referencias

- [rustdoc — The Rust documentation tool](https://doc.rust-lang.org/rustdoc/) — parte de la toolchain, `cargo doc` sin deps
- [TypeDoc — Documentación para TypeScript](https://typedoc.org/) — ✅ verificada 2026-08-16; requiere `npm i -D typedoc` y comentarios TSDoc
- [pdoc — Generate API Documentation for Python Projects](https://pdoc.dev/) — ✅ verificada 2026-08-16; auto-genera desde docstrings, sin config
- Estado del repo: verificado localmente (workflows §2, docstrings §2.3, `Cargo.toml:11` docs.rs)

---

## 8. Veredicto ROI

| Cambio | Costo | ROI |
|---|---|---|
| Job CI rustdoc + artifact | 🟢 1 job, 0 deps | **Alto** — API reference del estado actual + gate de docstrings rotos |
| `#![deny(missing_docs)]` core + completar docstrings | 🟡 autoría | Alto — sube la calidad del rustdoc |
| typedoc / pdoc | 🟡 dep nueva + autoría | Bajo hoy — sin docstrings el output es solo firmas |
| mkdocs + Pages | 🔴 infra nueva | Medio — defer hasta decidir docs site |